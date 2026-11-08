use std::{
    fs::{Fs, OpenOptions},
    io::{self, Write},
};
pub struct RotLog {
    dir: Fs,
    max: u64,
    size: u64,
}
impl RotLog {
    pub fn open(dir: &Fs, max_bytes: u64) -> io::Result<Self> {
        let size = match dir.metadata("current.log") {
            Ok(m) => m.len(),
            Err(e) if e.kind() == io::ErrorKind::NotFound => 0,
            Err(e) => return Err(e),
        };
        Ok(Self {
            dir: dir.clone(),
            max: max_bytes,
            size,
        })
    }
    pub fn append(&mut self, line: &str) -> io::Result<()> {
        if line.contains('\n') {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        let bytes = u64::try_from(line.len()).unwrap() + 1;
        if self.size > 0 && self.size.saturating_add(bytes) > self.max {
            match self.dir.rename("old.1.log", "old.2.log") {
                Ok(()) => (),
                Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                Err(e) => return Err(e),
            }
            self.dir.rename("current.log", "old.1.log")?;
            self.size = 0;
        }
        let mut f = self
            .dir
            .open_with("current.log", OpenOptions::new().append(true).create(true))?;
        writeln!(f, "{line}")?;
        self.size += bytes;
        Ok(())
    }
}
pub fn read_all(dir: &Fs) -> io::Result<Vec<String>> {
    let mut out = Vec::new();
    for name in ["old.2.log", "old.1.log", "current.log"] {
        match dir.read_to_string(name) {
            Ok(s) => out.extend(s.lines().map(String::from)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(e),
        }
    }
    Ok(out)
}
