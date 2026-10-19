# Ferox Design

Status: draft 2.1. Draft 1 was synthesized from `design/00-first-principles.md`
and `research/`; draft 2 answers the six round-1 reviews in `reviews/`. This
document owns the design decisions. Chapters elaborate them and must not
contradict them; a change of decision is made here first.

## Thesis

Ferox is a systems language for software that agents write and humans must
trust.

It keeps what makes Rust worth learning: memory safety and data-race freedom
without a garbage collector, zero-cost abstractions, algebraic data types,
`Result` and `?`, and "if it compiles, it probably works". It keeps Rust's
spelling wherever the meaning is the same, so a model's Rust knowledge
transfers. It removes what costs agents the most: lifetime parameters, user
macros, async coloring, an unbounded trait system, and slow builds.

It adds guarantees that a language can only make if it is designed for them
from the start, and that tools layered on Rust cannot give, because Rust code
can reach the outside world from anywhere:

1. **No ambient authority.** Code touches the world only through capability
   values it is handed. A function's signature bounds what it can do; code
   without capabilities cannot read the clock, the network, files, the
   environment, or randomness, in any dependency.
2. **Deterministic simulation of whole programs.** Because every source of
   nondeterminism is a capability, any Ferox program, dependencies included,
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
- **Rust coexistence**: Ferox builds C-ABI libraries with generated Rust
  binding crates, and imports synchronous Rust leaf crates through generated
  shims. Async Rust ecosystems (tokio, axum) and lifetime-heavy APIs
  (serde's `'de`) are out of scope for interop.
- **Bend tandem, not competition**: Ferox is the effectful, authority-checked
  shell; Bend 2 owns proved, massively parallel kernels (D16).
- **Evidence-gated**: the thesis is tested by the validation program (D0)
  before syntax freezes.

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

- **E1 learnability**: a fresh agent given only the Ferox reference writes a
  fixed task set; pass rates and iterations to green are compared with the
  same agent writing Rust.
- **E2 productivity**: equivalent multi-file tasks in Ferox (interpreter) and
  Rust, measuring wall time, iterations, and defects.
- **E3 review**: reviewers detect planted malicious or authority-widening
  changes using the trust diff versus the plain code diff.
- **E4 compile speed and E5 runtime**: benchmark suites against the D12
  targets and against Rust.
- **E6 plumbing**: the same application written with capabilities by agents,
  measuring how often they pass `Root`-like bundles everywhere and how large
  the resulting trust diffs are.

Initial thresholds, revised as data arrives: on a 30-task suite, Ferox
first-check pass rate at least 0.8x the Rust rate and final pass rate at least
the Rust rate within 1.5x the iterations (E1); reviewers using the trust diff
catch at least 1.5x as many planted authority-widening changes as with the
plain diff (E3). Gates: month 3, E1 on the interpreter; month 6, E1 and E2;
month 9, E3 to E6. Each gate decides continue, adjust, or stop. Syntax in D1 is
provisional until E1 meets its thresholds.

### D1. Surface: Rust spelling wherever the meaning is the same

- Rust syntax is Ferox syntax for every construct with the same meaning,
  including `where` clauses, `pub(crate)`, inline `mod` blocks,
  `#[derive(...)]` with Rust's trait names, `#[cfg(test)]`, `'static`,
  `From`/`TryFrom`/`Into`, `wrapping_add`, `checked_add`, `saturating_add`,
  `Wrapping<T>`, `Box<dyn Error>`, and `// SAFETY:` comments.
- A **fixed, compiler-implemented set of `!` forms** keeps Rust's most common
  macro spellings with their Rust meaning: `println!`, `print!`, `eprintln!`,
  `eprint!`, `format!`, `write!`, `writeln!`, `vec!`, `assert!`,
  `assert_eq!`, `assert_ne!`, `debug_assert!`, `matches!`, `panic!`,
  `todo!`, `unimplemented!`, `unreachable!`, `dbg!`, and the logging forms
  `error!`, `warn!`, `info!`, `debug!`, `trace!`. Users cannot define macros
  (D6).
- Named lifetimes whose use maps exactly onto origins are accepted as a
  legacy spelling and rewritten to `from` by the formatter; other uses are
  errors with suggested origins (D2).
- Where Ferox's meaning differs, the Rust spelling is rejected with a
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

### D2. Memory: ownership with origins instead of lifetime parameters

- Every value has one owner; moves are the default for non-`Copy` types.
  References `&T` and `&mut T` follow Rust's shared-xor-mutable rule.
- **Origins replace named lifetimes.** An origin names where a borrow comes
  from: a parameter, a place path, or `'static`.
  - Rust's lifetime-elision rules apply unchanged, so
    `fn get(&self, k: &str) -> Option<&V>` borrows from `self` exactly as in
    Rust.
  - Otherwise the signature names the origin with a positional `from` clause:
    `fn longest(a: &str, b: &str) -> &str from a, b` (the result may borrow
    from either), `fn next(&mut self) -> Option<&str from self.src>` (from the
    place `self.src`, not from `self`). `from` is a contextual keyword after a
    type in signatures, field declarations, and `dyn` types; inside generic
    arguments several origins are parenthesized: `Vec<&str from (a, b)>`.
  - In an impl, `Self`'s origins are named by field:
    `impl Iterator for Lexer { type Item = &str from Self.src; ... }`.
- **Types carry origins per reference field.** `struct Lexer { src: &str, pos:
  usize }` is a borrowing type with one origin, named `src`. A struct gets one
  origin per reference-typed field and inherits the origins of its type
  arguments; `Vec<&str>` borrows what its elements borrow. There is no
  `<'a>` parameter list. Variance follows Rust: shared references are
  covariant, `&mut T` and cells are invariant in `T`.
- **`'static` and `Owned`.** A type with no borrowed origins is `Owned`
  (Rust's `T: 'static`). `dyn Trait` means `dyn Trait + Owned` unless written
  `dyn Trait from x`. A closure that captures a borrow is itself a borrowing
  value; a borrowing value may be returned or stored only where its origins are
  named (`-> impl Fn() from xs`, a field with `from`). Detached tasks and
  global registrations require `Owned`.
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

Rejected: Rust lifetime parameters (annotation burden, the `'a` error class);
draft 1's "one implicit region per type" (unsound through `&mut` or too
restrictive, `reviews/round1-soundness.md` B1–B3); projections-only borrow
return (unfamiliar); shared-mutable-alias models (unproven at scale).

Accepted costs: self-referential structs need `unsafe`; two fields that must
share one origin need an explicit `from` on the field
(`b: &str from a`).

### D3. Types: small, decidable, explicit at boundaries

- Rust's scalar, string, slice, array, tuple, `Option`, `Result`, `Vec`,
  `Box`, and `String` types with their Rust meaning. No null.
- Nominal `struct` and `enum` with exhaustive `match`; `impl Trait` in
  argument and return position as in Rust.
- Every item signature is annotated; inference is bidirectional and local to
  a function body. No overloading: one function per name per scope, one
  method per name per type. No polymorphic recursion.
- **Integers trap on overflow, division by zero, and over-wide shifts in every
  profile.** The trap may be reported late, but before the overflowed value
  influences any observable effect or leaves the enclosing loop or function,
  so the optimizer can batch checks in vectorized loops. A proof may delete a
  runtime check only if every fact it relies on is either proved inside the
  same build or still checked at run time; a `pub` precondition that a deleted
  check depends on stays checked at function entry.
- `as` is accepted only where it is lossless; lossy conversions use
  `TryFrom`, `truncate()`, or explicit float rounding methods.
- Floats are IEEE 754; `to_bits()` canonicalizes NaN so observable results
  match across backends and targets.

Rejected: signature inference, implicit conversions, overloading, custom
operators.

### D4. Abstraction: traits with a hard power ceiling

- Rust traits with methods, default methods, flat associated types and
  constants, and `impl Trait for Type`. Impls live in the module of the type
  or the trait.
- No overlapping impls, specialization, higher-kinded types, generic
  associated types, rank-N types, negative bounds, or user auto traits. Impl
  where-clauses must be structurally smaller than the impl head.
- Generics are definition-checked: instantiation never produces a type error.
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
  current task. A structured scope observes a child's panic as an error value.
  On that error path, every place the child borrowed mutably is treated as
  moved out (it must be reassigned before use), and every cell the child held
  locked is poisoned, so no half-updated data is ever observed.

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
- Code generators are build steps that write ordinary source files.

Rejected: token and procedural macros (opaque to agents and tools, uncached,
supply-chain holes); unrestricted compile-time execution; reflection-based
serialization as the default (5–7x slower in prior art).

### D7. Authority: capabilities as values, no ambient authority

- Every operation that reads or changes the outside world, or that can
  observe nondeterminism, is a method on a capability value: `Fs`, `Net`,
  `Clock`, `Rand`, `Env`, `Stdin`, `Proc`, `Tasks`, `Shared`. The standard
  library has no free functions that do I/O, read the time, draw randomness,
  read the environment, or spawn.
- `main` may take `root: Root`, the only source of capabilities. `Root` can be
  named only in `main`; a simulated root (`Root::sim(seed)`) only in tests and
  in `ferox sim`.
- Capabilities are ordinary parameters passed by reference. Attenuation is a
  method returning a narrower capability: `root.fs.dir("/var/lib/app",
  Mode::ReadWrite)`, `root.net.only(["db:5432"])`.
- Safe code has no mutable globals and no thread-locals.
- **Write-only diagnostic sinks are not capabilities.** `print!`, `println!`,
  `eprint!`, `eprintln!`, `dbg!`, and the built-in `log` and `trace` APIs write
  to sinks owned by the current task and configured by `main`. They cannot
  read anything back, they do not affect purity, and inside `par` their output
  is buffered and emitted in sequential order. `dbg!` is an error in release
  builds; printing to stdout from a library package is a lint. Sink use is
  still reported as exercised effect in the trust surface, and `Secret<T>`
  values refuse `Debug` and `Display` so they cannot be logged by accident.
- **Authority-free types.** `Pure` is a compiler-known auto trait: a type is
  `Pure` if no capability, synchronization handle (channel ends, `Shared<T>`
  handles, task handles), or foreign handle is reachable from it. A function
  is authority-free if all its parameter and capture types are `Pure`. An
  authority-free function cannot observe nondeterminism: its results depend
  only on its arguments.
- **Frozen data.** `Frozen` is a second compiler-known auto trait: a type is
  `Frozen` if it has no interior mutability reachable through shared
  references (no `Cell`, `RefCell`, atomics). `par` (D9) combines `Pure` with
  `Frozen`; memoization and `const fn` additionally forbid `&mut`
  parameters. A call through a closure or `dyn` value may suspend exactly when
  that value's type is not `Pure`.
- **Foreign calls carry authority.** `extern` declarations are allowed only in
  packages granted `ffi`. Each foreign function is wrapped by a Ferox
  signature that takes the capabilities it may use, or is marked
  `#[trusted_pure]` with a reason, which counts as a trust hole.
- The manifest grants escape hatches per dependency: `unsafe`, `ffi`. There
  are no build scripts or install hooks. The lockfile records each package's
  public authority; an upgrade that widens it fails until approved (D13).
- Tooling reports **declared and exercised authority** for every public item
  and lints over-granted capabilities. `ferox fix` can thread a capability
  through a call chain.

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
  cancelled scope return `Err(Cancelled)`; suspended work unwinds and runs
  destructors; operations that consume a value return it inside the error.
- **Shared mutable state** is message passing over channels, or
  `lock (a, b) |x, y| { ... }` over `Shared<T>` cells with `T: Send + Pure`.
  The lock acquires all named cells atomically in a global order and checks
  at run time that the handles are distinct. Acquisition is not a
  cancellation point. Its body may call only authority-free functions and may
  not contain `lock`; payloads are `Pure`, so they hold no `Shared` handles,
  and the body can neither suspend nor take another lock. A panicking body
  poisons the cells, and `lock` returns `Result`. A cell's invariant is its
  payload type's invariant.
- Data-race freedom comes from ownership plus `Send` and `Sync`, derived
  structurally. The task, not the OS thread, is the isolation unit; values
  that never leave a task need not be `Send`.
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

- `par { a(), b() }`, `join`, and parallel iterators run closures that are
  authority-free and `Send`, capturing shared data only if it is `Frozen`, or
  holding disjoint `&mut` splits.
- `par_map` and `par_for` equal their sequential versions.
- `par_reduce` is defined by a canonical balanced grouping that depends only
  on input length, so results are bitwise identical on any core count. When
  the operation's trait carries a proved or tested associativity law, the
  result also equals the sequential fold; for float addition it does not, and
  the canonical grouping is the definition.
- If several parallel branches panic, the one first in sequential order is
  reported.
- Granularity is the runtime's job (heartbeat scheduling).
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
- Solver stability: bodies are opaque to callers by default; quantifiers need
  triggers; nonlinear and bit-vector reasoning stays in marked blocks; the
  solver version is pinned; CI checks proof stability under seed changes.
- Interactive proofs are out of v1; the escape hatch is `#[trusted]` with a
  reason.

### D11. Trust surface and transitive trust

- Each package interface exports, per public item, its signature, declared and
  exercised authority, contracts, and **proof status**: proved, tested, or
  assumed.
- Trust holes are counted transitively: `unsafe`, `#[trusted]`,
  `#[trusted_pure]`, `assume`, foreign calls, unproved impls of laws, and
  unproved dependency contracts that a proof relies on.
- `ferox review <base>` prints the trust diff. It also reports **spec
  coverage**: which public items have contracts and at what status. Items
  without contracts still need ordinary review and tests; the trust diff
  narrows review, it does not replace it.
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
  contracts, authority, proof status) with a content hash. Dependents read
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

- An **origin cross-check** tool translates signatures and bodies to Rust
  with explicit lifetimes and runs `rustc`'s borrow checker over the test
  corpus, so the Ferox checker is continuously compared with Rust's. It is a
  test oracle, not a backend.

Rejected: Rust emission as the release backend (53 s clean builds, coupling
to rustc's borrow precision, no route to stackless lowering of Ferox tasks);
witness tables by default (6–24x slower); C++ interop, MLIR, and GPU before
1.0.

### D13. Packages and supply chain

- Ferox packages have no build scripts, install hooks, or code that runs at
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

- `ferox check` returns one verdict and NDJSON diagnostics with stable codes,
  exact spans, related spans, and fix plans marked `safe` or `suggest`.
  Diagnostics for Rust habits (D1) carry safe fixes.
- `ferox test` and `ferox sim` return structured results: each failure has its
  seed, a replay command, and a capability event trace.
- **Typed holes**: `todo!()` or `_?name` in expression position checks the
  rest of the program and reports the expected type, in-scope bindings, and
  candidate functions.
- `ferox eval` runs an expression in the interpreter against the current
  package. `ferox verify` runs proofs and returns NDJSON results with proof
  status per item and counterexamples for failures. Coverage and spec coverage
  are reported.
- `#[protected]` tests are part of the trust surface; agents cannot change
  them without approval (D11).
- Every response carries a snapshot hash; every write request carries its base
  hash and applies atomically or not at all.
- `ferox doc` serves documentation matched to the installed compiler: a card
  under 8 KB for an agent's instructions file, `ferox explain <code>` with fix
  examples, and the delta-from-Rust reference. The core reference budget is
  25,000 tokens, counted in CI; the standard-library reference is separate
  and served on demand.
- `ferox fix` applies safe fixes and owns migrations between language
  versions.
- Source text is canonical; the formatter is deterministic and has no
  options. Definitions are content-hashed; symbols have stable IDs.

### D15. Deterministic simulation as a first-class mode

- `Root::sim(seed)` provides deterministic clocks, network, filesystem,
  randomness, environment, task scheduling, and fault injection (drops,
  delays, partitions, disk errors, crashes and restarts of node scopes).
  Virtual time advances only when every task is blocked; a global deadlock is
  reported with the blocked-task graph.
- `ferox sim <binary>` runs a program's `main` under many seeds. A failure
  prints seed and commit and replays bit for bit, dependencies included.
- Authority-free `par` regions run on real cores under simulation, because
  their results are deterministic.
- A determinism self-check runs a seed twice and diffs the event traces.
- Coverage: the build lists any reachable foreign effect with no simulator
  model.

### D16. Bend tandem

- Ferox calls Bend 2 for pure, proved, massively parallel kernels; Bend may
  call Ferox libraries through its foreign effects once Ferox exports C ABIs.
- **Stage 0, available now**: a `bend` dependency kind with an exact version
  pin and cached builds; a generated shim lets Bend's own checker verify the
  declared kernel signature; kernels run as a supervised child process with a
  generated text protocol. A Bend crash maps to `Err`.
- **Stage 1, if Bend ships a native library target**: in-process calls under
  an `ffi` grant. Ferox never depends on this stage.
- The boundary has a closed type map (no 64-bit Bend numbers yet, bounded
  naturals, power-of-two arrays, strings copied); values move or copy across,
  never share; closures and handles never cross.
- Calls require an `Accel` capability: one Bend runtime per process with a
  thread budget, kernels pure and total.
- A Ferox contract can be exported as a Bend law, proved by Bend, and imported
  as an external claim at a lower trust level than a Ferox proof.
- Deterministic `par` lets Ferox and Bend implementations of one kernel be
  tested against each other.

## The Feel

```ferox
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

/// Origins instead of lifetimes.
pub struct Lexer {
    src: &str,
    pos: usize,
}

impl Lexer {
    pub fn next_word(&mut self) -> Option<&str from self.src> {
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
5. The interpreter's role in `ferox test` once the development backend exists.
6. How spec coverage and exercised authority are presented so the trust diff
   stays small enough to read.
