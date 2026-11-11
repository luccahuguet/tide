//@ rules: REF-0402, PURE-2
// An ordinary default trait method and a single impl are legal.
pub trait Shape { fn sides(&self) -> u8 { 4 } }
pub struct Stone;
impl Shape for Stone {}
