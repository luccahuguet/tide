Model: gpt-6.1-sol · Effort: high · Date: 2026-11-12

Verdict: approve

No blocking, non-blocking, or nit findings. Confidence: high from static inspection; I did not independently execute the reported focused checks.

Reviewed frozen candidate `f6f14ad5b791fedc26951dc42bd1e92b8a1b3515` against `d02ccf539f699785b4f5beb2592f2f706e74d8e6`. Binary diff SHA-256: `49e451116125306c3362ad6632bc0651738e55d09e0c9001cf10a902fdeecc82`. The worktree was clean at the beginning and end of inspection. This approves this candidate's review; the planned rebase delta, coordinator-run suites, and subsequent Opus review remain outstanding.

What this change does: It supplies one owned MPSC queue for real and simulated channels, with bounded/rendezvous sends, cancellation-aware blocking operations, deadlines, and iteration. Channel waits and deadlocks use the existing task runtime; production definitions also replace the conformance placeholders.

Review conclusions

- Checkpoints and ordering agree with decisions 1–7 and the superseding lock-order comment. Every bounded send checks before enqueueing; receive checks cancellation before taking a value, then checks disconnect before timeout (`crates/tide-std/src/sync/mpsc.rs:110–165,294–334`). Unbounded sends and try operations do not observe cancellation. Each simulated send/receive takes its entry point before inspecting state; creation, clone and drop take none.
- Queue ownership is clear. The channel mutex owns buffer, pending tickets, receiver registration and endpoint counts. Both send forms reject barging. Receiving promotes the oldest pending sender into newly freed bounded capacity; zero-capacity blocking sends remain pending until consumed (`mpsc.rs:52–74,121–135,194–216`). Removing a cancelled pending sender cannot strand usable bounded capacity: pending senders exist behind a full buffer, and receives promote a head atomically. Zero-capacity sends need a receive.
- The shared wait remains the owner of parking, timers and cancellation wakeups. Deferred mode tests the runtime's existing cause without setting an echo; channel completion marks an echo only if it actually returns cancellation. An accepted send checks ticket disappearance before observing cancellation (`task.rs:79–95,553–634`; `mpsc.rs:140–160`). Existing wait callers retain their former Checkpoint/Shielded behavior, including limits, sleeps, joins, scope exit and isolation.
- I found no lock inversion or lost wakeup. Operations take runtime then channel; they release both before a point or wait. Signals retain readiness across registration/parking, and the real wait rechecks it under the runtime lock before parking. Simulation has no point between resource registration and blocking; resource mutations call the existing wake_waiters owner (`task/channel.rs:39–71`; `task.rs:553–634`). Cross-runtime real wakes use the resource's Signal rather than assuming the sender and receiver share a Runtime.
- Disconnect and destruction follow the recorded contract. Last-sender drop wakes the receiver, which drains accepted data; receiver drop marks it closed and wakes every pending sender. Buffered values are moved out of state and destroyed after both locks are released, including during unwind (`mpsc.rs:99–107,254–269`). Normal operation moves or returns payloads without running their destructors under these locks. Drop takes no scheduling point or cancellation observation.
- Simulation identity and channel IDs follow the existing Context/Scheduler owners. Channel waits extend the existing global deadlock report, not a separate detector; trace fields contain operation metadata and never T (`task/channel.rs:11–36,74–121`; `task/scheduler.rs:410–453,673–710`).
- Admission changes are limited to two data-only try-error path families and their assertions (`crates/tide-admit/src/policy.rs:198–200,331–339`). The host channel constructors remain unmatched. The stub includes production sync.rs/mpsc.rs and deletes its separate placeholder implementation. The touched README, crate docs, design §6.4 and conformance documentation describe the implemented surface consistently with the recorded decisions.

Judgment of the report's nine additional technical decisions

1. Accept the private task/channel.rs adapter. It follows the existing runtime-adapter arrangement and contains access to runtime state and lock ordering without exposing those internals to sync.
2. Accept Cancellation::Deferred. It is needed for the accepted-send success rule and preserves the single wait owner. It avoids clearing an echo that another checkpoint may legitimately have set.
3. Accept retained-runtime close with task 0 when no matching current task exists. Drop performs no domain validation or point, can close after simulation teardown, and releases locks before destroying buffered payloads. Ordinary operations still validate the simulation domain.
4. Accept try_send only on SyncSender. This follows the adopted Rust surface; the scope's method list does not require an extra unbounded method.
5. Accept unconditional Debug/Display/Error on SendError<T>. Formatting exposes the reason without requiring or printing T; the four value traits remain conditional.
6. Accept the two narrow admission entries. Retaining Rust's try-error enums keeps one data definition and does not admit host synchronization constructors.
7. Accept try_iter using try_recv. Research §1.6 explicitly distinguishes it from iter/IntoIterator, and the new normative text records that reading of decision 6's shorthand.
8. Accept the single accepted hand-off value for a successful zero-capacity try_send. It requires a registered receiver, excludes a second accepted value, respects pending senders, and preserves cancellation-before-message when the receiver resumes. Blocking rendezvous sends still wait until consumption.
9. Accept the owned VecDeque implementation without another dependency. Both runtimes need the same cancellable queue and runtime wait path; the rejected host blocking APIs do not provide that integration.

Proof and limits

The driver tests build and execute the shared channel fixture on real and simulated runtimes in debug and release; they separately execute the mixed send/receive deadlock and inspect the shared terminal report (`crates/tide-admit/tests/build.rs:782–836`). The fixture exercises blocking full sends through deadline cancellation, successful delivery after capacity is freed, iteration disconnect, and cancelled receive with the buffered value retained (`conformance/cases/tasks/channels.rs:39–133`). Unit tests add controlled FIFO/rendezvous cases, real cross-runtime wakeups, queued cancellation, accepted-send echo attribution, reentrant payload destruction during unwind, identities and exact simulated timeout (`crates/tide-std/src/sync/mpsc/tests.rs`). Existing task-control and simulation tests cover the unchanged wait modes and consumers. I found the combined checks sufficient for this candidate's risk; the implementer's passing results remain reported evidence, not fresh execution by this reviewer.

Independently verified that research/21-channels-tcp.md, after its provenance header, is byte-identical to the supplied research report. Read its channel recommendations and ambiguity discussion, the named beads and binding comments, AGENTS.md, GOVERNANCE.md, reference §7.3, relevant TASK/channel rules, all changed files, and surrounding runtime/admission code. `git diff --check d02ccf5..f6f14ad` passed.

No focused probe ran: at the resource-gate check, epoch 1794469261 was before quiet-until 1794470707, with an active booked Eon window. No full-workspace build or heavy.sh ran, and no held-out file was opened or searched. No repository or tracker state was changed.

Not checked: execution by this reviewer, the future rebase, and full suites. Residual risk is unobserved concurrency schedules; no concrete correctness defect was found. DONE
