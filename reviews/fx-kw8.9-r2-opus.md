# fx-kw8.9 round 2: Opus confirmation review

Candidate: commit 3dd55b7 on branch `fx-kw8.9`, fix delta 93c1dec..3dd55b7

Model: claude-opus-5-5 · Date: 2026-11-07

## Verdict

**Changes required (one blocker, B4).** B1, B2, B3 and the decision-2 README
line are resolved. `-Zinline-mir=no` closes the MIR-inlining route in every
profile and in `--test` builds. The B3 routing is correct, and the cache
identities and the output/staging separation are intact. But the release
profile still changes what the backstop checks, by a second route that the
flag does not cover: cross-crate-inline inference at `-Copt-level=3` removes
call-free exported library functions from the backstop's roots. I observed
it on the candidate's driver (B4). It belongs to the same class as B1 and
needs a code fix, so it blocks.

How I checked: I read the delta, `invocation.rs`, `walk/mono.rs` and
`tests/build.rs`, plus the pinned rustc source under
`nightly-2026-08-09/lib/rustlib/rustc-src`. I ran no cargo and no
conformance. I ran 8 metadata-only invocations of the worktree's existing
`target/debug/tide-admit` on two 3-line probe libraries. That binary
contains the `-Zinline-mir=no` string, so it was built from the fixed
source.

## Findings

### Round-1 findings

- **B1, release MIR inlining: resolved for MIR inlining; the class remains open (B4).**
  - The flag works. `Inline::policy` (`rustc_mir_transform/src/inline.rs:47-58`)
    takes `unstable_opts.inline_mir` before any `mir_opt_level` or
    `optimize` default, so `-Zinline-mir=no` turns the normal inliner off
    at every opt level.
  - `ForceInline` is `PassPolicy::Required`. It applies only to
    `#[rustc_force_inline]` callees. The pinned `library/` has none,
    `tide-std` and `tide-macros` have none, and `-Zallow-features=` stops
    candidates from using `rustc_attrs`.
  - The flag sits in `rustc_args` before the `self.link` and `self.test`
    branches, so it applies to metadata, bin link and `--test` link alike.
    It is also in the cache key, through `rustc_args`.
  - The other routes:
    - Polymorphization no longer exists in the pinned rustc.
    - `-Zshare-generics=no` is unchanged (`instance.rs:224-230`).
    - LTO and codegen-unit partitioning run after collection.
      `check_mono_items` reads every CGU's items, so per-CGU copies only
      duplicate items; none are lost.
    - Upstream `#[inline]` and inferred-inlinable non-generic functions are
      local mono items whatever the downstream profile
      (`collector.rs:1091`). Their `cross_crate_inlinable` value is decided
      upstream.
    - `InstSimplify`'s primitive-clone rewrite runs at `mir_opt_level > 0`,
      so it runs in both profiles.
  - The new test fails without the flag. R1 observed exit 0 and a linked
    `out/demo` for this exact source at link/release on 93c1dec, and the
    test requires exit 1. Codex reports the same before/after result.
- **B2, README dependency directory: resolved.** The recipe uses the
  `filenames` of Cargo's `compiler-artifact` message, as
  `conformance/run.sh` does. Cargo emits that message for fresh units too,
  so the recipe works on a rebuild, under the new build-dir layout or the
  old `deps/` one. The selected directory holds only the proc-macro's
  outputs, so `fingerprint` no longer hashes the whole target tree.
- **B3, the `--out-dir` warning on every link: resolved.**
  - Link mode passes `-o <stage>/executable` and `-Ztemps-dir=<stage>`,
    with `-Zunstable-options`. Metadata mode passes `--out-dir <stage>`.
  - In both modes rustc's outputs land in the per-pid staging directory,
    and the trust artifact goes to `Admission.out`, which is also that
    staging directory. Publication still copies only `executable`,
    `*.rmeta` and `*.json`.
  - Cache keys still differ by emit mode (`--emit=`), profile, test and
    out-dir. The driver-binary fingerprint invalidates entries written by
    the old argument shape.
  - The tightened `out.stderr.is_empty()` assertion is strictly stronger
    than the `HELLO_RAN` check it replaces, and it covers the cache-hit
    replay as well.
- **Decision 2, the README `-L/usr/bin` line: resolved.** It is accurate:
  the GCC driver adds each `-B` prefix to its library search path. See N8
  for wording.

The edits make no RUN-5 or RUN-1 claims. The README's
`-Zinline-mir=no` sentence and the ADMIT-2 addition say only what the flag
does, which is accurate.

### New findings

#### B4. Release still drops call-free exported library functions from the backstop's roots (not text-only)

The driver overrides `reachable_non_generics` with
`reachable_non_generics_helper` (`invocation.rs:48-51`). That helper
excludes every `cross_crate_inlinable` item (`symbol_export.rs:111`).

For an un-annotated function, `cross_crate_inlinable`
(`cross_crate_inline.rs`) infers inlinability only when
`optimize != No || inliner_will_run`. At debug, with the inliner now off,
the result is `false`. At release (`-Copt-level=3`) inference runs, and
any exported function with no calls, drops or resumes and at most 100
statements becomes inlinable. That function is then not a root, so its
reifies and vtables never reach `check_mono_items`. `-Zinline-mir=no` does
not change this, because the gate also passes on `optimize`.

This affects `--crate-type=lib` metadata checks, which are the documented
default operator workflow, and the holes in their trust artifact. Bins and
`--test` builds are not affected: they root at the entry point, and local
items are always collected.

Observed on the candidate's driver, with Rust's std and
`--tide-test-backstop`:

```rust
pub fn leaf(x: &fn()) -> &dyn PartialOrd<fn()> { x }   // or:
pub fn sorter() -> fn(&mut [fn()]) { <[fn()]>::sort }
```

| Profile | Result |
| --- | --- |
| debug | exit 1, `FX0520: impl core::cmp::PartialOrd for fn() … (it runs as std::ptr::<impl PartialOrd for fn()>::partial_cmp)` |
| release | exit 0, no diagnostic |

Without `--tide-test-backstop`, the walk rejects both in both profiles. So
this needs a route the walk misses, which is the same standard B1 was
judged by.

Fix: pass `-Zcross-crate-inline-threshold=never` in every profile, next to
`-Zinline-mir=no`. The parser accepts `never` (`options.rs:2087`). The
threshold step returns `false` after the attribute checks, so explicit
`#[inline]` functions stay excluded in both profiles. That matches
`mono.rs`'s documented "a library's `#[inline]` functions … are checked in
the crates that instantiate them". Release is then identical to debug.

The cost is negligible: bins have no downstream crates, and LLVM and ThinLTO
inlining still run. Record the flag in ADMIT-2 and in the README sentence.
Add a `--crate-type=lib`, metadata, `--tide-test-backstop`, release case
built on the `leaf` probe above. It fails on 3dd55b7. I have not verified
the fix by a build; I derived it from source.

#### N8. "Translates" is imprecise (text-only, nit)

The README says "GCC also translates `-B/usr/bin` into `-L/usr/bin`". GCC
keeps `-B` for finding `ld` and `collect2` and also adds `-L/usr/bin`. A
more precise wording: "GCC also passes `-L/usr/bin` to the linker for
`-B/usr/bin`."

#### N9. Release still runs mir-opt-level 2 (not text-only, nit, non-blocking)

GVN, jump threading and the other level-2 passes can prune branches that
are provably dead. So in backstop-only routes, a debug check can reject an
instance that release drops. Only instances that cannot run are affected,
so nothing reaches codegen unseen. `-Zmir-opt-level=1` in every profile
would make the MIR identical across profiles. It is optional.

Round-1 nits N1 to N7 are unchanged, which is acceptable for nits.
`CODEX-REPORT.md` and the review files are untracked and must stay out of
the landed commit. The fixes introduced no other bug or leftover.
