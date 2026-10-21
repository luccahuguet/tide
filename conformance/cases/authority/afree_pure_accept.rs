//@ rules: AFREE-1
// `#[pure]` on functions whose parameter types are `Pure`, a generic one by its bound.
#[pure]
pub fn total(xs: &[u32]) -> u32 {
    xs.iter().sum()
}

#[pure]
pub fn apply<F: Fn(u32) -> u32 + Pure>(f: F, x: u32) -> u32 {
    f(x)
}
