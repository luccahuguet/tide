//@ rules: CLOSURE-3
// Changed by Tide from rust-lang/rust (MIT OR Apache-2.0; NOTICE), whose suite keeps rust#25860 open
// in tests/ui/implied-bounds/implied-bounds-on-nested-references-plus-variance-2.rs at 5f4effd5:
// Tide made `bad` public, dropped `main` and the directives, and added its marks. rustc's #129021
// fix rejects the variant without the third argument; only the checker rejects this one.
static UNIT: &'static &'static () = &&();

fn foo<'a, 'b, T>(_: &'a &'b (), v: &'b T, _: &()) -> &'a T { v }

pub fn bad<'a, T>(x: &'a T) -> &'static T {
    let f: fn(_, &'a T, &()) -> &'static T = foo; //~ TD0206
    f(UNIT, x, &())
}
