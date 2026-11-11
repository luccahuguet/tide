use std::borrow::{Borrow, BorrowMut};
use std::hash::Hash;

use crate::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use super::{Gen, Kind};

/// Generates a property test input from reproducible choices (provisional).
pub trait Arbitrary: Sized {
    fn arbitrary(g: &mut Gen) -> Self;
}

/// Owned storage for a borrowed property test parameter.
#[doc(hidden)]
pub trait Borrowed {
    type Owned: Arbitrary + Borrow<Self> + BorrowMut<Self>;
}

impl<T: Arbitrary> Borrowed for T {
    type Owned = T;
}

impl Borrowed for str {
    type Owned = String;
}

impl<T: Arbitrary> Borrowed for [T] {
    type Owned = Vec<T>;
}

macro_rules! integers {
    ($($unsigned:ty => $signed:ty),* $(,)?) => {
        $(
            impl Arbitrary for $unsigned {
                fn arbitrary(g: &mut Gen) -> Self {
                    g.integer(Self::BITS, false) as Self
                }
            }

            impl Arbitrary for $signed {
                fn arbitrary(g: &mut Gen) -> Self {
                    let bits = g.integer(Self::BITS, true);
                    // Shrinking the encoding moves the magnitude toward zero.
                    ((bits >> 1) as Self) ^ -((bits & 1) as Self)
                }
            }
        )*
    };
}

integers!(u8 => i8, u16 => i16, u32 => i32, u64 => i64, u128 => i128, usize => isize);

macro_rules! float {
    ($float:ty, $bits:ty) => {
        impl Arbitrary for $float {
            fn arbitrary(g: &mut Gen) -> Self {
                let encode = |v: Self| v.to_bits().rotate_left(1) as u128;
                let bits = g.draw(Kind::Float(<$bits>::BITS as u8), |g| match g.next() % 12 {
                    0 => encode(0.0),
                    1 => encode(-0.0),
                    2 => encode(1.0),
                    3 => encode(-1.0),
                    4 => encode(Self::MAX),
                    5 => encode(Self::MIN),
                    6 => encode(Self::INFINITY),
                    7 => encode(Self::NEG_INFINITY),
                    8 => encode(Self::NAN),
                    _ => (g.next() as $bits) as u128,
                });
                Self::from_bits((bits as $bits).rotate_right(1))
            }
        }
    };
}

float!(f32, u32);
float!(f64, u64);

impl Arbitrary for bool {
    fn arbitrary(g: &mut Gen) -> Self {
        g.draw(Kind::Bool, |g| (g.next() & 1) as u128) != 0
    }
}

impl Arbitrary for char {
    fn arbitrary(g: &mut Gen) -> Self {
        let scalar = g.draw(Kind::Integer(0x10f7ff), |g| match g.next() % 4 {
            0 => 0,
            1 => 0x10f7ff,
            2 => g.next() as u128 % 128,
            _ => g.next() as u128 % 0x10f800,
        }) as u32;
        let code = if scalar < 0xd800 {
            scalar
        } else {
            scalar + 0x800
        };
        char::from_u32(code).unwrap()
    }
}

impl Arbitrary for String {
    fn arbitrary(g: &mut Gen) -> Self {
        g.collection::<char>().into_iter().collect()
    }
}

impl<T: Arbitrary> Arbitrary for Vec<T> {
    fn arbitrary(g: &mut Gen) -> Self {
        g.collection()
    }
}

impl<T: Arbitrary, const N: usize> Arbitrary for [T; N] {
    fn arbitrary(g: &mut Gen) -> Self {
        std::array::from_fn(|_| T::arbitrary(g))
    }
}

impl<T: Arbitrary> Arbitrary for Option<T> {
    fn arbitrary(g: &mut Gen) -> Self {
        bool::arbitrary(g).then(|| T::arbitrary(g))
    }
}

impl<T: Arbitrary, E: Arbitrary> Arbitrary for Result<T, E> {
    fn arbitrary(g: &mut Gen) -> Self {
        if bool::arbitrary(g) {
            Err(E::arbitrary(g))
        } else {
            Ok(T::arbitrary(g))
        }
    }
}

impl<K: Arbitrary + Eq + Hash, V: Arbitrary> Arbitrary for HashMap<K, V> {
    fn arbitrary(g: &mut Gen) -> Self {
        g.collection::<(K, V)>().into_iter().collect()
    }
}

impl<T: Arbitrary + Eq + Hash> Arbitrary for HashSet<T> {
    fn arbitrary(g: &mut Gen) -> Self {
        g.collection::<T>().into_iter().collect()
    }
}

impl<K: Arbitrary + Ord, V: Arbitrary> Arbitrary for BTreeMap<K, V> {
    fn arbitrary(g: &mut Gen) -> Self {
        g.collection::<(K, V)>().into_iter().collect()
    }
}

impl<T: Arbitrary + Ord> Arbitrary for BTreeSet<T> {
    fn arbitrary(g: &mut Gen) -> Self {
        g.collection::<T>().into_iter().collect()
    }
}

// The same tuple arities as Rust's Debug, which is needed for a test input.
macro_rules! tuples {
    () => {
        impl Arbitrary for () {
            fn arbitrary(_: &mut Gen) -> Self {}
        }
    };
    ($first:ident $(, $rest:ident)*) => {
        impl<$first: Arbitrary $(, $rest: Arbitrary)*> Arbitrary for ($first, $($rest,)*) {
            fn arbitrary(g: &mut Gen) -> Self {
                ($first::arbitrary(g), $($rest::arbitrary(g),)*)
            }
        }
        tuples!($($rest),*);
    };
}

tuples!(A, B, C, D, E, F, G, H, I, J, K, L);
