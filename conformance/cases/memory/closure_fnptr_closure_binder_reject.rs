//@ rules: CLOSURE-3, LIFE-3
// The rust#25860 hole through a closure coerced to a `fn` pointer: the closure takes its
// signature from an expected type whose two parameters share the higher-ranked origin `'b`, so
// its body may assume `'b: 'a`, and a later `fn` pointer subtyping drops that bound. Elision
// cannot write that type; it needs an explicit binder, and TD0404 keeps this route closed.
static UNIT: &'static &'static () = &&();

pub fn extend<'x, T>(x: &'x T) -> &'static T {
    let f: for<'a, 'b> fn(&'a &'b (), &'b T) -> &'a T = |_, v| v; //~ TD0404
    let g: fn(&'static &'static (), &'x T) -> &'static T = f;
    g(UNIT, x)
}
