//@ rules: PURE-1
// A recursive type takes the greatest fixed point, so a list of `Pure` items is `Pure`.
pub enum List {
    Nil,
    Cons(u32, Box<List>),
}

fn needs_pure<T: Pure>(_t: &T) {}

pub fn check(l: &List) {
    needs_pure(l)
}
