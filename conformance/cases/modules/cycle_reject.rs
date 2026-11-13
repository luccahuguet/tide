//@ rules: REF-0914
pub mod a {
    pub struct A;
    pub use super::b::B; //~ FX0914
}
pub mod b {
    pub struct B;
    pub use super::a::A; //~ FX0914
}
// A re-export and a glob import also form a cycle.
pub use child::T; //~ FX0914
pub mod child {
    pub struct T;
    pub use super::*; //~ FX0914
}
