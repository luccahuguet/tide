//@ rules: SCOPE-3, BORROW-3
// `spawn_isolated` takes a closure that owns everything it captures.
use std::task::{Panicked, Tasks};

pub fn run(tasks: &Tasks, name: &str) -> Result<usize, Panicked> {
    let owned = name.to_string();
    tasks.spawn_isolated(move || owned.len())
}

// A static borrow is owned for this rule; it must not be rejected by capture shape.
pub fn static_data(tasks: &Tasks, name: &'static str) -> Result<usize, Panicked> {
    tasks.spawn_isolated(move || name.len())
}
