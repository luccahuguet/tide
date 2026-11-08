# fx-zr3 round 3: Opus confirmation review of the round-2 fixes

Candidate: commit ffec143 on branch `fx-zr3`, with the fix delta
`git diff e727498 ffec143`: `design/01-design.md`, `design/04-contracts.md`,
`conformance/rules.md`.

Model: claude-opus-5-5 · Date: 2026-11-07

## Verdict

`request_changes`, confidence medium.

- **Round-2 B1 is resolved** by stating the limit. The statement is exact and
  in the right place. Accepting it is right for v0.
- **Round-2 B2 is resolved as to cost and shape.** The refinement is well
  defined, deterministic, and polynomial, and it is correct on A/B/C and on
  shared references.
- **One new text-only blocker (R3-B1).** The residual-tie key is "the
  member's stable ID (path and kind)", but the members of the only unordered
  sets the text names are impls. §11 gives no impl an ID. So the tie-break
  key, and the table slot each impl takes, are undefined in exactly the case
  the worked example's last sentences rely on.

The SEAL citations in `rules.md` match the bullet starts (1315, 1370, 1461,
1479, 1497, 1501). `git diff --check e727498 HEAD` is clean. `reference.md`
is unchanged and still agrees with SEAL-1..6.

## Findings

### Round-2 B1 (stale inventory): resolved

- **Exact.** SEAL-2 now claims detection only of shadowing while the old ID
  resolves, and of replacement by an item present at seal time. The example
  then names the hole: an unsealed referenced item is deleted, and an
  identical item added after the record was sealed takes its place. That is
  the whole hole. It needs (a) the old ID resolving to nothing, (b) the new ID
  absent from that record's inventory, and (c) an equal content encoding.
  Each case outside it is still rejected:
  - the old item still exists;
  - the new item was present at seal time;
  - the new item was deleted and re-added;
  - the inventory was refreshed;
  - the referenced type is sealed. Types are always sealed whole, so their
    path is fixed.
- **Honestly placed.** The limit sits in SEAL-2, next to the rule it limits.
  The sentence "The reader has only the fixed record and the candidate"
  explains why the hole cannot be closed there. Nothing in `reference.md`
  over-promises. It says only that *sealed* items are fixed, and that stays
  true.
- **Acceptable for v0.** The limit does not defeat SEAL-2:
  - An interface entry already "does not fix a named type's path".
  - SEAL-6 already tells the orchestrator to seal the modules that define
    the types its interfaces name.
  - The new advice ("seals it whole, directly or among its defining module's
    public items") is the same remedy, and it closes the case outright
    through the vacancy test.

  Closing the hole fully would need SEAL-4 to compute the base tree's ID
  inventory, which is a second rustc analysis of the base per target and
  configuration. It would also give `tide check` a different verdict from
  the gate. That cost belongs to the gate's implementation bead, not this
  text.
- **Re-addition rule: coherent, and within SEAL-1..6.**
  - It fills a writer case SEAL-1 left unstated.
  - It keeps re-sealing unchanged items an ungated no-op, as before the
    record hash existed.
  - A refresh is "remove, then add". SEAL-4 already gates that ("removing an
    entry or changing any of its hashes"), and the new sentence says so
    explicitly.
  - A kind change still replaces the record, as `--whole` did before.

  No new semantics. Nit: "limits detection of later replacements" reads as
  "replacements made later", which is all of them. "Replacement by items
  added after the record was sealed" says what is meant.

### Round-2 B2 (factorial tie rule): resolved as to cost and shape

- **Well defined.**
  - The initial colours are ranks of the name-independent node contents,
    with the root marked.
  - Each round ranks the tuple (previous colour, incoming multiset, outgoing
    multiset). Edge labels carry the direction, the role and any ordered
    position. Unordered membership carries no position.
  - Ranks are dense and assigned in lexicographic order over the whole graph,
    so they are invariant under isomorphism and independent of node
    enumeration.
  - Each colour contains its previous colour, so a partition only refines.
    "Stop when it no longer splits" therefore ends after at most n rounds.
  - Each round costs roughly O(m log m), so the whole is polynomial.
- **Deterministic**, given a total key for residual ties. See R3-B1.
- **A/B/C works.**
  - `A` and `B` start with equal colours.
  - In round 1, `A` gains the incoming (dyn-trait, from `Out`) edge, so the
    two split.
  - In round 2, their impls split by the colours of the traits they point to.
  - With no residual tie, all six orders give one walk and one table.
  - Retargeting `Out` to `B` fails at `A`'s slot: `A` still resolves.
- **Shared references.** Refinement only orders the walk. Visit positions
  still carry sharing and cycles, and distinct equal nodes stay distinct. A
  pair that 1-WL cannot tell apart still gets a complete encoding, so 1-WL's
  incompleteness costs only rejections, never collisions.
- **"Can conservatively give FX0920" is accurate.**
  - **When the tied members are interchangeable** (symmetric in the graph),
    the encoding is unchanged when a rename swaps their order. The positional
    ID check then fails, because the other member was present at seal time.
  - **When they are tied but not interchangeable**, the encoding can change
    too.
  - **When a rename keeps the order**, it passes.

  So "can" is right. The consequence is only ever a rejection, never an
  admission.
- **Name independence.** §11 now says "unless it reorders a residual tie",
  and D14 says "except for 04 §11's residual-tie ordering". Both are honest
  qualifications, and the cost is the one round 2 accepted. One consequence is
  stated only implicitly: the spec-closure hash shares this order. If a root
  spec's closure ever contains an unordered set, a pure rename that reorders a
  residual tie would change that hash, which v0 classes as "undecided" and
  gates. That is conservative. Today impl sets enter only where SEAL-2
  requires them, so this is not a finding.

### R3-B1. Impls have no stable ID, yet they are the residual-tie members (text-only, blocking)

- **Where:** `04-contracts.md:1216-1219`, the ID sentence: "its path and
  kind ... impl members by qualified path". Also `:1242-1251`, "breaking
  remaining ties by the member's stable ID (path and kind)" and "one slot per
  reached crate item". Also `:1273-1276`: "their stable IDs settle the
  order", about the two tied impls.
- **Problem.**
  - The unordered sets the text names are impl sets.
  - An impl has no path. The ID sentence covers impl *members* (as in
    `<Entry as PartialEq>::eq`) but not impls.
  - The tie-break key is therefore undefined for the members it most often
    applies to.
  - It is also unclear whether impls take table slots, which matters for the
    rename test and for the inventory that whole entries need ("the impls of
    it in the crate at seal time").
- **The model.** Codex's model sidesteps this with synthetic impl IDs (`IA`,
  `IB`). These behave like IDs written into source, which D14 rejected.
  Renaming trait `A` leaves `IA` unchanged in the model, but not under any
  path-based ID.
- **Fix.** One clause in the ID sentence. For example: "an impl's ID is its
  qualified header (`<S as A>`, or `<S>` for an inherent impl), with generic
  binders as positions." Also say that impls are crate items in the table.
- **The case that remains.** Coherence makes trait-impl headers unique.
  Inherent impls that share a header can tie only if they declare no items:
  otherwise their own item names separate them, or rustc reports E0592. Such
  empty impls are interchangeable, so their order cannot matter. Say so, or
  leave it to the identity owner.

### Leftovers and contradictions: none blocking

- **SEAL-4 still says "recomputing every hash it records".** It can recompute
  content hashes but not record hashes, which SEAL-1 says are never
  regenerated from candidate source. SEAL-1's new sentence ("SEAL-4 repeats
  the content and ID comparisons and compares these record hashes") settles
  what is meant. Nit.
- **Companion §18 still says `tide check` rejects deleting "the definitions
  of the types its signature ... names".** For the stated stale-inventory
  case, the check does not. This is delegate advice, stricter than the
  checker, in an unedited file, the same class as round-1 N8. Not a blocker.
  Align it when fx-kw8.15 or the companion pass touches §18.
- **The worked example's "body changed"** matches what the stored record
  supports. That fixes round-2 N3.
- **Round-2 N1 and N2** are fixed by the SEAL-1 clause and by the shadowing
  sentence.

### fx-djp comments

None. Every finding concerns v0 text.
