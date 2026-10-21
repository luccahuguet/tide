//@ rules: LIFE-1
// Lifetime parameters come before type parameters.
pub fn get<'a, T>(x: &'a T) -> &'a T {
    x
}

pub struct Wrap<'a, T> {
    pub item: &'a T,
}
