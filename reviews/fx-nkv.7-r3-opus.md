Model: claude-opus-5-5 · Date: 2026-11-15

## Verdict

request_changes. Two findings block, and probes on the candidate confirmed both. The replay boundary itself held up under review: untrusted manifests, strict identity, exact replay and the completion receipt.

## Scope

Converge round, report only, read-only, on `.worktrees/fx-nkv.7` at HEAD `c06d86a` (single task commit on `16698e1`). I read the full `git show HEAD`, the author's report, the Codex r1 review and both re-checks, the bead's Scope, Acceptance and 2026-11-12 decisions, AGENTS.md, design/03 (RUN-1, PANIC-1, §9.6 REPLAY-1/2, §9.7) and design/01 D15. I also read research/32.

Code traced:
- driver: `invocation.rs` and `invocation/replay.rs`, covering parse, cache and paired paths, receipt publication, identity, validation and the executable copy;
- std: `replay.rs` and `sim/manifest.rs`, plus the test runner and `Gen`;
- scheduler origin;
- the macro.

Probes ran on the candidate's built driver and std. The build was focused (`cargo test -p tide-admit --test build ordinary_const_tests_keep`), followed by `probes/run.sh`. Both went through `heavy.sh env CARGO_BUILD_JOBS=4 timeout …`, started at 1-minute load 3.7–5.5, and both exited 0. The probe script and sources are in `/tmp/claude-1000/-home-lucca-pjs-lab-tide/e3e607df-e147-4ed5-b3f9-ae12e97442dc/scratchpad/probes/`. Full suites were not repeated.

## Blocking

### 1. Calling a parameterized `#[test]` from another test now panics with "nested property evaluation"

The new guard is at `crates/tide-std/src/sim/manifest.rs:106`. It is reached from `run_config` at `crates/tide-std/src/test/runner.rs:361` and `:387`.

**Evidence.** On the parent, `run_config` had no evaluation slot, so it could be re-entered. `git grep "nested property evaluation" HEAD~1` finds nothing. On the candidate, `begin` asserts that the thread has no active evaluation. `run_plain` checks `active()` first (`runner.rs:318`), but `run_config` and `sim::__run` do not.

I probed this source by running the binary directly, with no driver context:
```rust
#[test] fn prop(x: u8) { assert!(u32::from(x) <= 255); }
#[test] fn caller() { prop(); }
```
Result: `prop ... ok`, but `caller ... FAILED`, with `cause: nested property evaluation at crates/tide-std/src/sim/manifest.rs:106:9`. This is the class of regression Codex r1 flagged for parameterless tests, now for parameterized ones. It also breaks plain binary runs, not only replay.

Two related effects:
- In fresh driver mode, `begin` reserves a run directory and writes an incomplete initial manifest *before* the assert. Every nested call therefore leaves an orphan manifest under `target/sim`.
- A property or `#[sim_test]` wrapper called from a simulated child task, or a `#[sim_test]` wrapper's child process, starts a separate top-level recording. That recording inherits `__TIDE_REPLAY_CONTEXT` through `CONTROLS`. In fresh mode it writes extra manifests, which breaks "one manifest per harness evaluation". In replay mode it fails the outer test with "replay test identity mismatch", so a deterministic outer test refuses to replay.

**Suggested fix.** In `run_config`, and in `sim::__run`'s non-child branch, when `manifest::active()` is true, run the nested property as the parent did: no `begin` and no persisted manifest, with its roots and choices kept out of the caller's recording. Also move `begin`'s nested assert ahead of the artifact reservation. Add one regression next to `ordinary_unit_tests_compose…`: a test calling a parameterized test, directly and under `--tide-run-test`, that asserts it passes and that exactly one manifest is written.

### 2. Path remapping on every linked build changes compile diagnostics and runtime locations

The flag is added at `crates/tide-admit/src/invocation.rs:339`. The remap roots are set at `:1084` and `:526`.

**Evidence.** `--remap-path-prefix=<source_root>=tide-source` is added whenever `self.link` is set. That covers `--crate-type=bin` builds too, which are never replayed. rustc's default remap scope includes diagnostics.

I probed this with one type error, `#[tide::main] fn main() { let x: u32 = "s"; let _ = x; }`:
- `--emit=link` prints `tide-source/demo.rs:2:26: error[E0308]: mismatched types…`;
- `--emit=metadata` prints `/tmp/…/probes/diag/package/src/demo.rs:2:26: error[E0308]…`.

So every linked build's errors now name a path that does not exist on disk, and the path differs between link and metadata modes. Panic and `Location` paths in shipped binaries change too; the TCP test's expectation was edited to match. README.md, `main.rs`'s docs and design/03 say nothing about this. The bead does not need it for bins.

The same remapping forced the new "foreign" reverse-mapping branch in `Sources::read_file` (`invocation.rs` around line 775). That branch touches the file-loader trust boundary only because of this change.

**Suggested fix.** Apply the remap only on the `--test` link path that replay uses. Scope it to embedded paths so diagnostics keep real paths: `-Zremap-path-scope=macro,object` on this nightly. Check that `file!()`, panic `Location`s and the proc-macro `Span::file()` used for `Config.call` stay remapped under that scope; if they do not, use the narrowest scope that keeps them. Then recheck whether the `read_file` workaround is still needed, and document whatever remapping remains in README.md.

## Non-blocking

1. **The replay command prints for passing evaluations.** See `crates/tide-std/src/sim/manifest.rs:212` (`if signature != "passed"`). An ordinary test that passes has signature `returned`, and a discarded property case has signature `rejected: …`, so both print `  replay: … --replay …`.
   - Probe: `--tide-run-test ok` on `#[test] fn ok() { let sim = Root::sim(3); … }` printed a replay command beside `test ok ... ok`.
   - A heavily filtered property prints one line per discard, up to 10,000.
   - README.md:392 says "Failures print…".
   - Fix: print only for `Outcome::Failed`, passing that fact into `finish` rather than matching on the signature text.
2. **Fresh runs persist every evaluation and grow `target/sim` without limit.** See `runner.rs:387` and `manifest.rs:77–118`. A property run writes up to 100 or more run directories (up to 10,000 on a vacuous run), and nothing cleans them up. `Journal::save` also rewrites the whole manifest after every draw, which is O(n²) for long tapes. The decision allows one manifest per evaluation; consider keeping only failing evaluations or adding a retention note, and batching saves for draws while keeping the "recorded before use" ordering for root inputs.
3. **`build_identity` runs on every invocation.** See `invocation.rs:1124`. It runs for metadata builds, non-test links and the internal `--tide-compile` stage too. It re-hashes the driver binary (120 MB debug), std and every dep-dir file, which the cache-key `fingerprint` loop has just hashed. Measured cost is small on this machine (0.32 s per cached metadata build), so this is a nit. Compute it only for `link && test`, or derive both from one pass, so one owner decides what dependency content is.
4. **The completion receipt does not record which evaluation wrote it.** See `manifest.rs:227`. Any `finish` in the process that matches the expected roots, trace and outcome writes it. I found no forging path:
   - test code holds no real capability (RUN-1);
   - `Root::sim` inside a simulated task panics, so a nested recording on a task thread cannot reproduce non-empty expected roots;
   - `process::exit` or an abort before `finish` leaves no receipt.

   Still, a once-per-process guard would turn this from an inference into a property: only the first top-level replay evaluation may write the receipt, and a second one fails. Finding 1's fix removes the reachable nested recordings.
5. **The driver prints no verdict of its own.** "replay reproduced" comes from the candidate's stderr. Exit code 2 is authoritative, so nothing reports success falsely. Printing `tide-admit: replay reproduced` from `run_test_process` after the receipt check would make the trusted line distinct from candidate output (compare RUN-6's spirit).
6. **A refused replay deletes the existing executable.** `clear_outputs()` runs before `validate_replay` (`invocation.rs:1082` vs `:1125`). A mismatched `--replay` deletes the existing executable and receipt before refusing. Harmless, but it surprises.
7. **Docs.**
   - Nothing links to research/32. design/03 §9.6, or the new paragraph near 03:1960, could cite it as 03:2180 cites research/30.
   - research/32 does not say which recommendations were adopted, or that the decision's "research/28" landed as 32. Its falsifier row "original and shrunk tapes replay" differs from what was built (original evaluation only, by decision). A two-line status header would align it.
   - design/03:1989 still says fs event "names are provisional until replay manifests". Reword now that `trace/1` exists.
   - The `--help` text and README do not say that const tests and a test-decorated `main` are not runnable through `--tide-run-test`.
8. **Path-relative identity.** `semantic_options` strips the canonical source root from the lexical `self.source` (`replay.rs:62–68`, with `package_root` at `invocation.rs:419–431`). When the cwd or source path goes through a symlink, the strip fails: the option embeds the absolute lexical path, and the remap prefix (canonical) does not match rustc's input path. Same-path replay still works; relocation silently loses normalization. Normalize from the canonical source instead.

## Reasoning

**Untrusted manifests.** The manifest is read once into memory (`validate_replay`). It supplies no path that is executed or read. The run directory comes from the driver's `OUT/target/sim`, and the artifact names use digests plus validated canonical decimals. The test name must be a key of the receipt's macro-sourced `tests` map, which the driver reads from HIR doc attributes whose expansion is the trusted exported `test`/`sim_test`, so forged doc attributes are ignored. Names starting with `-` are refused.

The manifest's root inputs are compared with what the source re-executes (`finish`), never substituted. Its choice tape is kind- and bound-checked before launch and consumed strictly at run time, so exhaustion, leftovers and kind mismatch fail. Its trace and outcome are only compared.

**Validation-to-launch race.** The source snapshot, which is in memory, both computes the identity and compiles. The internal compile stage refuses a changed snapshot through its key. The executable is copied into a private staging directory and its digest checked against the receipt, then those bytes run, and the sim child re-executes the same copy via `current_exe`. A concurrent publication can only cause a refusal.

**Strict identity.** `Build::compare` covers:
- source and paired library, with logical names, aliases and byte digests;
- compiler, target and profile;
- normalized options, including cfg, debug assertions and admission settings;
- ordered dependency artifact hashes, with std and macros;
- the driver digest and the format versions.

The configuration is compared against the current receipt. The table-driven tests, the same-length mtime-preserving edit, the dependency-directory, paired-library and cached-executable corruption cases, and relocation all exercise this. The sysroot and C runtime are trusted assumptions, as the decision states.

**Exact replay and fresh `--seed`.** The modes are mutually exclusive at parse time. In replay the driver sets no seed variable, and `selected_seed` takes the manifest seed. Shrinking and the determinism self-check are skipped on replay. Replay is compared against the original trace bytes and outcome, and the test compares the two replays with each other.

**Hostile tests.** `should_panic` cannot mask a mismatch, because the receipt is written only after every comparison in `finish` passes. A strict-tape assert fails before `finish`. Exit or abort leaves no receipt. See non-blocking item 4 for the residual hardening.

**Tests.** They are strong and mostly non-duplicative. They miss the parameterized-nesting case (finding 1). Their stdout-empty "no candidate launch" sentinel is adequate, since libtest always prints "running".

**Leftovers.** The README, lib docs, `main.rs` docs, design/03, conformance README and rules rows, and seed-hint relabels are updated. Findings 2 and 7 list what is undocumented or stale.

DONE
