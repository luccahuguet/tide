//@ rules: GLOBAL-1
// A static must be `Sync`; rustc rejects a `Cell`.
use std::cell::Cell;

pub static HITS: Cell<u32> = Cell::new(0); //~ E0277
