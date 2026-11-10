//! Deterministic float observers (reference §4.3). Every NaN is observed as the same positive
//! quiet NaN. Signed zeros remain distinct, with -0.0 below +0.0.
use core::cmp::Ordering;

#[cfg(all(target_arch = "x86", not(target_feature = "sse2")))]
compile_error!("Tide requires IEEE 754 binary32/binary64 native arithmetic; x86 without SSE2 uses x87 extended precision and is unsupported");

mod sealed {
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
}

/// The two IEEE float types supported by Tide. Sealed so observers cannot acquire authority.
pub trait Float: sealed::Sealed + Copy {
    type Bits: Copy + Eq + Ord + core::hash::Hash + core::fmt::Debug;
    #[doc(hidden)]
    fn canonical(self) -> Self;
    #[doc(hidden)]
    fn bits(self) -> Self::Bits;
    #[doc(hidden)]
    fn min(self, other: Self) -> Self;
    #[doc(hidden)]
    fn max(self, other: Self) -> Self;
    #[doc(hidden)]
    fn total_cmp(self, other: Self) -> Ordering;
}

/// IEEE bits, with every NaN mapped to 0x7fc00000 (f32) or 0x7ff8000000000000 (f64).
pub fn bits<T: Float>(x: T) -> T::Bits {
    x.bits()
}

/// The lesser value, ignoring a sole NaN. Two NaNs yield the canonical NaN; -0.0 < +0.0.
pub fn min<T: Float>(a: T, b: T) -> T {
    a.min(b)
}

/// The greater value, ignoring a sole NaN. Two NaNs yield the canonical NaN; -0.0 < +0.0.
pub fn max<T: Float>(a: T, b: T) -> T {
    a.max(b)
}

/// Total order: -infinity < negative finite < -0.0 < +0.0 < positive finite < +infinity < NaN.
/// Every NaN compares equal, regardless of its original sign, payload, or signaling bit.
pub fn total_cmp<T: Float>(a: T, b: T) -> Ordering {
    a.total_cmp(b)
}

macro_rules! floats {
    ($($t:ident, $bits:ty, $nan:expr);* $(;)?) => {$(
        impl Float for $t {
            type Bits = $bits;
            fn canonical(self) -> Self {
                if self.is_nan() { Self::from_bits($nan) } else { self }
            }
            fn bits(self) -> Self::Bits {
                Self::to_bits(self.canonical())
            }
            fn min(self, other: Self) -> Self {
                let (a, b) = (self.canonical(), other.canonical());
                if a.is_nan() || Self::total_cmp(&a, &b).is_gt() { b } else { a }
            }
            fn max(self, other: Self) -> Self {
                let (a, b) = (self.canonical(), other.canonical());
                if a.is_nan() || (!b.is_nan() && Self::total_cmp(&a, &b).is_lt()) { b } else { a }
            }
            fn total_cmp(self, other: Self) -> Ordering {
                Self::total_cmp(&self.canonical(), &other.canonical())
            }
        }
    )*};
}
floats! { f32, u32, 0x7fc00000; f64, u64, 0x7ff8000000000000; }
