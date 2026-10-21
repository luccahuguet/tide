//@ rules: LOCK-1
// A `Shared` handle as a payload is `Send` but not `Pure`: only the checker rejects it.
use std::sync::Shared;
use std::task::Tasks;

pub fn nest(tasks: &Tasks) {
    let inner = Shared::new(0u64, tasks);
    let _outer = Shared::new(inner, tasks); //~ FX0606
}
