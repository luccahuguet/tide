Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate d8d9425 (re-check of 524cc22 items)
Verdict: request_changes

1. **Finding 1's missing measurement — Resolved.** The new contracts-only test phase compares the same workload at 200,000 in four arms: ordinary Tide debug, Tide debug test, erased Tide release and Rust debug. `crates/tide-admit/benches/pairs.rs:269` appends a timed test calling `run` with that size, builds with debug configuration and `--test`, and selects only `bench_test::timed` for timing. Before the rounds, `bench_test::sampled` must pass its expected-panic test on an unsorted vector; a missing selected test cannot pass the additional “1 passed” check. `sampled_clause_ran:true` is recorded only after this succeeds. The timed process must exit successfully, and its stdout must contain the exact reference output; the added test at `crates/tide-admit/tests/benchmarks.rs:391` covers matching and mismatching program output inside libtest framing. The framing/containment boundary is documented rather than claimed as equality of the complete harness output.

   Independently recomputed from all 120 new raw measurements (sample standard deviation / mean):

   | Arm | Median seconds | CV | N |
   | --- | ---: | ---: | ---: |
   | tide-debug-test | 2.8022985 | 0.81458% | 30 |
   | tide-debug | 0.3624785 | 4.21368% | 30 |
   | tide-release | 0.0394420 | 5.86662% | 30 |
   | rust-debug | 0.3045460 | 3.17082% | 30 |

   The displayed medians/CVs/N and test-to-reference ratios (7.73x, 71.0x, 9.20x) agree with the records. Plain Tide debug / Rust debug is 1.19023x. All four arms have rounds 0–29, size 200000 and exit 0; only the test arm meets 1% CV. The note explicitly states “outside a quiet window” in its results introduction, summary row, test-build discussion and method (`research/36-benchmarks.md:385`), and gives all four CVs. It also discloses header load 15.37 and the three unstable reference arms. No new quiet-window or stability claim is implied.

2. **Invariant wording — Resolved.** `benchmarks/README.md:44` now separates constant cost per boundary-clause call from constant cost per invariant evaluation, at every head of the sieve's loop. This addresses the prior wording item without changing the workload or records.

3. **Trace-retention wording — Resolved.** `research/36-benchmarks.md:254` now says events remain as data and are rendered as text when read, avoiding the implication that reading removes them.

4. **Blocking defect introduced by the fix — Standalone `--phase run` does not build its inputs.** `crates/tide-admit/benches/pairs.rs:814` leaves `Phase::Run` preparation empty, while only Sim/Test call `keep` for their ordinary-program arms. The new selector accepts `--phase run`, and `benchmarks/README.md:247` explicitly promises missing programs are built unmeasured. With a fresh output directory, `--quick --pair contracts --phase run` reaches `execute` at line 1150 with no `bin/contracts/tide-debug`; timeout fails to execute it and the dataset stops without end. An earlier output directory cannot supply it, because the tool requires a new or empty directory. **Minimum fix:** prepare the Run phase's program arms with the existing `keep` helper, as Sim/Test already do. Add a focused regression or saved quick check exercising standalone Run preparation and successful completion. This does not invalidate either recorded dataset, but the newly advertised selector is broken and needs correction before approval.

5. **Non-blocking overclaim in the new note — The 7.7x explanation is not established.** `research/36-benchmarks.md:194` assigns the whole difference to the two scans, then says a larger whole-program ratio fits those scans roughly tripling insert_sorted's cost because the function dominates. If that function merely tripled and dominated, the whole-program ratio would approach three, not explain 7.7. These timings measure complete test/non-test processes, not that function's components. **Minimum fix:** state that the debug test build took 7.7x as long, and remove the unsupported explanation linking “roughly triple” to the larger ratio and its asserted size growth. Keep the measured size, ratios and warning that this workload is not representative of typical contract cost. No additional measurement is needed to make this wording honest.

6. **Non-blocking numerical wording — Peak versus median.** `research/36-benchmarks.md:391` says the largest process peaked at 3.6 MiB. The maximum raw recorded peak is 3,776 KiB = 3.6875 MiB, which rounds to 3.7 MiB; 3.6 MiB is the rounded median peak for the test arm (3,672 KiB). **Minimum fix:** write 3.7 MiB for the maximum, or explicitly call 3.6 MiB the median peak.

Earlier-number check: the main records file is byte-identical to 524cc22 (284,127 bytes; SHA-256 `6c7d946267e2d7110e48de0a6fc7b12969937534ce7adb8aa4e7066e5ed96c39`). All 24 earlier measurement-table rows are textually unchanged. The new file is 20,224 bytes with the stated SHA-256 `bf3c7456bd4abf1c580af48aa09215a25dabc3437f107dcb5324be42170c49bb`: one header, 120 measurements, four summaries and one end. Its header records clean revision c04dc44a6d6e3397a91fe87b86d62abe19811127, measured phase test, size 200000 and nice 19. Its header-to-end span is 108 seconds. Compared the saved final quick dataset's summary keys with the union of both committed datasets: identical 64-key sets, with clean d8d9425 revision and an end record. This corroborates the default table shape, not the untested standalone Run selection.

Scope and identity: inspected only `git diff 524cc22 d8d9425`, its finding-to-fix map, relevant changed paths and saved records. HEAD was d8d94253246178f40315369f71c987584a17e822, tracked tree clean. Interdiff SHA-256: `6acfb382f6deb453add680f9770d1657cc1095cbd2f9932460f90d4d7a73d08c`. `git diff --check 524cc22 d8d9425` passed. No new full review was run. The earlier findings remain resolved except for the introduced defect in item 4; items 5–6 are non-blocking.

Not checked: no builds, benches, tests, compiled workloads or subagents; only inspection and read-only Python. Only this requested report was written.
Risk: standalone Run selection currently fails; the test-build ratios come from the explicitly disclosed non-quiet run.
DONE
