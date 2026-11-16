//@ rules: CLOSURE-3
// Coercing a fn item to a `fn` pointer is accepted when it implies no outlives bound (`add`),
// or only one on a type parameter declared `'static` (`owned`). A closure that calls an item
// which does imply one is accepted too: rustc checks the call's bounds
// (`closure_fnptr_same_reject.rs` coerces `pick` itself).
fn pick<'a, 'b, T>(_: &'a &'b (), v: &'b T) -> &'a T { v }
fn add(a: &u32, b: &u32) -> u32 { a + b }
fn owned<T: 'static>(_: &T) -> u32 { 0 }

pub fn sum(x: u32, y: u32) -> u32 { let f: fn(&u32, &u32) -> u32 = add; f(&x, &y) }
pub fn same<'a>(unit: &'a &'a (), x: &'a u32) -> &'a u32 { let f: fn(&'a &'a (), &'a u32) -> &'a u32 = |u, v| pick(u, v); f(unit, x) }
pub fn kept() -> u32 { let f: fn(&u8) -> u32 = owned::<u8>; f(&1) }
