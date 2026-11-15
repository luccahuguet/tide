Model: gpt-6.1-sol · Effort: high · Date: 2026-11-15

## Verdict

request_changes — high confidence in the regression below.

## Scope

Read-only review of `git show HEAD` at `5d181c3`, the author’s report, fx-nkv.7’s Scope/Acceptance and November 12 decisions, AGENTS.md, GOVERNANCE.md, the required design sections and conformance rules.

Traced build identity, executable binding, manifest publication, strict choices, child environments, task-0 origins and affected callers.

## Blocking

1. **Preserve ordinary tests’ callable behavior.**
   **Files:** `crates/tide-macros/src/test.rs:37`, `crates/tide-macros/src/test.rs:281`; `crates/tide-std/src/sim/manifest.rs:103`.

   **Evidence:** The parent preserves parameterless tests as their original function items. This candidate routes them through a generated `fn name()` wrapper and starts a property evaluation on every call. Consequently, previously valid code breaks:

   ```rust
   #[test]
   fn check() -> Result<(), String> { Ok(()) }

   #[test]
   fn composed() { check().unwrap(); }
   ```

   `check()` now returns `()`, so `composed` fails compilation. Even the unit-returning version—`#[test] fn check() {}` followed by a test calling `check()`—fails at runtime with `nested property evaluation`. During driver replay, the nested call can instead fail the outer manifest’s test-identity assertion.

   **Fix:** Preserve ordinary functions’ signatures and direct-call behavior while recording their harness evaluation. Calls made inside an active evaluation should remain part of that evaluation. Add regressions covering both examples, including driver recording/replay of the composed unit-returning test.

## Non-blocking

None.

## Reasoning

The replay boundary otherwise appears sound on inspection: manifest data supplies no executable or host-input paths; identity and compiled configuration checks precede candidate launch; the private executable copy is hashed before execution. Exclusive directory reservation separates recordings, strict tape failures prevent completion, and child environments use explicit allowlists. The task-0 exit override uses the macro’s source while public `Root::sim` retains its caller chain.

The added tests meaningfully cover original-versus-replay trace equality, mismatch refusal, cached executable corruption and `should_panic` masking. They miss the ordinary-test compatibility regression introduced by wrapping every parameterless test.

Not run: additional builds or runtime probes; supplied full-suite results were treated as author evidence. No files changed.
Remaining risk: ordinary tests that call other tests currently regress despite those passing suites.