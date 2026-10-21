//@ rules: UNSAFE-1, PURE-1
//@ grants: unsafe
// Under the grant, `#[trusted_pure]` makes `Pure` a type the structural rule rejects. Its author
// vouches for its own fields, not for its users' type arguments: a generic one is `Pure` only
// when they are, as std's types are.
use std::time::Clock;

#[trusted_pure(reason = "the pointer is compared, never read")]
pub struct Tag {
    p: *const u8,
}

#[trusted_pure(reason = "the pointer is compared, never read")]
pub struct Tagged<T> {
    p: *const u8,
    value: T,
}

fn needs_pure<T: Pure>(_t: &T) {}

pub fn check(t: &Tag) {
    needs_pure(t)
}

pub fn tagged(t: &Tagged<u64>) {
    needs_pure(t)
}

pub fn tagged_clock(t: &Tagged<Clock>) {
    needs_pure(t) //~ FX0517
}
