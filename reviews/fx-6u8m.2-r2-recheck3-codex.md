Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 95ab967 (re-check of d8d9425 items)
Verdict: approve

4. **Resolved — Standalone Run prepares its programs.** `crates/tide-admit/benches/pairs.rs:841` now includes Run in the preparation path shared with Sim/Test. Every ordinary-program arm calls the existing `keep` helper before measurement. This builds missing binaries unmeasured and retains the existing no-op when a preceding Build phase already supplied them. `crates/tide-admit/tests/benchmarks.rs:405` exercises Run alone on contracts, in quick mode with a fresh output directory; it requires successful completion, zero measured build records, six run records, six summaries and one end record. Removing the preparation fix would make its first execution fail before those assertions. The test explicitly substitutes the pinned compiler for the stable arm and shares the suite's std builds: it tests preparation/completion without claiming performance equivalence for those substitutions. The start/open extraction preserves production toolchain resolution, std construction, pair validation and the new/empty output-directory check. No regression found in the changed paths. The author's saved workspace-test3.log records this regression passing and the benchmarks test binary finishing 12/0; this is saved evidence, not an independent execution.

5. **Resolved — The 7.7x statement reports the observed difference.** `research/36-benchmarks.md:194` now says that the debug test build took 7.7x as long at 200,000. It removes the unsupported explanation connecting a roughly threefold function cost to the larger whole-program ratio and removes the asserted ratio growth. The caution about this workload's sampled vector scans remains appropriate. Related README and TEST_SIZE comments no longer present the quadratic-growth explanation as established; the 70-second estimate remains explicitly an estimate. No new causal overclaim found in these edits.

6. **Resolved — Maximum peak is rounded correctly.** `research/36-benchmarks.md:387` gives 3.7 MiB. Independently parsed the unchanged test-build records: the maximum is 3,776 KiB = 3.6875 MiB, which rounds to 3.7 MiB. This corrects the prose's former confusion with the median; it changes no measurement.

Recorded-number check: both committed JSONL files are byte-identical to d8d9425:

- Main: 284,127 bytes; SHA-256 `6c7d946267e2d7110e48de0a6fc7b12969937534ce7adb8aa4e7066e5ed96c39`.
- Test-build: 20,224 bytes; SHA-256 `bf3c7456bd4abf1c580af48aa09215a25dabc3437f107dcb5324be42170c49bb`.

All 71 Markdown table rows in the note are textually unchanged, including the earlier and test-build results. No recorded number changed. The 3.6-to-3.7 prose correction is supported by those same records.

Scope and identity: checked only items 4–6 through `git diff d8d9425 95ab967`, their supporting changed paths, the fix map and saved evidence. No new full review was run. HEAD was `95ab967667144577f0391c520f0637d12df2fca3`, tracked tree clean. Interdiff SHA-256: `d70b49ac2794e429528694d85b59b61b700697e071107eb8df58b8131be0883b`. `git diff --check d8d9425 95ab967` passed. No remaining finding or newly introduced problem found in this scoped re-check. Confidence: high. No override.

Not checked: no builds, benches, tests, compiled workloads or subagents were run; only inspection and read-only Python. Only this requested report was written.
Risk: the disclosed unstable main run and non-quiet test-build run remain the limits of the benchmark evidence.
DONE
