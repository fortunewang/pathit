use ring::digest::SHA256;
use std::collections::VecDeque;
use std::fs::{self, ReadDir};
use std::io;
use std::path::{Path, PathBuf};

lazy_static::lazy_static! {
    static ref DIR_HASH: String = unsafe { String::from_utf8_unchecked(vec![b'-'; SHA256.output_len() * 2]) };
}

pub fn hash_file<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    if path.as_ref().is_dir() {
        return Ok(DIR_HASH.clone());
    }

    use ring::digest::Context;
    use std::io::BufRead;
    let mut file = std::io::BufReader::new(std::fs::File::open(path)?);
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

pub fn normalize_path<P: AsRef<Path>>(path: &Path, base: P) -> String {
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

#[derive(Debug)]
pub struct IterDir {
    stack: VecDeque<ReadDir>,
    next: Option<PathBuf>,
}

impl IterDir {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        let root = root.as_ref().to_path_buf();
        return Self {
            stack: VecDeque::new(),
            next: Some(root),
        };
    }

    fn try_next(&mut self) -> io::Result<Option<PathBuf>> {
        if let Some(pending) = self.next.take() {
            self.stack.push_back(fs::read_dir(&pending)?);
        }
        while let Some(reading) = self.stack.back_mut() {
            match reading.next().transpose()? {
                Some(entry) => {
                    let path = entry.path();
                    if path.is_dir() {
                        self.next = Some(path.clone());
                    }
                    return Ok(Some(path));
                }
                None => {
                    self.stack.pop_back();
                }
            }
        }
        Ok(None)
    }

    fn next_transposed(&mut self) -> std::io::Result<Option<std::path::PathBuf>> {
        let next = self.try_next();
        if next.is_err() {
            self.stack.clear();
        }
        next
    }
}

impl std::iter::Iterator for IterDir {
    type Item = std::io::Result<std::path::PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_transposed().transpose()
    }
}
