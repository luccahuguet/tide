//@ rules: CLAUSE-4
// In an `ensures`, a `&mut` parameter means the caller's referent at return. A
// parameter whose outer lifetime appears nowhere else may be named bare, and a
// prophetic one through `old`. A `Copy` parameter means its entry value even where
// the body reassigns it, and a parameter the body leaves alone is read in place. A
// `Clone + Pure + Sync` one is cloned at entry, so the body may mutate it.
// The executable fixture also checks entry-value behavior at run time.
pub struct Token(pub u32);

#[ensures(*x == *y)]
pub fn copy_in(x: &mut u32, y: &u32) {
    *x = *y;
}

#[ensures(result == old(x.len()))]
pub fn link<'a>(x: &'a mut Vec<&'a u32>, y: &'a u32) -> usize {
    let n = x.len();
    x.push(y);
    n
}

#[ensures(result == n + 1)]
pub fn inc(mut n: u32) -> u32 {
    n += 1;
    n
}

#[ensures(result == t.0 * 2)]
pub fn double(t: Token) -> u32 {
    t.0 * 2
}

#[ensures(result == v.len() + 1)]
pub fn grow(mut v: Vec<u32>) -> usize {
    v.push(1);
    v.len()
}
