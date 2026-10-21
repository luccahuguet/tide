//@ rules: MOVE-1, MOVE-2
// `Copy` never applies to a type holding `&mut T` or to a type with a destructor,
// and a `&mut T` used by value moves.
pub struct Cursor<'a>(&'a mut u8);
impl Clone for Cursor<'_> {
    fn clone(&self) -> Self {
        unimplemented!()
    }
}
impl Copy for Cursor<'_> {} //~ E0204

pub struct Guard;
impl Clone for Guard {
    fn clone(&self) -> Self {
        Guard
    }
}
impl Copy for Guard {} //~ E0184
impl Drop for Guard {
    fn drop(&mut self) {}
}

pub fn moves(x: &mut u8) {
    let _a = x;
    let _b = x; //~ E0382
}
