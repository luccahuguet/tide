//@ rules: PURE-1
// A `dyn Trait` whose trait has no `Pure` supertrait is not `Pure`.
pub trait Value {
    fn get(&self) -> u32;
}

fn needs_pure<T: Pure + ?Sized>(_t: &T) {}

pub fn check(v: &dyn Value) {
    needs_pure(v); //~ FX0517
}
