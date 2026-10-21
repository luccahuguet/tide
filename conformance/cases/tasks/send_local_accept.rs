//@ rules: SEND-3
// `Send` is checked only where a value crosses a task boundary: a child may
// build and use an `Rc` inside itself, and the body may hold a `Cell`, as long
// as neither is captured, returned, or sent.
use std::cell::Cell;
use std::rc::Rc;
use std::task::{TaskError, Tasks};

pub fn run(tasks: &Tasks) -> Result<u64, TaskError> {
    tasks.scope(|s| {
        let job = s.spawn(|| {
            let local = Rc::new(20u64);
            Ok(*local + 1)
        });
        let seen = Cell::new(0u64);
        seen.set(job.join()?);
        Ok(seen.get())
    })
}
