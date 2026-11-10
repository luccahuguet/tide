Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-09

## Verdict

**approve — high confidence (0.90).** No blocking finding and no patch required for approval of the recorded fx-nkv.2 lifecycle slice.

The change implements scoped OS-thread tasks, borrowed captures and results, automatic cleanup despite dropped or forgotten handles, and propagation of a single child panic or a body panic after children finish. It meets the bead's bounded Acceptance. It does not implement the full failure/cancellation contract, and the changed documentation states that limit.

## Scope

Reviewed candidate `f7f77a2ca49b59c6a31b2efb248588e0ba51674e` against `0583ecb`. Read:

- The review brief, AGENTS.md, GOVERNANCE.md including the Merge Rule, both beads through `br --no-db show`, and the implementer's report.
- All seven changed files, including all eight task unit tests; the production crate-root definitions and exports; the conformance std and its manifest; the admission build-test helpers and surrounding tests; and the relevant conformance runner code.
- Existing scope-borrow, escape, passed-scope, task-local-value, and Send/Sync cases; task callers and error-variant references in production and conformance code; the admission policy and its explicit treatment of Scope and Task in structural authority checking.
- Design 01's tier table and D18; design 02 §8; design 03 §6 and its nested-panic rule; reference §7.1 and §12.
- The pinned Rust implementation in `std/src/thread/scoped.rs` and `thread/lifecycle.rs`, particularly automatic joining and the packet destruction that decrements the running-thread count.

The candidate's seven-file fingerprint independently matches the implementation report: `269e79ab384f091b9486791f6d00d11d5c67f4d02c1b888f83e138afb48bba95`. `git diff --check 0583ecb f7f77a2` passed. The worktree matched the candidate on the reviewed files; its existing untracked `CODEX-REPORT.md` was left untouched.

Independent executable checks used an archive of the candidate in `/tmp/fx-nkv2-r1-codex.C6kDxP`, never the repository/worktree. Environment: Linux x86_64; pinned rustc `1.99.0-nightly (771916f90 2026-08-08)`. Each rustc invocation ran alone, under `nice -n 19`, after checking the prescribed quiet-window predicate.

**P1 — runtime and unwind probe.** `review_runtime.rs` includes the unchanged candidate `task.rs`, with the production `capability!` definition reproduced as its harness. Command, from the archive:

```sh
nice -n 19 timeout 45 /home/lucca/.rustup/toolchains/nightly-2026-08-09-x86_64-unknown-linux-gnu/bin/rustc --edition=2024 --test review_runtime.rs -o review_runtime
nice -n 19 timeout 20 ./review_runtime --test-threads=1
```

Result: compiler exit 0; **10 passed, 0 failed**. This runs the candidate's eight tests plus two supervisor probes: a borrowed child error whose destructor reads borrowed state and asserts a sibling has finished during body unwind; and a body panic combined with an already retained child panic and forgotten grandchild handles, asserting the body payload wins and the grandchild finishes. The first harness compilation failed with E0373 because my added grandchild closure borrowed its Scope local; changing that probe to move the Copy handle fixed the harness. No candidate code changed.

**P2 — compile-rejection probe.** `review_reject.rs` includes the unchanged candidate `task.rs` and the existing `scope_borrow_reject`, `scope_escape_reject`, and `scope_passed_reject` cases, changing only their import to the harness module. It adds four negative Task Send/Sync assertions, attempts to shorten Scope's invariant lifetimes and change its invariant error lifetime, and attempts to construct a scope with an Rc-bearing error. Command:

```sh
nice -n 19 timeout 45 /home/lucca/.rustup/toolchains/nightly-2026-08-09-x86_64-unknown-linux-gnu/bin/rustc --edition=2024 --crate-type=lib --emit=metadata --error-format=json review_reject.rs -o review_reject.rmeta
```

Result: expected compiler exit 1; a JSON-diagnostic comparison matched **all 17 marked primary line/code pairs**, with no missing or unexpected rejection. These are direct rustc probes, not admission-driver or full-conformance results.

No Cargo builds/tests or full conformance run were performed, as instructed. The implementer's reported eight unit-test passes, admitted debug/release execution, and 222-case conformance result remain attributed claims; I inspected their test boundaries and independently ran P1/P2. No held-out E1 checked task, reference solution, or writer program was read. No repository or Beads writes were made.

## Blocking

None.

## Non-blocking

None. No finding requiring only v1/Later-tier text changes was identified for fx-djp.

## Decisions

1. **Use native scoped threads — agree.** `crates/tide-std/src/task.rs:23` delegates the join/lifetime invariant to the specified standard-library owner. The pinned implementation catches a body unwind, waits for scoped jobs, then resumes the body payload. P1 exercises the corresponding Tide behavior. Keep this mechanism.

2. **Drop native handles and deliver results through a bounded channel — agree.** At `task.rs:74`, immediate native-handle destruction prevents callers from leaking the native thread packet that controls scope completion. The pinned packet source confirms that forgetting that handle can retain the running-thread count. A one-slot channel at `:91` lets the child's single send complete even if a Tide handle is forgotten. A leaked result is an ordinary consequence of forgetting its owning receiver. Keep this design; storing the native handle directly would fail the forgotten-handle contract. The disconnected-send result is dropped inside the child's catch boundary at `:113`, and the candidate tests its panicking destructor.

3. **Preserve the specified bounds through private erasure — agree.** The public scope/spawn/join bounds match the recorded signatures. Scope keeps unconditional Send/Sync; Task uses synchronized storage and a marker to obtain exactly the specified conditional Send/Sync. The unsafe lifetime/type argument is coherent for the reasons below. Adding `E: 'static`, `E: 'env`, or `E: 's` to the public methods would impose an unsupported restriction. Keep the current bounds and private erasure.

4. **Allow Task's U/E invariance — agree.** The removed covariance promise was explicitly a stub guess. The normative variance table does not require Task covariance, and production/conformance now share the actual definition. The added `E: Send + 's` requirement when a generic helper moves a Task into another child follows from that handle's representation and the closure's existing Send/outlives bounds; it does not widen the public spawn method's bounds. Bespoke unsafe handle impls solely to preserve the guess are unwarranted.

5. **Retain failures while deferring arbitration/cancellation — agree within this slice.** At `task.rs:119` and `:128`, the original error or panic is retained independently of its handle. A failed join therefore cannot erase the cause. The bead explicitly assigns multi-failure precedence and observable cancellation to fx-nkv.3, and `:32` marks the current selection shortcut. README and crate/API docs describe a partial lifecycle rather than complete TASK-2 behavior. Keep that boundary; do not treat this approval as approval of full failure arbitration.

6. **Expose only TaskError::Cancelled — agree.** The plan-review comment assigns DeadlineExceeded to fx-zc1; this slice has no deadline operation. No production/conformance caller still references the removed fixture variant. The public enum's implemented surface is shared verbatim and its limit is documented at `task.rs:151`. No replacement variant is needed here.

7. **Keep the admission policy unchanged — agree.** The new public items are Tide-owned, and no host thread, mutex, channel, or panic-catching operation is re-exported. Existing policy already recognizes Scope and Task as synchronization authority. The shared admitted fixture is an appropriate integration boundary. My direct probes do not independently repeat the reported admission execution, but inspection found no task-specific policy gap requiring widening.

## Reasoning

The unsafe surface consists of the two transfer impls for private `Failures`, the erased job call, and the pointer dereference (`task.rs:62`, `:64`, `:98`, `:127`). Only `Tasks::scope` constructs the pointer, and it does so for a mutex instantiated with the same E under `E: Send`. Scope's invariant E prevents a safe caller from reinterpreting that storage as another error type; P2 rejects the attempted lifetime substitution. Both scope lifetimes are invariant, and the higher-ranked body prevents a usable Scope or Task escaping. P2 independently verifies escapes and shorter borrowed captures are rejected.

The failure mutex is allocated before entering native `thread::scope`. Native joining finishes all accesses before normal return or body-unwind propagation can destroy it. The private Spawner trait erases the native environment lifetime without extending any reference or job lifetime: its reference and boxed jobs still carry the native `'s`. The erased run function and failure pointer cannot be supplied through a public constructor. No safe path examined lets the pointer survive native scope cleanup or be dereferenced with a different E. The mutex serializes children; consuming it after joining needs no concurrent access. Its push performs no user-defined destructor or callback while holding the lock.

The channel separates result ownership from join bookkeeping. Dropping or forgetting a Tide handle cannot detach the child or retain the native completion packet. Successful values can be received, destroyed with their receiver, or leaked by forgetting that receiver. Failed children retain their cause before sender destruction makes join return Cancelled. Closure and disconnected-result destruction occur within catch_unwind. Native joining also covers grandchildren spawned during body unwind. P1 covers these main lifecycle cases and the borrowed-error cleanup path; no borrowed-state lifetime failure was found.

The tests are proportionate to the bounded slice: they exercise normal borrowing, non-Send task-local values, dropped/forgotten handles, grandchildren, original panic payloads, body unwind, discarded-result destructor panic, retained errors, borrowed errors, and positive auto traits. Existing negative fixtures protect borrowing and transfer boundaries, and the new public-surface fixture is shared between conformance compilation and admitted production execution. Sharing the production task source removes the duplicate stub owner rather than attempting to maintain equivalent unsafe implementations.

Not checked: Cargo suites, full conformance/admission execution, Miri, or the future fx-ecb rebase. Cancellation and multi-failure arbitration remain deliberately unsupported; this approval covers the frozen lifecycle candidate only. DONE
