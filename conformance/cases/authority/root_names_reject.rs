//@ rules: ROOT-1
// Naming Root in imports, aliases, fields and signatures is forbidden.
use std::Root as Entry; //~ TD0503
pub type Alias = Entry; //~ TD0503
pub struct Holder { pub root: Root } //~ TD0503
pub fn borrow(_: &Root) {} //~ TD0503
pub fn return_root(root: Root) -> Root { root } //~ TD0503
