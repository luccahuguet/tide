# Tide

**A capability-safe, replayable subset of Rust.**

Tide is a checked subset of valid Rust 2024 for software that AI agents
write and humans have to trust. In the part of a program written in Tide,
a function's signature bounds what it can do, tests replay from a seed, and
reviews show what changed in what code may do, with Rust's speed and
safety. Each crate's trust artifact lists its `unsafe` sites, and the
bullets below state the other limits.

- **No accidental ambient authority.** Code the checker admits can touch
  files, the network, or the clock only through authority it is handed, so
  a function's signature bounds what it can do, and every site an `unsafe`
  grant admits is listed as a trust hole. Against a malicious dependency
  this rests on rustc's soundness, which has open `I-unsound` issues: the
  checker closes only the `fn`-pointer route of rust-lang/rust#25860
  (`TD0206`) and the trait-object route of rust-lang/rust#57893
  (`TD0402`), and the kernel sandbox for test and simulation runs (RUN-3)
  covers the runs `tide-admit` starts, in part (see the test runs'
  status). Today the driver admits a single crate against Tide's std;
  packages and dependencies (`tide.toml`, `tide.lock`) are planned.
- **Tests replay.** A test that takes a simulated root (`Root::sim(seed)`,
  `#[sim_test]`) runs under a seeded simulator of tasks, channels,
  `Shared`, files, TCP loopback, the clock, the environment, and stdin.
  A run through `tide-admit --tide-run-test` records a manifest, and
  `--replay` reproduces it bit for bit on the same machine, so concurrency
  bugs in Tide-written logic become ordinary failing tests. The simulator
  reproduces what it models: CPU time and real kernel behaviour are not
  modeled. Simulated HTTP, randomness and processes, fault injection,
  crashable nodes, and `tide sim` for whole programs are planned;
  production record and replay is v1.
- **Review what changed in what code may do.** `tide-admit --tide-diff`
  compares two builds' declared authority and `unsafe` holes, so reviewers
  see the risk before reading the code. Contracts and grants in the diff
  (`tide review`) are planned.
- **As safe and fast as Rust.** The same memory safety and data-race freedom
  without a garbage collector, and generics specialized like Rust's.
  Integer overflow never wraps silently, and no package code runs at build
  time beyond the toolchain's own proc macros and, from v1, the sandboxed
  build scripts of imported Rust crates.
- **Built for fast feedback.** No user proc macros, no build scripts, no
  inference across signatures, no unbounded trait solver. Checking an edit
  in tens of milliseconds is the target of Tide's incremental front end
  (D12), which is not built; today each check is a rustc run.
- **Already familiar.** Rust syntax wherever the meaning is the same,
  including lifetimes, without macros or `async`/`await`. Contracts run as
  checks and tests today and become proofs later.

*Tide is in the design phase. The bullets above state what the checker and
std deliver today, and Status below details it; the v0 core tier lists what
the design commits to, and `design/reference.md` labels each taught
surface not built yet.*

## Status

Tide is in the design phase. The decision record is `design/01-design.md`
(draft 3.7). Syntax is provisional until the E1 learnability experiment
meets its thresholds (D0). D18 makes Tide a checked dialect of Rust,
compiled by a pinned rustc against Tide's std; its feasibility spike
(`spikes/d18/`) passed, and draft 3.7 applies its changes to the other
decisions.

`cargo run -p tide-demo` runs the first demonstration, a partial-v0 one: a
lost update in a small concurrent Tide service found by a seed and replayed
identically, and a patch that widens the service's interface to carry `Net`,
flagged by the trust diff (`demo/README.md`). `cargo test --workspace` runs it
too. With gdb available, its last step debugs the failing seed's replay under
`rust-gdb`; [docs/debugging.md](docs/debugging.md) covers how Tide relates to
debuggers and to rr.

`examples/` holds small Tide programs, each with a README on what it shows
and what the Rust version would need instead. Each program declares its
outcome (it runs with given output, its tests pass, a test fails, or
admission rejects it with a named code), and `cargo test --workspace` checks
every declaration (`crates/tide-admit/tests/examples.rs`).

The v0 checker has started, and its first piece is production code:
`crates/tide-admit`, the admission driver of D18's architecture A. It is a
rustc driver that enforces the admission allowlist (ADMIT-1, `TD0520`),
decides `Pure` structurally under the codes of the rules that need it (D18:
statics, `const fn`, `#[pure]`, `Shared`, parallel closures, and `TD0517`),
requires the `unsafe` grant for unsafe code (UNSAFE-1, `TD0901`), requires
`#[tide::main]` on a binary's `main` (ROOT-1, `TD0512`), rejects code that
calls or names `main` (ROOT-5, `TD0506`), and passes rustc
`-Zallow-features=` and `-Zpolonius=no` (ADMIT-2). After rustc's
analysis, a backstop checks the crate's instances that can run for denied
impls and `Pure` where-clauses, whatever route reached them (ADMIT-1). It constructs
every compiler invocation, clears the compiler environment (keeping only
`RUSTC_ICE=0`, so a compiler crash writes no report file), snapshots candidate
Rust sources, rejects file-loading attributes (`TD0911`), user macro definitions
(`TD0101`), derives and bang forms outside the closed sets (`TD0102`, `TD0103`),
lifetime parameters on traits (`TD0201`), unresolved output elision, including
function types and `Fn` sugar (`TD0202`, LIFE-4), and lossy numeric casts (`TD0301`). It labels rustc's borrow errors for closures in
owned positions when the evidence is certain (`TD0203`) and struct literals that
move an owner into one field and borrow it in another (`TD0210`). Wrong-input elision keeps rustc's
region error. It rejects coercing a function item to a `fn` pointer when the item implies an
outlives bound that does not hold trivially (`TD0206`, CLOSURE-3), whatever the instantiation,
which closes the `fn`-pointer route of rust-lang/rust#25860. Generic functions over references,
trait methods with a reference receiver such as `Clone::clone`, `&self` methods of types with a
lifetime parameter, and functions such as `fn f(v: &mut Vec<&str>)` therefore do not coerce; a
closure that calls the item is accepted. It enforces the trait power ceiling
(`TD0401`–`TD0408`): impl placement, specialization and overlap (an impl for a type
parameter or a projection that may be unsized may overlap the built-in impl of its trait's `dyn`
type when the trait declares an associated type a `dyn` type can name, the route of
rust-lang/rust#57893), generic associated types,
explicit higher-ranked binders, negative impls and user auto traits, structural impl
bounds, polymorphic recursion and duplicate inherent method names. Definition
checks reject expanding cycles between function type parameters, including
selected local trait impls. Unresolved dispatch retains rustc's checks;
instance-depth failures alone do not receive `TD0407`.
The authority checks reject ambient operations from a closed policy table (`TD0501`),
forbidden naming or whole-value use of `Root` (`TD0503`), simulation outside
test code (`TD0504`), and handle creation missing the final `&Tasks` (`TD0505`).
Field moves, borrows and patterns are allowed. Test code includes test-only cfg ancestors,
plain/property/simulation tests and the separate test suite; ordinary helpers
in a `--test` build and the separate candidate library remain production code.
The concurrency and error checks reject async/await and core's future/pin APIs
(`TD0601`), unstructured spawning (`TD0602`), Mutex/RwLock habits (`TD0604`),
`select!` (`TD0605`), `catch_unwind` (`TD0701`), and a `Result` ignored in
statement position (`TD0702`). Explicit `let _ = result;` is accepted; other
`#[must_use]` types retain rustc's lint. `#[allow]` does not silence TD errors.
It also rejects `#![no_std]` (`TD0520`) and package build scripts
(`TD0915`), and caches successful checks/builds by source and trusted-input
content. A crate it admits against Tide's std also gets a trust artifact beside its metadata,
`<crate>.interface.json` (D11, 03 §4.1): each exported item's declared
authority, with the access path that carries each kind (`app.db.net: net`, or
`opaque(..)` where a type hides it), and the crate's trust holes, every site
the `unsafe` grant admits. It reports no exercised authority, effects, or proof
status. A crate whose `std` is neither Tide's nor Rust's fails the check; one
built on Rust's std has no capabilities and gets no artifact; and a failed
check leaves no artifact. `tide-admit --tide-diff BASE HEAD` compares two
artifacts: the items added, removed, or whose declared authority widened or
narrowed, with the paths that changed, and the holes added, removed, or
recounted; it prints nothing when neither changed. It is a partial precursor
to `tide review`'s trust diff (03 §4.3): no exercised authority, effects, or
proof status. The root Cargo workspace builds the driver with the nightly
that `rust-toolchain.toml` pins, with the `rustc-dev` component:

```
cargo test --workspace          # the driver's, Tide's std's, the demo's, and the runner's tests
cargo run -p tide-conformance   # the conformance corpus, through stable rustc and the driver
```

`cargo test --workspace` and the corpus also need the `stable` toolchain
(`rustup toolchain install stable`) and `/usr/bin/python3` with MPFR
(`libmpfr.so.6`; on Debian/Ubuntu, `python3` and `libmpfr6`), which the float
oracle uses.

The trusted build caller invokes the driver directly, for example:

```
target/debug/tide-admit --crate-type=lib --out-dir /tmp/tide-output path/to/package/src/lib.rs
```

The interface accepts one `.rs` entry, `--edition 2024`, `--crate-type lib|bin`,
`--crate-name NAME`, `--test`, `--emit metadata|link`,
`--tide-profile debug|release`, `--error-format short`, `--out-dir DIR`, and
`--tide-grant=unsafe` (value options also accept `=VALUE`).
`--tide-test-backstop`, for the driver's tests, skips the obligation steps of
the checks before analysis, so the backstop alone decides those routes, and
`--tide-test-ice` crashes the compiler after expansion.
Raw rustc flags and response files are rejected. The source directory is the
entry's parent; when it is named `src`, its parent is the package directory.
The package path must not traverse directory symlinks, and a resolved source
directory must stay beneath that package.
Output and `.tide-cache` state must be outside that package. Only canonical
`.rs` descendants of the source directory are served to rustc; committed
symlinks cannot grant access outside it. Each loaded source is checked before
expansion, and the expanded crate after it, which catches attributes and macro
definitions written in macro arguments. The cache records source bytes and
directory membership, driver and pinned compiler identity, grants, normalized
arguments, and trusted artifacts; failed checks/builds never become cache hits.
Metadata (the default), linked binaries, and linked test harnesses have distinct
cache identities, as do debug and release profiles. Both profiles force
`-C overflow-checks=on`; debug (the default) uses optimization 0 and debug
assertions, release uses optimization 3 without debug assertions.
Every profile, including test builds, passes `-Zinline-mir=no` and
`-Zcross-crate-inline-threshold=never` so MIR inlining cannot hide instances
and inferred cross-crate inlining cannot remove exported library roots from
the backstop. Explicit `#[inline]` functions and generics are checked where
instantiated; LLVM inlining still runs after the check.
Unsafe-granted link builds always compile afresh and discard their staging
directory after publication: assembly and native links can read inputs the source
snapshot does not hash. Their metadata checks can still be cached.

`--emit=link` requires `--crate-type=bin` or `--test`. On the supported
`x86_64-unknown-linux-gnu` host it continues the same compilation past admission
and the mono-item backstop, then invokes fixed `/usr/bin/cc` with `/usr/bin/ld`
(GNU cc flavor, `-B/usr/bin`, bundled linker disabled). GCC also passes
`-L/usr/bin` to the linker for `-B/usr/bin`. It writes
`<out-dir>/<crate-name>`, or `<crate-name>-test` for `--test`, only on success;
it removes that invocation's previous executable
before checking, so a rejected rebuild leaves no stale executable at that path.
Replay leaves published outputs intact. The shared `<crate-name>.interface.json` describes the latest invocation. Use
separate `--out-dir` values for profiles and bin/test modes to retain their
artifact pairs. rustc's codegen and link temporaries stay in private cache staging;
host tools may use their own temporary paths. Building alone never executes an
output; the explicit test/replay modes below run reviewed code. For a demo build:

```bash
mapfile -t TIDE_LIBRARIES < <(
  cargo build --offline --locked -p tide_std --message-format=json |
    grep -o '"filenames":\[[^]]*\]' |
    grep -o '"[^"]*\.\(rlib\|so\|dylib\|dll\)"' | tr -d '"'
)
TIDE_DEP_ARGS=()
for lib in "${TIDE_LIBRARIES[@]}"; do
  TIDE_DEP_ARGS+=(--tide-dep-dir "$(dirname "$lib")")
  if [[ "$lib" == */libtide_std.rlib ]]; then TIDE_STD="$lib"; fi
done
target/debug/tide-admit --crate-type=bin --emit=link --crate-name=hello \
  --tide-std "$TIDE_STD" "${TIDE_DEP_ARGS[@]}" \
  --out-dir target/demo path/to/demo/src/main.rs
target/demo/hello
target/debug/tide-admit --test --emit=link --crate-name=hello \
  --tide-std "$TIDE_STD" "${TIDE_DEP_ARGS[@]}" \
  --out-dir target/demo-tests path/to/demo/src/lib.rs
target/demo-tests/hello-test
```

In Bash, `TIDE_LIBRARIES` collects every library filename from Cargo's
`compiler-artifact` messages. `TIDE_STD` selects the std artifact and
`TIDE_DEP_ARGS` passes each library's directory (including macros and libm)
with `--tide-dep-dir`, independent of Cargo's build layout.

Execution is a separate trusted caller action. `--test` builds a libtest
binary with ordinary and parameterized `#[test]` entries. A parameterized
test accepts 100 cases by default and reports the seed and shrunk input
when a case fails. Its own `#[requires]` guides recognized shapes and filters all
cases, including in release test binaries; a directly called function's failed requires
discards that case when runtime contracts are enabled. The run
fails as `vacuous` if fewer than 100 cases are accepted within 10,000 generated
cases, naming the conjunct with the most first rejections. Passing runs report
accepted/generated. Its parameters need `Debug`; a user-defined generator
implements `std::test::Arbitrary`. List tests with `--list`. For a fresh rerun of a case,
set `TIDE_TEST_SEED` to the reported seed and run the test binary with
`--exact <test-name>`.
Structural generators are inserted into the defining modules before the final
admitted compilation. Insertion preserves original line numbers; columns of
following tokens on an insertion's physical line can shift.
Executable `#[requires]`, `#[ensures]`, and loop `invariant!` clauses lower through
production macros and std. Checked clauses run in debug/test builds; sampled clauses
run in the crate under test, with a 10,000-binder-value budget. Release builds
type-check clauses but execute neither checks nor snapshots. The `tide test` command and
remaining simulated capabilities are planned. Driver test builds generate one contract test
per ensures-bearing function, under `__tide_contract_tests`. They use the property
runner's guidance, filtering and vacuity floor. Ungeneratable signatures are
ignored with `unchecked: <reason>`; `#[allow(vacuous, reason = "..")]` passes
a vacuous entry and reports `unchecked (vacuous, allowed: ...)`. Use
`--show-output` or `--nocapture` to see successful entries' captured reports.

Const requires violations name the clause's location rather than the caller's.
The [std crate docs](crates/tide-std/src/lib.rs) own the supported contract forms
and current implementation limits.

Add `--tide-test-library path/to/library/src/lib.rs` to `--test` to check a
separate test crate against a library. Pass the test crate's source as the
positional argument. Its package must be disjoint from the library package,
and it imports the library as `tide_candidate` under Rust's crate visibility
rules. The driver admits both crates without grants and builds a fresh library
archive in private staging. Metadata mode checks the suite and `--emit=link`
publishes the test executable. The pair publishes no library or trust artifact.
The library is built without `cfg(test)`, but its contracts run as in a test
build, sampled clauses included.
A suite's property tests generate the suite's own types and any library type
for which the library implements `std::test::Arbitrary`. A property over
another library type is an error.
A killed build can leave a private `.tide-test-*` directory. Later invocations
create fresh staging and never read those leftovers.

`--tide-std PATH` and repeatable `--tide-dep-dir DIR` select trusted toolchain
artifacts outside the candidate package. These are operator inputs, like
grants; candidate source, Cargo configuration, toolchain files, and inherited
environment never set them. Without `--tide-std`, Rust's pinned std is used
for driver development tests, and `TD0512` is not checked, since Rust's std
has no `#[tide::main]`. Tide's std is `crates/tide-std`, with its macros in
`crates/tide-macros`: so far `#[tide::main]`, `Root` and the capability types
(`Rand` and `Proc` remain identities), `Pure`, and the `out`, `err` and log sinks
(`print!`, `println!`, `eprint!`, `eprintln!`, `error!` through `trace!`), ordinary and property `#[test]`, `#[sim_test]`,
the prelude's `Error` derive with thiserror semantics (without `error(fmt = ..)`,
backtrace detection/`#[backtrace]`, or `Error::provide`; mixed transparent and
unannotated enum variants with manual Display are also unsupported),
the prelude's `Encode` and `Decode` derives with `#[codec(rename, rename_all,
default, skip)]` and `std::json` (`encode`, `decode`, `json::Error`; JSON only,
with no other serde key),
and the thread task runtime (`Tasks::scope`, `Scope::spawn`,
`Task::join`, `Scope::cancel`), with scoped borrows, automatic joins, TASK-2
failure precedence and cancellation echoes, and level-triggered
`Tasks::checkpoint` / `Tasks::yield_now`. Checkpoints return `task::Cancelled`,
convertible to `io::Error` with `ErrorKind::Cancelled`; map explicitly to
`TaskError::Cancelled` when needed.
Unlimited spawn and joins are not checkpoints; in-memory I/O does not suspend
and ignores cancellation. `tasks.limit(n)` shares a FIFO concurrency bound
across its scopes and clones; nested limits take all their slots together.
A limited spawn is a checkpoint and starts nothing when cancelled; spawning
under a zero count panics. `Scope::set_deadline(t)` only lowers the deadline:
expiry cancels, successful partial results remain `Ok`, and a child error
after its own scope deadline becomes `TaskError::DeadlineExceeded`.
Real deadlines are detected lazily at checkpoints and bound cancellable waits;
a late wake returns `Cancelled`. `tasks.shield` hides enclosing cancellation,
while inner scopes keep their own deadlines. `tasks.spawn_isolated` runs an
owned closure in a new task, waits shielded without taking a limit slot,
and returns `Panicked { message }` for an unwinding panic. The process panic
hook still runs; aborts and stack overflows are not contained.
Suppressed failures/echoes/panics emit one count-only debug record per scope,
filtered at the default log level. Discarded task
values drop on the child or receiver thread depending on timing, so a destructor panic
is attributed to the thread that runs it. Suppressed payload drops on the panic
path follow PANIC-2 and can abort; on the non-panic path, a drop panic surfaces
in the scope's caller.

`std::sync::Shared::new(v, &tasks)` creates a cell with a `Send + Pure` payload.
Clones refer to the same cell; `Shared::same(&a, &b)` tests identity without
acquiring. `with`, `with2`, and `with_all` serialize `Pure` callbacks, including
in cancelled scopes. Multi-cell calls acquire in cell-id order, with FIFO arrival
at each cell, and return `LockError::Aliased` without running the callback when
an operand repeats. `with_all` passes payloads in the caller's order and runs
on an empty slice too. A callback panic poisons every held cell; later access
returns `LockError::Poisoned`. `reset(v)` replaces a healthy or poisoned payload
and clears poison, dropping the old value after release.

`std::sync::mpsc` provides `channel(&tasks)` and `sync_channel(n, &tasks)`
for unbounded and bounded FIFO channels on both runtimes; zero capacity is a
rendezvous. Senders clone, bounded senders support `try_send`, and receivers
support `recv`, `try_recv`, `recv_deadline`, `iter`, `try_iter`, and owned or
borrowed iteration. `recv`, `recv_deadline`, and every bounded `send` check
cancellation, even when ready; a cancelled receive preserves buffered values.
A zero-capacity `try_send` hand-off to a blocked receiver is already accepted
and is returned despite cancellation.
The receive/send error enums distinguish `Cancelled` from `Disconnected`,
with `is_cancelled()` and `SendError::into_inner()`; deadline receives also
report `Timeout`. Disconnect ends iteration. Simulation records payload-free
`chan` events and reports blocked channel waits through the shared deadlock report.

`std::par::join` and the prelude's `ParSlice` methods `par_map`, `par_for`, and
stable `par_sort` execute authority-free work with owned scoped helpers. Results
and sink output follow sequential order; when branches terminate, the lowest-index
panic propagates and later sink buffers are discarded. Simulation executes on the
calling task in order, without region scheduling events. Helpers share a process-wide core budget
and take no task-limit slots. Scheduling is partial against 03 §8.4: the
work-stealing pool and `par-threads` setting remain planned. Branch buffers are
unbounded, and losing branches' host panic-hook output is not suppressed.
Stop flags are per call; propagation across calls is partial. `join` and nested
regions run an already spawned right branch to completion after a left panic. A
right branch that never terminates diverges, and whether the region panics or
diverges can depend on machine load.
Every fork finding a free token spawns an OS thread, so fine-grained recursive
`join` is slow until the planned pool. Sort forks only for ranges of at least
4096 elements; smaller ranges follow the same tree inline. Helper-thread panic
metadata is not returned to the property runner: on the thread runtime,
scheduling can turn a `requires` discard into a failure.
Production and conformance use the same implementation.

`root.clock` reads time and sleeps: `now() -> Instant`, `system_time() ->
SystemTime`, `sleep(d)`, and `sleep_until(t)`. `std::time` has Rust's
`Duration`, and `Instant` and `SystemTime` with Rust's arithmetic and no
`now()` or `elapsed()`. A sleep is a checkpoint: it returns `task::Cancelled`
in a cancelled scope, and a cancel ends one in progress. A sleep whose target
is not after now does not block, and a sleep whose target overflows ends only
by cancellation. `monotonic_only()` gives a `Clock` whose `system_time()` panics.

Process I/O executes through the same driver: `root.env` reads real arguments,
variables and cwd; `only` intersects variable names and `args_only` denies variables
and cwd. `args`/`vars` are owned iterators that panic on a non-Unicode item;
`var` returns `VarError`. No `_os` forms are provided. `root.stdin` supports
`read_line`, `lines`, `read_to_string`, `read_to_end`, and `is_terminal`,
and implements `io::Read` owned or borrowed.
Reads check cancellation at entry; a real read blocked in the kernel waits for
input/EOF. `lines()` moves one line per fill so dropping it preserves later input.
`io::stdout()`/`stderr()` implement `io::Write` through the same synchronous sinks
as the print macros; they always return `Ok`. There is no `lock()`; batch with
`BufWriter::new(io::stdout())`. Unflushed writer Drop discards bytes and warns
with their count. Closed stdout exits quietly with 141. `main` uses Rust's
`Termination`: taught forms are `()` (0), `Result<(), E: Debug>` (1 on Err), and
`ExitCode` (given status); an unwinding panic exits with 101.
`root.log: std::log::LogConfig` has `&self` setters `level(Level)`, `format(Format)`,
and `to_file(File)` for the log sink. Defaults are Info/text/stderr. Arguments
always format before filtering. Text is `level: message`; JSON has fixed
`level`/`message` keys and no timestamp (provisional). The trace sink,
`log::`/`tracing::`-prefixed log macros, and `release-max-level` remain unimplemented.

Production std includes a seeded scheduler for structured tasks,
`Shared`, channels, TCP loopback, and `Clock`. It runs one task at a time between
scheduling points and records decisions and waits with source locations.
Virtual time starts at zero and jumps to the next timer only when every task
is blocked; with no timer left, the run ends in a deadlock report.
`Root::sim(seed)` in a test returns `std::sim::SimRoot`: its calling thread is
task 0 until the root drops. `sim.tasks`, `Shared`, channels, `sim.fs`, and
`sim.clock`, `sim.net`, `sim.env`, and `sim.stdin` work; `sim.elapsed()`, `sim.records()`, and `sim.trace()` inspect virtual time,
ordered sink records, and provisional NDJSON events without scheduling. `out`, `err`, and every log level append raw sink events under simulation and do not write to host streams.
`sim.world.file(path, contents)` creates or replaces a file and its parents
immediately, from simulated `/`, without scheduling or virtual time. Invalid
setup panics. `sim.world.args` appends arguments (including a supplied program
name), `env_var` overwrites a key, and `stdin` appends bytes; unset inputs are
empty. Env/input operations schedule with zero latency and no random draw.
`records()` assembles lines per stream, strips newlines, converts bytes lossily
to UTF-8, and emits unterminated tails last; the node is `main` outside nodes.
`SimRoot` has no log configuration. Other capabilities carry simulated identities; Rand-backed hash
initialization panics until its simulated backend exists.

Without requires discards, `#[sim_test] fn t(sim: SimRoot)` runs seeds `0..100`;
`#[sim_test(seeds = 20)]` runs `0..20`. Each entry re-executes its driver-built
test binary once with an exact libtest filter, empty stdin, and a wall-clock
backstop of 300 s per evaluation (under the driver's run modes, the driver
runs the evaluation in its sandboxed child and keeps the backstop). The child catches panics per seed, checks the first and
every failing seed twice for identical traces/output, and prints failing seeds,
causes, and traces on failure. Success emits at most one summary line, captured
by libtest; seed-start records stay internal. Signal/timeout or a missing summary
names the last started seed; ordinary failures name the failing seeds. The
internal `TIDE_SIM_SEED=<decimal u64>` hook selects one fresh seed when running
the test binary; it does no build/source identity validation. Simulated HTTP/randomness,
other world setup and faults remain planned. Extra parameters, such as
`#[sim_test] fn t(sim: SimRoot, sizes: Vec<u16>)`, use the property runner's
guidance, filtering, vacuity floor and shrinking. Each case and shrink attempt
gets a fresh simulator with the same case seed; only the parameters shrink.
A plain property's parameter spelled `seed: u64` also receives the case seed
and keeps it while shrinking.
A test binary run directly gets no OS sandbox; the driver's run modes below do.

The driver supplies a partial precursor to `tide sim` for one exact test:

```sh
tide-admit --test --crate-type=lib --emit=link \
  --tide-std /operator/libtide_std.rlib --tide-dep-dir /operator/artifacts \
  --out-dir /operator/out package/src/lib.rs \
  --tide-run-test module::test_name --seed 47
```

`--seed` selects a fresh evaluation of current source; omit it to use the test's
normal case range. Driver execution clears child environments and supplies only
explicit harness controls. Each evaluation writes an exclusively reserved
`OUT/target/sim/<run>/run.manifest.json` in format `tide-replay/2`, with a sibling
`trace.ndjson`. The initial manifest precedes user evaluation; choices and
source-defined root setup (arguments, environment, stdin and file bytes) append
before application. An aborted evaluation keeps an incomplete manifest.
Ordinary tests record ordered `Root::sim` calls without replacing source-written seeds.
Nested property and simulation test calls run within the caller's evaluation;
their generated choices and roots do not create a separate manifest or enter the
caller's input tape. Nested calls use their configured deterministic case range;
driver seed/replay controls select only the outer evaluation. A nested simulation
root's events are not part of the caller's trace, and its seed-start records stay
internal. Const tests and a test-decorated `main` retain native libtest
execution without recording and cannot run through `--tide-run-test` or `--replay`.
A `#[sim_test]` called from inside a simulated task fails with "a task runtime is
already active".

Linked test builds (including a paired test library) remap embedded source paths
to `tide-source` and `tide-library`: this covers panic locations and
macro-supplied trace spans, so moving a source tree preserves replay locations.
The scope is `--remap-path-scope=macro,object`; candidate diagnostics, debug-info
and coverage paths retain real spellings, as do non-test builds. This is path
normalization for same-machine replay, not a promise to remove all physical paths.

Failures print an executable, shell-quoted `tide-admit ... --replay <manifest>`
command using the same explicit build arguments. Replay validates format and
digests, current source snapshots (including a paired library), compiler,
target, profile, semantic options, hashes of the dependency libraries the build loaded, and test
configuration before starting any candidate process. The driver pairs each
executable with a `<crate>[-test].build.json` receipt and verifies its bytes.
Replay runs private build artifacts and leaves published outputs unchanged,
including on refusal.
Replay consumes the exact typed parameter choice tape once, without generation
or shrinking, and compares the exact NDJSON bytes and stable outcome with the
recording. A reproduced failure remains a failing test and prints `replay reproduced`.
The primary artifact belongs to the original evaluation, independently of the
minimized input reported afterward. Tape exhaustion, leftovers, changed kinds,
or divergent root setup, trace or outcome fail replay.

`--replay` accepts neither `--seed` nor a test/configuration override; there is no
force option. Use fresh `--seed` after edits. Direct test-binary invocation has
no driver-verified manifest guarantee. Replay currently promises the same
machine with identical pinned trusted artifacts, sysroot and C runtime;
dependency hashes describe supplied artifact contents, not source provenance.
Fault plans, durability, crash models, signing, and the `tide sim` frontend
remain outside this path.

**Sandbox (RUN-3, in part).** `--tide-run-test` and `--replay` start the test
binary in a new session with core dumps off (soft and hard limits of one byte),
no inherited descriptor but empty standard input and standard output and error
on pipes the driver drains, a cleared environment, and, between fork and exec,
Landlock and seccomp allowlists (`crates/tide-admit/src/invocation/sandbox.rs`).
Each run gets a fresh private directory under `OUT/.tide-cache`, created and
opened without following links. Landlock lets the child read and execute only
the binary's copy there and the loader and libraries it names, read its
context, and write only in the run directory's `sim/`, creating regular files
and directories; its scopes keep signals and abstract Unix sockets in the
child, and no TCP or UDP port is granted. seccomp fails every call the runtime
does not need with `EPERM`, sockets, process creation, io_uring, links,
metadata changes, and tracing among them. When the child exits, the driver
moves to `OUT/target/sim/` only the manifest directories whose names start
with the run's token, never replacing an existing entry, and removes the run
directory. A link or file at `target` or `target/sim` fails the publication
and is left in place, neither followed nor deleted. A `#[sim_test]` runs inside that child
rather than in a grandchild; the driver reads its seed starts and keeps the
300 s per-evaluation backstop (`--tide-eval-timeout SECS` sets another), and
a timeout or a fatal signal names the last started seed, even when whoever
reads the driver's standard output has stopped reading. Before the child
starts, the driver prints the level to standard error, with the kernel release
and Landlock ABI:

- `sandbox: full (Linux 7.2.6-…, Landlock ABI 10)`: Landlock ABI 6 or later
  and seccomp filters;
- `sandbox: partial, not enforced: … (…)`: names Landlock (absent or disabled
  at boot), Landlock's scopes (ABI 1 to 5), or seccomp, whichever the kernel
  lacks. The run continues under what remains, and RUN-1 then rests on the
  checker and compiler alone.

A host whose hard core-size limit is zero, a kernel without `close_range` and
`CLOSE_RANGE_CLOEXEC` (Linux 5.11), or a child that cannot install the level
the driver reported fails the run before the test starts.
`run3_kernel_denies_an_unadmitted_test_child_paths_and_sockets` in
`crates/tide-admit/tests/build.rs` swaps an unadmitted program into the build
cache and checks that the kernel denies its file reads and writes, a canary
and another run's artifact behind a planted `target/sim` link, and its TCP
connect, and that truncating its standard output leaves a redirected file
intact. The child gets `PR_SET_PDEATHSIG`, so it dies with the driver even
though its new session puts it outside the caller's process group. Off x86_64
Linux the driver still builds, with the sandbox compiled out (`none`), and its
run modes stay unavailable there, as linking already is. Still missing from
RUN-3: escaping the child's output (RUN-6), the level in `tide check` and
`tide review`, and RUN-2's build sandbox.

The `std::io` core over in-memory readers and writers is shared with conformance
through `crates/tide-std/src/io/shared.rs`. It follows Rust's supported API with
`BufWriter<W: Sized>`; per TASK-6, Drop discards pending bytes and warns with their count.
`Lines`, `Split`, and `Bytes` end after yielding `ErrorKind::Cancelled`.
`std::net` provides `Net::bind/connect`, `TcpListener::accept/incoming/local_addr`,
`TcpStream` with owned/borrowed `io::Read` and `io::Write`, address accessors and
`shutdown`. Real Linux sockets are nonblocking; socket waits observe cancellation and
scope deadlines. `incoming()` yields `Cancelled` once, then ends. `only(hosts)`
intersects literal case-insensitive host/port grants without lookup, affects only
outgoing connections, and `no_bind()` denies binding with `PermissionDenied`.
Address arguments are sealed data; IP literals and `localhost` work on both backends.
Other real names use Rust's inline resolver, which cannot yet be abandoned on
cancellation; simulation rejects them. `sim.net` models `127.0.0.1` (including
wildcard listeners), deterministic ephemeral ports, bounded streams, half-close and
EOF. Its 128-connection backlog, 65,536-byte directional capacity, 49152–65535
port range and 1–10 ms packet delays are provisional. One admitted suite compares
values/ErrorKinds on real `127.0.0.1:0` and simulated loopback, excluding ports and
timing; timing-dependent post-close errors use allowed sets. The model and
limits follow [the TCP research](research/21-channels-tcp.md#19-net-shape-addresses-and-attenuation).
HTTP, UDP, `Net::resolve`, owned I/O, socket options, clone and timeouts remain planned.

`std::fs` provides real and simulated files through the same `Fs`: path reads
and writes, `OpenOptions`, `File` with `Read`/`Write`/`Seek`, directory mutation
and listings, type/length metadata, and explicit sync and close.
`dir(path, Mode)` pins a directory;
confined paths use Linux 5.6+ `openat2` and fail closed with `Unsupported` when
unavailable. Absolute paths and escapes return `PermissionDenied`; `Mode::Read`
rejects writes and cannot widen to `ReadWrite`. Listings are snapshots sorted
by filename bytes. Directory metadata length is zero on both backends.
File operations check cancellation; `File::close` is shielded. Each simulated I/O call takes one scheduling point and a seed-drawn
latency of at most 1 ms, including failed calls and `close`; pre-cancelled calls
take no time or draw. Attenuation and data-only accessors do not schedule.
Open handles retain identity after rename/removal. Simulated errors carry only
kinds, with no OS code. The model has no links, disk crashes, fault injection or
partial effects within a compound operation; sync validates and takes time.
Real files panic inside a simulation, and simulated handles reject use outside
their own simulation. The conformance std includes production's implementation.
Metadata times/permissions and owned-buffer I/O remain planned.
`std::collections` keeps Rust's API, with fixed-key `HashMap`/`HashSet` and
`Rand`-seeded hashing.
The shared `std::float::{bits, min, max, total_cmp}` observers canonicalize NaNs
and order -0.0 below +0.0. `std::math` replaces the banned inherent math methods
for `f32` and `f64`, using pinned `libm` 0.2.16 with `force-soft-floats` on
every supported target; non-SSE2 x86 is rejected because x87 native arithmetic
does not meet D3. The functions document their deterministic compositions.
The prelude's sealed `std::convert` traits provide `x.truncate()` (same-signedness
low bits), `f64::round_from(n)` / `f32::round_from(x)` (nearest, ties to even), and
`i64::try_from_float(f.trunc())?`. Float-to-integer conversion rejects non-finite,
fractional, and out-of-range inputs; `-0.0` becomes zero. Float conversions can
produce infinity, including `f32::round_from(u128::MAX)` and `f32::round_from(1e39_f64)`.
Rust's bit-preserving `cast_signed()` / `cast_unsigned()` remain admitted. With Tide's
std injected, the driver allows `unstable_name_collisions`: the pinned prelude's
`Truncate` deliberately shares core's unstable inherent method name.
`std::arena::{Arena, Id}` supports cyclic graphs with generational handles,
last-freed-first slot reuse, checked disjoint mutation and stale-index panics.
`Id<T>` is `Pure` for every `T`; `Arena<T>` is `Pure` when `T` is. Handles must
stay with their originating arena; no cross-arena check is performed. `iter`
(slot order), `Id`'s `Ord` and `Debug` remain provisional. The prelude's
`#[pure]` asserts authority-free function parameters under the existing TD0511
check; non-functions and nonempty arguments are compile errors.
The crate docs list what it does not provide yet.
The conformance runner injects
a stub std that shares these definitions, including the production `#[test]`
macro, property generation/shrinking runtime, and executable contract lowering/runtime.
The generated-test synthesis pass also operates against this shared std. The
installed pinned toolchain is trusted and treated as immutable, as are the host
C toolchain, startup objects and libraries. Link cache keys also hash `/usr/bin/cc`
and `/usr/bin/ld`; they do not fingerprint all native transitive inputs.
This slice provides loader and invocation isolation, with no OS sandbox or protection against concurrent
filesystem mutation. Granted assembly can read outside the loader (`.include`,
`.incbin`); native link inputs can read host files, and the linker/toolchain can
start host tools. A caller running an output directly gets no Landlock/seccomp
policy, descriptor/environment/core-dump controls, simulated Root, or
snapshot/coverage gate from this driver; only the run modes' children get
RUN-3's sandbox. Builds and
runs are for reviewed/trusted demo code only: the operator must review source,
grants and dependencies, including granted assembly/native inputs, and trust the
toolchain and host. Admission is partial. This mode does **not** satisfy RUN-5 or
provide RUN-1's unreviewed-branch guarantee.

[conformance/](conformance/README.md) holds the conformance corpus: cases
with expected verdicts per rule ID. Stable rustc checks their rustc marks,
and the driver checks their marks for the codes it decides. `spikes/d18/`
holds the throwaway feasibility spike for D18, and the early Rust scaffold
from the first design, which does not implement Tide, is kept in
`archive/v0-design/scaffold/`.

## Reading Guide

### Design

- [design/00-first-principles.md](design/00-first-principles.md): the problem
  statement, hard constraints, measurable targets, and hypotheses H1–H7 with
  their verdicts.
- [design/01-design.md](design/01-design.md): the decision record (D0–D18)
  and the tier table; it owns every decision.
- [design/reference.md](design/reference.md): the agent-facing reference for
  the v0 core, written as a delta from Rust, within a 25,000-token budget,
  labelling what is not built yet;
  [design/reference-companion.md](design/reference-companion.md) holds its
  on-demand tables, examples, and later tiers.
- [design/02-memory.md](design/02-memory.md): ownership, origins, variance,
  drop check, `Send`, `Sync`, and `Pure`; the definition the checker
  implements.
- [design/03-authority-concurrency.md](design/03-authority-concurrency.md):
  capabilities, structured tasks, parallelism, the simulator, the Bend
  boundary, and Rust import.
- [design/04-contracts.md](design/04-contracts.md): contracts, proof modes,
  and the trust surface.
- [REFERENCES.md](REFERENCES.md): the languages, projects, papers, and data
  Tide draws on.

Chapters 02–04 elaborate draft 3.7 and label which sections are v0 core and
which are later tiers; on any conflict, `01-design.md` and its tier table
win.

### Research

Cited evidence behind the decisions, in [research/](research/):

- `00-old-design-salvage.md`: what to keep, drop, and re-decide from the
  archived first design.
- `01-bend-hvm.md`: Bend, HVM, and interaction nets, measured against native
  code.
- `02-memory-models.md`: memory-safety and aliasing models, from Rust to Hylo
  and Mojo.
- `03-compile-speed.md`: which language decisions set the ceiling on check
  and build speed.
- `04-verification.md`: practical program verification and how well LLMs
  produce verified code.
- `05-llm-code-empirics.md`: which language properties help or hurt LLM
  coding agents.
- `06-agent-native-tooling.md`: agent-first languages and the toolchain
  contract agents need.
- `07-effects-capabilities.md`: effect systems, capabilities, and purity
  tracking.
- `08-type-system-core.md`: types, generics, traits, and metaprogramming.
- `09-adoption-interop-backends.md`: adoption, interop, and backend and
  bootstrap strategy.
- `10-concurrency-parallelism.md`: concurrency, deterministic parallelism, and
  deterministic simulation testing.
- `11-bend-interop.md`: how Tide and Bend 2 divide the work.
- `12-newcomer-sweep.md`: 2025–2026 languages and tools, the ideas Tide
  adopted from them, and lust, a checked Rust subset with fast edit checks.
- `13-agent-fleets.md`: what people running many agents and subagents need
  from a language, ranked, and where Tide meets or misses each need.
- `14-autoresearch.md`: Karpathy's autonomous experiment loop, why it does
  not fit Tide's design work, and where it fits once the checker exists.
- `15-rust-dialect.md`: whether Tide should ship as a checked dialect of
  Rust, compiled by stock rustc against Tide's std; per-feature
  classification, experiments, Eon as a test case, and a recommendation.

### Reviews and Trials

- [reviews/](reviews/): adversarial reviews, one file per reviewer and
  round, named `<bead>-r<N>-opus.md` or `<bead>-r<N>-codex.md`; the
  `round<N>-*.md` files are the earlier, global numbering. Round 1
  (agent usability, performance, soundness, strategy, verification, and a
  second-model review) examined draft 1;
  `round1-resolution.md` audits draft 2 against it. Round 2 (consistency,
  simplicity, and a second-model review) examined draft 2.1 and led to the
  tier split. Rounds 3–7 (consistency and second-model reviews) checked
  drafts 3 to 3.3 until no v0-blocking objection remained. Round 8 reviewed
  the switch to Rust lifetime syntax and led D2 to take Rust 2024's rules
  whole. `dialect-codex.md` is the second-model review of
  `research/15-rust-dialect.md`.
- [trials/](trials/): E1 learnability trials. A fresh agent that saw only a
  reference wrote the programs, and `grading.md` grades them against the
  normative chapters.
  - `e1/`: the pilot. It estimated a 6/10 first-check pass rate against
    about 9/10 for Rust, with no leaked Rust habits; the misses trace to
    gaps in the reference and one design gap.
  - `e1-round2/`: the same tasks after the reference fixes, 9/10. The
    reference now mirrors these tasks, so later rounds use held-out tasks.
  - `e1-origins/`: held-out, borrow-heavy tasks written once with `from`
    clauses and once with Rust lifetime syntax. Both arms passed 10/10 with
    no borrow errors, and Rust syntax needed about 3.5 times the
    annotations; the tie decided D2's Rust spelling.
  - `e1-dialect/`: the first round under D18, on held-out tasks covering
    outlives bounds, `use<..>`, scopes, capabilities, the simulator, and
    attribute contracts. One writer passed 11/11 with no definite errors; a
    rustc oracle against a stub std (`rustc/run.sh`) confirmed every type
    and borrow verdict. It records the writer's token cost.

## v0 Core and Later Tiers

- **v0 core**: Rust surface and ownership, with lifetimes checked as origins; traits with a power
  ceiling; `Result` and `?`; built-in `!` forms and derives; capabilities and
  `Root`; the `Pure` marker trait; structured tasks on blocking and thread
  runtimes; channels, `Shared::with`, `join`, `par_map`, and `par_for`;
  executable contracts as runtime checks and property-test drivers; the
  simulator with crashes and durability; the declared-authority trust diff;
  sealed delegation boundaries; unreviewed branches tested without a VM
  per run, in a fixed OS sandbox on Linux; per-dependency grants; the admission
  allowlist; the agent toolchain on rustc builds gated by the admission
  driver, then Tide's incremental front end (D18). The largest parts not
  built yet: the `tide` commands and the incremental front end; packages
  (`tide.toml`, `tide.lock`, per-dependency grants, authority policies)
  and sealed interfaces; HTTP, UDP, `Rand` draws, and `Proc` operations; the
  simulator's faults, crashes, and nodes; the rest of the test sandbox and
  the unreviewed-branch guarantee (RUN-3, RUN-5); and contracts in the trust
  diff. `design/reference.md` labels each one **(planned)**.
- **v1**: `prove` mode with SMT proofs, exercised-authority tracking, the
  evented runtime, FFI, direct Rust crate import and Rust consumers of Tide
  crates,
  `par_reduce`, field-borrow declarations, and production record and replay.
- **Later**: the Bend tandem, deferred overflow traps, automatic
  parallelization, `Shape` reflection, Polonius-precision borrows, and
  interactive-proof export.

## Project Process

- [AGENTS.md](AGENTS.md): rules for agents working here, generated from
  pinned protocol profiles plus `.agent-protocols.local.md`.
- [GOVERNANCE.md](GOVERNANCE.md): production code is agent-produced; humans
  set architecture, write specifications, and review; every change to
  `main` except a generated Starcompass import update (no review) or a
  planning-only one (no Codex pass) gets a Codex supervisor pass and,
  once Codex reports it clean, an Opus review; each reviewer re-checks
  only the fixes to its own findings, and only findings in the tiers a
  change edits, or earlier ones, block it.
- [BR_WORKFLOW.md](BR_WORKFLOW.md): how `br` tracks work and which `.beads/`
  files are committed.
- Open work lives in beads (`.beads/issues.jsonl`), managed with `br`.
- `scripts/autorun.sh` runs fresh headless sessions back to back: it
  continues a handed-off bead, else works the top ready one, judges each
  outcome from `br` and `git`, logs it to `.autorun/log`, stops on no work,
  repeated handoffs, repeated failures, or its caps, and never pushes. After
  every 10th session of each run, and when it stops with no work left, it
  runs the `correct` skill in a session of its own and logs the beads it
  filed and its cost; those beads stay uncommitted in `.beads/`.
  It does not run while a coordinator has beads in flight (AGENTS.md,
  Parallel work).
  `scripts/autorun-test.sh` tests it offline with a stub agent.
- `scripts/handoff-guard.sh` is a Claude Code Stop hook for attended
  sessions: under 350k tokens of context, with a pickable bead ready, it
  blocks a stop that proposes a handoff or asks whether to go on, once.
  It runs once the owner adds its Stop entry to `.claude/settings.json`,
  which agents may not edit. `scripts/handoff-guard-test.sh` tests it
  offline.

## History

- [archive/v0-design/](archive/v0-design/): the first design, superseded by
  `design/`; its name predates the v0 core tier and is unrelated to it.
  `research/00-old-design-salvage.md` records what carried over.
- Naming: the language is named Tide (tidelang). Ferox was its working name;
  documents in `research/`, `reviews/`, `trials/`, `tasks/`, and `archive/`
  keep that name as written, and older records spell diagnostic codes
  `FX####`, with the same digits as today's `TD####`.
  [PUBLIC_NAME_REVIEW.md](PUBLIC_NAME_REVIEW.md),
  the `FEROX_*` documents, and the `ORBIS_*` documents record earlier naming
  reviews.
- [TASKS.md](TASKS.md) and `tasks/`: tasks from the first design's bootstrap
  work.
