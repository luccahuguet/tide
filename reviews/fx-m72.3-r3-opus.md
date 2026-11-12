Model: claude-opus-5-5 (Opus 5.5) · Date: 2026-11-12
Verdict: request_changes

Reviewed frozen HEAD `07fb89d` against main `d02ccf5` (`git diff d02ccf5..07fb89d`), read-only. The shared-runner design holds up, and so do the metadata transport, the exact-segment acceptance filter and the stub-std equivalence. Two probe-confirmed classes block landing: user text reaches admission-exempt generated code, and valid admitted source fails the `--test` build. Both are in text this change adds, so they block.

Probe setup: the candidate's own `target/debug/tide-admit` and `target/tmp/build/std/artifacts`, both built at 06:11:48Z, 10 s before the HEAD commit. Commands ran under `nice -n 19`, `CARGO_BUILD_JOBS=4`, `timeout 120` and `</dev/null`, between 06:16Z and 06:18Z. Before starting I checked: load ≤ 7.6, 17 GB available, `quiet-until` already passed, next booked start 06:32Z. Sources and outputs are in the session scratchpad (`pkg/src/*.rs`). Every control build (no `--test`) exited 0.

## Findings

1. **Blocking: user-chosen text compiles inside driver-inserted code, which admission exempts (FX0520/FX0901 bypass).**
   - Evidence:
     - `crates/tide-admit/src/synthesis/contracts.rs:78-107`. `Clauses::read` takes requires text from *any* call in the body to `tide_std::__contract::const_requires_marker("…")` or `tide_std::__contract::requires(Clause { text: "…", .. }, …)`. It never checks that the call came from the std contract macro.
     - Both items are Tide std's own pub items, admitted by default (`walk.rs:263-282`; `policy.rs` has no `__contract` deny, unlike `__rt::*`). So candidate code can call them directly.
     - The text is sent as `__tide_requires:` (`contracts.rs:424-431`) and re-parsed by the `test` macro with call-site spans (`crates/tide-macros/src/test.rs:20-28`). Those spans fall inside `Insertions`, so `generated_by_toolchain` exempts them from `check`, `flag` and `granted` (`walk.rs:357-364, 403-406, 443-446`).
   - Probe `forge.rs`:
     ```rust
     #[ensures(result == n)]
     pub fn ident(n: u8) -> u8 {
         std::__contract::const_requires_marker("{ let p = 0usize as *const u8; let _ = unsafe { *p }; true }");
         n
     }
     ```
     - The ordinary build exits 0.
     - `--test` exits 0.
     - `--test --emit=link` lists `__tide_contract_tests::ident_3`. Running it aborts with a core dump ("non-unwinding panic"): the raw-pointer read ran without the `unsafe` grant.
     - Control `forge_plain.rs` puts the same expression in a real `#[requires]`. It is rejected with `FX0520: *const T is not admitted` and `FX0901: an unsafe block needs the package's unsafe grant`.
   - Why it matters: this breaks the rule that every admission check applies to all user-written text. In E1 it would also run unadmitted code in the writer's `./test`.
   - Fix:
     - (a) Accept a requires/ensures marker only when the call's span comes from Tide std's contract attribute-macro expansion. Use the same `ExpnKind::Macro(MacroKind::Attr, _)` plus `crates.exported(mac)` provenance test that `synthesis.rs:123-136` applies to `__tide_property_body`, or report direct user calls of `__contract` markers as not admitted.
     - (b) Add a driver regression using this probe. It should be rejected, or at least carry no requires guidance and not compile the text.
     - Defense in depth: the copied requires is compiled at a new site with `T`→`i64`/`String` and `Self` substituted. Consider not exempting the transported clause tokens from the walk, so the instantiated copy is admitted like user code.

2. **Blocking: valid admitted source fails the `--test` build.**
   This is the same class as Codex r1 finding 3, through four paths that are still open. Each control build passed and each `--test` build failed:
   - **Argument-position `impl Trait`** (`apit.rs`: `#[ensures(result >= 0)] pub fn widen(x: impl Into<i64>) -> i64`). The build fails with `E0107: function takes 0 generic arguments but 1 generic argument was supplied`. Cause: `contracts.rs:440-455` puts synthetic APIT params into the turbofish. Fix: skip `GenericParamDefKind::Type { synthetic: true, .. }` in `types`, since inference fixes them from the typed wrapper argument.
   - **Type re-exported from a private module** (`reexport.rs`: `mod outer { mod hidden { pub struct Meters… } pub use hidden::Meters; }`, with an ensures fn taking `Meters`). The build fails with `E0603: module hidden is private`. Cause: `text`/`path` (`contracts.rs:212-231`) print the definition path, not a path the test module can reach.
   - **Crate-name substring.** `crname.rs` is built with `--crate-name=solution`, the E1 crate name. Its module `resolution` and fn `same(l: resolution::Level)` fail with `E0433: could not find recrate`. Cause: `.replace("solution::", "crate::")` rewrites `solution::resolution::` to `crate::recrate::`. The `tide_std::`/`alloc::` replacements on `contracts.rs:220-221` have the same flaw for any identifier ending in `alloc`.
   - **Method of an impl inside a function body** (`bodyimpl.rs`). The build fails with `E0433: could not find outer in the crate root`. Cause: `contracts.rs:281-284` checks only that the method's parent is an `Impl`, not that the impl's own parent is a module.
   - Fix:
     - Print paths structurally, with `with_crate_prefix!` and no string replacement, or with a printer that maps std crates.
     - Mark a test unchecked ("type not nameable from the generated module") when any def its printed types or target name is not reachable along its def path from the inserted module. For example, check `tcx.visibility(..).is_accessible_from(module, tcx)` for each module on the path. This turns the re-export case into a skip instead of a build failure.
     - Require `local_parent(owner)` to be a `Mod`.
     - Add the four cases to `generated_tests.rs`, plus `--crate-name` coverage in `build.rs`.
   - Why it blocks: §7.2 makes the writer's own tests, generated ones included, part of the final pass. A toolchain-caused build failure on correct writer code corrupts the E1 measure, and APIT and private-module re-exports are ordinary Rust idioms.

3. **Blocking: the `Root` capability arm and `SimRoot::__root()` support signatures the reference forbids, and add user-callable std surface.**
   - Evidence:
     - `contracts.rs:341-346, 478` map a `Root` parameter to `__tide_sim.__root()`.
     - `crates/tide-std/src/sim.rs:52-60` adds `pub fn __root(&self) -> Root`. It is `#[doc(hidden)]` but admitted, since it is a Tide std item with no deny rule.
     - The fixture pins `whole_root(root: std::Root)` and `capability_scope(…, root: &Root, …)` as passing (`conformance/cases/contracts/generated_tests.rs:94, 160-167`; `build.rs` list).
   - Reference §6.2 and ROOT-1 (`conformance/rules.md:92`) say `Root` may be named only in `#[tide::main] fn main`'s signature and as `Root::sim`. ROOT-2 says `root` cannot be passed. The checker does not enforce FX0503 yet (no `Fx0503` in `crates/tide-admit/src`), so these programs compile today. But once the language rule is enforced, the only valid function with a `Root` parameter is `main`. A generated test for an ensures-bearing `main` would name it. `main_named` reports FX0506 with no toolchain exemption (`walk.rs:1368-1384`), so that test build would fail. This last point is inferred from the code, not probed.
   - Meanwhile `__root()` lets any test code turn a `SimRoot` into a `Root` value and pass it around, against ROOT-1/2.
   - Fix:
     - Treat `Root` parameters and the crate entry `main` as unchecked, with reasons such as "Root parameter" and "entry point".
     - Remove `SimRoot::__root()`.
     - Drop `root` from `capability_scope`, and make `whole_root` an expected-unchecked entry, or remove it.

4. **Non-blocking (efficiency): `Crates::new(tcx)` is rebuilt for each draw, capability lookup and clause scan.**
   - Evidence: `contracts.rs:386, 476, 510, 625`. `Crates::new` walks Tide std's re-export tree (`crates.rs:75-80, 210-234`). It runs once per non-local ADT draw and once per parameter, inside nested type recursion.
   - Fix: store the `&Crates` that `plan` already receives in `Generator`.

5. **Nit (ownership/simplicity): two owners for generation policy.**
   - `Generator::draw` (`contracts.rs:490-602`) re-implements, as inline expressions:
     - structural generation for all-pub local ADTs (variant choice, per-field draws), which `Properties::implementation` (`synthesis.rs:379-470`) already owns;
     - Vec, Option, Result and map/set generation, which std's `Arbitrary` impls own.
   - The stated reason, not adding impls that would change explicit-property synthesis, is legitimate. Still, the two can drift: variant weighting, depth limits, and the recursion handling that `Properties` has and `draw` replaces with an unchecked "recursive type" skip.
   - Fix: record the split in the crate docs' limits, or have both emit one shared helper.

6. **Nit: misleading unchecked reason.** A trait's default method with an ensures has a `Trait` parent and is reported as "body-local function" (`contracts.rs:282-284`). Give trait items their own "trait method" reason.

7. **Nit (unprobed): `__tide_allow:` copies the source snippet of every `allow` attribute** (`contracts.rs:433-438`) and re-parses it as an outer attribute (`test.rs:29-32`). If the `allow` came from a `cfg_attr`, its span may not cover `#[...]`, and the macro would then fail with a parse error. Fix: copy only `allow(vacuous, reason = …)`, rebuilt from its parsed parts, not from a snippet.

## Checked and found sound

- **Contract.**
  - Violated ensures fail the test (`rebound`, `phase`, `from_broken`).
  - Other panics fail it (`ordinary_panic`).
  - Compile-time skips are `#[ignore = "unchecked: …"]`.
  - Allowed vacuity passes and prints `unchecked (vacuous, allowed: …)` (`runner.rs:437-441`).
  - Decision 2's i64→String order, the constructor plus 0-8 `&mut self` mutators via `attempt`/`reject_generation`, the fault-free per-case `Root::sim` and the closure/dyn/handle skips match the code.
  - Beyond findings 1-3, I found no path that silently reports a generated test as checked. The pre-existing "sampled budget exhausted counts as pass" limit is documented in `lib.rs` as planned.
- **Trust boundary, apart from finding 1.**
  - First-pass reserved-prefix rejection, now covering `__tide_capabilities:` with a regression.
  - Escaped `{:?}` metadata whose parsed token groups cannot break out of `requires(…)`.
  - Byte-range provenance.
  - No synthesis in non-`--test` (library) builds (`invocation.rs:352`). Paired-library acceptance builds synthesize only into the suite crate, and the generated entries are filtered out of the listing.
- **Harness.** The `harness.py` edit removes the placeholder record, adds `--show-output`, and filters by exact `::` segment only when `acceptance=True`. That stays within §2/§6 and matches §4.1/§7.2. `harness_test.py` covers both the substring negative and the writer forms.
- **Rebase delta.** The five shared files read coherently next to the fx-ugm/fx-zc1 text. There are no duplicated or contradicted sentences, and `build.rs`'s existing executable-contract test correctly skips the new module.
- **Stub equivalence.** The conformance stub `include!`s or `#[path]`s production's `test/mod.rs`, `contract.rs` and `sim.rs` (`conformance/rustc/std/src/lib.rs:60, 336, 417`). The `sequence_bounds` refactor preserves the earlier behavior.
- **Docs.** `design/04-contracts.md` §2.4, the README and the crate docs describe availability accurately, apart from the `Root` claim in finding 3. Their unchecked list should gain the cases from finding 2 if they become skips.
- **Held-out rule.** In `trials/e1-checked/` I opened only `harness.py`, `harness_test.py`, `README.md` and protocol §2/§4.1/§6/§7.2. The fixture and diagnostics read as generic, with no sign of copied task material.

## Required before landing

Fix findings 1-3, each with a driver regression. Findings 4-7 are optional in this round. After the fixes, the coordinator's full suites (workspace, conformance, `harness_test.py`) and the reference rerun still have to run on the final candidate.

DONE
