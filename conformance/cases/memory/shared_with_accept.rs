//@ rules: SHARED-1
// The `with` and `with2` closures use their payload references inside and
// return owned values.
use std::sync::{LockError, Shared};

pub fn transfer(from: &Shared<u64>, to: &Shared<u64>, amount: u64) -> Result<u64, LockError> {
    Shared::with2(from, to, |a, b| {
        *a -= amount;
        *b += amount;
        *b
    })
}

pub fn names(cell: &Shared<Vec<String>>) -> Result<String, LockError> {
    cell.with(|v| v.first().cloned().unwrap_or_default())
}
