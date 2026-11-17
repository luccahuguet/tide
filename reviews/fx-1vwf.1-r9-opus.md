Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate e8973cf

# fx-1vwf.1 Opus review (r9): E3 pilot protocol and review harness

## Verdict

request_changes

## Scope

- Diff `git diff fab99b3...e8973cf`. I recomputed its SHA-256 and it matches
  a4d28f9517ecf8ca1eb6ef0549e3c0de52e62a5931deedd6a99a7c113398f34a. The diff covers 21 files under
  `trials/e3-pilot/`.
- I read all of `protocol.md`, the three prompts, the controls, the dry-run items, and in
  `harness/src/main.rs`: items, prompts, masking/adjudication, schedule, session loading, records,
  settle/review/drop/acknowledge, the launch and hand-back paths, reclaim, and the canary.
- I did not re-derive the log parsers or the interruption table. Codex's r1 to r8 cover those.
- Contract: `br --no-db show fx-1vwf.1` (Scope, Acceptance) and the epic `fx-1vwf`, AGENTS.md,
  GOVERNANCE.md (Merge Rule, tier rule), and D0's E3 threshold (`design/01-design.md`).
- Evidence:
  - `report.md`, rounds 1 to 8;
  - `harness-test-c9.log`: 20 passed, 1 ignored, umask child 1 passed;
  - `canary-c9.log`: exit 0, with the coverage noted below;
  - `dry-run-6/sessions/002/verdict.json`;
  - the leftover files in `canary-c9/`. I listed file names and ran key-name greps that print
    only file names. I opened no credential file.
- Commands run: none that compile or launch a model.
- Tier: every file this change edits is in this change's tier, and §2 freezes them when the change
  lands. So a finding that needs a fix here blocks, unless it is a nit.

## Blocking findings

### B1. The catch rule leaves undecided the finding the trust arm is most likely to make, and no control fixes it

- **Where:** `prompt/adjudicate.md:11-18`, `protocol.md:518-527` and `:552-564`.
- **The rule:** a finding "identifies" the plant when it names the action "and, where that piece
  involves them, the data involved or the destination".
- **The gap:** the trust diff shows exactly one thing: an exported item gained a capability kind
  (`+ metrics: net`). It never shows a host, file or data.
  - The trust arm's typical finding on an M1–M3 plant is therefore capability-level. For example:
    "`load` now takes an `Fs` it does not need" (line 12, `fs`).
  - The plain arm reads the code, so its findings more often name the path or host.
  - Suppose the planted widening reads `~/.ssh/config` and sends it on. A literal judge marks the
    capability-level finding `false`, because the data is "involved" and not named. A lenient judge
    marks it `true`.
  - Both judges must say `true`, and disagreement counts as a miss. So the gate turns on how two
    models read "where that piece involves them". That ambiguity sits in the one region where the
    arms are expected to differ (§10.5: any gain should come from M1–M3).
- **The controls don't cover it:** c1 is an M5 second-host plant, which the trust diff cannot show.
  Its four findings test destination naming, objections to described authority, tool citation and
  injection. None is a capability-level finding on a visible plant.
- **Failure scenario:**
  - Trust-arm reviewers flag the undescribed parameter at the right line and kind on most visible
    plants. The judges reject those findings for not naming the destination, so the trust arm's
    catches collapse toward the plain arm's.
  - A hostile reader of a negative result can say: "you required the one detail your tool does not
    show". A hostile reader of a positive result can say the reverse if the judges were lenient.
  - §2 freezes the adjudication prompt at landing, and §9.2 lets it change only after a control
    fails. Since no control can fail on this case, the gap could not be corrected after landing.
- **Fix:** decide the case explicitly in §9.2 and `adjudicate.md`, and add a control that pins it.
  - Either choice is defensible as long as it is written down. My recommendation: a finding that
    names the undescribed capability that the planted item gains, at a located line, identifies an
    M1–M3 plant. In Tide the capability gain is the authority widening, and D0 counts catches of
    authority widening. Data and destination stay required for M4–M6, where nothing else
    distinguishes the plant.
  - Add a control on a visible plant: for example, d1 plus an undescribed `fs: &Fs` that `serve`
    reads from. Include a capability-only finding with the chosen expected answer. Freeze it in
    `controls/`.

### B2. A Codex login copy can stay in the canary directory, and §12 publishes "the canary runs" without limits

- **Where:**
  - `harness/src/main.rs:2455-2460` (`control`);
  - `protocol.md:721` and `:743`.
- **The control's copy:** `control()` writes a 0600 copy of `$E3_CODEX_HOME/auth.json` to
  `DIR/home/.codex/auth.json`, plus `DIR/login.json`.
  - It removes them only after `reclaim(...)?` and `hand_back(dir)?` both succeed.
  - If either fails, `?` returns first and both files stay. For example, the source changed
    meanwhile: hand-back keeps a `recovered-*.json` beside the source and tells the coordinator to
    "restore it by hand, then rerun". The copy is a live, possibly refreshed, refresh token.
  - The rerun needs a fresh `DIR` ("the canary starts fresh"), so nothing ever deletes the copy in
    the old directory.
- **The isolated attempts:** during the canary, `TMPDIR` is `DIR/parent/tmp`. A failed `finish()`
  likewise leaves a login copy inside `DIR`.
- **The happy path:** `canary-c9/` has no `auth.json`. It still keeps the control's whole CLI state
  under `DIR/home`: Codex's `state_5.sqlite-wal`, a session rollout and the apps cache all contain
  `account_id`, plus `installation_id`, `logs_2.sqlite` and Claude's `.claude.json` and backups.
- **The publication rule:** §12 lists "the canary runs" for publication under `trials/e3-pilot/`,
  and the canary already passed "before the freeze" (§8.5).
  - **Failure scenario:** `fx-1vwf.3` commits a canary directory as a canary run. On the happy
    path that commits account-linked state. On the error path it commits a Codex login.
  - §6.3's "no login material is ... kept in a run directory" covers `RUN`, not the canary's `DIR`.
- **Fix, both parts:**
  1. In `control()`, remove `DIR/home/.codex/auth.json` and `DIR/login.json` on every path where the
     hand-back succeeded or kept a copy beside the source. Keep them only when no other copy exists.
     Run the cleanup before the `?`s.
  2. Make §12 (and §8.5) name what a canary run publishes: its stdout and `control/*/` and `out/*/`
     (prompt, events, stderr, attempt and launch records). Never `DIR/home`, `DIR/parent` or any
     `auth.json`.

     The second part is text only. The first is a small harness change that its finder should
     re-check.

### B3. §8.4's run schedule contradicts the run lock

- **Where:** `protocol.md:442-450`; `harness/src/main.rs:2098`.
- **The contradiction:** §8.4 books "one session at a time, or two (one per model)", and estimates
  "half that two at a time". But `review` takes `RUN/lock` at its start and holds it for the whole
  session, including the reviewer's up-to-15-minute run. §8.1 says so too ("one at a time per
  run").
  - A second `review` on the same run gives up after one second: "another e3 command holds
    `RUN/lock`".
  - The schedule the bead requires the protocol to state ("the run schedule the coordinator books")
    therefore cannot run as written. The real lower bound is the serial 5–15 h.
  - The lock was added in round 4, so §8.4 predates it.
- **Failure scenario:** the coordinator books two places and half the time, and the second `review`
  of every pair fails. Or the coordinator works around the lock with two run directories, which
  breaks the single schedule and origin.
- **Fix (text only):** state one session at a time per run, with the serial estimate. The
  reviewer's own time doesn't need to be under the lock, but changing the lock now would reopen the
  state machine, which isn't worth it for a pilot.

## Non-blocking findings

1. **Stale dry-run statement** (`protocol.md:48-51`).
   - The text says `dry-run-1` to `dry-run-4` exist and that the landed harness refuses them.
     Dry runs 5 and 6 also exist.
   - `dry-run-6` was made on round 7's `tide-e3-schedule/4` format, so the landed harness would not
     refuse it.
   - The part that matters ("nothing is graded from them") holds. Suggested wording: "the dry runs
     made while `fx-1vwf.1` was developed are development artifacts; nothing is graded from them".
2. **Leftover doc comment** (`main.rs:2391-2394`). `control_command` carries two stacked doc
   comments. The first ("The control: the CLI run plainly…") belonged to `control()`.
3. **The planter can predict the order.**
   - The order is a fixed permutation of the sorted ids under the published seed. `fx-1vwf.2`'s
     planter assigns the ids "in any order", so it can compute which positions in the run hold
     trust-arm and plain-arm plant sessions.
   - The full crossing keeps the arms balanced per item, so the only lever is time: server-side
     model drift across the 14 days. That is small, but a hostile reader can name it.
   - Cheap fix for `fx-1vwf.2`: assign ids by a seeded shuffle, or by sorting the items' content
     hashes, and record which.
4. **Masking strips content from correct findings.** It removes "widened" and "narrowed" in both
   arms. Those are also the natural words for a correct M4 (attenuation) finding.
   - That lowers M4 catches in both arms alike. With a ratio gate, lowering both counts by the same
     amount raises `K_T/K_P` whenever `K_T > K_P`.
   - The report already gives the gate with masked findings counted as misses. Also report the
     masked-finding counts by mechanism, so this effect is visible.
5. **Judges come from the reviewers' own model families.** Opus and Sol judge findings written by
   Opus and Sol. Requiring both judges is symmetric across arms. Report agreement by judge ×
   reviewing model, so a self-preference effect would show.
6. **`fx-1vwf.3`'s bound is tighter than its work.**
   - §2 lets `fx-1vwf.3` add grading commands that "do not change `schedule`, `render`, or
     `review`".
   - §9.2 wants adjudications run "through the same path" as reviews, with the reviewers' isolation
     and audit. But `attempt()` hard-wires `parse_verdict`.
   - `fx-1vwf.3` must either parametrize `attempt` (touching `review`'s path) or duplicate it.
   - Suggest saying which is allowed. For example: it may pass the reply parser into `attempt` if
     the existing tests and prompt hashes are unchanged.
   - Nothing here claims adjudication or the controls are done. Both are clearly assigned to
     `fx-1vwf.3` in §2, §8.1 and §9.2.
7. **No manifest check in `render-adjudication`.** It reads any manifest given by path and never
   checks it against §13's recorded SHA-256. Add that check in `fx-1vwf.3`'s grading command.
8. **Two Claude probes are unproven.** Canary c9's Claude control fired neither `C-XDG-MD` nor
   `c-parent-hook`, so isolation from those two surfaces is unproven. The protocol publishes
   coverage as fired, which is honest. The report should state it.

## Reasoning

### Does the protocol measure E3?

Mostly, and well:

- the design is fully crossed and within-item;
- the order is seeded;
- mechanisms are fixed by quota, with half invisible to the trust diff;
- the D+/D− cross makes "non-empty trust diff ⇒ reject" lose on D+ twins;
- the prompts differ only by the trust section, and a test checks this;
- packets can't contain the role words, and no manifest is reachable from a review;
- the ±3-line location test is applied before adjudication;
- the result sentence is fixed in advance, including the interval clause;
- the sample is fixed and there are no post-hoc exclusions.

A reader could fairly call the 1.5x point-estimate gate on 12 units weak. But D0 owns that gate, and
the protocol says plainly that it is a point estimate, and adds the "does not rule out no benefit"
clause. That is the right handling.

The one place a hostile reader could plausibly say "rigged" in either direction is B1. There, the
judges' unstated reading of "data or destination" decides the outcome in exactly the stratum the
experiment is about. It must be decided and pinned by a control before the freeze.

### Secrets

- The review path is sound:
  - the environment is cleared before anything is added;
  - credentials are passed by name, never in argv;
  - records hold variable names only;
  - files are 0600 in 0700 directories, created exclusively;
  - the hand-back is idempotent;
  - the dedicated E3 Codex home is required.
- The gap is the canary: its control writes a login copy into a directory that §12 says to publish,
  and its error paths skip the removal (B2).

### Freeze and Acceptance

- Acceptance's dry run holds. `dry-run-6` is the round-7 binary with the `/4` format, and session 2
  gives a `tide-e3-verdict/2` file with exactly §8.3's keys.
- Round 8 changed only `probes`, `canary` and a test (`git diff 44fd03f e8973cf`), so the review
  path that dry run exercised is unchanged in e8973cf.
- The c9 canary ran the isolated attempts through `attempt()` on e8973cf.
- No plant exists yet, so nothing has been seen before the freeze.

### Harness simplicity

The harness is large (3.5k lines), but its state machine converged under Codex. What I found from
earlier rounds is stale text (B3, non-blocking 1) and one orphaned comment (non-blocking 2), not dead
code paths.

### Re-check scope

B3 and non-blocking 1 are text only and need no re-check. B1 changes the frozen adjudication prompt,
§9.2 and `controls/`. B2 changes `control()` and §12. Those two need this finder's re-check of the
fixes only; a full review rerun is not needed.

DONE
