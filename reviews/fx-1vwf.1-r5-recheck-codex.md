Model: gpt-6.1-sol (high) · Date: 2026-11-17

Verdict: request_changes

Read-only finder re-check of `a10d724b5e2a9a67c32d8463c33f8f59a5412fba`, limited to `git diff c97a740 a10d724`, the affected writers/readers and tests, my previous findings, and the updated report. HEAD matches, the worktree is clean, and the independently computed SHA-256 of `git diff fab99b3...a10d724` is `5684438b19e49155248257256d2a980e8394420b2c03d0908f42013f24b2cee5`, as supplied. Paths below are relative to `trials/e3-pilot/`.

The compatibility rebuttal holds. The interruption-table corrections hold, subject to the remaining predicate claim below. One blocking validation remainder is N10. Original finding 3, the live canary, stays open for the coordinator and is excluded from this verdict.

## N8 — Older-run deadline continuity

**Compatibility remainder rebutted; fresh-run fix remains sound.** The coordinator's stated fact is that no real E3 run exists: the earlier directories are development artifacts from unlanded candidates. On that premise there is no supported existing deadline to migrate. Refusing those directories explicitly, without resetting or adopting their state, is sufficient. The protocol records that boundary at §2 and §8.1.

`schedule()` now publishes `schedule.json` once with `schema: tide-e3-schedule/3`. `open_run()` reads only that file and checks its version. A directory with a missing, unparseable, or differently versioned schedule receives the common setup error; no lock, origin, recovery, record publication, CLI lookup, item inspection, or manifest read follows.

I traced every existing-run consumer:

| Command | First access to run state |
| --- | --- |
| `status` | `open_run()` directly |
| `render` | `load_session()` begins with `open_run()` |
| `review` | `open_run()` before the lock and session loading |
| `drop` | `open_run()` before the lock, drop/deadline readers, and settlement |
| `acknowledge` | `open_run()` before the lock and attempt reader |
| `render-adjudication` | `unsettled()` begins with `open_run()`, before drop/verdict readers and the sealed manifest |

`trust-diff`, `render-control`, and `canary` do not open existing E3 runs. `schedule` refuses an existing destination before doing work; it is a creator, not a run consumer. The source fixture invokes all six consumers on `/1` and `/2` run schedules and compares directory/file snapshots, including an interrupted scratch tree. The saved test log reports it passing. The saved dry-run-5 log also reports `status` and `review` refusing dry-run-4 without changing its files. I did not execute these commands.

There is a minor wording qualification: “before any other step” means before any run access or effect, not before CLI usage validation. `main()` parses positive session/finding numbers first, and `drop_model()` checks model/note first (`harness/src/main.rs:107–156,2169–2175`). Thus an invalid invocation such as `e3 drop OLD_RUN opus ''` returns the NOTE usage error rather than OLD_RUN. Neither path opens or modifies the run. This does not reopen N8 or undermine the non-destructive refusal for valid invocations; the literal claim of error precedence is broader than the implementation.

## N9 — Older verdict records

**Rebutted on the same supported-run boundary.** A `/1` verdict belongs to an unsupported development run, and the schedule-version gate stops before `verdict_of()` can classify it as incomplete. There is no need to read or convert it, rerun its reviewer, or discard its files. The first-candidate fixture includes the `/1` verdict shape and exercises every consumer through this refusal.

The known-version reader remains `/2` for newly created `/3` runs. I found no path that reaches an older verdict reader or creates a fresh deadline in an unversioned/older run. No compatibility code is needed under the coordinator's stated premise.

## Interruption table

**The requested transition corrections are satisfied.** The replacement table in report.md's Round 5 distinguishes pre-link from successful-link interruption for schedule, origin, ordinary attempt, recovered attempt, conclusion, acknowledgement, and drop. A complete synced temporary before linking remains uncommitted; a linked record remains committed even before temporary removal, directory sync, or return. The table correctly limits this to process interruption because directory-sync errors are ignored.

The acknowledgement-after-link row now says that settlement merges the committed acknowledgement and may continue; retrying acknowledgement refuses overwrite. The origin is accurately called the committed prelaunch origin, including delays and launch checks that subsequently fail. The attempt rows distinguish prompt/directory creation before the first launch record from root creation after that record, and distinguish an unpublished attempted verdict from a committed deciding attempt. Recovery repeats audit and records interruption; it does not resurrect a temporary's proposed verdict. The older-origin case is replaced by explicit refusal rather than adoption.

The drop rows preserve completed recoveries/conclusions on a later breach or error and distinguish an absent exclusion from a committed exclusion. The report also acknowledges that removal syscalls themselves were not injected. No new interruption defect was found in these edits. The general validation row is correctly conditional on failing the actual predicate, but the separate claim of a “§9.1-shaped verdict” still overstates that predicate, as N10 explains.

## Predicates — legitimate records and incomplete payloads

**No legitimate current writer output found to be rejected; incomplete finding payloads still pass.** `harness/src/main.rs:1926–1943,1972–1982`.

The ordinary attempt writer's successful verdict comes from `parse_verdict()`, so it satisfies the new approve/reject and list-length checks. Its fault, unavailable, and breach records have a string detail and null verdict. Recovery writes those same required fields. Conclusion preserves the validated attempt records, including merged acknowledgements, and writes the expected `/2` outcome/verdict/findings fields. Drops contain a float origin and time, non-empty note, and non-empty positive integer session list. The stricter checks therefore agree with all current writer paths I traced. The older schemas are refused at the run boundary, not accidentally rejected here.

### N10. [P2] A reject with no usable finding is accepted as a complete record

`valid_verdict()` checks only that reject has one to five array elements; it does not check those elements. Consequently this incomplete attempt passes `valid_attempt()`:

```json
{"result":"verdict","detail":"","verdict":{"verdict":"reject","findings":[null]}}
```

A complete current verdict file whose outer and nested attempt findings are both replaced with `[null]` likewise passes `verdict_of()`. This is not a demand to validate unrelated metadata or every key: the missing data is inside the expressly checked verdict/findings payload. §9.1 requires a finding object with line, item, authority, and what; `parse_verdict()` already enforces that for fresh reviewer output.

The effect is observable from the consumers: settlement treats the malformed attempt as deciding and can publish its final verdict; `status` counts the final file as completed, and `unsettled()` permits it through the completion gate. If the finding is rendered for adjudication, `evidence(null)` supplies null line/authority and an empty claim instead of rejecting the incomplete record. A finding object missing `what` has the same acceptance problem. Thus the new “§9.1-shaped” claim in protocol §8.2 and the report is not established, and the requested “no incomplete one accepted” check fails.

**Minimum correction:** validate the finding payload fields needed by consumers, reusing the existing finding validation rather than growing a second schema definition. Keep current ordinary and recovered records accepted. Add a small regression covering `[null]` and an object missing its claim through both the attempt and final-verdict readers, alongside a valid reject. This does not require compatibility support, a complete metadata schema, or extra model calls.

## New findings and review limits

N10 is the only new blocking finding from this re-check. The usage-validation precedence qualification above is non-blocking; all existing-run state access and mutation is version-gated. I found no other new defect in this amendment's schedule publication, refusal, status drop validation, or corrected transition descriptions. N6 and N7's previously accepted publication/settlement mechanisms remain intact.

`git diff --check c97a740 a10d724` passes. I inspected the source tests and saved `harness-test-r5.log` (three suites reporting 19 passed plus the separately invoked private-mode child) and `dry-run-5.log`. These are builder evidence, not independently reproduced runtime proof or a binding of its saved binary to this revision.

Only this requested review file was written. No repository or Beads changes, Cargo, harness execution, reviewer sessions, real credential reads, deletion, or process signalling.

Not checked: runtime recovery, injected crashes, real login rotation, or the live canary; finding 3 remains the coordinator's next proof step.
Risk: the completion gate still accepts incomplete finding payloads as finished review results (N10).

DONE
