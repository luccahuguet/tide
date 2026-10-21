//@ rules: LIFE-2
// `use<..>` that leaves out a type parameter in scope, an argument-position `impl Trait`
// (an anonymous type parameter), or a lifetime its bounds name.
pub fn drops_param<T>(x: T) -> impl Sized + use<> { //~ error
    x
}

pub fn with_apit(_x: impl Sized) -> impl Sized + use<> {} //~ error

pub fn drops_lifetime<'a>(x: &'a u8) -> impl Sized + 'a + use<> { //~ error
    x
}
