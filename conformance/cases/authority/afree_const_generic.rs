//@ rules: AFREE-2
// A `const fn`'s type parameters, including one in `Self`'s type, need `Pure + Sync` at the
// definition.
pub struct Stack<T> {
    items: [Option<T>; 4],
    len: usize,
}

impl<T> Stack<T> {
    pub const fn len(&self) -> usize { //~ TD0518
        self.len
    }
}

impl<T: Pure + Sync> Stack<T> {
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

pub const fn first<T: Copy>(xs: &[T; 2]) -> T { //~ TD0518
    xs[0]
}

pub const fn last<T: Copy + Pure + Sync>(xs: &[T; 2]) -> T {
    xs[1]
}
