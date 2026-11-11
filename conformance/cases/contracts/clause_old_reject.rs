//@ rules: CLAUSE-4
// A snapshot is borrow-checked as a local live from entry to return, so it cannot
// hold a borrow the body invalidates; `old` of a type that is neither `Copy` nor
// `Clone` is an error; and `old(e)` runs at entry, so `e` cannot name `result` or
// a quantifier's bound variable. Production reborrows mutable body arguments,
// so the held snapshot conflicts with that reborrow (E0502).
pub struct Token(pub u32);

#[ensures(old(v.get(0)).is_some() ==> v.len() > 1)] //~ E0502
pub fn push_one(v: &mut Vec<u32>) {
    v.push(1);
}

#[ensures(t.0 == old(*t).0 + 1)] //~ E0599
pub fn bump(t: &mut Token) {
    t.0 += 1;
}

#[ensures(old(result) > 0)] //~ error
pub fn one() -> u32 {
    1
}

#[ensures(forall |i: usize| i < v.len() ==> v[i] == old(v[i]) + 1)] //~ error
pub fn bump_all(v: &mut Vec<u32>) {
    for x in v.iter_mut() {
        *x += 1;
    }
}
