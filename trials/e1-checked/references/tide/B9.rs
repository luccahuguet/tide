use std::{collections::BTreeMap, fs::Fs, io};
pub struct Store {
    dir: Fs,
    data: BTreeMap<String, String>,
}
impl Store {
    pub fn open(dir: &Fs) -> io::Result<Self> {
        let text = match dir.read_to_string("store.kv") {
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
            dir: dir.clone(),
            data,
        })
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(String::as_str)
    }
    fn save(&self, data: &BTreeMap<String, String>) -> io::Result<()> {
        let mut text = String::new();
        for (k, v) in data {
            text.push_str(&format!("{k}\t{v}\n"));
        }
        self.dir.write_synced("store.kv.tmp", text)?;
        self.dir.rename("store.kv.tmp", "store.kv")?;
        self.dir.sync_dir(".")
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
