# fx-kw8.9 round 3: Opus confirmation review

Candidate: commit 99810c8 on branch `fx-kw8.9` (fix delta 8d1d077..99810c8, the rebased form of 3dd55b7..f015038)

Model: claude-opus-5-5 · Date: 2026-11-07

## Verdict

**Clean. No blockers.** `-Zcross-crate-inline-threshold=never` closes B4 in
both profiles and in `--test` builds. The regression test fails without the
flag. Codex's source audit holds up on spot-checks, and its correction of
round 2's `#[rustc_force_inline]` claim is right but changes nothing. N9's
profile-dependent pruning of provably dead code stays within the backstop's
stated contract. The rebase is clean. The fix adds one non-blocking nit
(N10).

How I checked: I read the delta, `invocation.rs::rustc_args`, the test
helper `admit`, `walk/mono.rs`'s header, CODEX-REPORT.md, the round-2
review, and the pinned rustc source. I ran no cargo, no conformance and no
driver. The result of the full checks on 99810c8 is still pending, since
they are queued.

## Findings

### B4: resolved

- **Mechanism.** `cross_crate_inlinable`
  (`rustc_mir_transform/src/cross_crate_inline.rs`) consults the opt level
  in only two places. Line 88 skips inference at `OptLevel::No` without the
  inliner. Lines 96–110 apply the threshold. With `Never`, line 99 returns
  `false` before any body is costed, so the opt-level branch no longer
  changes the result. Everything that can return `true` earlier is
  profile-independent:
  - def kind (Ctor, Closure, SyntheticCoroutineBody, line 28);
  - `RustcIntrinsic`, which needs `rustc_attrs`;
  - explicit inline attributes (line 56);
  - `hint_mostly_unused`, which defaults to `false` (`options.rs:2530`) and
    which the driver never passes;
  - f16/f128 in the signature, which is feature-gated at resolution
    (`ident.rs:768`).

  `reachable_non_generics_helper` filters on `requires_monomorphization`,
  `requires_inline` and `cross_crate_inlinable`
  (`symbol_export.rs:103–111`). `Instance::requires_inline`
  (`instance.rs:285`) depends on the def path and the shim kind, not on
  the profile. `reachable.rs` uses `cross_crate_inlinable` only to decide
  traversal (line 46) and has no `optimize` check of its own. The
  partitioning in `mono.rs:176–193` runs after collection, and the
  backstop reads every CGU. So the backstop's roots are now the same in
  debug and release.
- **Placement.** The flag is in the shared `rustc_args` vector, before the
  `self.link` and `self.test` branches, so it reaches metadata, bin-link
  and `--test` builds. Through `rustc_args` it also enters the cache key
  (`invocation.rs:774`).
- **`--test` builds.** A `--test` build is a harness executable, so
  `any_library` is false and its roots are `main`, the statics and
  `#[no_mangle]` items. Items with an extern indicator return `false` at
  line 22. The flag still applies there, and nothing in that path depends
  on the profile.
- **Regression test.** `admit` runs the real `Invocation` path, with
  `--crate-type lib --emit metadata`. The test loops over both of round 2's
  probes, in debug and in release, with `--tide-test-backstop`, and
  requires exit 1 with an `FX0520` line for `fn()`. It fails without the
  flag: round 2 independently saw release exit 0 for both probes on
  3dd55b7, and Codex reports the targeted test failing at
  `backstop-exported-vtable/release` before the fix. The test does not
  cover `--test`, which is acceptable because `--test` was never
  affected.
- **Docs.** ADMIT-2 and the README state what the flag does. They also say
  that explicit `#[inline]` functions and generics are checked where they
  are instantiated, which matches `mono.rs`'s header and is unchanged
  behavior. Neither claims identical verdicts across profiles.

### Codex's source audit: correct on spot-checks

I checked these citations against the pinned `rustc-src`:

- `cross_crate_inline.rs:26/52/83/99`;
- `symbol_export.rs:102–111`;
- `options.rs:2087` (`never` parses) and `:2530`;
- `ident.rs:768`;
- `inline.rs:47–58` (`inline_mir` takes precedence);
- `lib.rs:321` (`is_mir_available` is `mir_keys`, independent of emit);
- `encoder.rs:1115` and `:1824`;
- `collector.rs:1086–1101`, where local items always generate locally and
  missing upstream MIR is fatal;
- `instance.rs:224` (the `#[inline(never)]` share exception);
- `mono.rs:186`.

Each says what the report claims. I found no other route on this nightly by
which the opt level removes a root or an instance from collection.

**The `#[rustc_force_inline]` correction is right.**
`library/std/src/sys/personality/gcc.rs:207` marks the private,
non-generic `sign_lpad`, and round 2's "the pinned `library/` has none" was
wrong. It is the only occurrence in `library/`. The occurrence does not
matter:
- `ForceInline` acts only where a body calling the callee is optimized. That
  is std's own personality routine, compiled into std's trusted artifact.
- No candidate MIR, generic or otherwise, calls `sign_lpad`.
- On x86_64 the body is the identity (lines 229–232).
- Candidates cannot apply the attribute (`-Zallow-features=`).
- `crates/` has none.

### N9: acceptable, honestly documented by silence (nit, non-blocking)

The pruning differs only for instances that cannot run. ADMIT-1 scopes the
backstop to "the instances that can run … through the uses rustc records",
so release accepting an instance that sits in a provably dead branch, which
debug rejects, stays within the contract. Nothing reaches codegen
unchecked.

No committed text claims that the profiles give identical verdicts or MIR:
I searched the README, ADMIT-1/2, `01-design.md` and `mono.rs`.
`03-authority-concurrency.md:944` (D12) is about runtime behavior, not
admission. So the docs are honest.

Codex's "does not promise identical mono-item sets" appears only in the
scratch report. Adding a clause to ADMIT-2 would be optional. Its fix would
touch only text this change already edits, so it does not block either way.
I found no level-2 pass that adds a mention release-only, so a
release-only rejection remains unobserved.

### Rebase: clean

- `git diff d12d7e0 f015038` and `git diff 0f3b70c 99810c8` are identical
  except for `index` lines and one hunk header in
  `design/03-authority-concurrency.md`, which moves from `-582` to `-594`.
  The same holds for the fix commit alone (3dd55b7..f015038 against
  8d1d077..99810c8): only the `-583` hunk moves to `-595`.
- The only file that both main and the branch touch is that chapter.
- Main's edits there are:
  - fx-aw6's GLOBAL-4 cast paragraph, 12 lines that cause the offset;
  - `naked_asm!` in ADMIT-1's list, the CAP-4 unsafe-attribute list, and
    the RUN-2 and RUN-4 rows.
- None of these is in or next to ADMIT-2. ADMIT-2 reads coherently after the
  revised ADMIT-1 list.
- Main touched no file under `crates/`. Its new conformance cases are
  covered by the queued checks on 99810c8, so the pre-rebase pass (47
  tests; conformance 195 PASS, 0 FAIL) does not carry over to the
  rebased tree.

### New in this fix

- **N10. Release libraries lose inferred cross-crate inlining (nit,
  non-blocking).** A small non-`#[inline]` function in a Tide library can no
  longer be inlined into a downstream crate without LTO. This was Rust's
  behavior before 1.75, and `#[inline]` restores it. It is a performance
  cost, not a correctness cost. Round 2 called it negligible only for bins.
  The README's "LLVM inlining still runs after the check" is still true
  within a crate. Recording the cost in ADMIT-2 or the README is optional.
- The N8 rewording ("GCC also passes `-L/usr/bin` to the linker for
  `-B/usr/bin`") is accurate.
- I found no other leftover, bug, or doc drift. CODEX-REPORT.md and the
  `reviews/` files are untracked and must stay out of the landed commit.
