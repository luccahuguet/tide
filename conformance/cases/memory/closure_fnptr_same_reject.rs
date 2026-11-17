//@ rules: CLOSURE-3
// The conservative rule: `pick` implies `'b: 'a`, so coercing it to a `fn` pointer is rejected
// even at an instantiation that keeps the bound. rustc accepts it; the fix is a closure that
// calls `pick` (`closure_fnptr_accept.rs`).
fn pick<'a, 'b, T>(_: &'a &'b (), v: &'b T) -> &'a T { v }

pub fn same<'a>(unit: &'a &'a (), x: &'a u32) -> &'a u32 {
    let f: fn(&'a &'a (), &'a u32) -> &'a u32 = pick; //~ TD0206
    f(unit, x)
}
