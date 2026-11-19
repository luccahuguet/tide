Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 693a833 (re-check)

## Scope

This is the finder's re-check of the fixes to my S2a Opus review (target/work/s2a-opus-review.md, on 2745bc2). Per the brief, it covers only:
- the six deviations in report.md, "S2a Opus round fixes", "Deviations";
- the "Not changed (noted)" heading;
- any new problem a fix introduced.

What I ran:
- I read `git diff 2745bc2 693a833`; it touches only paper/tide-v0.typ and reviews/fx-qlek.7-paper-terms.md.
- `typst compile --root . paper/tide-v0.typ /mnt/build/tide-agents/fx-qlek.7-target/tide-v0-s2a-r2-opus.pdf` exited 0.
- pdftotext shows the semicolons restored (for example "par.rs; the core reference §6.8" and "design/01-design.md; first principles:") and no stray backslash.
- `git diff 2745bc2 693a833 --check` is clean.

## Deviations

1. **tokio defined in the abstract** (:141, "fails inside tokio, Rust's most used runtime for asynchronous code, which Tide does not admit"). **Resolved.** The terms rule requires a definition at first use, and tokio now first appears here. The definition matches the paper's existing §4.4.2 wording ("the most used async runtime for Rust"). The terms row was added.
2. **Eon sentence at :322** ("Eon [17], the author's own terminal workspace, has a supervisor that finds its terminal hosts through files."). **Resolved.** It is grammatical, keeps the original meaning, and matches the case-study READMEs ("Eon is the Tide owner's project").
3. **"a pinned, unmodified rustc nightly (rustc's unstable daily build)"** (:193). **Resolved.** The definition is accurate in plain words, and the reordering changes no meaning. The Dialect and Nightly rows quote it exactly.
4. **"Libraries and lints (extra compiler checks) that keep Rust's std"** (:155). **Resolved.** "Lints" is defined at first use, and the sentence no longer contradicts §2.5.
5. **"on the point estimates (the measured values, not their intervals), near ceiling (both arms passed almost every task)"** (:141). **Resolved.**
   - Both glosses are correct against E1: 58 of 60 final passes in each arm.
   - "Not their intervals" is a fair compression of §4.6's statement that the iteration ratio's interval crosses 1.5.
   - Both terms have rows in the terms artifact.
6. **":167 'no `async`/`await` (Rust's syntax for asynchronous functions, D8)'"**. **Resolved.** The definition is accurate, and the terms row quotes the paper.

## The §4.4.2 heading (:318), "Case studies: bugs outside admission's class"

**Not resolved: it still contradicts the hyper record under B2.**
- The hyper bug is titled "hyper's pool reaches for an ambient runtime". Its record says Tide's rules do address that reach: "Reject the ambient reach at admission, in its std spellings" (examples/case-studies/hyper-ambient-runtime/README.md:146–153).
- What sits outside admission is narrower: the three *reported failures*.
- The section's corrected first sentence ("Admission rejects none of the three reported failures") does not repair a heading that classifies the *bugs* themselves. Readers who skim headings see the heading.

**Exact replacement for :318:**

```
=== Case studies: failures admission does not reject <sec-case-studies>
```

After this change, "admission's class" no longer appears in the paper, which was its only remaining use. Delete the "Admission's class" row from reviews/fx-qlek.7-paper-terms.md. The abstract and :320 define nothing new.

## New problems introduced by the fixes

None blocking.

**Optional style item.** In the abstract, "There Tide's simulator does the work" now follows a sentence that includes hyper, whose record credits the fix's shape rather than the simulator. The following "for the other two, a simulated test checks the fix deterministically" still holds, because hyper's `fixed.rs` has a 20-seed `#[sim_test]`. So the sentence is accurate enough, and this item is not a condition for approval.

## Verdict

**Changes required.** One text-only change remains:
- replace the :318 heading with the exact line above;
- delete the "Admission's class" row from the terms artifact.

It is a verbatim, text-only fix, so under AP-PROOF-001 it needs no further re-check. With it applied, I approve S2a. Every other fix from my review, B1–B6 and non-blocking items 1–5 and 7, is resolved.

DONE
