//@ rules: CLOSURE-2
// With no expected type, a parameter annotation that elides its lifetime makes
// that origin higher-ranked, so the closure cannot return a borrow of it,
// whether its return type is annotated or not.
pub fn first(s: &str) -> usize {
    let f = |t: &str| &t[..1]; //~ error
    f(s).len()
}
pub fn annotated(s: &str) -> usize {
    let f = |t: &str| -> &str { &t[..1] }; //~ error
    f(s).len()
}
