//@ rules: LIFE-3
// Red team (fx-qlek.5): rust-lang/rust#100051, implied bounds read from an unnormalized
// projection in a signature: `<&'a &'b () as Trait>::Type` normalizes to `()`, so a caller
// might never prove the `'b: 'a` the body assumes. The pinned rustc requires it at the call.
pub trait Trait {
    type Type;
}

impl<T> Trait for T {
    type Type = ();
}

fn shorten<'a, 'b>(_: <&'a &'b () as Trait>::Type, s: &'b str) -> &'a str {
    s
}

pub fn extend<'a, 'b>(s: &'b str) -> &'a str {
    shorten((), s) //~ error
}
