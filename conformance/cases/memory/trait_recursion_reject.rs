//@ rules: REF-0407
// Uncalled definitions are checked, including mutual and method cycles.
pub fn direct<T>(x: T) { direct(vec![x]); } //~ TD0407
pub fn first<T>(x: T) { second(Some(x)); } //~ TD0407
fn second<T>(x: T) { first(x); }
pub fn reference<T>(x: T) { reference(&x); } //~ TD0407
pub struct Stone<T>(T);
impl<T> Stone<T> {
    pub fn grow(self) { Stone(Some(self.0)).grow(); } //~ TD0407
}
pub fn wrapped_closure<T>(x: T) { wrapped_closure(move || x); } //~ TD0407

// Both spellings select this local impl even when no instance is called.
pub struct Selected<T>(T);
pub trait Grow { fn dot(self); fn qualified(self); }
impl<T> Grow for Selected<T> {
    fn dot(self) { Selected(Some(self.0)).dot(); } //~ TD0407
    fn qualified(self) { <Selected<Option<T>> as Grow>::qualified(Selected(Some(self.0))); } //~ TD0407
}
// The caller's equality reveals a real constructor beneath the projection.
pub trait Project { type A; }
pub fn normalized<T, U: Project<A = Option<T>>>() { normalized::<U::A, Wrap<U::A>>(); } //~ TD0407
pub struct Wrap<T>(T);
impl<T> Project for Wrap<T> { type A = Option<T>; }

// Selection proves that Wrap uses the default, whose Self parameter grows on a cycle.
pub trait DefaultGrow: Sized {
    fn grow(self) { DefaultWrap(self).grow(); } //~ TD0407
}
pub struct DefaultWrap<T>(T);
impl<T> DefaultGrow for DefaultWrap<T> {}
