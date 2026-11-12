Model: gpt-6.1-sol · Effort: high · Date: 2026-11-12

Verdict: request_changes

Re-checked only the five round-1 findings and defects introduced by their fixes, on `0c85fc979e9dd3852864fafa0105d513efa7800a` against `7070c0237bdf27069a7cac93667648b3f32bde7a`. Base remains `dde6c57`. SHA-256 of the fix delta: `7a84a4c85a15a2055036e4256ba56d8c04592b63919a1d21ed7aae0ee34da464`. Findings 1, 2, 3 and 5 are fixed. Finding 4 still blocks because its new signalling mechanism can target a reused process-group ID. There were no rebuttals.

1. **Fixed — Probe errors are no longer accepted as confinement proof.**

   Evidence: `trials/e1-checked/coordinator.py:63–80`, `:117–120`, `:531–554`; regressions at `trials/e1-checked/harness_test.py:881` and `:935`.

   Both the probe and its audit exception use the same predicate. It requires complete, valid absolute-path requests, the matching structured permission-denial frame, the restricted-directory reason, the tool-use identity and matching error text. The malformed requests and unrelated validation errors from round 1 are rejected. Positive inside controls also check the actual control file. The positive and negative synthetic transcript tests pass. The live Claude probe remains an fx-ic2 pre-gate operation, as instructed; its absence is not a blocker for this bead.

2. **Fixed — Exhausting the run budget captures the final source and stops the writer.**

   Evidence: `trials/e1-checked/harness.py:438–447`, `:485–496`; `trials/e1-checked/broker.py:49–52`; `trials/e1-checked/coordinator.py:271–278`, `:349–354`; regression at `trials/e1-checked/harness_test.py:969`.

   The harness captures the source/hash and publishes exhaustion after retaining command 20's results. The coordinator waits for the broker's completion acknowledgement before stopping the writer, and later capture preserves the saved final. The regression exercises a late edit and delayed acknowledgement and passes. The dry controller's refused call 21 after writer termination is a reasonable way to preserve that acceptance observation without allowing the exhausted writer to continue. The four-slot dry-run log reports run_limit, 20 calls and both grades for each slot.

3. **Fixed — Grading infrastructure failures consume the infrastructure retry.**

   Evidence: `trials/e1-checked/harness.py:619–621`; `trials/e1-checked/coordinator.py:367–381`, `:389–403`, `:481–488`; regressions at `trials/e1-checked/harness_test.py:1005` and `:1039`.

   Setup flags propagate through the grader's exit and record. Unexpected exits, missing/malformed records and the guard timeout become infrastructure failures. Attempted grades move to grading_attempt, are withheld from scoring, and enter the existing single-fresh-attempt policy. Both boundary and retry regressions pass. Ordinary grade exit 1 remains a solution failure.

4. **Not fixed — The recovery fix introduces an identity-check/signalling race. Blocking.**

   Evidence: `trials/e1-checked/coordinator.py:191–197`, `:214–219`, `:302–305`; existing identity test at `trials/e1-checked/harness_test.py:920`.

   The parent-death guard and launch grant address the original orphan scenario (`writer_guard.py:15–24`), and the actual synthetic interruption test passes. However, recovery reads a matching /proc identity, releases that read, and then calls killpg with only the saved numeric group ID. If the old group exits and is reaped while recovery is descheduled between those operations, the ID can be reused for an unrelated group before line 196. The saved start time was checked before reuse and does not bind the later signal to the checked process. The current regression covers an identity already reused before the check, not reuse after the check.

   Active cancellation has the same ownership gap: stop_active polls under ACTIVE_LOCK, but a worker can reap the process outside that lock at line 304 before stop_active signals the numeric ID. The process object and dictionary entry do not reserve the ID after reaping.

   A mock-only control returned the original identity snapshot, modelled exit/reaping/reuse immediately afterward, and recorded reclaim_writer issuing SIGKILL against the now-unrelated numeric group. No real signal was sent. This is a safety defect in the added mechanism, not a request to broaden the review.

   Fix: avoid signalling a group based solely on a prior numeric-ID snapshot. For recovery, the smallest option is to rely on the installed parent-death guard, wait for quiescence and refuse recovery if it cannot be proved, without issuing that killpg. Alternatively, acquire and verify a stable process handle and signal the verified guard through it; the guard can terminate its own group. Also serialize active signalling with reaping, or use stable handles there. Add a deterministic regression for reuse between verification and signalling, including concurrent cancellation/reaping. Do not signal an unrelated group under that schedule.

5. **Fixed — tmpfs no longer covers executable binds below /tmp.**

   Evidence: `trials/e1-checked/harness.py:207–211`; regressions at `trials/e1-checked/harness_test.py:534` and `:1086`.

   The tmpfs mount precedes read-only inputs, including the executable, and the separate writable scratch bind follows. The construction regression passes. The author's full 48-test log reports OK and includes the actual compile/execute regression for an external /tmp layout.

Validation: independently ran all nine FixRound1Tests with Python bytecode writes disabled, CARGO_BUILD_JOBS=4, nice -n 19, a 60-second timeout and closed stdin: 9 passed. Read the final full-suite summary (48 tests, 36.816 seconds, OK), both canary logs/records, four-slot dry-run log and shortened-clock log. Both canary records have all three shapes, successful positive controls, all forbidden probes false and all unsandboxed negative controls true; their harness/coordinator/guard hashes match this candidate. The shortened-clock log reports PASS. These existing runtime results are accepted as author evidence and were not rerun. The tracked worktree remains clean at the stated HEAD.

Not rerun: compilation, Bubblewrap, canaries, dry runs or the live model probe. The remaining risk is an unrelated-process signal from finding 4; this candidate is not Codex-clean.
DONE
