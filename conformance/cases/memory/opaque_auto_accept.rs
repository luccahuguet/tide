//@ rules: OPAQUE-2
// An opaque type whose bounds state `Send`, `Sync`, and `Pure` has them.
fn need_send<T: Send>(_: &T) {}
fn need_sync<T: Sync>(_: &T) {}
fn need_pure<T: Pure>(_: &T) {}

pub fn make(n: u32) -> impl Fn() -> u32 + Send + Sync + Pure { move || n }
pub fn uses() {
    let f = make(1);
    need_send(&f);
    need_sync(&f);
    need_pure(&f);
}
