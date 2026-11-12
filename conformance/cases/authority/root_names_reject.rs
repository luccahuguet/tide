//@ rules: ROOT-1
// Naming Root in imports, aliases, fields and signatures is forbidden.
use std::Root as Entry; //~ FX0503
pub type Alias = Entry; //~ FX0503
pub struct Holder { pub root: Root } //~ FX0503
pub fn borrow(_: &Root) {} //~ FX0503
pub fn return_root(root: Root) -> Root { root } //~ FX0503
