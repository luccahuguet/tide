Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-03

## Verdict

**approve — moderate-high confidence.** All eight round-1 findings are correctly addressed by source inspection. I verified no remaining blocking defect in the fixes.

## Scope

Read-only re-check of `0141db4..6535df7`, focused on the round-1 findings and fixes through Opus r7. Read the bead and comments, review history, AGENTS.md, GOVERNANCE.md, relevant design sections, implementation, and tests.

HEAD and the supplied diff SHA-256 match. The worktree remained clean; no files were edited.

## Blocking

None verified.

## Non-blocking

None verified.

## Reasoning

The round-1 findings are resolved:

| Finding | Re-check |
|---|---|
| 1. Wrappers discard payload authority | The named-kinds branch also traverses type arguments, preserving both `sync` and `net` for `Arc<Net>` ([interface.rs:283](/home/lucca/pjs/lab/tide/.worktrees/fx-kw8.10/crates/tide-admit/src/interface.rs:283)). |
| 2. Granted impl selections disappear | Obligations record granted selections without suppressing toolchain-expanded selections; the backstop merges its holes ([walk.rs:1267](/home/lucca/pjs/lab/tide/.worktrees/fx-kw8.10/crates/tide-admit/src/walk.rs:1267), [mono.rs:103](/home/lucca/pjs/lab/tide/.worktrees/fx-kw8.10/crates/tide-admit/src/walk/mono.rs:103)). |
| 3. Depth failure becomes empty authority | A `Pure` depth failure now produces `opaque` before extraction can restart below the limit ([interface.rs:254](/home/lucca/pjs/lab/tide/.worktrees/fx-kw8.10/crates/tide-admit/src/interface.rs:254)). |
| 4. Nested declarations get the outer holder | Holder selection chooses the inner item while retaining closure collapsing ([walk.rs:377](/home/lucca/pjs/lab/tide/.worktrees/fx-kw8.10/crates/tide-admit/src/walk.rs:377)). |
| 5. Granted macros collapse at crate root | Post-walk uses locate their innermost source holder ([walk.rs:419](/home/lucca/pjs/lab/tide/.worktrees/fx-kw8.10/crates/tide-admit/src/walk.rs:419)). |
| 6. Early rejection preserves stale artifacts | Invalidation follows output-location validation and precedes source resolution and `build.rs` rejection ([invocation.rs:633](/home/lucca/pjs/lab/tide/.worktrees/fx-kw8.10/crates/tide-admit/src/invocation.rs:633)). |
| 7. Unsupported std silently succeeds | Missing Tide identity fails explicitly, with the documented Rust-std exception ([interface.rs:64](/home/lucca/pjs/lab/tide/.worktrees/fx-kw8.10/crates/tide-admit/src/interface.rs:64)). |
| 8. Aliases and external re-exports disappear | Aliases are extracted; external re-exports include target kind in their keys, preserving separate type/value entries ([interface.rs:80](/home/lucca/pjs/lab/tide/.worktrees/fx-kw8.10/crates/tide-admit/src/interface.rs:80)). |

The later Opus fixes are consistent with the recorded decisions: generic suppression checks instantiators instead of unrelated holders; deferred merging avoids order dependence; crate-root fallback bypasses generic queries; the root index and memo share the same identity. Recursion expands one growth step and explicitly reports further growth as `opaque`, including finite cases. The docs acknowledge these limits and make no exercised-authority or proof-coverage claim.

The integration assertions cover nested paths, pure items, new unsafe holes, generic attribution beside unrelated holders, crate-root fallback, and rejection cleanup. The source diff whitespace check passed.

Not checked: executable results. Both `cargo test --workspace --locked --offline` and `conformance/run.sh` stopped before testing because their Cargo build locks were read-only.  
Risk: this approval rests on static re-checking; I did not independently reproduce the recorded r7 test passes.