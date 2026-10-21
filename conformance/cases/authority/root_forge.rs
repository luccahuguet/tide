//@ rules: ROOT-4
// A capability cannot be built from data: a struct literal cannot reach `Fs`'s private field.
use std::fs::Fs;

pub fn forge() -> Fs {
    Fs(()) //~ E0423
}
