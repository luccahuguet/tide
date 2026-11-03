Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-03

## Verdict

**request_changes — high confidence.** The approach is sound, but the toolchain-span exemption permits a test-generated call to the real entry.

## Scope

Reviewed fx-kw8.14, its parent, the requested governance/design references, changed files, surrounding driver and macro code, conformance runner, prior FX0512 reviews, and pinned rustc sources. Ignored `.beads/`.

Confirmed unchanged at completion:

- HEAD: `5661fa8e158d314296a4d87dc0679560f0716cbc`
- Diff SHA-256: `f2fc03b72bde348bfe5e013f585bd09cc70b1120589e4cecdb0f71178273ae09`
- Candidate tree, including the named untracked case and ignoring `.beads/` changes: `2dd08da273efb8c3d7911164d508b234d9a04a3d`

## Blocking

1. **P1: A test descriptor can call the real `main` without FX0506.**  
   [crates/tide-admit/src/walk.rs:1324](/home/lucca/pjs/lab/tide/crates/tide-admit/src/walk.rs:1324), using the exemption at [walk.rs:329](/home/lucca/pjs/lab/tide/crates/tide-admit/src/walk.rs:329).

   Concrete input, checked as a binary and under `--test`:

   ```rust
   #[cfg_attr(test, core::prelude::v1::test)]
   #[tide::main]
   fn main() {}
   ```

   Under `--test`, the outer test attribute emits a descriptor containing the equivalent of `StaticTestFn(|| assert_test_result(main()))`. It preserves `#[tide::main]` on the original function, which subsequently becomes the runtime entry.

   The descriptor’s HIR path resolves to the recorded `main`, but `generated_by_toolchain` recognizes its core `#[test]` span and suppresses FX0506. The runtime call inside `main` remains trusted, and the backstop performs no FX0506 check.

   **Risk:** a test invokes the real entry and mints a real `Root`, violating ROOT-5. This counterexample follows from the pinned compiler’s expansion code; it was not executed.

   **Minimal fix:** remove the general toolchain-span exemption from `main_named`. Current entry and harness wiring require no HIR reference to this recorded function: Tide calls `__tide_main`, while the harness references test descriptors. Preserve the general exemption for other admission checks. Add this attribute-ordering regression and require FX0506 under `--test`.

## Non-blocking

None.

## Reasoning

Resolved function identity correctly covers explicit calls, imports, aliases, re-exports, and function values. The crate-root/executable restriction avoids rejecting ordinary library helpers, and excluding the harness’s generated `main` preserves normal entry recognition.

The defect is applying the broader admission exemption to ROOT-5. Compiler-generated references can still be induced by candidate source. Existing tests cover explicit references and library test mode, but miss this attribute-ordering route.

Not checked: builds or execution, as requested; recorded verification remains author-supplied.  
Risk: the generated test call above still bypasses ROOT-5.