//@ rules: UNSAFE-1, ADMIT-1
// `global_asm!` places machine code without an `unsafe` block; without the grant it is not
// admitted.
core::arch::global_asm!(""); //~ FX0520
