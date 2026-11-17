Model: gpt-6.1-sol (high) · Date: 2026-11-17

Verdict: approve

Read-only finder re-check, limited to N10 and regressions in `git diff a10d724 d53c3d1`, with the affected parser, record writers/readers, tests, protocol §9.1, and updated report. HEAD is `d53c3d15dbcba6748c10b8a5f8d3e44e1fb6e0bd`; the worktree is clean. The amendment changes only `trials/e3-pilot/harness/src/main.rs` (38 insertions, 7 deletions). The independently computed SHA-256 of `git diff fab99b3...d53c3d1` matches `7d3dad59b188dd783890e59988c80687f64ec7d511acadc09c51f48e3419e679`.

## N10 — Incomplete finding payloads accepted as complete records

**Fixed.** `check_verdict()` at `harness/src/main.rs:1298` contains the existing post-JSON validation extracted from `parse_verdict()` without changing its checks. Fresh replies retain the program's actual line limit. `valid_attempt()` checks the nested verdict directly with that same helper, and `valid_verdict()` constructs the two-field verdict object for the final-file reader (`:1934–1946`). There is one definition of finding validity.

Both record readers now require each reject finding to be an object with exactly `line`, `item`, `authority`, and `what`; a positive integer line; non-blank item and claim; and a recognized authority. `[null]` and a finding missing `what` fail before the record can count as a completed verdict. Approve still requires an empty list, and reject still requires one to five findings.

**No legitimate current writer output is rejected.** The ordinary attempt writer builds its two-field verdict from `parse_verdict()`'s successful result (`:1851–1852`), so it already satisfies these exact checks. Record readers use `usize::MAX` rather than the original program length: this relaxes the upper bound and cannot exclude a finding that the original parser accepted. The non-verdict branch is unchanged, so ordinary and recovered fault/unavailable/breach records retain acceptance. `conclude()` copies the parsed verdict/findings and preserves the attempt records; acknowledgements add data to the attempt record, not its nested verdict. These paths remain compatible. I also inspected the existing dry-run-5 session-002 verdict: its `/2` approve record and nested attempt satisfy the predicates.

The added regression (`:3160–3183`) exercises `[null]`, a missing `what`, and a valid reject through both the published attempt reader and `verdict_of()`. The existing parser schema tests still cover fresh replies, including their real line limit. The updated report's Round 6 accurately describes this change.

## New findings

None in this amendment. Approval is limited to this N10 re-check. Original finding 3, the live canary, remains open for the coordinator's next step; this verdict does not satisfy that proof requirement.

## Evidence and limits

`git diff --check a10d724 d53c3d1` passes. I inspected the source regression and saved `harness-test-r6b.log`, which reports three suites with 19 passed and the separately invoked private-mode child passing each time. That log is builder evidence, not independently reproduced runtime proof or independent binding of the tested binary to this revision.

Only this requested review file was written. No repository or Beads edits, Cargo, harness execution, or reviewer sessions.

Not checked: runtime behavior or the live canary; finding 3 remains open.
Risk: the outstanding canary still requires the coordinator's proof before it can close.

DONE
