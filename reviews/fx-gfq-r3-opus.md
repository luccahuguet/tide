Model/Effort/Date: claude-opus-5-5, low, 2026-10-31

# fx-gfq r3 Opus confirmation review: frozen checked-toolchain E1 protocol

## Verdict

approve, confidence 0.85. No blocking findings. Every blocking finding from
rounds 1 and 2 is resolved. The edits since round 2 introduce no defect or
contradiction that would make the gate run misleading.

## Scope

- Candidate: the working tree at `/home/lucca/.cache/eon-agents/tide-gfq`
  against HEAD 1e0cbac. `git diff HEAD | sha256sum` gives
  8e17847645ef97763dc381d653828de4e82d6dd8bfb6cbaadd7bffb5a870910a, which
  matches `r3.diff`.
- I diffed `r2.diff` against `r3.diff`. The only changes since round 2 are:
  - the added `reviews/fx-gfq-r2-opus.md`;
  - protocol §6 "Concurrency" and §7.3;
  - the tasks.md preface (lines 40-41), B6 (lines 522-525), and B7
    (line 560).

  The bead lines are byte-identical to round 2's.
- I read `trials/e1-checked/protocol.md` and `tasks.md` in full, along with
  the three earlier reviews, fx-gfq, fx-ic2, the Scopes of fx-u65, fx-zc1,
  fx-o8j, and fx-o4h, D0 in `design/01-design.md`, and GOVERNANCE.md's Merge
  Rule.
- Checks I ran:
  - `br dep cycles`: none.
  - `br list --label e1-prereq`: 14 beads, all of which are fx-ic2
    dependencies.
  - Parsing the JSONL: all 128 lines are valid.
- Nothing in the worktree was edited.

## Earlier findings

Round 1 (Opus 1-7, Codex 1-6) was confirmed resolved in round 2. I
rechecked them against the current text, and none regressed:
- the passing-trial iteration mean (§7.3);
- the freeze-start held-out rule (§2);
- "passed or failed" (§10.2);
- the `cancel` and `#[pure]` owners;
- the E1-P row IDs;
- fx-yg5's Acceptance;
- B7's preflight and content replay;
- the FX-rule owner fx-u65 with readiness item 3;
- A9's inventory;
- the read allowlist in §10.3.

Round 2:

1. **§6 claimed C3 and C5 were serial-proof and left C6's stop check
   unreviewed.** Resolved.
   - `protocol.md:143-150` now says only C10 has a serial-failing case.
   - The per-trial review covers C1 to C6 and C9.
   - The review explicitly fails three cases: a C3 trial without
     concurrent producers, a C5 trial whose jobs do not all start at once,
     and a C6 trial without a stop check every 4096 bytes.
   - §7.2 already gates on the review, and §6's "Grading is mechanical"
     bullet names it as the one exception.
   - The claim about C10 holds. In case 2, a serial server waits on client
     1 while client 1 waits on client 2's `exchange`, so it deadlocks.
2. **B6's payload type was stated inside B7.** Resolved.
   - `tasks.md:522-523` puts `String` in B6 itself.
   - `tasks.md:560` now covers only `PlanError`.

Round-2 non-blocking items that were taken:
- The preface now tells writers about the concurrency review
  (`tasks.md:40-41`).
- §7.3 now says post-green runs count (`protocol.md:189-190`).

## Blocking

None.

## Non-blocking

- **C3 never says "concurrently".**
  - The preface (`tasks.md:40-41`) and §6's general rule tie the review
    to work "the task says runs concurrently or in parallel".
  - C3 (`tasks.md:723-726`) says only "starts one producer per input" and
    "merges ... in the calling task". It is the one reviewed task whose
    text uses neither word.
  - §6's "in particular" clause still binds the grader, and a serial
    feed-from-the-caller design does not "start a producer". So this is
    not a gate defect.
  - Since task text freezes at the first writer, consider "starts one
    producer per input, running concurrently with the merge".
- **Reference solutions are not required to pass the concurrency review.**
  §10.2 item 2 lists `./check`, the structure check, and the acceptance
  suite. Adding "and the §6 concurrency review" would make the review's
  criteria face one dry run before the gate.
- **The review is not blind to the arm.** The grader necessarily sees the
  language. §6's fixed per-task failure list limits the room for
  judgement. The report could also say who reviewed each trial and give
  each arm's review-fail count.

## Reasoning

**D0.** D0 says: "first-check pass rate at least 0.8x the Rust rate and
final pass rate at least the Rust rate within 1.5x the iterations" on a
30-task suite. §8 restates this exactly. §7.3's passing-trial mean, which
now counts every run, is a faithful operational reading of "iterations to
green":
- a trial's runs all lead to its final green snapshot;
- failed trials are penalized under threshold 2;
- both arms are counted the same way.

**Round-2 edits.** I found no defect or contradiction.
- The concurrency review is symmetric across arms and recorded per trial.
  It is named in the preface, §6, and §7.2.
- The B6 and B7 payload sentences now sit in the tasks whose writers read
  them.
- The B6 reflow (`tasks.md:524-525`) leaves a short line but changes
  nothing.

**Cross-file consistency.**
- Seeds: 20 in the preface, §6, and C4/C10.
- Budget: 20 runs and 40 minutes in the preface and §4.2.
- Per-test limit: 10 seconds in §4.1, §6, and A6.
- Structure checks: each Tide and Rust structure requirement is checkable
  as a text search (§6).
- §10.1 rows: they cover every Tide surface the tasks name:
  - C6's `checkpoint`/`cancel` (fx-zc1, fx-nkv.3);
  - C5's `Clock` (fx-o8j);
  - C3's `sync_channel` (fx-o4h);
  - C10's `Net` (fx-hxw);
  - B7's `#[pure]` (fx-yg5).

**Beads.** The bead text is unchanged since round 2, which verified each
Scope against its row. The fx-u65 Acceptance I reread matches §10.2 item 3.
fx-ic2 has an edge to all 21 §10.1 owners and to fx-gfq, with no cycles.

**Checked cases.** Values are unchanged since round 2's full recomputation.
The B6 and B7 edits touch only payload-type wording, not checked values.
