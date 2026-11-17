//@ rules: AFREE-2
// A `const fn`'s parameter types are `Pure + Sync`, and `&Cell<u32>` is not `Sync`.
use std::cell::Cell;

pub const fn peek(c: &Cell<u32>) -> u32 { //~ TD0518
    c.get()
}
