//@ rules: ADMIT-1
// `core::sync::atomic` is outside the admission allowlist; Tide's atomics take a `Tasks`.
pub fn counter() {
    let _a = core::sync::atomic::AtomicU64::new(0); //~ FX0520
}
