//! Tide's std (D18): the crate the admission driver injects as `std` (`--extern std=`), so a
//! Tide program names its items as `std::...` and cannot name Rust's std. It owns the runtime
//! behavior of every item it exposes.
//!
//! This partial std provides:
//!
//! - the entry: `#[tide::main]`, `Root`, and the runtime entry it calls, which mints the
//!   process's one `Root` (03 ROOT-1, RUN-1);
//! - the `out` and `err` sinks: `print!`, `println!`, `eprint!`, and `eprintln!` (03 §2.4);
//! - `Pure` (reference 6.6), which the admission driver decides;
//! - the capability types `Root` carries: the identities `Fs`, `Net`, `Clock`, `Rand`, `Env`,
//!   `Stdin`, and `Proc`, and `Tasks`, which also names its simulation when it has one;
//! - structured tasks on OS threads: `Tasks::scope`, `Scope::spawn`, `Task::join`, and
//!   `Scope::cancel`, scoped borrows, automatic joins, TASK-2 failure selection and cancellation
//!   echoes, and level-triggered `Tasks::checkpoint` / `Tasks::yield_now` (`task.rs`);
//! - `std::sync::Shared`: `new(v, &Tasks)`, `clone`, and single-cell `with`, with FIFO
//!   acquisition and `LockError::Poisoned` after callback panic. `LockError::Aliased` is
//!   reserved for the planned multi-cell operations (03 §6.5);
//! - an internal seeded scheduler for structured tasks and single-cell `Shared`, with
//!   source-located scheduling and wait records (03 §9.1). The public entry and runner are pending;
//! - canonical float observers (`float`) and deterministic math (`math`), generic over
//!   `f32`/`f64`, shared verbatim with conformance (bead `fx-yg5.2`);
//! - Rust's std modules and macros whose API Tide keeps, as listed in `root.rs`, and the
//!   prelude's `Error` trait and derive and parameterless Rust `#[test]`;
//!   the derive excludes `error(fmt = ..)`, backtrace detection/`#[backtrace]`, and
//!   `Error::provide`, and requires a message on every variant if any has one (including
//!   transparent variants), even with a manual Display;
//! - `collections`: `HashMap`/`HashSet` with the pinned toolchain's full stable Rust API
//!   and fixed default keys, capability-seeded `RandomState`, and
//!   Rust's ordered collections, queue, and heap (see `collections`);
//! - `std::io`'s core (bead `fx-ecb`): `Read`, `Write`, `Seek`, `BufRead`, `BufReader`,
//!   `BufWriter`, `io::Error` with `ErrorKind::Cancelled`, and `io::Result`, over in-memory
//!   readers and writers (`io/mod.rs`).
//!
//! Suppressed failures, echoes and panics emit nothing at the default log level; their debug
//! record is deferred until the log sink exists. Discarded task values drop on the child or
//! receiver thread depending on timing, so a destructor panic is attributed to the thread that
//! runs it. Suppressed payload drops on the panic path follow PANIC-2 and can abort; on the
//! non-panic path, a drop panic surfaces in the scope's caller.
//!
//! Not provided yet: task limits, deadlines, shields and
//! isolation; other capability operations and attenuation of reference 6.1 (files, network,
//! time, random draws, environment, input, processes); `Root`'s `log` field;
//! the `dbg`, `log`, and `trace` sinks and `io::stdout()`/`io::stderr()`; `Secret`;
//! `time`'s Rust items (`Duration`, ...);
//! multi-cell `Shared` operations, `Shared::same` and `Shared::reset`, other `sync` items,
//! `par`, `arena`, `ptr`, and `sim`; property tests and
//! `#[sim_test]`; the contract attributes and the `Encode` and `Decode` derives. A
//! program that uses one fails to compile, as for any missing item.
//! Executable tests use rustc's ordinary harness, without simulated capabilities or process
//! isolation. They are for reviewed/trusted demos only, not RUN-5's unreviewed-branch guarantee.
//!
//! `root.rs`, `task.rs` (including its scheduler), and `sync.rs` share their definitions with the
//! conformance fixture std. The identity-only capability types use `root.rs`'s `capability!` in
//! both crates.

include!("root.rs");

crate::declare_prelude! {
    pub use core::prelude::v1::test;
}

pub mod fs {
    crate::capability!(
        /// File-system authority. It has no operations yet.
        Fs
    );
}

pub mod net {
    crate::capability!(
        /// Network authority. It has no operations yet: a program can declare that it holds
        /// `Net`, but cannot use it.
        Net
    );
}

pub mod time {
    crate::capability!(
        /// Authority to read time and sleep. It has no operations yet.
        Clock
    );
}

pub mod rand {
    crate::capability!(
        /// Randomness authority, used by `collections::hash_map::RandomState::from_rand`.
        /// It has no draw methods yet.
        Rand
    );
}

pub mod env {
    crate::capability!(
        /// Authority to read the arguments and the environment. It has no operations yet.
        Env
    );
}

pub mod io;

/// Canonical float observers (reference §4.3).
pub mod float;
/// Target-independent math (reference §4.3).
pub mod math;

pub mod process {
    pub use std::process::{ExitCode, Termination};

    crate::capability!(
        /// Authority to run processes and exit. It has no operations yet.
        Proc
    );
}

pub mod task;
pub mod sync;
