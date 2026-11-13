//@ rules: REF-0914
mod leaf { pub struct T; }
mod facade { pub use super::leaf::T; }
// Both edges point at leaf, which defines T, rather than at the facade.
mod consumer { use super::facade::T; pub fn make() -> T { T } }
pub struct Shared;
mod child { use super::*; pub fn make() -> Shared { Shared } }
// Body paths and mod declarations do not contribute edges.
mod a { pub fn f() { super::b::f(); } }
mod b { pub fn f() { super::a::f(); } }
