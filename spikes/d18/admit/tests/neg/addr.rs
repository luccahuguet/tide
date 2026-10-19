#![forbid(unsafe_code)]
fn main() {
    let x = 7u8;
    let a = (&x as *const u8).addr(); // FX0520
    let _ = a;
}
