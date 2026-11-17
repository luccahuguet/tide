//@ rules: GLOBAL-1
// No `static mut` in safe code.
pub static mut COUNT: u32 = 0; //~ TD0502
