#![forbid(unsafe_code)]
fn main() {
    let a = core::sync::atomic::AtomicU64::new(0); // FX0520
    let _ = a; // FX0520
}
