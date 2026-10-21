//@ rules: CLAUSE-4
// `old(e)` copies or clones `e`'s value at entry. A method call inside narrows a
// snapshot, a bound variable indexes a snapshot from outside, and a snapshot that
// copies out of a borrow holds no loan the body conflicts with.
pub struct Token(pub u32);

#[ensures(forall |i: usize| i < v.len() ==> v[i] == old(*v)[i] + 1)]
#[ensures(v.len() == old(v.len()))]
pub fn bump_all(v: &mut Vec<u32>) {
    for x in v.iter_mut() {
        *x += 1;
    }
}

#[ensures(t.0 == old(t.0) + 1)]
pub fn bump(t: &mut Token) {
    t.0 += 1;
}

#[ensures(old(v.get(0).copied()).is_some() ==> v.len() > 1)]
pub fn push_one(v: &mut Vec<u32>) {
    v.push(1);
}
