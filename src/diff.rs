use std::collections::{BTreeMap, BTreeSet};
use std::iter::FromIterator;

#[derive(Debug, Clone, Copy)]
pub enum Difference {
    New,
    Absence,
    HashDifference,
}

#[derive(Debug, Default, Clone)]
pub struct Entries {
    entries: BTreeSet<String>,
}

impl Entries {
    pub fn read<R: std::io::BufRead>(mut reader: R) -> std::io::Result<Self> {
        let mut entries = BTreeSet::new();
        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            let path = line.trim();
            if path.is_empty() {
                continue;
            }
            entries.insert(path.to_owned());
            line.clear();
        }
        Ok(Self { entries })
    }

    pub fn try_collect<E, I: IntoIterator<Item = Result<String, E>>>(iter: I) -> Result<Self, E> {
        let mut entries = BTreeSet::new();
        for item in iter {
            entries.insert(item?);
        }
        Ok(Self { entries })
    }
}

impl std::ops::Sub for Entries {
    type Output = BTreeMap<String, Difference>;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut diffs = BTreeMap::new();

        for path in self.entries.iter() {
            if !rhs.entries.contains(path) {
                diffs.insert(path.clone(), Difference::New);
            }
        }
        for path in rhs.entries.iter() {
            if !self.entries.contains(path) {
                diffs.insert(path.clone(), Difference::Absence);
            }
        }

        diffs
    }
}

impl FromIterator<String> for Entries {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        let mut entries = BTreeSet::new();
        for item in iter {
            entries.insert(item);
        }
        Self { entries }
    }
}

#[derive(Debug, Default, Clone)]
pub struct HashedEntries {
    entries: BTreeMap<String, String>,
}

impl HashedEntries {
    pub fn read<R: std::io::BufRead>(mut reader: R) -> anyhow::Result<Self> {
        use ring::digest::SHA256;

        let mut entries = BTreeMap::new();
        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            let trimed = line.trim();
            if trimed.is_empty() {
                continue;
            }
            if let Some((hash, path)) = trimed.split_once(", ") {
                if hash.len() != SHA256.output_len * 2 {
                    anyhow::bail!("digest length does not match")
                }
                entries.insert(path.to_owned(), hash.to_owned());
            } else {
                anyhow::bail!("line does not contain hash")
            }

            line.clear();
        }
        Ok(Self { entries })
    }

    pub fn try_collect<E, I: IntoIterator<Item = Result<(String, String), E>>>(
        iter: I,
    ) -> Result<Self, E> {
        let mut entries = BTreeMap::new();
        for item in iter {
            entries.extend(std::iter::once(item?));
        }
        Ok(Self { entries })
    }
}

impl FromIterator<(String, String)> for HashedEntries {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        let mut entries = BTreeMap::new();
        for (item, hash) in iter {
            entries.insert(item, hash);
        }
        Self { entries }
    }
}

impl std::ops::Sub for HashedEntries {
    type Output = BTreeMap<String, Difference>;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut diffs = BTreeMap::new();
        for (path, hash) in self.entries.iter() {
            if let Some(diff_hash) = rhs.entries.get(path) {
                if diff_hash != hash {
                    diffs.insert(path.clone(), Difference::HashDifference);
                }
            } else {
                diffs.insert(path.clone(), Difference::New);
            }
        }
        for path in rhs.entries.keys() {
            if !self.entries.contains_key(path) {
                diffs.insert(path.clone(), Difference::Absence);
            }
        }
        diffs
    }
}
