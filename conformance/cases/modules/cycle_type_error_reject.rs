//@ rules: REF-0914
mod left { pub struct L; pub use super::right::R; } //~ FX0913 FX0914
mod right { pub struct R; pub use super::left::L; } //~ FX0913 FX0914
pub fn f() -> u8 { "x" } //~ E0308
