//@ rules: GLOBAL-2
// A static's initializer runs at compile time, so it calls only `const fn`.
pub static GREETING: String = String::from("hello"); //~ E0015
