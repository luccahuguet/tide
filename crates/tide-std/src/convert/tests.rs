#![allow(unstable_name_collisions)]
use super::*;

#[test]
fn truncate_keeps_twos_complement_low_bits() {
    let low: u16 = 0x1234_5678u32.truncate();
    assert_eq!(low, 0x5678);
    assert_eq!((-1i64).truncate::<i8>(), -1);
    assert_eq!(i16::MIN.truncate::<i8>(), 0);
    assert_eq!(300u16.truncate::<u8>(), 44);
    assert_eq!(u8::MAX.truncate::<u8>(), u8::MAX);
    assert_eq!(usize::MAX.truncate::<usize>(), usize::MAX);
    assert_eq!(usize::MAX.truncate::<u16>(), u16::MAX);
    assert_eq!(isize::MIN.truncate::<i8>(), 0);
}

#[test]
fn round_from_is_nearest_ties_even_and_can_overflow_to_infinity() {
    assert_eq!(
        f64::round_from(9_007_199_254_740_993u64),
        9_007_199_254_740_992.0
    );
    assert_eq!(
        f64::round_from(9_007_199_254_740_995u64),
        9_007_199_254_740_996.0
    );
    assert_eq!(f64::round_from(u64::MAX), 18_446_744_073_709_551_616.0);
    assert_eq!(f32::round_from(16_777_217i32), 16_777_216.0);
    assert_eq!(f32::round_from(16_777_219i32), 16_777_220.0);
    assert_eq!(f32::round_from(u128::MAX), f32::INFINITY);
    assert_eq!(f32::round_from(1e39_f64), f32::INFINITY);
    assert_eq!(f32::round_from(-1e39_f64), f32::NEG_INFINITY);
    assert!(f32::round_from(f64::NAN).is_nan());
    assert_eq!(f32::round_from(-0.0f64).to_bits(), (-0.0f32).to_bits());
    // Both even neighbors of a floating-point narrowing tie, including negative values.
    assert_eq!(f32::round_from(1.0f64 + 2.0f64.powi(-24)), 1.0);
    assert_eq!(
        f32::round_from(1.0f64 + 3.0 * 2.0f64.powi(-24)),
        f32::from_bits(0x3f800002)
    );
    assert_eq!(f32::round_from(-16_777_217i32), -16_777_216.0);
}

#[test]
fn try_from_float_checks_exact_exclusive_bounds() {
    let bound = 9_223_372_036_854_775_808.0f64;
    assert_eq!(i64::try_from_float(-bound), Ok(i64::MIN));
    assert_eq!(
        i64::try_from_float(f64::from_bits(bound.to_bits() - 1)),
        Ok(i64::MAX - 1023)
    );
    for value in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        bound,
        f64::from_bits((-bound).to_bits() + 1),
        0.5,
        -0.5,
        f64::from_bits(1),
        -f64::from_bits(1),
    ] {
        assert_eq!(i64::try_from_float(value), Err(TryFromFloatError));
    }
    for value in [-1.0f64, 256.0, 0.5] {
        assert_eq!(u8::try_from_float(value), Err(TryFromFloatError));
    }
    assert_eq!(
        i64::try_from_float(f32::from_bits(0x8000_0001)),
        Err(TryFromFloatError)
    );
    assert_eq!(u8::try_from_float(-0.0f64), Ok(0));
    assert_eq!(u8::try_from_float(255.0f64), Ok(255));
    assert_eq!(i64::try_from_float(1.75f64.trunc()), Ok(1));
}

#[test]
fn every_float_integer_pair_rejects_non_finite_and_range_overflow() {
    macro_rules! pairs {
        ($f:ty; $($i:ty),+ $(,)?) => {$(
            assert_eq!(<$i>::try_from_float(-0.0 as $f), Ok(0));
            assert_eq!(<$i>::try_from_float(1.0 as $f), Ok(1));
            assert_eq!(<$i>::try_from_float(<$i>::MIN as $f), Ok(<$i>::MIN));
            for value in [<$f>::NAN, <$f>::INFINITY, <$f>::NEG_INFINITY, 0.5,
                (<$i>::MAX as f64 + 1.0) as $f] {
                assert_eq!(<$i>::try_from_float(value), Err(TryFromFloatError));
            }
        )+};
    }
    pairs!(f32; i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
    pairs!(f64; i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
    assert_eq!(
        u128::try_from_float(f32::MAX),
        Ok(u128::MAX - ((1u128 << 104) - 1))
    );
    assert_eq!(
        i128::try_from_float(f32::from_bits(0x7effffff)),
        Ok(i128::MAX - ((1i128 << 103) - 1))
    );
    assert_eq!(
        u128::try_from_float(f64::from_bits(0x47efffffffffffff)),
        Ok(u128::MAX - ((1u128 << 75) - 1))
    );
    assert_eq!(
        i128::try_from_float(f64::from_bits(0x47dfffffffffffff)),
        Ok(i128::MAX - ((1i128 << 74) - 1))
    );
}

#[test]
fn conversion_error_has_the_documented_traits() {
    fn traits<T: core::error::Error + Clone + Copy + PartialEq + Eq>() {}
    traits::<TryFromFloatError>();
    assert!(!TryFromFloatError.to_string().is_empty());
}
