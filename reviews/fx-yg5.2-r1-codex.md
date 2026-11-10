Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-09

## Verdict

**request_changes — high confidence.** The API direction and production/conformance ownership are sound. The candidate does not meet the all-target math contract: the pinned dependency can still select hardware transcendental instructions despite `force-soft-floats`, and I observed different result bits between its two paths. The numerical oracle and documented dependency setup also need correction. No override is recorded.

What this change does: it adds four canonical float observers and 36 generic math functions, shares their implementation with conformance, and exercises them through production admission. It also replaces two hand-built std test setups with a Cargo helper and lets the E1 harness honor an operator's build-job setting.

## Scope

Reviewed `git diff 07e104b 0823998`, all 15 changed files, both bead records using `br --no-db show`, AGENTS.md, GOVERNANCE.md's Merge Rule, README.md, the production std modules and their consumers, conformance std and runner, `policy.rs`, `pure.rs`, crate-identity/admission/invocation code, `tests/build.rs`, `tests/entry.rs`, and the relevant test fixture setup. Read reference §1/§4.3, D3/D18 and the tier table, memory §9, and E1 protocol §2/§10. Inspected the exact cached libm 0.2.16 manifest, configuration, dispatch macros, math adapter and relevant software/x87 routines. This is a v0 review; no finding belongs only to fx-djp.

All review execution and generated artifacts stayed in the private archive `/tmp/tide-fx-yg52-review.9YNV3s`, created with the brief's `git archive 0823998` command. No repository/worktree/tracker writes or Cargo builds/tests were performed. Every compiler/driver probe was sequential, ran under `nice -n 19` with a timeout and closed stdin, and passed the quiet-window, load <12 and available-memory >=6 GB guards immediately before launch. Environment: Linux x86_64, pinned `nightly-2026-08-09`, rustc 1.99.0-nightly, commit `771916f90`.

Independent observations:

- **P1, software versus x87:** compiled one Rust probe containing the candidate's unchanged `float.rs`/`math.rs`, the cached libm source modules, and its exact `arch/i586.rs` routines. The compile was `nice -n 19 timeout 90 rustup run nightly-2026-08-09 rustc --edition=2024 --cfg 'feature="force-soft-floats"' review-probe.rs -o review-probe </dev/null`; execution was `nice -n 19 timeout 20 ./review-probe </dev/null`. No `arch_enabled` or `intrinsics_enabled` cfg was set. Both exited 0. The probe compared candidate `math::exp`/`exp2` with `x87_exp`/`x87_exp2` and observed the bit differences in B1. Canonical negative/signaling NaN observation, signed-zero minimum, NaN-above-infinity comparison, and `powi(2.0f64, -1074)` assertions also passed. This executes the alternate routines on this x86_64 CPU; it is not an i586 cross-build or heterogeneous-target test.
- **P2, missing dependency:** copied the existing candidate std, macro and libm artifacts into separate private directories. For `pub fn sample() -> u64 { std::float::bits(0.0f64) }`, `nice -n 19 timeout 30 rustup run nightly-2026-08-09 rustc --edition=2024 --crate-type=lib --emit=metadata --extern std=readme-probe/std/libtide_std.rlib -L dependency=readme-probe/macros --out-dir readme-probe readme-probe/sample.rs </dev/null` exited 1 with `E0463: can't find crate for libm which tide_std depends on`. Repeating it with `-L dependency=readme-probe/libm` exited 0.
- **P3, production fixture execution:** copied the existing driver binary and the exact shared fixture to the private copy. Ran `nice -n 19 timeout 90 admission-probe/tide-admit --crate-type=lib --test --emit=link --crate-name=float_review --tide-std readme-probe/std/libtide_std.rlib --tide-dep-dir readme-probe/macros --tide-dep-dir readme-probe/libm --out-dir admission-probe/output admission-probe/fixture/src/main.rs </dev/null`, then `nice -n 19 timeout 30 admission-probe/output/float_review-test </dev/null`. Both exited 0; both tests passed, exercising both widths and the kept methods.
- **P4, production rejection:** the same copied driver/artifacts, with `--crate-type=bin --emit=metadata --crate-name=reject` and a private source containing `#[tide::main] fn main() { let _ = 0.5f64.sin(); }`, exited 1 with `FX0520: std::f64::<impl f64>::sin is not admitted`. Output went to private `admission-probe/rejected`.
- Python AST parsing of the archived harness passed; `git diff --check 07e104b 0823998` passed. Selectively inspected the existing A5 result record without reading held-out source: source check 0, structure true, 11 result codes 0 and two codes 2, no `std::float` diagnostic, overall passed=false.

P2–P4 reuse prebuilt artifacts from the reported candidate; they are not independent rebuild proof. I inspected the implementer's workspace/conformance proof logs, but their 76-test and 198-PASS results remain attributed evidence. I did not rerun all 88 negative cases, the full suites, A5, or another target.

## Blocking

### B1 — P1: `force-soft-floats` does not establish the required all-target result identity

**Location:** `crates/tide-std/Cargo.toml:9–10`, `crates/tide-std/src/math.rs:35–36` and `:100–101`; the same configuration is in `conformance/rustc/std/Cargo.toml:9`.

**Failure:** on x86 without SSE, libm's `exp` and `exp2` select `x87_exp`/`x87_exp2` through `use_arch_required: x86_no_sse`. The published dependency's `src/math/support/macros.rs:48–70` explicitly makes that selection independent of `force-soft-floats`. Its `src/math/arch/mod.rs:43–55` exports those paths with software mode enabled, and `src/math/arch/i586.rs:64–69` says their f64 error can depend on hardware. NaN canonicalization cannot remove a difference in an ordinary finite result.

P1 observed:

| Function | Input bits | Software result bits | Cached x87 routine result bits |
| --- | --- | --- | --- |
| `exp(0.019f64)` | `0x3f9374bc6a7ef9db` | `0x3ff04e916a97ec0e` | `0x3ff04e916a97ec0f` |
| `exp2(16.077f64)` | `0x403013b645a1cac1` | `0x40f0e08d5dae5f53` | `0x40f0e08d5dae5f52` |

This is a concrete backend difference, not merely missing cross-target testing. The report's claim that software mode suppresses architecture kernels so identical algorithms run everywhere overlooks the required-architecture branch. Upstream also ignores its near-pi sine bit test on `x86_no_sse` (`src/math/sin.rs:89`), so simply removing the x87 dispatch would leave intermediate-precision concerns unresolved.

**Smallest fix:** retain the public adapter, but replace or patch the backend/configuration so every admitted target follows an audited deterministic algorithm with the required binary32/binary64 rounding. Add these differing inputs to the regression evidence and exercise the affected target path. Reassess the dependency record against that mechanism. A comment admitting host-only proof, another feature flag that leaves `use_arch_required` active, or silently excluding targets does not satisfy the bead/D3. A narrower delivery contract would require an explicit scope decision.

### B2 — P2: the per-function numerical expectations were recorded from the implementation being tested

**Location:** `conformance/cases/authority/float_math.rs:2–4`, `:70–151`, `:250–420`; provenance is stated in impl-report.md's vector-recorder check.

**Failure:** the ordinary expected bits for the 36 functions were obtained by running the new Tide wrappers, then pasted into their tests. Those literals protect against later changes, but do not independently establish that the initial implementation computes the intended operation. For example, an initially incorrect `cosh => sinh` mapping could have supplied its own expected literal and passed this recording procedure. The two independently recorded large-angle sine cases do not validate the other kernels or the handwritten compositions. The observer order tables and exact edge expectations are useful independent checks and should stay.

**Smallest fix:** keep the golden regression values, and add independently justified numerical expectations for both widths across the exposed operations, with reproducible provenance. Derive exact composition cases from their stated formulas; obtain nontrivial transcendental cases from an independent reference/oracle and explain any accepted approximation difference. This does not require promising universal correct rounding. Label the existing recorded output as regression evidence rather than independent numerical validation.

### B3 — P2: the documented production build commands cannot locate libm

**Location:** dependency addition at `crates/tide-std/Cargo.toml:10`; affected caller at `README.md:131–147`.

**Failure:** the quickstart extracts only the directory containing `libtide_macros*.so`, then passes that one directory to both driver commands. On the pinned nightly's actual Cargo layout, libm is in a different package output directory. The new transitive dependency must be discoverable when rustc loads Tide std, including for callers that do not themselves use math. P2 reproduced E0463 with the documented dependency set and succeeded after adding libm's directory. The shared test helper fixes this for tests but leaves the human quickstart broken.

**Smallest fix:** collect all required library artifact directories from the Cargo messages and pass each with `--tide-dep-dir`, as the conformance runner and E1 harness already do. Update both binary and test examples and their explanation. No driver API change is needed.

## Non-blocking

### N1 — record the E1 job setting before comparing timed runs

`trials/e1-checked/harness.py:173` and `:487` preserve default 6 and do not alter suites, structure checks, grading predicates, sandbox mounts, or readiness markers. The operator's four-job requirement is a reasonable reason for the edit. It can nevertheless change build wall time and whether a timed compilation succeeds; `environment()` rereads the launching process's setting, and the value is not captured in session configuration. Before the gate, record/freeze the selected job value with the run identity, preferably once in config, so prepare/check/grade cannot silently use different budgets. This is a reproducibility observation, not evidence that the prerequisite probe changed an acceptance result.

### N2 — the two driver profiles reuse a debug-built math dependency

`crates/tide-std/tests/support/mod.rs:9,22` always builds and copies `debug` artifacts. `crates/tide-admit/tests/build.rs:415–424` varies the consumer profile, not the optimization of libm or the concrete float trait implementations compiled into std. The existing tests therefore support “debug and release consumers of debug-built std,” not optimized-backend equivalence. Include an optimized std/backend run when establishing the determinism evidence; do not count the existing profile loop as that proof.

The entry rewrite also leaves the old, unused `run` helper at `crates/tide-std/tests/entry.rs:24–32`; deleting it is optional cleanup and does not affect this verdict.

## Decisions

These are recommendations to the Opus decider, not substitutes for its decisions.

1. **Math dependency: disagree with approval of the implemented configuration for this contract.** Exact version/checksum, no defaults, private adapter, MIT license and zero runtime transitive dependencies are sensible. Rejecting platform libm and unspecified host math is also sensible; a mature library is preferable to casually owning range reduction and approximations. However, the decisive behavioral premise is false (B1), and self-recorded bits are insufficient initial proof (B2). Reconsider a narrowly maintained patch/fork or another verified software backend; update replacement cost and evidence accordingly. Using the same package version on both surfaces is necessary but insufficient.
2. **Value arguments and sealed f32/f64 dispatch: agree.** This fits the reference's provisional signatures and Copy scalars; `Bits=u32/u64` gives the requested result type. The private `sealed` module admits only the two primitives, and orphan rules prevent replacing their implementations downstream. The math trait inherits that restriction. There are no callbacks, capability constructors, arbitrary byte-to-type operations, or authority-bearing implementations. Keep it.
3. **Canonical positive quiet NaNs: agree.** D3 requires a single observed NaN and does not prescribe its sign/payload. `0x7fc00000` and `0x7ff8000000000000` are clear fixed choices. Canonicalizing at the observer/math boundary preserves ordinary native arithmetic and signed zeros. Keep the stated constants and edge tests.
4. **min/max ignoring one NaN: agree.** Reference §1's Rust-like fallback supports this choice where §4.3 does not separately decide NaN handling. The implementation returns the numeric argument for either argument order, canonicalizes two NaNs, and deterministically orders both zero signs. Keep it; no wider NaN-propagation rule is needed.
5. **Canonicalize before total_cmp, all NaNs equal above +infinity: agree as the provisional choice.** §4.3 does not spell out a complete NaN order. D3 does require sign/bit observers to treat all NaNs as one value. Applying Rust's total order to the chosen positive quiet canonical NaN gives exactly this placement and preserves `-0.0 < +0.0`. Retaining Rust's raw negative/signaling/payload order would conflict with canonical observation. Keep the implementation and document this as the selected provisional order, not an independently explicit sentence of §4.3.
6. **Fixed powi and the other compositions: agree with the shape, subject to backend/validation fixes.** `unsigned_abs` handles i32::MIN, and reciprocal-first negative powers avoid positive-power overflow before a representable subnormal answer. Converting the exponent to f32 would lose odd exponents; native powi would surrender the fixed algorithm. The loop is bounded by the exponent's 32 bits. Log, fract and Euclidean arithmetic have direct stated formulas; angle conversion, midpoint and clamp delegate specified basic operations rather than host transcendental precision. Keep these small compositions and validate them independently under B2. Excluding unstable functions and the unrequested classification/byte observers is appropriate.
7. **Cargo-built shared test helper: agree.** Adding a transitive dependency makes the previous direct-rustc std construction incomplete. One locked/offline Cargo build per test process and a flat library directory are reasonable for the two existing consumers. The helper retains failure reporting and closed stdin; the six entry tests retain their sources, compile assertions and runtime assertions. There is no evidence of lost entry behavior or a new production trust boundary. Fix the remaining public caller (B3), and describe the debug-only backend proof accurately (N2); a new general build abstraction is unnecessary.

## Reasoning

The observer implementation meets the requested NaN and signed-zero behavior by inspection and the focused execution. The shared fixture covers sign/payload/signaling NaNs, both argument orders for zero minima/maxima, a 13-value order matrix in each width, every kept method, every exposed math function, and useful domain/subnormal/extreme-exponent cases. The new rejection test isolates all 44 listed banned methods in each width and requires FX0520 for each; the unchanged broad float-denial rules remain in place. P3/P4 corroborate the positive and negative production paths without claiming a full independent rerun.

Production owns the two modules; conformance includes those exact files and pins the same backend/features/checksum. This satisfies the new surface's ownership/signature-equivalence requirement from fx-nkv and avoids two numeric implementations. I found no new authority or Pure hole in the sealed dispatch, and no reason to edit policy.rs or pure.rs for this surface.

The A5 observation supports the bead's narrow requirement that std::float cease being its failure. It does not mean the whole A5 readiness gate passes: the result remains false and its two unavailable-boundary markers remain. The diff contains no held-out task, solution or checked-case changes, and the numerical fixture does not use E1 source. I did not read held-out task code or claim to independently establish the implementer's browsing history.

The minimal approval path is to correct the deterministic backend mechanism, supply independent numerical evidence while retaining the existing execution/admission coverage, and repair the two README commands. The host's passing fixture and the implementer's broader logs are valuable, but cannot override the demonstrated backend difference or turn self-recorded outputs into an independent oracle.

Not checked: no Cargo rebuild/full-suite rerun, exhaustive numerical sweep, optimized backend, or heterogeneous-target execution; copied build artifacts retain their original provenance.
Remaining risk: all-target bit identity is currently contradicted by the alternate backend results. DONE
