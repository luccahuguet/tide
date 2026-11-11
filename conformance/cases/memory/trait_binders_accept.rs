//@ rules: LIFE-3
// Elision introduces higher-ranked lifetimes without an explicit binder.
pub type Callback = fn(&u8) -> &u8;
pub type Job = Box<dyn Fn(&u8) -> &u8>;
pub fn predicate<F>(f: F) where F: Fn(&u8) { f(&0); }
