//@ rules: CAP-2
// Each constructor takes `&Tasks`; cloning a handle needs nothing.
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, Shared};
use std::task::Tasks;

pub fn make(tasks: &Tasks) {
    let hits = Shared::new(0u64, tasks);
    let _same = hits.clone();
    let (_tx, _rx) = sync_channel::<u32>(8, tasks);
    let _table = Arc::new(vec![1u8, 2], tasks);
}
