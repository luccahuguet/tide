// C5: facade float functions. Built with `--extern libm=` only so the test can
// compare against the pinned libm directly.
use std::float::{self, CANONICAL_NAN_BITS};
use std::math;

fn main() {
    // Distinct NaN payloads and signs: one canonical bit pattern.
    let nans = [f64::NAN, -f64::NAN, f64::from_bits(0x7ff8_0000_0000_0001), f64::from_bits(0xfff4_dead_beef_0000)];
    let raw: Vec<u64> = nans.iter().map(|x| x.to_bits()).collect(); // inherent to_bits: still callable
    assert!(raw.windows(2).all(|w| w[0] != w[1]), "payloads should differ");
    assert!(nans.iter().all(|x| float::bits(*x) == CANONICAL_NAN_BITS));
    println!("bits: raw NaN payloads {raw:x?} -> all {CANONICAL_NAN_BITS:#x}");

    // math::* equals the pinned libm bit for bit.
    let mut x = 0x1234_5678_9abc_def0u64;
    let mut inherent_diff = 0;
    let n = 200_000;
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let v = f64::from_bits(x >> 2 | 0x3000_0000_0000_0000) * if x & 1 == 0 { 1.0 } else { -1.0 };
        assert_eq!(float::bits(math::sin(v)), float::bits(libm::sin(v)), "sin({v})");
        assert_eq!(float::bits(math::exp(v)), float::bits(libm::exp(v)), "exp({v})");
        if v.sin().to_bits() != math::sin(v).to_bits() || v.exp().to_bits() != math::exp(v).to_bits() {
            inherent_diff += 1; // inherent f64::sin/exp: platform libm, still callable
        }
    }
    for v in [0.0, -0.0, 1.0, 709.0, 710.0, -745.2, f64::INFINITY, f64::NAN, 1e-310] {
        assert_eq!(float::bits(math::sin(v)), float::bits(libm::sin(v)));
        assert_eq!(float::bits(math::exp(v)), float::bits(libm::exp(v)));
    }
    println!("math: sin/exp equal libm on {n} inputs + edge cases; inherent x.sin()/x.exp() differ on {inherent_diff}");

    // min/max order -0.0 below +0.0, in either argument order.
    for (a, b) in [(0.0, -0.0), (-0.0, 0.0)] {
        assert_eq!(float::bits(float::min(a, b)), (-0.0f64).to_bits());
        assert_eq!(float::bits(float::max(a, b)), 0.0f64.to_bits());
    }
    assert_eq!(float::min(f64::NAN, 1.0), 1.0);
    println!(
        "min/max: float::min(0.0, -0.0) = {:?}, float::max(-0.0, 0.0) = {:?}; inherent 0.0.min(-0.0) = {:?}",
        float::min(0.0, -0.0),
        float::max(-0.0, 0.0),
        0.0f64.min(-0.0)
    );
}
