//@ rules: CLOSURE-3
// The rust#25860 route through a type parameter: `&'a T` implies `T: 'a`, which `foo` uses to
// box `v` as `dyn Debug + 'a`. At `T = &'x u32` the coercion drops `'x: 'static`, so `bad`
// returns a borrow as `'static`. rustc accepts it.
use std::fmt::Debug;

static S: &'static &'static u32 = &&0;

fn foo<'a, 'c, T: Debug>(_: &'a T, v: T, _: &'c ()) -> Box<dyn Debug + 'a> { Box::new(v) }

pub fn bad<'x>(x: &'x u32) -> Box<dyn Debug + 'static> {
    let f: fn(_, &'x u32, &()) -> Box<dyn Debug + 'static> = foo; //~ FX0206
    f(S, x, &())
}

// The same route through an associated type: `&'a T::Out` implies `T::Out: 'a`.
pub trait Item {
    type Out: Debug;
}

pub struct Hold<'x>(pub &'x u32);

impl<'x> Item for Hold<'x> {
    type Out = &'x u32;
}

fn bar<'a, 'c, T: Item>(_: &'a T::Out, v: T::Out, _: &'c ()) -> Box<dyn Debug + 'a> { Box::new(v) }

pub fn alias<'x>(x: &'x u32) -> Box<dyn Debug + 'static> {
    let f: fn(_, &'x u32, &()) -> Box<dyn Debug + 'static> = bar::<Hold<'x>>; //~ FX0206
    f(S, x, &())
}
