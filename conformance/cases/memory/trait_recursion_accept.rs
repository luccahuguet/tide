//@ rules: REF-0407
// Fixed types, parameter permutations, and acyclic growth cannot grow a cycle.
pub fn direct<T>(x: T) { direct(x); }
pub fn permute<A, B>(a: A, b: B) { permute(b, a); }
pub fn first<T>(x: T) { second(x); }
fn second<T>(x: T) { first(x); }
pub fn outer<T>(x: T) { leaf(Some(x)); }
fn leaf<T>(_: T) {}
pub fn concrete<T>(_: T) { concrete(1u8); }
pub fn concrete_vector<T>(_: T) { concrete_vector(vec![1u8]); }

// The receiver's fixed S<T> shape is not a growing impl argument.
pub struct Stone<T>(T);
pub trait Same { fn same(self); }
impl<T> Same for Stone<T> {
    fn same(self) { self.same(); }
}
// A projection is not a constructor; the equality makes this chain idempotent.
pub trait Project { type A: Project<A = Self::A>; }
pub fn projected<T: Project>(n: u8) {
    if n > 0 { projected::<T::A>(n - 1); }
}
pub struct Fixed;
impl Project for Fixed { type A = Fixed; }
pub fn entry() { projected::<Fixed>(2); }
// Unresolved projections, even inside another constructor, prove no parameter growth.
pub fn unresolved<T: Project>() { unresolved::<T::A>(); }
pub fn wrapped_projection<T: Project, U>() { wrapped_projection::<T, Option<T::A>>(); }

// A selected default terminates through Wrap's override; parameter dispatch proves no edge.
pub trait K: Sized { fn f(self) { g(Wrap(self)) } }
pub struct Wrap<T>(T);
impl<T> K for Wrap<T> { fn f(self) {} }
pub fn g<T: K>(x: T) { x.f() }
pub struct S;
impl K for S {}
pub fn default_entry() { g(S) }

// The wrapped parameter is reset before it can return around the cycle.
pub fn reset<T>(n: u8, x: T) { if n > 0 { reset_next(n - 1, vec![x]) } }
fn reset_next<U>(n: u8, _u: U) { if n > 0 { reset(n - 1, 0u8) } }
pub fn reset_entry() { reset(3, 'c') }
// U becomes Option<T> once; the expanded T-to-U edge lies on no parameter cycle.
pub fn fixed_wrap<T: Clone, U>(n: u8, x: T, _u: U) {
    if n > 0 { fixed_wrap(n - 1, x.clone(), Some(x)) }
}
pub fn fixed_entry() { fixed_wrap(3, 1u8, ()) }
