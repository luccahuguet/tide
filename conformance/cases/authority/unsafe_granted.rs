//@ rules: UNSAFE-1
//@ grants: unsafe
// Under the `unsafe` grant, with a `// SAFETY:` comment on each `unsafe { .. }` block
// (`unsafe impl` needs none), `asm!`, `global_asm!`, a `#[unsafe(naked)]` function's
// `naked_asm!`, which needs no `unsafe` block, and `unsafe extern` blocks, an empty one too,
// and their items, `safe` ones included, are admitted.
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

unsafe extern "C" {
    pub safe fn abs(x: i32) -> i32;
}

pub fn magnitude(x: i32) -> i32 {
    abs(x)
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

#[link(name = "m")]
unsafe extern "C" {}

// SAFETY: `ret` returns at once and touches no callee-saved register.
#[unsafe(naked)]
pub extern "C" fn noop() {
    core::arch::naked_asm!("ret");
}
