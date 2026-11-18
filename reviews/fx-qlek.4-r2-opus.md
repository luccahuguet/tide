Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate 16fb3fb (docs review round)

## Verdict

**changes required.** Three blocking findings. Each one is a factual or caveat defect with a fix of one or two sentences. None of them changes a design decision, and none needs new evidence. The paper is otherwise accurate and well hedged, and it honours the owner's decisions. I found no layout defects.

## Scope

- Candidate: worktree `/mnt/build/eon-agents/worktrees/tide-fx-qlek.4`, branch `fx-qlek.4`, HEAD `16fb3fb5d0b3434541c805e4388bf6808e3cd5c6`, base `d4a91a2`.
- SHA-256 of `git diff d4a91a2..16fb3fb`: `978848d068fbc52b71dda90b501e1e0666fda30dcde20ffde66921e8c99b424f`. The diff changes one file, `paper/tide-v0.typ`, with 213 lines added.
- Read first: AGENTS.md, GOVERNANCE.md, and `br --no-db show fx-qlek.4` with all of its comments. Those include the A1/B decision of 2026-11-17, the Typst decision of 2026-11-18, the Effect/Rivet/Alchemy note, the positioning note, the Anthropic/Theorem and plusthree candidates, and the date correction.
- Read the whole paper and checked it claim by claim against these sources: `trials/e3-pilot/report.md` (all of it) and `protocol.md` §12, `trials/e1-checked/gate-report.md` (all of it), `research/36-benchmarks.md` §1–§2, `demo/README.md`, `reviews/2026-11-17-red-team-opus.md`, `docs/owner-guide/03-soundness-bugs.md`, `06-replay.md`, `08-sandbox.md` and `10-v0-and-planned.md`, both case-study READMEs, `design/01-design.md` (Thesis, Tiers, Evidence, D0, D7, D10, D14, D15, D18), `design/reference.md` (when clauses run), `reviews/2026-11-15-achilles-opus.md` (W1, W3, W7), `reviews/round1-agent-usability.md`, `research/05`, `research/07`, `research/16`, `research/28`, `docs/debugging.md` and the git history's co-author lines.
- Rebuilt the PDF with the pinned `typst 0.15.0`. It has 9 A4 pages, and its text is identical to `target/work/tide-v0.pdf`. I rendered every page to PNG and inspected pages 1, 5 and 9 at higher resolution.
- I read the author's claim table as claims to check, not as evidence. I read `r1-codex.md` and `r1-recheck-codex.md` only after my own pass. None of the findings below repeats theirs.

## Blocking findings

### B1. The disclosure names only Opus and Sol, but Claude Fable wrote production code

`paper/tide-v0.typ:22`: "Tide's code, its design text, and this paper were produced by AI agents (Claude Opus and GPT Sol models)".

- **Source that contradicts it:** the git history. Five commits carry `Co-Authored-By: Claude Fable 5.2`, and all five are production code still in `crates/`:
  - `84aebe8`, the seeded baton scheduler (fx-nkv.5);
  - `08a77c4`, Clock, Instant and virtual time (fx-o8j);
  - `d3106f1`, the capability-confined real file operations;
  - `adcb115`, the driver-built property tests;
  - `ab42dd6`, `derive(Encode, Decode)` and `std::json`.
- The parenthetical reads as a complete list. The author line's disclosure is a Scope requirement of the bead, so it must be exact.
- **Minimum fix:** "(mainly Claude Opus and GPT Sol models; some code by Claude Fable)". Alternatively, "(Claude and GPT models)" with no exclusive list.

### B2. "Cannot in general be replayed" is contradicted by the repository's own rr comparison

`paper/tide-v0.typ:40`: "a test of concurrent Rust code cannot in general be replayed, because its dependencies reach real time, real threads and real randomness."

- **Source that contradicts it:** `docs/debugging.md:158-177`, "rr compared". It says "rr is the closest tool to Tide's replay. It records a run of an ordinary Linux program and replays it deterministically under gdb". rr applies to "unmodified Linux programs, whatever language they were written in".
- A PL reader will know rr. As written, the motivation claims a gap that existing tools already fill. The real difference is narrower, and the repository states it:
  - Tide fixes the schedule from a seed and searches seeds, rather than recording one real run.
  - Tide needs no hardware performance counters.
  - A Tide seed is a number that can be rerun on the same source.
- §5's deterministic-simulation paragraph (`:160`) also leaves out rr, although the repository names it as the closest tool.
- **Minimum fix:**
  1. At `:40`, say that a concurrent Rust test cannot in general be *rerun deterministically from a seed*. Record-and-replay tools such as rr can record one real run and replay it, but they do not let a test search schedules.
  2. Add one sentence on rr to `:160`, citing `docs/debugging.md`.

### B3. The Eon case study drops the source's tokio paused-clock caveat

`paper/tide-v0.typ:146`: "Rust can write the same test. What Tide adds is that the test runs in virtual time, deterministically, with no fake clock written for it".

- **Source that contradicts it:** `examples/case-studies/eon-slow-close/README.md:8-11`. Right after the same sentence, the README says: "Async Rust gets the same from tokio's paused clock; Eon's thread-based supervisor would need a clock it is handed."
- Without that sentence, a reader would take virtual time with no hand-written fake clock as something only Tide gives. The source says it is not. The hyper README draws the same contrast at `:173-180`.
- **Minimum fix:** after "no fake clock written for it", add: "Async Rust gets the same from tokio's paused clock; Eon's thread-based supervisor would need a clock it is handed."

## Non-blocking findings

### N1. The 48 routes include controls

`paper/tide-v0.typ:30` and `:92` say the red team "tried 48 routes".

- **Source:** `reviews/2026-11-17-red-team-opus.md:10` says "48 routes tried, counting the accepted controls for route 6". The owner guide says "over 40".
- **Minimum fix:** "48 routes, controls included".

### N2. fx-qlek.5's remaining gaps are not in Limitations, and `:169` describes the record loosely

The bead's Scope asks Limitations to cover "fx-qlek.5's known holes". The paper covers rustc's other open `I-unsound` issues, the #100051 family and the two routes admitted on purpose (§4.1). It leaves out the gaps that the red-team record leaves open (`reviews/2026-11-17-red-team-opus.md:127-150`):

- Under a pin bump, `core::*` globs such as `core::mem::*` and `core::cell::*` still admit whatever a later rustc adds.
- RUN-3 does not confine metadata reads (`stat`, `access`).
- `TD0402` fails closed and rejects some harmless impls.

Separately, `:169` says the record "lists what it did and did not try". The record lists the routes it tried and a "Left for others" section. It has no list of routes it did not try.

**Minimum fix:** add one clause to the sandbox bullet at `:170` for the metadata reads, and one to `:169` for the pin-bump globs. Reword "did and did not try" to "what it tried and what it left open".

### N3. "In any dependency" is stated in the present tense for a v0 that has no dependencies

`paper/tide-v0.typ:50`: "A function that takes no capability cannot read … in any dependency." This sentence comes from the design thesis, but in v0 the driver admits a single crate. §5 (`:158`) hedges the same point correctly: "by design in dependencies too (v0 admits a single crate)".

**Minimum fix:** "in any dependency, by design".

### N4. The paper calls its own author "the owner"

`:80` ("the order the owner set for the alpha") and `:121` ("the owner declined a second-model run"). The author line names the owner, so this third-person reference reads as project-internal.

**Minimum fix:** "the author", or drop the attribution at `:80`.

### N5. Some terms are used before they are defined

- `:62` uses "the TD diagnostics" and "the `Pure` trait" without defining either.
- `:22` uses "Codex" before naming it as Sol's CLI.
- The body cites bead IDs (`fx-6xi2`, `fx-qlek.5`) without saying they are issue-tracker IDs.

**Minimum fix:**
- At `:62`, add a short gloss: TD codes are the driver's diagnostics, and `Pure` marks closures and values that can reach no authority.
- At the first bead ID, say "(an issue in the project's tracker)".

### N6. References 12–18 are never cited in the text

Only [11] and [19] appear in the body as numbered citations. The in-repository sources are cited inline as links.

**Minimum fix:** add the bracketed numbers where Rust, lust, Effect, Rivet, Bend, Eon and hyper are first named. Alternatively, label 12–18 as "further links".

### N7. The abstract says "neither" for more than two bugs

`:30` and `:144` say admission catches "neither of the real bugs studied". The studies cover four reports: eon-8wh7h, eon-pimct, hyper#2808 and the hyper#2595 panic.

**Minimum fix:** "none of the real bugs studied".

### N8. The in-repository links are relative

This is already known: Codex N1 and the author's report record it. The PDF's in-repository links resolve only beside a checkout. They must become public URLs before posting. This is the owner's step and does not block this round.

### N9. The status line's "every number" has one exception

`:24` says "Every number in this paper comes from a report in the repository". The exception is the date of the Anthropic and Theorem post ("October 2026", `:162`), which comes from the external post. This is a nit.

**Minimum fix:** "Every measurement in this paper…".

## Reasoning

**Scope sections.** All seven are present and in the order the bead asks for:

- §1, motivation;
- §2, the design principles: low entropy, capabilities, deterministic concurrency through the simulator, contracts, admission, and the trust surface;
- §3, the implementation: the driver, tide-std, the simulator and replay, RUN-3, and conformance;
- §4, the evidence;
- §5, related work: Rust, lust, Blaze, capabilities and effects including Effect, simulation including Rivet and plusthree, and verification including Bend and Anthropic/Theorem;
- §6 and §7, limitations and roadmap;
- §8, how it was built.

The Limitations section covers every Scope item: Linux only, a single crate (W1), replay of the model (W7), rustc soundness (W3, fx-6xi2), E1 near ceiling with one model, the failed E3 threshold, and the benchmarks. N2 lists the fx-qlek.5 gaps it leaves out. The PDF is 9 pages, within the 8–12 the bead asks for.

**Owner decisions.**

- **A1, alpha framing after E3.** The evidence leads with replay and admission. E3's first paragraph is quoted verbatim; I compared it against `trials/e3-pilot/report.md:3`, and it fills in protocol §12's sentence. The ceiling explanation is included. The trust diff is described only as "a summary of what a change may do". The paper says twice that the pilot found no evidence that the trust diff helps reviewers catch more (`:66`, `:106`), and it is not in the headline.
- **"Three rustc soundness holes closed".** The paper renders this as "three admitted routes from two rustc soundness bugs", which matches both A1 and the landed reports.
- **"Unforgeable capabilities".** A1's wording says this, but the paper deliberately states the narrower "no accidental ambient authority" (`:94`). I agree with that choice: `docs/owner-guide/03-soundness-bugs.md`, "The hardest question", sets that exact claim because rustc has other open `I-unsound` issues. I do not count it as a departure from A1. The owner should know the word is absent when approving.
- **B, post-alpha work.** The per-operation Fs authority and the E3 follow-up are described as planned after the alpha (`:106`, `:178`).
- **Typst format.** The source is `paper/tide-v0.typ`, the PDF is not committed, and the typst pin is recorded in the header.
- **Single-model limitation.** It is stated twice, with the owner's refusal of a second run.
- **Eon bugs.** They are named publicly, which the owner allowed on 2026-11-15.
- **Positioning.** The paper calls Tide complementary to Rust, "not a fork or a replacement", and "AI-welcome … because what they wrote is checkable" (`:154`, `:186`).
- **Anthropic and Theorem.** The post is cited as context only. The paper uses none of its figures and says explicitly that "Tide is not formally verified".
- **plusthree.** It is cited only as an independent peer. Codex's re-check verified it against the primary source.
- **lust and Blaze.** They appear as related work only, and the paper says the alpha does not benchmark them.
- **Effect and Rivet.** Both contrasts are as the 2026-11-15 note says.

**Claims checked and found accurate.**

- **Replay (demo/README).** 8 of 16 seeds pass and 8 fail; the failing seeds are 0, 2, 4, 8, 9, 10, 12 and 14. The recording and both replays have 40 events and identical hashes. The fixed revision passes 16 of 16. The `+ metrics: net` widening matches. "Same machine" matches `06-replay.md:180`, and the driver-only manifest matches `:190`.
- **Admission.** #25860 has been open since 2015. TD0206 and TD0402 match the owner guide, and the paper uses the TD prefix that fx-qlek.12 introduced. The record lists two routes admitted on purpose. The checked reproductions of #84591 and #100051 are covered, and the #100051 family stays open.
- **E3.** The ceiling arithmetic is right. 11 of the 12 pairs tie. Every miss is on i12, and 11 of its 12 sessions approved it. There were 3 false positives, all from Sol on i18, with the trust diff showing `fs.read, fs.write`. The items were 10–120-line changes reviewed with one prompt, one reply and no tools.
- **E1.** The paper uses 56/60, 59/60, 58/60 and 86/58 against 59/58, and the ratios 0.949153, 1 and 1.457627. The interval of 1.29–1.64 crosses 1.5. The median sessions were 40.3 s and 30.6 s. All four final failures are the C3 structure slip. The classification found 3 cases of a Rust habit leaking and 0 toolchain, reference or design gaps. The raw logs are outside the repository, as the report says.
- **Benchmarks.** Every ratio in the bullets matches research/36 §1.1 and §1.4 to the precision the note gives: 3.8–4.6x, 5.2–6.3x, +17% to +39%, 1.4–2.0x, 0.99x, 1.05x, 1.19x, 7.7x, 9.2x, 1.04x with 26 of 30 rounds and 1.07x, 1.18x, 2.29x with 1.74x, and 12.8x. The quoted stability hedge is verbatim, with one marked elision. The verdict quote is verbatim. The linker and std caveats are kept.
- **Case studies.** These figures match the READMEs: 6992 ms, 6011 ms, the 500 ms grace, 3.78 s, 20 seeds, E0433, the #2808 reaper that is admitted and fails its test, and the eon-pimct failing test.
- **Design.** These match the design record: the D18 driver flags and allowlist, the "no separate equivalence argument" point, the D10 split between checked and sampled clauses (`reference.md:877`), D14's formatter marked as planned, D15's rule that virtual time advances only when every task is blocked, the manifest contents, the RUN-3 fork/exec placement (`08-sandbox.md:45`), Landlock ABI 6, and the v1 and Later tier items in §7.
- **Prior evidence.** RustAssistant's 10.74% → 73.70% (`research/05:61`) and the 76.8% census figure match. The capability and simulation systems listed in §5 all appear in research/07 and research/16. The lust and Blaze characterisation matches research/28 §2.
- **Governance (§8).** It matches GOVERNANCE.md: the human-code exception, the review order, the stand-in rule, the planning-only and Starcompass carve-outs, and where reviews are recorded.

**Why B1–B3 block.** Each one either states something a landed source contradicts (B1 against the git history, B2 against docs/debugging.md) or drops a caveat that the cited source states in the next sentence (B3). The brief lists both as claim defects. A skeptical PL reader would catch B2 and B3 on a first read. All three are text fixes, so the author's fixes go back to me for a bounded re-check under the Merge Rule. GOVERNANCE says a text-only fix needs no re-check, but B1–B3 change what the paper claims, so I would still look at them.

**Readability.** The argument is clear and leads with its negatives. The abstract states each negative result, and every evidence subsection hedges before it concludes. Nothing reads as marketing except the single "AI-welcome" sentence, which carries owner-chosen positioning and is backed by a concrete reason in the same sentence. The terms in N5 are the only ones a reader new to the project would stumble on.

**Rendered PDF.** It has 9 pages with no clipping or overlap. The E1 table renders with its header and wrapped cells intact, and the block quotes are indented. The reference list wraps cleanly. The table has no caption, which is a nit.

Not checked: external pages (I relied on Codex's plusthree fetch), Rust builds, reruns of trials, demo or benchmarks, and live sandbox probes.

DONE
