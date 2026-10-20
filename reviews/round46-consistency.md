# Tide consistency review, round 46: confirmation of the round-45 fixes (fx-1am)

## Verdict

**approve**

No blocking problems. Round 45's blocker B1 is closed: the seal now records
its targets, hashes per configuration of each, and fails a build for a
target it does not list. N1, N3, and N4 are resolved. N2 is resolved in 04
but not carried into companion §18, which is a guidance gap and not an
escape, because the checker still rejects the change. Three non-blocking
notes follow, one of them on the new feature-cfg sentence. The worked
example is still valid.

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `5ec5c70`
("Decide the Pure status of std types that erase a payload (fx-cze)"):
`git diff -- design/ README.md` (SHA-256 `8c0bd1ee…8c4321`). This is a
narrow confirmation round. Scope, as asked:

- the status of each round-45 item (B1, N1–N4, leftover long lines);
- whether the edits contradict the rest of 04 §11.1, 04 §11, 01 D11 and
  D18, the reference §11 pointer, companion §18, or the worked example, and
  in particular whether "the driver never sets" a `feature` cfg agrees with
  D18 and 03 ADMIT-2;
- whether the reflow of SEAL-1..6, D11's sealed-boundaries bullet, and
  companion §18 changed any wording beyond the intended edits.

Nothing else was re-audited. The core count is unchanged:
`scripts/count-tokens.sh design/reference.md` prints 24,987, matching
`companion:13`.

## Round-45 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 (targets undefined; an unrecorded target checks nothing) | **Resolved** | `04:1242–1245`: `tide.seal` "lists the targets it was sealed for", and each entry has "a seal hash for each configuration (debug, test, and release, per sealed target) the item exists in". `04:1249–1250`: "`--target <triple>` seals for a target besides the host", so the host is always sealed. `04:1295–1299`: an entry is checked in the configurations it has a hash for, and "so is a build for a target the seal does not list, until the orchestrator seals for it" `FX0920`. D18 bars user `--cfg` (`01:957–958`), so a configuration is fixed by profile and target, and the set is now finite and recorded. The macOS-seal, Linux-CI case from round 45 now fails the gate (the gate checks against the base's seal, `04:1307–1308`). |
| N1 (approval record speaks of one hash) | **Resolved** | `04:1309–1312`: "changing any of its hashes ... its old and new hash sets, the new one empty for a removal". This agrees with §11's record (`04:1223–1228`), whose "new hash matches exactly" reads as the new set. |
| N2 (adding a trait impl; "does not fix" over-claims; naming impl methods) | **Resolved in 04, open in the companion** | Adding: `04:1274–1275` ("the set of trait impl headers written for it, so adding or deleting an `impl PartialEq` is a change") and `04:1290–1292` ("a delegate may add items, except trait impls of a type sealed whole"). Over-claim: `04:1269–1271` ("beyond the bodies its clauses call"). Naming: `04:1247–1248` (`<Entry as PartialEq>::eq`). Companion §18 still says "and add items" with no exception (`companion:471`); see N2 below. |
| N3 (feature configurations) | **Resolved**, with a wording note | `04:1252–1254`: "An item that exists in no sealed configuration, such as one under a `feature` cfg, which the driver never sets (D18), cannot be sealed." The gap is now admitted. N1 below covers the citation. |
| N4 (companion §18's scope of fixed types) | **Resolved** | `companion:472`: "the types its signature or contract names". |
| Leftover long and short lines | **Mostly resolved** | D11's bullet (`01:733–753`), SEAL-1..6 (`04:1242–1334`), and companion §18 (`companion:467–490`) all fit 79 columns, and the old `04:1294` is gone. Still off: `04:18` (119 columns, the status paragraph the §11.1 mention reflowed), `companion:9` (38, "to the core. Sections 14–16 tabulate"), and `04:1215` (38, "can strengthen a contract. A weakened,"). Layout only. |

### Feature cfg against D18 and ADMIT-2

D18 says the driver "adds no user-controlled `--extern`, `-L`, `--sysroot`,
`--cfg`, or semantic `-C` flags" (`01:957–958`), and ADMIT-2 says the same
(`03:414–416`). No document defines a manifest key or flag that enables a
feature: a grep of `design/`, `research/`, and `reviews/` for
`[features]`, `features =`, and `cfg(feature` finds only round 43's and
44's notes. So in v0 nothing sets `feature = ".."`, and the sentence is
true. No contradiction: reference 2.3 accepts the key "provisional beyond
`test`" (`reference.md:103–105`), and accepting a key that no build sets is
consistent. But D18 does not say it. D18 bars user-controlled flags, while
the driver itself sets `test`, `debug_assertions`, and the target cfgs, and
a later manifest `[features]` table would be a cfg the driver sets. The
fact comes from the absence of any feature mechanism, not from D18. See N1.

### Worked example

Unchanged by these edits, and it still follows the rules. Sealing on the
host records the host's configurations. The test exists only in test
configurations (`04:1295–1296`). `Entry` has no derives or trait impls, and
`FetchError`'s derives are covered as derives (`04:1274`). "sealed 5
entries" (`04:1384`) and every `FX0920` and `FX0921` in the two listings
(`04:1393–1415`) follow as round 45 traced.

### Reflow

The round-45 working tree was not available (no snapshot outside the
author's later edits). I compared two ways instead:

- A word diff of §11.1 against the session's earlier snapshot (the
  round-44 text). Every changed word maps to a round-44 fix that round 45
  recorded, or to a round-45 fix listed in the scope. That includes
  "for each configuration ... per sealed target", the `--target` and
  feature sentences, "it has a hash for", the unlisted-target clause, "any
  of its hashes", "hash sets", "except trait impls of a type sealed whole",
  and the qualified-path clause.
- Round 45's quoted phrases from 04, 01, the reference, and the companion.
  Each one that was not deliberately changed is still present verbatim,
  modulo line breaks. That includes "computed here for every sealed
  function, public or not, with or without a contract", "a test helper with
  one is free: seal oracle helpers whole", "whatever keys it holds", "gains
  nothing once its work merges through them", "that parallel delegates
  still assume", and "fixed, except bodies it leaves free".

The `04:14–18` status paragraph differs from HEAD only by "and sealed
delegation boundaries (§11.1)". I found no unintended wording change. The
only addition not named in the scope is "for that" after "seal the module
that defines the type" (`04:1271`), and it changes no meaning.

## New blocking problems

None.

## Non-blocking notes

### N1. "which the driver never sets (D18)" cites D18 for something it does not state

- **Where.** `04:1252–1253`.
- **Risk.** D18 and ADMIT-2 bar *user-controlled* `--cfg` (`01:957–958`,
  `03:414–416`). The driver does set cfgs (`test`, `debug_assertions`,
  target keys). The sentence is true only because no feature mechanism
  exists. If one is added later, such as a manifest `[features]` table read
  by the driver, the sentence turns false while its citation still looks
  authoritative, and SEAL-3 would again leave feature configurations
  unchecked.
- **Smallest fix.** "such as one under a `feature` cfg, which no v0 build
  sets (no manifest key enables a feature, and D18 bars user `--cfg`)".

### N2. Companion §18 still grants "add items" without the trait-impl exception

- **Where.** `companion:471` ("and add items"), against `04:1290–1292`
  ("except trait impls of a type sealed whole") and `04:1274–1275`.
- **Risk.** A delegate that reads only the companion adds
  `impl Display for Entry` and gets an `FX0920` the guidance said was
  allowed. The checker still rejects it, so there is no escape, but an
  agent that trusts the companion will hit the rejection.
- **Smallest fix.** "and add items, other than trait impls of sealed types".

### N3. Sealing for a new target: mechanics, gating, and dependencies' seals

- **Where.** `04:1249–1250` and `04:1298–1299` ("until the orchestrator
  seals for it"), against `04:1309–1313`.
- **Risk.** Three points, none of them an escape:
  - **How.** The text does not say that `tide seal --target X` with no
    paths re-hashes the existing entries. Every entry must carry hashes for
    every listed target (`04:1244–1245`), so the tool has to.
  - **Gating.** Adding a target adds hashes to every entry. On a protected
    base, if that counts as "changing any of its hashes", it needs one
    `unseal` approval per entry for a change that only adds checks.
  - **Dependencies.** "Each crate against its own `tide.seal`"
    (`04:1293–1294`) includes a dependency that ships one. A consumer
    building for another target then gets an `FX0920` it cannot fix, while
    the lock's content hash already fixes everything in that dependency.
- **Smallest fix.** One sentence in SEAL-1 or SEAL-3 covering all three:
  "`--target` re-hashes every entry for that target; adding a target's
  hashes is not gated; a dependency's `tide.seal` is not checked, since its
  locked content hash fixes it."

## Validation

Only this file was written; nothing was staged or committed. The diff hash
and the core token count were rerun as above. No checker exists, so every
verdict is a deduction from the text.
