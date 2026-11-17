//@ rules: ADMIT-1
// Generic code that wraps the constructors needs a `Pure` bound, checked at the definition.
use std::io;

pub fn wrap<E: Error + Send + Sync + 'static>(e: E) -> io::Error {
    io::Error::other(e) //~ TD0517
}
