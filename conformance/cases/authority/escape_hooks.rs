//@ rules: ADMIT-1, UNSAFE-1
// Red team (fx-qlek.5): process-wide hooks. Tide's std exports no `std::panic` module, so a
// panic hook does not resolve; a global allocator needs `core::alloc::GlobalAlloc`,
// which is not admitted, and an `unsafe impl`, which needs the grant; the code the attribute
// generates uses raw pointers, which need it too.
pub fn install() {
    std::panic::set_hook(Box::new(|_| {})); //~ E0433
}

pub struct Counting;

// SAFETY: allocates nothing.
unsafe impl core::alloc::GlobalAlloc for Counting { //~ TD0520 TD0901
    unsafe fn alloc(&self, _: core::alloc::Layout) -> *mut u8 { //~ TD0520 TD0901
        core::ptr::null_mut() //~ TD0520
    }
    unsafe fn dealloc(&self, _: *mut u8, _: core::alloc::Layout) {} //~ TD0520 TD0901
}

#[core::prelude::v1::global_allocator] //~ TD0520
static ALLOCATOR: Counting = Counting;

// The attribute without its path (Opus r2, M3): Tide's prelude does not export it, so it does
// not resolve.
#[global_allocator] //~ error
static IMPLICIT: Counting = Counting;
