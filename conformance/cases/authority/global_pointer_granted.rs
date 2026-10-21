//@ rules: GLOBAL-4
//@ grants: unsafe
// Under the `unsafe` grant, raw pointer types and `addr` are admitted.
pub fn address(x: &u8) -> usize {
    (x as *const u8).addr()
}
