//@ rules: REF-0902
//@ grants: unsafe
pub fn block() {
    // SAFETY: the block performs no unsafe operations.
    unsafe {}
}
pub fn statement() {
    // SAFETY: the block only produces an integer.
    let _n = unsafe { 1 };
}
pub struct Marker;
unsafe impl Send for Marker {}

pub fn arm(value: Option<u8>) -> u8 {
    match value {
        // SAFETY: this arm only returns its integer binding.
        Some(value) => unsafe { value },
        None => 0,
    }
}
pub struct Field { pub value: u8 }
pub fn field(value: u8) -> Field {
    Field {
        // SAFETY: this field only receives an integer.
        value: unsafe { value },
    }
}
// SAFETY: this initializer only produces an integer.
pub const CONST_VALUE: u8 = unsafe { 7 };
// SAFETY: this item only initializes an integer.
pub static STATIC_VALUE: u8 =
    unsafe { 7 };

pub struct Associated;
impl Associated {
    // SAFETY: this associated initializer only produces an integer.
    pub const VALUE: u8 =
        unsafe { 7 };
}
pub trait WithValue {
    // SAFETY: this default associated initializer only produces an integer.
    const VALUE: u8 =
        unsafe { 7 };
}
