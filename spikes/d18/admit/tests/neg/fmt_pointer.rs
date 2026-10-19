#![forbid(unsafe_code)]
fn main() {
    let x = 7u8;
    let s = format!("{:p}", &x); // FX0520
    let _ = s;
}
