# First Principles

Status: written before the research reports in `research/` and the reviews
in `reviews/`. Later design documents cite this one for the problem statement
and the targets. The hypotheses have verdicts, given in the final section and
based on that evidence and the decisions in `design/01-design.md` (draft 3).

## The Shift

Software cost has three parts:

1. producing candidate code;
2. establishing trust that the code does what it should and nothing else;
3. changing it later without breaking that trust.

Coding agents push the first cost toward zero. They do not remove the other
two; they increase them, because more code arrives faster than people can read
it. The bottleneck moves from writing to trusting and changing.

A language designed for this era should therefore minimize the cost of trust
and the cost of safe change, while keeping the performance and safety that make
Rust worth its learning curve. Writing convenience matters only where it
affects those costs.

## Three Readers

Every program has three readers, and the language must serve all of them.

| Reader | Needs | Fails when |
| --- | --- | --- |
| Machine | decidable, fast checking; code that compiles to fast native output | checking is slow, global, or undecidable; abstractions have hidden runtime cost |
| Agent | a small, regular language it can learn from a document; fast, precise, actionable feedback; local reasoning | the language is large, has many ways to do one thing, hides control flow, or produces cascading or vague errors |
| Human | to review intent, authority, and invariants without reading every line | the only way to know what code does is to read all of it |

The human is the scarce reader. The design goal is to move human attention
from implementations to interfaces: types, authority, and contracts.

## The Trust Ladder

Trust in a piece of code comes in levels. Each level should be cheap for an
agent to reach and visible to tools and reviewers.

| Level | Established by | Rules out |
| --- | --- | --- |
| 0 | parsing and formatting | malformed source, style drift |
| 1 | type and ownership checking | memory errors, data races, null, unhandled cases, undefined behavior in safe code |
| 2 | authority checking | code doing anything its signature does not grant: I/O, network, clocks, randomness, unsafe, FFI |
| 3 | deterministic tests and properties | regressions on exercised behavior, nondeterministic flakiness |
| 4 | proven contracts | violations of stated pre- and postconditions and invariants on all inputs |
| 5 | full functional proofs | any deviation from a complete specification |

Mainstream languages stop at level 1, and most stop short of it. Rust reaches
level 1 for memory and data races. The opportunity is to make levels 2 to 4
ordinary, so that "this function compiles" means much more than it does today,
and so that a reviewer can read a signature and know the envelope of what the
body can do.

## The Agent Loop

An agent works in a loop: read context, propose an edit, check, read feedback,
repeat. The language controls four parameters of that loop.

1. **Latency.** Check time per edit multiplies every iteration. An agent that
   gets feedback in 100 ms runs an order of magnitude more iterations per hour
   than one that waits seconds. Fast incremental checking is a core semantic
   requirement, not a tooling afterthought; it constrains the type system,
   generics, and metaprogramming.
2. **Signal quality.** Each diagnostic should identify one cause, point at the
   exact place, and when possible carry a machine-applicable fix. Cascading
   errors and errors at a distance waste iterations.
3. **Locality.** Understanding a function should require only its body and the
   signatures it uses. Global inference, implicit conversions, overloading,
   specialization, and macro expansion at a distance all break locality.
4. **Convergence.** When a language offers one obvious way to write something,
   independent attempts converge, diffs stay small, and review gets easier. Low
   entropy is an agent-performance property.

## The Learnability Constraint

A new language has no training data. Models write Python, TypeScript, and Rust
well because they have read enormous amounts of them. Two consequences follow.

- **The language must be learnable in context.** Its complete reference should
  fit in an agent's context window with room to spare. That is a hard size
  budget: the language is closer to Go, Lua, or Gleam in size than to Rust or
  C++.
- **Familiarity is borrowed capital.** Surface syntax should read as obvious to
  a Rust or TypeScript programmer, so that model priors transfer. Novel syntax
  must buy a large, specific benefit. Novel semantics can hide behind familiar
  syntax when the familiar reading is still correct.

## Hard Constraints

The design must satisfy all of these. A proposal that violates one is rejected
or must argue for an explicit exception.

1. Memory safety and data-race freedom in safe code, without a tracing garbage
   collector or pauses.
2. No undefined behavior in safe code. Integer overflow, out-of-bounds access,
   and division by zero have defined, checked behavior by default.
3. Predictable performance: allocation, copying, and dynamic dispatch are
   visible or explicitly chosen; release builds compete with Rust and C.
4. Decidable, fast type checking with separate compilation through explicit
   signatures.
5. Every side effect and every use of ambient authority is visible in
   signatures.
6. Specifications are part of the language, checked by the toolchain, and
   erased from release builds unless requested.
7. The full language reference fits in a budget an agent can load (target:
   under 25,000 tokens for the core reference).
8. One canonical formatting; one obvious way for each common task.
9. A stable, machine-readable compiler interface: diagnostics, fixes, semantic
   queries, and checked edits.
10. Interoperability with C, and a credible path to existing Rust libraries.

## Measurable Targets

"Perfect" needs numbers. These are initial targets; the research may move them.

| Property | Target |
| --- | --- |
| Core reference size | under 25,000 tokens |
| In-context learnability | a fresh agent given only the reference writes a fixed benchmark of small programs with a high first-check pass rate and converges on all of them within a few check iterations |
| Incremental check | under 100 ms after a one-function edit in a 100,000-line project |
| Cold check | under 2 s for 100,000 lines |
| Debug build | a few seconds for 100,000 lines, linking included |
| Release performance | within roughly 10–20% of equivalent Rust on a standard benchmark set |
| Authority | every function's possible effects derivable from its signature |
| Verification | contracts on core library functions proven by the toolchain, with no proof required for ordinary code |

Learnability so far: the first E1 pilot, ten programs by one agent that saw
only `design/reference.md`, graded without a compiler, estimated a 6/10
first-check pass rate against an estimated 9/10 for the same writer in Rust,
with no leaked Rust habits in 1,614 lines. Every first-check failure traced to
the reference lagging the normative chapters or to one design gap
(`trials/e1/grading.md`).

## Tensions

These trade-offs define the design space. Each needs an explicit decision.

1. **Familiarity versus novelty.** Better semantics, such as value semantics,
   capabilities, and contracts, without an unfamiliar look.
2. **Expressiveness versus smallness.** Rust's arbitrary references and
   lifetimes in data structures versus simpler models that make graphs and
   self-reference require arenas, indices, or reference counting.
3. **Compile speed versus runtime speed.** Monomorphization makes fast code and
   slow builds; dictionary passing does the reverse.
4. **Verification power versus usability.** Dependent types versus SMT-backed
   contracts versus tests.
5. **Effect precision versus annotation burden.** Effect-row polymorphism is
   precise and heavy; coarse capabilities are light and less precise.
6. **Text versus graph.** Source text is universal and reviewable; a semantic
   graph gives agents stable identities and checked edits.
7. **New ecosystem versus interop.** A new language has no libraries on day
   one.

## Hypotheses To Test

The research reports should confirm, refine, or refute each of these.

- **H1.** Mutable value semantics with second-class references removes nearly
  all lifetime annotations while keeping Rust-level safety and performance for
  most backend, CLI, and data-processing code.
- **H2.** Capabilities passed as ordinary values give reviewable authority
  without the cost of effect-row polymorphism.
- **H3.** SMT-backed contracts in the style of Dafny and Verus give the best
  agent proof success today; interactive proofs serve as an escape hatch.
- **H4.** Generics compiled with dictionaries in development builds and
  specialized in release builds keep checking and debug builds fast without
  giving up release performance.
- **H5.** A Rust-familiar surface, a reference that fits in context,
  compiler-served documentation, and excellent diagnostics overcome much of the
  low-resource penalty.
- **H6.** Bend's most inheritable idea is deterministic parallelism of pure
  value code; HVM is not yet suitable as the main backend for a systems
  language.
- **H7.** Typed holes and a structured query-and-edit interface improve agent
  success more than any single syntax feature.

## Verdicts

Each verdict rests on the reports in `research/`, the reviews in `reviews/`,
and the decisions in `design/01-design.md` (draft 3). "Open" means the
validation program (D0) has to settle it.

- **H1: refined, into origins.** Draft 1's version (second-class references
  with views, one implicit region per type) was unsound or unusable as
  written and stricter than Rust's elision on common accessors, so Tide
  keeps Rust's first-class `&` and `&mut`, its elision rules, and its
  lifetime spelling, with lifetimes checked as origins (D2). Draft 3's
  `from` clauses, which named a parameter or place, respelled lifetimes
  rather than removing them (`reviews/round2-simplicity.md`); on a held-out
  trial they tied Rust's spelling on correctness with a third of the
  annotations, and the tie went to Rust under D1 (`trials/e1-origins/`).
  Sources: `research/02-memory-models.md`, `reviews/round1-soundness.md`
  B1–B3, `reviews/round1-agent-usability.md` B2, `design/02-memory.md`,
  `trials/e1/grading.md`.
- **H2: supported.** Capabilities are ordinary parameters passed by
  reference, with attenuation methods and a `Pure` auto trait in place of
  effect variables; effect rows and implicit `with` clauses were rejected
  (D7). Review added two refinements, write-only diagnostic sinks that are
  not capabilities (so agents do not pass `Root` everywhere) and a `Pure`
  that also excludes synchronization handles, and the plumbing cost for
  agents stays unmeasured until E6.
  Sources: `research/07-effects-capabilities.md`,
  `reviews/round1-agent-usability.md` B3–B4, `reviews/round1-soundness.md`
  B6, `design/03-authority-concurrency.md`.
- **H3: refined.** Auto-active SMT contracts give the best agent results on
  the evidence (Dafny 82% against Verus 44% and Lean 27%; agents completed
  81% of real Verus systems proofs), but writing specs is the weak half, and
  agents game verifiers unless trust holes are policed mechanically. Tide
  keeps that style but narrows its role: contracts support the trust surface
  instead of leading it, which keeps Tide out of Bend's territory; v0 runs
  them as runtime checks and property tests, `prove` mode is v1, and the
  escape hatch is `#[trusted]` with a reason, with interactive-proof export a
  later tier.
  Sources: `research/04-verification.md`, `reviews/round1-verification.md`,
  `reviews/round1-strategy.md` B1, `reviews/round2-simplicity.md`.
- **H4: refuted by measurement.** Witness-table generics ran 5.6–23.9x
  slower than monomorphized code in optimized builds and another 1.6–2.1x
  slower on top of unoptimized code, and a separate development code path
  threatened the equivalence of debug and release builds. Tide monomorphizes
  in every profile through one content-addressed instantiation cache, with
  witness tables only as a budgeted fallback that must behave identically
  (D12).
  Sources: `reviews/round1-performance.md` M3 and B2,
  `reviews/round1-soundness.md` B9, `research/03-compile-speed.md` (which
  had recommended dictionaries).
- **H5: open.** The evidence favors the approach: most model failures in
  unfamiliar languages are borrowed syntax, execution feedback took one model
  from 45% to 82% on an unseen language, and 77% of files in a 6.08M-line
  Rust sample used constructs draft 1 would have rejected or respelled, which
  led to Rust spelling wherever the meaning is the same (D1). The first E1
  pilot leaked no Rust habits but missed all three E1 thresholds, mostly
  through gaps in a reference that already uses about 74% of its token
  budget, so the hypothesis stays open until E1 runs on the interpreter.
  Sources: `research/05-llm-code-empirics.md`,
  `research/09-adoption-interop-backends.md`,
  `reviews/round1-agent-usability.md` B1 and B7,
  `reviews/round2-simplicity.md`, `trials/e1/grading.md`.
- **H6: supported.** HVM does not fit as a backend: HVM2 is legacy, later
  versions are pre-release or private, and independent measurements show
  large single-thread and memory overheads. Deterministic parallelism of pure
  code is inherited natively instead (`join`, `par_map`, and `par_for` equal
  their sequential versions, D9), and Bend 2, the closest peer, works in
  tandem with Tide as the owner of proved, parallel kernels (D16).
  Sources: `research/01-bend-hvm.md`, `research/10-concurrency-parallelism.md`,
  `research/11-bend-interop.md`, `reviews/round1-strategy.md` B1.
- **H7: refined.** The strongest evidence is for a fast, single-verdict check
  loop with precise, fix-carrying diagnostics (precise feedback raised Rust
  repair rates from 10.7% to 73.7%), not for structured edits, which gave
  frontier models only 1–5 points. Tide keeps typed holes (D14), whose
  benefit as context is shown but whose effect on frontier agents is
  untested, keeps source text canonical with a base hash on every write, and
  treats check latency as a design requirement.
  Sources: `research/06-agent-native-tooling.md`,
  `research/05-llm-code-empirics.md`, `research/03-compile-speed.md`.
