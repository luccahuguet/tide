# Ferox Design

Status: draft 1, synthesized from `design/00-first-principles.md` and the
reports in `research/`. This document owns the design decisions. Chapters in
`design/` elaborate them and must not contradict them; a change of decision is
made here first.

## Thesis

Ferox is a systems language for software that agents write and humans trust.

It keeps what makes Rust worth learning: memory safety and data-race freedom
without a garbage collector, zero-cost abstractions, algebraic data types,
`Result` and `?`, and "if it compiles, it probably works". It removes the
parts that cost the most and buy the least for agent-written code: lifetime
annotations, macros, async coloring, an unbounded trait system, and slow
builds. It adds three things no mainstream systems language has together:

1. **Authority in signatures.** Code can only touch the outside world through
   capability values it is handed. A signature tells a reviewer everything a
   function can do to the world. Everything else is pure, deterministic, and
   safe to parallelize.
2. **Contracts the toolchain proves.** `requires` and `ensures` clauses are
   part of the language, checked by an SMT solver when the package opts in,
   executed as assertions and property tests otherwise.
3. **A trust surface.** Signatures, capabilities, contracts, and trust holes
   (`unsafe`, `assume`, FFI) form a small, hashed surface that the toolchain
   diffs between revisions. Humans review the trust diff; machines check the
   bodies.

The one-line pitch: Rust's guarantees, Go's build speed, Dafny's proofs, and
authority you can read in a signature, in a language small enough to fit in an
agent's context.

## Positioning

- **A Rust replacement for new systems code**: services, CLIs,
  infrastructure, data systems. Not a research language.
- **A Rust companion during adoption.** Ferox packages build into Rust crates
  and call Rust crates, so a team adopts Ferox one module at a time
  (`research/09-adoption-interop-backends.md`).
- **Not a Bend competitor.** Bend 2 owns proved, massively parallel pure
  computation on CPUs and GPUs. Ferox owns effectful systems code and calls
  Bend for kernels (`design/` chapter on Bend tandem, pending
  `research/11-bend-interop.md`).

## Evidence That Shaped The Design

The decisions below cite these findings; details and sources are in the
research reports.

- New languages fail on syntax first: in unfamiliar languages most model
  failures were syntax borrowed from other languages. A Rust-like surface
  learned better than a smaller-community alternative with seven times more
  code (`research/05`, `research/09`).
- Compiler feedback is the largest single lever: precise diagnostics raised
  Rust repair rates from 10.7% to 73.7%, and execution feedback took a model
  from 45% to 82% on an unseen language (`research/05`).
- Frontier agents fixing real issues do best in Rust despite more first-try
  compile errors: strictness pays off once the agent can iterate
  (`research/05`).
- Most LLM Rust errors are type mismatches and trait bounds, not lifetimes;
  ownership errors are a minority but hard to repair without good fixes
  (`research/02`, `research/05`).
- SMT-backed contracts lead every benchmark for agent-written verified code
  (Dafny 82%, Verus 44%, Lean 27% in one 2025 study); generating correct specs
  is the harder half (`research/04`).
- Compile speed is decided by language design more than by backends: faster
  backends alone gained 5–17% end to end, while design changes gave 100x on
  incremental rebuilds (`research/03`).
- Every trait-system crisis in Rust and Swift came from power added later
  (`research/08`).
- Library-level deterministic simulation fails because dependencies reach
  ambient time, randomness, and threads (`research/07`, `research/10`).
- Agents navigate by grep and edit text; structured edit operations add
  little for strong models. Fast feedback, docs in context, and
  machine-applicable fixes add a lot (`research/06`).
- HigherOrderCO itself abandoned interaction nets as Bend's runtime; HVM is
  not a viable backend for a systems language (`research/01`).

## Decisions

Each decision states the choice, the reason, and what it rejects.

### D1. Surface: Rust where the meaning is the same

Ferox uses Rust's syntax for every construct whose meaning is the same in
Ferox: `fn`, `let`, `let mut`, `struct`, `enum`, `impl`, `trait`, `match`,
`if let`, `while let`, `for`, `loop`, closures `|x| ...`, `&` and `&mut`,
`Option<T>`, `Result<T, E>`, `?`, generics `<T: Bound>`, `::` paths,
`#[derive(...)]`, `#[test]`, `pub`, `use`.

Where Ferox differs, it uses new syntax rather than giving familiar syntax a
new meaning, and the compiler recognizes the Rust habit and offers a
machine-applicable fix (`println!` → `print`, `x as u8` → `x.truncate()`,
`'a` → `from`). Idiomatic Rust is a near-correct first draft of Ferox.

Rejected: a fresh syntax optimized in isolation (pays the low-resource
penalty with no data to recover it); `[T]` generics and `T?`/`T ! E` sugar
(second spellings of things models already write correctly).

### D2. Memory: owned values, second-class references, declared views

- Every value has one owner. Moves are the default for non-`Copy` types;
  copying anything else is an explicit `.clone()`.
- References `&T` and `&mut T` follow Rust's shared-xor-mutable rule.
- **No lifetime parameters.** References may be passed as parameters, used
  locally, and returned when they derive from parameters. A returned
  reference's source is inferred when exactly one parameter is borrowed;
  otherwise the signature names it: `fn pick(a: &str, b: &str) -> &str from a`.
- **Views.** A struct with a reference field is a *view type*. Views may hold
  references and other views, may be returned with `from`, and cannot be
  stored in non-view types or outlive their sources. Generic containers of
  views are views (`Vec<&str>` is a view). This is Rust's lifetime system
  restricted to one implicit region per type and one region per borrowed
  parameter, which removes lifetime annotations without losing soundness.
- Borrow checking is per function, flow- and field-sensitive at the
  precision of Polonius, with methods allowed to declare the fields they
  borrow so disjoint borrows compose.
- Non-tree data uses the standard library: `Arena<T>` with generational
  handles, arena-scoped references, and `Rc`/`Arc`/`Weak` with compiler-elided
  count traffic. There is no hidden reference counting of ordinary values.
- Closures do not escape by default. An escaping closure is `move` and owns
  everything it captures.

Rejected: full Rust lifetimes (annotation burden, `'a` errors, lifetime
generics); Hylo-style projections as the only borrow return (unfamiliar);
implicit reference counting fallback (hides cost); shared mutable aliasing
models such as group borrowing (unproven at scale, lose aliasing-based
optimization). Costs accepted: long-lived references inside heap collections
need views, handles, or `Rc`; self-referential structs need `unsafe`
(`research/02`).

### D3. Types: small, decidable, explicit at boundaries

- Scalars `bool`, `i8`–`i128`, `u8`–`u128`, `isize`, `usize`, `f32`, `f64`,
  `char`; `String`, `&str`, `Vec<T>`, `[T; N]`, slices, tuples.
- Nominal `struct` and `enum` with exhaustive `match`.
- No null; `Option<T>` with niche layout.
- Every item signature is fully annotated: functions, fields, constants,
  trait items. Inference is bidirectional and local to a function body. No
  overloading: one function per name per scope, one method per name per type.
- Integer arithmetic traps on overflow, division by zero, and over-wide
  shifts in every build profile. Wrapping and saturating arithmetic are
  explicit methods. There is no `as`; conversions are `widen()` (lossless),
  `narrow()` (returns `Option`), and `truncate()`.
- Strings are UTF-8; `==` is byte equality.

Rejected: global or signature inference (breaks locality, prefix typing, and
separate compilation); implicit conversions; overloading and custom operators
(exponential checking, many meanings per name).

### D4. Abstraction: traits with a hard power ceiling

- Traits with methods, default methods, and flat associated types and
  constants. `impl Trait for Type` lives in the module that defines the type
  or the trait.
- No overlapping impls, no specialization, no higher-kinded types, no generic
  associated types, no rank-N types, no negative bounds, no user auto traits.
  Impl where-clauses must be structurally smaller than the impl head.
- Generics are definition-checked: instantiation never produces a type error.
- Bounds are written in one place, the generic parameter list.
- `dyn Trait` is always spelled out.
- Generic code cannot inspect concrete types, so the shared-code strategy of
  development builds and the specialized code of release builds behave
  identically (D11).

Rejected: GATs, HKTs, and specialization (each cost Rust or Swift years and
breaks dev/release equivalence); ML functors (unfamiliar); Zig-style
`comptime` generics (instantiation-time errors, slow and nonlocal checking)
(`research/08`).

### D5. Errors: values, with declared conversions

- Recoverable errors are `Result<T, E>`; `?` propagates.
- `?` converts an error only through a variant the destination enum declares
  with `from`: `enum AppError { Net(NetError) from, Timeout }`. The conversion
  is visible in one place, the destination type.
- Results are must-use.
- Bugs (overflow, out-of-bounds, failed assertions, `panic`) abandon the
  current task. Recovery happens only at structured task boundaries, which
  observe the panic as an error value.

Rejected: exceptions (hidden control flow); Rust's open-ended `From`
conversions in `?` (a second, nonlocal resolution rule); anonymous error
unions (novel syntax; nominal enums are what models write).

### D6. No macros; reflection, const evaluation, and visible codegen

- A closed set of compiler-implemented derives: `Eq`, `Ord`, `Hash`,
  `Clone`, `Copy`, `Debug`, `Default`.
- Every nominal type exposes a compiler-provided `Shape` (fields, variants,
  names, layout), opt-out for types with invariants. Serialization, printing,
  and diffing become ordinary generic library code.
- `const fn` evaluation is pure, deterministic, fuel-limited, and cached. It
  computes values, never declarations or types.
- Code generators are build steps that write ordinary `.fx` source files the
  agent and reviewer can read.
- Formatting is a built-in string form: `f"{name} has {count} items"`,
  type-checked at compile time. `print`, `assert`, and `dbg` are ordinary
  built-in functions, not macros.

Rejected: token and procedural macros (opaque to agents and tools, uncached,
a supply-chain hole); unrestricted compile-time execution (`research/03`,
`research/08`).

### D7. Authority: capabilities as values, no ambient authority

- Every operation that observes or changes the outside world is a method on a
  capability value: `Fs`, `Net`, `Clock`, `Rand`, `Env`, `Stdio`, `Proc`,
  `Tasks`. The standard library has no free functions that do I/O, read the
  time, draw randomness, or spawn.
- `main` receives `root: Root`, the only source of capabilities. Nothing else
  can create one.
- Capabilities are ordinary parameters, passed by reference:
  `fn fetch(net: &Net, clock: &Clock, url: &Url) -> Result<Page, FetchError>`.
  Attenuation is a method returning a narrower capability:
  `root.fs.dir("/var/log/app", Mode::Append)`, `root.net.only(hosts)`.
- Safe code has no mutable globals and no thread-locals.
- **Purity is a type fact.** A type is `Pure` if it contains no capability.
  A function is pure if every parameter and capture is `Pure`. Pure functions
  are deterministic, eligible for `const fn`, memoization, `par`, and proofs.
- A type that stores a capability is authority-bearing by its fields;
  tooling shows the computed authority of every public type and function.
- `dbg` is the one untracked effect, for debugging, and warns in release
  builds.
- The package manifest grants only escape hatches per dependency: `unsafe`,
  `ffi`. There are no build scripts and no install hooks. The lockfile records
  each package's public authority, and an upgrade that widens it fails until
  approved.

Rejected: an effect type system with effect variables or rows (signature
noise, inference cost; capabilities passed as values give the same
reviewability and contextual polymorphism through borrowed closures);
implicit capability resolution through `with` clauses (a second argument
passing mechanism; explicit parameters are simpler, greppable, and support
attenuation with no extra syntax); user-defined handlers and continuations in
v1 (research stage, unsound with affine types when multi-shot)
(`research/07`).

### D8. Concurrency: structured tasks on stackful runtimes, no async

- There is no `async`/`await` and no futures in user code. A function may
  suspend only by using a capability; a pure function never suspends.
- Tasks are spawned into a scope from the `Tasks` capability and cannot
  outlive it. Children may borrow from the parent's frames. The first failure
  cancels siblings, then propagates.
- Cancellation is level-triggered with deadlines: once a scope is cancelled,
  capability operations in it return `Err(Cancelled)`. A suspended
  computation is never dropped silently; it unwinds and runs destructors.
  Operations that consume a value return it inside the error.
- Shared mutable state is message passing over channels, or
  `lock (a, b) |x, y| { ... }` over `Cell`-like shared cells, which acquires
  all named cells atomically. Inside a `lock` body no capability is in scope
  and no `lock` nests, so this primitive cannot deadlock or hold a lock across
  a suspension.
- Data-race freedom comes from ownership plus `Send` and `Share`, derived
  structurally. The task, not the OS thread, is the isolation unit.
- The binary chooses the runtime: blocking (no runtime, for CLIs), threads,
  evented (stackful tasks on work-stealing schedulers over io_uring, epoll,
  kqueue, or IOCP), or simulated. Libraries never choose.

Rejected: Rust-style async (coloring, `Pin`, `Send` puzzles, executor
fragmentation); hidden effect-based suspension (OCaml's Eio lesson: invisible
suspension points); unstructured spawn (`research/10`).

### D9. Parallelism: deterministic, with sequential semantics

- `par { a(), b() }`, `join`, and parallel iterators (`par_map`,
  `par_reduce`) run only pure closures without shared interior mutability.
  Results equal sequential execution, so proofs and tests of sequential code
  hold for parallel runs.
- The runtime decides granularity (heartbeat scheduling), so `par` is cheap
  enough to use freely.
- Reductions use a fixed tree shape that depends on input size, so float
  results are bitwise identical on any core count.
- An optional compiler pass parallelizes independent pure calls
  automatically, behind a flag until benchmarks justify it.
- Massively parallel and GPU kernels belong to Bend 2 (D14), not to Ferox v1.

### D10. Contracts and proofs: one spec, three uses

- `requires`, `ensures`, `invariant`, and `decreases` clauses, using Dafny
  and Verus names. `result` names the return value in `ensures`. Specs are
  written in a pure, terminating spec sublanguage with unbounded `int` and
  `nat`, `Seq`, `Set`, `Map`, and quantifiers.
- One spec serves three uses, chosen per package:
  1. `assert`: specs compile to runtime checks in debug and test builds;
  2. `test`: specs also drive generated property tests and fuzzing;
  3. `prove`: an SMT solver proves the specs; counterexamples become
     diagnostics; ghost code and lemmas are allowed.
- Specs never affect typing or code generation; release builds erase them
  unless asked to keep them.
- Executable code keeps machine integers; under `prove`, overflow is a proof
  obligation.
- Design for solver stability: function bodies are opaque to callers by
  default, quantifiers need triggers, nonlinear and bit-vector reasoning stays
  in marked blocks, the solver version is pinned, and CI checks proof
  stability under seed changes.
- Trust holes are keywords the compiler counts: `assume`, `#[trusted]`,
  `unsafe`. Their count and locations are part of the trust surface.
- Interactive proofs are out of v1; the escape hatch is `#[trusted]` with a
  stated reason, or a lemma proved elsewhere.

Rejected: full dependent types (evaluation in the checker, slow and complex);
Lean-style tactic proofs as the main layer (weakest agent success on program
proofs); macro-embedded spec syntax (a major source of model syntax errors in
Verus) (`research/04`).

### D11. Compilation: compiled interfaces, function-level incrementality

- A module is a file; the module tree is the directory tree. Imports are
  explicit, cycles are errors, unused imports are errors.
- Each module compiles to an interface artifact (signatures, type layouts,
  contracts, authority) with a content hash. Dependents read only interfaces
  of their direct imports; a body edit never re-checks dependents.
- Checking, borrow checking, authority checking, and proving are per
  function, parallel, and cached by content hash.
- Generics compile once to shared code with witness tables in development
  builds; release builds monomorphize through a content-addressed
  instantiation cache across the whole package graph.
- Targets for a 100,000-line project: edit to diagnostics around 30 ms
  typical and under 100 ms at the 95th percentile; clean check under 2 s;
  clean debug build around 1 s plus link (`research/03`).

### D12. Backends and bootstrap

1. **Reference interpreter first.** The checker plus a reference interpreter
   is the first deliverable and the semantic oracle for everything after it.
   It also gives agents an instant edit-to-test loop before native backends
   exist.
2. **Rust emission for release and interop.** Ferox lowers to readable Rust
   that uses a small Ferox runtime crate. Release builds get LLVM
   optimization, and every Ferox package is a Rust crate, so Ferox lives
   inside Cargo workspaces. Ferox's own checker is authoritative; `rustc`
   is a second check, never the definition.
3. **Native development backend later**, Cranelift-class, with an incremental
   linker, once the interpreter stops being fast enough.
4. The compiler is written in Rust and is not self-hosted before 1.0.
5. No C++ interop, MLIR, or GPU backend before 1.0.

### D13. Toolchain contract for agents

- One command answers "is this right?": `ferox check` returns one verdict and
  NDJSON diagnostics with stable codes, exact spans, related spans, and fix
  plans marked `safe` or `suggest`.
- **Typed holes**: `_?name` in an expression position checks the rest of the
  program and reports the expected type, the in-scope bindings with their
  types, and candidate functions.
- Every response carries a snapshot hash; every write request carries its base
  hash and applies atomically or not at all.
- `ferox doc` serves documentation matched to the installed compiler,
  including a short always-on card (under 8 KB) for an agent's instructions
  file, `ferox explain <code>` with fix examples, and a full reference that
  fits in context.
- `ferox fix` applies safe fixes and owns migrations between language
  versions, so models never need to learn deprecated idioms.
- `ferox review <base>` prints the trust-surface diff: changed public
  signatures, authority, contracts, trust holes, and dependency authority.
- Specs, `#[protected]` tests, and the trust surface are hashed in a lockfile;
  agent edits to them are reported separately and require approval.
- Source text is canonical. The formatter is deterministic and has no
  options. Definitions are content-hashed for caching and conflict detection;
  symbols have stable IDs for queries.

### D14. Bend tandem

Ferox calls Bend 2 for pure, proved, massively parallel kernels through a
declared foreign boundary; the details wait for
`research/11-bend-interop.md`. Ferox does not ship interaction-net
semantics, superposition search, or a GPU backend of its own.

## The Feel

```ferox
use std::net::{Net, NetError, Url, UrlError};
use std::time::{Clock, Duration};
use std::json;

#[derive(Debug, Eq)]
pub struct Entry {
    pub title: String,
    pub link: Url,
}

pub enum FetchError {
    Net(NetError) from,
    Parse(json::Error) from,
}

/// Authority: network and clock, nothing else.
pub fn fetch_feed(net: &Net, clock: &Clock, url: &Url) -> Result<Vec<Entry>, FetchError> {
    let deadline = clock.now() + Duration::seconds(5);
    let body = net.get(url, deadline)?;
    let entries = parse_feed(&body)?;
    Ok(entries)
}

/// Pure: no capability in the signature, so deterministic and parallelizable.
pub fn parse_feed(body: &[u8]) -> Result<Vec<Entry>, json::Error> {
    json::decode(body)
}

/// Proved under `prove`; checked at run time under `assert`.
pub fn max_index(xs: &[i64]) -> usize
    requires xs.len() > 0
    ensures result < xs.len()
    ensures forall |i: usize| i < xs.len() ==> xs[result] >= xs[i]
{
    let mut best = 0;
    for i in 1..xs.len()
        invariant best < i
        invariant forall |j: usize| j < i ==> xs[best] >= xs[j]
    {
        if xs[i] > xs[best] {
            best = i;
        }
    }
    best
}

pub enum AppError {
    Fetch(FetchError) from,
    Url(UrlError) from,
}

fn main(root: Root) -> Result<(), AppError> {
    let net = root.net.only(["example.com:443"]);
    let url = Url::parse("https://example.com/feed.json")?;
    let entries = fetch_feed(&net, &root.clock, &url)?;
    root.stdio.print(f"{entries.len()} entries");
    Ok(())
}

#[test]
fn feed_times_out() {
    let sim = Root::sim(42);
    let net = sim.net.drop_all();
    let result = fetch_feed(&net, &sim.clock, &Url::parse("https://example.com/").unwrap());
    let Err(FetchError::Net(NetError::Timeout)) = result else {
        panic("expected a timeout");
    };
}
```

## Open Questions

1. Visibility levels: module-private by default with `pub` exporting from the
   package, or a separate package-internal level.
2. Exact view-type rules for generic containers and closures capturing views.
3. How the `Shape` reflection interacts with types that keep invariants
   private.
4. Whether stackful tasks meet memory targets for very high task counts, and
   the FFI cost of stack switching.
5. Bend 2 boundary: types, ownership, proof claims, runtime ownership.
6. The macro-free spellings of `matches!`, `vec!`, `format!`, and `assert_eq!`.
7. Whether the `Pure` property should be declarable on public functions so
   that adding authority to a pure public function is a visible breaking
   change rather than an inferred one.
