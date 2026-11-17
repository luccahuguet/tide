Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate d4a98d5

# fx-qlek.9 r2: Opus review (converge round, reports only)

Scope: `git diff 23d30f0...d4a98d5`. Its SHA-256, which I recomputed, is
93d780aa37f3e42e695750791da6d9066e843b8f01e8928e6dcfb05c48d37de5 and matches the brief. 15 files. I read
the whole diff and compared it line by line with `git show 23d30f0:demo/run.sh`. I also read the bead's
contract and tide-lead's decisions, GOVERNANCE.md's Merge Rule, AGENTS.md, and tide-std's `replay.rs`,
`codec.rs` and the sink encoding in `task/scheduler.rs`. Other files I read: tide-admit's `main.rs`
module includes and its `build.rs` rpath and sysroot code, the builder's report.md, and the logs
stable-outer-run.log and workspace-suite-r2.log. I ran no builds.

## Verdict

**approve**. Confidence: high.

## Blocking findings

None.

## Non-blocking findings

1. **A relative or configured target dir sends the workspace test to a second target**
   (`crates/tide-demo/src/lib.rs:146-148`).
   - **Relative `CARGO_TARGET_DIR`.** `cargo test` runs test binaries with their package root as the
     working directory. So `CARGO_TARGET_DIR=tgt cargo test --workspace` builds the outer suite into
     `<repo>/tgt`. The demo test, though, resolves the same value to `crates/tide-demo/tgt`. There it
     rebuilds the driver and std from scratch, which takes minutes and gigabytes. It also leaves a
     directory in the source tree that `.gitignore` does not cover (`.gitignore` ignores only `target`
     and `debug`).
   - **`--target-dir` or `build.target-dir`.** A user who sets the target dir with this flag or this
     Cargo config key gets a second full build into `<repo>/target`.
   - **Effect.** Correctness is unaffected; the cost is time, disk, and a stray directory.
     `cargo run -p tide-demo` keeps the user's working directory, so it matches Cargo there. The
     agents' fleet always sets an absolute `CARGO_TARGET_DIR`, so this never shows on the fleet.
   - **Minimum fix, if wanted.** Give `Demo` an optional target, and have `tests/demo.rs` pass the
     outer build's directory, for example the parent of `env!("CARGO_TARGET_TMPDIR")`.

2. **The demo now requires the `rustup` binary, and the planned no-rustup container cannot provide
   it** (`lib.rs:249-257`).
   - **Why it is acceptable here.** Round 1's fix made the inner builds run `rustup run <channel>`.
     That correctly stops a `cargo +stable` or `RUSTUP_TOOLCHAIN` outer toolchain from reaching the
     inner builds. demo/README.md already lists rustup as a requirement, so this candidate is
     consistent with its own docs.
   - **The downstream problem.** fx-qlek.2 is blocked by this bead. Its Acceptance runs the demo in a
     container installed from fx-qlek.1's flake. fx-qlek.1 requires "no Rust, no rustup" and "no step
     reads ~/.rustup". In that container the demo would exit 2 with "cannot run rustup". run.sh's
     plain `env -u RUSTUP_TOOLCHAIN cargo` would have used the Nix shell's cargo.
   - **Recommendation.** Leave this for when the flake exists (AP-MINIMAL: no build for forecasts).
     The coordinator should add a note to fx-qlek.2 saying two things:
     - its Acceptance still names `demo/run.sh`;
     - the demo will need a toolchain path for when rustup is absent, such as plain `cargo` when
       `rustup` is not on PATH.

3. **The outcome check depends on the harness's internal outcome text** (`lib.rs:41`).
   - `LOST_HITS` copies how tide-std's harness writes a panic outcome: the Debug form of an
     `Option<String>`, `panic: Some("…`.
   - If that text changes, the check fails closed: step 1 reports `Other` and exits 1, and the
     workspace suite catches it at once. So it cannot pass a broken demo.
   - It is also a new check that run.sh never had. A one-line comment naming where the harness formats
     outcomes would help the next person who changes it.

4. **The seed-set line is hard-coded** (`lib.rs:807-809`). It prints "seeds 0 to 15" even though
   `seed_set` has just read `cases` from the manifest. run.sh hard-coded it too. Printing it from
   `cases` would keep it true if the suite's seed count changes. Nit.

5. **Some exit codes are classified as step failures** (`lib.rs:528-565`). An I/O error while reading
   `out/*/target/sim`, such as a permissions problem, comes back as a step failure (exit 1), not a
   setup error (exit 2). This fails closed either way. Nit.

6. **The failure message names a log the test has already deleted** (`tests/demo.rs:24-28` with
   `lib.rs:84`). When a case fails, the `Work` guard removes the work dir, so the "(log: …)" path is
   gone. The log's contents are printed just before the message, so nothing is lost. Nit.

7. **The TOML parsing is hand-rolled** (`lib.rs:224-245`). `pinned_toolchain` would reject a valid
   `channel = "…" # comment` line, with exit 2 (fails closed). The current file parses. Nit.

8. **The AGENTS overlay no longer lists every workspace crate.** The overlay's Verification paragraph
   (`.agent-protocols.local.md`, rendered into AGENTS.md) lists the workspace crates, and `tide-demo`
   is now one of them. That file has a single holder, so it is out of this bead's scope. Fold the
   change into the next overlay edit.

## Reasoning

**Checks compared with run.sh.** I mapped every check run.sh made:

- **Step 0.** The Rust version is stricter. A missing pinned toolchain is now a setup error. run.sh
  only printed the toolchain inside an unchecked command substitution.
- **Step 1.** Stricter. It still requires exit 101 and a summary line, and it still requires the
  harness's failure reports to match the failing seeds one for one, each with its first cause ending
  in `failed: lost hits`. It also requires these:
  - a validated manifest for each seed 0..cases, exactly once each;
  - each recorded outcome to be either passed or the lost-hits panic;
  - "not every seed failed", read from the recordings instead of from a parsed `passed` count.
- **Step 2.**
  - **Same as run.sh:** exit 101 and exactly one manifest; the first cause line is lost hits; each
    replay exits 101 and prints the `replay reproduced: test=… seed=… ` marker; each replay writes
    exactly one new manifest; the traces are byte-identical; hits < views comes from the sink output.
  - **New:**
    - each manifest passes `Manifest::validate`, which checks the trace and input digests;
    - the exported `trace.ndjson` equals the manifest's own trace;
    - the recording's seed, test, and outcome match.
  - **Sink output source.** The sink output is read from `bytes`, which the scheduler writes next to
    the lossy `message` (`scheduler.rs:180-186`). So it is exact.
- **Step 3.** Stricter. Every recording must have passed and the seed set must be complete, and the
  run on the seed alone must have recorded a pass.
- **Step 4.** Stricter. `+ metrics: net` must sit inside the `~ hits::serve (function) widened` block,
  where run.sh only grepped for both lines anywhere.
- **The one check that went away.** run.sh, under `set -e`, implicitly required the patch to contain
  a `±pub fn` line. That is not the property being shown; the trust-diff check covers it.

I found no path where the demo passes while the property it shows is broken.

**Exit codes.**
- `main` maps `Setup` to 2 and `Step` to 1, and `cargo run` passes the child's code through.
- I found no reachable panic. `seed()`'s `unwrap` comes after `validate`, which checks for a canonical
  decimal u64, and `digest` is 64 hex characters, so `[..16]` is safe.
- A driver killed by a signal fails every status check, because `code()` returns `None`.

**Races and leftovers in the workspace test.** The three cases run concurrently. Their work dirs are
unique to each process and case, and a drop guard removes them on pass, fail, or panic.

The cases share the outer target. I checked whether one case's inner `cargo build` could relink
`target/debug/tide-admit` or `libtide_std.rlib` while another case is using it. Cargo relinks
uplifted files even when the unit is fresh, but it skips the relink when the destination is already
the same file. So:
- only the first inner build replaces the copy that the outer workspace build put there;
- that happens while it holds the build lock, before any case runs the driver;
- later builds leave the file alone.

I found no race.

**Toolchain and environment leaks.**
- `rustup run <pinned channel>` with `RUSTUP_TOOLCHAIN` removed fixes Codex's round-1 finding.
- tide-admit's `build.rs` bakes in `TIDE_SYSROOT` and an rpath from the pinned rustc. The driver also
  clears its environment and re-executes itself (`invocation.rs:1130-1133`). So inherited
  `RUSTUP_TOOLCHAIN` and `LD_LIBRARY_PATH` from `cargo run` or `cargo test` cannot reach it.
- The builder's stable-outer run printed the pinned nightly and exited 0.
- An inherited `RUSTC` or `build.rustc` would still override the inner cargo's choice of compiler,
  exactly as with run.sh. That is not a regression.

**Docs and a new user.**
- **run.sh references.** Outside the excluded records (trials/e3-pilot, reviews/, archive/,
  research/, .beads/), no doc names `demo/run.sh`. E3's protocol says "or its replacement".
- **The guide's line range.** The owner guide's `lib.rs:659-780` range matches `record_and_replay`,
  and the 06-replay walkthrough describes the code correctly. Dropping `sha256sum` from the
  requirements is right.
- **Clean-checkout start.** From a clean checkout, `cargo run -p tide-demo` goes like this:
  - the rustup proxy installs the pinned toolchain and its components from `rust-toolchain.toml`;
  - the inner `rustup run` then finds that toolchain;
  - the demo crate has no dependencies, and Cargo.lock adds only its own entry.

**Ownership and complexity.**
- **Shared wire modules.** The crate includes tide-std's `codec.rs`, `json.rs` and `replay.rs` with
  `#[path]`. That is exactly how tide-admit already consumes them (`tide-admit/src/main.rs:88-98`,
  including the `use std::collections;` shim). So it shares the one owner rather than duplicating it.
- **Lenient decoder.** The small `decodable!` macro is a lenient twin of `replay.rs`'s `object!`.
  The difference is that it ignores unknown fields, which Cargo's and the trace's open-ended records
  need.
- **Size.** The 942 lines replace 170 lines of Bash, but most of the added size is typed checks that
  run.sh did not make. I saw no speculative abstraction. `Demo`'s public fields exist only so the two
  negative cases can inject revisions.

**Acceptance.**
1. **`cargo run -p tide-demo` exits 0.**
   - On c051ccf, a clean-tree run exited 0.
   - On the d4a98d5 tree before its docs-only line-range edit, a stable-outer run exited 0.
   - `the_demo_passes` in the d4a98d5 workspace suite runs the same code path.
2. **Both negative cases.** They are standing tests, and they pass in workspace-suite-r2.log (demo
   3/3, 14.5 s).
3. **run.sh is gone** and no doc in scope names it.
4. **Merge Rule.** Codex is clean; this is the Opus round.

DONE
