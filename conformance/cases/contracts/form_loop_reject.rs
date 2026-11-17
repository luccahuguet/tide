//@ rules: FORM-4
// An `invariant!` that no attribute rewrote is a compile error, TD0806: one in a
// function without `#[invariants]` or a contract attribute, one after another
// statement, and one outside a loop. Production's fallback `invariant!` expands
// to `compile_error!`.
pub fn bare(n: u32) -> u32 {
    let mut k = 0;
    while k < n {
        invariant!(k <= n); //~ error TD0806
        k += 1;
    }
    k
}

#[invariants]
pub fn late(n: u32) -> u32 {
    let mut k = 0;
    while k < n {
        k += 1;
        invariant!(k <= n); //~ error TD0806
    }
    k
}

#[invariants]
pub fn outside(n: u32) -> u32 {
    invariant!(n < 10); //~ error TD0806
    n
}
