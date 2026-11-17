//@ rules: REF-0402
// Red team (fx-qlek.5): impls for a type parameter that may be unsized whose trait declares no
// associated type a `dyn` type can choose stay admitted beside `escape_dyn_overlap_transmute`'s
// rejection. `Bytes` inherits `Item`, which its supertrait bound fixes and which `Iterator`'s
// impls decide; `Describe`'s only associated type requires `Self: Sized`, so no `dyn Describe`
// has one.
pub trait Bytes: Iterator<Item = u8> {}

impl<T: Iterator<Item = u8> + ?Sized> Bytes for T {}

pub fn next_byte(it: &mut dyn Bytes<Item = u8>) -> Option<u8> {
    it.next()
}

pub trait Describe {
    type Owned
    where
        Self: Sized;
    fn describe(&self) -> usize;
}

impl<T: ?Sized> Describe for T {
    type Owned = ()
    where
        Self: Sized;
    fn describe(&self) -> usize {
        0
    }
}

pub fn described(value: &dyn Describe) -> usize {
    value.describe()
}
