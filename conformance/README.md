# Tide conformance corpus

D0's conformance oracle (`design/01-design.md`): every v0-core normative
rule that the checker or rustc decides carries an ID, and each ID is traced to
cases with expected verdicts that any implementation must reproduce. Because
every Tide program is a Rust 2024 crate (D18), a case also records rustc's
verdict, so the corpus compares the two checkers (`design/02-memory.md` §11).
[`rules.md`](rules.md) lists the rules and their IDs.

## Running

```
conformance/run.sh [case ...]   # check cases (default: all of cases/)
conformance/run.sh --trace      # each rule in rules.md and the cases that name it
```

By default, the driver builds in the repository's `target/`, the stable
oracle stub std in `conformance/target/`, the pinned-nightly stub std in
`conformance/target/admit/`, and case outputs in `conformance/target/conformance/`.
Set `CARGO_TARGET_DIR` to place the driver build there and use its
`conformance/` subdirectory instead of `conformance/target/` for both stub
builds and case outputs. The toolchains keep separate target directories.
Relative values resolve from the caller's working directory; an empty value
uses the defaults. Give each checkout its own target directory: two checkouts
sharing one would overwrite each other's driver and case outputs.

A case is named by its path under `cases/` (`authority/lock_accept`) or by an
absolute path. The runner type- and borrow-checks each case with stable rustc
twice, as in the E1 trial: as a 2024-edition library (a binary when a line
starts with `fn main`, so entry-point errors such as E0580 appear) and as a
test crate (`--test`, so test code is checked too). Both builds use the stub
Tide std in `rustc/`, injected as `std`, and pass `-Zallow-features=`, as the
admission driver does (ADMIT-2); the case's rustc errors are the union of the
two. It then checks the case the same two ways with the admission driver,
`crates/tide-admit`, against the stub std built by the pinned nightly, passing
`--tide-grant=unsafe` for `//@ grants: unsafe` and `--tide-profile=release`
for `//@ profile: release`. The driver's `TD` codes are
the ones `tide-admit --tide-print-codes` lists, such as `TD0520`; its diagnostics in
those codes, with their severity, are the union of the two runs. Library-only lints apply
to the library build, not its executable test harness. It needs an x86_64 host
(`authority/admit_hardware` names `core::arch::x86_64`), stable rustc and cargo
1.96, the nightly that the root `rust-toolchain.toml` pins with `rustc-dev`,
and the crates in `rustc/std/Cargo.lock`, offline. It prints one line per
case:

- `PASS`: rustc's errors match the case's rustc marks, the driver's diagnostics
  match its marks in the driver's codes, and the case has no other `TD`
  marks.
- `AWAITS CHECKER`: both match, and `TD` marks in other codes remain, which
  only later slices of the checker produce; the line lists those codes.
- `FAIL`: rustc's errors differ from the rustc marks (rustc's output
  follows, indented); the driver's diagnostics in its codes differ from those
  marks (including warning versus error), or the driver accepts a case that rustc or error marks reject, or
  rejects one that neither does, or it crashes (exits with a status other
  than 0 or 1, or panics; the driver's output follows); or the case
  is malformed (no rule line, an ID not in `rules.md`, a malformed mark, an
  unknown `//@` line).

It exits 0 when no case fails, 1 when one does, and 2 on a setup error (the
stub std or the driver does not build, or `rules.md` lists no IDs).

The runner is the trusted operator: it supplies the nightly stub with
`--tide-std` and its artifact directories with `--tide-dep-dir`. The driver
rejects raw `--extern`, `-L`, `--cfg`, `--sysroot`, `-C`, `-Z`, and response
files, constructs its own metadata invocation, and runs the compiler with an
environment holding only `RUSTC_ICE=0`, so a crash writes no report file.
Candidate Cargo and toolchain files do not configure it.
Its source snapshot is the case's source directory; output and successful
content-addressed cache entries live outside it, under the case output directory
described above.
This remains a compile-time stub oracle, without an OS build/test sandbox:
the stub never runs. Live outside-file and environment canaries belong to
the driver's integration tests, since the stable oracle would read them.

Only the driver's codes are enforced: in any other code, a wrong `TD` mark
still awaits the checker, and a missing one can let a case pass. The `TD`
marks are the design's expected verdicts, which each slice of the checker is
run against.

The surface slice enforces `TD0102`, `TD0103`, `TD0201`, `TD0202`, `TD0203`,
`TD0210` and `TD0301` as errors. `TD0202` covers LIFE-4's unresolved output
elision in function signatures, bare function types and `Fn` sugar wherever
they occur; wrong-input elision stays rustc's region error. `TD0203` labels
capture failures at an actual static object destination or
`Tasks::spawn_isolated`. E0373 uses rustc's own closure and borrowed-place spans,
including projections, independently of capture provenance. E0521/E0597 and
uncoded region errors require both a static destination and a certain capture
chain; E0515 requires a certain captured borrow at the returned destination.
Local errors inside the closure body receive no label.
`TD0210` labels E0505/E0515/E0597 inside a struct literal only when one field
moves a whole local owner and another stores a reference borrowed from it using `&`, `&mut`, or a
receiver-tied borrow method. A projected borrow counts when its loan root is
that moved owner. Copies, reborrows, raw-pointer results and loan paths through
stored references are omitted. The label
matches E0505 only at the exact span of the moved owner field's value, and E0515/E0597 only when
the primary span lies inside the paired borrow field's expression. E0515 on
an entire returned literal also qualifies when its secondary "borrowed here"
label lies inside that paired borrow field. Other fields' errors receive no
TD0210. The label
layer prefers precision over recall: every label is a supplementary
explanation of an error rustc already rejects and must never be false. When
provenance is uncertain, the label may be absent; rustc's error and rejection
remain. For TD0203 errors other than E0373, every step must be a whole binding with
one certain initializer or assignment, `&local`, `&mut local`, or a method on a
whole local whose signature returns a borrow tied to that receiver. An owned
local's storage terminates the chain; its factory's arguments are not traced.
A field, index, subslice, destructuring pattern, reassignment or unsupported
step anywhere discards the entire chain. Deferred assignments are followed
only as a single direct statement before capture in the declaring body;
multiple or conditional assignments are omitted. These are known limits on
labels, not admission exceptions. The rules
sourced only from the reference/decision record use `REF-<TD digits>` IDs in
`rules.md`. TD0902's comment rule
applies only to unsafe blocks, with or without the grant: a `// SAFETY:` line
in the comment lines directly above the block or its statement, never a
trailing same-line comment or a requirement on unsafe impls.

`TD0206` (CLOSURE-3) is an error at every coercion of a function item to a
`fn` pointer, whether implicit, an `as` cast, or the arms of an `if` or
`match`, for local and std items, methods and constructors alike, when the
implied bounds that rustc computes for the item's argument and return types,
and a method's impl header, include an outlives bound that does not hold
trivially: an origin other than `'static` outliving a different origin, or
a type parameter (unless it is declared `'static`) or associated type
outliving an origin. A trait method also counts its selected impl's method
and header; when selection finds no single impl, as through a type
parameter's bound, the coercion is rejected. The verdict depends only on the
item, not on the instantiation, so generic functions over references, trait
methods with a reference receiver such as `Clone::clone`, `&self` methods of
types with a lifetime parameter, and functions such as
`fn f(v: &mut Vec<&str>)` do not coerce; a closure that calls the item is
accepted. For an `if`, rustc reifies each arm's block, so the diagnostic
points at the block.

The contracts/modules slice enforces `TD0802`, `TD0803`, `TD0902`, and
`TD0910`–`TD0915` as errors. Module cycles use only `use` items (public uses
and globs included), targeting the module defining the imported item or the
imported module itself; declarations, body paths and external crates add no
edge. TD0915 is package-level, without a source line: the invocation test
`build_scripts_are_rejected_without_running` is its violating/accepted pair
and asserts error severity and exit status.

The trait slice enforces `TD0401`–`TD0408` as errors. Impl placement uses
exact module equality: the local ADT's module or the trait's module; for a
non-local self type only a local trait's module qualifies. If neither module is
local, the impl is not admitted; implement a trait on the local type (for
example, `Into<u16> for Local` rather than `From<Local> for u16`) or use a
newtype. A function-local impl uses the enclosing module. Specialization is checked before lowering. An impl whose self type is a type parameter or a projection that
may be unsized, of a dyn-compatible trait that declares an associated type without `where Self: Sized`,
is `TD0402`: it may cover the trait's `dyn` types (a projection can normalize to one), whose built-in impl rustc lets choose another
associated type (rust-lang/rust#57893, `memory/escape_dyn_overlap_transmute`). A sized
parameter, a trait whose associated types are all inherited (a supertrait's impls decide them,
and this check sees those impls), and one whose associated types require `Self: Sized` stay
admitted (`memory/escape_dyn_overlap_accept`). Other overlap
labels rustc's E0119. Associated type parameters and explicit binders are
rejected while elided `Fn` and bare `fn` lifetimes remain legal. Impl bounds
accept a parameter, its projection, a type containing no type parameter, or a
strict subterm of the self type or trait arguments. The definition graph uses
(function, type parameter) nodes, including uncalled generic definitions.
Each caller parameter occurring in a callee type argument has an edge to that
callee parameter; an occurrence under a constructor marks an expanding edge.
`TD0407` rejects an expanding edge on a cycle. Selected local trait impls use
their overriding method and substituted impl arguments, or the trait's default
body when the selected impl has no override. Other trait dispatch contributes
no edge. Projections are normalized in the caller's environment; unresolved
aliases do not prove parameter flow. Permutations, acyclic growth, and cycles
that reset or discard the wrapped parameter pass. Unresolved parameter or
dynamic dispatch and function pointers retain rustc's checks. An instantiation
recursion-limit failure alone does not prove forbidden growth: finite concrete
chains can hit the same limit. Without an expanding-cycle proof, rustc's
failure stays unlabelled. Inherent method
names are unique across all impls of the same type, including disjoint impls;
trait methods keep their separate namespaces. Trusted macro-generated forms
and property-test impls inserted by this driver run are not candidate
definitions. Insertion provenance is tracked by exact byte ranges; adjacent
user code remains checked. `#[allow]` does not silence these errors.

The authority slice enforces `TD0501`, `TD0503`, `TD0504` and `TD0505` as errors.
`policy.rs` owns a closed table of ambient Rust operations; missing Tide std
items are looked up at the corresponding Rust std path, including inherent
methods. Unresolved imports retain their source identity through renames and
re-exports (named or globbed) in their lexical scope; recovery requires one
source identity and respects shadowing. Same-spelled user methods and Tide capability methods are unaffected.
Unresolved paths apply each matching deny rule's code, including `TD0520`
for aliases of unavailable std items.
`Path` I/O observers and `io::Error::last_os_error` retain `TD0520`. `std::path` is
listed item by item, so an item a later rustc adds there is not admitted until the pin
review lists it.
`Root` may be named in the entry signature, in field patterns destructuring
`root` (including `main`'s parameter), or as `Root::sim`. A helper's signature
cannot name `Root`, even with a field pattern. Field destructuring moves or borrows
fields and is accepted; whole-root binding, passing and capture are rejected.
Guarded field-pattern matches may have non-binding wildcard fallback arms;
matches without a field pattern and whole-root binding arms remain rejected.
Individual field moves, borrows and disjoint closure captures pass. Simulation names and
operations require a test-only cfg ancestor, a plain/property/simulation test,
or the separate suite supplied with `--tide-test-library`. Merely compiling
ordinary production helpers with `--test` does not exempt them. The paired
candidate library remains production code. A test-only cfg becomes false when
`test` is disabled in the current target configuration;
`any(test, not(test))` is not test-only. `TD0505` uses the resolved Tide std
callee's final `&Tasks` parameter and missing-argument count, so constructors
need no separate registry. `#[allow]` does not silence these errors.
The deadlock fixtures run as tests; generated-contract fixtures use capability
parameters rather than whole `Root` parameters.

The concurrency/error slice enforces `TD0601`, `TD0602`, `TD0604`, `TD0605`,
`TD0701` and `TD0702` as errors. Async functions, blocks, closures and awaits
are checked before lowering; all `core::future::*` and `core::pin::*` uses have
`TD0601`, while `core::task::*` retains `TD0520`. Rust thread spawning, Builder
and thread scopes use `TD0602`; unresolved `tokio::spawn` and resolved Tokio
spawn items do too. Same-spelled local functions pass. Mutex/RwLock types and
methods use `TD0604`; Condvar, Barrier and Once keep their existing owners.
Missing std and Tokio paths, including module aliases and re-exports, use the
authority slice's source recovery.
Every user-written `select!` is `TD0605`, whether resolved, unresolved, or
renamed from another macro. Catching an unwind uses `TD0701`; task isolation
passes. `TD0702` checks the type of an expression statement, including macro
results such as `writeln!` and `Result<(), Infallible>`. Propagation, returned
results and explicit `let _ =` pass. Other must-use values retain rustc's
lint. `#[allow]` does not silence these errors. The cases also check against
production std in the focused driver build tests; trusted std bodies are not
candidate code.

## Case format

One `.rs` file per case, under `cases/<area>/`: `memory` (02), `authority`
(03 §0–§5 and §6.5: capabilities, admission, `Pure`, `Shared`),
`tasks` (the rest of 03 §6–§8: tasks, channels, parallelism), and
`contracts` (04).

```rust
//@ rules: LOCK-1
// A `Shared` handle as a payload is `Send` but not `Pure`: only the checker rejects it.
use std::sync::Shared;
use std::task::Tasks;

pub fn nest(tasks: &Tasks) {
    let inner = Shared::new(0u64, tasks);
    let _outer = Shared::new(inner, tasks); //~ TD0606
}
```

- The first line, `//@ rules: ID, ID`, names the rules the case exercises.
  Every ID must be in `rules.md`.
- Optional lines after it set what the manifest and build would:
  `//@ grants: unsafe` gives the case's package the `unsafe` grant (03
  RUN-4), which it otherwise lacks, and `//@ profile: release` checks it in
  the release profile instead of debug. Only the checker acts on them, and
  the runner rejects any other `//@` line.
- A comment says what the case shows.
- Each line that must get an error ends with `//~` and its codes: `E####`
  for a rustc error, `error` for a rustc error without a code (such as
  "lifetime may not live long enough"), and `TD####` for the checker; a line
  both report on carries both. A `Send`, `Sync`, or `Fn` failure carries an
  `TD` code only where the design gives that code for it (PAR-5 to PAR-7
  do; `rules.md` writes "E0277 for `Send`" where a rule's code covers only
  its `Pure` part), and otherwise the rustc mark alone. Where the driver
  reports a code in place of rustc's (an ambient call or a path that is not
  admitted), the rustc mark records what plain rustc says. A case with no
  marks must be accepted.
- A checker warning uses `//~ WARN TD####`. `WARN` makes every following
  code on that line a warning; error marks, if any, precede `WARN` (for
  example `//~ TD0507 WARN TD0607`). Warning marks require the code at
  warning severity without rejection unless the case also has error or rustc marks.
- rustc's errors must equal the rustc marks as a set of (line, code) pairs.
  An error on an unmarked line, an uncoded error where no `error` mark is,
  or an error in another file fails the case. A case whose only marks are
  `TD` must pass rustc. Rustc warnings and driver warnings without a TD code
  are ignored; driver TD warnings must match their warning marks.

The shape follows rustc's own UI tests (`//@` directives, `//~` marks) and
the D18 spike's per-line code marks (`spikes/d18/admit/tests/`), rather than
one verdict in a header: a per-line mark fails a case rejected on the wrong
line or for an incidental reason, which a single verdict would let pass.

The stub is a compile-time fixture of Tide's std (`crates/tide-std`), which
owns runtime behavior. It includes the production task runtime (`task.rs`:
`Tasks`, `Scope`, `Task`, `TaskError`, `Cancelled`, `Panicked`,
scope/spawn/join/cancel, limit/set_deadline, shield/spawn_isolated, and
checkpoint/yield_now) and Tide's std's crate-root items
(`crates/tide-std/src/root.rs`: `Root`, `Pure`, `tide::main`, the sinks and
`panic!`, the runtime entry, and Rust's re-exports) and declares the
capability types with that file's `capability!`, so those items share one
definition at the same paths. Its I/O core also includes
`crates/tide-std/src/io/shared.rs`, sharing the traits, errors, adapters and
buffered types with production. The supported I/O API follows Rust with
`BufWriter<W: Sized>`; per TASK-6, Drop discards pending bytes and warns with their count.
`Lines`, `Split`, and `Bytes` end after yielding `ErrorKind::Cancelled`.
The fixture includes production's `env.rs`, `io/stdio.rs`, and `log.rs` too:
real/simulated arguments, attenuated environment and cwd, stdin reads,
stdout/stderr handles and all five log macros have one implementation.
`authority/process_inputs.rs` and `process_sinks.rs` also execute through the
admission driver against production. Driver tests cover exit statuses, lazy
non-Unicode panics, stdin cancellation, invalid lines and early iterator drop,
log defaults/filtering/JSON/file routing, atomic records and closed stdout.
Simulation setup appends args/stdin and overwrites env keys; `records()` assembles
per-stream lines, including every log level, without host writes. Env/input
latency is zero. Real stdin cancellation is entry-only; a kernel-blocked read
waits for input/EOF. `_os`, output `lock()`, `dbg!` runtime, trace sinks,
`log::`/`tracing::`-prefixed log macros, and `release-max-level` are not implemented in production.
The stub includes the production `sync.rs` for `Shared::new`, `clone`, `same`,
`with`, `with2`, `with_all`, `reset`, and `LockError`; it has no separate Shared
runtime. `tasks/shared_counter.rs` and `tasks/shared_sets.rs` also execute against
production through the driver. The latter covers overlapping sets on real threads
and under `Root::sim`/`#[sim_test]`, aliases, caller-order payloads, poisoning, and
recovery. Runtime tests in `sync.rs` additionally check lazy FIFO tickets, ordered
simulation events, domain rejection, and reset's payload drop after release.
`tasks/task_controls.rs` executes limits, deadline failure/partial results, level-triggered
checkpoints, shields and isolation through the driver on both runtimes in debug and release.
`tasks/limit_deadlock.rs` also executes against production: its simulated recursive limit
waits abort after reporting every blocked task's wait kind and source span. These use the
same global report as sleeps, joins, and channel waits.
The stub includes production `par.rs` for `join` and the prelude `ParSlice`
methods `par_map`, `par_for`, and stable `par_sort`. `tasks/parallel_regions.rs`
executes all four through the driver on real and simulated runtimes in debug and
release, including nested regions, cancellation and regions inside `Shared::with`.
Driver tests check panic sink prefixes and Pure rejections; runtime tests force
helper threads even on a one-core host, check lowest-index panic selection,
per-call block stopping, stable owned sorting, nested sink ordering and simulation
thread identity. The scheduler uses budgeted scoped threads; the pool and `par-threads`
remain planned. Buffers are unbounded and losing host panic hooks are unsuppressed.
Stop flags are per call; propagation across calls is partial. `join` and nested
regions run an already spawned right branch to completion after a left panic. A
right branch that never terminates diverges, and whether the region panics or
diverges can depend on machine load.
Every fork finding a free token spawns an OS thread, so fine-grained recursive
`join` is slow until the planned pool. Sort forks only for ranges of at least
4096 elements; tests check the same sorted result and ordered comparator trace
with one and four workers below, at and above that cutoff. Simulated panic tests
assert the sink prefix on `sim.records()`. Helper-thread panic metadata is not
returned to the property runner: on the thread runtime, scheduling can turn a
`requires` discard into a failure. Simulation is unaffected by that metadata gap.
The stub includes production `sync/mpsc.rs` for bounded/unbounded channels,
sender clone, blocking/nonblocking operations, deadline receives, iteration,
and cancellation-aware error enums. `tasks/channels.rs` executes every exposed
item through the admission driver in debug and release, on real threads and
under the seeded scheduler. `tasks/channel_deadlock.rs` executes a blocked
send and receive through the shared global report, including both call sites.
Runtime tests additionally cover FIFO/no barging, rendezvous completion,
cancellation races and echo attribution, cross-runtime wakeups, payload drop
after lock release, simulation identities, and payload-free tracing.
The stub's `std::time` is the production `time.rs`: `Clock`, `Duration`,
`Instant`, `SystemTime`, `SystemTimeError`, and `UNIX_EPOCH`.
`authority/clock_time.rs` also executes against production through the driver.
Through `root.rs` the stub also includes the production codec engine and
`std::json` (`codec.rs`, `json.rs`), with the `Encode` and `Decode` derives;
`codec/json.rs` also executes against production through the driver.
The float observers and math functions use the production `float.rs` and `math.rs`
files directly, with the same pinned software `libm` dependency. Their shared
fixture, `authority/float_math.rs`, also executes against production through the
driver in debug and release, including recorded bit-exact math vectors.
Through `root.rs`, both std surfaces also use production's `convert.rs` and prelude
traits `Truncate`, `RoundFrom`, and `TryFromFloat`. `authority/numeric_conversions.rs`
checks every replacement spelling and executes against production in debug and release;
std unit tests cover low bits, rounding ties, overflow, and exact float-to-integer bounds.
`std::arena::{Arena, Id}` also shares production's `arena.rs` through `root.rs`.
`memory/arena_executable.rs` executes cyclic traversal, live access/mutation,
stale-handle checks, disjoint borrows, reuse, iteration and bound-free handle
traits through the driver. The build tests also execute stale Index/IndexMut
panics and check `#[pure]` acceptance/rejection against production.
`memory/arena_pure_reject.rs` pins `Arena<Fs>` rejection; the executing fixture
pins `Id<Fs>` acceptance. `afree_pure_*` still exercise the existing TD0511 owner.
The admission driver's rules (`policy.rs`, `pure.rs`) key the shared items
alike. Production and the stub share the `#[test]` macro and property
generation/shrinking runtime (`crates/tide-std/src/test/`). They share a compile-time fixture,
`authority/root_executable.rs`, also built and run against production std by
the driver's `tests/build.rs`; parameterized property tests also execute through
that driver suite. The fixture includes production's `sim.rs` and runner and uses its
`#[sim_test]` macro: `tasks/simulation_runner.rs` also executes through the driver,
covering seeded scopes, the split-update bug and ordered sink traces. Production driver
runtime tests also record `tide-replay/1` manifests, compare two replay traces to
the original, refuse source/build/dependency/configuration mismatches before launch,
and exercise strict tapes, root ordinals and user task-0 origins. The fixture's
shared surface alone cannot establish those runtime replay guarantees. Later world/fault/node
operations remain fixture stubs. `contracts/guided_properties.rs` reaches the same
requires guidance, vacuity floor, fixed case seed and simulation-parameter shrinking items;
the driver suite executes its passing and failing entries and checks replay.
`contracts/generated_tests.rs` independently exercises generated contracts, private-state
construction, explicit unchecked skips and allowed vacuity through the same production
macro/runtime. The driver's synthesis pass adds these entries in `__tide_contract_tests`;
E1 excludes that exact module segment only from acceptance listings and runs all entries
in the writer stage. Structural generator insertion
preserves original source lines, but columns of following tokens on an insertion's
physical line can shift. A case that uses only the shared items is checked against Tide's
std's own definitions. Every other item is the stub's own, a signature for an
item Tide's std does not provide yet. `Pure` is
a blanket impl, so rustc accepts every `Pure` bound and every `Pure` verdict
is the checker's (a `TD` mark). The stub's own bodies are `unimplemented!()`,
so the corpus decides compile-time verdicts only; rules decided at run time,
in the simulator, or in the sandbox are outside it (`rules.md` lists them).
`rustc/std` originated in `trials/e1-dialect/rustc/std`, whose copy stays
frozen as the trial's record, and uses Tide's std's macros (`crates/tide-macros`);
`#[pure]` comes from production through the shared prelude; it validates a
function target and empty arguments, preserving its tokens for TD0511.
`rustc/macros` adds only the identity `#[trusted_pure]` shim, which production
does not yet provide. Contract macros also come
from `crates/tide-macros`; the hidden clause runtime includes production's
`contract.rs`. `contracts/executable.rs` checks the shared surface and executes
against production through the driver's `tests/build.rs` in debug/release binaries
and test harnesses. Macro syntax errors carry uncoded rustc errors with a TD
identifier in their text; placement and purity diagnostics remain the admission
driver's responsibility.
`contracts/scoped_clauses.rs` and `contracts/clause_expressions.rs` also execute
scoped entry values, builtin expressions, loop scopes, snapshots and const contracts
through the production driver in both profiles. The
[std crate docs](../crates/tide-std/src/lib.rs) own current contract implementation limits.
Add an item to the stub when a case needs it and Tide's std does not provide
it, with the signature the design gives, and mark `GUESS` on a signature no
design document fixes. When Tide's std comes to provide one of the stub's
items, the stub includes Tide's std's definition, as it includes `root.rs`,
and drops its own. If the stub cannot express a verdict a rule needs, record
the gap on the case's bead.

## Rule IDs

A rule ID is `PREFIX-N`: an uppercase prefix naming a group of rules and a
number counted from 1 within the prefix, as 03 already does (`ROOT-1`,
`LOCK-4`). Each prefix belongs to one chapter, which defines each of its rules
with a bold `**PREFIX-N.**` lead, optionally followed by a title, at the start
of the rule's bullet, list item, or sentence. An ID is never renumbered or
reused: a removed rule's ID stays retired, noted where the rule stood, and
a new rule takes the next number. A rule another chapter restates keeps its
owner's ID, and the restatement cites it. In `rules.md`'s first column a
chapter's own ID goes bare (`MOVE-1`), and `= ID` marks a row that restates
another chapter's rule.

Some rules in 02, 03, and 04 have no ID yet (`none yet` in `rules.md`). The
bead that writes their cases assigns them from these prefixes, defining each
ID in its section:

| Prefix | Section | Rules |
| --- | --- | --- |
| `MOVE` | 02 §1.2 | moves and copies |
| `LOAN` | 02 §1.4 | loans and conflicting accesses |
| `ORIGIN` | 02 §2.1, §2.2, §2.7, §2.8 | origin flow, well-formedness, bodies |
| `LIFE` | 02 §2.3, §2.5 | lifetime syntax and elision |
| `BORROW` | 02 §3 | borrowing types |
| `VAR` | 02 §4 | variance and subtyping |
| `DYN` | 02 §5.1 | trait objects |
| `OPAQUE` | 02 §5.2 | `impl Trait` |
| `CLOSURE` | 02 §5.3, §5.4 | closures and higher-ranked origins |
| `DROP` | 02 §6 | drop check |
| `SEND` | 02 §8.1 | `Send`, `Sync`, task isolation |
| `SCOPE` | 02 §8.2 | scopes |
| `SHARED` | 02 §8.4 | `Shared::with` payload references |
| `PURE` | 02 §9 | `Pure` (03 §3.1 cites it) |
| `CAP` | 03 §0–§1 | capability methods and handle constructors |
| `UNSAFE` | 03 §2.6 | what the `unsafe` grant gates (RUN-4's hole 1) |
| `AFREE` | 03 §3.2, §3.5 | `#[pure]`, `const fn`, memoization |
| `PAR` | 03 §8.1 | `join` and parallel closures (`PAR-5` on) |
| `CHANNEL` | 03 §6.4 | channel diagnostics |
| `FORM` | 04 §1.2 | clause grammar |

`PURE` goes to 02 §9 because 02 is the definition the checker implements
(`README.md`); 03 §3.1 then cites it rather than restating it. 02 §8.5
restates 03 §8.1's `join` and parallel-closure rules, so they take `PAR`
IDs and 02 cites them.

The stub includes production `net.rs` and its TCP backends; no TCP runtime is
reimplemented in the stub. `authority/tcp.rs` is one equivalence suite executed
through tide-admit on real Linux `127.0.0.1:0` and `sim.net` in debug/release,
including listener/stream operations, owned/borrowed I/O, attenuation, cancellation,
addresses, EOF/half-close and a 1 MiB transfer. It compares values/ErrorKinds,
excluding ports/timing, and asserts allowed sets for timing-dependent post-close
outcomes. Simulation runs in tests; the trusted harness compares each seed's
observations with the real binary's output. Seed reruns compare traces;
simulation-specific checks cover port reuse,
bounded backlog/backpressure, wildcard binding and cancelled connect/read/write.
`authority/tcp_deadlock.rs` executes a test with a full socket and no reader and must report
`net.write(conn)` with the call site. These execute in the ordinary trusted harness,
not RUN-3's planned socket-denying sandbox. The TCP model's capacities and delays
remain provisional; other real hostnames use inline uncancellable Rust resolution,
while simulation supports only IP literals and `localhost`. HTTP remains a stub; UDP is absent.
See `research/21-channels-tcp.md` §§1.9–1.13 and §4 items 5–14 for the adopted limits.
