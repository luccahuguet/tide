Model: claude-opus-5-5 · Date: 2026-11-18 · Re-check 38dc27e

# fx-qlek.4 r4 re-check: Opus (finder re-checks its own findings)

## Verdict

**approve**, with high confidence. B1 and B2 are fixed, and the edits add no new overclaim or error. The nits I raised that the builder took up (N1, N2, N3, N4, N6) are applied, and the figures render cleanly.

## Scope

- Round diff: `git diff ba40864 38dc27e`, `paper/tide-v0.typ` only (+14/-14). SHA-256 of the diff: `045015da85f788a235cac2bf7afe30d2db44ccb37e366712522087c63b9353cb`. `git diff --check` is clean, and the worktree is clean. 38dc27e is the amended task commit on main `27b394e`.
- Build: pinned typst (nixpkgs 567a49d…, typst 0.15.0) through heavy.sh, with the load at 2.4, to `/mnt/build/tide-agents/fx-qlek.4-target/r4-recheck.pdf`. It has 11 pages, and its SHA-256 is `bc603edb4c1d45b6ab29c2d2d9e15d8f600f26f92d5595072c3e9389fd8f5461`, the same as the builder's two builds. I looked at pages 4 to 8, with zoomed crops of Figures 1 and 2.
- I checked the builder's "Round r4 fixes" account against the diff, treating it as claims. It matches the diff.

## Findings

- **B1 (unforgeable-condition paragraph): fixed.** See `paper/tide-v0.typ:204`.
  - Route 1 now says that fixing known issues "cannot show that no unknown one remains: this route only shrinks the known holes". Route 2 is the only one that "could support the word unforgeable".
  - The trusted code is now named: "`tide-std`'s `unsafe` runtime, and the completeness of admission's rules, whose gaps over rustc's holes are what the red team found". The model-fidelity hedge covers "rustc and that code".
  - The exit criterion now reads "Until it exists", meaning the proof. That is consistent with route 1 never verifiably holding.
  - The factual claims hold:
    - The #57893 routes were admission gaps that the red team found (§4.1, `reviews/2026-11-17-red-team-opus.md`).
    - `tide-std` has `unsafe` sites, and fx-ip8d lists them as trusted code.
    - fx-ip8d is still deferred and not started.
  - v0 is still never called unforgeable.
- **B2 (third case study's Rust alternative): fixed.** See `:336`.
  - The added sentence matches the README's "What the Rust version would need": loom or Shuttle, plus a simulated file system the project writes and maintains or adopts from a framework, and `Fs` as a program's only file access. It cites that README section.
  - The edits at `:338` and `:124` change what the gain is pinned on. It is now "over the simulated file system that Tide's std already provides" and "a simulated file system the project did not have to write". Both are accurate, and neither implies Rust cannot do it.
  - The abstract's new clause "once someone states the property" also restores the "unaided" qualifier (N4).
- **N1 (Figure 2): fixed.** The 2S/3 line is now muted and dotted, labelled "the ceiling 2S/3 = 48, above which no 1.5× gain fits". It no longer looks like a bar both arms cleared. The R label sits right of the 1× line on two lines, clear of both the 1× line and the threshold line. The plotted values are unchanged (66 and 67 of 72, 0.99, [0.95, 1.00], threshold 1.5×, "Gate: not met").
- **N2 (figure references): fixed.** Figures 1, 2 and 3 are now referenced at `:164`, `:254` and `:281`, and they resolve in the PDF.
- **N3 (sandbox box): fixed.** The box now reads "prints the level first: `full`, or `partial, not enforced`", and the text fits the box.
- **N4: fixed**, as described under B2.
- **N6 (Figure 3 caption): fixed.** The caption now reads "the counts are made from the records with the note's rule (§1)".
- **N5 and N7:** left for fx-qlek.7 as I recommended. They are not part of this re-check.

## New defects

None. The edits change no data, helper code or plotted numbers. Figure 3 is visually unchanged, with the instability hedge still above both panels and no row reading as a win.

DONE
