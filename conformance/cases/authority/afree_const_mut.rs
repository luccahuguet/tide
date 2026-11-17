//@ rules: AFREE-2
// A `const fn` takes no `&mut` parameter, although rustc allows one, nor one whose type holds a
// `&mut` in a field.
pub const fn bump(x: &mut u32) { //~ TD0518
    *x += 1;
}

pub struct Counter<'a>(&'a mut u32);

pub const fn bump_through(c: Counter<'_>) { //~ TD0518
    *c.0 += 1;
}
