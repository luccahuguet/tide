Model: gpt-6.1-sol (high) · Date: 2026-11-17

Verdict: request_changes

Read-only finder re-check of N2–N5 against `1a2a1abb4c161cf08fd83723426c6715de3518f4`. Read the three prior Codex reviews, `git diff f3b7706 1a2a1ab`, affected code and source tests, protocol changes, the updated builder report, and saved `harness-test-r3.log`. Independently computed SHA-256 of `git diff fab99b3...1a2a1ab`: `db37da0a2ac806d94127e6f5c1736a17a731ef90812b8d3c593357949851d78b`, matching the supplied identity. HEAD matches and the worktree is clean. This is not a new full review. No override. Paths below are relative to `trials/e3-pilot/`.

## N2. Idempotent refreshed-login hand-back

**Fixed for the reported cases.** `harness/src/main.rs:1370–1443,2913–2976`; `protocol.md:229–240`.

Before treating a changed source as a conflict, `hand_back()` now recognizes source bytes equal to the refreshed copy as a completed hand-back. This permits cleanup after interruption following the rename and after the documented manual restoration. On an unresolved conflict it recognizes an identical existing recovery copy instead of creating another secret file on each retry. A failure still retains the scratch root; new recovery copies still use exclusive 0600 creation. The source fixtures cover both successful recovery cases and repeated unresolved recovery with exactly one kept copy, using dummy bytes.

I found no new secret-write exposure in these changes. This is static inspection, not a real credential-rotation test.

## N3. Dropped-reviewer sessions and adjudication start

**The ordinary fallback transition is fixed; its new recovery paths have N6–N8 below.** `harness/src/main.rs:658–672,1864–1872,1936–2015,2833–2875`; `protocol.md:630–658`.

`drop` introduces an explicit terminal exclusion after the review deadline, requires a note, and refuses a model whose sessions all have final records. `unsettled()` accepts dropped sessions without inventing verdicts or faults; the renderer therefore permits retained findings once all other sessions are finished or dropped, and refuses findings from a dropped model. The protocol starts the adjudication clock at that transition and retains both judges and the no-gate deadline rule.

The added fixture covers one retained finished reviewer and one partially finished dropped reviewer with a recorded unavailable attempt. That resolves the original normal-path renderer deadlock. It does not cover interrupted attempts, interrupted drop publication, or a recovered first attempt, which expose the new problems below.

## N4. Canary control authentication survives environment clearing

**Fixed.** `harness/src/main.rs:1450–1544,2176–2243,2764–2830`.

Both launch paths now use `clean_command()`, which clears the environment before adding the allowlisted variables, HOME, attempt marker, and model-specific login. No later `env_clear()` removes the Claude credential or explicit CODEX_HOME. The isolated Opus settings and XDG/TMPDIR assignments remain on the reviewer path. Production callers obtain the credential through `claude_credential()` and pass it to the builder.

The source fixture executes an environment-printing script for reviewer/control commands for both models, using a dummy credential, and checks the final environment. It also checks refusal without an Opus credential. This directly targets the original ordering defect without a model call. The control now attempts reclaim and hand-back before propagating a `run_limited()` error.

## N5. Ownership before destructive reclaim

**Fixed for the reported stale-group incarnation.** `harness/src/main.rs:1567–1652,2690–2752`; `protocol.md:356–372`.

The absent-leader branch no longer authorizes signalling every member of the numeric process group. Processes need a working directory in the attempt root, the inherited unique `E3_ATTEMPT` marker, or membership while the recorded leader identity is still present. Unmarked members under an absent leader are classified as unknown; reclaim stops for coordinator inspection without signalling those unknown processes. A different live process occupying the recorded leader pid does not establish group ownership.

The new source fixture checks a stale start identity with an orphaned unrelated group, verifies reclaim/recovery fail and the child survives, checks an absent original leader without a marker, and checks a replacement live leader and another boot. The earlier surviving-child fixture now supplies the inherited marker. These checks address the destructive inference I reported. I did not execute process tests or signal any process.

## Original finding 3. Amended canary sensitivity proof

**Open; not a finding against this candidate, as instructed.** The amended control-versus-isolated canary has not run and awaits the owner's dedicated E3 Codex login. N4's static fix does not substitute for that observation. The coordinator must still record the actual pinned-CLI control coverage and isolated results. The separate adjudication controls remain the stated pre-session obligation of `fx-1vwf.3`.

## New findings introduced by these fixes

### N6. [P1] Dropping a model bypasses recovery and permanently disables its normal cleanup path

`harness/src/main.rs:1983–2015`, together with `:1867–1872,1945–1958`.

`drop_model()` classifies every missing `verdict.json` as pending and writes the exclusion without settling existing attempts. Suppose the deadline has passed, an earlier recorded attempt supplies its timestamp, and the dropped model's latest attempt was interrupted after launching or refreshing its copied login. It has `launch.json` and a scratch root but no `attempt.json`. Dropping succeeds without reclaiming that process, handing back the refreshed login, or auditing its log. `unsettled()` then permits retained-session adjudication. Subsequent `review` refuses the dropped model before reaching `settle()`, so the ordinary recovery path is no longer available. A still-running reviewer or the only refreshed credential can remain stranded; a breach in that interrupted log is never discovered. Even an already recorded unacknowledged breach can be bypassed this way.

The same missing-final-file shortcut can exclude a model whose last durable attempt already contains its completing verdict, rather than reconstructing that outcome first. This reintroduces the attempt/final-record boundary problem through the new terminal transition.

Minimum correction: settle existing attempts and reconstruct already durable terminal outcomes before deciding retention, without starting another reviewer. Require process reclaim, credential hand-back, and log audit to succeed, and honor any unacknowledged breach before publishing the drop. Preserve failures as recoverable errors. Add focused dummy-credential/interrupted-log checks showing that drop cannot strand a root or bypass a breach, and that a durable completing verdict prevents exclusion.

### N7. [P2] A partial or failed drop write is already an irreversible accepted drop

`harness/src/main.rs:1940–1942,2007–2015`.

The final `dropped-MODEL.json` is created before its contents are written, while every consumer tests only its existence. Interruption immediately after `open()`, or a write failure such as a full filesystem, leaves an empty or partial file. Although `drop` did not finish successfully, `review` now refuses the model, `unsettled()` treats its sessions as settled, and adjudication may start. Retrying `drop` cannot repair the record because `create_new(true)` refuses the existing file. The exclusion has no complete durable note, timestamp, or pending-session record, contrary to the new transition's contract.

Minimum correction: publish a complete drop record atomically while preserving refusal to replace an existing committed drop, and reject incomplete/invalid records as recovery errors rather than accepting them as terminal state. Add an interrupted-publication check. The existing atomic `write_json()` pattern illustrates the intended record boundary, although its replacing rename alone does not enforce the new write-once requirement.

### N8. [P2] The review deadline loses its origin when the first attempt is recovered

`harness/src/main.rs:1963–1970,2000–2005`, interacting with `:1756–1780,1797–1816` and `protocol.md:630–633`.

The new deadline claims to start when the run's first attempt starts, but `first_start()` reads only `attempt.json.started`. That field is written after an ordinary attempt has finished cleanup; recovery writes an attempt record with `recovered` and no `started`. If the first attempt is interrupted and recovered, a later ordinary attempt silently moves the deadline to its own start. If all existing attempts were recovered, `drop` reports “no attempt has started” and neither unavailable model can be settled through this transition, even after fourteen days. An interruption followed by recovery is supported behavior, not evidence that no review attempt ran.

Minimum correction: persist the deadline origin before launching the first reviewer and preserve it through recovery. Compute later deadline checks from that durable origin. Add a cheap state check where the first attempt is interrupted/recovered and verify that later attempts do not move the deadline and that drop becomes available at the original deadline.

## Evidence and limits

`git diff --check f3b7706 1a2a1ab` passes. The saved `harness-test-r3.log` reports three successful runs of 15 tests, with the private-mode child separately passing and its parent-suite entry ignored. I inspected the relevant test definitions and that log; I did not rerun or independently establish its binary identity. Those tests substantiate the reported cases, not N6–N8's missing interruption transitions.

Only this requested report was written. No repository or Beads edits, Cargo, harness execution, reviewer/adjudicator sessions, live canary, real credential reads, or process signalling. Confidence is high in the static traces above.

Not checked: runtime behavior, real login refresh, and the pending live canary.
Risk: the new drop path can strand processes/credentials, bypass breach recovery, accept an incomplete exclusion, or delay/block the promised fallback after interruption.

DONE
