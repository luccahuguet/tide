# fx-zr3 round 1: Opus review of the seal-mechanism gaps

Candidate: commit fc37e5e on branch `fx-zr3` (base 8c9a38a), diff
`git diff 8c9a38a fc37e5e`: `design/01-design.md`, `design/04-contracts.md`,
`conformance/rules.md`.

Model: claude-opus-5-5 · Date: 2026-11-07

## Verdict

`request_changes`, confidence medium-high.

The candidate applies all thirteen edits in the Opus decision as written. It
changes only the three files the decision names, leaves `design/reference.md`
untouched, and does not touch SEAL-4 or SEAL-5. The SEAL line references in
`rules.md` match the new bullet starts (1265, 1293, 1350, 1367, 1385, 1389),
and no live file still says "before rustc runs". I agree with all five
decisions:

- phase: in `after_analysis`;
- identity: path and kind, extending the trust artifact's key;
- release: clauses are type-checked but never evaluated;
- writer: the driver's seal mode;
- manifest: the driver as sole reader, with the caller's grants as a ceiling.

The reasons hold against the code (`invocation.rs` stages output, and
`crates.rs:143` gives the canonical-path key) and against research/06's
"derived index".

Four precision gaps remain in the new v0 text. All of them are text-only. Two
leave an implementer able to build a reading that reopens the UserId/OrderId
hole that the rename rule exists to close. The other two contradict the
worked example or the "moving code changes nothing" promise. All four are in
tiers the change edits, so they block.

## Findings

### B1. The rename rule does not say which IDs the seal records (text-only)

- **Where:** `04-contracts.md:1301-1305` (SEAL-2) and `:1290-1291` (SEAL-1's
  format, "the IDs SEAL-2's rename rule compares").
- **Failure:** "the seal records the IDs this compares" has two natural
  readings, and both are holes:
  1. *IDs from the walk only.* The seal records just the IDs of the items the
     hashed walk reaches. Seal `fn lookup(id: UserId)` while an identical,
     unreferenced `struct OrderId(u64)` exists. The delegate deletes `UserId`
     and writes `lookup(id: OrderId)`. The hash is unchanged, the old path no
     longer resolves, and `crate::OrderId` is not among the recorded IDs. So
     "its new ID named no item at seal time" looks true, the change reads as
     a rename, and it is admitted. That is the exact replacement the rule was
     written to reject.
  2. *IDs as a set.* Seal `fn lookup(a: UserId, b: OrderId)`, with both types
     identical and both referenced. The delegate swaps them to
     `lookup(a: OrderId, b: UserId)`. The positional encoding is
     byte-identical, the set of IDs is unchanged, and no reference reads as
     renamed or replaced, so the change is admitted.
- **Second gap:** the rule mixes "old *path*" with "new *ID*", and the ID is
  path *and kind*. So a braced `struct UserId` renamed to `Uid`, alongside a
  new `fn UserId()`, reads as a replacement because the old path still
  resolves in the value namespace. That error is conservative but arbitrary.
- **Fix:** in SEAL-2 and the SEAL-1 format, say that each hash records the ID
  of the crate item at each walk position. Say that the seal records, per
  configuration, the IDs of every crate item at seal time. Say that when the
  hashes are equal, the check compares the recorded IDs position by position.
  Write "its old ID resolves to no other item".

### B2. Ordering impl sets by "their own hashes" is not well-defined (text-only)

- **Where:** `04-contracts.md:1226-1227`.
- **Failure, recursion:** an impl's own hash is computed before the walk that
  sorts the impls. That hash walks to the impl's self type. If that type's
  encoding includes the impls that apply to it, the sort key needs itself.
  Positional citation does not break this, because a sort key is a separate
  walk. The text claims no fixed point is needed, but never says that a
  reached type is encoded without its impl set.
- **Failure, ties:** two impls can have equal own hashes: `impl A for Feed {}`
  and `impl B for Feed {}`, with `trait A {}` and `trait B {}` identical,
  since referenced names are left out. The tie order is unspecified, and it
  can matter in two ways:
  - It decides which ID sits at which position, so under B1's positional
    record the IDs come out as [A, B] against [B, A].
  - It decides the in-walk bytes when only one of the tied impls cites an
    item that was met earlier.

  If ties fall back to source or `DefId` order, moving one impl to another
  file gives a spurious FX0920. That contradicts "moving code between files
  changes nothing" on the same line.
- **Fix:** state that an impl's own hash, used as a sort key, is a walk rooted
  at the impl in which reached types are encoded without their impl sets.
  Impl sets enter a hash only where a whole entry or a trait lists them.
  State that ties break by the IDs the tied impls' headers cite, in walk
  order. A rename that reorders a tie then reads as a replacement, which is
  conservative, like the rename-plus-reuse case.

### B3. "Only when every one of them admits" contradicts the worked example's `--remove` (text-only)

- **Where:** `04-contracts.md:1283-1286` (the writer) against `:1473-1474`
  ("runs `tide seal --remove` on those three entries, its own `tide check`
  passes").
- **Failure:** in this chapter, "admitted" includes the seal check ("`FX0920`
  is an error: the crate is not admitted"). The delegate's tree fails FX0920
  on exactly the entries it removes, so a writer that requires every
  configuration to admit refuses the `--remove` the example shows succeeding.
- **Related gap:** the text does not say whether `--target` on a tree that
  violates a kept entry writes that tree's hashes for the new target. If it
  does, the seal fixes the edited item for that target only.
- **Fix:** "only when every one of them admits under the seal being written:
  the existing entries, less those removed, plus those added."

### B4. The seal format omits the old values the worked example's FX0920 prints (text-only)

- **Where:** `04-contracts.md:1287-1291` against `:1462-1469`.
- **Failure:** `tide check` has no base. It reads only `tide.seal`, and the
  format lists per-part hashes plus clause kind, hash, and text. The example
  prints values that a part hash cannot reproduce:
  - `authority  net -> net+fs  widened`;
  - `signature  + parameter \`cache: &Fs\``;
  - `grants  + "unsafe"`.

  SEAL-3's rule requires only the part names, but the format reads as
  exhaustive. fx-kw8.17 would have to choose between the design's format and
  its own worked example. The clause text was added to the format for exactly
  this purpose.
- **Fix:** add to the format "and the values the diagnostic compares: the
  declared authority, the signature's text, and the manifest's grants".
  Alternatively, mark the example's detail as illustrative.

### N1. Owner map and fx-mhd constraint not recorded

Acceptance asks that the decisions name the owners matching fx-kw8.15, .16,
.17 and .20. The design text names the owners as driver parts. The bead-ID
map, and decision 3's constraint on fx-mhd, are not yet on fx-zr3 or fx-mhd.
The fx-mhd constraint is that release lowering keeps clauses type-checked and
never evaluated. Record both before closing.

### N2. §2.6's new sentence over-promises, and its restriction is invisible to agents

- `04:636`, "none of it runs or is compiled into the program", contradicts the
  next sentence's `release_checks = "pub_requires" | "checked"`. Write
  "by default".
- The consequence "a clause cannot name an item the release configuration
  leaves out" is a new author-visible restriction: a
  `#[cfg(debug_assertions)]` helper called from an `ensures` fails in
  release. Reference §10.1 still says only "Release builds erase clauses".
  This is acceptable given the budget and how rare the case is, but it is
  worth one clause if the reference is trimmed.

### N3. "Before rustc emits anything" is looser than SEAL-3

`01:743-744` and `:982` say "before rustc emits anything", and rustc does emit
analysis diagnostics. SEAL-3's list (no metadata, trust artifact, executable,
or cache entry) is the precise statement. "Before rustc writes any artifact"
would match it.

### N4. Lost rationale for the rename rule

Neither D14 nor SEAL-2 says why path-equality is not enough: identical
definitions hash alike, so content alone cannot tell `UserId` from `OrderId`.
Add a clause to SEAL-2.

### N5. SEAL-2's opening still says it resolves names to stable IDs

`04:1294`, "It resolves names to stable IDs and content hashes as §11 does",
now reads as IDs inside the hash, against §11's "leave out the names of the
items they reference". Write "It resolves names as §11 does".

### N6. Generic parameter names in impl-member IDs

An impl member's qualified path, such as `<Feed<T> as Iterator>::next`,
contains binder names, while hashes treat binders as positions. Renaming `T`
therefore changes the ID, and an interface-sealed impl member gives FX0920.
This is conservative. Say that binders in an ID are positional, or accept the
behaviour explicitly. The same applies to items nested in bodies: rustc's
`def_path_str` can print two of them alike. That one is fx-kw8.15's to
resolve.

### N7. Grant ceiling edges, for fx-kw8.20

- "Without a manifest, the caller's grants apply" means that deleting an
  unsealed `tide.toml` widens the package's grants to the ceiling. The trust
  diff shows this, but the case is worth stating.
- Whether `[dependencies]` grants are bounded by the ceiling is unstated.
- README "Status" ("operator inputs, like grants") must change when
  fx-kw8.20 lands.

### N8. Companion §18 is stricter than SEAL-2

`reference-companion.md:480-482` forbids moving "a function its contract
calls". SEAL-2's rename rule admits moving an unsealed referenced item. The
companion's advice is conservative and that file is unedited, so it is not a
blocker, but the two now disagree visibly.

### fx-djp comments

None. Every finding concerns v0 text.
