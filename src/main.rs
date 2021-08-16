mod diff;
mod iter_dir;
mod iterator;

use self::diff::{Difference, Entries, HashedEntries};
use self::iterator::{HashedPathIterator, PathIterator};
use std::path::PathBuf;

fn print_differences(diffs: std::collections::BTreeMap<String, Difference>) {
    for (entry, diff) in diffs {
        match diff {
            Difference::New => println!("+ {}", entry),
            Difference::Absence => println!("- {}", entry),
            Difference::HashDifference => println!("x {}", entry),
        }
    }
}

fn parse_args() -> clap::ArgMatches {
    use clap::{Arg, ArgAction, ArgGroup};
    clap::Command::new("pathit")
        .disable_version_flag(true)
        .args(&[
            Arg::new("hash")
                .long("hash")
                .action(ArgAction::SetTrue)
                .help("iterate and/or compare including hashes of file content"),
            Arg::new("dir")
                .short('d')
                .long("dir")
                .value_name("PATH")
                .value_parser(clap::value_parser!(PathBuf))
                .help("compare with specified directory"),
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("PATH")
                .value_parser(clap::value_parser!(PathBuf))
                .help("compare with entries listed in specified file (or \"-\" for STDIN)"),
            Arg::new("PATH")
                .value_parser(clap::value_parser!(PathBuf))
                .required(true)
                .help("path to iterate"),
        ])
        .group(ArgGroup::new("diff").args(&["dir", "file"]))
        .get_matches()
}

fn main() -> anyhow::Result<()> {
    let args = parse_args();
    let root = args.get_one::<PathBuf>("PATH").unwrap();

    match (args.contains_id("diff"), args.get_flag("hash")) {
        (true, true) => {
            let diff_from = if let Some(diff_path) = args.get_one::<PathBuf>("dir") {
                HashedEntries::try_collect(HashedPathIterator::new(diff_path))?
            } else if let Some(diff_path) = args.get_one::<PathBuf>("file") {
                if diff_path.as_os_str() == "-" {
                    HashedEntries::read(std::io::stdin().lock())?
                } else {
                    HashedEntries::read(std::io::BufReader::new(std::fs::File::open(diff_path)?))?
                }
            } else {
                unreachable!()
            };
            let paths = HashedEntries::try_collect(HashedPathIterator::new(&root))?;
            print_differences(paths - diff_from);
        }
        (true, false) => {
            let diff_from = if let Some(diff_path) = args.get_one::<PathBuf>("dir") {
                Entries::try_collect(PathIterator::new(diff_path))?
            } else if let Some(diff_path) = args.get_one::<PathBuf>("file") {
                if diff_path.as_os_str() == "-" {
                    Entries::read(std::io::stdin().lock())?
                } else {
                    Entries::read(std::io::BufReader::new(std::fs::File::open(diff_path)?))?
                }
            } else {
                unreachable!()
            };
            let paths = Entries::try_collect(PathIterator::new(&root))?;
            print_differences(paths - diff_from);
        }
        (false, true) => {
            for item in HashedPathIterator::new(&root) {
                let (entry, hash) = item?;
                println!("{}, {}", hash, entry);
            }
        }
        (false, false) => {
            for entry in PathIterator::new(&root) {
                println!("{}", entry?);
            }
        }
    }

    Ok(())
}
