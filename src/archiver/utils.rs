use std::{
    fs::read_dir,
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub fn buf_copy(reader: &mut impl Read, writer: &mut impl Write) -> Result<(), std::io::Error> {
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        writer.write_all(&buffer[..bytes_read])?;
    }

    writer.flush()?;
    Ok(())
}

pub fn glob(path: impl AsRef<Path>) -> Result<Glob, std::io::Error> {
    fn recursive_inner(ps: &mut Vec<PathBuf>, p: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let p = p.as_ref();
        if p.is_file() {
            ps.push(p.to_path_buf());
            return Ok(());
        }

        let dir = read_dir(p)?;
        for entry in dir {
            let entry = entry?;
            let entry_type = entry.file_type()?;
            let entry_path = entry.path();

            if entry_type.is_file() {
                ps.push(entry_path);
            } else {
                recursive_inner(ps, entry_path)?;
            }
        }
        Ok(())
    }

    let path = path.as_ref();
    if path.is_dir() {
        let mut res = Vec::<PathBuf>::new();
        recursive_inner(&mut res, path)?;
        Ok(Glob::Dir(res))
    } else {
        Ok(Glob::File(path.to_path_buf()))
    }
}

pub enum Glob {
    File(PathBuf),
    Dir(Vec<PathBuf>),
}
