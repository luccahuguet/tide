Model: claude-opus-5-5 · Effort: converge round 1 (report-only) · Date: 2026-11-09

# fx-yg5.2 round 1: Opus review

## Verdict

**request_changes**, confidence 0.8.

The float observers, the libm choice, the admission behavior, and the production/conformance sharing are sound. Most of the Acceptance is met on this host. Two findings block landing:

- **B1.** `math::powi` takes the reciprocal before squaring. That is measurably less accurate than Rust's own `powi`, and it gives a different answer on common inputs (`powi(10.0, -2)`).
- **B2.** libm is the first registry dependency of the root workspace. That breaks the E1 harness's isolated `--offline` build of the production std on a clean state. The proof only passed because of a manual cache copy that nothing in the repository encodes.

## Scope

Read:
- AGENTS.md and the GOVERNANCE.md Merge Rule, via the brief.
- The bead `fx-yg5.2`, the implementer's report, and the full `git diff 07e104b 0823998`.
- `crates/tide-std/src/{float,math,lib}.rs`, `tests/support/mod.rs`, and `tests/entry.rs`.
- `crates/tide-admit/tests/build.rs` and `crates/tide-admit/src/policy.rs` (float rules), plus the `tide_std` uses in `pure.rs` and `crates.rs`.
- `conformance/rustc/std/{Cargo.toml,src/lib.rs}` and `conformance/cases/authority/float_math.rs`.
- `trials/e1-checked/harness.py`: `environment`, `rust_vendor`, `base_config`. Also `protocol.md` §10.
- `design/reference.md` §4.3 and `design/01-design.md` D3.
- Pinned libm 0.2.16 source: `configure.rs`, the `select_implementation!` macro, `fma.rs`, `exp*.rs`, `rem_pio2.rs`, `sincosf.rs`.
- From the pinned nightly-2026-08-09 rust-src: `core/src/num/f32.rs` and `f64.rs` (`midpoint`, `to_degrees`). From the nightly-2026-08-20 rust-src: `compiler-builtins/src/float/pow.rs`.
- The implementer's logs: `final-workspace.log` (all suites ok), `final-conformance.log`, `final-a5.log`, and the worktree's `target/e1-checked/tide-results.json`. The A5 records show code 0 everywhere except the two known code-2 markers (runner unavailable, library boundary). There are no float diagnostics.

Ran: no `rustc` or `cargo` probes. Two Python scripts, which are independent of the code under test, kept in my scratchpad:
- `chk.py` parses every `float::bits(math::f(literal…)), 0x…` vector in the fixture and recomputes it with CPython's `math` module, which uses host glibc. f32 values are computed in double and then rounded.
  - Every one of the 31 f32 and 31 f64 nontrivial vectors matches the host bit for bit or is within 1 ulp. The 1-ulp cases are f32 tanh/asinh/acosh, f64 atanh, and f64 log(3,2).
  - The round_ties_even(-0.5) entries agree once the sign of zero is accounted for. Python's `round` drops the sign.
  - So every function maps to the right kernel (exp_m1→expm1, ln→log, ln_1p→log1p, abs_sub→fdim, round_ties_even→roundeven), and the values are plausible.
- `powi.py` compares reciprocal-first with reciprocal-last against exact rationals (`fractions.Fraction`), for x ∈ {3, 7, 10, 1.1, 1.5, 2.5, 0.3, 9.81, 12} and n = -1..-39. Results are under B1.

## Blocking

**B1. `math::powi` uses reciprocal-first, which is less accurate than Rust's `powi` and differs from it on everyday inputs.**
`crates/tide-std/src/math.rs:38-49` (`if n < 0 { base = 1.0 / base; }`), with the vectors at `conformance/cases/authority/float_math.rs:148,184,412` and the f64 `powi(2.0, -1074)` line.

Failure scenario:
- `math::powi(10.0f64, -2)` returns `0.010000000000000002`. Rust's `10f64.powi(-2)` returns `0.01`.
- Likewise `powi(10, -5)` gives `1.0000000000000006e-05` and `powi(10, -20)` gives `1.0000000000000022e-20`.
- The rounding error of `1/x` is amplified roughly |n|-fold. Over the 351 cases in my sample:
  - reciprocal-first: worst error 31 ulp, 298 inexact results;
  - reciprocal-last: worst error 5 ulp, 115 inexact results.
- Decimal scaling (`x * math::powi(10.0, -d)`) is exactly what agent-written E1 code does. It would silently get worse answers than in Rust, and the recorded vectors would lock that behavior in as Tide's contract.

Rust's `powi` is not "unspecified" in practice:
- LLVM's constant-exponent expansion and compiler-builtins' `__powisf2`/`__powidf2` both compute `1/(x^|n|)`. See `compiler-builtins/src/float/pow.rs:4-21`: `if recip { F::ONE / mul } else { mul }`.
- So reciprocal-last is also the composition that matches Rust's observed results on every target. That fits reference §1's "otherwise unchanged Rust APIs" better.
- What reciprocal-first gains is only results near the subnormal boundary, where `x^|n|` overflows (`powi(2.0, -1074)`). That trades common-case accuracy for an edge case.

Smallest fix:
- Use compiler-builtins' order: square-and-multiply on `x`, then `if n < 0 { 1.0 / result }`.
- Re-record the four negative-exponent vectors. `powi(2.0f32, -149)` and `powi(2.0f64, -1074)` then become `+0.0`, matching Rust.
- Optional: keep the subnormal benefit deterministically by falling back to reciprocal-first only when `x` is finite and nonzero, `n < 0`, and `x^|n|` is infinite. If you do this, keep vectors for both paths.

**B2. The E1 harness can no longer build the production std from a clean state.**
`trials/e1-checked/harness.py:478-495` (`base_config`, Tide arm).

What changed:
- `base_config` builds `tide-admit` and `tide_std` with `cargo build --offline --locked` under an isolated `CARGO_HOME=target/e1-checked/cargo-home`.
- Before this change the root workspace had zero registry packages: `git show 07e104b:Cargo.lock | grep -c registry+` prints 0. With `libm` it has one.

Failure scenario:
- On a fresh clone, or after `target/` is cleaned, every Tide-arm harness action fails inside `base_config` with Cargo's offline "cannot download libm" error. That includes `references tide <task>`, the §10.2 readiness runs, and the gate run.
- The implementer's A5 proof passed only because they copied the libm tarball, index entry, and extracted crate by hand into the ignored `target/e1-checked/cargo-home`. Their Gaps section says so.
- The main checkout's `target/e1-checked` does not have that copy. Protocol §10.2 item 2 and the reproducibility of §5/§6 runs depend on a step no file records.

Smallest fix:
- Give the Tide arm the same pinned offline setup the Rust arm already has. Generalize `rust_vendor()` to take a manifest and lock, or add a sibling helper that runs `cargo vendor --locked --offline` on the root manifest into `target/e1-checked/tide-vendor-<lock-hash>`.
- Build with `--config source.crates-io.replace-with="vendored"` and `--config source.vendored.directory=<dir>`.
- Use the same one-time remedy message as the Rust arm (`cargo +nightly-2026-08-09 fetch --locked`).
- Add one line to `trials/e1-checked/README.md` beside the Rust setup, and one `harness_test.py` case mirroring `test_offline_vendor_setup_is_pinned_cached_and_cleans_failure`.

## Non-blocking

**N1. The vectors are self-recorded. Say so, and record the independent cross-check.**
- Apart from `sin(1e20)`, the recorder computed the literals with the code under test. That is acceptable for what the Acceptance asks: "match recorded vectors bit for bit" is a regression and cross-target falsifier.
- It proves nothing about correctness by itself, though. My glibc cross-check above (all within 1 ulp) is the missing independent evidence.
- Suggest a fixture comment like "cross-checked against host glibc within 1 ulp (f64 and f32), 2026-11-09", and the same note on the bead.
- Execution on any target other than x86_64-linux remains unproven, as the report says.

**N2. Record the x87 caveat for "bit-identical on every target".**
- With `force-soft-floats`, the libm 0.2.16 code paths are target-independent, with one exception: `use_arch_required: x86_no_sse` in `exp.rs:88`, `exp2.rs:329`, `expf.rs:37`, `exp2f.rs:80`, `exp10*.rs`, `floor.rs:18`, and `ceil.rs:32`. These choose x87 code regardless of `force-soft-floats` (see the `select_implementation!` docs, `support/macros.rs:48-50`).
- On those targets, which are i586-class, Rust's own f32/f64 arithmetic is not IEEE-reproducible either. D3's "arithmetic stays native" premise already fails there.
- Suggest stating it as a precondition in the `math.rs` module doc and on the bead: bit identity holds wherever Rust's f32/f64 arithmetic is IEEE binary32/binary64, which covers SSE2 x86, aarch64, riscv with d, wasm, and others. This change does not need to edit D3. Whether D3 should name the restriction is a separate design question.

**N3. `midpoint`, `to_degrees`, `to_radians`, and `clamp` delegate to Rust core rather than to the one pinned library.**
- `core::f32::midpoint` is `cfg_select!`ed per target (`core/src/num/f32.rs:1193-1222`): an f64-widening branch on x86_64, aarch64, wasm and others, and an f32 `(a+b)*0.5` branch elsewhere.
- By my analysis both branches yield the correctly rounded midpoint, so results probably agree. Even so, the determinism claim then rests on core's per-target code and on two different core versions: conformance compiles with stable 1.96, production with nightly 1.99.
- Cheap hardening: spell the compositions out in `math.rs` (f32: `((a as f64 + b as f64) * 0.5) as f32`; f64: core's HI branch; and the degree constants). Then the "one library" claim covers them too.

**N4. Make `tests/support/mod.rs` sturdier.**
- `copy_libraries(dir/debug)` assumes every artifact lives under `<target-dir>/debug`. A Cargo `build.build-dir` config placed outside the target directory would hide `libm`.
- The flat `artifacts` directory is never cleared between runs. After a dependency bump, a stale `libm-<oldhash>.rlib` stays in it. rustc's SVH match will most likely ignore it, but the directory only grows.
- The old builders compiled std hermetically: build.rs used `env_clear()` and the `TIDE_SYSROOT` rustc. The new helper inherits the test's environment, including any `RUSTFLAGS`.
- Suggest: take the filenames from `--message-format=json`, as `harness.py` already does. A substring scan for the `"filenames":[` array is enough, with no parser dependency. Also clear `artifacts` before copying, and pass `env_remove("RUSTFLAGS")`.
- Behaviorally, entry.rs keeps what it proved: same std crate, edition 2024, dev profile overflow checks, and the six entry tests pass.

**N5. Scope `harness.py`'s `CARGO_BUILD_JOBS` override.**
- The override is also applied in `environment()`, which is the writer/grading sandbox environment. That lets an unrecorded operator variable into trial runs.
- It changes no grading or result, and I see no §10 violation. Still, limit it to the trusted `build_env` (line 487), or record its value in the results. Then the harness identity (§10.2 item 6) stays complete.

**N6. `float::Float::Bits` has no bounds** (`float.rs:13`).
- Generic code cannot compare, hash, or print `float::bits(x)`.
- If generic callers need it, add `Copy + Eq + Ord + core::hash::Hash + core::fmt::Debug`. The signatures are provisional either way.

**N7. Stable float methods with no Tide replacement.**
- `next_up` and `next_down` (stable since 1.86) fall under the `std::f*::<impl f*>::*` deny, but have no replacement and no FX0520 test. So do the classifiers `is_infinite`, `is_normal`, `is_subnormal`, and `classify`.
- This is outside the bead's scope, since §4.3 marks classification provisional. Report it as a later allowlist/replacement decision (for fx-djp, or fx-yg5.5's allowlist review).

## Decisions

1. **libm =0.2.16, `default-features=false`, `force-soft-floats`: agree.**
   - `configure.rs:85-100` shows that `force-soft-floats` overrides both `arch` and `unstable-intrinsics` under feature unification. Pinning the exact version plus both lockfile checksums fixes the bytes.
   - The alternatives were rejected for sound reasons: host methods have unspecified precision; FFI and MPFR bring platform variation and weight; an owned port carries maintenance cost.
   - Determinism holds on every target with IEEE f32/f64 arithmetic. It does not hold on x87-only targets (N2), which is a Rust-wide limitation, not libm's.
   - The record satisfies AP-DEPENDENCY-001: capability, rejections, pin, replacement cost (`math.rs` plus manifests), and relied-on behavior proved by vectors. Post it on the bead together with the N1 and N2 notes.
   - On whether the vectors are independent: not by construction (N1). My independent glibc comparison supports them.
2. **Value arguments and sealed `f32`/`f64` dispatch: agree.**
   - The traits are sealed, so no user impl can bring authority or state.
   - `f32` and `f64` are `Pure`, and the free functions and hidden trait methods key as `tide_std::…` items.
   - Method syntax on a concrete float still resolves to the inherent method, so the FX0520 tests stay meaningful.
   - Calling the hidden methods via UFCS gives exactly the free functions' semantics. That is no hole.
3. **Canonical positive quiet NaN (0x7fc00000 / 0x7ff8000000000000): agree.** It equals Rust's `f32::NAN`/`f64::NAN` bits and is the narrowest reading of D3. Normalization happens only at observation and the math boundary.
4. **min/max ignore a sole NaN, canonicalize two NaNs, and order -0 < +0: agree.** This keeps Rust's semantics and satisfies D3's zero ordering. The fixture covers all four zero pairs and both argument orders with NaN.
5. **total_cmp canonicalizes and then uses Rust's totalOrder, so every NaN compares equal and above +∞: agree.** This is the only placement consistent with "any NaN reads as one canonical (positive) NaN". The 13×13 ordering matrix in the fixture is a strong test.
6. **powi: disagree.** Use reciprocal-last, as compiler-builtins and LLVM do (B1). The other compositions are fine as chosen: `log = ln/ln`, `fract = x - trunc`, and Euclidean division via `trunc`/`fmod`. Each matches Rust's std composition, and `fmod` is exact.
7. **Shared Cargo build helper instead of guessed artifact paths: agree,** with the N4 hardening.

## Reasoning

**Acceptance.** On this host every Acceptance item is met:
- `bits(NaN)` returns the canonical bits for every sign and payload, including signaling NaNs.
- `min(-0.0, 0.0)` is `-0.0`.
- total_cmp follows IEEE totalOrder, with NaN canonicalized.
- 74 vectors match bit for bit in debug and release, through the driver with the production std.
- Each of the 44 banned methods gives FX0520 in both widths, tested one per driver invocation, so one diagnostic cannot hide another.
- All ten kept methods execute in the fixture.
- A5 shows no std::float failure.
- Production and conformance share `float.rs` and `math.rs` verbatim via `#[path]`. Inherent methods win over the trait methods of the same name inside the impls, so the delegations are not recursive.

**Admission and Pure.** The sealed dispatch adds no admission or Pure hole: nothing outside `f32`/`f64` can implement the traits, and policy.rs's float deny/allow rules are unchanged and still exercised.

**Why the two findings block.**
- B1 is a decision I am asked to make. Its fix is small, but once the recorded vectors land they become Tide's contract, and a later fix would be an observable-result change.
- B2 is a regression the diff itself causes in a shared, reproducibility-critical tool. The candidate's own A5 proof depended on an unrecorded manual step.

Everything else is hardening or recording.

DONE
