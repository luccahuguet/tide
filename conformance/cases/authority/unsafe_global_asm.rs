//@ rules: UNSAFE-1, ADMIT-1
// `global_asm!` and `naked_asm!` place machine code without an `unsafe` block; without the
// grant neither is admitted, nor is `#[unsafe(naked)]`, nor `asm!`, whose `unsafe` block also
// needs the grant.
core::arch::global_asm!(""); //~ FX0520

pub fn barrier() {
    // SAFETY: an empty assembly block reads and writes nothing.
    unsafe { core::arch::asm!("") } //~ FX0901 FX0520
}

// SAFETY: `ret` returns at once and touches no callee-saved register.
#[unsafe(naked)] //~ FX0901
pub extern "C" fn noop() {
    core::arch::naked_asm!("ret"); //~ FX0520
}
