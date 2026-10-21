//@ rules: PURE-1
// `Pure` is deep: a `Shared` behind a tuple, a `Vec`, and a `Box` still fails, here at a
// coercion to a `Pure` trait object.
use std::sync::Shared;

pub trait Value {
    fn get(&self) -> u32;
}
pub trait PureValue: Value + Pure {}
impl<T: Value + Pure + ?Sized> PureValue for T {}

pub struct Deep {
    inner: (u8, Vec<Box<Shared<u8>>>),
}

impl Value for Deep {
    fn get(&self) -> u32 {
        self.inner.0 as u32
    }
}

pub fn erase(d: &Deep) -> &dyn PureValue {
    d //~ FX0517
}
