//@ rules: GLOBAL-4, ADMIT-1
// Without the `unsafe` grant, raw pointer types and address observers are not admitted.
pub fn address(x: &u8) -> usize {
    (x as *const u8).addr() //~ FX0520
}

pub fn rebuild(n: usize) {
    let _ = n as *const u8; //~ FX0520
}
