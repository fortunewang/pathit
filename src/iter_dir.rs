use std::collections::VecDeque;
use std::fs::{self, ReadDir};
use std::io;
use std::path::{Path, PathBuf};

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
