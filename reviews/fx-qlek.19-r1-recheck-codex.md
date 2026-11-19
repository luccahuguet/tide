Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate eb6cda6 (re-check)

## B1 — Resolved

The printed path is now corroborated by compiler provenance. `walk.rs:93–97,126–140` starts at the injected Tide std's CrateNum, resolves `task::Tasks` and its inherent method with canonical key `tide_std::task::Tasks::scope`, then records that DefId's definition span before body type-checking. `diag/labels.rs:178–193` requires E0283, the printed Scope path, and a child primary span contained by that recorded definition span. A local function named `Tasks::scope`, or a local type printed as `tide_std::task::Scope`, cannot satisfy the last condition through its own definition/bound span.

I checked the installed pinned `nightly-2026-08-09` compiler sources:

- `rustc_span/src/lib.rs:728–730,840–843`: `Span::contains` compares session-global byte positions.
- `rustc_span/src/source_map.rs:325–345,389–425`: imported source files are registered into separate byte ranges, with a gap between files. Their original byte offsets are not reused as session positions.
- `rustc_metadata/src/rmeta/decoder.rs:646–667`: metadata spans are translated using the imported source file's allocated start position.
- `rustc_span/src/lib.rs:2125–2155`: exported stable source-file identities incorporate the exporting crate identity; a matching local filename does not impersonate the imported file.

Consequently a local item's span cannot lie inside this external method's span merely by copying a type/function/module name, filename, line, or column. The check authenticates the bound's origin. It does not need to compare the child note's text or require a particular rendered spelling of the method name.

It fails closed without Tide std: `Crates::new` excludes Rust std/core/alloc from the Tide std role; `tide_scope_method` propagates missing std/module/type/method as `None`; `Sites::scope` defaults to `None`, and the matcher then returns false. Missing provenance or a missing child primary span also gives no hint.

For real calls, the new requirement follows the existing ambiguous bound. `tide-std/src/task.rs:1126–1130` declares `E: From<TaskError> + Send` in the method's definition. Pinned rustc's `error_reporting/traits/suggestions.rs:3677–3837` emits the where-clause note at the bound's span when visible, or at `tcx.def_span(item_def_id)` otherwise; both satisfy containment. `error_reporting/traits/ambiguity.rs:716` adds the obligation cause before emission. Chained receivers, nested calls, renamed imports and UFCS do not change the defining bound. I found no new exclusion of these shapes when rustc still chooses the matching Scope type as its E0283 subject. This is source analysis, not additional execution.

The original printed-subject limitation remains: an equivalent inference problem whose diagnostic names a different type/expression is outside this matcher. This fix does not introduce that limitation, and universal coverage of all inference spellings is not claimed here. The separate E0283 for the outer `?` conversion can lack the scope-bound note; the scope's own bound diagnostic supplies the hint instead.

`target/work/r1-collision-proof.log` records the exact original local-path collision receiving help under `0d34c99`, and the same E0283 receiving no help under the rebuilt candidate. Both real positives (`try`, `no-try`) retain the help. The build test now includes this collision negative alongside the previous local-Scope and sum negatives.

## B2 — Resolved

`diag/labels.rs:155` now says the scope's error type cannot be inferred and asks the user to name it, giving `TaskError` as an example. It no longer asserts that `?` exists, that the enclosing result uses `TaskError`, or that two conversions caused this particular diagnostic. For the matched ambiguity, the unknown parameter is the scope error parameter constrained by the genuine scope bound. The example need not be the user's chosen error type; the text explicitly presents it as an example.

`docs/driver.md:28–32` describes the actual trigger: E0283 naming Tide std's Scope with its error unknown and pointing to a bound of Tide std's Tasks::scope, with or without `?`. `design/03-authority-concurrency.md:1524–1528` retains the explanation of the `?` habit and describes help for an unannotated scope; it does not restrict the matcher to `?` or TaskError-returning bodies. The companion's row 49 describes that same habit and permits either TaskError or MyError. These descriptions are consistent: the driver documents the exact diagnostic predicate, while the design and companion explain its motivating usage.

The updated integration test covers a scope without `?` and checks that its explicitly annotated counterpart compiles, as well as retaining the original `?` pair. The no-`?` fixture still has a surrounding `Result<(), TaskError>` signature, but there is no return-context predicate in the matcher and the revised help makes no claim about that context. The collision proof log records the corrected wording on both real positives.

## Non-blocking companion cell — Resolved

`design/reference-companion.md:49` now explicitly says “rustc E0283, with tide-admit's help naming the turbofish”. `design/reference.md` remains unchanged. This addresses the original omission without changing the core reference or its count.

## Non-blocking same-location ordering check — Resolved

`crates/tide-admit/tests/build.rs:5822–5831` requires exactly one complete hint, computes the closure parameter's column, checks the full `demo.rs:3:<column>` location, and requires the immediately preceding line to be an E0283 with that identical full location. It applies to both `try` and `no-try`. The collision proof records immediate adjacency at 3:18 and 3:26 respectively. The emitter still forwards the original error before emitting the separate help at the same span.

## Evidence and limits

HEAD was `eb6cda606f6a4862deea80390a8f5383ac2b19ab`, with clean tracked status. Reviewed only `git diff 0d34c99 eb6cda6`, the original findings, builder's “Round r1 fixes”, lead's fix brief, and connected code/compiler sources needed to assess those fixes. The fix diff is six files, +99/−48; the complete amended candidate is seven files, +156/−5. `git diff --check 0d34c99 eb6cda6` passed.

Inspected builder evidence: `target/work/focused-eb6cda6.log` records build 118, code_prefix 2, and invocation 18 passing; `target/work/r1-hint.log` records the focused hint test passing. These are recorded builder runs, not independent executions by this reviewer. Full workspace and conformance suites remain pending, as instructed. No new issue introduced by the fixes was found within this re-check's scope.

## Verdict

approve — B1, B2, and both non-blocking items are resolved. This is the requested finder re-check, not a replacement full review or a claim that the pending suites or subsequent Opus review have passed.

Only this report was written. No build, test, compiler, Tide binary, commit, implementation edit, Beads write, delegation, or contact with another person/agent was performed.

DONE
