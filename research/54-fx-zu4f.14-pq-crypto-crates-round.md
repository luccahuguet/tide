# 54: fx-zu4f.14, a case-study round on post-quantum crypto Rust crates

Bead fx-zu4f.14. The protocol is `research/protocols/fx-zu4f.14-protocol.md`
as amended through R3a (called "the protocol"; "§n" is its section n), and
the pool is `research/protocols/fx-zu4f.14-pool/pool.tsv`. fx-zu4f.12's
protocol is P12 and fx-zu4f.10's is P10, as the protocol cites them.

## Disclosure and privacy

The disclosure policy recorded on fx-zu4f.10 and fx-qlek.7 applies with the
bead's crypto rules (protocol, Disclosure and privacy). Nothing undisclosed
is committed. Arm-A working files (property lists, path scripts, models,
logs) were private from their first write. Crypto finds are dangerous by
default. Agents only draft, and Lucca alone sends. A private find counts
toward Tide's release candidate (RC) only once it is fixed and disclosed,
and only if it meets the caught rule.

**Public signal for arm A (R3, R3a).** Arm A has two crates, and `pool.tsv`
names both with their post-quantum call sites. From go until arm A's last
outcome is in and it has no undisclosed candidate, this round's public text
gives no arm-A outcome, total or count of private candidates; its only
arm-A line in that period is the fixed text `arm A: in progress`. By
2026-11-21 at 04:12Z both arm-A crates had their final outcome, and arm A
has no candidate, so this record gives arm-A outcomes.
The residual timing leak: the hold ends once arm A's last outcome is in and
it has no undisclosed candidate, and it can last until disclosure (a fix, or
90 days) when arm A has a dangerous one, so the date on which arm-A outcomes
appear can hint that a private candidate existed.

Possible defects that a builder notices while reading, outside the method
(reading observations, builder rules), are not method results and not
catches. Under the builder rules any such observation goes only to the
coordinator and the private directory. This record does not say whether
there were any, or how many: with two named crates, either fact would point
at one of them, as an arm-A count would.

Nothing was drafted from the method, and nothing was sent or posted.

## Out of class, and the expected yield

- **Out of class.** Constant-time and side-channel bugs (KyberSlash's
  secret-dependent division is the known example), timing, cache and power
  leaks, and fault attacks are outside the round. Tide's model sees values,
  not time, cache state, power or injected faults; no find of that kind is
  claimed, and no negative result is evidence against one.
- **Low expected yield.** The bead said so before the round, and this record
  repeats it first. Published post-quantum implementations are tested
  against the standards' known-answer tests (KATs), are often small and
  heavily reviewed, and run their arithmetic through hashing (SHA-3/SHAKE)
  from external crates, which makes a property unportable under §7 step 2.
  A documented negative was the expected outcome.

## What this round does not show

- **The pool's post-quantum limit (R3).** Both pool crates delegate to
  `libcrux-ml-kem`: each was admitted on `delegate` evidence (a call to one
  of its ML-KEM operations), and neither shows `impl` evidence (`pool.tsv`,
  step-3 column). By §7 step 2 every row whose path reaches a post-quantum
  operation is unportable, so **no ML-KEM implementation, or any other listed
  algorithm's, is tested**. Arm A reaches only each crate's own non-PQ code.
  Tide's randomness angle (does keygen, encapsulation or signing reach
  ambient OS randomness?) is answered only descriptively, by the admission
  observations below.
- **Supplied properties.** Arm A tests only properties written before any
  run from each crate's docs, comments, asserted error arms and tests (§7
  step 1). A property nobody wrote down is not tested.
- **Finite seed sampling.** A generated-input test runs 1,000 seeds (0..999).
  Passing them is evidence for those inputs, not a proof over the domain.
- **KAT checks** are finite fixed-domain oracle checks, and **fixed-input
  rows** are one case repeated under the seed labels; neither is seeded
  exploration, and they are counted apart.
- **Model gaps.** Each model records its gaps in its header: omitted tails
  past an asserted early return (which panic if reached), error enums cut to
  the variants a ported path builds, external derives dropped where their
  code does not run, and every replacement of a randomness draw by a draw
  from Tide's `Rand` capability (§7 step 4). The models stay private; Arm A
  (Model gaps) lists their gaps, including both `Rand` replacements.
- **Self-attested chronology.** Git dates and local logs are self-attested.
  The OpenTimestamps proofs show that the registered files existed by the
  attested Bitcoin times, not that no earlier private screening happened.
- **Lexical scans.** The PQ gate, the classes and the admission observations
  are lexical (§3, §5): no macro expansion, `include!`, feature or platform
  selection, or cross-module name resolution. The gate misses
  implementations reached through those, through method calls on values,
  trait-qualified calls, re-exports, glob imports or general-purpose
  libraries, and a listed constant can occur in unrelated code (§3, "What
  step 3 cannot see").

## Result

- **Arm B (retrospective): no target.** The pool holds no class C crate, so
  no history export was run (§6).
- **Arm A (forward hunt): not caught, a documented negative.** 2 crates
  targeted, screened and modeled, 0 unresolved; 498 frozen property rows,
  of which 52 were portable and tested (25 generated-input, 27 fixed-input)
  by 23 tests (15
  generated-input, 8 fixed-input, 0 KAT checks: all 17 KAT rows are
  unportable). Every test passed all 1,000 seeds; 0 failing seeds, 0
  candidates, 0 caught. None of the tested rows reaches post-quantum
  code (R3). The 52 tested rows held in the Tide models on the tested
  inputs; the 446 untested rows, the ML-KEM code, side channels and the
  real crates as built are outside this result.
- **Admission observations:** descriptive only, never a catch (section
  below).

## Registration and identities

- Base: main `27881b6a4365fd3c2af81d9716cce01dfc117969`. Snapshot
  2026-11-19T16:48:17Z, recorded once (§1).
- Registration: commit `e1c04a4` (proofs pending `edd3804`, upgraded
  `bef9bcb`). Its eight unchanged pool files (`admission-obs.py`,
  `candidates.tsv`, `clone-refs.tsv`, `crates-io-hits.tsv`, `queries.jsonl`,
  `search-hits.tsv`, `search.py`, `star-slices.json`) are part of the set the
  go certifies.
- R1: commit `6ae022a` (proof `95d074a`). R2: commit `5b9298b` (proofs
  pending `e620d56`, upgraded `5edef08`): `classify.py`, `classes.tsv`,
  `repositories.tsv`, `pool.tsv`. R3 (text only): commit `3c52459` (proof
  `94a4bba`, kept as `fx-zu4f.14-protocol.md.r3.ots`). R3a (text only):
  commit `6d44c06` (proof pending `96e9811`, upgraded `8ad0e7a`, Bitcoin
  blocks 976760 and 976761).
- Go: the bead's GO comment at 2026-11-20T20:42:16Z; the run builder took it
  at 20:42:29Z (`date -u`), the start of per-crate examination. Nothing is
  backdated.
- Build revision: main `6c8ed2a`, merged into the branch as `35d1aa3` with
  no rebase; the build tree was checked identical to `6c8ed2a`
  (`git diff --stat 6c8ed2a HEAD -- crates Cargo.toml Cargo.lock
  rust-toolchain.toml` empty). tide-admit and tide_std were built through
  heavy.sh, and every model run used them.

## Pool, re-registration history and selection counts

| Rank | Class | Repository | Stars | Crate | Non-test lines | Arm | Outcome |
| ---: | --- | --- | ---: | --- | ---: | --- | --- |
| 1 | P | Helvesec/rmux | 2,698 | rmux-web-crypto | 578 | A | not caught: 11 rows tested, all pass; first failure of the other 84 rows: 72 unportable (external crate on the path), 12 not assertable |
| 2 | P | hudikhq/hoodik | 1,559 | cryptfns | 2,882 | A | not caught: 41 rows tested, all pass; first failure of the other 362 rows: 333 unportable (external crate on the path), 2 over budget, 27 not assertable |

**Re-registration history, all before any run.** The registration's pool
held **12** crates (`e1c04a4`). R1 cut it to **3** (`6ae022a`) after the
Codex supervisor pass found that the PQ gate admitted a package on a lexical
token alone. R2 cut it to **2** (`5b9298b`) after the Codex re-check found
that names and imports still passed as evidence. Each cut was a mechanical
narrowing by `classify.py`, checked row by row with `--check-narrowing`; no
row was dropped or added by hand, and the gate was not loosened to refill
the pool. R3 and R3a were text amendments after the Opus review of R2 and
its re-check: the public-signal rule, and the statement of what the pool can
test.

Counts, units kept apart (§1-§4; the TSVs in the pool directory list every
row with its first reason):

| Stage | Count |
| --- | ---: |
| Distinct repository names from search (23 GitHub, 895 crates.io, 6 both) | 912 |
| Repositories kept by the filter (886 dropped: 722 under 1,000 stars, 105 not found, 57 not Rust, 2 duplicate ids) | 26 |
| Repositories kept by the PQ gate (20 dropped: 1 no Cargo package, 9 no step-1 evidence, 7 no step-2 evidence, 3 no step-3 evidence) | 6 |
| Package rows in `classes.tsv` | 751 packages + 1 placeholder row |
| Packages past gate step 1 / step 2 / step 3 | 38 / 14 / 6 |
| Step-3 evidence kind (`impl` / `delegate`) | 1 / 5 |
| Class C / class P | 0 / 2 |
| Pool (arm A / arm B) | 2 (2 / 0) |

Excluded rows by first reason (`classes.tsv`): 700 no PQ evidence; 24 no PQ
implementation evidence; 8 no PQ code evidence; 7 no production source; 4
not a workspace member; 3 forbidden source or forbidden normal dependency; 2
unscannable; 1 no std concurrency use and std path; 1 placeholder (no Cargo
package).

## Arm B: retrospective fixes

No class C pool crate, so arm B had no target: 0 matched commits, 0
candidate rows, 0 unique commits, 0 duplicates, no E1-E5 judgement.

## Arm A: forward hunt at the pinned commits

Both class P crates were read in full from read-only `git archive` trees of
the pinned commits (the crate directory, the workspace manifests and the
root policy documents; §7 allows the full tree, and this reads less). For
each crate: the property list was written in source order and frozen
(hashed, made read-only) before any port, build or run; a path script then
listed each row's reached non-test items and decided portability and the
400-line union mechanically, in list order, before any model; the model
ported only that union into Tide v0; and every test ran as
`#[sim_test(seeds = 1000)]` through `tide-admit --tide-run-test`, through
heavy.sh. Every log was read in full. No compile or port correction was
needed, and no row was added after a run.

| | rmux-web-crypto (rank 1) | cryptfns (rank 2) |
| --- | ---: | ---: |
| Property rows (generated / KAT / fixed / not assertable) | 95 (33 / 5 / 45 / 12) | 403 (204 / 12 / 160 / 27) |
| Unportable (external crate on the path) | 72 | 333 |
| Over budget | 0 | 2 |
| Not assertable | 12 | 27 |
| Portable and tested (generated / fixed) | 11 (3 / 8) | 41 (22 / 19) |
| Union of the tested paths, nonblank non-comment lines | 99 | 400 |
| Tests (generated-input / fixed-input / KAT check) | 7 (2 / 5 / 0) | 16 (13 / 3 / 0) |
| Seeds per test, all passed | 1,000 | 1,000 |
| Failing seeds, candidates, caught | 0, 0, 0 | 0, 0, 0 |

KAT rows: the crates' own embedded vectors (N = 1 each); every one reaches an
external hash, cipher, encoding or key crate, so none was ported and the
round ran no KAT check.

What the tested rows cover, by crate:

- **rmux-web-crypto:** the record layer's own framing and sequence checks
  that return before any cipher call: malformed and truncated frames,
  out-of-order sequence numbers, and fail-closed behavior at the sequence
  limit for sealing and opening.
- **cryptfns:** the early error arms before any cipher, hash, encoding or
  signature call (short AEAD ciphertexts; out-of-range and short chunks;
  unknown cipher and key-type identifiers; empty search keys and empty PEM
  bodies; bad user-id lengths and key types in the key-transition
  certificate), key/nonce splitting and padding, the chunk-format tag test,
  the search-digest shape predicates, the tag-entry and token string
  parsers, the tokenizer, and the crate's fixed constants and wire values.

Model gaps (listed in each private model's header): omitted tails past each
asserted early return, which panic if reached (no test reached one);
error enums cut to the variants a ported path builds; external derives
dropped where their code does not run (rank 2's `der` `Enumerated`); types
named only in signatures kept as stand-ins and not counted in the union. Rank
1 draws no randomness on its tested paths and has no `Rand` replacement.
Rank 2 has two (§7 step 4): `Cipher::encrypt_chunk`'s ambient
`getrandom` nonce draw became one `Rand` fill of the same length, through a
model-side counter, so its index-error test asserts that the error arm draws
nothing; and the tokenizer's `HashMap::new()`, whose hasher key Rust draws
from the OS, became `HashMap::with_hasher(RandomState::from_rand(rand))`.
The tokenizer then sorts its output by a total order, so the key
cannot change the result.

Chronology (UTC, self-attested): rank 1's list was frozen at
2026-11-20T20:50:13Z, its paths decided at 20:51:05Z and its model at
20:52:46Z, and it ran from 21:27:37Z to 23:23:15Z. Rank 2's list was frozen
at 23:34:00Z, its paths decided at 23:41:10Z and its model at 23:46:04Z, and
it ran from 2026-11-21T00:00:02Z to 04:12:08Z. Wall time was mostly heavy.sh
holds; each test ran in about 3 to 32 seconds.

## Admission observations (descriptive)

`admission-obs.py --self-test` passed; the real run (`--after-go`,
2026-11-20T20:46:41Z to 20:46:42Z, exit 0) covered 7 crates: the 2 pool
crates and the 5 class-excluded binary packages in pool repositories. Output
`admission-obs.tsv`, SHA-256
`8b7489eedcb810286a9e7bc1b08af9ea8693c4ed6db9da7ff97463ff05a2ffc7`, kept
under the run's target directory. These are lexical counts, not defects, and
never a catch.

| Set | Kind | Observations | Key-flagged |
| --- | --- | ---: | ---: |
| pool | randomness: `getrandom` (1 import, 15 references) | 16 | 11 |
| pool | randomness: `OsRng` (1 import, 9 references) | 10 | 8 |
| pool | randomness: `thread_rng` (references) | 5 | 5 |
| excluded binaries | randomness: `/dev/urandom` literal | 1 | 0 |
| excluded binaries | ambient `std::env` (11 imports, 130 references) | 141 | 9 |
| excluded binaries | ambient `std::fs` (8 imports, 66 references) | 74 | 1 |
| excluded binaries | ambient `std::process` (9 imports, 75 references) | 84 | 2 |
| excluded binaries | spawns, unresolved | 3 | 0 |
| **all** | | **334** | **36** |

Randomness, 32 observations in all (31 in the pool set). Operation labels,
taken from the innermost enclosing fn or the callee whose argument list
holds the observation: 5 carry a label, 4 `keygen` (function names
`generate_key`) and 1 `sign`; none carries `encaps`. The other 27 carry
none. That includes draws inside functions whose names hold no operation
word but that build keys, nonces or protocol messages, so these labels
undercount where randomness reaches key material. The key flag marks 24 of
the 31 pool-set randomness observations. No call graph is followed (§5).

## Security programs and AI-report policies

Read after go and before any draft, for both pool repositories, from
read-only `git archive` trees of the pinned commits (each document read in
full) and by read-only GitHub GETs (2026-11-20T20:45:29Z to 20:46:12Z, at
least 2 s apart, logged with SHA-256). Off-GitHub pages were named, not fetched.
Absence claims cover only these documents.

| Repository | Security channel (source) | PVR | Program | AI-generated reports |
| --- | --- | --- | --- | --- |
| Helvesec/rmux | GitHub private vulnerability reporting, or a security email (SECURITY.md) | enabled | none stated; coordinated disclosure, keep private until a fix | no policy found |
| hudikhq/hoodik | security email (SECURITY.md) | disabled | none stated; confidential until an agreed date | no policy found |

Sources, read 2026-11-20 between 20:44Z and 20:46Z:
`https://github.com/Helvesec/rmux/blob/1f4571e74f36be0c033c6294d1616c7d3a6fbda1/SECURITY.md`,
`https://github.com/Helvesec/rmux/blob/1f4571e74f36be0c033c6294d1616c7d3a6fbda1/crates/rmux-web-crypto/SECURITY.md`,
`https://github.com/Helvesec/rmux/blob/1f4571e74f36be0c033c6294d1616c7d3a6fbda1/CONTRIBUTING.md` and
`https://github.com/Helvesec/rmux/blob/1f4571e74f36be0c033c6294d1616c7d3a6fbda1/GOVERNANCE.md` (no issue templates; the owner has no
`.github` repository);
`https://github.com/hudikhq/hoodik/blob/4dcfe21135d08b041e41cde116f7ed2a4855e9fd/SECURITY.md`,
`https://github.com/hudikhq/hoodik/blob/4dcfe21135d08b041e41cde116f7ed2a4855e9fd/CLA.md` and
`https://github.com/hudikhq/hoodik/blob/4dcfe21135d08b041e41cde116f7ed2a4855e9fd/DEVELOPMENT.md` (no CONTRIBUTING or
issue templates; the owner's `.github` repository holds only a profile
README).

## Deviations

- **Builder handoffs.** Three successive Claude Opus run builders did the
  after-go work. The first stopped at its account's five-hour limit after
  rank 1 at about 23:24Z. The second froze rank 2's property list and
  handed off at its context budget (AP-SESSION-001). The third did rank 2's
  paths, model and runs and this record. Each successor started from the
  written notes and repeated no step.
- **Read-window slip.** The header of rank 1's private property list gives
  its read window as ending at 20:58Z. Reading ended before the list's
  freeze at 20:50:13Z. The frozen file was left unchanged, and the slip was
  logged next to it.
- **Stray cache.** `admission-obs.py` imports `classify.py`, so Python wrote
  a `__pycache__/` into the pool directory. It was deleted at 21:14Z and
  never staged.
- **Reading observations.** This record gives neither their number nor
  whether there were any (Disclosure above). The protocol does not ask for one; fx-zu4f.12's record gave one.
- **Counting rule applied (rank 2).** Two items named only in signatures and
  not executed on any tested path (a key-length constant in a return type,
  and the certificate's result type) were not counted in the 400-line
  union, by the frozen rule "consts and types a path needs executing".
  Counting them would have made the union 401 and moved one fixed-input
  row to over budget.

## Framing

The record states the selection and what the method did and saw. It does
not judge any project or its authors, calls no crate buggy or unsafe from a
scan, a failed port or a model, and gives no discovery rate. Scan classes,
PQ-evidence labels and admission observations are mechanical labels, not
quality claims. Nothing was posted upstream.
