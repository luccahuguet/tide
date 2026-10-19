# Ferox second-model design review, round 2

## Verdict

**request_changes**

The direction remains credible, and several round-1 objections now have concrete answers. The remaining problems concern the guarantees themselves: the rules can classify suspending code as non-suspending, shared ownership still exposes scheduling, and failure cleanup can reveal supposedly unobservable writes. These affect the checker and interpreter's observable semantics, rather than just a future optimizer.

Reviewed on 2026-10-19 against the six requested documents, read completely, with the final decision-record delta checked at HEAD `1cb506e7d3901686799a8aab8a22399c2173f4e5`. During review, concurrent edits added durability and deadline rules to `01-design.md`; B9 includes the resulting deadline contradiction. The other five inputs were unchanged. The resolution audit describes draft 2; it is a checklist, not evidence that draft 2.1 and the subsequent chapters resolve an objection. In particular, its “unwritten memory chapter” observation is obsolete. See [round1-resolution.md, Summary and Resolution table](round1-resolution.md).

This is a document review. Examples below are schematic Ferox programs and deductions from the written rules, not demonstrated compiler exploits. B3 is explicitly conditional on an imported API's thread-affinity contract. No performance or adoption prediction is needed for the verdict.

## Round-1 objections status

Statuses concern the original scenarios in [round1-codex.md, Blocking objections](round1-codex.md), not whether the entire subsystem is ready.

| Round-1 objection | Status | Evidence and remaining limitation |
| --- | --- | --- |
| B1: capability freedom versus purity | **Partial** | `Pure` and `Frozen` are separate; synchronization handles, raw pointers, erased types without explicit bounds, and unbounded type parameters are excluded; memoization excludes `&mut`. This closes the original `increment` and passed-in shared-cell examples. See [02-memory.md, §9](../design/02-memory.md), [03-authority-concurrency.md, §§3.1–3.5](../design/03-authority-concurrency.md). Locally constructed handles, callable arguments, `Arc`, and sinks still break the argument: B1, B2, B5 below. |
| B2: authority escape routes | **Partial; foreign boundary substantially repaired** | Foreign claims, callback/destructor obligations, attenuation responsibility, unverified Rust labels, and refusal of unmodeled simulation are explicit: [03, §§5, 9.8, 11](../design/03-authority-concurrency.md). Diagnostics are now a deliberate ambient-output exception with package effects and an optional gate, rather than an invisible escape: [03, §2.4](../design/03-authority-concurrency.md). The absolute thesis still needs qualification, and sink noninterference is false as written: B5 and N1. |
| B3: suspended foreign frames and migration | **Original scenario resolved** | Authority-free callbacks cannot suspend; adapters catch panics; live `ThreadBound` values pin tasks: [03, §§5.3–5.4](../design/03-authority-concurrency.md). That closes the foreign-frame example, assuming the authority-free classification is repaired. Offloaded constructors introduce a different affinity problem: B3. |
| B4: composition across proof modes | **Original scenario resolved at the rule level** | Unproved postconditions, panic-freedom, termination, and instantiated laws become transitive assumptions; statuses distinguish resource limits; CI owns proof caches. Check deletion requires an empty trust closure and guaranteed preconditions, with bounds-check deletion off by default: [04-contracts.md, §§2.6, 3.1, 5.3, 8](../design/04-contracts.md). The invariant/failure boundary remains inconsistent: B4. N4 identifies a necessary clarification before enabling deletion. |
| B5: universal runtime fallback | **Resolved** | Clause classes, unchecked/skipped reporting, evaluation budgets, mathematical runtime integers, solver-free `check`, and separate proof outcomes are defined: [04, §§1.3, 2.2–2.4, 8.2](../design/04-contracts.md). Sampling no longer masquerades as an equivalent universal check. |
| B6: reduction versus sequential fold | **Original reduction objection resolved** | Fixed grouping, `Combine` versus proved `Monoid` laws, checked-integer partial sums, and panic selection are explicit: [01-design.md, D9](../design/01-design.md), [03, §§8.2–8.3](../design/03-authority-concurrency.md), [04, §5.3](../design/04-contracts.md). The broader claim about sequentially equivalent writes during panics still fails: B7. |
| B7: trust diff replacing behavior review | **Resolved as a claim limitation** | Ordinary review remains necessary; coverage, trivial-body checks, mutation results, and protected positive/negative spec examples supplement proofs: [01, D11](../design/01-design.md), [04, §10](../design/04-contracts.md). These mechanisms do not establish specification completeness, and the revised design need not claim that they do. |
| B8: editable hashes as approval | **Resolved for the original bypass** | Approval is against a protected base, signed with independently controlled keys; unprotecting tests and lowering verification need approval; writable caches are advisory: [01, D11 and D14](../design/01-design.md), [04, §§8.4, 11](../design/04-contracts.md). Regenerating a lockfile no longer supplies approval. |

The origins chapter also materially answers round-1 N1: nested reference positions, variance, parameter relations, borrowing closures, scoped spawning, drop check, and accept/reject examples are present. It explicitly identifies non-exact translation boundaries instead of treating Rust acceptance as a proof of the entire runtime. See [02, §§2–11](../design/02-memory.md). The incremental-constant objection, round-1 N2, remains: B8 below.

## Blocking objections

### B1. `Pure` inputs do not close locally created synchronization or callable arguments

**Decision/section.** [01, D7–D8](../design/01-design.md); [03, §§1.2, 3.2–3.3, 6.4, 7.1](../design/03-authority-concurrency.md).

**Problem.** Authority-freedom is determined from parameters and captures, while channel/cell construction needs no capability. There is no general body rule rejecting a synchronization operation reached through a locally created handle. Separately, SUSP-1 says an indirect closure call may suspend exactly when the *closure value's type* is not `Pure`. That ignores capabilities supplied as call arguments. These rules disagree with classifying function instances from both parameters and captures.

**Scenario.** A zero-argument function creates a channel, retains its sender, and calls `recv()`. Its parameter/capture test says authority-free, but `recv` waits and observes cancellation. A finite variant sends a value locally and then receives it: cancellation can change its result despite identical explicit inputs. No spawn or foreign code is necessary.

Also, `let f = |net: &Net| net.connect("db:5432");` captures nothing, so the closure value is structurally `Pure`. Calling `f(net)` can suspend. The function-instance rule sees `&Net`; the indirect-call rule sees a `Pure` value. Under evented lowering this is a calling-convention disagreement, not merely an inaccurate review label.

**Smallest fix.** Check authority-freedom transitively through calls, implicit drops, and local synchronization operations; reject a body that violates its signature-derived non-suspending classification. For indirect calls, inspect the callable's parameter types as well as its captures/erased bounds. Handle creation can remain free; it must not license checkpoint operations inside a function promised to be authority-free. Apply this one rule to ordinary functions, lock bodies, `par`, and foreign callbacks.

### B2. `Arc` still exposes reference counts through `Drop` and `Clone`

**Decision/section.** [01, D7 and D9](../design/01-design.md); [02, §9, especially `Pure` and the `Arc::make_mut` exception](../design/02-memory.md); [03, GLOBAL-6, §§3.3 and 8.2](../design/03-authority-concurrency.md); [04, §5.1, `Clone`](../design/04-contracts.md).

**Problem.** Removing count readers does not make count-dependent execution unobservable. Last-owner destruction runs user code; `make_mut` conditionally calls user `Clone`. A view-equality postcondition on `Clone` does not guarantee panic-freedom or absence of diagnostic output. No rule requires these properties for an `Arc<T>` to be `Pure` or `Frozen`.

**Scenario.** Let a fieldless, structurally `Pure + Frozen + Send + Sync` type have a destructor that panics. Put it in an `Arc`, clone it, and run `par { drop(a), drop(b) }`. Whichever branch drops last runs the destructor. Thus the branch whose panic is reported depends on scheduling, despite PAR-1/PAR-2. No explicit count reader, mutable payload, or unsafe code is involved.

Likewise, an authority-free function calling `Arc::make_mut` can panic or emit a clone diagnostic only when another task has not yet dropped its clone. Even a `Clone` that returns an equal view can exhibit this difference.

**Smallest fix.** Conservatively classify `Arc<T>` as a synchronization handle, hence non-`Pure` and non-`Frozen`, for the initial implementation. Parallel readers can borrow `&T` from an externally retained owner. Re-admit shared ownership only after accounting for last-drop behavior and conditional cloning, not just explicit count APIs. Merely removing `make_mut` does not fix `drop`.

### B3. Blocking-pool construction can put `ThreadBound` values on the wrong thread

**Decision/section.** [01, D8 and D13](../design/01-design.md); [03, FFI-5, §5.4, RUST-4](../design/03-authority-concurrency.md); [02, §8.1](../design/02-memory.md).

**Problem.** Effectful foreign calls default to the blocking pool. The inline exception covers a call *taking* a `ThreadBound` argument. It does not cover a call *producing* one. Pinning the receiving task to its current worker afterward does not recover the creation thread.

**Scenario — conditional counterexample.** An imported API creates a session that must be used and destroyed on its creation thread; its result is correctly marked `ThreadBound`. Its constructor takes filesystem authority and no existing session. FFI-5 runs creation on blocking worker B, then returns the session to evented worker A. The compiler pins the task to A, where the next method or destructor violates the foreign contract. All declarations can be honest; no suspending foreign callback is needed.

**Smallest fix.** Run calls that create, access, or destroy `ThreadBound` values inline on their owning worker, including constructors whose result contains such a value. Reject offloading across that boundary; blocking on that worker is the conservative v0 cost. Define pinning over the complete creation-to-destruction interval.

### B4. Child-failure recovery has opposite rules in the decision and memory chapter

**Decision/section.** [01, D5](../design/01-design.md); [02, §8.3 and §12 item 8](../design/02-memory.md); [04, §§2.6, 4.2–4.3](../design/04-contracts.md).

**Problem.** D5 requires repair or panic before returning with an invalidated referent. The normative memory chapter explicitly allows returning without repair and acknowledges that the caller receives a half-updated value. Contracts then say no other verified code assumes the invariant of an abandoned value, but give no cross-function invalidation mechanism for that return. An open question does not reconcile these definitions.

**Scenario.** In a type's defining module, an `assert`-mode helper lends `&mut Span` to a child. The child sets `lo > hi` and panics. In the default release configuration, the helper catches the scope's `Err` and returns normally without reading the span; invariant clauses are erased. This return is expressly permitted by §8.3 and forbidden by D5. A later caller still holds an ordinary `Span`, while §4.2 normally assumes type invariants at function entry and after calls. Verification modes cannot make two opposite checker outcomes a usable contract.

**Smallest fix.** Follow D5: every normal return path must reinitialize each invalidated place behind a reference, or the helper must panic. State this as a flow-checking obligation in §8.3 and its interface implications. Keep destruction of abandoned values invariant-free and specify exactly when owned abandoned storage is dropped; do not silently treat “valid memory” as “valid invariant.”

### B5. Logging configuration changes authority-free results

**Decision/section.** [01, D7](../design/01-design.md); [03, SINK-3, SINK-8, §§3.2–3.3](../design/03-authority-concurrency.md).

**Problem.** Sink configuration is ambient, and format arguments are evaluated only at enabled levels. Catching a panic in `Display::fmt` does not undo argument evaluation, prior mutations, or mutations performed by a formatter. Therefore the stated result/panic/memory noninterference guarantee does not hold.

**Scenario.** A function with only `x: &mut u32` executes `info!("{}", { *x = 7; *x });` and returns `*x`. With entry value zero it returns seven when `info` is enabled and zero when disabled. The function meets the written authority-free input criterion. A panicking argument expression similarly changes whether the function returns at all. Release-level elision introduces the same distinction between profiles.

**Smallest fix.** Evaluate arguments and formatting regardless of level or profile, then discard the completed record when disabled. Alternatively, require and check non-mutating, total argument/formatting evaluation before allowing elision. The first rule is the simpler semantic baseline. Catching formatter panics alone is insufficient.

### B6. Destructors make deferred overflow writes observable

**Decision/section.** [01, D3 and D5](../design/01-design.md); [02, §1.3](../design/02-memory.md); [03, §2.4](../design/03-authority-concurrency.md); [04, §§3.2 and 4.3](../design/04-contracts.md).

**Problem.** The deferred-trap argument relies on writes into abandoned memory being unobservable. D5 deliberately runs destructors, and destructors may inspect their fields and write sinks. Verifying `Drop` without an invariant protects against invalid assumptions; it does not hide the fields' values.

**Scenario.** A local `Report { value: 0 }` prints its value on drop. The function computes `n + 1` as `u8`, then independently assigns `report.value = 1`, then exits. For `n == 255`, precise trapping prints zero during unwinding. Deferring the trap until function exit prints one. The intervening assignment's control flow and address do not depend on the overflowed value, so §3.2's stated restriction does not exclude it. The interpreter and optimized backend can consequently produce different traces.

**Smallest fix.** Use precise traps initially. Later defer only when writes and all cleanup reachable from them are proved unobservable, including destructors and borrowed caller storage. Flushing only before the eventual sink call is too late: unwinding already observes the changed field.

### B7. Panicking `par` regions cannot promise sequentially identical writes

**Decision/section.** [01, D9](../design/01-design.md); [03, PAR-1, PAR-2, SINK-4, §8.4](../design/03-authority-concurrency.md); [04, §4.3](../design/04-contracts.md).

**Problem.** Selecting the leftmost panic and buffering branch output does not undo writes made by speculative branches to its right. PAR-1 qualifies equivalence only for resource exhaustion and divergence, not ordinary panics.

**Scenario.** Reuse a local `Report { value: 0 }` whose destructor prints its value. Execute `par { panic!("left"), report.value = 1 }`. The disjoint mutable capture is permitted. If the right branch runs before the stop flag, unwinding the enclosing function prints one; if it stops first, it prints zero. Sequential left-to-right evaluation always prints zero. This output comes from the enclosing destructor, so discarding the right branch's sink buffer cannot repair it. The same seed can therefore have different observable `par` cleanup under real-core simulation.

**Smallest fix.** Initially execute potentially panicking regions sequentially, and parallelize only regions established panic-free without unproved assumptions. A later failure protocol must account for speculative external writes and cleanup before restoring the stronger claim. Merely choosing one error deterministically is insufficient.

### B8. Exported `const fn` bodies are missing from compilation dependencies

**Decision/section.** [01, D1, D3, D6, D12](../design/01-design.md); [04, §8.1](../design/04-contracts.md); original [round1-codex.md, N2](round1-codex.md).

**Problem.** Interfaces include public spec-function bodies but do not identify executable `const fn` bodies or their evaluated results as interface dependencies. Dependents read only interfaces, and body edits purportedly never re-check them. Compile-time values can determine consumer types and layouts even though the const function itself returns only a value.

**Scenario.** An imported `pub const fn capacity() -> usize` changes its body from returning eight to returning sixteen. A consumer's `[u8; capacity()]` changes type and layout. Its producer's signature remains identical. Ignoring the body edit leaves stale consumer facts; re-evaluating it requires dependency information the stated interface omits. The added spec-function dependency rule does not cover this ordinary const function.

**Smallest fix.** Include exported const bodies and their transitive compile-time dependencies, or consumed evaluated results, in interface hashing. Restrict the body-only no-recheck promise to runtime bodies that leave all consumed compile-time facts unchanged.

### B9. The revised deadline decision contradicts scope semantics and its worked test

**Decision/section.** [01, D8](../design/01-design.md), as revised during this review; [03, TASK-4 and §12.2](../design/03-authority-concurrency.md).

**Problem.** D8 now says a scope deadline cancels children but fails the scope only when a child returns cancellation as its error, allowing partial results. TASK-4 instead makes expiration itself a `DeadlineExceeded` failure, even when the body returns `Ok`. These require different results from the same program.

**Scenario.** The §12.2 fetcher wraps each network result in `Ok(net.get(...))`. When its deadline cancels the slow request, that child returns `Ok(Err(NetError::Cancelled))`; the body collects results through shielded joins. Under revised D8, cancellation was handled as a value and the scope may return partial results. Under TASK-4 and the published test, it must return `Err(DeadlineExceeded)`. The example's explicit test cannot serve as an oracle for both.

**Smallest fix.** Follow the owning decision's partial-results policy: distinguish deadline cancellation from an unhandled child error in TASK-4 and update the fetcher expectation. If an API requires deadline expiration to be an error regardless of handled child results, make that an explicit API behavior rather than retaining the opposite universal scope rule.

## Non-blocking

### N1. State the narrower authority and replay guarantees consistently

[01, Thesis and D7](../design/01-design.md) still says code touches the world only through capabilities, although [03, §2.4](../design/03-authority-concurrency.md) deliberately permits ambient output and even process exit on a broken output pipe. SINK-10's assertion that exfiltration needs `Net`, `Fs`, or `Proc` is false for `info!("{}", secret.expose())` sent to the configured sink. `Secret` prevents accidental formatting of the wrapper; it cannot prevent deliberate disclosure of the exposed value. Say “no ambient input authority, with reported diagnostic-output exceptions,” and qualify foreign guarantees by their reviewed claims. The optional effects gate is an explicit policy choice, not an unconditional protection.

The replay identity also needs the scheduler strategy, fault-plan contents, arguments, setup inputs, and source/dependency snapshot in addition to seed and toolchain. For example, changing `--strategy` or `--faults` in [03, §9.6](../design/03-authority-concurrency.md) changes the experiment without changing its seed or commit. Include those inputs in the replay artifact and command; a clean commit identifier alone does not identify a dirty working tree. This clarification can precede simulator implementation without changing the language core.

### N2. Reconcile the remaining chapter drift before using examples as acceptance tests

[02, §8.4](../design/02-memory.md) makes aliased locks panic and returns `Result<R, Poisoned>`; [01, D8](../design/01-design.md), [03, LOCK-2/LOCK-5](../design/03-authority-concurrency.md), and [04, §4.4](../design/04-contracts.md) require the recoverable `Aliased` error. [02, §7](../design/02-memory.md) forbids mutation of any field in a spanning invariant, whereas [04, §4.2](../design/04-contracts.md) permits a method whose declared set contains the complete footprint. Adopt the latter complete-footprint rule and the decision's `LockError` behavior, then remove stale amendment requests that describe already-applied changes. These have small editorial fixes; they should not become separate implementation interpretations.

### N3. Proof-quality checks need an unreachable-path distinction

[04, §10 item 3](../design/04-contracts.md) requires the assumptions at every obligation to be satisfiable and rejects proof from contradiction. Read literally, this rejects `if false { panic!(...) }`, although [04, §3.1](../design/04-contracts.md) requires proving panic sites unreachable. Impossible path conditions are legitimate; inconsistent imported axioms or entry assumptions are a different defect. Separate these cases, and report solver inability to establish satisfiability as unknown rather than as a concrete counterexample. This matters for verifier usability, without invalidating the repaired cross-mode trust rule.

### N4. Check deletion must cover implicit facts and indirect entry paths

[04, §2.6](../design/04-contracts.md) gives a useful conservative policy, but “every caller ... is proved” must mean its relevant obligations have empty trust closures, not merely the exported status `proved, assuming N` from §8.2. Enumerate function-value calls, callbacks, and exported ABI entries as callers. Include assumed type invariants from §4.2 among the facts that require proof or retained checks, rather than checking only syntactically written `requires`. D3 already demands this broader condition; make the implementation rule match it before enabling deletion. Keeping bounds deletion off is appropriate.

### N5. Specify progress for the thread pool

[03, §1.3](../design/03-authority-concurrency.md) says pool tasks stay on one OS thread for life, waits block that thread, children never run inline, and all runtimes differ only in cost. With a bounded one-worker pool, a child that spawns a grandchild and waits can occupy the worker its grandchild needs; [03, §6.3](../design/03-authority-concurrency.md)'s shielded join does not supply progress. If compensation threads are intended, require them; otherwise use one thread per task initially. This is a conditional starvation scenario because the pool's growth policy is unspecified.

### N6. Do not apply the collection-length model blindly to strings

[01, D1 and D3](../design/01-design.md) retains Rust's string meaning, while [04, §1.4](../design/04-contracts.md) models strings as `Seq<char>` and §1.6 illustrates an accessor contract with `result == self@.len()`. That equation cannot describe byte length for a string containing a multibyte character. Specify separate byte/character accessors and contracts, or use a UTF-8 byte-sequence view. This is a modeling ambiguity, rather than evidence that the illustrative equation was explicitly assigned to `str::len`.

## Ready to implement?

**No, not as a checker and interpreter implementing the design as a whole.** B1 gives conflicting non-suspension classifications; B4 gives opposite accept/reject rules; B2 and B5–B7 invalidate the claimed deterministic observable behavior; B3 leaves a foreign safety boundary open; B8 leaves compile-time dependencies incomplete; and B9 gives opposite scope results.

The ownership/origin subset is concrete enough for a restricted experiment: [02, §§2–7 and 10–11](../design/02-memory.md) provides rules, examples, and a Rust cross-check boundary. That is useful progress, but it does not establish the whole design's readiness. Resolve the blockers with conservative rules first: precise traps, sequential execution where panic-freedom is unestablished, explicit synchronization classification, and one failure/invariant policy. Native backend performance, evented lowering, and SMT implementation can then be staged without asking the first interpreter to invent language semantics.

Validation was limited to complete document reading, cross-document checks, and reasoning through the scenarios above. No compiler, runtime, solver, benchmark, or independent-agent behavior test was run; these findings establish specification defects and gaps, not implementation exploits. Only this review file was created.
