//@ rules: LIFE-3
// Red team (fx-qlek.5): rust-lang/rust#84591, a higher-ranked subtrait bound that implied its
// supertrait with weaker implied bounds. Tide rejects the explicit binders, and the pinned
// rustc no longer accepts the extension either.
pub trait Supertrait<'a, 'b> { //~ TD0201
    fn convert<T: ?Sized>(x: &'a T) -> &'b T;
}
pub trait Subtrait<'a, 'b, R>: Supertrait<'a, 'b> {} //~ TD0201

fn need_super<S, T: ?Sized>(x: &T) -> &T where S: for<'a, 'b> Supertrait<'a, 'b> { //~ TD0404
    S::convert(x)
}
fn need_sub<S, T: ?Sized>(x: &T) -> &T where S: for<'a, 'b> Subtrait<'a, 'b, &'b &'a ()> { //~ TD0404
    need_super::<S, T>(x)
}

pub struct MyStruct;
impl<'a: 'b, 'b> Supertrait<'a, 'b> for MyStruct {
    fn convert<T: ?Sized>(x: &'a T) -> &'b T { x }
}
impl<'a, 'b> Subtrait<'a, 'b, &'b &'a ()> for MyStruct {}

pub fn extend<'a, 'b, T: ?Sized>(x: &'a T) -> &'b T {
    need_sub::<MyStruct, T>(x) //~ error
}
