//! Tide std stub for the conformance corpus's rustc oracle (throwaway).
//!
//! Copied from trials/e1-dialect/rustc/std, whose copy stays frozen as the
//! trial's record; cases extend this one. Injected as `std` (`--extern
//! std=`), as spikes/d18 does. It is a compile-time fixture: Tide's std
//! (`crates/tide-std`) owns runtime behavior, and this crate includes its
//! crate-root items (`root.rs`: `Root`, `Pure`, `tide::main`, the sinks, the
//! runtime entry, Rust's re-exports, and the codec engine and `std::json`,
//! `codec.rs` and `json.rs`), `std::io`'s core (`io/shared.rs`),
//! its real and simulated file surface (`fs.rs`), property tests (`test`), task runtime and scheduler
//! (`task.rs`), `Shared` (`sync.rs`), seeded simulations and their runner
//! (`sim.rs`), parallel regions (`par.rs`), `std::time` (`time.rs`), and executable
//! contracts (`contract.rs`). It declares
//! identity-only capabilities with `capability!`, giving those
//! types the same definitions and paths in both crates.
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

// The dbg sink (reference 6.5): as Rust's `dbg!`, it formats each argument with `Debug` and
// returns it.
#[macro_export]
macro_rules! dbg {
    () => { () };
    ($val:expr $(,)?) => { match $val { tmp => { let _ = ::core::format_args!("{:#?}", &tmp); tmp } } };
    ($($val:expr),+ $(,)?) => { ($($crate::dbg!($val)),+,) };
}

crate::declare_prelude! {
    pub use crate::dbg;
    pub use crate::Secret;
    pub use crate::test::test;
    pub use crate::test::sim_test;
    pub use conformance_macros::{pure, trusted_pure};
    pub use tide_macros::{ensures, invariant, invariants, requires};
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

#[path = "../../../../crates/tide-std/src/io/mod.rs"]
pub mod io;
#[path = "../../../../crates/tide-std/src/env.rs"]
pub mod env;
#[path = "../../../../crates/tide-std/src/log.rs"]
pub mod log;

pub mod rand {
    crate::capability!(Rand);
}


// Production owns the complete real/simulated file surface, including its runtime behavior.
#[path = "../../../../crates/tide-std/src/fs.rs"]
pub mod fs;

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
}

pub mod sync {
    use std::marker::PhantomData;

    include!("../../../../crates/tide-std/src/sync.rs");

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

// Production owns the signatures and execution of every parallel region.
#[path = "../../../../crates/tide-std/src/par.rs"]
pub mod par;

pub mod sim {
    include!("../../../../crates/tide-std/src/sim.rs");
    use crate::env::Env;
    use crate::fs::Fs;
    use crate::io::Stdin;
    use crate::marker::Pure;
    use crate::net::Net;
    use crate::process::Proc;
    use crate::rand::Rand;
    use crate::time::Clock;
    use std::marker::PhantomData;

    impl SimRoot {
        pub fn node(&self, name: &str) -> SimNode { unimplemented!() }
        /// GUESS: returns `()` (companion 17 gives no type).
        pub fn run_for(&self, d: Duration) { unimplemented!() }
    }

    impl World {
        pub fn http(&self, addr: &str, f: impl Fn(&Request) -> Reply + 'static) { unimplemented!() }
    }

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

#[doc(hidden)]
#[path = "../../../../crates/tide-std/src/contract.rs"]
pub mod __contract;
