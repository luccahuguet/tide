//@ rules: SCOPE-3, BORROW-3
// A `spawn_isolated` closure that borrows from the caller is not `Owned`.
use std::task::{Panicked, Tasks};

pub fn by_capture(tasks: &Tasks) -> Result<usize, Panicked> {
    let owned = String::from("x");
    tasks.spawn_isolated(|| owned.len()) //~ E0373 FX0203
}

pub fn by_parameter(tasks: &Tasks, name: &str) -> Result<usize, Panicked> {
    tasks.spawn_isolated(move || name.len()) //~ E0521 FX0203
}
