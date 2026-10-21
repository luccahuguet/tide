//@ rules: ORIGIN-5
// An impl method less general than the trait's: tying `b` to `'a`, or adding a bound.
pub trait Pick {
    fn pick<'a>(&self, a: &'a str, b: &str) -> &'a str;
}
pub struct Second;
impl Pick for Second {
    fn pick<'a>(&self, _a: &'a str, b: &'a str) -> &'a str { //~ error
        b
    }
}

pub trait Look {
    fn look(&self, x: &u8);
}
pub struct Static;
impl Look for Static {
    fn look<'a>(&self, _x: &'a u8) where 'a: 'static {} //~ E0195
}
