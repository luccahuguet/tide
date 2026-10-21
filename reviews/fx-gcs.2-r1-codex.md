Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-21.

## Verdict

**request_changes — high confidence.** Two small v0 corrections are needed. The overall direction and `Task` fix are sound.

## Scope

Base: `e2a7063`. Reviewed the working tree’s complete diff, including intent-to-add files.

Recomputed fingerprint, twice:

`cbd73fb9abe0a7b40cbf00fa6a747337358203aecfb1cc4cc0d2e5a842653271`

**Matches** the supplied fingerprint.

Read `GOVERNANCE.md` first, then `AGENTS.md`, both beads, `design/01-design.md` including the tier table, affected memory sections, relevant reference and concurrency text, FX definitions across `design/`, the conformance documentation, inventory, runner, stub, and all 86 memory cases. A delegated read-only investigation checked the task signatures, auto traits, and variance.

Ran structural coverage checks and `git diff --check`; both passed. Installed rustc is `1.96.0 (ac68faa20 2026-05-25)`.

Did **not** run `conformance/run.sh`: its target directory is unwritable. Targeted stdin compiler probes emitted MIR for the closure counterexamples and function-pointer exploit, then failed creating temporary metadata under the read-only sandbox. The independently elided closure produced the expected lifetime diagnostic. These observations do not constitute successful complete builds; the author’s reported 80 PASS / 11 AWAITS CHECKER result was not reproduced.

No files changed.

## Blocking

1. **CLOSURE-2 rejects valid standalone closures.**
   `design/02-memory.md:521`, `conformance/rules.md:56`, and `conformance/cases/memory/closure_hr_reject.rs:2`.

   The failure demonstrated by `|t: &str| -> &str { t }` does not justify saying every argument-borrowing closure without an expected type is erroneous. An explicit shared lifetime works:

   ```rust
   pub fn id<'a>(s: &'a str) -> &'a str {
       let f = |t: &'a str| -> &'a str { t };
       f(s)
   }
   ```

   Ordinary inference also permits `let f = |t| t; f(s)`. Both examples reached MIR emission on the installed compiler. Implementing the stated rule would reject valid Rust lifetime behavior, contradicting D2 and ORIGIN-6.

   **Smallest fix:** narrow the rule, inventory summary, and rejecting-case comment to independently elided annotations. Preserve named-lifetime and inferred closures, and add these examples to the existing accepting case.

2. **DROP-2’s rejecting case omits its grant diagnostic.**
   `conformance/cases/memory/drop_may_dangle.rs:7`.

   The case contains an `unsafe impl` but marks only `E0658`. The newly added no-manifest/no-grants contract in `conformance/README.md:72` explicitly requires `FX0901` for an `unsafe impl`. The expected diagnostics therefore disagree with this candidate’s own case format.

   **Smallest fix:** change the mark to `//~ E0658 FX0901`. Retain `E0658`, which tests the intended unstable-feature rejection.

Both fixes affect edited v0 material; neither is deferred to `fx-djp`.

## Non-blocking

- Granted-package acceptance for SEND-2 remains untested, as the README explicitly defers manifest-backed cases. Its structural accepting case does not prove that granted unsafe impls are admitted.
- LIFE-5 and BORROW-1 overlap on reference fields. Their cases could share coverage, but neither consolidation nor ID removal is necessary.
- No findings confined to later-tier text require a `for fx-djp` entry.

## Reasoning

All **46 assigned IDs** have exactly one bold definition, an inventory row, and accepting and rejecting cases. I found no missing in-scope compile-time rule or duplicate definition. Apart from the issues above, the cases exercise their stated boundaries rather than incidental failures.

The LIFE-2 relocation preserves meaning and matches Rust’s [precise-capture restrictions](https://doc.rust-lang.org/reference/types/impl-trait.html#precise-capturing). DROP-2’s unstable-feature explanation and SEND-2’s “only” clarification are correct. CLOSURE-2 correctly identifies the failure of independently elided annotations, but overgeneralizes it; Rust permits [closure parameter inference from context](https://doc.rust-lang.org/reference/expressions/closure-expr.html#closure-parameters).

`FX0205` and `FX0206` give diagnostics to existing checker obligations. Neither collides with another definition or parallel `FX0518`. Defining them beside their owning rules in 02 is consistent with 02’s normative ownership; the core reference need not become a complete diagnostic catalog. The function-pointer case targets the surviving higher-ranked form of [rust#25860](https://github.com/rust-lang/rust/issues/25860).

The `Task` marker suppresses automatic `Send`/`Sync`, and the explicit impls restore both exactly when `U, E: Send`. This correctly allows a `Cell<u8>` result while requiring its error to be `Send`. `'s` remains invariant; `U` and `E` remain covariant, which is sound for the consuming `join` interface. The unspecified variance is appropriately marked `GUESS`. `spawn_isolated` and `Panicked` match the design.

Not checked: a complete conformance rerun or Tide-checker execution. Exact corpus-wide diagnostic spans remain supported by the author’s run and static review.