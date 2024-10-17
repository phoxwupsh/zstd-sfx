use indicatif::ProgressBar;
use std::io::Read;

pub struct CompressReader<R: Read> {
    inner: R,
    hash: md5::Context,
    progress: ProgressBar,
}

impl<R: Read> CompressReader<R> {
    pub fn new(inner: R, progress: ProgressBar) -> Self {
        Self {
            inner,
            hash: md5::Context::new(),
            progress,
        }
    }

    pub fn md5_digest(&self) -> md5::Digest {
        self.hash.clone().compute()
    }
}

impl<R: Read> Read for CompressReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes = self.inner.read(buf)?;
        self.hash.consume(&buf[..bytes]);
        self.progress.inc(bytes as u64);
        Ok(bytes)
    }
}
