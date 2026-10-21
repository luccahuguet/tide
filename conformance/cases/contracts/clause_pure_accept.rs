//@ rules: CLAUSE-3
// Clauses call authority-free functions with `Pure + Sync` arguments, take `&mut`
// only to temporaries such as an iterator, and generic code states the bound.
fn valid(x: &u32) -> bool {
    *x < 100
}

#[requires(xs.iter().all(|x| *x > 0))]
#[requires(xs.iter().all(valid))]
pub fn total(xs: &[u32]) -> u32 {
    xs.iter().sum()
}

#[ensures(result == (a == b))]
pub fn same<T: PartialEq + Pure + Sync>(a: &T, b: &T) -> bool {
    a == b
}
