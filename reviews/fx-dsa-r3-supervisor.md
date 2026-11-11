Model: claude-opus-5-5 · Effort: low · Date: 2026-11-10

# fx-dsa round-3 supervisor re-check

Scope: only my round-1 findings 1–5, re-checked against `git diff cebd0dc 4523c2f` on branch `fx-dsa`. This pass was read-only. I ran no probe because the code and the author's focused tests settle each point, and I did not re-run the author's proof.

## Verdict

**approve.** All five findings are resolved, and the fix diff introduces no new finding.

## Per-finding status

1. **Resolved.** The staging nonce: `create_test_staging` (`invocation.rs:463`) names each directory `.tide-test-{pid}-{nanos}-{attempt}`. It uses `DirBuilder::create`, which creates the directory exclusively and never uses `create_all` on the leaf, with mode 0700 on Unix. It makes up to 16 attempts and moves on to the next name only on `AlreadyExists`. Any other error returns at once. An existing directory is never reused or cleared. A symlink planted at a candidate name also returns `AlreadyExists`, and mkdir never follows it. Both errors name the output location. If the clock reads before the epoch, the nonce falls back to 0 and the attempt counter still makes the names distinct. Cleanup (`remove_dir_all` after the pair compiles) is unchanged. The leftover behaviour is documented in `README.md:167` and `main.rs:28`.
2. **Resolved.** The suite-side FX0517: `tests/build.rs:175` writes a library `require_pure<T: Pure>` that the library never instantiates. The suite then instantiates it with `std::time::Clock`. The test requires exit 1, a `demo.rs:` line carrying `FX0517: \`Clock\` is never \`Pure\``, and no published executable. That pins the obligation at the suite's use site. The mono `Pure` item-clause block (`mono.rs:89-91`) still covers the library's items (`Role::Other`), because the impl exemption is a separate `if` that does not short-circuit it.
3. **Resolved, and nothing beyond the exact paired crate is widened.** The `CrateNum` is resolved once in `after_expansion` (`invocation.rs:64`), by matching `crate_extern_paths` exactly against the staging rlib path that the suite receives through `--extern`. Both `check_crate` and `check_mono_items` receive that one value in the same `tcx`. `owns_impl` (`walk.rs:272`) is `impl_def_id.is_local() || test_library == Some(krate)`. Since `Role::Local` is exactly `krate == LOCAL_CRATE` (`crates.rs:86`), the first arm is unchanged. In `mono.rs:59`, the old test was `role(method) != Local` and the new one is `!owns_impl(impl)`. These agree because a trait-impl method's crate is its impl's crate. The library compile itself passes `library_artifact = None`, so its own exemption is still `Local` only. Other upstream crates, Tide std and dep dirs included, still have their impl keys checked in both places. The suite now admits the library's impls just as the library's own walk did, and the library's bodies were checked in that walk. Its where-clauses are still checked at the suite's uses (item 2). The `Shape for fn()` regression (`build.rs:142`) links and runs through a trait bound and an `#[inline]` method, so it exercises both the walk and the mono path.
4. **Resolved.** `main.rs:26-28` now says that both modes build a fresh library rlib in private staging and that metadata mode only skips linking the suite.
5. **Resolved.** `readiness.md:107` is tied to `31a9800`, and so are the stage keys at `:49`. `trials/e1-checked/README.md:113` is reflowed with its meaning unchanged. The refreshed per-reference L table is in `fix-r1-report.md`, ready for the coordinator to record on fx-ic2.

## New findings from the fix diff

None. At the trust boundary, the impl exemption is keyed to one `CrateNum`, recognised by exact path, so it fails closed. If no crate matches, `test_library` is `None` and nothing is exempt. Staging creation is still exclusive and private. The other changes in the diff are in my scope only as far as they touch these fixes: the `extern crate` import in `harness.py`, the argument-error tests and the source-read regression. They change no trust decision.

DONE
