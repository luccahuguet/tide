//@ rules: UNSAFE-1, GLOBAL-4
// `#[trusted_pure]` needs the `unsafe` grant, as does the raw pointer type it vouches for.
#[trusted_pure(reason = "the pointer is compared, never read")] //~ TD0901
pub struct Tag {
    p: *const u8, //~ TD0520
}
