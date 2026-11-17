Model: gpt-6.1-sol (high) · Date: 2026-11-17

Verdict: request_changes

Read-only finder re-check of my findings against `c97a740b9e003ef5f296e72ff8a154e80832699a`, scoped to `git diff 1a2a1ab c97a740`, its surrounding recovery/record consumers, the four prior Codex reports, and the updated builder report. Independently computed SHA-256 of `git diff fab99b3...c97a740`: `b1ee1496951ee8f29b05fddece8dfe5abdecbd6044204cc6033673deefc1933c`, matching the supplied identity. HEAD matches and the worktree is clean. No override. Paths below are relative to `trials/e3-pilot/` unless absolute.

The shared settlement and atomic-publication mechanisms fix N6 and N7. The new origin fixes N8 for runs started by this candidate, but existing-run deadline continuity remains broken. New finding N9 concerns rejection of genuine older verdict records. The pending canary is expressly excluded from this verdict.

## N6. Drop must recover attempts before excluding a model

**Fixed for the reported transitions.** `harness/src/main.rs:1975–1992,2122–2155`.

After taking the run lock and checking the deadline, `drop_model()` calls the same `settle_session()` as `review()` for every session of the model. Missing attempt records go through reclaim, login hand-back/root removal, complete-log audit, and publication. An unacknowledged breach stops settlement with exit 4; cleanup/ownership errors stop with exit 2. Neither path publishes a drop. Already deciding attempt records produce their final verdict through `conclude()`. If every session is decided, the model is retained. Only still-open sessions enter the drop's pending list, after all settlements succeed.

The source fixtures cover a durable verdict without its final file, an interrupted live reviewer with a tool-use breach, refusal to drop until acknowledgement, a rotated dummy Sol login handed back before exclusion, and retention when every session is already decided. The saved `harness-test-r4b.log` reports these passing in three suites; I did not execute them.

I found no new destructive ownership inference in this change. Drop reuses the existing `owned()`/`reclaim()` rules, including refusal to signal unknown members under a missing leader. Its exclusive run lock prevents it from treating a concurrent review's live attempt as interrupted. The private secret writers and idempotent hand-back are unchanged, and a failed hand-back still prevents root deletion and drop publication.

## N7. Partial drop publication must not become a committed exclusion

**Fixed for interruption of records produced by this candidate.** `harness/src/main.rs:171–212,2094–2097,2153–2155`.

`publish()` creates an exclusive sibling temporary, writes and syncs the complete JSON, then hard-links it to the final name without replacement. Before that link there is no committed record; afterwards the complete record is visible. A failed second publication preserves the first. Stray temporary names are not consumed as records. Empty/unparseable final files and failed predicates produce setup errors rather than accepted drops.

`review`, `unsettled`, and the adjudication renderer use the new readers. `status` also validates the records it actually consults. The source tests exercise an ignored incomplete temporary, refusal to overwrite a published record, empty-record rejection, and conflicting run mutations. They are synthetic boundary checks, not injected crashes at every syscall.

The original empty/partial-write exclusion is closed. The new verdict predicate has the separate older-record compatibility defect N9. Also, these predicates validate selected state fields, not a complete schema; the report's generic claim about all corrupt records needs the qualification in the interruption-table check below.

## N8. Deadline origin must survive first-attempt recovery

**Fixed for fresh runs; P2 remains for existing runs resumed by this candidate.** `harness/src/main.rs:2020–2032,2050–2054,2132–2138`; `protocol.md:645–648`.

On a fresh run, `ensure_origin()` publishes `started.json` before any reviewer can spawn. The origin is immutable and independent of whether the first attempt later has an ordinary or recovered record. The source fixture verifies preservation after recovery. Publishing before launch deliberately permits an early origin if interrupted between those steps.

However, no earlier candidate wrote `started.json`. An existing run can therefore have genuine attempts and their original `started` timestamps, but `origin()` returns None. `drop` now says “no reviewer has launched in this run” and refuses, even after the original deadline. Resuming an open session instead calls `ensure_origin()` and silently starts a fresh fourteen-day period, forgetting the existing first start. If settlement already decides the session, it returns before `ensure_origin()` and cannot repair the missing origin at all.

This is observable in the supplied older artifacts without running the harness: `dry-run-3` has six ordinary attempt records, earliest `started = 1794888569.1264262`, and no origin file. Its records use the current `/2` verdict schema, so this issue is independent of N9. Before this amendment `first_start()` consumed those timestamps; the amendment removes that path without a transition for existing state.

**Minimum correction:** preserve an existing run's recorded origin when adopting the write-once origin file, and explicitly handle older recovered-only runs where an exact original launch time is unavailable. Do not silently treat existing attempts as a fresh run or require a new reviewer launch merely to enable drop. Add a focused older-run fixture with attempts but no origin, demonstrating that resume/drop preserve its deadline. This does not require changing the fresh-run origin mechanism.

## Original finding 3. Amended canary sensitivity proof

**Open; not a finding against this candidate, as instructed.** The coordinator will run the amended control-versus-isolated canary after this verdict. I ran no model calls. The separately frozen adjudication controls remain the pre-session obligation of `fx-1vwf.3`.

## Interruption-table check

**The main transitions are covered, but the table is not wholly complete or true as written.** I walked the code boundaries independently. The following separates safe transitions from missing or inaccurate descriptions.

| Transition / interruption point | Code trace and assessment |
| --- | --- |
| Publication before opening the temporary, during writing, after writing/syncing but before the hard link | No final record exists. A temporary may remain and is ignored. Repeating record publication is safe. The table omits the explicit complete-and-synced-before-link point; it has the same recovery state. For an uncommitted attempt record, recovery audits the raw log and records interruption; it does not resurrect the ignored temporary's proposed verdict. |
| Publication after the link, before temporary unlink, after unlink, or before directory sync/return | The final record is complete and committed. Subsequent readers use it; they must not repeat the logical transition. The table's generic after-link row is correct. Directory-sync errors are ignored at line 193, so this proves process-interruption behavior, not a power-loss durability guarantee. |
| Empty, unparseable, or predicate-invalid final record | Readers stop when they consult it. Correct for these cases. “Corrupt” is too broad: `valid_attempt()` checks only `result`, and `verdict_of()` only schema/outcome. For example, a `/2` object with `outcome: "verdict"` but no verdict/findings still passes that predicate. `status()` also does not inspect a model's drop record when that session already has a verdict. The table must not claim comprehensive schema validation or that every consumer reads every record. |
| Lock acquisition, session-directory creation, interruption before the origin/attempt | No reviewer exists yet; repeat is safe. The lock inode remains on disk but is not a persistent ownership marker. On Linux the Rust file implementation opens with `O_CLOEXEC` and uses nonblocking exclusive `flock`; closing the holder on crash releases the lock, and an execed reviewer does not retain it. A fork before exec can briefly retain the description, explaining the retry. No new stale-lock-file deadlock found. |
| Origin before/within/after publication | Before the link, no origin; afterwards, the same origin on every retry. Correct for fresh runs. The table is missing adoption of older runs without an origin (N8). “Slightly early” is not a bound: an interruption can last arbitrarily long, and `ensure_origin()` also precedes `attempt()`'s CLI/login/managed-config checks, so even a refused launch can leave the origin. Describe this as the committed prelaunch origin, without promising a small offset. |
| Attempt directory/prompt creation before initial `launch.json` publication | No root or child exists yet. Recovery audits any existing log and publishes an unavailable interruption when there was no output. The table's combined attempt row incorrectly implies that the scratch path is already recorded at every listed point; it is recorded only before root creation. This earlier point is safe for that reason. |
| Root/home/login creation, before spawn | `launch.json` already identifies the root. Recovery reclaims nothing, finishes/removes the partial home, and audits the log. A partially created login has not been used by a reviewer yet. No new secret-file exposure introduced by this round. |
| Spawn before pid metadata update; interruption during the metadata update or reviewer run | The initial launch record already names the root; cwd/marker ownership can recover the child before pid metadata is present. The update uses replacing atomic `write_json()`, so the initial or updated launch record remains. Existing unknown-group handling still stops safely. The table's reference to the earlier mechanism is accurate with this split. |
| Reclaim before hand-back; interruption during hand-back; interruption during root removal | The existing hand-back runs before removal. An already replaced source is recognized, an unresolved conflict preserves the root/private copy, and partial deletion is safe after completed hand-back. The report's recovery rows are correct; their cited fixtures do not simulate every deletion syscall. |
| Root removal/log audit before `attempt.json` publication | Recovery can rerun reclaim/finish and audit the same log. Before the publication link it records interrupted fault/unavailability/breach; after the link it uses the committed attempt. This must be distinguished from the deciding-record boundary in the next row. |
| Deciding attempt publication before `conclude()`, or interruption during final verdict publication | Once the attempt link exists, settlement reconstructs the same terminal outcome without another reviewer. Before the verdict link, repeat conclusion; after it, the session is already complete. Correct. |
| Acknowledgement before its link | Breach remains unacknowledged, next settlement exits 4, and acknowledgement can be retried. Correct. |
| Acknowledgement after its link but before unlink/sync/return | **The table's “before or within publish = breach unacknowledged” row is wrong.** The acknowledgement is committed, `settle()` merges it, and the session can continue. Retrying `acknowledge` refuses to overwrite it (setup error), rather than creating it again. No data-loss defect; this is an inaccurate interruption description. |
| Drop before deadline, between session settlements, on a settlement error, or before its final link | No exclusion is committed. Completed recoveries/conclusions remain durable, and retry resumes settlement. A breach/error can leave earlier records published, but no drop record. Correct. |
| Drop after its final link, before return | Complete drop already committed; later drop/review returns exit 3. Correct. A model with all sessions decided gets final files and no drop, as reported. |
| Schedule construction | Unchanged and correctly identified as requiring coordinator cleanup/restart after an incomplete schedule. It is outside the new terminal-record primitive. |

Thus the table needs the acknowledgement correction, the explicit pre-link/successful-link split at each publisher, the older-origin transition, and narrower wording about corruption and prelaunch state. The safe ordinary drop/recovery rows do not require a different mechanism.

## New finding N9. [P2] The validator rejects genuine older verdict records

`harness/src/main.rs:1936–1940`, consumed by `status()` at `:1018`, `review()` at `:2045`, and settlement/adjudication.

`verdict_of()` accepts only `tide-e3-verdict/2`. The original harness at `edec244` legitimately wrote `tide-e3-verdict/1`; the supplied `dry-run-1/sessions/001/verdict.json` and `002/verdict.json` are real examples, both with `outcome: "verdict"`. I read only their nonsecret record fields and independently evaluated the new predicate: both fail solely because of their schema version. Before this round, `status()` accepted their outcome and `review()` recognized their existing final file. The new shared reader now labels them “not a complete record: resolve it by hand” and aborts. This is rejection of supported older output introduced by validation, rather than detection of an interrupted write.

**Minimum correction:** recognize known older verdict records for the operations that can consume their existing fields, or provide an explicit non-destructive compatibility conversion without rerunning a reviewer or discarding its verdict. Continue rejecting malformed/unknown records. Add a fixture drawn from the actual `/1` shape alongside the `/2` and invalid cases; do not infer incompleteness merely from the known older schema identifier.

## Evidence and limits

`git diff --check 1a2a1ab c97a740` passes. The saved final test log reports three suites with 18 passed, one ignored child entry, and the separately invoked private-mode child passing. The saved dry-run-4 log and nonsecret JSON show the `/2` Opus approval, an origin preceding its attempt, repeated-review exit 3, pre-deadline drop exit 2, and status with 23 pending sessions. I inspected this evidence; I did not execute or independently bind the saved binaries to their reported revision.

Only this requested review file was written. No repository/Beads edits, compilation, Cargo, harness, reviewer/adjudicator/canary sessions, real credential reads, deletion, or process signalling. Confidence is high in the static publication/settlement traces and the observed older-record incompatibilities. The minimum route to approval is existing-run origin continuity, known older-verdict compatibility, and the report-table corrections above; the live canary remains the coordinator's next proof step.

Not checked: runtime recovery, crash injection, real login rotation, and live canary sensitivity.
Risk: older runs can lose their deadline continuity or become unreadable despite having legitimate completed records.

DONE
