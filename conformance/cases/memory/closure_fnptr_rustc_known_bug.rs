//@ rules: CLOSURE-3
// rust#25860 as rustc's own suite keeps it open (`known-bug: #25860`, check-pass):
// rust-lang/rust tests/ui/implied-bounds/implied-bounds-on-nested-references-plus-variance-2.rs
// at 5f4effd579500175af844241eb179e756190f1e7. rustc's fix for #129021 rejects the variant
// without the third argument; this one still passes rustc, so only the checker rejects it.
static UNIT: &'static &'static () = &&();

fn foo<'a, 'b, T>(_: &'a &'b (), v: &'b T, _: &()) -> &'a T { v }

pub fn bad<'a, T>(x: &'a T) -> &'static T {
    let f: fn(_, &'a T, &()) -> &'static T = foo; //~ TD0206
    f(UNIT, x, &())
}
