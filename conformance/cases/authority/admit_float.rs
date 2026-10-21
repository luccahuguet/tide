//@ rules: ADMIT-1
// Float bit observers and math methods are outside the allowlist; `sqrt` and `abs` stay (D3).
pub fn bits(x: f64) -> u64 {
    x.to_bits() //~ FX0520
}

pub fn wave(x: f64) -> f64 {
    x.sin() //~ FX0520
}

pub fn norm(x: f64, y: f64) -> f64 {
    (x * x + y * y).sqrt().abs()
}
