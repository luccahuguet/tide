//! tide_std: the D18 spike's facade std (throwaway).
//!
//! User crates are compiled with `--extern std=libtide_std.rlib`, so rustc's
//! std injection loads this crate as `std`. It re-exports the safe, pure parts
//! of Rust's std and adds Tide's capability, task, `Shared`, channel,
//! deterministic-hash, float, and contract surface. It confines names only:
//! `core`, `alloc`, and the real std's inherent methods stay reachable, and the
//! admission checker owns rejecting them (research/15, F2).

#![deny(unsafe_op_in_unsafe_fn)]

mod caps;

pub use caps::{fs, sync, task, time};

// Safe, pure parts of Rust's std, re-exported whole.
pub use std::{
    any, array, borrow, boxed, cell, char, clone, cmp, convert, default, error, f32, f64, fmt, iter, num, ops,
    option, rc, result, slice, str, string, vec,
};

/// Rust's macros that Tide keeps (reference 2.1). `debug_assert_eq` is an
/// Eon addition: orbit-protocol uses it, and reference 2.1 bans it (FX0103).
pub use std::{
    assert_eq, assert_ne, dbg, debug_assert, debug_assert_eq, eprint, eprintln, format, matches, print, println, todo, unimplemented,
    unreachable, write, writeln,
};

/// `panic!` as a macro only: re-exporting `std::panic` would also export the
/// module and `catch_unwind`.
#[macro_export]
macro_rules! panic {
    ($($t:tt)*) => { ::core::panic!($($t)*) };
}

pub mod prelude {
    pub mod rust_2024 {
        // Rust's 2024 prelude, minus the builtin macros Tide drops
        // (`env`, `option_env`, `include*`, `concat`, ...).
        pub use std::prelude::rust_2024::{
            AsMut, AsRef, Box, Clone, Copy, Debug, Default, DoubleEndedIterator, Drop, Eq, Err, ExactSizeIterator,
            Extend, Fn, FnMut, FnOnce, From, FromIterator, Hash, Into, IntoIterator, Iterator, None, Ok, Option, Ord,
            PartialEq, PartialOrd, Result, Send, Sized, Some, String, Sync, ToOwned, ToString, TryFrom, TryInto,
            Unpin, Vec, assert, cfg, column, derive, drop, file, format_args, line, module_path, stringify, test,
        };
        pub use crate::{
            assert_eq, assert_ne, dbg, debug_assert, debug_assert_eq, eprint, eprintln, format, matches, panic, print, println, todo,
            unimplemented, unreachable, vec, write, writeln,
        };
        // Tide additions.
        pub use crate::marker::Pure;
        pub use crate::tide;
        pub use crate::Root;
        pub use tide_macros::{ensures, invariant, invariants, requires};
    }
}

/// Toolchain attributes spelled with a path, e.g. `#[tide::main]`.
pub mod tide {
    pub use tide_macros::main;
}

pub mod marker {
    pub use std::marker::{Copy, PhantomData, PhantomPinned, Send, Sized, Sync, Unpin};

    /// No authority reachable from the type. The checker owns the analysis;
    /// the blanket impl makes rustc accept every `Pure` bound.
    ///
    /// # Safety
    /// Only the toolchain decides purity; nothing implements this by hand.
    pub unsafe trait Pure {}
    unsafe impl<T: ?Sized> Pure for T {}
}

pub mod mem {
    pub use std::mem::{align_of, discriminant, drop, replace, size_of, size_of_val, swap, take};
}

pub mod hash {
    pub use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher, Hash, Hasher};
}

pub mod io {
    pub use std::io::{Error, ErrorKind, Result};
}

pub mod path {
    // Leak (spike finding): Path's inherent methods `exists`, `metadata`,
    // `read_dir`, `canonicalize`, ... do I/O. The checker must reject them.
    pub use std::path::{Component, Path, PathBuf};
}

pub mod process {
    pub use std::process::{ExitCode, Termination};
}

pub mod collections {
    pub use std::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque, btree_map, btree_set};

    use std::borrow::Borrow;
    use std::hash::{BuildHasherDefault, DefaultHasher, Hash};
    use std::ops::{Deref, DerefMut, Index};

    /// Fixed keys: `DefaultHasher::default()` is SipHash with zero keys.
    type Fixed = BuildHasherDefault<DefaultHasher>;

    macro_rules! wrapper {
        ($name:ident<$($p:ident),*>, $inner:ty, $item:ty) => {
            /// Deterministic: a fixed hash key, so iteration order depends
            /// only on the operations. A distinct type from Rust's.
            #[derive(Clone)]
            pub struct $name<$($p),*>($inner);
            impl<$($p),*> $name<$($p),*> {
                pub fn new() -> Self { Self(Default::default()) }
            }
            impl<$($p),*> PartialEq for $name<$($p),*> where $inner: PartialEq {
                fn eq(&self, o: &Self) -> bool { self.0 == o.0 }
            }
            impl<$($p),*> Default for $name<$($p),*> { fn default() -> Self { Self::new() } }
            impl<$($p),*> Deref for $name<$($p),*> { type Target = $inner; fn deref(&self) -> &$inner { &self.0 } }
            impl<$($p),*> DerefMut for $name<$($p),*> { fn deref_mut(&mut self) -> &mut $inner { &mut self.0 } }
            impl<$($p: std::fmt::Debug),*> std::fmt::Debug for $name<$($p),*> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.0.fmt(f) }
            }
            impl<$($p),*> FromIterator<$item> for $name<$($p),*> where $inner: FromIterator<$item> {
                fn from_iter<I: IntoIterator<Item = $item>>(it: I) -> Self { Self(it.into_iter().collect()) }
            }
            impl<$($p),*> Extend<$item> for $name<$($p),*> where $inner: Extend<$item> {
                fn extend<I: IntoIterator<Item = $item>>(&mut self, it: I) { self.0.extend(it) }
            }
            impl<$($p),*> IntoIterator for $name<$($p),*> {
                type Item = <$inner as IntoIterator>::Item;
                type IntoIter = <$inner as IntoIterator>::IntoIter;
                fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
            }
            impl<'a, $($p),*> IntoIterator for &'a $name<$($p),*> {
                type Item = <&'a $inner as IntoIterator>::Item;
                type IntoIter = <&'a $inner as IntoIterator>::IntoIter;
                fn into_iter(self) -> Self::IntoIter { (&self.0).into_iter() }
            }
        };
    }

    wrapper!(HashMap<K, V>, std::collections::HashMap<K, V, Fixed>, (K, V));
    wrapper!(HashSet<K>, std::collections::HashSet<K, Fixed>, K);

    impl<K: Eq + Hash + Borrow<Q>, Q: Eq + Hash + ?Sized, V> Index<&Q> for HashMap<K, V> {
        type Output = V;
        fn index(&self, k: &Q) -> &V { &self.0[k] }
    }
}

/// Tide's one math library (`libm`, pure Rust), so results match across targets.
pub mod math {
    pub fn sin(x: f64) -> f64 { libm::sin(x) }
    pub fn cos(x: f64) -> f64 { libm::cos(x) }
    pub fn exp(x: f64) -> f64 { libm::exp(x) }
    pub fn ln(x: f64) -> f64 { libm::log(x) }
    pub fn powf(x: f64, y: f64) -> f64 { libm::pow(x, y) }
    pub fn sqrt(x: f64) -> f64 { libm::sqrt(x) }
}

/// Float observers that see every NaN as one canonical NaN.
pub mod float {
    pub const CANONICAL_NAN_BITS: u64 = 0x7ff8_0000_0000_0000;

    pub fn bits(x: f64) -> u64 {
        if x.is_nan() { CANONICAL_NAN_BITS } else { x.to_bits() }
    }

    /// NaN is ignored, as in Rust's `f64::min`; -0.0 orders below +0.0.
    pub fn min(a: f64, b: f64) -> f64 {
        if a.is_nan() { return b; }
        if b.is_nan() { return a; }
        if a == b { if a.is_sign_negative() { a } else { b } } else if a < b { a } else { b }
    }

    pub fn max(a: f64, b: f64) -> f64 {
        if a.is_nan() { return b; }
        if b.is_nan() { return a; }
        if a == b { if a.is_sign_negative() { b } else { a } } else if a > b { a } else { b }
    }
}

/// Every capability a program starts with; `main` receives it.
pub struct Root {
    pub fs: fs::Fs,
    pub clock: time::Clock,
    pub tasks: task::Tasks,
}

/// Toolchain-only runtime entry. The checker rejects naming it in user code;
/// at run time it mints `Root` once per process.
#[doc(hidden)]
pub mod __rt {
    use std::sync::atomic::{AtomicBool, Ordering};

    static MINTED: AtomicBool = AtomicBool::new(false);

    pub fn root() -> crate::Root {
        assert!(!MINTED.swap(true, Ordering::SeqCst), "Root is minted once, by the entry");
        crate::Root { fs: crate::fs::Fs::root(), clock: crate::time::Clock(()), tasks: crate::task::Tasks(()) }
    }
}
