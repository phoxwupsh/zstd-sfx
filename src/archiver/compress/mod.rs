use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::utils::Glob;
use context::{CompressContext, Context};
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reader::CompressReader;
use tempfile::{tempfile, tempfile_in};
use writer::CompressWriter;
use zstd::stream::copy_encode;

pub mod context;
pub mod reader;
pub mod writer;

pub fn compress_multi_to_temps(
    paths: Glob,
    root: PathBuf,
    temp_dir: Option<PathBuf>,
    include_root: bool,
    compress_level: i32,
) -> Result<Vec<CompressContext>, std::io::Error> {
    let tmp_dir = temp_dir.map(Arc::new);
    let root = Arc::new(root);
    let progress_bars = MultiProgress::with_draw_target(ProgressDrawTarget::stdout());
    match paths {
        Glob::File(file) => {
            let ctx = Context {
                tmp_dir,
                root,
                include_root: true,
                compress_level,
                progress_bars
            };
            let compressed = compress_to_temp(&ctx, file)?;
            Ok(vec![compressed])
        }
        Glob::Dir(paths) => {
            let paths = paths.into_par_iter();

            let ctx = Context {
                tmp_dir,
                root,
                include_root,
                compress_level,
                progress_bars,
            };

            let compressed_all = paths
                .filter_map(|path| {
                    let ctx = ctx.clone();
                    let compressed =
                        compress_to_temp(&ctx, path);
                    compressed.ok()
                })
                .collect::<Vec<_>>();

            Ok(compressed_all)
        }
    }
}

pub fn compress_to_temp(
    ctx: &Context,
    path: impl AsRef<Path>,
) -> Result<CompressContext, std::io::Error> {
    let path = path.as_ref();
    let tmp = match &ctx.tmp_dir {
        Some(dir) => tempfile_in(dir.as_path()),
        None => tempfile(),
    }?;
    let target = OpenOptions::new().read(true).open(path)?;
    let original_size = target.metadata()?.len();

    println!("{}", path.display());
    let path_str = if ctx.include_root {
        match ctx.root.parent() {
            Some(parent) => path.strip_prefix(parent).unwrap(),
            None => path.strip_prefix(ctx.root.as_path()).unwrap(),
        }
    } else {
        path.strip_prefix(ctx.root.as_path()).unwrap()
    }
    .to_string_lossy()
    .into_owned();

    let progress = ProgressBar::new(original_size)
        .with_style(ProgressStyle::with_template("{msg}: {pos} / {len} bytes compressed").unwrap())
        .with_message(path_str.clone());

    let mut progress_reader = CompressReader::new(target, progress.clone());
    ctx.progress_bars.add(progress.clone());

    let mut tmp_writer = CompressWriter::new(tmp);
    copy_encode(&mut progress_reader, &mut tmp_writer, ctx.compress_level)?;
    tmp_writer.flush()?;

    let compressed_size = tmp_writer.count() as u64;
    let hash = progress_reader.md5_digest();
    let file = tmp_writer.into_inner()?;

    progress.finish_and_clear();
    progress.println(format!(
        "\x1b[2K\r{}: {} \u{2192} {} bytes; md5={:?}",
        path_str, original_size, compressed_size, hash
    ));

    Ok(CompressContext {
        file,
        path_str,
        original_size,
        compressed_size,
        hash,
    })
}
