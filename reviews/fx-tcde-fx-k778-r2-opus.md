Model: claude-opus-5-5 · Date: 2026-11-13

Opus review round (docs-only, GOVERNANCE.md Merge Rule), read-only. Codex reported both trees clean (fx-tcde-r1-recheck-codex.md, fx-k778-r1-recheck-codex.md "Round 2 re-check"). The owner decisions are not under review: the headline "Capability-safe, replayable Rust", keeping the name Tide, and deferring disambiguation to fx-gbq4.

What I read: both diffs (`git diff HEAD^..HEAD` at fx-tcde `7f1ce80` and fx-k778 `9686e88`); `br --no-db show` for fx-tcde, fx-k778, fx-ayxr, fx-ho6o, fx-ayh2, fx-ssek, fx-dett, fx-4cdq, fx-jrvd, fx-gbq4, fx-nkv and fx-1n5n; the four Codex files; README.md; `design/01-design.md` (Thesis, Positioning, D0 E4, D12, D18); the intro of `design/reference.md`; the naming record (`tide-naming/decision.md`, `notes.md`); the Blaze reports (`tide-research-blaze/report.md`, `comparison-2026-11-13.md`, `eon-blaze-eval-work/report.md`); `eon-desk/tide-in-eon-report.md`; and, fetched today, the lust README (`next` branch) with its repository and release metadata from `gh api`, and the Rust Foundation trademark policy page. I ran no builds and made no writes except this file. I did not open or grep `trials/e1-checked/`.

## fx-tcde (README and 01 one-liner)

Verdict: approve

The diff meets the bead's Acceptance:
- README.md:3 is the short form.
- README.md:5–9 says Tide is a checked subset of valid Rust 2024 for software that agents write. This agrees with D18 and with `design/reference.md` §0 ("A Tide program is a valid Rust 2024 crate").
- design/01-design.md:53–55 matches the headline and labels the edit loop as D12's target, not a measurement. D12's figure is about 30 ms typical, under 100 ms at p95 (01-design.md:802–803).

The replay qualification at README.md:7–8 and :14–17 matches Thesis item 2 (01-design.md:36–41). The speed and target caveat at README.md:33–34 is unchanged. No other repo doc repeats the old headline (grep outside archive/ and reviews/). Nothing blocks.

1. **Nit: "wherever" can read as partial replay.** README.md:7–8 says "programs replay from a seed wherever their foreign effects have simulator models". A reader can take this to mean that the modeled parts of a program replay. The design means the whole program replays only when every reached effect has a model; otherwise the run is refused unless allowed as non-replayable (03 §§ cited by Codex). The bullet at :15–16 states this correctly. **Fix (optional):** "programs replay from a seed when every foreign effect they reach has a simulator model".

2. **Non-blocking, for the owner's awareness, not a change request: the plain form uses "Rust" as Tide's descriptor.** README.md:5 says "Tide is Rust that can't overreach…", and the headline does the same. This is owner wording and not under review. However, the trademark policy says the Rust word marks "may not be used: To refer to any other programming language; To refer to a modified version of the Rust programming language, except as permitted below for minor changes" (rustfoundation.org/policy/rust-trademark-policy, "The Basics: Referring to Rust/Cargo"). It also says no use may appear official to a casual observer. The note's assurance that the tagline is fine (fx-k778 finding 1) leaves this clause out. **Fix:** none in this bead. Make sure the owner sees fx-k778 finding 1 before any public use of the tagline, and that fx-gbq4's trademark check covers the tagline as well as the name. Not legal advice.

## fx-k778 (research/28-positioning-and-name.md)

Verdict: request_changes

What checks out against its sources:
- The lust figures match the README I fetched today: 19–25 ms body and macbody edits; 60–250x; 4.4–9.9 s at p95 for signature edits; about 2.5x cold; 3.8–4.9x slower runs; 5.7 GB peak versus 6.6 GB retained; Rust 1.99; no proc macros, async or unsafe; the four speed reasons, unranked; created 2026-10-15, pushed 2026-10-22, v0.2.0 on 2026-10-17, 108 stars.
- The Blaze cells match both Blaze reports.
- The Eon claims match `tide-in-eon-report.md` and fx-ho6o: seven blockers, the v0.2.0 cut, and the "no measurable regression" gate.
- The bead references fx-ayxr, fx-ayh2, fx-ssek, fx-dett, fx-4cdq, fx-jrvd, fx-gbq4, fx-nkv and fx-1n5n all point at the work described.

Three source mismatches remain.

1. **Blocking: the trademark conclusion in §3 leaves out the clause that governs it and presents an inference as evidence.**
   - research/28-positioning-and-name.md:66 says the policy "allows stating accurately that software is compatible with the Rust programming language [web: §4], so the tagline may say 'Rust'".
   - The tagline does not state compatibility. "Capability-safe, replayable Rust" and "Tide is Rust that…" use "Rust" to name Tide itself, a separate language with its own std that the note calls "the Tide language" (:96).
   - The cited policy says the marks "may not be used: To refer to any other programming language; To refer to a modified version of the Rust programming language, except as permitted below for minor changes". It also says that no use, including those not requiring approval, may "appear (to a casual observer) official, affiliated, or endorsed".
   - So "so the tagline may say 'Rust'" is a conclusion the source does not support. It carries a [web] label but is the author's inference.
   - **Fix:**
     - Quote that clause next to the compatibility allowance.
     - Label the conclusion [inference].
     - State the distinction: compatibility statements such as "a checked subset of valid Rust 2024" or "every Tide program is a Rust crate built by rustc" are allowed uses; using "Rust" as Tide's own descriptor is the case the restriction addresses.
     - Record that the owner's headline carries this risk, and that the trademark check (fx-gbq4, or before the §5 preview, which lists "the tagline lands" as a precondition) should cover the tagline.
   - This does not reopen the owner's headline decision; it corrects what the note says the policy permits. Not legal advice.

2. **Blocking: the fallback names contradict the naming record.**
   - research/28-positioning-and-name.md:80 lists "Fallback names, ranked by the brainstorm, if the owner renames: Neap, Ambit, Writ, Lockstep."
   - `tide-naming/decision.md` ("Runners-up, for the record") ranks **1. Neap, 2. Menir, 3. Shore**. It lists ambit, writ and lockstep under "Rejected with reasons in notes.md".
   - The note's own :103 says the owner compared Menir, Shore and Tide.
   - Line :80 is the pre-decision shortlist from notes.md:41/69–81.
   - **Fix:** "Runners-up recorded with the decision: Neap, Menir, Shore (decision.md); ambit, writ, lockstep and others were rejected (notes.md)."

3. **Blocking (trivial fix): two of the "other projects named Tide" are named Tidal, and one is unsourced.**
   - research/28-positioning-and-name.md:74 lists Tidal-Lang and TidalCycles as projects "named Tide".
   - notes.md:21–22 records "Tidal-Lang/Tidal" and "TidalCycles (named "Tidal" …)", and decision.md says "a music DSL named "Tidal"".
   - "Tibbo TIDE (an IDE)" appears in neither naming file and not in §Sources.
   - **Fix:** list Tidal-Lang and TidalCycles as near-collisions named "Tidal". Cite a source for Tibbo TIDE or drop it.

4. **Non-blocking: wrong date for the brainstorm.**
   - :76 dates the naming brainstorm 2026-11-13.
   - notes.md:1 ("2026-11-12, Opus brainstorm with Lucca"), decision.md and the note's own :103 give 2026-11-12.
   - **Fix:** 2026-11-12.

5. **Non-blocking: the `tide::main` count is stale and gives no scope.**
   - :101 says "about 84 uses of `tide::main`", from notes.md:28.
   - decision.md says "about 98 uses in design/ and crates/".
   - At this revision, `grep -rn --exclude-dir=e1-checked "tide::main" design crates | wc -l` gives 95 lines; adding conformance/ gives 153.
   - fx-gbq4's scope will be sized from this sentence.
   - **Fix:** give the count with its scope, for example "about 95 lines in design/ and crates/, about 150 with conformance/ (2026-11-13)".

6. **Non-blocking: a planned mechanism is in the present tense.**
   - :97 says "tide-admit maps `tide::` to the published `tidelang-*` crate".
   - No such mapping exists today. decision.md says it "can map", and fx-gbq4 makes it Acceptance ("tide:: source paths still resolve").
   - **Fix:** "would map (fx-gbq4)".

7. **Non-blocking: Tide's "Authority bounds: yes" cell mixes design with implementation.**
   - :33 gives Tide "yes", and Blaze "`bstd`, partial".
   - Blaze's cell describes implemented code. Tide's is the design, partly enforced today: tide-admit's ambient-operation table `FX0501`, README.md:71–73.
   - The replay cell (:34) and the edit-check cell (:29) already separate design from status; this cell does not.
   - **Fix:** "yes by design (D18); v0 checker enforces the ambient-operation table".

8. **Nit: wrong citation for the D12 target.** :18 cites "[repo: README Status note]" for D12's "tens of milliseconds". The targets caveat is the italic note at README.md:33–34, above "## Status". The figure belongs to D12 (01-design.md:802–803: about 30 ms typical, under 100 ms at p95). **Fix:** cite D12.

9. **Nit: "the three" is ambiguous.** :54 says "None of the three changes Eon's build today". The table has four columns, so "the three" could mean Rust, lust and Blaze. **Fix:** "Neither lust, Blaze nor Tide changes Eon's build today".

10. **Nit: "installable" overstates the Linux state.** :118 says "Nothing is installable outside Linux x86_64", which implies a Linux x86_64 install exists. Tide builds from the workspace there (README.md:128–131 per fx-1n5n), and :121 itself lists an install path (fx-1n5n) as still missing. **Fix:** "Tide builds only on Linux x86_64, from source; there is no packaged install".

11. **Non-blocking, out of scope (report to the coordinator; not a change to this file).**
    - fx-4cdq's bead body still carries the two claims Codex rejected in this note (k778 r1 finding 2): that Tide's subset already makes lust's "no unsafe" cut, and that "Most of its speed comes from cuts Tide's subset already makes".
    - The source `tide-research-blaze/comparison-2026-11-13.md:19,27–28` also still has the superseded memory cell, the claim that neither has authority bounds, and the "mostly from cuts" wording.
    - The note is now correct. The bead text is not.
    - **Fix:** the coordinator updates fx-4cdq's Why to match research/28 §2 conclusion 2: unranked reasons, granted unsafe, inference.

DONE

## Re-check

Model: claude-opus-5-5 · Date: 2026-11-13

Verdict: approve

Scope: fx-k778 blocking findings 1–3 only, checked with `git diff 9686e88..7a892ad` in the fx-k778 worktree (HEAD `7a892ad`, clean). I also checked whether the fixes introduced new problems. Read-only; no builds.

1. **Resolved: the trademark conclusion.** research/28-positioning-and-name.md §3 (:66):
   - It now quotes the "may not be used: To refer to any other programming language; To refer to a modified version of the Rust programming language, except as permitted below for minor changes" clause. The quote matches the policy page I fetched earlier today, word for word, as does the "official, affiliated, or endorsed" casual-observer rule.
   - It separates compatibility statements, which the policy allows, from using "Rust" as Tide's descriptor, and labels that as [inference].
   - It keeps the owner's headline and assigns the tagline to the trademark check. §4 (:110) and the §5 preview preconditions (:121) say the same.
   - Ledger commit d450d18 shows fx-gbq4 now covers the tagline.
2. **Resolved: the runners-up.** :80 now names Neap, Menir and Shore, citing decision.md, and lists ambit, writ and lockstep as rejected, citing notes.md. This matches both files and the note's own :103.
3. **Resolved: the other projects.** :74 now lists Tidal-Lang and TidalCycles as near-collisions named "Tidal", which matches notes.md:21–22 and decision.md. Tibbo TIDE is gone.

The fixes introduced no new blocking issues. The non-blocking fixes in the same diff check out:
- The date is now 2026-11-12.
- The `tide::main` counts, about 95 and about 150, match my counts of 95 and 153.
- The mapping is now future tense ("would map", fx-gbq4).
- The Authority cell is qualified.
- The D12 citation, "Neither lust, Blaze nor Tide", and the build/install wording are fixed.

The fx-tcde nit-1 change at a7a5813 was not re-checked, as instructed; text-only nits need no re-check (AP-PROOF-001).

DONE
