Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-07

## Verdict

**request_changes** — high confidence.

The five decisions are reflected in the text, but hash ordering remains incomplete, and the rename rule introduces a restriction outside the bead’s stated scope.

## Scope

Reviewed `git diff 8c9a38a fc37e5e`, the bead and decision comments, governing instructions, requested design sections, reference, cited research/reviews, conformance infrastructure, and relevant driver code.

Only the three authorized files changed. SEAL-4/5/6 prose is unchanged; their inventory line references are correct. No files were modified.

## Blocking

1. **Equal-hash impls need a canonical tie rule** — [design/04-contracts.md:1226](/home/lucca/pjs/lab/tide/.worktrees/fx-zr3/design/04-contracts.md:1226).

   Sorting unordered members by their own hashes does not fully determine the positional graph encoding. Consider a whole-sealed `S`:

   ```rust
   struct S;
   trait A {}
   trait B {}
   trait C { type Out; }

   impl A for S {}
   impl B for S {}
   impl C for S { type Out = Box<dyn A>; }
   ```

   The first two impls have equal name-independent content hashes. Visiting them in opposite orders assigns different positions to `A` and `B`; the third impl’s reference to `A` then encodes differently. Source-order or path-based tie-breaking can therefore produce a spurious FX0920 after moving declarations or renaming referenced traits. A small finite-graph model confirmed this encoding difference.

   **Fix:** specify a canonical, name-independent tie rule that preserves shared references and distinct items, and validate this case. “Order of their own hashes” alone does not settle the identity mechanism that fx-kw8.15 must implement.

2. **The rename rule freezes an additional namespace condition** — [design/04-contracts.md:1301](/home/lucca/pjs/lab/tide/.worktrees/fx-zr3/design/04-contracts.md:1301).

   Suppose a sealed clause calls private `pred`. Rename that helper to `valid`, update the clause, and add an unrelated helper at the freed `pred` path. The called definition and contract meaning remain unchanged. Existing SEAL-2 permits referenced-item renames and unrelated item additions, but the added vacancy test rejects this combination as a replacement.

   The Opus decision explicitly acknowledges this conservative rejection. It implements that decision faithfully, but changes accepted behavior despite the bead’s prohibition on new seal semantics and its scope stop condition.

   **Fix:** preserve the existing rename/addition behavior while detecting replacement, or obtain an explicit owner-approved scope amendment for this restriction. An implementation-mechanism decision alone does not satisfy that scope condition.

## Non-blocking

- [design/04-contracts.md:1286](/home/lucca/pjs/lab/tide/.worktrees/fx-zr3/design/04-contracts.md:1286): clarify that writer-mode admission checks the proposed seal, or excludes the comparisons being regenerated. Requiring ordinary admission against the existing seal would prevent `--target` from adding a target that SEAL-3 currently rejects.

## Reasoning

The post-analysis phase fits the driver’s callback structure, provided seal checking precedes the existing trust-artifact write. Release clause type-checking preserves resolved contract coverage. The named owners match the implementation beads, and the manifest grant ceiling preserves the trusted-caller boundary.

The unchanged reference remains compatible with the phase, release-erasure, and delegated-body decisions. Neither the driver nor the corpus currently implements SEAL-2/3, so existing conformance results cannot establish these mechanisms.

Not run: builds or the full conformance suite; validation used read-only structural checks and a finite-graph model.  
Risk: the proposed seal mechanism has no executable SEAL-2/3 cases yet.