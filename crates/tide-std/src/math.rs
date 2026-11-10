//! Deterministic math (D3, reference §4.3), using pinned libm with `force-soft-floats` on every
//! supported target. NaN inputs/results are canonicalized; signed zeros are retained. Accuracy is libm's,
//! not a promise of correct rounding for transcendental functions. Kept methods such as `sqrt`
//! and `mul_add` remain native IEEE operations.
//! Midpoint, angle conversions and clamp compose basic IEEE operations (native arithmetic,
//! deterministic under D3), rather than calling libm. Non-SSE2 x86 is rejected by `float`.
use crate::float::Float as _;

macro_rules! functions {
    ($($name:ident($($arg:ident),*) => $kernel:ident;)*) => {
        /// Math dispatch for f32 and f64, sealed by `float::Float`.
        pub trait Float: crate::float::Float {
            $(#[doc(hidden)] fn $name(self, $($arg: Self),*) -> Self;)*
            #[doc(hidden)] fn powi(self, n: i32) -> Self;
            #[doc(hidden)] fn sin_cos(self) -> (Self, Self);
            #[doc(hidden)] fn log(self, base: Self) -> Self;
            #[doc(hidden)] fn fract(self) -> Self;
            #[doc(hidden)] fn recip(self) -> Self;
            #[doc(hidden)] fn to_degrees(self) -> Self;
            #[doc(hidden)] fn to_radians(self) -> Self;
            #[doc(hidden)] fn div_euclid(self, rhs: Self) -> Self;
            #[doc(hidden)] fn rem_euclid(self, rhs: Self) -> Self;
            #[doc(hidden)] fn midpoint(self, rhs: Self) -> Self;
            #[doc(hidden)] fn clamp(self, min: Self, max: Self) -> Self;
        }
        $(pub fn $name<T: Float>(x: T, $($arg: T),*) -> T {
            x.$name($($arg),*)
        })*
        implement!(f32; $($name($($arg),*) => $kernel;)*);
        implement!(f64; $($name($($arg),*) => $kernel;)*);
    };
}

macro_rules! implement {
    ($t:ident; $($name:ident($($arg:ident),*) => $kernel:ident;)*) => {
        impl Float for $t {
            $(fn $name(self, $($arg: Self),*) -> Self {
                libm::Libm::<Self>::$kernel(self.canonical(), $($arg.canonical()),*).canonical()
            })*
            fn powi(self, n: i32) -> Self {
                // Match compiler-builtins: square and multiply, then take the reciprocal.
                let mut base = self.canonical();
                let mut power = n.unsigned_abs();
                let mut result = 1.0;
                while power != 0 {
                    if power & 1 != 0 { result *= base; }
                    power >>= 1;
                    if power != 0 { base *= base; }
                }
                (if n < 0 { 1.0 / result } else { result }).canonical()
            }
            fn sin_cos(self) -> (Self, Self) {
                let (s, c) = libm::Libm::<Self>::sincos(self.canonical());
                (s.canonical(), c.canonical())
            }
            fn log(self, base: Self) -> Self {
                (libm::Libm::<Self>::log(self.canonical()) /
                 libm::Libm::<Self>::log(base.canonical())).canonical()
            }
            fn fract(self) -> Self {
                let x = self.canonical();
                (x - libm::Libm::<Self>::trunc(x)).canonical()
            }
            fn recip(self) -> Self { (1.0 / self.canonical()).canonical() }
            fn to_degrees(self) -> Self { self.canonical().to_degrees().canonical() }
            fn to_radians(self) -> Self { self.canonical().to_radians().canonical() }
            fn div_euclid(self, rhs: Self) -> Self {
                let (x, rhs) = (self.canonical(), rhs.canonical());
                let q = libm::Libm::<Self>::trunc(x / rhs);
                let r = libm::Libm::<Self>::fmod(x, rhs);
                (if r < 0.0 { if rhs > 0.0 { q - 1.0 } else { q + 1.0 } } else { q }).canonical()
            }
            fn rem_euclid(self, rhs: Self) -> Self {
                let rhs = rhs.canonical();
                let r = libm::Libm::<Self>::fmod(self.canonical(), rhs);
                (if r < 0.0 { r + rhs.abs() } else { r }).canonical()
            }
            fn midpoint(self, rhs: Self) -> Self {
                self.canonical().midpoint(rhs.canonical()).canonical()
            }
            fn clamp(self, min: Self, max: Self) -> Self {
                self.canonical().clamp(min.canonical(), max.canonical()).canonical()
            }
        }
    };
}

functions! {
    sin() => sin;
    cos() => cos;
    tan() => tan;
    asin() => asin;
    acos() => acos;
    atan() => atan;
    sinh() => sinh;
    cosh() => cosh;
    tanh() => tanh;
    asinh() => asinh;
    acosh() => acosh;
    atanh() => atanh;
    exp() => exp;
    exp2() => exp2;
    exp_m1() => expm1;
    ln() => log;
    ln_1p() => log1p;
    log2() => log2;
    log10() => log10;
    cbrt() => cbrt;
    round_ties_even() => roundeven;
    powf(n) => pow;
    hypot(other) => hypot;
    atan2(other) => atan2;
    abs_sub(other) => fdim;
}

/// Integer power with a fixed exponentiation-by-squaring order (including i32::MIN).
pub fn powi<T: Float>(x: T, n: i32) -> T {
    x.powi(n)
}
/// Sine and cosine, in that order.
pub fn sin_cos<T: Float>(x: T) -> (T, T) {
    x.sin_cos()
}
/// Logarithm in `base`, computed as ln(x) / ln(base).
pub fn log<T: Float>(x: T, base: T) -> T {
    x.log(base)
}
/// Fractional part, computed as x - trunc(x).
pub fn fract<T: Float>(x: T) -> T {
    x.fract()
}
/// Multiplicative inverse.
pub fn recip<T: Float>(x: T) -> T {
    x.recip()
}
/// Convert radians to degrees.
pub fn to_degrees<T: Float>(x: T) -> T {
    x.to_degrees()
}
/// Convert degrees to radians.
pub fn to_radians<T: Float>(x: T) -> T {
    x.to_radians()
}
/// Truncated quotient, adjusted to give a nonnegative remainder.
pub fn div_euclid<T: Float>(x: T, rhs: T) -> T {
    x.div_euclid(rhs)
}
/// Least nonnegative remainder (which rounding may make equal to abs(rhs)).
pub fn rem_euclid<T: Float>(x: T, rhs: T) -> T {
    x.rem_euclid(rhs)
}
/// Average, avoiding overflow of the intermediate sum.
pub fn midpoint<T: Float>(x: T, rhs: T) -> T {
    x.midpoint(rhs)
}
/// Restrict x to [min, max]. Panics if a bound is NaN or min > max.
pub fn clamp<T: Float>(x: T, min: T, max: T) -> T {
    x.clamp(min, max)
}
