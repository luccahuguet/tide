//@ rules: LIFE-7
// In an impl header, `'_` is a fresh lifetime parameter of the impl.
pub struct Lexer<'a> {
    src: &'a str,
}
pub struct Collector<'s, 'o> {
    pub src: &'s str,
    pub out: &'o mut Vec<&'s str>,
}

impl Lexer<'_> {
    pub fn len(&self) -> usize {
        self.src.len()
    }
}

impl Collector<'_, '_> {
    pub fn count(&self) -> usize {
        self.out.len()
    }
}
