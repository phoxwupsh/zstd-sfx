use common::header::Header;
use std::{
    borrow::Cow,
    env,
    fs::{create_dir_all, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::PathBuf,
};
use zstd::stream::copy_decode;

fn main() {
    if let Err(err) = main_inner() {
        println!("Failed to decompress: {}", err);
    }
}

fn main_inner() -> Result<(), std::io::Error> {
    let this_exe_path = env::current_exe()?;
    let mut this_exe = OpenOptions::new().read(true).open(&this_exe_path)?;
    let (header, header_pos) = read_header_from_exe(&mut this_exe)?;

    let paths_pos = header_pos + Header::HEADER_LEN;
    this_exe.seek(SeekFrom::Start(paths_pos))?;
    let paths = this_exe.take(header.paths_len);
    let paths_reader = BufReader::new(paths);

    let mut sizes_exe = OpenOptions::new().read(true).open(&this_exe_path)?;
    let sizes_pos = paths_pos + header.paths_len;
    sizes_exe.seek(SeekFrom::Start(sizes_pos))?;
    let mut sizes_reader = BufReader::new(sizes_exe);
    let mut size_buffer = [0u8; 8];

    let mut hashes_exe = OpenOptions::new().read(true).open(&this_exe_path)?;
    let hashes_pos = sizes_pos + header.sizes_len;
    hashes_exe.seek(SeekFrom::Start(hashes_pos))?;
    let mut hashes_reader = BufReader::new(hashes_exe);
    // md5 length = 16 bytes
    let mut hash_buffer = [0u8; 16];

    let mut compressed_exe = OpenOptions::new().read(true).open(&this_exe_path)?;
    let compressed_pos = hashes_pos + header.hashes_len;
    compressed_exe.seek(SeekFrom::Start(compressed_pos))?;
    let mut compressed_reader = BufReader::new(compressed_exe);

    for line in paths_reader.lines() {
        let path = PathBuf::from(line?);

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                create_dir_all(parent)?;
            }
        }
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)?;

        sizes_reader.read_exact(&mut size_buffer)?;
        let original_size = u64::from_le_bytes(size_buffer) as usize;

        sizes_reader.read_exact(&mut size_buffer)?;
        let compressed_size = u64::from_le_bytes(size_buffer) as usize;

        // This is the expected hash
        hashes_reader.read_exact(&mut hash_buffer)?;

        let mut file_writer =
            DecompressWriter::new(BufWriter::new(file), original_size, path.to_string_lossy());

        let compressed_block = compressed_reader.by_ref().take(compressed_size as u64);

        copy_decode(compressed_block, &mut file_writer)?;

        let written_hash = file_writer.md5_digest();
        if written_hash.0 != hash_buffer {
            print!("; this file is corrupted");
        } else {
            print!("; ok")
        }

        println!();
    }
    Ok(())
}

fn read_header_from_exe(exe: &mut std::fs::File) -> Result<(Header, u64), std::io::Error> {
    let exe_len = exe.metadata()?.len();
    exe.seek(SeekFrom::Start(exe_len - 8))?;
    let mut buf = [0u8; 8];
    exe.read_exact(&mut buf)?;

    let data_length = u64::from_le_bytes(buf);
    let header_pos = exe_len - 8 - data_length;

    exe.seek(SeekFrom::Start(header_pos))?;

    Ok((Header::parse_stream(exe)?, header_pos))
}

struct DecompressWriter<'a, W: Write> {
    inner: W,
    count: usize,
    total: usize,
    path: Cow<'a, str>,
    hash: md5::Context,
}

impl<'a, W: Write> DecompressWriter<'a, W> {
    fn new(inner: W, total: usize, path: Cow<'a, str>) -> Self {
        Self {
            inner,
            total,
            path,
            count: 0,
            hash: md5::Context::new(),
        }
    }

    fn md5_digest(&self) -> md5::Digest {
        self.hash.clone().compute()
    }
}

impl<'a, W: Write> Write for DecompressWriter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let bytes = self.inner.write(buf)?;
        self.hash.consume(&buf[..bytes]);
        self.count += bytes;
        print!("\r{}: {} / {}", self.path, self.count, self.total);
        Ok(bytes)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
