Model: gpt-6.1-sol. Effort: xhigh. Date: 2026-10-31.

## Verdict

**approve — high confidence (0.94).**

The test exercises the fixed path and would catch the original ICE. No patch is required for approval.

## Scope

Read `AGENTS.md`, `GOVERNANCE.md`’s Merge Rule, `br --no-db show fx-kw8.12`, both cited reviews, the complete driver tests, and the relevant invocation, backstop, policy, and `Pure` code. Checked ADMIT-1/ADMIT-2 and the pinned rustc’s collection, MIR encoding, callback, and `str::to_lowercase` implementations.

Environment: x86_64 Linux, rustc `1.99.0-nightly (771916f90 2026-08-08)`.

Candidate HEAD at start and end: `bb17a0ab227b198ef9b25283f0708cfa06e6ff1a`.

`git diff HEAD -- crates | sha256sum`:

- Start: `4d51f6e9061fc94991a31ac9e860235646a8fe49ce29ecf7e2735301b3f74898`
- End: `4d51f6e9061fc94991a31ac9e860235646a8fe49ce29ecf7e2735301b3f74898`

Only `crates/tide-admit/tests/driver.rs` changed within that scope. The scoped whitespace check passed. Ignored unrelated `.beads/` and `research/` changes. No files or Beads state were written.

## Blocking

None.

## Non-blocking

None.

## Alternatives

No materially better alternative. Reusing the minimal std helper and testing both modes directly is sufficient; additional fixtures or instrumentation would add maintenance without strengthening this regression materially.

## Reasoning

**The backstop runs.** The source at `crates/tide-admit/tests/driver.rs:510` has no preceding rejection: the Rust-ABI function pointer is `Pure`, its type satisfies rustc’s static requirements, and the referenced string operation is admitted. The driver calls `check_mono_items` after analysis, including for metadata output. The `--tide-test-backstop` flag skips earlier obligation checks; it does not disable the backstop.

**The static reaches the intended linked function.** Pinned rustc’s root collector includes statics. Evaluating `LOWER` produces a function pointer to the upstream, non-generic `alloc::str::<impl str>::to_lowercase`. Rustc’s `collect_alloc` filters that function through `should_codegen_locally`, leaving it linked upstream. Tide’s `uses`/`pointees` at `crates/tide-admit/src/walk/mono.rs:122` still records the pointer edge.

**The test detects the old failure.** Before the fix, traversal followed that edge and requested `items_of_instance` for the linked function. Pinned rustc routes that query through `instance_mir` to `optimized_mir`, which this function lacks in upstream metadata. The exact program’s exit-101 ICE is recorded in `reviews/fx-kw8.4-r2-opus.md`. The assertion at `driver.rs:516` rejects both that exit status and any diagnostic output. Current traversal requests uses only for rustc-collected items, avoiding the invalid query.

**Both modes receive separate checks.** The cache key includes `backstop_only` at `crates/tide-admit/src/invocation.rs:644`, so the normal invocation cannot satisfy the backstop-only invocation’s cache entry.

**The change is complete and minimal.** Both std-helper callers supply distinct test directories, preventing parallel fixture builds from overwriting one another. Both artifacts remain outside their candidate packages. Existing negative tests retain coverage for backstop enforcement and static-pointer traversal.

Not run: builds, tests, or a pre-fix executable; this review was read-only.  
Risk: runtime acceptance relies on the author-reported passing `cargo test --workspace`; regression sensitivity is supported by source inspection and the recorded pre-fix reproduction.