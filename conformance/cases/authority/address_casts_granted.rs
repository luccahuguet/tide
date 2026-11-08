//@ rules: GLOBAL-4
//@ grants: unsafe
// Under the `unsafe` grant an address counts as a `usize` (D3): a cast between an address and
// an integer that loses no bits of either is accepted, and so is a cast between pointer types,
// which keeps the address even when it drops a slice's length.
pub fn to_int(p: *const u8, f: fn()) -> (usize, u64, u128, i128, usize) {
    (p as usize, p as u64, p as u128, p as i128, f as usize)
}

pub fn to_ptr(n: usize, m: u32, b: u8) -> (*const u8, *mut u8, *const u16) {
    (n as *const u8, m as *mut u8, b as *const u16)
}

pub fn between(x: &u8, s: *const [u8], m: *mut u8) -> (*const u8, *const u8, *const u32) {
    (x as *const u8, s as *const u8, m as *const u32)
}
