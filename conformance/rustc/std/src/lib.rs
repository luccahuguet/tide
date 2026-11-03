//! Tide std stub for the conformance corpus's rustc oracle (throwaway).
//!
//! Copied from trials/e1-dialect/rustc/std, whose copy stays frozen as the
//! trial's record; cases extend this one. Injected as `std` (`--extern
//! std=`), as spikes/d18 does. It is a compile-time fixture: Tide's std
//! (`crates/tide-std`) owns runtime behavior, and this crate includes its
//! crate-root items (`root.rs`: `Root`, `Pure`, `tide::main`, the sinks, the
//! runtime entry, and Rust's re-exports) and declares the capability types
//! with its `capability!`, so those have one definition and the same paths in
//! both. Beyond them it defines only the items and methods the trial's
//! programs and the cases use, with every field, variant, derive, and `Error`
//! impl the reference gives them (other impls as needed), with the signatures
//! `design/reference.md` and 02/03 give, and re-exports Rust's items where the
//! reference keeps their API. Each of its own bodies is `unimplemented!()`, so
//! it decides type and borrow verdicts only, except `with_hasher`, which builds
//! the value so that a `static` evaluates. Lines marked `GUESS` fix a signature
//! no design document fixes.

#![allow(unused_variables, dead_code)]

// Tide's std's crate-root items: `Root`, `Pure`, `tide::main`, the sinks and `panic!`, the
// runtime entry, Rust's re-exports, and the `capability!` and `declare_prelude!` macros.
include!("../../../../crates/tide-std/src/root.rs");

// The log sink (reference 6.5): arguments are evaluated and formatted.
#[macro_export]
macro_rules! __log { ($($t:tt)*) => {{ let _ = ::core::format_args!($($t)*); }} }

// The dbg sink (reference 6.5): as Rust's `dbg!`, it formats each argument with `Debug` and
// returns it.
#[macro_export]
macro_rules! dbg {
    () => { () };
    ($val:expr $(,)?) => { match $val { tmp => { let _ = ::core::format_args!("{:#?}", &tmp); tmp } } };
    ($($val:expr),+ $(,)?) => { ($($crate::dbg!($val)),+,) };
}

crate::declare_prelude! {
    pub use crate::{__log as error, __log as info};
    pub use crate::dbg;
    pub use crate::Secret;
    pub use crate::par::ParSlice; // stands in for inherent slice methods (reference 8)
    pub use oracle_macros::{Error, sim_test, test};
    pub use conformance_macros::{pure, trusted_pure};
    pub use spike_macros::{ensures, invariant, invariants, requires};
}

pub mod process {
    pub use std::process::{ExitCode, Termination};
    crate::capability!(Proc);
}

pub mod io {
    pub use std::io::{BufRead, BufReader, BufWriter, Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};

    crate::capability!(Stdin);
    /// Reference 6.1: `io::Stdin`'s methods with `&self` receivers.
    impl Stdin {
        pub fn read_line(&self, buf: &mut String) -> Result<usize> { unimplemented!() }
    }
}

pub mod env {
    /// GUESS: `args()` "as std::env's", so Rust's `Args`, an iterator of `String`.
    pub use std::env::Args;
    pub use std::env::VarError;
    crate::capability!(Env);
    impl Env {
        pub fn args(&self) -> Args { unimplemented!() }
    }
}

pub mod rand {
    crate::capability!(Rand);
}

/// Rust's collections, except that `HashMap` and `HashSet` are Tide std's own types (03
/// GLOBAL-5): `new()` uses a fixed key, and a random key comes only from `Rand`. Their methods
/// and impls keep Rust's signatures, for the methods listed here; the iterator and entry types
/// are Rust's. `std::hash` stays Rust's, so its `RandomState` is Rust's type, not this one.
pub mod collections {
    pub use hash_map::HashMap;
    pub use hash_set::HashSet;
    pub use std::collections::{
        BTreeMap, BTreeSet, BinaryHeap, LinkedList, TryReserveError, VecDeque, binary_heap, btree_map, btree_set,
        linked_list, vec_deque,
    };

    pub mod hash_map {
        use std::borrow::Borrow;
        use std::hash::{BuildHasher, Hash};
        pub use std::collections::hash_map::{
            Entry, IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys, OccupiedEntry, VacantEntry, Values, ValuesMut,
        };

        /// The hasher of `HashMap` and `HashSet`, with a fixed key from `new()` or a random one
        /// from `from_rand` (03 GLOBAL-5). GUESS: its `Hasher` type, `Debug`, and a `Default`
        /// with the fixed key.
        #[derive(Clone, Debug)]
        pub struct RandomState(());
        impl RandomState {
            pub fn from_rand(rand: &crate::rand::Rand) -> RandomState { unimplemented!() }
        }
        impl Default for RandomState {
            fn default() -> Self { unimplemented!() }
        }
        impl BuildHasher for RandomState {
            type Hasher = std::hash::DefaultHasher;
            fn build_hasher(&self) -> Self::Hasher { unimplemented!() }
        }

        /// Wraps Rust's map so that variance, drop check, and auto traits stay Rust's.
        pub struct HashMap<K, V, S = RandomState> {
            base: std::collections::HashMap<K, V, S>,
        }
        impl<K, V> HashMap<K, V> {
            pub fn new() -> HashMap<K, V> { unimplemented!() }
            pub fn with_capacity(capacity: usize) -> HashMap<K, V> { unimplemented!() }
        }
        impl<K, V, S> HashMap<K, V, S> {
            /// Builds the value, so that a `static` using it evaluates, as in Rust.
            pub const fn with_hasher(hasher: S) -> HashMap<K, V, S> {
                HashMap { base: std::collections::HashMap::with_hasher(hasher) }
            }
            pub fn len(&self) -> usize { unimplemented!() }
            pub fn is_empty(&self) -> bool { unimplemented!() }
            pub fn iter(&self) -> Iter<'_, K, V> { unimplemented!() }
            pub fn iter_mut(&mut self) -> IterMut<'_, K, V> { unimplemented!() }
            pub fn keys(&self) -> Keys<'_, K, V> { unimplemented!() }
            pub fn values(&self) -> Values<'_, K, V> { unimplemented!() }
            pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> { unimplemented!() }
            pub fn into_keys(self) -> IntoKeys<K, V> { unimplemented!() }
            pub fn into_values(self) -> IntoValues<K, V> { unimplemented!() }
            pub fn clear(&mut self) { unimplemented!() }
            pub fn retain<F: FnMut(&K, &mut V) -> bool>(&mut self, f: F) { unimplemented!() }
        }
        impl<K: Eq + Hash, V, S: BuildHasher> HashMap<K, V, S> {
            pub fn entry(&mut self, key: K) -> Entry<'_, K, V> { unimplemented!() }
            pub fn get_key_value<Q: ?Sized + Hash + Eq>(&self, k: &Q) -> Option<(&K, &V)>
            where
                K: Borrow<Q>,
            {
                unimplemented!()
            }
            pub fn get<Q: ?Sized + Hash + Eq>(&self, k: &Q) -> Option<&V>
            where
                K: Borrow<Q>,
            {
                unimplemented!()
            }
            pub fn get_mut<Q: ?Sized + Hash + Eq>(&mut self, k: &Q) -> Option<&mut V>
            where
                K: Borrow<Q>,
            {
                unimplemented!()
            }
            pub fn contains_key<Q: ?Sized + Hash + Eq>(&self, k: &Q) -> bool
            where
                K: Borrow<Q>,
            {
                unimplemented!()
            }
            pub fn insert(&mut self, k: K, v: V) -> Option<V> { unimplemented!() }
            pub fn remove<Q: ?Sized + Hash + Eq>(&mut self, k: &Q) -> Option<V>
            where
                K: Borrow<Q>,
            {
                unimplemented!()
            }
        }
        impl<K: Clone, V: Clone, S: Clone> Clone for HashMap<K, V, S> {
            fn clone(&self) -> Self { unimplemented!() }
        }
        impl<K: std::fmt::Debug, V: std::fmt::Debug, S> std::fmt::Debug for HashMap<K, V, S> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { unimplemented!() }
        }
        impl<K: Eq + Hash, V: PartialEq, S: BuildHasher> PartialEq for HashMap<K, V, S> {
            fn eq(&self, other: &Self) -> bool { unimplemented!() }
        }
        impl<K: Eq + Hash, V: Eq, S: BuildHasher> Eq for HashMap<K, V, S> {}
        impl<K, V, S: Default> Default for HashMap<K, V, S> {
            fn default() -> Self { unimplemented!() }
        }
        impl<K: Eq + Hash, V, S: BuildHasher + Default> FromIterator<(K, V)> for HashMap<K, V, S> {
            fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self { unimplemented!() }
        }
        /// A fixed key, as `new()`.
        impl<K: Eq + Hash, V, const N: usize> From<[(K, V); N]> for HashMap<K, V> {
            fn from(arr: [(K, V); N]) -> Self { unimplemented!() }
        }
        impl<K: Eq + Hash, V, S: BuildHasher> Extend<(K, V)> for HashMap<K, V, S> {
            fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) { unimplemented!() }
        }
        impl<'a, K: Eq + Hash + Copy, V: Copy, S: BuildHasher> Extend<(&'a K, &'a V)> for HashMap<K, V, S> {
            fn extend<I: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: I) { unimplemented!() }
        }
        impl<K, V, S> IntoIterator for HashMap<K, V, S> {
            type Item = (K, V);
            type IntoIter = IntoIter<K, V>;
            fn into_iter(self) -> IntoIter<K, V> { unimplemented!() }
        }
        impl<'a, K, V, S> IntoIterator for &'a HashMap<K, V, S> {
            type Item = (&'a K, &'a V);
            type IntoIter = Iter<'a, K, V>;
            fn into_iter(self) -> Iter<'a, K, V> { unimplemented!() }
        }
        impl<'a, K, V, S> IntoIterator for &'a mut HashMap<K, V, S> {
            type Item = (&'a K, &'a mut V);
            type IntoIter = IterMut<'a, K, V>;
            fn into_iter(self) -> IterMut<'a, K, V> { unimplemented!() }
        }
        impl<K, Q: ?Sized, V, S> std::ops::Index<&Q> for HashMap<K, V, S>
        where
            K: Eq + Hash + Borrow<Q>,
            Q: Eq + Hash,
            S: BuildHasher,
        {
            type Output = V;
            fn index(&self, key: &Q) -> &V { unimplemented!() }
        }
    }

    pub mod hash_set {
        use super::hash_map::RandomState;
        use std::borrow::Borrow;
        use std::hash::{BuildHasher, Hash};
        pub use std::collections::hash_set::{IntoIter, Iter};

        /// Wraps Rust's set so that variance, drop check, and auto traits stay Rust's.
        pub struct HashSet<T, S = RandomState> {
            base: std::collections::HashSet<T, S>,
        }
        impl<T> HashSet<T> {
            pub fn new() -> HashSet<T> { unimplemented!() }
            pub fn with_capacity(capacity: usize) -> HashSet<T> { unimplemented!() }
        }
        impl<T, S> HashSet<T, S> {
            /// Builds the value, so that a `static` using it evaluates, as in Rust.
            pub const fn with_hasher(hasher: S) -> HashSet<T, S> {
                HashSet { base: std::collections::HashSet::with_hasher(hasher) }
            }
            pub fn len(&self) -> usize { unimplemented!() }
            pub fn is_empty(&self) -> bool { unimplemented!() }
            pub fn iter(&self) -> Iter<'_, T> { unimplemented!() }
            pub fn clear(&mut self) { unimplemented!() }
            pub fn retain<F: FnMut(&T) -> bool>(&mut self, f: F) { unimplemented!() }
        }
        impl<T: Eq + Hash, S: BuildHasher> HashSet<T, S> {
            pub fn insert(&mut self, value: T) -> bool { unimplemented!() }
            pub fn contains<Q: ?Sized + Hash + Eq>(&self, value: &Q) -> bool
            where
                T: Borrow<Q>,
            {
                unimplemented!()
            }
            pub fn remove<Q: ?Sized + Hash + Eq>(&mut self, value: &Q) -> bool
            where
                T: Borrow<Q>,
            {
                unimplemented!()
            }
        }
        impl<T: Clone, S: Clone> Clone for HashSet<T, S> {
            fn clone(&self) -> Self { unimplemented!() }
        }
        impl<T: std::fmt::Debug, S> std::fmt::Debug for HashSet<T, S> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { unimplemented!() }
        }
        impl<T: Eq + Hash, S: BuildHasher> PartialEq for HashSet<T, S> {
            fn eq(&self, other: &Self) -> bool { unimplemented!() }
        }
        impl<T: Eq + Hash, S: BuildHasher> Eq for HashSet<T, S> {}
        impl<T, S: Default> Default for HashSet<T, S> {
            fn default() -> Self { unimplemented!() }
        }
        impl<T: Eq + Hash, S: BuildHasher + Default> FromIterator<T> for HashSet<T, S> {
            fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self { unimplemented!() }
        }
        /// A fixed key, as `new()`.
        impl<T: Eq + Hash, const N: usize> From<[T; N]> for HashSet<T> {
            fn from(arr: [T; N]) -> Self { unimplemented!() }
        }
        impl<T: Eq + Hash, S: BuildHasher> Extend<T> for HashSet<T, S> {
            fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) { unimplemented!() }
        }
        impl<'a, T: 'a + Eq + Hash + Copy, S: BuildHasher> Extend<&'a T> for HashSet<T, S> {
            fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) { unimplemented!() }
        }
        impl<T, S> IntoIterator for HashSet<T, S> {
            type Item = T;
            type IntoIter = IntoIter<T>;
            fn into_iter(self) -> IntoIter<T> { unimplemented!() }
        }
        impl<'a, T, S> IntoIterator for &'a HashSet<T, S> {
            type Item = &'a T;
            type IntoIter = Iter<'a, T>;
            fn into_iter(self) -> Iter<'a, T> { unimplemented!() }
        }
    }
}

pub mod fs {
    use std::io;
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    /// Tide std's own listing types, with Rust's methods (03 §1.1, "Directory listings").
    #[derive(Debug)]
    pub struct ReadDir(());
    impl Iterator for ReadDir {
        type Item = io::Result<DirEntry>;
        fn next(&mut self) -> Option<io::Result<DirEntry>> { unimplemented!() }
    }
    #[derive(Debug)]
    pub struct DirEntry(());
    impl DirEntry {
        pub fn path(&self) -> PathBuf { unimplemented!() }
        pub fn file_name(&self) -> OsString { unimplemented!() }
        pub fn metadata(&self) -> io::Result<Metadata> { unimplemented!() }
        pub fn file_type(&self) -> io::Result<FileType> { unimplemented!() }
    }
    /// Tide std's own, with Rust's methods (03 §1.1); the stub lists only these.
    #[derive(Clone, Debug)]
    pub struct Metadata(());
    impl Metadata {
        pub fn file_type(&self) -> FileType { unimplemented!() }
        pub fn is_dir(&self) -> bool { unimplemented!() }
        pub fn is_file(&self) -> bool { unimplemented!() }
        pub fn is_symlink(&self) -> bool { unimplemented!() }
        pub fn len(&self) -> u64 { unimplemented!() }
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct FileType(());
    impl FileType {
        pub fn is_dir(&self) -> bool { unimplemented!() }
        pub fn is_file(&self) -> bool { unimplemented!() }
        pub fn is_symlink(&self) -> bool { unimplemented!() }
    }

    pub enum Mode {
        Read,
        ReadWrite,
    }

    crate::capability!(Fs);

    impl Fs {
        pub fn read(&self, p: impl AsRef<Path>) -> io::Result<Vec<u8>> { unimplemented!() }
        pub fn read_to_string(&self, p: impl AsRef<Path>) -> io::Result<String> { unimplemented!() }
        pub fn write_synced(&self, p: impl AsRef<Path>, data: impl AsRef<[u8]>) -> io::Result<()> { unimplemented!() }
        pub fn create_dir_all(&self, p: impl AsRef<Path>) -> io::Result<()> { unimplemented!() }
        pub fn read_dir(&self, p: impl AsRef<Path>) -> io::Result<ReadDir> { unimplemented!() }
        pub fn metadata(&self, p: impl AsRef<Path>) -> io::Result<Metadata> { unimplemented!() }
        pub fn rename(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> { unimplemented!() }
        pub fn sync_dir(&self, p: impl AsRef<Path>) -> io::Result<()> { unimplemented!() }
        pub fn exists(&self, p: impl AsRef<Path>) -> io::Result<bool> { unimplemented!() }
        pub fn dir(&self, p: impl AsRef<Path>, mode: Mode) -> io::Result<Fs> { unimplemented!() }
        pub fn read_only(&self) -> Fs { unimplemented!() }
    }
}

pub mod time {
    use crate::task::Cancelled;
    pub use std::time::Duration;

    crate::capability!(Clock);
    impl Clock {
        pub fn now(&self) -> Instant { unimplemented!() }
        pub fn sleep_until(&self, t: Instant) -> Result<(), Cancelled> { unimplemented!() }
        pub fn system_time(&self) -> SystemTime { unimplemented!() }
    }

    /// A distinct type with no `now()` or `elapsed()` (03 CAP-1); otherwise Rust's `Instant` API.
    /// GUESS: which of Rust's methods the allowlist admits (`checked_add`, `duration_since` here).
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Instant(Duration);
    impl Instant {
        pub fn checked_add(&self, d: Duration) -> Option<Instant> { unimplemented!() }
        pub fn duration_since(&self, earlier: Instant) -> Duration { unimplemented!() }
    }
    impl std::ops::Add<Duration> for Instant {
        type Output = Instant;
        fn add(self, d: Duration) -> Instant { unimplemented!() }
    }

    /// A distinct type with no `now()` or `elapsed()` (03 CAP-1).
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SystemTime(Duration);
}

pub mod net {
    use crate::time::Instant;

    crate::capability!(Net);
    impl Net {
        pub fn get(&self, url: &Url, deadline: Instant) -> Result<Vec<u8>, NetError> { unimplemented!() }
        /// Simulation only (reference 6.8).
        pub fn drop_all(&self) -> Net { unimplemented!() }
    }

    #[derive(Debug)]
    pub enum NetError {
        Timeout,
        Cancelled,
        Denied,
        Refused,
        Reset,
        Dns,
        Status(u16),
        Io(std::io::Error),
    }
    impl NetError {
        pub fn is_cancelled(&self) -> bool { unimplemented!() }
        pub fn is_transient(&self) -> bool { unimplemented!() }
    }
    impl std::fmt::Display for NetError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { unimplemented!() }
    }
    impl std::error::Error for NetError {}

    pub struct Url(String);
    impl Url {
        pub fn parse(s: &str) -> Result<Url, UrlError> { unimplemented!() }
    }
    #[derive(Debug)]
    pub struct UrlError(());
    impl std::fmt::Display for UrlError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { unimplemented!() }
    }
    impl std::error::Error for UrlError {}
}

pub mod task {
    use std::marker::PhantomData;

    crate::capability!(Tasks);

    /// 02 §8.2 and 03 §6.1.
    impl Tasks {
        pub fn scope<'env, T, E, F>(&'env self, body: F) -> Result<T, E>
        where
            E: From<TaskError> + Send,
            F: FnOnce(Scope<'_, 'env, E>) -> Result<T, E>,
        {
            unimplemented!()
        }
        pub fn spawn_isolated<T: Send, F>(&self, f: F) -> Result<T, Panicked>
        where
            F: FnOnce() -> T + Send + 'static,
        {
            unimplemented!()
        }
        pub fn limit(&self, n: usize) -> Tasks { unimplemented!() }
    }

    /// Reference 7.1 (provisional); an `Error` (reference 2.2).
    #[derive(Debug)]
    pub struct Panicked {
        pub message: String,
    }
    impl std::fmt::Display for Panicked {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { unimplemented!() }
    }
    impl std::error::Error for Panicked {}

    /// `Scope<'s, 'env: 's, E>`: `Copy`, `'s` and `'env` invariant, as in std (02 §8.2).
    pub struct Scope<'s, 'env: 's, E> {
        _p: PhantomData<(&'s mut &'s (), &'env mut &'env (), fn(E) -> E)>,
    }
    impl<E> Clone for Scope<'_, '_, E> {
        fn clone(&self) -> Self { *self }
    }
    impl<E> Copy for Scope<'_, '_, E> {}

    impl<'s, 'env, E> Scope<'s, 'env, E> {
        pub fn spawn<U: Send + 's, F>(self, f: F) -> Task<'s, U, E>
        where
            F: FnOnce() -> Result<U, E> + Send + 's,
        {
            unimplemented!()
        }
    }

    /// `Send` and `Sync` exactly when `U, E: Send` (02 §8.1's table), through the
    /// impls below: a `Mutex<(U, E)>` marker would give `Sync` iff `Send` but make
    /// `U` and `E` invariant.
    /// GUESS: the variance (`'s` invariant, `U` and `E` covariant); 02 §4 does
    /// not table `Task`.
    pub struct Task<'s, U, E> {
        _p: PhantomData<(&'s mut &'s (), fn() -> (U, E), *const ())>,
    }
    unsafe impl<U: Send, E: Send> Send for Task<'_, U, E> {}
    unsafe impl<U: Send, E: Send> Sync for Task<'_, U, E> {}
    impl<U, E> Task<'_, U, E> {
        pub fn join(self) -> Result<U, E> { unimplemented!() }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TaskError {
        Cancelled,
        DeadlineExceeded,
    }
    impl std::fmt::Display for TaskError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { unimplemented!() }
    }
    impl std::error::Error for TaskError {}

    #[derive(Debug)]
    pub struct Cancelled;
    impl std::fmt::Display for Cancelled {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { unimplemented!() }
    }
    impl std::error::Error for Cancelled {}
}

pub mod sync {
    use crate::marker::Pure;
    use crate::task::Tasks;
    use std::marker::PhantomData;

    /// `Send + Sync`, invariant in `T` (reference 3.3, 7.4).
    pub struct Shared<T> {
        _p: PhantomData<std::sync::Mutex<T>>,
    }
    impl<T> Clone for Shared<T> {
        fn clone(&self) -> Self { unimplemented!() }
    }
    impl<T: Send + Pure> Shared<T> {
        pub fn new(v: T, tasks: &Tasks) -> Shared<T> { unimplemented!() }
        pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R + Pure) -> Result<R, LockError> { unimplemented!() }
        pub fn with2<U: Send + Pure, R>(
            a: &Shared<T>,
            b: &Shared<U>,
            f: impl FnOnce(&mut T, &mut U) -> R + Pure,
        ) -> Result<R, LockError> {
            unimplemented!()
        }
        pub fn same(a: &Shared<T>, b: &Shared<T>) -> bool { unimplemented!() }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum LockError {
        Aliased,
        Poisoned,
    }
    impl std::fmt::Display for LockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { unimplemented!() }
    }
    impl std::error::Error for LockError {}

    /// Rust's `Arc` without `strong_count`, `weak_count`, `get_mut`,
    /// `try_unwrap`, and `into_inner`; creation takes `&Tasks` (03 §1.2, GLOBAL-6).
    pub struct Arc<T: ?Sized> {
        _p: PhantomData<std::sync::Arc<T>>,
    }
    impl<T> Arc<T> {
        pub fn new(v: T, tasks: &Tasks) -> Arc<T> { unimplemented!() }
        pub fn downgrade(this: &Arc<T>, tasks: &Tasks) -> Weak<T> { unimplemented!() }
    }
    impl<T: Clone> Arc<T> {
        pub fn make_mut(this: &mut Arc<T>) -> &mut T { unimplemented!() }
    }
    impl<T: ?Sized> Clone for Arc<T> {
        fn clone(&self) -> Self { unimplemented!() }
    }
    impl<T: ?Sized> std::ops::Deref for Arc<T> {
        type Target = T;
        fn deref(&self) -> &T { unimplemented!() }
    }

    pub struct Weak<T: ?Sized> {
        _p: PhantomData<std::sync::Weak<T>>,
    }
    impl<T> Weak<T> {
        pub fn upgrade(&self) -> Option<Arc<T>> { unimplemented!() }
    }

    /// Needs the `unsafe` grant (03 §1.2). GUESS: no document gives their API;
    /// only the names are fixed.
    pub mod raw {
        use std::marker::PhantomData;
        pub struct Mutex<T: ?Sized>(PhantomData<std::sync::Mutex<T>>);
        pub struct RwLock<T: ?Sized>(PhantomData<std::sync::RwLock<T>>);
    }

    /// Rust's `std::sync::mpsc`, except that creation takes `&Tasks` (reference 7.3).
    pub mod mpsc {
        use crate::task::Tasks;
        use std::marker::PhantomData;

        pub fn sync_channel<T: Send>(bound: usize, tasks: &Tasks) -> (SyncSender<T>, Receiver<T>) {
            unimplemented!()
        }

        pub struct SyncSender<T>(PhantomData<std::sync::mpsc::SyncSender<T>>);
        pub struct Receiver<T>(PhantomData<std::sync::mpsc::Receiver<T>>);
        impl<T> SyncSender<T> {
            pub fn send(&self, t: T) -> Result<(), SendError<T>> { unimplemented!() }
        }
        impl<T> Receiver<T> {
            pub fn recv(&self) -> Result<T, RecvError> { unimplemented!() }
            /// There is no `recv_timeout`, which would read time without a `Clock` (03 CAP-1).
            pub fn recv_deadline(&self, t: crate::time::Instant) -> Result<T, RecvTimeoutError> { unimplemented!() }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct SendError<T>(pub T);
        /// GUESS: reference 7.2's "channel errors whose `is_cancelled()` is
        /// true"; Rust's `RecvError` is a unit struct with no such method.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct RecvError(());
        impl RecvError {
            pub fn is_cancelled(&self) -> bool { unimplemented!() }
        }
        impl std::fmt::Display for RecvError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { unimplemented!() }
        }
        impl std::error::Error for RecvError {}
        /// 03 §6.4: `Timeout`, `Disconnected`, and `Cancelled`, with Rust's traits.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum RecvTimeoutError {
            Timeout,
            Disconnected,
            Cancelled,
        }
        impl RecvTimeoutError {
            pub fn is_cancelled(&self) -> bool { unimplemented!() }
        }
        impl std::fmt::Display for RecvTimeoutError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { unimplemented!() }
        }
        impl std::error::Error for RecvTimeoutError {}
        impl From<RecvError> for RecvTimeoutError {
            fn from(e: RecvError) -> Self { unimplemented!() }
        }
    }
}

pub mod arena {
    use std::marker::PhantomData;

    pub struct Arena<T> {
        _p: PhantomData<Vec<T>>,
    }
    impl<T> Arena<T> {
        pub fn new() -> Arena<T> { unimplemented!() }
        pub fn insert(&mut self, v: T) -> Id<T> { unimplemented!() }
        pub fn get(&self, id: Id<T>) -> Option<&T> { unimplemented!() }
        pub fn get_mut(&mut self, id: Id<T>) -> Option<&mut T> { unimplemented!() }
        pub fn remove(&mut self, id: Id<T>) -> Option<T> { unimplemented!() }
    }
    impl<T> std::ops::Index<Id<T>> for Arena<T> {
        type Output = T;
        fn index(&self, id: Id<T>) -> &T { unimplemented!() }
    }
    impl<T> std::ops::IndexMut<Id<T>> for Arena<T> {
        fn index_mut(&mut self, id: Id<T>) -> &mut T { unimplemented!() }
    }

    /// GUESS: the listed traits hold for every `T` (reference 3.6 lists them
    /// unconditionally; derives would bound `T`).
    pub struct Id<T> {
        index: u32,
        generation: u32,
        _p: PhantomData<fn() -> T>,
    }
    impl<T> Clone for Id<T> {
        fn clone(&self) -> Self { *self }
    }
    impl<T> Copy for Id<T> {}
    impl<T> PartialEq for Id<T> {
        fn eq(&self, o: &Self) -> bool { unimplemented!() }
    }
    impl<T> Eq for Id<T> {}
    impl<T> std::hash::Hash for Id<T> {
        fn hash<H: std::hash::Hasher>(&self, h: &mut H) { unimplemented!() }
    }
    impl<T> PartialOrd for Id<T> {
        fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> { unimplemented!() }
    }
    impl<T> Ord for Id<T> {
        fn cmp(&self, o: &Self) -> std::cmp::Ordering { unimplemented!() }
    }
    impl<T> std::fmt::Debug for Id<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { unimplemented!() }
    }
}

pub mod par {
    use crate::marker::Pure;

    /// 03 §8.1 (PAR-5). GUESS: the result types' `Send` bounds; 03 gives only
    /// the closures' bounds.
    pub fn join<A, B, RA: Send, RB: Send>(a: A, b: B) -> (RA, RB)
    where
        A: FnOnce() -> RA + Pure + Send,
        B: FnOnce() -> RB + Pure + Send,
    {
        unimplemented!()
    }

    /// Reference 8 makes these inherent slice methods; a stub std cannot add
    /// inherent methods to `[T]`, so a prelude trait stands in.
    pub trait ParSlice<T> {
        fn par_map<U: Send>(&self, f: impl Fn(&T) -> U + Pure + Send + Sync) -> Vec<U>
        where
            T: Pure + Sync;
        fn par_for(&mut self, f: impl Fn(&mut T) + Pure + Send + Sync)
        where
            T: Pure + Send;
    }
    impl<T> ParSlice<T> for [T] {
        fn par_map<U: Send>(&self, f: impl Fn(&T) -> U + Pure + Send + Sync) -> Vec<U>
        where
            T: Pure + Sync,
        {
            unimplemented!()
        }
        fn par_for(&mut self, f: impl Fn(&mut T) + Pure + Send + Sync)
        where
            T: Pure + Send,
        {
            unimplemented!()
        }
    }
}

pub mod sim {
    use crate::env::Env;
    use crate::fs::Fs;
    use crate::io::Stdin;
    use crate::marker::Pure;
    use crate::net::Net;
    use crate::process::Proc;
    use crate::rand::Rand;
    use crate::task::Tasks;
    use crate::time::{Clock, Duration};
    use std::marker::PhantomData;
    use std::path::Path;

    /// `Root`'s capability fields, plus setup and inspection (reference 6.8).
    pub struct SimRoot {
        pub fs: Fs,
        pub net: Net,
        pub clock: Clock,
        pub rand: Rand,
        pub env: Env,
        pub stdin: Stdin,
        pub proc: Proc,
        pub tasks: Tasks,
        pub world: World,
        pub faults: Faults,
    }
    impl SimRoot {
        pub fn node(&self, name: &str) -> SimNode { unimplemented!() }
        pub fn elapsed(&self) -> Duration { unimplemented!() }
        pub fn records(&self) -> Vec<Record> { unimplemented!() }
        /// GUESS: returns `()` (companion 17 gives no type).
        pub fn run_for(&self, d: Duration) { unimplemented!() }
    }

    pub struct World(());
    impl World {
        pub fn file(&self, path: impl AsRef<Path>, contents: impl AsRef<[u8]>) { unimplemented!() }
        /// GUESS: element and return types of `args` and `stdin`.
        pub fn args<S: Into<String>>(&self, args: impl IntoIterator<Item = S>) { unimplemented!() }
        pub fn stdin(&self, bytes: impl AsRef<[u8]>) { unimplemented!() }
        pub fn http(&self, addr: &str, f: impl Fn(&Request) -> Reply + 'static) { unimplemented!() }
    }

    pub struct Faults(());
    impl Faults {
        pub fn at(&self, t: Duration, fault: Fault) { unimplemented!() }
    }
    pub struct Fault(());
    impl Fault {
        pub fn crash(node: &SimNode) -> Fault { unimplemented!() }
    }

    pub struct SimNode(());
    impl SimNode {
        /// Companion 17 and 03 §9.5: an `Fn` entry, `Send`, with `'static`,
        /// `Pure + Sync` captures.
        pub fn start<T, E, F>(&self, entry: F) -> NodeTask<T, E>
        where
            F: Fn(NodeRoot) -> Result<T, E> + Send + Sync + Pure + 'static,
        {
            unimplemented!()
        }
        pub fn fs(&self) -> Fs { unimplemented!() }
    }
    pub struct NodeTask<T, E>(PhantomData<fn() -> (T, E)>);

    pub struct NodeRoot {
        pub fs: Fs,
        pub net: Net,
        pub clock: Clock,
        pub rand: Rand,
        pub env: Env,
        pub stdin: Stdin,
        pub proc: Proc,
        pub tasks: Tasks,
    }

    pub struct Request(());
    pub struct Reply(());
    impl Reply {
        pub fn ok(body: impl Into<Vec<u8>>) -> Reply { unimplemented!() }
        pub fn status(code: u16) -> Reply { unimplemented!() }
        pub fn hang() -> Reply { unimplemented!() }
    }

    pub struct Record {
        pub node: String,
        pub time: Duration,
        pub sink: Sink,
        pub message: String,
    }
    pub enum Sink {
        Out,
        Err,
        Dbg,
        Log(Level),
    }
    pub enum Level {
        Error,
        Warn,
        Info,
        Debug,
        Trace,
    }
}

/// Test code only (ROOT-3).
impl Root {
    pub fn sim(seed: u64) -> sim::SimRoot { unimplemented!() }
}

/// Reference 6.5 and 03 SINK-10: neither `Debug` nor `Display`.
pub struct Secret<T>(T);
impl<T> Secret<T> {
    /// GUESS: no document gives the constructor.
    pub fn new(v: T) -> Secret<T> { unimplemented!() }
    pub fn expose(&self) -> &T { unimplemented!() }
}

/// 03 GLOBAL-4: identity by reference; core's `ptr::eq` takes raw pointers.
pub mod ptr {
    pub fn eq<T: ?Sized>(a: &T, b: &T) -> bool { unimplemented!() }
}
