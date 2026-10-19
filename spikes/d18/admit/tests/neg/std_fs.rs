#![forbid(unsafe_code)]
// Compiled against the REAL std: shows the checker catches ambient std if the facade is bypassed.
fn main() {
    let bytes = std::fs::read("/etc/hostname"); // FX0520
    let _ = bytes; // FX0520
}
