Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate 3711841 (re-check)

## Scope and identity

Finder-only re-check of B1–B5 in `target/work/r1-codex.md`, read first, against the dispositions in `target/work/r1-fixes.md`, `git diff 28b1314 3711841`, the registered protocol, GOVERNANCE.md, the coordinator's recorded decisions and tide-desk's confirmation, and the working evidence named below. No full review was restarted. No new blocking problem introduced by these fixes was found.

HEAD is `37118412d42a4f7dc7713724c8d5c63301df0ed3`. The working record is byte-identical to that commit. Its SHA-256 is `cc1740aebf31672f388da2bf82326f460863b4e36021915d6e7d459ccfc31b5e`; the output of `git diff 28b1314 3711841` hashes to `e33a4332ecbd52f7fc5ad62d7175240c02748f599696173c31228cd68e9114bd`. Verification used local git, `br --no-db` reads and plain `python3 -I` on 2026-11-19 in this worktree. Only this report was written; no commit, build, admission run, fetch or external contact was performed.

## B1 — Resolved

The own-search restriction is withdrawn in research/47 §5 and D1. Both RUSTSEC-2026-0264 and -0265 now have **not caught (port incomplete)** in `screen.tsv` and the record's row table and conclusion. The record applies §3's attribution rule rather than restricting evidence to the receiving row's search.

S176's saved source names `proc-macro1-1.0.107/build.rs` and quotes `run_unix_payload` using an undefined `path_str`; the download helpers are unquoted. The saved -0264 advisory expressly attributes the same build script to proc-macro-en. `sources/search/RUSTSEC-2023-0113.txt` preserves where SafeDep was discovered. The sole change to `sources/index.tsv` versus its r0 copy adds the two advisories to S176's `advisories_served`; its discovery, fetch and hash fields are unchanged. The incomplete-code gaps remain disclosed, and neither row is represented as ported or run.

## B2 — Resolved

Research/47 §2's correction, D2, the row table and §9 agree: env-test-guard and lwd-uri-util stop at **no portable source** under §3 applied at §4 step 2; onering remains **port incomplete**. S651's saved code block ends in ci_preflight's opening declarations. S677 quotes preparatory local_files and the beginning of cred_probes, without a function containing the effect-carrying call under the recorded narrow reading. Both rows retain their missing-code and mapping observations in `gaps`.

`RULING-0175.md` preserves its original text byte-for-byte as a prefix and appends a dated correction expressly withdrawing item 3 and its no-widening claim. The correction distinguishes onering's quoted git/request calls with missing helper bodies from those two source-gate failures. It agrees with the record and the coordinator's accepted B2 disposition.

## B3 — Resolved

Research/47 D3 discloses S127 as a forbidden mirror fetch and withdraws the previous no-mirror claim. All six TSV references explicitly mark S127 fetched but excluded. S126 and S127's complete index entries are unchanged from r0, including HTTP status and timestamps; S127's saved body still matches `f8467ebec73134c6c08e160294d749f6be618a802ae0e6a378a967832408cb65`. History has not been rewritten into a non-fetch.

Independent index accounting gives **285 actual fetches** (259 HTTP 200, 26 HTTP 403), versus **284 admissible fetched sources** (258 HTTP 200, 26 HTTP 403) after excluding S127. Section 6 agrees. The affected rows no longer rely on S127: the two reclassified rows use S176 and the saved attribution; the other four remain no portable source with their original missing-source reasons. No affirmative blanket no-mirror claim remains; the prohibition is quoted as the rule that was broken.

## B4 — Resolved

The historical final.log reports TD0501 at 10:5, 39:27, 58:28 and 70:19. The first span is the top-level import, so D4, the TSV and the historical run table now count **three TD0501**, with the import explicitly non-counted and TD0901 separately counted at 71:5.

The revised-port table and TSV apply the same distinction to its own run: counted TD0501 at **32:27, 51:28, 63:19**, TD0901 at **64:5**; import TD0501 at **13:5** is non-counted. These spans match r1.log and the revised source. No rerun is claimed for the historical bookkeeping correction.

## B5 — Resolved

Research/47 §5 states all four desk release conditions and discloses D5. The old port and final-pass result are retained and explicitly marked non-conforming history, unused for the outcome. Its file still hashes to `393c7abc853d078984b243fc0601bc7291d81188d3228422ad5acc0741c6c44c`.

The revised `ports/RUSTSEC-2025-0147-r1/buggy.rs` hashes to `20bde8f8fc5888cae8aca28203eb757d005ba6b3a0ef462ddc84a9ef1651ce5e`. The diff removes the encoded literal, DecoderReader, buffer, io::copy and from_utf8 match/early return, removes the now-unused io import, and supplies `String::from("http://placeholder.invalid/<path>")` to the existing check call. Apart from this disclosed replacement and comments, every other quoted statement is unchanged. All three cfg-selected check bodies are byte-identical between ports. Their effect-carrying operations are unchanged. The record expressly acknowledges the changed data flow and the desk condition's precedence over port fidelity, as authorized by tide-lead and confirmed by tide-desk.

The separate r1 run is bound to this port hash before and after admission. `ports/r1.log` hashes to `58bc8c019ffb844dbcdbf343f3b36d32ccdaee7ca5f8f9499caa02aef680afe2`; the saved diagnostics hash to `0f291596bdf0c55f66404fbb74142794bea0087f3d8217fbbaf19ce022907158`. The log records R, toolchain, driver/std identities, heavy.sh commands, load, metadata-only admission and exit 1. `screen-r1.sh` differs from the original harness only in labels, log/output paths and run list. The counted spans establish leg 1. The retained Client with options/header, Runtime and temp_dir have no permitted mapping under §5, so §4 step 7 supports **not caught (capability absent)** without a capability-version run.

D5 reports the audit of every archive-derived port. Reading uniswap-utils, finch-rust and all four chrono_anchor files supports its conclusions: the first two have no endpoint or decoding; chrono_anchor's endpoints use fixed placeholder constants, with inert process/program arguments and no decoder or harvesting logic. The original evm-units exception is separately disclosed and replaced for the outcome. All four artifact paths in `ports/undeleted-artifacts.sha256` are now absent, consistent with the coordinator's reported deletion. The record retains the fixer's historical deletion request; this is not an outstanding condition failure.

## Counts — Resolved

An independent `python3 -I` CSV recount of `screen.tsv` gives **87 unique rows: 0 caught; 71 no portable source, of which 13 have an access failure; 13 port incomplete; 2 no TD code; 1 capability absent**. The two B1 rows and two B2 rows move in opposite directions, preserving the category totals while changing the access-failure subset from 14 to 13.

A field-by-field in-memory comparison confirms all 87 Markdown rows match all 14 displayed TSV fields, including outcome/access-failure annotations and the corrected evm-units diagnostics. The record's summary, counts, conclusion and commit message agree. The fix introduces no claimed catch.

## Verdict

**Approve**, with high confidence within this finder-only scope. B1–B5 are resolved; no blocking finding remains from the original review or was introduced by these fixes.

Not checked: no new searches, fetches, builds, admission reruns, external timestamp validation or renewed full-corpus review. The saved admission evidence supports this documented negative; zero catches is not evidence of malware prevention.
DONE
