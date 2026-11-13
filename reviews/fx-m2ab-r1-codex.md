Model/Effort/Date: gpt-6.1-sol / high / 2026-11-13

Verdict: request_changes. Confidence: high. One blocking wording finding; the dated upstream corrections are supported.

1. **P2 — Blocking: distinguish upstream feasibility from delivered Tide support** — `design/01-design.md:951`.
   D16 still says **“Stage 0, available now”** immediately before describing a Tide `bend` dependency kind, cached builds, a generated signature-checking shim, supervised execution and `Err` mapping. A reader following the D16 link can reasonably read this as available Tide integration. The tier table at line 85 places the tandem in Later, and the new correction at `research/11-bend-interop.md:24` explicitly says neither stage is implemented in Tide. The author's report interprets “available now” as upstream feasibility, but that qualification is absent from D16 itself. This is an unchanged line, but it falls directly under this review's explicit check for remaining current-support claims.
   **Fix wanted:** change the label to **“Stage 0, planned for Tide; feasible with current Bend”**, or make the same distinction explicitly in the adjacent text. Keep the process mechanism, conditional Stage 1 and Later tier unchanged. This is an availability clarification, not a language or interop decision change.

Verification on the frozen candidate:

- Reviewed `git diff HEAD^..HEAD` at `d60b12031d54d17156817301029261fe788029e2`, clean working tree, in the Linux worktree `/home/lucca/pjs/lab/tide/.worktrees/fx-m2ab`. Diff SHA-256: `0bc2ae0091802bd69abf2ac885e2f5f531d293f98c7f9e943072000466a9bd79`. `git diff --check HEAD^ HEAD` passed. Read the bead with `br --no-db show fx-m2ab`, the author report, supplied upstream evidence, GOVERNANCE.md, D16 and the tier table. README fact 4 was treated as a coordinator decision, not reconsidered.
- Public read-only `gh api repos/HigherOrderCO/Bend` and `repos/HigherOrderCO/HVM` returned canonical homes [bendlang/bend](https://github.com/bendlang/bend) and [HigherOrderCO/HVM1](https://github.com/HigherOrderCO/HVM1).
- `gh api 'repos/bendlang/bend/releases?per_page=6'` confirmed GitHub publication dates: [v2.0.37](https://github.com/bendlang/bend/releases/tag/v2.0.37), October 13; [v2.0.38](https://github.com/bendlang/bend/releases/tag/v2.0.38), October 20; [v2.0.39](https://github.com/bendlang/bend/releases/tag/v2.0.39), October 23; [v2.0.40](https://github.com/bendlang/bend/releases/tag/v2.0.40), November 6, still latest. The pinned changelog separately dates .40 November 5, as the correction explains.
- `gh api repos/bendlang/bend/issues/813` confirmed [#813](https://github.com/bendlang/bend/issues/813) is `closed`, with `state_reason: not_planned`, closed September 19. The contents API at `f311527776559ce40095108ad6a7514a6c34aaa1` confirmed [WONTFIX.txt:152–154](https://github.com/bendlang/bend/blob/f311527776559ce40095108ad6a7514a6c34aaa1/WONTFIX.txt#L152-L154) still calls the target “planned, not scheduled”. The candidate correctly distinguishes the issue state from the feature's documented plan.
- Compared each research file with its parent revision: all four have one insertion only, preserving every original finding. Dated corrections cover the retained old-home links and launch-cadence statements. Repository-wide Bend/HVM searches found no other blocking current-support or uncorrected canonical-home claim outside the excluded directories.
- Verified the author's stale-reference claim: `design/01-design.md` has no F7 section or HigherOrderCO home claim and is unchanged by the candidate. The relevant cadence F7 is `research/11-bend-interop.md:98`, covered by its dated correction.

Not checked: no builds or runtime interop tests; Orbit feasibility relies on the supplied upstream report. `archive/` and `reviews/` were excluded from the leftovers search. `trials/e1-checked/` was never opened or searched; its exclusion was last in search globs. No repository edits, commits or br writes were made; only this requested report was written. Opus review remains pending after the Codex finding is resolved.

DONE

## Re-check

Verdict: approve. Finding 1 is resolved; no blocking findings remain from this Codex pass.

Re-checked only finding 1 on 2026-11-13 using `git diff d60b120..edef613`, at clean HEAD `edef613b839af50cf3829b5b55c5261fcb27ed6a`. `design/01-design.md:951` now explicitly labels Stage 0 “planned for Tide (Later tier); feasible with current Bend”, distinguishing Tide delivery from upstream feasibility. The diff changes only that label and paragraph wrapping; a whitespace-normalized comparison confirms the mechanism text is unchanged. Stage 1 and the tier assignment are unchanged. No broader review or builds were run; held-out files were not opened or searched. Opus review remains required.

DONE
