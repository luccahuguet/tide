# Simulated child processes, signals and PTYs (fx-f6xp)

Research and decision, 2026-11-13, drafted by GPT-6.1 Sol.
**[repo evidence]** Baseline: Tide
`26ad57227531349b1bfb939e8ae76d91ebb6b2f8`, clean branch `fx-f6xp`.
**All ten recommendations accepted by the coordinator, 2026-11-13
(fx-f6xp; Decision below). At that decision, sequential reviews had not run.**
No implementation, builds, Beads writes or Eon changes belong to this bead.

Evidence labels apply to each following paragraph, list or table:

- **[repo evidence]**: inspected Tide design, source or read-only bead state.
- **[Eon evidence]**: inspected Eon source or read-only beads at the identity below.
- **[report evidence]**: the named prior reports, not a fresh measurement.
- **[web evidence]**: linked primary sources fetched on 2026-11-13.
- **[inference]**: analysis, recommendations, proposed contracts and untested checks.

## 1. Smallest contract and status

**[inference]** A supervisor written against Tide's Proc capability can
spawn a modeled external child, exchange bytes, resize its PTY, receive
coalesced signal hints and collect its status. Given the same D15 source,
build and model identities, configuration and seed, the simulation produces
the same trace, including faults. A missing model fails coverage explicitly.
The cheapest eventual falsifier is two exited children with one SIGCHLD:
both remain independently reapable, and replay yields identical results.
This is a planned **Later** library/runtime feature, not a new language
construct or a claim that arbitrary subprocesses replay.

**[repo evidence]** fx-f6xp Acceptance requires a reviewed note naming
surface, determinism, faults, tier and follow-ups. The coordinator accepted
the ten recommendations (Decision below); at that decision, the review
condition had not passed.
fx-ayh2 already owns implementation. fx-m0f5 owns crash/durability controls.
fx-vjvb is OPEN: its item 2 promises a runnable supervisor lease-race model,
but its inspected body/comments contain no completed model or result.

**[Eon evidence]** `br --no-db show eon-vux87`, run in
`/home/lucca/pjs/eon/eon`, reports OPEN, with only the cross-project link.
It asks for one lifecycle path as a core that emits effects, seeded
clock/process faults, and planted-bug controls; it does not report which
path was chosen or a completed spike. **[inference]** Do not invent an
outcome or block this proposal on one. The coordinator should relay this
note through desk and require both pending inputs to be reread before
fx-ayh2 freezes its implementation contract.

## 2. Existing owners and Eon pressure

**[repo evidence]** 01 D15 owns replay identities, foreign-effect coverage
and crash/durability semantics; its tier table owns release scope.
03 §1.1 owns Proc's authority and Rust-shaped Command/Child APIs, including
public pipe fields, cached waits, no kill on Child drop, cancellation
retaining a Child, and shielded kill-and-wait for `output`/`status`.
03 §9.3 fixes virtual-time advancement and timer ordering. §9.8 excludes
signals and real children pending this model. Research/26 §A6–A7 motivates
captured inputs and manifest-owned setup; it does not supply a process model.

**[repo evidence]** Current `crates/tide-std/src/lib.rs:288` defines Proc
as an identity with no operations. `sim.rs` constructs that identity and
implements file/process-input setup, not `world.proc`. The scheduler in
`task/scheduler.rs` has separate task/fs/net random streams and orders
timers by `(time, deadline-or-operation class, registration id)`;
`task.rs` fires all due timers before choosing another task. These are
integration owners, not evidence that process faults already run.

**[report evidence]** Read:
`/home/lucca/.cache/eon-agents/tide-research-blaze/report.md` (especially
“Relevance to Eon”),
`/home/lucca/.cache/eon-agents/eon-blaze-eval-work/report.md`, and
`/home/lucca/.cache/eon-agents/eon-desk/tide-in-eon-report.md`.
They recommend taking Blaze's separation of lifecycle logic from I/O and
its fault-oracle ideas, using Eon as a reference workload, and deferring
adoption. They identify Unix sockets/peer credentials and stable build
integration as additional prerequisites. They do not establish simulator
coverage of Eon or Blaze's correctness by execution.

**[Eon evidence]** Read-only Eon HEAD was
`7d1f921427a6c094c85f4a4dd9688f92b2356db4`; `.beads/issues.jsonl` was
already dirty. Relevant observations:

| Owner/path at that HEAD | Observed pressure |
| --- | --- |
| eon-l4e7s, CLOSED; `runtime/crates/eon-runtime/src/sessions.rs:2000–2120`, `:3925` | The old rollback reused an expired startup deadline. Current rollback gets a fresh budget and retains unconfirmed children in FailedStarts. The test explicitly expires lease acquisition, then checks rollback removed the process and artifacts. |
| `sessions.rs:1787`, `:2202`, `:2240` | FailedStarts retains Child/claim pairs; discovery separately uses raw waitpid and accepts ECHILD; tombstone reconciliation waits for exact process and endpoint cleanup. These are distinct ownership paths, not one notification queue. |
| `runtime/crates/eon-runtime/src/supervisor.rs:1986` | A reconciliation failure keeps a terminal for retry and must not end other tabs. Removal follows confirmed completion. |
| eon-awmq.6, OPEN; `orbit/src/pty.rs:337–394` | Historical residual involved reaping about 135 μs after PTY close. Current read handles Linux EIO by trying to reap; a live child can close and reopen the slave. The bead requires judging present contracts, not copying old timing. |
| `orbit/src/pty.rs:444`; `orbit/src/platform.rs:179`, `:453` | Resize sets row/column and pixel dimensions; Orbit's signalling helper attempts the child's group, then the direct child. That is broader than Tide Child::kill. |

**[inference]** Required separations are creation versus readiness, deadline
versus cleanup budget, byte-stream closure versus waitability, notification
versus status, and process death versus storage failure. An opaque child
model can test the supervisor's decisions around these. It does not execute
Orbit's Bend policy, libghostty, arbitrary shell jobs or Unix peer-credential
recovery. Do not describe this first model as satisfying the whole
fx-dett supervisor-port gate.

## 3. Primary-source comparison

**[web evidence] FoundationDB:** its testing documentation describes a
single-threaded simulation of an entire cluster, controlled time and
machine/network failure scenarios including shutdown/reboot. Its external
workload documentation says replay requires the workload not to introduce
nondeterministic behavior, and warns about extra threads.
([Testing](https://github.com/apple/foundationdb/blob/main/documentation/sphinx/source/testing.rst),
[workload determinism](https://github.com/apple/foundationdb/blob/main/documentation/sphinx/source/client-testing.rst).)

**[inference]** Adopt the controlled world and replay discipline. A simulated
database process is not evidence of a POSIX subprocess API, waitpid,
SIGCHLD or PTY fidelity. Tide needs an explicit boundary for those effects;
merely giving a real child the same random seed cannot supply it.

**[inference]** Registering a model asserts an external child's behavior;
the simulator cannot discover drift in a binary it never runs. Replay
identities cover the parent and the model, not execution of that binary.
Real equivalence checks must pin their external fixture/binary identity
separately and check the modeled observations. Record this assumption in
the coverage report, rather than imply that absent foreign child behavior
is automatically detected from a command name.

**[web evidence] turmoil 0.7.2:** hosts execute Futures on one thread;
network hardship can be manual or seeded. Its Sim exposes `crash` and
`bounce` for host software. The runtime implementation cancels tasks by
replacing/dropping Tokio Runtime/LocalSet state.
([Crate](https://docs.rs/turmoil/latest/turmoil/),
[Sim](https://docs.rs/turmoil/latest/turmoil/struct.Sim.html),
[runtime source](https://docs.rs/turmoil/latest/src/turmoil/rt.rs.html).)

**[web evidence] madsim:** its published signal module (labelled 0.2.34)
exposes `ctrl_c`. The inspected source awaits the current node's ctrl-c
notification receiver, and its test uses `send_ctrl_c(node.id())` to
deliver the notification.
([Module](https://docs.rs/madsim/latest/madsim/signal/index.html),
[source](https://docs.rs/madsim/latest/src/madsim/sim/signal.rs.html).)

**[inference]** These are useful precedents for explicit node and signal
injection. Neither inspected interface establishes a child/zombie/reap or
PTY model; this is a limit of the evidence inspected, not an assertion
about every branch. In particular, cancellation/drop of tasks must not be
mistaken for Tide's process crash without destructors.

**[web evidence] Blaze, pinned `7260ad3a…`:** its core state machines emit
tagged Effects and receive tagged Completions. Drivers provide simulated
or real I/O. `sim.rs` supplies program models: a callback on Command and
the simulator RNG returns `Ran { exit, takes }`. It also models delayed,
reordered and failed completions.
([Core](https://github.com/indexable-inc/blaze/blob/7260ad3a8da324b2be3e92759d5c739e9019a38c/crates/blaze-core/src/lib.rs),
[simulator](https://raw.githubusercontent.com/indexable-inc/blaze/7260ad3a8da324b2be3e92759d5c739e9019a38c/crates/blaze-core/src/sim.rs).)

**[inference]** Adopt the lifecycle/I/O split, not Blaze's compiler or std.
An exit-plus-duration stub alone misses the Eon race: it needs independent
close, output, signal and wait transitions. Prefer declarative owned data
over an arbitrary callback for Tide's external-child model, so REPLAY-1 can
embed it completely. The real Tide program stays ordinary capability code;
no application-wide Effect enum rewrite is required.

## 4. POSIX constraints and proposed abstraction

**[web evidence]** Linux standard signals do not queue: repeated generation
while pending coalesces. SIGKILL cannot be caught, blocked or ignored. SIGTERM's
default terminates; SIGCHLD's default action is ignore.
([signal(7)](https://man7.org/linux/man-pages/man7/signal.7.html).)

**[web evidence]** For the supported receivable standard signals, Linux
signalfd returns the lowest-numbered pending signal: its dequeue path calls
`dequeue_signal`, whose `next_signal` chooses the lowest set bit for these
signals. This is specific mechanism evidence, not an unspecified-order claim.
([signalfd source](https://raw.githubusercontent.com/torvalds/linux/master/fs/signalfd.c),
[next_signal](https://raw.githubusercontent.com/torvalds/linux/master/kernel/signal.c).)

**[web evidence]** signalfd subscriptions normally block their signals
from default delivery. Linux disposition changes discard pending signals
when the new action ignores them; unblocking a pending signal allows its
delivery. Rust's Unix `ExitStatusExt::signal` returns `Option<i32>`.
([signalfd](https://man7.org/linux/man-pages/man2/signalfd.2.html),
[sigprocmask](https://man7.org/linux/man-pages/man2/sigprocmask.2.html),
[do_sigaction](https://raw.githubusercontent.com/torvalds/linux/master/kernel/signal.c),
[ExitStatusExt](https://doc.rust-lang.org/std/os/unix/process/trait.ExitStatusExt.html).)

**[web evidence]** A terminated unwaited child retains zombie status.
waitpid with WNOHANG can return zero while a matching child is still running;
ECHILD means the requested child is absent or not waitable by this parent.
An interruptible blocking wait can return EINTR. Explicit SIG_IGN or
SA_NOCLDWAIT for SIGCHLD suppresses zombies, unlike Linux's default action.
([wait(2)](https://man7.org/linux/man-pages/man2/waitpid.2.html).)

**[web evidence]** POSIX leaves the order of reporting multiple children's
statuses unspecified. A wait consumes the status it obtains. Its blocked,
nonqueued SIGCHLD rule requires clearing a pending signal unless another
child's status is available; otherwise, status availability at delivery is
unspecified. This POSIX text does not establish the Linux profile's behavior.
([wait(3p)](https://man7.org/linux/man-pages/man3/wait.3p.html).)

**[web evidence]** Linux's `kernel/exit.c::wait_task_zombie` consumes the
child's status and releases the child without clearing the parent's pending
SIGCHLD. The kernel developers' waitfd discussion explicitly describes and
acknowledges a pending SIGCHLD waking the parent after all children were reaped.
([Linux wait_task_zombie](https://github.com/torvalds/linux/blob/master/kernel/exit.c#L1111-L1227),
[waitfd discussion](https://lists.openwall.net/linux-kernel/2009/01/10/286).)

**[web evidence]** Rust's Child caches successful waits, including
try_wait; try_wait reaps an exited Unix child but leaves stdin open, whereas
wait closes its retained stdin. Drop neither waits nor stops a child.
([Child](https://doc.rust-lang.org/std/process/struct.Child.html).)

**[inference]** Keep a kernel pending signal bit and a runtime latch
separate. The first (Linux) profile retains a pending SIGCHLD across a reap,
even of the last zombie, until delivery/acceptance or a disposition change
discards it. A poll that reports running may be followed immediately by exit;
registration need not recover an earlier hint; one hint may cover several
children; a latched hint may remain after a wait. Status is the durable
fact within the process table. Clients subscribe, scan retained children,
then use hints to trigger rescans. Never require one recv per child, exit
order to equal reap order, or SIGCHLD before a successful wait.

**[inference]** If several signals are already latched at `recv`, choose
their order from the process sub-stream as an internal alternative. This
deliberately explores more orders than Linux signalfd's selection, so real
checks accept any order. The scheduler alone chooses when ready delivery
actions run. On subscription drop, discard its latches and restore defaults;
an ignoring default discards pending signals, while a terminating default
delivers a still-pending signal, even across a delivery hold. The Linux
backend owns a signalfd dispatcher and thread-mask coordination, serializes
deregistration with dispatch, restores `SIG_DFL` then unblocks without
first draining the signal. Parent subscriptions/masks do not leak into exec.

**[inference]** Keep the five-signal typed control set separate from death
status: `ExitStatus::signal() -> Option<i32>` reports any terminating signal
number, and model declarations can name any death signal. A child's write
after parent-only death closes the read end causes SIGPIPE death unless
its model catches or ignores SIGPIPE, in which case it observes broken pipe.

**[inference]** Retain Child-specific waits and one backend status owner.
Raw waitpid(-1) could steal a Child's kernel status and cause ECHILD, as
Eon's separate discovery path illustrates. Do not inject ECHILD randomly
into a valid sole-owned Child or hide a second raw reaper behind a fault.
That race needs an explicit raw/recovery adapter and model before a whole
Eon port can replay it. EINTR below Tide's Child wait is retried; byte I/O
may report Interrupted for callers to retry. This preserves the high-level
contract rather than copying every raw syscall failure into every method.

**[web evidence]** A PTY has master/slave ends, with terminal semantics on
the slave. TIOCSWINSZ sets four unsigned-short dimensions; a changed size
generates SIGWINCH for the foreground process group.
([pty(7)](https://man7.org/linux/man-pages/man7/pty.7.html),
[TIOCSWINSZ](https://man7.org/linux/man-pages/man2/TIOCSWINSZ.2const.html).)

**[web evidence]** Linux's PTY master close invokes slave hangup;
the terminal hangup path signals the session leader. In this proposal's
fixed topology the session leader is also the sole foreground process.
([PTY close](https://raw.githubusercontent.com/torvalds/linux/master/drivers/tty/pty.c),
[terminal hangup](https://raw.githubusercontent.com/torvalds/linux/master/drivers/tty/tty_io.c),
[session signalling](https://raw.githubusercontent.com/torvalds/linux/master/drivers/tty/tty_jobctrl.c).)

**[inference]** Size is terminal state, not a signal payload. Two resizes
may yield one notification whose handler reads the latest size. Resizing
must not signal the supervisor just because it owns the master. Model
normal short ordered byte I/O, slave close/reopen, EIO and exit separately.
Start with one fixed raw-mode PTY/foreground group, and explicitly exclude
termios/job-control mutations and descendants retaining descriptors. Hup
is included for master-drop cleanup; SIGCONT accompanying hangup has no
state effect in a model without stopped jobs. Real conformance must check
this bounded Linux profile; there is no cross-platform fidelity claim.

## 5. Choices for the coordinator

**[inference]** These are the author's alternatives and recommendations,
all accepted in the Decision below, not several APIs to implement:

1. **Tier and admission: v0 / v1 / Later. Recommend Later.** The scope
   includes signals, PTYs and process lifetime outside structured tasks;
   it must not widen the v0 simulator promise or displace E1/demo. Put it
   on 01's Later row and add a D15 boundary only after decision. Proc's
   existing capability remains v0; the extra modeled surface is Later.
2. **Execution boundary: real children / admitted child code / opaque
   model. Recommend opaque models through existing Proc handles.** Real
   execution loses determinism; admitted process entries demand another
   memory/runtime isolation design. The current consumer tests its parent
   decisions with a supplied external-child contract. Preserve the option
   of an admitted-entry model later without promising one now.
3. **Model form: exit/duration stub / arbitrary callbacks / owned event
   tables. Recommend owned event tables.** Separate close, output, exit,
   delivery and reap without host callbacks. Embed all data in replay.
   Use existing Rust builders/types to construct tables, not a new source
   language, public simulator trait or serializable closure system.
4. **Wait ownership: raw PID/wait-any / per-Child. Recommend per-Child.**
   Keep Rust-shaped caches, child-specific waits and drop semantics.
   Preserve code-versus-signal status with `ExitStatus::signal()`; do not
   turn Term into a normal exit code 143. A signal on a collected Child is
   a no-op, never a send to a potentially reused PID, except that Kill is
   rejected with InvalidInput and must use `child.kill()`.
   Exclude Eon's raw competing-reaper/recovered-PID path visibly, rather
   than assert this model proves its correctness. Distinct Child handles
   can be polled in arbitrary order; no global exit-order reaper.
5. **Signals: reliable exit queue / POSIX hints; arbitrary handlers /
   subscriptions. Recommend coalesced hints and subscriptions.** One live
   owner per selected signal avoids a second competing-consumer policy.
   Add child-directed signal for graceful Term and retain Kill's stronger
   meaning. Include Hup for PTY master closure. Do not add raw PID grants,
   realtime signals, thread-directed signals, handlers or job control.
6. **PTY surface: raw descriptors / general slave plumbing / one PTY
   launch. Recommend `spawn_pty(size) -> (Child, Pty)`.** It owns the
   controlling-terminal setup once, carries Proc authority, exposes all
   four size fields and raw ordered bytes, and rejects conflicting stdio.
   No new Root capability, terminal emulator or `Stdio` escape hatch.
7. **Determinism: randomize every ordering / preserve causality and existing
   timers. Recommend the latter with seeded ready-action selection.**
   Reuse the scheduler, including same-time deadline precedence; introduce
   only the process RNG sub-stream and model events. Exact registration
   order resolves event identity; enabled independent actions remain seed
   choices. Busy polling does not make time advance.
8. **Faults: arbitrary errors/crash effects / bounded causal faults.
   Recommend the six families in 03 §9.9.** Signal coalescing, EOF, EIO
   with no slave, and delayed child readiness are normal behavior even
   with faults off. No lost status, fake successful reap, reorder of stream
   bytes, duplicated exit or impossible post-exit mutation. Keep storage
   crash faults with fx-m0f5.
9. **Parent loss: kill everything / child survival / unspecified.
   Recommend explicit survival on parent-only exit and whole-node death
   on node crash.** World-owned instance bookkeeping outlives parent
   tasks, with an abstract system reaper adopting survivors. Parent
   descriptor closure still causes ordinary EOF/broken-pipe/PTY-Hup
   effects; survival is not immunity from them. No restarted parent gains
   wait authority over non-children. Recovery remains a separate model,
   not a new PID constructor in this one.
10. **Coverage: fallback / ordinary I/O error / D15 coverage failure.
    Recommend D15 failure with no implicit host execution.** Distinguish a
    configured spawn failure from absent model coverage. Unsupported real
    platforms return Unsupported; an unmodeled simulated operation ends
    the seed. An allow flag cannot make unknown behavior replayable or
    defeat the test sandbox.

## 6. Proposed model mechanics and reachability

**[inference]** 03 §9.9 is the decided behavioral surface. ProcessModel
is data: an initial state; command-shape expectations; an ordered table of
state/input matches and next states; ordered actions; and bounded delay or
finite-choice data. Input matching consumes ordered byte prefixes,
independent of read/write chunking; unmatched input is a coverage failure.
Spawn arguments and environment are owned OS strings, not lossy text;
effective simulated cwd/environment and Stdio routing are captured in the
spawn event. Models cannot call Fs, Net, clocks or callbacks. Future models
for other effects must declare them, rather than execute secretly on host.
Resize is not in the input alphabet: only delivered Winch handling can read
the PTY's current size, with no per-resize notification or count.

**[inference]** Start each directional pipe/PTY buffer at a provisional
65,536 bytes; record capacity in the manifest. A model output action waits
for room, and a parent write can return a short count. Bytes accepted before
exit remain drainable; an exit discards actions not yet performed and closes
only the exited process's endpoints. Broken pipe is reported when no reader
remains; an external child model with default SIGPIPE dies rather than
continuing after that write. Inherited output uses simulated sinks; File
and pipe-to-child Stdio conversions keep their actual simulated owners. Unsupported routing
must fail coverage, never touch a host descriptor. No cross-stream byte
ordering is promised beyond ordered model actions.

**[inference]** The process table, stream queues, signal pending/latch
state, and PTY size/foreground identity each have one runtime owner. Setup
and snapshot inspection are not scheduling points. Scheduling operations
record invocation, result and source span, with node/instance/endpoint
identity; child model actions also record their originating transition.
A deadlock report names the missing child event, stream room/data or signal
the task awaits. A finite event budget fails zero-delay model loops as a
model stall; it does not pretend completion or advance time. Non-finite or
reversed bounds and dangling state/endpoint references fail setup validation.

**[inference]** Stable candidate ordering precedes seeded selection.
Use the process sub-stream only for model-internal alternatives (including
latched-signal selection), delays, short I/O sizes and faults actually
reached, in recorded transition order; do not draw for deterministic setup, inspection or a
pre-cancelled checkpoint. A process timer fires in the existing timer order
and enqueues a ready model action. The scheduler then picks among ready
application tasks and model actions, including delivery, with no second
process-stream choice of interleaving. Runs that spawn nothing retain the
existing streams' draws exactly as today. The scheduler can expose an exit
between try_wait and signal recv without inventing a timer ordering that contradicts §9.3.
Process creation itself adds no virtual latency/checkpoint; readiness,
termination and model progress can wait on specified timers. Child exit
may occur at the spawn boundary before its caller next runs.
Pending model timers count as operation timers for §9.3's stall guard:
polling a future exit cannot wait only for the wall-clock backstop.

**[inference]** A bounded input example, in semantic table notation rather
than a promised Rust builder spelling:

| Child state/input | Actions and next state |
| --- | --- |
| Spawn | Arm Ready in chosen 4–6 ms; Running. |
| Ready timer | Emit `ready\n`; Ready. |
| Ready + input prefix `stop\n` | Close stdout; arm normal exit in 0–1 ms; Stopping. |
| Running/Ready + Term | Choose declared catch-and-exit-after-0–2-ms or ignore; state as declared. |
| Any live state + Kill delivery | Close endpoints; Zombie(Signal(9)) in the Linux profile; no user transition runs. |
| Stopping + exit timer | Zombie(Code(0)), unless already exited. |

**[inference]** Pair it with parent deadlines on both sides of Ready and
with two children. Different seeds can place closure before waitability,
waitability before a read drains bytes, and multiple exits before a signal
receive. To reproduce an exact-time deadline tie, use fixed bounds rather
than hope to draw it. A model declaring ignore-Term does not make ignored
SIGKILL an allowed alternative. Hold/release is explicit scenario data and
must be finite or end in a named deadlock/stall observation.

**[inference]** Fault API detail for fx-ayh2: target selectors contain
node incarnation plus a program/spawn ordinal (or an already issued exact
instance). Delay phases are progress, notification delivery and termination;
`proc_hold(target, phase)` and `proc_release(target, phase)` use those same
phases under `at(t, fault)`; an unreleased hold ends in a named deadlock/stall
report. Held events remain pending operations for the polling stall guard.
`proc_signal_error` permits denial of Term/Kill submission, not corruption of an existing status. `proc_io_error`
allows Interrupted, BrokenPipe or EIO only where that endpoint/profile
allows them. A configured resize failure is atomic. Fault probability and
delay bounds are part of the inline fault plan; `at` gives a targeted
witness independent of swarm exploration. A stale target is traced as stale
and has no effect on a replacement. No simulated PID is reused in this
first profile; PID reuse bugs therefore remain outside its coverage.

**[inference]** `sim.run_until(&handle) -> Result<T, E>` keeps its v0
signature and returns the entry's result. `Proc::exit` or signal death of the
observed entry ends the seed as a `sim_failure` carrying the recorded code
or signal, as for task 0. Tests expecting main-process death use a separate
Later observer:
`sim.run_until_exit(&handle) -> Result<Result<T, E>, process::ExitStatus>`.
It returns `Ok(entry_result)` on normal return and `Err(status)` on
`Proc::exit` or signal death; neither observer invents a T.
Restart after parent-only death starts a fresh main entry
without a node crash; adopted survivors keep their identities and run in
the same node, but are not its new children. The handle then follows the
new entry. Parent-only death preserves visible storage; whole-node crash
kills local processes and applies durability rules. In the test body
(task 0, outside nodes), `proc.exit` or fatal signal death is instead a
`sim_failure` seed report with the recorded code/signal, never an invented T.

**[inference]** Process FX0514 listings refuse startup without
`--allow-unmodeled`, as in §5.2. With that flag, an unregistered command
still stops the seed with the coverage report, marking it non-replayable;
no implicit real execution is permitted.

**[inference]** REPLAY-1 must include model schema/version, model tables,
initial bytes and sizes, capacities, target profile, selectors, fault bounds
and all injected external signals. D15 identities cover executable model
interpreter semantics too. The first/failing-seed double run compares full
traces, not just final statuses. Reachability is bounded by the configured
model: each allowed finite alternative needs a recorded seed witness;
“many random seeds passed” does not prove it reachable. Parameter changes
produce a different manifest, not a replay against the old one.

## 7. Follow-up text only: refine existing beads

**[inference]** No beads were filed. Refine **fx-ayh2**, retaining its
dependency on fx-f6xp and its required real backend, deterministic process,
signal and PTY I/O, conformance and manifest identities. Suggested acceptance
after the coordinator decides:

- Same supervisor fixture uses Proc/Child/Signals/Pty under Linux and sim;
  model setup is the only test adapter. Child wait/drop/cancellation rules
  from §1.1 remain observable. The library, conformance stub, admission
  classification of handles, conformance mapping and docs agree.
- The contract matrix below has passing observations and deliberately
  broken controls that fail. Every permitted finite outcome has a seed
  witness, with model/fault plan recorded. Fixed ties and holds have explicit
  scenarios. Real checks use allowed sets and causal order, with external
  watchdogs, rather than requiring the simulator's PIDs, timings or counts.
- The existing scheduler handles process events without changing its time,
  cancellation or deadlock rules. Models, faults, inputs and versioned
  profile identities replay inline under fx-nkv.7/REPLAY-1/2; altered model
  or build identity is refused. No-host canaries cover spawn, signals and
  PTY operations, including absent models and stale targets.
- First delivery is the bounded Linux profile specified here, with explicit
  coverage failures for unsupported process modes; it is not the Eon
  adoption milestone. Any staged split retains the parent bead's acceptance
  until all named surfaces and real/sim checks pass.

**[inference]** The size likely warrants child beads under fx-ayh2 when it
is scheduled: (1) common process/pipe ownership plus real/sim lifecycle and
signals; (2) PTY real/sim I/O and resize atop (1); (3) fault/manifest
integration and reachability across both. Each ends in a runnable boundary,
and the parent closes only after the complete matrix. These are possible
implementation units of the existing bead, not a duplicate implementation
epic or permission to create a milestone.

**[inference]** Reconcile other existing owners rather than file duplicates:

| Existing work | Proposed refinement or handoff |
| --- | --- |
| fx-vjvb | Pin Eon, deliver the item-2 lease fixture with explicit peer/deadline events under today's supported sim. Record its lease protocol assumptions; migrate those inputs to Proc only after fx-ayh2, without adding a circular prerequisite. |
| eon-vux87 | Coordinator relays this note through desk, requests the actual chosen path/faults/planted controls when it finishes, and carries useful differences back before implementation. No Eon writes from this session. |
| fx-m0f5 | Keep file-sync/dir-sync planted controls and atomic replacement there. Combine process crash with durable/unsynced records only once its contract is decided; child death alone does not lose filesystem state. |
| fx-dett / fx-ho6o | Keep whole supervisor replay and adoption gates there. Raw reaping/PID recovery, Unix sockets and peer credentials, termios/job control and descendant-held descriptors remain explicit prerequisites where the port reaches them. This note neither implements nor waives those gates. |

**[inference]** A later separate design bead is justified only when the
fixture reaches those raw recovery/job-control gaps with an exact failing
scenario. Do not pre-file a full POSIX emulator, generic process tree API,
new signal DSL or Eon rewrite from this proposal.

## 8. Cheapest falsifiers and planted controls

**[inference]** These are future checks, not executions in this author run:

| Contract observation | Deliberately broken control that must fail |
| --- | --- |
| Two exited children, one hint; both statuses collected exactly once and cached; later wait still returns the same status. | Treat notifications as a counted exit queue; clear all statuses on one receive. |
| Subscribe then initial scan; exit between poll and recv; late latched hint; waits before any hint. | Subscribe only after poll and require one future hint before scanning. |
| Subscribe to Child and hold delivery before exit; reap the last child; release delivery; receive a hint, then observe the same cached status and no additional reap. | Clear kernel-pending SIGCHLD on the last reap, so the later recv blocks, or reap again instead of returning cached status. |
| Close slave/output while child remains running, then exit; also exit with buffered bytes still unread. | Convert EIO/EOF directly into successful reap, or discard buffered output on exit. |
| Two resizes before delivery, one Winch, latest four-field size; no signal on unchanged size or a failed resize. | Route Winch to parent, attach obsolete dimensions as its payload, or update only rows/cols. |
| A model handles two coalesced resizes as one delivered Winch and reads only the latest size. | Feed raw resize events to a counting model, letting it emit two observations before any second Winch delivery. |
| Normal exit 143 has `code() == Some(143)` and `signal() == None`; Term death has no code and signal 15; model SIGPIPE/SIGSEGV deaths retain their actual numbers. | Map signal death to 143, return None for signals outside the five control variants, or reject those model deaths. |
| `run_until` retains `Result<T, E>` and fails the seed with recorded status on entry `Proc::exit` or signal death; separate Later `run_until_exit` returns `Err(status)` for those deaths and `Ok(entry_result)` for normal return. | Change run_until's v0 type, invent a T, confuse an entry error with process death, or wait until deadlock after the entry died. |
| Restart after parent-only death leaves an independent adopted survivor running under its old identity, next to a new main incarnation without wait authority over it. | Turn restart into a node crash, kill the survivor, or inherit it as the new main's child. |
| Test-body task 0 `proc.exit` and fatal signal each end the seed with a `sim_failure` carrying the recorded code/signal. | End the host harness, return success, or manufacture a value for the test. |
| Hold Term delivery while subscribed, then drop the subscription: the pending signal takes its restored terminating default; a restored ignoring default discards its pending signal. | Drain or discard pending Term on drop, keep it blocked indefinitely, or replay a discarded runtime latch as a fresh signal. |
| With Term and Winch already latched, seed witnesses select either recv order; real checks accept any order. | Hard-code lowest-number-first in the simulator and make the other declared alternative unreachable. |
| Drop a live PTY master: Hup reaches the foreground child and its default terminates; a caught/ignored Hup follows its declared model. | Silently close the master without Hup, signal the parent, or force death despite a declared Hup handler. |
| Overlapping subscriptions, including through cloned Proc values, fail with AlreadyExists; nonoverlapping subscriptions remain usable. | Key ownership by Proc clone, accept duplicate ownership or reject disjoint signal sets. |
| A scope deadline and process timer at the same instant fire in the specified deadline-before-operation order before ready actions run. | Let the process action bypass deadline cancellation or run a task between due timers. |
| A fault targeting an old incarnation is stale after restart and hits no new process; a surviving adopted child's exact old identity still names only that survivor. | Retarget by reused ordinal/PID to the new main or a replacement child. |
| `child.signal(Kill)` returns InvalidInput even after reap; `child.kill()` alone submits Kill. | Treat the two methods as interchangeable Kill spellings. |
| A try_wait spin with a future model-exit timer hits §9.3's operation-timer stall guard. | Exclude model timers and wait for the wall-clock timeout instead. |
| Spawn fails without an instance; Term caught/ignored; Kill ends child; submission denial retains ownership; Child drop does not kill/reap. | Always turn successful Term submission into exit, ignore Kill, or discard the last Child after kill denial. |
| Startup consumes its budget; cleanup gets a fresh bound or retains failed child ownership until confirmed reaped. | Reuse the expired startup deadline and drop the only handle on cleanup error (eon-l4e7s class). |
| Parent-only death adopts surviving children and closes the parent's descriptors; whole-node crash removes all local processes; stale faults cannot touch restarted instances. | Kill all children directly on parent exit, ignore resulting Hup/EOF, or reuse old handles after restart. |
| Missing model/unsupported raw process effect gives FX0514 coverage failure, and no host canary starts or receives a signal. | Treat missing model as `NotFound`, success, or call the real backend under simulation. |
| Same identity/seed gives byte-identical event traces; changed inline model identity is refused by replay. | Use host clock/PID or omit model data from the manifest. |

**[inference]** The lease fixture's peer protocol is modeled input, not a
proved Unix transport. The reap-race oracle must use Eon's present lifecycle
contracts to choose its permitted outcomes; the historical extra frame in
eon-awmq.6 is not automatically a Tide failure. Planted controls establish
that an oracle can detect its particular bug, not exhaustive correctness
of a model or a port.

## 9. Admission, owning-document edits and delivery boundary

**[inference]** Low-entropy admission favors this bounded proposal because
it makes lifecycle mistakes mechanically replayable using existing
capabilities, with one owner per process/status and no new grammar or
dependency. Its costs are a model data interpreter, signal bookkeeping,
PTY mechanisms and larger replay artifacts. Restricting it to Later and
keeping raw recovery/job control out avoids turning a language-design
addition into an OS emulator. No performance, implementation feasibility
or learnability claim has been tested here.
`Child::signal(Kill)` returns InvalidInput so `child.kill()` remains the
single spelling for direct-child SIGKILL; the general signal method is
needed for the other control signals, not as a duplicate kill API.

**[repo evidence]** The accepted rules are recorded in 03 §9.9 and the
owning 01 text. Applied Later-row
addition: “modeled child processes, signals and raw PTYs through Proc
(D15; 03 §9.9).” The row also states that the API additions
(`Child::signal`, `ExitStatus::signal`, `spawn_pty`/`Pty`) and subscription
rules (one live subscription with `AlreadyExists`, default disposition on
drop, five-signal set) are Later on both backends. Applied D15 bullet:

> Later: Proc child lifecycle, coalesced signals and raw PTY I/O/size have
> explicit external-child models (03 §9.9). Replay covers only declared
> models and process modes; missing coverage is reported and never silently
> starts a host process. Process death and whole-node crash have distinct
> lifetime and storage effects.

**[inference]** No reference edit is needed for this Later-only decision.
If a pointer is useful when the coordinator holds that slot, propose:

> Planned Later (D15; 03 §9.9): sim.world.proc models external child
> lifecycle, coalesced signals and raw PTY I/O/size. Unsupported process
> effects remain outside replay coverage; this surface is not available in v0.

**[repo evidence]** The author changed 01, 03, this research note and only
Appendix A's Later item in the companion; `design/reference.md`, code,
tests and Beads state were not edited. The coordinator recorded the decision
on fx-f6xp. At the initial decision, the candidate still required the
frozen-candidate Codex/Opus reviews before landing or bead closure; current
review verdicts belong to the review records and bead.

## Decision

**[repo evidence]** Coordinator decision, 2026-11-13, on author candidate
`c1283ec`, recorded in fx-f6xp's bead comment (`br --no-db show fx-f6xp`):
all ten recommendations in §5 are accepted as written.

1. **Later tier.** Preserve E1/demo priorities and v0's current exclusions.
2. **Opaque child models behind Proc.** No real children execute in simulation.
3. **Owned event tables.** Embed them completely in REPLAY-1.
4. **Per-Child wait ownership.** Cache status; dropping Child neither kills
   nor reaps; signal death remains distinct from normal exit code 143.
5. **Coalesced hints and subscriptions.** One live subscription per signal;
   initial signals Child, Term, Kill, Winch and Hup; Kill cannot be
   subscribed to or ignored.
6. **One raw-mode PTY launch.** `Command::spawn_pty(PtySize) -> (Child, Pty)`.
7. **Existing scheduler and timers.** Seeded model actions use a separate
   process RNG stream and preserve causal/deadline ordering.
8. **Six bounded fault families.** Spawn errors, delays/holds, external
   Term/Kill, signal-submission errors, endpoint I/O errors and resize failure.
9. **Parent-loss survival subject to descriptor closure.** Adopt survivors;
   EOF/broken-pipe/Hup still apply. A whole-node crash kills local processes.
10. **D15 failure for missing coverage.** No implicit host fallback.

**[repo evidence]** The coordinator's stated reasons are POSIX fidelity,
determinism without a new source language, and protecting E1/demo through
Later tier. Eon's needs affect fx-ayh2's scheduling, not this tier.
The author applies the decision in 01/03 and this note; the coordinator
applies the fx-ayh2 refinement in §7 at landing. No Beads write is delegated
to this author. At that decision, the next steps were the Codex pass followed
by an Opus round.

### Review round 2 decisions

**[repo evidence]** Coordinator, 2026-11-13, accepted the five blocking and
ten non-blocking findings from the Opus review of `881d2c0`, with the
following choices relayed to the author in this session:

- **1, 14:** Keep five typed control signals, but report/declaratively model
  any death signal through `ExitStatus::signal() -> Option<i32>`. Broken-pipe
  default is SIGPIPE death. `Child::signal` rejects Kill with InvalidInput;
  `child.kill()` is the sole direct-child Kill submission.
- **2 (R1 clarified by the coordinator, 2026-11-13):** Keep v0
  `run_until -> Result<T, E>`; abrupt entry exit ends the seed as a status-carrying
  `sim_failure`, as for task 0. Add separate Later
  `run_until_exit -> Result<Result<T, E>, process::ExitStatus>` for expected
  main-process death, distinct from the normal entry Result.
  Restart after parent-only death preserves adopted
  survivors without inheriting them; task-0 death is a status-carrying seed failure.
- **3:** Subscription drop restores defaults and unblocks pending terminating
  signals, using serialized Linux signalfd/mask coordination. The process
  sub-stream chooses among already latched signals; Linux lowest-number-first
  is evidence for a narrower real mechanism, not the model's order guarantee.
- **4:** Remove resize from model inputs. Only delivered Winch handling
  reads current size; coalesced resizes cannot be counted separately.
- **5, 6, 13:** The additions/subscription rules are Later on both backends;
  v0 real Proc keeps its existing specification. Pty I/O suspends and is a
  checkpoint; size/resize/signal/spawn_pty schedule without being checkpoints;
  Pty is kind proc. Mirror only the new Later item in companion Appendix A.
- **7–10:** Name proc_hold/proc_release under at, ending unreleased holds
  in deadlock/stall; count model timers for the stall guard. The scheduler
  alone owns ready-action interleaving, while process RNG owns internal
  alternatives/delays/short-I/O/fault choices. Unmodeled listings refuse
  startup without the allow flag; even with it, an unregistered command
  stops the seed with a non-replayable coverage report and no host fallback.
- **11, 12, 15:** Extend §8 with the requested falsifiers and planted
  controls; fx-ayh2 refinement at landing stays the coordinator's mandatory
  step, with no author Beads write. Status statements describe the time
  of decision, not a permanent pending-review state.
