//@ rules: CLAUSE-6
// Over `a..b`, a `for` loop's invariant sees the loop variable as the next index;
// over another iterator it names only places that exist at the loop head.
#[invariants]
pub fn sum(xs: &[u32]) -> u64 {
    let mut t = 0u64;
    for i in 0..xs.len() {
        invariant!(i <= xs.len());
        t += xs[i] as u64;
    }
    let mut seen = 0;
    for x in xs.iter() {
        invariant!(seen <= xs.len());
        t += *x as u64;
        seen += 1;
    }
    t
}
