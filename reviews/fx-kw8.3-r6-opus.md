Model: Opus 5.5. Date: 2026-10-22. Round 6 (converge, fixing), candidate 8f425419 (uncommitted on ce0f0cc). Clean.

# fx-kw8.3 round 6 (Opus): tide-admit owns every rustc invocation

This round found no defects and no needless complexity in the diff, so I changed nothing. Both checks pass on the candidate as it stands. I did not touch `reviews/` or `.beads/`, and I made no commits and no bead writes.

**Fixed:** nothing.

**New-surface item: `pub fn run()` in `/home/lucca/pjs/lab/tide/crates/tide-admit/src/invocation.rs:514` stays.**
- It is the module's only entry point, and `main()` calls it.
- `invocation` is a private module of a binary, so `pub` only reaches within the crate. That matches the crate's convention (`walk::check_crate`, `diag::Diagnostics` and the `pure.rs` items).
- `main.rs` has to call it, so it can't be narrower than `pub(crate)`, which means the same thing here.
- The other match the script reported is a test string literal, as you said.

**What I checked:**
- **Code read against the spec:** I read the whole diff and both new files against ADMIT-2 and RUN-2 (`design/03-authority-concurrency.md:563-636`) and against the pinned `FileLoader` trait and `ParseSess::new`. The diag codes match their sources: FX0101 is D6, FX0911 is `#[path]` in `reference.md`, and FX0915 is D13.
- **Probes on the built driver** (scratch dir `/tmp/tmp.mziwCx87uK`):
  - A relative entry with a `sub/mod.rs` module is admitted.
  - A cache hit replays the warnings and copies the `.rmeta`.
  - The default `--out-dir` inside the package exits 2.
  - A `bin` crate gets its verdicts.
  - `--tide-grant unsafe` written with a space is rejected.

**Left alone, with reasons:**
- **`fs::create_dir_all(&invocation.out)` at `invocation.rs:589` is redundant,** because line 642 creates the cache directory beneath it. But `output_and_trusted_inputs_cannot_be_candidate_owned` relies on it: it makes a symlink inside `out/`, and that directory only exists because an earlier failing run created it. Removing the line means editing the test, which saves nothing.
- **Recording `TIDE_COMPILER_ID` in the cache key is mostly covered already by hashing the driver binary.** I kept it because it states the intent openly and the cache test uses it to find the host triple.
- **Kept as earlier rounds decided:** `Sources` stays `Clone` with `Arc` fields, the two-stage self re-exec stays, and the three near-identical rustc fixture builds in the tests stay.
- **Out of scope, not fixed:**
  - A non-UTF-8 entry file, or an entry symlink that points outside, exits 2 ("candidate source is not admitted") rather than 1 (a rejection). Every RUN-2 assertion still holds, since nothing is read or disclosed; only the exit-code category is debatable.
  - Still open from earlier rounds:
    - FX0915 only looks one level up from `src/`.
    - `#[unsafe(path=..)]` inside macro arguments gets FX0520, not FX0911.
    - `fingerprint` follows symlinks in trusted directories with no cycle guard.
    - An unreadable source subdirectory exits 2.
    - `destination()` fails on an out-dir whose missing part contains `..`.
    - `rustc-ice` files land in the current directory.

**Checks** (unchanged candidate, Linux x86_64, pinned nightly-2026-08-09, run one at a time):
- `timeout 1200 cargo test --workspace < /dev/null`: exit 0, 20 tests passed (2 + 5 + 13), no warnings.
- `timeout 1800 conformance/run.sh < /dev/null`: exit 0, 180 PASS, 28 AWAITS CHECKER, 0 FAIL.
- `git status` shows the same files as at the start, with no stray `rustc-ice` files in the repo. The probe files and logs are in `/tmp/tmp.mziwCx87uK`, outside the repository.
