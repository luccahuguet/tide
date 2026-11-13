Model: gpt-6.1-sol / Effort: high / Date: 2026-11-13

Verdict: request_changes

Confidence: high. One blocking semantic finding; no additional nits.

1. **Blocking — retain pending SIGCHLD across a reap in the Linux profile.**
   `design/03-authority-concurrency.md:2223–2225` unconditionally clears a
   kernel-pending SIGCHLD when the last zombie is reaped. This differs from
   Linux, the real profile selected at `:2320`: collecting wait status does
   not itself consume the parent's pending signal. The POSIX rule cited at
   `research/30-sim-world-proc.md:170–174` does not establish Linux behavior.

   Concrete case: subscribe to Child; hold notification delivery using the
   proposed delivery-phase fault; let a child exit; collect its status with
   try_wait; then release delivery and receive. The specified model has
   erased the hint, so recv blocks (or reaches a deadline/deadlock), whereas
   Linux can still deliver SIGCHLD with no remaining waitable child. The
   separate runtime latch does not cover this case: delivery was held until
   after the reap. This changes the causal behavior that the proposed fault
   model is meant to exercise.

   Evidence: the current Linux
   [wait_task_zombie implementation](https://github.com/torvalds/linux/blob/master/kernel/exit.c#L1111-L1227)
   consumes status and releases the child without clearing the parent's
   pending SIGCHLD. The distinction is also explicitly described and
   acknowledged in the original
   [kernel developers' waitfd discussion](https://lists.openwall.net/linux-kernel/2009/01/10/286).
   The cited [POSIX wait text](https://man7.org/linux/man-pages/man3/wait.3p.html)
   describes its blocked-signal rule; it must not be substituted for the
   selected Linux mechanism.

   **Requested fix:** make the first Linux profile retain a pending SIGCHLD
   until delivery/acceptance or a disposition change discards it. Keep the
   latch/status distinction and subscribe-then-scan guidance. If a separate
   POSIX-clearing profile is wanted later, qualify that rule by profile.
   Add a future falsifier to research/30 §8: hold delivery, reap the last
   child, release delivery, and observe a hint followed by cached status
   and no additional reap. Align the research explanation with this rule.
   This changes Later-tier semantics, so it needs this reviewer's targeted
   re-check rather than being treated as a wording-only correction.

Review scope and evidence:

- Frozen candidate: HEAD `164f82d16f5450c9c438dae7e05a48170fc3fe2e`, parent
  `26ad57227531349b1bfb939e8ae76d91ebb6b2f8`; reviewed `git diff HEAD^..HEAD`.
  Binary diff SHA-256:
  `9dbce02101428043fa3827941a194a7bdfe249dde7e853d0219aff6bd1c8e4d5`.
  Worktree clean before and after review; diff whitespace check passed.
- Read the author report including its decision-applied amendment, bead
  Acceptance and 2026-11-13 05:07Z decision, all three changed documents,
  GOVERNANCE.md's admission filter, existing Proc/cancellation rules,
  simulator scheduler/timer/crash/coverage/REPLAY-1/2 rules, D15, tier table,
  and relevant reference/companion text. Checked current scheduler ownership
  and Proc's unimplemented identity without building.
- Spot-checked Eon file:line claims using git show at the note's pinned
  `7d1f921427a6c094c85f4a4dd9688f92b2356db4`, since Eon's current HEAD has
  advanced. The lifecycle, cleanup-budget, PTY EIO, resize and signalling
  claims checked there are supported. Eon was not changed.
- Primary-source spot checks covered Rust Child wait/cache/drop behavior,
  Linux/POSIX wait and signals, window-size ioctls, Linux PTY hangup source,
  pinned Blaze process callbacks, FoundationDB simulation and turmoil
  task-drop behavior. The madsim source URL was inaccessible through the web
  tool; that comparison is not independently verified here.
- No other blocking issue found in ownership, signal-versus-code status,
  raw PTY launch/resize, timer precedence, RNG/replay identity, bounded fault
  families, parent loss versus node crash, or coverage boundaries. The
  Later placement preserves v0 and passes the admission filter as a bounded
  library/runtime addition; design/reference.md is unchanged.

No override. Minimal patch for approval is finding 1. The subsequent Opus
round remains required after the Codex-clean candidate is frozen.

Skipped: builds and runtime checks, as requested; held-out trials/e1-checked/
was never opened or searched. Risk: model fidelity and reachability remain
unproven until implementation and the proposed real/sim checks.

DONE
