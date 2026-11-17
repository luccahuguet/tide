//@ rules: ADMIT-1
// A user `Into` impl into `Box<dyn Error + Send + Sync>` is not one of alloc's four `From` impls.
use std::io;

pub struct Key(pub u32);

impl Into<Box<dyn Error + Send + Sync>> for Key {
    fn into(self) -> Box<dyn Error + Send + Sync> {
        format!("key {}", self.0).into()
    }
}

pub fn keyed(k: Key) -> io::Error {
    io::Error::other(k) //~ TD0517
}
