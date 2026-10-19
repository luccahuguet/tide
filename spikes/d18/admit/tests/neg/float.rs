#![forbid(unsafe_code)]
const BITS: u64 = f64::to_bits(1.5); // FX0520 (const context)
fn main() {
    let a = 0.0_f64.sin(); // FX0520 std::f64::<impl f64>::sin, the real std's inherent method
    let b = 1.0_f64.to_bits(); // FX0520
    let c = f64::min(a, 2.0); // FX0520
    let g: fn(f64) -> f64 = f64::sqrt; // FX0520 (path, not a call)
    let h = [1.0_f64].map(<f64>::exp); // FX0520 (UFCS)
    let _ = (b, c, g, h, BITS);
}
