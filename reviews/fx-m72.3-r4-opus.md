Model: claude-opus-5-5 (Opus 5.5) · Date: 2026-11-12
Verdict: approve

This is a scoped re-check of my r3 findings 1-7 on HEAD `b45ac30` (parent `d02ccf5`, unchanged; worktree clean), via interdiff `git diff 07fb89d b45ac30`. It is read-only.

Evidence base and limits:
- I ran no probes this round. Eon's quiet windows run continuously until 09:20Z.
- Verdicts rest on source inspection of the interdiff and on the author's final focused logs. Each shows `1 passed; 0 failed`:
  - `fix-r3-generated-final-2.log`
  - `fix-r3-metadata-final.log`
  - `fix-r3-guided-final-2.log`

  These logs are author-run evidence, not independent reruns.
- A source search finds no `Crates::new`, no `__root` and no `.replace(` left in `synthesis/contracts.rs`, `tide-std/src/sim.rs` or the conformance stub.

## Per finding

1. **Fixed.**
   - What changed: `Clauses::read` now requires `contract_expansion(crates, expr.span)` (`contracts.rs:66-71, 120`). The call's outermost expansion must be an attribute macro exported by Tide std, the same provenance test `synthesis.rs` applies to `__tide_property_body`.
   - Why a direct call no longer counts: a user's own call to `std::__contract::const_requires_marker("…")` or `std::__contract::requires(Clause { text: … }, …)` keeps its root syntax context, even when it sits in a body or clause that a std attribute macro passes through. It therefore no longer supplies guidance or copied clauses.
   - Regressions: `forged_const` and `forged_runtime` (`generated_tests.rs`) carry my r3 probe string, and `build.rs` requires each to run 100/100. Without the fix, both would compile and execute the null read, which aborts the process, so the assertion fails.
   - Positive controls:
     - `invariant_clause` shows clauses lowered through `#[invariants]` are still accepted.
     - The `contract-admission` table entry shows the same expression in a real `#[requires]` is still rejected with FX0901 at `demo.rs:1:`.
   - The author's reason for not adding an admission pass over transported tokens: **accepted.** The re-parsed tokens get the test macro's call-site span, so the insertion-range exemption cannot tell them apart from boilerplate. Separating them would need a second provenance channel.
   - Residual risk: with provenance in place, the transported text is exactly the contract macro's stringification of a clause that ordinary admission already checked at its authored site. The `T`→`i64`/`String` instantiation it runs under is also the original function's own monomorphization, which the generated test calls and admission checks with user spans. I see no remaining bypass. A future change that lets a std attribute macro emit marker calls carrying user-chosen strings would reopen this gap, so the scanner comment should keep naming that assumption. Nit, non-blocking.

2. **Fixed, all four paths.**
   - Path printing: `text`/`path` (`contracts.rs:255-310`) render types structurally. Local paths use `with_crate_prefix!`. External ADTs use paths taken from Tide std's actual public exports (`exported_paths`, built once per plan). No string replacement remains.
   - Nameability: `reachable`/`nameable` (`:311-352`) make an unnameable private definition path, or an un-exported external type, a skip: `unchecked: type not nameable from the generated module`. They are applied to `Self`, each input and each draw.
   - APIT: synthetic parameters are left out of both the turbofish and the `__tide_type:` metadata (`:446-456, 601-603`).
   - Body-local impls: the container check uses the impl's own parent (`:415-418`).
   - Regressions: `argument_impl` and `crate_substrings` must run 100/100, and the fixture is now built with `--crate-name=meter`, which catches both the crate-name suffix collision and the old `alloc::` collision via the `thermometer` and `smallalloc` modules. `reexported` and `local_method` must list as ignored with their exact reasons. Before the fix, each of the four made the fixture's test build fail (E0107, E0603, E0433), so `checked(driver…)` would panic.
   - The conservative skip for local re-exports is a valid narrowing: unchecked, never a build failure. The crate docs list it.

3. **Fixed.**
   - `Root` parameters, direct or in a draw, return `unchecked: Root parameter` (`:488-490, 649-651`).
   - The crate entry, including Tide's nested `__tide_main` body, returns `unchecked: entry point` (`:403-408`).
   - `SimRoot::__root()` is removed from `sim.rs`. The stub `include!`s that file, so the stub loses it too.
   - `capability_scope` uses only `Fs`.
   - Regressions: `whole_root` is now asserted ignored with the Root reason (it previously ran 100/100), and a separate `#[ensures(true)] #[tide::main] fn main() {}` package must build and list `ignored, unchecked: entry point`. The crate docs (`lib.rs:80-81`) state the limit.

4. **Fixed.** `Generator` borrows the plan's `&Crates` and a shared export index (`:30-36, 161-167`). All per-use `Crates::new` calls are gone. This is an efficiency fix, so source inspection is the right proof; no regression test is expected.

5. **Fixed (documentation, as I asked).** `lib.rs:154-156` records that generated-test draws and `Properties`/`Arbitrary` synthesis are separate generators and that their policies can differ. No refactor was needed for this round.

6. **Fixed.** A `Trait` parent is classified as `trait method` before the body-local check (`:409-413`). `default_law` is asserted ignored with that exact reason; before the fix it reported "body-local function".

7. **Fixed.**
   - Only vacuity allowances are transported, rebuilt from parsed meta items, with `reason` escaped via `{:?}` (`:578-594`). Unrelated `allow`s and source snippets are no longer copied.
   - `cfg_allowed` (`cfg_attr(test, allow(vacuous, reason = …))` beside `#[allow(dead_code)]`) must pass and print its reason. My r3 failure case was unprobed, so this regression is a positive control rather than proof of a reproduced failure.
   - The missing-reason case is still diagnosed (`generated-vacuity-reason` table entry).
   - Nit, non-blocking, not new in substance: that diagnostic's location is the inserted module-end text, not the user's attribute. The table checks only `demo.rs:`. A user sees the right message at an imprecise column. If cheap, report it at the authored `allow` span from the driver. Otherwise this goes to `fx-djp`.

## New issues

None blocking. The fixes stay inside the touched generator, the fixture, the tests and the crate docs. I found no new trust-boundary or false-"checked" path in the interdiff.

Remaining before landing (coordinator, not this re-check):
- full workspace, conformance and `harness_test.py` suites on the final rebased candidate;
- the reference rerun;
- re-checking any rebase delta.

DONE
