//@ rules: LIFE-4, LIFE-6
// One input resolves the output in each position; a plain borrowing field names its lifetime.
pub fn pointer(_: fn(&str) -> &str) {}
pub fn sugar<F: Fn(&str) -> &str>(_: F) {}
pub type Callback = fn(&str) -> &str;
pub struct Field { pub f: fn(&str) -> &str }
pub struct Plain<'a> { pub text: &'a str }
