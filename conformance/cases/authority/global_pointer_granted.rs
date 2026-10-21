//@ rules: GLOBAL-4
//@ grants: unsafe
// Under the `unsafe` grant, raw pointer types, `addr`, and a function pointer's address are
// admitted.
pub fn address(x: &u8) -> usize {
    (x as *const u8).addr()
}

pub fn code(f: fn()) -> usize {
    f as usize
}
