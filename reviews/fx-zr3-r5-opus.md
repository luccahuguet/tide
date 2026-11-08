# fx-zr3 round 5: Opus confirmation review of the R4-B1 fix

Candidate: 4337bb2 on branch `fx-zr3` (fix delta `git diff ca3dbd2 4337bb2`,
`design/04-contracts.md` only, +3/-1).

Model: claude-opus-5-5 · Date: 2026-11-07

## Verdict

`approve`, confidence high. The clause resolves R4-B1. It agrees with the rest
of §11 and SEAL-1, and it adds no new guarantee or contradiction. Three
non-blocking nits follow.

## Findings

### R4-B1 is resolved

- **No siblings in the ID.** The clause defines an inherent member's ID as
  "its block's header without the member list, then its name". This rules
  out the `<crate::S>::{const N,fn get}::get` reading by construction.
  Adding, removing or renaming a sibling leaves every other member's ID
  unchanged.
- **No collision.** `impl S<u8>` and `impl S<u16>` give `<crate::S<u8>>::get`
  and `<crate::S<u16>>::get`, because the header carries the self type's
  arguments. Two blocks whose canonical headers are identical (same self
  type, binders and predicates) overlap trivially. A shared member name then
  hits E0592, so header plus name is unique.
- **No ambiguous strings.** Member names never contain `::`, so splitting a
  member ID at its last `::` recovers exactly one header and one name, even
  when a header ends in a predicate path such as
  `where type0:core::marker::Copy`. Distinct (header, name) pairs therefore
  never give the same string. The ID also carries kind (§11 and SEAL-2, "An ID
  includes kind"), so a `const` and a `fn` could not be confused even if
  E0592 allowed both.

### Consistency with §11 and SEAL-1

- **Trait impl members.** `<Entry as PartialEq>::eq` is the impl header plus
  `::eq`. Inherent members now have the same shape, with the member list
  playing no part. Only inherent blocks have a list to drop, so "impl members
  by qualified path" (line 1218) now reads one way for both kinds.
- **Binders and predicates.** "The same binder and predicate rules apply"
  covers the member form, giving `for<type0><crate::S<type0>>::get`, and
  with predicates, the header's `where` clause followed by `::get`. Trait impl
  members with predicates already had this shape, so nothing new is
  introduced.
- **SEAL-1/SEAL-2 "The entry's own path stays fixed".** This now holds for
  sealed inherent methods and associated constants, which a module seal adds
  as ordinary entries. A delegate's new sibling helper no longer changes a
  sealed member's path, and no FX0920 follows. That restores the intent R4-B1
  described.
- **The block-level ID still uses the member list.** It is unchanged, and
  the member list is still needed there: it separates blocks that share a
  header, it breaks residual ties, and it fills table slots. A sibling edit
  still changes the block's ID. That only affects impl-set slots where SEAL-2
  brings blocks in, and it is the conservative cost accepted in round 3.
  Member IDs and block IDs are now independent, which is the right split.

### Nothing new introduced

The clause defines one form that was previously undefined. It adds no
guarantee beyond uniqueness and stability under sibling edits, and both
follow from rules already in the text (E0592 and the header rules). It also
agrees with D3's stricter "one inherent method per name per type" (FX0408).
If D3 is read per type constructor, it makes the header redundant for
uniqueness, but it does no harm.

### Nits (non-blocking, text-only)

1. **"E0592 makes it unique" has two gaps.** E0592 covers duplicates across
   blocks, and E0201 covers duplicates within one block. Blocks with
   different headers are separated by the header itself, not by E0592. A
   possible wording: "the header and E0592/E0201 make it unique".
2. **Line wrap.** The rewrapped line 1231 ("sibling leaves it unchanged. The
   same binder ... with the same") runs past the paragraph's width.
3. **Scope of "The same binder and predicate rules apply".** The sentence now
   follows the member clause, but it is meant to cover both the inherent
   block ID and the member ID. It reads correctly as is. Moving it before the
   member clause would make that scope clearer.
