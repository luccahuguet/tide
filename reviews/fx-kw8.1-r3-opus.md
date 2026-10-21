# Tide review, fx-kw8.1 round 3 (Opus): converge round

Candidate: HEAD 93b0117 on 187a22b (worktree `/home/lucca/pjs/lab/tide-fx-kw8.1`, branch
fx-kw8.1). This round reviewed `git diff 187a22b HEAD` in full and looked hardest at round 2's
edits. It fixed what it found in the working tree, uncommitted. Probes are in this scratchpad's
`t3/`, run by `adm3.sh`, which is round 2's `adm2.sh` writing to `t3/`.

## Verdict

fixed

## Findings and fixes

**F1. Obligations from `?`, `for`, autoderef, and a type's where-clauses went unchecked
(admission hole).** ADMIT-1 checks "every … deref, impl selected for an obligation". Four
routes selected an impl the walk never saw. Each probe exited 0:
- **`?` and `for`.** For the lang-item calls these desugar to (`FromResidual::from_residual`,
  `Try::branch`, `IntoIterator::into_iter`, `Iterator::next`), typeck returns the `FnDef` type
  but records no node args. The pin's `rustc_hir_typeck/src/expr.rs:546-581` shows this. So
  `visit_path` never called `use_generic`.
  - `let r: Result<(), [(u8, u8); 1]> = Err([(1, 2)]); r?;` in a fn returning
    `Result<(), HashMap<u8, u8>>` built a `RandomState` through `From<[(K, V); N]>`. That
    bypasses GLOBAL-5 and the round-1 Codex fix B2 (`t3/q2.rs`, `t3/q3.rs`).
  - A user `impl<T: Debug> From<Wrap<T>>` reached through `?`, and a user
    `impl<T: Debug> IntoIterator` reached through `for`, formatted `fn()`. That is GLOBAL-4
    (`t3/q3.rs`, `t3/q5.rs`).
- **Autoderef.** For an overloaded-deref adjustment, the walk checked only the `Deref` trait,
  not the impl selected for the source type. `D(noop as fn(), s).len()` passed, through
  `impl<T: Debug> Deref for D<T>` (`t3/q5.rs`).
- **Where-clauses and `Drop`.** An ADT's where-clauses were never obligations: `use_ty` checked
  only the def, and `use_generic` skips constructors. With `struct P<T: Debug>` and its
  `Drop` impl, `let _p = P(noop as fn());` formats a code address on drop (`t3/q5.rs`).

*Fix (`walk.rs`):*
- `visit_path` falls back to the args of the path's `FnDef` type when typeck recorded none.
- An overloaded-deref adjustment now checks the obligation `source: Deref` (or `DerefMut`).
  `source` is updated after the adjustment, not before.
- `use_ty` runs `use_generic` on each ADT, so the ADT's where-clauses are checked.

The module doc names the new routes. The new case `conformance/cases/authority/admit_obligation_routes.rs`
(ADMIT-1, GLOBAL-4) has one route per mark. With the old `walk.rs` it FAILs: the driver
reports none of 21, 35, 48, or 59. It PASSes after the fix. All of round 2's probes give
their old verdicts. The E1 programs give the same diagnostics as before the fix, in both modes.

**F2. The same rejection on the same line of two files was reported once (lost diagnostic).**
`Diagnostics::report` deduplicated on `(code, message, line)` without the file. A crate with
`AtomicU64::new` on line 2 of both `lib.rs` and `sub.rs` got only one FX0520 (`t3/mf/`). The
crate was still rejected, but the second use was never reported. A Tide package spans module
files, so an agent would fix one use and only then learn of the next.
*Fix:* the key includes the file's `start_pos` (`diag.rs`). The driver test
`rejects_each_use_outside_the_allowlist_at_its_span` adds a `mod sub;` whose line 3 repeats
`lib.rs`'s rejection, and asserts both. The test helper now strips the test directory from
every path, not only `lib.rs`'s.

## Not changed

- **`core::intrinsics::unreachable` is not reachable from user code.** A direct call through
  `core::intrinsics::` gets E0658 (`core_intrinsics`), and so does one from a user
  `macro_rules!`. Rust's std has no `std::intrinsics` path that resolves. With `-Zallow-features=`
  the feature cannot be enabled, and `#[allow_internal_unstable]` is unstable too. The stable
  wrapper, `core::hint::unreachable_unchecked`, has a key of its own and gets FX0520 (`t3/u1.rs`).
  `discriminant_value` behaves the same way. The `unsafe` block in the derive's output is
  UNSAFE-1's concern (FX0901, a later slice).
- **Round 2's F1 (generated code no longer exempt from impl checks)** causes no false
  rejections in the toolchain's own expansions. Under `--test`, these were all admitted
  (`t3/gen.rs`):
  - `#[should_panic(expected = ..)]`, `#[ignore = ..]`;
  - test fns returning `Result<(), Box<dyn Error>>`, `io::Error`, a user error, or `ExitCode`.

  `#[tide::main]` was admitted with `Result<(), Box<dyn Error>>`, `ExitCode::from(3)`, and `()`
  (`t3/m1-m3.rs`). The F1 fix adds ADT where-clauses, and these probes, the corpus, and the E1
  programs still pass after it.
- **Folding `check_with` into `check`** is clean. Only `obligation` bypasses the exemption,
  and the comment there says why.
- **`ExitCode::SUCCESS`/`FAILURE` and `std::path::absolute`** are right. I read the pin's
  `std/src/path.rs`. Every `Path` method that calls `fs::` is denied, and `absolute` is the
  only free fn that reads the environment.
- **Only one diagnostic per owner for a repeated obligation.** `seen_obligations` is keyed on
  `(trait ref, owner)`. If `fn(): PartialOrd` arises from `(f,) < (g,)`, `Some(f) < Some(g)`,
  and `[f] < [g]` in one fn, only the first line is reported (`t3/fl.rs`). The crate is still
  rejected. Keying on the span would multiply selection work. This is diagnostic
  completeness, not admission, so I left it.
- **`LocalExpnId::fresh_empty()`** allocates one empty expansion id in order to bound the scan.
  It comes from the spike. No later pass in these builds trips on the empty id. rustc has no
  public iterator to use instead.
- **`async fn`** is not rejected by the allowlist itself. No rule in `rules.md` covers async
  syntax, and polling needs `Context`/`Waker`, which are denied. This is outside this bead.
- **Stub artifacts** (`HashMap::new`, `DirEntry`, `Metadata` in the E1 programs) are as round 2
  recorded.
- `alloc::io::*` and `core::io::*` cover more than their comments name. On this pin,
  `BufReader`, `BufWriter`, `Cursor`, `Read`, and `copy` live there. `t3/bio.rs` (BufReader,
  BufWriter, `io::Error` accessors, `SeekFrom`) is admitted, as reference §1 wants. The
  comments give examples, not a complete list.
- `design/reference.md` is untouched, at 24,992 tokens. No design text needed a change: ADMIT-1
  already says "impl selected for an obligation", and F1 makes the walk match it.

## Checked

- `git diff 187a22b HEAD`: every non-review file was read. The three earlier reviews and the
  bead were read too.
- Probes (`adm3.sh`, pinned-nightly stub std):
  - before the fixes:
    - `q2.rs`, `q3.rs`, and `q5.rs` were admitted on every route above. q5's only rejection
      was line 8, inside `glob`;
    - `mf/` reported one of the two AtomicU64 uses;
    - `gen.rs`, `m1-m3.rs`, `mac.rs` (common macros and derives), and `bio.rs` exited 0;
    - `fl.rs` rejected exactly the float observers and fn-pointer ordering, hashing, and
      formatting;
    - `u1.rs` gave E0658 or FX0520 for each direct intrinsic use.
  - after the fixes:
    - every hole route in `q2/q3/q5` is FX0520 at the `?`, the `for`, the receiver, or the
      constructor;
    - `mf/` reports both files;
    - `gen`, `m1-m3`, `mac`, and `bio` still exit 0, in both modes where relevant;
    - round 2's `mainhole`, `testhole`, `mainok`, `p3`, `p4`, and `spoof` give the verdicts
      round 2 recorded.
- E1 programs (`trials/e1-dialect/programs/*.rs`), both modes, before and after: the same
  diagnostics, which are only the stub artifacts above.
- The new case fails on the old `walk.rs`
  (`driver errors [], marked [21:FX0520 35:FX0520 48:FX0520 59:FX0520]`) and passes on the new
  one.
- `CARGO_BUILD_JOBS=4 cargo test --workspace`: before, 2 + 4 passed; after, 2 + 4 passed.
- `conformance/run.sh`:
  - before: exit 0, 139 PASS, 57 AWAITS CHECKER, 0 FAIL;
  - after: exit 0, 140 PASS (the new case), 57 AWAITS CHECKER, 0 FAIL.
- `rustfmt --edition 2024 --check` (stable) on the crate's sources is clean.
  `git diff --check` is clean.
- `scripts/count-tokens.sh design/reference.md`: 24,992.
- Load stayed below 1.1, and builds ran one at a time.
