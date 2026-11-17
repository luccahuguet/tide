//@ rules: CLOSURE-3
// Red team (fx-qlek.5): routes to rust-lang/rust#25860 that do not coerce a fn item to a `fn`
// pointer, so `FX0206` does not see them: the item passed as an `F: Fn` argument, unsized to
// `dyn Fn`, returned as `impl Fn`, or wrapped in a closure coerced to a `fn` pointer. rustc checks
// the item's implied bounds on each of these routes (for `impl Fn`, at the call), so all are
// rejected.
fn translate<'a, 'b, T>(_: &'a &'b (), v: &'b T) -> &'a T {
    v
}

fn through<'a, 'b, T, F: Fn(&'a &'b (), &'b T) -> &'a T>(f: F, v: &'b T) -> &'a T { //~ E0491
    f(&&(), v) //~ error
}

pub fn by_bound<'a, 'b, T>(v: &'b T) -> &'a T {
    through(translate, v)
}

pub fn by_dyn<'a, 'b, T>(v: &'b T) -> &'a T {
    let f: &dyn Fn(&'a &'b (), &'b T) -> &'a T = &translate; //~ error
    f(&&(), v)
}

fn pick<'a, 'b, T>() -> impl Fn(&'a &'b (), &'b T) -> &'a T {
    translate
}

pub fn by_impl<'a, 'b, T>(v: &'b T) -> &'a T {
    pick::<'a, 'b, T>()(&&(), v) //~ error
}

pub fn by_closure<'a, 'b, T>(v: &'b T) -> &'a T {
    let f: fn(&'a &'b (), &'b T) -> &'a T = |x, y| translate(x, y); //~ error
    f(&&(), v)
}
