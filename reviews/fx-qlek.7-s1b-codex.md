Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate 8b1b2ea (docs, S1b)

## Verdict

Changes required. The direction and audience advice are sound, but four capability/cost qualifications and the required first-use definitions need correction. Confidence: high on the source comparisons; this is a document review, not a new measurement of the implementation.

## Blocking findings

### B1 — `Net::only` does not confine all network authority to listed hosts

- **Location:** docs/tide-or-rust.md:26.
- **Evidence:** The page says `only([...])` keeps a network capability “to listed hosts”. design/reference.md:506–510 explicitly leaves bind-name resolution and unlisted inbound peers open; even `only([])` is not network silence. docs/at-a-glance.md:26–28 preserves this qualification. A reader following this page could delegate `net.only(["db:5432"])` believing it prevents listening or communicating with unlisted inbound peers.
- **Minimum fix:** Say that `only` restricts outbound destinations, while listening and unlisted inbound peers remain possible until `no_bind()` is applied. Keep the existing process-authority qualification.

### B2 — The ideal-user example implies that signatures reveal directory narrowing

- **Location:** docs/tide-or-rust.md:134–136.
- **Evidence:** The page offers knowing, without reading bodies, that the uploader's `Fs` is narrowed to one directory. docs/owner-guide/01-capabilities.md:72–75 explicitly distinguishes the kind of authority visible in a signature from its actual bounds: `&Fs` looks identical for the whole file system and one read-only directory. design/reference.md:353–354 likewise says attenuation returns the same type. Changing the caller's directory restriction leaves the uploader's signature unchanged, and today's trust diff does not reveal that change (docs/owner-guide/07-trust-diff.md:161–163). This repeats the distinction already required by the S1 supervisor finding recorded on this bead.
- **Minimum fix:** Make this example about knowing that the uploader can reach files. Add that the actual directory and mode require checking the capability value the caller passes; signatures show the kind of authority, not the attenuation.

### B3 — The test-sandbox claim drops its kernel-dependent enforcement limit

- **Location:** docs/tide-or-rust.md:105–107.
- **Evidence:** “Test runs the driver starts get a kernel sandbox (Landlock and seccomp)” reads as unconditional enforcement of both mechanisms. README.md:400–410 and docs/owner-guide/08-sandbox.md:53–58 say the driver can print `sandbox: partial, not enforced: …` and continue under whatever remains. docs/docker.md:52–56 makes this especially relevant to the container route recommended at line 88. Supported x86_64 Linux alone does not guarantee full enforcement.
- **Minimum fix:** Add that enforcement depends on kernel support, the driver reports `full` or `partial`, and partial runs continue with the available restrictions. Link the README's Sandbox section for that qualification. The existing statement that builds and directly run binaries are unsandboxed is correct.

### B4 — Benchmark ratios lose material limits and comparison details

- **Location:** docs/tide-or-rust.md:89–97.
- **Evidence:** research/36-benchmarks.md:3–5 and 19–22 scope these observations to four small programs on one machine, not a general performance result or a gate run. Lines 47–52 mark the main run's series unstable, including the build series; the page attaches that warning only to its runtime bullet. Its generic “A simulated run takes 12.8 times the real one” drops the specific `tasks` workload, one seed and release configuration (research/36:241–254). The 1.07× file and 1.18× TCP comparisons use release Rust with Tide's profile flags, and the release-contract tie is for erased clauses (research/36:70, 73–77, 162–169).
- **Minimum fix:** Introduce both cost bullets as descriptive observations from four small programs on one machine, with unstable series. Identify the I/O comparator as release Rust with matched profile flags and the contracts result as a tie with clauses erased. Change the simulator sentence to the observed result for one seed of the `tasks` workload in release. The numbers themselves match the source; no new benchmark is needed.

### B5 — Required first-use explanations are still missing

- **Location:** docs/tide-or-rust.md:43–45, 73 and 104–106.
- **Evidence:** The review brief applies the launch texts' terms rule, and the bead's 2026-11-18 19:51 comment requires plain explanations at first appearance. This page uses “AI”, “tokens”, “E3 pilot”, “`unsafe` grant”, “Landlock” and “seccomp” without those explanations. In particular, E3 is not identified as a small trial with model reviewers, and the grant is not identified as the build operator's explicit permission. docs/at-a-glance.md:39–46 supplies the E3/token explanations; docs/owner-guide/02-admission.md:43–48 defines the grant; docs/owner-guide/08-sandbox.md:9–17 defines the sandbox mechanisms. A link alone does not meet the first-use rule.
- **Minimum fix:** Expand AI once, define tokens as the units a model reads, describe E3 as the small trial with AI reviewers, and define the grant as explicit operator permission for `unsafe` operations. Explain Landlock as file-access restrictions and seccomp as system-call restrictions, or omit their names here and link the sandbox explanation. These can be short parenthetical phrases.

## Non-blocking findings

1. **docs/tide-or-rust.md:118:** Make “needs macros” say “needs custom macros”. The cost bullet correctly says “user macros”, while design/reference.md:55–78 retains fixed built-in macros and derives. The Rust advice should use the same distinction.
2. **docs/tide-or-rust.md:89–97:** The cost overview could also mention the measured compiler-memory and binary-size increases (research/36:68–69) and E1's reference-loading cost (docs/owner-guide/09-experiments.md:39–40). These would make the adoption tradeoff fuller, but omitting every secondary measurement is not independently blocking for this short page.

## Reasoning

Reviewed the exact two-file diff from c82030d to 8b1b2ea, the complete new page, the README, at-a-glance, the relevant owner-guide chapters and answers, research/36, /44, /47 and /49, design/reference.md's affected rules, the recorded reference count, E1's gate report, E3's report, the demo and container records, the toolchain pin, the bead and its comments, and the builder's Stage S1b account. HEAD remained 8b1b2eaef92f038497f0e3c6d032b36e06466384 on branch fx-qlek.7 throughout inspection.

The catch paragraph preserves the important binding wording: the 87-advisory screen caught nothing; Tide's std had a gap, the screen exposed it, the std was fixed and re-tested under a new registration; the malicious port was already rejected in the screen and its capability-passing counterpart is now admitted. It includes both the positive chronology qualification and the absence of independent proof, admission/type-checking only with no port executed, and no first-time catch or rate. The linked README Evidence section gives the fuller selection, reversal and intent disclosures. There is no forbidden “unforgeable” claim or claim that the trust diff improves catching. research/44 introduces no contrary claim here: the page does not characterize that search as a completed negative or a simulator success.

The E1 counts, ratios and model identity match its gate report. The reference's recorded 24,999 tokens fit the 25,000-token budget. The demo's 8 failing seeds out of 16, simulator surface list, one-crate/no-crates.io restriction, async/user-macro/build-script exclusions, missing surfaces, x86_64 Linux restriction and pinned nightly agree with the record. The page correctly limits replay to the simulator's model and the same machine, distinguishes Rust compiler bugs from an `unsafe` grant, and does not claim that admission judges malicious intent or makes untrusted execution safe.

The opening explicitly complements Rust and rejects replacement/fork positioning. The concrete reasons to choose Rust are appropriate to v0's ecosystem, platforms and production/toolchain limitations. The ideal user is grounded in the implemented capability and simulation surfaces once B2 is corrected. No alternative design is needed; the minimum patch is local wording and qualifications in the new page. The README Reading Guide addition is appropriate.

All local link targets and referenced README heading anchors in the page resolve. `git diff --check c82030d..8b1b2ea` passes. No build, runtime test, external lookup, contact, commit or tracker write was performed. Only this report was written; measured results and implementation behavior were checked against their records, not independently rerun.

DONE
