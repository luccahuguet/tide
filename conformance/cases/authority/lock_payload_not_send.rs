//@ rules: LOCK-1
// A payload that is not `Send`: rustc rejects it through `Shared::new`'s bound.
use std::rc::Rc;
use std::sync::Shared;
use std::task::Tasks;

pub fn make(tasks: &Tasks) {
    let _cell = Shared::new(Rc::new(0u64), tasks); //~ E0277
}
