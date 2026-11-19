//! Tide std stub for the conformance corpus's rustc oracle (throwaway).
//!
//! Copied from trials/e1-dialect/rustc/std, whose copy stays frozen as the
//! trial's record; cases extend this one. Injected as `std` (`--extern
//! std=`), as spikes/d18 does. It is a compile-time fixture: Tide's std
//! (`crates/tide-std`) owns runtime behavior, and this crate includes its
//! crate-root items (`root.rs`: `Root`, `Pure`, `tide::main`, the sinks, the
//! runtime entry, Rust's re-exports, numeric conversions (`convert.rs`), and the codec engine
//! and `std::json` (`codec.rs` and `json.rs`)), `std::io`'s core (`io/shared.rs`),
//! its real and simulated file surface (`fs.rs`), property tests (`test`), task runtime and scheduler
//! (`task.rs`), `Arc`, `Weak`, and `Shared` (`sync.rs`), seeded simulations and their runner
//! (`sim.rs`), parallel regions (`par.rs`), `std::time` (`time.rs`), `Rand` (`rand.rs`), and executable
//! contracts (`contract.rs`). It declares
//! identity-only capabilities with `capability!`, giving those
//! types the same definitions and paths in both crates.
//! Beyond them it defines only the items and methods the trial's programs and
//! the cases use, with every field, variant, derive, and `Error` impl the
//! reference gives them (other impls as needed), with the signatures
//! `design/reference.md` and 02/03 give, and re-exports Rust's items where the
//! reference keeps their API. The remaining stub methods use `unimplemented!()`,
//! so it decides type and borrow verdicts only. Collections share production's
//! module through `root.rs`, as do `arena::{Arena, Id}` and the prelude's
//! production `#[pure]` attribute. Lines marked `GUESS` fix a signature no design
//! document fixes.

#![allow(unused_variables, dead_code)]

// Tide's std's crate-root items: `Root`, `Pure`, `tide::main`, the sinks and `panic!`, the
// runtime entry, Rust's re-exports, and the `capability!` and `declare_prelude!` macros.
include!("../../../../crates/tide-std/src/root.rs");

crate::declare_prelude! {
    pub use crate::Secret;
    pub use crate::test::test;
    pub use crate::test::sim_test;
    pub use conformance_macros::trusted_pure;
    pub use tide_macros::{ensures, invariant, invariants, requires};
}

// Production owns float/math behavior and signatures in both std surfaces.
#[path = "../../../../crates/tide-std/src/float.rs"]
pub mod float;
#[path = "../../../../crates/tide-std/src/math.rs"]
pub mod math;

#[path = "../../../../crates/tide-std/src/test/mod.rs"]
pub mod test;

// Production owns `Proc`, `Command`, `Child`, the pipe ends, `Stdio` and both backends.
#[path = "../../../../crates/tide-std/src/process.rs"]
pub mod process;

#[path = "../../../../crates/tide-std/src/io/mod.rs"]
pub mod io;
#[path = "../../../../crates/tide-std/src/env.rs"]
pub mod env;
#[path = "../../../../crates/tide-std/src/log.rs"]
pub mod log;

// Production owns `Rand`, its draws and both backends.
#[path = "../../../../crates/tide-std/src/rand.rs"]
pub mod rand;


// Production owns the complete real/simulated file surface, including its runtime behavior.
#[path = "../../../../crates/tide-std/src/fs.rs"]
pub mod fs;

// Production owns `Clock`, `Instant`, `SystemTime`, and their runtime.
#[path = "../../../../crates/tide-std/src/time.rs"]
pub mod time;

// Production owns TCP, `Net::get`, `Net::resolve`, `Url` and `NetError`; simulated HTTP peers
// are in `sim`.
pub mod net {
    include!("../../../../crates/tide-std/src/net.rs");
    impl Net {
        /// Simulation only (reference 6.8).
        pub fn drop_all(&self) -> Net { unimplemented!() }
    }
}

pub mod task {
    // Production owns the exposed task items and their compile-time verdicts.
    include!("../../../../crates/tide-std/src/task.rs");
}

pub mod sync {
    include!("../../../../crates/tide-std/src/sync.rs");

    /// Needs the `unsafe` grant (03 §1.2). GUESS: no document gives their API;
    /// only the names are fixed.
    pub mod raw {
        use std::marker::PhantomData;
        pub struct Mutex<T: ?Sized>(PhantomData<std::sync::Mutex<T>>);
        pub struct RwLock<T: ?Sized>(PhantomData<std::sync::RwLock<T>>);
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
