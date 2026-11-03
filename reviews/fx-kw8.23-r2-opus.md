# Tide review, fx-kw8.23 round 2 (Opus): confirmation round

Candidate: base ff9900f, fixes dd40cf6..d6ff4f1 (`git diff dd40cf6 d6ff4f1 -- . ':!.beads' ':!reviews'`)

Model: claude-opus-5-5 · Date: 2026-11-03

## Verdict

**Clean.** Each fix resolves its finding, and none adds a defect. Three text-only nits remain. None blocks.

- **B1 is resolved.** `fn_item` now checks `header.is_unsafe() && !tcx.is_foreign_item(def)` (`walk.rs:1043`), so a foreign fn is reported only by `visit_foreign_item` as "a foreign item":
  - without the grant, one FX0901;
  - with the grant, one hole.
  `fn_item` is not reached for foreign fns any other way (`visit_fn` needs a body). A required trait method is not a foreign item, so it is unaffected. The driver test pins `lib.rs:3:5` (`fn probe();`) to exactly one line, and the holes list has exactly one `lib::probe` "a foreign item" entry. Only the driver test can pin these, since the runner dedups. The doc comment says the same.
- **Codex blocking 1 / Opus O1 is resolved.** `visit_item` sends every `ItemKind::ForeignMod` through `granted` (`walk.rs:1672-1674`), empty blocks included. On the paths I checked, `#[link]` only takes effect on an extern block, and edition 2024 rejects a plain `extern` block. So every native-link route a user can write (`#[link]` with any kind or modifiers, an empty block or not, `extern "Rust"` too) now needs the grant. The other link-affecting attributes (`link_section`, `export_name`, `no_mangle`) were already gated as unsafe attributes, and the driver passes no user `-l`/`-L`.
- **N1 is resolved.** The 03 UNSAFE-1 text and the ungranted case comment no longer name the foreign `unsafe fn` separately.
- **N2 is resolved.** It is on fx-djp (comment 9).
- **N3 needs nothing.** No new-surface items were added beyond the empty linked block in the ungranted case and the `probe` and empty-block lines in the driver test. All of them are needed: they are the regression inputs.

Specific checks you asked for:
- **Extern blocks the toolchain or std generate.** None are flagged:
  - Neither `tide-std` nor `tide-macros` contains an extern block.
  - The test harness and `#[tide::main]` emit none.
  - `granted` still skips `written_by_other_crate` and `generated_by_toolchain` (AstPass, core's `#[test]`, the entry `main`).
  - A block an external macro writes is skipped, and so are its items, which share that expansion. A block a local `macro_rules!` writes is counted. Both match the existing UNSAFE-1 ownership policy.
  - The only corpus cases with extern blocks are the two UNSAFE-1 cases. The runner compares line:code sets, so an unmarked extra FX0901 elsewhere would have failed the recorded 193-PASS run.
- **Interaction with `granted`.** `self.item` is set to the ForeignMod before the match. So the hole's holder is the block itself, or, for a block nested in a fn body, the block as the item (because `is_descendant_of(block, fn)` holds). Its key is the enclosing path (`lib`, or `lib::f`), its kind is "foreign module", and it is a site hole counted once per span. That is why two crate-root blocks give `count: 2`.
- **Is the attribution sensible?** Yes, and it matches the other site holes. The artifact names the module and a count, not the exact block. Adding `#[link]` to a block that already holds a hole does not change the artifact, which is the same as editing an existing `unsafe {}` body. That predates this change and is not introduced by it.
- **Design accuracy about `#[link]`.** "`unsafe extern` blocks, which link the native libraries their `#[link]` names even when empty" matches the Rust Reference (items.extern.attributes.link.empty-block). Whether a dylib nobody references survives `--as-needed` depends on the modifiers. "link" is still correct at the rustc level, and the claim does not depend on that.
- **Behaviour beyond the findings / trust boundary.** The block check fires for every block, not only empty or `#[link]` ones. So a block with n items now gets 1 + n FX0901 without the grant, and an existing granted package's artifact gains a "foreign module" row. That is a deliberate, documented tightening: 03, reference.md and rules.md all say "blocks and items". It is simpler than gating on `#[link]` and loosens nothing. The B1 fix removes the redundant "an `unsafe fn`" hole for unsafe foreign fns, which was the intended effect. No check crosses or relaxes a trust boundary.

## Findings

N1 (nit, text-only, non-blocking): "each block" in the SAFETY-comment rule is ambiguous now that extern blocks are listed next to it.
- **Where:** `design/reference.md:955`. The Spelling cell now lists "`unsafe extern` blocks", and the Rule cell still says "each block needs a `// SAFETY:` comment (`FX0902`…)". This can now be read as requiring a SAFETY comment on an extern block.
- **Same ambiguity:** `conformance/cases/authority/unsafe_granted.rs:3` says "with a `// SAFETY:` comment on each block" just before naming `unsafe extern` blocks. The case's own extern block (line 20) has no such comment.
- **Fix:** write "each `unsafe { .. }` block" in both places.

N2 (nit, text-only, non-blocking): the edited lines were not rewrapped.
- `conformance/cases/authority/unsafe_ungranted.rs:4` is 118 columns. The neighbouring comment lines are at most 100.
- `design/03-authority-concurrency.md:707` ("trait`), Rust 2024's unsafe") is a short line in the middle of a paragraph.
- `crates/tide-admit/src/walk.rs:1036` ("/// foreign item); a `const fn`") is a 35-column line inside the doc comment.
- **Fix:** reflow the three paragraphs.

N3 (nit, text-only, non-blocking): the conformance corpus does not show an empty linked block being admitted.
- **Where:** `unsafe_granted.rs` has only a non-empty block. Under the grant, the empty `#[link]` block's admission and its hole are covered only by the driver test (`driver.rs`, `extern_blocks_and_their_items_need_the_unsafe_grant`), which is enough for the Acceptance.
- **Optional fix:** add `#[link(name = "m")] unsafe extern "C" {}` to the granted case so the corpus mirrors the ungranted case.

## Verification

I only read; I edited no files and ran no builds or tests. What I read:
- `br show --no-db fx-kw8.23` and both round-1 reviews;
- `git diff dd40cf6 d6ff4f1`;
- in d6ff4f1's `walk.rs`: `generated_by_toolchain`, `hole`, `granted`, `take_holes`, `written_by_other_crate`, `fn_item`, `visit_item` and `visit_foreign_item`;
- `crates.rs` `key`;
- `conformance/run.sh`'s mark matching;
- the 03 UNSAFE-1 paragraph;
- `reference.md:955`;
- fx-djp's comments.

`git grep` found extern blocks only in the two UNSAFE-1 cases: none in `tide-std`, `tide-macros` or the driver source.

The recorded `cargo test --workspace` (exit 0) and `conformance/run.sh` (193 PASS) on d6ff4f1 were not rerun.
