//@ rules: CLAUSE-1
// An unbounded quantifier is FX0807. A signed binder needs a lower bound.
// The production macro rejects either with a compile error.
// A v1 form such as `#[decreases]` is not in v0's std
// prelude (04 §1.1), so rustc rejects the unresolved attribute; 04 gives it no FX
// code.
#[requires(forall |d: i64| d < 3 ==> x + d > 0)] //~ error FX0807
pub fn shift(x: i64) -> i64 {
    x
}

#[requires(forall |i: usize| xs[i % xs.len()] > 0)] //~ error FX0807
pub fn ring(xs: &[u32]) -> u32 {
    xs[0]
}

#[ensures(exists |i: u32| result == i * 2)] //~ error FX0807
pub fn even(n: u32) -> u32 {
    n * 2
}

#[decreases(n)] //~ error
pub fn down(n: u32) -> u32 {
    if n == 0 { 0 } else { down(n - 1) }
}
