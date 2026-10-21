//@ rules: CLAUSE-4
// Where `x`'s origin appears in another parameter's type, naming `x` in an
// `ensures` outside `old(..)` is FX0803. A parameter that is neither `Copy` nor
// `Clone + Pure + Sync` is read in place, so naming it is FX0809 when the body
// assigns, moves, or mutably borrows it; rustc accepts the D18 spike's lowering of
// the first three, and rejects the moved one (E0382).
pub struct Token(pub u32);

#[ensures(*x == *y)] //~ FX0803
pub fn copy_in<'a>(x: &'a mut u32, y: &'a u32) {
    *x = *y;
}

#[ensures(result == t.0)] //~ FX0809
pub fn spend(mut t: Token) -> u32 {
    t.0 += 1;
    t.0
}

#[ensures(result == t.0)] //~ FX0809
pub fn reset(mut t: Token) -> u32 {
    let n = t.0;
    t = Token(0);
    n
}

#[ensures(result == t.0)] //~ E0382 FX0809
pub fn burn(t: Token) -> u32 {
    let n = t.0;
    drop(t);
    n
}
