//@ rules: LOCK-1, LOCK-4
// The purity walk reaches a synchronization handle through a user's struct.
use std::sync::{LockError, Shared};
use std::task::Tasks;

struct Wrapped {
    _inner: Shared<u64>,
}

pub fn nest(inner: Shared<u64>, tasks: &Tasks) {
    let _outer = Shared::new(Wrapped { _inner: inner }, tasks); //~ TD0606
}

pub fn capture(target: &Shared<u64>, other: Shared<u64>) -> Result<(), LockError> {
    let wrapped = Wrapped { _inner: other };
    target.with(move |_| drop(wrapped)) //~ TD0603
}
