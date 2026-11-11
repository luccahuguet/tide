//@ rules: REF-0210
//@ grants: unsafe
// A raw pointer is not a borrowed field, even when formed from the moved owner.
pub struct Raw<'a> { pub owner: Box<u32>, pub ptr: *const u32, pub other: &'a str }
pub fn make<'a>() -> Raw<'a> { let owner = Box::new(1); let text = String::from("x"); Raw { ptr: &*owner, other: &text, owner } } //~ E0515
