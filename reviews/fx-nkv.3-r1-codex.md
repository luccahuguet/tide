Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-09

## Verdict

**request_changes — high confidence (0.95).** The runtime direction is sound for this bounded slice, and the failure-selection and cancellation tests pass independently. Two changes are required: preserve inference for existing `scope(...)?` callers (B1), and stop routing a default-disabled debug record to the unconditional stderr sink (B2).

The smallest approval patch removes the unnecessary `From<Cancelled> for TaskError`, restores the existing callers as regression checks, and makes suppressed-outcome logging obey the existing default log level without adding a public logging subsystem. No runtime redesign, dependency, deadline, limit, shield, or isolation work is required.

## Scope

Reviewed frozen commit `102a3e22d891c7e4603b6f68c4b97803bd84a150` against `07e104b`. The candidate worktree was clean and at that commit when inspected. Read:

- The common brief, AGENTS.md, GOVERNANCE.md including the Merge Rule, README.md, both beads through `br --no-db show`, and `impl-report.md`.
- The complete nine-file diff; the changed task implementation and all 17 tests; production exports, capability construction, runtime entry and sinks; the shared I/O error implementation; the conformance std and relevant runner code; the two existing edited task fixtures and the new cancellation fixture; and the admission build-test file and helpers.
- Call sites and task/error references across `crates/` and `conformance/`, including admission's structural treatment of Tasks, Scope and Task.
- Design 01's tier table, D8 and D18; design 02 §8; design 03 §6, PANIC-2 and sink rules; reference §7, §6.5 and §12; and both fx-nkv.2 round-1 reviews, including Opus N2.

Independent compilation and execution used the private archive `/tmp/tide-fx-nkv3-r1-codex.WSo2hL`, created with `git -C /home/lucca/pjs/lab/tide archive 102a3e2 | tar -x -C <private-dir>`. Environment: Linux x86_64 and pinned rustc `nightly-2026-08-09`. Each compiler invocation ran sequentially under `nice -n 19`, with stdin closed, only after the brief's quiet-window predicate, load < 12 and available memory >= 6 GB passed. No Cargo build/test or full conformance run was performed.

The probe scaffold includes the candidate's unchanged `task.rs`, shared `io/shared.rs`, and exact production `__sink` implementation, with the capability declaration reproduced locally. It uses host Rust as its outer harness; it does not exercise admission, std injection or the production main macro.

**P1 — inference regression and minimal alternative.** From the archive:

```sh
nice -n 19 /home/lucca/.rustup/toolchains/nightly-2026-08-09-x86_64-unknown-linux-gnu/bin/rustc --edition=2024 --emit=metadata probe_inference.rs -o probe_inference.rmeta
nice -n 19 /home/lucca/.rustup/toolchains/nightly-2026-08-09-x86_64-unknown-linux-gnu/bin/rustc --edition=2024 --emit=metadata probe_inference_restored.rs -o probe_inference_restored.rmeta
```

The candidate failed with E0283; rustc explicitly identified the new conversion and identity `From<T> for T` as competing implementations. The second probe used the same caller and a temporary copy of task.rs with only `impl From<Cancelled> for TaskError` removed: exit 0. The candidate source was unchanged.

**P2 — release output and nested echoes.**

```sh
nice -n 19 /home/lucca/.rustup/toolchains/nightly-2026-08-09-x86_64-unknown-linux-gnu/bin/rustc --edition=2024 -Copt-level=3 -Cdebug-assertions=no -Coverflow-checks=on probe_runtime.rs -o probe_runtime
nice -n 19 timeout 10 ./probe_runtime
```

Both exited 0. Ordered assertions verified earliest-spawned independent-error selection over later errors and the body, an earlier child's nested cancellation echo losing to the independent cause, and context restoration. The executable printed `RUNTIME_PROBE_OK debug_assertions=false`; stderr nevertheless contained three unconditional `debug: task scope suppressed ...` records, one per scope with suppressed outcomes.

**P3 — candidate task tests through a single-file rustc harness.**

```sh
nice -n 19 timeout 45 /home/lucca/.rustup/toolchains/nightly-2026-08-09-x86_64-unknown-linux-gnu/bin/rustc --edition=2024 --test -Coverflow-checks=on probe_tasks.rs -o probe_tasks
nice -n 19 timeout 20 ./probe_tasks --test-threads=1
```

Compiler and executable exited 0: **17 passed, 0 failed**. This is independent direct-rustc evidence for the task module, not a repeat of the implementer's workspace/admission proof. Initial temporary harness drafts had an invalid include attribute and a borrowed non-Sync receiver capture; those harness errors were corrected before relying on P1/P2.

The implementer's 83 workspace tests and 198 PASS / 25 AWAITS / 0 FAIL corpus result remain attributed claims. A read-only `git diff --check 07e104b 102a3e2` also passed; I inadvertently ran that inspection in the source checkout rather than the archive. It wrote nothing. No repository/worktree or Beads mutation occurred. No held-out E1 checked task, reference solution or writer program was read.

## Blocking

### B1. The convenience conversion breaks existing scope error inference

**Location:** `crates/tide-std/src/task.rs:401`; symptom patched around at `conformance/cases/memory/scope_borrow_accept.rs:9` and `conformance/cases/tasks/task_lifecycle.rs:26`.

Adding `From<Cancelled> for TaskError` makes an existing program fail to compile even when it never uses cancellation:

```rust
fn existing_caller(tasks: &Tasks) -> Result<(), TaskError> {
    tasks.scope(|s| {
        s.spawn(|| Ok(())).join()?;
        Ok(())
    })?;
    Ok(())
}
```

P1 reproduces E0283 against the candidate and acceptance after removing only that conversion. The outer `?` no longer uniquely constrains the scope error type. Annotating the two repository fixtures repairs those files but leaves the public regression in every equivalent user program. Those fixtures previously protected this ordinary use of the reference's generic scope API.

This does not mean every reference example fails: the tail-returning `fetch_both` form in reference §7.1 gets its error type directly from the function return and remains constrained. Programs using the same documented API followed by more work and an outer `?` do regress.

**Smallest fix:** remove this optional conversion. Keep `checkpoint` returning the specified `Cancelled`; use an explicit `.map_err(|_| TaskError::Cancelled)?` where the new code needs a TaskError. Keep the independent `Cancelled -> io::Error` conversion. Revert the two unrelated fixture annotations so their existing unannotated calls remain regression checks through production and conformance. No scope-signature change or extra error wrapper is needed.

**If left:** adoption of cancellation silently breaks programs that do not call a checkpoint, and the edited acceptance fixtures conceal the regression.

### B2. Suppressed debug records bypass the specified default log filter

**Location:** `crates/tide-std/src/task.rs:227`, especially the direct `crate::__sink::err` call at `:229`; `crates/tide-admit/tests/build.rs:110` asserts this behavior in both profiles.

TASK-2 sends suppressed outcomes to one `debug!` record. Design 03 SINK-2 and reference §6.5 enable log records at **info and above** by default; debug records are discarded. `err` is a separate, unconditional sink. Prefixing an err record with `debug:` does not give it log filtering or log routing.

A scope with two independent child errors, or a child error plus a propagated body error, therefore writes unsolicited stderr under the default configuration, including release. P2 independently confirms this using the exact production sink. The integration test enshrines the same mismatch rather than detecting it. This changes observable output for ordinary failure handling, even when the caller deliberately handles the scope's result.

**Smallest fix:** keep the outcome accounting but treat this record as default-disabled debug logging. Until configurable logging exists, discard it rather than sending it to `err`; use the existing sink owner if an internal debug entry point is needed. Update the integration assertion to require empty default stderr in debug and release, and remove the README's promise of unconditional stderr summaries. Do not expand this bead into implementing `root.log`.

A `shortcut:` label does not authorize changing a v0 sink contract. This is a current v0 behavior issue, not a later-tier text finding for fx-djp.

**If left:** callers cannot obtain the reference's quiet default failure handling, and later logging configuration cannot control these records because they enter the wrong sink.

## Non-blocking

None. No finding confined to v1/Later text was identified for fx-djp. The recorded destructor-attribution limitation is addressed under Decision 6 rather than refiled as a new defect.

## Decisions

1. **Expose `Scope::cancel` — agree.** It is already specified in design 03 §6.1 and reference §7.1, and the bead allows it. The implementation only records the first cancellation cause and propagates to descendants. Explicit cancellation sets no failure-echo bit; all-Ok work stays Ok and an Err retains its own error. Keep it.

2. **Only checkpoint/yield are checkpoints in this slice; yield returns `Result<(), Cancelled>` — agree, except for the conversion in B1.** The capability table names yield as a cancellation point and no owning document fixes a conflicting signature. Checking before and after native yield is appropriate. Unlimited spawn and joins are expressly not checkpoints; supported in-memory I/O introduces no kernel wait requiring cancellation. Preserve those boundaries and the distinct Cancelled return type. Use explicit TaskError mapping rather than weakening existing scope inference.

3. **One Runtime transition owner and its mutex — agree.** Runtime owns cancellation causes, child ordinals, echo flags, completion categories and selection. The separate payload mutex stores typed errors and panic payloads; it does not select winners. Native scoped threads still own the lifetime/join invariant. This is coherent ownership, not two competing runtimes. Keep the non-generic metadata owner, passive typed payloads and scoped TLS guard. A mutex is sufficient for this unscheduled thread slice; no scheduler abstraction is needed here.

4. **Causal echo propagation — agree.** A nested checkpoint marks enclosing children only while every traversed scope carries the inherited first cause. An inner scope whose own failure or explicit cancellation came first stops that propagation. Panics remain panics even after an observation. Failed joins observe at the caller, rather than at the joined handle's owner; successful joins do not observe cancellation. This matches the detailed TASK-2 rule without inspecting E. Keep it.

5. **Unconditional stderr suppressed-outcome summary — disagree, blocking B2.** A count-only record is reasonable without imposing `E: Debug`; the transport is wrong. Choose the report's deferred-output option, with default-disabled debug semantics, until the existing sink owner supports log configuration. Do not add a public logging facility or an error-formatting bound for this bead.

6. **Preserve the bounded channel and attribute destructor panics to the executing task — agree within the recorded thread-runtime limitation.** Opus N2 and the bead explicitly accept that a discarded U can drop in the sending child or in the receiver's task, depending on whether it was buffered. A rendezvous channel would break forgotten-handle completion. The candidate now describes both paths accurately and preserves PANIC-2 when destruction panics during an unwind. Keep the channel. This is not a promise of timing-independent destructor attribution or replay, and fx-nkv.5 must account for it when scheduling result delivery and handle destruction. It does not justify changing the specified TASK-2 ordering among the child outcomes actually recorded.

## Reasoning

The runtime satisfies the central Acceptance in the inspected and exercised cases. Spawn assigns an ordinal while holding the metadata mutex; final selection scans ordinals for Panic before independent Error. Completion order therefore cannot displace an earlier independent failure merely because it finishes later. Echo classification uses observations rather than the shape of the returned E, so a cancellation echo carrying an unrelated-looking error cannot hide the real cause. Body panic takes precedence and triggers cancellation before native scope cleanup completes.

The cancellation transitions are linearized by one mutex. A new nested scope inherits the current cause under that mutex. Existing descendants receive cancellation in ascending scope-ID order, so a parent is processed before its children; a scope with a prior cause retains it. Failed-join observation occurs after the failed child's payload and metadata are recorded, because its sender is not dropped until after `finish`. A successful join deliberately does not mark an echo. First-cause races have a defined transition order; the code does not promise the same result for genuinely different real-thread event orders. For any recorded outcomes and observations, TASK-2's winner is selected by spawn order.

I found no user callback, error conversion, formatting implementation or typed-payload destruction inside the Runtime critical sections. The payload push releases its guard before acquiring Runtime; observation and cancel do not acquire payload storage. Final arbitration occurs after native joins. There is no examined lock cycle, and ordinary user panics are caught outside these locks, so they do not poison Runtime. Indexing and unwraps depend on private live-scope/child invariants; no safe public path examined violates them.

The four inherited unsafe sites remain bounded by native scope cleanup and invariant E. Only `Tasks::scope` constructs the erased pointer for a matching `Mutex<Vec<(usize, thread::Result<E>)>>` under `E: Send`. Jobs cannot outlive the native scope, and payloads are consumed only after joining. Adding the ordinal and the separate metadata Arc does not widen that lifetime erasure. The changed code introduces no additional unsafe site or unsupported public lifetime bound.

The tests use channels, failed joins and observed cancellation to establish ordering, rather than sleeps. They cover reverse completion order, panics outranking errors, body-panic priority, earlier echo versus later independent cause, successful joins under cancellation, explicit cancellation, repeated observations and nested first causes. P3 independently passes them. Their main deficiency is that they were made green around B1 and positively assert B2. Correcting the existing fixtures and output assertion supplies focused checks for both blockers.

Production/conformance equivalence is strong for the exposed task subset: both include the same task.rs, and the former duplicate Cancelled is removed. Remaining stub-only isolation/limit/Panicked items remain labelled future compile-time fixtures, not competing implementations of this slice. Shared I/O cancellation conversion uses the same ErrorKind in both builds. The new executable fixture crosses the production boundary according to the inspected build test, although I did not independently run that admission boundary.

Not checked: Cargo workspace suites, full conformance/admitted execution, Miri/loom or process-abort probes. The future rebase is unreviewed; destructor attribution retains the recorded timing dependence. DONE
