//@ rules: ROOT-4
// No capability has a `Default` impl.
use std::fs::Fs;

pub fn blank() -> Fs {
    Fs::default() //~ E0599
}
