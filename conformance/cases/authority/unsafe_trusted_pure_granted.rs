//@ rules: UNSAFE-1, PURE-1
//@ grants: unsafe
// Under the grant, `#[trusted_pure]` makes `Pure` a type the structural rule rejects.
#[trusted_pure(reason = "the pointer is compared, never read")]
pub struct Tag {
    p: *const u8,
}

fn needs_pure<T: Pure>(_t: &T) {}

pub fn check(t: &Tag) {
    needs_pure(t)
}
