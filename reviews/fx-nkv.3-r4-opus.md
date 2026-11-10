Model: claude-opus-5-5 · Effort: medium · Date: 2026-11-09

# fx-nkv.3 round 4: rebase re-check (Opus, read-only)

Candidate: branch `fx-nkv.3` at `c485d87` on main `efa21df` (rebased from `102a3e2` + `a29875a` on `07e104b`).
Scope: only what the rebase changed. No cargo builds or probes; the 1-minute load was 12.48, so I ran no single-file probe and checked everything statically. The coordinator's two suite runs are the runtime proof.

## Verdict

approve

## Per-item status

1. **`git range-diff 07e104b..a29875a efa21df..fx-nkv.3`**: OK. Both commits differ only in context. The build.rs hunk's anchor moved from `task_lifecycle_…` to `collections_cannot_draw_ambient_keys_through_a_generic_hasher`, and the crate-docs context took main's `collections` bullet and its "randomness" → "random draws" rewording. The bead's own `+`/`-` lines are byte-identical.

2. **`crates/tide-admit/tests/build.rs` conflict resolution**: OK. main `efa21df` has 13 `#[test]`s and the candidate has 14. The only net change against main is the new `task_failures_and_cancellation_build_and_run_against_production_std`, which matches `a29875a`'s version exactly. It sits after the three collections tests, and `task_lifecycle_…` is unchanged. All four collections tests (`collections_execute_…`, `collections_disjoint_lookups_enforce_safety` with both packages, `collections_cannot_draw_ambient_keys_…`) are intact, with no truncated bodies or duplicated tests. Package names don't collide: `task-cancellation` is distinct from the collections and `ambient-collection-*` names.

3. **README.md (auto-merged)**: OK. The task-runtime paragraph (bead) comes first, then the io core paragraph, then main's `std::collections` sentence, which is followed by "The crate docs list what it does not provide yet." All three describe the current state and none contradicts another. Main's autorun/coordinator line elsewhere in the README doesn't touch this passage.

4. **`crates/tide-std/src/lib.rs` crate docs (auto-merged)**: OK. The provided list has tasks (with cancel, TASK-2, checkpoint/yield_now), root.rs re-exports, `collections`, and io core. The "Not provided yet" list has lost both "task cancellation, multi-failure selection" (this bead) and "collections" (fx-yg5.1), and keeps "random draws" (fx-yg5.1). The two sets don't overlap. The `Rand` doc comment (main) is accurate.

5. **Interaction with FX0507/FX0508/FX0607/FX0608 (`walk/sinks.rs`)**: no interaction.
   - The lints run only on the admitted crate's HIR (`sink_expansion`, `lint_sink_expr`, `LockPrints`). `task.rs`'s internal `mpsc::channel`/`sync_channel` calls are inside tide-std, which is not walked. Within tide-std, `std::sync::mpsc` resolves to Rust's std, not `tide_std::sync::mpsc::channel`, so FX0608's key could not match them anyway.
   - `conformance/cases/tasks/task_cancellation.rs` has a top-level `fn main` (run.sh `kind=bin`; driver `--crate-type=bin` and `--test`, both Executable per `library_target`). So its single `println!` in `main` cannot trigger FX0508. The case has no `dbg!`, which rules out FX0507 under any profile. It has no `Shared::with*` closure, which rules out FX0607, and no `channel` call, which rules out FX0608. It carries no `//~` marks, so the runner expects driver output with no diagnostics, and the fixture gives it none.
   - The build.rs test runs the same fixture as an executable under debug and release. In release FX0507 could fire only on `dbg!`, which the fixture doesn't use. It asserts empty stderr, and since the driver's warnings go to the driver's stderr, not the binary's, that assert is unaffected.

6. **Collections or lint changes in shared files**: no conflict.
   - The bead's net diff against main (`git diff efa21df..fx-nkv.3 --stat`) doesn't touch `root.rs`, `collections.rs`, `walk.rs`, `walk/sinks.rs`, `policy.rs`, `diag.rs`, `run.sh` or `rules.md`. Main's changes to those files are taken unchanged.
   - Conformance stub `lib.rs`: the bead's only changes are the comment rewording and the removal of the stub-local `task::Cancelled`, which `task.rs` now defines. Main's large stub reduction (collections moved into shared `root.rs`) is elsewhere. The stub's `time` module (`use crate::task::Cancelled;`, line 141) and its `sync::mpsc` `Cancelled` variant still resolve against the production `Cancelled` included via `task.rs`.
   - `policy.rs` only denies `core::task::*` and `alloc::task::*`, not `tide_std::task`. Main added two policy lines and none of them are task paths.
   - `design/03-authority-concurrency.md`: main's GLOBAL-5 sentence and CHANNEL-1 label sit in other sections from the bead's added `yield_now` signature line (§ Tasks API block); they merged cleanly.

## New findings

None blocking.

- (nit, pre-existing, not caused by the rebase; no action needed for landing) `crates/tide-std/src/lib.rs:5` still says "This partial runtime (beads `fx-nkv.1`, `fx-nkv.2`)" even though fx-nkv.3 and fx-yg5.1 content now sits in the same list. Both sides of the merge already had this stale wording.

DONE
