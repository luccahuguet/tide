//@ rules: REF-0102
// The nine Rust derives and Tide's three toolchain derives are the closed set.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug, Default, Encode, Decode)]
pub struct Wire { pub value: u8 }

#[derive(Debug, Error)]
#[error("a failure")]
pub struct Failure;
