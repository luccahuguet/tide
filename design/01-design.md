# Tide Design

Status: draft 3.7. Draft 1 was synthesized from `design/00-first-principles.md`
and `research/`; draft 2 answers the six round-1 reviews in `reviews/`; draft
3.4 returned D2 to Rust's lifetime spelling after the origin trial
(`trials/e1-origins/`); draft 3.5 takes Rust 2024's lifetimes whole
after review rounds 8 to 10; and draft 3.6 adopts the checked Rust dialect
as the v0 direction (D18), whose feasibility spike passed; draft 3.7
applies D18's changes to the other decisions. This
document owns the design decisions. Chapters elaborate them and must not
contradict them; a change of decision is made here first.

## Thesis

Tide is a systems language for software that agents write and humans must
trust.

It keeps what makes Rust worth learning: memory safety and data-race freedom
without a garbage collector, zero-cost abstractions, algebraic data types,
`Result` and `?`, and "if it compiles, it probably works". It keeps Rust's
spelling wherever the meaning is the same, so a model's Rust knowledge
transfers; under D18 every Tide program is a valid Rust crate. It removes
what costs agents the most: user macros, async coloring, an unbounded trait
system, and slow edit loops. It keeps Rust 2024's
lifetimes, minus the forms beyond D4's power ceiling.

It adds guarantees that tools layered on Rust cannot give, because Rust code
can reach the outside world from anywhere through Rust's std. Tide gets them
by compiling checked Rust against its own std and admitting only the `core`
and `alloc` items that cannot reach the world (D18):

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

Contracts (`#[requires]`, `#[ensures]`) are toolchain attributes and serve as
runtime checks, property tests, and optional SMT proofs. They support the
thesis; they are not the headline. Proof-first, massively parallel pure
computation is Bend's territory (D16).

One line: a capability-safe, replayable subset of Rust. Authority you can
read in a signature and whole-program deterministic replay, with Rust's
guarantees; a much faster edit loop is D12's target, not yet a measurement.

## Positioning And Scope

- **v0 target**: command-line tools, developer tooling, and infrastructure
  components (agents, proxies, storage engines, schedulers). Blocking and
  thread runtimes plus the simulator; the evented runtime follows.
- **Rust coexistence**: under D18 a Tide crate is a Rust crate: Rust code
  depends on it directly (v1), sharing the std types Tide re-exports, and Tide
  links synchronous Rust crates directly as trust holes behind capability
  wrappers. Async Rust ecosystems (tokio, axum) and
  lifetime-heavy APIs (serde's `'de`) are out of scope for interop.
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
| v0 core | Rust surface and ownership, with lifetimes checked as origins; traits with the power ceiling; `Result` and `?`; built-in `!` forms and derives (`Encode`/`Decode` for JSON only); capabilities and `Root`; `Pure`; sinks; structured tasks on blocking and thread runtimes; channels and `Shared::with`; `join`, `par_map`, `par_for`; executable contracts (`#[requires]`, `#[ensures]`, loop `invariant!`, `old`, `result`, bounded quantifiers) as runtime checks and property-test drivers; the simulator with crash, restart, and durability; declared authority and trust holes in the trust diff; sealed delegation boundaries (D11); the unreviewed-branch guarantee and the test sandbox on Linux (D13); per-dependency grants; `#[trusted_pure]` on types as a trust hole under an `unsafe` grant; the admission allowlist (D7); the bounded process profile, real and simulated: `Proc::command`, `Command`, `Child`, pipes, exit status and `only` (D7, D15; 03 §1.1, §9.9's lifecycle subset; research/40); the agent toolchain on rustc builds gated by the admission driver, then Tide's incremental front end for the edit loop (D12, D18) |
| v1 | `prove` mode with `spec fn`, views, `final`, `decreases`, laws, and proof status; exercised-authority tracking; the evented runtime; trait-method contracts and their inheritance by impls; FFI with `#[trusted_pure]` wrappers; direct Rust crate import with RUST-2 wrappers; the exported `Cargo.toml` for Rust consumers (RUST-6); strict JSON decoding and internally tagged enums (D6); `par_reduce` with canonical grouping; field-borrow declarations; production record and replay, plan roots, and replay queries (D17); kernel enforcement of authority for deployed binaries (D7); behavior diffs in `tide review` (D11) |
| Later | Bend tandem (D16); deferred overflow traps; automatic parallelization; read-only `Shape` reflection; Polonius-precision borrows; interactive-proof export; the rest of 03 §9.9: modeled signals, raw PTYs, parent-only process death and process faults through Proc (D15). `Proc::signals`, `Signals`, the API additions (`Child::signal`, `ExitStatus::signal`, `spawn_pty`/`Pty`) and subscription rules (one live subscription with `AlreadyExists`, default disposition on drop, five-signal set) are Later on both real and simulated backends. |

`prove` stays in v1 (owner delegated the choice, 2026-10-19): v0 contracts
already use the clause attributes agents will later prove, so moving proofs in
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
- **E2 productivity**: equivalent multi-file tasks in Tide and Rust, measuring
  wall time, iterations, and defects.
- **E3 review**: reviewers detect planted malicious or authority-widening
  changes using the trust diff versus the plain code diff.
- **E4 compile speed and E5 runtime**: benchmark suites against the D12
  targets and against Rust.
- **E6 plumbing**: the same application written with capabilities by agents,
  measuring how often they pass `Root`-like bundles everywhere and how large
  the resulting trust diffs are.
- **Conformance oracle**: every normative rule in the chapters carries an ID
  (`TASK-2`, `PAR-1`, ...), and each ID is traced to conformance tests that
  any implementation must pass, in the style of Rue. The corpus, its case
  format, and the ID scheme are in `conformance/README.md`. E1 reruns on every
  release with a fresh held-out task set.
- **Mechanized soundness**: before 1.0, the origin calculus (D2) and the
  determinism of authority-free code (D7) are proved in Lean, with agent
  help; a Lean proof of Move's borrow checker took about 27 days with Claude
  Code (`research/12-newcomer-sweep.md`).

Initial thresholds, revised as data arrives: on a 30-task suite, Tide
first-check pass rate at least 0.8x the Rust rate and final pass rate at least
the Rust rate within 1.5x the iterations (E1); reviewers using the trust diff
catch at least 1.5x as many planted authority-widening changes as with the
plain diff (E3). Gates: month 3, E1 on the first checked toolchain; month 6,
E1 and E2;
month 9, E3 to E6. Each gate decides continue, adjust, or stop. Syntax in D1 is
provisional until E1 meets its thresholds.

### D1. Surface: Rust spelling wherever the meaning is the same

- **Every Tide program is a valid Rust 2024 crate** (D18), and Rust syntax
  is Tide syntax for every construct with the same meaning, including `where`
  clauses, `pub(crate)`, inline `mod` blocks,
  `#[derive(...)]` with Rust's trait names, `#[cfg(test)]`, `'static`,
  `From`/`TryFrom`/`Into`, `wrapping_add`, `checked_add`, `saturating_add`,
  `Wrapping<T>`, `Box<dyn Error>`, and `// SAFETY:` comments.
- Blocks use braces and semicolons exactly as in Rust; indentation is never
  significant. Braces match model priors, survive diffs, markdown, and logs
  intact, and keep a mis-indented agent edit cosmetic instead of silently
  moving a statement into another block. Generics use angle brackets and
  Rust's turbofish.
- Source files are `.rs`. Modules are declared with `mod`, as in Rust;
  `#[path]` and `extern crate` are errors (`design/reference.md` §12.1–§12.2).
- A **fixed, compiler-implemented set of `!` forms** keeps Rust's most common
  macro spellings with their Rust meaning: `println!`, `print!`, `eprintln!`,
  `eprint!`, `format!`, `write!`, `writeln!`, `vec!`, `assert!`,
  `assert_eq!`, `assert_ne!`, `debug_assert!`, `matches!`, `panic!`,
  `todo!`, `unimplemented!`, `unreachable!`, `dbg!`, and the logging forms
  `error!`, `warn!`, `info!`, `debug!`, `trace!`. Users cannot define macros
  (D6).
- **Tide's own forms are toolchain attributes and `!` forms**, so they parse
  as Rust:
  - contracts are `#[requires(..)]` and `#[ensures(..)]` on functions (D10);
  - a loop invariant is `invariant!(..)` at the top of the loop body, in a
    function marked `#[invariants]` or carrying any contract attribute, which
    rewrites each loop so the invariant is checked as D10 specifies;
  - the entry point is `#[tide::main] fn main(root: Root)`, because rustc
    rejects parameters on a plain `main` (E0580); the attribute generates
    Rust's entry point and mints the `Root` (D7). Every binary's `main`
    carries it, so every binary starts the runtime and sinks and runs under
    `tide sim`; a `main` without it is `TD0512`, whose safe fix adds it;
  - a typed hole is `todo!()` or `hole!(name)` (D14).
- Lifetimes are Rust 2024's, except the forms beyond D4's power ceiling
  (D2).
- Where Tide's meaning differs, the Rust spelling is rejected with a
  machine-applicable fix: `async`/`.await`
  (→ capabilities, D8), lossy `as` (→ `try_from` or `truncate`, D3), ambient
  I/O such as `std::env::args()` or `std::fs::read` (→ capabilities, D7),
  float observers and math methods such as `x.to_bits()` and `x.sin()`
  (→ `float::bits(x)`, `math::sin(x)`, D3), other items outside the
  admission allowlist (D7), `impl` blocks outside the type's or trait's module
  (D4), user macros (D6).
- The reference is written as a delta from Rust and token-counted (budget
  in D14; CI counting is planned).

Rejected: fresh syntax (low-resource penalty with no data to recover it);
respelling constructs that mean the same thing (`f"..."`, `print()`,
`assert()`, `T?`, `[T]` generics), which draft 1 proposed and the usage
measurement rejected; draft 3.5's Tide-only spellings that rustc cannot
parse (bare `requires` and `ensures` clauses after a signature, a bare
`invariant` clause on a loop, `fn main(root: Root)` without an attribute,
`_?name` holes), which D18 replaced.

### D2. Memory: ownership with Rust's lifetimes, checked as origins

- Every value has one owner; moves are the default for non-`Copy` types.
  References `&T` and `&mut T` follow Rust's shared-xor-mutable rule.
- **Lifetimes are Rust 2024's.** Their syntax, elision, outlives bounds,
  implied bounds, object-lifetime defaults, and return-position `impl
  Trait` capture mean what they mean in Rust 2024. A lifetime names an
  **origin**: the set of loans a borrow may come from (Polonius's reading).
  - Elision: `fn get(&self, k: &str) -> Option<&V>` borrows from `self`;
    `fn next_word(&mut self) -> Option<&'a str>` in `impl<'a> Lexer<'a>`
    returns a borrow of the source, not of `self`.
  - Accepted as in Rust: lifetime parameters on structs, enums, impls, and
    functions; `'_` and `'static`; outlives bounds (`'a: 'b`, `T: 'a`,
    `+ 'a`); `+ use<..>`; and the relations that well-formed signature types
    imply. A returned `impl Trait` captures every generic parameter in
    scope, lifetimes included, unless `+ use<..>` lists fewer.
    `T: 'static` is written as in Rust; the formatter does not rewrite it.
  - Rejected (`TD0201`, `TD0403`, `TD0404`), because they lie beyond D4's
    power ceiling: lifetime parameters on traits, explicit `for<'a>`
    (higher-ranked lifetimes come from elision in `Fn` sugar and `fn`
    pointer types), and generic associated types.
- **Borrowing types declare lifetime parameters as in Rust**:
  `struct Lexer<'a> { src: &'a str, pos: usize }`. Each parameter names one
  origin; fields that use one name share it, and fields from different
  sources take different names
  (`struct Collector<'s, 'o> { src: &'s str, out: &'o mut Vec<&'s str> }`).
  Variance follows Rust: shared references are covariant, `&mut T` and
  cells are invariant in `T`.
- **`'static` and `Owned`.** A type that borrows nothing except `'static`
  data is `Owned` (Rust's `T: 'static`). Outside a body (signatures,
  fields, impl headers, associated types, consts and statics),
  `Box<dyn Trait>` is `Owned` and `&'a dyn Trait` is `dyn Trait + 'a`, by
  Rust's object-lifetime defaults; inside a body the object lifetime is
  inferred.
  `Tasks::scope` is typed like `std::thread::scope`:
  `impl FnOnce(Scope<'_, 'env, E>) -> Result<T, E>`, where `Scope<'s, 'env,
  E>` declares `'env: 's`, so the higher-ranked `'s` is bounded below by
  the `&Tasks` borrow. A closure that captures a borrow is a borrowing
  value. A task may borrow only data that outlives the scope it is spawned
  into, so tasks spawned into a long-lived scope capture `Owned` values.
- **Drop check** uses Rust's conservative rule: data borrowed by a value with
  a destructor must strictly outlive that value; standard containers carry an
  audited exemption.
- Borrow checking is per function at NLL precision plus, in v1, declared
  field borrows on methods, so disjoint-field calls compose. Draft 3.5 spelled
  them `fn advance(&mut self.{pos})`, which is not Rust; under D18 their
  spelling (a source transform or Rust's view types) is open until v1.
  Polonius-level precision is a later, compatible relaxation.
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
tie goes to Rust's spelling under D1. Draft 3.4 first kept a narrower
subset (Rust 2018 capture; no `'a: 'b` or `use<..>`); round 8
(`reviews/round8-consistency.md`) found that each deviation left programs
without a spelling, so D2 takes Rust 2024's rules whole. Also rejected:
draft 1's "one
implicit region per type" (unsound through `&mut` or too restrictive,
`reviews/round1-soundness.md` B1–B3); projections-only borrow return
(unfamiliar); shared-mutable-alias models (unproven at scale).

Accepted costs: about 3.5 times as many lifetime annotations as `from`
clauses on the trial's tasks (a declaration on every borrowing type and
impl, and a name at every use); Rust's `&'a mut T<'a>` trap stays writable,
as in Rust, though elision avoids it; Rust 2024's capture rule can
over-capture, fixed with `+ use<..>`; self-referential structs need
`unsafe`. That trial did not exercise outlives bounds, `use<..>`, or
`Scope<'s, 'env, E>`; the held-out round `trials/e1-dialect/` did, with no
borrow-related error (provisional: one writer, eleven tasks).

### D3. Types: small, decidable, explicit at boundaries

- Rust's scalar, string, slice, array, tuple, `Option`, `Result`, `Vec`,
  `Box`, and `String` types with their Rust meaning. No null.
- Nominal `struct` and `enum` with exhaustive `match`; `impl Trait` in
  argument and return position as in Rust, except that a returned
  `impl Trait` is `Send`, `Sync`, or `Pure` only if it names them
  (`impl Iterator + Send`), so a body edit cannot change what callers may
  assume; `Owned` follows what it borrows, as Rust's `'static` does.
- Every item signature is annotated; inference is bidirectional and local to
  a function body. No overloading: one function per name per scope and one
  inherent method per name per type; trait methods with the same name are
  disambiguated by trait path, as in Rust. No polymorphic recursion.
- **Integers trap on overflow, division by zero, and over-wide shifts in every
  profile**; the driver builds every profile with `-C overflow-checks=on`.
  v0 traps are precise. Deferred reporting, which lets the
  optimizer batch checks in vectorized loops, is a later tier that must first
  show the writes it delays are unobservable. A proof may delete a
  runtime check only if every fact it relies on is either proved inside the
  same build or still checked at run time; a `pub` precondition that a deleted
  check depends on stays checked at function entry.
- `as` is accepted only where it is lossless; lossy conversions use
  `TryFrom`, `truncate()`, or explicit float conversion methods. The prelude's
  sealed traits in `std::convert` supply `truncate` (same-signedness low bits),
  `round_from` (nearest, ties to even; overflow may yield infinity), and
  `try_from_float` (finite, integral, in-range only; `-0.0` becomes zero).
  Write `i64::try_from_float(f.trunc())?` to discard the fractional part
  explicitly (reference §4.2;
  [adopted mechanism](../research/25-casts-arena-allowlist.md#1-fx-yg53-cast-replacements)). Under the
  `unsafe` grant an address, a raw or function pointer's, counts as a
  `usize` (03 GLOBAL-4), so `p as u64` is lossless and `p as u32` is not:
  the grant admits unsafe code, not silent truncation, and a truncated
  address is the same bug as a truncated length.
- Floats are IEEE 754. Every operation that reads a float's sign or bits
  treats any NaN as one canonical NaN, and math functions use one libm on
  every target, so observable results match across targets while arithmetic
  stays native; targets whose native float arithmetic is not IEEE 754
  (x86 without SSE2) are unsupported, and the std refuses to build there.
  Rust leaves NaN bits and math precision unspecified, and its
  inherent float methods cannot be overridden, so the observers and math
  methods (`to_bits`, `is_sign_positive`, `is_sign_negative`, `copysign`,
  `signum`, `total_cmp`, `min`, `max`, `sin`, `exp`, and the rest) are outside
  the admission allowlist (D7). Tide's std provides functions in their place:
  `float::bits(x)`, `float::min(a, b)`, `float::max(a, b)`, `math::sin(x)`,
  `math::exp(x)`, each generic over `f32` and `f64`. `float::min` and
  `float::max` order -0.0 below +0.0. `from_bits` builds a float and observes
  nothing, so it stays admitted; so do `sqrt` and `mul_add`, which IEEE 754
  and Rust define as correctly rounded. Formatting, byte conversions, and
  foreign calls are audited against the same rule.

Rejected: signature inference, implicit conversions, overloading, custom
operators.

### D4. Abstraction: traits with a hard power ceiling

- Rust traits with methods, default methods, flat associated types and
  constants, and `impl Trait for Type`. Impls live in the module of the type
  or the trait.
- No overlapping impls, specialization, higher-kinded types, generic
  associated types, lifetime parameters on traits, rank-N types
  (higher-ranked lifetimes from elision in `Fn` sugar and `fn` pointer
  types are allowed), negative bounds, or user auto traits. Impl
  where-clauses must be structurally smaller than the impl head: a bound is
  accepted when its bounded type is a type parameter, a projection of one,
  contains no type parameter, or is a strict subterm of the impl's self type
  or trait arguments; otherwise it is `TD0406`.
- Generics are definition-checked: instantiation never produces a type error.
- Coherence never treats a missing auto or marker trait (`Send`, `Sync`,
  `Pure`) as making two impls disjoint.
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
  process. Stack overflow aborts the process, as in Rust; under the
  simulator it fails the seed, which the test driver reports from outside
  the process (03 PANIC-1). A child task's panic propagates to its scope,
  which panics in turn, as with Rust's `std::thread::scope`; no code ever
  observes data a
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
- Derives `Error`, `Encode`, and `Decode`, the contract attributes,
  `#[tide::main]`, and `#[test]` are proc macros owned by the toolchain and
  exported through Tide's std prelude; they are not user macros. Tide's
  `test` replaces Rust's built-in one, which rejects parameters, so a test
  with parameters is a property test (`04-contracts.md` §2.4).
  `#[error]`, `#[from]`, `#[source]`, and `#[codec]` are their derives'
  helper attributes.
- `const fn` evaluation is authority-free, deterministic, fuel-limited, and
  cached; it computes values, never declarations or types.
- Code generators are programs the developer runs explicitly; their output is
  ordinary source that is committed and reviewed. Nothing generates code
  during a build.

Rejected: token and procedural macros (opaque to agents and tools, uncached,
supply-chain holes); unrestricted compile-time execution; reflection-based
serialization as the default (5–7x slower in prior art).

#### Strict and internally tagged JSON (v1, fx-v4c9)

Decided 2026-11-13 (coordinator, fx-v4c9); not implemented. Research and
alternatives: [`research/29-strict-tagged-codecs.md`](../research/29-strict-tagged-codecs.md).
The v0 surface stays JSON-only `Encode`/`Decode`, `std::json::encode` and
`std::json::decode`, with `rename`, `rename_all`, `default`, and `skip`;
unknown fields are ignored and enums are externally tagged by default.
v1 extends the toolchain-owned `#[codec]` helper, without adding a derive,
user macro, reflection dependency, or serde lifetime API. Decoding builds
owned values; the input buffer is not borrowed by the result.

- **CODEC-1. Strict objects** (v1). `#[codec(deny_unknown_fields)]` on a
  named-field struct or an enum makes its derived `Decode` reject keys
  outside that struct's or selected variant's non-skipped fields, after
  renaming. For an internally tagged enum the tag key is also allowed.
  On an externally tagged enum this governs named-field variant payloads;
  other variant shapes keep their existing decoding rules. Strictness is
  local to that object: nested structs need their own annotation, while
  map entries remain data, not schema fields. A skipped payload field adds
  no allowed key. Missing `Option` fields and explicit defaults retain their
  existing behavior; a default never repairs an unknown key or a malformed
  present value. The key has no effect on `Encode`.
- **CODEC-2. Internal tags** (v1). `#[codec(tag = "type")]` on an enum
  selects a single JSON object with a string discriminator at the literal
  key `"type"`; any non-empty string key, such as `"kind"`, may be
  specified. Only
  unit and named-field variants are supported. A named-field variant
  encodes as `{"type":"Set","key":"a"}` and a unit variant as
  `{"type":"Clear"}`. Variant `rename` and enum `rename_all` determine
  the discriminator value as they determine external variant names today;
  they do not rename the tag key or the variant's fields. Field `rename`
  still applies. The tag may occur anywhere in the input object. A missing,
  repeated (even identical), non-string, or unknown tag is an error, with
  or without CODEC-1. A strict tagged unit accepts only its tag; a loose
  tagged object ignores extra keys as v0 objects do. Repeated known payload
  fields remain errors. Key comparisons use decoded JSON strings.
- Both derives validate these keys, including with only `Encode` or only
  `Decode` derived. Invalid placement, repeated or malformed helper keys,
  `deny_unknown_fields` on an enum with neither `tag` nor a named-field
  variant (where it would do nothing), `tag` on an enum containing any tuple
  or newtype variant, a tag key that collides with a non-skipped payload
  field after renaming, and duplicate wire field or variant names are
  compile errors. Without `tag`, enum representation is unchanged; adding it
  intentionally changes wire bytes. Decode failures return `json::Error`,
  preserve whole-input consumption and the existing nesting/codec-call
  limits, and never select another variant as a fallback. Decoding cannot
  construct capability handles (ROOT-4).

01 D6 owns the `CODEC` rule prefix. Deferred: tagged newtype/map payloads,
`flatten`, adjacent and untagged representations, catch-all variants, and a
public codec builder or format plug-in API. These two keys require no new
public trait methods or runtime dependency. The implementation must handle
tag order without losing duplicates, number spelling, error positions, or
the decoder's limits; the research note proposes private input-span
scanning.

### D7. Authority: capabilities as values, no ambient authority

- Every operation that reads or changes the outside world, or that can
  observe nondeterminism, is a method on a capability value: `Fs`, `Net`,
  `Clock`, `Rand`, `Env`, `Stdin`, `Proc`, `Tasks`, and `Accel` for Bend
  kernels (D16). The standard
  library has no free functions that do I/O, read the time, draw randomness,
  read the environment, or spawn.
- `main`, written `#[tide::main] fn main(root: Root)`, may take `root: Root`,
  the only source of capabilities. `Root` can be named only in `main`; a
  simulated root (`Root::sim(seed)`) only in tests and in `tide sim`. No code
  calls `main`, so a test cannot mint a real `Root` through it.
- **Admission allowlist.** Under D18 `core` and `alloc` are always present,
  and std's inherent methods stay callable on the types Tide's std
  re-exports. Tide's checker therefore admits only listed `core` and `alloc`
  items and listed inherent methods, keyed on crate identity, and rejects the
  rest (`TD0520`): safe hardware intrinsics such as `__cpuid`; raw pointer
  types without an `unsafe` grant, since their `Debug`, `Hash`, and `Ord`
  impls observe addresses, and every address observer (`addr`, `{:p}`);
  atomics not created through `Tasks`; float observers and math methods
  (D3); I/O methods on re-exported std types (`Path::exists`, `metadata`);
  `io::Error::last_os_error`; compile-time reads of files and the build
  environment (`include_str!`, `env!`, `option_env!`); and `asm!`, `global_asm!`,
  and `naked_asm!` without an `unsafe` grant. The allowlist is reviewed with
  every rustc pin.
- Files support append and explicit durability (`File::sync`,
  `Fs::sync_dir`); durability is part of the simulator's filesystem model
  (D15).
- **Network** (v0; decided 2026-11-18 under owner decision A1, bead
  `fx-ff13`, research/42). `net.get(&url, deadline)` is one HTTP/1.1 GET,
  TLS for `https` through rustls with compiled-in roots, a fixed request
  and no redirect, proxy or session state, real and simulated
  (`sim.world.http`). `net.only(["host:port"])` grants exact destinations
  for outbound peer operations (`get`, `connect`, planned peer
  operations), checked before their lookup (03 NET-HOST). It does not
  limit `bind` or accepted connections: bind can resolve arbitrary names
  even if it fails, and unlisted inbound peers can exchange data.
  `only([])` alone is not network silence; delegate `only([..]).no_bind()`
  for an outbound-only destination grant. It bounds destinations, not
  data: path and query reach the allowed host; an allowed relay host is
  full network authority. `net.resolve(host, deadline)` (admitted
  2026-11-19 under A1 (2026-11-18), bead `fx-6d50`, research/48; planned
  until built) uses the shared resolver. An `only` entry on any port
  permits resolution of its host. A name grant does not grant its
  addresses. See 03 §1.1.
- Capabilities are ordinary parameters passed by reference. Attenuation is a
  method returning a narrower capability: `root.fs.dir("/var/lib/app",
  Mode::ReadWrite)`, `root.net.only(["db:5432"])`.
- **Processes.** A child process is not capability-checked, so `Proc` is
  ambient authority for the child. `proc.only(["git"])` bounds which
  program starts, by exact spelling, with bare names looked up by std in
  the start-time `PATH` and loader and C-library variables refused; it
  does not bound what the program does, and an allowed shell, interpreter
  or script, or a program steered by its arguments, variables or the files
  it reads, is full authority (03 §1.1 PROC-ONLY; research/40, bead
  `fx-ayh2.1`).
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
- **Authority-free types.** `Pure` is a marker trait the checker owns: a type
  is `Pure` if no capability, synchronization handle, or foreign handle is
  reachable from it. Tide's std declares it with a blanket impl, so rustc
  accepts every bound and coherence never treats `Pure` as making impls
  disjoint; the checker decides it structurally wherever a type meets a
  `Pure`-implying obligation or is coerced to a `Pure` trait object. `Arc`
  exposes no reference-count readers, and
  `Arc` and `sync::Weak` are synchronization handles, because drop timing and
  conditional cloning can reveal scheduling; authority-free code shares data
  by reference inside scopes instead. Classification is structural over every
  reachable type, including closure captures and generic arguments. A function
  is authority-free if all its parameter and capture types are `Pure`. An
  authority-free function cannot observe nondeterminism: its results depend
  only on its arguments.
- **`Pure` trait objects** use a companion supertrait, because
  `dyn Trait + Pure` is not Rust (E0225): `trait PureValue: Value + Pure {}`
  with a blanket impl, so `&dyn PureValue` upcasts to `&dyn Value`. A type
  the structural rule rejects that is authority-free in fact, such as a
  foreign handle, is marked `#[trusted_pure]` with a reason in a package
  granted `unsafe`, which counts as a trust hole; `unsafe impl Pure` is not
  used. Rust code that calls a Tide crate (RUST-6, v1) is unchecked, so
  closures and impls it passes in are trust holes,
  as FFI callbacks are. v1's `ThreadBound` trait objects take the same
  companion form, since `dyn Trait + ThreadBound` is E0225 as well.
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
- **Kernel enforcement.** The type system alone cannot bound what admitted
  code reaches: rustc has open soundness bugs that safe code can exploit to
  forge a capability (probe `fx-6xi2`). Test and simulation
  processes therefore get their kernel sandbox in v0 (RUN-3, D13; bead
  `fx-ip4x`), since agent-written code mostly runs as tests on developer
  machines. Deployed binaries get theirs in v1: when `main` attenuates its
  `Root`, the runtime installs the matching operating-system sandbox
  (Landlock and seccomp on Linux) and drops what was not kept, so a bug or a
  malicious dependency that escapes the type system still meets the kernel.
  A WASI target maps capabilities onto component imports. Until RUN-3
  lands, public claims say that safety against a malicious dependency rests
  on rustc's soundness, which has open `I-unsound` issues; the checker closes
  only the `fn`-pointer route of rust-lang/rust#25860 (02 CLOSURE-3) and,
  since fx-qlek.5, the trait-object route of rust-lang/rust#57893 (`TD0402`).
  Decided 2026-11-15 (coordinator, owner-delegated; bead `fx-iwjq`).

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
  destructors; operations that hand a value over return it inside the error.
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
  suspending calls allocate their frames. The evented runtime is v1, and its
  lowering under D18 is open (D18, reversal conditions).
- Foreign callbacks cannot suspend. A task holding a `ThreadBound` value is
  pinned to its thread while it holds it.

Rejected: Rust-style async; hidden effect-based suspension; unstructured
spawn; stackful tasks in compiled code (thread-local caching across migration
is unsound with foreign frames, and per-task stacks cap task counts,
`reviews/round1-performance.md` B3). The simulator may use stackful
execution internally; it runs tasks on baton-passed threads.

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

- Clauses keep Dafny and Verus names, spelled as toolchain attributes and
  `!` forms, as Prusti and Creusot spell them (D18): `#[requires(..)]` and
  `#[ensures(..)]` on functions; `invariant!(..)` statements at the top of a
  loop body in a function marked `#[invariants]` or carrying any contract
  attribute; and in v1 `#[decreases(..)]` and `#[diverges]` on functions,
  `decreases!(..)` in loops, and `#[invariant(..)]` on types. The
  function-level attribute rewrites each loop, so an invariant is checked
  where control reaches the loop head: before each evaluation of a `while`
  or `while let` condition, and before each step of a `for` loop. Debug
  builds check a `for` over `a..b` at entry (also when the loop runs zero
  times) and exit, and every other loop at every head, since its exit is
  known only once its condition or iterator ends it; `tide test` checks
  every head. A closure's clauses (v1) are `requires!(..)` and `ensures!(..)`
  statements at the top of its block body, rewritten the same way, because
  attributes on closure expressions are unstable Rust. The `!` clause forms
  are not exported macros: the enclosing function attribute recognizes and
  removes them, so they may share names with the attributes, and an
  unrewritten one is a checker error.
  `result` names the
  return value; `old(x)` the entry value of a `&mut` parameter; `final(x)` the
  value a returned or stored `&mut` holds when its borrow ends. From v1, a
  bare `*x` of a `&mut` parameter in `ensures` is an error: write `old(*x)`
  or `final(*x)`.
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
  `*x` of a `&mut` parameter means the caller's referent at return, even
  where the body reassigns the parameter (`cur = &mut n.next`), as v1's
  `final(*x)` does, and `old(*x)` its value at entry. Where `final(*x)`
  would be a prophecy (`x`'s origin appears in another parameter's type,
  the result type, or a bound other than on the right of an outlives
  bound, 04 §4.1), naming `x` outside `old(..)` is `TD0803`. `old(e)`
  evaluates `e` at entry and snapshots the value it
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
- **Uses**, chosen per package in the manifest and overridable per item
  with `#[contracts(..)]` (custom inner attributes such as `#![contracts]`
  are unstable Rust):
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
- Trait methods may carry contracts (v1); impls may weaken preconditions and
  strengthen postconditions, never the reverse.
- **Spec quality.** The toolchain checks that preconditions are satisfiable,
  postconditions are not trivially true, and assumptions are not
  contradictory, and runs mutation testing of specs in CI.
- **Capabilities have no spec model in v1.** A function that uses a
  capability may state postconditions about its results that hold for every
  possible response of the world.
- Specs never affect typing or code generation, except that proved facts may
  delete runtime checks; how a build deletes one check under D18, where
  every profile sets `-C overflow-checks=on`, is open until v1. Release builds
  erase specs unless asked to keep them.
- Solver stability: executable function bodies are opaque to callers by
  default (`spec fn` bodies are transparent); quantifiers need
  triggers; nonlinear and bit-vector reasoning stays in marked blocks; the
  solver version is pinned; CI checks proof stability under seed changes.
- Interactive proofs are out of v1; the escape hatch is `#[trusted]` with a
  reason.
- v1's proof forms (`spec fn`, `proof fn`, `law fn`, `ghost let`, proof
  blocks, views, spec integer types) are still written in draft 3.5's
  grammar in `design/04-contracts.md`. Under D18 they too must become
  attributes, toolchain `!` forms, or library types; their spelling is
  decided before v1 and E1 does not cover it.

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
  reviewers see what the change does, not only what it may do. It runs
  only for a branch that 03's RUN-5 calls covered, on a host whose test
  sandbox is full, unless the operator asks.
- Contract hashes cover the bodies of every function a clause calls,
  transitively, so editing a predicate a contract relies on is a contract
  change.
- **What needs approval**: weakening or removing a contract, widening
  authority, adding a trust hole (including `diverges`), downgrading a
  package or item from `prove` to `test` or `assert`, changing a
  `#[protected]` test, and removing or changing a seal entry.
- **Approval lives outside the agent's write scope.** Approvals are records
  signed by a human-held key and verified by CI. A hash written in the same
  change is never approval, and proof caches are rebuilt in CI rather than
  trusted. Hashes cover everything a spec's meaning depends on, including the
  bodies of `spec fn`s it calls.
- **Sealed delegation boundaries** (v0). An orchestrator fixes an interface
  and hands its implementation to delegates (bead `fx-1am`; research/13, needs
  2 and 3). `tide seal` writes `tide.seal`, a lock of stable IDs (D14) and
  hashes: a function's signature, declared authority, contract use, and
  spec-closure hash; whole items, such as tests, types, or `main`; and the
  manifest's grants, policies, and `[contracts]` table. The admission driver
  (D18) rejects a crate whose sealed item differs from its entry before rustc
  writes any artifact for it; `tide review` and the CI gate check a candidate
  against the base's seal, so a delegate that edits `tide.seal` gains nothing
  once its work merges through them. Whoever writes the base changes its seal;
  on a protected branch that takes an approval, the same records and keys as
  above. Rules: `04-contracts.md` §11.1. Rejected: a `#[sealed]` attribute,
  which cannot seal the manifest and spreads one delegation's boundary across
  the source files delegates edit, where one lock file states it whole;
  changes to sealed items gated by approval rather than rejected, which lets a
  delegate's loop run on against a changed interface that parallel delegates
  still assume; refinement-aware seals that admit a stronger contract, which
  need v1's solver and buy little; a frozen count of trust holes, a second
  gate on what the grants already bound and D11 already gates; and seals
  signed with orchestrator-agent keys, which would add an approver class to
  the model above.

### D12. Compilation and backends

- A module is a `.rs` file or an inline `mod` block, declared with `mod` as
  in Rust; the module tree follows the directory tree. Imports are explicit;
  cycles and unused imports are errors.
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
- **Checker architecture: A, then B** (D18). (A) A `rustc_private` driver on
  the pinned nightly that builds the code is the authoritative admission
  gate and the oracle. (B) Tide's own incremental front end, which produces
  the interface artifacts above, serves the edit-loop targets below and is
  tested against A; where they disagree, A decides.
- **Backend**: a pinned, unmodified nightly rustc with LLVM, in every
  profile: debug builds for development, tests, and the simulator, release
  builds for production. The driver owns every rustc invocation and keeps a
  content-addressed artifact cache instead of Cargo's build directory.
  Generics are monomorphized, as rustc does. An interpreter is built only if
  `tide eval`, E4's edit-to-test target, or v1's spec evaluation
  (`by(compute)`, spec tests) needs it; counterexamples replay as debug-build
  runs.
- **Targets** for a 100,000-line project: edit to diagnostics around 30 ms
  typical, under 100 ms at the 95th percentile; edit to running tests under
  1 s; clean and incremental builds and development-build runtime set by E4
  against rustc's debug profile (drafts up to 3.5 targeted a few seconds and
  3x of release with their own backends); release runtime within 10–20% of
  equivalent Rust.
- The compiler is written in Rust and not self-hosted before 1.0.

- **The origin cross-check is the build.** Because every Tide program is
  Rust, rustc's borrow checker runs on every build, so a program Tide's
  checker wrongly accepts is still rejected; the conformance corpus compares
  the two checkers' verdicts.

Rejected: witness tables by default (6–24x slower); C++ interop, MLIR, and
GPU before 1.0. History: drafts up to 3.5 planned a reference interpreter, a
Cranelift-class development backend, and a C-emitting release backend from
one mid-level IR, and rejected Rust emission as the release backend (53 s
clean builds, coupling to rustc's borrow precision, no route to stackless
lowering of Tide tasks); D18 reversed that.

### D13. Packages and supply chain

- Tide packages have no build scripts, install hooks, or code that runs at
  fetch or build time, beyond the toolchain's own proc macros. The one
  exception is an imported Rust crate's build
  script, which runs sandboxed and counts as a trust hole.
- Lockfiles are on by default with content hashes; publishing requires
  trusted publishing; package names are namespaced; the client rejects
  dependencies that do not exist rather than guessing near names.
- Each package's manifest lists escape-hatch grants per dependency (`unsafe`,
  `ffi`), and the lockfile records each package's public authority and trust
  holes. Widening either needs approval (D11).
- **Rust crates** with synchronous APIs link directly (D18), behind Tide
  wrappers that take the capabilities they may use (RUST-2). Each imported
  crate declares its authority in the manifest; the wrapper requires those
  capabilities; a scan of the linked symbols checks the declaration; its
  build script, if any, runs sandboxed without network and with recorded
  outputs.
- **Rust consumers** (v1) depend on a Tide crate as on any Rust crate; their
  own manifest and `std` are unchanged. The Tide crate's exported `Cargo.toml`
  (D18) declares `std = { package = "tide_std" }`, so Cargo builds that crate,
  and only it, against Tide's std. Types Tide's std re-exports from Rust's are
  shared, and Tide-owned types (`HashMap`, `Arc`, channel ends, capabilities)
  need adapters at the boundary.
- **Unreviewed branches** (v0; bead `fx-arx`; research/13, X3, X8, E6).
  `tide check`, `tide test`, and `tide sim` on a branch no one has reviewed
  run its code only in rustc's compile-time evaluation and in test and
  simulation processes whose capabilities are all simulated, so it reaches
  nothing outside the run except through listed holes (03 §2.6, RUN-1). The
  build reads only the branch's tree, under an environment the toolchain
  sets and in a build sandbox that also bounds assembler file reads; the
  branch cannot choose the toolchain, linker, or flags (RUN-2).
  Each test or simulation process starts with that environment, no terminal,
  and a fixed OS sandbox: Landlock and seccomp allowlists on Linux, which
  admit file opens only of its binary, libraries, and output directory and
  refuse new processes, sockets, System V IPC, keyrings, and tracing
  (RUN-3). The holes are `unsafe` grants (unsafe attributes and
  `global_asm!` included), `ffi` grants, imported Rust crates and their
  sandboxed build scripts, the Bend boundary, toolchain and kernel bugs, and
  resource use and side channels (RUN-4). Before anything runs, `tide review
  <base>` reports the host's sandbox level and `run: covered` when, relative
  to a reviewed base, the branch adds no hole or grant and changes no
  package with an `unsafe` or `ffi` grant, no dependency of one, and no
  imported Rust crate or Bend dependency (RUN-5). On a host whose sandbox is
  full, such a branch can be tested inside the agent's session without a VM
  per run, though a toolchain bug or an unsound reviewed abstraction can
  still forge its results (RUN-6). This complements, and does not replace,
  one microVM or OS sandbox per agent session, which agents need because
  they run arbitrary shell commands, and inside which the build itself runs;
  kernel enforcement (D7, v1) gives the deployed process its own sandbox. A
  microVM, for its part, does not protect the credentials inside it, gives
  reviewers no evidence, and is gone in production. Rejected: a VM per test
  run, which adds a boot and a queue to every validation (X3) where RUN-1
  needs neither for covered branches; and a guarantee resting on the type
  system alone, since rustc has open soundness bugs that safe code can
  exploit.

### D14. Toolchain contract for agents

- `tide check` returns one verdict and NDJSON diagnostics with stable codes,
  exact spans, related spans, and fix plans marked `safe` or `suggest`.
  Diagnostics for Rust habits (D1) carry safe fixes.
- `tide test` and `tide sim` return structured results: each failure has its
  seed, a replay command, and a capability event trace.
- **Typed holes**: `todo!()` or `hole!(name)` in expression position checks
  the rest of the program and reports the expected type, in-scope bindings, and
  candidate functions.
- `tide eval` runs an expression against the current package, through a
  scratch crate built by rustc or the interpreter if one is built (D12).
  `tide verify` runs proofs and returns NDJSON results with proof status per
  item and counterexamples for failures. Coverage and spec coverage are
  reported.
- `#[protected]` tests are part of the trust surface; agents cannot change
  them without approval (D11).
- Every response carries a snapshot hash; every write request carries its base
  hash and applies atomically or not at all.
- `tide doc` serves documentation matched to the installed compiler: a card
  under 8 KB for an agent's instructions file, `tide explain <code>` with fix
  examples, and the delta-from-Rust reference. The core reference budget is
  25,000 tokens, to be counted in CI; its companion and the standard-library
  reference are separate and served on demand.
- `tide fix` applies safe fixes and owns migrations between language
  versions.
- Source text is canonical; the formatter is deterministic and has no options.
  Its style is rustfmt's default style (four-space indentation, 100-column
  lines, opening braces on the same line), so formatting a Rust-habit draft
  changes almost nothing. Definitions are content-hashed; symbols have stable
  IDs. In v0 an item's ID is its path and kind (04 §11), which serves every v0
  consumer: a sealed item's path is fixed anyway, and content hashes abstract
  referenced names except for 04 §11's residual-tie ordering. Rejected for v0:
  IDs written into source, which every edit must keep and copying an item
  duplicates, and which a delegate can move anyway.

### D15. Deterministic simulation as a first-class mode

- `Root::sim(seed)` provides deterministic clocks, network, filesystem,
  randomness, environment, task scheduling, and fault injection (drops,
  delays, partitions, disk errors, crashes and restarts of node scopes).
  Virtual time advances only when every task is blocked; a global deadlock is
  reported with the blocked-task graph. On a simulated crash, writes not made
  durable by `sync` or `sync_dir` may be lost or torn, as the seed decides;
  each unsynced directory-entry change, a `rename` included, survives whole
  or not at all, so atomic replace works as on a journaling file system.
- `tide sim <binary>` runs a program's `main` under many seeds. A failure
  prints seed and commit and replays bit for bit, dependencies included.
- Every simulated run writes a replay manifest: seed, the content hashes of
  the source and every dependency, a semantic build identity (compiler
  version, target, and every option that affects semantics), the simulated
  root's configuration (arguments, environment, files), and all injected
  inputs. Replay refuses to run against a mismatched identity. The standard
  library ships
  one math library for every target, `float::bits` canonicalizes NaN, and raw
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
- **Processes** (v0; decided 2026-11-18 under owner decision A1, bead
  `fx-ayh2.1`, research/40): `Proc::command` runs a program on the real
  backend, and under simulation a declared `sim.world.proc` model stands in
  for it: spawn, exit status, per-`Child` wait ownership and piped I/O (03
  §9.9's lifecycle subset). Replay covers only declared models and process
  modes; missing coverage is reported and never silently starts a host
  process.
- Later: coalesced signals, raw PTY I/O and size, parent-only process death
  and process faults have explicit external-child models (03 §9.9). Process
  death and whole-node crash have distinct lifetime and storage effects.

### D16. Bend tandem

- Tide calls Bend 2 for pure, proved, massively parallel kernels; Bend may
  call Tide libraries through its foreign effects once Tide exports C ABIs.
- **Stage 0, planned for Tide (Later tier); feasible with current Bend**: a
  `bend` dependency kind with an exact version pin and cached builds; a
  generated shim lets Bend's own checker verify the declared kernel signature;
  kernels run as a supervised child process with a generated text protocol. A
  Bend crash maps to `Err`.
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

### D18. A checked dialect of Rust (v0 direction)

Owner decisions, 2026-10-19: the direction (bead `fx-enq`; study
`research/15-rust-dialect.md`; review `reviews/dialect-codex.md`), and the
gate passed with checker architecture A then B (bead `fx-7qu`; spike
`spikes/d18/`, results in research/15).

- **Every Tide program is a valid Rust 2024 crate** that a pinned, unmodified
  nightly rustc compiles, with Tide's meaning, against Tide's std. The driver
  passes
  that std as `std` (`--extern std=`), so rustc's std injection loads it and
  the real std's paths cannot be named.
- **Tide's checker gates every crate before rustc writes any artifact for
  it.** It owns the subset, authority, `Pure`, contracts, TD diagnostics, and
  the trust diff. It also enforces an **admission allowlist** over `core`,
  `alloc`, and std's inherent methods, keyed on crate identity. That closes
  what the std swap leaves open: safe hardware queries, raw pointer types and
  every address observer, atomics created without `Tasks`, float methods, I/O
  methods on re-exported std types such as `Path::exists`, and
  `io::Error::last_os_error`.
- **Checker architecture: A, then B.** (A) A `rustc_private` driver on the
  same pinned nightly that builds the code is the authoritative admission gate
  and the oracle; it checks rustc's own resolution, so no separate equivalence
  argument is needed. It passes `-Zallow-features=`, so user code cannot
  enable unstable features, and `-Zpolonius=no`, so borrows get NLL's verdicts
  (02 ORIGIN-6). It checks resolved uses before rustc's analysis; after it, a
  backstop checks the instances that can run (03 ADMIT-1), and the seal check
  compares the crate with its `tide.seal` (04 SEAL-3). (B) Tide's own
  incremental front end serves D12's edit loop and is tested against A; where
  they disagree, A decides.
- **The driver owns every rustc invocation.** It adds no user-controlled
  `--extern`, `-L`, `--sysroot`, `--cfg`, or semantic `-C` flags, runs no
  build scripts or proc macros for Tide crates except the toolchain's
  (imported Rust crates' build scripts run sandboxed, D13), and keeps its own
  content-addressed cache instead of Cargo's build directory. It is the one
  reader of the package's `tide.toml`: it snapshots the manifest with the
  sources, so its bytes are part of the cache key, rejects malformed input and
  unknown keys, and gives every consumer the same parsed content, the grant
  check and the seal's manifest entry (04 §11.1) among them. The package's
  grants are the manifest's, and each must be one the trusted build caller
  passed, so candidate source never grants itself authority; without a
  manifest, the caller's grants apply. A generated `Cargo.toml` is an export
  for Rust consumers (v1).
- **rustc and LLVM are the release backend**, replacing D12's C backend and
  its Cranelift-class backend; open question 7 lapses. An interpreter is
  built only on D12's conditions.
- **The gate passed** (2026-10-19): the spike met the seven criteria in
  `research/15-rust-dialect.md` at prototype scale. Its gaps (a panic
  inside a clause, sampled-clause budgets, `a..=b` next-index binding,
  entry values of by-value parameters in `ensures`, `loop {}` rewriting,
  a `&mut` parameter the body reassigns, which its closure lowering
  rejects with E0521, and snapshot choices that need the checker's `Pure`
  decision) are implementation work. Its `tide-admit` reports every failed
  `Pure` obligation as `TD0510`; the design reports one under the code of
  the rule that needs `Pure` (`TD0502`, `TD0518`, `TD0603`, `TD0606`,
  `TD0610`, `TD0611`, `TD0612`), and otherwise as `TD0517` (03 §3.1). A
  broken `#[pure]` is `TD0511`, and `TD0510` is the least-authority lint (03
  §4.2). Its float fixture still rejects `sqrt`, which D3 admits.
- **Changes applied to the other decisions** (draft 3.7, bead `fx-pcb`):
  - D1: contract, loop-invariant, and `main` spellings; declared modules;
    `.rs` files.
  - D2: no `T: Owned` rewrite.
  - D3: float observers and math through Tide functions.
  - D5: stack overflow aborts.
  - D7: the allowlist, `#[trusted_pure]` instead of `unsafe impl Pure`, and
    companion traits instead of `dyn Trait + Pure`.
  - D10: attribute contracts.
  - D12: the backends, and the origin cross-check becoming the build itself.
  - D13: direct Rust import.
  - D14: the typed-hole spelling.
  - The tier table.
- **Reversal conditions:**
  - E1 rejects the attribute spellings;
  - v1's evented runtime cannot be built without exposing coloring to users
    (`research/15-rust-dialect.md` lists its requirements).

Rejected for now: keeping the new-language path as the v0 direction. Its
first demo is not shown to be cheaper, and after the demo it must build an
interpreter, two native backends, and C-ABI interop that rustc and direct
linking provide.

## The Feel

```rust
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

/// Checked at run time under `assert`, fuzzed under `test`, proved under `prove` (v1).
#[requires(xs.len() > 0)]
#[ensures(result < xs.len())]
#[ensures(forall |i: usize| i < xs.len() ==> xs[result] >= xs[i])]
pub fn max_index(xs: &[i64]) -> usize {
    let mut best = 0;
    for i in 1..xs.len() {
        invariant!(best < i);
        invariant!(forall |j: usize| j < i ==> xs[best] >= xs[j]);
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

#[tide::main]
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
3. The simulator model for foreign effects and for imported Rust crates.
4. Whether authority-free public functions should be declarable (`#[pure]`)
   so that adding authority is a visible breaking change.
5. Whether `tide eval`, E4's edit-to-test target, or v1's spec evaluation
   needs an interpreter (D12, D18).
6. How spec coverage and exercised authority are presented so the trust diff
   stays small enough to read.
7. (Lapsed under D18.) How the C-emitting release backend (D12,
   v1) implements unwinding for panics, which must drop values (D5): a flag
   check after every call, compiler-specific extensions, or abort in release builds. If none is
   acceptable, this is the main reason to switch to a direct LLVM backend.
