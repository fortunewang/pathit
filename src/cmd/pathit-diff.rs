// use pathit::iterator::{HashedPathIterator, PathIterator};
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

    /// compare with entries listed in specified file (or "-" for STDIN)
    #[arg(short = 'f', long = "file")]
    diff_file: PathBuf,

    /// collect different files into specified directory
    #[arg(short, long)]
    collect: Option<PathBuf>,

    /// path to iterate
    path: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let root = args
        .path
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    if args.hash {
        let mut src = if args.diff_file.as_os_str() == "-" {
            pathit::diff::read_hashed(std::io::stdin().lock()).context("read file")?
        } else {
            pathit::diff::read_hashed(std::io::BufReader::new(
                std::fs::File::open(args.diff_file).context("open file")?,
            ))
            .context("read file")?
        };

        for entry in IterDir::new(root.clone()) {
            let entry = entry.context("iterate path")?;
            let filepath = pathit::iter::normalize_path(entry.as_path(), &root);
            let filehash = pathit::iter::hash_file(entry.as_path()).context("hash file")?;
            if let Some(srchash) = src.remove(&filepath) {
                if srchash != filehash {
                    println!("x {}", filepath);
                    if let Some(base) = &args.collect {
                        let intopath = base.join(filepath);
                        if entry.is_file() {
                            std::fs::create_dir_all(intopath.parent().unwrap())
                                .context("create dir")?;
                            std::fs::copy(entry, intopath).context("copy file")?;
                        } else {
                            std::fs::create_dir_all(intopath).context("create dir")?;
                        }
                    }
                }
            } else {
                println!("+ {}", filepath);
                if let Some(base) = &args.collect {
                    let intopath = base.join(filepath);
                    if entry.is_file() {
                        std::fs::create_dir_all(intopath.parent().unwrap())
                            .context("create dir")?;
                        std::fs::copy(entry, intopath).context("copy file")?;
                    } else {
                        std::fs::create_dir_all(intopath).context("create dir")?;
                    }
                }
            }
        }

        for (entry, _) in src {
            println!("- {}", entry);
        }
    } else {
        let mut src = if args.diff_file.as_os_str() == "-" {
            pathit::diff::read(std::io::stdin().lock()).context("read file")?
        } else {
            pathit::diff::read(std::io::BufReader::new(
                std::fs::File::open(args.diff_file).context("open file")?,
            ))
            .context("read file")?
        };

        for entry in IterDir::new(root.clone()) {
            let entry = entry.context("iterate path")?;
            let filepath = pathit::iter::normalize_path(entry.as_path(), &root);

            if !src.remove(&filepath) {
                println!("+ {}", filepath);
                if let Some(base) = &args.collect {
                    let intopath = base.join(filepath);
                    if entry.is_file() {
                        std::fs::create_dir_all(intopath.parent().unwrap())
                            .context("create dir")?;
                        std::fs::copy(entry, intopath).context("copy file")?;
                    } else {
                        std::fs::create_dir_all(intopath).context("create dir")?;
                    }
                }
            }
        }
        for entry in src {
            println!("- {}", entry);
        }
    }

    Ok(())
}
