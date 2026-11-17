// Benchmark pair `files`, Rust arm: `tide.rs` in plain Rust, through `std::fs`. In a new
// directory `files-workload` under the working directory, it writes `count` files, appends a
// line to each, lists and reads them back, renames each, and removes them all, the directory
// included. It refuses to start if `files-workload` exists. Both arms make the same calls in
// the same order, with the same read loop.
// Usage: `files [count]`; the default count is the small size the equivalence check runs.
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

const DEFAULT_COUNT: u64 = 50;
const DIR: &str = "files-workload";

#[derive(Debug)]
pub enum RunError {
    Size(String),
    Exists,
    Io(io::Error),
}

impl From<io::Error> for RunError {
    fn from(e: io::Error) -> Self {
        RunError::Io(e)
    }
}

/// The contents of file `i`: `i % 7 + 1` numbered lines.
pub fn contents(i: u64) -> String {
    let mut text = String::new();
    for line in 0..i % 7 + 1 {
        text.push_str(&format!("file {i} line {line}\n"));
    }
    text
}

/// FNV-1a, 64-bit.
pub fn fnv(hash: u64, bytes: &[u8]) -> u64 {
    let mut hash = hash;
    for b in bytes {
        hash = (hash ^ u64::from(*b)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Reads `file` to its end through one fixed 8 KiB buffer, retrying an interrupted read.
pub fn read_all(mut file: File) -> io::Result<Vec<u8>> {
    let (mut data, mut buf) = (Vec::new(), [0; 8192]);
    loop {
        match file.read(&mut buf) {
            Ok(0) => return Ok(data),
            Ok(n) => data.extend_from_slice(&buf[..n]),
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
}

pub fn run(count: u64) -> Result<(), RunError> {
    if fs::exists(DIR)? {
        return Err(RunError::Exists);
    }
    fs::create_dir_all(DIR)?;
    let dir = Path::new(DIR);
    for i in 0..count {
        let mut file = File::create(dir.join(format!("{i:06}.txt")))?;
        file.write_all(contents(i).as_bytes())?;
    }
    for i in 0..count {
        let path = dir.join(format!("{i:06}.txt"));
        let mut file = OpenOptions::new().append(true).open(path)?;
        file.write_all(format!("end of file {i}\n").as_bytes())?;
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(dir)? {
        names.push(entry?.file_name().to_string_lossy().into_owned());
    }
    names.sort();
    let (mut bytes, mut lines, mut hash) = (0, 0, 0xcbf2_9ce4_8422_2325);
    for name in &names {
        let data = read_all(File::open(dir.join(name))?)?;
        bytes += data.len();
        lines += data.iter().filter(|b| **b == b'\n').count();
        hash = fnv(hash, &data);
    }
    for name in &names {
        fs::rename(dir.join(name), dir.join(format!("{name}.done")))?;
    }
    for name in &names {
        fs::remove_file(dir.join(format!("{name}.done")))?;
    }
    fs::remove_dir(DIR)?;
    println!("files: {}", names.len());
    println!("first: {:?}", names.first());
    println!("bytes: {bytes}");
    println!("lines: {lines}");
    println!("fnv: {hash:016x}");
    Ok(())
}

fn main() -> Result<(), RunError> {
    let count = match std::env::args().nth(1) {
        Some(arg) => arg.parse().map_err(|_| RunError::Size(arg))?,
        None => DEFAULT_COUNT,
    };
    run(count)
}
