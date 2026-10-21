//! Tide std stub for the conformance corpus's rustc oracle (throwaway).
//!
//! Copied from trials/e1-dialect/rustc/std, whose copy stays frozen as the
//! trial's record; cases extend this one. Injected as `std` (`--extern
//! std=`), as spikes/d18 does. It defines only the items and methods the
//! trial's programs and the cases use, with every field, variant,
//! derive, and `Error` impl the reference gives them (other impls as needed),
//! with the signatures `design/reference.md` and 02/03 give, and re-exports
//! Rust's items where the reference keeps their API. Every body is
//! `unimplemented!()`, so it decides type and borrow verdicts only. Lines
//! marked `GUESS` fix a signature no design document fixes.

#![allow(unused_variables, dead_code)]

pub use std::{
    any, array, borrow, boxed, cell, char, clone, cmp, collections, convert, default, error, f32, f64, fmt, hash,
    iter, mem, num, ops, option, path, rc, result, slice, str, string, vec,
};
pub use std::{
    assert_eq, assert_ne, dbg, debug_assert, eprint, eprintln, format, matches, print, println, todo, unimplemented,
    unreachable, write, writeln,
};

#[macro_export]
macro_rules! panic {
    ($($t:tt)*) => { ::core::panic!($($t)*) };
}

// The log sink (reference 6.5): arguments are evaluated and formatted.
#[macro_export]
macro_rules! __log { ($($t:tt)*) => {{ let _ = ::core::format_args!($($t)*); }} }

pub mod prelude {
    pub mod rust_2024 {
        // A subset of Rust's 2024 prelude, as in spikes/d18/facade, without the
        // builtin `test` (replaced below).
        pub use std::prelude::rust_2024::{
            AsMut, AsRef, Box, Clone, Copy, Debug, Default, DoubleEndedIterator, Drop, Eq, Err, ExactSizeIterator,
            Extend, Fn, FnMut, FnOnce, From, FromIterator, Hash, Into, IntoIterator, Iterator, None, Ok, Option, Ord,
            PartialEq, PartialOrd, Result, Send, Sized, Some, String, Sync, ToOwned, ToString, TryFrom, TryInto,
            Unpin, Vec, assert, cfg, derive, drop, format_args, stringify,
        };
        pub use crate::{__log as error, __log as info};
        pub use crate::{
            assert_eq, assert_ne, dbg, debug_assert, eprint, eprintln, format, matches, panic, print, println, todo,
            unimplemented, unreachable, vec, write, writeln,
        };
        // Tide additions (reference 12.3).
        pub use crate::marker::Pure;
        pub use crate::{Root, Secret};
        pub use crate::par::ParSlice; // stands in for inherent slice methods (reference 8)
        pub use crate::tide;
        pub use oracle_macros::{Error, sim_test, test};
        pub use std::error::Error;
        pub use conformance_macros::{pure, trusted_pure};
        pub use tide_macros::{ensures, invariant, invariants, requires};
    }
}

pub mod tide {
    pub use tide_macros::main;
}

pub mod marker {
    pub use std::marker::*;
    /// The checker decides `Pure`; the blanket impl lets rustc accept every bound (reference 6.6).
    pub unsafe trait Pure {}
    unsafe impl<T: ?Sized> Pure for T {}
}

pub mod process {
    pub use std::process::{ExitCode, Termination};
    #[derive(Clone)]
    pub struct Proc(());
}

pub mod io {
    pub use std::io::{BufRead, BufReader, BufWriter, Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};

    /// Reference 6.1: `io::Stdin`'s methods with `&self` receivers.
    #[derive(Clone)]
    pub struct Stdin(());
    impl Stdin {
        pub fn read_line(&self, buf: &mut String) -> Result<usize> { unimplemented!() }
    }
}

pub mod env {
    /// GUESS: `args()` "as std::env's", so Rust's `Args`, an iterator of `String`.
    pub use std::env::Args;
    pub use std::env::VarError;
    #[derive(Clone)]
    pub struct Env(());
    impl Env {
        pub fn args(&self) -> Args { unimplemented!() }
    }
}

pub mod rand {
    #[derive(Clone)]
    pub struct Rand(());
}

pub mod fs {
    use std::io;
    use std::path::Path;
    /// GUESS: `read_dir` "as std::fs", so Rust's `ReadDir`/`DirEntry`.
    pub use std::fs::{DirEntry, Metadata, ReadDir};

    pub enum Mode {
        Read,
        ReadWrite,
    }

    #[derive(Clone)]
    pub struct Fs(());

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

    #[derive(Clone)]
    pub struct Clock(());
    impl Clock {
        pub fn now(&self) -> Instant { unimplemented!() }
        pub fn sleep_until(&self, t: Instant) -> Result<(), Cancelled> { unimplemented!() }
    }

    /// A distinct type with no `now()`; otherwise Rust's `Instant` API.
    /// GUESS: which of Rust's methods the allowlist admits (`checked_add` here).
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Instant(Duration);
    impl Instant {
        pub fn checked_add(&self, d: Duration) -> Option<Instant> { unimplemented!() }
    }
    impl std::ops::Add<Duration> for Instant {
        type Output = Instant;
        fn add(self, d: Duration) -> Instant { unimplemented!() }
    }
}

pub mod net {
    use crate::time::Instant;

    #[derive(Clone)]
    pub struct Net(());
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

    #[derive(Clone)]
    pub struct Tasks(());

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

    /// Reference 8 makes these inherent slice methods; a stub std cannot add
    /// inherent methods to `[T]`, so a prelude trait stands in.
    pub trait ParSlice<T> {
        fn par_map<U: Send>(&self, f: impl Fn(&T) -> U + Pure + Send + Sync) -> Vec<U>
        where
            T: Pure + Sync;
    }
    impl<T> ParSlice<T> for [T] {
        fn par_map<U: Send>(&self, f: impl Fn(&T) -> U + Pure + Send + Sync) -> Vec<U>
        where
            T: Pure + Sync,
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

/// Every capability a program starts with (reference 6.2). The `log` field
/// (sink configuration) is left out: no document fixes its type. The private
/// field keeps user code from building one (03 §1.3, ROOT-4).
pub struct Root {
    _private: (),
    pub fs: fs::Fs,
    pub net: net::Net,
    pub clock: time::Clock,
    pub rand: rand::Rand,
    pub env: env::Env,
    pub stdin: io::Stdin,
    pub proc: process::Proc,
    pub tasks: task::Tasks,
}
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

/// Used by `#[tide::main]`'s expansion.
#[doc(hidden)]
pub mod __rt {
    pub fn root() -> crate::Root { unimplemented!() }
}
