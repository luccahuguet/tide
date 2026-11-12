Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: request_changes

Scope: `git diff dde6c57..a810843` (worktree `.worktrees/fx-iob`, HEAD `a810843`), read-only. Read
AGENTS.md, the bead (decisions 1-7, the 2026-11-09 plan-review and held-out comments), reference §8-9,
03 §2.3-2.4 (PANIC, SINK) and §8 (PAR), the research report, the Codex r1/r2 reports, and the
touched files with the code they call (`task.rs` Context/CURRENT, `test/runner.rs`, `contract.rs`,
`tide-admit/src/pure.rs` NEEDS). `trials/e1-checked/` was not opened or searched.

## What holds

- **Soundness.** `par.rs` has no `unsafe`. Every cross-thread capture is bounded correctly: `&mut [T]`
  halves need `T: Send`; `Mutex<Enumerate<Chunks*>>` and `&F` need `F: Sync`. The scoped helper is
  always joined before any `resume_unwind` (`par.rs:79`), so `thread::scope` never sees an unjoined
  panicked thread. `merge` makes every comparison while building `order` and does only `slice::swap`
  afterward (`par.rs:243-271`), so a comparator panic leaves the slice untouched. Each element is
  dropped exactly once by the slice's owner. The cycle-following permutation is correct, O(n). Budget
  tokens are RAII and return on spawn failure, since the closure that owns the token is dropped.
- **Panic semantics.** `fork` joins first, appends left records, and resumes left's panic. Otherwise it
  appends right's records and resumes right's panic. Surviving values drop during the winner's unwind
  (PANIC-2, Codex r1 fix). `blocks` claims in index order under the lock, so every block below the
  first panic was claimed and runs to completion. It sorts the outcomes by index and appends records
  through the winner, which gives PAR-2's lowest-index rule and SINK-4. Within one sort, the stop
  check uses `start` and is only an optimization: the order of fork results alone decides the winner.
  A right subtree that skips its merge (`par.rs:234`) is always overridden by the left panic at the
  lowest common fork.
- **Decisions 1-7.** Signatures match the stub verbatim plus `par_sort` (T: Ord + Pure + Send), with
  no `par_sort_by`. Owned scoped threads, no rayon. `available_parallelism()` is read once. Helpers
  set no CURRENT, have no Runtime registration and take no limit slot (`fork` uses `std::thread`
  only). Blocks are claimed in index order; one worker or `len <= 1` runs inline. In simulation,
  `is_simulating()` (`task.rs:721`) reads only CURRENT's task identity, which is `Some` exactly when a
  baton exists (`task.rs:672`, `:1316-1325`). That is the same predicate `record_sink` uses, so there
  is one owner. Nested regions and `Shared::with` need nothing extra (`parallel_regions.rs`).
- **Decision 4, Sol's reading: accepted.** No comparison sort makes the compared element pairs
  independent of the data. The decision's rationale was "survives toolchain bumps", which (b) meets:
  the tree is fixed by length, and the comparison sequence is a deterministic function of the input
  under an owned algorithm. Singleton leaves are the faithful choice. "Stable-sort leaves with
  `slice::sort`" would bring driftsort's comparison order back at the leaves.
- **`root.rs`.** With no capture active, `emit` takes the old path: `record_sink(stream, message,
  span)` with the same `"out"`/`"err"` identities and the `#[track_caller]` span from `out`/`err`,
  then the stdout write with the broken-pipe `exit(141)`, or the stderr write. Formatting still
  happens before `emit` (SINK-3), and nested captures restore the previous buffer. Behavior outside
  regions is unchanged.
- **Admission.** No `tide-admit/src` change. The existing `pure.rs` NEEDS keys
  (`tide_std::par::join`, `tide_std::par::ParSlice::par_*`, `pure.rs:231-238`) cover `par_sort`
  (FX0612). The prelude edit is the one `declare_prelude` line, and the stub's own line is removed.
- **Equivalence and docs.** The stub mounts production `par.rs` via `#[path]`, and the
  signature-only `par` is gone. `AtomicUsize::try_update` compiles on the stable oracle (rustc 1.96;
  probe 1). `research/23-parallel-regions.md` is byte-identical to the research report after its
  2-line header (probe 2).

## Findings

1. **Blocking. PAR-2's stop flag does not cross `join` or reach nested regions, so a region can
   diverge where the sequential program panics, and the docs don't label this partial.**
   PAR-2 (03 §8.2) says "A panicking branch sets a stop flag that branches to its right check at their
   forks and block boundaries". It allows divergence only when a right branch diverges *without
   reaching one*. `fork` has no stop flag (`par.rs:53-89`): after the left branch panics it waits for
   the helper to finish whatever it runs. The flags in `blocks` (`par.rs:161`) and `sort`
   (`par.rs:222`) are local to one call, so a nested region inside a right branch never sees an outer
   panic. Probe 3: `join(|| { sleep 50ms; panic!() }, || loop { [i, i+1].par_map(..); join(..); })`
   hangs (`timeout 10` exit 124). The right branch reaches a fork on every iteration, so PAR-2 says
   the region must report the panic. Without a free token, `fork` runs inline and the same program
   panics, so the outcome also depends on machine load. The 03 §8.4 status note
   (`design/03-authority-concurrency.md:1830-1832`), README (`README.md:277-281`), crate docs
   (`lib.rs:25-30`) and the `par.rs:3-6` header all present the stop behavior as implemented.
   Fix: the minimum is to label it partial in those four places and in `conformance/README.md`
   ("stop flags are per call: `join` and nested regions run a right branch to completion after a
   left panic, so a right branch that never terminates diverges"). Record the full mechanism as
   follow-up work for the planned pool on fx-djp; filing a new bead is the coordinator's call. That
   mechanism: an inherited stop chain checked at fork entry and each block claim, unwinding with a
   private sentinel that the left winner always overrides. Decision 5 names only the blocks' flag, so
   the label is the change I'd require now. The fix is text-only, so it needs no re-check under
   AP-PROOF-001.

2. **Non-blocking (fx-djp). Panic metadata from helper threads is lost, so non-simulated property
   tests can flip between discard and failure depending on scheduling.**
   The property runner identifies a `requires` discard by the hook-captured panic location plus the
   contract `VIOLATION`. Both are thread-locals of the case thread (`runner.rs:68-91`, `:234-262`;
   `contract.rs:59`, `:64-65`). When the violating call runs in a block or right branch on a helper,
   the helper's hook finds no CAPTURE slot and prints the panic, and its `VIOLATION` is never read. The
   case thread then sees `location = None` and `violation = None`, so the case counts as a failure. The
   same call on the calling thread is discarded. Which thread runs a block depends on timing.
   `same_kind` (`runner.rs:138-152`) compares the same fields, so shrinking also becomes
   schedule-dependent. Simulated tests are unaffected because regions run inline. `contract.rs`
   documents this gap for child tasks, but there it is deterministic. Found by inspection; no probe.
   Fix: carry `take_violation()` from a captured `Err` and restore it before the winning
   `resume_unwind` in `fork`/`blocks`. Have the runner fall back to the violation's own
   file:line:column when the hook location is missing. Meanwhile, add this to the partial list.

3. **Non-blocking (fx-djp). `par_sort` forks at every node down to singleton leaves and allocates two
   index vectors per merge. It is about 14x slower than the sequential std sort and churns OS
   threads.**
   `sort_range` calls `fork` for every internal node (`par.rs:229`). Whenever a helper finishes and
   returns its token, the next fork anywhere, even for a 2-element node, spawns an OS thread. Probe 4,
   1M random `u64` on 16 cores, `-O`: std `sort` 22 ms; the same tree with 1 worker 370 ms;
   `par_sort` 304 ms with 10,503 helper spawns. With 10k elements: 0.3 ms, 2.4 ms, and 5.7 ms with 251
   spawns. `par_map` spawned 15 helpers, as expected. A user's recursive `join` behaves the same way:
   a depth-20 tree took 52 ms with 1,020 spawns, against 2.9 ms sequential (probe 5).
   Fix: in `sort_range`, call `fork` only when `xs.len()` is at least a cutoff (for example 4096),
   else run both halves inline. The comparison sequence is unchanged, so this is unobservable and
   keeps decision 4. Optionally reuse one scratch buffer per sort instead of per-merge `Vec`s. For
   `join`, the partial label should say that every fork finding a free token spawns an OS thread, so
   fine-grained recursion is slow until the pool lands.

4. **Nit. The simulated panic cases don't test SINK-4.** `build.rs:698-717` asserts empty stdout
   under simulation, which holds whatever the sink order, because sim records go to the trace. Assert
   on `sim.records()` (`["0\n","1\n"]` for map/for, `["left\n"]` for join) instead, or in a
   `par::tests` case.

5. **Nit. `recursive_forks_share_a_bounded_budget` cannot fail on oversubscription.**
   `assert!(budget.free < budget.workers)` holds by construction. It would catch a double token
   return, not more than `workers-1` live helpers. A live-helper gauge with an asserted maximum
   (research test 7) would test the claim.

6. **Nit (bead record). Decision 4's wording.** The bead text "so the comparison sequence depends
   only on the input length" should read "is fixed by Tide's owned algorithm (tree by length, merge
   comparisons by the input), independent of the toolchain". Record this when noting the accepted
   reading.

## Probes

All ran 11:31-11:35Z with load 1.5-3.2 and 19 GB available. quiet-until (11:22Z) had expired and the
next booked start was 15:00Z. Each ran under `ulimit -c 0`, `nice -n 19`, `timeout` and
`</dev/null`. Sources are in the session scratchpad
`/tmp/claude-1000/-home-lucca-pjs-lab-tide--worktrees-fx-iob/61cb56d1-1d82-4bd7-8c55-cfb2c0d139c9/scratchpad/`.
The standalone probes include a copy of `par.rs` with its `//!` and `mod tests` lines stripped and one
added spawn counter after `let _token = token;` (the diff was checked). They shim `Pure` as a blanket
trait, `is_simulating` as `false`, and `__sink` as the same capture/append. There was no cargo build.

1. `rustc +stable --edition 2024 tu.rs` (`AtomicUsize::try_update`, rustc 1.96.0): compiles, prints
   `Ok(1)`.
2. `tail -n +3 research/23-parallel-regions.md | cmp - .../tide-research-par/report.md`: identical.
3. `PAR_BODY=par_count.rs rustc +stable -O probe2.rs; timeout 10 ./probe2`: the left panic message
   prints, then it hangs; exit 124 (finding 1).
4. `rustc +stable -O probe.rs; ./probe 10000; ./probe 1000000`: the timings in finding 3; all sort
   results equal std `sort`.
5. `rustc +stable -O probe3.rs; ./probe3`: the recursive join: 52.5 ms and 1,020 spawns against 2.9 ms
   sequential.

DONE
