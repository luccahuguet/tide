//@ rules: LOAN-5
// Rust's three two-phase cases: a method receiver's autoref, a mutable reborrow in a
// call argument, and the `&mut` of a compound assignment on a non-primitive type.
pub fn autoref(v: &mut Vec<usize>) {
    v.push(v.len());
}

pub fn reborrow_arg(v: &mut Vec<usize>) {
    Vec::push(v, v.len());
}

pub fn compound(s: &mut String) {
    *s += &s.len().to_string();
}
