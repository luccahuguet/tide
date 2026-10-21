//@ rules: CLAUSE-2
// Clauses are typed as code: `a + b` is a `u8`, so comparing it with a `u16` result
// is a type error, a clause must be `bool`, and an unresolved name is an error.
#[ensures(result == a + b)] //~ E0308
pub fn widen(a: u8, b: u8) -> u16 {
    a as u16 + b as u16
}

#[requires(n)] //~ E0308
pub fn count(n: u32) -> u32 {
    n
}

#[requires(limit > 0)] //~ E0425
pub fn cap(n: u32) -> u32 {
    n
}
