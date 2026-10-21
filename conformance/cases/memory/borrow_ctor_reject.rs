//@ rules: BORROW-2
// `fn new(src: &str) -> Self` gives `src` a fresh origin, not the impl's `'a`.
pub struct Lexer<'a> { src: &'a str, pos: usize }

impl<'a> Lexer<'a> {
    pub fn new(src: &str) -> Self { Lexer { src, pos: 0 } } //~ E0621
}
