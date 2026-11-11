Model: claude-opus-5-5 · Effort: low · Date: 2026-11-10

# fx-dsa round-1 Opus review

Candidate: `cebd0dc` on `8b0e134`. I recomputed the diff's SHA-256 and it matches:
`ff2c7283e9d7fe82bcf170f6547a3bf89da9993d39922e9324f0c12b0280865b`. The review was read-only.
I ran no probe because the 1-minute load was 15.4, above the gate of 12. Findings marked
"by reading" are not executed.

## Verdict

`approve_with_alternatives`

I found nothing blocking. The trust boundary holds where it matters, and every gap I found
fails closed:

- **Candidate code never becomes a trusted input.** `std` and `--tide-dep-dir` are checked
  against both packages (`invocation.rs` `trusted_inputs`). The output directory may not sit
  inside the library package or any dependency directory (`compile_with_library`). The paired
  path never reads or writes `.tide-cache`. It writes no `interface.json`
  (`write_interface=false`), and the suite gets the candidate only through one exact
  `--extern` path, with no `-L` to staging.
- **Staging is created exclusively.** It is made with `mkdir` at mode 0700 and no
  `create_dir_all`, so a planted dir or symlink fails with EEXIST. It is removed on every
  normal and error path.
- **Each crate reads only its own sources.** The library and the suite each get their own
  `Sources` snapshot. Their packages are lexically and canonically disjoint, and
  symlink-traversing package paths are rejected.
- **The library is built without `--test`,** so its `cfg(test)` code is off. The test proves
  this with `compile_error!`.
- **The walk matches only the exact loaded rlib path.** A mismatch leaves the candidate as
  `Role::Other`, which is rejected, so it fails closed.
- **Candidate macros are excluded from the exception.** Re-exports keep their defining crate.
- **Downstream generics stay checked.** The mono backstop still covers candidate generics
  instantiated in the suite: `Role::Other` items get `item_clauses`, and non-local impl methods
  get the policy decision.

I checked the harness L stage too:

- **No placeholder remains.**
- **First check uses metadata mode**, which matches Rust's `cargo check --all-targets` over
  `tests/acceptance.rs`.
- **Full grading links and runs the tests.** `test_command` always returns its listing record,
  and a listing that differs from the frozen names fails. A successful link therefore cannot
  pass as executed tests.

**The recorded reference results match the report.** I read
`target/e1-checked/tide-results.json`:

- A1–A8 and A10 pass L with full frozen listings.
- A9, B3, B4, B6, B7, B9, B10, C1–C5 and C7–C10 fail L on the mapped missing-std or attribute
  errors shown in the report table.
- C6 fails on the suite's `sim_test` (see finding 4).

## Findings

1. **non-blocking: candidate impls are keyed by policy, while the same impl in a local crate
   is free.** Found by reading.
   - Where: `walk.rs:281-296` (`admit_impl`) and `walk/mono.rs`, the non-local impl-method
     check (`role != Role::Local && trait_impl_of_assoc`).
   - What happens: `admit_def` now treats the paired library like the local crate, but
     `admit_impl` and the backstop still treat a candidate impl as foreign. They look up its key
     in `policy::RULES` before the trait+head test.
   - Failure: say the candidate writes `pub trait Shape {} impl Shape for fn() {}` and the
     suite uses `fn(): Shape`. The key `impl tide_candidate::Shape for fn()` matches
     `granted("impl * for fn()")` (`policy.rs:83`), so the suite is rejected without a grant.
     The same code in one crate, and the Rust arm, pass. The library session admitted that
     impl's body as local, so the rejection is spurious.
   - Impact: it fails closed and is exotic, but it is a §11 asymmetry at the exact boundary
     this bead owns.
   - Fix: keep one owner for "the admitted paired crate". Treat
     `impl_def_id.krate == test_library` like `Role::Local` in `admit_impl`. Pass the same
     `CrateNum`, or the path, to `check_mono_items` and skip the policy decision there for that
     crate. Add one synthetic regression.

2. **non-blocking: the staging name relies on a unique pid. A killed run leaves a candidate
   rlib behind and blocks the next run in that directory.**
   - Where: `invocation.rs`, `compile_with_library`, `self.out.join(format!(".tide-test-{}", std::process::id()))`.
   - Background: the harness runs the driver under `bwrap --unshare-all`, which gives it a new
     pid namespace, so the clean child's pid is the same small number on every sandboxed run.
     Exclusive creation keeps this safe, since a collision fails rather than reusing a
     directory.
   - Failure: the harness SIGKILLs the process group on timeout. That leaves
     `out/.tide-test-<pid>/libtide_candidate.rlib`, so a library is published after all,
     against the doc's "publishes no library" in this edge case. The next run in the same
     output directory then exits 2 with a bare `File exists (os error 17)`.
   - Fix: give staging a unique name, for example pid plus a random nonce, retrying on EEXIST.
     Add context to the error. Document that a killed run may leave a `.tide-test-*` directory
     that nothing reads.
   - Related, out of scope: the single-crate cache path's `{key}.{pid}` stage, which runs
     `remove_dir_all` before create, has the same pid-namespace assumption. Report it, don't
     fix it here.

3. **non-blocking (test strength): the new invocation cases would pass even without the new
   checks.** The trust boundary's own tests also miss one of the boundary's claims.
   - Where: `tests/invocation.rs:38-45`.
   - What happens: with cwd = the crate directory, `demo.rs` resolves its package to cwd. The
     default `--out-dir .` then fails with "output and cache must be outside the candidate
     package", exit 2, even if the new `parse` validations in `invocation.rs:204-224` were
     deleted. The test asserts only the exit code.
   - Fix: assert the specific messages:
     - "requires `--test` and full admission without grants"
     - "requires a .rs source"
   - Add the reserved crate-name case (`--crate-name=tide_candidate`) and a
     `--tide-test-backstop` combination.
   - Also: `tests/build.rs:55-107` proves visibility, cfg(test), unsafe in either crate and the
     dependency-directory refusal. It does not prove that neither crate can read the other's
     sources.
   - Fix: add one synthetic case where the suite runs `include_str!` on the library's source
     path, and the library does the same on the suite's. Expect FX0520 for both. Finding 1's
     regression can share this fixture.

4. **non-blocking (Acceptance record): C6 passes its library compile but fails L on the
   suite's mapped prerequisite.**
   - The Acceptance reads: the L stage passes for every reference that compiles, and the
     others fail only on their own mapped compile errors.
   - C6's library compiles, but its L fails on the *suite's* `#[sim_test]` and `std::sim`
     (E0433), owned by fx-nkv.6. That is a mapped prerequisite (readiness.md lists the C3–C6
     simulated suites under fx-nkv.5/fx-nkv.6), not a defect at the boundary.
   - Judgement: Acceptance is met. Nine pass, more than the four named, because the std grew,
     and every failure has a mapped owner.
   - Fix: record the C6 reading explicitly on the bead when closing, so later readers do not
     read it as a weakened check.

5. **nit: the suite import is a crate-root `use` alias, not an extern-prelude name.**
   - Where: `harness.py:271`, `use tide_candidate as solution;`.
   - Difference: Rust's `tests/acceptance.rs` sees `solution` in the extern prelude, so
     `::solution::X` and a nested module's `use solution::X` resolve there. In Tide they
     resolve only at the crate root.
   - Impact: latent only. I grepped `suites/`; none uses nested modules or `::solution`.
   - Fix: write `extern crate tide_candidate as solution;`, which enters the extern prelude.
     The admission walk places no restriction on `extern crate`. That gives full parity before
     the suites freeze.

6. **nit: readiness.md has two stale lines.**
   - `readiness.md:104` still says "fx-dsa owns separate admitted-library import and blocks
     fx-ic2", in the present tense. The new sentence at `:128` correctly ties the L marker to
     the recorded revision.
   - The stage key "A = linked acceptance-harness build" (`:50`) no longer exists as a separate
     Tide stage for libraries.
   - Fix: qualify both as describing 31a9800, as `:30-33` already does for counts. Optionally
     note that fx-dsa supplied L once it closes, as `:106` does for fx-kw8.9.

7. **nit: the usage line omits the new flag.** `main.rs:35` does not list
   `--test --tide-test-library SOURCE.rs`. The module doc above it does describe the mode.
   Fix: add the flag to the usage line.

## Checked and fine

- **Argument parsing.** The flag is rejected when given twice, empty, starting with `-`,
  without `--test`, with a grant, with `--tide-test-backstop`, with `--tide-test-ice` or with
  `--tide-compile`. A non-`.rs` path and the reserved crate name are also rejected. Argument
  errors exit 2 and compile errors exit 1.
- **Single-crate refactor.** Behavior is preserved. The only change is the order of `rustc_args`
  in link mode (`-Ztemps-dir` now precedes `-C linker`), which invalidates old link cache keys
  but is harmless.
- **Two `run_compiler` sessions in one clean process.** Each has its own loader and admission
  state, and rustc panics and fatal errors are caught per session.
- **Docs.** README.md, `trials/e1-checked/README.md` and `main.rs` match the behavior.
- **Held-out rule.** Driver tests use synthetic `pigment` code. No E1 task, case or reference
  text appears in driver tests, docs or diagnostics. `harness_test.py`'s use of `h.suite` is
  the trial's own infrastructure and was there before.

DONE
