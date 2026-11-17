//@ rules: CLOSURE-3
// The rust#25860 coercion over a capability reference: a borrowed `Fs` comes back as
// `&'static Fs`, outliving the grant it was lent under. rustc accepts it.
use std::fs::Fs;

static UNIT: &'static &'static () = &&();

fn foo<'a, 'b, T>(_: &'a &'b (), v: &'b T, _: &()) -> &'a T { v }

pub fn keep<'a>(fs: &'a Fs) -> &'static Fs {
    let f: fn(_, &'a Fs, &()) -> &'static Fs = foo; //~ TD0206
    f(UNIT, fs, &())
}
