Model: gpt-6.1-sol. Effort: xhigh. Date: 2026-10-31.

## Verdict

**approve — high confidence (0.90).** All five round-1 findings are resolved. No new blocking defect found in the round-2 changes.

## Scope

Narrow re-check of the corrections since round 1, including the query override, simplified mono walk, added tests, acceptance case, and edited ADMIT-1/ADMIT-2/D18/README text. Unchanged implementation was read only where needed to assess those changes.

Read `br --no-db show fx-kw8.4` and its comments. Applied the latest correction comment’s lazy-collection decision.

Candidate: HEAD `5472a9336661e2cbe9e16f436f8d842d2d76b868` plus working tree. Hash command:

```sh
git diff HEAD -- . ':(exclude).beads' | sha256sum
```

- Start: `998fb35d417c361610abb3859f4b9a9cc6fe19f9664d97572cc40f3b92673897`
- End: `998fb35d417c361610abb3859f4b9a9cc6fe19f9664d97572cc40f3b92673897`

**Unchanged.** Ignored concurrent `.beads/` changes. No files, commits, or Beads state were written. The scoped `git diff HEAD --check` passed.

## Round-1 findings

1. **Generic parallel captures: resolved.** [walk.rs:664](/home/lucca/pjs/lab/tide/crates/tide-admit/src/walk.rs:664) substitutes the closure’s parent arguments into capture-place types before checking them. This preserves the HIR path’s generic arguments and supplies concrete types in the backstop. [admit_backstop_accept.rs:56](/home/lucca/pjs/lab/tide/conformance/cases/authority/admit_backstop_accept.rs:56) exercises the original `paired<T: Pure + Sync>` case through an exported caller.

2. **Unavailable conditional provided method: resolved.** Eager collection is removed. The query override at [invocation.rs:38](/home/lucca/pjs/lab/tide/crates/tide-admit/src/invocation.rs:38) restores build-equivalent library roots while retaining lazy collection. An uncalled provided method is no longer instantiated merely because its impl exists. The acceptance case covers both the conditional `Pure` method and round 2’s failing inline-const method.

3. **Overridden trait method’s clauses: resolved.** [mono.rs:99](/home/lucca/pjs/lab/tide/crates/tide-admit/src/walk/mono.rs:99) finds the corresponding trait method and rebases the instance arguments onto the implemented trait before checking its clauses. This retains method arguments and includes inherited trait clauses. [driver.rs:427](/home/lucca/pjs/lab/tide/crates/tide-admit/tests/driver.rs:427) checks the omitted `Self: Pure` condition in backstop-only mode and expects FX0517 at the call.

4. **Static-initializer provenance: resolved.** [mono.rs:122](/home/lucca/pjs/lab/tide/crates/tide-admit/src/walk/mono.rs:122) records allocation-derived function/static edges, including nested memory and vtables. Dummy or imported spans fall back to the local user’s definition. The driver regression expects the `need::<Clock>` failure at the static declaration.

5. **Upstream-instantiation wording: resolved.** [ADMIT-2:578](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:578) explicitly preserves the `#[inline(never)]` upstream-reuse exception. This matches the pinned `Instance::upstream_monomorphization` implementation.

## Blocking

None. No patch is required for approval.

## Non-blocking

**Automate the upstream-function-static regression** at [driver.rs:427](/home/lucca/pjs/lab/tide/crates/tide-admit/tests/driver.rs:427).

Round 2 verified `pub static LOWER: fn(&str) -> String = str::to_lowercase;` manually, but no checked-in test protects that exact ICE fix. The smallest addition is an acceptance test using the existing helper and minimal Tide std, expecting exit 0 in backstop-only mode.

## Reasoning

The override delegates root selection to rustc’s existing Miri-exposed helper. In the pinned source, lazy collection uses the resulting export reachability alongside statics and the entry point. Library exports use the Rust threshold; binary/test exports use the C threshold. This supports the new tests’ distinction between exported library functions and entry-reachable test/binary code.

The override leaves metadata’s exported-symbol providers unchanged: they still return empty lists for metadata-only output. It therefore restores collection roots without advertising unavailable object symbols downstream.

The simplified mono walk requests function uses only for rustc-collected items. Allocation pointers no longer cause it to inspect linked upstream functions’ unavailable MIR, resolving round 2’s ICE. Independent denied-impl and `Pure` checks remain intact, and failed `Pure` obligations are located only after the instance’s verdict is known.

The edited design and README text match this mechanism. Round 3 records passing workspace tests and conformance; those are prior verification evidence.

Not run: builds, tests, conformance, or runtime probes; they write artifacts and this review is read-only.  
Risk: runtime confidence relies on source inspection and the recorded Opus verification.