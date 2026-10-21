//@ rules: CLOSURE-3
// Coercing a fn item to a `fn` pointer whose instantiation keeps the item's implied bounds.
fn pick<'a, 'b, T>(_: &'a &'b (), v: &'b T) -> &'a T { v }
fn add(a: &u32, b: &u32) -> u32 { a + b }

pub fn sum(x: u32, y: u32) -> u32 { let f: fn(&u32, &u32) -> u32 = add; f(&x, &y) }
pub fn same<'a>(unit: &'a &'a (), x: &'a u32) -> &'a u32 { let f: fn(&'a &'a (), &'a u32) -> &'a u32 = pick; f(unit, x) }
