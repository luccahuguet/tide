//@ rules: OPAQUE-2
// No auto trait leaks through an opaque type: rustc lets `Send` and `Sync` leak
// from the hidden type, the checker does not; and a stated `Pure` must hold of
// the hidden type.
use std::fs::Fs;

fn need_send<T: Send>(_: &T) {}
fn need_sync<T: Sync>(_: &T) {}
fn need_pure<T: Pure>(_: &T) {}

pub fn make(n: u32) -> impl Fn() -> u32 { move || n }
pub fn uses() {
    let f = make(1);
    need_send(&f); //~ FX0205
    need_sync(&f); //~ FX0205
    need_pure(&f); //~ FX0517
}
pub fn reader(fs: Fs) -> impl Fn() -> u32 + Pure { move || { let _ = &fs; 0 } } //~ FX0517
