//@ rules: LIFE-3
// The suggested rewrites: a lifetime on the impl instead of the trait, `Fn` sugar
// instead of `for<'a>`, and an owned item instead of a generic associated type.
pub trait Parse {
    fn parse(src: &str) -> Self;
}

pub struct Word<'a>(pub &'a str);
impl<'a> Word<'a> {
    pub fn new(src: &'a str) -> Self {
        Word(src)
    }
}

pub fn apply<F>(f: F) -> u8
where
    F: Fn(&u8) -> &u8,
{
    *f(&0)
}

pub trait Source {
    type Item;
    fn next_item(&mut self) -> Option<Self::Item>;
}
