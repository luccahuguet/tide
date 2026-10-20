# Tide consistency review, round 45: confirmation of the round-44 fixes (fx-1am)

## Verdict

**request changes**

Confidence: high that all eight round-44 items are resolved or reduced to
nits; medium on the one new blocker (B1), which turns on the phrase "for each
target" that the B4 fix introduced and nothing defines. Its fix is a clause in
SEAL-1 and one in SEAL-3. The worked example is still valid, and its outputs
follow the per-configuration rules.

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `5ec5c70`
("Decide the Pure status of std types that erase a payload (fx-cze)"):
`git diff -- design/ README.md` (SHA-256 `a24753bb…0fe1cf`). Narrow scope:
04 §11.1 SEAL-1..SEAL-3 and SEAL-6, the worked example, 01 D11's
sealed-boundaries bullet, reference §10.3 and §11, and companion §18.

Read for context: bead `fx-1am`; `reviews/round44-consistency.md`; 04 §2.4,
§2.6, §11; 01 D11, D14 (content hashes, stable IDs), and D18 (admission
driver, no user `--cfg`); reference 2.3 (kept attributes and `#[cfg]` keys);
03 §4.1 (authority kinds). I reran the core count:
`scripts/count-tokens.sh design/reference.md` prints 24,987, which matches
`companion:13`.

## Round-44 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 (hash defined only for root specs) | **Resolved** | `04:1259–1261`: the spec-closure hash is "computed here for every sealed function, public or not, with or without a contract". `04:1255–1257`: std enters by toolchain version and a dependency by its locked content hash, stated as SEAL-2's own rule, and "already" is gone. N2 notes a residual over-claim about trait impls. |
| B2 (named type's path, methods, impls) | **Resolved** | `04:1265–1267`: an interface entry "does not fix a named type's path, methods, or trait impls: seal the module that defines the type". `04:1270–1271`: a whole type entry fixes "the headers of the trait impls written for it". `04:1323–1324`: SEAL-6 tells orchestrators to seal the defining modules. `companion:472` adds "the definitions of the types its signature names". N2 notes that adding an impl is left ambiguous. |
| B3 (oracle through interface-sealed helpers) | **Resolved** | The false sentence is gone. `04:1277–1280`: the closure "stops only at functions with interface entries ... a test helper with one is free: seal oracle helpers whole". `04:1273–1274`: "`--whole` on a function replaces its interface entry". |
| B4 (one hash cannot match every configuration) | **Resolved**, but the fix opens B1 | `04:1243–1245`: SEAL-1 records "for each build configuration the item existed in, its seal hash in that configuration". `04:1251–1252`: hashes are computed "per configuration, after `cfg` evaluation". `04:1290–1293`: SEAL-3 checks against "that configuration's hash". |
| N1 (core and companion looser than SEAL-2) | **Resolved** | `reference.md:964–965`: "fixed, except bodies it leaves free". A test's body is not free, so the core no longer reads as permission to edit it. Companion §18 (`companion:470–475`) now lists named types' definitions, contract callees, and a sealed test's reach. |
| N2 (test closure breadth) | **Resolved** | Frozen functions under test: `04:1278–1279`. std and dependency entry, and toolchain or lock bumps: `04:1255–1257`. "Fewer seeds is a change" (`04:1280`); a test's attributes fall under "full definition" (`04:1268`). The vacuity floor (`04:616`) and `release_checks` (`04:631`) are still not placed in `[contracts]`, but "whatever keys it holds" (`04:1282–1283`) makes the manifest entry's coverage at least well defined. |
| N3 (round-43 residuals) | **Mostly resolved** | D11's "gains nothing" now carries the merge condition (`01:740–741`). The approval-gate rejection adds "that parallel delegates still assume" (`01:748–750`), which is accurate for a delegate that does not edit the seal. The `Debug` derive on `Entry` is dropped (`04:1340`). Still open: feature sets (N3 below) and `net -> net+fs` (`04:1390`, `04:1408`, `companion:481`), which abbreviates 03 §4.1's `fs.read+fs.write`. |
| N4 (layout) | **Not resolved** | `04:17` (119 columns), `04:1294` (106), `01:740` (92) and `01:741` (108) are long. The short lines are `04:1245` ("configuration. `tide seal") and `companion:9`. |

### Worked example under the per-configuration rules

The test sits inside `#[cfg(test)] mod tests`, so it exists only in test
configurations. Its entry records only those, and the debug and release
builds do not check it (`04:1290–1293`). `fetch_feed`, `Entry`, and
`FetchError` exist in every configuration with nothing `cfg`-dependent, so
their per-configuration hashes agree. The listing "sealed 5 entries" is
correct: three public items, the named test, and the manifest. Every
`FX0920` and `FX0921` still follows from the rules:

- The test's closure stops at `fetch_feed`, which has an interface entry, so
  its body change is the added argument.
- The `--remove` run leaves `Entry` and `FetchError` unchanged, so the
  delegate's own `tide check` passes.
- The review uses the base's seal, so it shows three removals and three
  changes.

Without `#[derive(Debug)]`, nothing in the example needs `Entry: Debug`:
`matches!` and `assert!` need no `Debug`.

### Reference 10.3 trim

`reference.md:944–947` drops only "to tell weakening from strengthening",
which was part of the reason. The rule is intact: any change to a public
item's contract needs approval, except adding one where there was none. It
also agrees with 04 §11 (`04:1211–1216`). No rule was lost.

## New blocking problems

### B1. "For each target" names no target set, so a build for an unrecorded target checks nothing

- **The text.** SEAL-1 records a hash "for each build configuration the item
  existed in" (`04:1243–1245`). SEAL-3 checks "in each configuration the
  driver builds (debug, test, and release, for each target)", and "an entry
  is checked in the configurations its item existed in when sealed, against
  that configuration's hash" (`04:1289–1293`). No document defines the
  targets a package builds for: no manifest key, no `tide seal` flag. A grep
  of `design/` for "target" finds only this line and `reference.md:283`.
  D18 forbids user `--cfg` (`01:958–959`), but `target_os` and `target_arch`
  are accepted `#[cfg]` keys (`reference.md:102–104`).
- **Failure.** `tide seal` can only hash configurations it evaluates, and the
  rules do not say which ones. A natural implementation records the host.
  Take an orchestrator that seals on macOS while the CI gate builds on Linux.
  The entries have no hash for any Linux configuration, so on the text
  "checked in the configurations its item existed in when sealed" nothing is
  checked and nothing is reported. The gate then passes a widened grant or a
  weakened contract, which is the guarantee D11 and SEAL-4 state. Reading
  "existed in" as a fact about the source instead asks `tide seal` for an
  unbounded set of hashes, the same can't-implement defect as round 44's B4.
  This is the B4 fix's own text, and it does not admit the gap.
- **Fix** (two clauses). In SEAL-1: "`tide seal` records the configurations
  of the host target and of each `--target` it is given." In SEAL-3: "A build
  for a target the seal records no configuration of fails, so the CI gate
  must build a sealed target." If failing is too strict, a softer option
  admits the gap instead: "checks no entries, and `tide review` says so."

## Non-blocking notes

### N1. SEAL-4's approval record still speaks of one hash per entry

`04:1304–1307`: "Removing an entry or changing its hash ... its old and new
hashes, the new one empty for a removal." An entry now holds one hash per
configuration (`04:1243–1245`), so "its hash" is ambiguous. It could mean
the set, any one configuration's hash, or one record per configuration. The
choice decides how many approvals a toolchain bump needs. Smallest fix: "or
changing any of its hashes ... and its old and new hash sets (a digest of
them)".

### N2. Trait impls: adding one is unstated, and "does not fix" over-claims

- A whole type entry fixes "the headers of the trait impls written for it"
  (`04:1270–1271`), so a hash over that set changes when a delegate adds
  `impl Display for Entry`. Companion §18 says "You may ... add items"
  (`companion:471`), and SEAL-2 says "a delegate may add items"
  (`04:1287`). A delegate cannot tell whether a new impl on a sealed type is
  allowed. Say which: "adding or deleting one is a change" (simplest, and it
  matches a set hash), with companion §18 then reading "add items, other than
  trait impls of sealed types".
- "It does not fix a named type's ... trait impls" (`04:1265–1267`) is too
  strong when a clause calls one. An `ensures` using `==` on `Entry` fixes
  `impl PartialEq for Entry`'s body through §11 (`04:1196–1198`). Smallest
  fix: "does not, beyond the impls its clauses call, fix ...".
- Related: a trait impl method a sealed test reaches has no interface entry
  (a module seal gives them only to inherent methods, `04:1246–1247`), so it
  is frozen. SEAL-1 does not say how to name one for an interface entry
  (`<Feed as Iterator>::next`). One clause fixes this.

### N3. Feature configurations

`#[cfg(feature = "..")]` is accepted, provisionally (`reference.md:104`).
SEAL-3's configuration list (`04:1290`) has no feature axis, so an item
present only under a non-default feature is recorded with no configurations
and never checked. Since features are provisional and D18 controls `--cfg`,
one sentence that admits it is enough: "feature-gated items are checked only
in the feature set the driver builds."

### N4. Companion §18's scope of fixed types

`companion:472` names "the types its signature names". SEAL-2 also fixes the
types its clauses name (`04:1262–1263`). Write "the types its signature or
contract names".

## Acceptance against `fx-1am`

- 01 states the feature, its tier, and rejected alternatives with reasons:
  **met** (`01:82`, `01:733–754`).
- The chapter says what is sealed, who may unseal, what the checker reports,
  and how the trust diff shows it: **met except B1**. Which configurations
  are sealed and checked is undefined across targets.
- Worked example with a contract and a `Net` grant, where widening and
  weakening are rejected with a stable code: **met** (`04:1331–1414`).
- Core budget: **met**, 24,987 by rerun, under 25,000.
- No blocking contradiction between 01, the chapter, and the reference:
  **met**. B1 sits inside 04 §11.1.
- Stop condition: not hit. D11's approval model is unchanged.

## Validation

Only this file was written; nothing was staged or committed. The token
count was rerun as above. No checker exists, so every verdict is a deduction
from the text.
