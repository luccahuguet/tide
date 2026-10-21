//@ rules: DROP-1, DROP-2
// The fix for a destructor that blocks a borrow is to move it onto an inner
// owned type, not to exempt it: `Outer` has no destructor of its own, so its
// borrow may dangle when it drops.
pub struct Buffer {
    pub bytes: Vec<u8>,
}
impl Drop for Buffer {
    fn drop(&mut self) {}
}

pub struct Outer<'a> {
    pub name: &'a String,
    pub buffer: Buffer,
}

pub fn ok() {
    let outer;
    let name = String::from("x");
    outer = Outer { name: &name, buffer: Buffer { bytes: Vec::new() } };
    let _ = outer.buffer.bytes.len();
}
