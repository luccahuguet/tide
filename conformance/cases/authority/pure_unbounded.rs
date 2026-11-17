//@ rules: PURE-1
// An unbounded type parameter is not `Pure`, so the coercion fails at the definition.
pub trait Value {
    fn get(&self) -> u32;
}
pub trait PureValue: Value + Pure {}
impl<T: Value + Pure + ?Sized> PureValue for T {}

pub fn erase<U: Value>(u: &U) -> &dyn PureValue {
    u //~ TD0517
}
