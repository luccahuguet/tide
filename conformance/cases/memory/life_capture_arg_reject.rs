//@ rules: LIFE-2
// `use<..>` is written only on a return-position `impl Trait`.
pub fn arg(_x: impl Sized + use<>) {} //~ error
