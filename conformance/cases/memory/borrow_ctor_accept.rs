//@ rules: BORROW-2
// In `impl<'a> Lexer<'a>`, a constructor returning `Self` takes `src: &'a str`.
pub struct Lexer<'a> { src: &'a str, pos: usize }

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self { Lexer { src, pos: 0 } }
}

pub fn lex(text: &str) -> Lexer<'_> { Lexer { src: text, pos: 0 } }
