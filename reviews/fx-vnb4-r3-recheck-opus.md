Model: claude-opus-5-5 · Date: 2026-11-13

Verdict: approve

Scope: a scoped re-check of my round-2 findings 1 and 3, with findings 2 and 5 (text-only), on HEAD `5959b32`, one commit on `c7c431b`. I reviewed `git diff cb229d1..5959b32` in the context of the full files, and read the author's "Fix round 3" section. Findings 4 and 6 were deferred to fx-djp by the coordinator, so I did not re-check them.

## Findings

### 1. Per-access validation: resolved

**Evidence.**
- In `crates/tide-std/src/sync.rs`, `Deref`, `Clone` and every forwarded `PartialEq`/`PartialOrd`/`Ord`/`Hash`/`Debug`/`Display`/`Pointer` method on `Arc`, plus `Clone`/`Debug` on `Weak`, now use the private Rust fields directly.
- Both `inner()` helpers and `ArcHandle::validate()` are gone (`task.rs`).
- Domain validation and scheduling stay where an outcome can change, through `ArcHandle::begin()`:
  - `new` (`ArcHandle::new`);
  - `downgrade` (`tasks.context()` plus `begin`);
  - `make_mut`;
  - both phases of `upgrade` (`sync.rs:118-125`).
- In the real runtime, `ArcHandle::clone` clones `None` and a `usize`. A deref or clone of a real `Arc` therefore no longer touches the shared runtime `Arc`.

**Tests.**
- The unit test `arc_operations_validate_domains_but_payload_access_does_not` now checks that reads, clones and formatting work after the simulation ends and inside a foreign simulation, and that they append no trace events.
- It still checks that `make_mut`, `downgrade`, `upgrade` and `new` reject wrong domains.
- The build test `completed_simulation_results_can_be_read_and_compared` covers my probe B shape.

**Probes.** Rerun below:
- the `Arc` deref loop now matches the plain-reference loop, 0.014–0.016 s against 0.012–0.016 s, down from 12 s;
- both outlive tests pass.

### 2. Unsized payloads documented: resolved (text)

`crates/tide-std/src/lib.rs:230` now lists "unsized Arc/Weak payloads (`Arc<dyn Trait>`, `Arc<str>`, `Arc<[T]>`)" under "Not provided yet". Recording the gap on fx-djp or a follow-up bead stays with the coordinator.

### 3. Drop lifecycle events: resolved

**Evidence.**
- `ArcHandle` (`task.rs:801-805`) now holds only `simulation` and `id`.
- `end`, `strong()`, `close()` and `impl Drop for ArcHandle` are removed. Destruction is now plain field drop, with no runtime lock and no event.
- The `upgrade` acquisition and `downgrade` reuse the same metadata through `clone()`.
- The obsolete field-order drop-check comment is gone. Drop-check is now trivially Rust's: `RustArc<T>`'s `may_dangle` plus a non-generic field that has no `Drop`. The borrowed-payload fixture in `arc_and_weak_build_and_execute_against_production_std` still covers it.
- The `poll_trace` fixture no longer expects `arc.drop`/`weak.drop`.
- `Arc<T>` shrinks from 40 bytes to 24.

### 5. Review scaffolding: resolved (text)

`build.rs` no longer prints `TRANSIENT_OUTCOMES`, writes `outcomes.txt` or writes `trace-a/b.ndjson`, and `--show-output` is dropped. The 256-seed both-outcomes assertion and the two-process replay comparison remain, and the expected pass count is updated to 7.

## New blocking issues introduced by the fixes

None found.
- **Clones crossing domains.** With validation gone from `Clone`, a simulated `Arc` can be cloned outside its simulation. Safe code can move a handle out only after that simulation has ended, though: a real channel used inside it panics in `outside_simulation()`. So the extra count cannot affect any outcome in the originating world.
- **Runtime lifetime.** Each simulated handle still keeps the runtime alive through its `Option<Arc<Mutex<Runtime>>>`, as `Channel` and `SimCell` already do.
- **Unchanged parts.** No new scheduling point, scheduler or API was added. The `weak.upgrade_acquired` boundary, which Codex verified, is unchanged.

## Probes

One batch ran through `(ulimit -c 0; CARGO_BUILD_JOBS=4 timeout 1200 bash /home/lucca/.cache/eon-agents/eon-overseer/heavy.sh nice -n 19 bash <script> </dev/null)`. The slot was granted at 00:59:43Z and the batch finished at 00:59:46Z with exit 0. The load average at the end was 3.04.

**Provenance.**
- I reused the author's round-3 rebuild instead of rebuilding.
  - `target/debug/tide-admit`: mtime 00:56:09Z.
  - `target/tmp/build/std/artifacts/libtide_std.rlib`: mtime 00:56:12Z.
- Both are newer than every file under `crates/tide-std`, `crates/tide-macros` and `crates/tide-admit/src`. `find -newer` returned nothing; the latest source mtime is `sync.rs` at 00:56:08Z.
- `git status --porcelain` was empty at HEAD `5959b32`, and the author reports the binary diff hash unchanged across the amend.
- Toolchain: pinned nightly-2026-08-09, with `LD_LIBRARY_PATH` set to its `lib`.
- Sources and outputs were in my scratchpad, outside the checkout. Each build and run had its own timeout of 60–120 s.

**Probe A (deref cost).** The same `arcderef` and `plainderef` sources as round 2, built `--crate-type=bin --tide-profile=release` (exit 0) and run three times each. Both printed `39999960000000`.
- `arcderef`: 0.0136, 0.0159, 0.0162 s (round 2: 12.0–12.4 s).
- `plainderef`: 0.0145, 0.0163, 0.0120 s.

**Probe B (outlive).** The same source as round 2, built `--crate-type=lib --test --emit=link` (exit 0). Both `compare_results_across_runs` and `read_result_after_run` passed (2 passed, exit 0); in round 2 both panicked.

No repository, bead, stage or commit writes, and no access to `trials/e1-checked/`. I ran no full suites; they stay with the coordinator.

DONE
