//@ rules: FORM-2
// `result` is valid only in an `ensures`, and a function with an `ensures` cannot
// have a parameter named `result`; the safe fix renames the parameter. Inside a
// clause, `result` is always the clause word (04 §1.1), so `check`'s `requires`
// does not name its parameter. rustc accepts the parameter, and rejects `result`
// where the lowering has not bound it.
#[ensures(result > 0)]
pub fn first(result: u32) -> u32 { //~ FX0805
    result + 1
}

#[requires(result > 0)] //~ FX0805
pub fn check(result: u32) -> u32 {
    result
}

#[requires(result > n)] //~ E0425 FX0805
pub fn grow(n: u32) -> u32 {
    n + 1
}

#[ensures(result == n)]
pub fn walk(n: u32) -> u32 {
    let mut k = 0;
    while k < n {
        invariant!(k <= result); //~ E0425 FX0805
        k += 1;
    }
    k
}
