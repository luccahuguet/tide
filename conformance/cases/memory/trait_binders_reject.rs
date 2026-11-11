//@ rules: LIFE-3
// Explicit binders in bare pointers, dyn bounds and where subjects share the rule.
pub type Callback = for<'a> fn(&'a u8) -> &'a u8; //~ FX0404
pub type Job = Box<dyn for<'a> Fn(&'a u8) -> &'a u8>; //~ FX0404
pub fn predicate<F>(f: F) where for<'a> F: Fn(&'a u8) { f(&0); } //~ FX0404
