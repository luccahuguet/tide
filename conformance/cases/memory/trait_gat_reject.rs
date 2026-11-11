//@ rules: LIFE-3
// Type, const, and lifetime parameters all make an associated type generic.
pub trait Source {
    type Typed<T>; //~ FX0403
    type Array<const N: usize>; //~ FX0403
    type Borrowed<'a>; //~ FX0403
}
pub struct Stone;
impl Source for Stone {
    type Typed<T> = Option<T>; //~ FX0403
    type Array<const N: usize> = [u8; N]; //~ FX0403
    type Borrowed<'a> = &'a u8; //~ FX0403
}
