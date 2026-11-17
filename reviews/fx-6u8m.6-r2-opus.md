Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate c8f9b20

Bead fx-6u8m.6. This is the Opus converge round under the GOVERNANCE.md Merge Rule. It reports and fixes nothing. It follows Codex r1 (approve) and r1-recheck (both fixes resolved).

## Verdict

**approve**. Confidence: high for the static reading. No build, test or bench was run. The runtime evidence is the builder's recorded quick run, which I re-parsed.

## Blocking findings

None.

## Non-blocking findings

1. **A missing stable toolchain leaves an empty `records.jsonl`, and the same `--out` is then refused.**
   - Where: `crates/tide-admit/benches/pairs.rs:256-276`.
   - What happens: `Bench::start` creates the output directory and `records.jsonl` before it resolves the pinned and stable compilers. Without `stable`, or without `rustup`, the run exits 1 with "rustc --print sysroot failed" (or "cannot run rustc", which misnames `rustup`). The empty records file stays behind. A retry with the same `--out` fails with "is not empty". The informational configuration also blocks every other measurement, and the README does not say a rustup `stable` toolchain is required.
   - Minimum fix: resolve both compilers before `create_dir_all`/`File::create`. Add one README line saying the tool needs rustup's `stable` toolchain.

2. **`cargo bench` without `--bench pairs` starts a full measurement.**
   - Where: `crates/tide-admit/Cargo.toml:13-17`.
   - What happens: a `harness = false` bench runs on any `cargo bench -p tide-admit`, `cargo bench --workspace`, or `cargo test --benches`/`--all-targets`. With no arguments it is in full mode: about 5-25 min, with default output. Nothing in the repository runs those commands today (I grepped: clippy `--all-targets` in fx-k3h5 only compiles). Still, an agent could start an unbooked heavy run by accident.
   - Minimum fix: add `bench = false` to the `[[bench]]` entry. An explicit `--bench pairs` still selects the target, so the README command keeps working.

3. **The header records the Tide linker but not the Rust arms' linker.**
   - Where: `pairs.rs:384,399-400`.
   - What happens: `tide_linker`/`tide_ld` give the `cc`/GNU `ld` versions. Nothing in `records.jsonl` says the rustc arms link through bundled `rust-lld`. The README states it (`benchmarks/README.md:137-139`), but someone holding only the dataset cannot see that the build times include a linker difference. On a future stable before 1.90 the README sentence would also be wrong for `stable-release`.
   - Minimum fix: a header field such as `"rust_linker":"cc + bundled rust-lld (self-contained)"`, or the `linker-flavor`/`link-self-contained` values read from each rustc's `--print target-spec-json` under `-Zunstable-options`.

4. **The std disclosure is a little narrower than the bias.**
   - Where: `benchmarks/README.md:140-142`.
   - What happens: in `tide-debug`, Tide's std is a dev-profile build. That means opt-level 0, plus debug assertions and overflow checks in std's own code, plus full debuginfo. It is linked by GNU `ld`. So both its run time (unoptimized std code) and its build/link time (larger rlibs with debuginfo) carry the cost. `rust-debug` links the toolchain's prebuilt optimized std. The README names only the optimization level and only the run side.
   - Minimum fix: one clause: "...its non-generic code runs at that optimization level and with that profile's checks, and its debug information adds to the debug arms' link."
   - The choice itself (Cargo builds dependencies in the build's profile) is defensible and recorded as `std_profile` in the header.

5. **The Tide build time includes cache writes and a second process. This is fine, but unstated.**
   - Where: README "build" bullet, `:146-150`.
   - What happens: a fresh directory means the driver's cache never hits. But each Tide build still pays for the re-executed compile child, which loads `librustc_driver` a second time, and for staging, the receipt and the copy out of `.tide-cache` (`invocation.rs:1206-1290`). That is Tide's real cold-build cost and is right to include. A reader comparing against rustc should be told it is in there.
   - Minimum fix: one sentence in the README.

6. **The run directory follows `--out`, and the header does not record niceness.**
   - Where: `pairs.rs:581`.
   - What happens:
     - `files` does its I/O under `<out>/run/`, so its absolute numbers depend on the filesystem `--out` is on. The quick run used `/home/...`, while the default is the build disk. Arms within one pair share the directory, so the ratios hold. Absolute numbers from runs split with `--pair` across different `--out` filesystems do not compare.
     - `heavy.sh` runs everything at `nice -n 19`, and the header does not record that.
   - Minimum fix: either run under `scratch()` or record the run directory. Record `getpriority` (or "nice 19 via heavy.sh") in the header. Both are optional.

7. **Run peak RSS is mostly `timeout`'s.**
   - Where: `pairs.rs:468,600`.
   - What happens: in the quick data, run peaks are 2.3-3.2 MiB against `timeout`'s ~2.4 MiB, so the run-phase `median_peak_rss_kib` mostly measures `timeout`. It is disclosed (`README:153-154`) and costs nothing to collect. But its summary invites a comparison that it cannot support for small-memory programs.
   - Fix: none needed. Optionally drop it from run summaries.

8. **The wall-time boundary is exact but undocumented for runs.**
   - What happens: wall time runs from just before the spawn to `wait4`. It includes `timeout`'s fork+exec (about 1 ms, the same for all arms, about 0.1% at the 0.5-2 s timed sizes) and pipe draining.
   - Fix: a README half-sentence ("including `timeout`'s own start") would make it exact.

9. **The order is fixed, with no warm-up round.**
   - What happens: `tide-debug` is always first in each round, and round 0 of each phase follows the std builds or the previous pair. This is fx-ayxr's ABAB as written. Medians absorb a first-round outlier, but CV does not, so it can only add rounds and cannot bias a median.
   - Fix: none required. It is noted for fx-6u8m.2's reading of CV.

10. **Simplicity: small deletions that keep the contract.**
    - (a) `tests/benchmarks.rs:17-22` includes `peak_rss.rs` and `stats.rs` a second time, beside the copies `pairs.rs` already brings in. The test crate therefore compiles each twice, and `admitted.rs`/`build_std` twice too. Declare `pub mod stats; pub mod peak_rss;` in `pairs.rs`, use `pairs::stats`/`pairs::peak_rss` in the tests, and delete the two `#[path]` includes and one `allow`.
    - (b) `header`'s `sizes` (`pairs.rs:371`) repeats `Bench::arg`. Use `self.arg(p)`.
    - (c) `Config.informational` always equals `matches!(compiler, Compiler::Stable)`, so it can be derived.
    - (d) The pinned-rustc lookup (`RUSTC` else `rustc` in the manifest dir, then `--print sysroot`) exists in both `build_std` (`tide-std/tests/support/mod.rs:9-20`) and `sysroot_rustc`. A shared helper would give it one owner, but the two differ in panic versus `Result`, so this is optional.
    - Beyond these, I found nothing to delete. At 717 lines the tool is proportionate to its contract: six configurations, four record kinds, a header with every recorded identity, the interleaved loop with raise-N, and the console table. The console table (about 40 lines) is the only part the contract does not strictly need, and it is what an operator reads.

11. **Doc nits.**
    - `README:169`: the default results path is `<CARGO_TARGET_DIR>/tmp/bench-pairs/results/`, and is `target/tmp/...` only without `CARGO_TARGET_DIR`. The full-run example sets `CARGO_TARGET_DIR`.
    - `README:128`: "Cargo's dev profile" leaves out Cargo's `codegen-units=256` and incremental mode. Both the driver and the rustc arms use rustc's non-incremental default, so the arms stay comparable. Only the label overstates.

## Reasoning, by lens

**1. Correctness.**
- Interleaving: `interleaved` is round-major over all six arms (`stats.rs:53-57`), and every added batch covers every arm (`pairs.rs:440`), so any two arms alternate A B A B.
- Statistics:
  - Median handles odd counts, even counts and empty input.
  - CV is the sample standard deviation over the mean, and is `None` below two values or for a non-positive mean.
  - `stable` needs CV ≤ 1%.
  - `more_rounds` returns 0 at the cap or when every series is stable, and otherwise returns `min(step, cap - done)`. With 5/5/30 the run stops at 5, 10, … 30.
  - Unstable series at the cap are marked `stable:false` and listed.
  - Quick mode writes `cv`/`stable` as null.
  - NaN formatting cannot arise, because every series has at least one value.
- Peak RSS: `wait4` on the exact pid returns `ru_maxrss` as the maximum over the child and its waited-for descendants. The driver's outer process `.output()`s its compile child (`invocation.rs:1241`), rustc waits for `cc`, `cc` waits for `ld`/the lld wrapper, and `timeout` waits for the program. No competing waiter exists. EINTR retries. Both pipes are drained concurrently, so no deadlock.
- Output check: the reference is round 0 arm 0 (`tide-debug`) of each pair's run phase. A mismatch writes `error` and stops before the sample enters a series. `close` writes `end` only on success. This was verified by the new test.
- Arguments:
  - `--bench` is swallowed.
  - `--out` must be absolute and new or empty.
  - An unknown `--pair` fails before anything is created.
  - Full mode refuses a pair with no `TIMED` size.
  - Bad arguments exit 2 and failures exit 1.
- Configuration flags against `invocation.rs:342-398`:
  - Debug: opt 0, debug-assertions on, overflow on, debuginfo 2, plus `share-generics=no`, `inline-mir=no` and `cross-crate-inline-threshold=never`.
  - Release: opt 3, debug-assertions off, overflow on, plus the same three.
  - `rust-debug` matches the debug set minus the three Tide flags, and `rust-release-tide-flags` matches the release set exactly. Edition 2024 everywhere.
- Re-parsed records: `quick-c8f9b20/records.jsonl` has 1 header, 24 build, 24 run, 48 summary and 1 end. That is 4 pairs × 6 configurations per phase. The header shows revision `c8f9b20…` with `dirty:false`, pinned 1.99.0-nightly 771916f90, stable 1.96.0, and `std_profile` dev and release. Together these meet the bead's Acceptance check.

**2. Fairness.**
- Already handled:
  - Rustc runs from its sysroot, with no rustup proxy.
  - The driver is built in the bench profile and links the same sysroot `librustc_driver`.
  - Each build gets a new directory for both arms, on the same filesystem (`CARGO_TARGET_TMPDIR/build`).
  - Both std builds happen before any timing.
  - The release std is optimized.
  - Every run is under the same `timeout` wrapper.
- Disclosed in the README: the linker difference, the std profile, and the driver's forced flags.
- Gaps (non-blocking findings 3-6, 8):
  - the header has nothing for the Rust linker;
  - the std disclosure covers only the optimization level;
  - cache writes and the second process are unstated;
  - the run filesystem and niceness are not recorded;
  - the `timeout` start-up inside run wall time is unstated.
- None of these hides a bias inside a number without any disclosure that would change a reader's conclusion. The largest, the linker, is disclosed in the README.

**3. Ownership and leftovers.**
- Ownership:
  - Driver arguments have one owner, `driver_with`, and `driver` delegates to it.
  - The std build has one owner, `build_std(dir, release)`.
  - The rustc-arm command is local by necessity (per-configuration flags, measured).
- Callers: every `build_std` caller is updated (`admitted.rs:33`, `tide-std/tests/entry.rs:29`). No other includer of `tide-std/tests/support/mod.rs` exists. The `driver` callers are unchanged.
- Type-checking: `cargo test` type-checks `pairs.rs` through `#[path] mod pairs` and runs only `check_output`/`close`.
- Dependencies: none added. `libc` was already a dependency, and the JSON is hand-encoded.
- Leftovers: nothing orphaned. The new `allow`s each carry a reason.

**4. Simplicity.** See non-blocking finding 10. The deletions are small, and the tool is not oversized for its contract.

**5. Docs and claims.**
- Commands, flags, paths, record kinds, `--pair` splitting, the time budget (about 50 s per round, so 5-25 min, against `timeout 3600`) and the heavy.sh form match the code and AGENTS.md. Remaining nits are in findings 1 and 11.
- No performance claim appears in the committed text. The `TIMED` comment's run durations are calibration notes, and the root README still says no measurements are published.
- fx-6u8m.2 can run the full mode from `benchmarks/README.md` alone, given that the machine has rustup `stable` (finding 1).

Not checked: no builds, tests, benches or compiled workloads were run, and no subagents were used. Full-mode behavior (raise-N in practice, timing on a booked slot) was reviewed statically only.

DONE
