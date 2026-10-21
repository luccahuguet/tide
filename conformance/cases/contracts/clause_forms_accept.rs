//@ rules: CLAUSE-1
// v0's clause forms: `result`, `old`, `==>`, and `forall` and `exists` over a
// bounded binder. An unsigned binder needs only an upper bound.
#[requires(forall |i: usize| 0 <= i && i < xs.len() ==> xs[i] > 0)]
#[ensures(exists |i: usize| i < xs.len() && xs[i] == result)]
pub fn least(xs: &[u32]) -> u32 {
    *xs.iter().min().unwrap()
}

#[requires(forall |d: i64| -2 <= d && d < 3 ==> x + d > 0)]
#[ensures(forall |y: u32| ys.contains(&y) ==> y <= result)]
pub fn top(x: i64, ys: &[u32]) -> u32 {
    ys.iter().copied().max().unwrap_or(0)
}

#[ensures(v.len() == old(v.len()) + 1)]
#[ensures(old(v.is_empty()) ==> v[0] == x)]
pub fn push(v: &mut Vec<u32>, x: u32) {
    v.push(x);
}
