# Tide consistency review, round 43: sealed delegation boundaries (fx-1am)

## Verdict

**request changes**

Confidence: high on B1, B2, B6, B7, and B8, which are textual; medium-high
on B3, B4, and B5, which turn on readings the text leaves open. Every fix
is small, and none needs D11's approval model changed.

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `5ec5c70`
("Decide the Pure status of std types that erase a payload (fx-cze)"):
`git diff -- design/ README.md`, which changes:

- `04-contracts.md`: the tier paragraph, §11 "Classifying changes" (v0
  labels a removed `ensures` or an added `requires` `weakened`), and the
  new §11.1 (SEAL-1..SEAL-6 and the worked example);
- `01-design.md`: the tier table, and D11's "What needs approval" and
  new "Sealed delegation boundaries" bullet;
- `reference.md`: the header, §11's pointer, and §13's trim;
- `reference-companion.md`: the header and the new §18;
- README's v0 core list.

Read for context: bead `fx-1am` and its 2026-10-20 comment;
`research/13-agent-fleets.md` (needs 2 and 3, implication 3, open
question 1); 01's tier rules, D11, D12, D14, D17, and D18; 03 §2.5
(ADMIT-1, ADMIT-2), ROOT-3, §4.1, and §4.3; 04 §1.8, §2.1, §2.4, §8, and
§11; reference 2.2, 2.3, 6.7, 6.8, 6.9, 11, 12.4, and 13; and companion
15.2. The core token count (24,996) was not rerun.

## Scope answers

| Question | Answer | Evidence |
| --- | --- | --- |
| Contradictions between 01, 04, the reference, and the companion | **Two.** The core reference says sealed items "cannot change at all", which forbids the delegated work (B2). D11 rejects "seal changes gated by approval", which SEAL-4 adopts (B8). Otherwise D11, the tier table, §11's spec-closure hash, D14's stable IDs, D17's policies, and D18's admission driver agree with §11.1. | `reference.md:966–967`, `01:747–749`, `04:1278–1289` |
| Escapes | **Four real holes the text does not admit:** types and methods outside a sealed signature's hash (B3), a sealed test's oracle running through unsealed production code (B4), the manifest's checking budget (B5), and SEAL-3 failing on a sealed `#[cfg(test)]` item, a defect in the rule rather than an escape (B1). Routing around sealed functions and new trust holes under sealed grants are admitted by SEAL-6. Renames, moves, deleting and recreating an item, stable-ID reuse, edits to `tide.seal`, `cfg` variants, and new dependency grants are all caught, with caveats in N1, N2, and N4. | escape table below |
| Acceptance criteria | **Partly met.** 01 gives the feature, its tier, and rejected alternatives with reasons (one reason is misworded, B8). The rules cover who may unseal, what the checker reports, and the trust diff. "What is sealed" is incomplete (B3, B5). The worked example exists, but its interface cannot be implemented (B6), one of its claims is false (B7), and its own seal fails the normal build (B1). The bead's verification criterion fails on B2 and B8. | bead `fx-1am` Acceptance |
| D11 approval model changed (stop condition)? | **No.** Same signed records, the same human-held keys, the same CI verification against the protected base. The change adds one gated change, the way `#[protected]` tests were added, and one change class (`unseal`). An unprotected base is written by its owner, as §11 already implies. The record's fields need one sentence of adaptation (N3), which is not a model change. | `01:724–732`, `04:1222–1231`, `04:1278–1289` |
| Needless complexity | **None is blocking.** Each part carries a guarantee: FX0921 is separate from FX0920 because the seal file merges with the candidate, and declared authority is hashed in its own right for private capability fields. `#[protected]` and sealed tests overlap (N5). | N5 |
| Worked example valid? | **It is valid Rust as written, apart from `Url`'s impls, which nothing states (N6). Its design fails:** the sealed interface cannot be implemented, its seal breaks the normal build, and one stated outcome is false. | B1, B6, B7, N6 |

### Escape attempts

| Attempt by a delegate | Result | Status |
| --- | --- | --- |
| Rename, move, or re-export a sealed item | Path is in every entry (`04:1250`, `04:1255`) | Caught (FX0920) |
| Delete and recreate a sealed item, or reuse its stable ID | The hash is compared whatever the ID | Caught |
| Change a sealed function's body | Allowed by design (`04:1254`) | Intended |
| Edit a helper a sealed clause calls | Covered by the spec-closure hash (`04:1251–1252`, §11) | Caught |
| Add an inherent method that shadows a trait method a sealed clause calls | Changes name resolution; §11 hashes resolved names | Caught for clauses; unstated for whole entries (B4) |
| Change a crate-local type, alias, constant, or trait that a sealed signature names but that is not itself sealed | The signature hash does not cover it (`04:1252–1254`) | **Hole (B3)** |
| Change the public methods of a type in a sealed module | "Public item" may not include associated functions | **Hole under Rust's reading (B3)** |
| Bump a dependency whose type a sealed signature names | `tide.lock` and dependency versions are not sealed | **Hole (B3)** |
| Rewrite a manual `impl PartialEq` that a sealed test's `assert_eq!` calls, or a non-test helper the test calls | Entries cover only "test-only items" and type definitions without impls (`04:1257–1262`) | **Hole (B4)** |
| Shadow a production function inside `mod tests` behind `use super::*` | Caught only if the whole-entry hash resolves names | Unstated (B4) |
| Lower the sampled-clause step budget in `tide.toml` | Not part of the manifest entry (`04:1263–1264`, `04:572–573`) | **Hole (B5)** |
| Lower the package `[contracts] use` | Changes each sealed function's contract use | Caught |
| Add a dependency with `unsafe`, or add `grants` | Part of the manifest entry | Caught |
| Add a dependency with no grants | Not a seal violation; D13 gates any authority it widens | Acceptable |
| `#[cfg(not(debug_assertions))]` or `feature` variant of a sealed item | FX0920 in any configuration that is built; the gate may build none of them | Caught late (N2) |
| Branch on `cfg!(test)` in a free body | Not a seal matter | Not admitted (N4) |
| Edit or delete `tide.seal` | FX0921 against the base's seal (`04:1280–1284`) | Caught at the gate |
| Merge into an unprotected orchestrator branch without running `tide review` | Nothing forces the gate | Not admitted (N1) |
| Route around a sealed function from unsealed callers | SEAL-6 | Admitted |
| Add a trust hole that the sealed grants allow | SEAL-6; D11 gates it | Admitted |

## Blocking findings

### B1. The worked example's sealed test fails SEAL-3 in the normal build

- **The rule.** "The admission driver (D18) checks each crate against its
  own `tide.seal` in every configuration it builds, normal and test ... A
  sealed item ... that is missing from a configuration, is `FX0920`"
  (`04:1269–1273`).
- **The example.** The sealed test `times_out_when_network_drops` is
  inside `#[cfg(test)] mod tests` (`04:1326–1337`), sealed at `04:1341`.
  In the normal configuration it does not exist, so the literal rule
  rejects every non-test build of the orchestrator's own tree. Any sealed
  item under `#[cfg(test)]`, `#[cfg(target_os = ..)]`, or
  `#[cfg(feature = ..)]` (all admitted, `reference.md:101–104`) hits the
  same rule. SEAL-1 does not say in which configuration `tide seal` reads
  "the current tree" (`04:1246–1247`).
- **Risk.** The rule cannot be implemented as written, and the obvious
  workaround, checking only configurations where the item exists, opens
  the hole the "missing" clause was meant to close: a delegate makes a
  sealed function `#[cfg(test)]`-only.
- **Fix** (04 SEAL-1 and SEAL-3, one sentence each). An entry records
  the configurations its item existed in at seal time, and `tide seal`
  reads every configuration. FX0920 fires when the item is missing from
  or differs in one of those configurations.

### B2. The core reference forbids the delegated work

- `reference.md:966–967`: "Items sealed in `tide.seal` cannot change at
  all (`FX0920`, companion 18)."
- SEAL-2: an interface entry fixes the interface, "The body is free"
  (`04:1254`). The companion agrees: "You may change the bodies of sealed
  functions" (`reference-companion.md:469–470`). The 01 bullet says a
  delegate implements behind the interface (`01:733–735`).
- **Risk.** The core is what every delegate loads, and the companion is
  loaded on demand. A delegate reading only the core is told it may not
  write the body it was asked to implement, which is the most common
  case. It may stop and report on every sealed `todo!()`. This is also
  the bead's verification criterion: no contradiction between 01, the
  chapter, and the reference.
- **Fix** (core, a few tokens; trim elsewhere to stay under 25,000).
  "Sealed items (`tide.seal`) cannot change, except a sealed function's
  body (`FX0920`, companion 18)."

### B3. A sealed signature fixes the names of its types, not their definitions

- **The text.** An interface entry fixes "path, signature, declared
  authority, contract use, and its contract's spec-closure hash"
  (`04:1249–1252`). The justification for hashing declared authority, "a
  capability field added to a parameter's type widens it without touching
  the signature" (`04:1252–1254`), shows that the signature hash does not
  cover the definitions of the types it names. A module seal covers "each
  public item in it" (`04:1243`) and "the items it has at seal time"
  (`04:1266`).
- **Escapes the text does not admit** (SEAL-6, `04:1294–1298`, admits only
  routing and new trust holes):
  1. *Named types outside the sealed set.* The orchestrator seals
     `pub fn store(db: &Db, rec: &Record)` in `crate::api`, and `Record`
     lives in `crate::model`. A delegate changes `pub link: Url` to
     `pub link: String`, removes a field, or adds an enum variant to an
     error type. The signature is unchanged, there is no FX0920, and
     parallel delegates written against the sealed interface break at
     merge. That is the collision research/13 need 3 asks seals to
     prevent. The same holds for type aliases (`type Feed = Vec<Entry>`),
     constants in array lengths (`[u8; N]`), and traits used as bounds
     (new supertraits or required methods).
  2. *Dependency types.* A signature naming `dep::Thing` changes meaning
     with a `tide.lock` bump. The manifest entry fixes dependency grants,
     not versions (`04:1263–1264`).
  3. *Public methods in a sealed module.* In Rust's terms associated
     functions are not module items, and a type's whole entry lists
     "fields, variants, generics, bounds, attributes, and invariants"
     (`04:1256–1258`) but not its methods. Under that reading, sealing
     `crate::store` leaves `Store::get`'s signature free: a delegate adds
     `fs: &Fs` to a public method in a sealed module.
- §11's spec-closure hash includes "type definitions" that the meaning
  depends on (`04:1198–1203`). That may cover the types a *contracted*
  function's clauses reach, but it does not cover a function without a
  contract, and whether it covers a parameter type a clause never reads
  is unclear.
- **Fix** (04 SEAL-2, two sentences; companion 18, one clause). An
  interface entry's hash covers the public shape of every type, alias,
  constant, and trait its signature names, transitively, by content hash
  (D14), dependency items included: public fields, variants, generics,
  bounds, derives, and the signatures of public methods. Private fields
  stay free, which keeps the separate declared-authority hash meaningful.
  A module seal covers the public methods of the module's types. If the
  author prefers not to widen the hash, SEAL-6 must say that a seal fixes
  named types only by identity and that the orchestrator seals them whole.

### B4. A sealed test's oracle can run through unsealed production code

- **The text.** "A test's entry also covers the test-only items it calls,
  transitively, so a delegate cannot weaken an assertion helper"
  (`04:1259–1261`). A type's whole entry does not include its trait impls
  (`04:1256–1258`). The companion promises more: "You may not change ...
  a sealed test or the test helpers it calls" (`reference-companion.md:470–471`).
- **Escape.** A sealed test asserts `assert_eq!(parse(s), expected)` on a
  type with a manual `impl PartialEq`. The delegate rewrites `eq` to
  return `true`, and the test passes for any `parse`. Likewise, a sealed
  test that calls a non-test helper (`pub fn valid(e: &Entry) -> bool`)
  sees it weakened. Neither is "test-only", so neither is fixed. The bead's
  need 2 is exactly "tests ... the agent cannot weaken". For clauses, §11
  closes this by hashing the impls behind operators (`04:1199–1201`). For
  tests nothing does.
- **Second gap.** Whether a whole entry hashes resolved names is unstated.
  §11 says so for spec closures (`04:1204–1205`) and SEAL-2 does not. If the
  hash is syntactic, adding `fn fetch_feed(..) -> .. { Err(..) }` inside
  `mod tests` shadows the glob `use super::*` (valid Rust), and the
  unchanged sealed test calls the delegate's stub.
- **Fix** (04 SEAL-2). (a) A type's whole entry covers its crate-local
  trait impls, derived or written. (b) A test's entry covers, by §11's
  rule, the impls behind operators and the functions called inside its
  assertions (`assert!`, `assert_eq!`, `matches!`), not only test-only
  items, while the code under test stays free. (c) Entry hashes resolve
  names to stable IDs, as §11's do. If (b) is judged too costly, SEAL-6
  must admit that a test's oracle may call unsealed production code, and
  the companion's sentence must narrow to match.

### B5. The manifest entry leaves out the settings that decide whether sealed clauses are checked

- **The text.** The manifest entry fixes "the package's grants, each
  dependency's grants, the authority policies (D17), and the contract
  uses" (`04:1263–1264`). The sampled-clause step budget is a package
  setting: "Packages can change the budget" (`04:572–573`), and a clause
  over budget "is reported as unchecked, never as checked or failed".
- **Escape.** A delegate lowers the budget in `tide.toml`. Every
  `sampled` clause of a sealed contract (CLAUSE-5, `04:451–457`: any
  quantifier or user call) and every generated property test then reports
  "unchecked", and the gate passes. The sealed contract's hash is
  unchanged, so there is no FX0920, and nothing in D11 gates the budget.
  The vacuity floor ("by default 100 accepted cases", `04:616–617`) is the
  same if it is configurable.
- **Fix** (04 SEAL-2, a few words; D11's "grants, policies, and uses"
  likewise). The manifest entry fixes the whole `[contracts]` table (uses,
  budgets, and test floors), not only the uses.

### B6. The worked example seals an interface no delegate can implement

- `pub fn fetch_feed(net: &Net, url: &Url, max: usize)` (`04:1322`). The
  only HTTP call is `Net::get(&Url, deadline: Instant)`
  (`reference.md:331`). An `Instant` comes from `Clock::now`
  (`reference.md:332`), and `Instant::now` is ambient (`FX0501`,
  `reference.md:549`). Companion 15.2 threads a `Clock` for exactly this
  reason (`reference-companion.md:194–197`). With no `Clock` or deadline in
  the sealed signature, no body can call `Net::get` and return
  `NetError::Timeout`, as the sealed test requires (`04:1335`).
- **Risk.** The acceptance's single example shows an orchestrator sealing
  an interface whose first honest implementation needs an interface
  change. Under SEAL-3 that change is rejected, so the delegate can only
  stop. The example teaches the wrong lesson and misses the one it sets
  out to show.
- **Fix** (04 example). Add `deadline: Instant` to `fetch_feed` (and
  `use std::time::Instant`). In the test, pass
  `sim.clock.now() + Duration::from_secs(1)`. The FX0920 diagnostics are
  unchanged.

### B7. "`tide check` passes in its tree" is false

- `04:1360–1362`: after `tide seal --remove crate::feed::fetch_feed
  --manifest`, "`tide check` passes in its tree".
- The delegate added a fourth parameter (`04:1345`), and the sealed test,
  still unchanged and still sealed, calls `fetch_feed(&.., &url, 10)` with
  three (`04:1334`). The test configuration does not type-check (rustc
  `E0061`, or the front end's equivalent). The delegate cannot update the
  call without a third FX0920 on the sealed test.
- **Fix** (04, one clause). "the seal check passes in its tree (the
  sealed test no longer compiles against the new signature, but suppose
  it also drops the test's entry)". Simpler: "its seal check passes". If
  B6's fix changes the call, recheck this sentence.

### B8. D11 rejects what SEAL-4 adopts

- D11's rejected alternatives: "seal changes gated by approval rather than
  rejected, which lets a delegate's loop run on against a changed interface
  that parallel delegates still assume" (`01:747–749`).
- SEAL-4 gates seal changes by approval: "Removing an entry or changing its
  hash is `FX0921` unless an approval record covers it" (`04:1282–1284`),
  and D11's own list adds "removing or changing a seal entry" to what needs
  approval (`01:727`).
- The intended alternative is "changes to *sealed items* gated by approval
  (as `pending_approval`) rather than rejected (FX0920 with no override)".
  Read literally, 01 rejects its own mechanism. The stated reason is also
  only partly true of the adopted design: a delegate that edits its own
  `tide.seal` keeps its loop running too. What differs is that this takes
  a deliberate edit, which review shows as FX0921.
- **Fix** (01, a few words). "changes to sealed items held for approval
  rather than rejected, which keeps a delegate's loop green against a
  changed interface that parallel delegates still assume; under the seal
  it must first edit `tide.seal`, which review reports."

## Non-blocking findings

### N1. An unprotected orchestrator branch is guarded only if the merger runs `tide review`

`01:742–744` says a delegate that edits `tide.seal` "gains nothing", and
`04:1360–1362` has "the orchestrator's merge" run `tide review`. Nothing
makes a merge into an orchestrator's unprotected branch run it. If the
orchestrator merges with plain git, the delegate's seal edit lands. When
the entries were added on that branch and never reached `main`, removing
them is never FX0921 against `main`. **Fix:** one sentence in SEAL-4 or
SEAL-6: the guarantee is mechanical only on a branch the CI gate writes;
elsewhere it holds when the merger runs `tide review <base>`.

### N2. "Normal and test" does not name the configurations the gate checks

`04:1270–1271`. `#[cfg]` admits `debug_assertions` and `feature = ".."`
(`reference.md:101–104`). A `#[cfg(not(debug_assertions))]` copy of a
sealed `main`, with wider attenuations, passes a gate that builds debug and
test, and fails admission only at the release build. That is not silent,
but it is late. **Fix:** name the configurations (debug, test, release,
and each declared feature set), and say that `tide review` and the CI gate
check all of them, or say that they check only debug and test and that
release admission is the backstop.

### N3. The approval record does not fit seal entries, and two lists were not updated

- §11's record holds "the item's stable ID, the old and new closure hashes,
  the change class" (`04:1222–1223`). The manifest entry has no stable ID,
  a removal has no new hash, and a seal hash is not a closure hash. One
  sentence in SEAL-4 should give an `unseal` record's fields.
- Unstated: whether an unseal that also weakens a contract or widens
  authority needs one record or two (`unseal` plus the D11 class).
- §11's "As D11 lists, approval is needed for ..." (`04:1217–1220`) and
  reference 11's approval list (`reference.md:963–967`) omit "removing or
  changing a seal entry", which `01:727` adds. For the reference, B2's
  sentence suffices.
- "Adding an entry is not gated" (`04:1281–1282`): a delegate can seal
  items in the base that another delegate must change, and on a protected
  branch undoing that takes a human approval. SEAL-5 lists added entries,
  so it is visible. Consider saying that the reviewer should reject
  additions the orchestrator did not make.

### N4. A free body can branch on `cfg!(test)`

A delegate can pass a sealed test with `if cfg!(test) { return
Err(..) }` in a free body, and ship different behavior. This is gaming of
the E1 kind, not a seal violation, but research/13 need 2 is the reason
seals exist. **Fix:** SEAL-6 admits it. Optionally, the trust diff lists
`cfg(test)` and `cfg!(test)` in non-test code.

### N5. Two mechanisms protect tests (admission filter)

`#[protected]` tests (D14, approval-gated) and sealed tests (rejected,
unsealed by approval) overlap. Neither text says how they relate or which
an orchestrator should use, and a test can carry both. GOVERNANCE asks
whether a feature adds overlapping ways to solve one problem. **Smallest
fix:** one sentence in §11.1 (a sealed test that is also `#[protected]`
follows the seal; use `#[protected]` for tests that outlive a delegation).
**Alternative worth weighing:** define a `#[protected]` test as a test with
a seal entry on the protected branch, which removes one concept. Also, the
first reason given against `#[sealed]` ("sits in the files the delegate
edits", `01:745–746`) applies equally to `#[protected]`, which works
because CI compares with the base. The other two reasons (the manifest,
and one reviewable record of what was sealed) carry the rejection, so lead
with them.

### N6. Worked-example nits

- `Entry` derives `PartialEq` and `Decode` (`04:1308`), which need
  `Url: PartialEq + Decode`. Reference 6.7 gives `Url` only its accessors
  and `Display` (`reference.md:507–509`). Neither derive is used: derive
  `Debug` only.
- `#[error("network: {0}")]` with `#[from]` prints the source twice in a
  `source()` chain, against reference 2.2's advice (`reference.md:80–82`).
  Use `#[error("network")]` or `#[error(transparent)]`.
- `authority  net -> net, fs` (`04:1353`, companion 18) abbreviates 03
  §4.1's declared form (`{net: net, cache: fs.read+fs.write}`). The trust
  diff's form would read `+ cache: fs.read+fs.write`.
- `tide seal --remove crate::feed::fetch_feed --manifest` (`04:1360–1361`):
  SEAL-1 defines `--manifest` as adding the manifest entry
  (`04:1244–1245`). Say that `--remove --manifest` removes it.
- Companion 18 says editing `tide.seal` "reports the edit as `FX0921`"
  (`reference-companion.md:482–483`). Adding an entry is not reported.
  Write "a removed or changed entry".

### N7. Terminology and edge cases

- D11 says "contract closure hash" (`01:737`) where 04 says "spec-closure
  hash". Use one term.
- A trait's whole entry lists "its items' signatures and contracts and its
  laws" (`04:1258`). Say whether default method bodies are free.
- A whole entry's "attributes" include `#[doc]`, so fixing a typo in a
  sealed type's doc comment is FX0920. Exclude doc attributes, or say that
  this is intended.

### N8. Reference trim and layout

- The trimmed §13 and "Not in v0" (`reference.md:1056–1075`) lose nothing
  the core needs: 10.1 enumerates the v0 clause forms, 2.2 says JSON only,
  and companion Appendix A keeps the rest.
- `04:14–15` was not rewrapped (one line runs to about 100 columns).
- B2's fix costs core tokens at 24,996 of 25,000. Recount after paying for
  it.
