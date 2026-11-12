//@ rules: REF-0604
// Shared limits locked access to a closure and exposes no guard.
use std::sync::Shared;
use std::task::Tasks;
pub fn update(tasks: &Tasks) -> u8 {
    let value = Shared::new(2, tasks);
    value.with(|v| { *v += 1; *v }).unwrap()
}
struct Local;
impl Local { fn lock(&self) -> u8 { 1 } }
pub fn local_lock() -> u8 { Local.lock() }
