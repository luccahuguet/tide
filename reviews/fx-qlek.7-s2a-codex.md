Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate 35d22ee (docs, S2a)

## Verdict

**changes required**. The direction and the central results are supported. Six bounded corrections are needed: search-denominator descriptions, visible repository-link placeholders, missing availability limits, first-use definitions, the opening description of parallel regions, and the replay-manifest caption. Confidence: high for these findings; this is a source and document review, not new runtime proof.

## Blocking findings

### B1 — Search counts need their actual populations and outcome labels

**Where:** paper/tide-v0.typ:338, :341, :354.

**Evidence:**

- Table 2 calls the fx-zu4f.6 pool “160 GitHub results, Rust, at least 1,000 stars.” research/38-external-concurrency-search.md:7–14 and :45–55 record 160 raw results, 158 unique issues, and 126 unique issues rejected by the language/star filters. Only the 32 screened issues passed those filters. The table attaches the filtered population's properties to the raw denominator.
- The fx-zu4f.10 row calls 5,076 entries “fix rows.” research/44-fx-zu4f.10-concurrency-round.md:105–115 calls them keyword-matching crate-membership rows, covering 4,524 distinct repository commits; only 306 pass E1, the actual non-test bug-fix criterion. The record does not establish 5,076 fixes.
- The paragraph says “most candidates failed on portability,” citing research/38, /39 and /41. Those records have 32 + 119 + 100 = 251 screened rows, of which 15 + 26 + 68 = 109 fail portability (E4). That is the largest first-failure category, but not most screened rows. All candidates that reached E4 failed it; that is the narrower supported statement.

**Minimum fix:** Label .6 “160 raw results (158 unique), before language/star filters”; label .10 “5,076 keyword-match rows” and retain the unresolved row; replace “most candidates failed on portability” with “every candidate that reached the portability check failed it.” The other table totals and the caught counts can stay.

### B2 — Default repository links are broken links without a visible placeholder notice

**Where:** paper/tide-v0.typ:30–37 and :143.

**Evidence:** The helper always creates clickable GitHub URLs, including with defaults `https://github.com/OWNER/tide` and `RC-TAG`. Its visible label contains only the cited path. The compiled PDF contains those placeholder destinations in its link annotations, but its extracted visible text contains neither `OWNER` nor `RC-TAG`; the Status paragraph instead says links point at the release tag. Thus a reader sees normal evidence links that cannot resolve, not visibly unfinished placeholders. This is separate from the figure placeholders, which visibly say “Figure to come.” All 50 distinct repository destinations actually cited exist in candidate Git, including the computed benchmark-record path and the directory link.

**Minimum fix:** While either input is a placeholder, visibly label the repository links as placeholders or render their labels without hyperlinks, and make Status explicitly say the repository/tag are pending. Keep ordinary pinned links when real inputs are supplied. Real release URLs are not required to finish S2a.

### B3 — The self-contained availability account omits application randomness and the remaining network/process operations

**Where:** paper/tide-v0.typ:175 and :506–515.

**Evidence:** The capability account discusses randomness as capability-mediated; the implementation/limitations account says only that *simulated* randomness is planned (:508). It never tells the reader that application randomness draws are unavailable on the real backend too. `Rand` has no draw methods at this revision. README.md:372–374 and design/reference.md:338–343 explicitly identify UDP, `Rand` draws and `Proc::exit` as planned, while `Proc::command`, TCP, `get` and `resolve` are built. UDP and `Proc::exit` are absent from the paper's availability account altogether. The real-versus-simulated distinction is material to deciding whether v0 can express a program; a link to the owner guide does not make the paper self-contained on it.

**Minimum fix:** Add one compact scope sentence near the capability inventory or limitations: UDP, application randomness draws and `Proc::exit` remain planned; `Rand` currently exists only as a capability identity, with no draw methods. Keep `command`, `get` and `resolve` as implemented. Do not place these operations in a later release tier without its owning decision.

### B4 — The terms artifact reports definitions that the paper does not supply

**Where:** paper/tide-v0.typ:137, :139, :175; reviews/fx-qlek.7-paper-terms.md:12 and its Protocol/Capability rows.

**Evidence:** The binding 2026-11-18 19:51 comment requires plain definitions at first appearance, including abstract and captions. Concrete misses: `AI` appears in the abstract without “artificial intelligence”; `HTTP` first appears in “hyper, an HTTP library” without an explanation; §2.2 adds `TCP` and `TLS` without explaining either. “Its written protocol” does not define a protocol, although the terms artifact supplies the parenthetical “the trial's plan, fixed in advance,” which is not in the paper. Likewise the abstract uses `std` without explicitly identifying it as the standard-library abbreviation. These are verification gaps in the claimed completed terms pass, rather than a request for a Rust tutorial.

**Minimum fix:** Give short plain definitions at these first occurrences (including `std` and protocol), add rows for TCP/TLS and any other missing acronyms, and make the artifact quote what is actually present. Retain MISSING markers until the corresponding paper text is fixed.

### B5 — Parallel regions are not the only code that runs in parallel outside simulated scheduling

**Where:** paper/tide-v0.typ:183, first sentence.

**Evidence:** “Parallel regions are the one place where admitted code runs in parallel without the simulator choosing the order” is broader than D9. Ordinary structured tasks on the real thread runtime also execute on OS threads without the simulator controlling their order: design/03-authority-concurrency.md §7.3, crates/tide-std/src/task.rs:1372–1375 and :1521–1529. The distinction in research/45 §3 is schedule-independent successful results for the Pure fragment versus replay of a particular effectful execution, not an exclusive right to physical parallel execution.

**Minimum fix:** Replace that opening with “D9's parallel regions give schedule-independent successful results without exploring task schedules.” Preserve the rest of the paragraph. Its sequential-equivalence contract, panic qualification, trust-hole qualification, explicit non-theorem statement, partial runtime and sequential simulator execution accurately track D9, reference §8 and research/45.

### B6 — The manifest caption promises refusal before execution for checks made during execution

**Where:** paper/tide-v0.typ:252.

**Evidence:** After listing seed, source hashes, build identity and injected inputs, the caption says “Replay refuses to run when any of them differs.” REPLAY-2 (design/03-authority-concurrency.md:2385–2389) promises pre-execution refusal for source/build identity. docs/driver.md:411–427 distinguishes those checks from consuming recorded choices and checking root setup, trace and outcome during replay. In crates/tide-std/src/sim/manifest.rs:205–222, the root/input comparison is in `finish`, after evaluation. This caption extends the pre-execution guarantee to injected inputs; it also suggests an independent immutable-seed check that the manifest does not establish. The caption is final S2a text even though its figure is deferred.

**Minimum fix:** Say “Replay checks source and build identity before running, uses the recorded seed and choices, and fails if the run's inputs, trace or outcome differ from the recording.” This states the actual validation boundaries without adding an integrity guarantee.

## Non-blocking findings

1. **Red-team provenance should distinguish recorded facts from unavailable ones** (paper/tide-v0.typ:271–300). Models, design-only/read-only scope, attack surfaces and the admission controls are described accurately. The required independence-from-author and goal-only briefing details are not established by the linked reports or supplied in the paper. The builder explicitly acknowledges this in its S2a account. Add a short “independence/blinding not established by these records” qualification unless a committed brief actually establishes them; do not invent either property. The admission red team is correctly disclosed as also writing fixes.
2. **Tone still repeats some limits and opens several sentences with negatives** (for example paper/tide-v0.typ:137, :201, :300, :411, :539). The main result-led structure is much improved: the catch precedes the case studies, and the negative search results remain visible. A later editorial pass can consolidate repeated scope statements and rephrase negative openings, preserving the mandated E3 quotation and the catch's required limitations. This is non-blocking style, not a reason to hide a negative.

## Reasoning

Reviewed frozen HEAD `35d22ee186eb4e0bfe65f0f55fa003bbf6137eb5`, parent `90de8d5`, with initially clean tracked status. SHA-256 of `git diff 90de8d5 35d22ee -- paper/tide-v0.typ reviews/fx-qlek.7-paper-terms.md`: `e6fff5f990282cfd5977eed32c4383d313786eb817a212e461ea8523f50d9d22`. Read AGENTS.md, GOVERNANCE.md, the bead and comments through `br --no-db`, plan rows 2.1–2.10 and the builder's S2a account. Checked the paper in reading order against the local source records at this revision, including README/at-a-glance, the relevant decision/reference/chapter contracts, research/36–49, the linked red teams and implementation reviews/re-checks, E3, E1 and the demo/case-study records.

Specific supported results:

- Table 1: admission 48 routes including controls, two forms fixed and two intentional admitted routes; Proc 27 held attack rows, nine first-round findings (three blocking), two additional blocking findings in re-check, then three non-blocking text items; HTTP 12 held classes, four findings (one blocking), all addressed; resolve 15 held classes and two non-blocking limits. Models and rounds match the reports.
- Implementation-review totals are Proc 7 + 3 + 2 = 12, HTTP 8 + 3 + 1 = 12, resolve 3. Final finder dispositions support their resolution. These counts include documentation and missing-proof findings, not 27 independently demonstrated exploits; the prose describes that distinction. The GCONV_PATH equals-key bypass matches Proc s2-r1 B1.
- Search totals otherwise match their records: .15 one selected row/one catch; .8 outcomes 71 + 13 + 2 + 1 = 87 and zero catches; .5 87 qualifying advisories plus 60 GitHub results and zero catches; .6 32 screened/zero eligible; .7 119/zero; .9 100/zero; .10 5,076 rows plus ten core screens, zero eligible with B23.128 unresolved; .11 42 candidates plus seven cores, zero eligible. The registration column retains the early self-attestation/late-commit disclosures and the later OTS existence-only qualification.
- The abstract and full catch account keep the std gap, screen, fix and fresh registration; qualified chronology; admission/type-checking only with no port executed; the prior 0 of 87; predictable post-screen pool; and the conversion's removal/reversal. No first-time discovery or catch-rate claim is made. research/49 is linked. “Unforgeable” appears only in the explicitly conditional paragraph, not as the launch claim; “race-free” is absent; trust-diff catches are measured E3 outcomes, not a claimed benefit. Rust is explicitly complementary, with no fork/replacement claim.
- E3's block quotation is byte-for-byte the report's published opening paragraph. Counts, interval, ceiling, false positives and the declared-authority limit match that report. E1's counts and uncertainty, benchmark ratios and unstable-run scope, and the demo's failing seeds/40-event replay agree with their records. The discovery case alone is credited with seed discovery over simulated Fs. The Solana paragraph preserves the research-only, conditional two-of-five result and existing Anchor/SDK checks.
- The chosen title and opening slogan/mechanism are present. Every major numbered section has a visual or an explicitly deferred visual; producing those figures remains S2b.

Document check: `typst compile --root . paper/tide-v0.typ /mnt/build/tide-agents/fx-qlek.7-target/tide-v0-s2a-codex.pdf` exited 0, with no diagnostics; 18 pages. Inspected extracted PDF text and link annotations: all 12 figure boxes visibly say “Figure to come”; both tables render as text; all cited repository paths exist in Git. `git diff 90de8d5 35d22ee --check` passed. No Rust/Nix build, test suite, external access, contact, commit or Beads write was performed. Only this report and the expressly permitted PDF were written.

Not checked: S2b figure generation and visual polish, live release URLs, new runtime/security proof, or independent verification of self-attested chronology. Those remain outside this pass; the six text findings above block approval of this candidate.

DONE
