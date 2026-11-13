Model: claude-opus-5-5 · Date: 2026-11-13

Verdict: request_changes

Candidate `881d2c0` (one commit on `26ad572`), `git diff HEAD^..HEAD`, read-only.
Five blocking findings. Each has a small fix inside the Later-tier text this
change edits (03 §9.9, research/30), so under the Merge Rule they block this
round, not `fx-djp`. Nothing here asks to reverse any of the ten decided
choices. The fixes close gaps where an implementer could follow the text and
still produce behaviour that differs from Linux or from another reading of
the same text.

Not blocking, checked and clean:
- §9.3 timer precedence: deadline before operation, all due timers first.
- Blocked-only time advance and the separate RNG sub-stream.
- REPLAY-1 inline model data.
- Per-Child cached waits and drop behaviour.
- The SIGCHLD retention fix from r1.
- Raw-mode PTY launch and stdio conflict.
- Session-leader hangup evidence.
- Node crash kills all local processes.
- `design/reference.md` is untouched.
- The Eon citations I spot-checked at `7d1f921`: `orbit/src/pty.rs` try_reap
  and the EIO note; `platform.rs` `set_window_size` and `signal` (group, then
  direct child); `sessions.rs` exact-PID `waitpid` with ECHILD.

## Blocking

1. **Blocking: `ExitStatus::signal() -> Option<Signal>` cannot represent
   real or modeled deaths outside the five supported signals.**
   `design/03-authority-concurrency.md:2180-2184`
   - `Signal` is given as Child, Term, Kill, Winch and Hup, and status
     reports its death signal through the same type.
   - The note's own consequences make other death signals reachable:
     - parent-only death closes pipes, so a child that writes gets SIGPIPE
       (`:2302-2303`, and §1.1's cancelled `output`/`status` names SIGPIPE);
     - on the Linux real profile, any child can die by SIGSEGV, SIGABRT or
       SIGINT.
   - The text is silent on what each side does then:
     - the real backend could return `None`, which then looks like a normal
       exit with no code;
     - it could panic;
     - it could map the death to the nearest variant.
     - A model that declares death by SIGPIPE is either malformed or
       inexpressible.
   - Fix: separate the two sets.
     - The subscribable/sendable set is the five signals.
     - Status reports any signal number, for example
       `signal() -> Option<i32>` as Rust's `ExitStatusExt`, or a
       non-exhaustive `Signal` with an `Other(i32)` arm that `signals()`
       and `Child::signal` reject with `InvalidInput`.
   - State that models may declare death by any signal number, and that
     the broken-pipe consequence of parent loss is SIGPIPE death unless the
     model catches or ignores it.

2. **Blocking: what parent-only death returns, how restart treats it, and
   what it means outside nodes, are unspecified.** `:2300-2313`
   - **`sim.run_until(&handle)`.** It returns the entry's `Result<T, E>`
     (§9.5), but `Proc::exit` or an unhandled Term/Kill on a node's main
     process "stops that entry's tasks" and records the status only in the
     run report. The text does not say what `run_until` returns. Implementers
     will choose among panicking, hanging until deadlock, or inventing a
     value.
   - **`restart(&node)` after a parent-only death with no crash.**
     - §9.5 defines restart after a crash, and "Restart creates a fresh
       incarnation, never reviving children" assumes the children are
       dead.
     - After parent-only death the adopted survivors are alive. Either
       they keep running in the node next to the new incarnation (the real
       systemd-restart case, which matters for Eon), or restart kills them.
       Each reading gives a different trace.
   - **The `#[sim_test]`/`Root::sim` test body.** It runs as task 0 and is
     not a node. The text does not say what `proc.exit`, or `proc_signal`
     aimed at "the node's main process", does there. "Never the host
     harness" rules out one outcome but specifies none.
   - Fix: add three sentences.
     - `run_until` on an entry ended this way returns or reports a distinct
       outcome carrying the code or signal.
     - Whether restart is allowed after parent-only death, and that adopted
       survivors keep running and are not inherited as children (or the
       chosen alternative).
     - What happens in the test body: for example, the run fails with the
       recorded status as a seed failure, or the test body counts as an
       implicit node whose death ends the run.
   - Add a §8 row for each.

3. **Blocking: two signal-state transitions are left open, and the fault
   model makes both reachable.** `:2221-2238`
   - **(a) A subscription dropped while one of its signals is
     kernel-pending.**
     - "Drop removes its interest and latched bits, restoring the default
       dispositions." For SIGCHLD the default is ignore, so the
       disposition change discards the pending bit, which is consistent.
       For Term and Hup the default terminates.
     - `proc_delay(.., delivery)` holds a Term pending across the drop.
       After the drop, the Term either kills the parent (Linux semantics
       when a blocked, signalfd-style mask is lifted with the default
       disposition) or is discarded. The text allows both. Real
       conformance then depends on how the real backend implements
       subscriptions.
     - Fix: state the rule, and pin the real backend's mechanism to match.
       Recommended: restoring a terminating default disposition delivers a
       still-pending signal with that default action, as Linux does. That
       follows the existing "disposition change" wording.
   - **(b) Several latched signals at one `recv`.** "`recv` consumes a
     latch" does not say which one.
     - Linux signalfd returns the lowest-numbered pending standard signal.
       research/30:157 cites signal(7)'s "unspecified" instead.
     - If the simulator fixes an order, reachability of other orders is
       moot. If the seed chooses, the order has to be a declared process
       sub-stream choice, and real checks use allowed sets.
     - Fix: pick one. Recommended: a seed choice among latched signals, so
       order bugs are exposed; real checks accept any order.
   - The fixes for (a) and (b) need a §8 row each.

4. **Blocking: the model's input alphabet lets a child see each resize,
   which undoes Winch coalescing.** `:2196-2199` against `:2252-2256`
   - Model transitions consume "spawn, input bytes/closure, signal, resize
     or timer events", so a model sees a resize event and a Winch signal
     separately.
   - On Linux a child sees only SIGWINCH and then reads the current size.
     Two resizes before delivery produce one handling (`:2254-2256`,
     research/30 §8 row 5).
   - A model that consumes "resize" can emit one output per resize. That
     trace replays but cannot happen on the real profile, and the §8
     falsifier checks only what the parent observes, so it will not catch
     it.
   - Fix: drop "resize" from the model's input alphabet. A model observes
     size only by reading the PTY's current size when it handles a
     delivered Winch, or state that a resize event is the coalesced Winch
     delivery carrying the size at delivery time.
   - Add a planted control: a model that counts resize events must not see
     two.

5. **Blocking (tiering): §9.9 states real-backend rules for v0 surfaces
   without saying which tier they belong to.** `:2172-2186`, `:2228-2231`,
   `:2300`
   - `Proc::signals`/`Signals::recv` are taught in v0
     (`reference-companion.md:615-616`, 03 §1.1's Proc row). `Proc::exit`
     is in v0 `reference.md:339`.
   - §9.9, which is Later, states rules that bind real execution too ("the
     same … in real and simulated execution"):
     - one live subscription, with `AlreadyExists`;
     - drop restores the default disposition;
     - the five-signal set;
     - `Proc::exit` ending a node's entry.
   - It also adds real-backend surface: `Child::signal`,
     `ExitStatus::signal`, `spawn_pty`/`Pty`.
   - 01's new Later row says only "modeled … through Proc", which reads as
     simulation-only.
   - So a v0 implementer of real `Proc::signals` cannot tell whether
     `AlreadyExists` and the five-signal limit are v0 contract. Nor can
     they tell whether `Child::signal` and `spawn_pty` are Later on the
     real backend too.
   - Fix: one sentence each in §9.9 and in the 01 row. Name the API
     additions and the subscription rules as Later on both backends
     (recommended), or name which ones bind v0's real `Proc::signals` and
     `Proc::exit`. v0's companion stays unchanged either way.

## Non-blocking

6. **Minor: Pty I/O and `spawn_pty` are missing from §1.1's suspend and
   checkpoint list.**
   - §9.9 defers to "§1.1 unchanged" for cancellation (`:2277-2279`), but
     §1.1's Proc row (`:93`) lists pipe-end reads and writes, not `Pty`
     reads and writes, `size` or `resize`.
   - Fix: state in §9.9 that `Pty` reads and writes suspend and are
     checkpoints like pipe ends; that `size`, `resize`, `Child::signal` and
     `spawn_pty` are scheduling points but not checkpoints; and that `Pty`
     is exercised kind `proc`.

7. **Minor: the hold/release control has no name.**
   - `:2291` and research/30 §6 (`:363`) say "explicit hold/release" is
     part of `proc_delay`, but no spelling is given.
   - The r1 falsifier (research/30:438) needs release after a reap, so
     release is either an `at(t, …)` scenario with fixed bounds or keyed to
     an event.
   - Fix: name it, for example `proc_hold(target, phase)` and
     `proc_release(target, phase)` under `at`. State that an unreleased
     hold ends in the named deadlock or stall report.

8. **Minor: pending process-model timers and the 1,000,000-point stall
   guard.** `:2274`, against §9.3
   - A `try_wait` spin loop waiting for a future model exit timer never
     advances time.
   - Fix: state that pending process-model timers count as operation
     timers for §9.3's stall guard, so the loop fails with a stall report
     rather than only at the wall-clock backstop.

9. **Minor: the text has two owners for the interleaving choice.**
   - `:2261-2263`: the process sub-stream chooses "signal delivery
     opportunities".
   - `:2267-2269`: ready model transitions "compete with tasks in scheduler
     order".
   - Fix: name one owner, recommended the scheduler's existing choice among
     ready tasks and ready model actions. The process stream then covers
     only model-internal choices: alternatives, delays, short-I/O sizes and
     faults. Also state that runs which spawn nothing draw exactly as
     today.

10. **Minor: FX0514 and `--allow-unmodeled` mean something different for
    processes.** `:2314-2320`
    - §5.2 and RUN-3 text (`:858`, `:1260`) make FX0514 a foreign effect
      that `--allow-unmodeled` lets act on the host.
    - For processes, the flag supplies nothing and no host process starts.
    - Fix: say explicitly that under the flag an unregistered command still
      stops the seed with the coverage report, and the run is only marked
      non-replayable. Also say whether build-time FX0514 process listings
      make `tide sim` refuse to start, as §5.2 does for foreign wrappers.

11. **Minor: the falsifier matrix has gaps.** research/30:434-445
    - Add rows for:
      - a model exit code 143 vs Term death (`code()`/`signal()` differ);
      - master drop leads to Hup, and its default termination;
      - `AlreadyExists` on an overlapping subscription, including through a
        cloned `Proc`;
      - an equal-time scope deadline vs a pending process transition
        (`:2272-2273`);
      - a stale target that hits nothing after restart.
    - Plus the rows asked for in findings 2-4.

12. **Minor: fx-ayh2's current Acceptance does not falsify the contract on
    its own.**
    - Its two bullets do not name:
      - resize/Winch;
      - Hup;
      - parent-only vs node-crash lifetime;
      - coverage failure;
      - REPLAY identity refusal.
    - "Every simulated outcome … reachable by some seed" also misses
      outcomes that only fixed `at` scenarios reach.
    - research/30 §7 covers all of these, so the coordinator must actually
      apply §7 to fx-ayh2 at landing, as the decision says. Treat that as a
      landing step to check, not optional.

13. **Nit: stale mirror in the companion.**
    `reference-companion.md` Appendix A ("From the tier table of 01") does
    not list the new Later item. Add it there, or say why the mirror lags.

14. **Nit: admission.**
    - research/30 §9 (`:456-463`) does not address the overlap between
      `child.kill()` and `child.signal(Signal::Kill)`, which are two
      spellings of one act.
    - Fix, either:
      - have `signal` reject `Kill` (`InvalidInput`), keeping one way; or
      - state why the duplicate is kept.

15. **Nit: status wording that will go stale.** research/30:6-7 and
    `:486-487` say "Sequential reviews remain pending" and "ready for review,
    not approved". These go stale on landing. Move them to the bead or
    reword them as at-decision status.

Scope and rules: read-only, no builds. `trials/e1-checked/` was never opened
or searched. I confirmed Eon claims with `git show 7d1f921:…` only and
changed nothing in Eon. Web sources were not re-fetched. I reused Codex's
r1 spot-checks for them, and the madsim source remains unverified, as Codex
noted.

DONE

## Round 2 re-check

Model: claude-opus-5-5 · Date: 2026-11-13
Candidate `3aa719c`. I read `git diff 881d2c0..3aa719c`, `fix2-msg.md` and
the author's "Fix round 2" map. Scope: findings 1-11 and 13-15 only.
Read-only, no builds. `trials/e1-checked/` was never opened.

Verdict: request_changes

Resolved:
- **1.** `ExitStatus::signal() -> Option<i32>` is separate from the
  five-signal control set. Models may declare any death signal, and SIGPIPE
  after parent-only death is specified (03:2188-2194, 03:2344-2346).
- **3.** Drop with a pending terminating or ignoring default is pinned
  (03:2249-2259), and so is the real signalfd mechanism. Seeded order among
  latched signals is in 03:2235-2239. research/30:157-163 is corrected to
  Linux's lowest-number-first, labelled as mechanism evidence.
- **4.** Resize is out of the model alphabet (03:2207-2210), with a planted
  control.
- **5.** The both-backends Later sentence is in 01:85, 03:2172-2176 and the
  companion's Appendix A. The v0 companion lines are unchanged.
- **6.** Pty checkpoints, scheduling points and exercised kind:
  03:2286-2288.
- **7.** `proc_hold`/`proc_release` under `at`; an unreleased hold ends in a
  deadlock or stall report.
- **8.** Model timers count for the stall guard (03:2308-2311).
- **9.** The scheduler alone owns interleaving; the process stream owns
  internal choices; runs that spawn nothing draw exactly as today.
- **10.** FX0514 refusal is consistent with §5.2. Under the flag, an
  unregistered command still stops the seed.
- **11.** The five requested rows and the rows for findings 2-4 are added
  (research/30 §8).
- **13.** Companion Appendix A mirrors 01.
- **14.** `Child::signal` rejects Kill with InvalidInput; the admission text
  is updated.
- **15.** Status lines are reworded as at-decision status.
- **2.** Restart after parent-only death and the task-0 seed failure are
  resolved. The `run_until` part is not; see R1.

Remaining:

R1. **Blocking (tiering): the Later fix for finding 2 changes the
    signature of the v0-taught `run_until`.**
    `design/03-authority-concurrency.md:2359-2363` (and the §9.5 pointer
    at :2090, research/30's §6 paragraph, its §8 row and the Decision note)
    - §9.9 makes `sim.run_until(&handle)` return
      `Result<Result<T, E>, process::ExitStatus>`.
    - v0 teaches `sim.run_until(&node_task) -> Result<T, E>`
      (`design/reference-companion.md:438`), and §9.5 states that signature.
    - 03 §12.3's worked example relies on it:
      `let acked = sim.run_until(&client).unwrap();` at 03:2658.
    - So once this Later model ships, every v0 test that uses `run_until`
      changes type, and the worked example stops type-checking. That is a
      Later rule rewriting a v0 surface, which finding 5's both-backends
      sentence was meant to prevent.
    - Fix: keep `run_until -> Result<T, E>` unchanged.
      - If the observed entry ends by `Proc::exit` or signal death,
        `run_until` ends the seed as a `sim_failure` carrying the recorded
        code or signal. That matches the task-0 rule, and no `T` is
        invented.
      - Add a separate Later observer for tests that expect main-process
        death, for example
        `sim.run_until_exit(&handle) -> Result<Result<T, E>, process::ExitStatus>`.
      - Update the §9.5 pointer, the research/30 §6 paragraph, the §8 row
        and the Review round 2 decisions note to match.

No other remaining finding.

DONE

## R1 re-check

Model: claude-opus-5-5 · Date: 2026-11-13
Candidate `2ac3bf9`, `git diff 3aa719c..2ac3bf9`, R1 only. Read-only, no
builds; `trials/e1-checked/` never opened.

Verdict: approve

R1 is resolved.
- `sim.run_until(&handle) -> Result<T, E>` keeps the v0 signature.
  - The §9.5 pointer at 03:2090 says so explicitly.
  - At 03:2360-2363, `Proc::exit` or signal death of the observed entry ends
    the seed as a status-carrying `sim_failure`, matching task 0.
- The separate Later observer
  `run_until_exit -> Result<Result<T, E>, process::ExitStatus>` is at
  03:2363-2367.
- research/30 matches on the same split: the §6 paragraph, the §8 row with
  its "change run_until's v0 type" control, and the Review round 2 decisions
  note.
- Nothing else still uses the old type: a grep of design/ and research/ at
  `2ac3bf9` finds no nested-Result `run_until` outside `run_until_exit`.
- §12.3's `sim.run_until(&client).unwrap()` again yields `T`, and the
  companion's v0 line 438 is untouched.

No remaining finding.

DONE
