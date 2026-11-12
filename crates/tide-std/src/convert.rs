//! Explicit numeric conversions (reference §4.2), shared with the conformance std.
pub use std::convert::*;

mod sealed {
    pub trait Integer {}
    pub trait TruncateTarget<T> {
        fn low_bits(self) -> T;
    }
    pub trait RoundFrom<T> {}
    pub trait TryFromFloat<F> {}
}

/// Keep the low bits of an integer, with the target inferred or named by `truncate::<T>()`.
///
/// Targets have the same signedness: each fixed-width integer supports itself and narrower
/// fixed-width integers; `usize`/`isize` support themselves and their 16- and 8-bit counterparts.
/// This matches core's unstable `truncate` pair set. The conversion never fails.
pub trait Truncate: sealed::Integer + Sized {
    #[must_use]
    fn truncate<T>(self) -> T
    where
        Self: sealed::TruncateTarget<T>,
    {
        self.low_bits()
    }
}

/// Convert to a float, rounding to nearest with ties to even. Never fails or panics.
///
/// `f64` accepts `i64`, `u64`, `i128`, `u128`, `isize`, and `usize`; `f32` additionally
/// accepts `i32`, `u32`, and `f64`. Exact conversions use `From` instead.
/// `f32::round_from(u128::MAX)` and `f32::round_from(1e39_f64)` yield positive infinity.
/// NaN stays NaN; its payload is unspecified.
pub trait RoundFrom<T>: sealed::RoundFrom<T> + Sized {
    #[must_use]
    fn round_from(value: T) -> Self;
}

/// Convert a finite, integral, in-range `f32` or `f64` to any integer; `-0.0` becomes zero.
///
/// Fractional values, NaN, infinities, and out-of-range values return an error. To discard a
/// fractional part explicitly, use `i64::try_from_float(value.trunc())?`.
pub trait TryFromFloat<F>: sealed::TryFromFloat<F> + Sized {
    fn try_from_float(value: F) -> Result<Self, TryFromFloatError>;
}

/// The float was non-finite, fractional, or outside the target integer's range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TryFromFloatError;

impl core::fmt::Display for TryFromFloatError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("float is not a finite, integral value in the target integer's range")
    }
}
impl core::error::Error for TryFromFloatError {}

macro_rules! truncate_pairs {
    ($($from:ty => $($to:ty),+;)*) => {$(
        impl sealed::Integer for $from {}
        impl Truncate for $from {}
        $(impl sealed::TruncateTarget<$to> for $from {
            fn low_bits(self) -> $to { self as $to }
        })+
    )*};
}
// Mirror core's integer_widen_truncate pairs so stabilization keeps method lookup compatible.
truncate_pairs! {
    u8 => u8;
    u16 => u16, u8;
    u32 => u32, u16, u8;
    u64 => u64, u32, u16, u8;
    u128 => u128, u64, u32, u16, u8;
    usize => usize, u16, u8;
    i8 => i8;
    i16 => i16, i8;
    i32 => i32, i16, i8;
    i64 => i64, i32, i16, i8;
    i128 => i128, i64, i32, i16, i8;
    isize => isize, i16, i8;
}

macro_rules! round_pairs {
    ($to:ty; $($from:ty),+ $(,)?) => {$(
        impl sealed::RoundFrom<$from> for $to {}
        impl RoundFrom<$from> for $to {
            fn round_from(value: $from) -> Self { value as Self }
        }
    )+};
}
round_pairs!(f64; i64, u64, i128, u128, isize, usize);
round_pairs!(f32; i32, u32, i64, u64, i128, u128, isize, usize, f64);

macro_rules! float_integer_pairs {
    ($f:ty; $($i:ty),+ $(,)?) => {$(
        impl sealed::TryFromFloat<$f> for $i {}
        impl TryFromFloat<$f> for $i {
            fn try_from_float(value: $f) -> Result<Self, TryFromFloatError> {
                // MAX + 1 in f64 gives the exact exclusive power-of-two bound; f32/u128
                // becomes +inf, safe because non-finite input is rejected first.
                let upper = (<$i>::MAX as f64 + 1.0) as $f;
                if value.is_finite() && value.trunc() == value
                    && value >= <$i>::MIN as $f && value < upper {
                    Ok(value as Self)
                } else {
                    Err(TryFromFloatError)
                }
            }
        }
    )+};
}
float_integer_pairs!(f32; i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
float_integer_pairs!(f64; i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

#[cfg(test)]
#[path = "convert/tests.rs"]
mod tests;
