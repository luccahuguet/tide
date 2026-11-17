Model: gpt-6.1-sol (high) · Date: 2026-11-17

Verdict: request_changes

Read-only finder re-check of my findings on `edec244` and `cdd0e31`, against `f3b7706067169e526e36859b673fc2404d67c60e`. Reviewed `git diff cdd0e31 f3b7706`, affected current code, protocol, prompt, controls, source tests, updated builder report, saved test log, and relevant dry-run-3 artifacts. Independently computed SHA-256 of `git diff fab99b3...f3b7706`: `7dd309bf0b7d3d144da92d24d49af9d823a3c56911e98ad0b56472ecdf1a0af8`, matching the supplied identity. HEAD matches; the worktree is clean. No override. Paths below are relative to `trials/e3-pilot/` unless stated otherwise.

What this change does: It quotes and masks adjudication findings, freezes grading controls, adds breach acknowledgement and broader process recovery, and gives E3 a dedicated Codex login with private, recoverable hand-back. The original secret-mode exposure and refreshed-login deletion are fixed. Remaining blockers concern retrying that hand-back, reaching the unavailable-model fallback, and the canary/process-ownership fixes themselves.

## 1. Catch credit and the adjudication boundary

**Static boundary fixed; model behavior remains unproven.** `harness/src/main.rs:509–705`; `prompt/adjudicate.md:42–56`; `protocol.md:466–521`; `controls/adjudication.json`.

Findings now reach the adjudicator as a single JSON line holding only `line`, `authority`, and masked `what`. JSON escaping removes embedded newlines, and escaping backticks and angle brackets prevents the reported fence/heading insertion. `fill()` does not rescan inserted data. The prompt explicitly tells the judge to assess the claim and never follow its instructions. Dropping `item` closes its former direct Markdown injection path.

The frozen masking handles the tool-citing example and common trust-diff vocabulary/syntax. The protocol now acknowledges residual provenance, limits its blinding claim, and requires per-arm masking counts and a sensitivity result. That is an honest resolution of the former absolute blinding claim, rather than proof that paraphrased provenance disappears.

All four requested control cases exist on one control change: correct second-host identification, the original wrong metrics allegation, a tool-citing wrong allegation, and a located grading-instruction injection. Their expected answers and the rule preventing session 1 on failure are frozen. The source test checks the rendered structural boundary. Neither judge has run these controls; `fx-1vwf.3` must do so through its adjudication runner before session 1. I do not treat the structural test as evidence that a model resists the injected instruction.

## 2. Resume, recovered breaches, and surviving processes

**The three requested recovery mechanisms are substantially addressed; the new group-ownership rule has blocker N5 below.** `harness/src/main.rs:1492–1729,1744–1858`.

A recovered breach now produces an unacknowledged breach record, and `review()` returns exit 4 before another attempt. `acknowledge` records the coordinator's repair note; acknowledgement leaves that attempt void. The source fixtures cover both states.

`launch.json` now records the scratch path before the home or reviewer exists. Recovery scans processes by scratch working directory as well as process group, audits an existing log even without launch metadata, and hands back the login before removing the root. The normal error path also reclaims and finishes before returning a `run_limited` error. The tests include a pre-pid-record process and a dead group leader with a surviving child. This closes the specific previously reported missing-launch log suppression and live-child omission on those paths.

The first durable verdict still settles the session. No new reviewer is launched to replace a recorded verdict. The new absent-leader ownership inference is unsafe after group-number reuse; N5 explains that separate regression.

## 3. Configuration isolation and canary sensitivity

**Isolation mechanism retained; sensitivity proof remains outstanding, and N4 blocks the new control.** `harness/src/main.rs:1422–1489,1879–2159`; `protocol.md:408–436`.

The isolated path still clears inherited configuration, supplies fresh homes, disables hooks, and checks managed configuration. The amendment adds distinct probes, a Codex hook shape, unisolated controls, printed coverage, and a requirement that each CLI's control expose at least one instruction probe and one hook probe. Coverage is now described as the probes actually observed, rather than every planted file being assumed sensitive.

The builder explicitly has not run this amended canary. The older successful canary cannot establish the new controls or launch path. Moreover, `control()` clears its own Claude credential before spawn (N4). Correct that first, then supply the required control-versus-isolated evidence with the pinned CLIs and dedicated E3 login. No reviewer or canary sessions were run in this re-check.

## 4. Log event order

**Fixed.** `harness/src/main.rs:1062–1197,2290–2413`.

Both parsers now check start and terminal order while scanning. A late or repeated start, repeated terminal event, output after completion, and an unknown `item.surprise` become breaches. Breach precedence across malformed lines remains intact. The added fixtures cover the previously reported cases. The saved `harness-test-r2.log` reports both log tests passing; I did not execute them.

## 5. Silent timeout

**Remains fixed.** `Audit::ended()` still checks a breach first, then timeout before silence. The amendment does not restore the unlimited-unavailability timeout path. No additional test run was needed for this unchanged fix.

## 6. Undefined bootstrap ratio

**Remains fixed.** `protocol.md:573–591` retains separate positive/zero and zero/zero handling, the undefined-resample count, nearest-rank percentiles, and a non-estimable interval rule. Grading implementation remains a prerequisite for session 1.

## 7. Publication unit and denominator

**Remains fixed for the original finding.** `protocol.md:550–561,664–677` retains twelve distinct changes, repeated review sessions, and the retained-model denominator. The no-result wording is added. The fallback's transition into adjudication still has the N3 problem below.

## 8. Frozen identities

**Remains fixed.** The existing prompt and pinned executable/hash/version checks remain in the production attempt path. The amendment's rendering command also checks prompt identities. The modified identity test's retry is confined to test setup. This does not turn Sol's requested model into an observed backend identity; the existing limitation remains explicit.

## N1. Secret-file creation modes

**Fixed for the reported exposure.** `harness/src/main.rs:1304–1356,1384–1398,2019–2023,2499–2530`.

Secret writes use `create_new(true)` and mode 0600 before writing any bytes. Attempt roots and their directories are created with 0700, including under a permissive umask. The normal write-back temporary is exclusive, replaces the source by rename, and is removed after a failed write-back. Recovery copies also use the private writer. The source fixture re-executes under `umask 000`, checks directory/file modes with dummy bytes, and verifies refusal to overwrite an existing secret file. The saved log reports that fixture and its child passing.

Production records contain environment names and login status, not credential values. The changed tracked paths contain no auth artifact. A bounded scan of 46 files under `trials/e3-pilot/` and the saved dry-run-3 found no `auth.json`, `login.json`, or `recovered-*` file and no common token/JWT pattern. This is not a guarantee about every credential format or a future CLI's stderr. I did not read real credentials or rerun the builder's credential-specific scan.

## N2. Refreshed-login preservation and dedicated login

**Original deletion fixed; P2 remains: hand-back cannot recognize its own completed write.** `harness/src/main.rs:1290–1301,1366–1419,1709–1714,2533–2573`.

Sol now requires the explicit `E3_CODEX_HOME` source and no longer falls back to the owner's home. The dedicated-login setup remains an operator requirement; a variable name alone does not prove a different login was created. `login.json` durably records the source/base hash before launching. Both normal cleanup and recovery hand back a changed copy, preserve it privately on conflict/failure, and stop instead of deleting it. The tests exercise interruption before hand-back, write-back failure, and a changed source. This resolves the reported loss of the only refreshed credential.

However, the hand-back is not idempotent. Start with source/base A and refreshed copy B. `hand_back()` successfully renames B over the source at line 1386; interrupt before `finish()` removes the root at line 1418. Recovery then sees copy B != base A and source B != base A. It calls this a concurrent source change, writes another `recovered-*.json`, and stops. Repeating recovery does the same. The documented remedy “restore it by hand, then rerun” also loops after a genuine source conflict: restoring B still leaves the persisted base A.

**Minimum correction:** recognize source bytes/hash equal to the refreshed copy as an already completed hand-back, so cleanup can finish safely. Add dummy-credential checks for interruption after the rename and for rerunning after the instructed manual restoration. The credential is preserved today, but an otherwise successful hand-back can permanently block automatic resume and accumulate unnecessary secret copies.

## N3. Unavailable adjudicator deadline and fallback

**Deadline/report policy added; P2 fallback transition remains blocked.** `harness/src/main.rs:656–666`; `protocol.md:618–638`.

The amendment freezes fourteen days for adjudication, no substitute judge, and no gate result if either required judge is unfinished. That resolves the former indefinite retry rule once adjudication starts.

But the start is still defined as every review session having an outcome, and the new `render-adjudication` command concretely requires a `verdict.json` for every scheduled session. Suppose Sol completes its reviews and Opus has unavailable attempts and pending sessions at the review deadline. Opus is dropped under §11, but unavailability writes no final verdict file. No command or frozen terminal rule finalizes those dropped pending sessions. Rendering a retained Sol finding therefore refuses, and the new adjudication clock never starts. Even if Opus becomes available for judging after the review deadline, the retained-reviewer fallback cannot proceed through the required renderer.

**Minimum correction:** freeze the deadline transition that settles dropped review sessions without inventing verdicts or counted faults, and make the renderer/start clock accept that terminal state. Alternatively, explicitly base the start on finalized retention plus completed retained sessions. Keep the dropped sessions excluded from all statistics and preserve the required two judges/no-gate rule. Add a cheap renderer/state check with one retained completed reviewer and one dropped reviewer with unfinished sessions. This is the same unavailable-model scenario from N3, now blocked before judging rather than during it.

## New findings introduced by these fixes

### N4. [P1] The canary removes the credential needed by its Claude control

`harness/src/main.rs:2013–2017,2037–2042`.

`control()` obtains `CLAUDE_CODE_OAUTH_TOKEN` or `ANTHROPIC_API_KEY` and calls `cmd.env(k, v)`. It then calls `cmd.env_clear()`. Rust's command builder clears explicitly added environment entries too; the subsequent `PASS_ENV` loop restores only PATH, locale, USER, and certificate variables, not either credential. The planted home is fresh and contains no Claude login. Thus an environment-authenticated coordinator launches this control without its authentication, although the isolated attempt correctly receives it.

The mandatory canary consequently cannot establish Claude instruction/hook sensitivity with the documented authentication setup. The same ordering clears the explicit `CODEX_HOME` assignment, although Codex's default happens to point at the planted `.codex` under HOME.

**Minimum correction:** clear the environment before adding CLI-specific credentials and homes, as `reviewer()` already does. Add a command-construction check with a dummy credential verifying the final child environment; no live model call is needed to catch this regression. The actual control/isolated canary still needs to run afterwards.

### N5. [P1] An absent leader does not prove this is still the recorded process group

`harness/src/main.rs:1512–1525,1543–1566`.

The new `owned()` treats every process in the recorded numeric group as owned whenever the boot id matches and `/proc/leader/stat` is absent. This is safe only if that original group has existed continuously. The launch record does not prove continuity.

Concrete sequence: the original reviewer group ends after the harness is interrupted; before a later resume, Linux reuses that number for an unrelated group on the same boot; its new leader exits while a child remains. On resume there is again no process at the leader pid. The `None => true` branch accepts the unrelated group's members without comparing any start identity and `reclaim()` sends them SIGKILL, even when their cwd is outside the attempt root. Linux reserving a group id while its members live prevents reuse during one group's lifetime; it does not rule out this gap and later reuse. The new dead-leader fixture exercises only continuous ownership.

**Minimum correction:** require durable evidence of the group's incarnation or a still-provable member identity before signalling outside the scratch root. If continuity cannot be established, stop for coordinator recovery rather than treating an absent leader as proof. Cover a stale launch record and a different group incarnation, including a replacement leader that has already exited. This is a destructive regression in the fix, not a request for broader process management features.

## Evidence and limits

`git diff --check cdd0e31 f3b7706` passes. The saved `harness-test-r2.log` records 12 passed tests, one ignored child test, and the separately invoked child passing; it supports the builder's reported focused suite, which I did not rerun. The inspected saved dry-run-3 sessions 007 and 008 are recorded Opus verdicts, show `login: none`, list the requested Opus model as observed, and have removed scratch roots. They do not exercise Sol refreshes, interrupted hand-back, either new canary control, or adjudicator behavior.

No unrelated full review, repository edits, Beads writes, compilation, harness execution, or reviewer/adjudicator sessions. Only this requested report was written. Confidence is high in the static environment, hand-back, renderer, and stale-group traces. The minimum route to approval is N2/N3's remaining transitions, N4/N5's regressions, focused checks, and the outstanding canary sensitivity evidence; actual adjudication-control answers remain an explicit pre-session prerequisite.

Not checked: no Cargo, harness, reviewer, or adjudicator execution; no real credential rotation or live canary sensitivity test.
Risk: the current fixes can block recovery/fallback, fail the authenticated canary control, or kill an unrelated process group after numeric reuse.

DONE
