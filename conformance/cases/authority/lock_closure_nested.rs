//@ rules: LOCK-4
// A `with` closure that captures another `Shared` handle, to nest a second `with`.
use std::sync::{LockError, Shared};

pub fn copy(a: &Shared<u64>, b: &Shared<u64>) -> Result<(), LockError> {
    a.with(|x| { //~ FX0603
        let _ = b.with(|y| *y = *x);
    })
}
