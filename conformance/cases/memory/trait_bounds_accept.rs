//@ rules: REF-0406
// Parameters, their projections, concrete types and strict subterms qualify.
pub trait Needs { type Item; }
pub trait Shape<A> {}
impl<T: Pure> Shape<()> for T {}
pub struct Stone<T>(T);
impl<T: Needs> Stone<T> where T::Item: Clone, u8: Copy {}
pub trait Nested {}
impl<T> Nested for Vec<Vec<T>> where Vec<T>: Nested {}
pub trait WithArg<A> {}
impl<T> WithArg<Vec<Option<T>>> for () where Option<T>: Clone {}
