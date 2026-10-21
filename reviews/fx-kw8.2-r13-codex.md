Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-21.

## Verdict

**approve — high confidence.** The prior finding is resolved. No false rejection or other defect found in this diff.

## Scope

Read-only round-13 re-check of `5295926..2a82549 -- crates conformance`, against AGENTS.md, GOVERNANCE.md, both prior reports, ROOT-4, surrounding code, and pinned rustc sources.

HEAD remains `2a82549290f147bc5908691826b9e3eb312587af`. No files changed; worktree clean.

## Blocking

None.

## Non-blocking

None.

## Reasoning

- [raw_place](/home/lucca/pjs/lab/tide-fx-kw8.2/crates/tide-admit/src/walk.rs:524) now checks adjustments on each projection’s base before climbing. Any builtin or overloaded dereference stops traversal before the raw-borrow or assignment exemption can suppress FX0517.
- Existing-driver probes confirm **18 rejecting cases**, including `*const Box<[Fs; 1]>`, indexed writes through `*mut &mut [Fs]`, overloaded `Deref`/`DerefMut`, and nested projection variants.
- **21 accepting controls** remain free of admission errors: direct array, slice, and field raw borrows and stores, nested projections, generic forms, and implicit dereferences whose loaded types satisfy `Pure`.
- All **23 expected diagnostics** in [root_zeroed.rs](/home/lucca/pjs/lab/tide-fx-kw8.2/conformance/cases/authority/root_zeroed.rs:87) match exactly. Fixture and probe checks ran in library and test modes.
- “At most `FnMut`” is accurate; that edit changes only diagnostic wording.
- Checks used existing pinned-nightly artifacts. Probes ran through stdin with an unrelated lifetime error preventing artifact output. `git diff --check` passed.

Not rerun: `cargo test --workspace` or `conformance/run.sh`; the read-only sandbox forbids their writes. Full-suite results remain author-reported: 2 + 5 tests passed; 180 PASS, 28 AWAITS, 0 FAIL.