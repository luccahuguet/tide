Model: claude-opus-5-5 · Effort: medium · Date: 2026-11-09

# fx-yg5.1 rebase re-check, round 4 (Opus, read-only)

Candidate: `9db728f` (on main `ecba95f`), compared with the approved `e581f25`+`f434d13` (base `b4f1302`) through `git range-diff b4f1302..f434d13 ecba95f..fx-yg5.1` and `git diff ecba95f fx-yg5.1`. I re-checked only what the rebase changed. I ran no builds; the coordinator is running both suites.

## Verdict

approve. F1 is a one-word, text-only fix and should go in before landing. It needs no re-check.

## Per-item status

1. **README.md std summary: OK.** Main's sentence listing `#[tide::main]`, `Root`, the sinks, `#[test]` and the task lifecycle is kept. So is its `std::io` paragraph. The new sentence "`std::collections` keeps Rust's API, with fixed-key `HashMap`/`HashSet` and `Rand`-seeded hashing" sits before "The crate docs list what it does not provide yet", and that is accurate. "sharing those definitions" with the stub stays true, because the stub gets collections through `root.rs`. One nit: line 170 is longer than the paragraph's wrap width (F2).
2. **Conformance stub crate doc (`conformance/rustc/std/src/lib.rs:1-18`): OK.** Main's text about `io/shared.rs` and `task.rs` is kept. The `with_hasher` exception is gone, which is correct because the stub's own `collections` module (and so its const `with_hasher`) is deleted and production's module comes in through `root.rs`. "The remaining stub methods use `unimplemented!()`, so it decides type and borrow verdicts only. Collections share production's module through `root.rs`." matches the code. The stub's local `collections` module is fully removed, with no leftover. It keeps `pub mod rand { capability!(Rand) }`, which the shared `RandomState::from_rand(&crate::rand::Rand)` needs.
3. **`crates/tide-admit/tests/build.rs`: OK.** All of main's test function names are unchanged and in order (diffed). The added lines are byte-identical to what `f434d13` added over `b4f1302` (`SAME_ADDITIONS`). The three collections tests now sit after `task_lifecycle_builds_and_runs_against_production_std` and before `checked_hello_builds_without_running_and_ignores_candidate_configuration`, and that test and its neighbours are intact.
4. **`crates/tide-std/src/lib.rs` crate doc, provided list: OK.** It lists main's task lifecycle bullet (`task.rs`), the collections bullet (round-1 wording, kept intact) and main's `std::io` core bullet. The `;`/`.` punctuation chains correctly. `collections` is removed from "Not provided yet" and `io` is correctly absent from it. The closing note that `root.rs` and `task.rs` share their definitions with the fixture still covers collections, through `root.rs`'s `pub mod collections;`.
5. **`crates/tide-std/src/lib.rs` crate doc, not-provided list: one regression (F1).** Round 1 changed "randomness" to "random draws". In resolving the second hunk the rebase took main's reworded sentence and dropped that change.
6. **Interactions in shared files: none found.**
   - `root.rs`: `pub mod collections;` sits beside main's unchanged items. The stub includes `root.rs` and also `io/shared.rs` (in `mod io`) and `task.rs` (in `mod task`), and neither of those names `crate::collections`, so no names clash.
   - `io/shared.rs:315` implements `From<std::collections::TryReserveError> for io::Error` against Rust's type. `collections.rs:11` re-exports Rust's `TryReserveError`, and `try_reserve` returns `super::TryReserveError`. Tide's `std::collections::TryReserveError` is therefore the same type, and `?` into `io::Error` still works.
   - `policy.rs`: main replaced `core::io::*`/`alloc::io::*` with Tide-owned io rules. The branch's two `DefaultHasher` allows are separate entries and do not overlap them. Neither collections nor its cases use `core::io`/`alloc::io` paths.
   - `walk.rs`: main moved `Role::TideStd if IO_ERROR_NEW…` ahead of `Role::TideStd => pure::needs(&key)`. `tide_std::collections::*` keys never match `IO_ERROR_NEW`, so they reach `pure::needs` as before.
   - The new sink lint, FX0508 (println from a library target), cannot fire on `collections_executable.rs`. `run.sh` makes it `kind=bin` because `^fn main` is at line 235, and `--test` builds count as executable (`sinks.rs:library_target`). The case has no `dbg!` (FX0507) and no channels. The diagnostic format `FX0520: …` that `collections_cannot_draw_ambient_keys_through_a_generic_hasher` asserts is unchanged in `diag.rs`, and FX0901 is still an error.

## New findings

- **F1 (minor, text-only; should fix before landing):** `crates/tide-std/src/lib.rs:24-26` says "Not provided yet: … other capability operations and attenuation of reference 6.1 (files, network, time, randomness, …)". The branch does provide one `Rand` operation, `collections::hash_map::RandomState::from_rand(&Rand)`, so "randomness" lists something that exists as missing. Round 1 had fixed this as "random draws", and the conflict resolution lost it. The `rand` module's doc at `lib.rs:69-70` already says "It has no draw methods yet". Fix: change "randomness" to "random draws" on line 26.
- **F2 (nit, non-blocking):** `README.md:170` ("`Rand`-seeded hashing. The crate docs list what it does not provide yet. The conformance runner injects") is about 100 columns wide, where the surrounding paragraph wraps at about 80. Re-wrapping it is optional.

Out of scope, noted only: `io/shared.rs` gives Tide's `Read`/`Write` to `&[u8]`, `Vec<u8>` and `Cursor`, but not to `VecDeque<u8>`, which Rust's std gives. That gap belongs to fx-ecb's io surface, and the rebase did not introduce it.

DONE
