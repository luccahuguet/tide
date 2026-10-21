//@ rules: FORM-1
// A top-level comma in a clause is an error, whose safe fix splits the clause in
// two. The D18 spike's lowering reads the clause as a tuple and negates it (E0600),
// reporting a loop clause's error on the function's attribute.
#[requires(a > 0, b > 0)] //~ E0600 FX0804
pub fn sum(a: u32, b: u32) -> u32 {
    a + b
}

#[invariants] //~ E0600
pub fn count(n: u32) -> u32 {
    let mut k = 0;
    for i in 0..n {
        invariant!(i <= n, k == i); //~ FX0804
        k += 1;
    }
    k
}
