Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 3711841 (research record)

## Verdict

**Changes required.** Two blocking findings, both fixable by editing text in research/47 (no rerun, no re-determination, no count change). No judgment hides or manufactures a catch. Safety holds for everything committed. The 0/71/13/2/1 result follows the registered protocol as the rulings and the desk's B5 decision read it.

## Scope

- Candidate: worktree /mnt/build/eon-agents/worktrees/tide-fx-zu4f.8, branch fx-zu4f.8, HEAD `37118412d42a4f7dc7713724c8d5c63301df0ed3`; the record commit adds only `research/47-malware-rescreen.md`. The tracked tree is clean. One untracked file is present (`research/protocols/fx-zu4f.8-inputs.sha256.ots.bak`, see N7).
- Read: AGENTS.md, GOVERNANCE.md, `br --no-db show/comments fx-zu4f.8`, the protocol, RULING-0039 and RULING-0175 (with its correction), r1-codex.md, the record in full (§1–§9; §10 checked byte for byte), screen.tsv, sources/index.tsv, images.tsv, the search records, the port files, final.log, r1.log, and the build-target outputs.
- Mechanical checks (python3 -I, git, grep; no build, no network, no execution):
  - §10 is byte-identical to the protocol (SHA-256 4e2016df…7e26).
  - All 87 table rows match screen.tsv field by field. The only differences are comma spacing and the "(access failures: N)" suffix. POOL order is checked against pool.txt.
  - Outcome counts recomputed: 0 / 71 / 13 / 2 / 1. 13 of the 71 have a 403.
  - Source counts: 690 index rows = 401 rejected + 285 fetched (259 × 200, 26 × 403) + 4 archive copies; 87 search files, all with the registered query string and standard mode; 4 images.
  - Every port hash in §8 matches its file. final.log, r1.log, screen.sh and screen-r1.sh match their recorded hashes.
  - At R: `e0b0d62` and `8ac1317` are ancestors of R. The reference §6.1 rows for `Proc` and `Net` read as quoted. rules.md keeps ambient `Command::new` at TD0501. `git diff --quiet R 3711841 -- crates Cargo.toml Cargo.lock rust-toolchain.toml` holds.
- Judgment sampling, beyond Codex's rows, with the sources read:
  - S176 (proc-macro1/-en) code blocks, read with string literals masked;
  - S575 and S642 (proc-macro1 fragments);
  - S651 and S677;
  - the S194 image (a Wiz prevalence bar chart, as recorded);
  - the evm-units r1 port and its diagnostics;
  - the amaperf port;
  - the std Command, Stdio and fs APIs at R.
- Spread of the 71 no-portable-source rows: I scanned all 218 HTTP-200 pages they cite, as data. The scan looked for Rust API markers and `fn name(` in `<pre>`/`<code>` blocks and in whitespace-collapsed text, which catches token-split highlighted code. Every hit is a source the index already marks as quoting, or a non-Rust false positive (Wiz cookie scripts, KQL). No missed quoting source.
- Not done: Bitcoin anchoring and live pages, which need network; reading all 285 pages in full.

## Blocking findings

### B1. §1 says no row is one API away from a catch, but the record's own evidence shows chrono_anchor is

- **Where:** research/47-malware-rescreen.md:49–50 ("Every row's gaps are listed in section 7, so no row reads as one API away from a catch."). It is contradicted by :45–47, :564, :596–602 ("The registered table form fails only because R's `TaskError` has no `From<Cancelled>`") and by screen.tsv row RUSTSEC-2026-0039.
- **Evidence:**
  - chrono_anchor passes leg 1 (TD0602 ×2, TD0501 ×3).
  - Its capability-passing version has exactly one diagnostic: E0277 at 34:44, on the table form `clock.sleep(d)?`. Its diff passed §6's line check.
  - The disclosed `.map_err` and `let _` variants both exit 0 with the same interface.json.
  - At R, std has `From<Cancelled> for io::Error` (crates/tide-std/src/task.rs:1878) but none for `TaskError`.
  - So, given that one std conversion, this port would very likely meet the registered caught rule. The detached-spawn difference is a mapping the protocol accepts, not a blocker.
- **Why it blocks:** The protocol's rule (§4, §9) requires listing every gap, so that no write-up overstates how close a row came. The record turns that rule into a factual claim, and for one row the claim is false. A skeptical reader who reaches §8 sees the results section contradict the evidence. The error is an overstated negative: the nearest miss looks farther away than it is.
- **Minimum fix (text only):** Replace :49–50 with a statement of what the gaps show. For example: "Every row's gaps are listed in section 7. The nearest miss is chrono_anchor: leg 1 passes, and leg 2 fails only on E0277, because `TaskError` has no `From<Cancelled>` at R. A run at a revision with that conversion would be a new registered test, not this one." Optionally name proc-macro1/-en's single determining gap (N1) in the same place.

### B2. RULING-0039 item 6 is not carried out: the record does not list the rows where a table form brings in a `Result`

- **Where:** RULING-0039 item 6: "Apply the same reading to every other row where a table form returns a `Result` the source did not … and list those rows in the record." The record has no such list (§2 :81–84, §8 :464–474 and :526–530). Neither r1-fixes.md nor report.md addresses item 6.
- **Evidence:**
  - Two rows reached a capability-passing version, and both add a table-required `?` that the source did not have:
    - chrono_anchor's `clock.sleep(d)?`, which gives E0277;
    - amaperf's `Url::parse(..)?` ×2, at :494 and :497, which converts into `Box<dyn Error>` and raises no error.
  - No other row has a capability version: evm-units ends at step 7, and the rest end at steps 2, 4 or 6.
- **Why it blocks:** The contract for this round includes the binding rulings. This item exists so readers can check that the strict reading that sank chrono_anchor was applied evenly and not to one row only. Without the list, the record cannot show that.
- **Minimum fix (text only):** Add one sentence after the RULING-0039 bullet in §2 or in §8's diff check, for example: "Rows where a table form brings in a `?` the source lacked (RULING-0039 item 6): chrono_anchor (`clock.sleep(d)?`, E0277) and amaperf (`Url::parse(..)?` ×2, which converts into `Box<dyn Error>` with no error). No other row reached a capability-passing version."

## Non-blocking findings

- **N1. proc-macro1/-en: the determining gap is stated loosely.**
  - Where: :168–169, :243–245, :628–630, and screen.tsv rows -0264/-0265.
  - S176 quotes `run_unix_payload` whole, from signature to closing brace. Its `path_str` is used but never defined; nothing marks it as cut. That one missing definition, needed to type-check, is the §5 reason the row is port incomplete. The verdict is right, and it matches sha-rust's `filename`.
  - The unquoted `download_bytes`/`src_download_url` are not a step-4 reason. Under §5 they would never be ported, and `run_unix_payload` does not call them. `main` contains no in-class operation (`env::consts::OS` is a constant), so it is not an affected function either.
  - Std at R has everything else the function uses: `fs.write`, `proc.command`, `.args`, `.stdin/stdout/stderr`, `Stdio::null`, `.status`, `.spawn`. `end_url()` may still be a problem, depending on how scaffolding supplies a body.
  - Suggest: "uses `path_str`, which the quote never defines", named as the determining gap. Keep the helpers and the TLS client as context gaps, and drop `main` from the affected functions.
- **N2. amaperf: say where leg 1 fired.** Its three counted TD0501 sit on the `OUT_DIR` read and the run-once mutex file, both preparation. No TD code fires on the exfiltrating `reqwest` calls: they are third-party, so rustc gives only E0433. Under RULING-0175 this counts, because it is one function, and the record states that rule (:93–95). §1 :39–47 and §9 should also say so plainly, so that "passed leg 1" is not read as "admission flagged the exfiltration". The same contrast with the onering probe is already drawn at :607–609.
- **N3. chrono_anchor's label.** "Port incomplete" comes from §4 step 8 (rustc errors remain). The error, though, sits on a mapped authority form, which §6's own port-incomplete clause ("non-authority code … third-party crate, missing language feature or fragmentary quote") does not describe. Say once, in §1 or §9, that the label is step 8's and does not mean the quote was incomplete. RULING-0039 item 4 asked for "which category and why".
- **N4. D5 wording (:286–290).**
  - The revision also removes a control path: the `from_utf8` match's early return, which §5 lists separately.
  - Add that the removed statements contain no in-class operation: an in-memory decoder, a buffer and `io::copy`.
  - Add that the first port's run gives the same leg-1 codes and the same step-7 gaps, so the outcome does not depend on the revision. That supports "the screen is complete".
- **N5. Readability for a cold reader.**
  - Define these at first use, in §1: "leg 1/leg 2" (first defined at :617), "in-class", R, WORK, and the TD codes cited (TD0501, TD0602, TD0901, TD0601, TD0702, TD0902, TD0915, TD0101), one line each.
  - The rustdecimal `stated_effect` cell (:339) carries raw `#` heading markup and a dangling reference link `[were notified][1]`.
  - :330 calls the 26 HTTP-403 pages "admissible fetched sources". A 403 page is an access failure, not a source.
  - :329 lists tokio `Runtime` among capability-absent's "missing operations", but §4 step 7 covers authority operations only. `Client` and `temp_dir` decide the row.
- **N6. Minimum identifiers.** Rows -0264/-0265 (:411–412) print a program literal from the quote (`Command::new("chmod")`). The ports' own rule turns such literals into `"<program>"`, and the record needs only "makes it executable". This is public and harmless, but more than the minimum.
- **N7. Housekeeping.**
  - The untracked `research/protocols/fx-zu4f.8-inputs.sha256.ots.bak` must not land.
  - Under TARGET/screen-out, the onering probe's `libcapability.rmeta` and the chrono_anchor observation `interface.json` files remain. Desk limit 2 does not cover the onering probe, which is not archive-derived, and the interface files are cited evidence. Delete them with the target at retirement.
- **N8. Operational detail in WORK, not committed.** The amaperf port (`ports/RUSTSEC-2023-0097/*.rs`) reproduces the message assembly (host-info fields formatted into a bot `sendMessage` query) against `.invalid` hosts and a `"<redacted>"` token. It is not archive-derived, so the desk's limit 3 does not bind it. The bead's safety terms hold: nothing is committed. Note it for WORK cleanup only.

## Reasoning

**Judgments.**

- The 13 port-incomplete rows each rest on a §5/§6 rule:
  - amaperf ×7: serde_json, rustc error after the allowed fixes;
  - chrono_anchor: E0277 (see N3);
  - rustdecimal, sha-rust and proc-macro1/-en: a cut or undefined local needed to type-check. sha-rust and rustdecimal also lack table rows for their HTTP clients, so neither is close.
  - onering: its effect-carrying calls are quoted only as calls to unquoted helpers, and a POST has no table row anyway.
- The two no-TD-code rows quote only a call into the malicious dependency, so leg 1 cannot fire. Calling them out of class instead would also not be a catch.
- evm-units r1 passes leg 1. Step 7 is decided by `Client` (a POST/headers/TLS client, "never in this test") and `env::temp_dir` ("never in this test"). The r1 diagnostics fit: E0599 on ambient `File::create`/`Command::new` and on `!`-typed chains after the unresolved third-party types, E0432 on the `temp_dir` import, E0433 on `Runtime` and `Client`.
- lwd-uri-util and env-test-guard at no portable source follow the corrected RULING-0175. Even under a broader reading, lwd-uri-util's `Path::is_file` has no table row, so the correction hides no catch.
- No row was moved toward or away from a catch by a stretched port or a widened reading. The onering and chrono_anchor variants stay disclosed and uncounted.
- UDP-only and resolve-only are correctly empty. sha-rust's UDP sits beside a cut `filename` and an unmapped POST. No quoted affected function makes a DNS lookup.

**Safety.**

- The committed file contains only advisory-db links (89), five `placeholder.invalid` strings, and the RFC 5737 ranges inside the protocol text.
- It has no encoded strings and no live hosts. `timeapi.io` and `vercel.app` appear only inside quoted advisory text.
- Its diff hunks are changed lines only, with placeholder literals.
- Every port used for an outcome has `.invalid` endpoints or none. The archive-derived ones (uniswap-utils, finch-rust, chrono_anchor, evm-units r1) meet limits 1, 3 and 4. Limit 2's four chrono_anchor `.rmeta` files were deleted at 04:30Z, the text fix tide-lead already knows about.
- Every run is metadata-only admission.

**Honesty.**

- The negative comes first.
- D1–D5 and both rulings are disclosed plainly. RULING-0175's first text is kept beside its correction.
- "Nothing known either way" is stated for the 71 rows, and the counts are labelled as not a rate.
- No claim that Tide stops malware: §9's "would not have shown" list holds.
- The two blocking findings are the remaining honesty gaps. One overstates a negative (B1). The other leaves out a binding ruling's evenness list (B2).
- Both fixes change wording or add facts the evidence already holds. Neither changes a decision, an outcome or a count, so the finder's re-check is light.

DONE
