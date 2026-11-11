//@ rules: FORM-4
// Loop clauses open the loop body, in a function carrying `#[invariants]` or a
// contract attribute, which rewrites each loop in the body. No case has a `loop`:
// production lowering rewrites its head checks too (04 CLAUSE-7).
#[invariants]
pub fn sum(xs: &[u32]) -> u64 {
    let mut t = 0u64;
    for i in 0..xs.len() {
        invariant!(i <= xs.len());
        t += xs[i] as u64;
    }
    t
}

#[requires(n < 100)]
pub fn halve(n: u32) -> u32 {
    let mut k = n;
    while k > 1 {
        invariant!(k <= n);
        invariant!(k > 0);
        k /= 2;
    }
    k
}

#[ensures(result <= xs.len())]
pub fn nonzero(xs: &[u32]) -> usize {
    let mut it = xs.iter();
    let mut c = 0;
    while let Some(x) = it.next() {
        invariant!(c <= xs.len());
        c += (*x != 0) as usize;
    }
    c
}
