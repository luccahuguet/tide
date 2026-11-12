Model: claude-opus-5-5 · Date: 2026-11-13

Verdict: request_changes

Candidate: worktree `.worktrees/fx-vnb4`, HEAD `cb229d1`, one commit on `c7c431b`. I reviewed `git diff c7c431b..cb229d1` in full: the touched files, the surrounding runtime (`Context`, `Channel`, mpsc's destructors, `Shared`), tide-admit's sink/surface/Pure owners, the corpus cases, reference §1, §2.1, §2.2, §6.4, §6.5 and §6.8, chapter 03 GLOBAL-6, SINK-1..8, §9.1, §9.3 and §9.8, and `conformance/rules.md`. I also read the author's report and the three Codex files.

Summary. Most of the change is sound:
- No count observer is reachable by method, path, `Deref`, `AsRef`, `Borrow`, conversion or trace event. The fields are private, `Deref` targets only the payload, and trace IDs are per-family creation IDs with no clone events.
- `Send`/`Sync` follow `RustArc<T>`, and Pure classification is the existing `NOT_PURE` entry.
- Admission goes through re-export ownership, with the `alloc::sync::*` denial unchanged.
- `dbg!` evaluates once, keeps the match-based temporary lifetimes, returns its argument or a tuple, and keeps FX0507 through `sink_expansion`'s `item_name` check.
- The simulation adapter now covers every interval with a modelled outcome that I could construct (argument under finding 1's context below).

One defect blocks: the per-access domain validation that the fix rounds added to `Deref` and the forwarded traits.

## Findings

### 1. Blocking: `Deref`, `Clone` and the forwarded traits validate on every access, which costs a contended atomic pair per deref and panics on values that outlive their simulation

**Evidence.**
- `crates/tide-std/src/sync.rs:22-25` and `:123-126`: `inner()` calls `ArcHandle::validate()`. `Deref` (`:61`), `Clone` (`:53`), and every forwarded `PartialEq`/`PartialOrd`/`Ord`/`Hash`/`Debug`/`Display`/`Pointer` method go through `inner()`.
- `crates/tide-std/src/task.rs:826-828`: `validate()` is `Context::of(&self.simulation)`.
  - For a real handle that runs `outside_simulation()` and then `Context::current()`.
  - For a simulated handle it runs `Context::simulated()`, which also calls `current()`.
- `task.rs:473-474`: `current()` is `CURRENT.with(|slot| slot.borrow().clone())`. That clones a `Context`, whose `runtime: Arc<Mutex<Runtime>>` is one allocation shared by every task of the runtime (`task.rs:1141-1153`, `:1728`). Every `*arc`, `arc[i]`, `arc == other`, `{:?}` and hash therefore does an atomic `fetch_add` and `fetch_sub` on one cache line that all tasks share. The compiler cannot hoist this out of a loop.

**Measured** (probe A below). Release build of production std, real runtime, 4 tasks, each summing a 1,000,000-element `Arc<Vec<u64>>` 20 times by index:
- through `&Arc<Vec<u64>>`: 12.28 s, 12.42 s, 12.00 s;
- the identical program through `&*arc` (one deref per pass): 0.029 s, 0.027 s, 0.028 s.

That is about 440× slower, roughly 150 ns per deref. Reference §1 says `Arc` keeps its Rust API apart from creation and count observers. Rust's `Arc` deref is a plain load, and sharing read-only data across tasks is the main reason anyone holds an `Arc`. An E1 writer who follows the reference gets a large hidden slowdown on Tide's most ordinary `Arc` use.

**Second effect.** A simulated `Arc` that outlives its `SimRoot` now panics on any read (probe B):
- `fn run(seed) -> Arc<Vec<u32>> { let sim = Root::sim(seed); Arc::new(vec![1,2,3], &sim.tasks) }`;
- then `assert_eq!(run(1), run(1))`, or `run(2).len()`;
- both panic with "simulation handle used outside its simulation" (`task.rs:513`).

Returning a run's result and comparing it across seeds is a normal determinism test. No rule in chapter 03 or the reference requires a read-only access to fail here. The sync.rs test at `:501` and `:509` pins this behaviour.

**Why validation is not needed there.** Deref, comparison, hashing and formatting observe only payload memory. They are not handle operations in §9.3's sense, and they cannot change any outcome another task can observe. Any synchronization handle inside the payload validates its own operations.

Clone and drop take no scheduling point by design: GLOBAL-6's cases are the last drop, `make_mut` and `upgrade`, and §9.3 covers them. The existing handle policy does not validate on clone either:
- reference §7.4 says "Cloning needs nothing";
- `Shared::clone` and `Shared::same` (`sync.rs:336-340`, `:433-435`) do not validate.

Every Arc/Weak operation that can change a modelled outcome already validates through `begin()`: `new`, `downgrade`, `make_mut` and both phases of `upgrade`.

**Fix wanted.**
1. Remove the `validate()` call from `inner()`, or use the fields directly, for `Deref`, `Clone` and the forwarded traits on both `Arc` and `Weak`. Keep validation in the operations that already go through `begin()`.
2. If the author keeps clone validation for symmetry with something I have missed, it must at least not clone `Context`. Compare `Arc::ptr_eq` against `CURRENT` under `borrow()`, and do only `outside_simulation()`'s borrow for real handles.
3. Update the sync.rs unit test (`:501`, `:509` and the clone cases) to match.
4. Add a check that an `Arc` returned from a finished simulation can still be read and compared.

The fix changes only std code in this change and no design tier.

### 2. Non-blocking: unsized payloads (`Arc<dyn Trait>`, `Arc<str>`, `Arc<[T]>`) cannot be created

**Evidence.** `sync.rs:19` is a plain tuple struct with no `CoerceUnsized`, and `new` is only for `T: Sized` (`:28-34`). Probe A's `dynarc` case, `let a: Arc<dyn Fn() -> u32 + Send + Sync> = Arc::new(|| 1, tasks);`, fails with E0308 "expected `Arc<dyn Fn() -> u32 + Send + Sync>`, found `Arc<{closure}>`".

The `?Sized` bounds on `Clone`, `Deref`, the trait impls and `Weak` are therefore unreachable. Reference §1 says `Arc` keeps its Rust API, and `Arc<dyn Trait + Send + Sync>` is common Rust. The bead scope ("matching the conformance stub's signatures") and the old stub had the same limit. `CoerceUnsized` is unstable, and the stub builds on stable rustc, so a fix needs a design choice outside this bead.

**Fix wanted.**
- In this change: one line in `crates/tide-std/src/lib.rs`'s "Not provided yet" list (around `:227`) naming unsized `Arc`/`Weak` payloads, so the documented contract is honest (AP-CONTRACT-001).
- For the coordinator: record the gap where the reference holder will see it (fx-djp or a follow-up) before E1 grading relies on `Arc`.

### 3. Non-blocking: Arc/Weak drop events model nothing, carry a std-internal span, and make every handle 40 bytes

**Evidence.**
- `task.rs:805` (`end: &'static str`), `:847-853` (`strong()`), `:856-880` (`close`) and `:882-886` (`Drop`). These exist only to record `arc.drop`/`weak.drop` events.
- No waiter depends on an `Arc` drop, so the event drives no scheduling decision. Unlike channel close, it has no semantic role.
- `Drop::drop` is not `#[track_caller]`, so every such event's span is `crates/tide-std/src/task.rs:884:20`, not the user's source. See `fix-r1-arc-seed17-a.ndjson` lines 18 and 24. §9.1 asks for source-located records.
- The fields make `ArcHandle` 32 bytes, so `Arc<T>` is 40 bytes against Rust's 8, in the real runtime too.

**Fix wanted.**
- Preferred: drop the lifecycle events, the `end` field, `strong()`, `close` and the `Drop` impl. That also removes the drop-check reasoning the field-order comment at `sync.rs:18` has to carry. Adjust the fixture's `"arc.drop"`/`"weak.drop"` assertions (`build.rs` `poll_trace`).
- Otherwise: shrink `end` to a `bool` and note in lib.rs that drop events carry no user span.

### 4. Nit: `dbg!`'s expression text differs from Rust's

**Evidence.** `crates/tide-macros/src/lib.rs:171` stringifies the re-emitted token stream. In probe A's `dbgspell` case:
- `dbg!(&v[1..])` prints `& v [1 ..] = ...`;
- `dbg!(Some(a).map(|x| x * 2))` prints `Some(a).map(| x | x * 2) = ...`;
- the author's own test saw `Value { n : 3 }`.

Rust prints `&v[1..]` and `|x| x * 2`. Rust documents dbg's format as unstable, so this is not a contract break. It is still visible and contradicts the report's "Rust-style expression spelling".

**Fix wanted.** Optional. For example, use per-token `Span::source_text()` (stable) with a fallback to `stringify!`.

### 5. Nit: review scaffolding in a permanent test

**Evidence.**
- `crates/tide-admit/tests/build.rs:943` prints `TRANSIENT_OUTCOMES`.
- `:1031-1036` greps it and writes `outcomes.txt`.
- `:1052-1054` writes `trace-a/b.ndjson`.

These produced evidence for this review. They are not part of the contract (AP-TEST-001: "delete ... scaffolding").

**Fix wanted.** Keep the two-process replay comparison and the both-outcomes assertion, and drop the prints and file writes.

### 6. Note, out of scope, no change asked: reference §6.8 "Order" omits `Weak::upgrade`

Reference §6.8 "Order" lists the scheduling points without `Weak::upgrade`, which 03 §9.3 names. It also omits the `Arc::new`/`downgrade`/`make_mut` points this change takes. They are consistent with `Shared::new`'s `CellNew` point and with §9.3's "every handle operation". The omission predates this change, and `design/reference.md` is single-holder. Suggest a comment on fx-djp.

### Checked without findings

- **Weak::upgrade's two points** (`sync.rs:133-141`). I worked through the transient cases: a clone from a borrowed `&Arc`, a temporary weak, an un-joined child's last drop, and make_mut's dissociation. After the `weak.upgrade_acquired` point, no single run segment can acquire and release a reference whose existence another task can observe:
  - `make_mut`'s clone needs strong count > 1, and only a transient upgrade can raise that from 1 within a segment;
  - a transient weak never causes a clone and can observe dissociation only through a later upgrade, which is a point.
- **Lock re-entry.** `begin()`'s `point` releases the runtime lock before Rust's `make_mut` runs user `Clone`. The payload drops before `ArcHandle::close` locks, and close runs no user code.
- **Drop-check.** The payload still drops under `RustArc`'s `may_dangle`, and the borrowed-payload fixture covers it.
- **Ended or foreign simulations.** Operations validate through `begin()`. Drops in ended or foreign simulations record nothing.
- **SINK-4/6 for the `Dbg` sink.** Real output goes to stderr under its lock, branches use the capture buffer, simulation uses `record_sink`, and line assembly treats `dbg` like `err` (`scheduler.rs:362`, `:372`, `:383`, `:389`).
- **`std::__sink::dbg` cannot bypass FX0507.** A direct call is rejected with FX0103 on `format_args!` in both debug and release (probe A).
- **Conformance stub.** Sharing production `sync.rs` and the `tide_macros::dbg` re-export keeps each corpus case's claim:
  - `global_arc_count` (E0599 / FX0520);
  - `global_arc_payload` (FX0606);
  - `cap_handle_tasks`;
  - both `sink_dbg` cases;
  - `lock_print`'s dbg FX0607 (Codex probe).

## Probes

All probes ran sequentially through `(ulimit -c 0; CARGO_BUILD_JOBS=4 timeout <T> bash /home/lucca/.cache/eon-agents/eon-overseer/heavy.sh nice -n 19 bash <script> </dev/null)`, with T = 1500 s for probe A and 400 s for probe B.

**Environment.**
- Linux x86_64, 16 CPUs, load average about 9.9 during probe A.
- Pinned nightly-2026-08-09, with `LD_LIBRARY_PATH` set to its `lib`.
- Reused the candidate's `target/debug/tide-admit` and `target/tmp/build/std/artifacts`. `libtide_std.rlib` has mtime 2026-11-13 00:36:00Z, after round 2's source change and before the 00:38Z amend, matching the author's and Codex r2's provenance. Nothing was rebuilt.
- Sources and outputs were under my scratchpad (`/tmp/claude-1000/-home-lucca-pjs-lab-tide--worktrees-fx-vnb4/560a386d-7467-4c9b-adf2-7830a8961449/scratchpad/probes/`), outside the checkout.
- Each driver run had a 120 s timeout.

**Probe A** (slot granted 00:46:58Z, finished 00:47:47Z, exit 0):
- `sinkdbg`: `std::__sink::dbg(format_args!(..), true)` in a library is rejected with FX0103 in both debug and `--tide-profile=release`, exit 1.
- `dynarc`: `Arc<dyn Fn() -> u32 + Send + Sync>` from `Arc::new(closure, tasks)` gives E0308, exit 1 (finding 2).
- `dbgspell`: builds and runs, exit 0. Its stderr gives finding 4's spellings; `a+b` prints `a + b`.
- `arcderef` vs `plainderef`: both built `--crate-type=bin --tide-profile=release` and were run three times each.
  - Same output, `39999960000000`.
  - Wall times 12.28/12.42/12.00 s vs 0.029/0.027/0.028 s (finding 1).

**Probe B** (slot granted 00:49:41Z, finished 00:49:41Z):
- `outlive`: built `--crate-type=lib --test --emit=link`, exit 0.
- Running the binary, both tests failed with "simulation handle used outside its simulation" (`task.rs:513`):
  - `assert_eq!(run(1), run(1))`;
  - `run(2).len()` (finding 1).

**Also:**
- Read the preserved `fix-r1-arc-seed17-a.ndjson` for drop-event spans (finding 3).
- No repository, bead, stage or commit writes; no access to `trials/e1-checked/`.
- Full workspace and conformance suites were not run; they stay with the coordinator.

DONE
