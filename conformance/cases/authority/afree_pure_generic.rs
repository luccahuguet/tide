//@ rules: AFREE-1
// `#[pure]` covers every instantiation the bounds allow, and an unbounded `F` may capture one.
#[pure]
pub fn apply<F: Fn(u32) -> u32>(f: F, x: u32) -> u32 { //~ FX0511
    f(x)
}
