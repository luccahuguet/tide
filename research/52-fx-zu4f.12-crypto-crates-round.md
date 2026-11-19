# 52: fx-zu4f.12, a case-study round on crypto and blockchain Rust crates

Bead fx-zu4f.12. The protocol is `research/protocols/fx-zu4f.12-protocol.md`
(called "the protocol"; "§n" is its section n), and the pool is
`research/protocols/fx-zu4f.12-pool/pool.tsv`. Judging rules come from
fx-zu4f.10's protocol (called P10), as the protocol cites it.

## Disclosure and privacy

The disclosure policy recorded on fx-zu4f.10 and fx-qlek.7 applies with the
bead's crypto additions (protocol, Disclosure and privacy): nothing
undisclosed is committed; arm-A working files are private from their first
write; a find that could affect funds, consensus or keys is dangerous by
default; agents only draft and Lucca alone sends; a private find counts
toward Tide's release candidate (RC) only once it is fixed and disclosed,
and only if it meets the caught rule.

This record therefore names no project, crate or detail of anything held
privately. Held privately: **1 private model-only candidate (2
properties), no crate named**, which the caught rule does not count (Arm A
below); and **12 reading observations** (in 6 reports), possible defects
the builders noticed while reading arm-A source, which the method did not
catch. Reading observations are not method results and are not
catches.
Both were classified provisionally in the private notes and handed to the
coordinator. Nothing was drafted from the method, and nothing was sent or
posted.

**Correction notice.** This is the second version of the record. After
the first version (commit `1270d9e`) and the Codex supervisor pass, two
registered rules that the first version had applied incompletely were
applied as written: the full evidence reading for arm B's rustls rows, and
the property listing without a conflict-based exclusion (Methods and
deviations, "Round 1"). These are corrections that apply registered rules,
not new rules; the counts below include them. Applying §7 step 1 per claim
meant listing claims after the first runs, against §7 step 3's "no
property added after a run": the private candidate's 2 properties are two
of those claims, taken from rows the frozen list already held.

## What this round does not show

- That these crates are free of bugs, or of concurrency bugs. Arm B sees
  only fixes whose message matches the regex in the crate's own directory.
  Arm A tests only properties the crates themselves state, and only the
  ones whose paths are portable and fit the budget: of 430 property rows,
  47 were modeled. Most stop at external crates (§7 step 2).
- Seeded exploration for every modeled property. §7 step 5 asks each test
  to derive its inputs from the case seed. **29 of the 45 tests** (31
  property rows) do; **16 tests** (16 rows, taken from the crates' own
  tests or constants) take no seed-derived input, so their 1,000 seeds
  repeat one deterministic case. Those 16 are fixed checks, not seeded
  explorations (a departure from §7 step 5, Methods and deviations).
- Anything Tide would find unaided. Every property is supplied by upstream
  text, and the generated-input tests sample 1,000 seeds of a frozen
  generator; finite sampling can miss rare inputs, and generator
  distributions were not measured.
- That a model is the crate. Each port records its gaps (integer
  operations, collections, omitted derives that do not execute on the
  path); a model that passes says the model passes, and a model that fails
  is a model-only result until a control and the real crate confirm it.
- An independently verified chronology. Examination times are
  self-attested. The OpenTimestamps proofs show that the protocol and pool
  existed by their attested times, not that no screening took place before.
  The model copies before three compile-only corrections and their first
  compile logs were not retained, so those before/after comparisons rest on
  the logged hashes and descriptions and cannot be reproduced independently
  (no test had run on them). Round 1's claims were listed after the first
  runs (Methods and deviations).
- Anything about the quality of these projects. A scan class is a
  mechanical label (§3), and a failed port says what Tide v0 cannot yet
  express.

The admission observations further below are descriptive lexical counts,
not bugs, properties or catches.

## Result

The pool's crates carry one of two mechanical scan classes (§3): **class
C** (std concurrency: std thread, channel, lock or atomic use in a reached
file and no async or other forbidden source or dependency, by fx-zu4f.10's
lexical filter) and **class P** (deterministic logic: no forbidden
dependency, no `async`, no `std::net`, `std::process`, `std::fs` or
`std::thread` path, no `unsafe` block or fn, 200 to 5,000 non-test lines and
at least one own `#[test]`, by the bead's lexical filter). A crate passing
C is class C even when it also passes P.

The round has two arms. **Arm B** screens past fixes, in the six class C
pool crates, whose commit message matches the registered word-bounded
regex. It judges each candidate against P10's five criteria in order and
records only the first one it fails: **E1** a non-test bug fix in the
crate; **E2** an observable symptom, quoted; **E3** concurrency-shaped;
**E4** small (an affected path of at most 400 lines) and faithfully
expressible in Tide v0, with no network, subprocess or external-crate
operation on it; **E5** explorable by Tide's simulator under sequential
consistency. **Arm A** takes pool ranks 1 to 10 at their pinned commits.
For the one class C crate it locates the concurrent core (P10 §6). For the
nine class P crates it freezes the complete list of properties the crate's
own docs, tests and asserted invariants state, then ports to Tide v0, in
list order, the properties whose paths are portable and fit a 400-line
union, and runs each as a test over seeds 0 to 999
(`#[sim_test(seeds = 1000)]`). A candidate is **caught** only when its
model fails on a seed, the lowest failing seed replays twice with
byte-identical traces, a fixed model (arm B) or fault-removed control
(arm A) passes all 1,000 seeds of every frozen property, and, for arm A,
the same failure reproduces on the real crate (§6, §7).

**Arm B: not caught.** All 9 candidate rows (7 unique commits) fail E1 on
the full evidence (commit, diff, the associated PRs with their comments
and reviews, and linked issues): none is a bug fix in the crate's non-test
code. No model was built.

**Arm A: not caught.** The class C core is unportable. Of 430 property rows
listed for the nine class P crates, 47 were modeled, in 45 tests: 29 with
seed-generated inputs (31 rows) and 16 with fixed inputs (16 rows, the
§7 step 5 departure above). 43 tests passed all 1,000 seeds. 2 tests
(2 properties) failed on every seed; their lowest failing seed replayed
twice with byte-identical traces, but the fault-removal control failed a
frozen property, so the caught rule was not met and no reproduction on a
real crate ran. That property is the competing claim from the same
doc-versus-own-test conflict as the 2 failing properties, so no control
can pass all three by construction. The 2 properties were listed after the
first runs, in round 1, from rows the frozen list already held. They are
the 1 private model-only candidate above, not a catch. Three class P
crates had no portable property.

**No catch, so no case study.** Nothing upstream was built or executed.

## Registration and identities

- Registration commit `9b90de8` (2026-11-19T05:16:55Z) on branch
  `fx-zu4f.12`, base main `a4ab41c975442fe360363677d038aca71d9d4c10`. It
  holds the protocol and the pool directory, byte-identical to the
  builder's work copies.
- OpenTimestamps: pending proofs `cd741c8` (05:18:37Z); upgraded `644bdf9`
  (06:34:23Z), every file anchored in Bitcoin block 976536.
- Snapshot 2026-11-19T04:57:46Z. GO sent 06:37:08Z; the builder recorded
  receipt and the start of per-crate examination at
  **2026-11-19T06:37:12Z**. The first per-crate examination was the
  admission-observation run, 06:38:11Z to 06:38:45Z, which read every pool
  crate's non-test source mechanically.
- Tide revision for every build and run: main
  `b1f044744dea32e69cf0e6a92c4542496070f8ed`, merged into the branch as
  `b79cbbe` (crates/, Cargo.toml, Cargo.lock and rust-toolchain.toml
  tree-identical). The driver and std were built once, 07:14:47Z to
  07:15:42Z, through heavy.sh, by the recipe in docs/driver.md.
- Environment: Python 3.14.10, Git 2.43.0, gh 2.104.0, Linux x86_64.

SHA-256 of the registered files (`research/protocols/`):

| File | SHA-256 |
| --- | --- |
| `fx-zu4f.12-protocol.md` | `09a4f5b957a1496f464e5dc93f65c0c623edbdd690b0217a8f77ff8d10361dc6` |
| `fx-zu4f.12-pool/admission-obs.py` | `5823c3440408412ee5d25b3ac712141ffd67f1fdc7906a39080c8949a23b5fc9` |
| `fx-zu4f.12-pool/candidates.tsv` | `1ebbf4984c7087fe47e7b747d672f3be468e910d8d79c86bd5880f3fa7c49853` |
| `fx-zu4f.12-pool/clone-refs.tsv` | `1be2ebaac7846fa04dd72ff470e525638cf1ef53742b5ab5aacb7ee70667ea8c` |
| `fx-zu4f.12-pool/pool.tsv` | `b06412725ec259ecd2ecebf733d638de6b3ba2d8773e3c02b637daa4e9e3b723` |
| `fx-zu4f.12-pool/queries.jsonl` | `7487b52432563977d90e7eb31129f09e61145722a40204a58955e13c860041ab` |
| `fx-zu4f.12-pool/scan-output.tsv` | `d73756c79da4a6a2e7a092eccc4b867315c8e5a02484cf3d67fe645f669e51f1` |
| `fx-zu4f.12-pool/scan.py` | `a7a983011884e1ac2df3d0d63e6b187ca2a098c6d4b41a03de2c6af870c696c4` |
| `fx-zu4f.12-pool/search-hits.tsv` | `c1d78900406f4e86b4ec77b1911e57405b437476f299b5776584a87ad216e876` |
| `fx-zu4f.12-pool/search.py` | `524314e73c74b0feeb98c42b2a8f314fafc3d0528f804e754b258bf579141570` |
| `fx-zu4f.12-pool/star-slices.json` | `25fc57cf9beb5f6082cb88d651da1989fb161801f5175ddf4384d024d2e41f55` |

**Private material.** Arm-A properties, path analyses, models, run notes
and the reading observations are only in the coordinator's private
directory (Disclosure and privacy), never in this repository. Its
`armA/property-log.txt` records the SHA-256 and UTC time of every frozen
property list, path analysis and model before any build or run that used
it, and every correction after; its SHA-256 at the time of this version
is `2628892bbff6dced0cd0ed496364f7a742c00ad731d706d9a1f32927142bdd2b` (the
first version's was `3c833ce3df872062da75cf15fea6e1a38b4e854ece076d8f96790c949e8048f4`;
round 1 appended four lines). Run logs are under the builder's TARGET
directory.

## Methods and deviations

The run was done by twelve Claude Opus builder sessions in sequence, each
handing off at a checkpoint in its report near its context budget: R1 from
06:37:12Z, R2 06:53:28Z, R3 07:04:07Z, R4 07:09:21Z, R5 07:17:47Z, R6
07:25:41Z, R7 07:43:35Z, R8 08:07:44Z, R9 08:16:33Z and R10 08:41:04Z
(2026-11-19, UTC); after the first record and the Codex pass, round 1 was
done by R10 from 09:02Z, R11 from 09:04Z and R12 from 09:12Z. No session
started a subagent. Every compile and seeded run went through heavy.sh.

Choices the protocol left to the builder, recorded before use:

- Arm-A trees: one sparse, hook-free checkout per repository at the pinned
  commit holding every file of the targeted crate directories and the
  workspace manifests, not the full monorepo tree (§7 says the builder
  "may" check out the full tree; this reads less). Same-repository helpers
  were read from the pinned commit with `git show`.
- Property-list rules fixed in each list's header before the list: a doc
  quote is a property of the item it documents (trait docs bind every
  hand-written in-crate impl; generated impls are not triggers); an
  ambiguous quote asserts only the weaker common claim; the crate's own
  asserted error arms are their own rows; `//` comments with checkable
  claims count; state fields take only values the crate's own writers
  store. Rows that state no checkable property ("no property") or whose
  trigger is outside the crate ("no trigger") are counted together as
  **not assertable**. The first version also excluded a row where a doc and
  the crate's own test contradict ("conflicting sources"); §7 has no such
  exclusion, so round 1 withdrew it and listed each competing claim as its
  own row (Round 1, below).
- Path rule: external-crate code executed on a property's path, including
  code an external derive or macro generates, makes it unportable; an
  external derive or type whose code does not execute on the path is a
  recorded gap. The 400-line union was counted mechanically in list order
  by a script, before any model was written.
- Rows from the crates' own tests, and a few rows on constants, are
  modeled on fixed inputs; where such an input is an external type that the
  path does not execute, a fixed stand-in replaces it (recorded per row).
  This departs from §7 step 5 (see the deviations).

Deviations, in time order (2026-11-19, UTC):

| When | Deviation | Reason | Effect |
| --- | --- | --- | --- |
| 06:39Z (R1) | A `git ls-tree -r -l` listing on the blobless clones fetched blob sizes lazily from GitHub (the same read-only source) and started git auto-gc; stopped after one repository | Listing files for the security-program check | Nothing was read from it |
| 06:42:08Z-06:43:00Z (R1) | The security-program notes' header says "read 06:40-06:58Z"; the logged GitHub GETs ran 06:42:08Z to 06:43:00Z and R1 handed off at 06:46Z | Header written as an estimate | This record states the logged interval |
| 06:54Z-06:56Z (R2) | Arm B row B1: `examples/Cargo.lock`'s changed lines were checked identical to the root lockfile's by `diff`, not reread line by line | Duplicate lockfile hunk | None on the judgment (E1 rests on the crate's source diff) |
| 06:55Z-06:56Z (R2) | rustls rows B2-B9: each commit's message, complete diff and associated PR description were read; PR comment threads, reviews and linked issues were not fetched | R2 read P10 §3's inventory deadline ("before any failure at E2 or later") as waiving its reading rule at E1; it does not | Corrected in round 1 (below); every row still fails E1 |
| 07:14Z-08:50Z (R4-R10) | 16 arm-A tests (16 rows: the crates' own tests and constants) take no seed-derived input, so their 1,000 seeds repeat one case | §7 step 5 asks every test to derive its inputs from the case seed; the builders modeled test rows on the tests' own fixed inputs | Those rows are fixed checks, not seeded explorations; split reported in Arm A |
| 07:36:52Z (R5) | Rank 4 model: compile-only correction after the first build failed to compile (no test ran) | Tide v0 rejected user macros, an impl's placement and `==` on an error type without `PartialEq` | Re-frozen and logged before the next build; properties unchanged. The pre-correction copy and its compile log were not retained |
| 07:51:49Z (R7) | Rank 5 property list re-frozen for citation line numbers only (one file's lines were offset by 34) | Citation error | Properties unchanged; both hashes logged |
| 08:00:10Z (R7) | One run with a prefix test filter exited 2; no test ran | The driver accepts exact test names only | None |
| 08:00:27Z (R7) | One batch of three arm-A tests started at 1-minute load 14.12, above the limit of 12; load was printed in the same command instead of checked first | Gating error | Load does not enter a test's result; every later batch was gated in a separate step |
| 08:18Z-08:37Z (R9) | Property-log lines for ranks 8 and 9 corrected for row counts, and the read windows in ranks 8-10's list headers marked as estimates; each crate's reads ended before its logged freeze | Count and time slips in notes | Files unchanged; corrections logged |
| 08:32Z-08:41Z (R9, R10) | R9's background batch for three rank-8 tests died with its session before running; R10 reran them | Session ended | Logs from R10's runs |
| 08:43:05Z (R10) | Rank 9 model: compile-only correction after the first build failed to compile (TD0301 on a lossy `u128 as u64`, which Tide spells `truncate`; no test ran) | Tide v0 rejects lossy `as` | Same semantics; re-hashed and logged before the next build. The pre-correction copy and its compile log were not retained |
| 08:45:49Z (R10) | Rank 8 path analysis: its totals line gives 3 fewer unportable rows than its own row table names | Arithmetic slip | This record uses the row table's count; correction logged, file unchanged |
| 08:48:16Z (R10) | Rank 10 model: compile-only correction after the first build failed to compile (TD0301 on five `i64 as u64` casts of non-negative values in the tests, which Tide spells `cast_unsigned`; no test ran) | Tide v0 rejects lossy `as` | Same semantics; re-hashed and logged before the next build. The pre-correction copy and its compile log were not retained |
| 09:02:33Z-09:06:59Z (R10, R11) | Round 1, arm B: for rustls rows B2-B9, the associated PRs' complete comments, reviews and review comments and their linked issues were fetched (26 read-only GETs, at least 2 s apart, all successful) and read; every row re-judged on the full evidence | Codex r1 finding 1: P10's evidence rule binds every row, E1 included | All 9 rows still fail E1; no row continues to E2. Reading reductions: benchmark-bot table rows matching one strict pattern were checked by the pattern (the 15 that did not match were read), and PR #1502's review-comment diff hunks were cut to their headers (that code was read from the frozen clone) |
| 09:09:54Z-09:18:27Z (R11, R12) | Round 1, arm A: the conflict-based exclusion was dropped; the 4 former "conflicting sources" rows' competing claims were listed per §7 step 1 as 7 claims (hashed 09:09:54Z), disposed by steps 2-4 (5 unportable, 2 portable within the 400-line union), and the 2 portable claims modeled and run under steps 5-6 (model hashed 09:13:50Z before its build; control hashed 09:15:53Z) | Codex r1 finding 2: §7 has no conflict-based exclusion | Counts below include them, per claim (rows 427 to 430). The claims were listed after the first runs (against §7 step 3's "no property added after a run"); they come from rows the first list held as "conflicting sources", not from new sources |

## Selection counts

| Stage | Unit | Count |
| --- | --- | ---: |
| Distinct repository-search results (12 topic queries, one slice each) | repositories | 76 |
| Dropped by the repository filter (none owner-affiliated) | repositories | 0 |
| Pinned / holding a Cargo package | repositories | 76 / 75 |
| Scanned | packages | 3,921 |
| Class C (of them also passing P) | packages | 112 (13) |
| Class P | packages | 601 |
| Excluded | packages | 3,208 |
| Kept (C or P) | packages / repositories | 713 / 57 |
| Pool (at most 3 per repository, by stars) | crates / repositories | 30 / 12 |
| Pool class C / class P | crates | 6 / 24 |

The scan is lexical (§3): no macro expansion, feature or platform
selection, or complete name resolution. Passing it does not make a core
portable.

## Pool and coverage

Arm A targets ranks 1 to 10; arm B targets the class C crates (ranks 6,
14, 23, 24, 25, 27). Ranks 11 to 30 that are class P are in neither arm;
they were scanned and observed for admission only.

| Rank | Class | Repository | Crate | Pinned commit | Arm A outcome | Arm B outcome |
| ---: | --- | --- | --- | --- | --- | --- |
| 1 | P | unionlabs/union | access-managed | `031785bb6dc6` | unportable (every property) | - |
| 2 | P | unionlabs/union | access-manager | `031785bb6dc6` | unportable (every property) | - |
| 3 | P | unionlabs/union | aptos-types | `031785bb6dc6` | ≥1 modeled; not caught | - |
| 4 | P | FuelLabs/fuel-core | fuel-core-storage | `8bd82d4d1ff4` | ≥1 modeled; not caught | - |
| 5 | P | FuelLabs/fuel-core | fuel-gas-price-algorithm | `8bd82d4d1ff4` | ≥1 modeled; not caught | - |
| 6 | C (also P) | linera-io/linera-protocol | linera-witty | `672d97bb2fce` | concurrent core unportable | 1 candidate, E1 |
| 7 | P | linera-io/linera-protocol | linera-witty-macros | `672d97bb2fce` | unportable (every assertable property) | - |
| 8 | P | zama-ai/fhevm | kms-connector-api | `3054cadc7969` | ≥1 modeled; not caught | - |
| 9 | P | zama-ai/fhevm | oft | `3054cadc7969` | ≥1 modeled; not caught | - |
| 10 | P | zama-ai/fhevm | primitives | `3054cadc7969` | ≥1 modeled; not caught | - |
| 11 | P | diem/diem | accumulator | `fc4714a8ea27` | - | - |
| 12 | P | diem/diem | consensus-types | `fc4714a8ea27` | - | - |
| 13 | P | diem/diem | diem-crypto | `fc4714a8ea27` | - | - |
| 14 | C | QuipNetwork/quip-solver | quip-solver-exec | `8ee9f8159c2d` | - | no candidate |
| 15 | P | foundry-rs/foundry | anvil-core | `9b2e485999ac` | - | - |
| 16 | P | foundry-rs/foundry | anvil-rpc | `9b2e485999ac` | - | - |
| 17 | P | foundry-rs/foundry | forge-script-sequence | `9b2e485999ac` | - | - |
| 18 | P | datahaven-xyz/datahaven | datahaven-runtime-common | `edcb13dbbcd3` | - | - |
| 19 | P | datahaven-xyz/datahaven | pallet-datahaven-native-transfer | `edcb13dbbcd3` | - | - |
| 20 | P | datahaven-xyz/datahaven | pallet-evm-precompile-balances-erc20 | `edcb13dbbcd3` | - | - |
| 21 | P | MystenLabs/sui | invalid-mutations | `6463a0de12c8` | - | - |
| 22 | P | MystenLabs/sui | move-abstract-interpreter-v2 | `6463a0de12c8` | - | - |
| 23 | C | MystenLabs/sui | move-bytecode-utils | `6463a0de12c8` | - | no candidate |
| 24 | C | rustls/rustls | bogo | `836e411eccaf` | - | 2 candidates, E1 |
| 25 | C | rustls/rustls | rustls | `836e411eccaf` | - | 6 candidates, E1 |
| 26 | P | rustls/rustls | rustls-aws-lc-rs | `836e411eccaf` | - | - |
| 27 | C | aptos-labs/aptos-core | aptos-abstract-gas-usage | `4cddc9f17e6a` | - | no candidate |
| 28 | P | aptos-labs/aptos-core | aptos-accumulator | `4cddc9f17e6a` | - | - |
| 29 | P | aptos-labs/aptos-core | aptos-aggregator | `4cddc9f17e6a` | - | - |
| 30 | P | paradigmxyz/reth | jsonrpsee-types | `8fa8b31ea218` | - | - |

## Arm B: retrospective fixes

Each class C crate's frozen clone was exported from its pinned HEAD
(`arm-b-enum.py`, 06:44:29Z to 06:44:39Z), the registered regex applied to
every full message, and each match's changed paths checked against the
crate's own non-test directory (§6).

| Crate (rank) | Commits exported | Regex matches | Candidate rows |
| --- | ---: | ---: | ---: |
| linera-witty (6) | 5,465 | 90 | 1 |
| quip-solver-exec (14) | 2,098 | 105 | 0 |
| move-bytecode-utils (23) | 21,017 | 246 | 0 |
| bogo (24) | 5,856 | 9 | 2 |
| rustls (25) | 5,856 | 9 | 6 |
| aptos-abstract-gas-usage (27) | 25,325 | 201 | 0 |

bogo and rustls share one repository export, so its 9 matches are counted
once per crate. Matches are commits, counted per repository export.

| Measure | Unit | Count |
| --- | --- | ---: |
| Targeted / screened / unresolved | crates | 6 / 6 / 0 |
| Candidate rows / unique commits / duplicate rows | rows, commits, rows | 9 / 7 / 2 |
| First failure E1 / E2 / E3 / E4 / E5 | candidate rows (9) | 9 / 0 / 0 / 0 / 0 |
| Eligible / modeled / properties / caught | rows | 0 / 0 / 0 / 0 |

Every row's matched word is an idiom or a name, not a concurrency symptom:
"stuck" (a toolchain version), "hangs" and "hanging" (what depends on
what), "hang on to" (retaining a value), and `race` (once_cell's feature
and module name). Evidence: each commit's message and complete diff from
the frozen clone (`git show`), and read-only GitHub GETs at least 2 s
apart for associated PRs and linked issues (12 GETs, 06:54:47Z to
06:56:02Z; in round 1, 26 more for rows B2-B9, 09:02:33Z to 09:06:59Z; all
successful). Links inside these threads to design history or to other
projects' issues were not followed: none is cited as reporting or
explaining a bug these commits fix, so none bears on E1 (P10 §3 follows
links "needed to establish E1/E2").

| Row | Crate | Commit | First parent | First failure | Reason | Sources read | E2-E5 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| B1 | linera-witty | `7b4ddf7c352d` | `fe0166650062` | E1 | Toolchain update: the crate's only change rewrites two impl headers with elided lifetimes (same semantics). The bug the PR closes (#3196) was fixed in a dependency | message, full patch (56 files), PR #3622 with its comments, reviews and review comments, issue #3196 with its comments | not assessed |
| B2 | bogo | `9c4e53871318` | `13eddd029536` | E1 | API removal ("Remove version selection builder API"); bogo only adapts its config construction. The linked issue #1659 is an API design request; the PR thread discusses where builder errors surface, and no wrong behavior is reported | message, full patch, PR #2599 with its comments, reviews and review comments, issue #1659 with its comments | not assessed |
| B3 | bogo | `ce318437dc6b` | `fe9c178b0674` | E1 | Feature removal (`tls12` crate feature); bogo only drops the feature from its Cargo.toml. Linked issues #224 (discard unused protocol versions at link time) and #2401 (whether CryptoProvider should be a trait) are a feature request and a design question | message, full patch, PR #2568 with its comments, reviews and review comments, issues #224 and #2401 with their comments | not assessed |
| B4 | rustls | `15535493295a` | `29c3a7421d4e` | E1 | Feature work ("add std feature"); manifests only. PR #1502 is no-std support phase I per RFC #1399; #1534 is a no-std example and #1583 the API it builds on; the one defect named (in #1583) is in that PR's own new code, fixed before merge | message, full patch, PR #1502 with its comments, reviews and review comments, #1399, #1534 and #1583 with their comments | not assessed |
| B5 | rustls | `8ed615e5d989` | `ea571f2740cb` | E1 | no-std support (`race::OnceBox` variant); the std path is unchanged. The OnceBox choice came from review as a lighter dependency, not a bug | as B4 | not assessed |
| B6 | rustls | `9c4e53871318` | `13eddd029536` | E1 | Duplicate of B2: API redesign, no bug named | as B2 | not assessed |
| B7 | rustls | `c84675e34b52` | `788b0df12216` | E1 | Secret-lifetime hardening refactor; no defect or failure stated. PR #2540 is "a step toward" issue #945, an API redesign request after RFC 8446's SHOULD-level advice to erase secrets early; its threads are review nits | message, full patch, PR #2540 with its comments, reviews and review comments, issue #945 with its comments | not assessed |
| B8 | rustls | `ce318437dc6b` | `fe9c178b0674` | E1 | Duplicate of B3: cfg gates removed, feature deleted | as B3 | not assessed |
| B9 | rustls | `e5998cd100b8` | `9620bec130da` | E1 | Key-schedule refactor; no bug named (PR #2540 and issue #945 as B7) | as B7 | not assessed |

## Arm A: forward hunt at the pinned commits

| Measure | Unit | Count |
| --- | --- | ---: |
| Targeted / screened / unresolved | crates | 10 / 10 / 0 |
| Class C core unportable | crates | 1 |
| Class P crates with every property unportable or not assertable | crates | 3 |
| Class P crates with at least one modeled property | crates | 6 |
| Property rows listed (class P; 427 frozen first, 3 more from round 1's per-claim listing) | rows | 430 |
| Modeled / over budget / unportable / not assertable | rows | 47 / 23 / 315 / 44 |
| Split rows (one part modeled or over budget, the other unportable) | rows | 4 |
| Tests run (1,000 seeds each) | tests | 45 |
| with seed-generated inputs / with fixed inputs (§7 step 5 departure) | tests (rows) | 29 (31) / 16 (16) |
| Passing every seed / failing every seed | tests | 43 / 2 |
| Failing tests whose lowest failing seed replayed twice with byte-identical traces | tests | 2 |
| Fault-removal controls passing every frozen property | controls | 0 of 1 |
| Private model-only candidates / caught | candidates | 1 / 0 |

"Not assertable" is 27 rows with no checkable property and 17 whose
trigger is outside the crate. Round 1 replaced the first version's 4
"conflicting sources" rows by their 7 competing claims, one row each
(2 modeled, 5 unportable). Split rows: one row whose V0 half is over
budget and V1 half unportable, counted in neither column; three test rows
whose fixed-input part is modeled and whose other part is unportable,
counted as modeled. 430 = 47 + 23 + 315 + 44 + 1.

| Rank | Crate | First failure of the unportable rows |
| ---: | --- | --- |
| 1 | access-managed | external crates on every path: cosmwasm-std storage, querier and Addr; serde; serde-json-wasm |
| 2 | access-manager | cosmwasm-std storage on every entry point; also sha2, serde, serde_json |
| 3 | aptos-types | hex, bcs, sha2, serde, sha3 |
| 4 | fuel-core-storage | fuel-vm crates (fuel-merkle, fuel-types), anyhow, postcard, strum, derive_more, impl_tools |
| 5 | fuel-gas-price-algorithm | tracing macros executed on every V1 update path; one exponent table alone is 678 lines, so its properties are over budget |
| 7 | linera-witty-macros | syn, proc-macro2, quote on every trigger |
| 8 | kms-connector-api | strum and serde derives executed on the path, alloy, actix-web |
| 9 | oft | Anchor, spl-token-2022, solana Pubkey, the OApp crates |
| 10 | primitives | serde, thiserror, uuid, alloy |

Rank 4's list counts a property with eight parts, all unportable, as
one row. Rows, modeled, over-budget, unportable and not-assertable counts,
tests, and pass and fail are given only as arm totals (Disclosure and
privacy).

**Rank 6, linera-witty (class C, P10 §6).** Every std synchronization use
in the non-test source was located; the crate spawns no thread and uses
no scope or channel. Two roots: the mock runtime (a `Mutex` memory and user
data taken with `try_lock`, an `AtomicUsize` call counter) and the wasmer
environment (`OnceLock` exports and memory, a `Mutex` for user data). The
union of the core is 1,053 nonblank non-comment lines. It is unportable on
all four P10 §6 conditions; the first in P10's order is unresolved
dispatch through caller-supplied `dyn Fn` handlers and macro-generated
source, and the others are external crates on the core path (wasmer,
frunk), operations Tide v0 lacks (`Box<dyn Any>` downcasts, generic
associated types, `OnceLock`), and the size (over 400 lines). Under P10 §6
the core's location precedes its properties, so no property was listed.

**Runs.** Each modeled property is one test, or several tightly related
properties share one test (in one crate; 47 rows in 45 tests overall). Every
test ran through heavy.sh with the 1-minute load checked below 12 in a
separate step first (except R7's one batch above). 43 logs report "1000
passed; 0 failed (1000 seeds)", "test result: ok" and exit 0; every line
of them was read. 16 tests take no seed-derived input (fixed inputs from
the crates' own tests or constants), so their 1,000 seeds repeat one
deterministic case.

**Round 1 runs (09:14Z-09:18Z).** The 2 round-1 tests take seed-generated
inputs inside their claims' preconditions, and both failed on all 1,000
seeds (exit 101), lowest failing seed 0. Seed 0 of each was replayed twice
from the driver's printed replay command: the trace lines were
byte-identical; the full outputs differed only in the per-run directory
name, elapsed time and the interleaving of stdout and stderr. The
fault-removal control, the model with only the suspected fault removed and
frozen before its build, passed both round-1 tests and every other frozen
property of that crate except one, which failed on all 1,000 seeds. That
one is the competing claim from the same doc-versus-own-test conflict, so
no control can pass it and the 2 round-1 properties together. The
caught rule needs a control that passes every frozen property, so it
failed there: no reproduction on a real crate ran, and the §7 `cargo fetch`
was never used.

## Admission observations (descriptive)

`admission-obs.py --after-go` ran on 369 packages (the 30 pool crates and
339 class-excluded packages with a binary target in a pool repository),
06:38:11Z to 06:38:45Z, exit 0: **3,394 observations, 506 key-flagged**.
Its output digest is
`ab1a3a33b4ee0e204050795e24d50efaafba31e10cc7ad0eca70ab16bb03682c`. The
counts are lexical: ambient `std` imports and references, and spawn
handles, that Tide admission would make explicit. They are not execution
counts, compiler diagnoses, bugs or catches. The key flag marks an
observation whose enclosing function also names a key-like identifier
(§5); `key` is a common word (map keys), so the flag over-approximates key
handling and is a lexical count, never a finding.

| Kind | Pool crates: not flagged / key-flagged | Excluded binary packages: not flagged / key-flagged |
| --- | ---: | ---: |
| `std::env` imports | 2 / 0 | 74 / 0 |
| `std::env` references | 1 / 2 | 358 / 95 |
| `std::fs` imports | 0 / 0 | 243 / 0 |
| `std::fs` references | 2 / 0 | 957 / 170 |
| `std::net` imports | 1 / 0 | 142 / 0 |
| `std::net` references | 7 / 5 | 242 / 30 |
| `std::process` imports | 5 / 0 | 132 / 0 |
| `std::process` references | 11 / 8 | 444 / 103 |
| spawns, detached | 1 / 0 | 104 / 22 |
| spawns, joined | 0 / 0 | 30 / 15 |
| spawns, unresolved | 1 / 0 | 131 / 56 |
| **Total** | **31 / 15** | **2,857 / 491** |

Two pool crates have any observation: quip-solver-exec (15) and bogo (31).
Of the 339 excluded binary packages, 202 have at least one. Imports are
never key-flagged because they sit outside any function.

## Security programs and AI-report policies

Read after GO, before any draft, for all 12 pool repositories: each
repository's root SECURITY.md, CONTRIBUTING.md (root or `.github/`) and
bug-report issue templates at its pinned commit, in full, from the frozen
clones; and, by read-only GitHub GETs logged 06:42:08Z to 06:43:00Z on
2026-11-19, the repository's private-vulnerability-reporting (PVR) status
and, where the repository has no SECURITY.md, its owner's `.github`
SECURITY.md. Off-GitHub program pages were named, not fetched (Safety
allows only GitHub and crates.io GETs). Absence claims cover only these
documents. The sources table after the policy table gives each document's
URL (pinned to the commit for repository files) and when it was read; the
API GETs' responses and digests are kept with the builder's notes.

| Repository | Security channel (source) | PVR | Program | AI-generated reports |
| --- | --- | --- | --- | --- |
| unionlabs/union | none found (no SECURITY.md; no owner `.github` SECURITY.md) | disabled | none stated | no policy found |
| FuelLabs/fuel-core | GitHub private advisory (SECURITY.md) | enabled | none stated | no policy found |
| linera-io/linera-protocol | none found (no SECURITY.md; no owner `.github` SECURITY.md) | disabled | none stated | no policy found |
| zama-ai/fhevm | GitHub draft security advisory (SECURITY.md) | enabled | none stated | no policy found |
| diem/diem | SECURITY.md points to an off-GitHub page (not fetched); the bug template names a security email | disabled | not determined | no policy found |
| QuipNetwork/quip-solver | GitHub private advisory (owner `.github` SECURITY.md) | enabled | none stated | no policy found |
| foundry-rs/foundry | security email (owner `.github` SECURITY.md) | disabled | none stated | no policy found |
| datahaven-xyz/datahaven | none found (no SECURITY.md; no owner `.github` SECURITY.md) | disabled | none stated | no policy found |
| MystenLabs/sui | HackenProof only, not GitHub, email or Discord (SECURITY.md) | enabled | HackenProof bounty (page not fetched) | no policy found |
| rustls/rustls | GitHub draft security advisory (SECURITY.md) | enabled | none (RustSec advisory after a fix) | **restricts**: CONTRIBUTING's AI policy says comments to maintainers are to be written by humans and an issue's problem described in the reporter's own words; SECURITY.md asks reporters to disclose AI assistance upfront |
| aptos-labs/aptos-core | HackenProof bounty (SECURITY.md); the bug template names a security email | disabled | HackenProof bounty (page not fetched) | no policy found |
| paradigmxyz/reth | security email (SECURITY.md; the bug template names a different address, SECURITY.md is the stated policy) | enabled | none stated | no policy found |

Sources:

| Repository | Documents read at the pinned commit (2026-11-19, R1, 06:37Z-06:46Z) | Read by API (2026-11-19, UTC) | Named, not fetched |
| --- | --- | --- | --- |
| unionlabs/union | [CONTRIBUTING.md](https://github.com/unionlabs/union/blob/031785bb6dc6b957c624e62bc64c184409c97d7b/CONTRIBUTING.md), [.github/ISSUE_TEMPLATE/app-bug-report.md](https://github.com/unionlabs/union/blob/031785bb6dc6b957c624e62bc64c184409c97d7b/.github/ISSUE_TEMPLATE/app-bug-report.md), [.github/ISSUE_TEMPLATE/site-bug-report.md](https://github.com/unionlabs/union/blob/031785bb6dc6b957c624e62bc64c184409c97d7b/.github/ISSUE_TEMPLATE/site-bug-report.md) | [PVR status](https://api.github.com/repos/unionlabs/union/private-vulnerability-reporting) 06:42:08Z; owner `.github` SECURITY.md at root, `.github/` and `docs/`: 404, 06:42:56Z-06:43:00Z | - |
| FuelLabs/fuel-core | [SECURITY.md](https://github.com/FuelLabs/fuel-core/blob/8bd82d4d1ff4e6da7a0e9febbe96e6a864d79f1f/SECURITY.md), [CONTRIBUTING.md](https://github.com/FuelLabs/fuel-core/blob/8bd82d4d1ff4e6da7a0e9febbe96e6a864d79f1f/CONTRIBUTING.md) | [PVR status](https://api.github.com/repos/FuelLabs/fuel-core/private-vulnerability-reporting) 06:42:10Z | - |
| linera-io/linera-protocol | [CONTRIBUTING.md](https://github.com/linera-io/linera-protocol/blob/672d97bb2fce3127db351069df72f1d225d8f263/CONTRIBUTING.md) | [PVR status](https://api.github.com/repos/linera-io/linera-protocol/private-vulnerability-reporting) 06:42:12Z; owner `.github` SECURITY.md at root, `.github/` and `docs/`: 404, 06:42:50Z-06:42:54Z | - |
| zama-ai/fhevm | [SECURITY.md](https://github.com/zama-ai/fhevm/blob/3054cadc7969a17eda46ca606fa1d1a110a4743e/SECURITY.md), [.github/CONTRIBUTING.md](https://github.com/zama-ai/fhevm/blob/3054cadc7969a17eda46ca606fa1d1a110a4743e/.github/CONTRIBUTING.md), [.github/ISSUE_TEMPLATE/config.yml](https://github.com/zama-ai/fhevm/blob/3054cadc7969a17eda46ca606fa1d1a110a4743e/.github/ISSUE_TEMPLATE/config.yml), [.github/ISSUE_TEMPLATE/general_issue.yml](https://github.com/zama-ai/fhevm/blob/3054cadc7969a17eda46ca606fa1d1a110a4743e/.github/ISSUE_TEMPLATE/general_issue.yml) | [PVR status](https://api.github.com/repos/zama-ai/fhevm/private-vulnerability-reporting) 06:42:14Z | - |
| diem/diem | [SECURITY.md](https://github.com/diem/diem/blob/fc4714a8ea273b6efe8b13dbce72ea60aad9a16c/SECURITY.md), [CONTRIBUTING.md](https://github.com/diem/diem/blob/fc4714a8ea273b6efe8b13dbce72ea60aad9a16c/CONTRIBUTING.md), [.github/ISSUE_TEMPLATE/bug_report.md](https://github.com/diem/diem/blob/fc4714a8ea273b6efe8b13dbce72ea60aad9a16c/.github/ISSUE_TEMPLATE/bug_report.md) | [PVR status](https://api.github.com/repos/diem/diem/private-vulnerability-reporting) 06:42:16Z | <https://developers.diem.com/docs/reference/security> (from SECURITY.md); a security email (bug template) |
| QuipNetwork/quip-solver | none of the listed documents exists | [PVR status](https://api.github.com/repos/QuipNetwork/quip-solver/private-vulnerability-reporting) 06:42:18Z; owner SECURITY.md, [QuipNetwork/.github `SECURITY.md`](https://github.com/QuipNetwork/.github/blob/main/SECURITY.md), default branch, as blob [`70d76a24`](https://api.github.com/repos/QuipNetwork/.github/git/blobs/70d76a24a2212833f70c3a2b39c2176c63157b58) 06:42:32Z | - |
| foundry-rs/foundry | [CONTRIBUTING.md](https://github.com/foundry-rs/foundry/blob/9b2e485999acad0a92df356f60e068848e7e62a7/CONTRIBUTING.md), [.github/ISSUE_TEMPLATE/BUG-FORM.yml](https://github.com/foundry-rs/foundry/blob/9b2e485999acad0a92df356f60e068848e7e62a7/.github/ISSUE_TEMPLATE/BUG-FORM.yml), [.github/ISSUE_TEMPLATE/config.yml](https://github.com/foundry-rs/foundry/blob/9b2e485999acad0a92df356f60e068848e7e62a7/.github/ISSUE_TEMPLATE/config.yml) | [PVR status](https://api.github.com/repos/foundry-rs/foundry/private-vulnerability-reporting) 06:42:20Z; owner SECURITY.md, [foundry-rs/.github `SECURITY.md`](https://github.com/foundry-rs/.github/blob/main/SECURITY.md), default branch, as blob [`5880fe5b`](https://api.github.com/repos/foundry-rs/.github/git/blobs/5880fe5b1d3e8d9e9f0e2732fbcd991b32826266) 06:42:44Z | the security email in the owner's SECURITY.md |
| datahaven-xyz/datahaven | none of the listed documents exists | [PVR status](https://api.github.com/repos/datahaven-xyz/datahaven/private-vulnerability-reporting) 06:42:22Z; owner `.github` SECURITY.md at root, `.github/` and `docs/`: 404, 06:42:38Z-06:42:42Z | - |
| MystenLabs/sui | [SECURITY.md](https://github.com/MystenLabs/sui/blob/6463a0de12c80520f1a6186a1109202ec4b73c7f/SECURITY.md), [CONTRIBUTING.md](https://github.com/MystenLabs/sui/blob/6463a0de12c80520f1a6186a1109202ec4b73c7f/CONTRIBUTING.md), [.github/ISSUE_TEMPLATE/code-bug.md](https://github.com/MystenLabs/sui/blob/6463a0de12c80520f1a6186a1109202ec4b73c7f/.github/ISSUE_TEMPLATE/code-bug.md) | [PVR status](https://api.github.com/repos/MystenLabs/sui/private-vulnerability-reporting) 06:42:24Z | <https://hackenproof.com/programs/sui-protocol> and <https://hackenproof.com/sui/sui> (from SECURITY.md) |
| rustls/rustls | [SECURITY.md](https://github.com/rustls/rustls/blob/836e411eccaf7331f4c289aabef0de32474de3b8/SECURITY.md), [CONTRIBUTING.md](https://github.com/rustls/rustls/blob/836e411eccaf7331f4c289aabef0de32474de3b8/CONTRIBUTING.md), [.github/ISSUE_TEMPLATE/bug_report.md](https://github.com/rustls/rustls/blob/836e411eccaf7331f4c289aabef0de32474de3b8/.github/ISSUE_TEMPLATE/bug_report.md) | [PVR status](https://api.github.com/repos/rustls/rustls/private-vulnerability-reporting) 06:42:26Z | - |
| aptos-labs/aptos-core | [SECURITY.md](https://github.com/aptos-labs/aptos-core/blob/4cddc9f17e6a37e055edcd05f6873d1137dd7cfe/SECURITY.md), [CONTRIBUTING.md](https://github.com/aptos-labs/aptos-core/blob/4cddc9f17e6a37e055edcd05f6873d1137dd7cfe/CONTRIBUTING.md), [.github/ISSUE_TEMPLATE/bug_report.md](https://github.com/aptos-labs/aptos-core/blob/4cddc9f17e6a37e055edcd05f6873d1137dd7cfe/.github/ISSUE_TEMPLATE/bug_report.md), [.github/ISSUE_TEMPLATE/config.yml](https://github.com/aptos-labs/aptos-core/blob/4cddc9f17e6a37e055edcd05f6873d1137dd7cfe/.github/ISSUE_TEMPLATE/config.yml) | [PVR status](https://api.github.com/repos/aptos-labs/aptos-core/private-vulnerability-reporting) 06:42:28Z | <https://hackenproof.com/aptos> (from SECURITY.md); a security email (bug template) |
| paradigmxyz/reth | [SECURITY.md](https://github.com/paradigmxyz/reth/blob/8fa8b31ea2185f30d57f1ff3fec53e03b13ad024/SECURITY.md), [CONTRIBUTING.md](https://github.com/paradigmxyz/reth/blob/8fa8b31ea2185f30d57f1ff3fec53e03b13ad024/CONTRIBUTING.md), [.github/ISSUE_TEMPLATE/bug.yml](https://github.com/paradigmxyz/reth/blob/8fa8b31ea2185f30d57f1ff3fec53e03b13ad024/.github/ISSUE_TEMPLATE/bug.yml), [.github/ISSUE_TEMPLATE/config.yml](https://github.com/paradigmxyz/reth/blob/8fa8b31ea2185f30d57f1ff3fec53e03b13ad024/.github/ISSUE_TEMPLATE/config.yml) | [PVR status](https://api.github.com/repos/paradigmxyz/reth/private-vulnerability-reporting) 06:42:30Z | the security emails in SECURITY.md and the bug template |

These rules would bind any draft (Disclosure and privacy). The sui
program's reward rules ask for a runnable private-testnet proof of concept
and forbid testing on mainnet or public testnets; this round's Safety
forbids any testnet contact, so a sui draft would carry no such proof. The
method's one candidate is model-only, so no report was drafted from it,
and nothing was sent or posted.

## Framing

The pool is a mechanical selection: repository search by topic and stars,
then a lexical crate scan. It is not a judgment of the projects or their
authors. A crate that is unportable or excluded is outside what Tide v0
expresses today; a property that passes says the model passes on 1,000
seeds (one repeated case for the 16 fixed-input tests).
