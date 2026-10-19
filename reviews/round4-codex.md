# Tide second-model design review, round 4

## Verdict

**request_changes**

Seven round-3 v0 objections are resolved, including the parallel-panic
objection through an explicit alternative contract. Replay identity remains
partial, and the inherited floating-point API exposes a second determinism
gap. Neither finding requires bringing a later tier into v0; v0 already
includes the simulator and ordinary scalar types ([01:71–75, Tiers](../design/01-design.md#tiers)).
The detailed evidence and smallest fixes follow below.

Reviewed on 2026-10-19 at HEAD
`206c5468ea05ba8ab63225fba6063da85db5614c`. Read all six requested files;
they had no working-tree diff against that revision. Unrelated workspace
changes were present and left alone. This review follows the decision
record's ownership and tier precedence, rather than treating stale later-tier
paragraphs as v0 requirements ([01:3–6](../design/01-design.md),
[01:62–85](../design/01-design.md#tiers)). Scenarios below are specification
deductions, not observed Tide implementation failures; conditional assumptions
are identified explicitly.

## Round-3 objection status

These statuses track V1–V8 in
[round3-codex.md:47–290](round3-codex.md#blocking-objections-for-v0).

| Round-3 v0 blocker | Status | Cited text and assessment |
| --- | --- | --- |
| **V1. Parallel panic cleanup exposes scheduling** | **Resolved by an alternative contract** | D9 now limits sequential equivalence to executions without branch panics; on panic, writable data has “unspecified (memory-safe) contents” observable by destructors. Simulation always executes parallel regions sequentially ([01:456–469](../design/01-design.md#d9-parallelism-deterministic-with-defined-grouping)). The memory chapter agrees about abandoned writes ([02:761–764](../design/02-memory.md#85-join-par_map-par_for)); PAR-1/PAR-2 and simulator engines implement the exception ([03:818–833](../design/03-authority-concurrency.md#82-semantics), [03:875–889](../design/03-authority-concurrency.md#84-sinks-and-scheduling)). The old destructor may still print different values in production schedules, but that behavior is expressly permitted; it no longer prevents seeded simulation replay. |
| **V2. Lazy statics hide ambient synchronization** | **Resolved** | D7 says “in v0 no lazy statics”; runtime initialization moves into `main` ([01:342–344](../design/01-design.md#d7-authority-capabilities-as-values-no-ambient-authority)). GLOBAL-2 requires compile-time initialization and explicitly excludes first-access effects ([03:191–197](../design/03-authority-concurrency.md#22-no-mutable-globals-no-thread-locals)); the memory chapter and reference agree ([02:386–389](../design/02-memory.md#37-owned-and-static), [reference:580–585](../design/reference.md#64-no-globals-deterministic-hashing)). |
| **V3. Executable clauses mutate application state** | **Resolved for the blocking mutation and evaluation-outcome scenarios** | D10 and the normative v0 clause section require authority-free calls with `Pure + Sync` arguments, and permit mutable access only to clause-created temporaries. The explicit deep rule is “no `&mut` reaches a place that exists outside” evaluation; it covers methods, operators, indexing, and closure calls ([01:492–502](../design/01-design.md#d10-contracts-one-spec-three-uses-defined-semantics), [04:357–370](../design/04-contracts.md#18-v0-clause-semantics)). Thus the earlier `touch(&Cell)` and mutating `PartialEq` scenarios are excluded: `Sync` rejects reachable `Cell`/`RefCell`, while the external-mutable-access prohibition must also cover wrapped borrows and captures. Panics count as violations, and sampled-budget exhaustion is unchecked ([04:388–393](../design/04-contracts.md#18-v0-clause-semantics)). This closes the original application-memory counterexample; it does not establish absence of diagnostic output, as noted below. |
| **V4. Predicate-body edits evade contract approval** | **Resolved** | D11 hashes transitive predicate bodies ([01:554–556](../design/01-design.md#d11-trust-surface-and-transitive-trust)). The approval chapter now expressly includes ordinary functions and operator impls, excludes the specified function's body only when the predicate does not call it, and makes changed existing hashes undecided and approval-gated without a solver ([04:1047–1071](../design/04-contracts.md#11-approval-and-anti-gaming)). The `positive(x)`-to-`true` and custom-equality changes now reach the gate. |
| **V5. Thread-runtime pool starvation** | **Resolved** | D8 chooses “one OS thread per task”; the runtime table agrees and restricts `workers` to the later evented runtime ([01:431–433](../design/01-design.md#d8-concurrency-structured-tasks-no-async-compiled-to-state-machines), [03:136–159](../design/03-authority-concurrency.md#13-root-and-the-four-runtimes)). A nested child no longer needs a worker held by its waiting parent. An explicitly requested `Tasks::limit` is a separate program concurrency bound ([reference:881–887](../design/reference.md#71-scopes-and-tasks)); it does not restore the original unconditional bounded-pool policy. |
| **V6. Replay identifies too few inputs** | **Partial** | D15 requires a replay manifest; §9.6 includes arguments, environment, files, scheduler strategy, fault plan, and injected inputs, and the replay command loads that manifest ([01:665–669](../design/01-design.md#d15-deterministic-simulation-as-a-first-class-mode), [03:959–989](../design/03-authority-concurrency.md#96-tide-sim)). This repairs the original argument/setup/fault-selection omission. Source is still identified by commit, with no required source/dependency snapshot identity, semantic-build identity, or mismatch rejection. D14's response snapshot hash is not explicitly bound into the manifest ([01:643–644](../design/01-design.md#d14-toolchain-contract-for-agents)). See B1. |
| **V7. Indirect calls disagree about suspension** | **Resolved** | D7 and SUSP-1 now inspect both the callable value and every argument's type. SUSP-1 explicitly explains why a `Pure` function pointer or captureless closure can still suspend when passed `&Net` ([01:373–376](../design/01-design.md#d7-authority-capabilities-as-values-no-ambient-authority), [03:746–752](../design/03-authority-concurrency.md#71-which-functions-may-suspend)). This closes the explicit-argument omission without requiring evented lowering in v0. |
| **V8. Restart reuses consumed or mutable captures** | **Resolved** | D15 replaces consuming entries with function items or `Fn` closures ([01:670–672](../design/01-design.md#d15-deterministic-simulation-as-a-first-class-mode)). Crucially, §9.5 also requires `Owned`, `Pure + Sync` captures and `Send`, so the earlier warning about a persistent captured `Cell` is addressed, not merely the `FnOnce` spelling ([03:940–955](../design/03-authority-concurrency.md#95-nodes), [reference:731–745](../design/reference.md#68-simulation)). Immutable configuration can survive as input; mutable volatile state is created inside each invocation. |

Two qualifications do not reopen those resolved counterexamples:

- **Clause read-only means application memory, not necessarily silence.**
  An admissible ordinary helper can still print: sinks do not affect
  authority-freedom, and `Pure` does not exclude sink output ([03:246–250](../design/03-authority-concurrency.md#24-write-only-sinks),
  [03:368–371](../design/03-authority-concurrency.md#34-what-it-does-not-guarantee)).
  Consequently, I do not read “differs only by the missing checks” as proof
  of identical diagnostic traces after clause erasure ([04:365–366](../design/04-contracts.md#18-v0-clause-semantics)).
  Qualify that sentence explicitly, or forbid sink effects transitively if
  diagnostic transparency is intended. No remaining application-memory
  mutation scenario is established by this observation.
- **Temporary iterators have a contradictory example.** The v0 clause
  section first permits `xs.iter().all(...)`, then forbids `all`/`any`
  ([04:357–367](../design/04-contracts.md#18-v0-clause-semantics)); the reference
  repeats both readings ([reference:1124–1131](../design/reference.md#101-clauses)).
  D10 expressly permits the temporary-iterator case ([01:494–495](../design/01-design.md#d10-contracts-one-spec-three-uses-defined-semantics)).
  Apply that owning decision and remove the blanket prohibition; this needs
  no further language choice.

## Remaining v0 blocking objections

### B1. A replay manifest does not identify the executed program

**Evidence.** The manifest enumerates commit, toolchain version, target,
root configuration, and injected inputs, then promises bit-for-bit replay
([01:665–669](../design/01-design.md#d15-deterministic-simulation-as-a-first-class-mode),
[03:985–989](../design/03-authority-concurrency.md#96-tide-sim)). It does not
require the source/dependency snapshot or semantic build settings to be
recorded and validated. D14 already supplies snapshot hashes, and the
reference admits configuration-dependent compilation through `cfg`
([01:643–644](../design/01-design.md#d14-toolchain-contract-for-agents),
[reference:163–167](../design/reference.md#23-const-fn-attributes-cfg)).

**Scenario — conditional on accepting an uncommitted source tree.** Run a
simulation while a local source edit makes its entry print `"A"`; retain
the manifest. Change that uncommitted edit to print `"B"` and replay. The
commit, toolchain, target, seed, setup, and fault plan are unchanged. If
replay builds the current tree, its sink trace changes; if it instead builds
the recorded commit, neither dirty version is necessarily reconstructed.
Sink records are trace events, so this is an observable replay failure
([03:981–989](../design/03-authority-concurrency.md#96-tide-sim)). No cited
rule rejects dirty runs or requires preserving their source. This is a gap
in the promised reconstruction contract, not a claim about an existing CLI.

**Smallest fix.** Bind the replay manifest to D14's exact source/dependency
snapshot and the resolved semantic build configuration, and require replay
to reject mismatches before running. A hash plus rejection is enough; v0
need not archive or reconstruct arbitrary historical worktrees. Alternatively,
explicitly restrict replayable runs to a verified committed snapshot and
still record/check build configuration. Retain the newly recorded setup,
scheduler, fault plan, and injected inputs.

### B2. Canonicalizing only `to_bits` leaves observable NaN nondeterminism

**Evidence.** D3 says floats are IEEE 754 and that `to_bits()` canonicalizes
NaN “so observable results match across backends and targets”
([01:261–262](../design/01-design.md#d3-types-small-decidable-explicit-at-boundaries)).
PAR-3 additionally forbids implicit FMA/reassociation and uses one math
library ([03:834–838](../design/03-authority-concurrency.md#82-semantics)).
However, the reference inherits Rust's scalar API and explicitly keeps
`total_cmp`, while its NaN exception still names only `to_bits`
([reference:40–41, 64–72](../design/reference.md#1-what-is-unchanged),
[reference:385–390](../design/reference.md#43-floats)). The simulator requires
one trace across interpreter and compiled simulation engines
([03:883–890](../design/03-authority-concurrency.md#91-engines)).

**Scenario — deduction from that inherited API, conditional on engines using
different permitted arithmetic NaN signs.** With `x = 0.0`, evaluate
`(x / x).total_cmp(&0.0)`. A negative NaN sorts below zero; a positive NaN
sorts above it. No `to_bits` call occurs. Likewise,
`1.0_f64.copysign(x / x)` turns that sign into an ordinary finite `-1.0`
or `1.0`. Printing the result therefore permits different traces from the
same inputs. The Rust meaning inherited by the cited reference allows
arithmetic NaN sign variation and exposes it through these operations;
this was cross-checked against the primary
[Rust `f64` documentation](https://doc.rust-lang.org/std/primitive.f64.html#method.copysign)
and [`total_cmp`](https://doc.rust-lang.org/std/primitive.f64.html#method.total_cmp).
This establishes an unspecified observable choice, not a tested divergence
between existing Tide engines.

**Smallest fix.** Define one canonical result for NaN-producing arithmetic
in every engine, including constant evaluation, before a sign/payload
observer can see it. Give NaN constants a fixed representation too.
Explicit bit construction and sign operations can retain separately defined,
deterministic behavior. Alternatively, define every NaN-sensitive observer
to operate on canonicalized values; changing only `to_bits` is insufficient.
The required decision is the shared observable rule, not a new float
abstraction or dependency.

## Ready to implement v0?

**No, for a checker and reference interpreter that claim the complete
documented v0 contract.** Two small decisions remain: bind replay to an exact
source/build identity with mismatch rejection (B1), and define deterministic
arithmetic NaN observations consistently across engines (B2). Both are part
of the existing v0 determinism promise, rather than later-tier additions
([01:71–75](../design/01-design.md#tiers),
[01:261–262](../design/01-design.md#d3-types-small-decidable-explicit-at-boundaries),
[01:665–669](../design/01-design.md#d15-deterministic-simulation-as-a-first-class-mode)).

The ownership checker can be started independently: the memory chapter
defines acceptance through the Rust NLL translation and names the
cross-check boundary ([02:283–292](../design/02-memory.md#28-precision),
[02:1248–1259](../design/02-memory.md#11-soundness-sketch-translation-to-rust)).
The original eight objections do not justify restoring moved-out tracking,
lazy statics, SMT, or evented execution to v0; the cited resolutions provide
smaller contracts for those areas. Apply the temporary-iterator cleanup
above before using the reference as a conformance oracle.

Validation was document comparison and scenario analysis, with independent
read-only memory/runtime scrutiny and the cited Rust API check. No Tide
compiler, interpreter, runtime, solver, benchmark, or learnability trial was
run. Only this review file was written; runtime behavior remains unproven.
