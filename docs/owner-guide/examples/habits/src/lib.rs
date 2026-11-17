// Owner's guide, chapter 4: three Rust concurrency habits that Tide replaces.
// Admission rejects all three marked lines.
use std::fs::Fs;
use std::sync::{LockError, Shared};

// A bare OS thread: it has no parent scope, so nothing waits for it or cancels it.
pub fn background() {
    let _ = std::thread::spawn(|| 1); // FX0602: spawn through `Tasks` and a scope
}

// A lock you hold while doing anything at all, including waiting on another lock.
pub fn counter() {
    let _ = std::sync::Mutex::new(0); // FX0604: use `Shared<T>` and `with`
}

// The closure given to `with` runs while the cell is held. It must be `Pure`: it may not
// hold a capability, so it cannot do I/O, wait, or take another cell while holding this one.
pub fn record(hits: &Shared<u64>, fs: &Fs) -> Result<bool, LockError> {
    hits.with(|n| { // FX0603
        *n += 1;
        fs.read("/tmp/flag").is_ok()
    })
}
