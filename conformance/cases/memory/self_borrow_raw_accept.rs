//@ rules: REF-0210
//@ grants: unsafe
pub struct Raw<'a> { pub owner: Box<u32>, pub ptr: *const u32, pub other: &'a str }
pub fn make(text: &str) -> Raw<'_> { let owner = Box::new(1); Raw { ptr: &*owner, other: text, owner } }
