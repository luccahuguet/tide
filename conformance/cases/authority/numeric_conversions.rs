//@ rules: REF-0301, ADMIT-1
// The reference's §4.2 spellings, shared with the production-std executing test.
fn conversions() -> Result<(), Box<dyn Error>> {
    let low: u8 = 300u16.truncate();
    assert_eq!(low, 44);
    assert_eq!(0x1234_5678u32.truncate::<u16>(), 0x5678);
    assert_eq!((-1i64).truncate::<i8>(), -1);
    assert_eq!(i16::MIN.truncate::<i8>(), 0);
    assert_eq!((-1i32).cast_unsigned(), u32::MAX);
    assert_eq!(u8::MAX.cast_signed(), -1);
    assert_eq!((-1i32).cast_unsigned().truncate::<u8>(), u8::MAX);
    assert_eq!(u8::try_from(44u16)?, 44);

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

    assert_eq!(i64::try_from_float(1.75f64.trunc())?, 1);
    assert_eq!(u8::try_from_float(-0.0f32)?, 0);
    assert_eq!(u8::try_from_float(255.0f64)?, 255);
    let bound = 9_223_372_036_854_775_808.0f64;
    assert_eq!(i64::try_from_float(-bound)?, i64::MIN);
    assert_eq!(
        i64::try_from_float(f64::from_bits(0x43dfffffffffffff))?,
        i64::MAX - 1023
    );
    for value in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        bound,
        f64::from_bits(0xc3e0000000000001),
        0.5,
    ] {
        assert_eq!(
            i64::try_from_float(value),
            Err(std::convert::TryFromFloatError)
        );
    }
    for value in [-1.0f32, 256.0] {
        assert!(u8::try_from_float(value).is_err());
    }
    Ok(())
}

#[tide::main]
fn main() -> Result<(), Box<dyn Error>> {
    conversions()?;
    println!("NUMERIC_CONVERSIONS_RAN");
    Ok(())
}

#[test]
fn numeric_conversion_spellings_execute() -> Result<(), Box<dyn Error>> {
    conversions()
}
