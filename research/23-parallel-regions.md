Source: `/home/lucca/.cache/eon-agents/tide-research-par/report.md`, 2026-11-10; copied 2026-11-12.

# Parallel regions for Tide (fx-iob)

Research report, 2026-11-10. Read-only: nothing in the repository or in `br` was changed, no cargo build ran, and nothing under `trials/e1-checked/` was opened. The brief's item 1 names fx-08m; I read that as fx-iob.

Evidence labels: **[design]** is Tide's text, **[code]** is the tree at `8804612`, **[web]** is a cited page (quotes were read from the page by fetch subagents unless marked *unverified*), **[inference]** is my reasoning.

## 0. What is already fixed

- **Surface.** `std::par::join` and slice methods `par_map`, `par_for`, `par_sort` (reference §8; 03 §8.1). `par_chunks_mut` is in 03 §8.1 but not in reference §8 or the bead's Scope; `par_reduce` is v1. Both stay out.
- **Bounds.** `join` closures `FnOnce + Pure + Send` (PAR-5); slice-method closures `Fn + Pure + Send + Sync` (PAR-6); `par_map` needs `T: Pure + Sync`, the others `T: Pure + Send` (PAR-4); `par_map<U: Send>` (reference §8).
- **Semantics.** With no panic, values, `&mut` writes and sink output equal sequential left-to-right evaluation "on any core count, runtime, or schedule, up to resource exhaustion" (PAR-1). With panics, the first in sequential order propagates, later branches' sink buffers are discarded, and a stop flag is checked "at their forks and block boundaries" (PAR-2).
- **Not tasks.** A branch "has no checkpoints and cannot be cancelled" (03 §8.1).
- **Simulator.** Regions "run sequentially, branches in sequential order" in every engine (03 §8.4, §9.1); a region "must never create OS threads" there (fx-nkv.5 comment, 2026-11-09).
- **Checker.** [code] `pure.rs` already keys `tide_std::par::join` and `tide_std::par::ParSlice::par_*` to `FX0610`/`FX0612`, and `walk.rs` owns `FX0611`. The module, trait and method names are therefore fixed, and no checker change is needed if the production items keep them.
- **Policy freedom.** "When no branch panics, scheduling is unobservable, so the policy can change freely" (03 §8.4).

## 1. Design choices

### 1.1 Signatures

**Recommendation.** Take the conformance stub's signatures verbatim and add `par_sort`:

```rust
pub fn join<A, B, RA: Send, RB: Send>(a: A, b: B) -> (RA, RB)
where A: FnOnce() -> RA + Pure + Send, B: FnOnce() -> RB + Pure + Send;

pub trait ParSlice<T> {
    fn par_map<U: Send>(&self, f: impl Fn(&T) -> U + Pure + Send + Sync) -> Vec<U> where T: Pure + Sync;
    fn par_for(&mut self, f: impl Fn(&mut T) + Pure + Send + Sync) where T: Pure + Send;
    fn par_sort(&mut self) where T: Ord + Pure + Send;
}
impl<T> ParSlice<T> for [T] { .. }
```

- **Extension trait.** Std cannot add inherent methods to `[T]`; the stub already uses a prelude trait and the owner's bead comment confirms it. One line in `declare_prelude`: `pub use crate::par::ParSlice;`.
- **`RA: Send, RB: Send`.** The stub marks these a GUESS; 03 is silent. Keep both. A result computed on another thread must be `Send`, and `rayon::join` requires the same ([rayon join](https://docs.rs/rayon/latest/rayon/fn.join.html)). Requiring both keeps the choice of which branch runs on a helper thread private.
- **`par_sort` shape (design silent).** 03 speaks of "`par_sort`'s comparator", but no document gives a signature. Options: (a) `par_sort(&mut self) where T: Ord`; (b) a comparator parameter; (c) both, as Rust's `sort`/`sort_by`. Recommend (a) alone: it mirrors `slice::sort`, which is what a Rust-trained writer types, and no document names a `par_sort_by`. The "comparator" is then `T::cmp`, authority-free because `T: Pure`. The `ParSlice::par_*` key already gives its element bound `FX0612`.
- **One source file.** Write `par.rs` so the conformance std includes it, as it includes `sync.rs`, and delete the stub's own `par` module (conformance README: the stub "includes Tide's std's definition … and drops its own"). That satisfies fx-nkv's equivalence acceptance without a second surface.

### 1.2 Owned scoped threads vs. rayon (AP-DEPENDENCY-001)

**Capability needed:** run borrowed closures on other cores and join them.

| | Owned (`std::thread::scope`) | rayon 1.12 |
| --- | --- | --- |
| Dependencies | none | `rayon`, `rayon-core`, `either`, `crossbeam-deque`, `crossbeam-epoch`, `crossbeam-utils` ([rayon](https://docs.rs/rayon/latest/rayon/fn.join.html), [rayon-core](https://docs.rs/crate/rayon-core/latest), [crossbeam-deque](https://docs.rs/crate/crossbeam-deque/latest)); MIT OR Apache-2.0 |
| PAR-2 panic choice | owned | `join` gives "the panic value from the first closure" ([rayon join](https://docs.rs/rayon/latest/rayon/fn.join.html)); for iterators the choice is not documented, so Tide would wrap every item in `catch_unwind` anyway |
| Sink buffers | owned | owned anyway, and harder: a waiting worker "will look for other work" ([rayon join](https://docs.rs/rayon/latest/rayon/fn.join.html)), so unrelated branches nest on one thread |
| `par_sort` structure | owned | `par_sort` is a timsort-like merge sort over parallel chunks ([ParallelSliceMut](https://docs.rs/rayon/latest/rayon/slice/trait.ParallelSliceMut.html)); its comparator panic order is undocumented, so D9's structure would be owned anyway |
| Simulator | no threads when the switch is set | the same; the pool is bypassed |
| Scheduling quality | thread per fork, coarse balancing | work stealing, low fork cost, matches 03 §8.4's pool |
| Global state | one atomic budget | a process-global pool; `build_global` errors on a second call ([ThreadPoolBuilder](https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html)) and it reads `RAYON_NUM_THREADS` from the environment, an ambient input Tide's std otherwise never reads |

**Recommendation: owned.** [inference] Rayon would supply only the scheduler; every observable rule (PAR-1 sink order, PAR-2 panic choice, the stop flag, the sort structure, the sequential switch) is Tide's to implement either way. Std's only external dependency today is `libm`, pinned exactly; five more crates of lock-free unsafe code would link into every Tide program for a performance property that no acceptance check measures. `thread::scope` is already the mechanism `task.rs` uses. AP-MINIMAL-001's order puts the standard library before a dependency.

**What this defers.** 03 §8.4 says branches "run on one process-wide work-stealing pool" with lazy binary splitting ([Tzannes et al., PPoPP 2010](https://researchr.org/publication/TzannesCBV10)). The owned version is not that pool. Because scheduling is unobservable, swapping it in later changes no API or result. The std crate docs should label the v0 scheduler as partial (AP-CONTRACT-001), and E5 is the design's own trigger for scheduler work. Replacement cost: one private function (§1.3's `fork`).

### 1.3 Thread count, budget and chunking

**Thread count.** 03 §1.3 gives `par-threads = "cores"` in the manifest; [code] nothing reads a manifest yet, so the key stays planned. Use `std::thread::available_parallelism()`, read once into a `OnceLock`: the docs say the value "is not cached" and "should not be called from hot code", and that it "may overcount … when limited by a process-wide affinity mask or cgroup quotas" ([available_parallelism](https://doc.rust-lang.org/std/thread/fn.available_parallelism.html)). Overcounting costs speed, never results.

**Process-wide helper budget (the oversubscription control).** One `AtomicUsize` holds `threads − 1` helper tokens. A fork tries to take a token; with one it starts a helper thread, without one it runs the work inline, left to right. PAR-1 makes the inline fallback always correct. Consequences:

- recursive `join` (divide and conquer) cannot spawn an exponential number of threads;
- many tasks calling `par_map` at once create at most `threads − 1` helpers in total, which is the purpose of 03's "one process-wide pool";
- a token returns when its helper ends, so later forks pick it up.

Start helpers with `thread::Builder::spawn_scoped`, and treat `Err` as "no token": `thread::scope`'s plain `spawn` panics when the OS refuses a thread, and PAR-1 exempts resource exhaustion only from equality, not from good behaviour.

**`join`.** Run the left closure on the calling thread, the right on a helper (Cilk's and rayon's order). With no token, call left then right.

**`par_map` / `par_for`: blocks claimed in index order.** Options:

- (a) One contiguous chunk per worker, as OpenMP `schedule(static)` does; its thread assignment is reproducible only under listed conditions and programs must not depend on it ([OpenMP 5.0 §2.9.2](https://www.openmp.org/spec-html/5.0/openmpsu41.html)). Simple, but skewed work leaves cores idle.
- (b) Cut the slice into blocks (about `len / (8 × workers)`, at least 1) and let workers claim the next block from a `Mutex` around the `chunks`/`chunks_mut` iterator.

Recommend (b). It is safe code throughout: `par_for` hands out disjoint `&mut` blocks from the iterator; `par_map` collects `(block index, Vec<U>)` and concatenates in index order. It balances skewed work, and it gives PAR-2 for free (§1.5), with "block boundaries" being the design's own phrase. The block size is unobservable and may change.

**Small inputs.** A helper thread costs tens of microseconds on Linux ([clone benchmark, ~5 µs launch](https://eli.thegreenplace.net/2018/launching-linux-threads-and-processes-with-clone/); pooled handoff is several times cheaper per [a Java measurement](https://essential-books.onrender.com/essential/java/pitfall-thread-creation-is-relatively-expensive-568c276968024fdeb8a6e84247fd169b.html)). Go inline for `len ≤ 1` or one worker, and add no element-count threshold: two elements can be two expensive parses, and std cannot see closure cost. Rayon exposes `with_min_len` for the same problem ([IndexedParallelIterator](https://docs.rs/rayon/latest/rayon/iter/trait.IndexedParallelIterator.html)); Tide's design gives granularity to the runtime (D9), so no knob. Record the overhead as a known v0 limit.

### 1.4 Result order and determinism

**One code path.** [inference] The strongest way to meet PAR-1 is to write each operation once over a private `fork` primitive and let the mode decide only whether `fork` starts a thread. "Sequential" is then not a second implementation that could drift.

- **`par_map`** returns results in index order: rayon promises parallel iterators match their sequential counterparts except for side-effect order ([rayon README](https://github.com/rayon-rs/rayon)); Tide closes that exception with sink buffers (§1.8).
- **No reductions in v0.** Rayon's `reduce` leaves the combining order unspecified and needs associativity *(summary of [ParallelIterator](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html), not read verbatim)*; TBB had to add `parallel_deterministic_reduce`, which "executes the same set of split and join operations no matter how many threads participate" and still "might differ from … an equivalent sequential" result ([oneTBB spec](https://oneapi-spec.uxlfoundation.org/specifications/oneapi/latest/elements/onetbb/source/algorithms/functions/parallel_deterministic_reduce_func)). That is 03 §8.3's v1 design; do not let a helper `par_sum` slip in.
- **`par_sort` is stable** (reference §8, D9), and for a lawful `Ord` a stable sort has exactly one result, so its structure is unobservable without panics. For an unlawful `Ord`, Rust says the sort "may panic; even if the function exits normally, the resulting order … is unspecified", while "all original elements will remain in the slice" ([alloc slice.rs](https://raw.githubusercontent.com/rust-lang/rust/master/library/alloc/src/slice.rs)). So the result then depends on the algorithm, and the simulator path must run the same split tree as the threaded path. This is the main reason for the one-code-path rule.

**`par_sort` structure.** D9: "splits and merges depend only on the input length". Recommend: split at `len / 2` down to a fixed leaf size, stable-sort leaves with `slice::sort`, `join` the halves, merge at each node on the node's own `&mut [T]`. Halves are disjoint `&mut` splits, which is why `T: Send` suffices. For the merge:

- (a) Call `slice::sort` on the node's range. The two halves are sorted runs, and std's stable sort is driftsort ([alloc slice.rs](https://raw.githubusercontent.com/rust-lang/rust/master/library/alloc/src/slice.rs)), a run-detecting merge sort *(that it merges two presorted halves in linear time is unverified; §3 test 9 checks it)*. About ten lines, no `unsafe`.
- (b) An owned two-finger merge: compute the merged order by comparisons into an index vector, then apply the permutation with `slice::swap`. No `unsafe`, about forty lines, and a comparator panic in the comparison phase leaves the slice untouched.
- (c) A pointer-copying merge with a drop guard: fastest, needs `unsafe` and a panic-safety argument.

Recommend (a), falling back to (b) if test 9 fails. **Ambiguity for the coordinator:** under (a) which ranges are merged depends only on length, but the comparison sequence inside a merge is the pinned toolchain's, so a toolchain bump could change which comparator panic comes first for an unlawful or panicking `Ord`. If D9's sentence is read to fix the comparison sequence across Tide versions, take (b). Merges stay sequential in v0; the root merge bounds speedup, which no acceptance check measures.

### 1.5 Panics: which one wins, and how it propagates

**Prior art.** Systems disagree, which is why Tide must own the rule:

- `rayon::join`: "If both closures panic, `join()` will panic with the panic value from the first closure", and "both closures will always be executed" ([rayon join](https://docs.rs/rayon/latest/rayon/fn.join.html)).
- Java `ForkJoinTask.invokeAll`: "If more than one task encounters an exception, then this method throws any one of these exceptions" ([JDK 21](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/concurrent/ForkJoinTask.html)).
- .NET's guidance wraps all loop exceptions in an `AggregateException` ([Microsoft Learn](https://learn.microsoft.com/en-us/dotnet/standard/parallel-programming/how-to-handle-exceptions-in-parallel-loops)).
- JCilk aborts side computations so a single exception is handled *(search summary of [Danaher et al.](https://dspace.mit.edu/handle/1721.1/30275), not read in full)*.
- `thread::scope` alone loses the payload: "If any of the automatically joined threads panicked, this function will panic"; to handle panics, "join them before the end of the scope" ([thread::scope](https://doc.rust-lang.org/std/thread/fn.scope.html)).

**Recommendation.**

- Wrap every branch or block in `catch_unwind(AssertUnwindSafe(..))`, keep the payload with its index, and after the scope `resume_unwind` the lowest-index payload. `resume_unwind` "triggers a panic without invoking the panic hook" ([resume_unwind](https://doc.rust-lang.org/std/panic/fn.resume_unwind.html)), so the winner is reported once.
- **Stop flag.** An atomic "stopped" flag set by any panic. Workers check it before claiming a block. Since blocks are claimed in index order, every unclaimed block lies to the right of the panic, so skipping them is exactly PAR-2; blocks already running finish, which gives "a later branch's panic waits for every branch to its left". Within a block elements run in order, so its first panic is its sequential first. For `join`, a right branch that has not started when the left panicked need not run; rayon's "always executed" is not Tide's rule (PAR-1: right "may or may not have written").
- **Inline fallback** stops at the first panic like the sequential program.
- **Partial results** of `par_map` are dropped; only destructors see them (PAR-1).

**Gap to record: the panic hook.** The default hook "prints a message to standard error" when a thread panics, before unwinding ([set_hook](https://doc.rust-lang.org/std/panic/fn.set_hook.html)). So every panicking branch prints at once, losers included, in thread order, and from an unnamed helper thread. PAR-2 says the first "is reported". Options: (a) accept it; (b) name each helper after the calling thread, so the winner's message matches the sequential one; (c) install a hook that writes the message into the branch's sink buffer, so losers' messages are discarded and order is sequential. Recommend (b) now and record the rest as partial: [code] std installs no hook today, `task.rs` has the same exposure for TASK-2's suppressed child panics, and a process-wide hook belongs to the runtime entry and the log-sink work, not to this bead. Tests must not assert stderr bytes when several branches panic.

**Divergence.** A branch right of a panic that never returns makes the region hang where the sequential program would panic; PAR-2 allows it. Do not add timeouts.

### 1.6 Cancellation and task limits (fx-zc1)

The design is clear on cancellation and silent on limits.

- **Cancellation.** A branch has no checkpoints, so a cancelled scope waits for the region; the task sees `Cancelled` at its next checkpoint. Do not wire the scope's cancel into the stop flag: an early stop would return a result the sequential program never produces. Deadlines are already "soft" (reference §7.2).
- **Limits (silent; recommend).** `Tasks::limit(n)` bounds children. A helper thread is not a child: it must not register in `Runtime`, must not set `CURRENT`, takes no limit slot, and never appears in the 03 §9.3 deadlock report. The separate budget is `par-threads` (03 §1.3). Total OS threads are then at most live tasks plus `threads − 1`.
- **Why helpers need no task context.** Everything a branch can reach is `Pure`, so it cannot hold `Tasks`, a `Shared` cell or a channel end (PAR-4); nothing it runs reads `CURRENT`.

### 1.7 Under the simulator

**Prior art.** Deterministic simulators do not let uncontrolled threads run. madsim: "Spawning system thread is not allowed by default because it may cause non-determinism", enforced by overriding `pthread_attr_init` ([runtime](https://raw.githubusercontent.com/madsim-rs/madsim/main/madsim/src/sim/runtime/mod.rs), [task](https://raw.githubusercontent.com/madsim-rs/madsim/main/madsim/src/sim/task/mod.rs)). Shuttle works "by controlling the scheduling of each thread", and requires that "all uses of synchronization primitives from `std` must be replaced by their Shuttle equivalents" ([shuttle](https://docs.rs/shuttle/latest/shuttle/)). A thread pool fits neither. Tide's options were to run branches as scheduled tasks under the baton, or inline.

**The design chose inline**, and it is the right fit with fx-nkv.5's decisions: a branch is not a task, so it has no task id and no point; fx-nkv.5 decision 3's point list does not include regions; and a point "with one runnable task draws nothing". Running branches as baton tasks would add seed draws that depend on a block size the design calls unobservable.

**Recommendation.**

- **Behaviour.** In sequential mode `fork` calls its work in order on the calling task's thread. No thread, no token, no buffer: records reach the task's sink in program order, and a panic stops the region as in the sequential program, so "a seed replays exactly even when a branch panics" (03 §8.4).
- **Trace.** Record nothing for the region itself. Its sink records appear as ordinary `sink` events of the calling task (fx-nkv.6 decision 6). No `par` event kind: fx-nkv.6 decision 1 lists none, and 03 §9.1 requires one trace across engines.
- **The switch (owner comment, 2026-11-09: "one runtime-mode switch the scheduler can set").** Options: (a) a process-global flag; (b) a thread-local flag; (c) derive it from `CURRENT`'s runtime having a simulation. Reject (a): fx-nkv.6 decision 3 runs `Root::sim` in-process in plain `#[test]`s, which libtest runs on concurrent threads, so one test ending its simulation would clear the flag under another and let it spawn threads. Option (c) has the cleanest ownership but needs fx-nkv.5's field, and that bead is rewriting `task.rs` now. Recommend (b): a `thread_local!` `Cell<bool>` in `par.rs` behind one `pub(crate)` setter that returns the old value. fx-nkv.5 sets it wherever it enters a simulated task's thread (its `Enter`), with a test; once both have landed, the getter can become a read of the simulation identity so the fact has one owner (AP-OWNERSHIP-001). Helpers never inherit it, because none start while it is set.
- **Test here.** Unit-test the sequential path through the setter in `par.rs`, as the owner's comment asks; a Tide program cannot reach a `pub(crate)` item, so this is a Rust-level test.

### 1.8 Sink output (SINK-4)

[code] `__sink::out`/`err` write straight to the locked stream. PAR-1 needs branch buffers.

**Recommendation.**

- A thread-local `Option<Vec<(Stream, String)>>` in `__sink`. When set, `out` and `err` push the formatted record instead of writing. One buffer for both streams keeps a branch's stdout/stderr interleaving.
- A helper installs a fresh buffer per block and returns it with the block's result. The region appends buffers in index order through the same `out`/`err` path, so they land in the enclosing buffer when nested ("emitted when the outermost join appends them", SINK-4) and on the real stream otherwise.
- Append up to and including the winning panicking block; drop the rest (SINK-4).
- `join`'s left branch and all inline work write directly: they are first in order.
- Buffers are unbounded; SINK-5's backpressure does not act inside a region. Note it in the docs.

### 1.9 Nested regions

- The budget (§1.3) handles depth: inner forks get threads while tokens last, then run inline.
- A waiting caller only blocks in `thread::scope`; it never runs another region's work. That avoids rayon's known hazard, "a work-stealing threadpool can cause it to implicitly recurse the lock" ([rayon #592](https://github.com/rayon-rs/rayon/issues/592)), and rayon's own caveat that blocking closures "could lead to a deadlock" ([rayon join](https://docs.rs/rayon/latest/rayon/fn.join.html)).
- A region inside a `Shared::with` closure is legal (the closure is `Pure`, and so are `par` calls with `Pure` arguments). Branches cannot reach the cell, so it cannot deadlock; it only lengthens the hold. Under the simulator it adds no point, consistent with LOCK-4.

## 2. Pitfalls

1. **Oversubscription.** Thread-per-fork without the budget explodes on recursive `join`. The budget must be process-wide, not per region.
2. **Token leak.** Return the token in a drop guard inside the helper; a panic path that skips it turns the process sequential, silently.
3. **Nondeterministic reductions.** Any internal combine must be index-ordered concatenation. No float or user-`Combine` folding in v0.
4. **Several panics.** Choosing by arrival time instead of index breaks PAR-2 only under load, so it passes casual tests. Dropping a losing payload can itself panic; drop losers before `resume_unwind`, not during unwinding (PANIC-2 aborts).
5. **Panic-hook output** (§1.5) is the one observable that differs from the sequential program in v0.
6. **Locks.** Branches cannot lock. The implementation's own mutexes (block iterator, result list) must not be held while user code runs, or a user panic poisons them: claim, unlock, then run.
7. **Small inputs** pay thread start-up (§1.3).
8. **Stack size.** Helpers get the spawned-thread default, "2 MiB on all Tier-1 platforms" ([std::thread](https://doc.rust-lang.org/std/thread/index.html)), less than a typical main thread. Deep recursion can overflow on a helper and not inline. PAR-1's "up to resource exhaustion" covers it; [code] task children have the same default, so match them and leave sizing to the bead that sets task stack caps (03 §7.3).
9. **`available_parallelism` = 1** (small CI machines) makes every region inline, so a test that calls the public API may never start a thread. Tests need the explicit-thread-count entry (§3).
10. **Two implementations.** A separate sequential `par_sort` diverges from the threaded one for an unlawful `Ord` (§1.4).
11. **Sim switch scope.** A process-global switch races between concurrent in-process simulations (§1.7).
12. **Leaked helper state.** A helper must clear its sink buffer and never set `CURRENT`; threads are fresh per fork, so this holds by construction and breaks if a pool is added later.

## 3. Cheapest falsifying tests

Unit tests in `par.rs` call a private entry that takes the thread count, so they exercise real threads on any machine. Toolchain tests are admitted Tide programs (the bead's "executes, not only type-checks").

1. **Equality.** For lengths 0, 1, 2, 7, 1000 and thread counts 1 and 4: `par_map` equals `iter().map().collect()`, `par_for` equals `iter_mut().for_each`, `par_sort` equals `sort`, `join` equals the pair. Falsifies order preservation and chunk-boundary errors.
2. **Stability.** `par_sort` on `(key, original index)` with `Ord` on the key only, length above twice the leaf size: indices ascend within equal keys.
3. **First panic wins.** `par_map` where indices 3 and 900 panic with their index as payload, the low one after a delay, 4 threads: the payload is 3, on repeated runs. Same for `join` with both panicking.
4. **Stop flag.** After a panic at index 0, a counter of executed elements stays well below `len` for a long slice. A weak bound, since PAR-1 leaves it unspecified.
5. **Sink order.** A toolchain test whose branches print their index, run as a child process: stdout equals the sequential program's. With a panic at index k: stdout holds exactly the records before it, and nothing from later blocks.
6. **Sequential switch.** With the switch set, a region records `thread::current().id()` per element: all equal the caller's. Falsifies "never create OS threads". fx-nkv.5 repeats it under the scheduler and checks the trace has only `sink` events.
7. **Budget.** Recursive `join` to depth 20 with an atomic gauge of live helpers: the maximum never exceeds `threads − 1`, and it completes.
8. **Token return.** After a panicking region, a second region still starts a helper (gauge reaches 1).
9. **Merge cost.** With a counting `Ord`, sorting two presorted halves at one node costs O(n) comparisons, not n·log n. Falsifies §1.4 option (a).
10. **Rejections.** The existing conformance cases (`par_join_reject`, `par_shared_reject`, `par_capture_reject`, `par_elements_reject`) must still pass with the included production `par.rs`; add one `par_sort` element case. Falsifies key drift against `pure.rs`.
11. **Cancellation.** A child cancelled while inside `par_map` returns the full result from the region, then sees `Cancelled` at its next checkpoint.

## 4. Where the design is silent or ambiguous

| Point | Status | Recommendation |
| --- | --- | --- |
| `par_sort` signature | silent | `par_sort(&mut self) where T: Ord + Pure + Send`; no `par_sort_by` |
| `join` result bounds | silent (stub GUESS) | `RA: Send, RB: Send` |
| "merges depend only on the input length" | ambiguous for comparison order | ranges fixed by length; take §1.4 (b) if the comparison sequence must survive toolchain bumps |
| v0 scheduler vs. §8.4's pool | text names a work-stealing pool | owned threads plus budget now, labelled partial; unobservable |
| `par-threads` | no manifest reader exists | `available_parallelism()` once; key stays planned |
| Regions and `Tasks::limit` | silent | not counted, not registered |
| Panic-hook output of losing branches | PAR-2 says only the first "is reported" | name helpers after the caller; record the rest as partial |
| Trace of a region | implied by §9.1 | no events beyond the task's `sink` records |
| Helper stack size | silent | same as task children |
| `par_chunks_mut` | in 03, not in reference §8 or the bead | out of scope; report to the coordinator |

DONE
