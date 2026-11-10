//@ rules: LOCK-4
// Logging in a lock callback is allowed. Nested fn items are not part of the callback.
use std::sync::{LockError, Shared};

pub fn report(a: &Shared<u32>) -> Result<(), LockError> {
    a.with(|n| {
        info!("value: {n}");
        fn unused() { eprintln!("not called"); }
    })
}
