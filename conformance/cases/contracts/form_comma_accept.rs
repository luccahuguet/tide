//@ rules: FORM-1
// One expression per clause: a conjunction is split into repeated clauses, and a
// comma that is not at the top level, as in a call's arguments, is allowed.
#[requires(a > 0)]
#[requires(b > 0)]
#[ensures(result >= u32::max(a, b))]
pub fn sum(a: u32, b: u32) -> u32 {
    a + b
}

#[invariants]
pub fn count(n: u32) -> u32 {
    let mut k = 0;
    for i in 0..n {
        invariant!(i <= n);
        invariant!(k == u32::min(i, n));
        k += 1;
    }
    k
}
