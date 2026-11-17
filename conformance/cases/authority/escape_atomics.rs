//@ rules: ADMIT-1, GLOBAL-1
// Red team (fx-qlek.5): atomics without `Tasks`, as a `static`, through `Default`, and through
// their operations. Every name and impl is `FX0520`; the static is not `Pure` either.
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering}; //~ FX0520

pub static FLAG: AtomicBool = AtomicBool::new(false); //~ FX0502 FX0520

pub fn count() -> usize {
    let n: AtomicUsize = Default::default(); //~ FX0520
    FLAG.store(true, Ordering::SeqCst); //~ FX0520
    n.fetch_add(1, Ordering::Relaxed) //~ FX0520
}
