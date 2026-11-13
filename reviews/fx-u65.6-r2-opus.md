Model: claude-opus-5-5 · Date: 2026-11-13

# fx-u65.6 Opus review, round 2 (read-only)

Candidate: worktree `.worktrees/fx-u65.6`, branch `fx-u65.6`, HEAD `3f3342c`, reviewed as `git diff b3b19dc..3f3342c`. Nothing in the worktree was edited, staged or committed, and br was used only as `br --no-db show`. `trials/e1-checked/` was not opened.

## Verdict

request_changes

The FX0802, FX0803, FX0910–FX0915 work is sound and agrees with the reference, the binding FX0914 decision (bead comment 2026-11-11; research/27 C12 and its table row 88, "or the module itself for a module import") and the FX0915 package-level alternative. One finding blocks: FX0902 rejects legal code. A `// SAFETY:` comment sits directly above the line where the unsafe block starts, but the block does not open that line and its enclosing statement starts higher up. Match arms, struct-literal fields, method-chain lines and const/static items all hit this. FX0902 is an error, so the whole crate is rejected.

## Findings

1. **blocking — FX0902 false positives when the block does not open its line and is not inside a statement whose first line carries the comment.**
   Evidence: `crates/tide-admit/src/walk/modules.rs:135-136`. `documented` returns false whenever the text before `span.lo()` on the block's own line is not blank, so `Some(p) => unsafe { *p },` and `b: unsafe { *p },` never have their own comment checked. The parent walk at `modules.rs:153-157` accepts only a `Stmt` or `Expr` whose span opens a line, and it stops at the first `Block` or `Item`. That misses arms (`Arm`), struct fields (`ExprField`), and `const`/`static` initialisers, whose parent is an `Item`. The contract (`conformance/rules.md` REF-0902: "directly above the block or its statement"; `conformance/README.md`: "a `// SAFETY:` line in the comment lines directly above the block or its statement") and the diagnostic's own text are satisfied by a comment on the line directly above the block, yet the driver rejects it. Probed (lib, `--tide-grant=unsafe`):
   - match arm: comment line 3, `Some(p) => unsafe { *p },` line 4 → `4:20: error: FX0902`, exit 1;
   - struct literal: comment above `b: unsafe { *p },` → `6:12: error: FX0902`, exit 1;
   - `// SAFETY:` above `pub const X: u8 = unsafe { seven() };` → `3:19: error: FX0902` (my probe also hit an unrelated E0015 because `seven` is not `const`; the FX0902 does not depend on it);
   - controls that pass: a block that opens its own line inside a multi-line call argument list exits 0. Comment above `let y = unsafe {..} + unsafe {..};` reports no FX0902 for either block. A blank line between the comment and the block is still FX0902.

   The corpus misses this because `modules/safety_accept.rs` covers only a block that opens its line and a `let` statement.
   **Fix wanted:** start the comment search at the line that contains the block's start, whatever precedes the block on that line. Only the lines above it count, so the same-line trailing comment stays rejected; `safety_reject.rs` `trailing` and `detached` keep their result. Treat an enclosing `const`/`static` item (or any item) like a statement: check `documented(item.span)` before breaking. Add to `safety_accept.rs` a match-arm block, a struct-field block, and a commented `static`/`const` initialiser. Keep a reject twin with no comment.

2. **non-blocking — the edit to the `unsafe_granted.rs` header garbles its sentence.** Evidence: `conformance/cases/authority/unsafe_granted.rs:3-6`. Removing "and `unsafe impl`;" leaves "with a `// SAFETY:` comment on each `unsafe { .. }` block and `asm!`, `global_asm!`, …", which now reads as if `asm!` needs a comment. The case still contains `unsafe impl`s (lines 32, 37), which need no comment. **Fix:** "with a `// SAFETY:` comment on each `unsafe { .. }` block (`unsafe impl` needs none), `asm!`, `global_asm!`, …".

3. **non-blocking (ownership note, no change required) — the unused-trait-import half of FX0913 re-implements rustc's `check_unused_traits`.** Evidence: `crates/tide-admit/src/walk/modules.rs:56-77`. It has the same body-owner union of `used_trait_imports`, the same `maybe_unused_trait_imports` iteration, and the same public and dummy-span skips. It is needed because `-Aunused-imports` (`invocation.rs:265`) makes FX0913 independent of `#[allow]`, which `unused_trait_reject.rs` proves. It is small and its gate is now correct (Codex r1 finding 3; my `unused_gate` probe shows a walk-level FX0902 does not suppress it). If rustc changes that function, this copy must follow. A one-line comment naming the mirrored rustc function would keep that visible.

### Checked, no finding

- **FX0802.** Trait and trait-impl methods are caught both by the attribute (`std_attr_made`) and by generated helper calls. That covers a contract attribute that `#[invariants]` consumes on a required method: probe `hidden_required` gives FX0802 at the fn line. `#[invariants]` alone stays legal (`entry`/`const_entry` are excluded). Duplicate reports land on the same callsite line.
- **FX0803.** It flags when another parameter's type contains the region, the output does (opaque types: only non-bivariant args), a bound does (`T: 'v` and `'x: 'v` exempt, `'v: 'x` flagged), or an implied bound has `'v` outliving another region. That matches §10.1. Pattern-copied scalars (r1 finding 1) and alias entry snapshots (r1 finding 2) are fixed and have corpus cases. Probe: `&'static mut` and a distinct-lifetime `&mut` with `ensures` both accept.
- **FX0910.** E0583 is mapped through the existing `labels.rs` RULES table, with no parallel table. Inactive and inline modules are accepted, and the in-snapshot module-file driver test exists.
- **FX0911.** Behaviour is unchanged, and the `path_accept`/`path_reject` proof is new.
- **FX0912.** The AST check skips expansion. Existing `extern crate` fixtures gained marks and IDs (`admit_no_std`, `root_main_no_std`, `admit_crate_identity`), and the build test's suite moved to `use tide_candidate as solution`. All four are justified.
- **FX0913.** It reads the resolver's buffered lint, excludes expansions and synthesis insertions, and stays independent of lint levels. The edits to existing cases only remove imports that really are unused (`provenance_matrix_*`) or move them into test scope (`tcp_deadlock`, `limit_deadlock`, `#[cfg(test)]` in `channel_deadlock`). That is consistent with the reference teaching `#[cfg(test)] mod tests`. Release erasure is `cfg!(debug_assertions)` in an expression, so clause-only imports stay used in release.
- **FX0914.** Edges come only from `use` items (`ListStem` skipped), public uses and globs included. A named import targets its defining module, through re-exports. A glob or module import targets that module, as C12's table row says. Local targets only, no edges from `mod` or body paths. My `cycle_modimport` probe (module import plus a reverse body path) is accepted.
- **FX0915.** `tests/invocation.rs:392-417` asserts accepted exit 0 with no FX0915, rejected exit 1, and an unlocated `error: FX0915:` prefix (error severity), and it keeps the build-script canary. This meets the Acceptance's invocation-test alternative.
- **Existing driver tests that gain FX0902.** Several tests that assert FX0901 (`build.rs:574-585`, `build.rs:1890/1902`, `invocation.rs:352`) use `contains`, so the extra FX0902 does not break them.
- **Complexity.** About 400 lines of Rust in `contracts.rs` and `modules.rs`. The FX0803 region analysis is the bulk, and the spec's lifetime rule requires it. I saw no removable abstraction.

### Not checked

- Full `cargo test --workspace` and `conformance/run.sh`. These are reserved for the coordinator. Leftovers elsewhere in the corpus (unused imports, undocumented unsafe blocks, `extern crate`) surface only there.
- The `fx-u65.7` interaction.
- Runtime execution of the probes, which emit metadata only.
- Whether `target/debug/tide-admit` (built 22:59:49 -03) reflects the final amend (commit 23:02:11 -03). The author's fix-round driver and conformance runs rebuilt it, so I reused it as the candidate's binary.

## Probes

Environment: Linux x86_64. Binary: the candidate's existing `target/debug/tide-admit`. Std: `conformance/target/admit/debug/libtide_std_stub.rlib`, with `--tide-dep-dir` set to each rlib/so directory under `conformance/target/admit/debug`. Flags: `--tide-grant=unsafe --crate-type=lib --edition 2024 --emit metadata --error-format=short`. Sources are under `/tmp/claude-1000/-home-lucca-pjs-lab-tide--worktrees-fx-u65-6/6b301c09-3e49-4995-9f14-e948d27b4c37/scratchpad/pkg/`, outside the worktree, and the harness script is `scratchpad/p/run.sh`. Each batch went through the queue:

    (ulimit -c 0; CARGO_BUILD_JOBS=4 timeout 300 bash /home/lucca/.cache/eon-agents/eon-overseer/heavy.sh nice -n 19 bash <scratchpad>/p/run.sh <files> </dev/null)

| Probe | Result |
| --- | --- |
| first batch (all six files) | exit 2 for each: the output dir lay inside the source dir ("output and cache must be outside the candidate package"); setup only, no evidence; rerun with a separate out dir |
| `safety_arm.rs` | FX0902 at 4:20, exit 1 (finding 1) |
| `safety_field.rs` | FX0902 at 6:12, exit 1 (finding 1) |
| `safety_const.rs` | FX0902 at 3:19 plus an unrelated E0015 from the probe, exit 1 (finding 1) |
| `safety_arg.rs` (block opens its own line in a multi-line call) | exit 0 |
| `safety_tail.rs` | blank-line-detached block: FX0902 (correct). Comment above a `let` with two blocks: no FX0902. Also a parse error from my malformed tail expression (not evidence) |
| `unused_gate.rs` (unused trait import plus an undocumented unsafe block) | FX0913 at 2:5 and FX0902 at 4:32: a walk error does not suppress trait FX0913 |
| `unused_underscore.rs` (`use m::T as _;` unused) | FX0913 at 2:5 |
| `cycle_modimport.rs` (a: `use crate::b;`, b: body path into a, c: `use crate::a::A`) | exit 0 |
| `hidden_required.rs` (`#[invariants] #[requires]` on a required trait method) | FX0802 at 4:5 |
| `static_mut.rs` (`ensures` naming a `&mut`/`&'static mut` param with a distinct-lifetime other param) | exit 0 |

All probes finished by 02:15Z.

DONE
