Model: claude-opus-5-5 · Effort: low · Date: 2026-11-10

# fx-dsa round-2 Opus re-check

Fix commit `4523c2f` over candidate `cebd0dc` (`git diff cebd0dc 4523c2f`). Scope: my own round-1
findings only. The re-check was read-only, by reading; I ran no probe.

## Verdict

`approve`. Every one of my findings is resolved. The fix diff introduces no new blocking finding,
and nothing at the trust boundary widens.

## Per-finding status

1. **Resolved.** Paired-crate impls are now treated as local through one crate identity.
   - **One resolution, shared by both checks.** `Admission::after_expansion` resolves
     `test_library` once, by an exact `crate_extern_paths` match on the staged archive path
     (`invocation.rs:64`). It passes the same `CrateNum` to `check_crate` and
     `check_mono_items`. `Walk::owns_impl` (`walk.rs:272`) is the one predicate that
     `admit_impl` and the mono backstop (`mono.rs:59`) both use.
   - **Nothing widens.** `owns_impl` is `is_local() || test_library == Some(krate)`.
     - `is_local()` is equivalent to the old `role == Role::Local`, because `Crates::role` maps
       only `LOCAL_CRATE` to Local.
     - In mono, the old test was on the method's crate and the new one is on the impl's crate.
       These are the same for a trait impl method.
     - `test_library` is `None` in single-crate builds and in the library's own session, which
       passes `library_artifact = None`.
     - A path mismatch still leaves the candidate as `Role::Other`, which fails closed.
     - By the orphan rule, a paired-crate impl is always of a local trait or for a local type,
       and the library session admitted its body.
     - Other upstream impls still go through the policy key, then the trait+head test.
   - **Downstream clause checks stay active.** In `obligation`, `where_clauses(impl)` are still
     pushed as nested obligations after `admit_impl` returns Free (`walk.rs:~1298`). In mono,
     the `item_clauses` block is independent of the `owns_impl` gate and still covers
     `Role::Other` items, including impl methods, through their trait method's clauses. The new
     test `separate_library_tests_accept_local_impls_and_check_pure_at_use` covers both sides:
     - The `Shape for fn()` case matches `granted("impl * for fn()")` (`policy.rs:83`). It would
       have been rejected before the fix, and it now links and runs without a grant, through both
       the walk and the inline-method mono path.
     - `require_pure::<Clock>` still gets FX0517 at `demo.rs`, and no executable is produced.
   - **Side effect, safe.** The mono `Walk` now also carries `test_library`, so any `admit_def`
     it reaches treats candidate definitions as the walk does. That is a consistency fix, not a
     widening.

2. **Resolved.** Staging now has a unique name and a bounded retry.
   - **Name and creation.** `create_test_staging` (`invocation.rs:463`) names staging
     `.tide-test-{pid}-{nanos}-{attempt}` and creates it with `DirBuilder::create` at mode 0700.
     There is no `create_dir_all` on the leaf, so a planted dir or symlink gets EEXIST.
   - **Retry.** It retries only on `AlreadyExists`, up to 16 attempts with distinct names. Every
     other error returns at once.
   - **Errors and docs.** Errors name the output location. README.md:167 and main.rs:28
     document the leftover `.tide-test-*` that a killed run can leave.
   - **What a collision does.** Retrying never follows a symlink, and a reused sandbox pid now
     only costs a retry instead of exit 2.

3. **Resolved.** The invocation tests now assert exact messages.
   - **Exact stderr.** `tests/invocation.rs:50` compares stderr exactly
     (`tide-admit: <message>`). The cwd-package output error can no longer satisfy it.
   - **New cases.** It covers missing `--test`, a non-`.rs` source, the reserved
     `--crate-name=tide_candidate`, `--tide-grant=unsafe` and `--tide-test-backstop`. The
     messages match `parse` (`invocation.rs:214-232`).
   - **Cross-read regression.** The test at `tests/build.rs:115` checks both directions: exit 1,
     FX0520, and the canary absent from the diagnostics. The canary check is the binding proof.
     If either read succeeded, `compile_error!` would print the file's text during expansion.
   - **Note, not blocking, and older than this diff.** In the suite session,
     `Sources::read_file` returns `Ok("")` for a path that is an imported source file of a loaded
     crate. Once the suite decodes candidate spans before the `include_str!`, that includes the
     candidate's own `lib.rs`. A suite read of the library source can then yield an empty string
     rather than FX0520. Nothing leaks either way, but the FX0520 assertion in the
     suite→library direction relies on lazy source-map import order. The canary assertion does
     not.

5. **Resolved.**
   - `harness.py:271` writes `extern crate tide_candidate as solution;`, which enters the extern
     prelude, and `harness_test.py:359` checks it.
   - The new build test imports `solution::Shape` from a nested module, which proves nested
     resolution.
   - The reported reference run (A1–A8 and A10 pass L) shows the suites still compile with it.

6. **Resolved.**
   - `readiness.md:49` scopes the stage keys, including A, to 31a9800.
   - `:107` restates fx-dsa's ownership as of 31a9800 in the past tense.

7. **Resolved.** `main.rs:37` adds `tide-admit --test --tide-test-library SOURCE.rs [options]
   <suite.rs>`, and `:26-28` describe staging accurately for both modes.

4. This was an acceptance-record note for the coordinator; no re-check was needed. The fix
   report's table gives C6's reading: its library compiles, and the suite's `#[sim_test]` and
   `std::sim` belong to fx-nkv.6.

## New findings introduced by the fix diff

None that block. The only observation is the order-dependence note under finding 3, which comes
from older loader behavior and leaks nothing.

DONE
