# Search record: expressibility-first concurrency round (fx-zu4f.10)

Bead fx-zu4f.10, the round after [research/41](41-qin-dataset-search.md),
run under the relayed owner decision "until we found, our search continues".
It has two arms over one frozen pool of 24 popular crates.io crates: arm B
judges historical fixes, and arm A tries to port each crate's concurrent core
and look for new bugs. Registration (protocol and pool) is by the step-1
builder, Codex gpt-6.1-sol. Arm B was judged by builders 1-10: builder 1
(Codex) enumerated, builder 2 (Codex) ran the path pass and the first serde
rows, and builders 3-10 were Claude Opus 5.5. Arm A was done by builder A1,
Claude Opus 5.5. This record was written by Claude Opus 5.5 on 2026-11-18
and corrected after the Codex review round 1 (r1) by the record fixers,
Claude Opus 5.5, on 2026-11-18/19, and after the Opus review by fixer 12,
Claude Opus 5.5, on 2026-11-19. Its counts describe this pool and
selection only. They are not a catch rate.

Terms used throughout: E1-E5 are protocol §4's five arm-B eligibility
criteria, applied in order (E1 a non-test bug fix in this crate; E2 a stated
symptom of this bug; E3 concurrency-shaped; E4 small and faithfully
expressible in Tide v0; E5 explorable by the simulator). A row's first
failing criterion is its counted outcome. SC is sequential consistency, the
only memory model the simulator explores. tide-lead is the agent session
that coordinates Tide's builders; tide-desk is the owner's contact session,
whose decisions reach builders through tide-lead ("desk decision"); RC is
Tide's release candidate.

## What this round does not show

- **Neither arm says anything about whether these crates are correct.** No
  model was built and nothing was run, so there are no supplied-property
  limits, no finite SC seed sampling, no model gaps and no predicted seed
  failure. The negatives are screening outcomes under the registered
  criteria, not simulator results.
- **Arm B covers only known, fixed bugs.** Every arm-B row is a commit already
  in public upstream history; even a catch there would have replayed a bug
  someone had already found and fixed. Arm A was the only route to an
  independently reproduced pinned-version failure, and it reached no model.
- **The chronology is self-attested.** The OpenTimestamps proofs show only
  that the protocol and pool existed by Bitcoin block 976462, not that no
  screening happened earlier. Examination times come from the agents' own
  `date -u` and file logs.
- **The arm-B decisions are hand judgments for 1,220 rows**, made by
  builder 2 (Codex; 11 serde rows) and eight Opus builders under context
  budgets and then corrected in r1.
  Before r1, up to 182 rows that failed at E2 or later may have been settled
  from abridged readings, against protocol §3: 41 marked as such and 141
  possible. All 182 were re-read in full in r1, in two passes (deviation 8):
  a reader that printed human-written text whole but filtered bot reports
  and dropped HTML comments and query-string URLs, then a second reader
  that printed every portion the first omitted, whole, for the 80 rows that
  had any; B10.010's pasted trace log was read line by line. 27 of the 182
  had a missing linked report fetched first. After the Opus review the
  same link-gap check ran over all 306 rows that fail at E2 or later; it
  found two more closing references (B02.277, B09.092), whose reports were
  fetched and read, and both rows stay E3. Four rows changed from E2 to
  E3, and B15.047 changed from E1 to E3. No row passed E4.
- **Arm A's core maps are lexical**, traced by hand from a lexical search of
  the verified archives in 4.5 minutes for ten crates, not compiler
  reachability. Each crate stops at its first failing criterion; later
  concerns are "additional" and were not checked exhaustively.
- **One arm-B row is unresolved, not negative.** B23.128 lacks required
  evidence (see [Unresolved row](#unresolved-row)).
- **The scan's limits carry over** (protocol §2): lexical detection, no macro
  expansion, no cfg selection, no cross-module name resolution. The pool is
  "what the registered scan kept", not "every std-only concurrency crate".
- This round makes no claim that Tide found anything, aided or unaided, and
  gives no discovery rate.

The registered protocol is
[fx-zu4f.10-protocol.md](protocols/fx-zu4f.10-protocol.md). The pool files
are in [fx-zu4f.10-pool/](protocols/fx-zu4f.10-pool/). The arm-B per-row
ledger is
[fx-zu4f.10-record/arm-b-ledger.tsv](protocols/fx-zu4f.10-record/arm-b-ledger.tsv).

## Verdicts

- **Arm B (retrospective): NOT CAUGHT.** Every one of the 5,076 keyword-match
  rows was screened. 5,075 rows stop at a first failure from E1 to E4. One row,
  B23.128, is unresolved. No row passed E1-E5, so no row was eligible. No
  property was written, no model was built, and nothing was run.
- **Arm A (forward hunt): NOT CAUGHT.** All 10 targeted crates were screened.
  8 have a concurrent core that cannot be ported, and 2 have no concurrent
  core. No core was portable, so no property was frozen, no model was built,
  nothing was run, and there was no candidate and no real-crate reproduction.
- No catch means no case study (protocol §8). Neither arm found any new or
  private result, so nothing is held back from this record.

## Counts

Selection (registered, unchanged since 63eb32e):

| Stage | Count |
| --- | ---: |
| crates.io snapshot (top 1,000 by all-time downloads) | 1,000 |
| GitHub repository with 1,000 or more stars | 369 |
| Kept by the mechanical scan | 24 |
| Pool (fewer than 30 kept, so all of them) | 24 |

Arm B:

| Measure | Count |
| --- | ---: |
| Crates targeted / screened | 24 / 24 |
| Distinct repositories cloned | 21 |
| Commits enumerated (all fetched refs) | 50,868 |
| Keyword-matching unique repository commits | 4,524 |
| Crate-match rows (memberships; shared repositories count once per crate) | 5,076 |
| of which merge commits (judged against the first parent) | 238 rows, 218 commits |
| Root commits among matches | 0 |
| Rows screened to a first failure | 5,075 |
| Unresolved | 1 |
| Rows linked as the same bug site as an earlier row (`duplicate_of`) | 78 |
| Rows passing E1 | 306 (270 distinct bug sites after removing 36 duplicate rows) |
| Rows passing E2 (an E2 quote recorded) | 197 |
| Rows passing E3 | 10 |
| Eligible (E1-E5 all pass) | 0 |
| Modeled / properties / runs | 0 / 0 / 0 |
| Caught | 0: not caught |

Arm-B first failures, all 5,076 rows:

| First failure | Rows |
| --- | ---: |
| E1, path pass (no own non-test source, test-, doc-, bench-, example- or CI-only) | 3,779 |
| E1, declared comment/whitespace class | 76 |
| E1, hand judgment | 914 |
| E2 | 109 |
| E3 | 187 |
| E4 | 10 |
| E5 | 0 |
| Unresolved | 1 |
| **Total** | **5,076** |

The 3,779 path failures break down as: no own non-test source (a workspace
neighbour or configuration at most) 2,715; doc-only 599; test, doc, bench,
example or CI-only 289; test-only 176. Of the 78 duplicate rows, 42 fail at
E1, 9 at E2, 26 at E3 and 1 at E4.

Arm A:

| Measure | Count |
| --- | ---: |
| Crates targeted (pool ranks 1-10) / screened | 10 / 10 |
| Unresolved | 0 |
| Concurrent core unportable | 8 |
| No concurrent core (E3 applied to the core) | 2 |
| Portable cores | 0 |
| Properties frozen | 0 |
| Models / seed runs / candidates | 0 / 0 / 0 |
| Controls / real-crate reproductions | 0 / 0 |
| Caught | 0: not caught |

## Registration, proofs and chronology

- Base commit: `a28b4df0b49731a20e9f181ab788fc5ce498a53a`. The frozen
  criteria were committed in that ledger commit at 16:30:03Z, before the
  step-1 builder started.
- Registration commit: `63eb32eb31a593354d520474d21416bf248492d4`
  (2026-11-18T17:03:02Z), with 18 files. PROTOCOL-READY was written at
  17:04:11.971095Z.
- OpenTimestamps proofs, one `.ots` per registered file:
  `8ef20c7d2d983a1d32f66942bf641eccb58309a1` (pending, 17:05:29Z). They were
  upgraded in `979ca1900e2ebdcb7e53a4790fcb5b2805054557` (the commit is dated
  19:32:11Z; the bead records the upgrade at 19:31:51Z). All 18 are anchored
  in Bitcoin block 976462.
- GO: tide-lead's bead comment at 19:32 UTC. The GO file `target/work/go.md`
  is git-ignored.
- Metadata acquisition: crates.io snapshot 16:32:08.021571Z-16:32:18.626595Z,
  10 pages. GitHub repository metadata from 16:32:55.856567Z to
  16:40:04.555880Z. Archive downloads from 16:41:56.475915Z to
  16:48:43.193287Z. Per-response digests are in the committed
  `snapshot-manifest.json`, `stars.tsv` and `archive-hashes.tsv`.
- First post-GO per-crate examination (arm B): 2026-11-18T19:35:09.950593Z,
  the read-only clone of serde-rs/serde. All 21 clones ran from 19:35:09Z to
  19:36:19.815376Z.
- Arm B screening, from the first evidence request to the last decision:
  19:47:50Z-21:50Z. The last REST request was at 21:47:53Z and the last
  GraphQL query at 21:48:08Z.
- r1 corrections: 2026-11-18T22:16Z-2026-11-19T03:41Z for new evidence
  requests (REST GETs through `b-evidence.py`'s flock; the retained ones are
  at least 2.09 s apart), and record edits by fixers 6-11 until commit
  `0b12748` at 2026-11-19T06:23:33Z. After the Opus review, fixer 12 made
  four more REST GETs (rustls issue 1992 and bindgen issue 2498 with their
  comments, 06:40:02Z-06:40:08Z, 2.1 s apart) and edited the record and
  ledger from 06:40Z until the amended commit.
- Arm A: started at 21:02:57Z. The archives for ranks 1-10 were re-verified
  and extracted at 21:03:07Z, the first per-crate examination was at
  21:03:26Z (serde), and the screen ended at 21:07:27Z.
- Registered order (protocol line 239): arm B on every pool crate first, then
  arm A. Actual order: arm A ran while arm B was still running, between
  21:02:57Z and 21:07:27Z. See deviation 1.

SHA-256 of the registered files, verified unchanged at 2026-11-18T21:57Z
against the bead's list (paths under `research/protocols/`):

```
44490b7ddf5b73844d2f7057bd950bfa15fe54321a29fdf5809a8a7ef82b73af  fx-zu4f.10-protocol.md
f4c693bb8a804a1540bf02f6ba4844ae908189b8984f46c453929c57da83a3c7  fx-zu4f.10-pool/archive-hashes.tsv
6dfbdd19fc7ea9b1eedbdbfc0695d984a4a0f10be45ca89daaea9673c3299339  fx-zu4f.10-pool/pool.tsv
0c5f5189347353fc0ad791ebf93a1ad779a82b88b58bb9a514cfd68825bf25c7  fx-zu4f.10-pool/scan-output.tsv
8f2afde9707f49a9821093680e36cb1fe7827acec13ae40e7d2158256a0a7898  fx-zu4f.10-pool/scan.py
51ad681fd9421c2a2ed1af5f438dd385f930141e74b0119db234989e17b60e8c  fx-zu4f.10-pool/selection.json
c80b8b7f2b6c2529ede1c6581f4e1d05727455c97698acb0142bbb762f9ebb9f  fx-zu4f.10-pool/snapshot-manifest.json
516fe860462601993844fd16df8e1b2a3773279b7ddfdefbead4d165c8b9a29d  fx-zu4f.10-pool/snapshot/page-01.json
3e5a1ffc9cda8f141784cce103c5959bfea84800708e23050f04b200d5c7988e  fx-zu4f.10-pool/snapshot/page-02.json
d587e1f22d6c7f00b9365e7c079b3b3b8b7c88b437b6e859cc2ab561d0022acf  fx-zu4f.10-pool/snapshot/page-03.json
2e3c3920c4533a7363779464d44c9a0b81952398c6d3b5b9f6904cf0ee350d4e  fx-zu4f.10-pool/snapshot/page-04.json
f0b1babd7496d6d84b3dcb27afceb5191dec37f076978cc727adfac2378db4e5  fx-zu4f.10-pool/snapshot/page-05.json
efb167c97cdcdc276c8835b3b97550a86ea521364982f3126679cded57360f14  fx-zu4f.10-pool/snapshot/page-06.json
76454366185834a3794684e645349475decd32b3b1a99e4057e00a88c49a8622  fx-zu4f.10-pool/snapshot/page-07.json
f91c9f49aee4336f1ad032723cf8a7176fe5fd27dd0558b83adad085ff3fc0ff  fx-zu4f.10-pool/snapshot/page-08.json
3d45bb5d7589938aa662e7ccdcd8b51ee87749373a9dc04d83c6995ee11790b1  fx-zu4f.10-pool/snapshot/page-09.json
88b24c49f13d34de741a8fdc602d0bff9e9950eeda8cb101ab060db09f4c4f69  fx-zu4f.10-pool/snapshot/page-10.json
cfd82c5ca4427b3639fe37a2a0ec9382497a62abcd3a994843036a6e48fd522f  fx-zu4f.10-pool/stars.tsv
```

The record's own committed artifact:

```
5a884b74909c9dfe233c326ebd6d41d29f063662e2ea5b9c2a1f487b8fb5ed95  fx-zu4f.10-record/arm-b-ledger.tsv
```

## Tooling and environment

Linux 7.2.6 (x86_64). Python 3.14.10 (standard library only). git 2.43.0.
gh 2.104.0. No compiler, no heavy.sh run, no Tide driver run, and no upstream
code built or executed, in either arm. The git-ignored helper scripts in
`target/work/` were written for this round and are not shipped tools:

```
c4aae60fe936feec794c718c934f10d961fe736a358c3bb331d179d6587c9659  e1-paths.py      (path pass)
2c7944f1c6bae4a6736331d46f946162bf0319535e9965741b89abbe27e5f133  e1-comments.py   (declared comment class)
58f6697e4a8c2dbae927c3386f4e920f2d1d7203b73015f9e37111e49b93275b  b-evidence.py    (throttled REST evidence)
27342a151deb01369c59b93a90d86ea7a0f778266698f7de0963138972f7842e  inventory-b.py   (evidence inventories)
9966e9e8e5d17882d7593a0847269035df5bc9d5461d465dd7501c6ee596ef3b  record-b.py      (append-only decisions)
cccc53ee70b5a65ca7513ee14e4564f3559bb2486d7b0b75de0de63b3236dac1  coverage-b.py    (layer merge, counts)
12db032d9fc499410eebdf2c1690cc63d72f7f4ba0ec070d827be21075e8ae8f  record-ledger.py (builds the public ledger)
16be1030e12813607e96a292eb547786ef714a5da591e88e414530fe8c514de4  read-inv.py      (inventory viewer; its brief mode is deviation 8's filter)
c59e9d63cadfd0be172f5ab26c9349a558062d4d57730667bb7509e7c96abedc  view-b.py        (compact E1 view; message and PR bodies cut at 15 lines, 6 with --brief; deviation 3)
86d2cde1c73c219aa2b1cda3825dbd51c1761654f7e009235199810992b3737b  read-b.py        (full message, PR bodies and first-parent diff)
```

Builder 5's one-line triage viewer `terse.py` lived outside `target/work/`
and was not retained, so it has no hash (deviation 3).

Added for the r1 corrections (same directory, plain Python and one shell wrapper, no network
except through `inventory-b.py`):

```
75b0e561f046d4096fad1c8202e121325e903cb4c093ca63776cbabe073dc2f6  full-read-r1.py      (inventory reader; bot reports filtered, see deviation 8)
6487c6aa637935ed5d21e00bc30c47f49e37d505d735e38d33865ae2ea6dacf1  full-read-omitted-r1.py (prints what full-read-r1.py omitted, whole)
8e140e3dc8d1f598cc360d471f701e8f75adcb36eba0b7351e5e8ed8349521e9  dump-row-raw-r1.py   (one row's inventory, raw and whole)
038a017110b87e186cee3f8572b156c2d9e5c483035101dd3ce83f731045897d  record-b-r1.py       (superseding rejudgments)
ba8b90398d78f3e15a8ce2d50d835dbb85a1674ba95bdfd920bb430da057d3f3  confirm-b-r1.py      (superseding confirmations)
b2456bdea8af674bba1399f632f352870ce3fd9917775a535be8fd1cb01317dd  conf-omitted-r1.sh   (confirm-b-r1.py wrapper for the second pass)
92d24fd4efec1b691210f17b3960fe679c37f399b7a0fff3562d6884132a5573  cap-b-r1.py          (fills ledger cap cells: B09.003, B16.018, B16.128)
2df9ee781ffd1491046352cb22cb955fbcb51dcd54e7002bc26910fb6a98d595  field-b-r1.py        (one-field text corrections after the Opus review)
769c0a37e959b5c97f29e384f5bbbaf72d089ddbb4b915998b81c3818cba857c  sym-set-r1.py        (bounds the sym.py-possible set)
b18c8aab37f0cb63daa72736610a5a56c15d3877610bbd929f883736cf7cfb63  link-gap-r1.py       (linked reports missing from inventories; --all: every E2-or-later row)
937aeaef806ba3f624e2b18fd59247f6daa32c58084af89cf26d76c923968c77  link-gap-fetch-r1.py (re-inventories those rows)
a0c3f309342fe87db58ca583fce472c9faa3ca577daf58f527f42aa34f7d4a5e  e4count-r1.py        (arm-B E4 path counts)
667a880b8acf58e683baaf03a7267e59cd0e9c335a84b488e65ed4d87b41c246  arm-a-count-r1.py    (arm-A core counts)
```

The decision files stay append-only: each r1 change is a superseding line
carrying an `r1` field that names the finding and the superseded line, and
`coverage-b.py` and `record-ledger.py` take the last line per ID. Pre-r1
copies of each touched decision file are kept privately; for quinn-proto,
insta, opentelemetry, quinn-udp and ureq the copy was made after the first r1
append and was rebuilt as the exact prefix before the first `r1` line.

The first two hashes match the `code_sha256` stored with their private output
manifests. Every private path-layer and comment-layer output still matches
its recorded digest (checked at about 21:56Z). Python was used, not Rust, because these
are one-off transforms of private evidence files that nobody runs again and
the launch does not cite. The scan itself is the registered `scan.py`.

## The pool

All 24 pool crates, in download order. Ranks 1-10 are arm A's targets.

| Pool | Crate @ pinned version | Repository | Stars | Arm-B rows | Arm A |
| ---: | --- | --- | ---: | ---: | --- |
| 1 | serde 1.0.229 | serde-rs/serde | 10,976 | 138 | unportable: unresolved dispatch |
| 2 | rustls 0.24.0-dev.3 | rustls/rustls | 7,762 | 346 | unportable: union over 400 lines |
| 3 | uuid 1.29.3 | uuid-rs/uuid | 1,255 | 142 | unportable: external crate on the core path |
| 4 | darling_core 0.24.1 | TedDriggs/darling | 1,278 | 40 | no concurrent core |
| 5 | serde_core 1.0.229 | serde-rs/serde | 10,976 | 138 | unportable: unresolved dispatch |
| 6 | tracing-log 0.2.0 | tokio-rs/tracing | 6,986 | 878 | unportable: external crate on the core path |
| 7 | schemars 1.2.3 | GREsau/schemars | 1,433 | 62 | no concurrent core |
| 8 | env_logger 0.11.11 | rust-cli/env_logger | 1,064 | 130 | unportable: guard held across I/O |
| 9 | bindgen 0.73.2 | rust-lang/rust-bindgen | 5,327 | 318 | unportable: subprocess (cap:proc) |
| 10 | quinn-proto 0.11.19 | quinn-rs/quinn | 5,357 | 297 | unportable: external crate on the core path |
| 11 | quinn-udp 0.6.3 | quinn-rs/quinn | 5,357 | 297 | not targeted |
| 12 | console 0.16.6 | console-rs/console | 1,203 | 21 | not targeted |
| 13 | arc-swap 1.9.5 | vorner/arc-swap | 1,455 | 53 | not targeted |
| 14 | opentelemetry 0.33.1 | open-telemetry/opentelemetry-rust | 2,774 | 335 | not targeted |
| 15 | plotters 0.3.7 | plotters-rs/plotters | 4,652 | 94 | not targeted |
| 16 | ureq 3.5.2 | algesten/ureq | 2,234 | 133 | not targeted |
| 17 | colored 3.1.1 | colored-rs/colored | 2,045 | 18 | not targeted |
| 18 | regex-lite 0.1.9 | rust-lang/regex | 4,058 | 192 | not targeted |
| 19 | mockall 0.15.0 | asomers/mockall | 1,853 | 117 | not targeted |
| 20 | mockall_derive 0.15.0 | asomers/mockall | 1,853 | 117 | not targeted |
| 21 | windows-collections 0.100.0 | microsoft/windows-rs | 12,926 | 45 | not targeted |
| 22 | wit-bindgen-rust-macro 0.63.0 | bytecodealliance/wit-bindgen | 1,478 | 151 | not targeted |
| 23 | insta 1.49.0 | mitsuhiko/insta | 2,988 | 155 | not targeted |
| 24 | thrift 0.25.0 | apache/thrift | 10,969 | 859 | not targeted |

## Arm B: retrospective fixes

### Enumeration

Each of the 21 repositories got one read-only, non-shallow clone with no
checkout, no hooks and no submodules, filtered with `blob:none`. The refs were
frozen at clone time. Blobs needed later for diffs were fetched lazily as
immutable objects, and no ref was ever fetched again. All 21 clones were
recomputed and verified at 19:39:36Z: refs, exports and candidate lists match
their digests, no clone is shallow, and the registered regex, run again over
every full `%B` message, gives the saved candidates exactly. The clone
manifest's SHA-256 is
`6f26f8968410c395722c3dc7d2e079c181a218f046e3352a86c91aeef8a2b42e`; the
file is private, since it lists local paths.

| Repository | Frozen HEAD | Commits | Matches | Pool crates |
| --- | --- | ---: | ---: | --- |
| serde-rs/serde | `e306b0c7c40ad21d5bdf52bcd20da513904682ae` | 4,374 | 138 | serde, serde_core |
| rustls/rustls | `836e411eccaf7331f4c289aabef0de32474de3b8` | 6,782 | 346 | rustls |
| uuid-rs/uuid | `266c044a9cd9845de011251525f45a6bee341774` | 1,872 | 142 | uuid |
| TedDriggs/darling | `4237d61d00b5d30291a8a51b369d66e3a4480b1b` | 622 | 40 | darling_core |
| tokio-rs/tracing | `8d4d4d6ceaba222a1fcf94d98f030d2970a26611` | 2,752 | 878 | tracing-log |
| GREsau/schemars | `4791bb7352555d339e45658ca09c092ed9fa1f5d` | 691 | 62 | schemars |
| rust-cli/env_logger | `3ab29b70ccf34075d04abac22204d2404505ff31` | 897 | 130 | env_logger |
| rust-lang/rust-bindgen | `0d213bc3dbad03e27fe729b7fb1bb5c77cffda1d` | 4,320 | 318 | bindgen |
| quinn-rs/quinn | `c7b46a969cd315398b504792e9e9af6a15a7dcc6` | 4,815 | 297 | quinn-proto, quinn-udp |
| console-rs/console | `c1220d680ee805612c85a8644205fd57d7df273e` | 434 | 21 | console |
| vorner/arc-swap | `c88f8bfad9e013fad3fd6e2bba26ac6b975edb55` | 574 | 53 | arc-swap |
| open-telemetry/opentelemetry-rust | `83e99267b22ac9ee1605eb5d1342c3b820f3bf51` | 2,471 | 335 | opentelemetry |
| plotters-rs/plotters | `c63248e0cf7c611185361ac492003e8a87d52ea8` | 1,687 | 94 | plotters |
| algesten/ureq | `51bb001154f44cfab69653a74f7d45fbe1995b22` | 1,403 | 133 | ureq |
| colored-rs/colored | `5204a26619e10fda0adc77f07588811d6102a40d` | 177 | 18 | colored |
| rust-lang/regex | `4226f214c56e584304520adb9080bcc4c792390e` | 1,693 | 192 | regex-lite |
| asomers/mockall | `a47295fb08a178e1f5c570c38c8b40cd663a2750` | 1,146 | 117 | mockall, mockall_derive |
| microsoft/windows-rs | `59ec4ec7118b7d709740c3ffb98cf38873449047` | 3,160 | 45 | windows-collections |
| bytecodealliance/wit-bindgen | `2dae8a619b285966d02f76261dad4576a135c808` | 1,402 | 151 | wit-bindgen-rust-macro |
| mitsuhiko/insta | `0e17c7a6ed65b73def1101813987a053269ef4a5` | 1,036 | 155 | insta |
| apache/thrift | `9f1e81eed106e05440693f01251758621cf1f822` | 8,560 | 859 | thrift |

Row IDs are `Bpp.nnn`: the pool rank, then the matched SHA in lexical order
within that crate. The regex has no word boundaries, so most matches are
textual, such as "change", "trace" or "brace". Each one still has a row, as
the protocol requires.

### How rows were judged

Three layers, merged per row in this order: the path pass first, then the
hand decision, then the comment class.

1. **Path pass (E1, mechanical).** A row fails E1 when the first-parent diff
   touches none of the crate's own non-test source. That covers changes that
   are test-, doc-, bench-, example- or CI-only, and changes that touch only
   a workspace neighbour or configuration. The protocol defines each of these
   as an E1 failure. The source roots include predecessor package paths (for
   example, `tokio-trace-log/src/` for tracing-log), so the pass is a
   conservative superset. Inline tests and docs inside source files are left
   for hand review. Renames are disabled, so both paths of a rename count.
   Result: 3,779 rows.
2. **Declared comment/whitespace class (E1, mechanical).** This class was
   declared in `target/work/report.md` at 19:57:49Z, before it was applied to
   any row, under tide-lead's decision of 19:56 UTC. For each own-source path
   of a still-pending code row, the parent and fixed blobs are compared as
   token streams, with comments and whitespace removed and literals kept
   exact. If every own-source path is token-equal and no `Cargo.toml`,
   `Cargo.lock` or `build.rs` changed, the row fails E1 as doc-only.
   Signature-only changes and dependency bumps are never pre-passed. Hand
   decisions already made were cross-checked, never overridden. Result: 76
   rows.
3. **Hand judgment, E1 first.** E1 was decided from the matched commit's
   message, every associated PR's title and body, and the diffstat, read
   through the viewers in deviation 3 (some cap long messages and PR
   bodies, and one triaged rows on a single line). The own-source hunks were read in full whenever the message or PR claimed a
   fix. Only rows that passed E1 got the full evidence inventory: commit
   comments, PR bodies, comments and reviews with all pages, and explicitly
   linked issues. Protocol §3 requires that inventory to be read in full
   before any E2-or-later failure. Before r1 that was not always done: some
   rows were settled from abridged views, and some inventories lacked a
   linked report (deviation 8). In r1 the 183 rows that could have been
   settled from abridged views (182 plus B15.047) were re-read in full
   (human text first, then every bot report, HTML comment and URL the
   first reader omitted; deviation 8), and each was confirmed or rejudged.
   The link-gap check first covered those 183 rows; after the Opus review
   it covered all 306 rows that fail at E2 or later. Every missing report
   in the fetched classes was fetched and read: 24 rows in r1, plus
   B09.199, B11.082 and B09.162, and B02.277 and B09.092 after the review.
   The 61 references still absent are all in the declared not-fetched
   classes (deviation 8). Result: 1,220 hand-judged rows (914 E1, 109 E2, 187 E3, 10 E4).

The E2 rule is fx-zu4f.9 r2's: a stated failure that the evidence ties to
this row's bug passes, even tentatively. A bare cause label, a "might/could"
failure, or a failure that belongs to another defect does not pass. The
builders recorded the consistency rules they applied in each checkpoint, and
they hold across builders. Some examples:

- A std/rustc compatibility repair against an upstream change the crate did
  not pin passes E1. If its evidence says only "fix for upstream changes", it
  fails E2.
- A quoted build error or a stated broken build passes E2 tentatively.
- Adapting to a deliberately bumped dependency fails E1.
- A spec-conformance gap fails E2 unless an occurrence or a concrete wrong
  output is stated.
- Hardening that only changes an API to return `Option` fails E1.
- "No behavior change" fails E1.

### Coverage by crate (first failures)

| Pool | Crate | Rows | E1 path | E1 comment | E1 hand | E2 | E3 | E4 | Unresolved |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | serde | 138 | 92 | 3 | 26 | 5 | 12 | 0 | 0 |
| 2 | rustls | 346 | 112 | 11 | 171 | 29 | 23 | 0 | 0 |
| 3 | uuid | 142 | 22 | 12 | 97 | 8 | 3 | 0 | 0 |
| 4 | darling_core | 40 | 21 | 0 | 17 | 0 | 2 | 0 | 0 |
| 5 | serde_core | 138 | 135 | 0 | 3 | 0 | 0 | 0 | 0 |
| 6 | tracing-log | 878 | 793 | 7 | 59 | 5 | 14 | 0 | 0 |
| 7 | schemars | 62 | 44 | 4 | 10 | 0 | 4 | 0 | 0 |
| 8 | env_logger | 130 | 110 | 3 | 11 | 1 | 5 | 0 | 0 |
| 9 | bindgen | 318 | 109 | 5 | 110 | 29 | 64 | 1 | 0 |
| 10 | quinn-proto | 297 | 162 | 2 | 105 | 11 | 13 | 4 | 0 |
| 11 | quinn-udp | 297 | 287 | 0 | 5 | 1 | 4 | 0 | 0 |
| 12 | console | 21 | 12 | 2 | 6 | 0 | 1 | 0 | 0 |
| 13 | arc-swap | 53 | 39 | 0 | 7 | 4 | 2 | 1 | 0 |
| 14 | opentelemetry | 335 | 196 | 5 | 116 | 6 | 11 | 1 | 0 |
| 15 | plotters | 94 | 51 | 10 | 31 | 0 | 2 | 0 | 0 |
| 16 | ureq | 133 | 85 | 3 | 31 | 4 | 8 | 2 | 0 |
| 17 | colored | 18 | 10 | 0 | 7 | 1 | 0 | 0 | 0 |
| 18 | regex-lite | 192 | 192 | 0 | 0 | 0 | 0 | 0 | 0 |
| 19 | mockall | 117 | 100 | 8 | 9 | 0 | 0 | 0 | 0 |
| 20 | mockall_derive | 117 | 66 | 0 | 43 | 4 | 4 | 0 | 0 |
| 21 | windows-collections | 45 | 45 | 0 | 0 | 0 | 0 | 0 | 0 |
| 22 | wit-bindgen-rust-macro | 151 | 131 | 0 | 18 | 0 | 2 | 0 | 0 |
| 23 | insta | 155 | 114 | 0 | 28 | 0 | 11 | 1 | 1 |
| 24 | thrift | 859 | 851 | 1 | 4 | 1 | 2 | 0 | 0 |
| | **Total** | **5,076** | **3,779** | **76** | **914** | **109** | **187** | **10** | **1** |

### The ten rows that reached E4

All ten passed E1-E3. Each failed E4 against the registered revision, which
was never moved (desk decision, 2026-11-18). E5 was not assessed for any of
them. The first restriction named is the one that decided the row.

| Row | Crate, commit | E2 quote (abridged) | Deciding E4 restriction | Capability-only |
| --- | --- | --- | --- | --- |
| B09.003 | bindgen `02da6536e25a` | "For very large logs to stdout, this prevents deadlocks." (commit message) | subprocess on the path (frozen ban): the parent waits on a clang child whose stdout pipe is full | **yes, cap:proc** |
| B10.064 | quinn-proto `30e1d6fe14` | "the 0-RTT packet will not find any associated connection and thus be dropped" | external crate (rustls crypto session for 0-RTT); additional: async accept | no |
| B10.157 | quinn-proto `81b8d1e3ca` | "Missing retransmits when first ACK from client is lost" | external crate (rustls crypto session driving the handshake); additional: a timer | no |
| B10.233 | quinn-proto `c09ff21012` | "`send_datagram_wait` blocks indefinitely during a benchmark" | async/Future/Waker path (rejected by design); no reconstruction allowed | no |
| B10.226 | quinn-proto `bb23ebff3d` | as B10.233 (backport, `duplicate_of` B10.233) | as B10.233 | no |
| B13.031 | arc-swap `97a65cd1e9` | "MIRI considers it a data race" | memory-unsafety symptom (sanitizer data race) narrowing; additional: raw-pointer list beyond the two unsafe mappings | no |
| B14.259 | opentelemetry `c697b58008` | "deadlocks a `BatchSpanProcessor` if it was spawn on a different runtime" | async worker and external runtimes (async excluded by design; external crates) | no |
| B16.018 | ureq `17d7e147eb` | test panicked with `ConnectionReset` | network I/O, and the race is decided by kernel RST timing and error kinds | no |
| B16.128 | ureq `f60896d6b6` | "`set_read_timeout` fails with EINVAL after the server has closed" | network I/O plus macOS kernel behavior (setsockopt EINVAL) | no |
| B23.129 | insta `d03c2a67b5` | "called `Result::unwrap()` on an `Err` value: PoisonError" | subprocess (`cargo metadata`) with the WORKSPACES guard held across it; additional: OS-error trigger, std Mutex poisoning has no mapping | no |

The full quotes, their URLs and the per-row reasons are in the ledger.

Path counts (Codex r1 B4; `e4count-r1.py`, plain Python over the frozen
clones at each first parent, no build). For each row: the items enclosing
every own-source changed line (the innermost `fn`; a changed line outside any
`fn` counts singly), as nonblank, non-comment lines with comments and literal
contents masked, counting the union once. Each number is a **measured
changed-item union**: it counts every item the fix touched, including any
formatting or configuration item, and no source map establishes that those
items lie on the affected path, so it is not a lower bound of that path.
The affected (racing or waiting) path and its helpers were **not measured**,
because each row fails E4 first on a restriction that is not the budget. No
row is excluded by the 400-line cap.

| Row | Changed items at the first parent (nonblank, non-comment lines) | Changed-item union |
| --- | --- | ---: |
| B09.003 | `src/lib.rs` `dump_preprocessed_input` 55 | 55 |
| B10.064 | `endpoint.rs` `handle` 135, `handle_first_packet` 61, `accept` 121, `early_validate_first_packet` 21, `refuse` 10, `retry` 36, `insert_initial` 3, `get` 24, two `fmt` 17, 6 item lines; `config.rs` `new` 14, `preferred_address_v6` 4, `fmt` 11, 1 item line | 464 |
| B10.157 | `connection/mod.rs` `pto_time_and_space` 33 | 33 |
| B10.226 | `datagrams.rs` `drop_oversized` 14; `connection/mod.rs` `detect_lost_packets` 107 | 121 |
| B10.233 | `datagrams.rs` `drop_oversized` 14; `connection/mod.rs` `detect_lost_packets` 116 | 130 |
| B13.031 | `debt/list.rs` `default` 9, `traverse` 11, `get` 30, 1 item line | 51 |
| B14.259 | `span_processor.rs` `new` 94, `builder` 22, 7 item lines; `export/trace/mod.rs` three `send` 32, 12 item lines; `global/trace.rs` `drop` 8, `set_tracer_provider` 15, 4 item lines; `stdout.rs` `install` 11, 3 item lines; `provider.rs` `with_exporter` 10, 7 item lines; `global/mod.rs` 2, `lib.rs` 2, `util.rs` 1 item lines | 230 |
| B16.018 | `error.rs` `is_bad_status_read` 6, `status` 19, `status_text` 19, `body_text` 19, 2 item lines; `response.rs` `do_from_read` 28; `stream.rs` `connect_https` 16, `connect_host` 96; `unit.rs` `connect` 66 | 271 |
| B16.128 | `unversioned/transport/tcp.rs` `maybe_update_timeout` 13, `await_input` 30 | 43 |
| B23.129 | `insta/src/env.rs` `get_cargo_workspace` 49 | 49 |

Before r1, B10.064's and B10.157's decisions also named "path over 400
lines" as an additional concern. B10.157's changed path is 33 lines, so that
claim was unsupported; B10.064's 464 lines exceed 400 only if every changed
`endpoint.rs`/`config.rs` item is on the racing path, which was not
established. Both decisions now carry superseding lines that call the path
size not established and give the measured changed-item union instead.

Which constraints decided rows (protocol §4, "How failures are described"):

- The frozen experiment bans decided six: subprocess (B09.003, B23.129),
  external crate (B10.064, B10.157) and network (B16.018, B16.128). The two
  network rows also depend on kernel behavior, which a Net capability would
  not supply.
- Async/Waker, rejected by design, decided three: B10.226, B10.233 and
  B14.259.
- The memory-unsafety symptom narrowing decided one: B13.031.
- The 400-line cap decided no row and is not established as an additional
  concern for any row (see the path counts above).
- E5, the Condvar/select reconstruction ban and the Once/lazy-initialization
  ban decided no row.
- No E4 reason names a v0 primitive that is planned but unshipped (raw
  Mutex/RwLock, SC atomics).

### Capability-only failures

Under the desk decision recorded on the bead at 20:47 UTC, each row whose
only first-failure reason is a missing Proc or Net capability is listed here
for the planned landed-capability re-screen round. E4 in this round stays at
the registered revision.

| Arm | Row | Tag | Reason |
| --- | --- | --- | --- |
| B | B09.003 (bindgen `02da6536e25a0c45dd8ee77ee37d7993721666e8`) | cap:proc | drains the clang child's stdout before `Child::wait`; the path is a subprocess pipe |
| A | rank 9, bindgen 0.73.2 | cap:proc | the core is the rustfmt child's stdin/stdout pipes (a thread writes stdin while the main thread reads stdout) |

B23.129, B16.018 and B16.128 also touch subprocesses or sockets. They are not
capability-only: a Proc or Net capability would not supply the OS error, the
kernel behavior or mutex poisoning they depend on.

### Unresolved row

**B23.128**, insta `cb0c6af9645b9934e176328db7bdc192ddecf73a` (first parent
`f6b6fd363980ae659bb0628b6a1cc637030bbd1a`), subject ending "(#949)". The
required PR thread is unavailable. GitHub's associated-PR list for the commit
is empty (200, 0 items). REST `pulls/949` returned 404 four times, at
21:41:56, 21:42:26, 21:43:27 and 21:44:27Z. An unlogged `issues/949` probe
returned 404 at about 21:47Z. A GraphQL `pullRequest(number: 949)` query
returned NOT_FOUND at 21:48:08Z. Under protocol §3, unavailable required
evidence stays unresolved and does not become a completed negative.

*Note, not counted:* builder 10's provisional reading is that E1 passes on
the message, and that the message and diff show a single-thread
whitespace-normalization fault, so E3 would fail. This reading is not part of
any count above.

### The per-row ledger

[`fx-zu4f.10-record/arm-b-ledger.tsv`](protocols/fx-zu4f.10-record/arm-b-ledger.tsv)
has one row per keyword-match membership: 5,076 rows plus a header. It is
UTF-8 and tab-separated, with no quoting; tabs and line breaks inside fields
are replaced by spaces. Columns, in file order:

| Column | Meaning |
| --- | --- |
| `id` | `Bpp.nnn` |
| `pool_rank`, `crate`, `repository` | pool identity |
| `commit`, `first_parent`, `parents` | the matched commit, its first parent (the buggy revision) and its parent count (2 = merge) |
| `stage` | which layer decided: `path`, `comment`, `manual` or `unresolved` |
| `first_failure` | `E1`-`E4` or `UNRESOLVED` |
| `reason` | why that criterion failed |
| `passed` | the recorded E1/E3 pass notes for rows that got past them |
| `e2_quote`, `e2_source` | the E2 quote and its URL/identity, when E2 passed or was quoted |
| `duplicate_of` | the earlier row with the same bug site |
| `cap` | capability tag (`cap:proc`) or "not cap-only" |
| `later_criteria` | which later criteria were not assessed |
| `diff_sha256`, `inventory_sha256` | SHA-256 of the saved first-parent diff and of the evidence inventory (hand-judged rows) |
| `assessed_utc` | decision time |
| `note` | additional, uncounted observations and disclosed slips |
| `sources` | the public commit URL, plus every PR, issue or discussion URL in the row's evidence inventory (never empty) |

Builder 9 made a disclosed slip on two duplicate pairs: it put `duplicate_of`
on the earlier ID (B20.013 → B20.049, B20.015 → B20.080). The decision lines
are append-only, so the ledger keeps them as recorded and adds a `note` on all
four rows: the first row of each pair is B20.013 and B20.015. No count
changes.

## Arm A: forward hunt on pool ranks 1-10

Builder A1 extracted its own copies of the ten verified archives. All ten
SHA-256s matched `archive-hashes.tsv` at 21:03:07Z, and extraction used
tarfile's `data` filter. Starting from every non-test thread, channel, lock
and atomic use, it traced the concurrent core by hand. Per-crate maps are
kept privately and listed below by digest. They contain no find, and their
substance is reproduced in this table.

| Pool | Crate @ version (archive SHA-256 prefix) | First examined (UTC) | Concurrent core and line count | First failing criterion | Additional (not counted) |
| ---: | --- | --- | --- | --- | --- |
| 1 | serde 1.0.229 (`4148590a`) | 21:03:26Z | `Serialize` for `Mutex`/`RwLock` (lock, then serialize under the guard); atomic `Serialize` (Relaxed load); `Deserialize` constructors. 143 lines, **partial** (the named sites; the lock impls' callees are caller-generic `Serialize`/`Serializer`/`Deserializer` dispatch, unresolved and not counted; not measured: the in-crate primitive impls the atomic impls call statically, `Serialize` on the loaded value via `ser/impls.rs`'s `primitive_impl!` (lines 7-36) and, in the constructors, `Deserialize` via `de/impls.rs`'s `BoolVisitor`/`bool` impl and `impl_deserialize_num!` impls with their visitor macros (lines 53-530)) | unportable: unresolved dispatch. The guard is held across caller-supplied generic `T::serialize` / `S: Serializer` | lock poisoning has no §4 mapping; no packaged doc or test states a concurrency property |
| 2 | rustls 0.24.0-dev.3 (`e7378b5d`) | 21:04:02Z | `conn/split.rs` `Arc<Mutex<SendInner>>` shared by the send/receive halves; ticketer `RwLock`; compression cache; client/server session caches. ≥ 1,317 lines, enumerated lower bound (the named roots plus `send.rs`, before the state machine) | unportable: union over 400 lines | unresolved dispatch (`dyn RecordEncrypter`, `StateMachine`, crypto/compressor providers) |
| 3 | uuid 1.29.3 (`3c14b15d`) | 21:04:33Z | timestamp contexts: `ContextV1` `AtomicU16` fetch_add; shared `Mutex<ContextV7>`; `ThreadLocalContext`; miri-only `Mutex` clock. ≥ 540 lines, enumerated lower bound (the four contexts plus `rng.rs`, before `Timestamp` and the `ClockSequence` defaults) | unportable: external crate on the core path (`ContextV7` reseed calls `crate::rng::u64()`, i.e. getrandom/rand, under the shared mutex) | guard held across that call; thread-local storage; `LazyLock`; generic `Mutex<C>` dispatch |
| 4 | darling_core 0.24.1 (`6837e2cf`) | 21:05:15Z | none: `AtomicBool::new` construction and a `thread::panicking()` query. 0 synchronizing lines (the two sites: 18 lines) | no concurrent core (E3 applied to the core) | none |
| 5 | serde_core 1.0.229 (`67dca2c9`) | between 21:03:26Z and 21:04:02Z (byte-identical to serde's `src/core`) | as serde. 143 lines, **partial** (as serde) | unportable: unresolved dispatch (as serde) | lock poisoning |
| 6 | tracing-log 0.2.0 (`ee855f1f`) | 21:05:35Z | `interest_cache.rs`: `AtomicUsize` epoch, `Lazy<Mutex<config>>`, thread-local LRU state. 166 lines in the crate, **partial** (the module, `lib.rs`'s statics and the in-crate callers in `log_tracer.rs`: `enabled`, `init`, `with_interest_cache`; not measured: the same-crate helpers `enabled` reaches through `metadata.as_trace()` while `try_cache` computes the cached interest, `AsTrace::as_trace` (`lib.rs:229-241`) and `loglevel_to_cs` (`lib.rs:339-353`), which dereferences the counted statics, and the statics' initializer `Fields::new` (`lib.rs:261-275`); external crates not counted) | unportable: external crates on the core path (lru, ahash, once_cell `Lazy`, tracing_core's callsite registry) | `thread_local!` state; ordering-relevant lazy initialization |
| 7 | schemars 1.2.3 (`ab508826`) | 21:06:00Z | none: type-level `JsonSchema` impls for `Mutex`/`RwLock`/atomics with no value access (the named impls and macro: 57 lines) | no concurrent core (E3 applied to the core) | none |
| 8 | env_logger 0.11.11 (`de671bd2`) | 21:06:13Z | `Mutex<dyn io::Write>` pipe target. 253 lines in the crate, measured: the map's sites plus §6's same-repository helpers (initialization: `Builder::target`/`build`/`try_init`, `Target::Pipe`, `writer::Builder`; producer: the `Logger` struct, `matches`, `log`, `Formatter`'s `new`/`write_style`/`print`/`clear` and its `io::Write` impl in `fmt/mod.rs`, `Writer`; consumer: `BufferWriter`'s `pipe`/`buffer`/`print`, `adapt`, `Buffer`, `WritableTarget`; the empty `Log::flush`). Not counted: the caller's writer, the record formatting reached through `FormatFn` (it writes only the thread-local buffer, before the lock), and the external `log`, `env_filter` and `anstream` crates | unportable: guard held across I/O (`write_all`/`flush`) on a caller-supplied `dyn io::Write` (§4) | unresolved dispatch; thread-local formatter; std stdout/stderr locks |
| 9 | bindgen 0.73.2 (`787ef8ef`) | 21:06:32Z | `lib.rs` `rustfmt_generated_string`: a thread writes rustfmt's stdin while the main thread reads its stdout, then wait and join. 107 lines, **partial** (the enclosing `fn format_tokens`, `lib.rs` 1006-1086, its helper `rustfmt_path` and its caller `write`; not measured: the same-crate helpers `format_tokens` calls, `RustTarget::latest_edition` (`features.rs:212-220`, with its `is_available` callee) for the child's edition argument and, on the child-status-3 error branch, `rustfmt_non_fatal_error_diagnostic` (`lib.rs:1089-1106`)) | unportable: subprocess on the core path (frozen ban); **cap:proc** | none |
| 10 | quinn-proto 0.11.19 (`0e750cca`) | 21:06:45Z | three `Mutex` components: qlog stream, `BloomTokenLog`, `TokenMemoryCache`. 276 lines, **partial** (the three components' files; not measured: the `TokenLog`/`TokenStore` traits, their endpoint/connection callers, and `QlogStream`'s construction at `config/transport.rs:620` and its emit callers) | unportable: external crates on the core path (fastbloom, rustc_hash, lru_slab, bytes, qlog) | qlog guard across I/O; `SystemTime` wall clock |

- **Line counts** (Codex r1 B4; `arm-a-count-r1.py`, plain Python over the
  verified extracted archives, after arm-B judging had finished). Each count
  is the union of the file/line ranges and enclosing functions in the
  counter's spec, as nonblank, non-comment lines with `e4count-r1.py`'s
  masking, excluding `#[cfg(test)]` modules. Labels, per row: **measured**
  means the spec covers every in-crate site the map names and the
  same-repository helpers §6 requires on the path to and from the shared
  state, with anything left out named in the row; **enumerated lower bound**
  means the counted part alone exceeds 400 lines and the core continues
  beyond it; **partial** means the count is under 400 and code the core
  needs was not counted, named in the row. darling_core's 18 and schemars's
  57 count inspected sites, not a core. For env_logger, tracing-log and
  bindgen the r1 fix-up added helpers and callers the private maps do not
  list (the maps are kept unchanged; the counter's spec names each added
  range). Before r1 the table gave estimates (~58, ~40, "under 400"). Two
  lower bounds exceed 400 (rustls 1,317, uuid 540); for uuid that is an
  additional failure, since its first failure stays the external crate on
  the core path. Only env_logger's 253 is measured. tracing-log's partial
  166 and quinn-proto's partial 276 are under 400, but their cores call
  external crates, which decides them; serde's and serde_core's partial 143
  and bindgen's partial 107 are decided by unresolved dispatch and the
  subprocess. No count changes a first failure and no budget claim rests on a partial
  count. Roots and included sites per crate are in the private maps
  (digests below) and the counter's spec.
- **Core scope.** For each crate, the core is the union of every interacting
  root. Partial ports were not tried, since §6 forbids presenting part of a
  core as the whole. For example, uuid's `ContextV1` counter alone would map,
  but the crate is unportable.
- **Property quotes.** None. Every core failed portability, or had no
  concurrent core, before the property step. So no property set was frozen,
  and §6's property-freeze step never ran.
- **Constraints that decided.** Frozen experiment bans: external crate
  (uuid, tracing-log, quinn-proto) and subprocess (bindgen). Protocol budget:
  400 lines (rustls). Safe-design limits on `Shared`, a pure, non-nesting
  closure with no I/O: unresolved dispatch (serde, serde_core) and a guard
  across I/O (env_logger). No core was decided by a missing v0 primitive.
- **Count.** Unresolved dispatch 2, union over 400 lines 1, external crate on
  the core path 3, guard across I/O 1, subprocess 1, no concurrent core 2.

SHA-256 of the private per-crate maps, final at 21:07:27Z and re-verified at
about 21:56Z:

```
f7781f5b779dca1914d27f230c42046630e24a6db2b1db6b311a696dcb9388a0  A01-serde.md
9820a9117908299f0cfdd31e448a932f67b74223e4e3cfd055ec66b786f126c6  A02-rustls.md
a00313e0a8eda515085c93c8125f7f0a14a56b9e0a0e304b00e40a683e1d539b  A03-uuid.md
2004da039990795b203bef516426e0be4397c0f3952d1eca63fc375142e3c4c0  A04-darling_core.md
0805cc4a66b71e66615134e76842b8c691c3ab974819cca9b663cda106030abe  A05-serde_core.md
0a50c2d7f9b47bc1b95cab0ef370575818a5e402e1c1e33666e50d112393413e  A06-tracing-log.md
8d6915cc8eaf1e75c4f40d61540c34c76a1738a5f267ff72ad097c365eaca6eb  A07-schemars.md
dd59bc8c13c267f4a9bf62bd3868404048ca2af6dd3b382b88a88e7167b9739c  A08-env_logger.md
840f255533ea2b2e01400a976e1cff7a671150b221f9a3c234dbefd34dc2474b  A09-bindgen.md
be7b1697d35fd34325d3ac934f45518ba2540bb6b342b27c888d2d9c013be5d0  A10-quinn-proto.md
```

## Disclosed deviations and method notes

None of these changes a pool entry, an eligibility criterion, a
faithful-model requirement or a caught rule.

1. **Arm order.** Protocol line 239 registers "Arm B runs first on every pool
   crate, then arm A". Under tide-lead's decision of 19:56 UTC, arm A instead
   ran beside arm B (21:02:57Z-21:07:27Z, while arm B ran from 19:35Z to about
   21:50Z). The reason given: the unbounded registered regex turned arm B into
   days of hand judgment, and arm A was the likelier route to the public catch
   the RC waits on. To keep the arms independent, each builder used separate
   files. Builder A1 recorded that it read no arm-B file. The arm-B builders
   were told never to read `WORK/arm-a/` or `PRIVATE/arm-a/`: builder 3's
   declaration says so, and so does the reusable successor brief that
   builders 6-10 worked from.
2. **Codex safety-filter handoff.** Builder 2 (Codex gpt-6.1-sol) stopped at
   OpenAI's safety filter ("This content can't be shown … bypassing security
   controls") while reading an upstream diff during manual E1 rows, around
   B01.035-.037. No rewrite option was taken, and the task's framing did not
   change. The remaining hand work moved to Claude Opus 5.5 (builder 3
   onward), following the standing practice for Codex's cyber filter.
3. **E1-first method** (tide-lead's operational reading, bead comment 19:43
   UTC). The protocol requires the full evidence inventory only before a
   failure at E2 or later. So E1 was decided first, from the message,
   associated-PR titles and bodies, and the diff. Only rows that passed E1
   got full inventories. The E1 viewers: `view-b.py`, the compact E1 view,
   prints the message and each associated PR body cut at 15 lines (6 with
   `--brief`), saying how many lines it omitted, plus the diffstat and, with
   `--diff N`, the first N own-source diff lines; `read-b.py` prints the full
   message, the PR bodies and the full first-parent diff; `terse.py`, a
   one-line scratch viewer that builder 5 wrote outside `target/work/` and
   that builder 7 also had, was used for triage and was not retained.
   Builder 5 read every non-obvious row with `view-b.py --diff` or the full
   message. The Opus review checked the parts `view-b.py` omits for all 914
   hand-judged E1 rows (195 messages and 93 PR bodies over 15 lines) and
   found no strong concurrency term in them; of the 166 rows decided in
   builder 5's window, the four that combine concurrency and bug words
   anywhere (B03.069, B06.049, B06.271, B06.397) are correct E1 failures.
4. **Path pass.** This is the deterministic E1 class described above.
   Intermediate checkpoint totals differ from the final figure: 3,569 in the
   19:55 UTC bead comment and 3,774 in builder 2's handoff, against 3,779 in
   the final path layer from builder 3's checkpoint onward. This record uses
   the final layer. It matches the output digests stored with the path rules,
   whose `code_sha256` equals the current `e1-paths.py`. The earlier figures
   were not re-derived here. They are flagged for review as an unexplained
   intermediate difference.
5. **Declared whitespace/comment E1 pre-pass.** Declared at 19:57:49Z, before
   it was applied, and refined at 19:58:45Z during its self-test, still before
   any row was classified. It is applied only to rows still pending after the
   path pass. Result: 76 rows.
6. **Request spacing.** An offline audit of all 2,867 retained REST attempts
   (19:47:50Z-21:47:53Z) found exactly four gaps under 1 s, all 5-16 µs apart.
   They are four serde pairs between 19:48:59Z and 19:49:35Z, where two
   collectors in builder 2 woke on a shared timestamp without a lock. An
   exclusive flock was added before any further request. Every other gap is
   at least 2.02 s. Later, builder 5 ran two inventory queues for bindgen at
   the same time, and builder 6 ran five, all overlapping a prepare chain.
   Every one of them went through the same flock, so they caused no further
   gap under 1 s. Three requests fall outside that audit: builder 1's GraphQL
   `rateLimit` probe, which returned a schema dump and provided no evidence;
   builder 10's unlogged `issues/949` probe; and builder 10's single GraphQL
   query (insta discussion 794 and PR 949, 21:48:08Z). GraphQL requests are
   read-only queries sent by POST, not REST GETs.
7. **Arm A's core maps.** They were traced by hand from lexical search in
   4.5 minutes for ten crates, not from compiler reachability. tide-lead
   flagged the speed for this record's review, and the review covers the
   arm-A table too.
8. **Abridged reading before E2-or-later failures (Codex r1 B1), corrected
   in r1.** Protocol §3 requires the inventory to be read in full before any
   failure at E2 or later. Before r1 two abridged views settled some rows:
   - `read-inv.py`'s brief mode: for each body over 300 characters, its first
     three lines cut to 300 characters, then at most six keyword-matching
     lines of the rest, each cut to 160 characters. 41 rows at E2 or later
     carry a compact-reader or partial-read marker (10 E2, 28 E3, 3 E4),
     including partial diff reads ("first N lines, rest by diffstat"). 14
     more partial-read rows fail at E1, which needs no inventory.
   - `sym.py`, a scratch viewer of builders 7-9 (not kept): each endpoint's
     subject plus at most four keyword sentences. It left no per-row marker,
     so its possible set is bounded by time: the 141 rows at E2 or later that
     builders 7-9 decided between 21:01Z and 21:38Z without a compact marker
     (50 E2, 86 E3, 5 E4; `sym-set-r1.py`).

   In r1 all 182 rows were re-read in two passes. The first,
   `full-read-r1.py`, printed every human-written body, comment and review
   whole, except that it removed HTML comments and replaced parenthesized
   URLs carrying a query string with "(url)". It did not print bot reports
   (rustls-benchmarking, codecov, other `[bot]` authors) whole: for each it
   printed only the lines that were neither table rows nor template text,
   plus every line that matched a failure word, each cut to 200 characters.
   It skipped `commits/<sha>/pulls` records as duplicates of their
   `pulls/<n>` records. The second pass, `full-read-omitted-r1.py`, printed
   exactly what the first omitted, with no filter or truncation: every bot
   body whole at full length (35 rows), the removed HTML comments and URLs
   (45 more rows), and a byte-identity check of each skipped
   `commits/<sha>/pulls` record against its `pulls/<n>` record (all 134
   identical). Those 80 rows' omitted portions were read in full, line by
   line, including B02.038's and B02.213's long rustls-benchmarking reports;
   the other 103 r1 rows (182 plus B15.047) had nothing omitted. B10.010's
   reporter pasted two trace logs (issue 1472 comments 2 and 5, 224 and 771
   lines); `dump-row-raw-r1.py` printed the row's inventory raw and every
   line was read, repeats included. The log shows the client discarding
   every server reply as from an "unrecognized peer" after connecting to
   0.0.0.0, matching the maintainer's diagnosis, so E3 stands. The omitted
   material was coverage, benchmark, CI and template text, and none of it
   changed a row. The re-read also found linked reports that
   the inventories lacked (B09.199's issue 2239, B11.082's issue 2491), so
   `link-gap-r1.py` checked all 183 r1 rows for issue or PR references in the
   message or the PR and issue bodies whose target was not inventoried. It
   fetched every target named with a closing or fix keyword, a report the
   commit names as its source, and "part of"/"Related" targets: 24 rows
   re-inventoried, each read in full. Not fetched: prior-work, release-list,
   upstream-toolchain and other-language links, renamed-repository self
   references and related feature requests (36 references, listed by the
   script). The Opus review found that this scope left out the 122 other
   rows that fail at E2 or later, so fixer 12 ran `link-gap-r1.py --all`
   over all 306 such rows (every row whose last decision line fails at E2,
   E3 or E4). Two of the 27 references absent from those 122 rows had a
   closing keyword in the row's own PR body: PR1993's "fixes #1992"
   (B02.277) and PR2499's "Fixes #2498" (B09.092). Both reports were
   fetched with `inventory-b.py` and read in full, body and every comment
   (issue 1992 and its one comment; issue 2498 and its four). Issue 1992
   reports that a FIPS-mode provider offers X25519, so a FIPS server
   answers with a HelloRetryRequest; issue 2498 reports that
   `--wrap-static-fns` emits a C wrapper for a static variadic function
   that drops the `...`. Both are deterministic, single-threaded faults, so
   both rows stay E3 with confirming superseding lines. The other 25 are in
   the not-fetched classes: prior-work and release-list URLs (18: B02.108,
   B02.335, B02.342's eleven, B06.340, B06.447, B23.066, B23.140, B23.148),
   "see" links to other repositories or features (5: B02.303, B06.114,
   B06.152, B06.477, B23.142), the feature request that the regressing PR
   closes (1: B23.155's #211), and a discussion read by GraphQL rather than
   as an issue (1: B23.129's #794, whose digest is in its sources). The
   final `--all` run reports 306 rows checked and 61 absent references,
   the 36 above plus these 25, all in not-fetched classes. Outcome: B02.038, B02.213, B09.199 and B10.255 moved from E2 to
   E3 (each has a stated failure the abridged view or the missing report
   had hidden, and none is concurrent); B11.082 kept E2 after its source
   repair; the other rows were confirmed. Each change or confirmation is a
   superseding decision line naming the finding.
9. **Bookkeeping.**
   - Builder 8 reported two identical decision lines for B10.064. The
     pre-r1 decision files held one line for it (1,220 lines, 1,220 IDs).
     r1's superseding lines bring them to 1,518 lines, still 1,220 IDs, with
     four for B10.064: that one line and three r1 superseding lines. Either
     the report was mistaken or a line was removed; this record did not
     establish which. No count is affected.
   - Builder 8's interim checkpoint lines carried wrong clock times. Its
     final checkpoint replaced them with times from `date -u`.
   - Builder 10 changed two judgments that builder 9 had pencilled but not
     recorded: B24.711 became E2 and B23.146 became E3. Neither was ever an
     E1 line in the decision file.
10. **AI-report policies were not read.** The protocol asks that each pool
    crate's policy on AI-generated reports be recorded "at classification
    time". No find was made, so nothing was classified and the policies were
    not read. One thread (B02.031) points to
    `https://github.com/rustls/rustls/blob/main/CONTRIBUTING.md#ai-policy`.
    That URL is recorded here and was not assessed.

11. **Arm-B assessment order (Codex r1 B5).** The protocol registers pool
    order, then matched SHA in lexical order. Actual judgment order departed
    from it: builder 4 took darling_core and env_logger before uuid, and
    builder 5 judged bindgen (rank 9) rows that had saved evidence while the
    throttled acquisition chain was still fetching serde_core onward. The
    reason was evidence availability: rows were judged as their prepare and
    inventory queues finished. Row IDs keep the registered lexical order;
    `assessed_utc` records the actual chronology. No judgment depends on the
    order.
12. **Other r1 corrections.** B15.047 (plotters) failed E1 as "API
    improvement, no malfunction stated", but PR 43 quotes the E0277
    diagnostic "`std::rc::Rc<rusttype::Error>` cannot be shared between
    threads safely" and the fix changes `Rc` to `Arc` in non-test code. It now
    passes E1 and E2 and fails E3: a compile-time `Send`/`Sync` bound on an
    error type, with nothing running concurrently (Codex r1 B2). B09.162's E2
    quote was PR 1809's post-fix validation ("the second build failed due to
    `clang_sys` not finding any libclang"), and the issue it closes was not
    inventoried. Issue 1766 was fetched and read; its pre-fix observation
    ("`cargo build` did not rebuild any of the crates that depend on bindgen")
    is now the E2 quote, and the row still fails E3 (Codex r1 B3). Its
    duplicate B09.288 got the same quote. B09.297's inventory had been
    overwritten after its decision (its hash no longer matched); the original
    was rebuilt from B09.093's two issue-833 entries to the recorded SHA-256
    `590f7a8f...`, kept privately beside the overwritten file, and the row was
    then re-inventoried. B09.251's E2 fields now repeat B09.065's quote and
    source instead of "as B09.065".

## Privacy and disclosure check

- Every arm-B row is a commit that is already public in upstream history,
  and the ledger cites only public commit, PR, issue and discussion URLs.
  Before committing, the ledger's reasons and notes were searched for
  unfixed, open or security language. The only hit is a fixed rustls
  configuration-identity change (B02.031) and the AI-policy URL above.
- Arm A made no find. Its table describes porting obstacles, not suspected
  bugs.
- The record contains no private candidate, so the count of private
  candidates is 0.
- Not committed: everything under the private directory, including saved
  evidence bodies, raw HTTP responses, inventories, staging files, the
  arm-A maps and the clone manifest. The GitHub Discussion fetch (insta
  discussions/794, GraphQL, 21:48:08.767801697Z-21:48:09.371426697Z, response
  SHA-256 `0b4233d3c7822716180851a83f5f5e51307fccd591ab319e30f47da0a961c5ba`)
  is cited only by URL, time and digest. The untracked `*.ots.bak` files
  belong to tide-lead and are not committed.
- Nothing was posted or sent upstream.

## Follow-up

The desk decision of 20:47 UTC plans one more fx-zu4f round, a
landed-capability re-screen, filed as bead fx-zu4f.13. Its pool is to be
defined mechanically from fx-zu4f.5-.11's committed records. From this round
it would take B09.003 and arm-A rank 9 (bindgen), both cap:proc. This record
files nothing.
