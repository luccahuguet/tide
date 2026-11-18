Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate 45c167b (docs re-check)

## Verdict

**approve.** All three blocking findings (B1–B3) are fixed, and N1–N7 are fixed. Under N2 I accept the author's choice to leave out one of the gaps I listed. One changed line has a small new issue that does not block. It is listed below and needs a wording change only.

- Full HEAD: `45c167b297ba9c45601af5ca331913382d44033e`, branch `fx-qlek.4`. The worktree is clean.
- SHA-256 of `git diff d4a91a2..45c167b`: `9715a130e6d61664db9731b3aa03933598e60d691f25e7034baec7e6b3a7db6d`.
- The fix diff `16fb3fb..45c167b` changes only `paper/tide-v0.typ`, which is still 213 lines.

## Findings

- **B1 (disclosure lists only some models): fixed.** `paper/tide-v0.typ:22` now reads "(Claude and GPT models: mainly Claude Opus and GPT Sol, with some earlier production code by Claude Fable)". The list is no longer exclusive, and it matches the five commits co-authored by Claude Fable 5.2.
- **B2 (rr): fixed.**
  - `:40` now says "cannot in general be rerun deterministically from a seed", and it names rr.
  - `:160` adds a sentence on rr that cites `docs/debugging.md`, "rr compared". Each part of that sentence matches `docs/debugging.md:160-170`: rr is the closest tool, works on unmodified Linux programs, replays under gdb, needs hardware performance counters, and finds a failure by recording runs, where Tide searches seeds.
  - The new sentence at `:40` has one small error; see the new issue below.
- **B3 (tokio's paused clock): fixed.** `:146` adds the README's sentence word for word. I checked it against `examples/case-studies/eon-slow-close/README.md:9-11`.
- **N1 (48 routes include controls): fixed.** `:30` and `:92` both read "48 routes, controls included", which matches `reviews/2026-11-17-red-team-opus.md:10`.
- **N2 (fx-qlek.5's remaining gaps): fixed, and the author's choice to leave one gap out is accepted.**
  - `:169` adds the `core::*` glob gap on a toolchain pin bump, and now says "what it tried and what it left open".
  - `:170` adds "RUN-3 does not confine metadata reads such as `stat` and `access`".
  - Both match the red-team record at `:146-150` and `:134-135`.
  - The author left out `TD0402` failing closed, because it rejects harmless code rather than letting anything through. That is sound. My own minimum fix named only the other two gaps.
- **N3 (dependencies stated in the present tense): fixed.** `:50` now reads "in any dependency, by design".
- **N4 (the paper calls its author "the owner"): fixed.** `:80` reads "the order chosen for the alpha", and `:121` reads "the author declined". The author is the owner, so the second wording stays accurate.
- **N5 (terms used before they are defined): fixed.**
  - `:62` defines `Pure` the way D7 does (`design/01-design.md:529-531`): no capability, synchronization handle or foreign handle is reachable from the type. It also defines TD codes as the driver's own error codes.
  - `:22` introduces Codex as "GPT Sol through OpenAI's Codex CLI".
  - `:90` says what bead IDs are.
- **N6 (references 12–18 never cited): fixed.** [12]–[18] now appear at the first mention of each project (`:146`, `:148`, `:154`, `:156`, `:158`, `:160`, `:162`), and each number matches its entry in the reference list.
- **N7 ("neither" for four bug reports): fixed.**
  - `:30` reads "none of the real bugs studied".
  - `:144` reads "Two case studies rewrite four real bug reports in Tide. Admission catches none of them."
  - That matches the case-study READMEs. The four reports are eon-8wh7h, eon-pimct, hyper#2808 and the hyper#2595 panic, and no admission code in either study catches any of them.
- **New issue (does not block): `:40`.** The new sentence says rr does "not let a test search schedules".
  - `docs/debugging.md:168` says rr's chaos mode "randomizes that scheduling to make rare failures show up". That is a search of schedules, though a random one that no seed can reproduce.
  - `:160` describes rr accurately. Only this short summary at `:40` overstates.
  - Suggested wording: "but they cannot rerun a schedule from a seed", or "their chaos mode randomizes schedules, but no seed reproduces a run".
  - I don't block on it. The fuller and accurate description at `:160` is in the same paper, and the remaining gap is a word choice, not a wrong fact about what Tide does.

## Reasoning

I followed AP-PROOF-001 and checked only my own findings. I read the author's "r2 fixes" table in `target/work/report.md` as a set of claims, then checked each fix against `git diff 16fb3fb 45c167b` and the landed source it cites:

- `docs/debugging.md:158-177`;
- the Eon README at `:8-11`;
- the red-team record at `:10`, `:127-150`;
- `design/01-design.md:529-531`;
- the reference list at `paper/tide-v0.typ:205-213`.

N8 and N9 were not in the scope of this re-check. The author handled N9 with "Every measurement" at `:24`, and N8 is still the owner's step before posting.

I did not re-review the rest of the paper.

Build check: I rebuilt the PDF with the pinned `typst 0.15.0`. It compiles with no warnings, has 9 pages, and its text is identical to `target/work/tide-v0.pdf`. `git diff --check d4a91a2..45c167b` passes.

The new issue at `:40` is a wording fix. GOVERNANCE says a text-only fix needs no re-check, so if the author makes it, it does not reopen this round.

The A1 note still stands for the owner. The paper deliberately states "no accidental ambient authority" and does not use A1's word "unforgeable".

DONE
