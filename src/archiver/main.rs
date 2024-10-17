use args::Args;
use common::header::Header;
use compress::compress_multi_to_temps;
use std::{
    env,
    fs::OpenOptions,
    io::{BufReader, BufWriter, Seek, SeekFrom, Write},
    path::PathBuf,
};
use utils::{buf_copy, glob};

mod args;
mod compress;
mod utils;

fn main() -> Result<(), std::io::Error> {
    let args = argh::from_env::<Args>();

    let output = match args.output {
        Some(output) => output,
        None => {
            let file_name =
                args
                    .target
                    .file_name()
                    .map(PathBuf::from)
                    .ok_or(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Unable to automatically create output",
                    ))?;
            let cwd = env::current_dir()?;
            let mut res = cwd.join(file_name);

            #[cfg(windows)]
            {
                if !res.extension().map_or(false, |ext| ext == "exe") {
                    res.set_extension("exe");
                }
            }

            res
        }
    };

    std::fs::copy(&args.unarchiver, &output)?;
    let paths = glob(&args.target)?;

    let mut compressed_data = compress_multi_to_temps(
        paths,
        args.target,
        args.temp,
        args.root,
        args.level as i32,
    )?;

    // The `+ compressed_all.len() as u64` is for those `\n` characters produced by `writeln!`
    let paths_len = compressed_data
        .iter()
        .map(|compressed| compressed.path_str.len() as u64)
        .sum::<u64>()
        + compressed_data.len() as u64;

    let sizes_len = (compressed_data.len() * size_of::<u64>() * 2) as u64;

    // md5 digest = 16bytes
    let hashes_len = (compressed_data.len() * 16) as u64;

    let compressed_data_len = compressed_data
        .iter()
        .map(|compressed| compressed.compressed_size)
        .sum::<u64>();

    let header = Header {
        paths_len,
        sizes_len,
        compressed_data_len,
        hashes_len,
    };

    let unarchiver_exe = OpenOptions::new().append(true).open(&output)?;
    let mut unarchiver_writer = BufWriter::new(unarchiver_exe);

    // 1. Write header
    unarchiver_writer.write_all(&header.to_bytes())?;

    // 2. Write paths
    for compressed in compressed_data.iter() {
        // This introduce that `compressed_all.len() as u64` bytes
        writeln!(unarchiver_writer, "{}", compressed.path_str)?;
    }

    // 3. Write sizes pairs
    for compressed in compressed_data.iter() {
        unarchiver_writer.write_all(&compressed.original_size.to_le_bytes())?;
        unarchiver_writer.write_all(&compressed.compressed_size.to_le_bytes())?;
    }

    // 4. Write hashes
    for compressed in compressed_data.iter() {
        unarchiver_writer.write_all(&compressed.hash.0)?
    }

    // 5. Write files
    for compressed in compressed_data.iter_mut() {
        compressed.file.seek(SeekFrom::Start(0))?;
        let mut reader = BufReader::new(&compressed.file);
        buf_copy(&mut reader, &mut unarchiver_writer)?;
    }

    let header_and_data_length = header.header_and_data_len();
    unarchiver_writer.write_all(&header_and_data_length.to_le_bytes())?;
    unarchiver_writer.flush()?;

    Ok(())
}
