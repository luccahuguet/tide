Model: gpt-6.1-sol / Effort: high / Date: 2026-11-13

Verdict: approve

Finding 1 is resolved. No remaining finding within this targeted re-check.

- `design/03-authority-concurrency.md:2223–2227` now explicitly retains
  Linux's pending SIGCHLD across the last reap, independently of the runtime
  latch. Delivery/acceptance and disposition changes own its removal;
  subscribe-then-scan guidance remains intact.
- `research/30-sim-world-proc.md:170–197` distinguishes the cited POSIX rule
  from Linux behavior, supplies the Linux evidence requested in finding 1,
  and aligns the model explanation with the corrected contract.
- `research/30-sim-world-proc.md:438` adds the requested future falsifier:
  hold delivery before exit, reap the last child, release delivery, receive
  the hint, then observe cached status without another reap. The broken
  control detects the original error.

Scope: only finding 1 and `git diff 164f82d..881d2c0`, plus the author's
“Fix round 1” report and the directly affected paragraphs. The fix is
confined to the identified signal rule, its evidence and its future check;
it does not require restarting the full review.

Candidate: `881d2c051d46d42cc61f046765dc2baf2241759c`.
Fix diff SHA-256 (`git diff 164f82d..881d2c0 --binary`):
`0e59471eb9eabfb59d2d2564e2c2f92427ab3edf4ad5e6d5d194f0ba073379be`.
`git diff 164f82d..881d2c0 --check` passed. Worktree clean.
No override. The required Opus round remains outstanding.

Skipped: builds, runtime checks and unrelated review scope; held-out
trials/e1-checked/ was never opened or searched. Risk: this approves the
design correction; implementation fidelity and reachability remain unproven.

DONE
