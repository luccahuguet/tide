# Deterministic seeded scheduling and simulation (research for fx-nkv.5, fx-nkv.6)
Drafted by Fable on 2026-11-10. The decisions are recorded on fx-nkv.5 and fx-nkv.6.

Research report, 2026-11-10. Read-only: nothing in the repository or in `br` was changed, no cargo build ran, and nothing under `trials/e1-checked/` was opened.

Evidence labels: **[design]** is Tide's own text, **[code]** is the current tree at `b153512`, **[web]** is a cited source, **[inference]** is my reasoning. Web facts were read from the cited page by research subagents; the few they could not confirm are marked *unverified*.

## 0. What the design already fixes

These are settled, so the report does not reopen them.

- **Engine.** v0 has one: "baton-passed OS threads, one runnable at a time, in a rustc debug build". A simulation "is defined by its scheduler decisions, so any engine that makes them at the same points can run it" (03 §9.1, §1.3).
- **Scheduling points.** Every capability operation, every handle operation (`spawn`, `join`, `Shared::with*` acquire and release, …) and every checkpoint; "between two points a task runs alone" (03 §9.3; reference §6.8).
- **Strategies.** `random` (default) and `fifo` for debugging. `pct(depth)` is "not in v0" (03 §9.3; companion Appendix A).
- **Test surface.** `#[sim_test]` / `#[sim_test(seeds = N)]` on `fn t(sim: SimRoot)`, 100 seeds by default, a fresh `SimRoot` per seed, the body as the simulation's main task; `Root::sim(seed)` in any test (reference §6.8; 03 §9.2).
- **Self-check.** "The first seed of every run, and every failing seed, runs twice and the trace hashes are compared", reporting the first diverging pair of events (03 §9.7).
- **Trace events.** Virtual time, node, task, kind (`cap`, `sched`, `fault`, `sink`, `spawn`, `exit`, `cancel`, `cell`, `chan`, `assert`), the operation with its source span, arguments and result (03 §9.6).
- **Aborts.** `tide test` reruns each test and seed in flight alone, reports each that aborts, and goes on in a fresh process (reference §9).
- **Join and locks.** `Task::join` and scope exit are shielded, not checkpoints (JOIN-1, JOIN-2). `Shared` acquisition is FIFO per cell, not a checkpoint, and its closure is `Pure` and "cannot suspend" (LOCK-3, LOCK-4).

## 1. Design choices

### 1.1 Baton over the existing thread runtime, or a separate executor

**Options.**

- (a) Keep one OS thread per task and add a baton: only the holder runs, the rest park.
- (b) Stackful coroutines on one thread.
- (c) An async executor.

**Evidence.**

- [design] v0 is (a); the evented lowering is v1.
- [code] `Tasks::scope` runs `thread::scope`, and children borrow the parent frame and may hold non-`Send` locals. Option (a) keeps this code path and its safety argument unchanged.
- [web] Real-thread serialization is established practice:
  - CHESS wraps the concurrency API and enables one thread at a time ([OSDI'08](https://www.usenix.org/legacy/event/osdi08/tech/full_papers/musuvathi/musuvathi_html/index.html)).
  - Fray adds "shadow locks" around JVM primitives so "only one application-level thread executes at a given time". It chose not to replace primitives because that "opens up a Pandora's box of special cases", and measures a median 3.2× slowdown ([Fray](https://arxiv.org/html/2501.12618v2)).
  - Coyote signals the next operation and pauses the caller ([CoyoteRuntime.cs](https://raw.githubusercontent.com/microsoft/coyote/main/Source/Core/Runtime/CoyoteRuntime.cs)).
- [web] Option (b) is what Shuttle and Loom do:
  - Shuttle uses `corosensei` coroutines on one thread ([continuation.rs](https://github.com/awslabs/shuttle/blob/855ae5d92a02dd3c02ddd3ca99dfaeb3b5e31d5b/shuttle-engine/src/runtime/thread/continuation.rs)).
  - Loom uses the `generator` crate ([scheduler.rs](https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/src/rt/scheduler.rs)).
  - It is faster per switch, but needs a coroutine dependency (AP-DEPENDENCY-001) and a second task implementation.
- [web] Option (c) is madsim, turmoil and FoundationDB: a single-threaded executor over async code ([madsim task](https://raw.githubusercontent.com/madsim-rs/madsim/main/madsim/src/sim/task/mod.rs), [turmoil](https://docs.rs/turmoil/latest/turmoil/), [FDB](https://apple.github.io/foundationdb/testing.html)). Tide's v0 tasks are ordinary functions, so it does not apply.

**Recommendation: (a), in two layers.**

1. A pure decision core with no threads. It holds task states, `block(task, wait, span)`, `wake`, `pick(rng)`, the trace, and the list of blocked tasks.
2. A thin baton over it: one mutex, and one `Condvar` per task, waited on in a `while current != me` loop.

Put the core inside the existing `Runtime` as an optional field. `Runtime` already owns every lifecycle transition (AP-OWNERSHIP-001), and one lock then covers lifecycle and scheduling.

Reason: the split makes the bead's "finite scheduler-internal fixture with two waiters that are never woken" a plain unit test of the core. It is also what §9.1 means by an engine-independent decision sequence. A per-task condvar avoids the thundering herd of one shared condvar and is safe against spurious wakeups. No new dependency.

### 1.2 How code knows it is simulated

**Evidence.**

- [code] `Tasks` is a unit struct, `Tasks(())`.
- [code] A top-level `scope` creates a fresh `Runtime`; children find it through the thread-local `CURRENT`.
- [code] `Shared::with` and `Task::join` take no `&Tasks`.
- [design] "Capability types are concrete … Test builds add the simulator behind one predictable branch" (03 §1.3).

**Options.**

- (a) `Tasks` carries a private `Option<Arc<…>>` simulation handle. `Shared::new(v, &tasks)` copies it into the cell; `Scope` and `Task` reach it through their runtime.
- (b) An ambient thread-local "current simulation" set by `Root::sim`.

**Recommendation: (a).** It is the "one predictable branch" the design describes. It survives `Tasks::clone`. A cell knows its simulation without ambient state. `CURRENT` stays as the answer to "which task am I", as today.

Under (a), a `Shared` made from one simulation and used by a thread that belongs to another, or to none, is detectable. Panic with a clear message instead of scheduling it.

The design is silent on that misuse. A panic is the conservative reading and adds no language rule.

### 1.3 Where scheduling points go

[design] fixes the list. The choices left are placement and mechanism.

| Operation | Recommended placement |
| --- | --- |
| `Scope::spawn` | The parent registers the child (task id, parking slot) in scheduler state itself, starts the OS thread, then takes one point. The child's first act is to wait for the baton. |
| `Task::join` | One point on entry. If the child is unfinished, `block(Join(child), span)`; the child's finish wakes it. |
| Scope exit | The same, as `block(ScopeExit(scope), span)` until every child has finished, placed before `thread::scope`'s implicit OS join. |
| `Shared::with` | One point before the ticket is reserved, then the existing body, then one point after release (§1.4). |
| `checkpoint` | One point, then the cancellation check. |
| `yield_now` | Check, one point, check. No `thread::yield_now` under simulation. |
| Task finish | Record the outcome, wake waiters, pick the next task, hand over the baton. This is the thread's last user-visible act. |
| Sleep, timers, channels | Not in this slice (bead scope). |

**Evidence and reasons.**

- **Spawn.** Coyote blocks the scheduler until the new operation has started and paused (`WaitOperationsStart`). Fray blocks the child right after `Thread.start()`. CHESS brackets the closure with begin and end calls. The purpose is that OS thread start-up timing never reaches scheduler state. Registering in the parent gives the same property with no handshake wait [inference].
- **Join.** Shuttle's `join` switches if the target has finished and otherwise blocks, then switches ([shuttle thread.rs](https://github.com/awslabs/shuttle/blob/855ae5d92a02dd3c02ddd3ca99dfaeb3b5e31d5b/shuttle-std/src/thread.rs)). [code] `Task::join` blocks in `mpsc::Receiver::recv`, and scope exit blocks in `thread::scope`. Both are OS waits that a baton holder must never enter while the child still needs the baton.
- **Finish.** Shuttle runs thread-local destructors before publishing the join result, because the thread "can't be considered finished" otherwise ([shuttle thread.rs, docs.rs source view](https://docs.rs/shuttle/latest/src/shuttle/thread.rs.html); seen in a search snippet only). [code] In `run`, the closure and the sent value are consumed before `finish`, so no user destructor runs after it today. Keep that as an invariant: a user `Drop` that ran after the baton was released could call `Shared::with` without holding it.
- **Why a release point as well as an acquire point.** Sinks are not scheduling points. Without a release point, a task's `with` and its following `println!` are atomic, so the order "A writes, B reads the write, B prints, A prints" is unreachable although real threads produce it [inference]. The design lists "acquire and release"; implement both.

A point at which only one task is runnable makes no random draw and need not emit a decision record. Shuttle's PCT likewise counts a step only when more than one task is runnable ([pct.rs](https://github.com/awslabs/shuttle/blob/855ae5d92a02dd3c02ddd3ca99dfaeb3b5e31d5b/shuttle-schedulers/src/pct.rs)).

### 1.4 How `Shared`'s condvar wait cooperates

This is the constraint recorded from fx-nkv.4.

**Options.**

- (a) Take the scheduling point before the ticket is reserved, and never yield between reservation and release.
- (b) Reserve, then make the wait a scheduler `block` that hands off the baton.
- (c) Replace the condvar with a simulated lock.

**Evidence.**

- [web] Shuttle's `Mutex::lock` yields before acquiring: "the first poll of the acquire is the scheduling point before the lock operation". Unlock switches and wakes the waiters ([mutex.rs](https://github.com/awslabs/shuttle/blob/855ae5d92a02dd3c02ddd3ca99dfaeb3b5e31d5b/shuttle-std/src/sync/mutex.rs)).
- [web] CHESS calls the "try" version of a blocking call first and, on failure, moves the task to that resource's waiting set ([OSDI'08](https://www.usenix.org/legacy/event/osdi08/tech/full_papers/musuvathi/musuvathi_html/index.html)).
- [web] Fray keeps the real primitive and guards it ([Fray](https://arxiv.org/html/2501.12618v2)).
- [inference, from LOCK-4] Under (a) the cell is never contended in simulation. The closure is `Pure`, so it reaches no handle and no capability and has no scheduling point inside it. A baton holder that reaches `with` therefore reserves, acquires, runs `f` and releases with no other task running. `serving == ticket` always holds and the condvar wait is never entered.
- [inference] Option (b) reaches no additional observable behavior. A reservation has no effect that another task can see, so "A reserves, is descheduled, B queues behind A" produces the same orders of `f` as scheduling A's whole `with` before B's. Option (a) is a sound reduction, and FIFO holds trivially because arrival order is acquisition order.

**Recommendation: (a).** Under simulation, turn the unreachable wait into a hard internal panic ("contended cell under the simulator"), not a condvar wait. A violated invariant then fails loudly instead of hanging with the baton held.

The same shape covers fx-ugm: one point before the first lazily reserved ticket of a multi-cell acquire, then no yield until release. (a) is also the least code: the production path is unchanged, and the simulator adds two points and one assertion.

**Ambiguity to record.** fx-nkv.5's Acceptance asks that "the trace records each task blocked in join or Shared acquisition with its span". Under LOCK-4 a task is never non-runnable in `Shared` acquisition in simulation; it is only parked at the acquire point awaiting its turn.

Recommended reading: every scheduling point records the task's pending operation (kind, object, span) as its wait state. A `join` wait is `blocked`. A cell wait is `ready` at `cell(id)`. The internal fixture exercises `block` directly with two waiters of each kind.

This keeps the check as strong as the text for join and honest for cells. It is a technical reading, not a weakened check, but the coordinator should record it on the bead.

### 1.5 Seed-to-schedule strategy, and finding the lost update

**Options.** Uniform random walk, PCT, bounded preemption or DFS, POS, SURW.

**Evidence.**

- [design] `random` is the v0 default and PCT is out of v0.
- [web] PCT guarantees probability at least 1/(n·k^(d-1)) per run for a depth-d bug. Atomicity violations "in general, have depth 2" ([PCT, ASPLOS'10](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/asplos277-pct.pdf)).
- [web] Thomson et al. found "a naive random scheduler is at least as effective as schedule bounding" ([PPoPP'14](https://www.doc.ic.ac.uk/~afd/homepages/papers/pdfs/2014/PPoPP.pdf)). In the follow-up, PCT with d=3 was best overall, random beat preemption bounding, and 18 of 49 benchmarks' bugs appear at least half the time under random scheduling ([TOPC'16](https://www.doc.ic.ac.uk/~afd/papers/2016/TOPC.pdf)).
- [web] Most bugs need at most two preemptions ([ICB, PLDI'07](https://www.cs.columbia.edu/~junfeng/08fa-e6998/sched/readings/context-bounding.pdf); CHESS: "very serious bugs using just two preemptions").
- [web] Fray with plain random search found 70% more bugs than JPF and 77% more than rr's chaos mode ([Fray abstract](https://arxiv.org/abs/2501.12618)).
- [web] POS and SURW improve on random walk for deep or skewed programs ([POS, CAV'18](https://www.cs.columbia.edu/~junfeng/papers/pos-cav18.pdf); [SURW, ASPLOS'25](https://www.comp.nus.edu.sg/~umathur/publications/2025-asplos-zhao-surw-concurrency-testing/)). Shuttle ships random, PCT, DFS and URW ([check.rs](https://github.com/awslabs/shuttle/blob/855ae5d92a02dd3c02ddd3ca99dfaeb3b5e31d5b/shuttle-schedulers/src/check.rs)).
- [web] Loom's exhaustive search is capped at 5 threads and grows exponentially ([loom lib.rs](https://github.com/tokio-rs/loom/blob/9d549cd5357a5485900f4c53951fc1726319659f/src/lib.rs)).

**The counter service** [inference]. Each increment is a read `with`, then a write `with`. An update is lost whenever another worker's write lands between one worker's read and its own write.

Under a uniform choice among runnable tasks there is at least one decision between every read and its write. A seed misses the bug only if no increment is ever interrupted while another worker still has work.

- For 2 workers and 1 increment each, counting one decision per gap, 2 of the 6 interleavings are clean and each has probability 1/4. A seed exposes the bug with probability about 1/2.
- Each further increment of the first-running worker roughly halves the miss probability. The release-plus-acquire pair gives two decisions per gap and lowers it further.
- With N ≥ 2 and K ≥ 4, well over 90% of seeds should fail. The expected number of seeds to the first failure is under 2. Twenty seeds miss with probability below 10⁻⁶ even at the K = 1 rate.

This is a "trivial" bug in TOPC'16's sense. PCT would add nothing here.

**Recommendation.**

- **Strategy.** Uniform random among the runnable tasks, the current one included, at every point with more than one runnable. Add `fifo` only if a test needs it. No PCT.
- **Generator.** Each seed owns a fresh generator, as Shuttle's random scheduler reseeds per iteration ([random.rs](https://github.com/awslabs/shuttle/blob/855ae5d92a02dd3c02ddd3ca99dfaeb3b5e31d5b/shuttle-schedulers/src/random.rs)) and madsim iterates `seed..seed+count` ([builder.rs](https://raw.githubusercontent.com/madsim-rs/madsim/main/madsim/src/sim/runtime/builder.rs)). Use an owned SplitMix64 ([reference code](https://prng.di.unimi.it/splitmix64.c)): about five lines, no dependency. Draw the index by rejection or multiply-shift from a `u64`, never from `usize`.
- **Sub-streams.** Derive the scheduler stream from the seed with a fixed domain constant, so later consumers (latency, faults) get their own streams. TigerBeetle derives network, storage and cluster sub-seeds from one seed ([vopr.zig](https://raw.githubusercontent.com/tigerbeetle/tigerbeetle/main/src/vopr.zig)). This keeps a seed's schedule stable when fx-o8j and fx-byk.2 add draws.
- **Seed set.** `seeds = N` means seeds `0..N`. The design is silent on which seeds; a fixed range is the only choice that makes "a fixed revision over an explicit fixed seed set" reproducible without extra state.

### 1.6 Trace format, and what makes two replays identical

**Evidence.**

- [web] CHESS records the thread chosen at each point plus the enabled set, and treats a mismatch on replay as divergence ([OSDI'08](https://www.usenix.org/legacy/event/osdi08/tech/full_papers/musuvathi/musuvathi_html/index.html)).
- [web] Shuttle encodes the chosen task per step in a compact hex string. Its replay panics with "scheduled task is not runnable" on divergence, and `check_uncontrolled_nondeterminism` replays each schedule and compares runnable sets ([serialization.rs](https://github.com/awslabs/shuttle/blob/855ae5d92a02dd3c02ddd3ca99dfaeb3b5e31d5b/shuttle-engine/src/scheduler/serialization.rs), [replay.rs](https://github.com/awslabs/shuttle/blob/855ae5d92a02dd3c02ddd3ca99dfaeb3b5e31d5b/shuttle-schedulers/src/replay.rs), [uncontrolled_nondeterminism.rs](https://github.com/awslabs/shuttle/blob/855ae5d92a02dd3c02ddd3ca99dfaeb3b5e31d5b/shuttle-schedulers/src/uncontrolled_nondeterminism.rs)).
- [web] Coyote writes a JSON `.trace` for `coyote replay` ([docs](https://microsoft.github.io/coyote/get-started/using-coyote/)).
- [web] madsim's check logs one byte per RNG draw mixed with virtual time and fails at the first divergent draw ([runtime](https://raw.githubusercontent.com/madsim-rs/madsim/main/madsim/src/sim/runtime/mod.rs), [rand.rs](https://raw.githubusercontent.com/madsim-rs/madsim/main/madsim/src/sim/rand.rs)).
- [web] FoundationDB prints an "unseed", a final draw from the deterministic RNG ([fdbserver.actor.cpp](https://raw.githubusercontent.com/apple/foundationdb/release-7.3/fdbserver/fdbserver.actor.cpp)). That a harness compares unseeds across reruns is *unverified* from primary sources.
- [web] FoundationDB, TigerBeetle and madsim replay by seed (plus commit), not by recorded schedule ([FDB client testing](https://apple.github.io/foundationdb/client-testing.html), [vopr.md](https://raw.githubusercontent.com/tigerbeetle/tigerbeetle/main/docs/internals/vopr.md)).

**Recommendation.**

- **Format.** NDJSON, one event per line, written by hand with a fixed key order and no serializer dependency. It is the shape §9.6 already names, and it diffs line by line, which §9.7's "first diverging pair of events" needs.
- **Kinds in this slice.** `sched` (step, sorted runnable ids, chosen id), `spawn`, `exit` (outcome), `cancel`, `cell` (acquire or release, cell id), `join` and `scope_exit` wait and wake, and `sink` (stream, message).
- **Fields.** Each event carries step, task, kind, operation, span, and the task's wait state where it has one. Emit virtual time as `0` and node as the single main node only if the implementer wants the §9.6 shape now. The trace is not a stable contract until fx-nkv.7, and the design leaves field names open.
- **Replay.** Rerun the seed; record no schedule. Identity is then a property the self-check can test.

Two runs are identical when all of these hold:

1. Every decision is a function of the seed and the sequence of runnable sets.
2. Runnable sets are ordered by task id.
3. Task, scope and cell ids are simulation-local counters assigned by the baton holder.
4. Spans are source `file:line:col`. `#[track_caller]` with `Location::caller()` on `spawn`, `join`, `with`, `checkpoint` and `yield_now` yields them without changing a signature.
5. No address, OS thread id, wall time or hash order enters an event.

- **Self-check.** Compare the two traces byte for byte and print the first differing line pair. A printed `trace_hash` can wait for fx-nkv.7, which needs real content hashes anyway. The design names no hash algorithm.
- **"Output".** Define it as the ordered `sink` events, not the process's raw stdout and stderr (see §2).

### 1.7 The runner: `#[sim_test]`, `Root::sim`, process isolation

**Evidence.**

- [web] Rust attribute tests are thin wrappers:
  - `#[madsim::test]` emits a core `#[test]` whose body calls a runtime builder ([macro](https://raw.githubusercontent.com/madsim-rs/madsim/main/madsim-macros/src/lib.rs)).
  - `#[tokio::test]` does the same ([docs](https://docs.rs/tokio/latest/tokio/attr.test.html)).
- [web] Seed reporting:
  - madsim prints "run with `MADSIM_TEST_SEED={seed}` … to reproduce".
  - TigerBeetle logs `SEED=` at start and "you can reproduce this failure with seed=…", and exits with a distinct failure code ([vopr.zig](https://raw.githubusercontent.com/tigerbeetle/tigerbeetle/main/src/vopr.zig)).
- [web] Process isolation without `fork`:
  - rusty-fork spawns "a fresh instance of the current process, after adjusting the command-line" to run one test, with parameters in environment variables ([docs](https://docs.rs/rusty-fork/latest/rusty_fork/)).
  - libtest's own subprocess mode re-executes the binary with `__RUST_TEST_INVOKE=<test>` ([libtest](https://raw.githubusercontent.com/rust-lang/rust/master/library/test/src/lib.rs)).
  - proptest's fork mode exists to survive aborts, overflows and hangs, and is "generally slower" ([forking](https://proptest-rs.github.io/proptest/proptest/forking.html)).
  - nextest runs a process per test so one crash does not take others down ([nextest](https://nexte.st/docs/design/why-process-per-test/)).
- [code] There is no `tide` CLI. Tests are driver-built libtest binaries (`--test`), run by `crates/tide-admit/tests/build.rs`.

**Recommendation.**

- **`Root::sim(seed)`.** Builds a `SimRoot` whose `tasks` carries a new simulation. The calling thread becomes task 0 and holds the baton from that moment, with no closure wrapper. Dropping the `SimRoot` ends the simulation. Structured scopes guarantee no task is alive then; assert it. A plain `#[test]` calling it runs in-process: the design gives it no isolation, and the caller chose one seed.
- **`#[sim_test(seeds = N)]`.** Expands to a parameterless `#[test]` that calls one hidden std entry with the test's path, N and the body as `fn(SimRoot)`. The entry has two roles, chosen by a private environment variable:
  - *Parent.* Re-executes `current_exe()` with `--exact <path> --nocapture` and the variable set, stdin closed, with a wall-clock timeout (AP-PROOF-001).
  - *Child.* Runs seeds `0..N` in order with a fresh `SimRoot` each, prints one line as each seed starts, catches a panic per seed inside std, and runs the first seed and any failing seed twice for the self-check. On failure it prints the seed, the failure and the trace, then exits nonzero.
- **Aborts and hangs.** If the child dies abnormally or times out, the parent attributes it to the last started seed, reports that seed, and fails. Continuing the remaining seeds in a fresh child is what reference §9 describes; it can be added in the same loop.
- **Exit status.** The parent test panics with the failing seed in its message, so libtest exits 101 (nonzero), as the Acceptance requires.

One child per test, not per seed, gives "process-isolated" at one spawn per test instead of a hundred.

**Silent or ambiguous in the design.**

- **How one seed is selected.** `tide test --seed N` is specified, but no `tide` binary exists. Recommend the same private environment variable for "run exactly this seed", documented as toolchain-internal. It is the hook fx-nkv.8's "one command" and fx-nkv.7's `--seed`/`--replay` split will call.
- **Where simulated output goes.** The reference says every sink record "enters `sim.records()`". Recommend: under simulation, sinks append `sink` events and do not write to the real stdout; the runner prints a failing seed's records. Otherwise a 100-seed test prints everything 100 times.
- **Parameter type.** `fn t(sim: SimRoot)` takes it by value. `Root::sim` returning the same type covers both forms.
- **FX0504** (test code only) belongs to fx-u65.4. Until it lands, nothing stops non-test use.

## 2. Pitfalls others hit

- **Hash order.** Rust's `RandomState` seeds per thread from OS entropy and increments a key per map, so iteration order varies by run and by how many maps the thread made ([std source](https://raw.githubusercontent.com/rust-lang/rust/master/library/std/src/hash/random.rs)). madsim has to override `getrandom` and warns when that is done too late. Tide user code is already covered: `HashMap::new()` uses fixed keys (reference §6.4). The scheduler's own state must use `Vec` or `BTreeMap`, as `Runtime` does today.
- **Thread identity.** Never put `ThreadId`, thread names or OS ids in ids or events. The default panic hook prints `thread '{name}' ({tid}) panicked at …`, with an OS thread id that varies per run ([panicking.rs](https://raw.githubusercontent.com/rust-lang/rust/master/library/std/src/panicking.rs)). Raw stderr is therefore never identical across runs. This is why §1.6 defines output as sink events. Do not install a global panic hook; it would fight libtest's.
- **Process-global counters.** LOCK-3 calls the cell id "a process-unique counter … deterministic under the simulator". libtest runs tests on parallel threads ([rustc tests](https://doc.rust-lang.org/rustc/tests/index.html)), and a `#[sim_test]` child runs many seeds in one process. A process-global counter would differ between two runs of one seed. Under simulation, allocate cell and task ids from the simulation. This is consistent with LOCK-3's "deterministic under the simulator", but fx-ugm should know it.
- **Thread start timing.** A child that registers itself races the parent. Register in the parent (§1.3).
- **Blocking with the baton.** An uninstrumented wait while holding the baton hangs the run or fakes a deadlock:
  - CHESS reports false deadlocks from locks shared with the environment.
  - Coyote needs a timer to report "potential deadlock or hang".
  - Loom says code not using its types "is invisible".
  - Go's synctest excludes mutexes and I/O from "durably blocked" ([go.dev](https://go.dev/blog/synctest)).

  In Tide's slice the OS waits are `recv` in `join`, the `thread::scope` join, and the `Shared` condvar. Each needs the treatment in §1.3 and §1.4. The runner's timeout is the backstop.
- **Panics while scheduling.** A panicking task must still release the baton. [code] `run` already catches the unwind before `finish`; do the hand-off there, not in a destructor that a second panic could skip. A panic inside the scheduler's own lock poisons it; treat poison as fatal for the seed. A double panic aborts, which the parent process reports (§1.7).
- **Work after finish.** No user code may run on a thread after its hand-off (§1.3).
- **Time.** Nothing in this slice reads a clock. Keep `Instant` and `SystemTime` out of the scheduler and trace. The runner's timeout only decides when to give up and never enters a trace.
- **Timed waits made untimed.** Fray turns timed waits into untimed ones and reports "practically improbable schedules" that developers refused to fix. This is relevant when fx-o8j adds virtual time, not now.
- **Spin loops.** A `yield_now` loop terminates with probability 1 under `random` but can livelock under `fifo`. Shuttle fails after 1,000,000 steps ([config.rs](https://github.com/awslabs/shuttle/blob/855ae5d92a02dd3c02ddd3ca99dfaeb3b5e31d5b/shuttle-engine/src/config.rs)) and Loom after 1,000 branches. The design names no step limit. Rely on the runner timeout in this slice, and report an internal step ceiling as a candidate for the slice that adds `fifo`.
- **Absolute paths in spans.** `Location::caller().file()` is the path given to rustc. Traces match across reruns of one build but not across checkouts. That is fine for fx-nkv.5 and .6; fx-nkv.7's manifests must handle it.
- **Allocation addresses.** Nothing may print or compare addresses. Admission already rejects address comparisons (`the_backstop_rejects_address_comparisons_in_every_profile`).
- **Parallel regions.** `par` is not in the tree yet ([code] `lib.rs`). The fx-nkv.5 comment about running regions sequentially under the scheduler applies only once fx-iob lands.

## 3. Cheapest falsifying tests

| Recommendation | Test that would falsify it |
| --- | --- |
| Pure core with wait states (§1.1, §1.4) | Unit test: two tasks, `block` both, nothing runnable. `blocked()` must list both with wait kind and span. This is the bead's fixture. |
| Baton serializes (§1.1) | Two tasks each bump a non-atomic "inside" flag between points, asserting it is clear on entry, over 200 seeds. |
| Spawn registration (§1.3) | Sleep a few milliseconds at child thread start under `cfg(test)`. The trace for a fixed seed must not change. |
| Join and scope exit hand off (§1.3) | Parent joins a child that has not run; child joins a grandchild; a dropped handle. Every seed in 0..200 must finish under a timeout. |
| `Shared` point before reservation (§1.4) | Three workers × many `with` calls over 200 seeds: no hang and no "contended cell" panic. Then move the point after reservation in a scratch build and expect the panic within a few seeds. |
| Release point (§1.3) | A reads-then-prints test: some seed in 0..100 must produce the order "A writes, B reads it, B prints, A prints". |
| Random finds the lost update (§1.5) | Buggy counter (N=2, K=4): count failing seeds in 0..100. Expect a clear majority; fewer than 10 falsifies the estimate. The fixed revision must pass all 100. |
| Two interleavings (bead Acceptance) | Seeds 0..20 yield at least two distinct `sched` sequences for the two-task workload. |
| Byte-identical traces (§1.6) | Run one seed twice in one process and once in a second process; compare bytes. Repeat with other tests running in parallel, which catches process-global ids. |
| Self-check reports divergence (§1.6) | A test-only hook injects one extra draw on the second run. The runner must print the first differing line pair and fail. |
| Runner (§1.7) | Driver-built test binary with `#[sim_test(seeds = 20)]` on the buggy counter: exit status nonzero and the failing seed in the output. With a body that calls `abort`, the parent must name the in-flight seed. A plain `#[test]` with `Root::sim(3)` runs a `sim.tasks` scope. |
| Production unchanged (parent Acceptance) | The existing `task.rs` and `sync.rs` tests and `shared_counter` still pass with no simulation handle. The conformance fixture still includes the same files. |

## 4. Design gaps, collected

Each has a recommendation above. None needs a new language rule.

1. "Blocked in Shared acquisition" cannot be a non-runnable state under LOCK-4. Record a wait state at the acquire point (§1.4).
2. LOCK-3's "process-unique" cell id against per-seed determinism. Use simulation-local ids under simulation (§2).
3. Which seeds `seeds = N` means. Use `0..N` (§1.5).
4. Whether simulated sink output also reaches real stdout. Do not forward it (§1.7).
5. How a single seed is selected before a `tide` CLI exists. Use a toolchain-internal environment variable (§1.7).
6. The `trace_hash` algorithm and exact trace field names. Compare bytes now and fix names in fx-nkv.7 (§1.6).
7. No step limit is specified. Rely on the timeout now; raise it with `fifo` (§2).
8. A handle used across simulations. Panic (§1.2).
9. Whether a point with one runnable task emits a `sched` event. Emit none, since there is no decision (§1.3).

DONE
