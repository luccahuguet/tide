use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};
pub struct RotLog {
    dir: PathBuf,
    max: u64,
    size: u64,
}
impl RotLog {
    pub fn open(dir: &Path, max_bytes: u64) -> io::Result<Self> {
        let size = match fs::metadata(dir.join("current.log")) {
            Ok(m) => m.len(),
            Err(e) if e.kind() == io::ErrorKind::NotFound => 0,
            Err(e) => return Err(e),
        };
        Ok(Self {
            dir: dir.into(),
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
            match fs::rename(self.dir.join("old.1.log"), self.dir.join("old.2.log")) {
                Ok(()) => (),
                Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                Err(e) => return Err(e),
            }
            fs::rename(self.dir.join("current.log"), self.dir.join("old.1.log"))?;
            self.size = 0;
        }
        let mut f = OpenOptions::new()
            .append(true)
            .create(true)
            .open(self.dir.join("current.log"))?;
        writeln!(f, "{line}")?;
        self.size += bytes;
        Ok(())
    }
}
pub fn read_all(dir: &Path) -> io::Result<Vec<String>> {
    let mut out = Vec::new();
    for name in ["old.2.log", "old.1.log", "current.log"] {
        match fs::read_to_string(dir.join(name)) {
            Ok(s) => out.extend(s.lines().map(String::from)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(e),
        }
    }
    Ok(out)
}
