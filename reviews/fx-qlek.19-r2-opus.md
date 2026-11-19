Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate eb6cda6 (code)

## Verdict

approve. No blocking findings. Four non-blocking findings follow; none changes behavior on the pinned toolchain with the production or stub std.

## Scope

Read AGENTS.md, the bead and its comments, the full diff `64af5e9..eb6cda6` (7 files, +156 −5), the builder's report including "Round r1 fixes", r1-codex.md, r1-recheck-codex.md and r1-collision-proof.log. I did not repeat Codex's B1/B2 analysis. I also read the surrounding code: `diag/labels.rs` (the whole emitter), `walk.rs::check_crate` and its call site in `invocation.rs:170-195`, `crates.rs` (`Crates::new`, `injected_std`, `key`), the `--error-format` gate (`invocation.rs:325`), the stub std's `task` module (`conformance/rustc/std/src/lib.rs:83`, which `include!`s the production `task.rs`), and Tide std's `Tasks::scope` (`crates/tide-std/src/task.rs:1126-1130`). Against the pinned `nightly-2026-08-09` rustc sources I checked: `rustc_parse/src/parser/function.rs:115-158` (a fn's signature span), `rustc_middle/src/hir/map.rs:937-956` (`def_span` of an impl fn), `rustc_trait_selection/.../traits/suggestions.rs:3673-3841` (the where-clause note), `rustc_span` `is_visible`/`is_span_accessible`, `need_type_info.rs:940-1000` (how rustc picks the inference subject), and `rustc_metadata/.../decoder.rs:1327-1352` (`get_module_children`).

Full suites on eb6cda6 (`target/work/suites/`): `cargo test --workspace` exit 0 (test.exit; 16 result lines, 470 passed, 0 failed; 07:46:15Z to 07:53:36Z). `cargo run -p tide-conformance` was still running when I wrote this (conformance.log has PASS lines; no conformance.exit or DONE yet). Acceptance needs it at exit 0; tide-lead should read conformance.exit before landing. I ran nothing myself.

## Blocking findings

None.

## Non-blocking findings

**N1 — `walk.rs:126-133`: the lookup copies `synthesis::child` but drops its `DefKind` filter, and an unfiltered match can ICE.**
`synthesis.rs:217` already has `child(tcx, module, name, kind)`, which matches on `DefKind`. The new closure matches any namespace by name only. The metadata decoder's `get_module_children` calls `expect("provided `DefIndex` must refer to a module-like item")` (decoder.rs:1341). Scenario: the injected std's root lists a non-module item named `task` (a `fn task` or a re-exported `task` macro) before `mod task`. Then `module_children` on that item panics, and every crate admitted against that std ICEs. Likewise a tuple-struct `Tasks` could match its constructor first, which fails closed and loses the hint. Neither can happen with today's production or stub std (the test passes on both shapes), so this is hardening only. Minimum fix: call `synthesis::child` (made `pub(crate)`) with `DefKind::Mod` and `DefKind::Struct`, which also removes the duplicate.

**N2 — `labels.rs:234-236, 247-248, 289`: the per-span `hinted` dedup has no test that exercises it.**
On the pinned toolchain only the first E0283 (the scope bound `E: From<TaskError>`) has a child note inside `Tasks::scope`, so the second E0283 at that span (from the `?` conversion) never matches, and the dedup never fires (report.md says as much). If the dedup were deleted, the test's `hints.len() == 1` would still pass. It is a three-line guard against a future rustc that also notes the `?` obligation. Either keep it and say so in its doc comment ("guard; unreachable on the pinned toolchain"), or delete it under AP-MINIMAL-001. Either way is acceptable.

**N3 — `design/03-authority-concurrency.md:1527-1528`: "for an unannotated scope" says more than the trigger does.**
The help fires only when rustc's chosen inference subject prints as Tide std's `Scope` (docs/driver.md:28-32 states this exactly). rustc picks the cheapest subject (`need_type_info.rs:970-990`). With a closure body, the closure parameter `Scope<'_, '_, _>` costs 5 and wins, so every closure shape gets the help: `|s|`, `|_|`, a `let r = tasks.scope(..)` binding (`Result<(), _>` costs 10), nested scopes, and method chains. A non-closure body, such as `tasks.scope(run)?` with a generic `fn run<E>(..)`, has no closure parameter. The subject is then the method's generic argument, the message doesn't name `Scope`, and the help is missing although the scope is unannotated. That shape is rare and the miss fails closed, so this is non-blocking. Minimum fix (text-only): "…as help to rustc's E0283 when it names the scope's type, as it does for a closure body". Cheapest check, if wanted: run the driver on that `run` shape.

**N4 — `tests/build.rs:5841-5844`: the `local-scope` negative is redundant.**
`local-tide-std` is the same program with a stronger collision: its printed path is exactly `tide_std::task::Scope`, while `local-scope` prints a bare `Scope<_>`. Any matcher that passes `local-tide-std` also passes `local-scope`. You could delete `local-scope` (AP-TEST-001: few strong tests). Nit.

## Reasoning

**Provenance mechanism is sound on the pinned toolchain.** My main worry was that the bound `E: From<TaskError> + Send` sits in a `where` clause (task.rs:1127-1129) while `def_span` of an impl fn is `sig.span` (hir/map.rs:946-955). I checked this one. `parse_fn` sets `sig_hi` after `parse_where_clause` (function.rs:146-153), so the signature span, and hence `def_span`, includes the where clause. When Tide std's source is readable, rustc's note has the predicate span as its only primary span (`MultiSpan::from(span)`; the "required by a bound in this" ident label is secondary), and that span lies inside `def_span`. When the source is not readable, the note goes on `tcx.def_span(item_def_id)` itself (suggestions.rs:3812-3841), which `contains` trivially. Either way the containment holds, and the collision log shows it empirically. If Tide std ever moved the method into a macro, or its bounds outside the signature, the check would fail closed and the positive test would catch it.

**Identity boundary.** `Crates::tide_std` is the crate behind the `extern crate std` that rustc's std injection adds (`ExpnKind::AstPass(StdImports)`), with Rust's std, core and alloc excluded (crates.rs:68-73, 172-186). A user's `extern crate x as std`, a local `mod tide_std`, or a dependency that re-exports `tide_std` cannot become that crate. A local item can't produce a span inside an imported file's byte range; Codex's re-check covers the source-map argument, and I agree with it. `include!` of task.rs into a user crate creates a separate local `SourceFile`, and user macros are already rejected. Under the stub std (`tide_std_stub`), `key` normalises the crate name to `tide_std::…` and the stub `include!`s production task.rs, so the hint works there too. That doesn't matter for the conformance runner, which reads only codes.

**Diagnostic stream.** `emit_diagnostic` forwards rustc's `DiagInner` untouched and first. TD labels come next (none apply to E0283), then the help, which goes straight to the inner emitter: it adds no count, changes no exit status, and never re-enters the dcx. The `span.unwrap()` at :288 is guarded because `hint` implies a primary span. The lock guard is dropped before emitting. On an abort, rustc's error has already been written when the help is emitted, so at worst the help is lost, never the error. The only accepted `--error-format` is `short` (invocation.rs:325), and the emitter fixes short output anyway, so JSON can't arise. The conformance runner reads only `error[...]` and TD lines, so an uncoded `help:` line can't change a verdict. Ordering held in the log: E0283, help, E0283 for `try`.

**False-positive surface.** A match needs three things: code 283, a message containing `` `<std crate>::task::Scope< ``, and a child primary span inside `Tasks::scope`'s signature. An E0283 that meets all three is an ambiguity on one of `Tasks::scope`'s own where-clause predicates, whose subject rustc prints as Tide std's `Scope`. `Sized` ambiguities on `T`, `E` and `F` go to E0282, not E0283. That leaves `E: From<TaskError>` / `E: Send`, which is exactly what the help says ("the scope's error type can't be inferred here"). I found no realistic E0283 that would get the help wrongly.

**Pre-typeck cost and safety.** On a crate without Tide std, `crates.tide_std()?` returns early with no lookup. With Tide std, the lookup reads the root and `task` module children from metadata, `inherent_impls(Tasks)` (decoded from metadata, so no local coherence or typeck), and `key` for each associated item, formatted with no trimmed paths so it doesn't force `trimmed_def_paths`. That is a few dozen items per admitted crate, which is negligible. The only panic path is N1, which today's std can't reach. Nothing type-checks bodies before `check_crate` (`check_expanded`, `synthesis::plan` and `replay::test_configs` don't), and the positive test would fail if that ever changed.

**Contract and docs.** docs/driver.md:28-32 states the code's predicate exactly: an E0283 naming Tide std's `Scope` with its error unknown and pointing to a bound of `Tasks::scope`, with or without `?`, a help line right after it at the same place, and none for other E0283s. The companion row (:49) says only that the help exists, which is accurate. reference.md is unchanged, and the change doesn't touch the companion's count. design/03 §6.1 is accurate for every closure-bodied scope. N3 is the one shape it overstates.

**Tests (AP-TEST-001).** One integration test with a loop over cases:
- Positives `try` and `no-try` each check exit 1, exactly one help line, its exact `path:line:col` (column computed from the source), and that the line just before it is an E0283 at the same location.
- Their annotated counterparts compile, so the advice works.
- Negatives: `sum` (an unrelated E0283), `local-tide-std` (the identity collision; the log shows it fired on 0d34c99), and `local-scope` (redundant, N4).

All four categories the brief asks for are present, and they are strong. The test failed without the change (without-change.log). The only unexercised logic is the dedup guard (N2).

**Leftovers and dead code.** The `pub use … TideScope` export has one consumer (walk.rs). The changed module doc line is accurate. No caller, fixture or config is orphaned. `SCOPE_HINT` matches the test's `HINT` and the docs. The duplicated `child` helper is N1. No dead code beyond the N2 guard.

DONE
