# Tide consistency review, round 44: confirmation of the round-43 fixes (fx-1am)

## Verdict

**request changes**

Confidence: high on B2 and B3, which are textual; medium-high on B1, which
turns on whether §11's hash exists for a function that is not a root spec;
medium on B4, which turns on when seal hashes are computed. Every fix is a
sentence or a clause. None needs D11's approval model changed, and the
worked example is now valid and its outputs follow the rules.

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `5ec5c70`
("Decide the Pure status of std types that erase a payload (fx-cze)"):
`git diff -- design/ README.md` (SHA-256 `1a968026…8be8733`), which changes
04 §11.1 (SEAL-1..SEAL-6 and the worked example), §11's "As D11 lists"
sentence and v0 `weakened` label, and the tier paragraph; 01's tier table
and D11 (approval list and the sealed-boundaries bullet); reference §11's
pointer, header, and §13 trim; companion header and §18; README's v0 list.

Read for context: bead `fx-1am` and its comment; `reviews/round43-consistency.md`;
01 tier table, D11, D12, D14, D17, D18; 03 §2.5, §4.1, §4.3; 04 §2.1,
§2.3 (budgets), §2.4 (vacuity floor), §2.6, §11; reference 1, 2.2, 2.3,
6.2, 6.7, 6.8, 10.1, 11, 12.3, 12.4; companion 15.2. The core count was
rerun: `scripts/count-tokens.sh design/reference.md` prints 24,993, as the
companion records.

## Round-43 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 (sealed `#[cfg(test)]` test fails the normal build) | **Resolved**, with a new defect in the same rule (B4) | Entries record "the build configurations the item existed in" (`04:1243–1244`) and are checked only there (`04:1281–1285`); a sealed function made `#[cfg(test)]`-only is still "missing from one of those configurations". |
| B2 (core forbids the delegated work) | **Resolved**, wording point in N1 | `reference.md:966–967`: "Sealed items ... may change only in a function body". |
| B3 (signature fixes names, not definitions) | **Partly resolved** | (3) Public methods: closed, "methods of its inherent impls included" (`04:1245–1246`). (2) Dependency types: closed by SEAL-2's own rule, "a dependency's by its locked content hash" (`04:1258`), though §11 does not say it (B1). (1) Named types: the author's choice, relying on §11's spec-closure hash, closes the escape for a public function with a contract. §11 hashes "the signature (types, ...)" and "type definitions", with "Names resolve to the content hashes of D14" (`04:1194–1201`), and a type's content hash is its definition, so the reading holds transitively there. It does not close it for a function with no contract, which is round 43's own example (B1 below), and it fixes a named type's content but not its path, methods, or trait impls (B2 below). |
| B4 (oracle through unsealed code) | **Partly resolved** | The author's choice (a test covers everything it reaches, stopping at interface entries, `04:1266–1271`) closes the manual `impl PartialEq` behind `assert_eq!`, a non-test helper with no entry, and `mod tests` shadowing (names resolve to stable IDs, `04:1250–1252`). It leaves open an oracle helper that has an interface entry, which a module seal gives every public function, including round 43's `pub fn valid` (B3 below). |
| B5 (budget outside the manifest entry) | **Resolved** | `04:1272–1274` and `01:738`: the whole `[contracts]` table. The vacuity floor (`04:616`) and `release_checks` (`04:631`) are covered only if they live in that table, which 04 does not say (N2). |
| B6 (unimplementable interface) | **Resolved** | `deadline: Instant` (`04:1347`) matches `Net::get(&Url, deadline: Instant)` (`reference.md:331`); the test takes it from `sim.clock.now()` (`04:1361`; `SimRoot` has `Root`'s fields, `reference.md:513–515`). |
| B7 (false "`tide check` passes") | **Resolved** | The delegate now "passes one in the test" (`04:1373–1374`), so after `--remove` on all three entries the test configuration type-checks and "its own `tide check` passes" (`04:1390–1391`) is true; `tide check` is advisory on approvals (`04:1291–1292`). |
| B8 (D11 rejects what SEAL-4 adopts) | **Resolved**, residual in N3 | `01:747–749` now rejects "changes to sealed items gated by approval rather than rejected". |
| N1 (unprotected branch) | **Resolved in 04**, residual in N3 | `04:1304–1306`. `01:741–742` still says a seal edit "gains nothing" without the caveat. |
| N2 (configurations) | **Partly resolved** | `04:1280–1281` names debug, test, and release. Feature sets and target `cfg`s are not mentioned (N2). |
| N3 (approval record) | **Resolved** (the optional reviewer note was not taken) | Fields: `04:1295–1298`; one record per gated change: `04:1298–1299`; §11 list: `04:1220–1221`. |
| N4 (`cfg!(test)`) | **Resolved** | `04:1314–1315`. |
| N5 (two test mechanisms) | **Resolved** | `04:1317–1319`; 01 now leads with the manifest and merge reasons (`01:745–746`). |
| N6 (example nits) | **Mostly resolved**, residual in N3 | `#[error("network")]` (`04:1338`); `--remove` defined (`04:1247–1248`); companion says "a removed or changed entry" (`companion:483–485`). `Entry` still derives `Debug`, which needs `Url: Debug`, unstated in `reference.md:507–509`; `net -> net+fs` is unchanged (`04:1381`). |
| N7 (terms and edge cases) | **Resolved** | `01:737` "spec-closure hash"; default bodies sealed (`04:1264–1265`); doc comments not hashed (`04:1252–1253`). |
| N8 (trim and layout) | **Resolved** except layout | §13 trim loses nothing the core needs; count rerun (24,993). `04:14–16` is still unwrapped, now longer (N4). |

### Worked example

Valid under the reference: `Root` is in the prelude (`reference.md:993`)
and `Root::sim` is test-only (`reference.md:377`); `sim.net.drop_all() -> Net`
(`reference.md:527`) borrowed as a temporary lives to the end of the
statement; `Instant + Duration` and `Url::parse(..).unwrap()` are std API;
`matches!` is how `NetError` is tested (`reference.md:506`); `Error` with
`#[from]` on a single-field variant and a message without the source follows
2.2 (`reference.md:77–82`); the `ensures` matches on `&result`
(reference 10.1, `reference.md:836–838`); `grants = ["unsafe"]` is the package form
(`reference.md:1027`); `use super::*` brings the parent's private imports
into `tests`.

Outputs follow the rules: `tide seal crate::feed ...` seals the three public
items, the named private test, and the manifest, so 5 entries (`04:1370`);
removing the `ensures` is labelled `weakened` (`04:1213–1215`); the second
listing is three `FX0921` (removed entries) and three `FX0920` (items against
the base's seal), as SEAL-4 requires. The only nits are in N3.

## Blocking findings

### B1. An interface entry relies on a hash §11 defines only for root specs

- **The text.** An interface entry fixes "contract use (§2.1), and
  contract's spec-closure hash (§11). That closure already covers,
  transitively, the definitions of the types ... the signature and clauses
  name, a dependency's by its locked content hash" (`04:1255–1259`). §11
  defines the hash per root spec: "Each root spec has a *spec-closure
  hash*" (`04:1194`), and root specs are "the contracts of public items"
  among others (`04:1189–1191`). §11 says nothing about dependencies or
  lock hashes; it names D14 content hashes and, for std, "the standard
  library's spec catalog version" (`04:1199–1201`).
- **Escape.** A sealed function with no contract, or a private one sealed by
  explicit path (SEAL-1 allows it, `04:1244–1245`), has no root spec and so,
  on §11's text, no spec-closure hash. Its entry then fixes the signature's
  names but not the named types' definitions or the declared authority
  derived from them (`04:1259–1260`). Round 43's B3 example,
  `pub fn store(db: &Db, rec: &Record)` with no contract, is that case: a
  delegate changes `Record.link: Url` to `String` with no `FX0920`. The
  claim "That closure already covers ... a dependency's by its locked
  content hash" is SEAL-2's rule stated as §11's.
- **Fix** (04 SEAL-2, one sentence). "An interface entry's hash is
  computed by §11's rule for every sealed function, with or without a
  contract and whatever its visibility; dependency items enter by their
  locked content hash, and std's by the toolchain pin." Then drop
  "already".

### B2. A named type is fixed in content, not in path, methods, or trait impls

- **The text.** Names resolve to stable IDs, "and a rename does not"
  change the hash (`04:1250–1252`). A type's whole entry fixes "fields,
  variants, generics, bounds, attributes, and derives" (`04:1263–1264`).
  A module seal covers public items and inherent methods (`04:1245–1246`).
  SEAL-6 admits only routing, `cfg!(test)`, and new trust holes
  (`04:1311–1319`).
- **Escapes the text does not admit.**
  1. `fetch_feed`-style signature names `crate::model::Record`, whose
     module is not sealed. The delegate renames it to `Rec`, moves it to
     another module, or renames `Record::new`, or changes the signature of
     a public method on it. The stable ID and content are unchanged, so the
     interface hash is unchanged; parallel delegates written against
     `crate::model::Record` or `Record::new` break at merge, the collision
     research/13 need 3 asks seals to prevent.
  2. Even in a sealed module, a hand-written trait impl on a sealed type
     (`impl Display for Entry`, `impl Iterator for Feed`, `impl From<X> for
     FetchError`) is not a public item, an inherent method, or a derive.
     The delegate deletes it, or narrows its bounds, with no `FX0920`.
- **Fix** (two clauses). In SEAL-2, a type's whole entry also fixes the
  headers (trait, generics, bounds) of its crate-local trait impls. Keep
  their method bodies out of the type's entry, so the test closure still
  covers them (`04:1266–1269`). In SEAL-6: "A sealed signature fixes the
  definitions of the types it names, not their paths or impls; seal the
  modules that define them." Companion 18 then adds "a type a sealed
  signature names" to what may not change.

### B3. "A delegate can weaken neither the test nor its oracle" is false for interface-sealed helpers

- **The text.** A test's entry covers everything the test reaches "and stops
  only at functions with interface entries, whose bodies are the
  delegate's. So a delegate can weaken neither the test nor its oracle"
  (`04:1266–1271`). A module seal gives every public function and inherent
  method an interface entry (`04:1245–1246`, `04:1254`).
- **Escape.** The orchestrator seals `crate::feed` and a test that asserts
  `assert!(entries.iter().all(valid))`, with `pub fn valid(e: &Entry) ->
  bool` in `crate::feed`. That is round 43's B4 example. `valid` has an
  interface entry, so the test's hash stops there, and the delegate rewrites
  `valid` to return `true`. The same holds for a manual `impl PartialEq`
  whose `eq` calls an inherent method of a sealed-module type. The closure
  cannot tell an oracle helper from the code under test, which is a sound
  reason to stop, but the text claims more.
- **Fix** (04 SEAL-2 and SEAL-6, a clause each). Replace "So a delegate can
  weaken neither the test nor its oracle" with "So a delegate cannot weaken
  the test, or any part of its oracle without an interface entry". In SEAL-6:
  "A helper a sealed test uses as its oracle is fixed only if it has no
  interface entry or is sealed whole; `--whole` on an item replaces its
  interface entry." The last clause also says what `--whole` does to an
  item a module seal already covered, which SEAL-1 leaves open.

### B4. One seal hash per entry cannot match every configuration

- **The text.** An entry records "the build configurations the item
  existed in, and its seal hash" (`04:1243–1244`), one hash. SEAL-3 checks
  it in each recorded configuration, and a differing hash in any is
  `FX0920` (`04:1281–1285`). Seal hashes "resolve names to stable IDs"
  (`04:1250`), and names resolve only after `cfg` evaluation, so the hash is
  per configuration.
- **Failure.** An item whose definition or closure varies by configuration
  has different hashes in debug and release: a sealed whole `main` (as
  SEAL-6 recommends, `04:1313`) with `if cfg!(debug_assertions)`, a sealed
  type with a `#[cfg(test)]` or `#[cfg(debug_assertions)]` field, or a
  contract whose clause calls a function with such a body. Whichever single
  hash `tide seal` records, another configuration fails `FX0920` on every
  build. It is the same class of defect as round 43's B1: the rule cannot
  be implemented as written.
- **Fix** (04 SEAL-1, a few words). "the build configurations the item
  existed in, and its seal hash in each". SEAL-3 then reads "whose seal
  hash in a configuration differs".

## Non-blocking findings

### N1. The core and companion texts are looser than SEAL-2

- `reference.md:966–967`: "Sealed items ... may change only in a function
  body". Strictly true, but a delegate reading only the core will take it as
  permission to edit a sealed test's body, which is the exact change the
  worked example rejects (`04:1383–1384`). Sealed `main` whole is the same.
  Suggested: "may change only in the body of a function sealed by
  interface, never a test's". That costs about 8 tokens at 24,993, so trim
  as many elsewhere.
- Companion 18 (`companion:470–473`) omits the named types' definitions
  (and, after B2, their impls) from what may not change.

### N2. The test closure's breadth is unstated where it bites

- "Everything the test reaches" (`04:1266`) freezes the body of every
  function the test calls that has no interface entry. Examples are a
  private function under test, which a module seal does not cover, and a
  function in an unsealed module. Its callees are frozen too. That is
  correct, but an orchestrator only learns it from `FX0920`. SEAL-6 or the
  companion should say: seal by interface every function under test whose
  body you delegate. A delegate's private helpers *below* an interface-sealed
  function are free, as intended; the worked example is fine.
- It also reaches std and dependency bodies. Say they enter by the toolchain
  pin and locked content hash (B1's fix), and that a pin or lock bump
  therefore changes those entries. On a protected branch each change is an
  `FX0921` that needs its own record (`04:1298`), which is costly but
  visible.
- `#[ignore]` (`04:1271`) is not among the kept attributes
  (`reference.md:101–103`). Write "an added attribute or fewer seeds".
- Whether the vacuity floor and `release_checks` sit in `[contracts]`
  (`04:616`, `04:631`) is unsaid, so B5's coverage of them is unsaid.

### N3. Residuals of round 43

- D11's rejection reason ("lets a delegate's loop run on against a changed
  interface", `01:747–749`) is partly true of the adopted design too, since
  `tide check` is advisory (`04:1291–1292`). What differs is that the
  delegate must make a deliberate seal edit, which review reports. Round 43's
  B8 wording said so.
- `01:741–742` "gains nothing" lacks SEAL-4's unprotected-branch caveat
  (`04:1304–1306`).
- SEAL-3 names debug, test, and release only. A sealed item under
  `#[cfg(feature = "..")]` or a foreign `target_os` exists in none of
  them, so it is recorded with no configurations and never checked. Say so,
  or name feature sets.
- `Entry` derives `Debug` (`04:1330`), and nothing states `Url: Debug`
  (`reference.md:507–509`). Nothing needs it, so drop the derive.
- `authority net -> net+fs` (`04:1381`, `companion:479`) abbreviates 03
  §4.1's kinds (`fs.read+fs.write`); 03 §4.3 prints full kinds
  (`03:622`).

### N4. Layout

`04:16` (about 125 columns, from the tier-paragraph edit), `companion:7`
(about 105), and the short line at `companion:485` are not rewrapped.

## Acceptance against `fx-1am`

- 01 states the feature, its tier, and rejected alternatives with reasons:
  **met** (`01:82`, `01:733–753`).
- The chapter says what is sealed, who may unseal, what the checker
  reports, and how the trust diff shows it: **met in structure**, but "what
  is sealed" is still incomplete (B1, B2) and partly misstated (B3, B4).
- Worked example with a contract and a `Net` grant, the widening and
  weakening rejected with a stable code: **met** (`FX0920`, `04:1378–1388`).
- Core budget: **met**, 24,993 by rerun.
- Verification (no blocking contradiction between 01, the chapter, and the
  reference): **met for 01 and the reference**; the blockers are within
  04 §11.1 and between §11.1 and §11 (B1).
- Stop condition: not hit. SEAL-4 adds a change class, not a key or
  approver class (`04:1300–1304`).

## Validation

Only this file was written; nothing was staged or committed. The token
count was rerun as above. No checker exists, so every verdict is a
deduction from the text.
