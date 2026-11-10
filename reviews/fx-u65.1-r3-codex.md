Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-09

## Verdict

**request_changes — high confidence.** B2 and N1 are resolved. B1's original example is fixed, but the new coercion check skips call and method-call expressions. A statically known channel constructor still loses FX0608 when a method returns it at the coercion site. Fix that remaining B1 gap before approval.

## Scope

Re-checked only my round-1 B1, B2, and N1, their fixes and coverage, and defects within those changed lines, as required by `r2-brief-common.md` and GOVERNANCE.md's Merge Rule. Reviewed `git diff baee486 fx-u65.1`, ending at `aabea4cc8faa3604c28fb2ae92b5b0c6a8ea7b62` (`fx-u65.1: fix round-1 findings`), plus the relevant caller, existing callback adapter pattern, diagnostics, reference §§7.3–7.4, coordinator decisions, and author proof. The author's report describes the fixes before their subsequent commit. I did not reopen the other round-1 decisions or independently re-review Opus-only findings.

The worktree was clean before and after inspection. No repository edits, staging, commits, or Beads writes were made. Execution used a private `git archive fx-u65.1` copy at `/tmp/tide-fx-u65-r3-codex.OZ11N6`; every compiler invocation ran serially under `nice -n 19 timeout 45` after checking the prescribed quiet-window gate. No Cargo builds ran.

## Per-finding status

### B1 — Partly resolved; remains blocking

**Location:** `crates/tide-admit/src/walk/sinks.rs:49–57`, after the call and method-call arms at `:34–47`; caller updated at `crates/tide-admit/src/walk.rs:1813`.

The new handler identifies an expression's original `FnDef` type when its adjusted type is a function pointer. This fixes the original typed alias. My unchanged round-1 `channel_alias.rs` now exits 0 with FX0608 at both the untyped alias's call (line 5) and the typed alias's coercion (line 6). The committed regression is `conformance/cases/tasks/channel_library.rs:10`; its binary counterpart is `conformance/cases/tasks/channel_binary_accept.rs:10`. Both pass my driver checks in normal and test modes.

However, the coercion arm is reached only when neither earlier expression-kind arm matches. A method call can itself return the concrete channel function item and coerce it to a pointer:

```rust
use std::sync::mpsc::{channel, Sender, Receiver};
use std::task::Tasks;
pub fn queues(tasks: &Tasks) {
    let held = Some(channel::<u8>);
    let pointer: fn(&Tasks) -> (Sender<u8>, Receiver<u8>) = held.unwrap();
    let _ = pointer(tasks);
}
```

**Observed:** the isolated `probe-src/channel_wrapped_alias.rs` above compiled as a library with exit 0 and no diagnostics. In a separate control probe, the plain typed alias, `std::convert::identity(channel::<u8>)`, and an explicit function-pointer cast all emitted FX0608; the `held.unwrap()` variant did not.

Here `held` contains the concrete function-item type, so the original type of `held.unwrap()` still identifies `channel`. The earlier `MethodCall` arm instead selects `Option::unwrap` as `def` and bypasses the new coercion arm. The eventual pointer call has no `FnDef` and returns early. This is the same loss of a statically known constructor at the function-pointer boundary as B1, within the added handler's coverage; it needs no inference of runtime pointer targets or transitive effects.

**Smallest fix:** give the existing `FnDef`-to-function-pointer coercion check precedence over the `Call` and `MethodCall` arms, retaining the ordinary call handling otherwise. Add the example above to the library warning case and its accepted binary counterpart. The library should warn at the coercion and still exit 0. No new abstraction or pointer-target analysis is needed.

### B2 — Resolved

**Location:** `crates/tide-admit/src/walk/sinks.rs:81–88`; regression at `conformance/cases/authority/lock_print.rs:20–23`.

The callback resolver repeatedly unwraps references and `Box<F>` with the existing `def.is_box()` / `args.type_at(0)` pattern before selecting a local closure or function body. The original boxed closure now emits FX0607 at `probe-src/lock_callbacks.rs:6`, alongside the direct and referenced callbacks. The committed boxed-callback case passes in normal and test modes. Logging and the nested-function exclusion remain accepted in `lock_log_accept.rs`.

### N1 — Resolved under the coordinator's chosen lexical rule

**Location:** `crates/tide-admit/src/walk/sinks.rs:99–109`; coverage at `conformance/cases/authority/lock_print.rs:10–11` and `conformance/cases/authority/lock_log_accept.rs:8`.

`LockPrints` now uses `nested_filter::OnlyBodies` and supplies `maybe_tcx`, so it enters nested closure bodies while excluding nested function items. The unchanged round-1 immediately invoked closure now emits FX0607 at `probe-src/lock_callbacks.rs:8`. The committed iterator-callback case also warns.

The implementation, comment, and cases consistently reflect the coordinator's explicit choice to warn on uncalled nested closures too: `_later` moved from the accepted case to the warning case. This resolves N1 by extending the lexical walk. Arbitrary callees remain outside that walk, as documented and previously accepted.

## New findings

No separate findings. The remaining coercion-arm ordering defect is recorded under B1 above rather than counted twice. No broader review restart is requested.

## Validation and limits

- `git diff --check baee486 fx-u65.1`: exit 0.
- Re-ran the two unchanged round-1 source probes with the copied driver: both exit 0. `channel_alias.rs` emits two FX0608 warnings; `lock_callbacks.rs` emits four FX0607 warnings, including the formerly missed boxed and immediately invoked nested closures.
- `python3 probe-checks.py` in the private archive: eight driver-only assertions pass. They check exact FX lines and severity plus exit 0 for `lock_print`, `lock_log_accept`, `channel_library`, and `channel_binary_accept`, each in normal and test modes. Library-only warnings are absent in executable test mode, consistent with the coordinator's decision. This is not a full conformance run and does not execute the stable-rustc oracle.
- The isolated remaining-B1 probe used the same invocation shape as the successful controls: `nice -n 19 timeout 45 probe-tools/tide-admit --crate-type=lib --edition 2024 --tide-std "$PWD/probe-tools/std/libtide_std_stub.rlib" --tide-dep-dir "$PWD/probe-tools/std" --tide-dep-dir "$PWD/probe-tools/std/deps" --crate-name channel_wrapped_alias --emit metadata --out-dir "$PWD/probe-out/channel_wrapped_alias" --error-format=short "$PWD/probe-src/channel_wrapped_alias.rs" </dev/null`. Exit 0, empty output, missing FX0608.
- The copied current driver has SHA-256 `4bbc821e3342efc3b29f558c5b5710011e71e7fb9b8b9e4ca480f711efd3f538`; the copied stub std has SHA-256 `753daf95c3d4ed713206a5cddd9d08cef81087490c60b7ee643ef8599be0b0df`. The std hash matches round 1; its unchanged macro dependencies were reused from the round-1 private copy. An initial invocation exited 2 before compilation because the copied dependency directory was missing; supplying those dependencies resolved the harness setup. Build-to-commit provenance was not independently rebuilt.
- Read the author's final suite logs: workspace totals are 47 tests passed, zero failed; conformance totals are 207 PASS, 24 AWAITS CHECKER, zero FAIL (231 cases). These support the author's reported results; they are not independent suite reruns and do not cover the remaining B1 example.

Not rerun: Cargo builds/tests, the full conformance suite, or unrelated review decisions, as required by the narrow brief. Remaining risk: the demonstrated statically known channel coercion still bypasses FX0608. DONE
