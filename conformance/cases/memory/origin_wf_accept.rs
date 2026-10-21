//@ rules: ORIGIN-2
// `&'a &'b u8` is well formed only when `'b: 'a`: a parameter type implies it, and an
// impl may declare it.
pub fn inner<'a, 'b>(x: &'a &'b u8) -> &'a u8 {
    *x
}

pub trait Holder {
    type Out;
}
pub struct Two<'a, 'b>(&'a u8, &'b u8);
impl<'a, 'b: 'a> Holder for Two<'a, 'b> {
    type Out = &'a &'b u8;
}
