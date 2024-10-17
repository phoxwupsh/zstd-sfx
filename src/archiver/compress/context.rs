use indicatif::MultiProgress;
use md5::Digest;
use std::{fs::File, path::PathBuf, sync::Arc};

#[derive(Debug)]
pub struct CompressContext {
    pub file: File,
    pub path_str: String,
    pub original_size: u64,
    pub compressed_size: u64,
    pub hash: Digest,
}

#[derive(Debug, Clone)]
pub struct Context {
    pub tmp_dir: Option<Arc<PathBuf>>,
    pub root: Arc<PathBuf>,
    pub include_root: bool,
    pub compress_level: i32,
    pub progress_bars: MultiProgress,
}
