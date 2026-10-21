//@ rules: DYN-2
// A `dyn` is `Send` or `Sync` when it names them, and `Pure` through a companion supertrait.
pub trait PureFn: Fn() -> u32 + Pure {}
impl<F: Fn() -> u32 + Pure> PureFn for F {}

fn need_send<T: Send + ?Sized>(_: &T) {}
fn need_sync<T: Sync + ?Sized>(_: &T) {}
fn need_pure<T: Pure + ?Sized>(_: &T) {}

pub fn named(f: &(dyn Fn() -> u32 + Send + Sync), g: &dyn PureFn) {
    need_send(f);
    need_sync(f);
    need_pure(g);
}
pub fn coerce(n: u32) -> u32 { let g: &dyn PureFn = &move || n; g() }
