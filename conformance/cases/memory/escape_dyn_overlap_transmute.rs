//@ rules: REF-0402
// Red team (fx-qlek.5): rust-lang/rust#57893. An impl for a type parameter that may be unsized
// also covers `dyn Object<U, Output = T>`, whose built-in impl says `Output = T` while the
// blanket impl says `Output = U`; rustc accepts both, so safe code gets a `transmute` that
// could make a capability out of any value of its size. The impl overlaps the built-in one and
// is rejected; a sized blanket impl does not cover a `dyn` type and stays admitted.
pub trait Object<U> {
    type Output;
}

impl<T: ?Sized, U> Object<U> for T { //~ TD0402
    type Output = U;
}

fn convert<T: ?Sized, U>(x: <T as Object<U>>::Output) -> U {
    x
}

pub fn transmute<T, U>(x: T) -> U {
    convert::<dyn Object<U, Output = T>, U>(x)
}

pub fn forge(fs: &u64) -> &std::fs::Fs {
    transmute(fs)
}

pub trait Sized2<U> {
    type Output;
}

impl<T, U> Sized2<U> for T {
    type Output = U;
}

// The same overlap with a projection as the self type (Opus r2, B1): `<P as Tr>::Assoc` may be
// unsized and normalizes to `dyn Object<..>`, which E0371 does not see. The driver at 6a30b7c
// admitted this program; rustc accepts it.
pub mod projection {
    use std::marker::PhantomData;

    pub trait Tr {
        type Assoc: ?Sized;
        type Out;
    }

    pub trait Object<P: ?Sized> {
        type Output;
    }

    impl<P: Tr + ?Sized> Object<P> for <P as Tr>::Assoc { //~ TD0402
        type Output = <P as Tr>::Out;
    }

    fn convert<P: Tr + ?Sized>(x: <<P as Tr>::Assoc as Object<P>>::Output) -> <P as Tr>::Out {
        x
    }

    pub struct Key<T, U>(PhantomData<(T, U)>);

    impl<T, U> Tr for Key<T, U> {
        type Assoc = dyn Object<Key<T, U>, Output = T>;
        type Out = U;
    }

    pub fn transmute<T, U>(x: T) -> U {
        convert::<Key<T, U>>(x)
    }

    pub fn forge(fs: &u64) -> &std::fs::Fs {
        transmute(fs)
    }
}
