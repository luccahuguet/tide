//@ rules: ADMIT-1
// An already-boxed error reaches the reflexive `From<T> for T`, not one of alloc's four impls.
use std::io;

pub fn rewrap(e: Box<dyn Error + Send + Sync>) -> io::Error {
    io::Error::other(e) //~ FX0517
}
