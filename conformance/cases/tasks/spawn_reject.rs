//@ rules: REF-0602
// Rust threads bypass Tasks; missing imports retain their source identities.
use std::thread::spawn as launch; //~ E0432 TD0602
pub fn direct() { let _ = std::thread::spawn(|| 1); } //~ E0433 TD0602
pub fn renamed() { let _ = launch(|| 2); } //~ TD0602
pub fn builder() { let _ = std::thread::Builder::new(); } //~ E0433 TD0602
pub fn scoped() { std::thread::scope(|_| ()); } //~ E0433 TD0602
pub fn runtime() { let _ = tokio::spawn(|| 3); } //~ E0433 TD0602

mod aliased {
    use tokio as runtime; //~ E0432
    pub fn run() { let _ = runtime::spawn(|| 4); } //~ TD0602
}
mod exports {
    pub use tokio as runtime; //~ E0432
}
pub fn reexported() { let _ = exports::runtime::spawn(|| 5); } //~ TD0602
use exports::runtime as relay;
pub fn reimported() { let _ = relay::spawn(|| 6); } //~ TD0602
mod globbed { pub use super::exports::*; }
pub fn glob_reexported() { let _ = globbed::runtime::spawn(|| 7); } //~ TD0602
mod named_export {
    pub use tokio; //~ E0432
}
pub fn named_reexported() { let _ = named_export::tokio::spawn(|| 8); } //~ TD0602
