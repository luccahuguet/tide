//@ rules: LOCK-5
// `with` returns `Result<R, LockError>`, not `R`.
use std::sync::Shared;

pub fn read(hits: &Shared<u64>) -> u64 {
    hits.with(|n| *n) //~ E0308
}
