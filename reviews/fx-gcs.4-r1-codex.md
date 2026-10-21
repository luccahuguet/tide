Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-21.

## Verdict

**request_changes — high confidence.** The corpus and diagnostic assignments are sound. Two small corrections to the v0 safe-fix wording remain.

## Scope

Reviewed `git diff 755d86e`, all 20 contract cases, the requested governance, instructions, beads, design sections, conformance runner, stub std, and macro crates. Searched FX codes across `design/`, `conformance/`, `trials/`, and `reviews/`.

Recomputed fingerprint at the beginning and end:

`7fc93014466bfacfea6d9ceff528debfb51b3a0b767fdcdd5748295cbc7e438e`

**Matches.** No files changed.

Structural coverage and `git diff --check` passed. A read-only diagnostic replay with rustc `1.96.0 (ac68faa20 2026-05-25)`, using existing stub artifacts, matched **92 PASS, 19 AWAITS CHECKER, 0 FAIL**. Contract cases account for **13 PASS and 7 AWAITS CHECKER**. Both ordinary and test configurations were checked through `RUSTC_BOOTSTRAP=1 -Zunpretty=mir`, avoiding metadata writes.

## Blocking

1. **P2 — FORM-4 still promises an ineffective safe fix for misplaced clauses.**
   `design/04-contracts.md:220`.

   The new condition checks only whether the function lacks an attribute. A function without an attribute can also contain a late invariant or an invariant outside a loop:

   ```rust
   pub fn f() {
       while false {
           let _ = 0;
           invariant!(true);
       }
   }
   ```

   Adding `#[invariants]` leaves this rejected. Compiler probes confirmed that it repairs a correctly positioned clause, but repairs neither a late clause nor one outside a loop. Implementing the specified fix would therefore leave these FX0806 errors unresolved.

   **Smallest fix:** promise attribute insertion only for a correctly positioned clause whose function lacks the required attribute. Leave misplaced clauses without an automatic fix.

2. **P2 — FORM-2 extends parameter renaming to errors with no corresponding parameter.**
   `design/04-contracts.md:188`.

   “Either error is FX0805, and the safe fix renames the parameter” associates that fix with both parameter collisions and invalid contextual uses. The candidate’s own `form_result_reject.rs:15` uses an unbound `result` in `requires`; its invariant at line 24 does likewise. Neither has a `result` parameter to rename.

   This gives the checker an impossible fix contract for cases the corpus explicitly introduces.

   **Smallest fix:** keep FX0805 for both errors, but restrict the renaming promise to cases where an existing parameter causes the conflict. Do not promise an automatic replacement for an unbound `result`.

Both findings concern edited v0 text and therefore block under the tier rule.

## Non-blocking

- `conformance/cases/contracts/form_comma_accept.rs:2` claims coverage of commas inside call arguments, but its expressions contain no such comma. Replacing `a.max(b)` with `std::cmp::max(a, b)` would exercise that boundary within the existing case.
- No findings confined to later tiers require a **for fx-djp** entry.

## Reasoning

All nine in-scope IDs have one owning definition and both accepting and rejecting coverage: FORM-1–4 and CLAUSE-1, 2, 3, 4, and 6. The cases exercise their stated restrictions, and every rustc mark matches the replay.

E0600 for comma tuples and E0500 for snapshot borrow conflicts are acceptable records of the D18 lowering. The case comments, inventory, README, and separate FX marks distinguish them from Tide diagnostics. CLAUSE-6’s E0425 classification correctly follows the invariant’s placement before iterator bindings.

FX0804–FX0809 each identify an existing checker obligation and have no collisions. FX0801 remains retired; historical FX0810–FX0812 remain untouched; parallel FX0518 does not conflict.

The `loop` spike-gap addition agrees with D18 and §2.4. SEAL’s manifest-backed cases, `prove`, and test-driver behavior are appropriately deferred. Apart from the safe-fix promises above, I found no conflicting design text or unnecessary machinery.

Not checked: the exact `conformance/run.sh` invocation, rebuilding dependencies, or Tide-checker execution. The replay validates rustc diagnostics; FX verdicts remain design assertions awaiting the checker.