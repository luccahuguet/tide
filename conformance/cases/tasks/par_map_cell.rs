//@ rules: PAR-4
// `par_map` needs `T: Pure + Sync`; `Cell<u32>` is not `Sync`, so rustc
// rejects it.
use std::cell::Cell;

pub fn doubled(xs: &[Cell<u32>]) -> Vec<u32> {
    xs.par_map(|c| c.get() * 2) //~ E0277
}
