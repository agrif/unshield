use std::fs::File;
use std::io::{Result, Write};
use std::path::{Path, PathBuf};

fn help(cmd: &str) {
    let basename = Path::new(cmd).file_name().unwrap().to_str().unwrap();
    eprintln!("Usage:");
    eprintln!("  {} list <ARCHIVE>", basename);
    eprintln!("  {} extract [--raw] <ARCHIVE> <DESTINATION>", basename);
}

fn list(archive: &str) -> Result<()> {
    let file = File::open(archive)?;
    let ar = unshield::Archive::new(file)?;
    for file in ar.list() {
        println!("{}\t{}", file.size, file.path);
    }
    Ok(())
}

fn extract(archive: &str, destination: &str, raw: bool) -> Result<()> {
    let file = File::open(archive)?;
    let mut ar = unshield::Archive::new(file)?;
    let files: Vec<unshield::FileInfo> = ar.list().cloned().collect();
    for file in files {
        let mut dest = PathBuf::new();
        dest.push(destination);
        for part in file.path.split('\\') {
            dest.push(part);
        }

        eprintln!("{}", dest.to_str().unwrap());
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut f = File::create(dest)?;

        if raw {
            let data = ar.load_compressed(&file.path)?;
            f.write_all(&data)?;
        } else {
            let data = ar.load(&file.path)?;
            f.write_all(&data)?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let argsref: Vec<&str> = args.iter().map(|v| v.as_str()).collect();
    match &argsref[..] {
        &[_, "list", archive] => list(archive)?,

        &[_, "extract", archive, destination] => {
            extract(archive, destination, false)?
        }
        &[_, "extract", "--raw", archive, destination] => {
            extract(archive, destination, true)?
        }

        &[cmd, "--help"] => help(cmd),
        &[cmd, "-h"] => help(cmd),
        &[cmd] => help(cmd),
        a => {
            eprintln!("Invalid arguments.");
            help(a[0]);
        }
    }
    Ok(())
}
