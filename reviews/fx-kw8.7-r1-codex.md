Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-01

**Verdict:** `request_changes` — high confidence.

**Scope:** Reviewed fx-kw8.7 against its parent, GOVERNANCE.md, RUN-2, the surrounding invocation code, tests, README, and pinned rustc sources. HEAD is `8673252`; the candidate matches SHA-256 `4ed1225e4b0559a76c2dcc89274ea999688d328c61d94eb31cf07123ce9004f6`. Excluded the specified research and `.beads/` changes.

**Blocking**

1. **An empty environment bypasses ICE suppression.**  
   [invocation.rs:579](/home/lucca/pjs/lab/tide/crates/tide-admit/src/invocation.rs:579)

   `vars_os().any(...)` returns false for an empty environment. A direct internal invocation with `env -i`, a valid `--tide-compile=<matching-key>.<pid>`, and `--tide-test-ice` therefore reaches `run_compiler` without `RUSTC_ICE=0`. On the pinned nightly, that enables reports in the current directory, potentially the candidate package.

   Require the environment to equal the singleton `{RUSTC_ICE=0}`; re-exec when the variable is missing as well as when anything differs.

2. **The original process installs a file-writing panic hook before sanitization.**  
   [main.rs:42](/home/lucca/pjs/lab/tide/crates/tide-admit/src/main.rs:42), [invocation.rs:570](/home/lucca/pjs/lab/tide/crates/tide-admit/src/invocation.rs:570)

   A concrete path is `tide-admit --tide-print-codes` with stdout directed to `/dev/full` and stderr working. `println!` panics before the environment check. The installed rustc hook then creates a report in the current directory when `RUSTC_ICE` is unset, or in an inherited `RUSTC_ICE` directory. Child environment changes cannot protect this parent.

   Move hook installation past successful environment normalization, or install it only immediately before `run_compiler`. Earlier driver failures can use Rust’s default stderr-only panic hook.

**Non-blocking**

[The regression test](/home/lucca/pjs/lab/tide/crates/tide-admit/tests/invocation.rs:386) meaningfully checks failure cleanup and package cleanliness, but starts with a cold cache. It would pass even without the new cache-key field. Warm a successful normal cache entry first, then require the seeded invocation to crash. Also assert `internal compiler error` on stderr: the seeded panic text alone does not prove rustc’s ICE diagnostic remains installed.

**Alternatives**

Retain `RUSTC_ICE=0`. The pinned compiler explicitly supports it, and it suppresses both hook reports and the session’s delayed-bug report destination. Redirecting reports with `-Zmetrics-dir` would still require handling parent panics and report cleanup.

**Minimal patch for approval**

- Require exactly the prescribed singleton environment.
- Install the rustc ICE hook only after that invariant holds.
- Add regressions for the empty-environment direct worker and the parent stdout-failure path; retain nonzero exit and ICE-diagnostic assertions.

**Reasoning**

The normal spawned compiler path is fixed correctly. Both spawn sites set the constant after `env_clear()`. Accepting exactly `RUSTC_ICE=0` exposes no operator-controlled value and does not otherwise weaken fx-kw8.3’s isolation.

The new flag follows the existing operator-only interface, never becomes a user-controlled rustc flag, and is correctly included in both parent and worker cache-key computation. Ordinary metadata and long-type diagnostic files remain directed into staging. The blockers are incomplete coverage of the intended crash-report boundary. `git diff --check` passed, and the frozen diff remained unchanged.

Not checked: builds, tests, or runtime reproductions, as required by the read-only brief.  
Risk: the two remaining paths can still write crash reports into the candidate tree.
## Resolution (author)

- B1: the clean-environment check re-execs unless the environment is exactly `RUSTC_ICE=0`
  (`vars_os().ne([..])`), so an empty one re-execs too. The direct-worker path this covers
  needs the internal cache key, so the regression test does not reach it; it holds by
  construction.
- B2: `install_ice_hook` moved from `main.rs` to `run()`, after the environment is normalized;
  earlier panics use Rust's default hook. Regression: `--tide-print-codes` with stdout on
  `/dev/full` (fails with the hook at the start of `main`, verified by mutation).
- Non-blocking: the test now warms the cache with a successful run first, and asserts rustc's
  crash diagnostic ("the compiler unexpectedly panicked") as well as the seeded message.
