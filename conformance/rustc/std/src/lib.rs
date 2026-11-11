//! Tide std stub for the conformance corpus's rustc oracle (throwaway).
//!
//! Copied from trials/e1-dialect/rustc/std, whose copy stays frozen as the
//! trial's record; cases extend this one. Injected as `std` (`--extern
//! std=`), as spikes/d18 does. It is a compile-time fixture: Tide's std
//! (`crates/tide-std`) owns runtime behavior, and this crate includes its
//! crate-root items (`root.rs`: `Root`, `Pure`, `tide::main`, the sinks, the
//! runtime entry, and Rust's re-exports), `std::io`'s core (`io/shared.rs`),
//! its task runtime and scheduler (`task.rs`), single-cell `Shared` (`sync.rs`), and
//! `std::time` (`time.rs`). It declares
//! identity-only capabilities with `capability!`, giving those types the same definitions and
//! paths in both crates.
//! Beyond them it defines only the items and methods the trial's programs and
//! the cases use, with every field, variant, derive, and `Error` impl the
//! reference gives them (other impls as needed), with the signatures
//! `design/reference.md` and 02/03 give, and re-exports Rust's items where the
//! reference keeps their API. The remaining stub methods use `unimplemented!()`,
//! so it decides type and borrow verdicts only. Collections share production's
//! module through `root.rs`. Lines marked `GUESS` fix a signature no design
//! document fixes.

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
    pub use crate::test::test;
    pub use oracle_macros::sim_test;
    pub use conformance_macros::{pure, trusted_pure};
    pub use spike_macros::{ensures, invariant, invariants, requires};
}

// Production owns float/math behavior and signatures in both std surfaces.
#[path = "../../../../crates/tide-std/src/float.rs"]
pub mod float;
#[path = "../../../../crates/tide-std/src/math.rs"]
pub mod math;

#[path = "../../../../crates/tide-std/src/test/mod.rs"]
pub mod test;

pub mod process {
    pub use std::process::{ExitCode, Termination};
    crate::capability!(Proc);
}

pub mod io {
    // Tide's std's io core, included as `root.rs` is, so each item has one definition.
    include!("../../../../crates/tide-std/src/io/shared.rs");

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


pub mod fs {
    use crate::io;
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

// Production owns `Clock`, `Instant`, `SystemTime`, and their runtime.
#[path = "../../../../crates/tide-std/src/time.rs"]
pub mod time;

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
        Io(crate::io::Error),
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
    // Production owns the exposed task items and their compile-time verdicts.
    include!("../../../../crates/tide-std/src/task.rs");

    // Later task APIs remain compile-time fixtures only.
    impl Tasks {
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
}

pub mod sync {
    use std::marker::PhantomData;

    include!("../../../../crates/tide-std/src/sync.rs");

    // Later Shared APIs remain compile-time fixtures only.
    impl<T: Send + Pure> Shared<T> {
        pub fn with2<U: Send + Pure, R>(
            a: &Shared<T>,
            b: &Shared<U>,
            f: impl FnOnce(&mut T, &mut U) -> R + Pure,
        ) -> Result<R, LockError> {
            unimplemented!()
        }
        pub fn same(a: &Shared<T>, b: &Shared<T>) -> bool { unimplemented!() }
        pub fn with_all<R>(cells: &[Shared<T>], f: impl FnOnce(&mut [&mut T]) -> R + Pure)
            -> Result<R, LockError> { unimplemented!() }
    }

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
        pub fn channel<T: Send>(tasks: &Tasks) -> (Sender<T>, Receiver<T>) {
            unimplemented!()
        }

        pub struct Sender<T>(PhantomData<std::sync::mpsc::Sender<T>>);
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
