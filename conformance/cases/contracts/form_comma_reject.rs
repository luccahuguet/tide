//@ rules: FORM-1
// A top-level comma in a clause is an error, whose safe fix splits the clause in
// two. The production macro reports the clause's own span.
#[requires(a > 0, b > 0)] //~ error FX0804
pub fn sum(a: u32, b: u32) -> u32 {
    a + b
}

#[invariants]
pub fn count(n: u32) -> u32 {
    let mut k = 0;
    for i in 0..n {
        invariant!(i <= n, k == i); //~ error FX0804
        k += 1;
    }
    k
}
