//@ rules: FORM-2
// `result` names the return value only in an `ensures`. A function without an
// `ensures` may have a parameter named `result`, and a body may bind a local
// named `result` or `old`.
#[requires(n > 0)]
pub fn store(n: u32, result: &mut u32) {
    *result = n;
}

#[ensures(result == n + 1)]
pub fn next(n: u32) -> u32 {
    let old = n;
    let result = old + 1;
    result
}
