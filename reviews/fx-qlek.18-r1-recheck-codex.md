Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate 4d692e6 (re-check)

## Scope and identity

Read `target/work/r1-codex.md` first. This pass re-checks only its two blocking findings and non-blocking precision point, including problems introduced by their fixes. Read the builder's corrected account and “Round r1 fixes”, the complete fix diff, and the relevant code and input readers. No full review was restarted.

HEAD is `4d692e6b1b9b5e2726a0dca6bc0c90c285971911`; tracked working tree clean. `git diff 63d9533 4d692e6` is exactly three documentation files, +11/-8. `git diff --check 63d9533 4d692e6` passes.

## Findings re-checked

1. **Blocking finding 1 — resolved.** `design/reference-companion.md:49` explicitly includes `TaskError` in the E0283 case and annotates the body with the chosen scope error, showing both `Result<(), TaskError>` and `Result<(), MyError>`. This meets the minimum correction without editing `design/reference.md`. The annotation fixes `E` in `Tasks::scope`'s closure-result signature (`crates/tide-std/src/task.rs:1126–1129`).

   `docs/driver.md:241–244` now states automatic conversion into `TaskError::Cancelled`, retains the I/O conversion with `ErrorKind::Cancelled`, and describes intermediate `.map_err(TaskError::from)?` for a user enum. These claims match both `From<Cancelled>` implementations (`task.rs:1885–1894`). The I/O conversion constructs `crate::io::ErrorKind::Cancelled.into()`; `From<ErrorKind> for Error` preserves that kind as `Repr::Simple(kind)` (`crates/tide-std/src/io/shared.rs:310–315`). For a user enum carrying a `#[from] TaskError` variant, the explicit map produces `TaskError`, then `?` invokes the enum's conversion. The derive generates `From` for the annotated field type, not transitive conversion from `Cancelled` (`crates/tide-macros/src/error.rs:512–526`). The owning paragraph explicitly names this variant (`design/03-authority-concurrency.md:1523–1525`). The stale workaround guidance is removed.

2. **Blocking finding 2 — resolved.** `design/03-authority-concurrency.md:1517–1527` separates ordinary functions returning `TaskError`, `io::Error`, or a boxed error from task bodies with a suitable scope error. It gives `TaskError` and `Box<dyn Error + Send + Sync>` and explicitly states `E: From<TaskError> + Send`, the actual bound at `task.rs:1128`. Spawned bodies return the same `E` (`task.rs:1405–1407`). `Cancelled` and `TaskError` both implement `std::error::Error` and are fieldless value types (`task.rs:1871–1912`), supporting the standard boxed-error conversions for the stated example. Ordinary functions do not inherit the scope bound. The paragraph no longer promises that `io::Error` or bare `Box<dyn Error>` is a legal scope error. No scope bound or conversion implementation changed in this fix.

3. **Non-blocking precision point — resolved.** `target/work/report.md:39–42` now distinguishes `.map_err(TaskError::from)?` with the new impl from `.map_err(|_| TaskError::Cancelled)?` before it. Inspection of parent `04dde61` confirms that its cancellation conversion targets only `crate::io::Error`; candidate `task.rs:1891–1894` supplies the new `TaskError` conversion. Both specified mappings produce `TaskError::Cancelled`, which the user enum can accept through `From<TaskError>`. The incorrect “either way” claim is gone.

## Compiled-input claim

Confirmed for this amendment. The complete name/status diff changes only `design/03-authority-concurrency.md`, `design/reference-companion.md`, and `docs/driver.md`; all code, tests, conformance cases, manifests, scripts and toolchain inputs are identical between `63d9533` and `4d692e6`.

Repeated the builder's targeted path searches in `crates`, `conformance`, and `scripts`. The sole reference to an edited file in those non-Markdown inputs is a policy citation at `crates/tide-admit/src/invocation/sandbox.rs:4`, not a file read. The script documentation reader targets untouched `docs/install.md`. Inspected the build script and conformance discovery: `crates/tide-admit/build.rs` obtains toolchain metadata; `crates/tide-conformance/src/main.rs:168–220` reads `conformance/rules.md` and discovers `.rs` cases under `conformance/cases`. Neither reads these edited files. No reader or inclusion of their contents was found in the build/test/conformance paths searched.

The recorded full-suite results on `63d9533` remain applicable to the unchanged compiled inputs in the same recorded environment, under AP-PROOF-001's documentation-follow-up rule. This confirms input identity; it does not claim a fresh run on `4d692e6`.

## Verdict

**approve**. Confidence: high. Both blocking findings and the non-blocking point meet their minimum fixes; no problem introduced by these fixes was found. No further patch is required by this re-check. Opus review and existing landing prerequisites remain coordinator gates.

Not run: builds, tests, compiler probes or runtime execution, as instructed. Only this report was written; no commit made. Existing risk remains: external inferred `scope(..)?` callers may need an explicit error annotation; these documentation fixes introduce no additional runtime risk.

DONE
