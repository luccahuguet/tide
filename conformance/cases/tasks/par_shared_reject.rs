//@ rules: PAR-6
// A slice method's closure is shared by branches, so it must be `Fn + Sync`
// as well as `Pure + Send`: an owned `Cell` is not `Sync`, a closure that
// moves out a capture is only `FnOnce`, one that assigns to a capture is only
// `FnMut`, passed directly, by `&`, or by `&mut`, and a callable through a
// `&mut`, boxed or borrowed, is never `Fn`, whatever it captures; one that owns
// a capability is not `Pure` (only the checker rejects that).
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

pub fn counted(xs: &[u64]) -> Vec<u64> {
    let mut seen = 0u64;
    xs.par_map(|x| { seen += 1; x + seen }) //~ E0594 FX0610
}

pub fn counted_by_reference(xs: &mut [u64]) {
    let mut seen = 0u64;
    let count = |_: &mut u64| seen += 1; //~ E0525
    xs.par_for(&count); //~ FX0610
}

pub fn counted_by_mut_reference(xs: &mut [u64]) {
    let mut seen = 0u64;
    let mut count = |_: &mut u64| seen += 1;
    xs.par_for(&mut count); //~ E0277 FX0610
}

pub fn counted_boxed(xs: &mut [u64]) {
    let mut seen = 0u64;
    let mut count = |_: &mut u64| seen += 1;
    xs.par_for(Box::new(&mut count)); //~ E0277 FX0610
}

pub fn boxed_mut_reference(xs: &mut [u64]) {
    let mut skip = |_: &mut u64| {};
    xs.par_for(Box::new(&mut skip)); //~ E0277 FX0610
}

pub fn by_reference_to_boxed_mut(xs: &mut [u64]) {
    let mut skip = |_: &mut u64| {};
    let boxed = Box::new(&mut skip);
    xs.par_for(&boxed); //~ E0277 FX0610
}

pub fn owns_capability(xs: &mut [u64], clock: &Clock) {
    let clock = clock.clone();
    xs.par_for(move |_| drop(clock.now())); //~ FX0610
}
