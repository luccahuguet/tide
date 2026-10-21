//@ rules: ORIGIN-4
// Data behind a reference does not die with the function, so a borrow of it may be
// returned.
pub fn first(v: &[u8]) -> &u8 {
    &v[0]
}

pub struct Lexer<'a> {
    src: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn all(&self) -> &'a str {
        &self.src[..]
    }
}
