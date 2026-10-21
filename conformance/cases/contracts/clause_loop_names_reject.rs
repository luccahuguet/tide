//@ rules: CLAUSE-6
// Over an iterator other than `a..b`, the head comes before `next`, so the
// invariant cannot name the loop pattern's bindings.
#[invariants]
pub fn sum(xs: &[u32]) -> u64 {
    let mut t = 0u64;
    for x in xs.iter() {
        invariant!(*x < 100); //~ E0425
        t += *x as u64;
    }
    for (i, x) in xs.iter().enumerate() {
        invariant!(i < xs.len()); //~ E0425
        t += *x as u64;
    }
    t
}
