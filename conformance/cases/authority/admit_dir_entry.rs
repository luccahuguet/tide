//@ rules: ADMIT-1
// Tide's `ReadDir`, `DirEntry`, `Metadata`, and `FileType` are its std's own types, so their
// methods are admitted (03 §1.1, "Directory listings"), unlike Rust's.
use std::fs::Fs;
use std::io;
use std::path::PathBuf;

pub fn files(fs: &Fs) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in fs.read_dir(".")? {
        let entry = entry?;
        if entry.metadata()?.is_file() && !entry.file_type()?.is_symlink() {
            out.push(entry.path());
        }
    }
    Ok(out)
}

pub fn size(fs: &Fs, name: &str) -> io::Result<u64> {
    let meta = fs.metadata(name)?;
    Ok(if meta.is_dir() { 0 } else { meta.len() })
}
