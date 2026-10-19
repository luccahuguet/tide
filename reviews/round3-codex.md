# Tide second-model design review, round 3

## Verdict

**request_changes**

The tier split removes several reasons to delay v0. I would keep it. Six of
the nine round-2 blockers now have sufficient answers for v0, one is partial,
and two remain open, including one deferred to v1. The remaining v0 problems
concern observable semantics and the review boundary: parallel failure still
exposes scheduling, lazy statics evade the `Pure` argument, and executable
contracts need rules for state changes and their semantic dependencies.

Reviewed on 2026-10-19 at HEAD
`f142e7fabd3ae03a8135b6b4cda9b84ca794adf8`. I read the four requested design
documents and both earlier Codex reviews in full, and consulted the relevant
parts of `design/reference.md`. The six requested inputs stayed unchanged
relative to that HEAD; the workspace contained unrelated changes. A concurrent
edit updated the supporting reference during final validation. I rechecked
the cited reference sections and incorporated its node-entry disagreement
below, without modifying that file. This review follows
[01, Tiers](../design/01-design.md#tiers): proofs, type invariants, evented
lowering, FFI, Rust import, and reductions are not v0 requirements. A stale
later-tier paragraph is not, by itself, a v0 blocker.

The scenarios below are deductions from the specification, not exploits of
an implementation. Conditional scenarios identify an unspecified choice;
they do not assert that an implementation already made that choice.

## Round-2 objection status

These statuses track B1–B9 of
[round2-codex.md](round2-codex.md#blocking-objections).

| Round-2 blocker | Status | Current text and assessment |
| --- | --- | --- |
| **B1. Locally created synchronization and callable arguments** | **Partial** | [01, D7](../design/01-design.md#d7-authority-capabilities-as-values-no-ambient-authority) now says creating synchronization handles “takes `&Tasks`”; [03, §1.2](../design/03-authority-concurrency.md#12-synchronization-handles) applies this to every constructor. That closes the local-channel scenario. But [03, §7.1, SUSP-1](../design/03-authority-concurrency.md#71-which-functions-may-suspend) still classifies an indirect call solely from the callable value's `Pure` trait, ignoring explicit arguments. D7 repeats that rule. See V7. |
| **B2. `Arc` drop and conditional cloning** | **Resolved** | [01, D7](../design/01-design.md#d7-authority-capabilities-as-values-no-ambient-authority), [02, §9](../design/02-memory.md#9-pure), and [03, GLOBAL-6](../design/03-authority-concurrency.md#22-no-mutable-globals-no-thread-locals) agree that `Arc` and `sync::Weak` are synchronization handles and not `Pure`, because “drop timing and conditional cloning can reveal scheduling.” They cannot enter the old authority-free parallel example. Removing `Frozen` does not reopen it. |
| **B3. Offloaded `ThreadBound` construction** | **Open, deferred to v1** | The [tier table](../design/01-design.md#tiers) excludes FFI, Rust import, and evented execution from v0. That removes the scenario from v0, rather than repairing it. [03, FFI-5 and §5.4](../design/03-authority-concurrency.md#5-foreign-calls-v1) still offload effectful constructors and exempt only a call “taking a `ThreadBound` argument.” A thread-bound result still lacks creation-thread protection. |
| **B4. Recovery with a half-updated borrowed referent** | **Resolved for the original scenario** | [01, D5](../design/01-design.md#d5-errors-values-as-in-rust), [02, §8.3](../design/02-memory.md#83-child-failure), and [03, TASK-1](../design/03-authority-concurrency.md#62-failure-cancellation-and-deadlines) now propagate a child's panic through its scope. Recovery occurs only at `spawn_isolated` with an `Owned` closure. The memory chapter distinguishes normal `Err` returns from panics and permits observation of abandoned borrowed data only during destruction. [04, §4.3](../design/04-contracts.md#43-panics-and-abandoned-values) requires invariant-free verification of `Drop` in v1. The moved-out rule is gone; the old contradictory normal-return rule is gone too. |
| **B5. Logging configuration changes results** | **Resolved** | [03, SINK-3](../design/03-authority-concurrency.md#24-write-only-sinks) requires both argument evaluation and formatting at every level and profile; only the completed record may be discarded. Formatting panics propagate. SINK-8 refers back to this rule for release elision. The original mutation and panic counterexamples no longer apply. The separate broken-pipe exception remains a qualification on the broader guarantee. |
| **B6. Deferred overflow writes observed by destructors** | **Resolved for v0** | [01, D3](../design/01-design.md#d3-types-small-decidable-explicit-at-boundaries) says “v0 traps are precise”; [03, §9.1](../design/03-authority-concurrency.md#91-engines) and [04, §3.2](../design/04-contracts.md#32-arithmetic-and-traps) agree. The optimizer rule moves to a later tier and explicitly owes an unobservability argument. Deferral is not established as sound merely by postponing it. |
| **B7. Parallel panic exposes speculative writes** | **Open in v0** | Replacing `par { }` with `join` leaves the admission and failure rules intact. [03, PAR-1](../design/03-authority-concurrency.md#82-semantics) still promises identical values, writes, and output on every schedule, with only exhaustion and divergence exceptions; PAR-2 still stops right branches opportunistically. See V1. |
| **B8. Compile-time body dependencies** | **Resolved in substance** | [01, D12](../design/01-design.md#d12-compilation-and-backends) and [04, §8.1](../design/04-contracts.md#81-what-callers-see) now include “every `const fn` body evaluated at compile time” in consumers' cache keys. That includes bodies evaluated transitively. D12's following blanket “a body edit never re-checks dependents” should say *runtime-only body edit that changes no consumed compile-time fact*. The explicit exception supplies the missing mechanism; this wording cleanup is not another blocker. |
| **B9. Deadline policy contradicts the fetcher test** | **Resolved** | [03, TASK-4](../design/03-authority-concurrency.md#62-failure-cancellation-and-deadlines) now says “expiry alone is not a failure.” [03, §12.2](../design/03-authority-concurrency.md#122-a-concurrent-fetcher-with-a-deadline) wraps the cancelled request in `Ok`, expects partial results, and checks `NetError::Cancelled`. This matches [01, D8](../design/01-design.md#d8-concurrency-structured-tasks-no-async-compiled-to-state-machines). Soft deadlines and shielded joins are explicit, so this does not promise that cancellation interrupts divergent computation. |

## Blocking objections for v0

### V1. `join` still makes panic cleanup depend on scheduling

**Section.** [01, D5 and D9](../design/01-design.md#d9-parallelism-deterministic-with-defined-grouping);
[02, §§1.3 and 8.5](../design/02-memory.md#85-join-par_map-par_for);
[03, PAR-1/PAR-2 and §8.4](../design/03-authority-concurrency.md#82-semantics).

**Problem.** `Pure + Send` allows mutation through disjoint `&mut` captures
and does not establish panic-freedom. Selecting the leftmost panic and
discarding later sink buffers cannot undo writes into the enclosing frame.
The enclosing destructor's output belongs to neither discarded branch.

**Scenario.** A local `Report { value: 0 }` prints its value in `Drop`.
Run the schematic call
`join(|| panic!("left"), || report.value = 1)`. Rust-style disjoint capture
borrows `report.value`; the program does not move that field out of a type
with `Drop`. If the right branch writes before noticing PAR-2's stop flag,
unwinding prints one. If it stops first, unwinding prints zero. Sequential
left-to-right execution prints zero. D5's propagation prevents normal
continuation but still runs the destructor, so it does not fix this case.
Running these branches on real cores under simulation also defeats seeded
replay.

**Smallest fix.** Give v0 `join` and `par_*` left-to-right sequential execution
unless the compiler establishes panic-freedom for the entire parallel region,
including calls, formatting, contracts, and destruction. With no such
analysis in the first interpreter, use sequential execution throughout.
Keep true parallel execution behind a later admission gate. Merely excluding
panics from the equivalence sentence would still leave the simulator's
real-core execution non-replayable.

### V2. Lazy statics introduce ambient synchronization into `Pure` code

**Section.** [03, GLOBAL-2](../design/03-authority-concurrency.md#22-no-mutable-globals-no-thread-locals),
[§§3.3–3.5](../design/03-authority-concurrency.md#33-what-authority-freedom-guarantees),
and [LOCK-4](../design/03-authority-concurrency.md#65-sharedwith);
[reference, §6.4](../design/reference.md#64-no-globals-deterministic-hashing).

**Problem.** GLOBAL-2 admits a shared lazy initializer because its builder is
authority-free and its successful value is deterministic. The reference
explicitly makes `Lazy<T>` `Pure` when `T` is. But the builder may print,
panic, recurse, or force another lazy static. Enforcing initialization “at
most once” introduces shared initialization state and waiting without
`&Tasks`. The general `Pure` argument does not cover this exception.

**Scenario.** A successful `build() -> u8` prints `"init"` and returns one.
Two `join` branches force `static X: Lazy<u8> = Lazy::new(build)` and then
print `"left"` and `"right"`. If the left branch initializes X, ordered
buffers produce `init, left, right`; if the right initializes X, they
produce `left, init, right`. No panic is needed, so V1's panic-free gate
alone would not repair this. Another admitted builder can force its own
static, or two builders can force each other: the text gives no cycle rule
or failure outcome. Such waits also contradict the premise that authority-free
code cannot synchronize or suspend, including inside `Shared::with`.

**Smallest fix.** Omit lazy statics from v0. Use `const` for compile-time
tables, or initialize in `main` and pass shared references. Re-admit laziness
only with explicit rules for initialization effects, panic/poison behavior,
cycles, and forcing under parallel execution. Classifying a lazy handle as
non-`Pure` while still allowing ambient access to it would not close the
signature boundary.

### V3. Authority-free executable clauses can change the program's state

**Section.** [01, D10](../design/01-design.md#d10-contracts-one-spec-three-uses-defined-semantics);
[03, §§3.3–3.5](../design/03-authority-concurrency.md#33-what-authority-freedom-guarantees);
[04, Tier, §1.3, and §2.6](../design/04-contracts.md);
[reference, §10.1](../design/reference.md#101-clauses).

**Problem.** v0 clauses may call ordinary authority-free functions and trait
methods, and `==` invokes `PartialEq`. Authority-freedom permits task-private
interior mutation and sink output. It supplies no read-only or termination
guarantee. This contradicts §1.3's promise that spec evaluation “never
changes program state.” Erasing successful checks can therefore change the
program, not just remove error detection. Banning assignment or `&mut` syntax
in the clause does not constrain its callees.

**Scenario.** Define an authority-free helper
`touch(c: &Cell<u32>) -> bool` that sets `c` to one and returns `true`.
Attach `requires touch(c)` to `read(c: &Cell<u32>) -> u32`, whose body returns
`c.get()`. Starting at zero, a debug call returns one and a release call
returns zero. The precondition succeeds. A custom `PartialEq` can hide the
same mutation behind a clause's `==`. A diverging helper also turns a
nominally bounded clause into an unbounded check.

**Smallest fix.** Define and check a conservative read-only executable
clause subset, transitively through helpers, trait operators, and cleanup;
reject mutable access to program storage, including through `Cell` and
`RefCell`. Specify the outcome when clause evaluation panics or exceeds its
budget. This can begin with compiler-known accessors and a restricted helper
body checker; it does not require v1 SMT proofs. If v0 instead intentionally
adopts side-effecting debug-assert semantics, record that decision and withdraw
the state-preservation promise. The checker must not silently choose between
these meanings.

### V4. The approval hash rule does not cover v0's ordinary predicate bodies

**Section.** [01, D11](../design/01-design.md#d11-trust-surface-and-transitive-trust);
[04, Tier and §11, “What is hashed”](../design/04-contracts.md#11-approval-and-anti-gaming);
[reference, §§10.1 and 11](../design/reference.md#101-clauses).

**Problem.** §11 belongs to v0 without its solver checks. Its spec-closure
hash includes `spec fn` and `#[spec]` bodies, then excludes “other exec
bodies.” But v0 has neither of those special function categories and allows
ordinary functions in clauses. The general instruction to hash everything
the meaning depends on conflicts with that explicit exclusion. An
implementation following the exclusion would miss semantic weakening.

**Scenario, conditional on that exclusion.** A public contract says
`ensures positive(result)`, where the ordinary public helper `positive`
returns `x > 0` and has no contract of its own. Change its body to `true`.
The clause text, signatures, authority, and trust holes stay unchanged. If
ordinary helper bodies stay outside the closure hash, no root-spec change
reaches the approval classifier, although the check accepts a weaker result.
Changing a user `PartialEq` used by `ensures` creates the same problem
through an implicit call.

**Smallest fix.** Include the transitive bodies and resolved implementations
used to evaluate v0 clauses, including implicit operators and relevant
constants, in the protected contract closure. In solver-free v0, gate a
changed semantic closure conservatively unless the tool can establish an
accepted equivalence. Do not hash the ordinary implementation body being
specified merely because it has a contract; hash code used by the predicate.
Retain the external approval authority already defined in §11.

The v0 trust claim also needs its stated limits. [03, Tier and SINK-9/10](../design/03-authority-concurrency.md)
defer sink-effect reporting to v1, yet SINK-10 says exfiltration needs `Net`,
`Fs`, or `Proc`. `info!("{}", secret.expose())` disproves that sentence.
Keep the chosen sink exception, describe it as ambient diagnostic output
unreported by v0's effect summary, and qualify the thesis's absolute “no
ambient authority” wording. Declared authority is an upper bound, not proof
that a body edit has no new output or exercises no additional existing grant.

### V5. The default thread runtime lacks a progress rule

**Section.** [03, §1.3](../design/03-authority-concurrency.md#13-root-and-the-four-runtimes),
[§6.3](../design/03-authority-concurrency.md#63-joins), and
[§7.3](../design/03-authority-concurrency.md#73-the-other-runtimes-and-foreign-code).

**Problem.** The default `threads` runtime uses pooled OS threads, keeps each
task on one thread for life, blocks that thread on waits, and never executes
children inline. The manifest has `workers = "cores"`. No rule requires
replacement workers when a task waits, yet §1.3 promises runtimes “differ in
cost, not meaning.” This is the still-unanswered round-2 N5; it matters to a
v0 runtime, rather than only to later optimization.

**Scenario, conditional on a bounded pool.** With one worker, a child starts
a nested scope, queues a grandchild, and joins it. The child occupies the
only worker; the grandchild cannot start. The grandchild's body can simply
return `Ok(())`, with no channel cycle or divergent computation. A
thread-per-child runtime completes this program. Shielding the join supplies
memory safety, not a worker.

**Smallest fix.** Specify one OS thread per live task in both v0 execution
modes, or defer `threads` and ship `blocking` first. If a pool remains in v0,
require a concrete compensation/progress mechanism and say that `workers`
does not cap runnable child capacity while parents block. Do not leave this
as an implementation freedom under the runtime-equivalence promise.

### V6. Replay identifies too few inputs

**Section.** [01, Thesis and D15](../design/01-design.md#d15-deterministic-simulation-as-a-first-class-mode);
[03, §§9.2–9.7](../design/03-authority-concurrency.md#92-rootsim);
[01, D14](../design/01-design.md#d14-toolchain-contract-for-agents).

**Problem.** §9.6 promises bit-for-bit replay from seed, commit, and
toolchain. Its replay command preserves only test target and seed. The same
section allows external fault plans and program arguments; §9.2 allows
world setup, and §9.3 allows scheduler strategies. None is identified by the
printed replay command. A commit also omits a dirty working tree. This was
round-2 N1 and is part of v0's central simulator contract.

**Scenario.** Run a test with `--faults none`, then replay the emitted
command under the default `swarm` plan. Alternatively, edit `plan.toml`
outside the source snapshot or change program arguments. The target, seed,
commit, and toolchain can stay identical while the experiment changes. The
two-run self-check can pass both times; it tests repetition of the current
configuration, not reconstruction of the original one.

**Smallest fix.** Define a replay manifest containing the source/dependency
snapshot hash, target and arguments, setup inputs, scheduler and model
versions/configuration, full fault-plan contents, seed, and semantic build
settings. Reuse D14's snapshot identity. The replay command should load that
manifest and reject a mismatch or identify an explicit changed-program
experiment. This need not be production response recording, which stays v1.

Also narrow [03, §9.3](../design/03-authority-concurrency.md#93-time-and-scheduling)'s
claim that synchronization interleavings reach “every observable outcome.”
The chapter itself excludes CPU-time behavior, kernel details, and other
phenomena in §9.8. Promise coverage of the stated simulator model, not all
real-world behaviors of data-race-free programs.

### V7. Indirect calls still have two suspension classifications

**Section.** [01, D7](../design/01-design.md#d7-authority-capabilities-as-values-no-ambient-authority);
[02, §9](../design/02-memory.md#9-pure);
[03, Tier and SUSP-1](../design/03-authority-concurrency.md#71-which-functions-may-suspend).

**Problem.** The function-instance rule considers parameters and captures.
The indirect-call rule considers only the callable value. The tier note
explicitly keeps §7.1's may-suspend flag in v0. Ordinary blocking execution
avoids an immediate state-machine calling-convention failure, but a checker
cannot export one unambiguous flag using these rules.

**Scenario.** `let f = |net: &Net| net.connect("db:5432");` captures nothing,
so its value is `Pure`. A Tide `fn(&Net) -> ...` pointer is also `Pure`, as
§9 explicitly says. Calling either with `net` can suspend. SUSP-1's
function-instance sentence says yes; its indirect-call sentence says no.
There is no violation of the `Pure` auto trait here: authority arrives in
the explicit argument.

**Smallest fix.** For an indirect call, inspect its callable signature's
parameter types as well as the callable's capture/erased auto-trait bounds.
Use that rule consistently for the interface flag. Keep the no-suspension
guarantee for a call whose inputs and captures are all `Pure`, subject to
closing the lazy-static exception in V2.

### V8. Restart cannot call an entry that has consumed its captures

**Section.** [01, D15](../design/01-design.md#d15-deterministic-simulation-as-a-first-class-mode);
[03, §9.5](../design/03-authority-concurrency.md#95-nodes);
[reference, §§6.8 and 16](../design/reference.md#16-where-the-documents-disagree).

**Problem.** §9.5 accepts `entry: FnOnce(NodeRoot) -> Result<T, E>` with
`Owned + Send + Pure` captures, but promises that `restart` “reruns the
entry.” `Owned` excludes borrowed captures; it does not make moved captures
reusable or duplicable. The concurrently updated reference recognizes this
conflict and provisionally requires a function item or `Fn` closure. The
chapter still accepts the consuming case.

**Scenario.** A node entry captures a `Vec<u8>` by move and passes ownership
of it to `boot(root, image)`. The first call consumes the closure's image.
After a crash frees the node's memory, a second call has no image to move.
The text specifies neither an initial-state snapshot nor a factory that
reconstructs the consumed environment.

**Smallest fix.** For v0, require a captureless function pointer as the
restartable entry, including coercible captureless closures. Each invocation
receives a fresh `NodeRoot` over the surviving disk and creates its own
in-memory state. Put configuration in the node's modeled setup inputs. A
later captured factory needs an explicit rule for which state persists
outside a crash; changing only `FnOnce` to `Fn` still admits a factory with
mutable `Cell` captures that remember earlier starts.

## Later-tier concerns

These do not need implementation in v0.

- **Thread-bound construction remains unsafe if offloaded.** Round-2 B3 is
  still open in [03, FFI-5 and §5.4](../design/03-authority-concurrency.md#5-foreign-calls-v1).
  Run construction, use, and destruction on the owning thread, including
  results containing a `ThreadBound` value. Pinning after receiving the
  value cannot change where the constructor ran.

- **The reduction recipe permutes inputs, so associativity is insufficient.**
  [03, §8.3 steps 2–3](../design/03-authority-concurrency.md#83-par_reduce-and-the-canonical-grouping-v1)
  use eight strided accumulators. For nine strings `a` through `i`, lane
  zero concatenates `a` and `i`, and combining lanes in order yields
  `aibcdefgh`. A sequential fold yields `abcdefghi`. Concatenation is
  associative and has an identity; neither side traps. Thus even a proved
  `Monoid` does not justify the advertised fold equality in that recipe or
  [04, §5.3](../design/04-contracts.md#53-laws). Use contiguous subranges
  to preserve input order, or require and prove commutativity as well.
  Canonical arithmetic grouping fixed the original round-1 objection;
  this is a different defect in the later operational recipe.

- **Proof-only consistency checks still need an unreachable-path exception.**
  [04, §10 item 3](../design/04-contracts.md#10-spec-quality) still requires
  satisfiable assumptions at each obligation, while §3.1 proves panic sites
  unreachable. Distinguish inconsistent entry axioms from an impossible
  control-flow path, as round-2 N3 requested. In contrast, §2.6 now addresses
  N4 with empty trust closures, indirect-entry restrictions, type-invariant
  obligations, and per-instantiation law status. Keep check deletion in v1.

- **Machine clauses and mathematical clauses need a migration contract.**
  [01, D10](../design/01-design.md#d10-contracts-one-spec-three-uses-defined-semantics)
  intentionally gives v0 machine arithmetic and `PartialEq`, while `prove`
  uses mathematical integers and model equality. That tier decision wins
  over [04, §1.3](../design/04-contracts.md#13-spec-expressions)'s “same in
  all three uses” sentence. Likewise, the v0 tier excludes clause classes,
  while §2.3 still describes evaluating only `checked` clauses. Write down
  the v0 override before turning chapter examples into acceptance tests;
  do not import v1 sampling or arithmetic semantics by accident. Views of
  strings still need byte/character-length distinctions before v1 proofs
  use them ([04, §§1.4 and 1.6](../design/04-contracts.md#14-spec-types-and-views),
  round-2 N6).

- **Production redaction cannot itself preserve exact replay.**
  [01, D17](../design/01-design.md#d17-record-replay-and-authority-policies)
  promises response replay and record-time redaction of `Secret<T>`.
  If a secret response influences a branch and only a redacted placeholder
  survives, the replayer lacks that input. Specify protected replay payloads
  with redacted display, require separately supplied inputs, or qualify the
  exact-replay claim. This is a conditional information-loss scenario;
  the trace format is not defined yet.

- **Deferred traps still owe a cleanup argument.**
  [04, §3.2](../design/04-contracts.md#32-arithmetic-and-traps) labels its
  flush rule a candidate and acknowledges destructor observability. That is
  the appropriate status. Neither the Rust origin cross-check nor a future
  proof of borrow safety establishes equivalence of delayed panic traces.

One small chapter cleanup does not require a new language decision.
[01, D5](../design/01-design.md#d5-errors-values-as-in-rust) already requires
any child panic to propagate: qualify TASK-2's first-error return rule so it
does not read as overriding that requirement after an earlier `Err`.
The concurrent reference update already replaced its stale contradiction
inventory. Neither cleanup requires reviving moved-out tracking or enlarging
v0.

## Ready to implement v0?

**No, as a checker and interpreter claiming to implement the documented v0
contract.** The origins/ownership subset is concrete enough to prototype:
[02, §§2–6 and 11](../design/02-memory.md#2-origins) define formation,
variance, elision, modular checking, drop check, and the Rust cross-check
boundary. The absence of SMT, FFI, or evented lowering is not an obstacle to
that work. The v0 runtime and contract oracle still need these decisions:

1. Adopt sequential parallel-API semantics until complete panic-freedom
   justifies parallel admission (V1).
2. Remove lazy statics from v0's ambient environment (V2).
3. Choose read-only clause evaluation and define clause failure/budget
   behavior, or explicitly accept its profile-dependent state effects (V3).
4. Protect the executable semantic dependencies of v0 contracts; gate
   undecided closure changes without an SMT solver (V4).
5. Choose a thread-runtime progress policy that handles nested spawning and
   blocking joins (V5).
6. Define the replay manifest and limit coverage claims to its model (V6).
7. Correct indirect-call classification to include explicit parameters (V7).
8. Give restart a reusable entry and a defined boundary for initial state
   (V8).

These decisions do not require the v1 features to move back into v0. They
give the first interpreter one observable result to implement and the checker
one set of rules to enforce.

Validation covered document reading, cross-document comparison, and the
scenarios above. I ran no compiler, runtime, solver, benchmark, or separate
agent experiment. Only this review file was created; the scenarios establish
specification contradictions and conditional gaps, not implementation bugs.
