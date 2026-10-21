//@ rules: ORIGIN-3
// Returning `x` as `&'b u8` needs `'a: 'b`, which neither a bound nor a parameter type
// gives.
pub fn unrelated<'a, 'b>(x: &'a u8, _y: &'b u8) -> &'b u8 {
    x //~ error
}
