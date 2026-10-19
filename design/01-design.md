# Tide Design

Status: draft 3.4. Draft 1 was synthesized from `design/00-first-principles.md`
and `research/`; draft 2 answers the six round-1 reviews in `reviews/`; draft
3.4 returns D2 to Rust's lifetime spelling after the origin trial
(`trials/e1-origins/`). This
document owns the design decisions. Chapters elaborate them and must not
contradict them; a change of decision is made here first.

## Thesis

Tide is a systems language for software that agents write and humans must
trust.

It keeps what makes Rust worth learning: memory safety and data-race freedom
without a garbage collector, zero-cost abstractions, algebraic data types,
`Result` and `?`, and "if it compiles, it probably works". It keeps Rust's
spelling wherever the meaning is the same, so a model's Rust knowledge
transfers. It removes what costs agents the most: user macros, async
coloring, an unbounded trait system, and slow builds, and it narrows Rust's
lifetimes to a subset without written outlives relations.

It adds guarantees that a language can only make if it is designed for them
from the start, and that tools layered on Rust cannot give, because Rust code
can reach the outside world from anywhere:

1. **No ambient authority.** Code touches the world only through capability
   values it is handed. A function's signature bounds what it can do; code
   without capabilities cannot read the clock, the network, files, the
   environment, or randomness, in any dependency.
2. **Deterministic simulation of whole programs.** Because every source of
   nondeterminism is a capability, any Tide program, dependencies included,
   runs under a seeded simulator and replays bit for bit from seed and commit,
   provided every foreign effect it reaches has a simulator model; the build
   lists any that do not. Agents get a reproducible oracle for concurrent and
   distributed bugs.
3. **A reviewable trust surface.** Public signatures, declared and exercised
   authority, contracts with their proof status, and trust holes (`unsafe`,
   `assume`, FFI, unproved dependency contracts) are computed by the compiler
   and diffed between revisions, so reviewers see what changed in what code
   may do before reading how it does it.

Contracts (`requires`, `ensures`) are part of the language and serve as
runtime checks, property tests, and optional SMT proofs. They support the
thesis; they are not the headline. Proof-first, massively parallel pure
computation is Bend's territory (D16).

One line: authority you can read in a signature and whole-program
deterministic replay, with Rust's guarantees and much faster builds.

## Positioning And Scope

- **v0 target**: command-line tools, developer tooling, and infrastructure
  components (agents, proxies, storage engines, schedulers). Blocking and
  thread runtimes plus the simulator; the evented runtime follows.
- **Rust coexistence**: Tide builds C-ABI libraries with generated Rust
  binding crates, and imports synchronous Rust leaf crates through generated
  shims. Async Rust ecosystems (tokio, axum) and lifetime-heavy APIs
  (serde's `'de`) are out of scope for interop.
- **Bend tandem, not competition**: Tide is the effectful, authority-checked
  shell; Bend 2 owns proved, massively parallel kernels (D16).
- **Evidence-gated**: the thesis is tested by the validation program (D0)
  before syntax freezes.

## Tiers

Round 2 found that closing every review hole with a new rule had made the
full design heavier than Rust (`reviews/round2-simplicity.md`). The design is
therefore split into tiers. **v0 core** is what ships first and what
`design/reference.md` teaches; later tiers are designed so v0 does not block
them, but they are not part of v0. Where a decision below describes a
later-tier feature, the tier table wins.

| Tier | Contents |
| --- | --- |
| v0 core | Rust surface and ownership, with lifetimes checked as origins; traits with the power ceiling; `Result` and `?`; built-in `!` forms and derives (`Encode`/`Decode` for JSON only); capabilities and `Root`; `Pure`; sinks; structured tasks on blocking and thread runtimes; channels and `Shared::with`; `join`, `par_map`, `par_for`; executable contracts (`requires`, `ensures`, loop `invariant`, `old`, `result`, bounded quantifiers) as runtime checks and property-test drivers; the simulator with crash, restart, and durability; declared authority and trust holes in the trust diff; per-dependency grants; the agent toolchain on the interpreter, then the Cranelift-class backend |
| v1 | `prove` mode with `spec fn`, views, `final`, `decreases`, laws, and proof status; exercised-authority tracking; the evented runtime; the C-emitting release backend; FFI with `#[trusted_pure]`; Rust crate import; `par_reduce` with canonical grouping; field-borrow declarations; production record and replay, plan roots, and replay queries (D17); kernel enforcement of authority (D7); behavior diffs in `tide review` (D11) |
| Later | Bend tandem (D16); deferred overflow traps; automatic parallelization; read-only `Shape` reflection; Polonius-precision borrows; interactive-proof export |

`prove` stays in v1 (owner delegated the choice, 2026-10-19): v0 contracts
already use the clause syntax agents will later prove, so moving proofs in
later costs no rewrite, while shipping them in v0 would roughly double the
reference and put Tide's pitch on Bend 2's ground.

v0 removes these from draft 2: the `Frozen` trait (`Pure + Sync` covers it),
the `lock` keyword (replaced by `Shared::with` methods), `par { }` blocks
(replaced by the library `join`), and the moved-out rule (a child task's panic
propagates to its scope, as in Rust's `std::thread::scope`).

## Evidence That Shaped The Design

Details and sources are in `research/` and `reviews/`.

- In unfamiliar languages most model failures are syntax borrowed from
  familiar ones; a Rust-like surface learned better than alternatives
  (`research/05`, `research/09`).
- In a 6.08M-line Rust sample, 77% of files used constructs that draft 1
  would have rejected or respelled despite identical meaning
  (`reviews/round1-agent-usability.md`).
- Precise compiler feedback raised Rust repair rates from 10.7% to 73.7%;
  execution feedback took a model from 45% to 82% on an unseen language
  (`research/05`).
- Frontier agents fixing real issues do best in Rust despite more first-try
  compile errors (`research/05`).
- SMT-backed contracts lead agent verification benchmarks; spec generation is
  the weak half, and agents game verifiers unless trust holes are policed
  (`research/04`).
- Compile speed is decided by language design more than backends
  (`research/03`); emitted Rust measured 53 s clean and 17 s incremental on a
  100k-line crate, and witness-table generics ran 6–24x slower than
  monomorphized code (`reviews/round1-performance.md`).
- Every trait-system crisis in Rust and Swift came from power added later
  (`research/08`).
- Library-level deterministic simulation fails because dependencies reach
  ambient time, randomness, and threads (`research/07`, `research/10`).
- Bend 2 already pitches "proof that blocks AI mistakes" for backend code;
  leading with proofs would compete with it (`reviews/round1-strategy.md`).

## Decisions

### D0. Validation program with gates

The design is a hypothesis. Before syntax freezes, the project runs:

- **E1 learnability**: a fresh agent given only the Tide reference writes a
  fixed task set; pass rates and iterations to green are compared with the
  same agent writing Rust.
- **E2 productivity**: equivalent multi-file tasks in Tide (interpreter) and
  Rust, measuring wall time, iterations, and defects.
- **E3 review**: reviewers detect planted malicious or authority-widening
  changes using the trust diff versus the plain code diff.
- **E4 compile speed and E5 runtime**: benchmark suites against the D12
  targets and against Rust.
- **E6 plumbing**: the same application written with capabilities by agents,
  measuring how often they pass `Root`-like bundles everywhere and how large
  the resulting trust diffs are.
- **Conformance oracle**: every normative rule in the chapters carries an ID
  (`TASK-2`, `PAR-1`, ...), and each ID is traced to conformance tests that
  any implementation must pass, in the style of Rue. E1 reruns on every
  release with a fresh held-out task set.
- **Mechanized soundness**: before 1.0, the origin calculus (D2) and the
  determinism of authority-free code (D7) are proved in Lean, with agent
  help; a Lean proof of Move's borrow checker took about 27 days with Claude
  Code (`research/12-newcomer-sweep.md`).

Initial thresholds, revised as data arrives: on a 30-task suite, Tide
first-check pass rate at least 0.8x the Rust rate and final pass rate at least
the Rust rate within 1.5x the iterations (E1); reviewers using the trust diff
catch at least 1.5x as many planted authority-widening changes as with the
plain diff (E3). Gates: month 3, E1 on the interpreter; month 6, E1 and E2;
month 9, E3 to E6. Each gate decides continue, adjust, or stop. Syntax in D1 is
provisional until E1 meets its thresholds.

### D1. Surface: Rust spelling wherever the meaning is the same

- Rust syntax is Tide syntax for every construct with the same meaning,
  including `where` clauses, `pub(crate)`, inline `mod` blocks,
  `#[derive(...)]` with Rust's trait names, `#[cfg(test)]`, `'static`,
  `From`/`TryFrom`/`Into`, `wrapping_add`, `checked_add`, `saturating_add`,
  `Wrapping<T>`, `Box<dyn Error>`, and `// SAFETY:` comments.
- Blocks use braces and semicolons exactly as in Rust; indentation is never
  significant. Braces match model priors, survive diffs, markdown, and logs
  intact, and keep a mis-indented agent edit cosmetic instead of silently
  moving a statement into another block. Generics use angle brackets and
  Rust's turbofish.
- A **fixed, compiler-implemented set of `!` forms** keeps Rust's most common
  macro spellings with their Rust meaning: `println!`, `print!`, `eprintln!`,
  `eprint!`, `format!`, `write!`, `writeln!`, `vec!`, `assert!`,
  `assert_eq!`, `assert_ne!`, `debug_assert!`, `matches!`, `panic!`,
  `todo!`, `unimplemented!`, `unreachable!`, `dbg!`, and the logging forms
  `error!`, `warn!`, `info!`, `debug!`, `trace!`. Users cannot define macros
  (D6).
- Lifetimes keep Rust's spelling in an origin-equivalent subset; the few
  forms outside it are errors with a suggested rewrite (D2).
- Where Tide's meaning differs, the Rust spelling is rejected with a
  machine-applicable fix: `async`/`.await`
  (→ capabilities, D8), lossy `as` (→ `try_from` or `truncate`, D3), ambient
  I/O such as `std::env::args()` or `std::fs::read` (→ capabilities, D7),
  `impl` blocks outside the type's or trait's module (D4), user macros (D6).
- The reference is written as a delta from Rust and token-counted in CI
  (budget in D14).

Rejected: fresh syntax (low-resource penalty with no data to recover it);
respelling constructs that mean the same thing (`f"..."`, `print()`,
`assert()`, `T?`, `[T]` generics), which draft 1 proposed and the usage
measurement rejected.

### D2. Memory: ownership with Rust's lifetimes, checked as origins

- Every value has one owner; moves are the default for non-`Copy` types.
  References `&T` and `&mut T` follow Rust's shared-xor-mutable rule.
- **Lifetimes keep Rust's spelling.** A lifetime names an **origin**: the
  set of loans a borrow may come from (Polonius's reading).
  - Rust's lifetime-elision rules apply unchanged, so
    `fn get(&self, k: &str) -> Option<&V>` borrows from `self` exactly as in
    Rust. A borrowing type in a signature elides its lifetimes as
    `Lexer<'_>` or as bare `Lexer`, as in Rust.
  - Otherwise signatures name lifetimes as in Rust:
    `fn longest<'a>(a: &'a str, b: &'a str) -> &'a str`, and
    `fn next_word(&mut self) -> Option<&'a str>` in `impl<'a> Lexer<'a>`
    returns a borrow of the source, not of `self`.
  - **Accepted:** lifetime parameters on structs, enums, impls, and
    functions; `'_` and `'static`; type-outlives bounds in every spelling
    (`T: 'a`, `T: Trait + 'a`, `impl Trait + 'a`, `dyn Trait + 'a`); and
    the outlives relations that well-formed argument types imply
    (`&'m BTreeMap<K, Vec<&'t str>>` lets a result be `&'m [&'t str]`).
  - **Rejected with a suggested rewrite** (`FX0201`): written
    lifetime-outlives bounds (`'a: 'b`), lifetime parameters on traits,
    explicit `for<'a>`, and `+ use<..>`. `T: 'static` is accepted and
    rewritten to `T: Owned`.
- **Borrowing types declare lifetime parameters as in Rust**:
  `struct Lexer<'a> { src: &'a str, pos: usize }`. Each parameter names one
  origin; fields that use one name share it, and fields from different
  sources take different names
  (`struct Collector<'s, 'o> { src: &'s str, out: &'o mut Vec<&'s str> }`).
  Variance follows Rust: shared references are covariant, `&mut T` and
  cells are invariant in `T`.
- **`'static` and `Owned`.** A type that borrows nothing except `'static`
  data is `Owned` (Rust's `T: 'static`). Trait objects follow Rust's
  object-lifetime defaults: `Box<dyn Trait>` is `Owned`, while `&'a dyn
  Trait` is `dyn Trait + 'a`; any other borrowing object is written `dyn
  Trait + 'a`. Closure-bound argument types are higher-ranked through `Fn`
  sugar, which is how `Tasks::scope` is typed
  (`impl FnOnce(Scope<'_, E>) -> Result<T, E>`). A closure that captures a
  borrow is a borrowing value; it may be returned or stored only where its
  lifetime is named (`-> impl Fn() + 'a`, a `Box<dyn Fn() + 'a>` field). A
  returned `impl Trait` captures every lifetime its bounds name, elided or
  written; with none it is `Owned` (Rust 2018's rule, not Rust 2024's
  implicit capture). A task may borrow only data that outlives the scope it
  is spawned into, so tasks spawned into a long-lived scope capture `Owned`
  values.
- **Drop check** uses Rust's conservative rule: data borrowed by a value with
  a destructor must strictly outlive that value; standard containers carry an
  audited exemption.
- Borrow checking is per function at NLL precision plus declared field
  borrows on methods (`fn advance(&mut self.{pos})`), so disjoint-field calls
  compose. Polonius-level precision is a later, compatible relaxation.
- Non-tree data uses the standard library: `Arena<T>` with generational
  handles, `Rc`/`Arc`/`Weak`, `Cell` and `RefCell`. No hidden reference
  counting of ordinary values.
- The normative origin calculus (formation, variance, elision, subtyping, drop
  check) lives in the memory chapter and is the definition the checker
  implements.

Rejected: draft 3's `from` clauses, which named origins by places
(`fn next(&mut self) -> Option<&str from self.src>`, `struct Lexer { src:
&str }`). On the held-out origin trial (`trials/e1-origins/grading.md`) they
needed about a third of the annotations, but both spellings passed 10 of 10
programs on first check with no borrow-related error, Rust's predicted `'a`
error class did not appear, and the `from` writer raised 14 origin
questions against 5, six of them on forms its reference never showed. A
tie goes to Rust's spelling under D1. Also rejected: draft 1's "one
implicit region per type" (unsound through `&mut` or too restrictive,
`reviews/round1-soundness.md` B1–B3); projections-only borrow return
(unfamiliar); shared-mutable-alias models (unproven at scale).

Accepted costs: about 3.5 times as many lifetime annotations as `from`
clauses on the trial's tasks (a declaration on every borrowing type and
impl, and a name at every use); Rust's `&'a mut T<'a>` trap stays writable,
as in Rust, though elision avoids it; a result that borrows from several
inputs through an invariant position cannot use `'a: 'b` and must be
restructured; self-referential structs need `unsafe`.

### D3. Types: small, decidable, explicit at boundaries

- Rust's scalar, string, slice, array, tuple, `Option`, `Result`, `Vec`,
  `Box`, and `String` types with their Rust meaning. No null.
- Nominal `struct` and `enum` with exhaustive `match`; `impl Trait` in
  argument and return position as in Rust, except that a returned
  `impl Trait` exposes only the auto traits it names (`impl Iterator + Send`),
  so a body edit cannot change what callers may assume.
- Every item signature is annotated; inference is bidirectional and local to
  a function body. No overloading: one function per name per scope and one
  inherent method per name per type; trait methods with the same name are
  disambiguated by trait path, as in Rust. No polymorphic recursion.
- **Integers trap on overflow, division by zero, and over-wide shifts in every
  profile.** v0 traps are precise. Deferred reporting, which lets the
  optimizer batch checks in vectorized loops, is a later tier that must first
  show the writes it delays are unobservable. A proof may delete a
  runtime check only if every fact it relies on is either proved inside the
  same build or still checked at run time; a `pub` precondition that a deleted
  check depends on stays checked at function entry.
- `as` is accepted only where it is lossless; lossy conversions use
  `TryFrom`, `truncate()`, or explicit float rounding methods.
- Floats are IEEE 754. Every operation that reads a float's sign or bits
  (`to_bits`, `is_sign_positive`, `is_sign_negative`, `copysign`, `signum`,
  `total_cmp`, hashing, formatting, foreign calls) treats any NaN as one
  canonical NaN, so observable results match across backends and targets
  while arithmetic stays native. `min` and `max` order -0.0 below +0.0.

Rejected: signature inference, implicit conversions, overloading, custom
operators.

### D4. Abstraction: traits with a hard power ceiling

- Rust traits with methods, default methods, flat associated types and
  constants, and `impl Trait for Type`. Impls live in the module of the type
  or the trait.
- No overlapping impls, specialization, higher-kinded types, generic
  associated types, rank-N types (higher-ranked lifetimes through `Fn`
  sugar are allowed), negative bounds, or user auto traits. Impl
  where-clauses must be structurally smaller than the impl head.
- Generics are definition-checked: instantiation never produces a type error.
- Coherence never treats a missing auto trait (`Send`, `Sync`, `Pure`) as
  making two impls disjoint.
- `dyn Trait` is always spelled out.
- Traits may declare **laws** (D10), which impls must satisfy.

Rejected: GATs, HKTs, specialization; ML functors; Zig-style comptime
generics.

### D5. Errors: values, as in Rust

- `Result<T, E>` and `?` with Rust's meaning, including conversion through
  `From`. `#[derive(Error)]` with `#[from]` on variants generates `Display`,
  `Error`, and `From` impls, so the common `thiserror` style is built in.
  `Box<dyn Error>` works.
- Results are must-use.
- Bugs (overflow, out-of-bounds, failed assertions, `panic!`) abandon the
  current task. A panic raised while unwinding from another panic aborts the
  process, and stack overflow is a panic. A child task's panic propagates to
  its scope, which panics in
  turn, as with Rust's `std::thread::scope`; no code ever observes data a
  panicking task was mutating. Recovery happens only at
  `Tasks::spawn_isolated`, which takes an `Owned` closure and returns
  `Result<T, Panicked>`. Abandoned values are dropped, as in Rust's unwinding,
  so resources are released; `Drop` impls are therefore verified without
  assuming their type's invariant. A cell whose `Shared::with` closure
  panicked is poisoned.

Rejected: exceptions; draft 1's `from` keyword on variants (collides with
`String::from`, `reviews/round1-agent-usability.md` B2).

### D6. No user macros; built-in derives, read-only reflection, const evaluation

- A closed, compiler-implemented derive set: `PartialEq`, `Eq`, `PartialOrd`,
  `Ord`, `Hash`, `Clone`, `Copy`, `Debug`, `Default`, `Error`, `Encode`,
  `Decode`. `Encode` and `Decode` are compiler-generated serialization with
  format plug-ins, so serialization does not depend on reflection speed.
- Every nominal type exposes a read-only `Shape` (fields, variants, names,
  layout) for printing, diffing, and inspection. Constructing a value through
  `Shape` is allowed only for types that opt in and whose fields are all
  visible; capability types can never be constructed this way.
- `const fn` evaluation is authority-free, deterministic, fuel-limited, and
  cached; it computes values, never declarations or types.
- Code generators are programs the developer runs explicitly; their output is
  ordinary source that is committed and reviewed. Nothing generates code
  during a build.

Rejected: token and procedural macros (opaque to agents and tools, uncached,
supply-chain holes); unrestricted compile-time execution; reflection-based
serialization as the default (5–7x slower in prior art).

### D7. Authority: capabilities as values, no ambient authority

- Every operation that reads or changes the outside world, or that can
  observe nondeterminism, is a method on a capability value: `Fs`, `Net`,
  `Clock`, `Rand`, `Env`, `Stdin`, `Proc`, `Tasks`, and `Accel` for Bend
  kernels (D16). The standard
  library has no free functions that do I/O, read the time, draw randomness,
  read the environment, or spawn.
- `main` may take `root: Root`, the only source of capabilities. `Root` can be
  named only in `main`; a simulated root (`Root::sim(seed)`) only in tests and
  in `tide sim`.
- Files support append and explicit durability (`File::sync`,
  `Fs::sync_dir`); durability is part of the simulator's filesystem model
  (D15).
- Capabilities are ordinary parameters passed by reference. Attenuation is a
  method returning a narrower capability: `root.fs.dir("/var/lib/app",
  Mode::ReadWrite)`, `root.net.only(["db:5432"])`.
- Safe code has no mutable globals, no thread-locals, and in v0 no lazy
  statics: statics are evaluated at compile time by `const fn`, and anything
  that needs run-time initialization is built in `main` and passed down.
- **Write-only diagnostic sinks are not capabilities.** `print!`, `println!`,
  `eprint!`, `eprintln!`, `dbg!`, and the built-in `log` and `trace` APIs write
  to sinks owned by the current task and configured by `main`. They cannot
  read anything back, they do not affect purity, and inside `par` their output
  is buffered and emitted in sequential order. Arguments of sink forms are
  always evaluated, whatever log level is enabled, so configuration cannot
  change results or panics. `dbg!` is an error in release
  builds; printing to stdout from a library package is a lint. Sink use is
  still reported as exercised effect in the trust surface, and `Secret<T>`
  values refuse `Debug` and `Display` so they cannot be logged by accident.
- **Synchronization handles** (`Shared<T>` cells, channel ends, atomics,
  `Arc`, task handles) let tasks observe each other's timing and
  cancellation, so creating one takes `&Tasks`. Authority-free code can
  neither create nor receive them; `Rc` and `Cell` stay capability-free
  because they never cross tasks.
- **Authority-free types.** `Pure` is a compiler-known auto trait: a type is
  `Pure` if no capability, synchronization handle, or foreign handle is
  reachable from it. `Arc` exposes no reference-count readers, and
  `Arc` and `sync::Weak` are synchronization handles, because drop timing and
  conditional cloning can reveal scheduling; authority-free code shares data
  by reference inside scopes instead. Classification is structural over every
  reachable type, including closure captures and generic arguments. A function
  is authority-free if all its parameter and capture types are `Pure`. An
  authority-free function cannot observe nondeterminism: its results depend
  only on its arguments.
- Parallel code (D9) requires `Pure + Send` closures and `Pure + Sync` shared
  captures: `Pure` excludes atomics and synchronization handles, and `Sync`
  excludes `Cell` and `RefCell`, so no interior mutability is observable
  across branches. Memoization and `const fn` additionally forbid `&mut`
  parameters. A call through a closure, function pointer, or `dyn` value may
  suspend exactly when the callee value's type or any argument's type is not
  `Pure`.
- **Foreign calls carry authority.** `extern` declarations are allowed only in
  packages granted `ffi`. Each foreign function is wrapped by a Tide
  signature that takes the capabilities it may use, or is marked
  `#[trusted_pure]` with a reason, which counts as a trust hole.
- The manifest grants escape hatches per dependency: `unsafe`, `ffi`. There
  are no build scripts or install hooks. The lockfile records each package's
  public authority; an upgrade that widens it fails until approved (D13).
- Tooling reports **declared and exercised authority** for every public item
  and lints over-granted capabilities. For each exercised authority it shows
  a witness path, the call chain that uses it. `tide fix` can thread a
  capability through a call chain.
- **Kernel enforcement** (v1): when `main` attenuates its `Root`, the runtime
  installs the matching operating-system sandbox (Landlock and seccomp on
  Linux) and drops what was not kept, so a bug or a malicious dependency
  that escapes the type system still meets the kernel. A WASI target maps
  capabilities onto component imports.

Rejected: effect rows and effect variables; implicit capability resolution
through `with` clauses; user-defined handlers and continuations in v1;
capabilities for write-only diagnostics (the plumbing cost would push agents
to pass `Root` everywhere, `reviews/round1-agent-usability.md` B3–B4).

### D8. Concurrency: structured tasks, no async, compiled to state machines

- There is no `async`/`await` and no futures in user code. A function can
  suspend only by using a capability; an authority-free function never
  suspends.
- Tasks are spawned into scopes from the `Tasks` capability and cannot
  outlive them. Children may borrow from the parent. The first failure cancels
  siblings, then propagates. Joining a scope cannot be cancelled.
- Cancellation is level-triggered with deadlines: capability operations in a
  cancelled scope return `Err(Cancelled)`. A scope deadline cancels the
  children but fails the scope only if a child returns the cancellation as its
  error, so a scope can collect partial results; suspended work unwinds and runs
  destructors; operations that consume a value return it inside the error.
- **Shared mutable state** is message passing over channels, or `Shared<T>`
  cells with `T: Send + Pure`, accessed only through `Shared::with(&a, |x|
  ...)` and `Shared::with2(&a, &b, |x, y| ...)`. `with2` acquires both cells
  atomically in a global order and returns `Err(Aliased)` if they are the
  same cell. The closure must be `Pure`, so it can hold no capability and no
  `Shared` handle: it can neither suspend nor acquire another cell, which
  makes this primitive deadlock-free. Acquisition is not a cancellation point.
  A panicking closure poisons the cells; both methods return
  `Result<R, LockError>`. A cell's invariant is its payload type's
  invariant.
- Data-race freedom comes from ownership plus `Send` and `Sync`, derived
  structurally. The task, not the OS thread, is the isolation unit; values
  that never leave a task need not be `Send`.
- **Scope results** follow one precedence rule once every child has
  finished, unless the body panicked, in which case the panic keeps
  unwinding after the children are joined. An error a child returns after
  observing a cancellation caused by the body's `Err` or a sibling's
  failure is an **echo** and never counts. Then: if a child panicked, the
  scope re-raises the earliest-spawned child's panic; otherwise, if a child
  failed (echoes excluded), it returns the earliest-spawned failing child's
  error, where a child cancelled by the scope's own deadline reports
  `DeadlineExceeded`; otherwise it returns the body's result, `Ok` or
  `Err`. So the body's error is reported when its children only echo it,
  an independent child failure is never hidden, and a body that fails
  through `x.join()?` reports `x`'s error, not `Cancelled`.
- In v0 the threads runtime runs one OS thread per task, so blocking tasks
  cannot exhaust a pool and production cannot deadlock where the simulator
  makes progress; `Tasks::limit` bounds concurrency when needed.
- **The binary chooses the runtime**: blocking, threads, evented, or
  simulated. Libraries never choose. The compiler knows from signatures which
  functions may suspend; under the evented runtime it compiles them to
  stackless state machines with non-moving heap frames, and under the blocking
  and thread runtimes it compiles them as ordinary functions. Recursive
  suspending calls allocate their frames.
- Foreign callbacks cannot suspend. A task holding a `ThreadBound` value is
  pinned to its thread while it holds it.

Rejected: Rust-style async; hidden effect-based suspension; unstructured
spawn; stackful tasks in compiled code (thread-local caching across migration
is unsound with foreign frames, and per-task stacks cap task counts,
`reviews/round1-performance.md` B3). The interpreter and simulator may use
stackful execution internally.

### D9. Parallelism: deterministic, with defined grouping

- `join(|| a(), || b())`, `par_map`, and `par_for` are library functions.
  `join` runs `Pure + Send` closures; `par_map` and `par_for` run
  `Fn + Pure + Send + Sync` closures, because they call one closure from
  several threads. Captured shared data must be `Pure + Sync`; `&mut` data
  is reachable only through disjoint splits. Element types follow the same
  rule: `par_map`, `par_for`, `par_chunks_mut`, and `par_sort` require
  elements that are `Pure` plus `Sync` when shared or `Send` when split or
  moved, so no branch can reach a `Shared` cell, a channel, or a capability.
  `par_sort` is a stable merge sort whose splits and merges depend only on
  the input length; that structure's left-to-right order is its sequential
  order for panics and buffered sink output.
- When no branch panics, `join`, `par_map`, and `par_for` equal their
  sequential versions. When a branch panics, the panic propagates as in D5,
  and data the region could write has unspecified (memory-safe) contents
  that only destructors can observe. Under the simulator, parallel regions
  run sequentially in sequential order, so runs stay reproducible.
- `par_reduce` is defined by a canonical balanced grouping that depends only
  on input length, so results are bitwise identical on any core count. When
  the operation's trait carries a proved associativity law, the result also
  equals the sequential fold (a merely tested law gives that equality only as
  a counted assumption); for float addition it does not, and
  the canonical grouping is the definition.
- If several parallel branches panic, the one first in sequential order is
  reported, and buffered sink output from later branches is discarded; the
  region may still diverge if a branch after it diverges.
- Granularity is the runtime's job: lazy splitting first, with heartbeat
  promotion only if benchmarks justify it.
- An automatic parallelization pass for independent authority-free calls is
  behind a flag until benchmarks justify it.

### D10. Contracts: one spec, three uses, defined semantics

- Clauses `requires`, `ensures`, `invariant`, `decreases`, with Dafny and
  Verus names, written in the grammar, not in macros. `result` names the
  return value; `old(x)` the entry value of a `&mut` parameter; `final(x)` the
  value a returned or stored `&mut` holds when its borrow ends. A bare `*x` of
  a `&mut` parameter in `ensures` is an error: write `old(*x)` or `final(*x)`.
- **Spec language.** Specs call only `spec fn`s, the built-in spec types, and
  functions marked `#[spec]`, which must be authority-free and proved total;
  the standard library marks its pure accessors (`len`, indexing, `get`,
  `contains`, and similar), so `xs.len()` and `xs[i]` mean the same in specs
  as in code. `spec fn`s must terminate. Spec expressions carry
  well-formedness obligations (no division by zero, indices in bounds), as in
  Dafny. Floats in specs are IEEE values, not reals.
  Spec integers are mathematical; `/` and `%` truncate as in Rust; `==` is
  model equality; quantifiers range over spec types. Closures appear in specs
  only through `f.requires(args)` and `f.ensures(args, r)`.
- In v0, executable clauses use code semantics: machine integers that trap,
  and `==` through `PartialEq`. Clauses may call only authority-free
  functions whose argument types are `Pure + Sync` and that take `&mut` only
  to temporaries the clause itself creates (so `xs.iter().all(...)` works),
  so evaluating a clause cannot change program state. A clause whose
  evaluation panics counts as a violation; a sampled clause that exceeds its
  step budget is reported as unchecked, not as failed. In `ensures`, a bare
  `*x` of a `&mut` parameter means its value at return and `old(*x)` its
  value at entry. `old(e)` evaluates `e` at entry and snapshots the value it
  denotes, copying `Copy` values and cloning others; `old` of a type that is
  neither `Copy` nor `Clone` is an error, and a panic while taking a snapshot
  counts as a violation of the clause that needed it. One skip rule decides
  when a clause runs: clauses built only from operators, literals, and
  standard-library accessors are `checked` in debug and test builds;
  quantifiers and clauses that call user-defined functions are `sampled`
  and run under `tide test`, on every call and within their step budget
  (steps count evaluated operations, so exhausting a budget is
  deterministic), so
  a clause's outcome depends only on its inputs and the build mode, never on
  how many times it has run. In generic code, a clause that passes a value of
  type `T` to a function requires `T: Pure + Sync`. Mathematical integers
  and model equality
  apply under `prove` (v1), and the compiler warns where a clause would read
  differently under the two.
- **Uses**, chosen per package and overridable per item:
  1. `assert`: executable clauses become runtime checks in debug and test
     builds;
  2. `test`: clauses also drive property tests and fuzzing (`requires`
     filters and guides generation), and loop invariants are checked;
  3. `prove`: an SMT solver proves **total correctness**. Every panic site,
     including arithmetic, indexing, `unwrap`, and assertions, is a proof
     obligation, and every loop and recursion needs `decreases` or an explicit
     `diverges` marker. Lemmas must terminate.
- The compiler classifies each clause as `checked` (cheap to execute),
  `sampled` (executable but expensive; checked only in `test` mode), or
  `proof-only` (unbounded quantifiers), and reports unchecked clauses.
- **Trait laws.** Traits declare bodiless `law fn`s, such as transitivity for
  `Ord` or associativity for `par_reduce` operations. Under `prove` each impl
  proves them; under `test` they become property tests. Derived impls come
  with generated proofs.
- Trait methods may carry contracts; impls may weaken preconditions and
  strengthen postconditions, never the reverse.
- **Spec quality.** The toolchain checks that preconditions are satisfiable,
  postconditions are not trivially true, and assumptions are not
  contradictory, and runs mutation testing of specs in CI.
- **Capabilities have no spec model in v1.** A function that uses a
  capability may state postconditions about its results that hold for every
  possible response of the world.
- Specs never affect typing or code generation, except that proved facts may
  delete runtime checks. Release builds erase specs unless asked to keep them.
- Solver stability: executable function bodies are opaque to callers by
  default (`spec fn` bodies are transparent); quantifiers need
  triggers; nonlinear and bit-vector reasoning stays in marked blocks; the
  solver version is pinned; CI checks proof stability under seed changes.
- Interactive proofs are out of v1; the escape hatch is `#[trusted]` with a
  reason.

### D11. Trust surface and transitive trust

- Each package interface exports, per public item, its signature, declared and
  exercised authority, contracts, and **proof status**: proved, tested, or
  assumed, always shown with its assumptions (`proved, assuming 2`).
- Trust holes are counted transitively: `unsafe`, `#[trusted]`,
  `#[trusted_pure]`, `assume`, foreign calls, unproved impls of laws, and
  unproved dependency contracts that a proof relies on.
- `tide review <base>` prints the trust diff. It also reports **spec
  coverage**: which public items have contracts and at what status. Items
  without contracts still need ordinary review and tests; the trust diff
  narrows review, it does not replace it.
- When a contract changes, `tide review` shows concrete inputs on which the
  old and new contracts disagree, so approvers judge examples rather than
  logic.
- **Behavior diff** (v1): `tide review` reruns the same simulation seeds on
  base and head and diffs the capability events each run produced, so
  reviewers see what the change does, not only what it may do.
- Contract hashes cover the bodies of every function a clause calls,
  transitively, so editing a predicate a contract relies on is a contract
  change.
- **What needs approval**: weakening or removing a contract, widening
  authority, adding a trust hole (including `diverges`), downgrading a
  package or item from `prove` to `test` or `assert`, and changing a
  `#[protected]` test.
- **Approval lives outside the agent's write scope.** Approvals are records
  signed by a human-held key and verified by CI. A hash written in the same
  change is never approval, and proof caches are rebuilt in CI rather than
  trusted. Hashes cover everything a spec's meaning depends on, including the
  bodies of `spec fn`s it calls.

### D12. Compilation and backends

- A module is a file or an inline `mod` block; the module tree follows the
  directory tree. Imports are explicit; cycles and unused imports are errors.
- Each module compiles to an interface artifact (signatures, layouts,
  contracts, public `spec fn` bodies, views, type invariants, laws,
  authority, proof status, and which trait methods generic code calls) with a
  content hash. Editing a public `spec fn` body is an interface change, and
  every `const fn` body evaluated at compile time is part of its consumers'
  cache keys. Dependents read
  only their direct imports' interfaces; a body edit never re-checks
  dependents.
- Checking, borrow checking, authority checking, and proving are per
  function, parallel, and cached by content hash.
- **Generics are monomorphized in every profile** through one
  content-addressed instantiation cache shared across the package graph and
  across builds. Witness-table code is a budgeted fallback for very large or
  cold instantiations and must behave identically.
- **Backends**, all from one mid-level IR:
  1. a reference interpreter: the semantic oracle, `const fn` evaluator, and
     simulator engine;
  2. a Cranelift-class development backend with mid-IR inlining and an
     incremental linker, in v1;
  3. a release backend that emits C and builds it with an optimizing C
     compiler, which gives LLVM-class code, portable targets, and WASM through
     clang. A direct LLVM backend is a later option if C limits debug info or
     performance.
- **Targets** for a 100,000-line project: edit to diagnostics around 30 ms
  typical, under 100 ms at the 95th percentile; edit to running tests under
  1 s; clean debug build a few seconds; development-build runtime within 3x of
  release; release runtime within 10–20% of equivalent Rust.
- The compiler is written in Rust and not self-hosted before 1.0.

- An **origin cross-check** tool translates signatures and bodies to Rust,
  which for lifetimes is nearly the identity, and runs `rustc`'s borrow
  checker over the test corpus, so the Tide checker is continuously compared with Rust's. It is a
  test oracle, not a backend.

Rejected: Rust emission as the release backend (53 s clean builds, coupling
to rustc's borrow precision, no route to stackless lowering of Tide tasks);
witness tables by default (6–24x slower); C++ interop, MLIR, and GPU before
1.0.

### D13. Packages and supply chain

- Tide packages have no build scripts, install hooks, or code that runs at
  fetch or build time. The one exception is an imported Rust crate's build
  script, which runs sandboxed and counts as a trust hole.
- Lockfiles are on by default with content hashes; publishing requires
  trusted publishing; package names are namespaced; the client rejects
  dependencies that do not exist rather than guessing near names.
- Each package's manifest lists escape-hatch grants per dependency (`unsafe`,
  `ffi`), and the lockfile records each package's public authority and trust
  holes. Widening either needs approval (D11).
- **Rust crates** are imported through generated C-ABI shims for synchronous
  APIs. Each imported crate declares its authority in the manifest; the shim
  requires those capabilities; a symbol scan of the compiled crate checks the
  declaration; its build script, if any, runs sandboxed without network and
  with recorded outputs.

### D14. Toolchain contract for agents

- `tide check` returns one verdict and NDJSON diagnostics with stable codes,
  exact spans, related spans, and fix plans marked `safe` or `suggest`.
  Diagnostics for Rust habits (D1) carry safe fixes.
- `tide test` and `tide sim` return structured results: each failure has its
  seed, a replay command, and a capability event trace.
- **Typed holes**: `todo!()` or `_?name` in expression position checks the
  rest of the program and reports the expected type, in-scope bindings, and
  candidate functions.
- `tide eval` runs an expression in the interpreter against the current
  package. `tide verify` runs proofs and returns NDJSON results with proof
  status per item and counterexamples for failures. Coverage and spec coverage
  are reported.
- `#[protected]` tests are part of the trust surface; agents cannot change
  them without approval (D11).
- Every response carries a snapshot hash; every write request carries its base
  hash and applies atomically or not at all.
- `tide doc` serves documentation matched to the installed compiler: a card
  under 8 KB for an agent's instructions file, `tide explain <code>` with fix
  examples, and the delta-from-Rust reference. The core reference budget is
  25,000 tokens, counted in CI; the standard-library reference is separate
  and served on demand.
- `tide fix` applies safe fixes and owns migrations between language
  versions.
- Source text is canonical; the formatter is deterministic and has no
  options. Its style is rustfmt's default style (four-space indentation,
  100-column lines, opening braces on the same line), so formatting a
  Rust-habit draft changes almost nothing. Definitions are content-hashed; symbols have stable IDs.

### D15. Deterministic simulation as a first-class mode

- `Root::sim(seed)` provides deterministic clocks, network, filesystem,
  randomness, environment, task scheduling, and fault injection (drops,
  delays, partitions, disk errors, crashes and restarts of node scopes).
  Virtual time advances only when every task is blocked; a global deadlock is
  reported with the blocked-task graph. On a simulated crash, writes not made
  durable by `sync` or `sync_dir` may be lost or torn, as the seed decides.
- `tide sim <binary>` runs a program's `main` under many seeds. A failure
  prints seed and commit and replays bit for bit, dependencies included.
- Every simulated run writes a replay manifest: seed, the content hashes of
  the source and every dependency, a semantic build identity (compiler
  version, target, and every option that affects semantics), the simulated
  root's configuration (arguments, environment, files), and all injected
  inputs. Replay refuses to run against a mismatched identity. The standard library ships
  one math library for every target, `to_bits` canonicalizes NaN, and raw
  pointer addresses are observable only under an `unsafe` grant.
- Node entry points are `fn` items or `Fn` closures with `Owned` captures, so
  a restart reruns the entry against the node's durable storage and fresh
  volatile state.
- **Plan roots** (v1): `tide run --plan` runs a program with real reads but
  captured writes, network sends, and process spawns, prints the plan, and
  applies it only after approval. Agent-operated tools get a dry run that is
  exact rather than simulated.
- A determinism self-check runs a seed twice and diffs the event traces.
- Coverage: the build lists any reachable foreign effect with no simulator
  model.

### D16. Bend tandem

- Tide calls Bend 2 for pure, proved, massively parallel kernels; Bend may
  call Tide libraries through its foreign effects once Tide exports C ABIs.
- **Stage 0, available now**: a `bend` dependency kind with an exact version
  pin and cached builds; a generated shim lets Bend's own checker verify the
  declared kernel signature; kernels run as a supervised child process with a
  generated text protocol. A Bend crash maps to `Err`.
- **Stage 1, if Bend ships a native library target**: in-process calls under
  an `ffi` grant. Tide never depends on this stage.
- The boundary has a closed type map (no 64-bit Bend numbers yet, bounded
  naturals, power-of-two arrays, strings copied); values move or copy across,
  never share; closures and handles never cross.
- Calls require an `Accel` capability: one Bend runtime per process with a
  thread budget, kernels pure and total.
- A Tide contract can be exported as a Bend law, proved by Bend, and imported
  as an external claim at a lower trust level than a Tide proof.
- Deterministic `par` lets Tide and Bend implementations of one kernel be
  tested against each other.

### D17. Record, replay, and authority policies

- **Production record and replay** (v1): a binary can run with a recording
  root that logs every capability response to a compact trace. Replaying the
  trace under the simulator reproduces the run bit for bit, so an agent can
  debug a production incident deterministically, add a test from it, and
  verify a fix against the recorded inputs. Traces are queryable structured
  data. `Secret<T>` values are redacted at record time.
- **Snapshot and branch** (later): the simulator can snapshot a run and
  explore alternative schedules and faults from that point.
- **Authority policies** (v0): a package can declare rules over module paths,
  such as `parser::* is authority-free` or `only net::* may use Net`, which
  the checker enforces and the trust diff reports. Policies are append-only
  across releases unless an approval (D11) removes one.
- **Replay queries** (v1): recorded and simulated runs answer structured
  questions such as which task last wrote a value or which capability call
  returned an error, searching backward from the failure, which models handle
  better than stepping forward.

## The Feel

```tide
use std::net::{Net, NetError, Url};
use std::time::{Clock, Duration};
use std::json;

#[derive(Debug, PartialEq, Decode)]
pub struct Entry {
    pub title: String,
    pub link: Url,
}

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("network: {0}")]
    Net(#[from] NetError),
    #[error("bad feed: {0}")]
    Parse(#[from] json::Error),
}

/// Authority: network and clock, nothing else.
pub fn fetch_feed(net: &Net, clock: &Clock, url: &Url) -> Result<Vec<Entry>, FetchError> {
    let deadline = clock.now() + Duration::from_secs(5);
    let body = net.get(url, deadline)?;
    info!("fetched {} bytes from {url}", body.len());
    Ok(parse_feed(&body)?)
}

/// Authority-free: deterministic, parallelizable, simulable.
pub fn parse_feed(body: &[u8]) -> Result<Vec<Entry>, json::Error> {
    json::decode(body)
}

/// Checked at run time under `assert`, fuzzed under `test`, proved under `prove`.
pub fn max_index(xs: &[i64]) -> usize
    requires xs.len() > 0
    ensures result < xs.len()
    ensures forall |i: usize| i < xs.len() ==> xs[result] >= xs[i]
{
    let mut best = 0;
    for i in 1..xs.len()
        invariant best < i
        invariant forall |j: usize| j < i ==> xs[best] >= xs[j]
        decreases xs.len() - i
    {
        if xs[i] > xs[best] {
            best = i;
        }
    }
    best
}

/// Rust's lifetimes: the word borrows the source, not the lexer.
pub struct Lexer<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn next_word(&mut self) -> Option<&'a str> {
        let rest = &self.src[self.pos..];
        let word = rest.split_whitespace().next()?;
        self.pos += rest.find(word)? + word.len();
        Some(word)
    }
}

fn main(root: Root) -> Result<(), Box<dyn Error>> {
    let net = root.net.only(["example.com:443"]);
    let url = Url::parse("https://example.com/feed.json")?;
    let entries = fetch_feed(&net, &root.clock, &url)?;
    println!("{} entries", entries.len());
    Ok(())
}

#[test]
fn feed_times_out() {
    let sim = Root::sim(42);
    let net = sim.net.drop_all();
    let url = Url::parse("https://example.com/").unwrap();
    let result = fetch_feed(&net, &sim.clock, &url);
    assert!(matches!(result, Err(FetchError::Net(NetError::Timeout))));
}
```

## Open Questions

1. The exact deferred-trap rule for overflow and its interaction with proofs.
2. Field-borrow declaration syntax on methods and its effect on public API
   stability.
3. The simulator model for foreign effects and for Rust crates imported
   through shims.
4. Whether authority-free public functions should be declarable (`#[pure]`)
   so that adding authority is a visible breaking change.
5. The interpreter's role in `tide test` once the development backend exists.
6. How spec coverage and exercised authority are presented so the trust diff
   stays small enough to read.
7. How the C-emitting release backend (D12, v1) implements unwinding for
   panics, which must drop values (D5): a flag check after every call,
   compiler-specific extensions, or abort in release builds. If none is
   acceptable, this is the main reason to switch to a direct LLVM backend.
