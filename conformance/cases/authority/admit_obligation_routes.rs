//@ rules: ADMIT-1, GLOBAL-4
// The impl selected for an obligation is checked wherever the obligation arises: in `?`'s
// conversion, a `for` loop's `IntoIterator`, an autoderef, a type's where-clauses, which are
// also its `Drop` impl's, including where its constructor is passed as a function, a trait
// impl's supertraits and associated-type bounds, and a builtin impl's obligations on a type's
// parts. Each route here selects `fn(): Debug`, which observes
// an address.
use std::fmt::Debug;
use std::ops::Deref;

pub fn noop() {}

pub struct Wrap<T>(pub T);
pub struct Failure;
impl<T: Debug> From<Wrap<T>> for Failure {
    fn from(w: Wrap<T>) -> Failure {
        let _ = format!("{:?}", w.0);
        Failure
    }
}

pub fn convert() -> Result<(), Failure> {
    let r: Result<(), Wrap<fn()>> = Err(Wrap(noop));
    r?; //~ TD0520
    Ok(())
}

pub struct Items<T>(pub T);
impl<T: Debug> IntoIterator for Items<T> {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;
    fn into_iter(self) -> Self::IntoIter {
        vec![format!("{:?}", self.0)].into_iter()
    }
}

pub fn iterate() {
    for _ in Items(noop as fn()) {} //~ TD0520
}

pub struct Named<T>(pub T, pub String);
impl<T: Debug> Deref for Named<T> {
    type Target = String;
    fn deref(&self) -> &String {
        let _ = format!("{:?}", self.0);
        &self.1
    }
}

pub fn autoderef(n: &Named<fn()>) -> usize {
    n.len() //~ TD0520
}

pub struct Logged<T: Debug>(pub T);
impl<T: Debug> Drop for Logged<T> {
    fn drop(&mut self) {
        let _ = format!("{:?}", self.0);
    }
}

pub fn dropped() {
    let _l = Logged(noop as fn()); //~ TD0520
}

// std builds and drops each `Logged<fn()>`; no expression has that type.
pub fn mapped() -> usize {
    let fs: [fn(); 1] = [noop];
    fs.into_iter().map(Logged).count() //~ TD0520
}

// A generic `T: Shown` or a `dyn Shown` formats with the supertrait the impl relies on.
pub trait Shown: Debug {}
impl Shown for fn() {} //~ TD0520

// Code generic over `T: Make` formats `T::Out` with the bound the impl's type meets.
pub trait Make {
    type Out: Debug;
    fn make(&self) -> Self::Out;
}
impl Make for u8 {
    type Out = fn(); //~ TD0520
    fn make(&self) -> fn() {
        noop
    }
}

// Cloning a tuple clones each element, through the element's impl.
pub struct Twin<T>(pub T);
impl<T: Debug + Copy> Clone for Twin<T> {
    fn clone(&self) -> Self {
        let _ = format!("{:?}", self.0);
        Twin(self.0)
    }
}

pub fn cloned(t: &(Twin<fn()>,)) -> (Twin<fn()>,) {
    t.clone() //~ TD0520
}
