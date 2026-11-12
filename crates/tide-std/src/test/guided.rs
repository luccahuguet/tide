//! Constructive choices for the test macro's recognized requires shapes.
use super::{Arbitrary, Gen, Kind};

/// Ordered scalar encoding used only by generated test code.
#[doc(hidden)]
pub trait Scalar: Arbitrary + Copy {
    fn key(self) -> u128;
    fn from_key(key: u128) -> Self;
    const MIN_KEY: u128 = 0;
    const MAX_KEY: u128;
}
macro_rules! integers {
    ($($unsigned:ty => $signed:ty),* $(,)?) => {$ (
        impl Scalar for $unsigned {
            fn key(self) -> u128 { self as u128 }
            fn from_key(key: u128) -> Self { key as Self }
            const MAX_KEY: u128 = <$unsigned>::MAX as u128;
        }
        impl Scalar for $signed {
            fn key(self) -> u128 { ((self as $unsigned) ^ (1 << (Self::BITS - 1))) as u128 }
            fn from_key(key: u128) -> Self { ((key as $unsigned) ^ (1 << (Self::BITS - 1))) as Self }
            const MAX_KEY: u128 = <$unsigned>::MAX as u128;
        }
    )*};
}
integers!(u8 => i8, u16 => i16, u32 => i32, u64 => i64, u128 => i128, usize => isize);

// Float keys follow numeric order; NaNs stay outside the guided range.
macro_rules! floats {
    ($($float:ty => $bits:ty),*) => {$ (
        impl Scalar for $float {
            fn key(self) -> u128 {
                let bits = self.to_bits();
                if bits & (1 << (<$bits>::BITS - 1)) != 0 { (!bits) as u128 }
                else { (bits ^ (1 << (<$bits>::BITS - 1))) as u128 }
            }
            fn from_key(key: u128) -> Self {
                let key = key as $bits;
                Self::from_bits(if key & (1 << (<$bits>::BITS - 1)) != 0 {
                    key ^ (1 << (<$bits>::BITS - 1))
                } else { !key })
            }
            const MIN_KEY: u128 = (!Self::NEG_INFINITY.to_bits()) as u128;
            const MAX_KEY: u128 = (Self::INFINITY.to_bits() ^ (1 << (<$bits>::BITS - 1))) as u128;
        }
    )*};
}
floats!(f32 => u32, f64 => u64);

/// Intersect bounds before drawing. An impossible intersection keeps ordinary generation.
#[doc(hidden)]
pub fn range<T: Scalar>(g: &mut Gen, lower: &[(T, bool)], upper: &[(T, bool)]) -> T {
    let mut lo = T::MIN_KEY;
    let mut hi = T::MAX_KEY;
    for &(value, inclusive) in lower {
        let Some(key) = value.key().checked_add(u128::from(!inclusive)) else {
            return T::arbitrary(g);
        };
        lo = lo.max(key);
    }
    for &(value, inclusive) in upper {
        let Some(key) = value.key().checked_sub(u128::from(!inclusive)) else {
            return T::arbitrary(g);
        };
        hi = hi.min(key);
    }
    if lo > hi || lo > T::MAX_KEY {
        return T::arbitrary(g);
    }
    let width = hi - lo;
    let offset = g.draw(Kind::Integer(T::MAX_KEY - T::MIN_KEY), |g| {
        let random = (u128::from(g.next()) << 64) | u128::from(g.next());
        match g.next() % 4 {
            0 => 0,
            1 => width,
            _ => width.checked_add(1).map_or(random, |size| random % size),
        }
    });
    T::from_key(lo + offset.min(width))
}

/// Construct a bounded sequence without retrying an arbitrary value.
#[doc(hidden)]
pub trait Sequence: Arbitrary {
    fn with_bounds(g: &mut Gen, lo: usize, hi: usize) -> Self;
}
impl<T: Arbitrary> Sequence for Vec<T> {
    fn with_bounds(g: &mut Gen, lo: usize, hi: usize) -> Self {
        bounded_vec(g, lo, hi, &mut T::arbitrary)
    }
}
impl Sequence for String {
    fn with_bounds(g: &mut Gen, lo: usize, hi: usize) -> Self {
        // ASCII makes the clause's byte length equal the constructed length.
        let mut text: String = (0..lo).map(|_| char::from(g.index(128) as u8)).collect();
        while text.len() < hi && g.draw(Kind::More, |g| (g.next() % 4 != 0) as u128) != 0 {
            text.push(char::from(g.index(128) as u8));
        }
        text
    }
}
#[doc(hidden)]
pub fn sequence<T: Sequence>(g: &mut Gen, lower: &[(usize, bool)], upper: &[(usize, bool)]) -> T {
    match sequence_bounds(lower, upper) {
        Some((lo, hi)) => T::with_bounds(g, lo, hi),
        None => T::arbitrary(g),
    }
}

#[doc(hidden)]
pub fn sequence_with<T>(
    g: &mut Gen,
    lower: &[(usize, bool)],
    upper: &[(usize, bool)],
    mut draw: impl FnMut(&mut Gen) -> T,
) -> Vec<T> {
    match sequence_bounds(lower, upper) {
        Some((lo, hi)) => bounded_vec(g, lo, hi, &mut draw),
        None => g.collection_with(draw),
    }
}
fn bounded_vec<T>(
    g: &mut Gen,
    lo: usize,
    hi: usize,
    draw: &mut impl FnMut(&mut Gen) -> T,
) -> Vec<T> {
    let mut values: Vec<T> = (0..lo).map(|_| draw(g)).collect();
    while values.len() < hi && g.draw(Kind::More, |g| (g.next() % 4 != 0) as u128) != 0 {
        values.push(draw(g));
    }
    values
}
fn sequence_bounds(lower: &[(usize, bool)], upper: &[(usize, bool)]) -> Option<(usize, usize)> {
    let mut lo = 0;
    let mut hi = usize::MAX;
    for &(value, inclusive) in lower {
        lo = lo.max(value.checked_add(usize::from(!inclusive))?);
    }
    for &(value, inclusive) in upper {
        hi = hi.min(value.checked_sub(usize::from(!inclusive))?);
    }
    // Keep constructive allocation bounded even for hostile or contradictory clauses.
    (lo <= hi && lo <= 4096).then_some((lo, hi.min(lo.max(32))))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounds_cover_signed_extremes_and_empty_intersections_without_overflow() {
        for seed in 0..100 {
            let mut g = Gen::new(seed, None);
            assert_eq!(
                range(&mut g, &[(i128::MIN, true)], &[(i128::MIN, true)]),
                i128::MIN
            );
            assert_eq!(range(&mut g, &[(u128::MAX, true)], &[]), u128::MAX);
            let n: i8 = range(&mut g, &[(-5, false)], &[(3, false)]);
            assert!((-4..3).contains(&n));
            let _: u8 = range(&mut g, &[(255, false)], &[]);
            let _: u8 = range(&mut g, &[], &[(0, false)]);
            let xs: Vec<u16> = sequence(&mut g, &[(17, true)], &[(17, true)]);
            assert_eq!(xs.len(), 17);
            let text: String = sequence(&mut g, &[(7, true)], &[(7, true)]);
            assert_eq!(text.len(), 7);
        }
    }
}
