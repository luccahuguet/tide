//@ rules: CLAUSE-4
// A reference pattern copies a scalar; the binding retains no mutable borrow.
#[ensures(x == x)]
pub fn copied<'a>(&mut x: &'a mut u32, _peer: &'a u32) { let _: u32 = x; }

#[ensures(x == x)]
pub fn shared<'a>(&x: &'a u32, _peer: &'a u32) { let _: u32 = x; }

// A copied shared reference remains independent even when it points at a mutable reference.
#[ensures(**x == **x)]
pub fn copied_reference<'a, 'b, 'c>(&mut x: &'a mut &'b &'c mut u32, _peer: &'a u32) {
    let _: &'b &'c mut u32 = x;
}
