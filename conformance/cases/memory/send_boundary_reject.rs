//@ rules: SEND-3
// Each task boundary rustc checks: a captured value that is not `Send`, a
// captured borrow of data that is not `Sync`, a result that is not `Send`, and
// a channel message that is not `Send`.
use std::cell::Cell;
use std::rc::Rc;
use std::sync::mpsc::sync_channel;
use std::task::{TaskError, Tasks};

pub fn by_value(tasks: &Tasks) -> Result<(), TaskError> {
    let shared = Rc::new(1u64);
    tasks.scope(|s| {
        s.spawn(move || Ok(*shared)); //~ E0277
        Ok(())
    })
}

pub fn by_reference(tasks: &Tasks, count: &Cell<u64>) -> Result<(), TaskError> {
    tasks.scope(|s| {
        s.spawn(|| Ok(count.get())); //~ E0277
        Ok(())
    })
}

pub fn result(tasks: &Tasks) -> Result<(), TaskError> {
    tasks.scope(|s| {
        s.spawn(|| Ok(Rc::new(2u64))); //~ E0277
        Ok(())
    })
}

pub fn message(tasks: &Tasks) {
    let _pair = sync_channel::<Rc<u64>>(1, tasks); //~ E0277
}
