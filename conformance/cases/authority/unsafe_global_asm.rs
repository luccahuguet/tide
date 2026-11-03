//@ rules: UNSAFE-1, ADMIT-1
// `global_asm!` places machine code without an `unsafe` block; without the grant it is not
// admitted, and neither is `asm!`, whose `unsafe` block also needs the grant.
core::arch::global_asm!(""); //~ FX0520

pub fn barrier() {
    // SAFETY: an empty assembly block reads and writes nothing.
    unsafe { core::arch::asm!("") } //~ FX0901 FX0520
}
