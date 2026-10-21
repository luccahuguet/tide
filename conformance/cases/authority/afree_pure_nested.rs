//@ rules: AFREE-1
// `#[pure]` asserts authority-freedom of the item it marks only: not of a helper nested in its
// body, which may take a capability, but of a nested helper marked itself, and of a required
// trait method, which has no body (`Self` is bounded by `Pure`, so only `&Clock` fails).
use std::time::Clock;

#[pure]
pub fn next(x: u32) -> u32 {
    fn stamp(clock: &Clock) -> u32 {
        let _ = clock.now();
        0
    }
    x + 1
}

#[pure]
pub fn same(x: u32) -> u32 {
    #[pure]
    fn stamp(_clock: &Clock) -> u32 { //~ FX0511
        0
    }
    x
}

pub trait Stamped: Pure {
    #[pure]
    fn at(&self, clock: &Clock) -> u32; //~ FX0511
}
