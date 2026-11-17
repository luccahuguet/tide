//@ rules: ADMIT-1
// Red team (fx-qlek.5, Achilles W5): every I/O method of Rust's `Path` at the pinned nightly,
// through `&Path`, `PathBuf`'s deref and a qualified path, and `std::path::absolute`, which reads
// the working directory. Admission lists `std::path`'s items one by one, so a method a later
// rustc adds is not admitted until someone lists it.
use std::path::{Path, PathBuf};

pub fn probe(p: &Path, b: PathBuf) -> bool {
    let _ = std::path::absolute(p); //~ TD0520
    let _ = p.try_exists(); //~ TD0520
    let _ = b.read_dir(); //~ TD0520
    let _ = p.canonicalize(); //~ TD0520
    let _ = p.read_link(); //~ TD0520
    let _ = p.symlink_metadata(); //~ TD0520
    let _ = p.is_dir(); //~ TD0520
    let _ = p.is_symlink(); //~ TD0520
    let _ = <Path>::metadata(p); //~ TD0520
    p.exists() || b.is_file() //~ TD0520
}

pub fn pure(p: &Path) -> PathBuf {
    let mut b = p.join("x").with_extension("y");
    b.push(p.file_name().unwrap_or_default());
    let _ = p.components().count() + p.ancestors().count();
    let _ = p.strip_prefix("/").is_ok() && p.is_absolute() && p.starts_with("/");
    for c in p.components() {
        let _ = match c {
            std::path::Component::Prefix(prefix) => prefix.kind().is_verbatim(),
            std::path::Component::Normal(s) => s.is_empty(),
            std::path::Component::RootDir
            | std::path::Component::CurDir
            | std::path::Component::ParentDir => false,
        };
    }
    b
}
