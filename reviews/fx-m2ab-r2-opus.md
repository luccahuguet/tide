Model: claude-opus-5-5 · Date: 2026-11-13

## Verdict

request_changes. One blocking finding (a remaining present-tense Bend tandem claim outside the edited files); two non-blocking wording findings. Every dated fact in the diff checks out against upstream, and every research correction is a dated note that leaves the original findings intact.

Reviewed `git diff HEAD^..HEAD` at `edef613` in `/home/lucca/pjs/lab/tide/.worktrees/fx-m2ab` (clean tree). Read the bead (`br --no-db show fx-m2ab`), the author's report.md, the Codex r1 file and re-check, the eon-desk upstream report, design/01 D16 and the tier table. Spot-checked with public read-only `gh api` on 2026-11-13:

- `repos/HigherOrderCO/Bend` → `bendlang/bend`; `repos/HigherOrderCO/HVM` → `HigherOrderCO/HVM1`; `repos/HigherOrderCO/HVM2` still exists, so REFERENCES' "HVM2's paper remains at its version-specific URL" holds.
- `repos/bendlang/bend/releases`: v2.0.37 published 2026-10-13, v2.0.38 2026-10-20, v2.0.39 2026-10-23, v2.0.40 2026-11-06 (02:06Z); `releases/latest` is v2.0.40. v2.0.35 and v2.0.36 (10-03, 10-07) confirm the slowdown after the launch burst. Intervals since .37 are 7, 3 and 14 days, so "roughly one every one to two weeks" is a fair summary.
- `repos/bendlang/bend/issues/813`: `closed`, `not_planned`, closed 2026-09-19.
- `contents/WONTFIX.txt?ref=f311527…`: lines 152–154 read "A native library target for pure defs (#813) … is planned, not scheduled." The cited `#L152-L154` anchor is correct.
- The `11-bend-interop.md#correction--2026-11-12` anchor matches GitHub's slug for "Correction — 2026-11-12".
- research/10's note scopes its correction to "F7 and Sources", and its old HVM links are at :127 (inside F7, which starts at :111) and :406 (Sources). That is correct.
- `rg 'HigherOrderCO/Bend\b'`, excluding `archive/` and `reviews/`, with the `trials/e1-checked/**` exclusion last, finds only the three correction notes and research/09:343, which its correction covers. The Acceptance grep passes.

Rules kept: read-only, no builds, `trials/e1-checked/` never opened or searched.

## Findings

1. **Blocking (low): H6 still says Bend 2 "works in tandem with Tide" in the present tense, without the Later tier.** `design/00-first-principles.md:246-247`.
   The H6 verdict reads "…and Bend 2, the closest peer, works in tandem with Tide as the owner of proved, parallel kernels (D16)." This is the same present-tense claim this bead removes from README ("Works with Bend") and from D16 ("Stage 0, available now"). Nearby verdicts in the same section give tiers (H3 says "`prove` mode is v1 … interactive-proof export a later tier"); H6 gives none. A reader of the verdicts would take the tandem as part of Tide today. The author's report lists this line as "governed by D16's Later tier", but the sentence never says so. The fix touches Later-tier text, and this change already edits Later-tier text in D16, so the finding belongs in this round rather than on `fx-djp`.
   **Fix wanted:** keep the verdict and the source list. Change only the tense and add the tier, e.g. "…and Bend 2, the closest peer, is planned to work in tandem with Tide as the owner of proved, parallel kernels (Later tier, D16)."

2. **Non-blocking: "Stage 0 remains supported upstream" overstates what Bend provides and cites no source.** `research/11-bend-interop.md:23-24`.
   Bend has no Stage 0 feature to support. Stage 0 is Tide's own design: a generated shim and wire format around an ordinary Bend binary. A reader could take "supported upstream" to mean Bend endorses or ships the tandem boundary. This sentence also carries the correction's only 2026-11-12 claim without a citation; its evidence is the eon-desk Orbit example, which is internal. D16 now says "feasible with current Bend", which is the accurate wording.
   **Fix wanted:** "Stage 0 remains feasible with current Bend through a process boundary (standalone binaries with file and stdio effects, F5)". Alternatively, cite what the claim rests on.

3. **Non-blocking: research/01's note gives #813 only as "planned, not scheduled", without the issue's closed `not_planned` state.** `research/01-bend-hvm.md:10-11`.
   REFERENCES.md and research/11 both give both states. research/01 cites only WONTFIX, so a reader of research/01 alone may assume the issue is open. The note does link to report 11's correction, so this is minor.
   **Fix wanted:** add "(the issue itself is closed as `not_planned`)" after "(#813)", or leave it to the link.

Checked and not raised:

- design/01:51 and :67-68 ("Bend's territory"; "Bend 2 owns proved, massively parallel kernels") are statements about scope and division of labour, not availability. Adding "(Later tier)" to :67 would match the "(v1)" tag on the Rust-coexistence bullet beside it, but that is optional.
- design/03 §10 is headed "(later tier, D16)". The reference-companion, README:571 and the e1-origins references all label the tandem Later. design/04:1125 lists Bend claims as trust holes, as design. The `=2.0.37` pins are examples. None of these presents Bend support as current.
- research/11's note says it supersedes "repository-home" claims, but that report's own text already used `bendlang/bend` in its Sources; "Bend 2 (HigherOrderCO)" at :29 names the company, which is still accurate. This overstates slightly and misleads no one.
- D16's new label "planned for Tide (Later tier); feasible with current Bend" is clear. The Stage 0 mechanism, Stage 1's condition and the tier table are unchanged. No decision changed.

DONE

## Re-check

Verdict: approve. Finding 1 is resolved, and no blocking findings remain from this Opus round.

On 2026-11-13 I re-checked finding 1 only, with `git diff edef613..HEAD` at clean HEAD `c898ec3`. `design/00-first-principles.md:246-248` now reads "…and Bend 2, the closest peer, is planned to work in tandem with Tide as the owner of proved, parallel kernels (Later tier, D16)." That is the requested tense and tier label. The rest of the H6 verdict and its Sources line are unchanged; the diff for that file changes only this sentence and its line wrapping. Findings 2 and 3 were non-blocking, so I did not re-check them. Their diffs are in the same range. No builds, and `trials/e1-checked/` was not opened.

DONE
