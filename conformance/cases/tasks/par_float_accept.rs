//@ rules: ADMIT-1, PAR-3
// Arithmetic, the correctly rounded `sqrt` and `mul_add`, and `from_bits`
// stay admitted: none reads a NaN's sign or bits (D3).
pub fn hypot_fma(x: f64, y: f64, z: f64) -> f64 {
    (x * x + y * y).sqrt().mul_add(z, f64::from_bits(1) - 1.0 / 3.0)
}
