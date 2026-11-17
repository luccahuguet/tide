//@ rules: ADMIT-1, REF-0912
// The allowlist is keyed on crate identity, not crate names: `core` renamed `std` is still
// `core`, and its atomics are still outside the list. `#![no_std]` itself is not admitted
// (`admit_no_std`).
#![no_std] //~ TD0520
extern crate core as std; //~ TD0912

pub fn counter() -> u64 {
    std::sync::atomic::AtomicU64::new(0).into_inner() //~ TD0520
}
