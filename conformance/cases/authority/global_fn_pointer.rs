//@ rules: GLOBAL-4, ADMIT-1
// A function pointer's code address is observable only under the `unsafe` grant: through casts
// to integers, ordering, hashing, and formatting, a derived `Debug` included. `==` and
// `std::ptr::eq` compare identity without exposing it.
use std::hash::{Hash, Hasher};

pub fn noop() {}

pub fn int_addr(f: fn()) -> usize {
    f as usize //~ FX0520
}

pub fn item_addr() -> usize {
    noop as usize //~ FX0520
}

pub fn order(f: fn(), g: fn()) -> bool {
    f < g //~ FX0520
}

pub fn hash<H: Hasher>(f: fn(), h: &mut H) {
    f.hash(h) //~ FX0520
}

pub fn show(f: fn()) -> String {
    format!("{f:?}") //~ FX0520
}

#[derive(Debug)] //~ FX0520
pub struct Callback(pub fn());

pub fn same(f: fn(), g: fn()) -> bool {
    f == g
}

pub fn same_ref(a: &u8, b: &u8) -> bool {
    std::ptr::eq(a, b)
}

// The impls that a toolchain expansion selects for the user's types are checked too: the test
// harness formats the `Err`.
#[test] //~ FX0520
fn failing() -> Result<(), fn()> {
    Err(noop)
}
