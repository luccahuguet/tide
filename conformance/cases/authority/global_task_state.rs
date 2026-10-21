//@ rules: GLOBAL-3
// Per-task state is a value the caller owns and passes down.
use std::cell::Cell;

pub fn bump(hits: &Cell<u32>) -> u32 {
    hits.set(hits.get() + 1);
    hits.get()
}
