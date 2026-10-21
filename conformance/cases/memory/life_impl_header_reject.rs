//@ rules: LIFE-7
// The bare name of a borrowing type in an impl header is an error.
pub struct Lexer<'a> {
    src: &'a str,
}

impl Lexer { //~ E0726
    pub fn len(&self) -> usize {
        self.src.len()
    }
}

impl Clone for Lexer { //~ E0726
    fn clone(&self) -> Self {
        Lexer { src: self.src }
    }
}
