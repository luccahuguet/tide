# fx-zr3 round 4: Opus confirmation review of the R3-B1 fix and the rebase

Candidate: ca3dbd2 on branch `fx-zr3` (fix delta `git diff 372d064 ca3dbd2`;
rebase checked with `git diff d9b13e0 ca3dbd2 -- conformance/rules.md design/01-design.md`).

Model: claude-opus-5-5 · Date: 2026-11-07

## Verdict

`request_changes`, confidence medium. One text-only blocker, fixable in a
single clause.

- **R3-B1 is resolved for impls.** Every impl now has a resolved,
  name-resolution-based ID that doesn't depend on source order, and impls
  take table slots and inventory entries. The residual-tie key is defined.
- **The rename consequence is stated honestly.** It agrees with SEAL-2 and
  with the round-2 ruling.
- **New blocker R4-B1.** The fix defines an inherent impl's "path" as
  `<SELF>::{MEMBERS}`. Read the way the trait case reads
  (`<Entry as PartialEq>::eq` is the impl path plus `::eq`), an inherent
  member's ID then contains its sibling members. The text does not rule this
  reading out. The plain alternative, `S::get`, collides for legal generic
  blocks. Either way, inherent member IDs are now ill-defined in text this
  change edits.
- **The rebase is coherent.** All six SEAL citations in `rules.md` are
  correct.

## Findings

### Question 1: the impl-ID definition (resolves R3-B1, except for R4-B1)

Each impl form:

- **Trait impls.** `<crate::S as crate::A>` with trait arguments. Coherence
  (E0119) makes trait-ref headers unique within a crate, so the ID is
  injective without members. The key is resolved, so moving code or changing
  imports leaves it alone.
- **Generic impls.** Binders are positional, predicates are canonical, and
  const binders carry their types. The impl's ID is well defined. Two impls
  that differ only by predicate cannot both exist (overlap). Even if they
  could, the predicates in the ID would separate them.
- **Blanket impls.** `for<type0><type0 as crate::A> where ...` is
  well formed and unique by coherence.
- **Const generics.** Covered (`const0: usize`). Const-expression headers
  need `generic_const_exprs`, which `-Zallow-features=` bars.
- **Auto-trait impls.** `unsafe impl Send for S {}` is an ordinary trait
  impl. Synthesized auto impls are not items and never enter.
- **Negative impls.** The ID omits polarity. This is harmless:
  `negative_impls` is unstable and barred in user code, and coherence forbids
  `Send` and `!Send` for the same type. Std impls are keyed by toolchain
  version. Nit only: one word ("polarity") would make the ID complete on
  paper.
- **Inherent impls.**
  - The sorted member list separates blocks with the same self type.
  - Non-empty blocks with overlapping headers have disjoint member names
    (E0592), so their lists differ. Blocks whose predicates differ are
    already separated by the predicates.
  - Empty blocks with the same header share an ID. The text's claim that they
    are interchangeable holds exactly, not just under 1-WL. They have equal
    content and identical outgoing edges, and they are referenced only
    through set membership, so swapping them is a true automorphism. The
    order can change neither encoding nor table.
  - A third empty block adds a slot and is detected.

On determinism and source order:

- No source order enters at item level.
- One residual dependency is local: binder positions follow declaration
  order, so reordering `impl<T, U>` to `impl<U, T>` changes the ID. This is
  the general "binders are positions" rule already applied to every
  definition, and the effect is only conservative.
- Not a finding.

### R4-B1. Inherent member IDs are ambiguous after the fix (text-only, blocking)

- **Where:** `04-contracts.md:1216-1230`, specifically "impl members by
  qualified path" together with "An inherent impl uses `<SELF>::{MEMBERS}`".
  Also SEAL-1 at `:1338-1340`.
- **Problem.** For trait impls, a member's qualified path is the impl path
  plus the member name (SEAL-1: `<Entry as PartialEq>::eq`). Applied the same
  way to the newly defined inherent "path", that gives
  `<crate::S>::{const N,fn get}::get`. Under that reading:
  - adding, removing or renaming any sibling in the block changes the IDs of
    all the members;
  - "The entry's own path stays fixed" then turns a delegate's added sibling
    helper into FX0920 for an interface-sealed inherent method. SEAL-1 makes
    such methods ordinary entries when a module is sealed.

  That is conservative, but it contradicts the intent that only sealed things
  are fixed.
- **The other reading fails too.** `crate::S::get` is not unique.
  `impl S<u8> { fn get() {} }` and `impl S<u16> { fn get() {} }` are both
  legal, since the self types don't overlap and E0592 doesn't fire. So the
  member ID needs the block's header but not its member list.
- **Fix.** One clause, for example: "An inherent member's ID is its block's
  header without the member list, then its name, such as
  `<crate::S<u8>>::get`. E0592 makes this unique, and adding a sibling does
  not change it."

### Question 2: the rename statement is honest, and consistent with SEAL-2 and round 2

- **A/B/C.** Renaming `A` to `Z` changes `A` and `<S as A>`. Neither old ID
  resolves, and neither new ID is in the inventory, so each slot passes the
  vacancy test. Colours keep the order, so the encoding is unchanged and the
  rename passes. The text's "although the rename changes `<S as A>` to
  `<S as Z>`" is accurate.
- **Without C.** The impls stay tied and the tie is broken by ID. Renaming
  `A` to `Z` reverses it. The encoding is unchanged (the two impls are
  symmetric), but the first slot now holds `<S as B>`. That ID is in the
  seal-time inventory, so the slot fails and FX0920 follows. The definite
  "gives FX0920" is correct for this case. The general sentence ("can
  conservatively give FX0920") remains correct for the general case.
- **Self-type renames.** These reorder tied headers in the same way, for
  example the impls of a whole-sealed trait. Stating it is accurate.
- **Consistency.**
  - SEAL-2 allows a referenced-item rename only if it passes the ID
    comparison and "preserve[s] the content encoding, including residual-tie
    order (§11)". §11 says "A rename leaves the hash unchanged unless it
    reorders a residual tie". D14 says "except for 04 §11's residual-tie
    ordering".
  - The new closing sentence is therefore a stated limit, not a
    contradiction.
  - Refinement still uses no IDs; only the last-resort sort does.
  - This is the same cost the round-2 ruling accepted, now made concrete
    because impl IDs carry names.
  - Rejection is the only outcome, never a false admission.

### Question 3: leftovers, the rebase, and citations

- **No new contradiction** beyond R4-B1. The tie-break sentence now points
  to the header IDs. Under "impls take slots", R3's inventory question is
  answered.
- **Nit: the example uses `crate::Copy`.** `Copy` resolves to
  `core::marker::Copy`. Right after the example, the text says "resolved
  paths, not import spellings", so `crate::Copy` reads like a spelling or
  like a crate-local trait. Use `core::marker::Copy`, or a crate trait with a
  different name.
- **Nit: "declared members" for inherent IDs.** Say "after `cfg`", to match
  the per-configuration inventory. A `#[cfg(test)]` helper makes the block's
  ID differ between configurations. That is coherent, because tables and
  inventories are per configuration, but the text should say so.
- **Citations.** The SEAL citations in `rules.md` match the bullet starts at
  ca3dbd2: SEAL-1 :1334, SEAL-2 :1389, SEAL-3 :1480, SEAL-4 :1498, SEAL-5
  :1516, SEAL-6 :1520. No other tracked file cites 04 lines past §11.
  Untracked `DECISION.md` still cites 1241/1247, but it is not part of the
  candidate.
- **The rebase is coherent.**
  - Main's d9b13e0 range (fx-aw6, fx-kw8.9) touched `01-design.md` only in
    D3's lossless-address rule, and `rules.md` only in the GLOBAL-4 row.
  - Both survive unchanged in ca3dbd2.
  - fx-zr3's edits to D11 ("before rustc writes any artifact"), D14 (path
    and kind IDs, the residual-tie exception) and D18 (seal check after
    analysis; the driver as sole manifest reader) don't overlap main's text
    and agree with SEAL-3.
  - The seal text in `reference.md` and companion §18 is unchanged and still
    consistent, apart from the companion §18 advisory nit carried from
    round 3.

### fx-djp comments

None. Every finding concerns v0 text.
