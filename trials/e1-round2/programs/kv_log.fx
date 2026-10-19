//! A key-value store persisted as an append-only log in one directory.
//!
//! `kv set KEY VALUE`, `kv get KEY`, `kv del KEY`; data lives in `./kv-data`.

use std::collections::BTreeMap;
use std::fs::{File, Fs, Mode, OpenOptions};
use std::io::{self, Read, Write};
use std::json;
use std::process::ExitCode;

const DATA_DIR: &str = "kv-data";
const LOG: &str = "kv.log";

/// One log record, stored as JSON on its own line. JSON escapes newlines
/// inside strings, so an encoded record never contains a raw `\n`.
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
enum Record {
    Set { key: String, value: String },
    Delete { key: String },
}

impl Record {
    fn apply(self, map: &mut BTreeMap<String, String>) {
        match self {
            Record::Set { key, value } => {
                map.insert(key, value);
            }
            Record::Delete { key } => {
                map.remove(&key);
            }
        }
    }
}

pub struct KvStore {
    file: File,
    map: BTreeMap<String, String>,
}

impl KvStore {
    /// Opens the log in `dir`, creating it, and rebuilds the map by
    /// replaying every complete record. A torn last record (a crash before
    /// its sync) was never acknowledged; it is cut off so that later appends
    /// start on a clean line.
    pub fn open(dir: &Fs) -> io::Result<KvStore> {
        let mut file = dir.open_with(LOG, OpenOptions::new().read(true).append(true).create(true))?;
        dir.sync_dir(".")?;                             // the log's directory entry survives a crash
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let complete = bytes.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
        let mut map = BTreeMap::new();
        for line in bytes[..complete].split(|&b| b == b'\n').filter(|l| !l.is_empty()) {
            let record: Record = json::decode(line)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            record.apply(&mut map);
        }
        if complete < bytes.len() {
            file.set_len(complete as u64)?;
            file.sync()?;
        }
        Ok(KvStore { file, map })
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Durable when it returns `Ok`: acknowledge the write only after that.
    pub fn set(&mut self, key: &str, value: &str) -> io::Result<()> {
        self.commit(Record::Set { key: key.to_string(), value: value.to_string() })
    }

    /// Durable when it returns `Ok`.
    pub fn delete(&mut self, key: &str) -> io::Result<()> {
        self.commit(Record::Delete { key: key.to_string() })
    }

    /// Appends and syncs `record`, then applies it in memory. After an error
    /// the log may end in a torn record: drop the store and reopen it.
    fn commit(&mut self, record: Record) -> io::Result<()> {
        let mut line = json::encode(&record);
        line.push(b'\n');
        self.file.write_all(&line)?;
        self.file.sync()?;
        record.apply(&mut self.map);
        Ok(())
    }
}

#[derive(Debug, Error)]
enum CliError {
    #[error("usage: kv set KEY VALUE | kv get KEY | kv del KEY")]
    Usage,
    #[error("no such key: {0:?}")]
    Missing(String),
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Authority: the data directory only.
fn run(data: &Fs, args: &[String]) -> Result<(), CliError> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["set", key, value] => KvStore::open(data)?.set(key, value)?,
        ["get", key] => {
            let kv = KvStore::open(data)?;
            let value = kv.get(key).ok_or_else(|| CliError::Missing(key.to_string()))?;
            println!("{value}");
        }
        ["del", key] => KvStore::open(data)?.delete(key)?,
        _ => return Err(CliError::Usage),
    }
    Ok(())
}

/// Attenuates `fs` to the data directory, creating it durably on first use.
fn open_data_dir(fs: &Fs) -> io::Result<Fs> {
    fs.create_dir_all(DATA_DIR)?;
    fs.sync_dir(".")?;
    fs.dir(DATA_DIR, Mode::ReadWrite)
}

fn main(root: Root) -> ExitCode {
    let args: Vec<String> = root.env.args().skip(1).collect();
    let data = match open_data_dir(&root.fs) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("kv: cannot open {DATA_DIR}: {e}");
            return ExitCode::FAILURE;
        }
    };
    match run(&data, &args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e @ CliError::Usage) => {
            eprintln!("{e}");
            ExitCode::from(2u8)
        }
        Err(e) => {
            eprintln!("kv: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sim::{NodeRoot, SimRoot};
    use std::time::Duration;

    #[test]
    fn replays_sets_and_deletes() {
        let sim = Root::sim(3);
        sim.fs.create_dir_all("/d").unwrap();
        let dir = sim.fs.dir("/d", Mode::ReadWrite).unwrap();
        {
            let mut kv = KvStore::open(&dir).unwrap();
            kv.set("a", "1").unwrap();
            kv.set("b", "line\nbreak").unwrap();
            kv.set("a", "2").unwrap();
            kv.delete("b").unwrap();
        }
        let kv = KvStore::open(&dir).unwrap();
        assert_eq!((kv.get("a"), kv.get("b"), kv.len()), (Some("2"), None, 1));
    }

    #[test]
    fn a_torn_tail_is_cut_off_before_new_appends() {
        let sim = Root::sim(4);
        sim.fs.create_dir_all("/d").unwrap();
        let dir = sim.fs.dir("/d", Mode::ReadWrite).unwrap();
        let mut log = json::encode(&Record::Set { key: "a".into(), value: "1".into() });
        log.push(b'\n');
        let torn = json::encode(&Record::Set { key: "b".into(), value: "2".into() });
        log.extend_from_slice(&torn[..torn.len() / 2]);
        dir.write(LOG, &log).unwrap();
        let mut kv = KvStore::open(&dir).unwrap();
        assert_eq!((kv.get("a"), kv.get("b")), (Some("1"), None));
        kv.set("c", "3").unwrap();                      // must start on a clean line
        let kv = KvStore::open(&dir).unwrap();
        assert_eq!((kv.get("a"), kv.get("c")), (Some("1"), Some("3")));
    }

    /// The node's entry. It rebuilds the store, then writes keys `k{i}` =
    /// `v{i}` continuing after the ones it finds, logging each key only
    /// after `set` returned (that is, after the sync). Keys are contiguous:
    /// every earlier `set` was synced before the next began.
    fn writer(r: NodeRoot) -> io::Result<()> {
        r.fs.create_dir_all("/data")?;
        r.fs.sync_dir("/")?;                            // /data itself survives a crash
        let mut kv = KvStore::open(&r.fs.dir("/data", Mode::ReadWrite)?)?;
        let first = kv.len();
        for i in first..first + 40 {
            kv.set(&format!("k{i}"), &format!("v{i}"))?;
            info!("acked {i}");
        }
        Ok(())
    }

    #[sim_test(seeds = 200)]
    fn acknowledged_writes_survive_crashes(sim: SimRoot) {
        let node = sim.node("kv");
        node.start(writer);
        for _ in 0..3 {
            sim.run_for(Duration::from_millis(sim.rand.gen_range(1..40)));
            sim.faults.crash(&node);
            sim.faults.restart(&node);                  // reruns `writer` over the surviving disk
        }
        sim.run_for(Duration::from_secs(10));           // the last run finishes undisturbed
        sim.faults.crash(&node);                        // unsynced state is lost before we look

        let disk = node.fs();
        let kv = KvStore::open(&disk.dir("/data", Mode::ReadWrite).unwrap()).unwrap();
        let mut acked = 0;
        for record in sim.records().iter().filter(|r| r.node == "kv") {
            if let Some(i) = record.message.strip_prefix("acked ") {
                assert_eq!(kv.get(&format!("k{i}")), Some(format!("v{i}").as_str()), "acknowledged k{i} lost");
                acked += 1;
            }
        }
        assert!(acked >= 40, "the final run acknowledges 40 writes");
        for i in 0..kv.len() {
            assert_eq!(kv.get(&format!("k{i}")), Some(format!("v{i}").as_str()), "k{i} missing or wrong");
        }
    }
}
