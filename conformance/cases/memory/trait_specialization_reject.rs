//@ rules: REF-0402
// Specialization is rejected even before stable Rust could accept the feature.
pub trait Shape { fn sides(&self) -> u8; }
pub struct Stone;
impl Shape for Stone {
    default fn sides(&self) -> u8 { 4 } //~ E0658 TD0402
}
pub trait Empty {}
default impl<T> Empty for T {} //~ E0658 TD0402
pub trait Constants { const N: u8; type Item; }
impl Constants for u8 {
    default const N: u8 = 0; //~ E0658 TD0402
    default type Item = u8; //~ E0658 TD0402
}
