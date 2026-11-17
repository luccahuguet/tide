Model: gpt-6.1-sol (high) · Date: 2026-11-17

Verdict: approve

## Scope and candidate

Read-only finder re-check of `git diff d53c3d1 44fd03f`, its affected callers, readers, writers, tests, protocol, and the updated `report.md` Round 7. HEAD is `44fd03f995a0d549b9482e376c03b5d9acc1a8a9`; the worktree is clean. The amendment changes only `trials/e3-pilot/harness/src/main.rs` and `trials/e3-pilot/protocol.md` (50 insertions, 27 deletions). Independently computed SHA-256 of `git diff fab99b3...44fd03f` matches `fa83558f9c8344a1d02a5dec17b09dfcb408f47fe1c041db4398fac6dfc7a6c9`.

The contract checked is unchanged: the first reviewer launch publishes one immutable origin; dropping a model is refused before the origin plus 14 days; incompatible runs are refused without changing their state. Commands that write run state remain serialized by the existing run lock.

## Diagnosis — confirmed float read-back, not a rewrite

`harness-test-c7.log` fails at the old test's comparison of the second `ensure_origin()` result with the first in-memory result, before its byte comparison. The values are `1794892489.0675385` and `1794892489.0675383`.

I inspected the surviving `/mnt/build/tide-agents/fx-1vwf.1-target/tmp/e3-test-origin-235908/started.json`: its bytes contain the latter value. The old `ensure_origin()` returns the first value directly from memory but obtains the second through `origin()` and JSON parsing. The unchanged `publish()` uses exclusive temporary creation and a hard link that cannot replace an existing target; a second successful publication over that file is not possible through this path.

The pinned serde_json 1.0.154 source and manifest confirm the mechanism. Its default features omit `float_roundtrip`; the selected decimal parser converts the integer significand to a double before dividing by a power of ten. A plain Python calculation of those same operations, `float(17948924890675383) / 1e7`, produces exactly the failed read-back value. `math.nextafter` independently confirms the two values are adjacent doubles, differing by `2.384185791015625e-7` seconds. This supports the report's diagnosis rather than an N8 publication regression.

## Integer deadlines and write-once checks

**Fixed.** `now_ms()` (`harness/src/main.rs:222`) obtains integer milliseconds directly from `Duration::as_millis()`, with checked conversion to `u64`. `origin()` (`:2080`) accepts and returns only a JSON `u64` `at_ms`. `ensure_origin()` (`:2088`) publishes that value only when the record is absent, returns `()`, and propagates malformed-record errors. Its production caller already discards its return value, so the signature change leaves no broken caller.

`drop_model()` (`:2197–2203`) computes `origin.saturating_add(DEADLINE_MS)` and compares `now_ms()` with that integer deadline. `DEADLINE_MS` is exactly 1,209,600,000. Saturation prevents overflow from wrapping into an early deadline. The drop writer and `drop_of()` reader both use integer `origin_ms` (`:2219`, `:2160`). Both expired-origin test fixtures now use integer subtraction.

I traced all remaining time reads and comparisons. No float comparison or subtraction decides a deadline or write-once check. The reviewer limit compares `Duration` with `LIMIT` (`:1782`); reclamation compares `Instant` with its deadline (`:1718`, `:1732`). Float conversions of elapsed time occur only when returning informational seconds. Other float timestamps are recorded metadata; `valid_note()` checks their numeric representation without comparing their values.

The revised `the_origin_is_written_once` (`:3210`) checks the actual stored bytes before and after the second call, then the values read from disk. It retains the recovery-origin and early-drop checks and adds rejection of fractional `at_ms`. It now tests persistence directly without the unrelated float round-trip assumption.

## Run-format gate and documentation

**Consistent.** Scheduling writes the shared `RUN_FORMAT`, now `tide-e3-schedule/4` (`:75`, `:950`). `open_run()` (`:983`) accepts only equality with that constant. Consequently `/3` is refused with the same exit-2 older-candidate error as `/1` and `/2`, before origin or drop records are interpreted.

I traced status, rendering, review, drop, acknowledgement, and adjudication through that gate, before their run mutations or recovery work. Protocol §8.1 names `/4`, and §11 defines the integer `at_ms` origin. The run-format bump covers both changed record fields; no verdict-format change is needed because that schema is unchanged.

The existing old-run unit test explicitly covers `/1` and `/2`. The exact-equality gate also covers `/3`; the saved `dry-run-6.log` reports refusal of `dry-run-5` with exit 2 and unchanged hashes. I independently read the saved schedules: dry-run-5 is `/3`, dry-run-6 is `/4`. The latter's integer origin plus 14 days equals the logged refusal deadline `1796102315993` exactly.

## New findings and outstanding finding 3

None in this amendment. No further patch, alternative design, or non-blocking improvement is required. Confidence is high for this source-level re-check. The updated report accurately describes the fix and its saved evidence.

Original finding 3, the live canary, remains open for the coordinator's next step. This approval covers this amendment only and does not satisfy that proof requirement or authorize closing the finding. No override.

## Evidence and limits

`git diff --check d53c3d1 44fd03f` passes. I inspected `harness-test-r7.log`: three suites each report 19 passed, 0 failed, 1 ignored, with the private-mode child separately passing each time. I also inspected `dry-run-6.log` and the saved origin and schedules. These are saved builder observations, not independently reproduced execution or independent binding of the tested binary to this revision.

Only this requested review file was written. No repository or Beads edits, Cargo, harness execution, or reviewer sessions.

Not checked: independent runtime execution or the live canary.
Risk: finding 3 still requires the coordinator's canary proof before closure.

DONE
