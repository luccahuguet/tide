# Clocks, virtual time, deadlines and task limits (research for fx-o8j, fx-zc1)
Drafted by Fable on 2026-11-10; its decisions are recorded on fx-o8j and fx-zc1.

Research report, 2026-11-10. It prepares the implementers of fx-o8j and fx-zc1. Nothing here adds a language rule. Where the design is silent, section 4 says so and recommends a reading.

Sources read locally: both beads, fx-nkv.3, the 2026-11-10 decisions on fx-nkv.5 and fx-nkv.6, fx-o4h, fx-hxw, `design/03-authority-concurrency.md` (§0, §1.1, §2.3, §6, §9), `design/reference.md` (§6.1, §6.8, §7.1–7.2), `crates/tide-std/src/task.rs` on `main` at 0e2457a, and the conformance stub std. The fx-nkv.5 worktree held no scheduler code yet, so the scheduler interface below is the one its decision comment describes. No `trials/e1-checked/` file was opened.

Web evidence was fetched on 2026-11-10. Two fetches failed (FoundationDB `sim2.actor.cpp`, turmoil `sim.rs`) and JEP 505 returned 403; claims about those three rest on search snippets and are marked "unverified".

## 1. Design choices

### 1.1 `Instant`, `SystemTime` and `Clock` representation

**What the design fixes.** `Instant` is a distinct type with no `now()` or `elapsed()` and otherwise Rust's API; the stub declares `Instant(Duration)` and `SystemTime(Duration)` with `Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash`. Simulated monotonic time starts at zero and wall time at 2000-01-01T00:00:00Z (03 §9.3).

**Options.**

- A. One representation on both sides: `Instant(Duration)` is the offset from an origin. In simulation the origin is zero. On the real runtime it is a process anchor, a private `OnceLock<std::time::Instant>` set at first use.
- B. An enum or `cfg` split that wraps `std::time::Instant` on the real side.
- C. A branded instant that records which clock made it.

**Evidence.**

- Rust's `Instant` is opaque: "Instants are opaque types that can only be compared to one another" (https://doc.rust-lang.org/std/time/struct.Instant.html). An offset from a hidden anchor keeps that property.
- madsim uses a zeroed base instant plus an elapsed offset, which is option A (https://raw.githubusercontent.com/madsim-rs/madsim/main/madsim/src/sim/time/mod.rs).
- Trio warns against mixing clocks instead of branding them: "You should not assume that Trio's internal clock matches any other clock you have access to" (https://trio.readthedocs.io/en/stable/reference-core.html).
- Go's synctest starts its fake clock at "midnight UTC 2000-01-01" (https://pkg.go.dev/testing/synctest), the constant Tide chose. madsim instead draws the wall base from the seed.

**Recommendation: A.** It matches the stub, gives one `Debug` form and one arithmetic path, and needs no `cfg`. The process anchor is required anyway: `Scope::set_deadline(at)` takes no `Clock`, so the runtime must turn an `Instant` into a wait without one. Reject C: a brand costs `Copy` data or a runtime check on every comparison, and the admitted program has one `Root`.

**Arithmetic.** Follow Rust exactly:

- `Add<Duration>` and `Sub<Duration>` panic on overflow, with `checked_add` and `checked_sub`. Rust: "This function may panic if the resulting point in time cannot be represented" (same page).
- `duration_since` and `Sub<Instant>` saturate to zero, with `checked_duration_since` and `saturating_duration_since`. Rust: "Previous Rust versions panicked when `earlier` was later than `self`. Currently this method saturates" (same page).
- `SystemTime` is the offset from `UNIX_EPOCH`. `duration_since` returns `Result`, because "this time measurement is not monotonic" (https://doc.rust-lang.org/std/time/struct.SystemTime.html). Simulated `system_time()` is 946,684,800 s plus virtual elapsed time.
- A real wall clock set before 1970 clamps to the epoch, since the offset is unsigned.

**`Clock`.** Give it the same private simulation identity that fx-nkv.5 decision 2 gives `Tasks` (`Option<Arc<sim>>`). `now()` reads the anchor or the simulation's `now`. It is a scheduling point (a capability operation, 03 §9.3) but never blocks and takes no virtual time (reference §6.8).

**Real clock source.** `std::time::Instant` (on Linux `clock_gettime` with `CLOCK_MONOTONIC`, same Rust page). Rust does not specify whether suspend counts as elapsed time; Tide inherits that.

### 1.2 How virtual time advances

**What the design fixes.** "Virtual time advances only when every task on every node is blocked, and then jumps to the next timer or delivery"; if no task can run again the run fails with a deadlock report (03 §9.3). Computation takes no virtual time.

**Prior art.**

| System | How time moves | Source |
| --- | --- | --- |
| tokio paused clock | "If time is paused and the runtime has no work to do, the clock is auto-advanced to the next pending timer" | https://docs.rs/tokio/latest/tokio/time/fn.pause.html |
| Go synctest | "Time in a bubble only advances when every goroutine in the bubble is durably blocked", then "to the next time that will unblock at least one goroutine"; otherwise "there is a deadlock and Test panics" | https://pkg.go.dev/testing/synctest |
| Trio `MockClock(autojump_threshold=0)` | "the clock will jump whenever all tasks are blocked", to "the run loop's next scheduled timeout" | https://trio.readthedocs.io/en/stable/reference-testing.html |
| madsim | jumps to the next timer plus 50 ns, and also adds 50–100 ns per task poll; with no timer it panics "no events, all tasks will block forever" | https://raw.githubusercontent.com/madsim-rs/madsim/main/madsim/src/sim/time/mod.rs, https://raw.githubusercontent.com/madsim-rs/madsim/main/madsim/src/sim/task/mod.rs |
| turmoil | fixed tick per step, 1 ms by default (unverified) | https://docs.rs/crate/turmoil/0.7.2/source/src/builder.rs |
| TigerBeetle | tick-driven, with time sped up in the simulator (secondary source) | https://docs.tigerbeetle.com/about/vopr |
| FoundationDB | one process that "steps through time, synchronized across the system" (queue order unverified) | https://apple.github.io/foundationdb/testing.html |
| Coyote | no time at all: a mock timer "removes all concept of real time intervals, and replaces that with a random decision to timeout or not" | https://microsoft.github.io/coyote/concepts/actors/timers/ |

**Recommendation.** Implement the jump inside fx-nkv.5's decision core, in `pick`:

1. If a task is runnable, pick as today. Time does not move.
2. Otherwise, if the timer queue is non-empty, set `now` to the earliest deadline, fire every timer with that deadline (1.3), and pick among the tasks that became runnable.
3. Otherwise report a global deadlock (1.9).

Take no per-step tick and no epsilon. madsim's 50 ns exists "to make sure 'now >= deadline' and avoid deadlock" in its own timer wheel; Tide compares `now >= deadline` directly, so a jump lands on the exact instant. That is what fx-o8j's Acceptance needs ("wakes at the exact virtual instant and `sim.elapsed()` reports it"). Reject Coyote's model: it cannot give `sim.elapsed() == 5 s` (03 §12.2).

Store virtual `now` as a `Duration`. `sim.elapsed()` returns it. fx-o8j draws no random numbers: modeled latencies belong to fx-byk.2 and fx-hxw, which use their own sub-streams (fx-nkv.5 decision 5).

### 1.3 Timer ordering and determinism

**Constraints from the design.** "When a scope deadline and an operation's own deadline fall on the same instant, the scope's cancellation is observed" (TASK-4). "An enclosing deadline at the same instant as the scope's own comes first" (TASK-2).

**Evidence.** Neither Trio's nor Go's documentation states an order for equal deadlines (pages above). A third-party FoundationDB-style scheduler keys its heap by wake time and then by an increasing task id (https://docs.rs/crate/kora-core/0.2.0/source/src/sim/scheduler.rs); FoundationDB's own comparison is unverified. madsim randomises the ready queue (`try_recv_random`) rather than the timer order.

**Recommendation.** Keep timers in a `BTreeMap` keyed by `(deadline, class, id)`:

- `class` 0 is a scope deadline, with `id` the scope id. Parents have lower ids than descendants in the existing `Runtime`, so an enclosing deadline fires first.
- `class` 1 is an operation timer (sleep, later `recv_deadline` and `Net`), with `id` a simulation-local sequence number.

Fire all timers of the earliest instant in key order before any task runs. A sleeper whose scope deadline shares its instant then wakes into a cancelled scope and returns `Cancelled`. Several tasks woken at one instant all become runnable, and the seeded scheduler picks among them by task id order, so both orders are explored and the trace stays a function of the seed.

Remove a scope's timer at scope exit and when `set_deadline` lowers it. A stale entry would cause a jump that wakes nobody: it shows a wrong time in a deadlock report and adds phantom `advance` events.

### 1.4 `sleep` and `sleep_until`

**Recommendation.**

- `sleep(d)` is `sleep_until(now.checked_add(d))`; an overflow means "never". `sleep(Duration::MAX)` should not panic. Trio defines `sleep_forever` as `sleep(math.inf)` (Trio reference-core). In simulation a sleep that never ends, with no other timer, ends in the deadlock report, which is correct.
- Order inside the call: check cancellation; if the target is not after `now`, take one scheduling point, check cancellation again and return; otherwise register a timer and block with wait state `sleep(until)`. On wake, check cancellation first, then return `Ok(())`.
- A zero or past sleep is therefore a checkpoint that does not block. Trio: `sleep(0)` "is an idiomatic way to execute a checkpoint without doing anything else", and a past `sleep_until` "executes a checkpoint but does not block" (Trio reference-core).
- Real side: loop on a timed park until the target, because "the underlying syscall may be interrupted by a spurious wakeup" and a sleep must "never sleep less" (https://doc.rust-lang.org/std/thread/fn.sleep.html). A cancel unparks the sleeper (1.6).

**Shuttle** documents `sleep` only as "Puts the current thread to sleep for at least the specified amount of time" and `yield_now` as giving a timeslice to its scheduler; its docs describe no virtual clock (https://docs.rs/shuttle/latest/shuttle/thread/index.html). It is no model for exact wake times.

### 1.5 Deadline representation and `set_deadline`

**What the design fixes.** `set_deadline(t)` lowers the deadline; the effective deadline is the minimum over ancestors; expiry cancels but is not a failure; a child that returns `Err` after the scope's own deadline cancelled it is reported as `DeadlineExceeded` if it is the earliest-spawned failing child (TASK-4, TASK-2 rule 2). A scope's cancellation has one cause, the first event.

**Evidence.**

- Go: "If the parent's deadline is already earlier than d, WithDeadline(parent, d) is semantically equivalent to parent"; the error is `DeadlineExceeded` or `Canceled`, and "successive calls to Err return the same error" (https://pkg.go.dev/context). That is the minimum rule and a sticky first cause.
- Trio: "Cancellations in Trio are 'level triggered', meaning that once a block has been cancelled, all cancellable operations in that block will keep raising" (Trio reference-core). Smith: "Timeouts are naturally level-triggered" and "Edge-triggered APIs are notoriously tricky to use" (https://vorpus.org/blog/timeouts-and-cancellation-for-humans/).
- Kotlin shows the cost of a hard timeout: cancellation "may happen at any time", even "after the block finishes executing but before the caller gets resumed with the result" (https://kotlinlang.org/api/kotlinx.coroutines/kotlinx-coroutines-core/kotlinx.coroutines/with-timeout.html). Tide's rule that expiry alone is not a failure avoids losing a finished result.
- Java removed `joinUntil(Instant)` in favour of a timeout in the scope's configuration, and "the scope is cancelled if the timeout expires" (https://bugs.openjdk.org/browse/JDK-8342487; JEP text unverified). Swift's SE-0304 has a sticky flag ("once this flag is set, it is never cleared") and leaves deadlines to later work (https://github.com/swiftlang/swift-evolution/blob/main/proposals/0304-structured-concurrency.md).

**Recommendation.**

- `ScopeState` gains `deadline: Option<Instant>` and `Cause` gains `Deadline(scope)`. `set_deadline(t)` stores `min(old, t)`. If `t` is not after `now`, it cancels at once with that cause.
- The existing `Runtime::cancel` already keeps the first cause and propagates to descendants that have none. Reuse it unchanged for the deadline cause.
- In `finish`, an `Outcome::Error` whose scope's cause is `Deadline(that scope)` becomes a new `Outcome::Deadline`. `finish_scope` treats it as a failure for rule 2, and `scope` returns `E::from(TaskError::DeadlineExceeded)` for it. No echo bit is involved: TASK-4 keys on "returns `Err` after the deadline cancelled the scope", not on an observed checkpoint.
- A nested scope that inherited an ancestor's deadline cause reports its children's own errors, since the deadline is not its own.
- `set_deadline` and `cancel` are not scheduling points. They change only state that other tasks read at their own points, and one task runs at a time, so a point here adds no observable interleaving. 03 §9.3's list of points does not name them.

**How expiry is detected.**

- Simulation: a class-0 timer (1.3). When it fires, call `cancel(scope, Deadline(scope))` and wake every cancellable waiter in the subtree.
- Real runtime, option A: a timer thread per runtime. Option B: lazy evaluation. Whenever the runtime lock is taken to read or record a cause, first compare each relevant scope's deadline with the anchor clock and record `Deadline` if it has passed; blocked operations park with a timeout of the earliest deadline among their own and their non-shielded enclosing scopes.
- **Recommend B.** It adds no thread and no global, and "first event" stays correct, because a child failure that arrives after the instant finds the deadline cause already recorded. Deadlines are soft by TASK-5, so detection at the next checkpoint is all the design promises.

### 1.6 `shield` and how cancellation reaches blocking operations

**Shield evidence.** Trio: inside a shielded scope "you'll be protected from outside cancellations", but "this only applies to outside cancellations: if `CLEANUP_TIMEOUT` expires then ... will still be cancelled" (Trio reference-core). TASK-6 says the same. Kotlin's `NonCancellable` has no such inner bound.

**Shield recommendation.** Model `shield` as a pseudo-scope node in the `Runtime`:

- Its parent is the caller's `Location`. It has no children of its own and it is flagged `shielded`.
- `cancel` propagation and the effective-deadline minimum stop at a shielded node. A checkpoint directly in the shield body is located at that node, so it sees no outer cause. A cause that predates the shield is ignored inside it and seen again after it returns, which keeps cancellation level-triggered.
- Scopes opened inside the shield hang under the node and keep their own deadline, `cancel()` and failures.
- `join` must set the echo bit "exactly where a checkpoint at the same point and moment would if no shield were in force" (TASK-2). Implement that as `observe` at the shield node's parent location when the join fails.
- `shield` is not a scheduling point.

**Blocking operations.** Give the runtime one wait primitive and make every blocking operation use it:

```
wait(kind, span, until: Option<Instant>, cancellable: bool) -> Ready | Timeout | Cancelled
```

- `join`, scope exit, `Shared::with` and `spawn_isolated`'s wait pass `cancellable: false` (JOIN-1, JOIN-2, LOCK rules, 03 §1.1 table).
- `sleep`, a limited `spawn`'s slot wait, and later `recv`, `recv_deadline`, a full `send` and `Net` operations pass `true`. Their own `until` gives `Timeout`; the scope's cancellation gives `Cancelled`; on a tie the wake path checks cancellation first (TASK-4).
- Simulation: `wait` is `block(task, wait, span)` plus an optional class-1 timer. `cancel` marks the cancellable waiters of the affected subtree runnable.
- Real runtime: each waiter parks with `std::thread::park_timeout` in a loop that rechecks its condition, and registers its `Thread` handle with the runtime; `cancel` and the resource it waits on both call `unpark`. Parking tolerates spurious wakeups and needs no extra condition variable per resource. The slot count lives in the `Tasks` handle while the cancellation state lives in the scope's `Runtime` (today one per top-level scope), so a waiter must be wakeable from two owners; a parked thread handle is the smallest mechanism that allows it.

fx-o4h and fx-hxw then add wait kinds and no new cancellation logic. This is the one owner AP-OWNERSHIP-001 asks for.

### 1.7 `Tasks::limit`

**What the design fixes** (03 §6.1). One count per `limit` call, shared by clones and by all scopes from that `Tasks`. A nested `limit` adds a count, and a spawn takes a slot in each "at once". A limited `spawn` is a checkpoint that starts nothing in a cancelled scope even when a slot is free. `spawn` under `limit(0)` always panics. `spawn_isolated` takes no slot.

**Evidence on fairness.**

- tokio: "This `Semaphore` is fair, which means that permits are given out in the order they were requested", with the cost that a large request "can prevent a call to `acquire` from completing, even if the semaphore has enough permits" (https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html).
- Trio's `CapacityLimiter` gives a freed token "to the task which has been waiting longest" (Trio reference-core; read from a summary, not verbatim).
- `Shared::with` already waits FIFO per cell (reference §7.4; fx-nkv.5 comment of 2026-11-10).

**Recommendation.**

- `Tasks` holds a chain of `Arc<Limit>` in derivation order. Each `Limit` has `n`, a live count and a FIFO queue of tickets.
- Order inside a limited `spawn`:
  1. Panic if any count in the chain is zero.
  2. Check cancellation. If cancelled, return the not-started `Task` (below).
  3. Lock the chain in derivation order and take a slot in every count, or take none and queue a ticket in each. Never hold one count while waiting for another, as §6.1 requires.
  4. Wait with `cancellable: true` (waiters under `shield` see no outer cause, per TASK-6).
  5. On wake, check cancellation again. If cancelled, pass any slot on and return the not-started `Task`.
  6. Register and start the child, then take the ordinary spawn scheduling point.
- The child holds its slots until its task finishes. Release them in `run`, after the outcome is recorded and in the same step as the scheduler's task-finish point (fx-nkv.5 decision 3), so release adds no new point.
- Hand a freed slot directly to the head ticket, with no barging. That gives FIFO in both runtimes and the same grant order in real and simulated runs. Under the scheduler the grant only makes the waiter runnable; the seed still chooses who runs next, and it already chose who queued first.
- The not-started child: register it in the scope spawned into. If that scope is cancelled, record `E::from(TaskError::Cancelled)` as its error and call `observe` at the child's location, then `finish` it as an `Error`; the existing rules turn that into an echo, a deadline outcome or an independent failure, as §6.1 specifies. If that scope is not cancelled (the caller sits in a cancelled nested scope), record `Outcome::Echo` and leave the scope alone. Either way call `observe` at the caller's location, drop `f` unrun, and return a `Task` whose `join` gives `Err(TaskError::Cancelled.into())`.
- Simulation wait state: `blocked at limit(id)`, with a simulation-local count id (fx-nkv.5 decision 6).

### 1.8 `spawn_isolated` and `Panicked`

**What the design fixes.** It "runs the `Owned` closure `f` as a new task, waits for it, inherits the caller's cancellation, and returns `Err(Panicked)` if `f` panicked" (TASK-1). The wait is shielded (03 §1.1). It takes no slot. A stack overflow or abort is not contained (PANIC-1).

**Options.** (a) `catch_unwind` on the caller's thread. (b) A new thread whose body runs under `catch_unwind`.

**Evidence.**

- `catch_unwind` "only catches unwinding panics, not those that abort the process", and "It is not recommended to use this function for a general try/catch mechanism" (https://doc.rust-lang.org/std/panic/fn.catch_unwind.html).
- `JoinHandle::join`: "If the associated thread panics, `Err` is returned with the parameter given to `panic!`" (https://doc.rust-lang.org/std/thread/struct.JoinHandle.html).
- Erlang reports a crash to a monitoring process as a message, `{'DOWN', Ref, process, Pid2, Reason}`, and uses that to build "hierarchical program structures where some processes are supervising other processes" (https://www.erlang.org/doc/system/ref_man_processes.html). `Result<T, Panicked>` is the same idea as a return value.

**Recommendation: (b).** The text says "a new task", the `Send + 'static` bound is a thread's bound, and under simulation the task needs its own id and `spawn`/`exit` events.

- Spawn with `std::thread::Builder`; run `f` under `catch_unwind` inside `run`'s existing path, so the baton hand-off stays out of destructors (fx-nkv.5 decision 7).
- Clone the caller's `Context` into the new thread. Checkpoints in the isolated task then observe the caller's scopes, shield included, and set the caller's echo bit, which is exactly TASK-2's "in a task it waits for with `spawn_isolated`".
- The caller waits with `cancellable: false` and wait state `isolated(task)`.
- `Panicked.message`: the payload downcast to `&'static str` or `String`, else a fixed text such as `"Box<dyn Any>"`, which is what Rust's default hook prints.
- Leave the process panic hook alone. It is process-global state, and replacing it races with other tests.

### 1.9 Deadlock report and trace events

**Deadlock.** When `pick` finds no runnable task and no timer, write the report: for each blocked task its id, wait kind (`join`, `scope_exit`, `sleep`, `limit(id)`, `isolated`, later `chan(id)`), and the span of the blocking call. It extends fx-nkv.5's blocked list and is the single report fx-zc1 and fx-o4h share.

Then end the process with a nonzero status. Every task is a parked OS thread with a live stack. Go and madsim can panic from a run loop that sits outside all tasks; Tide has no such place. Waking each task to panic instead would run user destructors that can block again, and a second panic aborts anyway (PANIC-2). fx-nkv.6 decision 4 already attributes an abort to the last started seed. For a plain `#[test]` that calls `Root::sim` in-process, this aborts the test binary after printing the report; record that as a v0 limit.

**Trace events** (NDJSON, fx-nkv.6 decision 1; kinds from 03 §9.6):

| Kind | Operation | Added fields |
| --- | --- | --- |
| every event | | `time`: virtual nanoseconds as an integer (fx-nkv.6 emits 0 today) |
| `sched` | `advance` | `from`, `to`; emitted once per jump |
| `cap` | `clock.sleep`, `clock.sleep_until` | `until`; wait `sleep(until)`; result `ok` or `cancelled` |
| `cap` | `clock.now`, `clock.system_time` | result |
| `cancel` | `deadline`, `explicit`, `failure` | scope id; for `deadline`, the instant |
| `cancel` | `set_deadline` | scope id, effective instant |
| `spawn` | `spawn` under a limit | wait `limit(id)` and wake; result `started` or `not_started` |
| `spawn`, `exit` | `spawn_isolated` | `isolated: true`; exit result `ok` or `panicked` |
| `sched` | `deadlock` | the blocked list, as the last event |

`shield` needs no event: its effect shows in checkpoint results.

## 2. Pitfalls

1. **Timer ties.** Without the class key of 1.3, a sleeper and its scope's deadline at one instant return `Ok` or `Cancelled` by insertion order, against TASK-4.
2. **Time going backwards.** Set virtual `now` only in the jump, and assert the new value is not below the old. Never derive it from a timer that was registered in the past; fire those at registration (1.4, 1.5).
3. **Overflow.** `now + d` near `Duration::MAX` must not panic inside std. Use `checked_add` and treat `None` as "never". User arithmetic on `Instant` keeps Rust's panic.
4. **Deadlines that never fire in simulation.** Time moves only when nothing is runnable. A task that polls with `yield_now`, `checkpoint` or `sleep(0)` in a loop while waiting for a deadline keeps itself runnable, so the deadline never comes. Go's synctest has the same property. fx-nkv.6 decision 7 has no step limit, so the runner's wall-clock timeout is the only stop. Recommend a deterministic guard: count consecutive scheduling points during which a timer is pending and time has not advanced; past a large fixed bound, fail the seed with the yielding tasks' spans. TASK-5 already says the simulator reports a task that runs too long "by step count".
5. **Real blocking under the baton.** A task that blocks in an OS primitive the scheduler does not know (a raw `std::sync::mpsc` receive, a std mutex) holds the baton forever. The existing unit tests in `task.rs` use `mpsc` and `thread::yield_now` this way; they are real-runtime tests and must not be reused under the simulator.
6. **Real time leaking into simulation.** With a simulation present, no std path may call `park_timeout`, `thread::sleep` or the anchor clock. Assert it in the wait primitive. Admitted code cannot call `Instant::now` (FX0501).
7. **Stale timers** from finished scopes or lowered deadlines (1.3).
8. **Lazy real deadlines.** Record the deadline cause before any later cause (1.5), or a child failure that arrives after the instant wins "first event" and the scope reports the child's error where TASK-4 wants `DeadlineExceeded`.
9. **Slot hand-off to a cancelled waiter.** The waiter must pass the slot on, or the count leaks a slot and later spawns block forever.
10. **Nested limits.** Taking the outer slot and then waiting for the inner one is hold-and-wait across unrelated scopes. Take all or none.
11. **Isolated panics and `Shared`.** A panic inside `Shared::with` in an isolated task poisons the cell (fx-nkv.4); the caller survives and later sees the poison. That is intended, but tests should cover it.
12. **Mixed instants.** An `Instant` from a real `Clock` passed to a simulated one compares as a plain offset. Nothing detects it; say so in the docs.

## 3. Cheapest falsifying tests

Each is a unit test in `tide-std` or a driver-built toolchain test. Simulated ones use `Root::sim(seed)`.

| Recommendation | Test |
| --- | --- |
| 1.1 arithmetic | `t + d - t == d`; `earlier.duration_since(later) == ZERO`; `checked_add(Duration::MAX)` is `None`; simulated `system_time()` at start equals the 2000-01-01 offset |
| 1.2 exact jump | One task sleeps 5 s. Afterwards `sim.elapsed() == 5 s` exactly, and the test's wall time is far below a second |
| 1.2 no tick | A loop of 10,000 `checkpoint` calls leaves `sim.elapsed() == 0` |
| 1.3 ties | A child sleeps until `T` in a scope whose deadline is `T`: it returns `Cancelled` on every seed. Two sleepers until `T`: both wake orders appear across seeds, and one seed run twice gives identical traces |
| 1.3 stale timer | A scope with a 5 s deadline exits at 1 s; a later 2 s sleep ends with `elapsed == 3 s` and the trace has no `advance` to 5 s |
| 1.4 | `sleep(ZERO)` returns `Ok` and adds no time; `sleep` in a cancelled scope returns `Cancelled` at once; a real `sleep(20 ms)` measured with the real clock takes at least 20 ms; `sleep(Duration::MAX)` alone ends in a deadlock report, not a panic |
| 1.5 | Deadline at 5 s, children that turn cancellation into `Ok`: `scope` returns the partial `Ok` and `elapsed == 5 s`. Same with a child returning `Err` after cancellation: `Err(DeadlineExceeded)`. A child that fails at 1 s: its own error. `set_deadline` in the past cancels immediately. A later `set_deadline` does not raise the deadline |
| 1.5 lazy real | Real runtime: a 10 ms deadline, then a child that sleeps 30 ms with a plain timed wait and returns `Err`: the scope reports `DeadlineExceeded` |
| 1.6 shield | Inside a cancelled scope, `checkpoint` under `shield` is `Ok` and fails again after it. A scope with its own deadline inside `shield` is cancelled by that deadline. A failed `join` under `shield` still marks the echo |
| 1.6 one primitive | `sleep` blocked in a scope that a sibling's failure cancels returns `Cancelled` without waiting out its duration, on both runtimes |
| 1.7 | `limit(2)` with six children that each record a concurrent count: the maximum is 2. A spawn blocked on a slot in a scope that gets cancelled returns, its closure never runs, and `join` gives `Cancelled`. `limit(0)` spawn panics, also when cancelled. Three queued spawners receive slots in arrival order. A nested `limit(1)` inside `limit(2)` never exceeds either bound |
| 1.8 | `spawn_isolated(|| panic!("x"))` returns `Err` with message `"x"` and the caller continues. A `checkpoint` inside the isolated task in a cancelled scope fails. Under `limit(1)`, a child holding the slot calls `spawn_isolated` and it completes |
| 1.9 | `limit(1)` where the one running child spawns into the same count: the simulated run exits nonzero with both tasks listed at `limit(id)` with spans. This is fx-zc1's deadlock Acceptance |
| Pitfall 4 | A child that loops on `yield_now` until a 1 s deadline fails the seed with the guard's message instead of hanging |

## 4. Where the design is silent or ambiguous

1. **`monotonic_only()`.** Only the tables name it. `system_time()` returns a plain `SystemTime`, so a denied call has no error channel. Recommend that `system_time()` on a monotonic-only `Clock` panics with a message naming the attenuation. Returning the epoch would be a silent wrong answer. The owning text (03 §1.1) should state it.
2. **Reference §7.2 on `sleep_until`.** "`clock.sleep_until(t)` and `rx.recv_deadline(t)` likewise" follows a sentence about `NetError::Timeout`, but `sleep_until` returns `Result<(), Cancelled>`. Read "likewise" as "take an `Instant` from a `Clock`": `sleep_until` returns `Ok(())` at `t`.
3. **`Instant` method set.** The stub marks it a guess. Recommend Rust's full set minus `now` and `elapsed`, as the bead's Scope says.
4. **Zero and past sleeps, and overflow** (1.4): unstated. Recommend the Trio reading.
5. **Whether `set_deadline`, `cancel` and `shield` are scheduling points** (1.5, 1.6): §9.3 does not list them. Recommend no.
6. **Limit fairness** (1.7): unstated. Recommend FIFO with direct hand-off, matching `Shared::with`.
7. **How a deadlocked run ends** (1.9): "the run fails" does not say how. Recommend report then process exit.
8. **Yield loops and virtual time** (pitfall 4): the design gives a step-count report only for tasks without checkpoints. Recommend the guard as a simulator diagnostic.
9. **`Panicked.message` for a non-string payload, and thread-creation failure in `spawn_isolated`**: unstated. Recommend the fixed text of 1.8, and a panic in the caller for the second, in line with 03 §2.3 treating exhaustion as outside every guarantee.
10. **A real sleep that ends after the scope's deadline passed.** TASK-3 says an operation "that completes before the cancel takes effect returns its own result". With lazy detection a late wake reports `Cancelled`. That is within "deadlines are soft", but the text could say which result a late wake gives.

DONE
