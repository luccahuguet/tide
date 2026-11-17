//@ rules: LIFE-3
// Type, const, and lifetime parameters all make an associated type generic.
pub trait Source {
    type Typed<T>; //~ TD0403
    type Array<const N: usize>; //~ TD0403
    type Borrowed<'a>; //~ TD0403
}
pub struct Stone;
impl Source for Stone {
    type Typed<T> = Option<T>; //~ TD0403
    type Array<const N: usize> = [u8; N]; //~ TD0403
    type Borrowed<'a> = &'a u8; //~ TD0403
}
