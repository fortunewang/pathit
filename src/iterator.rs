use crate::iter_dir::IterDir;
use ring::digest::SHA256;
use std::io;
use std::path::{Path, PathBuf};

lazy_static::lazy_static! {
    static ref DIR_HASH: String = unsafe { String::from_utf8_unchecked(vec![b'-'; SHA256.output_len * 2]) };
}

fn hash_file<P: AsRef<Path>>(filepath: P) -> std::io::Result<String> {
    use ring::digest::Context;
    use std::io::BufRead;
    let mut file = std::io::BufReader::new(std::fs::File::open(filepath)?);
    let mut ctx = Context::new(&SHA256);
    loop {
        let buf = file.fill_buf()?;
        if buf.is_empty() {
            break;
        }
        ctx.update(buf);
        let length = buf.len();
        file.consume(length);
    }

    Ok(base16::encode_lower(ctx.finish().as_ref()))
}

fn normalize_path<P: AsRef<Path>>(path: &Path, base: P) -> String {
    use std::path::MAIN_SEPARATOR;
    let mut normalized = path
        .strip_prefix(base)
        .unwrap()
        .to_string_lossy()
        .replace(MAIN_SEPARATOR, "/");
    if path.is_dir() {
        normalized.push('/');
    }
    normalized
}

fn hash_and_normalize<P: AsRef<Path>>(path: &Path, base: P) -> std::io::Result<(String, String)> {
    if path.is_dir() {
        Ok((normalize_path(path, base), DIR_HASH.clone()))
    } else {
        Ok((normalize_path(path, base), hash_file(path)?))
    }
}

pub struct PathIterator {
    root: PathBuf,
    it: IterDir,
}

impl PathIterator {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        let root = root.as_ref().to_path_buf();
        let it = IterDir::new(&root);
        Self { root, it }
    }
}

impl Iterator for PathIterator {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.it.next() {
            Some(Ok(path)) => Some(Ok(normalize_path(&path, &self.root))),
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }
}

pub struct HashedPathIterator {
    root: PathBuf,
    it: IterDir,
}

impl HashedPathIterator {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        let root = root.as_ref().to_path_buf();
        let it = IterDir::new(&root);
        Self { root, it }
    }

    fn try_next(&mut self) -> io::Result<Option<(String, String)>> {
        match self.it.next().transpose()? {
            Some(path) => Ok(Some(hash_and_normalize(&path, &self.root)?)),
            None => Ok(None),
        }
    }
}

impl Iterator for HashedPathIterator {
    type Item = io::Result<(String, String)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().transpose()
    }
}
