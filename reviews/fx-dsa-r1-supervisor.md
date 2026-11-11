Model: claude-opus-5-5 · Effort: high · Date: 2026-11-10

# fx-dsa round-1 supervisor pass

Candidate: `.worktrees/fx-dsa`, `cebd0dc` on `8b0e134`. I recomputed the diff SHA-256 and it matches (`ff2c7283…865b`). This pass was read-only. I ran no probe because the 1-minute load was 13.7, above the gate of 12. The findings below come from reading the code, its callers and the tests. I did not re-run the author's proof.

## Verdict

**approve_with_alternatives**

Nothing blocks. The trust boundary holds:

- **No path for candidate code into a trusted role.** The library comes only from a `.rs` source (`invocation.rs:215`, tested with `libfake.rlib`). It is admitted without grants, backstop-only, test-ICE or `--tide-compile` (`:204-213`). It is compiled into a staging directory that is created exclusively with mode 0700 inside the canonical out-dir. The out-dir must lie outside both packages and every dep dir (`:418-423`). The suite never gets that directory as a search path. The rlib is imported through one exact `--extern` path. The pair never reads or writes `.tide-cache` and writes no `interface.json` (`write_interface: false`). `remove_dir_all` does not follow symlinks.
- **Exact-path recognition fails closed.** `walk.rs:79-85` matches `crate_extern_paths` against the exact staging path. If it found nothing, every `tide_candidate` use would be FX0520. Re-exports keep their defining crate's DefId, so a library `pub use` of a std item or a dep-dir item is still judged by that crate's role. The exception covers `admit_def` only. It excludes `DefKind::Macro`, and the library cannot define macros anyway (FX0101).
- **Downstream use is still checked.** Library impls still go through `admit_impl`'s key and head checks. In `mono.rs`, test-library items are `Role::Other`, so the suite's backstop checks impl keys (`mono.rs:54`) and `Pure` item clauses (`Local | TideStd | Other`). `use_generic` and `clause_obligations` treat a library function like a local one (`Need::Bound`). The driver keeps no process-global caches (I grepped for statics, OnceLock and thread_local), so running two rustc sessions in one process cannot carry DefIds from one session into the other.
- **Neither crate can read the other's sources.** Each compile has its own `Sources` snapshot. The library compiles before the suite, and the suite's path is not in its loader, so a read attempt fails with FX0520. When the suite asks for a library file by its imported path, the loader's `foreign` branch returns an empty string, never the contents. The packages must be disjoint in both directions (`:414`). A symlinked library source that leaves its src directory is rejected by `source_paths`.
- **Only the public interface.** rustc's privacy rules apply, and the regression test asserts E0603 for `pub(crate)` in both emit modes. The library builds without `cfg(test)` (the `compile_error!` in its `#[cfg(test)]` module proves it). So the test listing holds only suite tests, which also closes fx-o8o r3's note about writer tests appearing in the listing.
- **Harness L stage.** First check runs L in metadata mode. Full grading links the suite and runs it through `test_command(acceptance=True)`, which still requires the frozen name list and the logfile verdicts. If the build fails, no test runs (`harness.py:277-280`). The placeholder record and the `acceptance::` prefix are gone, and no leftover remains in code, tests or docs. This matches the Rust arm: `cargo check --all-targets` covers first check, `cargo test --test acceptance` covers final grading, and test names are crate-root `caseN` in both arms. `use tide_candidate as solution; use solution::*;` gives the suite the same paths as Rust's extern crate `solution`.
- **Acceptance is met, and more.** L passes for 9 references: A1–A8 and A10, a superset of the A2/A3/A4/A10 the bead names, because the base has gained std surface since. 16 fail while compiling their own library, each on an open owner bead; I checked that each one exists. C6 fails on the suite's `#[sim_test]`/`std::sim`, which belong to fx-nkv.6, a prerequisite readiness.md already names. That counts as "fail only on their own mapped compile errors". The pub/pub(crate) driver check uses synthetic code, so it respects the held-out rule. No E1 material reached driver tests, docs or diagnostics.
- **readiness.md stays honest.** The edit keeps the 31a9800 table and labels it as describing the recorded revision. The current-state sentence is accurate.

## Findings

1. **non-blocking: a leftover staging directory permanently blocks reuse of an out-dir under the sandbox's PID namespace.**
   - **Evidence.** `invocation.rs:434-442` names the directory `.tide-test-<pid>` and creates it exclusively. A killed run (the harness SIGKILLs the process group on timeout) leaves that directory behind. Inside bwrap `--unshare-all` the driver's PID is small and the same on every run. So every later paired build into the same out-dir fails with `File exists`, exit 2, until someone deletes it by hand. The ordinary path removes leftovers for exactly this reason (`:977`). The current harness always uses fresh work trees, so no E1 run hits this today.
   - **Fix.** Keep exclusive 0700 creation but add a nonce to the name: PID plus nanoseconds, or retry with a counter on `AlreadyExists`. Do not reuse or clear an existing directory, which would weaken the exclusivity argument.

2. **non-blocking (test strength): no regression test covers a candidate generic or impl that the suite instantiates.**
   - **Evidence.** `tests/build.rs:56` proves visibility, `cfg(test)` exclusion, and FX0901 from a direct `unsafe` in either crate. The new trust-relevant line is `walk.rs:259-261`, which admits every library DefId. No test shows that a library item's where-clauses or impls still produce obligations at the suite's use sites or in its backstop. I traced the code and found them enforced (see above), but nothing pins it.
   - **Fix.** Add one synthetic case against production std, which the test already uses: a library `pub fn f<T: Pure>(t: T)`, or a generic whose instance fails a `Pure` clause, called from the suite with a non-`Pure` type. Expect the suite's FX0517 or the code `clause_obligations` gives.

3. **non-blocking: library impls are judged by policy key in the suite, but admitted outright in the library.**
   - **Evidence.** `admit_impl` (`walk.rs:276`) and `mono.rs:54` treat `Role::Local` impls as free. The test library's impls are `Role::Other`, so the suite looks up their keys. A writer's `impl fmt::Pointer for T` or `impl MyTrait for fn()` passes when the library compiles. The suite's use then hits `granted("impl core::fmt::Pointer for *")` or `granted("impl * for fn()")` and is rejected without a grant, though the same use inside the library is admitted.
   - **Risk.** This fails closed, so trust is not weakened. It is a small Tide-side asymmetry that §11 does not list, and no E1 reference hits it.
   - **Fix.** Either record it as a known limit or treat the paired library's impls like the library's own verdict. In `admit_impl` that means the `Local` early return. In `mono.rs` it means keeping the key check, since the library's walk already decided them.

4. **nit: the main.rs summary undersells the cost of metadata mode.** "Metadata mode checks the same import without linking" (`main.rs:27`) is true of the suite only. The library is always built as a full rlib, with codegen, because the backstop needs its MIR. Say so, as README.md already does ("builds a fresh library archive").

5. **nit: two text leftovers.**
   - `readiness.md:107`: "fx-dsa owns separate admitted-library import and blocks fx-ic2" stays in the present tense and becomes false once fx-dsa closes. Tie it to the recorded revision as line 128 does.
   - `trials/e1-checked/README.md:113`: the short line "names. Every expected" needs reflowing.
   - When the bead closes, record the new per-reference L table on fx-ic2. readiness.md does not carry it.

DONE
