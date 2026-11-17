//@ rules: ADMIT-1, PAR-3
// Rust's float sign and bit observers and its math methods are outside the
// allowlist (std's `float::bits`, `float::min`, `float::max`, and `math::*`
// take their place); rustc accepts them.
pub fn observers(x: f64, y: f64) {
    let _ = x.to_bits(); //~ TD0520
    let _ = x.is_sign_negative(); //~ TD0520
    let _ = y.is_sign_positive(); //~ TD0520
    let _ = x.total_cmp(&y); //~ TD0520
    let _ = x.min(y); //~ TD0520
    let _ = x.max(y); //~ TD0520
    let _ = x.copysign(y) + y.signum(); //~ TD0520
}

pub fn math(x: f64) -> f64 {
    x.sin() + x.exp() + x.ln() + x.powf(2.0) //~ TD0520
}
