Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate 95ab967

# fx-6u8m.2 — Opus converge round (r3), read-only

## Verdict: request_changes

Two blocking findings. Both are fixed by editing text in `research/36-benchmarks.md`. Neither needs a rerun: the records already hold the data.

## Blocking findings

### B1. Ties that the paired data shows are losses (summary and §1.4/§1.6)

The note calls a ratio a tie when it is within about one CV of 1.00 (lines 50-51, 271-273). That rule ignores the interleaving the method exists for. Each round measures every arm back to back, so the per-round paired ratio is the sharper test, and it is in the records. Recomputed from `records.jsonl`:

**(a) `files` capability I/O, `tide-release` against `rust-release-tide-flags`.**
- Tide's arm is slower in **26 of 30** interleaved rounds.
- Paired per-round median ratio 1.066, IQR 1.029-1.090.
- One-sided sign test p ≈ 3e-5.

The note says the opposite in three places:
- line 65 (summary row): "1.04x … a tie";
- lines 198-200: "this run does not resolve the extra system calls' cost";
- lines 272-273: "the `files` I/O cost … [is a tie]".

This is the line a README or paper would quote ("capability file I/O: a tie"). It turns a small loss into "no cost".

Controls, from the same check: the other ties hold under it.
- `contracts` release: 11/30 rounds.
- `contracts` debug at 1.2M: 16/30.
- Profile flags on `contracts`, `files`, `tcp`: 15, 18 and 16 of 30.
- `stable-release`: 10-16/30.

Position in the round is a weak explanation for `files`. Positions 4→5 (`rust-release` → `rust-release-tide-flags`) show 18/30. The note's §3 fixed-order caveat still applies.

**(b) Contracts in debug, summary row (line 63): "1.05x, a tie".**
- The note's own second run measured the same comparison at 200,000: `tide-debug` / `rust-debug` paired median 1.175, slower in **30 of 30** rounds.
- The body says so (lines 196-197). The summary row drops it, so a quote of the row loses the loss.

Minimum fix (text only):
- Define a tie by the paired rounds, e.g. "Tide's arm slower in no more than about 20 of 30 rounds", or simply state the per-round count next to each tie call.
- Line 65: `files` 1.04x by medians, Tide slower in 26/30 rounds, paired median 1.07x: a small loss, of a few percent.
- Fix lines 198-200 and 272-273 to match.
- Line 63: add "1.18-1.19x at 200,000 in the separate test-build run, slower in every round".
- Line 71 ("Every comparison above is a loss for Tide or a tie") stays true.

### B2. The measured revisions are not in the candidate's history, and §2.6 misstates what the commit adds

- `git merge-base --is-ancestor bf6b958 95ab967` fails, and so does `c04dc44`. Both are amended-away states of this single commit (report.md: "amends d8d9425, c04dc44, 524cc22, ff6b2a1, bf6b958 and 49ed992"). No branch or tag contains either.
- After landing, the worktree and branch are removed (AP-GIT-001). Then the `revision` that both records' headers carry, and that line 10-11 tells a reader to trust ("the rerun tool at the revision its header records"), names a commit nobody can check out. Garbage collection will eventually delete it.
- Lines 404-408 say: "This note's commit adds the note and both record files to `c04dc44`." That is false. 95ab967's parent is `11a0bbb` (main). Against `c04dc44` it also changes:
  - `pairs.rs`: Phase::Run now calls `keep`; the `start`/`open` split; public items.
  - `tests/benchmarks.rs`: +38 lines.
  - `benchmarks/README.md`.

  The claim beside it, that the measurement code did not change, holds. I diffed `bf6b958`→`c04dc44`→`95ab967`:
  - the build, rebuild, run and sim paths are unchanged in a full run;
  - the new `keep` call in the run phase is a no-op after a build phase.

  The sentence about what the commit adds is still wrong.

Failing case: a paper reviewer runs `git show bf6b958:crates/tide-admit/benches/pairs.rs` on the landed repository and gets "unknown revision".

Minimum fix, either:
1. **(Recommended)** Keep both revisions reachable: the coordinator lands with refs such as tags `bench/fx-6u8m.2/bf6b958` and `bench/fx-6u8m.2/c04dc44`. Correct lines 404-408 to say that the two revisions are earlier, unlanded states of this commit, reachable by those tags, and that against the landed tree they differ in the measured paths only by `keep` in the run phase, a no-op after a build phase.
2. Or, with no new refs: make the same correction, and drop "at the revision its header records" from line 10-11 as a reproducibility claim. Instead, give the SHA-256 of `pairs.rs` as measured, or state that the landed `pairs.rs` measures identically.

## Non-blocking findings

- **N1. `research/36-benchmarks.md:60`**: "+18% to +39%". By the stated "same profile" (`tide-release` vs `rust-release`), `files` is 157,830/134,906 KiB = 1.170, which is **+17%**. Against `rust-release-tide-flags` (134,404 KiB) it is 1.174, also +17%. Change to +17%.
- **N2. `research/36-benchmarks.md:61`**: "19-21 MB against 4.6-5.4 MB" divides KiB by 1000.
  - 18,992-21,374 KiB is 19.4-21.9 MB or 18.5-20.9 MiB.
  - 4,588-5,400 KiB is 4.7-5.5 MB or 4.5-5.3 MiB.
  - Pick one unit and say it.
- **N3. `research/36-benchmarks.md:118`**: "Rebuilds peak the same as cold builds" is true for `tide-debug` (163-193 both). It is false for `rust-debug`: cold 117-152 MiB, incremental rebuild 117-122 MiB (`tasks` 152 → 120, `tcp` 139 → 121). Say it for `tide-debug` only.
- **N4. `research/36-benchmarks.md:69, 237-238`**: "with the release std / dev std". The arms are the whole `tide-release` / `tide-debug` configurations: program optimization and profile flags as well as the std. Say "under `tide-release` / `tide-debug`".
- **N5. `pairs.rs` `options()` / `open()`, phase selection.**
  - `--phase` keeps the user's order and duplicates. `--phase run --phase run` measures and summarizes run twice. `--phase run --phase build` changes which phase follows the std builds.
  - A phase that measures no named pair is silent. `--pair files --phase sim` writes a header and an `end` record, then exits 0 having measured nothing.
  - Fix and simplification: `phases: PHASES.into_iter().filter(|p| options.phases.is_empty() || options.phases.contains(p)).collect()` gives canonical order and no duplicates. Also error when no (pair, phase) is measured. This needs `Phase: PartialEq`, which it already derives.
- **N6. `pairs.rs` `build_harness`** returns `Result<u64, String>` with a dummy `Ok(0)` for the test harness. It also restates the module names (`"bench_sim::events"`, `"bench_test::sampled"`) that `Arm::module()` already owns.
  - Deletion: derive the check as `format!("{}::{}", arm.module()?, if sim {"events"} else {"sampled"})`.
  - Return `Option<u64>` (events for sim, none for test), assigned straight to `series.events`.
  - This removes the duplicated mapping (one owner per name).
- **N7. Test coverage.**
  - `edited`, `binary_size` (exact synthetic fixture) and `within` have strong unit tests that fail on a regression.
  - The new run-phase regression test fails on the bug it targets.
  - The sim and test harness paths (`build_harness`, the `sampled` check, `trace_events`) have no suite test. Only real runs exercise them (`quick-final2` on d8d9425 shows both phases complete).
  - Acceptable given cost. Note it on the bead so a later change to `sim_module` or `test_module` knows to run `--quick --phase sim --phase test`.
- **N8. `research/36-benchmarks.md:194-196`**: "This workload is not representative … its sampled clauses scan a vector on every insert" reads as the cause of the 7.7x. The test build differs from the plain one in other ways too: the libtest harness, and `cfg(test)` code. Label the clause [repo evidence] for the scan and [inference] for the attribution, as the rest of §1.4 does.
- **N9. `--quick` reproducibility (Acceptance 2)** is shown on d8d9425 (`target/work/quick-final2`), not on 95ab967.
  - The output has the same 64 summary keys as the two full runs' union: 60 + 4 test.
  - The only later tool change is `keep` in the run phase, a no-op after a build phase, so the shape claim holds.
  - Record on the bead that the evidence is d8d9425 plus this argument, or rerun `--quick` on the landed tree after rebase.
- **N10.** The tables were rendered by `target/work/tables.py`, which is not committed. The summary records carry every median, CV and N, so a reader can recompute, as I did. No change is needed, but §2.6 could say "ratios are quotients of summary medians".

## Reasoning

### Lens 1: claims against data
I recomputed every summary record from the measurement records, with read-only Python, for both files:
- 64/64 match in median, CV (sample SD over mean) and N.
- All 1,800 + 120 measurements exited 0.
- Every build of a series wrote one binary size.
- Rounds are contiguous 0-29.

I recomputed every ratio in §1.1-§1.5:
- cold and rebuild ratios;
- `tide-release`/`rust-release-tide-flags` build ratios: 2.10/1.88/1.69/1.98;
- loaded-size ratios: 1.36/1.73/1.63/1.95;
- run ratios;
- 1.74 × 1.31 ≈ 2.29;
- test-build ratios: 7.73/71.0/9.20, and 1.19;
- simulator: 12.84x and 16.11x, 1.774 and 3.781 µs per event, about 430 B per event.

All match, except the RSS lower bound (N1) and units (N2).

The stability disclosure is accurate: 0 of 60 stable in the main run, CV 1.04-46.7%, and the largest CVs are round-0 outliers. For example, the `contracts` `stable-release` build is 0.819 s in round 0, and its CV drops to 8.2% without that round.

The test-build run's off-window and load-15.37 header are disclosed in the summary row, §1.4 and §2.5. The hedging is otherwise consistent. The one material problem is that the tie rule hides a measurable `files` loss, and the summary drops the 200,000 debug-contracts loss (B1). "Every comparison is a loss or a tie, none a win" stays true after the fix.

### Lens 2: method
These disclosures are present and accurate against the header:
- linkers: `gnu-lld-cc`/bundled `rust-lld` against `cc` 13.3.0 + GNU ld 2.42;
- the memory cap (`HEAVY_MEM=8G`, peak 305 MiB);
- machine state (load 3.15 → 5.20, nice 19, shared laptop);
- the toolchains;
- N;
- "not D0's E4/E5" (top of the note).

The std-profile asymmetry, the fixed order, process start and the lightest-edit bias are all in §3.

Reproducibility of the table shape from `--quick` holds (N9). Reproducibility of the exact measured code does not survive landing as written (B2).

### Lens 3: tool correctness
- **Rebuild:** `package()` and `rebuild()` resolve to the same `CARGO_TARGET_TMPDIR/build/bench-pairs-rebuild/<pair>/<config>` path. The edit differs every round, so the driver cache cannot hit; `tide-debug` rebuild ≈ cold confirms it. `-Cincremental` applies to rustc only, and the warm build uses it too.
- **Binary size** reads the first 64 KiB and sums `PT_LOAD` `p_filesz` with checked arithmetic. It rejects non-ELF64-LE input and truncated program headers, and the test pins both.
- **Simulator:**
  - The timed test reads `sim.records()`, not `trace()`, so trace rendering stays out of the timed span, as the note says.
  - The events count comes from an untimed test.
  - The output is checked against the real arm.
- **Test build:**
  - The `sampled` should_panic check passes only on "1 passed".
  - Output is located with `within`, which falls back to the whole output, so a mismatch still fails.
- **Phase selection:**
  - Standalone `run`, `sim` and `test` now build missing programs through `keep`, and `rebuild` warms its own.
  - The run-alone regression test covers the bug Codex found.
  - The remaining rough edges are non-blocking (N5).
- **Ownership:** each measurement has one owner: `build`/`rebuild` for size, `execute` for runs, `build_harness` for harness binaries. The one duplication is module names (N6).

### Lens 4: records hygiene
- No binaries are committed: two JSONL files, 284,127 and 20,224 bytes.
- The SHA-256s in the note match `sha256sum`.
- `dirty:false` in both headers.
- Each header records a revision, but those revisions are unreachable from the candidate (B2).
- The header's machine-local paths (`/home/lucca/...`, `/mnt/build/...`) are expected for a provenance header, and they leak nothing beyond the repository's existing records.

### Lens 5: simplicity and leftovers
Concrete deletions are in N5 (canonical phase filter replaces the ordered push) and N6 (dummy `Ok(0)` and duplicated module names). I found no orphaned helpers. `sim_arg`/`arg` were folded into `phase_arg`, and the old `simulated: bool` is gone.

DONE
