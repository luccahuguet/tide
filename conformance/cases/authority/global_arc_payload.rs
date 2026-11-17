//@ rules: GLOBAL-6, LOCK-1
// `Arc` is a synchronization handle, not `Pure`, so it is no `Shared` payload.
use std::sync::{Arc, Shared};
use std::task::Tasks;

pub fn nest(tasks: &Tasks) {
    let table = Arc::new(7u32, tasks);
    let _cell = Shared::new(table, tasks); //~ TD0606
}
