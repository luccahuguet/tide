//@ rules: CLAUSE-3
// A clause that could change program state is FX0808, though rustc accepts each:
// an argument that is not `Sync` (`Cell`), a capability call, a `&mut` to a place
// outside the clause, and a generic argument whose type lacks `Pure + Sync`.
use std::cell::Cell;
use std::fs::Fs;

fn bump(v: &mut Vec<u32>) -> bool {
    v.push(0);
    true
}

#[requires(c.get() > 0)] //~ FX0808
pub fn read(c: &Cell<u32>) -> u32 {
    c.get()
}

#[requires(fs.exists("cfg").unwrap_or(false))] //~ FX0808
pub fn load(fs: &Fs) -> u32 {
    0
}

#[ensures(bump(v))] //~ FX0808
pub fn clear(v: &mut Vec<u32>) {
    v.clear();
}

#[ensures(result == (a == b))] //~ FX0808
pub fn same<T: PartialEq>(a: &T, b: &T) -> bool {
    a == b
}
