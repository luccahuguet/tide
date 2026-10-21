//@ rules: AFREE-2
// A `const fn` takes no `&mut` parameter, although rustc allows one.
pub const fn bump(x: &mut u32) { //~ FX0518
    *x += 1;
}
