//@ rules: PAR-5
// A `join` closure must be `FnOnce + Pure + Send`: one that owns an `Rc` is not
// `Send` (rustc and the checker reject it), and one that owns a capability or
// a `Shared` handle is `Send` but not `Pure` (only the checker rejects it).
use std::par::join;
use std::rc::Rc;
use std::sync::Shared;
use std::time::Clock;

pub fn not_send() -> (u64, u64) {
    let count = Rc::new(1u64);
    join(move || *count, || 2) //~ E0277 FX0610
}

pub fn owns_capability(clock: &Clock) -> u64 {
    let clock = clock.clone();
    let (_now, n) = join(move || clock.now(), || 2u64); //~ FX0610
    n
}

pub fn owns_shared(cell: Shared<u64>) -> u64 {
    let (_r, n) = join(move || cell.with(|v| *v), || 2u64); //~ FX0610
    n
}
