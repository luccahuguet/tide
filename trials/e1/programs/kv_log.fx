// src/main.fx of package `trials/kv-log`
//
// A key-value store persisted as an append-only log in one directory.
// `kv_log DIR get KEY | put KEY VALUE | del KEY`
//
// Record format (little-endian):
//   [len: u32][checksum: u32][payload: len bytes]
//   payload = [tag: u8][key_len: u32][key][value]
// A write is acknowledged (returns Ok) only after its record is synced. On
// startup the log is replayed up to the first incomplete or corrupt record;
// that tail was never acknowledged and is truncated away.

use std::collections::BTreeMap;
use std::fs::{File, Fs, Mode, OpenOptions};
use std::io::{self, Write};

pub const LOG_FILE: &str = "kv.log";
const HEADER: usize = 8;
const TAG_PUT: u8 = 1;
const TAG_DELETE: u8 = 2;
const USAGE: &str = "usage: kv_log DIR (get KEY | put KEY VALUE | del KEY)";

#[derive(Debug, Error)]
pub enum KvError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("record too large: {0} bytes")]
    TooLarge(usize),
    #[error("a previous write failed; reopen the store to recover")]
    NeedsRecovery,
}

/// One decoded record; borrows from the log buffer.
#[derive(Debug, PartialEq)]
enum Op {
    Put { key: &str, value: &str },
    Delete { key: &str },
}

/// FNV-1a. Checksums must say that they wrap (4.1).
fn checksum(bytes: &[u8]) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for &b in bytes {
        h ^= u32::from(b);
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

fn read_u32(buf: &[u8], at: usize) -> Option<u32> {
    let bytes: [u8; 4] = buf.get(at..at.checked_add(4)?)?.try_into().ok()?;
    Some(u32::from_le_bytes(bytes))
}

/// Authority-free.
fn encode(tag: u8, key: &str, value: &str) -> Result<Vec<u8>, KvError> {
    let key_len = u32::try_from(key.len()).map_err(|_| KvError::TooLarge(key.len()))?;
    let mut payload = Vec::with_capacity(5 + key.len() + value.len());
    payload.push(tag);
    payload.extend_from_slice(&key_len.to_le_bytes());
    payload.extend_from_slice(key.as_bytes());
    payload.extend_from_slice(value.as_bytes());
    let len = u32::try_from(payload.len()).map_err(|_| KvError::TooLarge(payload.len()))?;
    let mut record = Vec::with_capacity(HEADER + payload.len());
    record.extend_from_slice(&len.to_le_bytes());
    record.extend_from_slice(&checksum(&payload).to_le_bytes());
    record.extend_from_slice(&payload);
    Ok(record)
}

/// Authority-free. The record at the start of `buf` and its size, or `None`
/// if `buf` does not start with a complete, valid record.
fn decode(buf: &[u8]) -> Option<(Op, usize)> {
    let len = read_u32(buf, 0)? as usize; // u32 -> usize is lossless
    let sum = read_u32(buf, 4)?;
    let end = HEADER.checked_add(len)?;
    let payload = buf.get(HEADER..end)?;
    if checksum(payload) != sum {
        return None;
    }
    let (&tag, rest) = payload.split_first()?;
    let key_len = read_u32(rest, 0)? as usize;
    let key_end = 4usize.checked_add(key_len)?;
    let key = std::str::from_utf8(rest.get(4..key_end)?).ok()?;
    let value = std::str::from_utf8(&rest[key_end..]).ok()?;
    let op = match tag {
        TAG_PUT => Op::Put { key, value },
        TAG_DELETE if value.is_empty() => Op::Delete { key },
        _ => return None,
    };
    Some((op, end))
}

/// Authority-free. Rebuilds the map from a log; also returns the length of the
/// valid prefix.
fn replay(log: &[u8]) -> (BTreeMap<String, String>, usize) {
    let mut map = BTreeMap::new();
    let mut pos = 0;
    while let Some((op, size)) = decode(&log[pos..]) {
        match op {
            Op::Put { key, value } => {
                map.insert(key.to_string(), value.to_string());
            }
            Op::Delete { key } => {
                map.remove(key);
            }
        }
        pos += size;
    }
    (map, pos)
}

pub struct KvStore {
    log: File,
    map: BTreeMap<String, String>,
    broken: bool,
}

impl KvStore {
    /// Authority: the directory `dir`, nothing else.
    pub fn open(dir: &Fs) -> Result<KvStore, KvError> {
        let bytes = match dir.read(LOG_FILE) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e.into()),
        };
        let (map, valid) = replay(&bytes);
        let log = dir.open_with(LOG_FILE, OpenOptions::new().create(true).append(true))?;
        if valid < bytes.len() {
            // A torn or corrupt tail: written but never acknowledged. Cut it off
            // so new records are not appended after garbage.
            warn!("{LOG_FILE}: truncating {} bytes of unacknowledged tail", bytes.len() - valid);
            log.set_len(valid as u64)?;
            log.sync_all()?;
        }
        Ok(KvStore { log, map, broken: false })
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `Ok` only once the write is durable.
    pub fn put(&mut self, key: &str, value: &str) -> Result<(), KvError> {
        let record = encode(TAG_PUT, key, value)?;
        self.append(&record)?;
        self.map.insert(key.to_string(), value.to_string());
        Ok(())
    }

    /// Returns whether the key existed. Durable on `Ok`.
    pub fn delete(&mut self, key: &str) -> Result<bool, KvError> {
        if !self.map.contains_key(key) {
            return Ok(false);
        }
        let record = encode(TAG_DELETE, key, "")?;
        self.append(&record)?;
        self.map.remove(key);
        Ok(true)
    }

    fn append(&mut self, record: &[u8]) -> Result<(), KvError> {
        if self.broken {
            return Err(KvError::NeedsRecovery);
        }
        if let Err(e) = self.log.write_all(record).and_then(|()| self.log.sync_data()) {
            // The log may now end in a partial record. Appending after it would
            // hide later records from replay, so stop until a reopen truncates it.
            self.broken = true;
            return Err(e.into());
        }
        Ok(())
    }
}

fn main(root: Root) -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = root.env.args().collect();
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    let [_, dir, cmd @ ..] = argv.as_slice() else {
        return Err(USAGE.into());
    };
    root.fs.create_dir_all(dir)?;
    let data = root.fs.dir(dir, Mode::ReadWrite)?; // the store sees one directory
    let mut store = KvStore::open(&data)?;
    match cmd {
        ["get", key] => match store.get(key) {
            Some(value) => println!("{value}"),
            None => return Err(format!("no such key: {key}").into()),
        },
        ["put", key, value] => store.put(key, value)?,
        ["del", key] => {
            if !store.delete(key)? {
                return Err(format!("no such key: {key}").into());
            }
        }
        _ => return Err(USAGE.into()),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sim::{Faults, SimRoot};

    #[test]
    fn replay_stops_at_a_torn_tail() {
        let mut log = encode(TAG_PUT, "a", "1").unwrap();
        log.extend(encode(TAG_PUT, "b", "2").unwrap());
        log.extend(encode(TAG_DELETE, "a", "").unwrap());
        let whole = log.len();
        log.extend_from_slice(&encode(TAG_PUT, "c", "3").unwrap()[..5]); // crash mid-append

        let (map, valid) = replay(&log);

        assert_eq!(valid, whole);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("b").map(String::as_str), Some("2"));
    }

    #[test]
    fn decode_rejects_a_flipped_bit() {
        let mut rec = encode(TAG_PUT, "key", "value").unwrap();
        assert_eq!(decode(&rec), Some((Op::Put { key: "key", value: "value" }, rec.len())));
        let last = rec.len() - 1;
        rec[last] ^= 0x01;
        assert_eq!(decode(&rec), None);
    }

    fn data_dir(sim: &SimRoot) -> Fs {
        sim.fs.create_dir_all("/data").unwrap();
        sim.fs.dir("/data", Mode::ReadWrite).unwrap()
    }

    #[test]
    fn state_survives_a_clean_restart() {
        let sim = Root::sim(5);
        let dir = data_dir(&sim);
        {
            let mut store = KvStore::open(&dir).unwrap();
            store.put("a", "1").unwrap();
            store.put("b", "2").unwrap();
            store.put("a", "3").unwrap();
            assert!(store.delete("b").unwrap());
        }
        let store = KvStore::open(&dir).unwrap();
        assert_eq!(store.get("a"), Some("3"));
        assert_eq!(store.get("b"), None);
        assert_eq!(store.len(), 1);
    }

    fn open_eventually(dir: &Fs) -> KvStore {
        for _ in 0..20 {
            if let Ok(store) = KvStore::open(dir) {
                return store;
            }
        }
        panic!("store did not open in 20 attempts");
    }

    /// Crash and restart the node repeatedly under random disk errors.
    /// Every acknowledged write must survive. The one write in doubt at each
    /// crash (it returned an error) may or may not have landed.
    #[test]
    fn no_acknowledged_write_is_lost(seed: u64) {
        let sim = Root::sim(seed);
        let dir = data_dir(&sim);
        sim.world.faults(Faults { disk_error: 0.05, ..Faults::none() });

        let mut acked: BTreeMap<String, String> = BTreeMap::new();
        let mut in_doubt: Option<(String, String)> = None;

        for round in 0..6 {
            let mut store = open_eventually(&dir);

            if let Some((key, value)) = in_doubt.take() {
                if store.get(&key) == Some(value.as_str()) {
                    acked.insert(key, value); // it reached the disk before the crash
                }
            }
            for (key, value) in &acked {
                assert_eq!(store.get(key), Some(value.as_str()), "round {round}: lost write to {key:?}");
            }
            assert_eq!(store.len(), acked.len(), "round {round}: phantom keys");

            for i in 0..10 {
                let key = format!("k{}", (round * 3 + i) % 7);
                let value = format!("r{round}w{i}");
                match store.put(&key, &value) {
                    Ok(()) => {
                        acked.insert(key, value);
                    }
                    Err(_) => {
                        in_doubt = Some((key, value));
                        break;
                    }
                }
            }

            sim.world.crash(); // power loss: unsynced data is gone, open files are dead
        }
    }
}
