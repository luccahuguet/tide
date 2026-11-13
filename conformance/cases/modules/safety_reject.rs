//@ rules: REF-0902
//@ grants: unsafe
pub fn missing() {
    unsafe {} //~ FX0902
}
pub fn trailing() {
    let _ = 0; // SAFETY: this does not document the next block.
    unsafe {} //~ FX0902
}
pub fn detached() {
    // SAFETY: a blank line detaches this comment.

    unsafe {} //~ FX0902
}

pub fn arm(value: Option<u8>) -> u8 {
    match value {
        Some(value) => unsafe { value }, //~ FX0902
        None => 0,
    }
}
pub struct Field { pub value: u8 }
pub fn field(value: u8) -> Field {
    Field {
        value: unsafe { value }, //~ FX0902
    }
}
pub const CONST_VALUE: u8 = unsafe { 7 }; //~ FX0902
pub static STATIC_VALUE: u8 =
    unsafe { 7 }; //~ FX0902

pub struct Associated;
impl Associated {
    pub const VALUE: u8 =
        unsafe { 7 }; //~ FX0902
}
pub trait WithValue {
    const VALUE: u8 =
        unsafe { 7 }; //~ FX0902
}
