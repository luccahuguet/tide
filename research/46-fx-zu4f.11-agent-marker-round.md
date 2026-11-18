# 46: fx-zu4f.11, a case-study round on agent-marker Rust repositories

Bead fx-zu4f.11. The protocol is `research/protocols/fx-zu4f.11-protocol.md`
with amendments A1 and A2; the working pool is
`research/protocols/fx-zu4f.11-pool/pool-amended.tsv`.

## Result, negatives first

Arm B judges each candidate fix against five criteria in order (protocol
§4), and records only the first one it fails: **E1** a non-test bug fix
with a publicly resolvable fix/first-parent source pair; **E2** an
observable symptom of this bug, quoted from its commit, PR or issue;
**E3** concurrency-shaped (a race, ordering fault, deadlock or hang between
concurrent activities); **E4** small (an affected path of at most 400
lines) and faithfully expressible under the registered mappings, with no
network, subprocess or external-crate operation on it; **E5** explorable by
Tide's simulator under sequential consistency (**SC**: every memory
operation takes effect in one global order, so weak-memory reorderings are
not explored).

The round has two arms. Arm B screens past fixes whose commit message
matches a keyword regex. Arm A ports each repository's pinned concurrent core
to a Tide model and tests properties the upstream text documents. A
candidate is **caught** only when its buggy model fails a property on a
seed, a recording and two replays of the lowest failing seed give
byte-identical traces, and a fixed model (arm B) or fault-removed control
(arm A) passes all 1,000 seeds; for arm A the same failure must also
reproduce on the real repository code (protocol §5-§7).

**Arm B (retrospective): not caught.** None of the 42 candidates passes
E1-E5, so no model was built and no seed was run.
**Arm A (forward hunt): not caught.** The concurrent core of all 7
repositories is unportable under the registered mappings, so no property was
frozen and no model was built. **No catch, so no case study.** There are
**0 private candidates**: no finding is withheld.

This experiment does not show the following:

- That these repositories are free of concurrency bugs. Arm B looks only at
  fixes whose message matches the registered keyword regex. Arm A stops at
  the first portability failure of each concurrent core.
- Anything about agent-written code compared with other code. The pool is an
  operational selection by commit-message markers. Marker share is not a
  measured fraction of agent-written source.
- What Tide would find unaided. The method supplies each property from
  upstream text; this round froze none and sampled no seed. The simulator
  explores SC interleavings only, and no model ran.
- An independently verified chronology. The examination times below are
  self-attested. The OpenTimestamps proofs show that the protocol and pool
  existed at their attested times. They do not prove that no screening took
  place before.

The admission observations further below are descriptive. They are not
bugs, properties or catches.

## Registration and identities

- Source worktree base: `ea7bb313eac1a66f37b643393bbd5f7dfdeb91e2`.
  Snapshot: 2026-11-18T18:01:51Z. Window: [2025-11-18T18:01:51Z,
  2026-11-18T18:01:51Z).
- Commits, all 2026-11-18 (UTC):
  - `759a196` registration (protocol and pool), 20:54:22Z;
  - `9d797ef` amendment A1, 20:58:13Z;
  - `db130d0` pending OpenTimestamps proofs of the 22 registered files (the
    protocol as amended by A1), 20:58:24Z;
  - `2357834` amendment A2, which supersedes A1 and adds pending proofs of
    the A2 protocol and `pool-amended.tsv`, 21:07:43Z;
  - `584b291` all 24 proofs upgraded to complete Bitcoin attestations
    (block 976487), 21:58:28Z.
- SHA-256 of the protocol: as registered (`759a196`)
  `1698708afd7105fe5707f8ade28027e7c3bf7f31281e9f75254979817275bb5d`;
  with A1 (`9d797ef`, kept as `fx-zu4f.11-protocol.md.a1.ots`)
  `a78bba1d817a2cdafab5b4ebded9283732e9b57fb27f6376b0a95b1e1bd77d83`;
  with A2, current
  `40e3e6612a219924c5f3255ddb58d4004b6ddd48de6c92188e2a4f191709753b`.
- SHA-256 of the pool: registered `pool.tsv` (8 rows)
  `1e64f31da1be9f16c44ccf9f9be2b9ef73172ac9db4facb6c2ee2d443250c90b`;
  amended `pool-amended.tsv` (7 rows)
  `3fbbb2056370a1207271e4dd29276d05f3fb37b579ea82184d193b90221f59c2`.
- GO read: 2026-11-18T21:58:59Z. Arm B screening finished at about
  23:05Z and arm A at 23:08Z on 2026-11-18. The builder handovers are in
  the methods and deviations table below.

**Acquisition intervals (2026-11-18, UTC).** These are sequences of
observations, not one measurement at the snapshot time.

| Observation | Interval | Hash-bearing record |
| --- | --- | --- |
| Repository search, 25 requests | 18:21:38.922946 to 18:23:36.087378 | `queries.jsonl` |
| Repository metadata, 2,139 rows | 18:25:39.656634 to 20:34:10.787669 (`observed_utc`) | `candidates.tsv` (response URL and SHA-256 per row) |
| Commits-only clone and marker count | 18:25:40.322825 to 20:34:12.193916 | `activity-markers.tsv`, `clone-refs.tsv` |
| Last API response / acquisition complete | 20:34:11.384518 / 20:34:15.052773 | `acquisition-summary.json` |
| Provenance normalization / validation | 20:37:03.339010 / 20:46:24.374112 | `acquisition-summary.json`, `validations.json` |
| Post-GO full-history fetch (first examination) | 22:09:20 to 22:09:35 | pool table below |

All 2,164 registration API responses were HTTP 200. Each response's URL,
times and SHA-256 are in `queries.jsonl`.

SHA-256 of the committed provenance files in
`research/protocols/fx-zu4f.11-pool/`:

| File | SHA-256 |
| --- | --- |
| `queries.jsonl` | `bcf00e6d7ef7b32894d9ca64657958b7a42ae1d084c660d2823a03b3305dc4de` |
| `candidates.tsv` | `2dc6a566cdeccb9cb8467b28d8ae1851f4bb37ef5280b037bbe32f06f3a01426` |
| `activity-markers.tsv` | `0012a607207a9da06ac3c35bdf9c36dfb2be6a14ee65ff14ba29cf1de535f6d5` |
| `clone-refs.tsv` | `b1f053877a3ea74ff159675f971f749550cfe9ba939f21435fd8353fc2222e74` |
| `scan-output.tsv` | `3cb1c509c503ec3331637da04e77ece92642c359167e42282fcf91e7dfb91ae4` |
| `b-source-dirs.tsv` | `6af2e2ee2bec08fee11ff196fe6e4d14031e0e84a2c99421c013d4357989759e` |
| `acquisition-summary.json` | `e9631db12587697f2b960e3a5ae8d16ae1eb1a7358278d2b63eeef2bbd007757` |
| `selection.json` | `ecea728af827484a22605191b9da24e04b44ec4ecb5357879afe06a3ac359b4c` |
| `snapshot.json` | `169bf2d020310db1068f8e5f11a042953e1ed9f5d60230d938f173322477d29f` |
| `star-slices.json` | `1e07f263ce346dce74ead310a17c14b11dde43da04696141fa9877b0c5feddc6` |
| `candidate-source-change.json` | `f14666d09300545feea56dbcf729655accb738fff571a2440845ae6e6af9983b` |
| `mechanical-corrections.json` | `e669bd008bc09c80141ecb202e4e1c662f3a9d01a4071612c7ab0b367b56c74f` |
| `tooling.json` | `6b8cfcd8861fe5cba83660855753c070a267e6e5017720d8c88d5fae2346531c` |
| `scan.py` | `3b1d1334b09afbfc250951f156478e9794f330cd869a51e82bb8df87fb6a1bff` |
| `admission-obs.py` | `f36cef2e4572aea0b9a2602407526f02af2b49a5e54d66f655779ff87c38458a` |

**Environment** (`tooling.json`): Python 3.14.10, Git 2.43.0, gh 2.104.0
(2026-11-12), platform `Linux-7.2.6-76070206-generic-x86_64-with-glibc2.44`.
The post-GO screening ran on the same host, with the same Python and Git
versions. The tools were the registered Python standard-library scripts
(`scan.py`, `admission-obs.py`, `count-activity-markers.py`,
`pool-build.py`), Git on the frozen clones, and read-only GitHub REST GETs.
Nothing upstream was built or run, and no Tide model was compiled.

**Exclusion (A2).** Yazelix/nova, registered pool rank 8, is excluded
because it is owner-affiliated: it is the Tide owner's own project. This
says nothing about its code. There is no backfill, because the next
repository by the mechanical rule falls below the registered T floor. All
denominators use 7.

**Disclosed changes of source.** The original per-marker GitHub commit
search was aborted after five secondary-rate-limit refusals (HTTP 403) in 39
requests. Its 1,825 commits and 828 names are an incomplete list that was
never used as the pool. Before registration the candidate source became
repository search sliced by stars (`candidate-source-change.json`). The
pool definition did not change.

**Disclosed pacing limitation.** After GO, 263 REST responses were collected
(all HTTP 200), with at least 1.100054 s between request starts. Two
read-only `git show` batches during that collection may have triggered lazy
blob fetches outside the REST pacer, so the combined Git and REST spacing is
not verified. No refusal was observed.

## Methods and deviations

Registration was at 2026-11-18T20:54:22Z (`759a196`). Rows 1-5 came before
it and no repository had been examined by then. All times are UTC on
2026-11-18.

| When | Change | Reason | What it changed |
| --- | --- | --- | --- |
| 18:20Z, before registration | Candidate source became repository search sliced by stars (`candidate-source-change.json`) | The per-marker commit search hit five HTTP 403 secondary-rate-limit refusals in 39 requests | The source of candidate names. The pool definition did not change |
| 19:44Z, before registration (coordinator directive on the bead) | Arm B narrowing 1: the bead's substring keyword regex became the whole-word regex of protocol §3 | fx-zu4f.10's substring matcher gave 5,076 rows, including embedded words such as `change` and `trace` | A different match set, not a strict subset: the exact registered expression matches whole words, including its listed inflections (`racy`, `racing` and `frozen` are new), and drops embedded-word matches |
| same directive | Arm B narrowing 2: a candidate must touch a registered non-test source directory (`b-source-dirs.tsv`, frozen at each pin) | Same directive | 22 of the 64 matches were excluded |
| before registration (`mechanical-corrections.json`) | Scannability narrowing: every Cargo.toml must be UTF-8 and parse under Python `tomllib`, or the repository is excluded as `unscannable-manifest` | One Cargo.toml could not be parsed strictly (a duplicate value), so its dependencies could not be checked mechanically | The repository filter. This is an input limit, not a Tide restriction |
| before registration (`mechanical-corrections.json`) | Seven scanner and acquisition corrections, among them: test-only parameter masking (a regression test failed first; every earlier scan was invalidated and recomputed under the corrected scanner digest); the date-shallow clone fallback (a full commits-only clone counts the same window); promisor-fetch `noop` negotiation; in-tree symlink resolution; manifests checked first; unresolved-alias handling; metadata `observed_utc` taken from the original request | Mechanical defects found while collecting | Recomputed scan rows and provenance labels. The activity counts, the predicates and the pool rule did not change |
| 20:58Z (`9d797ef`), before stamping and GO | A1: Yazelix/nova (rank 8) is owner-affiliated. It stays in the pool, but a catch there would not count toward the external-catch bar | It is the owner's project | Nothing in the pool |
| 21:07Z (`2357834`), before GO | A2 replaces A1: nova is removed and the working pool has 7 rows, with no backfill | Relayed owner decision | All denominators are 7 |
| about 22:26Z | Builder 1 (Codex) stopped at its context budget, at a checkpoint (`post-go-checkpoint.sha256`). State: admission observations 7/7 done; arm B 42 candidates enumerated with evidence collected, 0 screened; arm A 0/7 screened; no model, property or run; both arms incomplete | Context budget, not a block | Nothing judged |
| before about 22:56Z | Builder 2 (Codex) stopped at its provider's safety filter after rereading the protocol, before resuming | Provider refusal | No screening state changed |
| from about 22:57Z | Builder 3 (Claude Opus) checked both checkpoint manifests (0 mismatches over 897 private files) and screened B 42/42, then A 7/7 | Resumption | Every judgment in this record |
| r1 review fix, after 23:13Z | `db8db55bcd58` re-judged from E3 to E1 (arm B below) | Codex r1 finding B2: missing source cannot support an E3 negative | E1 +1, E3 -1 |
| Opus review fix | Ten rows' appendix `sources` gained the PRs/issues read for them (from their own evidence inventories, except #778 for `5bbb9f6`, which came from `859f691`'s): `2d3f2f8` and `2e1479b` (#54, #62), `476247f` (#982, #1021-#1023), `553d9bc` (#781), `204906a`, `5bbb9f6`, `80e2d19`, `859f691` (root-body links), `8a76bbf` (#801) and `e2d89d9` (#801, #802). All were read in full. For `2d3f2f8`, `2e1479b` and `e2d89d9` the first screening record had wrongly noted no associated PR | Opus review finding O1: the appendix did not list every thread read | No judgment changed |

## Selection counts

A **marker** is an agent trailer in a commit message: `Co-Authored-By:
Claude`, `Generated with Claude Code`, or a `Co-Authored-By:` line naming Codex,
Copilot, Cursor, Aider, Devin, OpenHands, Jules, Amp or Gemini, a `Generated
with` or `Generated by` line naming one of the first five, or a message that
starts with `aider:` (protocol §2 gives the exact
expression). A repository's **marker share** is the fraction of its
default-branch window commits that carry at least one marker, and **T** is
the minimum share a repository needs to enter the pool. The names match as
substrings, so a longer name containing one (for example `Amp`) can match
by accident. A share of 20% means at least one commit in five carries a
marker; it does not mean the code is agent-written.

| Stage | Count |
| --- | ---: |
| Distinct repository-search candidates | 2,139 |
| After the repository filter (Rust, ≥1,000 stars, public, not archived, not a fork) | 2,105 |
| After activity (≥50 window commits) | 1,521 |

| Marker T | After activity and markers | After scan |
| --- | ---: | ---: |
| 50% | 66 | 5 |
| 40% | 98 | 5 |
| 30% | 137 | 5 |
| 25% | 164 | 5 |
| 20% | 208 | 8 |

No T yields ten scan survivors, so T = 20% (the registered floor), with 8
scan survivors. The registered pool has 8 repositories, and the working pool
after A2 has 7.

The scan is lexical (`scan.py`; protocol §2). It does no macro expansion,
complete name resolution or Cargo graph. Passing it does not make a
concurrent core portable.

## Pool and coverage

| Rank | Repository | Stars | Marker share (marked / window commits) | Pinned commit | First examination (UTC, 2026-11-18) | B candidates | B outcome | A outcome |
| ---: | --- | ---: | --- | --- | --- | ---: | --- | --- |
| 1 | RedisJSON/RedisJSON | 3943 | 22% (25 / 114) | `65337b872b1e` | 22:09:20 | 4 | none eligible | unportable |
| 2 | TimelyDataflow/differential-dataflow | 3040 | 64% (203 / 318) | `e7c24703f4b1` | 22:09:22 | 13 | none eligible | unportable |
| 3 | mitsuhiko/insta | 2988 | 64% (49 / 77) | `0e17c7a6ed65` | 22:09:26 | 1 | none eligible | unportable |
| 4 | anthropics/claudes-c-compiler | 2837 | 65% (2,575 / 3,982) | `6f1b99acb2f4` | 22:09:28 | 11 | none eligible | unportable |
| 5 | jamesondh/chimera-vocal-synth | 2719 | 91% (245 / 270) | `eeb0c1940d2d` | 22:09:31 | 3 | none eligible | unportable |
| 6 | libraries/gameboy | 1429 | 23% (27 / 116) | `17e56feafdd4` | 22:09:33 | 0 | no candidate | unportable |
| 7 | confeden/Antigravity | 1422 | 74% (80 / 108) | `4c9711805fd8` | 22:09:35 | 10 | none eligible | unportable |

Marker shares are from `activity-markers.tsv`. Ranks 1 and 6 sit just above
the 20% floor. The first examination was the registered full-history
acquisition for arm B.

## Arm B: retrospective fixes

Full-history blob-less clones were fetched from the registered head and tag
object IDs only: 10,853 commits, none unavailable. The whole-word keyword
regex (protocol §3) matched 64 commits. 22 of them failed the registered
source-directory gate. The other **42 candidates** are below, in pool order
and then SHA order. One of them (`db8db55`) matched only an inflection the
bead's substring regex lacked (`racing`, which does not contain `race`); it
fails E1.

| Measure | Unit | Count |
| --- | --- | ---: |
| Targeted / screened / unresolved | repositories | 7 / 7 / 0 |
| Keyword matches / path-gate exclusions / candidates | commits | 64 / 22 / 42 |
| Screened / unresolved | candidate rows | 42 / 0 |
| Distinct matched commit IDs | commits | 42 |
| Duplicate-site rows / deduplicated candidate units | rows | 3 / 39 |
| First failure E1 / E2 / E3 / E4 / E5, raw | candidate rows (42) | 16 / 4 / 16 / 6 / 0 |
| First failure E1 / E2 / E3 / E4 / E5, deduplicated | candidate units (39) | 16 / 2 / 16 / 5 / 0 |
| Unportable (first failure E4) | rows / units | 6 / 5 |
| Eligible / modeled / properties / caught | units | 0 / 0 / 0 / 0 |

A duplicate-site row repeats the production change of another row at the
same bug site (a backport, squash or merge). It is linked to that row and
left out of the deduplicated counts. A candidate unit is a screening unit,
not an established bug fix: 16 of the 39 fail E1, meaning they are not
non-test bug fixes with a public source pair. **Unportable** means a row
passed E1-E3 and then failed E4: its affected path is too large, or needs an
operation with no registered faithful mapping. No row was unresolved. Every
registered object was acquired and every inventoried source was read.

**Evidence appendix.** `research/protocols/fx-zu4f.11-record/b-candidates.tsv`
(SHA-256 `20b19ebff79e6559af0848a72c2d96bfb80cd84a3e0799106dbe70567defc45e`)
has one row for each of the 42 commits. Each row gives:

- source links: the commit, the first-parent tree and the PRs/issues read;
- the status of each of E1-E5 (pass, tentative pass, first failure, or not
  assessed), with any additional concerns;
- the exact E2 symptom quote and its source identity wherever E2 passed
  (the commit message at that commit, or an issue body or comment with its
  author and time);
- for E2 failures, the context quote that was not accepted as this row's
  symptom;
- for every E4 row, the changed files and functions, what they race or wait
  against, the trigger, and the line count, marked INCOMPLETE wherever the
  union was not counted.

The appendix holds only public commit messages, public PR/issue text and
the judgments.

These narrowings decided the rows:

- **E1, 16 rows.** Features, refactors and doc-only changes; fixes in code
  new to the same commit or PR; one change its own message says did not fix
  the bug; one fix whose repair lies in a file absent from the public
  repository (`db8db55`, below). Many keyword hits are design notes (a
  fairness discipline, a designed provider race), idioms ("leave it
  hanging") or names (`Freeze`, `--param=allow-store-data-races`).
- **E2, 4 rows.** The RedisJSON shared-string-cache fix (three rows, one
  site) addressed a race stated only as potential. Its PR attributes the
  observed corruption to another cause: "Bug persists with thread-safe cache
  enabled" (#1437). One claudes-c-compiler row states its encoding fixes
  with no observed failure.
- **The Antigravity row `db8db55`, E1.** Its message names two faults: a
  bench escalated by tunnels already in flight, and a relay that nothing
  re-checked. Both are repaired in `src/relay.rs`. No registered ref
  contains that file: `git log --all -- src/relay.rs` is empty on the full
  history, and `proxy.rs` includes it only under `cfg(relay)`
  (`#[path = "relay.rs"] mod relay`). The public production diff adds
  warm-loop calls to `proxy::probe_relay` and `proxy::relay_is_benched`,
  which in the public build are a no-op and `false`, plus a version bump.
  The answer-cache guard is test-only. So no public fix/first-parent pair
  repairs this bug, and E1 fails. The row was first recorded as an E3
  failure of the visible timer change. The r1 review rejected that, because
  missing source cannot make a completed E3 negative. The row is not
  unresolved: the history was acquired in full, and the file is absent from
  it, not unavailable.
- **E3, 16 rows.** Most are deterministic single-threaded faults: compiled
  kernels hanging from miscompilation, dataflow non-convergence, UI or state
  logic.
- **E4, 6 rows.** The two differential-dataflow bug sites (#183/#184 and
  #801/#802) are races and hangs (#801's E3 pass is tentative) whose paths
  are timely dataflow operators:
  frontiers, capabilities, scheduling and exchange, all in an external crate
  with no registered mapping. That external-crate boundary decides E4. The
  whole files involved in #801 are `join.rs` (348 nonblank non-comment
  lines) and `spine_fueled.rs` (458) at the first parent. The function-level
  path union was not counted, so this record does not claim it exceeds 400
  lines. Of the three Antigravity rows, two are tagged capability-only,
  provisionally (below).
  The third (`8566ded`) races the proxy's idle timer with the client's Go
  `IdleConnTimeout` over TCP. That path also involves an external peer
  process and OS RST semantics.

No row reached E5. The deciding constraints were E1 and E3 for most rows,
and E4's external-crate, network and subprocess bans for the rest. No
recorded first failure was attributed to a missing Tide v0 primitive. Later
criteria were not assessed for most rows.

| Rank | Commit | First parent | First failure | Duplicate of | Capability | Reason |
| ---: | --- | --- | --- | --- | --- | --- |
| 1 | `23bfd3aaa96f` | `54fd1b2f9697` | E1 |  |  | investigative change, said not to fix the bug; reverted in #1437 |
| 1 | `6bd3fc9f24f9` | `00dafd96609f` | E2 |  |  | cache race stated only as potential; the observed corruption persisted with this change and was attributed elsewhere (#1437) |
| 1 | `b5ca63cb63f5` | `3344f2506fa6` | E2 | `6bd3fc9f24f9` |  | backport of #1437 (#1440) |
| 1 | `b6d4e7b15342` | `775f56e943d3` | E2 | `6bd3fc9f24f9` |  | squash of #1437 |
| 2 | `204906a15b00` | `45e2e315bdce` | E1 |  |  | refactor; "starve" is a fairness design note |
| 2 | `476247f7028b` | `68a11a461a16` | E3 |  |  | tick hang is a dataflow non-convergence from reading uncompacted history, not a race |
| 2 | `553d9bc121f6` | `77043652d4d8` | E1 |  |  | docs and debug asserts, no behavior change |
| 2 | `59dfd3df852b` | `972f66fda6bc` | E4 |  |  | #801 hang/wrong output; path is timely operators (external crate) against the shared Spine trace |
| 2 | `5bbb9f68854b` | `79911ecb5b23` | E1 |  |  | feature; the folded fix repairs code new in the same PR |
| 2 | `80e2d19df7b5` | `fb8c6d598b48` | E1 |  |  | new server crate; the exit hang fix is in code new to this commit |
| 2 | `859f6912a6ad` | `aa8745f93ea8` | E1 |  |  | purely additive feature squash (3101+/0-) |
| 2 | `8a76bbff6a8b` | `f521267310b9` | E1 |  |  | deletes a doc comment only |
| 2 | `984162659a4a` | `15889bf83445` | E4 |  |  | #183 nondeterministic triangles; path is timely frontiers, capabilities and worker exchange (external crate) |
| 2 | `a6eee82b10be` | `ba6d7e166f03` | E1 |  |  | feature removal; matches the name Freeze |
| 2 | `e1edeb3d665b` | `92134466b01f` | E4 | `984162659a4a` |  | merge of #184 |
| 2 | `e2d89d96f4a8` | `59dfd3df852b` | E1 |  |  | restores a removed function and adds tests; the fix is 59dfd3d |
| 2 | `f16c6cff2b96` | `b185de363eab` | E3 |  |  | tick hang is unconsolidated feedback, a deterministic dataflow fault |
| 3 | `02837742793b` | `f2783e5339eb` | E1 |  |  | design reversal of snapshot metadata; "leave it hanging" is an idiom |
| 4 | `0de4307824a4` | `f3c369add8c0` | E3 |  |  | compiled kernel hang from a missing relocation; single-threaded compiler fault |
| 4 | `100560cd639a` | `eaf6cdcd49dd` | E3 |  |  | compiler fixes; "starvation" is an inline budget |
| 4 | `1d26685da430` | `bbbaf3434ba8` | E3 |  |  | compiler infinite loop in struct init; single-threaded |
| 4 | `6e3663db5904` | `8d29cdf3a6dc` | E3 |  |  | compiled kernel hang from unresolved branches |
| 4 | `8da492f32d04` | `fa36670e77b8` | E1 |  |  | feature; "race" names an external test flake |
| 4 | `92072a3233c3` | `510ec8637468` | E3 |  |  | compiled kernel hang from SSA renaming |
| 4 | `a55065e40bdb` | `b98fd6eed978` | E1 |  |  | task-lock commit; the code edits are not described as a fix |
| 4 | `a60ec2d1c9ab` | `5a1987e32118` | E3 |  |  | flag filtering; the keyword is a GCC flag name |
| 4 | `b5e6a2f127b2` | `540ca0e0fc1b` | E3 |  |  | peephole wrong value; "race" names an external build flake |
| 4 | `d59e32974399` | `296f88fe383a` | E2 |  |  | mostly a feature; the folded encoding fixes state no observed failure |
| 4 | `dc2c5d5db07a` | `33daedfe2d80` | E3 |  |  | compiled kernel hang from relocation state |
| 5 | `2d3f2f8bc8d4` | `7af1e4a805e5` | E3 |  |  | gate stays open and audio-thread allocation; single-threaded faults |
| 5 | `2e1479b0534f` | `1dbdf51cde01` | E1 |  |  | allocation-free refactor; "race" is a rejected design |
| 5 | `30d9d54e4d27` | `657af3d34a6d` | E3 |  |  | stuck note is single-threaded UI event ordering |
| 7 | `12a36c940b10` | `5ab40fac5439` | E4 |  | cap:net | provider-race winner passed with a long TTL; every racing thread does network DNS |
| 7 | `5ab40fac5439` | `71f45cfa4f2f` | E3 |  |  | NRPT nameserver list construction; the timing lies in the OS resolver |
| 7 | `7c93f613d7bf` | `821365267b04` | E1 |  |  | policy change; designed provider race |
| 7 | `8566deddfd99` | `834e628a955b` | E4 |  |  | idle-timer race with the client; TCP, an external peer process and OS RST semantics |
| 7 | `9c52fb4559b9` | `3219781c6469` | E3 |  |  | socket read timeout left set; deterministic |
| 7 | `af614c3b4b60` | `71d053de019c` | E3 |  |  | GUI feature; the off-switch fault is state classification |
| 7 | `bc5222419d58` | `8dcb27171428` | E1 |  |  | feature; designed provider race |
| 7 | `db8db55bcd58` | `fa7acd0ba7d2` | E1 |  |  | both stated faults are repaired in src/relay.rs, which no registered ref contains; the public diff only calls no-op stubs (re-judged from E3 in r1) |
| 7 | `df7e31eebb54` | `7b824ea868fe` | E3 |  |  | close_notify handling: the server had already sent its end signal and the tunnel did not act on it, a single-threaded protocol fault (unlike fbcbbb8, where the other party itself hangs) |
| 7 | `fbcbbb8b47d0` | `db8db55bcd58` | E4 |  | cap:proc | eternal hang waiting on a hung PowerShell child |

## Capability-only failures

For these rows, the recorded E4 failure is a network or subprocess
operation on the path. No other restriction was found in the part of the
path that was counted. Both tags are provisional, because neither path
union was counted in full. They are listed for fx-zu4f.13, a later round
that re-screens rows which failed only for a Proc or Net capability Tide has
since landed. It should count each union before relying on the tag.

| Repository | Commit | Tag | Path and size | Note |
| --- | --- | --- | --- | --- |
| confeden/Antigravity | `12a36c940b10` (parent `5ab40fac5439`) | cap:net | `resolvers.rs` resolve_best 131, ask_provider 11, classify 35, remember_choice 7, forget_choice 7; `dns_forwarder.rs` relay 10, run 36 (about 237 lines) | Provisional: the dns_client transport helpers were not counted. The racing threads send UDP DNS queries, so fx-zu4f.13 should check that the landed network capability covers UDP DNS, not only HTTP (`Net::get`). Provider threads race through mpsc `recv_timeout`; a passthrough winner's answer kept the provider TTL ("one lost race pinned Windows to a blocked address for 53 minutes"). |
| confeden/Antigravity | `fbcbbb8b47d0` (parent `db8db55bcd58`) | cap:proc | `utils.rs` powershell 5, no_window 5; their `dns.rs` callers were not counted | `Command::output()` waited forever on a PowerShell/WMI child ("an eternal hang"). E3 passes tentatively, treating a parent blocked on a hung child as blocking between concurrent activities. A second folded cause is sequential network slowness and is not tagged. |

## Arm A: forward hunt at the pinned commits

| Measure | Count |
| --- | ---: |
| Targeted / screened / unresolved | 7 / 7 / 0 |
| Unportable / portable / modeled | 7 / 0 / 0 |
| Properties frozen / caught | 0 / 0 |

| Rank | Concurrent core at the pin | First portability failure |
| ---: | --- | --- |
| 1 | Module-global atomics (`json_path.rs:36`, `backward.rs:35`, `rdb_persistence.rs:61-70`, `auto_create.rs:88`), written by configuration and ASM/RDB callbacks and read by command handlers. No thread, channel or lock is created in production source. | Every root is entered through Redis Module API callbacks over FFI, through the external redis-module crate. |
| 2 | The interactive server's TCP session threads and mpsc command channels feed timely workers (`interactive/server/src/main.rs:136,193,232,244`). The library operators run inside timely. | External crate (timely) and network on the path. |
| 3 | `once_cell::sync::Lazy<Mutex<..>>` caches (`env.rs:15,17`, `runtime.rs:29-33`, `glob.rs:21`). Concurrency comes from the caller's test threads. | External crate (`Lazy`) on every shared state, a subprocess (`cargo metadata`, `env.rs:50,494`) and filesystem operations with no mapping. |
| 4 | One `AtomicU64` counter (`src/common/temp_files.rs:12,28`; 51 lines). No thread, channel or lock. | `std::process::id`, `std::env::temp_dir` and `fs::remove_file` on the path have no mapping. No concurrent activity exists to port. |
| 5 | Three `thread::scope` work queues (an `AtomicUsize` index, `Mutex<Vec>` results) in `chimera-cli/src/compare.rs:1067` and `compare/tsp.rs:106,871`. | The workers do filesystem and stdout I/O, which has no registered mapping. |
| 6 | The APU pushes samples into `Arc<Mutex<Vec<(f32,f32)>>>` (`apu.rs:755,794`), consumed by the audio callback (`main.rs:143`). | The consumer is a callback of the external cpal crate (an OS audio thread). |
| 7 | DNS forwarder, TLS proxy and route threads (spawn/scope in 10 files). | Network, subprocess and rustls (external crate) on the path. |

No arm-A repository is capability-only. The core of each fails for an
external crate or for operations with no mapping, alongside or instead of
network or subprocess operations. No arm-A failure rests on size, because
no union was counted in full.

**Scope, map and count per repository.** The method was the same for all
seven. Roots came from a lexical search of the pinned checkout for thread
spawn/scope, Mutex, RwLock, atomics and mpsc, with test directories
excluded; the search matches scan.py's positive evidence. Calls and shared
state were then followed by reading the source at the cited lines.
Enumeration stopped at the first boundary that was independently decisive
(an external crate, FFI, or an operation with no mapping). So **no
repository's union was counted in full**, and every count below is a
partial file or function count, not the §6 union. Every row stopped before
a property freeze. Line counts are nonblank non-comment lines.

| Rank | Roots (pinned commit) | Call/state map enumerated | Not enumerated | Count | Stopping boundary |
| ---: | --- | --- | --- | --- | --- |
| 1 | `json_path.rs:36` HIDE_USER_DATA_FROM_LOG; `backward.rs:35` LEGACY_RDB_DEPTH_LIMIT; `rdb_persistence.rs:61-70` four atomics; `auto_create.rs:88` AUTO_CREATE_DEEP_PATHS (`65337b872b1e`) | Writers: configuration and ASM/RDB callbacks. Readers: command handlers. The only lock (`ivalue_manager.rs:876`) is test-only | The individual handler and callback functions and their helpers | None | Every root is entered through Redis Module API callbacks over FFI (external redis-module crate) |
| 2 | `interactive/server/src/main.rs:136` timely::execute; `:193,232,244` thread::spawn; mpsc/atomics/Mutex in `loop_.rs`, `cmd.rs`; `diagnostics/src/server.rs`; `columnar/trace/spill.rs`; `interactive/src/lambda.rs` (`e7c24703f4b1`) | TCP session threads and mpsc command channels feeding timely workers | The diagnostics, spill and lambda roots; every function-level path; the timely internals | Whole files main.rs 394, loop_.rs 537, cmd.rs 1,066 (not a union) | timely (external crate) and std::net on the server core |
| 3 | `insta/src/env.rs:15,17`, `runtime.rs:29-33`, `glob.rs:21` Lazy<Mutex> statics; `settings.rs:718` a !Send marker (`0e17c7a6ed65`) | Concurrency comes from the caller's test threads. The assertion path reads and writes snapshot files (`runtime.rs:366,508,514,556,574`). WORKSPACES runs `cargo metadata` (`env.rs:50,494`) | The assertion functions' helpers | None | once_cell `Lazy` (external crate) on every shared state; subprocess and fs on the path |
| 4 | `src/common/temp_files.rs:12` TEMP_COUNTER, `:28` fetch_add in make_temp_path (`6f1b99acb2f4`) | The only std synchronization in production source. `Instruction::Atomic*` names IR variants, not Rust synchronization. No thread, scope, channel or lock exists | The callers of make_temp_path | temp_files.rs 51 (whole file) | No concurrent activity to port; std::process::id, env::temp_dir and fs::remove_file have no mapping |
| 5 | `chimera-cli/src/compare.rs:1067-1088`; `compare/tsp.rs:106-130, 871-890`: thread::scope work queues with an AtomicUsize index and Mutex<Vec> results (`eeb0c1940d2d`) | The workers fetch an index, run one item and push its result. compare_song (`compare.rs:829`) reads CSV/WAV files and prints. The tsp.rs pipeline runs the external `mfa` aligner as a subprocess (`tsp.rs:231`) | say(), the synthesis engine, the tsp.rs worker bodies | compare_song 168 alone (not a union) | fs and stdout I/O in the workers, with no mapping |
| 6 | `src/apu.rs:755,776,794` Arc<Mutex<Vec>> producer; `src/main.rs:143` hand-off; `src/rng.rs:10,25-28` SEED AtomicU64 CAS loop (`17e56feafdd4`) | The APU pushes samples, and the consumer is the cpal output-stream callback on an OS audio thread. No std thread is created | rng.rs's callers; the producer's call graph | None | cpal callback (external crate) as the consumer root |
| 7 | spawn/scope in 10 files (`dns_forwarder.rs`, `proxy.rs`, `resolvers.rs`, `routes.rs`, `loopback.rs`, `dns_route.rs`, `ops.rs`, `watchdog.rs`, `utils.rs`, `patch_binary.rs`); Mutex/atomics/mpsc across `dns*.rs`, `gate.rs`, `group.rs`, `health.rs`, `net.rs`, `proxy.rs`, `resolvers.rs`, `routes.rs`, `settings.rs`, `upstream.rs` (`4c9711805fd8`) | The threads do UDP/TCP/TLS I/O (15 files, rustls) and drive PowerShell/netsh/tasklist subprocesses (14 files) | Every function-level path | None at the pin (resolvers.rs is 1,161 lines at the earlier commit 834e628, not the pin) | std::net, subprocess and rustls (external crate) on the core |

## Admission observations (descriptive)

`admission-obs.py` ran on all 7 pinned checkouts. These counts are lexical
observations of syntax that Tide admission would make explicit. They are not
execution counts, compiler diagnoses, bugs or catches. Aliases and shadowing
limit the heuristic, and the spawn-discard count is not a semantic lower
bound.

| Rank | std::fs imports/references | std::net | std::process | std::env | clock (SystemTime/Instant) | thread::spawn handles discarded / unresolved |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0/0 | 0/0 | 1/3 | 1/1 | 1/2 | 0 / 0 |
| 2 | 6/12 | 3/6 | 0/3 | 0/79 | 8/54 | 1 / 4 |
| 3 | 6/35 | 0/0 | 4/17 | 9/46 | 1/1 | 0 / 0 |
| 4 | 0/77 | 0/0 | 1/12 | 0/36 | 0/13 | 0 / 0 |
| 5 | 0/33 | 0/0 | 1/5 | 0/7 | 1/6 | 0 / 0 |
| 6 | 1/15 | 0/0 | 0/5 | 0/0 | 1/11 | 0 / 0 |
| 7 | 20/170 | 43/261 | 16/108 | 4/87 | 22/222 | 17 / 3 |
| Total | 33/342 | 46/267 | 23/153 | 14/256 | 34/309 | 18 / 7 |

## AI-report policies

All seven were read on 2026-11-18. The documents were fetched between
22:19:53Z and 22:21:02Z and classified at 23:06:11Z. The documents read
were the root README/AGENTS, CONTRIBUTING/SECURITY and `.github` issue/PR
templates of each default-branch tree current at fetch time. For ranks 1-4,
6 and 7 that tree is the pool pin. For rank 5 it is `c6147d019889`, a later
commit than the pin `eeb0c1940d2d`, because the default branch had moved.
Each document was searched for AI, LLM, agent, bot, generated and automated
terms, and every matching passage was read in context. Documents with no
match were not otherwise read line by line. Absence claims cover only these
documents and terms, not every policy a repository may have.

| Rank | Revision read | Documents (revision-specific URLs) | Fetched (UTC) | Rejects AI-generated reports? |
| ---: | --- | --- | --- | --- |
| 1 | `65337b872b1e` | [`AGENTS.md`](https://github.com/RedisJSON/RedisJSON/blob/65337b872b1eebc93ca868c00d663082421ea8ad/AGENTS.md)<br>[`CONTRIBUTING.md`](https://github.com/RedisJSON/RedisJSON/blob/65337b872b1eebc93ca868c00d663082421ea8ad/CONTRIBUTING.md)<br>[`README.md`](https://github.com/RedisJSON/RedisJSON/blob/65337b872b1eebc93ca868c00d663082421ea8ad/README.md)<br>[`SECURITY.md`](https://github.com/RedisJSON/RedisJSON/blob/65337b872b1eebc93ca868c00d663082421ea8ad/SECURITY.md) | 22:19:53 | No rejection rule found; AGENTS.md gives guidelines for AI coding agents |
| 2 | `e7c24703f4b1` | [`CONTRIBUTING.md`](https://github.com/TimelyDataflow/differential-dataflow/blob/e7c24703f4b1935cd6c99779ed0c140b0fea8ac6/CONTRIBUTING.md)<br>[`README.md`](https://github.com/TimelyDataflow/differential-dataflow/blob/e7c24703f4b1935cd6c99779ed0c140b0fea8ac6/README.md) | 22:20:11 | No rule found |
| 3 | `0e17c7a6ed65` | [`.github/ISSUE_TEMPLATE/1-bug.yml`](https://github.com/mitsuhiko/insta/blob/0e17c7a6ed65b73def1101813987a053269ef4a5/.github/ISSUE_TEMPLATE/1-bug.yml)<br>[`.github/ISSUE_TEMPLATE/2-feature.md`](https://github.com/mitsuhiko/insta/blob/0e17c7a6ed65b73def1101813987a053269ef4a5/.github/ISSUE_TEMPLATE/2-feature.md)<br>[`CONTRIBUTING.md`](https://github.com/mitsuhiko/insta/blob/0e17c7a6ed65b73def1101813987a053269ef4a5/CONTRIBUTING.md)<br>[`README.md`](https://github.com/mitsuhiko/insta/blob/0e17c7a6ed65b73def1101813987a053269ef4a5/README.md) | 22:20:19 | No rule found |
| 4 | `6f1b99acb2f4` | [`README.md`](https://github.com/anthropics/claudes-c-compiler/blob/6f1b99acb2f4ec2414592136c2009fe7713deec3/README.md) | 22:20:34 | No rule found; the README states Claude wrote the code |
| 5 | `c6147d019889` | [`.github/pull_request_template.md`](https://github.com/jamesondh/chimera-vocal-synth/blob/c6147d0198890475dbe8a65a0bb8f56240c539b7/.github/pull_request_template.md)<br>[`AGENTS.md`](https://github.com/jamesondh/chimera-vocal-synth/blob/c6147d0198890475dbe8a65a0bb8f56240c539b7/AGENTS.md)<br>[`CONTRIBUTING.md`](https://github.com/jamesondh/chimera-vocal-synth/blob/c6147d0198890475dbe8a65a0bb8f56240c539b7/CONTRIBUTING.md)<br>[`README.md`](https://github.com/jamesondh/chimera-vocal-synth/blob/c6147d0198890475dbe8a65a0bb8f56240c539b7/README.md) | 22:20:44 | No rejection rule found; CONTRIBUTING.md welcomes agent-written PRs with a disclosure line |
| 6 | `17e56feafdd4` | [`README.md`](https://github.com/libraries/gameboy/blob/17e56feafdd412d5c8e5fe1471f112ec4b40322c/README.md) | 22:20:48 | No rule found |
| 7 | `4c9711805fd8` | [`README.md`](https://github.com/confeden/Antigravity/blob/4c9711805fd8a1c868502e22df67f8e85c659c4c/README.md) | 22:21:02 | No rule found |

There are no finds, so no report was drafted and nothing was sent.

## Framing

The pool is selected by commit-message agent markers. That is the
experiment's operational selection, not a judgment of the authors or of the
code. Agent-written code is Tide's target; this round claims nothing about
its quality.
