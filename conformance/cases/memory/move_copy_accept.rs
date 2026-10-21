//@ rules: MOVE-1
// A struct of a shared reference and an integer is `Copy`, so using it by value twice copies it.
#[derive(Clone, Copy)]
pub struct Span<'a> {
    pub src: &'a str,
    pub at: usize,
}

pub fn twice(s: Span<'_>) -> (Span<'_>, Span<'_>) {
    (s, s)
}
