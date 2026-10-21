//@ rules: ADMIT-1
// I/O methods on re-exported std types, and `io::Error::last_os_error`, are not admitted.
use std::io;
use std::path::Path;

pub fn present(p: &Path) -> bool {
    p.exists() //~ FX0520
}

pub fn last() -> io::Error {
    io::Error::last_os_error() //~ FX0520
}
