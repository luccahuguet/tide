//@ rules: CLOSURE-3
// rust#25860: `pick` implies `'b: 'a` from its argument `&'a &'b ()`. Coerced to a
// `fn` pointer whose first argument is higher-ranked, the implied bound is lost, so
// `extend` hands back any borrow as `&'b T` for a caller-chosen `'b`. rustc accepts
// it; the checker re-checks `pick`'s implied bounds at the coercion.
static UNIT: &'static &'static () = &&();

fn pick<'a, 'b, T>(_: &'a &'b (), v: &'b T) -> &'a T { v }

pub fn extend<'b, T>(x: &T) -> &'b T {
    let f: fn(_, &T) -> &'b T = pick; //~ FX0206
    f(UNIT, x)
}
