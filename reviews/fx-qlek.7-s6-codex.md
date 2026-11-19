Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidates Tide c1b8053 + site 4ff61db (code, S6)

## Verdict

**request_changes — high confidence.** One blocking finding remains: audit finding 9 is corrected in the shared at-a-glance block and glossary, but two other site paragraphs still promise TD diagnostics for all failures. The Tide changes, generated figures, governance responsibility lists and site pin otherwise satisfy this pass.

Read-only review of Tide `d6ecd25..c1b80537f4405911c65ef76502d7385a7c6eecf2` excluding `.beads`, and site `0f586a2..4ff61db8013882a00bd32ccf93b5ffca7926424f`. Both source trees were clean at entry. No contact, commits, source edits or `.beads` writes. Private launch drafts are outside this report.

Frozen diff SHA-256:

- Tide (`git diff d6ecd25 c1b8053 -- . ':!.beads'`): `1baaf04e92e40e3b1ab804c7344785967d391f6f70851e186dbf7b6771686a7d`.
- Site (`git diff 0f586a2 4ff61db -- .`): `af61e9df6801724435ef55cf21d7c0f9dc840976bf722f873a03d04fc039a6e2`.

## Blocking findings

### B1. Two site paragraphs retain the diagnostic guarantee finding 9 removes

**Locations:** site:`src/pages/index.astro:110`–111; site:`src/pages/how-it-works.astro:90`–91.

The home page says the driver “names each rejection with a TD code.” The walkthrough says “A program that fails gets TD errors.” Both are unqualified. A crate rejected by rustc itself can produce only rustc errors. Tide's paper gives a concrete existing example at `paper/tide-v0.typ:275`: the hyper timer rewrite gets only `E0433` because the external crate is unavailable. The corrected `docs/at-a-glance.md:13`–17 and site:`src/pages/status.astro:33` explicitly distinguish these cases.

**Failure scenario:** a visitor follows either paragraph, gets a rustc-only failure, and cannot find the promised TD code. The home page also contradicts its own regenerated at-a-glance block. This leaves scoped audit finding 9 incomplete, even though the builder's report marks it fixed.

**Minimum fix:** qualify both remaining sentences as failures of Tide's admission rules, and state that code rustc itself rejects gets rustc's own errors, using the same distinction already adopted in the glance and glossary. Update the report's finding-9 row to include both locations. No driver change is needed.

## Non-blocking findings / follow-up

1. **Preserved governance wording needs its already-recorded follow-up.** Tide:`GOVERNANCE.md:32` still says “Humans define the system”; `GOVERNANCE.md:52` and `paper/tide-v0.typ:426` retain final human merge authority. The builder lists both in `target/work/report.md:55`–56. These are inherited wording tensions, preserved under the explicit instruction to change only the responsibility lists and leave the Merge Rule unchanged. They are not grounds to expand this patch's authorized governance edit.
2. **List the rendered overlay for the single-holder follow-up.** Tide:`AGENTS.md:331` and `.agent-protocols.local.md:72` restate the owner's scope, priorities and direction duties; `AGENTS.md:306` and `.agent-protocols.local.md:47` assign technical decisions and landing approval to Opus. Their current division is consistent with steering. Include these sections in the governance follow-up inventory, as requested, and update through the overlay plus renderer only if an alignment change is chosen. Do not hand-edit AGENTS.md.
3. **Site clip instructions still describe the superseded unconditional step.** Site:`README.md:103`–106 says to rebuild the clip at every pin even when only data changed. The S6 builder brief explicitly authorizes reuse when inputs are unchanged, and the report records that exception. Align this maintenance text with the chosen conditional rule in a follow-up; the existing clip itself has sufficient input-identity evidence for S6.

## Scope results and evidence

| Audit item | Review result |
| --- | --- |
| 1: red-team coverage | Five design/admission red teams are represented in the paper's prose, abstract's coverage list, Table 1 and generated figure. The Rand source has exactly 20 held bullets and R1–R4, with no blocking break. The new data row is held 20, blocking 0, nonblocking 4. The two Rand implementation findings are supported by their Codex/Opus reports and finder re-checks. Searches of the other current launch texts found no competing total. |
| 2: rust#57893 attribution | Both site rewrites distinguish the red team's admitted known route from the fix review's second form, and explicitly say both were already known upstream. No Tide-discovery claim is introduced. |
| 5, 13: Docker | README adds the Linux-host-only qualification. The comparison guide separates that observed limit from Apple-silicon emulation. Both match docs/docker.md. |
| 6: registration | The site now qualifies fx-zu4f.5's local timestamp evidence and discloses the post-hoc pools of .13 and .15. |
| 7: benchmark abstract | The abstract retains cold-build costs and adds incremental-rebuild costs and the larger sampled-contract test-build cost. This agrees with research/36. |
| 8: fx-zu4f.5 counts | research/37 says all 1,507 in-date advisories were read and all 60 GitHub results were open. Table 2 and figure now use pool = screened = 1,567, while retaining the distinction that 87 advisories qualified and 12 had quoted code ported or assessed. The site's separate advisory/result stages are compatible with this combined total. |
| 9: diagnostics | **Incomplete; B1 above.** The changed shared copies and status glossary are correct. |
| 10: demo scope | Questions now promises replay and the trust diff, rather than every shipped capability. |
| 12: trust diff | Shared heading now says any change can get a trust diff, consistent with the on-request command. |
| 14: E1 token attribution | The comparison guide now reports the cached-input observation and reference-bearing prompt without assigning the entire difference to loading the reference. |
| 15: check latency | The walkthrough limits the under-one-second observation to the small benchmark programs and states the rebuild ratio, matching research/36. |
| 16: chart table | The comparison column now identifies cold build versus release run as well as configuration names. |

### Figure pipeline

`node paper/figures/build.mjs --check` exited 0: “figures match their records (22 files).” Thus the committed SVGs and data match regeneration from the candidate's specifications and records; no SVG-only edit is present.

The new optional held-list selector is a small extension of the existing table reader, used by the one new list-form record. It counts the current Rand section correctly and rejects unmatched bullet/table lines. The search-selector change reads the inventory count directly rather than confusing qualifying advisories with the inventory.

The unchanged final check compares every generated string with its committed file and detects missing/stale outputs (`build.mjs:550`–554). The existing in-memory falsifiers also run on every check. I inspected the builder's `tamper.sh` and `tamper.log`: baseline exits 0; rewording a held bullet, dropping a held bullet and rewording a residual-limit heading each exit 1. Those mutation runs are builder evidence, not new reviewer filesystem mutations. This pass leaves the frozen source untouched.

### Governance

A byte comparison confirms that all content before the responsibility lists and all content beginning with “Human-written production code” is unchanged. In particular, the entire Merge Rule is identical to the base. The changed responsibilities implement steering for humans and design, specification, writing, review and validation for agents.

README, CONTRIBUTING, the paper's disclosure and “How it was built,” and the site's disclosure, footer and llms.txt use AI-led, human-steered roles without adding human technical approval. The unchanged Merge Rule quotation is explicitly preserved, as instructed; its inherited alignment question is listed above.

### Site pin, media and privacy

- `src/site.ts`, `status.json`, `charts.json` and generated glance links consistently name Tide `c1b80537f4405911c65ef76502d7385a7c6eecf2`. I independently verified all 34 recorded input hashes against `git show` at that pin.
- `bun scripts/status-data.ts TIDE --check` exited 0 at the candidate pin. Chart-data has no native `--check` flag. I evaluated its existing generator in memory, replacing only its output write with an exact comparison; `charts.json` matches regeneration byte for byte, without a file write.
- The committed PDF's SHA-256 is `dca9e1b09f4c444b5e9abe04a85283736a4c7e1026cb34a08af9a7e988b3493f`, exactly the builder's `paper.log` output. Its metadata names Typst 0.15.0. Independent decoding confirms 22 pages and the strings “Five red teams,” “20 attacks held,” “4 limits,” “AI-led and human-steered” and “1,567.” The build script archives precisely tideRevision before compilation, supporting the rebuilt-at-pin account.
- Reviewer `typst compile --root . paper/tide-v0.typ /mnt/build/tide-agents/fx-qlek.7-target/s6c/tide-v0.pdf` exited 0 without warnings. This uses the installed Typst 0.15.1; I do not claim byte equality with the 0.15.0 site artifact.
- The clip reuse decision holds. `git diff e80f788 c1b8053` is empty for crates, conformance, demo, examples, flake files, Cargo files and the toolchain pin. The recorder recipe and derivation are unchanged. The builder's clip-inputs log gives identical driver/std store identities at both pins. The recipe copies the same admission case and demo fixtures and applies the same patch; the changed source-root store path is not filmed. The caption remains supported. No new recording was run.
- I read the five page sources and llms.txt against their linked Tide records, including the catch qualifiers and negatives. B1 is the remaining contradiction found. Source and existing built text contain no old pin or examined host/private markers. The changed Tide/site text and the decoded PDF introduce no private draft material, host paths or details of undisclosed finds. This is a candidate-content review, not a certification of the repositories' entire histories.

### Other checks and limits

Both candidate text diffs pass `git diff --check` (site excludes the binary PDF). The site's offline `bun test scripts` passes all 20 tests / 35 assertions under installed Bun 1.4.4. I inspected the builder's pinned-shell log: type/typo checks, build, status falsifier and all 30 browser tests passed under Bun 1.3.13. I did not rerun write-producing Astro/build/browser checks in this read-only pass, visually render the PDF, access the network, or run Rust, Nix, Docker or clip builds. Publication placeholders remain explicit and unchanged.

DONE
