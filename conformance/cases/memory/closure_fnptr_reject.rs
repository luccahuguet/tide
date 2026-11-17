//@ rules: CLOSURE-3
// rust#25860: `pick` implies `'b: 'a` from its argument `&'a &'b ()`. Coerced to a
// `fn` pointer whose first argument is higher-ranked, the implied bound is lost, so
// `extend` hands back any borrow as `&'b T` for a caller-chosen `'b`. rustc accepts
// it; the checker rejects the coercion, since `pick` implies `'b: 'a` (CLOSURE-3).
static UNIT: &'static &'static () = &&();

fn pick<'a, 'b, T>(_: &'a &'b (), v: &'b T) -> &'a T { v }

pub fn extend<'b, T>(x: &T) -> &'b T {
    let f: fn(_, &T) -> &'b T = pick; //~ TD0206
    f(UNIT, x)
}
