//@ rules: REF-0406
// Growing a parameter in the bounded type exceeds the subterm ceiling.
pub trait Needs {}
pub trait Shape {}
impl<T> Shape for T where Vec<T>: Needs {} //~ TD0406
pub struct Stone<T>(T);
impl<T> Stone<T> where Option<T>: Needs {} //~ TD0406
pub struct Lived<'a, T>(&'a (), T);
impl<'a, T> Lived<'a, T> where Vec<T>: 'a {} //~ TD0406
pub trait Equal {}
impl<T> Equal for Vec<T> where Vec<T>: Clone {} //~ TD0406
pub trait Loop {}
impl<T> Loop for T where Vec<T>: Loop {} //~ E0275 TD0406
