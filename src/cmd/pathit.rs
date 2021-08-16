use anyhow::Context;
use clap::Parser;
use pathit::iter::IterDir;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(long_about = None)]
struct Args {
    /// iterate including hashes of file content
    #[arg(long)]
    hash: bool,

    /// path to iterate
    path: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let root = args
        .path
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    for entry in IterDir::new(root.clone()) {
        let entry = entry.context("iterate path")?;
        let filepath = pathit::iter::normalize_path(entry.as_path(), &root);
        if args.hash {
            let filehash = pathit::iter::hash_file(entry.as_path()).context("hash file")?;
            println!("{}, {}", filehash, filepath);
        } else {
            println!("{}", filepath);
        }
    }

    Ok(())
}
