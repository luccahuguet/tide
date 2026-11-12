//! Tide's std (D18): the crate the admission driver injects as `std` (`--extern std=`), so a
//! Tide program names its items as `std::...` and cannot name Rust's std. It owns the runtime
//! behavior of every item it exposes.
//!
//! This partial std provides:
//!
//! - executable contract attributes (`requires`, `ensures`, `invariants`) and loop
//!   `invariant!`, with `old`, `result`, implication and bounded quantifiers (04 §1.8).
//!   Checked clauses run in debug/test builds, sampled clauses only in the crate under test;
//!   release type-checks but executes no clauses or snapshots. Hidden `__contract` helpers
//!   own violations and unchecked records for runners (limits below);
//! - the entry: `#[tide::main]`, `Root`, and the runtime entry it calls, which mints the
//!   process's one `Root` (03 ROOT-1, RUN-1);
//! - the `out` and `err` sinks: `print!`, `println!`, `eprint!`, and `eprintln!` (03 §2.4);
//! - `Pure` (reference 6.6), which the admission driver decides;
//! - the capability types `Root` carries: the identities `Net`, `Rand`, `Env`, `Stdin`,
//!   and `Proc`; `Fs` (real and simulated files, below); and `Tasks` and `Clock`, which also
//!   name their simulation when they have one;
//! - `std::time` (`time.rs`): `Clock` with `now`, `system_time`, `sleep`, `sleep_until`, and
//!   `monotonic_only`, whose `system_time()` panics; Rust's `Duration`; and `Instant`,
//!   `SystemTime`, `SystemTimeError`, and `UNIX_EPOCH` with Rust's arithmetic and no `now()` or
//!   `elapsed()`. A sleep is a checkpoint: it returns `task::Cancelled` in a cancelled scope,
//!   and a cancel ends one in progress. Instants before a clock's origin and wall times before
//!   1970 are not representable;
//! - structured tasks on OS threads: `Tasks::scope`, `Scope::spawn`, `Task::join`, and
//!   `Scope::cancel`, scoped borrows, automatic joins, TASK-2 failure selection and cancellation
//!   echoes, and level-triggered `Tasks::checkpoint` / `Tasks::yield_now` (`task.rs`);
//!   `Tasks::limit` shares FIFO counts across scopes and clones, with atomic acquisition of
//!   nested counts. A limited spawn is a checkpoint; cancellation starts nothing and a zero
//!   count panics on spawn. `Scope::set_deadline` lowers a soft deadline: expiry cancels,
//!   partial `Ok` results survive, and a subsequent child error under its own scope deadline
//!   becomes `TaskError::DeadlineExceeded`. Real deadlines are checked lazily and bound
//!   cancellable waits; a late wake reports `Cancelled`. `Tasks::shield` hides enclosing
//!   causes while inner scopes keep their own deadlines. `Tasks::spawn_isolated` waits
//!   shielded for a new owned task without taking a slot and returns `Panicked { message }`
//!   for an unwinding panic, with the process hook unchanged; aborts are not contained;
//! - `std::sync::Shared`: `new(v, &Tasks)`, `clone`, `same`, `with`, `with2`, `with_all`,
//!   and `reset`. Multi-cell calls acquire in global cell-id order with FIFO arrival per
//!   cell; aliases return `LockError::Aliased` before acquiring. Callbacks get payloads in
//!   caller order and a panic poisons every held cell. `reset` clears poison and drops the
//!   old payload after release. These operations work in cancelled scopes (03 §6.5);
//! - `Root::sim(seed)`, `std::sim::SimRoot`, and a seeded scheduler for structured tasks,
//!   `Shared`, files, and `Clock`, with
//!   source-located scheduling and wait records (03 §9.1) and virtual time, which starts at zero,
//!   jumps to the next timer only when every task is blocked, and ends in a deadlock report
//!   when none is left (03 §9.3). `SimRoot` keeps its calling thread as task 0 until it drops;
//!   `elapsed`, `records`, and `trace` inspect without scheduling. Sinks record ordered events
//!   instead of writing to host streams;
//! - `#[sim_test] fn t(sim: SimRoot)` (100 accepted cases) and `#[sim_test(seeds = N)]`
//!   (N accepted cases), using seeds from zero in order,
//!   one child process per test with a wall-clock backstop of 300 s per evaluation and empty stdin.
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
//!   failing case with `TIDE_TEST_SEED` set to the reported seed. Requires guide recognized
//!   shapes and filter every candidate, including in release test binaries. With runtime
//!   contracts enabled, direct callee requires failures discard a case;
//!   deeper ones fail. The runner accepts 100 cases within 10,000 generations or fails as
//!   vacuous, naming the conjunct with the most first rejections (source-order ties).
//!   Passing properties report accepted/generated; shrink attempts do not count.
//!   Extra `#[sim_test]` parameters use the same runner with `seeds = N` accepted cases
//!   within `100*N` generations, a fresh simulator per evaluation, and an unshrunk seed.
//!   A plain test's parameter spelled `seed: u64` receives that unshrunk case seed;
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
//! - real and simulated files: `Fs`, `Mode`, `OpenOptions`, `File`, directory listings,
//!   and type/length metadata (`fs.rs`); directory length is zero on both backends.
//!   Real `dir` confines paths with Linux `openat2`. Simulated handles use inode identity,
//!   sorted listings, and one seed-drawn latency of at most 1 ms per I/O call. Calls check
//!   cancellation before drawing; `File::close` is shielded. `sim.world.file` creates or
//!   replaces a file and creates parents immediately, without time or a scheduling point.
//!   Setup errors panic. Simulated compound operations apply whole; links, crashes and disk
//!   faults are not modeled. Real files reject use in simulation; simulated handles reject
//!   use outside their own simulation. Simulated `Rand` rejects Rand-backed hashing.
//!
//! # Contract implementation limits
//!
//! Mutable-reference aliases named bare in ensures require explicit `&mut T` spelling;
//! parenthesized reference types work. `old(&T)` requires the referent to be Clone/Copy.
//! Const checked clauses run during const evaluation and at runtime in debug/test builds,
//! and are erased in release. Const ensures supports receiver-free functions with identifier
//! parameters and Copy entry values, retaining enclosing generics and Self. Const old and
//! sampled const contracts in debug tests have explicit unsupported diagnostics; sampled
//! const clauses are type-checked but skipped in const evaluation/non-test builds.
//! Const violations use literal panics naming the clause's location: a const requires
//! violation names that location rather than the caller's, with no structured metadata
//! or clause-panic wrapping.
//! On a runtime function with ensures, a user-written `#[track_caller]` does not
//! propagate to the body; requires-only functions preserve explicit caller tracking.
//! In a user-tracked requires-only function, a failed requires in a function it calls
//! is blamed on the outer caller.
//! Runtime violation/unchecked records are local to the current OS thread; child-task
//! panic propagation does not transport them. Runners must treat missing child metadata
//! as an ordinary failure, never a requires rejection.
//! Clause classification is syntactic until admission refines it; unknown or unparsed
//! builtin macros remain untouched and classify as sampled. Unchecked-record printing
//! and generated contract tests remain planned.
//!
//! # Property implementation limits
//!
//! A called function's requires discards cases but does not guide generation.
//!
//! Guidance matches syntax, not resolved aliases: concrete integer/float ranges and scalar
//! endpoint relations, Vec/String/slice length bounds, indices below those lengths,
//! Vec/slice `is_sorted`, Vec/slice `contains(&x)` with primitive/String inputs or literals,
//! Option `is_some`, and scalar equality. Bounds use literals, concrete scalar parameters
//! (copied from borrowed inputs), and sequence lengths; unresolved constant paths only filter.
//! Other shapes and aliases only filter. The full
//! requires still runs after constructive generation, including while shrinking. Cyclic
//! dependencies disable guidance; contradictory bounds use ordinary generation.
//! Constructive sequences have at most 32 elements unless a larger minimum is requested;
//! minima over 4,096 fall back to ordinary generation. Guided Strings use ASCII to meet
//! byte lengths. Contains guidance needs a clonable primitive/String input; arbitrary
//! element types only filter. Float sorting may still reject NaNs through the full clause.
//! Ordinary collection generation allows 32 elements and four nesting levels; tuples
//! support twelve elements. Inputs need Debug. Shrinking budgets 1,000 evaluations,
//! stopping after 128 without improvement. Replay selects one generated case; a rejected
//! replay fails as vacuous. A should_panic property never discards requires failures.
//! Source-boundary classification uses current-thread runtime violation records; const
//! requires and child-task violations lack those records and fail normally.
//!
//! Suppressed failures, echoes and panics emit nothing at the default log level; their debug
//! record is deferred until the log sink exists. Discarded task values drop on the child or
//! receiver thread depending on timing, so a destructor panic is attributed to the thread that
//! runs it. Suppressed payload drops on the panic path follow PANIC-2 and can abort; on the
//! non-panic path, a drop panic surfaces in the scope's caller.
//! A panic while dropping an isolated task's panic payload aborts (PANIC-2).
//!
//! Not provided yet: other capability operations and attenuation of reference 6.1 (network,
//! random draws, environment, input, processes); `Root`'s `log` field;
//! the `dbg`, `log`, and `trace` sinks and `io::stdout()`/`io::stderr()`; `Secret`;
//! other `sync` items,
//! `par`, `arena`, `ptr`; other world setup, faults, nodes, replay manifests,
//! and generated contract tests. A
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
    pub use tide_macros::{requires, ensures, invariants, invariant};
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

/// Executable clause runtime; reserved for toolchain macros and test runners.
#[doc(hidden)]
#[path = "contract.rs"]
pub mod __contract;
