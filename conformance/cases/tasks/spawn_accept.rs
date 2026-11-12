//@ rules: REF-0602
// Children belong to a Tasks scope; same-spelled user functions are ordinary functions.
use std::task::{TaskError, Tasks};
pub fn scoped(tasks: &Tasks) -> Result<u8, TaskError> {
    tasks.scope(|scope| scope.spawn(|| Ok(7)).join())
}
pub mod tokio { pub fn spawn(f: impl FnOnce() -> u8) -> u8 { f() } }
pub fn local() -> u8 { tokio::spawn(|| 3) }
use tokio as runtime;
mod exports { pub use crate::tokio as runtime; }
use exports::runtime as relay;
pub fn local_alias() -> u8 { runtime::spawn(|| 4) }
pub fn local_reexport() -> u8 { exports::runtime::spawn(|| 5) }
pub fn local_reimport() -> u8 { relay::spawn(|| 6) }
mod globbed { pub use super::exports::*; }
pub fn local_glob_reexport() -> u8 { globbed::runtime::spawn(|| 7) }
mod named_export { pub use crate::tokio; }
pub fn local_named_reexport() -> u8 { named_export::tokio::spawn(|| 8) }
