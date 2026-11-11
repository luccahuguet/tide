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
//! - the capability types `Root` carries: the identities `Net`, `Rand`, `Env`, `Stdin`,
//!   and `Proc`; `Fs` (real files, below); and `Tasks` and `Clock`, which also name their
//!   simulation when they have one;
//! - `std::time` (`time.rs`): `Clock` with `now`, `system_time`, `sleep`, `sleep_until`, and
//!   `monotonic_only`, whose `system_time()` panics; Rust's `Duration`; and `Instant`,
//!   `SystemTime`, `SystemTimeError`, and `UNIX_EPOCH` with Rust's arithmetic and no `now()` or
//!   `elapsed()`. A sleep is a checkpoint: it returns `task::Cancelled` in a cancelled scope,
//!   and a cancel ends one in progress. Instants before a clock's origin and wall times before
//!   1970 are not representable;
//! - structured tasks on OS threads: `Tasks::scope`, `Scope::spawn`, `Task::join`, and
//!   `Scope::cancel`, scoped borrows, automatic joins, TASK-2 failure selection and cancellation
//!   echoes, and level-triggered `Tasks::checkpoint` / `Tasks::yield_now` (`task.rs`);
//! - `std::sync::Shared`: `new(v, &Tasks)`, `clone`, and single-cell `with`, with FIFO
//!   acquisition and `LockError::Poisoned` after callback panic. `LockError::Aliased` is
//!   reserved for the planned multi-cell operations (03 §6.5);
//! - `Root::sim(seed)`, `std::sim::SimRoot`, and a seeded scheduler for structured tasks,
//!   single-cell `Shared`, and `Clock`, with
//!   source-located scheduling and wait records (03 §9.1) and virtual time, which starts at zero,
//!   jumps to the next timer only when every task is blocked, and ends in a deadlock report
//!   when none is left (03 §9.3). `SimRoot` keeps its calling thread as task 0 until it drops;
//!   `elapsed`, `records`, and `trace` inspect without scheduling. Sinks record ordered events
//!   instead of writing to host streams;
//! - `#[sim_test] fn t(sim: SimRoot)` (100 seeds) and `#[sim_test(seeds = N)]` (0..N),
//!   one child process per test with a wall-clock backstop of 300 s per seed and empty stdin.
//!   The first and each failing seed runs twice for byte identity. Failures print seeds,
//!   causes, and traces; success emits only a single summary, captured by libtest.
//!   Signal/timeout or a missing summary names the last seed started; ordinary failures name
//!   the failing seeds. `TIDE_SIM_SEED` selects one fresh decimal u64 seed through the internal
//!   runner hook, without source/build identity checks;
//! - canonical float observers (`float`) and deterministic math (`math`), generic over
//!   `f32`/`f64`, shared verbatim with conformance (bead `fx-yg5.2`);
//! - Rust's std modules and macros whose API Tide keeps, as listed in `root.rs`, and the
//!   prelude's `Error` trait and derive;
//!   the derive excludes `error(fmt = ..)`, backtrace detection/`#[backtrace]`, and
//!   `Error::provide`, and requires a message on every variant if any has one (including
//!   transparent variants), even with a manual Display;
//! - the prelude's `#[test]`: ordinary libtest entries for parameterless tests and
//!   deterministic generation and shrinking for tests with parameters. Replay a
//!   failing case with `TIDE_TEST_SEED` set to the reported seed;
//! - `collections`: `HashMap`/`HashSet` with the pinned toolchain's full stable Rust API
//!   and fixed default keys, capability-seeded `RandomState`, and
//!   Rust's ordered collections, queue, and heap (see `collections`);
//! - `std::io`'s core (bead `fx-ecb`): `Read`, `Write`, `Seek`, `BufRead`, `BufReader`,
//!   `BufWriter`, `io::Error` with `ErrorKind::Cancelled`, and `io::Result`, over in-memory
//!   readers and writers (`io/mod.rs`);
//! - the prelude's `Encode` and `Decode` traits and derives with `#[codec(rename, rename_all,
//!   default, skip)]`, and `std::json`: `encode`, `decode`, and `json::Error` (bead `fx-08m`;
//!   `json.rs` states the encoding, `codec.rs` lists the std types with impls). The derives
//!   take no other serde key, and no key on a tuple field or `skip` on a variant;
//! - real files: `Fs`, `Mode`, `OpenOptions`, `File`, directory listings, and type/length
//!   metadata (`fs.rs`). `dir` confines paths with Linux `openat2`; file operations check
//!   cancellation, except the shielded `File::close`, and panic inside a simulation until
//!   simulated files exist (bead `fx-byk.2`); simulated `Fs` identities also reject operations
//!   after their simulation ends. Simulated `Rand` rejects Rand-backed hash initialization.
//!
//! Suppressed failures, echoes and panics emit nothing at the default log level; their debug
//! record is deferred until the log sink exists. Discarded task values drop on the child or
//! receiver thread depending on timing, so a destructor panic is attributed to the thread that
//! runs it. Suppressed payload drops on the panic path follow PANIC-2 and can abort; on the
//! non-panic path, a drop panic surfaces in the scope's caller.
//!
//! Not provided yet: task limits, deadlines, shields and
//! isolation; other capability operations and attenuation of reference 6.1 (network,
//! random draws, environment, input, processes); `Root`'s `log` field;
//! the `dbg`, `log`, and `trace` sinks and `io::stdout()`/`io::stderr()`; `Secret`;
//! multi-cell `Shared` operations, `Shared::same` and `Shared::reset`, other `sync` items,
//! `par`, `arena`, `ptr`; world setup, faults, nodes, replay manifests, generated simulation
//! parameters, requires-guided generation,
//! generated contract tests, and the contract attributes. A
//! program that uses one fails to compile, as for any missing item.
//! Executable tests use rustc's harness; only `#[sim_test]` entries isolate their run in a
//! child process. They are for reviewed/trusted demos, without RUN-3's OS sandbox or RUN-5's
//! unreviewed-branch guarantee.
//!
//! `root.rs` (with `codec.rs` and `json.rs`), `fs.rs`, `test`, `task.rs` (including its
//! scheduler), `sync.rs`, `sim.rs` (with its runner), and `time.rs`
//! share their definitions with the conformance fixture std. The identity-only capability
//! types use `root.rs`'s `capability!` in both crates.

include!("root.rs");

crate::declare_prelude! {
    pub use crate::test::{test, sim_test};
}

pub mod test;
pub mod fs;

pub mod net {
    crate::capability!(
        /// Network authority. It has no operations yet: a program can declare that it holds
        /// `Net`, but cannot use it.
        Net
    );
}

pub mod time;

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

pub mod sim;
