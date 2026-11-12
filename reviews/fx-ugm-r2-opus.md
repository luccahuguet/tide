Model: claude-opus-5-5 · Date: 2026-11-11

# fx-ugm Opus review, round 2 (converge, read-only)

Verdict: approve_with_alternatives

Candidate: `315b5fe` on `a5552c3`; `git diff a5552c3 315b5fe | sha256sum` =
`e788eb93edcba1ffeb99e6b83ef724608d9776e72f755044ee098071cf952080` (verified). Worktree clean.
No builds or probes run: each doubt below was settled by the code and the existing passing tests.
Disclosure: a repository-wide `grep` for leftover "planned multi-cell" text printed two matching
lines from `trials/e1-checked/` (readiness.md:116, protocol.md:270). Both only name fx-ugm as a
prerequisite, which the bead body already says. I opened no file there, and nothing in this
review is based on those lines.

## What I checked and found sound

- **No deadlock, no stranded ticket.** In `acquire_set` (sync.rs:109-153), every operand's domain
  check, the sort, and the alias check run before `begin_set`'s point and before the first
  `fetch_add`. Each ticket is reserved in `SharedCell::acquire` (sync.rs:50-74), and the same
  thread then waits on that cell. So every reserved ticket belongs to a thread queued on that
  cell, holding only lower-id cells. Every wait therefore points to an earlier ticket on the same
  cell or to a higher cell, so no cycle can form. This is research §B1's argument, and the
  implementation matches it.
- **Every path advances every held ticket.** Between the first reservation and `release_set`, the
  real path can hit nothing that unwinds:
  - `record_set` is a thread-local read with no lock when the task is real.
  - The simulated path's `sim_lock` aborts instead of unwinding.
  - `push` aborts when out of memory.
  - The `f` panic is caught in `run_set`.
  - Poison stops early through the same `release_set`.
  - `Aliased` returns before any ticket.

  Cancellation is never observed inside acquisition: real waits are condvar waits, and the
  simulator's `point` does not check cancellation. The two scheduling points can still unwind
  (stall panic, sync.rs:131/169), but both lie outside the ticket window.
- **Atomicity.** `f` runs only after the whole set is held. Under `Root::sim`, the only scheduling
  points are `begin_set`/`end_set`; `scheduler.cells` writes trace records without scheduling, and
  each batch shares one step (scheduler.rs:118-133). fx-nkv.5 decision 3 (one point before
  reservation, one after release) and decision 4 (an uncontended simulated cell) still hold.
- **Decisions 1-6.**
  - Alias detection compares pointers after the id sort and comes before poison (LOCK-2).
  - `f` receives operands in the caller's order through a safe index re-sort.
  - Stop at the first poisoned cell: its ticket is consumed and everything held is released.
  - A panic poisons every held cell before release (LOCK-6).
  - `reset` drops the old payload after release. `reset_releases_before_dropping_the_old_payload`
    tells the two orders apart: dropping under the guard would re-poison the cell.
  - Mixing domains panics before any ticket (`set_domain_checks_precede_tickets_and_aliases`).
  - Real ids come from one static `AtomicU64` (Relaxed); simulated cells use fx-nkv.5's id.
  - `with_all(&[])` takes both points.
  - The stub's `unimplemented!` block is deleted, and LOCK-2/3/6 moved from deferred to covered.
- **Single owner.** `with`, `with2`, `with_all` and `reset` all go through
  `acquire_set`/`run_set`/`release_set`. fx-nkv.4's rebuttal (one unconditional
  advance/drop/notify step before branching) still holds: `release_set` (sync.rs:155-170) runs on
  every exit. Production and the conformance stub share `sync.rs` with no second runtime.
- **Shape.** I accept `Held` + `PairGuard` as the simplest safe shape for heterogeneous pairs.
  `State<T>` keeps control and payload under one `Mutex`, so a typed guard is needed. The
  alternatives would need erased pointers, boxing, or a second protocol.
- **Tests.**
  - The conformance case runs opposite-order `with2` and rotating `with_all` on real threads with
    a cancelled scope, and under plain `Root::sim` and an eight-seed `sim_test`.
  - The sync tests cover the X/Y lazy-ticket interleaving, the counters left unchanged by an
    aliased call, and deterministic ordered traces.
  - These are the falsifiers that research §3 names.
- **Research file.** `research/26-process-io-and-shared-sets.md` is byte-identical to the cited
  report, and the `#part-b--fx-ugm` anchor resolves.

## Findings

1. **Non-blocking. The one-cell `with` path now allocates on every call, where it used to allocate
   nothing.**
   - Evidence: every uncontended `with` now makes five heap allocations:
     - `ordered` (sync.rs:119);
     - `ids` (sync.rs:127);
     - `held` (sync.rs:132);
     - `cells` and `ids` in `release_set` (sync.rs:157-158).

     It also does four `Context::current()` thread-local reads, each cloning an `Arc` when the
     calling thread has a real scope context (`begin_set`, two `record_set`, `end_set`,
     task.rs:533-561). The base `with` made no allocation and one thread-local read.
   - The `ids` vectors feed only the simulator, so on the real runtime they are pure waste. The
     `cells` vector exists only so that cells can be notified after all guards are dropped.
   - For v0 this cost is acceptable: performance is not a contract, `Pure` closures are short, and
     correctness and single ownership matter more. It is still avoidable without a second
     protocol owner, and every `Shared` user pays it.
   - Fix I'd want (cheap; keeps one owner and fx-nkv.4's step):
     - (a) Look up the simulated context once per set and pass it through. Collect `ids` only when
       that context is a simulated task, for example by letting `SimCell::begin_set/record_set`
       take an iterator or closure.
     - (b) In `release_set`, advance `serving` on every guard, then drain `held`, dropping each
       guard and calling `notify_all` on its cell in turn. This removes the `cells`/`ids` vectors.
       Every cell is still advanced before any guard drops, and notified after its own guard
       drops, so the rebuttal's argument holds unchanged.
     - (c) Optional: a const-generic array path for `with`/`with2`/`reset` would remove the
       remaining two vectors.

     (a) and (b) are enough. If the coordinator prefers not to reopen the code, record this as a
     follow-up comment on the bead rather than a new bead.

2. **Nit. The simulated-points rule doesn't mention aliased calls.**
   - Evidence: `design/03-authority-concurrency.md:1575` says "Simulated calls take one point
     before the whole set and one after release, including empty sets". The rustdoc and README say
     nothing either way. In the code, an aliased call returns before `begin_set` (sync.rs:121-126,
     131), so it takes no scheduling point. Research §B3 and decision 5 leave this open, and the
     implementer's report (decision 4) chose this behaviour deliberately.
   - Fix: add one clause, such as "an aliased call returns before either point." Text only; no
     re-check needed.

3. **Nit. Simulated cells store a dummy id and `order_key` branches on every call.**
   - Evidence: `Shared::new` stores `id: 0` for simulated cells (sync.rs:200-205), and `order_key`
     re-derives the key from `SimCell::id()` on each comparison (sync.rs:37-39).
   - Fix (optional): store the key once at construction, as either the simulated id or the
     counter value, and make `order_key` return `self.id`. `SimCell::id()` can then go. Decision 2
     ("one private order_key()") is still met.

No blocking findings. Proof status: my approval covers code and text only. Landing still needs the
coordinator's rebased conformance run on `315b5fe` to pass; it was in progress when this round
started.

DONE
