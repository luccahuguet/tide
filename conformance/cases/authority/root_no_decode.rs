//@ rules: ROOT-4
// No capability has a `Decode` impl, and a skipped field needs the `Default` it also lacks.
use std::net::Net;

#[derive(Decode)]
pub struct Holder {
    pub net: Net, //~ E0277
}

#[derive(Decode)]
pub struct Skipped {
    #[codec(skip)]
    pub net: Net, //~ E0277
}
