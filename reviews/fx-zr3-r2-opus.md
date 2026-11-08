# fx-zr3 round 2: Opus confirmation review of the round-1 fixes

Candidate: commit e727498 on branch `fx-zr3`, with the fix delta
`git diff fc37e5e e727498`: `design/01-design.md`, `design/04-contracts.md`,
`conformance/rules.md`.

Model: claude-opus-5-5 · Date: 2026-11-07

## Verdict

`request_changes`, confidence medium-high.

The fixes resolve Opus B1, B3 and B4. They also resolve the determinism part
of Opus B2 and Codex Blocking 1. The rebuttal of Codex Blocking 2 holds. The
inventory, the stored old values and the record hash are the least that
SEAL-1..6 need once the check reads only `tide.seal`, so none of them is new
seal semantics.

Two new problems remain in v0 text that the change edits, so both block, and
both have text-only fixes:

- **New B1:** the seal-time inventory goes stale, so SEAL-2 over-promises
  replacement detection. Its refresh behaviour is also unstated, and it is now
  gated.
- **New B2:** the tie rule is factorial in search and in what the seal
  stores. A polynomial rule exists, and it is simpler.

The SEAL line references in `rules.md` match the bullet starts (1307, 1357,
1437, 1455, 1473, 1477). `git diff --check` is clean. SEAL-2's covered parts
are unchanged. The corpus run is still pending: `rules.md` changed, so it must
pass before landing.

## Findings

### Round-1 findings

- **Opus B1 (which IDs the seal records): resolved.**
  - Positional tables close the parameter swap.
  - The record's complete inventory closes the case where an `OrderId`
    present at seal time replaces a deleted `UserId`.
  - "An ID includes kind" closes the namespace case.
  - The rename test reads "old ID resolves to no item".

  The rule has a gap for items added *after* sealing, reported as new B1.
- **Opus B2 (impl-set ordering): resolved as to definition.** There is one
  graph, with no recursively computed sort keys, and the self type reuses its
  node. Ties are settled name-independently, so moving an impl changes
  nothing. The rule is well defined: it takes a finite minimum over a finite
  product of permutations. Its cost is new B2.
- **Opus B3 (writer against `--remove`): resolved.** "Admits under the proposed
  seal", which drops removed entries, makes the worked example's `--remove`
  succeed. Checking every retained target closes the `--target` gap. The added
  ceiling sentence makes the example's `grants = ["unsafe"]` consistent with
  decision 5.
- **Opus B4 (old values for FX0920): resolved** for the signature, authority
  and grants lines. A nit remains for the test's body line (N3).
- **Codex Blocking 1 (canonical tie rule): resolved as to determinism.** In the
  A/B/C case, the reference from `C::Out` breaks the symmetry under the
  minimum. Retargeting `Out` to `B` fails the ID comparison, because `A` still
  resolves. The cost is new B2.
- **Codex Blocking 2 (rename plus reuse): the rebuttal is sound. Keep the
  rejection.**
  - **Final states only.** A path/kind ID system compares only final states.
    Codex 2's final state (`valid = P` fresh, clause calls `valid`,
    `pred = Q`) cannot be told apart from "copy `pred` to `valid`, redirect
    the clause, edit `pred`". That second history replaces the item a sealed
    clause names, which pre-change SEAL-2 says changes the entry.
  - **Shadowing.** There is a stronger reason than the history argument the
    text gives. The vacancy test ("old ID resolves to no item") is the only
    thing that catches shadowing, which SEAL-2 names. A fresh item that
    shadows a glob-imported `pred`, with identical content, passes every
    other test. So no rule that treats IDs uniformly can admit Codex 2's case
    and still reject shadowing.
  - **What could admit it.** A rule could admit it only in one of two ways:
    - Exempt functions by kind, on the argument that a called function's
      meaning is its hashed body. This is plausibly sound for clause-called
      functions but not for nominal types or traits, and it is a new
      kind-dependent rule.
    - Track history with persistent IDs, which decision 2 rejected.

    Both go past the minimum. The rejection is conservative: it admits
    nothing SEAL-2 forbids.
  - **Scope.** Pre-change SEAL-2 did not define this snapshot: it is both a
    rename and a replacement. Choosing to reject it settles that ambiguity and
    adds no semantics.
  - **Attribution.** `CODEX-REPORT.md` credits the "soundness-first fallback"
    to the user. It was the orchestrator's instruction. Do not record it on
    the bead as an owner decision.
- **Opus N1-N8:**
  - N2, N3, N4 and N5 are fixed.
  - N1: comment 175 records the owner map and the fx-mhd constraint, as the
    report says. I did not re-verify that comment beyond the report.
  - N6, N7 and N8 are deferred to fx-kw8.15, fx-kw8.20 and the unedited
    companion, as intended.

### Scope ruling (question 2)

None of the additions is new seal semantics:

- **The inventory** is what the decision's "its new ID named no item at seal
  time" needs in order to be checked from `tide.seal` alone. My round-1 B1 fix
  asked for it.
- **The stored old values** are what SEAL-3's existing diagnostic and the
  worked example already print.
- **The record hash** is the least needed to keep SEAL-4's existing promise
  once the seal holds inputs to admission. Without it, a merged candidate
  could shrink an inventory without touching any gated hash. The next
  candidate could then use a removed ID as a "rename" target, which reopens
  the replacement hole.

Two parts of the fixes go past the minimum:

- The record hash has an unstated side effect (new B1(b)).
- The multi-table tie machinery is mechanism the contract does not need
  (new B2).

### New B1. The seal-time inventory goes stale (text-only)

- **Where:** `04-contracts.md:1325-1356` (SEAL-1, the inventory and the record
  hash) and `:1357-1388` (SEAL-2's rule and its claim "together detect
  replacing or shadowing").
- **(a) Over-promise.**
  - **Setup.** At t0 the orchestrator seals `fn lookup(id: UserId)` as an
    interface entry and leaves `UserId` unsealed, which SEAL-2 permits ("does
    not fix a named type's path"). At t1 an identical `struct OrderId(u64)` is
    added and merged as an unrelated addition. At t2 a delegate deletes
    `UserId` and changes `lookup` to take `OrderId`.
  - **Result.** The record's table says `UserId`, which resolves to nothing,
    and `OrderId` is absent from the t0 inventory. So the change counts as a
    rename, the content hash is equal, and it is admitted.
  - **Why it matters.** This is the UserId/OrderId replacement the rule exists
    to reject. It only fails to be rejected because the replacing item is
    newer than the record. Records never refresh ("does not replace a kept
    record's tables or inventory"), so every record's protection decays as
    the crate grows.
  - **Why the seal file cannot close it.** No reader of only `tide.seal` and
    the candidate can close it: the t2 tree can equal an honest t2 rename of
    `UserId` to a fresh `OrderId`. Only the base can tell them apart, and only
    the SEAL-4 gate has the base.
  - **Where it is already closed.** If `UserId` is sealed whole, as SEAL-6
    advises ("the modules that define the types ... its interfaces name"),
    its path is fixed. The vacancy test then closes the case outright.
- **(b) Refreshing a record is unstated, and it is now gated.**
  - **Before.** Re-running `tide seal crate::feed` on unchanged items wrote
    identical hashes, so SEAL-4 gated nothing.
  - **Now.** The record hash binds the inventory, so re-sealing replaces the
    record and changes the hash. That is `FX0921` on a protected branch, even
    when no content changed.
  - **The gap.** SEAL-1 says "plus the entries explicitly added or replaced",
    but not whether re-adding an existing entry of the same kind keeps its
    record or replaces it.
- **Fix, text-only, in the SEAL-2 example paragraph and SEAL-1:**
  1. State the limit. A record's inventory is fixed when that record is
     sealed. An item added later can therefore stand in for a deleted,
     unsealed referenced item. An orchestrator that relies on a named item's
     identity seals its defining module (SEAL-6), which fixes the item's
     path.
  2. Weaken "together detect replacing or shadowing" to match.
  3. State that re-adding an existing entry of the same kind keeps its record
     when its SEAL-2 comparison passes. This keeps the pre-change ungated
     behaviour. Refreshing a record is then `--remove` followed by an add,
     which SEAL-4 gates.

  The mechanism that keeps the full promise exists: the gate compares the
  candidate's IDs with the tables and items of the base tree it already
  checks against. It belongs to SEAL-4, which this bead excludes, so file it
  as a follow-up rather than add it here. If the orchestrator judges that
  item 1 changes "what a seal fixes" under the bead's stop condition, that is
  the owner's call. My view is that it does not: the covered parts are
  unchanged, and SEAL-6 already makes the identity of unsealed named items
  the orchestrator's job.

### New B2. The tie rule is factorial in search and in output (text-only)

- **Where:** `04-contracts.md:1225-1248` (§11, "Canonical order and ties") and
  `:1368-1379` (SEAL-2's "every minimizing table" and "at least one pair of
  whole tables").
- **Search cost.**
  - A set of k impls costs k! orderings, and nested sets multiply.
  - "May prune only if it gives the same result" rules out nauty-style
    canonical forms, because nauty's form is not the lexicographically least
    DFS encoding.
  - Automorphism pruning would keep the result, but the text does not name
    it.
  - Impl sets and associated types can encode arbitrary graphs. Any
    *complete* canonical form here is therefore graph canonisation, with no
    known polynomial bound. So naming nauty would not give a bound either.
- **Output cost, which is worse.**
  - Take a type sealed whole with k identical crate marker traits, the
    `trait Sealed {}` pattern. It has k! distinct minimising ID tables, and
    the seal must "keep every distinct whole table".
  - "An equivalent compact representation" quietly requires a
    permutation-group representation.
  - "At least one pair of whole tables must satisfy every comparison" then
    becomes a coset-matching search.
  - fx-kw8.15 and fx-kw8.17 cannot implement this within a bound.
  - A delegate can add impls to a trait sealed whole ("new impls of it may be
    added"). If those impls enter the graph, the cost becomes a checker
    slowdown the delegate controls.
- **Fix, text-only, which also removes text:**
  1. Order members that have no source order by colour refinement over the
     covered graph (1-WL partition refinement, polynomial). This is
     name-independent and needs no recursive hash keys.
  2. Break only the ties refinement leaves by the IDs the tied members cite,
     in walk order.
  3. Record exactly one ID table.

  In the A/B/C case, refinement separates `A` from `B` through `C::Out`, so
  there is no tie and the round-1 spurious FX0920 does not occur. Moving an
  impl still changes nothing. The only cost is a conservative FX0920 when a
  rename reorders a residual tie. That case needs equal-content crate items
  in one unordered set, and it is the same class as the rename-plus-reuse
  rejection already accepted.

  Delete "every minimizing table", "compact representation", and "at least
  one pair of whole tables". Refinement is only an ordering key. The complete
  DFS encoding is still the content hash, so distinct graphs never collide,
  and 1-WL's incompleteness costs at most a conservative FX0920.

  If the author keeps the lexicographic minimum instead, the text must at
  least do two things:
  - allow automorphism pruning by name;
  - replace "keep every distinct whole table" with one table chosen by cited
    ID among the minimising ones, with the same conservative consequence.

### N1. "Seal hash" now means two things (text-only)

- SEAL-1's opening line, §2.6's "a release seal hash", and SEAL-4's "changing
  any of its hashes" now mean the record hash.
- SEAL-4's "recomputing every hash it records" can only mean the content
  hashes and the ID comparison, because SEAL-1 says candidate admission "does
  not regenerate the stored record from candidate source".
- One clause in SEAL-1 would settle it. For example: "SEAL-4 recomputes the
  content comparison and compares these record hashes."

### N2. Give the shadowing reason in SEAL-2 (text-only)

The history argument in the `pred` paragraph is correct. The shorter reason is
that the vacancy test is what detects shadowing, as in the Codex 2 ruling
above. One clause would make the rejection read as necessary rather than
cautious.

### N3. Worked example: "body: call to `fetch_feed` changed" (text-only)

The stored values give the test entry only a body part hash. The driver can
derive "call to `fetch_feed`" only by noting that the reached interface entry
of `fetch_feed` itself failed. That derivation is plausible, but neither the
format nor the text states it. Either mark the detail as illustrative or let
fx-kw8.17 own it.

### fx-djp comments

None. Every finding concerns v0 text.
