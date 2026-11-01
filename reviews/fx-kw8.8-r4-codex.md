Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-01

## Verdict

**approve — high confidence.** All three round-1 blockers are resolved. No new in-scope defects found.

## Scope

Read-only re-check of changes since the frozen round-1 candidate, against HEAD `1e0cbac`. Verified both supplied diff hashes and that the current tree matches final SHA-256 `dba4333feeac2dc8e5025f07491608c2403746cb6c2c73e8dad6e0e6729f6ae9`.

Reviewed the intervening code, tests, documentation, ROOT-1, and relevant pinned rustc sources. Excluded `research/12-newcomer-sweep.md` and `.beads/`.

## Blocking

None.

## Non-blocking

None requiring action in this change.

## Reasoning

- **B1 resolved:** [invocation.rs:698](/home/lucca/pjs/lab/tide/crates/tide-admit/src/invocation.rs:698) derives enforcement from the operator’s `--tide-std`, so candidate `#![no_std]` cannot disable FX0512. The new conformance case requires that diagnostic.
- **B2 resolved:** [walk.rs:1195](/home/lucca/pjs/lab/tide/crates/tide-admit/src/walk.rs:1195) reports FX0512 before the locality return and uses the local import span. The new driver test checks the imported-entry regression and diagnostic location.
- **B3 resolved:** [invocation.rs:538](/home/lucca/pjs/lab/tide/crates/tide-admit/src/invocation.rs:538) rejects expanded crate-root `no_main` on executable targets. This matches rustc’s entry selection; nested attributes remain inert. The regression case checks both locations.

The fixes preserve the generated-entry exemption and local `Termination` analysis. README correctly explains the development mode without `--tide-std`. No alternative design or further patch is needed for approval.

Not checked: builds or execution in this sandbox. The reported workspace and conformance results were not independently reproduced; runtime verification remains author-supplied.