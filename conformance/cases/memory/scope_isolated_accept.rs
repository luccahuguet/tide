//@ rules: SCOPE-3, BORROW-3
// `spawn_isolated` takes a closure that owns everything it captures.
use std::task::{Panicked, Tasks};

pub fn run(tasks: &Tasks, name: &str) -> Result<usize, Panicked> {
    let owned = name.to_string();
    tasks.spawn_isolated(move || owned.len())
}
