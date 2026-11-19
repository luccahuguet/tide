Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate 28b1314 (research record)

## Verdict

**Changes required.** The recorded totals and run results reconcile, but the record does not establish the claimed fidelity to the registered method. Five blocking findings follow. None establishes a catch or supports a claim that Tide would have stopped malware.

## Scope (rows sampled)

Read-only review of candidate 28b1314a9232d1fd5dcc6573c04ec50db2328a15, its sole added file, the registered protocol, GOVERNANCE.md, bead body/comments, WORK/report.md, both REQUEST/RULING pairs, source index/search records, saved pages/images, ports/diffs/fix history, harnesses, and development/final logs. No build, admission run, fetch, external contact, or commit was performed.

All 87 rows received structural checks: identity/order against WORK/pool.txt, unique IDs, outcome/count reconciliation, access-failure attribution, TSV-to-Markdown field comparison, and review of their stated reasons/gaps. These checks do not substitute for independently reading every page.

Source-level sampling covered 24 rows:

- All 13 labelled port incomplete: RUSTSEC-2022-0042; RUSTSEC-2023-0097 through -0103 (one shared Veracode quote/port); RUSTSEC-2025-0146; RUSTSEC-2026-0039, -0175, -0469, -0483.
- Both no-TD-code rows: RUSTSEC-2025-0145 and -0148.
- The capability-absent row: RUSTSEC-2025-0147.
- Eight no-portable-source rows: RUSTSEC-2023-0110; RUSTSEC-2026-0036, -0084, -0102, -0155, -0259, -0264, -0265. Checked selected saved sources and discovery records for these, including Socket archives, S176, S518, S544, S575 and S642. The Sonatype rustdecimal screenshots were read visually without decoding their encoded strings.

Both counted capability diffs were checked against the actual port pairs and §6, line by line. All seven final admission results were reconciled with diagnostics. The two chrono_anchor observation interfaces were read and hashed; the onering observation and its warning were checked in dev.log.

## Blocking findings

### B1. The record adds an unregistered restriction on sharing discovered sources

**Location:** research/47-malware-rescreen.md:137; corresponding rows at :286 and :287; WORK/report.md, “Session 3 finish”, decision 5.

**Evidence:** The record excludes SafeDep S176 from proc-macro1 and proc-macro-en solely because another advisory's search found it. The registered §3 fixes discovery searches, but does not state that a discovered quote must also appear in the receiving advisory's own search. It expressly discusses a quote serving another advisory, conditioned on evidence that the crate shipped the same code (protocol :144–152). S176 explicitly identifies the quoted proc-macro1 build.rs; the saved RUSTSEC-2026-0264 advisory explicitly says proc-macro-en contained the same build script. WORK/assess.md:14–18 originally identifies this source relationship. The finish decision adds an eligibility restriction after the evidence was read and moves these two rows from port incomplete to no portable source. That is a consequential narrowing, not merely a description of the registered searches.

**Fix:** Remove the added own-search eligibility rule. Evaluate the already discovered quotes under the registered attribution rule, preserving where each was actually found. In particular, account for S176's proc-macro1 quote and the same-script attribution for proc-macro-en; retain the incomplete-code gaps rather than calling known quoted affected code unavailable. Reconcile affected rows, counts and source attribution together. No additional search or fetch is needed. If retaining the extra restriction, disclose it as a post-registration deviation instead of claiming it was registered.

### B2. RULING-0175 changes the source gate for two rows

**Location:** research/47-malware-rescreen.md:82, :296, :300; WORK/RULING-0175.md:7 (item 3).

**Evidence:** Registered §3 defines a qualifying quote as the crate's Rust source for a function containing a call carrying the malicious effect (protocol :144–157); §4 step 2 stops a row without that source before the paper-port step. The lwd-uri-util row expressly says there is no quoted function performing the effect. Under RULING-0175's own narrow reading, local_files only prepares paths; cred_probes is cut before any effect-carrying call. S677 confirms that description. Likewise, S651 reproduces only ci_preflight's opening declarations, not its environment reads or process calls. WORK/assess.md:35–37 correctly identifies that absence before the ruling is extended. Nevertheless these rows reach step 4 because the ruling treats any quoted Rust as a “quoting source.” That drops the registered affected-code requirement from step 2. This finding does not reject the exclusion of onering's preparatory environment read: its other excerpts actually quote the git/request calls that carry the effect, with missing helper bodies explaining its paper-port failure.

**Fix:** Apply §3's affected-code definition before §4 step 4. On the recorded narrow reading and present quotes, env-test-guard and lwd-uri-util stop at no portable source; retain the known missing-code and mapping observations as disclosed gaps. Correct the ruling's claim of unchanged interpretation and the record/TSV/counts accordingly, or explicitly disclose a deviation. Do not widen “affected function” to manufacture a catch.

### B3. A mirror was fetched despite the registered prohibition and the contrary disclosure

**Location:** research/47-malware-rescreen.md:151–153; WORK/sources/index.tsv:127–128.

**Evidence:** S126 is the www.socket.dev “popular-rust-crates-compromised” page, fetched at 03:37:33Z with HTTP 403. S127 is the same article through badge.socket.dev, fetched at 03:37:35Z with HTTP 200. Its saved HTML identifies https://socket.dev/blog/popular-rust-crates-compromised as canonical. WORK/report.md's search notes already describe badge.socket.dev as a mirror. Registered §3 (protocol :135–142) forbids any archive/cache/mirror route except the four approved copies; no approval for S127 is present in the bead comments. Finding the mirror in the original search results does not remove that prohibition. The claim that no mirror was used is therefore false.

**Fix:** Disclose this fetch as a protocol deviation and exclude S127 from admissible source evidence. Preserve the historical fetch/status/hash; do not relabel it as never fetched. Separate actual fetch totals from allowed-source totals and establish the affected row outcomes from allowed evidence. Remove the blanket no-mirror claim. A past fetch cannot be repaired by retroactive approval or by rewriting its log.

### B4. An import diagnostic is incorrectly counted as an affected-function operation

**Location:** research/47-malware-rescreen.md:245 and :432; WORK/screen.tsv:33.

**Evidence:** The record counts evm-units TD0501 four times, including 10:5. WORK/ports/RUSTSEC-2025-0147/buggy.rs:10 is a top-level use declaration. final.log:148 confirms that diagnostic's primary span; the three operation spans are 39:27, 58:28 and 70:19 (final.log:150–152). Registered §4 permits a code to count only when its primary span is on an in-class operation of an affected function (protocol :199–208). The harness summary counts all diagnostics, which is appropriate for a raw error inventory but not the counted subset.

**Fix:** Report three counted TD0501 occurrences, put the import occurrence among other/non-counted diagnostics, and keep TD0901 separately. Correct the TSV and both record tables. The row still passes leg 1; no rerun is needed for this bookkeeping correction.

### B5. evm-units does not meet the disclosed hard-coded endpoint condition

**Location:** research/47-malware-rescreen.md:177–188 and :329; WORK/ports/RUSTSEC-2025-0147/buggy.rs:21–33, :46, :91, :137.

**Evidence:** The pre-go desk release recorded on the bead at 2026-11-18 14:04 requires every archive-derived endpoint to be a hard-coded RFC 5737/6761 placeholder, never derived from archive text, and forbids reproducing obfuscation/data encoding or packing. The evm-units port retains the DecoderReader -> buffer -> decoded -> check data path, and every HTTP call receives chsum. There is no .invalid or RFC 5737 endpoint in this port. Replacing the encoded literal with “<redacted>” removes the live indicator, but does not make the endpoint a hard-coded RFC placeholder or remove that decoder path. The record omits these release conditions and states that every endpoint is .invalid. The saved source and port make the mismatch directly inspectable. No execution or live-endpoint contact is alleged.

**Fix:** Disclose the actual conditional release and this deviation; do not claim that the existing evm-units run satisfied it. Resolve the conflict between that condition and preserving the quoted data flow through the coordinator, then use the protocol's safety/outcome rules honestly. Preserve the existing port and log as historical evidence; do not silently replace the decoder-derived endpoint or reuse its old hash/run as proof for a revised port. Any permitted revised port needs its own appropriately identified evidence.

## Non-blocking findings

None added. The limitations below concern review coverage, not requests to expand this bead.

## Reasoning

The accounting is internally consistent: plain Python recounts 87 unique rows in pool order, with 71 no portable source, 13 port incomplete, 2 no TD code, and 1 capability absent; 14 of the 71 involve a recorded 403. The 87-row Markdown table matches the TSV fields. The two source-gate findings can move rows in opposite directions, so these matching totals alone cannot validate individual classifications. Recount after resolving the findings rather than assuming any particular corrected total.

All saved source-file hashes listed in index.tsv match their files. The eight copied archive-file hashes occur in the registered input manifest. The registered protocol hashes to 4e2016df…7e26 and its embedded copy is byte-for-byte identical. Git confirms the build inputs match R and that the Proc and Net::get landings are ancestors of R. Registration history and bead comments place the protocol before go; this review did not independently verify Bitcoin anchoring against the network.

The saved port hashes, capability-diff hashes and final-log hash match the committed record. The actual changed lines match the saved diffs and published hunks. RULING-0039 follows the referenced companion mapping at R, which specifies clock.sleep(d)?, and §6's closed list permits map_err only to preserve an existing question-mark operation. Its corrected E0277 outcome and the two excluded successful variants are reported accurately. Threading capability parameters through quoted callers follows §5. The amaperf counterpart retains serde_json rather than inventing a replacement. The missing filename in sha-rust and the truncated rustdecimal function are visible in the saved sources; refusing to invent their data flow is justified.

The seven final runs use metadata-only library admission, fresh per-run output paths and heavy.sh; the log records load checks, revision/environment, commands, exits and diagnostics. All seven exit 1. Neither counted capability version admits cleanly. The three development observations remain outside the counted pass, and the two chrono_anchor interfaces match the disclosed hash and authority. Existing final diagnostics substantiate the reported results, subject to B4's distinction between raw and counted TD codes.

No new port body or working payload is committed by candidate 28b1314. The record's changed-line hunks use placeholders, and the inspected ports have no test attributes. Logs show admission commands rather than payload execution. Missing source is explicitly described as nothing known either way; no claim that Tide stopped this malware is made. The UDP-only and resolve-only negatives are supported by the inspected quoted functions, with sha-rust also missing its POST mapping and local definition. These strengths support retaining the defensive research record after correction, not approving its current fidelity assertions.

Not checked: no builds, reruns, new searches/fetches, external timestamp validation, or independent full-text examination of all 285 fetched pages. Risk: current protocol-compliance and category claims remain unsupported until B1–B5 are resolved; zero catches is not a malware-prevention result.
DONE
