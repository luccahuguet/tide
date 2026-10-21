//@ rules: LOCK-1, LOCK-4, LOCK-5
// A `Send + Pure` payload and a `Pure` closure; `with` returns `Result<R, LockError>`.
use std::sync::{LockError, Shared};
use std::task::Tasks;

pub fn bump(tasks: &Tasks) -> Result<u64, LockError> {
    let hits = Shared::new(0u64, tasks);
    hits.with(|n| {
        *n += 1;
        *n
    })
}
