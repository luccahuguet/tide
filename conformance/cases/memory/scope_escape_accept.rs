//@ rules: SCOPE-2
// A scope's result may be an owned value its children computed.
use std::task::{TaskError, Tasks};

pub fn run(tasks: &Tasks) -> Result<Vec<u64>, TaskError> {
    tasks.scope(|s| {
        let a = s.spawn(|| Ok(vec![1u64, 2]));
        a.join()
    })
}
