//@ rules: LIFE-4, LIFE-6
// Function-type and Fn-sugar output elision follows LIFE-4 wherever the type occurs.
pub fn pointer(_: fn(&str, &str) -> &str) {} //~ E0106 FX0202
pub fn sugar<F: Fn(&str, &str) -> &str>(_: F) {} //~ E0106 FX0202
pub type Callback = fn(&str, &str) -> &str; //~ E0106 FX0202
pub struct Field { pub f: fn(&str, &str) -> &str } //~ E0106 FX0202
pub struct Plain { pub text: &str } //~ E0106
