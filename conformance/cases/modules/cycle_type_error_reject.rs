//@ rules: REF-0914
mod left { pub struct L; pub use super::right::R; } //~ TD0913 TD0914
mod right { pub struct R; pub use super::left::L; } //~ TD0913 TD0914
pub fn f() -> u8 { "x" } //~ E0308
