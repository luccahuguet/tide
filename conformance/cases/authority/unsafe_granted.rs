//@ rules: UNSAFE-1
//@ grants: unsafe
// Under the `unsafe` grant, with a `// SAFETY:` comment on each block and `unsafe impl`;
// `asm!` and `global_asm!` are admitted.
pub unsafe fn raw_len(p: &[u8]) -> usize {
    p.len()
}

pub fn len(p: &[u8]) -> usize {
    // SAFETY: `raw_len` reads only the slice's length.
    unsafe { raw_len(p) }
}

#[unsafe(no_mangle)]
pub fn tide_probe() -> u32 {
    7
}

pub unsafe trait Zeroable {}

// SAFETY: every bit pattern is a valid `u32`.
unsafe impl Zeroable for u32 {}

pub struct Token(std::cell::Cell<u8>);

// SAFETY: no method reads or writes the `Cell`.
unsafe impl Sync for Token {}

pub fn barrier() {
    // SAFETY: an empty assembly block reads and writes nothing.
    unsafe { core::arch::asm!("") }
}

core::arch::global_asm!("");
