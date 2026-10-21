//@ rules: ADMIT-1
// `io::Error::new` and `other` from a `Pure` error or a string, and a generic wrapper whose
// parameter is bounded by `Pure`.
use std::fmt;
use std::io;

#[derive(Debug)]
pub struct Corrupt {
    pub offset: u64,
}

impl fmt::Display for Corrupt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "corrupt at {}", self.offset)
    }
}

impl Error for Corrupt {}

pub fn corrupt(offset: u64) -> io::Error {
    io::Error::other(Corrupt { offset })
}

pub fn bad_header() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "bad header")
}

pub fn wrap<E: Error + Send + Sync + Pure + 'static>(e: E) -> io::Error {
    io::Error::other(e)
}
