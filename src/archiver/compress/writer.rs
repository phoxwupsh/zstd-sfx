use std::io::Write;

pub struct CompressWriter<W: Write> {
    inner: W,
    count: usize,
}

impl<W: Write> CompressWriter<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            count: 0,
        }
    }

    pub fn into_inner(mut self) -> Result<W, std::io::Error> {
        self.inner.flush()?;
        Ok(self.inner)
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

impl<W: Write> Write for CompressWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let bytes = self.inner.write(buf)?;
        self.count += bytes;
        Ok(bytes)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}