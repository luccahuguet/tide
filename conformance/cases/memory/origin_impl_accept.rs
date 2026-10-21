//@ rules: ORIGIN-5
// An impl method may be more general than the trait's (an unrelated `b` in place of a
// shared `'a`), or state the trait's bound on early-bound parameters.
pub trait Pick {
    fn pick<'a>(&self, a: &'a str, b: &'a str) -> &'a str;
}
pub struct First;
impl Pick for First {
    fn pick<'a>(&self, a: &'a str, _b: &str) -> &'a str {
        a
    }
}

pub trait Narrow {
    fn narrow<'a, 'b>(x: &'a u8, y: &'b u8) -> &'b u8
    where
        'a: 'b;
}
pub struct Keep;
impl Narrow for Keep {
    fn narrow<'a: 'b, 'b>(x: &'a u8, _y: &'b u8) -> &'b u8 {
        x
    }
}
