//@ rules: ADMIT-1, PAR-3
// Float contract and regression vectors; no trial task source is used.
// Numerical literals record libm 0.2.16 force-soft-floats, plus the documented compositions.
// These self-recorded values are regression vectors, not a claim of correct transcendental rounding.
// Independent MPFR/formula expectations are in the generated oracle section below.
use std::cmp::Ordering::{Equal, Greater, Less};
use std::{float, math};

fn f32_contract() {
    let canonical: u32 = 0x7fc00000;
    let n = f32::from_bits(canonical);
    assert_eq!(float::bits(float::min(-0.0f32, 0.0)), 0x80000000);
    assert_eq!(float::bits(float::min(0.0f32, -0.0)), 0x80000000);
    assert_eq!(float::bits(float::max(-0.0f32, 0.0)), 0);
    assert_eq!(float::bits(float::max(0.0f32, -0.0)), 0);
    assert_eq!(float::bits(float::min(-0.0f32, -0.0)), 0x80000000);
    assert_eq!(float::bits(float::max(-0.0f32, -0.0)), 0x80000000);
    for payload in [1u32, 2, 0x12345, 0x7fc00000 & !0x7f800000] {
        for sign in [0u32, 0x80000000] {
            let x = f32::from_bits(0x7f800000 | payload | sign);
            assert_eq!(float::bits(x), canonical);
            assert_eq!(float::total_cmp(x, n), Equal);
            assert_eq!(float::total_cmp(x, f32::INFINITY), Greater);
            assert_eq!(float::total_cmp(x, 0.0), Greater);
            assert_eq!(float::min(x, 3.0), 3.0);
            assert_eq!(float::min(3.0, x), 3.0);
            assert_eq!(float::max(x, 3.0), 3.0);
            assert_eq!(float::max(3.0, x), 3.0);
            assert_eq!(float::bits(float::min(x, n)), canonical);
            assert_eq!(float::bits(float::max(x, n)), canonical);
            assert_eq!(float::bits(math::sin(x)), canonical);
            assert_eq!(float::bits(math::powf(x, 0.0)), float::bits(1.0f32));
            assert_eq!(float::bits(math::powf(1.0, x)), float::bits(1.0f32));
            assert_eq!(float::bits(math::powf(-1.0, x)), canonical);
        }
    }
    let ordered = [
        f32::NEG_INFINITY,
        -f32::MAX,
        -1.0,
        -f32::MIN_POSITIVE,
        -f32::from_bits(1),
        -0.0,
        0.0,
        f32::from_bits(1),
        f32::MIN_POSITIVE,
        1.0,
        f32::MAX,
        f32::INFINITY,
        n,
    ];
    for (i, a) in ordered.iter().enumerate() {
        for (j, b) in ordered.iter().enumerate() {
            assert_eq!(float::total_cmp(*a, *b), i.cmp(&j));
        }
    }
    assert_eq!(float::total_cmp(-0.0f32, 0.0), Less);
    assert_eq!(float::bits(f32::MAX), 0x7f7fffff);
    // Every kept method executes, with its observable result checked.
    assert_eq!(float::bits(f32::from_bits(1)), 1);
    assert_eq!(9.0f32.sqrt(), 3.0);
    assert_eq!(2.0f32.mul_add(3.0, 4.0), 10.0);
    assert_eq!((-1.5f32).floor(), -2.0);
    assert_eq!((-1.5f32).ceil(), -1.0);
    assert_eq!((-1.5f32).round(), -2.0);
    assert_eq!((-1.5f32).trunc(), -1.0);
    assert_eq!((-1.5f32).abs(), 1.5);
    assert!(n.is_nan());
    assert!(!f32::INFINITY.is_finite());
    assert!(1.0f32.is_finite());
    // Recorded nontrivial vectors for every exposed math function.
    assert_eq!(float::bits(math::sin(0.5f32)), 0x3ef57744, "f32::sin");
    assert_eq!(float::bits(math::cos(0.5f32)), 0x3f60a940, "f32::cos");
    assert_eq!(float::bits(math::tan(0.5f32)), 0x3f0bda7b, "f32::tan");
    assert_eq!(float::bits(math::asin(0.5f32)), 0x3f060a92, "f32::asin");
    assert_eq!(float::bits(math::acos(0.5f32)), 0x3f860a92, "f32::acos");
    assert_eq!(float::bits(math::atan(0.5f32)), 0x3eed6338, "f32::atan");
    assert_eq!(float::bits(math::sinh(0.5f32)), 0x3f056680, "f32::sinh");
    assert_eq!(float::bits(math::cosh(0.5f32)), 0x3f90560c, "f32::cosh");
    assert_eq!(float::bits(math::tanh(0.5f32)), 0x3eec9a9e, "f32::tanh");
    assert_eq!(float::bits(math::asinh(0.5f32)), 0x3ef66166, "f32::asinh");
    assert_eq!(float::bits(math::acosh(1.5f32)), 0x3f766166, "f32::acosh");
    assert_eq!(float::bits(math::atanh(0.5f32)), 0x3f0c9f54, "f32::atanh");
    assert_eq!(float::bits(math::exp(0.5f32)), 0x3fd3094c, "f32::exp");
    assert_eq!(float::bits(math::exp2(0.5f32)), 0x3fb504f3, "f32::exp2");
    assert_eq!(float::bits(math::exp_m1(0.5f32)), 0x3f261299, "f32::exp_m1");
    assert_eq!(float::bits(math::ln(0.5f32)), 0xbf317218, "f32::ln");
    assert_eq!(float::bits(math::ln_1p(0.5f32)), 0x3ecf991f, "f32::ln_1p");
    assert_eq!(float::bits(math::log2(0.5f32)), 0xbf800000, "f32::log2");
    assert_eq!(float::bits(math::log10(0.5f32)), 0xbe9a209b, "f32::log10");
    assert_eq!(float::bits(math::cbrt(0.5f32)), 0x3f4b2ff5, "f32::cbrt");
    assert_eq!(
        float::bits(math::round_ties_even(2.5f32)),
        0x40000000,
        "f32::round_ties_even"
    );
    assert_eq!(float::bits(math::fract(-1.75f32)), 0xbf400000, "f32::fract");
    assert_eq!(float::bits(math::recip(0.5f32)), 0x40000000, "f32::recip");
    assert_eq!(
        float::bits(math::to_degrees(0.5f32)),
        0x41e52ee1,
        "f32::to_degrees"
    );
    assert_eq!(
        float::bits(math::to_radians(30.0f32)),
        0x3f060a92,
        "f32::to_radians"
    );
    assert_eq!(
        float::bits(math::powf(1.5f32, 0.75f32)),
        0x3fad7dd8,
        "f32::powf"
    );
    assert_eq!(
        float::bits(math::hypot(1.25f32, 0.75f32)),
        0x3fba9728,
        "f32::hypot"
    );
    assert_eq!(
        float::bits(math::atan2(1.25f32, -0.75f32)),
        0x40071e29,
        "f32::atan2"
    );
    assert_eq!(
        float::bits(math::abs_sub(1.25f32, 0.75f32)),
        0x3f000000,
        "f32::abs_sub"
    );
    assert_eq!(
        float::bits(math::log(3.0f32, 2.0f32)),
        0x3fcae00d,
        "f32::log"
    );
    assert_eq!(
        float::bits(math::div_euclid(-7.0f32, 4.0f32)),
        0xc0000000,
        "f32::div_euclid"
    );
    assert_eq!(
        float::bits(math::rem_euclid(-7.0f32, 4.0f32)),
        0x3f800000,
        "f32::rem_euclid"
    );
    assert_eq!(
        float::bits(math::midpoint(1.25f32, 0.75f32)),
        0x3f800000,
        "f32::midpoint"
    );
    assert_eq!(float::bits(math::powi(1.5f32, -7)), 0x3d6fbab4);
    assert_eq!(float::bits(math::clamp(1.5f32, 0.0f32, 1.0f32)), 0x3f800000);
    let (s, c) = math::sin_cos(0.5f32);
    assert_eq!((float::bits(s), float::bits(c)), (0x3ef57744, 0x3f60a940));
    // Domains, infinities, subnormals, sign-preserving zeros, overflow and underflow.
    assert_eq!(float::bits(math::sin(-0.0f32)), 0x80000000);
    assert_eq!(float::bits(math::sin(f32::from_bits(1))), 1);
    assert_eq!(float::bits(math::ln(-1.0f32)), canonical);
    assert_eq!(float::bits(math::ln(0.0f32)), 0x7f800000 | 0x80000000);
    assert_eq!(float::bits(math::exp(f32::MAX)), 0x7f800000);
    assert_eq!(float::bits(math::exp(-f32::MAX)), 0);
    assert_eq!(float::bits(math::atanh(1.0f32)), 0x7f800000);
    assert_eq!(float::bits(math::acosh(0.5f32)), canonical);
    assert_eq!(float::bits(math::hypot(n, f32::INFINITY)), 0x7f800000);
    assert_eq!(math::powi(-1.0f32, i32::MIN), 1.0);
    assert_eq!(math::powi(-1.0f32, i32::MAX), -1.0);
    assert_eq!(math::powi(n, 0), 1.0);
    assert_eq!(float::bits(math::powi(-0.0f32, 3)), 0x80000000);
    assert_eq!(
        float::bits(math::powi(-0.0f32, -3)),
        0x7f800000 | 0x80000000
    );
    assert_eq!(float::bits(math::recip(-0.0f32)), 0x7f800000 | 0x80000000);
    assert_eq!(float::bits(math::round_ties_even(-0.5f32)), 0x80000000);
    assert_eq!(float::bits(math::midpoint(f32::MAX, f32::MAX)), 0x7f7fffff);
    // Large argument reduction: recorded independently with the host C math library.
    assert_eq!(float::bits(math::sin(1e20f32)), 0x3f281569);
    assert_eq!(float::bits(math::ln_1p(f32::from_bits(1))), 1);
    assert_eq!(float::bits(math::exp_m1(-0.0f32)), 0x80000000);
    assert_eq!(float::bits(math::atan2(0.0f32, -0.0)), 0x40490fdb);
    assert_eq!(
        float::bits(math::atan2(-0.0f32, -0.0)),
        0x40490fdb | 0x80000000
    );
    assert_eq!(float::bits(math::powf(-2.0f32, 0.5)), canonical);
    assert_eq!(float::bits(math::fract(f32::INFINITY)), canonical);
    assert_eq!(float::bits(math::powi(2.0f32, -149)), 0);
    assert_eq!(math::cbrt(-8.0f32), -2.0);
}

fn f64_contract() {
    independent_numerical_contract();
    let canonical: u64 = 0x7ff8000000000000;
    let n = f64::from_bits(canonical);
    assert_eq!(float::bits(float::min(-0.0f64, 0.0)), 0x8000000000000000);
    assert_eq!(float::bits(float::min(0.0f64, -0.0)), 0x8000000000000000);
    assert_eq!(float::bits(float::max(-0.0f64, 0.0)), 0);
    assert_eq!(float::bits(float::max(0.0f64, -0.0)), 0);
    assert_eq!(float::bits(float::min(-0.0f64, -0.0)), 0x8000000000000000);
    assert_eq!(float::bits(float::max(-0.0f64, -0.0)), 0x8000000000000000);
    for payload in [1u64, 2, 0x12345, 0x7ff8000000000000 & !0x7ff0000000000000] {
        for sign in [0u64, 0x8000000000000000] {
            let x = f64::from_bits(0x7ff0000000000000 | payload | sign);
            assert_eq!(float::bits(x), canonical);
            assert_eq!(float::total_cmp(x, n), Equal);
            assert_eq!(float::total_cmp(x, f64::INFINITY), Greater);
            assert_eq!(float::total_cmp(x, 0.0), Greater);
            assert_eq!(float::min(x, 3.0), 3.0);
            assert_eq!(float::min(3.0, x), 3.0);
            assert_eq!(float::max(x, 3.0), 3.0);
            assert_eq!(float::max(3.0, x), 3.0);
            assert_eq!(float::bits(float::min(x, n)), canonical);
            assert_eq!(float::bits(float::max(x, n)), canonical);
            assert_eq!(float::bits(math::sin(x)), canonical);
            assert_eq!(float::bits(math::powf(x, 0.0)), float::bits(1.0f64));
            assert_eq!(float::bits(math::powf(1.0, x)), float::bits(1.0f64));
            assert_eq!(float::bits(math::powf(-1.0, x)), canonical);
        }
    }
    let ordered = [
        f64::NEG_INFINITY,
        -f64::MAX,
        -1.0,
        -f64::MIN_POSITIVE,
        -f64::from_bits(1),
        -0.0,
        0.0,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        1.0,
        f64::MAX,
        f64::INFINITY,
        n,
    ];
    for (i, a) in ordered.iter().enumerate() {
        for (j, b) in ordered.iter().enumerate() {
            assert_eq!(float::total_cmp(*a, *b), i.cmp(&j));
        }
    }
    assert_eq!(float::total_cmp(-0.0f64, 0.0), Less);
    assert_eq!(float::bits(f64::MAX), 0x7fefffffffffffff);
    // Every kept method executes, with its observable result checked.
    assert_eq!(float::bits(f64::from_bits(1)), 1);
    assert_eq!(9.0f64.sqrt(), 3.0);
    assert_eq!(2.0f64.mul_add(3.0, 4.0), 10.0);
    assert_eq!((-1.5f64).floor(), -2.0);
    assert_eq!((-1.5f64).ceil(), -1.0);
    assert_eq!((-1.5f64).round(), -2.0);
    assert_eq!((-1.5f64).trunc(), -1.0);
    assert_eq!((-1.5f64).abs(), 1.5);
    assert!(n.is_nan());
    assert!(!f64::INFINITY.is_finite());
    assert!(1.0f64.is_finite());
    // Recorded nontrivial vectors for every exposed math function.
    assert_eq!(
        float::bits(math::sin(0.5f64)),
        0x3fdeaee8744b05f0,
        "f64::sin"
    );
    assert_eq!(
        float::bits(math::cos(0.5f64)),
        0x3fec1528065b7d50,
        "f64::cos"
    );
    assert_eq!(
        float::bits(math::tan(0.5f64)),
        0x3fe17b4f5bf3474a,
        "f64::tan"
    );
    assert_eq!(
        float::bits(math::asin(0.5f64)),
        0x3fe0c152382d7366,
        "f64::asin"
    );
    assert_eq!(
        float::bits(math::acos(0.5f64)),
        0x3ff0c152382d7366,
        "f64::acos"
    );
    assert_eq!(
        float::bits(math::atan(0.5f64)),
        0x3fddac670561bb4f,
        "f64::atan"
    );
    assert_eq!(
        float::bits(math::sinh(0.5f64)),
        0x3fe0acd00fe63b97,
        "f64::sinh"
    );
    assert_eq!(
        float::bits(math::cosh(0.5f64)),
        0x3ff20ac1862ae8d0,
        "f64::cosh"
    );
    assert_eq!(
        float::bits(math::tanh(0.5f64)),
        0x3fdd9353d7568af3,
        "f64::tanh"
    );
    assert_eq!(
        float::bits(math::asinh(0.5f64)),
        0x3fdecc2caec5160a,
        "f64::asinh"
    );
    assert_eq!(
        float::bits(math::acosh(1.5f64)),
        0x3feecc2caec5160a,
        "f64::acosh"
    );
    assert_eq!(
        float::bits(math::atanh(0.5f64)),
        0x3fe193ea7aad030a,
        "f64::atanh"
    );
    assert_eq!(
        float::bits(math::exp(0.5f64)),
        0x3ffa61298e1e069c,
        "f64::exp"
    );
    assert_eq!(
        float::bits(math::exp2(0.5f64)),
        0x3ff6a09e667f3bcd,
        "f64::exp2"
    );
    assert_eq!(
        float::bits(math::exp_m1(0.5f64)),
        0x3fe4c2531c3c0d38,
        "f64::exp_m1"
    );
    assert_eq!(float::bits(math::ln(0.5f64)), 0xbfe62e42fefa39ef, "f64::ln");
    assert_eq!(
        float::bits(math::ln_1p(0.5f64)),
        0x3fd9f323ecbf984c,
        "f64::ln_1p"
    );
    assert_eq!(
        float::bits(math::log2(0.5f64)),
        0xbff0000000000000,
        "f64::log2"
    );
    assert_eq!(
        float::bits(math::log10(0.5f64)),
        0xbfd34413509f79ff,
        "f64::log10"
    );
    assert_eq!(
        float::bits(math::cbrt(0.5f64)),
        0x3fe965fea53d6e3d,
        "f64::cbrt"
    );
    assert_eq!(
        float::bits(math::round_ties_even(2.5f64)),
        0x4000000000000000,
        "f64::round_ties_even"
    );
    assert_eq!(
        float::bits(math::fract(-1.75f64)),
        0xbfe8000000000000,
        "f64::fract"
    );
    assert_eq!(
        float::bits(math::recip(0.5f64)),
        0x4000000000000000,
        "f64::recip"
    );
    assert_eq!(
        float::bits(math::to_degrees(0.5f64)),
        0x403ca5dc1a63c1f8,
        "f64::to_degrees"
    );
    assert_eq!(
        float::bits(math::to_radians(30.0f64)),
        0x3fe0c152382d7365,
        "f64::to_radians"
    );
    assert_eq!(
        float::bits(math::powf(1.5f64, 0.75f64)),
        0x3ff5afbb0fd2812c,
        "f64::powf"
    );
    assert_eq!(
        float::bits(math::hypot(1.25f64, 0.75f64)),
        0x3ff752e50db3a3a2,
        "f64::hypot"
    );
    assert_eq!(
        float::bits(math::atan2(1.25f64, -0.75f64)),
        0x4000e3c51fa059c4,
        "f64::atan2"
    );
    assert_eq!(
        float::bits(math::abs_sub(1.25f64, 0.75f64)),
        0x3fe0000000000000,
        "f64::abs_sub"
    );
    assert_eq!(
        float::bits(math::log(3.0f64, 2.0f64)),
        0x3ff95c01a39fbd68,
        "f64::log"
    );
    assert_eq!(
        float::bits(math::div_euclid(-7.0f64, 4.0f64)),
        0xc000000000000000,
        "f64::div_euclid"
    );
    assert_eq!(
        float::bits(math::rem_euclid(-7.0f64, 4.0f64)),
        0x3ff0000000000000,
        "f64::rem_euclid"
    );
    assert_eq!(
        float::bits(math::midpoint(1.25f64, 0.75f64)),
        0x3ff0000000000000,
        "f64::midpoint"
    );
    assert_eq!(float::bits(math::powi(1.5f64, -7)), 0x3fadf75680feb65f);
    assert_eq!(
        float::bits(math::clamp(1.5f64, 0.0f64, 1.0f64)),
        0x3ff0000000000000
    );
    let (s, c) = math::sin_cos(0.5f64);
    assert_eq!(
        (float::bits(s), float::bits(c)),
        (0x3fdeaee8744b05f0, 0x3fec1528065b7d50)
    );
    assert_eq!(float::bits(math::exp(0.019f64)), 0x3ff04e916a97ec0e);
    assert_eq!(float::bits(math::exp2(16.077f64)), 0x40f0e08d5dae5f53);
    // Domains, infinities, subnormals, sign-preserving zeros, overflow and underflow.
    assert_eq!(float::bits(math::sin(-0.0f64)), 0x8000000000000000);
    assert_eq!(float::bits(math::sin(f64::from_bits(1))), 1);
    assert_eq!(float::bits(math::ln(-1.0f64)), canonical);
    assert_eq!(
        float::bits(math::ln(0.0f64)),
        0x7ff0000000000000 | 0x8000000000000000
    );
    assert_eq!(float::bits(math::exp(f64::MAX)), 0x7ff0000000000000);
    assert_eq!(float::bits(math::exp(-f64::MAX)), 0);
    assert_eq!(float::bits(math::atanh(1.0f64)), 0x7ff0000000000000);
    assert_eq!(float::bits(math::acosh(0.5f64)), canonical);
    assert_eq!(
        float::bits(math::hypot(n, f64::INFINITY)),
        0x7ff0000000000000
    );
    assert_eq!(math::powi(-1.0f64, i32::MIN), 1.0);
    assert_eq!(math::powi(-1.0f64, i32::MAX), -1.0);
    assert_eq!(math::powi(n, 0), 1.0);
    assert_eq!(float::bits(math::powi(-0.0f64, 3)), 0x8000000000000000);
    assert_eq!(
        float::bits(math::powi(-0.0f64, -3)),
        0x7ff0000000000000 | 0x8000000000000000
    );
    assert_eq!(
        float::bits(math::recip(-0.0f64)),
        0x7ff0000000000000 | 0x8000000000000000
    );
    assert_eq!(
        float::bits(math::round_ties_even(-0.5f64)),
        0x8000000000000000
    );
    assert_eq!(
        float::bits(math::midpoint(f64::MAX, f64::MAX)),
        0x7fefffffffffffff
    );
    // Large argument reduction: recorded independently with the host C math library.
    assert_eq!(float::bits(math::sin(1e20f64)), 0xbfe4a5e605fd6450);
    assert_eq!(float::bits(math::ln_1p(f64::from_bits(1))), 1);
    assert_eq!(float::bits(math::exp_m1(-0.0f64)), 0x8000000000000000);
    assert_eq!(float::bits(math::atan2(0.0f64, -0.0)), 0x400921fb54442d18);
    assert_eq!(
        float::bits(math::atan2(-0.0f64, -0.0)),
        0x400921fb54442d18 | 0x8000000000000000
    );
    assert_eq!(float::bits(math::powf(-2.0f64, 0.5)), canonical);
    assert_eq!(float::bits(math::fract(f64::INFINITY)), canonical);
    assert_eq!(float::bits(math::powi(2.0f64, -1074)), 0);
    assert_eq!(math::cbrt(-8.0f64), -2.0);
}

// BEGIN independent oracle (generated by conformance/oracles/float_math.py)
// MPFR: 256-bit directed intervals certify nearest-even expectations.
// ULP bounds: pinned upstream libm-test/src/precision.rs, linked in the script.
fn independent_numerical_contract() {
    assert!(float::bits(math::sin(0.5f32)).abs_diff(0x3ef57744) <= 1, "MPFR f32::sin");
    assert!(float::bits(math::cos(0.5f32)).abs_diff(0x3f60a940) <= 1, "MPFR f32::cos");
    assert!(float::bits(math::tan(0.5f32)).abs_diff(0x3f0bda7b) <= 1, "MPFR f32::tan");
    assert!(float::bits(math::asin(0.5f32)).abs_diff(0x3f060a92) <= 1, "MPFR f32::asin");
    assert!(float::bits(math::acos(0.5f32)).abs_diff(0x3f860a92) <= 1, "MPFR f32::acos");
    assert!(float::bits(math::atan(0.5f32)).abs_diff(0x3eed6338) <= 1, "MPFR f32::atan");
    assert!(float::bits(math::sinh(0.5f32)).abs_diff(0x3f056680) <= 2, "MPFR f32::sinh");
    assert!(float::bits(math::cosh(0.5f32)).abs_diff(0x3f90560c) <= 1, "MPFR f32::cosh");
    assert!(float::bits(math::tanh(0.5f32)).abs_diff(0x3eec9a9f) <= 2, "MPFR f32::tanh");
    assert!(float::bits(math::asinh(0.5f32)).abs_diff(0x3ef66165) <= 2, "MPFR f32::asinh");
    assert!(float::bits(math::acosh(1.5f32)).abs_diff(0x3f766165) <= 4, "MPFR f32::acosh");
    assert!(float::bits(math::atanh(0.5f32)).abs_diff(0x3f0c9f54) <= 2, "MPFR f32::atanh");
    assert!(float::bits(math::exp(0.5f32)).abs_diff(0x3fd3094c) <= 1, "MPFR f32::exp");
    assert!(float::bits(math::exp2(0.5f32)).abs_diff(0x3fb504f3) <= 1, "MPFR f32::exp2");
    assert!(float::bits(math::exp_m1(0.5f32)).abs_diff(0x3f261299) <= 1, "MPFR f32::exp_m1");
    assert!(float::bits(math::ln(0.5f32)).abs_diff(0xbf317218) <= 1, "MPFR f32::ln");
    assert!(float::bits(math::ln_1p(0.5f32)).abs_diff(0x3ecf991f) <= 1, "MPFR f32::ln_1p");
    assert!(float::bits(math::log2(0.5f32)).abs_diff(0xbf800000) <= 1, "MPFR f32::log2");
    assert!(float::bits(math::log10(0.5f32)).abs_diff(0xbe9a209b) <= 1, "MPFR f32::log10");
    assert!(float::bits(math::cbrt(0.5f32)).abs_diff(0x3f4b2ff5) <= 0, "MPFR f32::cbrt");
    assert!(float::bits(math::powf(1.5f32, 0.75f32)).abs_diff(0x3fad7dd8) <= 1, "MPFR f32::powf");
    assert!(float::bits(math::hypot(1.25f32, 0.75f32)).abs_diff(0x3fba9728) <= 1, "MPFR f32::hypot");
    assert!(float::bits(math::atan2(1.25f32, -0.75f32)).abs_diff(0x40071e29) <= 2, "MPFR f32::atan2");
    assert!(float::bits(math::sin(1e+20f32)).abs_diff(0x3f281569) <= 1, "MPFR f32::sin");
    assert!(float::bits(math::sin_cos(0.5f32).0).abs_diff(0x3ef57744) <= 1, "MPFR f32::sin_cos.0");
    assert!(float::bits(math::sin_cos(0.5f32).1).abs_diff(0x3f60a940) <= 1, "MPFR f32::sin_cos.1");
    // Exact rational formulas: fract=x-trunc(x), q=floor(x/r), r=x-q*r,
    // midpoint=(a+b)/2, clamp=min(max(x,lo),hi), powi=x^n, log_2(8)=3.
    assert_eq!(float::bits(math::fract(-1.75f32)), 0xbf400000, "exact f32::fract");
    assert_eq!(float::bits(math::recip(0.5f32)), 0x40000000, "exact f32::recip");
    assert_eq!(float::bits(math::div_euclid(-7.0f32, 4.0)), 0xc0000000, "exact f32::div_euclid");
    assert_eq!(float::bits(math::rem_euclid(-7.0f32, 4.0)), 0x3f800000, "exact f32::rem_euclid");
    assert_eq!(float::bits(math::midpoint(1.25f32, 0.75)), 0x3f800000, "exact f32::midpoint");
    assert_eq!(float::bits(math::clamp(1.5f32, 0.0, 1.0)), 0x3f800000, "exact f32::clamp");
    assert_eq!(float::bits(math::powi(2.0f32, -3)), 0x3e000000, "exact f32::powi");
    assert_eq!(float::bits(math::powi(10.0f32, -2)), 0x3c23d70a, "exact f32::powi");
    assert_eq!(float::bits(math::round_ties_even(2.5f32)), 0x40000000, "exact f32::round_ties_even");
    assert_eq!(float::bits(math::abs_sub(1.25f32, 0.75)), 0x3f000000, "exact f32::abs_sub");
    assert_eq!(float::bits(math::log(8.0f32, 2.0)), 0x40400000, "exact f32::log");
    // Angle conversions: x * rounded(180/pi), x * rounded(pi/180).
    assert_eq!(float::bits(math::to_degrees(0.5f32)), 0x41e52ee1, "formula f32::to_degrees");
    assert_eq!(float::bits(math::to_radians(30.0f32)), 0x3f060a92, "formula f32::to_radians");
    assert!(float::bits(math::sin(0.5f64)).abs_diff(0x3fdeaee8744b05f0) <= 1, "MPFR f64::sin");
    assert!(float::bits(math::cos(0.5f64)).abs_diff(0x3fec1528065b7d50) <= 1, "MPFR f64::cos");
    assert!(float::bits(math::tan(0.5f64)).abs_diff(0x3fe17b4f5bf3474a) <= 1, "MPFR f64::tan");
    assert!(float::bits(math::asin(0.5f64)).abs_diff(0x3fe0c152382d7366) <= 1, "MPFR f64::asin");
    assert!(float::bits(math::acos(0.5f64)).abs_diff(0x3ff0c152382d7366) <= 1, "MPFR f64::acos");
    assert!(float::bits(math::atan(0.5f64)).abs_diff(0x3fddac670561bb4f) <= 1, "MPFR f64::atan");
    assert!(float::bits(math::sinh(0.5f64)).abs_diff(0x3fe0acd00fe63b97) <= 2, "MPFR f64::sinh");
    assert!(float::bits(math::cosh(0.5f64)).abs_diff(0x3ff20ac1862ae8d0) <= 1, "MPFR f64::cosh");
    assert!(float::bits(math::tanh(0.5f64)).abs_diff(0x3fdd9353d7568af3) <= 2, "MPFR f64::tanh");
    assert!(float::bits(math::asinh(0.5f64)).abs_diff(0x3fdecc2caec5160a) <= 2, "MPFR f64::asinh");
    assert!(float::bits(math::acosh(1.5f64)).abs_diff(0x3feecc2caec5160a) <= 4, "MPFR f64::acosh");
    assert!(float::bits(math::atanh(0.5f64)).abs_diff(0x3fe193ea7aad030b) <= 2, "MPFR f64::atanh");
    assert!(float::bits(math::exp(0.5f64)).abs_diff(0x3ffa61298e1e069c) <= 1, "MPFR f64::exp");
    assert!(float::bits(math::exp2(0.5f64)).abs_diff(0x3ff6a09e667f3bcd) <= 1, "MPFR f64::exp2");
    assert!(float::bits(math::exp_m1(0.5f64)).abs_diff(0x3fe4c2531c3c0d38) <= 1, "MPFR f64::exp_m1");
    assert!(float::bits(math::ln(0.5f64)).abs_diff(0xbfe62e42fefa39ef) <= 1, "MPFR f64::ln");
    assert!(float::bits(math::ln_1p(0.5f64)).abs_diff(0x3fd9f323ecbf984c) <= 1, "MPFR f64::ln_1p");
    assert!(float::bits(math::log2(0.5f64)).abs_diff(0xbff0000000000000) <= 1, "MPFR f64::log2");
    assert!(float::bits(math::log10(0.5f64)).abs_diff(0xbfd34413509f79ff) <= 1, "MPFR f64::log10");
    assert!(float::bits(math::cbrt(0.5f64)).abs_diff(0x3fe965fea53d6e3d) <= 0, "MPFR f64::cbrt");
    assert!(float::bits(math::powf(1.5f64, 0.75f64)).abs_diff(0x3ff5afbb0fd2812c) <= 1, "MPFR f64::powf");
    assert!(float::bits(math::hypot(1.25f64, 0.75f64)).abs_diff(0x3ff752e50db3a3a2) <= 1, "MPFR f64::hypot");
    assert!(float::bits(math::atan2(1.25f64, -0.75f64)).abs_diff(0x4000e3c51fa059c4) <= 2, "MPFR f64::atan2");
    assert!(float::bits(math::sin(1e+20f64)).abs_diff(0xbfe4a5e605fd6450) <= 1, "MPFR f64::sin");
    assert!(float::bits(math::exp(0.019f64)).abs_diff(0x3ff04e916a97ec0f) <= 1, "MPFR f64::exp");
    assert!(float::bits(math::exp2(16.077f64)).abs_diff(0x40f0e08d5dae5f52) <= 1, "MPFR f64::exp2");
    assert!(float::bits(math::sin_cos(0.5f64).0).abs_diff(0x3fdeaee8744b05f0) <= 1, "MPFR f64::sin_cos.0");
    assert!(float::bits(math::sin_cos(0.5f64).1).abs_diff(0x3fec1528065b7d50) <= 1, "MPFR f64::sin_cos.1");
    // Exact rational formulas: fract=x-trunc(x), q=floor(x/r), r=x-q*r,
    // midpoint=(a+b)/2, clamp=min(max(x,lo),hi), powi=x^n, log_2(8)=3.
    assert_eq!(float::bits(math::fract(-1.75f64)), 0xbfe8000000000000, "exact f64::fract");
    assert_eq!(float::bits(math::recip(0.5f64)), 0x4000000000000000, "exact f64::recip");
    assert_eq!(float::bits(math::div_euclid(-7.0f64, 4.0)), 0xc000000000000000, "exact f64::div_euclid");
    assert_eq!(float::bits(math::rem_euclid(-7.0f64, 4.0)), 0x3ff0000000000000, "exact f64::rem_euclid");
    assert_eq!(float::bits(math::midpoint(1.25f64, 0.75)), 0x3ff0000000000000, "exact f64::midpoint");
    assert_eq!(float::bits(math::clamp(1.5f64, 0.0, 1.0)), 0x3ff0000000000000, "exact f64::clamp");
    assert_eq!(float::bits(math::powi(2.0f64, -3)), 0x3fc0000000000000, "exact f64::powi");
    assert_eq!(float::bits(math::powi(10.0f64, -2)), 0x3f847ae147ae147b, "exact f64::powi");
    assert_eq!(float::bits(math::round_ties_even(2.5f64)), 0x4000000000000000, "exact f64::round_ties_even");
    assert_eq!(float::bits(math::abs_sub(1.25f64, 0.75)), 0x3fe0000000000000, "exact f64::abs_sub");
    assert_eq!(float::bits(math::log(8.0f64, 2.0)), 0x4008000000000000, "exact f64::log");
    // Angle conversions: x * rounded(180/pi), x * rounded(pi/180).
    assert_eq!(float::bits(math::to_degrees(0.5f64)), 0x403ca5dc1a63c1f8, "formula f64::to_degrees");
    assert_eq!(float::bits(math::to_radians(30.0f64)), 0x3fe0c152382d7365, "formula f64::to_radians");
}
// END independent oracle

#[tide::main]
fn main() {
    f32_contract();
    f64_contract();
    println!("FLOAT_MATH_RAN");
}

#[test]
fn float_math_contract() {
    f32_contract();
    f64_contract();
}

#[test]
#[should_panic]
fn clamp_rejects_nan_bound() {
    math::clamp(0.0f64, f64::NAN, 1.0);
}
