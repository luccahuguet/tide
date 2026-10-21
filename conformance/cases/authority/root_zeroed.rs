//@ rules: ROOT-4, UNSAFE-1
//@ grants: unsafe
// Under the grant, `mem::zeroed` still produces only a `Pure` type, so it cannot forge a
// capability.
use std::fs::Fs;

pub fn forge() -> Fs {
    // SAFETY: none; the `Pure` bound rejects the call.
    unsafe { std::mem::zeroed::<Fs>() } //~ FX0517
}

pub fn zero() -> u64 {
    // SAFETY: an all-zero `u64` is valid.
    unsafe { std::mem::zeroed::<u64>() }
}
