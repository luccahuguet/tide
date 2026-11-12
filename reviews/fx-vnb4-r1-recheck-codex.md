Model: gpt-6.1-sol · Effort: high · Date: 2026-11-13 (UTC)

Verdict: request_changes

Re-checked only round-1 findings 1 and 2 and the changes implementing their fixes. Candidate: `8c311d2aabb5f02ca6a1792234f52c1616cc7091`, base `c7c431b19d25e5a498c8d4d7bcd08012d8990280`. Reviewed `eb2aaba..8c311d2` in the complete candidate context. Candidate binary diff SHA-256: `a27518ed3b71934c3cc7265257247aa814dee9c3fbdbde645940bad617e3103b`. Confidence: high.

1. **Finding 1 — resolved.** `crates/tide-macros/src/lib.rs:167` now emits `match (#expr)`, so struct literals are valid scrutinees while evaluation remains once and match-based temporary lifetimes stay intact. `crates/tide-admit/tests/build.rs:790` adds the failing struct-literal case and verifies its returned field; lines 793–795 exercise a borrowed RefCell temporary. The author's passing production test on the identical candidate is applicable evidence. No issue introduced by this fix found.

2. **Finding 2 — unresolved, blocking: successful weak upgrades still hide a live strong-reference interval from the scheduler.** The original polling starvation is fixed: `ArcHandle` retains the world identity, `Context::of` validates it, and `Context::point` now schedules upgrade and the other named operations. The preserved seed-17 trace shows the runnable dropper actually selected at `weak.upgrade`. However, `crates/tide-std/src/sync.rs:132–134` schedules only BEFORE Rust increments the strong count. The caller can then immediately drop the acquired Arc; `task.rs:855–877` records cleanup without scheduling, AFTER the Rust pointer field has already dropped (`sync.rs:19`). No other task can run while this temporary extra strong reference exists.

   Concrete failure: one task owns the sole Arc and calls `Arc::make_mut`; a child holds only a Weak, upgrades it, does ordinary computation, then drops the acquired Arc. If the parent's mutation overlaps that acquired handle's lifetime, Rust must clone the payload. A legal custom Clone implementation returning `Value(self.0 + 1)` makes that result observable without a forbidden count observer. The same production-std function returned the cloned value in **16/16 real-runtime runs** and the original value in **16/16 simulated runs**. A shorter simulation-only version returned the original value for **all 256 seeds**. This is also a structural impossibility in the current simulator, not merely insufficient seeds: either the parent mutates before acquisition or the child completes acquisition AND release before handing off. The real-runtime arithmetic loops only widen the race window; they read no clock or performance measurement.

   This violates chapter 03 §9.3 (`design/03-authority-concurrency.md:2020–2026`), which promises synchronization interleavings cover modeled outcomes, and the GLOBAL-6 rationale (`:511–517`) explicitly includes timing-dependent Clone calls from `make_mut`. CPU-time exclusion in §9.8 does not exclude the reference-count/COW result. The new COW test (`crates/tide-admit/tests/build.rs:916–937`) keeps another strong clone alive throughout, so it cannot detect this missing transient interval.

   **Fix wanted:** add a scheduling boundary after a successful upgrade has acquired its strong handle, retaining that handle across the boundary, so another task can observe its live interval through `make_mut`. Preserve the current no-lock payload destruction and borrowed-payload drop-check behavior. Add a regression that demonstrates both the cloning and non-cloning outcomes under seeded simulation for this transient-upgrade program. Another operation/lifecycle model is acceptable if it covers the same interval without weakening those safety properties.

Other blocking issues introduced by the fixes: none found beyond this remaining part of finding 2. In particular, the non-generic metadata guard does not introduce a generic Drop implementation on Arc/Weak; actual Rust pointer/payload destruction occurs before its runtime-locking cleanup. Scheduling returns with the lock released before Rust operations and user Clone/Drop code run. The adapter uses the existing baton rather than a second scheduler. IDs are simulation-local creation-family IDs, retained across clone, upgrade and COW, and cleanup reports no last-owner/count result; I found no new count observer through those IDs. The documented domain checks, outliving cleanup and reentrant destructor coverage are appropriate. Stable replay of a restricted outcome set does not resolve the missing interleaving above.

Probes:

- Both batches ran sequentially through `(ulimit -c 0; CARGO_BUILD_JOBS=4 timeout <T> bash /home/lucca/.cache/eon-agents/eon-overseer/heavy.sh nice -n 19 env CARGO_BUILD_JOBS=4 python3 -)`, supplying the script by heredoc and setting every child process stdin to DEVNULL. Outer timeouts were 600 and 300 seconds; per-compilation timeouts 60 seconds and per-execution timeouts at most 60 seconds. The queue granted them at **2026-11-13 00:31:18Z** and **00:31:57Z**; both finished by **00:31:59Z**, before every stated cutoff.
- Linux, pinned nightly-2026-08-09. Reused the amended candidate's existing `target/debug/tide-admit` and `target/tmp/build/std/artifacts`, with provenance from the author's matching report; no rebuild claimed. `LD_LIBRARY_PATH` selected that toolchain's `lib`. All source/output directories were temporary and outside the checkout. Compilation used `tide-admit --crate-type=lib --test --emit=link --tide-std <artifacts>/libtide_std.rlib --tide-dep-dir <artifacts> --out-dir <case>/out <case>/package/src/demo.rs`; the real-runtime witness used `--crate-type=bin` without `--test`.
- **256-seed simulation probe:** built and ran successfully, exit 0. Each child acquired and immediately dropped a temporary strong handle while the parent called make_mut. Custom Clone incremented the payload. An assertion confirmed the outcome counts were `[256 original, 0 cloned]`.
- **Same-function real/simulated witness:** both compilations and executions exited 0. Function `run(&Tasks)` created `Arc::new(Value(0), tasks)` and its Weak, spawned a child that upgraded, computed a checksum and dropped the temporary strong handle, then mutated through make_mut on the parent and joined the child. It returned `make_mut`'s payload value. Real output was `REAL [0, 16]`; simulated seeds 0–15 produced `SIM [16, 0]`. The complete shared function is reproduced below; only the calling Tasks differed.

```rust
use std::sync::Arc;
use std::task::Tasks;
struct Value(u32);
impl Clone for Value {
    fn clone(&self) -> Self { Value(self.0 + 1) }
}
fn work(n: u64) -> u64 {
    (0..n).fold(0u64, |a, b| a.wrapping_add(b))
}
fn run(tasks: &Tasks) -> u32 {
    let mut a = Arc::new(Value(0), tasks);
    let weak = Arc::downgrade(&a, tasks);
    tasks.scope::<_, std::task::TaskError, _>(|s| {
        let child = s.spawn(move || {
            let held = weak.upgrade();
            let sum = work(2_000_000);
            let alive = held.is_some();
            drop(held);
            Ok((sum, alive))
        });
        assert_eq!(work(200_000), 19_999_900_000);
        let n = Arc::make_mut(&mut a).0;
        assert_eq!(child.join().unwrap().0, 1_999_999_000_000);
        Ok(n)
    }).unwrap()
}
```

- Read and checked the author's preserved seed-17 trace and the added polling/COW/domain/re-entry/drop-check fixtures. Their passing results establish the behaviors they exercise, not the missing transient-reference case.
- `git diff --check eb2aaba..8c311d2` passed; the working tree stayed clean. No repository, bead, staging or commit writes, and no held-out access.

Full workspace/conformance suites were not run; this was the requested focused re-check. No override. DONE
