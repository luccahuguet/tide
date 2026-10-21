//@ rules: ORIGIN-2
// Without `'b: 'a`, `&'a &'b u8` is not well formed: in an associated type (E0491) and
// in a body annotation.
pub trait Holder {
    type Out;
}
pub struct Two<'a, 'b>(&'a u8, &'b u8);
impl<'a, 'b> Holder for Two<'a, 'b> {
    type Out = &'a &'b u8; //~ E0491
}

pub fn annotate<'a, 'b>(_x: &'b u8) {
    let _r: Option<&'a &'b u8> = None; //~ error
}
