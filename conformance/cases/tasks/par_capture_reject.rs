//@ rules: PAR-7
// Shared captures must be `Pure + Sync`, and `&mut` captures disjoint: a
// borrowed `RefCell` is not `Sync`, a capability or `Shared` handle, borrowed
// or moved in by reference, is `Sync` but not `Pure` (only the checker rejects
// those), also when the closure is passed by reference, boxed, or both, and two
// `join` closures cannot both borrow one place mutably.
use std::cell::RefCell;
use std::par::join;
use std::sync::Shared;
use std::time::{Clock, Instant};

pub fn ref_cell(log: &RefCell<Vec<u64>>) -> (usize, u64) {
    join(|| log.borrow().len(), || 2) //~ E0277 FX0611
}

pub fn capability(xs: &[u64], clock: &Clock) -> Vec<Instant> {
    xs.par_map(|_| clock.now()) //~ FX0611
}

pub fn moved_in(xs: &[u64], clock: &Clock) -> Vec<Instant> {
    xs.par_map(move |_| clock.now()) //~ FX0611
}

pub fn by_reference(xs: &[u64], clock: &Clock) -> Vec<Instant> {
    let stamp = |_: &u64| clock.now();
    xs.par_map(&stamp) //~ FX0611
}

pub fn boxed(xs: &[u64], clock: &Clock) -> Vec<Instant> {
    xs.par_map(Box::new(|_: &u64| clock.now())) //~ FX0611
}

pub fn boxed_reference(xs: &[u64], clock: &Clock) -> Vec<Instant> {
    let stamp = |_: &u64| clock.now();
    xs.par_map(Box::new(&stamp)) //~ FX0611
}

pub fn shared(xs: &[u64], cell: &Shared<u64>) -> Vec<u64> {
    xs.par_map(|x| x + cell.with(|v| *v).unwrap_or(0)) //~ FX0611
}

pub fn overlapping(total: &mut u64) {
    join(|| *total += 1, || *total += 2); //~ E0524
}
