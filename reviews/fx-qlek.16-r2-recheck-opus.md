Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidates Tide 14ecb5d, site 048ee97 (re-check)

This is the finder re-check of my r2 findings B1 to B5, and of the non-blocking items the builder took (N1 to N6, N9 and N12), against `git diff 7eb5585 14ecb5d` (Tide) and `git diff 555cc9c 048ee97` (site).

- I read report.md "Round r2 fixes" and WORK/r2-checks.log.
- Both worktrees are clean, and their HEADs are 14ecb5d and 048ee97.
- I started no run. The runtime results below are the builder's, read from r2-checks.log.

## Run record coverage

`git diff --name-only a4ab41c 14ecb5d -- conformance crates Cargo.toml Cargo.lock rust-toolchain.toml` lists only `conformance/run-record.txt`, which the generator excludes. The a4ab41c record (389 PASS / 3 AWAITS / 0 FAIL) still covers 14ecb5d. The generator ran at 14ecb5d without refusing the record (r2-checks.log:93, "match 14ecb5d…").

## Blocking

- **B1 resolved.**
  - The h2 is now "`{pass}` of `{total}` conformance cases give every expected verdict" (389 of 392).
  - A linked paragraph follows: "The other `{awaits}` match every mark today's checker decides, and `{fail}` fail."
  - Both match conformance/README.md:61-62 and 90-93.
  - The e2e test asserts both, and the falsifier still passes (log:92).
- **B2 resolved.** status.astro now says "E3 ran under a protocol frozen before the run, an order that rests on Tide's own records (self-attested)". This agrees with glance item 9.
- **B3 resolved.**
  - The gloss is now "The search's frozen portability criterion, E4 in its bead, barred…", which matches research/41:150 and :192.
  - A new glossary entry, "E4: The searches' eligibility criterion that a bug can be ported to Tide v0", follows the entry that defines port and comes before tile 4. It matches research/41:97 and research/38:14.
  - The e2e glossary test requires E4.
- **B4 resolved.**
  - `bullet()` reads the one item that starts with the given text, including every two-space continuation line, and throws on 0 or more than 1 matches.
  - `rowOf()` goes through `table()`, so the header, separator and cell count are checked, and it requires exactly one row with that label.
  - The atomics reader uses `bullet`. The weaker-orderings reader uses `rowOf` under `## 11. Unsafe, trust holes, and review`, which is reference.md:951 verbatim.
  - The anchor is computed from that heading, so it is the same as before.
  - New tests cover a four-line bullet read whole, a bullet stopping at a non-continuation line, 0 and 2 matches, and a reordered §11 table that throws /header/.
  - status.json's design texts are unchanged.
- **B5 resolved.**
  - `checkProtocols()` lists `research/protocols/` at `tideRevision` and requires each `<bead>-protocol.md` to be a listed round or a key of `notRounds`. A `notRounds` key with no protocol throws as stale.
  - At 14ecb5d the protocols are fx-qlek.17 (excluded, with its reason in the generator), fx-zu4f.8, .9 and .11 (all rows).
  - Tests cover the accepted case, an unknown protocol and a stale exclusion.
  - A future fx-zu4f.10 or .12 to .15 protocol under the existing naming now stops the generator until tide-lead lists or excludes it.
  - Residual risk, not a defect: a protocol filed under a different name, or in a subdirectory, would not be seen. The `qualifier()` lookup uses the same naming convention, so the two stay consistent.

## Non-blocking items taken

- **N1 resolved.**
  - The glance adds "When those searches examined their candidates still rests on our records."
  - The README block still equals docs/at-a-glance.md byte for byte (checked in Python).
  - The site copy equals docs/at-a-glance.md once its two pinned links are mapped back to root-relative form (checked with `diff`).
  - The status glossary's anchored entry carries the same sentence.
- **N2 resolved.** The label "16 with a portable source, the pool less those its record finds none for" now says the count is derived (87 − 71).
- **N3 resolved.** The caption is now "The limits research/41 maps…", the region's aria-label matches, and the Planned gloss is "raw locks" only (research/41:191).
- **N4 resolved.** The text is now "Three of the searches recorded, per bug, the first criterion it failed."
- **N5 resolved.** The record's `inputs` must include all five runner paths. Today's header passes.
- **N6 resolved.** The generator reads "For the pilot and N of the searches" from the whitespace-normalized glance and throws unless N matches the count of self-attested round beads. That count is 4: fx-zu4f.5, .6, .7 and .9.
- **N9 resolved.** The fidelity note is now read only inside the "### Simulator fidelity" section, up to the next heading.
- **N12 resolved.** "tests pass" now links to examples/capabilities/README.md, whose table declares that outcome (:13).
- **Not taken:** the builder's reasons for leaving N7, N8, N10 and N11 are sound. All four were non-blocking, and N8, N10 and N11 can go to the fx-qlek.7 cold reader.

## New problems from the fixes

None found.
- The added prose has no semicolons outside code, and the e2e semicolon test passes.
- status.json changed only in `tideRevision`, the input SHA-256s, the two glosses and the fx-zu4f.8 label.
- charts.json changed only in `tideRevision`.
- Both pins name 14ecb5d.
- reference.md is untouched, so the 24,999 count still stands.

Builder-reported checks (r2-checks.log), all through heavy.sh:
- Typos clean.
- astro check: 0 errors, 0 warnings, 0 hints.
- test:status: 18 pass, 0 fail.
- Build succeeded, and the falsifier changed the tile.
- `--check` matches 14ecb5d.
- The pending proof at db130d0 is rejected, and four complete proofs are anchored.
- e2e: 30 passed.

## Verdict

**Approve.** B1 to B5 are resolved as my minimum fixes asked, and the non-blocking items taken are resolved. No fix introduced a new problem.

DONE
