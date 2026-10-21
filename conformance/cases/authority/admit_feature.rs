//@ rules: ADMIT-2
// The driver passes rustc `-Zallow-features=`, so no crate can enable an unstable feature.
#![feature(core_intrinsics)] //~ E0725

pub fn noop() {}
