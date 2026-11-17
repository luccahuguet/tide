Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 9c0649a

# fx-qlek.12 r2: Opus review (converge round, report only)

Candidate 9c0649a8c9358fc506cccac177e95fb4b1d01515 on base 3345f5c. Diff SHA-256
6987ca2d86e26ee7358eed19af38a06a1a377ad09caa29c738ec63ccb1cf120c, which matches the brief.
This review read the bead, AGENTS.md, report.md, r1-codex.md, the guard test, the runner,
rules.md and README diffs, and the suite log's guard lines. It ran no builds and no tests,
and it left Codex's mechanical replacement check alone.

## Verdict

approve

## Blocking findings

None.

## Non-blocking findings

1. **Lowercase `fx` names for the code prefix are still in the code, and the report says they are gone.**
   - `crates/tide-admit/src/walk/reify.rs:60` defines `fn fx0206`, called at :43 and :56. It reports `Code::Td0206`.
   - `crates/tide-admit/tests/build.rs:3006` has the test `float_banned_methods_still_report_fx0520_with_production_std`. Its name appears in `cargo test` output.
   - `crates/tide-admit/tests/driver.rs:38` defines the helper `fx_lines`, which filters `": error: TD"` and has 37 call sites. `driver.rs:1973` has the loop variable `fx`.

   The builder renamed the same kind of `fx` variable in tests/invocation.rs but missed these. The report says "the code has no Ferox spelling left", which is not true. The guard can't see them because it matches only uppercase `FX`. Users never see these names, so the change still meets its contract.

   Recommendation: rename them (`td0206`, `..._report_td0520_...`, `td_lines`, `td`) in this candidate if it gets rebuilt anyway. Otherwise rename them with the next tide-admit bead and correct the report's claim on the bead. A rename like this changes no behavior, so only the finder needs to re-check it.

2. **The guard skips more than the historical set.** `code_prefix.rs:8-15` skips all of `trials/` and `spikes/`. The bead keeps "trials/e1-origins/ and the other frozen trial copies". The coordinator also kept spikes/, and that is fine. But `trials/e3-pilot/` (fx-1vwf.1, in flight) is live material, and skipping all of `trials/` leaves it unguarded once it lands. Today it has no FX codes, so nothing fails yet.

   Recommendation: change the skip entry to `"trials/e1"`. That still covers e1, e1-checked, e1-dialect, e1-origins and e1-round2, and it guards e3-pilot and any later trial.

3. **The guard needs a git checkout and the working tree's untracked files.** It runs `git ls-files` at `code_prefix.rs:26-31` and asserts success.
   - Failure scenario: someone runs `cargo test --workspace` from a source archive without `.git`, such as a GitHub tag tarball for the alpha (fx-qlek.7). The test fails with "git ls-files: fatal: not a git repository", and no other workspace test needs git.
   - Second risk: because it reads untracked files, an unrelated untracked scratch file in the checkout that quotes an old code fails the suite. One example is a review draft dropped into the repo root. Today the main checkout has only `.beads/` files untracked, and the guard skips those.

   This doesn't block a rename bead.

   Recommendation: decide before the RC. Either return early with an `eprintln!` when `root.join(".git")` is missing, or have the release notes say the suite needs a git checkout.

4. **The mapping note is where the bead put it, but readers won't find it there.** The note in `conformance/rules.md:15-19` is correct and its wording is clear. But readers meet the old FX codes in research/, reviews/ and trials/, mostly by following README links (README.md:528, 628, 664, 676). `conformance/rules.md` is a rule inventory that those readers have no reason to open. README's History section (`README.md:756-760`) already says that those directories "keep [the Ferox] name as written". It doesn't say that they also spell the codes `FX####`.

   Recommendation: add one clause there, in this bead or in fx-qlek.3's README rewrite: "and spell diagnostic codes `FX####`, the same digits as today's `TD####`". The guard allows the literal `FX####`. reference.md can't hold the note because it is at its token budget, and it doesn't need it.

5. **One frozen script still matches on `FX`.** `trials/e1-checked/measures.py:90,95` extracts codes with `(?:FX|E)\d{4}`.
   - `gate_reproduce.py` runs the frozen harness in a separate checkout at the gate commit, which uses that commit's driver, so the recorded gate results are safe.
   - If someone runs the README commands (`harness.py references tide`, then grading) from the current tree, it uses the current driver, which prints TD codes. `diagnostic_codes` then silently drops the TD codes, and TD-coded warnings count as unnamed lints.

   The directory is frozen evidence, so leave it unchanged. Mention this on the bead in case E1 is ever rerun on the current driver.

   The other matchers fail loudly, which is what we want:
   - `conformance/run.sh` now marks a stale `//~ FX####` mark as "malformed marks" and fails.
   - Its driver-output regex expects TD, so a driver that printed FX would fail the case.
   - The closed branch fx-6xi2 still adds two `//~ FX0206` marks. If it is ever revived, the guard and run.sh both catch them.

6. **Nit:** the note in rules.md says the bead landed "(2026-11-17)". If it lands on another day, update the date.

## Reasoning

- **The guard is strong, cheap and simple.**
  - **It runs.** It is an auto-discovered integration test of tide-admit, and Cargo.toml doesn't set `autotests = false`. Lines 146-150 of the workspace log show both tests passing.
  - **It reads the right files.** It reads every tracked or untracked file that git doesn't ignore: 578 live files, all text (483 `.rs`, 72 `.md`, plus toml, sh, lock, json and py). No binary files are there to cause false positives.
  - **Its matcher is right.** It finds `FX` followed by four ASCII digits anywhere in a line, which is exactly the Acceptance's `rg 'FX[0-9]{4}'`.
  - **It can't pass empty.** It asserts that it read more than 100 files.
  - **It doesn't flag itself.** The matcher test builds its positive sample at run time, so the guard's own file passes.
  - **Its negative samples are the right ones.** They cover `FXB1` (the Bend request header in design/03:2456), `fx-` bead IDs, `Fx0206` and `FX####`.
  - **It doesn't flag legitimate text.** Without a word boundary it could in principle match `FX` plus four digits inside a longer token, but the Acceptance pattern behaves the same way and no live file has such text.
  - **It sees every place users see codes.** The `Code` enum's strings, the conformance marks, docs, the design and the README are all literal text. The one exception is case: lowercase identifiers aren't user-visible (finding 1).

- **User-facing text.**
  - The diag.rs module doc, the README, conformance/README.md and rules.md now consistently say "TD codes".
  - conformance/README.md now calls the D18 spike's marks "per-line code marks", and that is correct.
  - `design/reference.md:17` still calls the codes placeholders. That design question is out of scope, and the report says so.
  - No paper or launch draft is in the repository yet: fx-qlek.3, .4 and .7 are ordered after this bead.
  - Nothing describes the codes' history wrongly. The only gap is where the note sits (finding 4).

- **External matchers.**
  - run.sh's awk and grep patterns and its renamed `td` variable are consistent.
  - `--tide-print-codes` gives the CODES alternation, so the runner never hard-codes the prefix in that path.
  - demo/ and scripts/ match no codes.
  - The E3 pilot branch parses no codes.
  - The one frozen matcher, `trials/e1-checked/measures.py`, is covered in finding 5.

- **Rebase.** main has moved to 9a5e657 since 3345f5c. The two new commits change only `.beads/issues.jsonl`, which the guard skips and no check reads. So a rebase changes none of the checks' inputs, and the reference recount still stands.

- **Why nothing blocks.** Every finding is either an internal name, a hardening change, or a note on a historical or frozen surface. None breaks the bead's Acceptance or a design tier. Findings 1 and 2 are cheap enough to fold into this candidate, and I'd do that if anything forces a rebuild.

DONE
