Model: claude-opus-5-5 · Effort: medium · Date: 2026-11-09

# fx-u65.1 round 2, Opus narrow re-check

## Verdict

**approve.** B1 is resolved, and so are the taken N3, N4, N5 and N7. The changed lines add no new defect.

## Scope and evidence

- Read: the r2 brief, `fix1-brief.md`, `fix1-report.md`, `git diff baee486 fx-u65.1` (10 files) and all of `crates/tide-admit/src/walk/sinks.rs`. Also the relevant parts of `invocation.rs` (`after_expansion`, `check_expanded`, `after_analysis`) and `conformance/run.sh:94,118,136`. I checked `design/03-authority-concurrency.md` at `a3b0177`, lines 1257-1260.
- Candidate identity: the committed tree `aabea4c` is the tree the author proved. `git diff --binary baee486 aabea4c`, excluding the two new cases, hashes to `6c96bf57…defe8`, the report's pre- and post-suite hash. The two new cases match the report's SHA-256 values.
- `target/fx-u65.1-fix1-conformance.log` shows PASS for `lock_print`, `lock_log_accept`, `sink_library_test_accept`, `sink_user_macro`, `channel_library` and `channel_binary_accept`. The regression log, run before the implementation fixes, records `lock_print` and `sink_library_test_accept` failing. So the new marks discriminate.
- No compiler probes were needed. The author's regression-then-pass pair already covers each finding's shape.

## Per-finding status

- **B1: resolved.** `sinks.rs:106` sets `NestedFilter = nested_filter::OnlyBodies`, and `sinks.rs:108-110` adds `maybe_tcx`. This filter visits nested bodies (closures) but not nested items, so closures inside `f` are linted and nested `fn` items are not. Coverage:
  - `lock_print.rs:10`: the executed iterator closure, `//~ WARN FX0607`.
  - `lock_print.rs:11`: the moved `_later` closure, `//~ WARN FX0607`. This is the accepted lexical reading.
  - `lock_log_accept.rs:8`: the nested `fn unused` stays unflagged.
  - The doc comment at `sinks.rs:99-100` now matches the mechanism.
  - Dedup prevents double reports when the outer `Walk` also reaches the closure.
- **N3: resolved, as decided.** `sinks.rs:8-11`: `library_target` now reads `tcx.crate_types()`, which is `[Executable]` under `--test`. So FX0508 and FX0608 skip the test harness, and FX0507 and FX0607 do not depend on it.
  - A library's non-test code still gets both lints: the runner (`run.sh:118`) and E1's `./check` (protocol line 86) both also build the library itself, and results are unioned.
  - `sink_library_test_accept.rs` failed before the fix (FX0508 and FX0608 were emitted) and passes after it.
  - `conformance/README.md:29-30` records the rule.
  - No stale comment about `sess.opts.crate_types` remains (grep is clean).
- **N4: resolved.** `conformance/rules.md:112` cites `03:1259`. At `a3b0177`, line 1259 is "unbounded `channel` in a library is lint `FX0608`", one line after SEND-3's `03:1258`.
- **N5: resolved.** `conformance/README.md:109` says "error or rustc marks", which matches `run.sh:136`'s `$want$errors`.
- **N7: resolved, with an accepted deviation.** The admission gate rejects user `macro_rules!` with FX0101, so the case cannot be fully accepted. Expecting only FX0101 still non-vacuously guards against false positives:
  - `after_expansion` (`invocation.rs:53-58`) runs `check_expanded`, which emits FX0101 without aborting, and then runs `check_crate`.
  - So `check_expansions` and `LockPrints` do walk `a.with(|n| println!("{n}"))` at `sink_user_macro.rs:8`.
  - A wrong macro identity would add FX0508 or FX0607 there, and the case would fail.

## New findings in the changed lines

None that block.

- The fn-pointer arm at `sinks.rs:49-58` fires only when an `FnDef` expression is reified, so direct calls and the untyped `make` alias are not reported twice. The pointer's later call (`channel_library.rs:11`) has an `FnPtr` callee and stays silent.
- The empty `args` in that arm makes the `with` check return early, which is correct.
- Passing `channel::<T>` as an uncoerced generic value (my round-1 N2 second shape) is still not caught. That is the coordinator's chosen scope, so it is noted here, not raised as a defect.
- The `Box` unwrap at `sinks.rs:81-87` stops at `Box<dyn …>`, which is correct.
- Nit, non-blocking: `README.md:29` could name the two library-only lints (FX0508, FX0608) instead of "Library-only lints".

DONE
