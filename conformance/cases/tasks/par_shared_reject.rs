//@ rules: PAR-6
// A slice method's closure is shared by branches, so it must be `Fn + Sync`
// as well as `Pure + Send`: an owned `Cell` is not `Sync`, a closure that
// moves out a capture is only `FnOnce`, and one that owns a capability is
// not `Pure` (only the checker rejects that).
use std::cell::Cell;
use std::time::Clock;

pub fn owned_cell(xs: &[u64]) -> Vec<u64> {
    let hits = Cell::new(0u64);
    xs.par_map(move |x| x + hits.get()) //~ E0277 FX0610
}

pub fn only_once(xs: &[u64]) -> Vec<Vec<u64>> {
    let seed = vec![1u64];
    xs.par_map(move |_| seed) //~ E0507 FX0610
}

pub fn owns_capability(xs: &mut [u64], clock: &Clock) {
    let clock = clock.clone();
    xs.par_for(move |_| drop(clock.now())); //~ FX0610
}
