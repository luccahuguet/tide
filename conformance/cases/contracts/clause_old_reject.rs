//@ rules: CLAUSE-4
// A snapshot is borrow-checked as a local live from entry to return, so it cannot
// hold a borrow the body invalidates; `old` of a type that is neither `Copy` nor
// `Clone` is an error; and `old(e)` runs at entry, so `e` cannot name `result` or
// a quantifier's bound variable. The D18 spike runs the body in a closure, so the
// borrow conflict is E0500 there.
pub struct Token(pub u32);

#[ensures(old(v.get(0)).is_some() ==> v.len() > 1)] //~ E0500
pub fn push_one(v: &mut Vec<u32>) {
    v.push(1);
}

#[ensures(t.0 == old(*t).0 + 1)] //~ E0599
pub fn bump(t: &mut Token) {
    t.0 += 1;
}

#[ensures(old(result) > 0)] //~ E0425
pub fn one() -> u32 {
    1
}

#[ensures(forall |i: usize| i < v.len() ==> v[i] == old(v[i]) + 1)] //~ E0425
pub fn bump_all(v: &mut Vec<u32>) {
    for x in v.iter_mut() {
        *x += 1;
    }
}
