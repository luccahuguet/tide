# Tide review, fx-kw8.10 round 1 (Opus): converge round, report only

Candidate: the staged diff on base 0141db4, SHA-256
`f768ba29969675d1d7bf651028464f00a7902f11d24ea18e95fca955a3de19d0`.

Model: claude-opus-5-5 · Date: 2026-11-03

## Verdict

Findings: 4 blocking, 1 non-blocking. The reviewer verified the hash.

## Findings

1. **Blocking: sync handles drop the authority they carry**
   (`crates/tide-admit/src/interface.rs:215`). The `Carries::Kinds` arm adds
   the table's kinds and never walks the type arguments. Reproduced with the
   conformance fixture std: `pub fn shared_net(a: Arc<Net>)` gives
   `{"a": ["sync"]}`, and `Receiver<Fs>` and a field `Arc<Net>` do the same.
   Changing `Arc<u32>` to `Arc<Net>` would show no change, which breaks
   fx-kw8.11. Fix: after the kinds, also walk `args.types()` at the same path.
2. **Blocking: impls admitted under the grant are never recorded as holes**
   (`walk.rs:1229`, `walk/mono.rs:56`). `obligation` keeps only `Err`, and the
   backstop's `Walk` discards its holes. With the grant, `fn()` comparisons,
   `&f as &dyn Debug`, and `f.hash(..)` admit with `"holes": []`; without it,
   FX0520 rejects `impl … for fn()`. Fix: record `Ok(Granted(key))` in
   `obligation`; return and merge the backstop's holes.
3. **Blocking: holes found while checking macro expansions land on the crate
   root** (`walk.rs:1287`, `hole` at :373). `check_expansions` runs after the
   HIR walk, so two functions that each use `asm!` give one hole for the
   crate; `naked_asm!` and `addr_of!` do the same. Fix: hold each use by the
   innermost local definition whose `source_span` contains the call site.
4. **Blocking: a failed FX0915 check leaves the earlier artifact behind**
   (`invocation.rs:641` vs :654). The `build.rs` rejection and the earlier
   path errors return before the removal. Fix: remove the artifact before the
   FX0915 check.
5. **Non-blocking: the search can hang on an admitted type.** `Search::walk`
   has no memo or budget across sibling paths, so polymorphic recursion
   (`pub struct Poly<T> { pub a: T, pub b: Option<Box<Poly<(T, T)>>> }`)
   ran past 90 s and 3.3 GB; private, the crate admits in 0.16 s. Fix: a
   node budget per item that ends in `opaque(<type>)`.

Nit: items nested in a function body get their declaration holes
(`unsafe fn`, `unsafe impl`) attributed to the enclosing function, since the
body owner wins over the item in `hole`.

Out of scope, not caused by this diff: a `safe fn` in an `unsafe extern`
block is admitted without the grant and called without `unsafe`; the
artifact does list it as a foreign-item hole.

## Checked

- The staged diff's hash; `cargo test --workspace` and `conformance/run.sh`
  exit 0.
- The FX0901 refactor against `git show 0141db4:crates/tide-admit/src/walk.rs`:
  `fn_item`, `visit_block`, `unsafe impl`, `check()`, and `trusted_pure` are
  equivalent; `unsafe_attrs` now also exempts attributes another crate's
  macro wrote, and `#[pure]` with `#[unsafe(no_mangle)]` or
  `#[unsafe(export_name)]` is still flagged without the grant.
- Scratch packages against Tide's std and the fixture std: struct, variant,
  tuple, and pattern paths; `Root` and `SimRoot` fields; `dyn`, `impl Trait`,
  parameters, aliases, trait and impl methods; hole kinds, counts, and
  closure attribution; the artifact restored on a cache hit, removed after a
  missing grant, absent without `--tide-std`, and valid JSON.
