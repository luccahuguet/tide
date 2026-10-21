//@ rules: LIFE-3
// Beyond D4's power ceiling, though rustc accepts them: a lifetime parameter on a trait,
// an explicit `for<'a>`, and a generic associated type.
pub trait Parse<'a> { //~ FX0201
    fn parse(src: &'a str) -> Self;
}

pub fn apply<F>(f: F) -> u8
where
    F: for<'a> Fn(&'a u8) -> &'a u8, //~ FX0404
{
    *f(&0)
}

pub trait Lend {
    type Item<'a> //~ FX0403
    where
        Self: 'a;
}
