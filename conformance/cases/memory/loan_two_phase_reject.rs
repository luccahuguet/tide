//@ rules: LOAN-5
// An explicit `&mut` is not one of the three cases, so it is active at once and the
// shared borrow in the next argument conflicts with it.
pub fn explicit_arg(mut v: Vec<usize>) {
    Vec::push(&mut v, v.len()); //~ E0502
}

pub fn explicit_receiver(mut v: Vec<usize>) {
    (&mut v).push(v.len()); //~ E0502
}
