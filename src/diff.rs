use anyhow::Context;
use std::collections::{BTreeMap, BTreeSet};

pub fn read<R: std::io::BufRead>(mut reader: R) -> anyhow::Result<BTreeSet<String>> {
    let mut entries = BTreeSet::new();
    let mut line = String::new();
    while reader.read_line(&mut line).context("read line")? > 0 {
        let path = line.trim();
        if path.is_empty() {
            continue;
        }
        entries.insert(path.to_owned());
        line.clear();
    }
    Ok(entries)
}

pub fn read_hashed<R: std::io::BufRead>(mut reader: R) -> anyhow::Result<BTreeMap<String, String>> {
    use ring::digest::SHA256;

    let mut entries = BTreeMap::new();
    let mut line = String::new();
    while reader.read_line(&mut line).context("read line")? > 0 {
        let trimed = line.trim();
        if trimed.is_empty() {
            continue;
        }
        if let Some((hash, path)) = trimed.split_once(", ") {
            if hash.len() != SHA256.output_len() * 2 {
                anyhow::bail!("digest length does not match")
            }
            entries.insert(path.to_owned(), hash.to_owned());
        } else {
            anyhow::bail!("line does not contain hash")
        }

        line.clear();
    }
    Ok(entries)
}
