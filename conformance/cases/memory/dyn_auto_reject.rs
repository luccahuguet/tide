//@ rules: DYN-2
// A `dyn` that does not name `Send` or `Sync` lacks them; `Pure` cannot be added with `+`, and a
// `dyn` without a `Pure` supertrait is not `Pure`.
use std::fs::Fs;

pub trait PureFn: Fn() -> u32 + Pure {}
impl<F: Fn() -> u32 + Pure> PureFn for F {}

fn need_send<T: Send + ?Sized>(_: &T) {}
fn need_sync<T: Sync + ?Sized>(_: &T) {}
fn need_pure<T: Pure + ?Sized>(_: &T) {}

pub fn unnamed(f: &dyn Fn() -> u32) {
    need_send(f); //~ E0277
    need_sync(f); //~ E0277
    need_pure(f); //~ TD0517
}
pub fn spelled(f: &(dyn Fn() -> u32 + Pure)) {} //~ E0225
pub fn coerce(fs: Fs) -> u32 { let g: &dyn PureFn = &move || { let _ = &fs; 0 }; g() } //~ TD0517
