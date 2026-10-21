//@ rules: LIFE-5
// Elided lifetimes in consts and statics are `'static`.
pub const NAMES: &[&str] = &["a", "b"];
pub static GREETING: &str = "hi";

pub fn first() -> &'static str {
    NAMES[0]
}

pub fn greeting() -> &'static str {
    GREETING
}
