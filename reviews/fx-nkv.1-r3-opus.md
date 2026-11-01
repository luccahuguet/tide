Model: Opus 5.5. Date: 2026-10-31. Round 3 (confirmation), candidate b2a4eb2a on dbb1270, read-only converge round.

# fx-nkv.1 round 3 (Opus): confirmation

**Verdict: clean.** Every finding from rounds 1 and 2 is resolved, and a fresh pass over the whole candidate found nothing blocking. Two nits and one out-of-scope note follow.

## Proof re-run

Candidate identity: `git diff -- . ':!.beads'` in the worktree hashes to b2a4eb2a…0fb700, the same as `candidate-r3.diff`; HEAD is dbb1270.

- `CARGO_BUILD_JOBS=4 CARGO_TARGET_DIR=…/tide-nkv1-work/review-target cargo test --workspace </dev/null`: exit 0, with no warnings. tide-std `entry.rs` ran 6 tests and all passed; the driver's suites also passed.
- `conformance/run.sh </dev/null`: exit 0 across 214 cases (190 PASS, 24 AWAITS CHECKER). `root_main_nested.rs`, `root_main_library.rs`, `admit_runtime_entry.rs`, `root_main_accept.rs`, `root_main_decoy.rs`, `root_main_unmarked.rs`, `root_main_plain.rs`, and `root_main_no_std.rs` pass. `root_main_called.rs` awaits FX0506, as before.
- Probe: I ran the driver once per mode on `root_main_library.rs`. `--crate-type=lib` reports `4:1 FX0520` (exit 1) and `--test` admits it (exit 0); see the out-of-scope note.

## Earlier findings, rechecked

- **r1 Opus B1 / Codex 2 (sinks wrote while formatting):** resolved. `__sink::format` builds the whole record with `fmt::format` before taking the lock. `records_are_formatted_before_they_are_written` tells the old and new behavior apart even if the child writes stderr before the parent closes it, because the `outer[x]` line differs either way.
- **r1 Codex 1 (off-entry minting):** resolved and now tested.
  - The fix: `crates.rs` `entry_expansion` returns an expansion only for an executable crate whose crate-root `fn main` an expansion generated. The `walk.rs` tide::main arm then requires `is_entry(expn)`.
  - Mutant check, by reasoning: drop the crate-root check and `inner::main` becomes the entry, so lines 7 and 19 of `root_main_nested.rs` flip. Drop the `CrateType::Executable` check and `root_main_library.rs` loses its FX0520 in both modes. Drop `is_entry` and both cases fail.
- **r1 Opus N1–N4:** resolved. `print!()` is rejected (tested at 3:5). The crate docs list what is not provided. The prelude's `Error` and `path` are shared through `root.rs`. The package is named `tide_std`.
- **r2 B1 (nested case was library-shaped):** resolved. `root_main_nested.rs` is now binary-shaped: the crate-root `#[tide::main]` is followed by `fn main() {}` at column 0, and the decoys come first. `root_main_library.rs` covers the executable-crate check.
- **r2 Nit 1 (formatting error dropped the record):** resolved. A formatting error now panics as Rust does, nothing is written, and the exit is 101. The test asserts this, and the `__sink` doc no longer cites SINK-5 for it.
- **r2 Nit 2 (reflow):** resolved. None of the changed Markdown lines exceeds 80 columns.
- **r2 Nit 3 (stale descriptions):** resolved.
  - `.agent-protocols.local.md` names tide-std and tide-macros. `AGENTS.md`'s local section matches it byte for byte, and `.agent-protocols.json`'s overlay sha256 equals the file's hash (f42c5af4…).
  - The `tide-macros` header states the crate-root executable condition.
- **r2 rebase note:** resolved. The candidate sits on dbb1270 and composes with fx-kw8.8's `entry_point`.
- **r2 out-of-scope note (FX0506):** filed as fx-kw8.14, which blocks fx-kw8.9.

## Fresh review

Spec fit:
- **03 §2.4:** SINK-1 (`()`, never fails), SINK-3 (format first, panics propagate), SINK-4 (whole records under the stream lock, flushed), and SINK-5 (closed stdout exits 141; other write errors are discarded) all hold.
- **Reference §6.2:** the forms of `main`, the `Error: {e:?}`/1 and panic/101 reporting through `Termination`, and the once-per-process guard all hold.
- **Reference §2.1:** the sink forms match Rust's arity.

Bead scope and acceptance:
- Production `crates/` has no `unimplemented!`.
- The stub shares every production item through `include!` and `capability!`.
- `policy.rs` and `pure.rs` remain the owners of admission and `Pure`.
- The std surface that `policy.rs` and `run.sh` target is recorded in `policy.rs`, `conformance/README.md`, and the bead.

Leftovers:
- The `_print` and `_eprint` allows were removed; nothing else referenced them.
- `__rt::root` no longer appears outside frozen spikes, research, and old reviews.
- `README.md` and `conformance/README.md` are current.

The tests protect what they claim to:
- ROOT-4 privacy, with E0451, E0603, and E0277 at exact positions.
- The second-entry panic.
- The 141 exit.
- Result and panic reporting.
- The attribute's misuse errors.

**Blocking findings:** none.

**Nits**
1. `crates/tide-admit/src/crates.rs:159-171` (`entry_expansion`): under `--test`, two crate-root `fn main` items come from expansions: the user's `#[tide::main]` one and the test harness's `AstPass` one. `find_map` picks the user's only because rustc appends the harness `main` after it. If that order changed, every binary would get FX0520 under `--test`. The corpus would catch that (`root_main_accept.rs` and `root_main_nested.rs` would fail), so it is not silent. Skipping `ExpnKind::AstPass` expansions in the filter would make the intent explicit. Optional.
2. `conformance/README.md:127-129`: the sentence "uses the trial's `macros` crate as is, and Tide's std's macros (`crates/tide-macros`)" chains two "and"s awkwardly. A rewrite would read better: "uses the trial's `macros` crate as is and Tide's std's macros (`crates/tide-macros`)", or split the sentence.

**Out of scope**
- `entry_expansion` treats any executable crate as having an entry, and a library compiled with `--test` is one. Under `--test`, a library's crate-root `#[tide::main] fn main` is therefore trusted, and the driver admits it (probe above).
  - `root_main_library.rs` passes only because `run.sh` merges the two modes' marks.
  - This is the same exposure as a binary under `--test`: the generated `main` is dead code that a test could call to mint a `Root`. fx-kw8.14 (FX0506) is meant to close it.
  - fx-kw8.14's scope names "the binary's `main`". It should cover any crate-root `main` that `#[tide::main]` generates, including a library's under `--test`. Otherwise fx-kw8.9 must also run the library-mode check on library packages before it builds their tests.
  - Recommend adding this as a comment on fx-kw8.14.
