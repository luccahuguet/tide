# fx-ic2.11 — Codex supervisor, round 1

Model: gpt-6.1-sol. Reasoning effort: high. Read-only fresh reviewer.
Base: 27921c9. Frozen candidate: b362ac3c0d6aa97b96ddd24432404ad37ea3fc88.
Date: 2026-11-15. No override.

The reviewer returned: **Codex content review: clean. GO to the Opus review
and coordinator-run regrade.** Confidence: high. Formal landing verdict:
`request_changes` until publication proof is complete.

No implementation or factual defect was found. Sampled first/final regrading
remains mandatory before publication, and the report accurately discloses it.
The report must record the actual window, result, matches.json hash and all
12 comparisons, without inferring success from stored-record reproduction.

Scheduling note: report lines 468–495 requested 130 minutes and a 7200-second
runtime, while the latest bead records 16:00–16:45Z. Those guards correctly
refuse the shorter window. Align booking and documented guard consistently.
This is fail-closed, not an unsafe start.

The two new-surface exports STAY: E1_BOOKED_END_EPOCH/QUIET_LOG are needed by
child shells, and REPORT_ROOT/G/E1_COPY/E1_REGRADES by the inner bash -c.
Removing them changes executable behavior and is not text-only.

Checks reported by reviewer:
- Archive contents match 1,266 original files; all 120 usage-only records
  match the last cumulative result events from the original streams.
- Archive/output hashes pass; the member manifest covers all other archive
  members, with safe relative paths and no special files.
- No sk-ant marker in stored contents. Official JSON matches measures-r3.
- Counts, times, turns, tokens, classifications, adjudications and cited
  concurrency/integrity reviews agree with the stored evidence.
- Helper preserves frozen harness/sandbox behavior and compares verdicts/test
  identities, omitting resource/path noise. No extra dependency needed.
- Two non-compiling helper tests, three Bash syntax checks and diff check pass.

No build, grading or duplicate measures run was performed. Material landing
conditions: sampled regrade and remaining Opus review.
