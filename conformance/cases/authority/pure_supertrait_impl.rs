//@ rules: PURE-1
// An impl of a trait with a `Pure` supertrait, for a type that holds a handle.
use std::sync::Shared;

pub trait Job: Pure {
    fn go(&self);
}

pub struct Counter {
    hits: Shared<u64>,
}

impl Job for Counter { //~ TD0517
    fn go(&self) {}
}
