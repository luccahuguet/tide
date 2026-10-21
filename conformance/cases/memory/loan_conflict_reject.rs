//@ rules: LOAN-2
// An access that conflicts with a live loan: a second mutable borrow, a mutable borrow
// under a shared loan, a move, and a write.
pub fn two_mut(mut v: Vec<u8>) {
    let a = &mut v;
    let b = &mut v; //~ E0499
    a.push(1);
    b.push(2);
}

pub fn mut_under_shared(mut v: Vec<u8>) -> u8 {
    let first = &v[0];
    v.push(1); //~ E0502
    *first
}

pub fn move_under_loan(v: Vec<u8>) -> usize {
    let r = &v;
    drop(v); //~ E0505
    r.len()
}

pub fn write_under_loan(mut x: u8) -> u8 {
    let r = &x;
    x = 2; //~ E0506
    *r + x
}
