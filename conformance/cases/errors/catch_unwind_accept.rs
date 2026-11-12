//@ rules: REF-0701
// Isolate risky work in an owned task.
use std::task::{Panicked, Tasks};
pub fn recover(tasks: &Tasks) -> Result<u8, Panicked> { tasks.spawn_isolated(|| 4) }
