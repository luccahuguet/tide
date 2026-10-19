# Ferox adversarial design review, round 1

## Verdict

**request_changes**

The proposed direction has value, but the accepted decisions do not yet support the pitch of Rust's guarantees, Go's build speed, and Dafny's proofs together. Several claims contradict other decisions. Others depend on trust boundaries that the design has not defined. I would authorize small experiments to resolve those boundaries; I would not choose Ferox over Rust for a production service on this evidence.

I read [the first principles](../design/00-first-principles.md) and [draft 1 of the design](../design/01-design.md) in full, and checked the relevant research reports. Review date: 2026-10-19; repository HEAD: `794d89da2308b78208c18f63c506c5d5753af30c`. The review concerns these proposed decisions, rather than the older specification or bootstrap implementation. Research reports supply evidence, not established Ferox results. Examples below are schematic where syntax remains unspecified. A **contradiction** follows from the written rules; a **conditional counterexample** identifies a combination the implementation must prohibit. Neither label alleges an existing compiler exploit. Adoption and performance predictions are marked as hypotheses.

## Blocking objections

### B1. Capability freedom does not establish mathematical purity

**Decision IDs:** D7, D8, D9, D10.

**Problem.** D7 defines `Pure` as containing no capability, then grants determinism, memoization, parallelism, and proof eligibility. Those uses require different conditions. D8 permits shared mutable cells without saying that they carry authority. D9 adds an interior-mutability restriction, but only for parallel closures; that restriction does not repair D7's other promises. The evidence cited for this step is stronger than Ferox's rule: Joe-E's purity result requires immutable argument types ([research/07](../research/07-effects-capabilities.md), F6, line 143), not just capability-free types.

**Concrete example.** Under D7, this function qualifies as pure:

```ferox
fn increment(x: &mut i32) -> i32 {
    *x += 1;
    *x
}
```

Memoizing its return value without replaying the mutation changes the program. Separately, `read(c: &SharedCell<i32>)`, implemented with D8's `lock`, observes other tasks' writes and can wait for a lock. Unless shared-cell handles count as capabilities, the same signature qualifies as deterministic and non-suspending. A hidden nested lock through a helper or trait method also defeats D8's deadlock argument if the compiler checks only lexical nesting. Erased callbacks and destructors require the same transitive accounting.

**Smallest fix.** Separate authority freedom from the eligibility rules for memoization, specifications, and parallel execution. Require immutable reachable inputs/captures for mathematical functions; model exclusive mutation as a state transition. Give synchronization handles explicit authority, and preserve the relevant properties through generic bounds, trait objects, callbacks, and destructors. Enforce the no-suspension/no-nested-lock restriction across calls. This can remain a small set of compiler-known properties; it need not become an open effect-row system.

### B2. The authority guarantee has executable escape routes

**Decision IDs:** D7, D12, D13.

**Problem.** D7 permits untracked `dbg` output and merely warns in release builds. This contradicts its own no-ambient-authority rule and first-principles hard constraint 5. D12 then admits ordinary Rust libraries, which do not obey Ferox's capability discipline. A package-level `ffi` grant records permission to cross a boundary; it does not describe the authority of each callable function or make that authority simulable. The design gives no default foreign-call classification.

**Concrete example.** A dependency can call `dbg(secret)` without receiving `Stdio`. The reviewer sees a capability-free parsing signature while the program prints the input. A Ferox wrapper `load(path: &str) -> Vec<u8>` around Rust filesystem code has the same problem if the importer accepts its Rust signature without adding authority. Classifying that wrapper from its parameters would also make a disk read eligible for pure execution and proofs. This second example is a **conditional counterexample**, because the foreign authority rules have not been written.

The reports already identify these boundaries: [research/07](../research/07-effects-capabilities.md), open questions 3–4, calls out logging and address-dependent nondeterminism; [research/09](../research/09-adoption-interop-backends.md), the Rust glue-mode proposal, describes type and callback transport without an authority model.

**Smallest fix.** Require a trace/stdio capability for executable debugging output. Default foreign calls to explicit broad foreign authority; allow narrower declarations only through a reviewed, counted trust boundary. A foreign-pure declaration must cover observable behavior, callbacks, and destruction. Simulation must reject or identify foreign operations without models. Qualify the signature guarantee by the resulting trusted boundary.

### B3. Task isolation does not make suspended Rust frames transferable

**Decision IDs:** D8, D12.

**Problem.** D8 combines stackful suspension, work stealing, and `Send`/`Share` checks at task boundaries. Rust interop introduces stack frames and thread-affine values that Ferox cannot discover from Rust signatures. A value can remain inside one task throughout its life while moving between OS threads. Making the task the isolation unit does not discharge OS-thread affinity requirements.

**Concrete example.** A Rust function acquires a `std::sync::MutexGuard`, invokes a Ferox callback, and releases the guard after the callback returns. The callback uses `Net` and suspends. If the runtime resumes the task on another worker, it also migrates the hidden Rust frame and its guard. Rust documents that the guard must release its mutex on the acquiring thread on pthread platforms. [Rust's `MutexGuard` documentation](https://doc.rust-lang.org/std/sync/struct.MutexGuard.html) establishes that requirement; [corosensei's `Send` documentation](https://docs.rs/corosensei/latest/corosensei/struct.Coroutine.html#send) explains why stack transfer needs constraints on the data live throughout the suspended stack.

This is a **conditional safety counterexample** if Ferox permits both suspending foreign callbacks and migration. [Research/10](../research/10-concurrency-parallelism.md), lines 234 and 296–310, proposes `ThreadBound` pinning and leaves its soundness open. D8 drops that safeguard. [Research/09](../research/09-adoption-interop-backends.md), lines 225–238, proposes callbacks across the Rust boundary.

**Smallest fix.** Make foreign callbacks non-suspending in v1, isolate unknown blocking calls on blocking workers, and define how imported thread-affine values restrict task migration. Any later support for suspension through foreign frames must pin those frames and define unwinding across the boundary. Test this boundary before promising that ordinary Rust libraries compose with every runtime choice.

### B4. Optional proof modes lack a sound rule for composing contracts

**Decision IDs:** D10, D11, D13.

**Problem.** D10 chooses verification per package. D11 exports contracts and makes bodies opaque to clients. Neither decision says which imported contracts a prover may assume, or how a proof records unverified dependencies. Counting explicit `assume` and `#[trusted]` sites cannot catch an implicit assumption introduced by importing an unchecked postcondition.

**Concrete example.** An `assert` package exports:

```ferox
pub fn answer() -> i32
    ensures result == 0
{
    1
}
```

A `prove` package returns `answer()` and claims its result is zero. If it uses the imported postcondition as a theorem, its proof succeeds while the release program returns one. Debug assertions would detect the library's defect when exercised, but release erasure removes that check. This is a **conditional counterexample** to the missing composition rule, not a finding about a verifier implementation.

The inverse boundary matters too: an unverified caller must not satisfy a verified function's safety precondition merely because a debug assertion exists. Erased contracts cannot justify unchecked memory access reachable through an otherwise safe API.

**Smallest fix.** Export proof status and transitive assumptions alongside contracts, bound to the implementation and toolchain identity. A proved client may rely only on discharged obligations or explicit trusted assumptions that appear in its verdict. Keep memory safety independent of erased, unproved preconditions. Preserve distinct `proved`, `refuted`, `unknown`/resource-limit, and `trusted` statuses; [research/06](../research/06-agent-native-tooling.md), line 430, already recommends most of this distinction.

### B5. The specification language cannot have the promised universal runtime fallback

**Decision IDs:** D10, D13.

**Problem.** D10 admits unbounded mathematical integers, sets, maps, and quantifiers, then says those specs compile to runtime checks under `assert`. A terminating predicate over one integer does not give a terminating procedure for checking that predicate over all natural numbers. Property tests cannot supply an equivalent fallback either. The research recommendation says executable specs “wherever possible” ([research/04](../research/04-verification.md), lines 194–198); the design removes that limitation. Dafny also distinguishes compilable checks from general specifications; its [reference manual](https://dafny.org/latest/DafnyRef/DafnyRef.html) describes runtime testing of the compilable portions of contracts.

**Concrete example.** `ensures forall |n: nat| predicate(n)` cannot in general execute by enumeration. Sampling finitely many `n` changes the assertion. Even the draft's bounded `max_index` example has a cost problem: executing a quantified invariant on every loop iteration can turn its linear scan into quadratic debug work. That matters when agents run the tests on every edit.

**Smallest fix.** Define an executable subset and proof-only clauses within the contract language. Reject non-executable clauses where an actual runtime check is required; otherwise report them as unchecked in that mode. Expose instrumentation cost or budgets. Distinguish failed proofs from solver resource limits, and state which obligations `ferox check` includes. Keep the sub-100-ms typing target separate from unrestricted proof work.

### B6. Reproducible reductions do not imply sequential equivalence

**Decision IDs:** D3, D9, D10.

**Problem.** D9 promises both results equal to sequential execution and fixed-tree floating-point reductions. A fixed tree makes the grouping reproducible across core counts. It does not make that grouping equivalent to a sequential left fold, or make an arbitrary reduction operator associative. Checked integer arithmetic also makes regrouping observable through traps. [Research/10](../research/10-concurrency-parallelism.md), F7, notes the associativity requirement in its discussion of Cilk, but the recommendation and D9 omit it.

**Concrete example.** For binary64 values `[1e16, 1.0, -1e16, 1.0]`, the left fold `((a+b)+c)+d` produces `1.0`; the balanced tree `(a+b)+(c+d)` produces `0.0`. I checked those results with Python binary64 arithmetic. Likewise, `(i64::MAX + 1) + (-1)` traps, whereas `i64::MAX + (1 + (-1))` does not. A sequential proof about the first grouping does not establish the second program's behavior.

**Smallest fix.** Specify a canonical tree as the semantics of the reduction, including its single-worker execution, and restrict proof transfer to that semantics. Alternatively require an associative operation under the actual arithmetic model. Define failure selection and termination qualifications for parallel calls that panic or diverge. Do not advertise equivalence to arbitrary existing sequential folds.

### B7. A trust-surface diff cannot replace review of behavior without adequate specifications

**Decision IDs:** D10, D13; design thesis and trust ladder.

**Problem.** The thesis says humans review the trust diff while machines check bodies. D10 requires no proof for ordinary code, and even a successful proof establishes only the stated contract. D13's unchanged signatures, authority, and spec hashes therefore do not establish that a body edit preserves intended behavior. This directly affects the proposed cost advantage over Rust: the design risks moving human work away from the place where an unchecked behavioral change remains possible.

**Concrete example.** A function `authorized(user: &User, object: &Object) -> bool` has no functional contract. Replacing its body with `true` changes no signature, authority, or trust-hole count. An unexercised authorization case can escape the tests. Even `ensures result == true || result == false` could be proved without establishing access control. Locking the original contract preserves its weakness.

This limitation is supported by [research/04](../research/04-verification.md), F4 and design implication 7: expert specifications contain mistakes, and the report recommends positive/negative spec tests and vacuity checks. Its headline 82/44/27 percentages concern a union across models on supplied specifications, not an end-to-end demonstration that agents produce adequate specs and trustworthy implementations.

**Smallest fix.** Make the trust diff an additional review view. Permit skipping body review only for identified obligations covered by independently reviewed specifications and valid proofs. Report the achieved trust level and unchecked behavior in the verdict. Retain independent behavioral examples and negative tests; add spec satisfiability/vacuity checks without claiming they establish completeness. The narrower claim is useful and honest.

### B8. Hashing an editable oracle does not establish approval

**Decision IDs:** D13, with D7 and D10 configuration.

**Problem.** D13 requires approval for edits to specs, protected tests, and the trust surface, but puts their hashes in a lockfile without identifying an independent approval authority. The agent can edit source text. Unless a separate gate owns the baseline, it can also edit the file that asserts what the approved source text is. Content hashes detect a difference from a trusted baseline; they cannot create that trust.

**Concrete example.** A hypothetical agent weakens `ensures`, removes a test's `#[protected]`, changes the package from `prove` to `assert`, and regenerates the lockfile. Comparing only the working tree with its regenerated hashes would show consistency. Counting explicit trust holes would miss the verification downgrade. This is a **threat-model gap**, not a demonstrated bypass of an implemented gate. The research's oracle-protection proposal ([research/06](../research/06-agent-native-tooling.md), lines 519–523) also needs an authority outside the editing agent to enforce approval.

**Smallest fix.** Define approval against an independently approved base revision or manifest, with approval credentials outside the editing agent's write authority. Include protection removal, proof-mode/configuration changes, trusted foreign declarations, and dependency grants in the protected comparison. `ferox check` can report pending approval locally; a separate trusted gate decides whether it exists. No elaborate signing infrastructure is necessary to define this boundary.

## Non-blocking issues

### N1. D2 removes lifetime notation before defining the remaining lifetime rules

One implicit region per view can be a sound restriction. It does not itself define variance, reborrowing, nested mutable references, source unions, closure escape, or arena reset. For example, `store(slot: &mut &str, value: &str)` needs a relation between the stored reference's validity and `value`; taking the shorter lifetime is not safe through an invariant mutable slot. The implementation may reject the operation, but the design must say how a valid version is expressed. A `move` closure can own a reference value while still borrowing its referent.

D2 acknowledges an open question about generic views, so I do not infer an inevitable soundness failure. The unsupported step is claiming that removing lifetime parameters has already removed the associated reasoning and error classes. [Research/02](../research/02-memory-models.md), open questions 1–5, explicitly leaves view containers, arena soundness, and coverage of real Rust programs unresolved. Define a conservative core with accept/reject examples before promising Rust-like ergonomics. An arena reset must invalidate handles and exclude live references; multi-cell `lock(a, b)` must also reject aliased identities before creating two mutable bindings. Neither boundary follows from ownership slogans alone.

### N2. D6 and D11 need exceptions to body-only invalidation

D11 says body edits never recheck dependents and lists only signatures, layouts, contracts, and authority in interfaces. D6 admits `const fn`. If `capacity()` changes its body from `8` to `16`, an importer declaring `[u8; capacity()]` must recompute its layout and any affected checks. Exported evaluators or their evaluated dependencies must participate in interface hashes. Narrow the no-recheck promise to runtime-only bodies that leave consumed compile-time facts unchanged. Apply the same dependency discipline to revealed proof bodies, generated source, and release instantiations; a body-only runtime edit need not force type checking, but it still changes emitted code.

### N3. D4 and D6 disagree about generic type inspection

D4 forbids inspecting concrete types; D6 gives generic library code a `Shape` containing fields, variants, names, and layout. These can coexist through an explicit reflection bound and witness interface, but the design does not specify one. Deserialization also needs a safe construction mechanism for invariant-bearing private types, not merely layout metadata. Default opt-out places the burden on every type author to anticipate reflective access.

Define read-only metadata separately from construction authority, require explicit consent for generic construction, and give reflection a definition-checked bound. Check one serializer across shared and specialized code generation before claiming both broad macro replacement and identical semantics. [Research/08](../research/08-type-system-core.md), §§4.2–4.3 and open questions 2–3, acknowledges these costs and unresolved behavior.

### N4. D5 and D8 need an operational failure contract

Level-triggered cancellation is cooperative. A child in `loop {}` never reaches a capability operation, so a cancelled scope cannot join it and propagate its sibling's failure. Deadlines need a stated scope: detecting expiry is different from guaranteeing completion. Destructors that perform cleanup also need rules when their capabilities already return cancellation. [Research/10](../research/10-concurrency-parallelism.md), lines 224 and 245, includes shielding and leaves preemption open; D8 omits those qualifications.

Task abandonment can also expose a partially updated shared invariant after releasing a lock. Specify poisoning, discard, or recovery obligations rather than suggesting supervision rolls back state. For ownership-consuming I/O, returning the buffer on cancellation must wait until the kernel no longer accesses it; returning a message does not imply that a remote side effect did not occur. These are concrete failure boundaries to specify, rather than reasons to abandon structured concurrency.

### N5. D11/D12's speed evidence does not establish the combined product claim

The 30-ms/100-ms targets are legitimate targets. [Research/03](../research/03-compile-speed.md), lines 171–185, labels its stage budgets as inferences assembled from different languages and workloads. Its Roc comparison changes compiler implementation, backend, incremental machinery, and source size; it does not isolate a 100-fold language-design effect. The report's suggested aborting development panics also differ from D5's task recovery. An interpreter can shorten test startup, but it does not demonstrate a fast native debug build with Rust dependencies.

Keep the targets; qualify the causal claims. Measure one end-to-end vertical slice including ownership, generic dictionaries, unwind behavior, and a representative Rust boundary. Report proof time separately. **Hypothesis:** until those costs are combined, the promised engineering simplicity and loop-speed advantage over Rust will shrink. The current evidence neither proves nor refutes that hypothesis.

### N6. D1/D13's agent advantage needs a controlled adoption test

Familiar syntax, documentation, and diagnostics are supported directions, but the design overstates some causal evidence. [Research/05](../research/05-llm-code-empirics.md), line 61, attributes the 10.7-to-73.7 repair change to prompt presentation and localization, rather than an experiment changing the compiler. [Research/04](../research/04-verification.md), lines 48 and 60–63, distinguishes model-union proof success from much weaker end-to-end specification/code/proof results. Those numbers are not an expected Ferox success rate.

**Hypothesis:** views, erased authority, proof modes, cancellation, and foreign-boundary rules may consume more agent context than lifetime syntax saves. Compare fresh agents on the same maintenance tasks, with Rust given equally good docs, feedback, and time budgets. Measure total time/tokens to independently tested behavior, human review time, unsafe/trusted escape use, and release performance. [Research/09](../research/09-adoption-interop-backends.md), adoption recommendations 1–4, already favors a narrow flagship library and measurable agent economics. Until that comparison wins, verified parsers/codecs shipped as Rust crates offer a more credible adoption case than replacing Rust for whole services.

### N7. D14 commits to an unavailable boundary in its own evidence

D14 names Bend 2 as the kernel partner before defining types, ownership, runtime interaction, or proof transport. [Research/01](../research/01-bend-hvm.md), F4, reports no native library target, no effect ABI promise, differing execution semantics, and limitations on numeric proofs. I did not independently recheck Bend's availability; this objection concerns the mismatch between the decision and its cited report.

Label the tandem an optional experiment with no v1 dependency. Do not propagate a kernel proof through an unspecified foreign translation boundary. A conventional trusted native kernel boundary is sufficient until a usable Bend interface exists.

### N8. D6/D7 leave the generator execution owner unclear

D6 says generators are build steps; D7 says there are no build scripts. If generators run automatically, they need declared inputs, bounded execution, capabilities, and generated-output identity in the snapshot. If they run manually and check in ordinary source, say so. Moving arbitrary execution from macros into a build-step label does not remove its supply-chain or reproducibility cost. The smallest v1 rule is explicit generation outside ordinary builds, with checked-in output.

## Strongest ideas

1. **D7: attenuated capability values as the ordinary dependency interface.** Passing a restricted filesystem, network, or clock value makes authority review and test substitution use the same API. Keep this idea after closing the ambient and foreign escape routes; it offers value without promising that capability absence proves functional purity.

2. **D13: canonical text with version-matched feedback and snapshot-bound edits.** Stable diagnostics, usable fixes, typed holes, installed-version documentation, and atomic writes address the agent loop directly. They also help humans and avoid requiring a separate canonical graph representation. Keep the stale-write boundary and the distinction between safe fixes and suggestions.

3. **D3/D4/D11: explicit interfaces and definition-checked abstraction with a power ceiling.** Local inference, bounded trait resolution, and compiled interface artifacts give the project a credible mechanism for predictable checks. Preserve the ceiling when a library asks for more power. Treat shared versus specialized generic code as an implementation choice with differential conformance checks, rather than a source-language distinction.

No compiler, proof, performance, or agent benchmark ran for this review. I checked the numerical reduction counterexample and the cited Rust stack/thread-affinity requirements; the remaining conclusions are document-level reasoning and explicitly identified evidence gaps.
