Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 20f26c7

Review round under GOVERNANCE.md's Merge Rule (docs-only research), bead fx-gbq4. Read-only. Scope: `git diff main...20f26c7`, which is research/35-trademark-check.md (new) and the one-line pointer at the end of research/28 §4. I read AGENTS.md, GOVERNANCE.md, fx-gbq4 and fx-1n5n (`br --no-db`), research/28 §3–§5, README.md (headline, License and Name), design/01-design.md (D18), the builder's report, r1-codex.md, r1-recheck-codex.md, and the saved evidence in `target/review/dl/`.

## Verdict

**request_changes**. Two findings block. Both are small text fixes, and neither is likely to change a verdict. Once they are fixed and re-checked, the note can land.

## Blocking findings

### B1. The tagline analysis describes the driver less fully than D18 does, and skips the std substitution (research/35-trademark-check.md:58–59)

- **Problem.**
  - Point 2 says "Every Tide program is a valid Rust 2024 program that stock rustc compiles."
  - Point 3 says tide-admit "rejects programs outside the subset; it does not change the meaning of the programs it admits."
  - design/01-design.md D18 (line 1065 on) says more than that. Every Tide program is a valid Rust 2024 crate "that a pinned, unmodified nightly rustc compiles, with Tide's meaning, against Tide's std. The driver passes that std as `std` (`--extern std=`), so rustc's std injection loads it and the real std's paths cannot be named."
  - So the driver does more than reject. It swaps the library that `std` names. That is the strongest reading against the tagline: a reader could call it "a modified version of the Rust programming language", or say "subset of Rust" is inaccurate because `std::…` in Tide is not Rust's std. The analysis does not deal with it.
  - Line 39 mentions "against Tide's std", but the points the verdict rests on (2 and 3) leave it out. That makes the reasoning look more airtight than the design allows.
  - This is a mismatch with the design record (lens 4) in the evidence behind the verdict the owner will receive (lens 3).
- **Minimum fix.**
  - Make point 2 match D18's wording: a pinned, unmodified rustc compiles it against Tide's std.
  - Rewrite point 3 to name the mechanism. The driver also passes Tide's std as `std`. That substitutes a library through a standard rustc option. It does not change the language or the compiler, and Tide does not call that std "Rust's std" or call anything "Rust".
  - Add this to the "keep it low" conditions: never describe Tide's std as Rust's std.
  - The verdict can stay "low (preliminary)" if that reasoning holds.

### B2. The note leaves out the closest known unregistered uses of "tide", which are already recorded in research/28 and the bead (research/35-trademark-check.md:23, 123–127, 149–158, 238–243)

- **Problem.**
  - §2 lists, as the unregistered TIDE developer tools to follow up, only a Tcl IDE, Tibbo's IDE and "Tide Software Inc.".
  - It leaves out the same-field uses that fx-gbq4's own Why and research/28 §4 record:
    - http-rs/tide, a Rust async web framework: about 3.7M downloads, the `tide` crate and the `tide-*` plugin crates, so the same ecosystem and the same audience as Tide;
    - IlanCosman/tide, a fish prompt with a `tide` command;
    - ananthakumaran/tide, an Emacs TypeScript IDE;
    - Stanislas7/tide, an experimental language named Tide.
  - For rights that come from use, which the note itself calls unassessed, http-rs/tide is the nearest earlier user in Tide's field. It is closer than anything in the USPTO table.
  - A reader who has only research/35 gets a "low, provisional" name verdict without these hits. The follow-up list would also miss them.
  - This fails "each mark's sources, hits and verdict" (lens 1) and weakens the calibration (lens 3).
  - It does not contradict keeping "Tide": research/28 §4 accepted these clashes and chose `tidelang` and "the Tide language" to tell Tide apart from them.
- **Minimum fix.**
  - Under "Other sources", add one item that cites research/28 §4 and names these four projects as known unregistered uses in software, with http-rs/tide as the closest in field.
  - Add them to the list of unregistered-rights searches at lines 123–127.
  - Mention them in the "Not assessed" line of the summary and of §2's verdict.
  - Present the disambiguation steps as the answer to these uses, as research/28 decided.
  - The verdict word can stay as it is.

## Non-blocking findings

1. **Tide Platform's home registers should be the first gap query** (lines 102–113, 138). Tide Platform Ltd is a UK fintech. The saved sample also holds its cancelled Madrid designation 79193510 (class 36, `dl/uspto.json`), so registrations at home or in the EU are likely. Their class 9 and 42 specifications may not carry the US filing's "banking, finance, accounting, and tax" limit. Make the UK IPO and EUIPO queries name Tide Platform Ltd first. They should check whether its class 9 and 42 specifications are limited to a field.
2. **The Nix default should run on its own** (lines 322–338). `apps.default` runs tide-admit. Per D18, the driver cannot admit a crate without Tide's std. Of the two options left to fx-1n5n, only wiring `tidelang-std` into the driver's runtime closure keeps `nix run` on the default working. "Or documents it" would ship a default app that does not work on its own. Wiring it in also meets fx-1n5n's Acceptance ("`nix build` produces tide-admit and tide-std"). Recommend the closure option, or say that the default must work with nothing else installed. The names themselves are sound:
   - `tidelang-admit` and `tidelang-std`;
   - `packages.default` and `apps.default` both point at the driver;
   - no bare `tidelang`, which stays reserved for the CLI;
   - `tidelang-macros` is internal, a build input of the std.
3. **"Change" with no flake yet** (lines 324–325). "only Nix attribute names and package names (`pname`) change". The repository has no flake.nix, so these names are being set, not changed. Say "are set to `tidelang-*`".
4. **A section refers to itself** (line 211). "(§2's registration findings)" appears inside §2. Say "the registration findings below".
5. **The secondary legal source can't be checked** (lines 201–202). https://www.stites.com/?p=16166 returned HTTP 403 to my fetch and is not saved in `dl/`. The "super marks" claim depends on that one law-firm blog. Save it, or label it a secondary opinion. It is already only background to an inference.
6. **Say what "low" measures** (Summary). A reader who is not a lawyer may read "Low" as cleared. One clause would fix that, for example "low: no evidence found of a conflict likely to force a rename; not a clearance".

## Reasoning

**Acceptance (lens 1).**
- §1 covers the tagline: the policy's words, the analysis and a verdict.
- §2 covers "Tide": sources, a hits table and a verdict.
- §3 covers `tidelang`: hits and a verdict.
- §4 covers the Nix names, with reasons.
- The tagline risk is stated plainly enough for tide-lead to relay. Recording it on fx-gbq4 is tide-lead's step and is still pending, which is not a fault of the note.
- The gap is B2: the known same-field hits are missing from "Tide"'s hits.

**Evidence fidelity (lens 2).** Checked against the saved evidence and live sources:
- Rust policy quotations: all match `dl/rustpolicy.txt` (lines 33, 37–41, 50, 60, 65), including the "'official', 'original' or 'only'" example.
- `dl/uspto.json`: exactly 100 hits, `totalValue` 1265.
  - Every class 9 or 42 hit in the sample appears in the table: indices 5, 11, 16, 26, 42/43, 63, 72, 79, 83 and 98. Owners, classes and live or dead statuses match.
  - Both owner names for 90452380 are shown.
  - Tide Platform's class 9 and 42 goods are limited to its field, as quoted.
  - P&G's 1063680 (laundry detergent), 3495689 (charitable volunteer programs), 6555117 and 6555118 (class 37 laundry), 8161353 (class 39 laundry pick-up) and 3373965 (apparel, class 25) all match.
  - No P&G record in the sample is in class 9 or 42.
  - The live third-party registrations listed are correct.
- GitHub: the saved and live repository search gives 2 repos, with matching dates and stars.
- Code search: the saved response has 0 hits with `incomplete_results` false, and the control search returns 2 files.
- crates.io: all five names return 404 live.
- MELPA has a `tide` package. No bare `tide` attribute appears in the saved package-set files.
- `dl/tmview.json` is the bot-challenge page, as the note says.
- Live descriptions: Verus is "Verified Rust for low-level systems code", and hax's README says "a large subset of".
- Evidence and inference are labelled, apart from B1, where the D18 mechanism is stated incompletely.

**Calibration (lens 3).**
- After Codex round 1, the hedging is honest:
  - verdicts marked provisional;
  - 100 of 1,265 records stated;
  - registers and unregistered rights listed as not assessed;
  - dilution left unresolved;
  - not-legal-advice up front.
- The follow-up queries are concrete: a field-tag query, register URLs and filters.
- B2 and non-blocking 1 aim those follow-ups at the most likely conflicts.

**Consistency (lens 4).**
- The headline quoted matches README.md line 3.
- The note agrees with research/28's reasons for adopting "subset of Rust" and rejecting "rerust".
- It agrees with research/28's disambiguation steps, README's License and Name section, and fx-1n5n's scope (names under `tidelang-*`, binary and crate renames out of scope).
- The research/28 pointer is accurate and adds no new decision.
- Nothing argues against keeping "Tide".
- B1 is the one place where the note departs from design/01-design.md.

**Clarity (lens 5).**
- An owner can act on the summary table alone.
- Repetition is modest: the gap list appears in the summary, in §2's sources and in §2's verdict, which is acceptable for a record that is read section by section.

No builds, subagents, edits other than this file, contacts, filings or posts. Web reads: crates.io API, the GitHub API and raw README, and stites.com (403).

DONE
