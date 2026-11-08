use std::{
    collections::BTreeMap,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};
pub struct Store {
    dir: PathBuf,
    data: BTreeMap<String, String>,
}
impl Store {
    pub fn open(dir: &Path) -> io::Result<Self> {
        let text = match fs::read_to_string(dir.join("store.kv")) {
            Ok(s) => s,
            Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(e),
        };
        let mut data = BTreeMap::new();
        for line in text.split_terminator('\n') {
            let (k, v) = line.split_once('\t').ok_or(io::ErrorKind::InvalidData)?;
            data.insert(k.into(), v.into());
        }
        Ok(Self {
            dir: dir.into(),
            data,
        })
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(String::as_str)
    }
    fn save(&self, data: &BTreeMap<String, String>) -> io::Result<()> {
        let mut file = fs::File::create(self.dir.join("store.kv.tmp"))?;
        for (k, v) in data {
            writeln!(file, "{k}\t{v}")?;
        }
        file.sync_all()?;
        fs::rename(self.dir.join("store.kv.tmp"), self.dir.join("store.kv"))?;
        fs::File::open(&self.dir)?.sync_all()
    }
    pub fn set(&mut self, key: &str, value: &str) -> io::Result<()> {
        if key.contains(['\t', '\n']) || value.contains('\n') {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        let mut next = self.data.clone();
        next.insert(key.into(), value.into());
        self.save(&next)?;
        self.data = next;
        Ok(())
    }
    pub fn remove(&mut self, key: &str) -> io::Result<bool> {
        let mut next = self.data.clone();
        if next.remove(key).is_none() {
            return Ok(false);
        }
        self.save(&next)?;
        self.data = next;
        Ok(true)
    }
}
