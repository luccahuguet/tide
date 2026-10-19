# First Principles

Status: working draft, written before the research reports in `research/`
landed. Later design documents cite this one for the problem statement and the
targets; the hypotheses at the end are tested against the evidence, not
assumed.

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
