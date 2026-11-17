//@ rules: PURE-1
// A recursive type whose recursion changes its type arguments never repeats, so no fixed point
// stops the walk: the checker gives up past 64 nested types and rejects the type, although
// these hold only `u8`s (02 §9). A capability among the components is found first.
use std::marker::PhantomData;
use std::time::Clock;

pub enum Nested<T> {
    Leaf(T),
    Deeper(Box<Nested<Vec<T>>>),
}

pub struct Paired<T> {
    tag: PhantomData<T>,
    next: Option<Box<Paired<(T, T)>>>,
}

fn needs_pure<T: Pure>(_t: &T) {}

pub fn bytes(n: &Nested<u8>) {
    needs_pure(n) //~ TD0517
}

pub fn pairs(p: &Paired<u8>) {
    needs_pure(p) //~ TD0517
}

pub fn clocks(n: &Nested<Clock>) {
    needs_pure(n) //~ TD0517
}
