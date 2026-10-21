//@ rules: LOCK-4
// A `with` closure that captures a capability is not `Pure`.
use std::fs::Fs;
use std::sync::{LockError, Shared};

pub fn record(hits: &Shared<u64>, fs: &Fs) -> Result<bool, LockError> {
    hits.with(|n| { //~ FX0603
        *n += 1;
        fs.read("/tmp/flag").is_ok()
    })
}
