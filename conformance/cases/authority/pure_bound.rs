//@ rules: PURE-1
// A struct holding a capability fails a user-written `Pure` bound.
use std::fs::Fs;

pub struct Holder {
    fs: Fs,
}

fn needs_pure<T: Pure>(_t: &T) {}

pub fn check(h: &Holder) {
    needs_pure(h); //~ TD0517
}
