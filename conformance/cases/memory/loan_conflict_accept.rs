//@ rules: LOAN-2
// A loan is live only while its origin is: the shared borrow ends at its last
// use, so the push does not conflict.
pub fn last_use(v: &mut Vec<u8>) {
    let first = &v[0];
    let x = *first;
    v.push(x);
}
