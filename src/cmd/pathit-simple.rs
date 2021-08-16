use anyhow::Context;
use pathit::iter::IterDir;
use std::io::Write;

fn main() -> anyhow::Result<()> {
    let exe = std::env::current_exe().unwrap();
    let exe = exe.file_name().unwrap();
    
    let output_filename = chrono::Local::now().format("%Y%m%d.txt").to_string();
    let mut output = std::fs::File::create(&output_filename).context("create output file")?;

    let root = std::env::current_dir().unwrap();
    for entry in IterDir::new(root.clone()) {
        let entry = entry.context("iterate path")?;
        let filepath = pathit::iter::normalize_path(entry.as_path(), &root);
        if &filepath == &output_filename {
            continue
        }
        if filepath.as_str() == exe {
            continue
        }

        let filehash = pathit::iter::hash_file(entry.as_path()).context("hash file")?;
        writeln!(&mut output, "{}, {}", filehash, filepath).context("write output file")?;
        println!("{}", filepath);
    }

    Ok(())
}
