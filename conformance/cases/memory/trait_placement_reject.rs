//@ rules: REF-0401
// A descendant is not the defining module; aliases do not move ownership.
pub struct Local;
mod child {
    impl super::Local {} //~ FX0401
}
mod owner { pub trait Shape {} pub struct Stone; }
mod elsewhere {
    type Alias = super::owner::Stone;
    impl Alias {} //~ FX0401
    impl super::owner::Shape for super::Local {} //~ FX0401
}
impl owner::Shape for u8 {} //~ FX0401

// A foreign trait and a non-local self type leave no legal defining module.
impl From<Local> for u16 { fn from(_: Local) -> u16 { 0 } } //~ FX0401
