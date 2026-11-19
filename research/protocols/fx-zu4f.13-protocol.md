# fx-zu4f.13: landed-capability re-screen protocol

Status: registered 2026-11-19 on branch fx-zu4f.13, before any row is
re-screened. The tide-fx-zu4f.13 drafter (Claude Opus) drafted it, and the
tide-fx-zu4f.13 builder (Claude Opus) registered it. The registration base is
`bcc758c016c421ce25f28786891aebd9da3d105c`, which merges main at
`0aa55425faf6fcb1db8bd1db4c869b66cdf26802`. That main holds fx-zu4f.10's
landed record: record commit `9dae1f1`, merge
`f85b3d311bb4e04ce19f8f4ea969602a3a99cdc9`. Before "go", tide-lead stamps
this file and the pool directory with OpenTimestamps, records their SHA-256s
on the bead and upgrades the proofs. This protocol may narrow the bead's
frozen criteria. It never widens them.

## 1. The pool was chosen after its rows were seen

This is stated first, as the bead requires.

- Every pool row was screened before this round existed. Each one failed its
  own round and is listed in that round's committed record.
- The pool is defined after those rows were seen. What triggered it was a
  capability landing on main (Proc::command, Net::get, Net::resolve), not
  the bugs. The bead filed the round on 2026-11-18 and named two members in
  advance: fx-zu4f.10's B09.003 and arm-A rank 9 (bindgen, both cap:proc).
- One pool row, chrono_anchor (fx-zu4f.5), was run again after that round.
  fx-zu4f.8 reached its capability-passing port at a revision that already
  had Proc::command, and the port stopped on rustc `E0277`, because
  `TaskError` had no `From<Cancelled>` (research/47 §1, RULING-0039). Its
  outcome at this round's R is therefore partly known before go (§4.4).
- A catch here is reported as "caught on re-screen after the capability
  landed". It is never reported as caught the first time. Whether it counts
  toward the RC's public catch is judged by fx-qlek.7's rule and its
  cold-reader pass, not by this protocol.

## 2. The pool

### 2.1 The rule (bead fx-zu4f.13, frozen 2026-11-18; comments bind)

- **Capability-only, mechanical.** The pool holds every row in the
  committed records of fx-zu4f.5 through fx-zu4f.11 (both arms) whose only
  first-failure reason is a capability Tide v0 lacked at that round's
  registered revision and that has since landed on main. A row with any
  other failing reason is not in the pool. Other reasons include async,
  external crates, a core over 400 lines, an OS trigger, memory unsafety
  and unresolved dispatch. Rows come from committed records only, never
  from private notes.
- **Matched by operation (2026-11-18 23:53Z).** Each network row is
  checked against the operation that actually landed. `Net::get` is one
  HTTP/1.1 GET. `Net::resolve` is a system DNS lookup. UDP has not landed.
- **UDP-only rows (relayed owner decision, 2026-11-19 00:14Z).** The
  record lists every row whose only remaining blocker is UDP (`bind_udp`,
  `UdpSocket` or UDP DNS), with its source and operation. Such a row is
  never counted as caught in this round. If one would meet its caught rule
  except for UDP, tide-lead files a UDP bead (00:15Z).
- **Resolve-only rows and fx-zu4f.8 (2026-11-19 00:21Z).** The pool also
  takes every fx-zu4f.8 row whose only remaining blocker is Net::resolve,
  or another capability that lands before go, by the same rule, judged at
  this round's R. Every resolve-only row is listed.
- **No Result-adaptation rule (relayed owner decision, 2026-11-19 05:14Z).**
  This round adds no rule for a table form that returns a `Result` the
  source did not, such as a `.map_err(..)?` into the enclosing error type.
  chrono_anchor's conversion gap is handled by fx-qlek.18 (the std fix) and
  fx-zu4f.15 (a fresh registered test). The rule set is unchanged.

### 2.2 How the rule is applied

`fx-zu4f.13-pool/select.py` (plain `python3 -I`, standard library only)
reads the
committed records and decides each row in three steps. It makes no network
call, runs no subprocess and does not use git.

1. **Gate.** A row passes only if its recorded first failure is the round's
   expressibility criterion:
   - E4, in the bug rounds (.6, .7, .9, .10 arm B, .11 arm B);
   - "not caught" after a ported or assessed capability-passing version
     (.5);
   - "capability absent" (.8);
   - "unportable", for an arm-A target (.10, .11).

   A row that failed earlier never enters the pool. Earlier failures
   include E1-E3, no portable source, port incomplete, no TD code and
   unresolved.
2. **Capability-only.**
   - Where the record has a capability column (.10's `cap`, .11's
     `cap_tag`), the round's own tag decides. The record's "Capability-only
     failures" table is part of the committed record, so a row tagged there
     but left blank in the ledger counts as tagged, and the mismatch is
     shown in the tag text.
   - Where the record has no such column (.5-.9, .8), the recorded reason
     must name a subprocess or network operation and none of the other
     grounds. The script lists both term sets.
   - Tagged records also go through the term test as a cross-check. A
     disagreement is reported, never resolved silently.
3. **Landed.** The operation's capability must be absent at the source
   round's revision and present on main.
   - A network row is matched by operation, in this order: UDP, then a
     system DNS lookup, then an HTTP GET.
   - A UDP row goes to the UDP-only list.
   - A network row whose operation the record does not name goes to a
     manual reading. None did.
   - Ancestry was checked with `git merge-base --is-ancestor` on 2026-11-19
     (§3.2).

The term lists are in `fx-zu4f.13-pool/select.py`. Every gated row that
names a capability or carries a tag is in `fx-zu4f.13-pool/screen.tsv`,
with its decision.

### 2.3 Inputs (registration run, 2026-11-19T07:09Z)

| Source | Committed record | SHA-256 |
| --- | --- | --- |
| fx-zu4f.5 | research/37-external-admission-search.md | `a2ac59f8…510c610` |
| fx-zu4f.6 | research/38-external-concurrency-search.md | `ad54b1ee…18be47a` |
| fx-zu4f.7 | research/39-std-concurrency-search.md | `a79aa358…10d84e1` |
| fx-zu4f.8 | research/47-malware-rescreen.md | `6c6111b7…3b896e` |
| fx-zu4f.9 | research/41-qin-dataset-search.md | `c88be55c…340c5d` |
| fx-zu4f.10 | research/44-fx-zu4f.10-concurrency-round.md (landed: `9dae1f1`, merge `f85b3d3`) | `249c1815…bc042` |
| fx-zu4f.10 | research/protocols/fx-zu4f.10-record/arm-b-ledger.tsv (landed, as above) | `5a884b74…ed95` |
| fx-zu4f.11 | research/46-fx-zu4f.11-agent-marker-round.md | `48f11d35…3f84dc` |
| fx-zu4f.11 | research/protocols/fx-zu4f.11-record/b-candidates.tsv | `20b19ebf…c45e` |

The full digests are in `fx-zu4f.13-pool/selection.json`. The script ran
with no `--ten-root` from the worktree's git-ignored `target/work`
directory, at the registration base:

`python3 -I select.py --root ../.. --out out-final`

The committed outputs are byte-identical to that run. selection.json keys
each input by its path relative to the directory the script ran in
(`../../research/…`). Running it from another directory changes only those
key strings.

**Draft run vs registration run.** The draft ran at 05:26Z on base
`04dde61`. It read fx-zu4f.10's rows from branch fx-zu4f.10 at `707ab78`,
marked provisional. The registration run reads the landed record. Every
count is unchanged, and so are the pool, the UDP-only list and the
resolve-only list. The outputs differ in three places only:
- The `provisional` column is empty for every fx-zu4f.10 row, and
  selection.json says `"fx-zu4f.10_provisional": false`. fx-zu4f.10's two
  input paths and SHA-256s changed.
- The landed ledger fills three `cap` cells that were empty at `707ab78`:
  B09.003 is `cap:proc`, and B16.018 and B16.128 are `not cap-only`.
  B09.003's tag now comes from the ledger. The record-table fallback still
  exists in the script, but it no longer fires. The two B16 rows were
  already excluded as "not capability-only (round's tag)".
- None of the 5,076 arm-B ledger rows changed its `first_failure`. Two
  reasons were reworded, and neither changes a decision: B06.397 fails at
  E1, and B10.064 is untagged. fx-zu4f.10's arm-A table and its
  "Capability-only failures" table are byte-identical between the two
  copies.

### 2.4 Result of the registration run

| Source | Rows read | Passed gate | Name a capability | Pool | UDP-only | Resolve-only |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| fx-zu4f.5 | 87 | 12 | 10 | 1 | 0 | 0 |
| fx-zu4f.6 | 32 | 15 | 7 | 0 | 0 | 0 |
| fx-zu4f.7 | 162 | 26 | 0 | 0 | 0 | 0 |
| fx-zu4f.8 | 87 | 1 | 1 | 0 | 0 | 0 |
| fx-zu4f.9 | 100 | 68 | 4 | 0 | 0 | 0 |
| fx-zu4f.10 | 5,100 (5,076 B + 24 A) | 18 | 5 | 2 | 0 | 0 |
| fx-zu4f.11 | 49 (42 B + 7 A) | 13 | 6 | 1 | 1 | 0 |
| **Total** | | | | **4** | **1** | **0** |

The cross-check found no disagreements in the draft's final run or in the
registration run. The draft's first run found three, all parser or
term-list defects, and all three were fixed before its final run:
- a second fx-zu4f.10 arm-A table was read twice;
- "External", capitalized, was missed;
- "no mapping" was not in the list of other grounds.

**Pool**

| # | Source, arm | Row | Capability (bead, landing commit) | Recorded reason |
| ---: | --- | --- | --- | --- |
| P1 | fx-zu4f.5, P1 | RUSTSEC-2026-0039 chrono_anchor | Proc::command (fx-ayh2.1, `e0b0d62`) | the capability-passing port fails only on `E0599`: `Proc` has no `command` in v0 |
| P2 | fx-zu4f.10, B | B09.003 bindgen `02da6536e25a` | Proc::command (fx-ayh2.1, `e0b0d62`) | the parent waits on a clang child whose stdout pipe is full. The ledger's `cap` is `cap:proc` |
| P3 | fx-zu4f.10, A | rank 9, bindgen 0.73.2 | Proc::command (fx-ayh2.1, `e0b0d62`) | the core is the rustfmt child's stdin/stdout pipes |
| P4 | fx-zu4f.11, B | rank 7, confeden/Antigravity `fbcbbb8b47d0` | Proc::command (fx-ayh2.1, `e0b0d62`) | `Command::output()` waits forever on a hung PowerShell child. Tag provisional in .11: the path union was not counted |

**UDP-only (listed, never counted)**

| Source | Row | Operation |
| --- | --- | --- |
| fx-zu4f.11, B | rank 7, confeden/Antigravity `12a36c940b10` (parent `5ab40fac5439`) | racing provider threads send UDP DNS queries (`dns_client.rs` `UdpSocket` at the first parent, per the record and fixer 2) |

**Resolve-only:** none. This agrees with fx-zu4f.8's own report. No
gated row in any round names a system DNS lookup as its only blocker.

**Excluded rows that name a capability** (all in
`fx-zu4f.13-pool/screen.tsv`):
- fx-zu4f.5's amaperf crates, evm-units and sha-rust: external crates, plus
  async and unsafe for evm-units. sha-rust's UDP sits beside `reqwest`, so
  it is not UDP-only.
- fx-zu4f.6's A1, A6, A7, A12, A15, A19 and A24: Python, signals, Unix
  sockets, kernel RST, async and external crates.
- fx-zu4f.8's evm-units: `reqwest` with TLS and a header, `env::temp_dir`,
  tokio.
- fx-zu4f.9's B09, B13, B45 and N18: external crates, OS buffers and lock
  shapes.
- fx-zu4f.10's B16.018, B16.128 and B23.129: kernel behavior, OS errors and
  Mutex poisoning. The round tagged them "not cap-only".
- fx-zu4f.11's `8566ded` (TCP with OS RST) and arm-A ranks 2, 3 and 7:
  external crates and unmapped operations.

## 3. Revision and capability set

### 3.1 The run revision R

R is main's HEAD when tide-lead sends "go". tide-lead records it before
any port or model exists and without any result. The branch merges main at
R. The record states two things:
- `git diff --quiet R HEAD -- crates Cargo.toml Cargo.lock rust-toolchain.toml`
  holds at the end;
- the tracked tree is clean.

Every row is judged against v0 at R.

### 3.2 Capability set (checked at registration, rechecked against R)

| Capability | Owning bead | Landing commit | At the source revisions |
| --- | --- | --- | --- |
| Proc::command (`Command`, `Child`, pipes, statuses; real and simulated) | fx-ayh2.1 | `e0b0d6216bed121c784c76e27b10582a2e0053ed` | absent at .5 `5dc6b40`, .6 `7863e8f`, .7 `afb17ea`, .9 `1e673c5`, .10 `a28b4df`, .11 `ea7bb31`; present at .8's R `972aba0` |
| Net::get | fx-ff13 | `8ac13175c98ec29b3f313a860d657e1c34c1fafe` | as Proc::command |
| Net::resolve | fx-6d50 | `d378a7d39e0edc70cbb409e53d28ad042f0c1606` | absent at every source revision, .8's R included |
| UDP (`bind_udp`) | none | not landed | absent |

At registration, `git merge-base --is-ancestor <commit> bcc758c` holds for
all three landed commits (`e0b0d62`, `8ac1317`, `d378a7d`). The selection ran
on main `0aa5542`, merged at `bcc758c`. The record repeats the check against
R.

### 3.3 Go order

1. Done: fx-zu4f.10's record landed on main (merge `f85b3d3`), and the
   selection was rerun on it (§2.3).
2. Commit, stamp, record the SHA-256s and upgrade the proofs.
3. "go" under the agent cap. fx-ff13 and fx-6d50 have landed, so the bead's
   network and resolve conditions are already met. If another capability
   lands before go, the selection is rerun and registered again before go,
   never after.

## 4. Rules each row is judged by

### 4.1 The source round's rules, unchanged

Each row is judged by its source round's registered protocol, rulings and
caught rule, applied as written to v0 at R. "The capability's landing commit
named" means the record cites the landing commit (§3.2) in each row's E4
(or leg-2) entry.

| Row | Protocol (SHA-256) | Criteria | Caught rule |
| --- | --- | --- | --- |
| P1 chrono_anchor | fx-zu4f.5's operational protocol, reproduced verbatim in research/37 (`0e5cebada4f0c379f72480a10331a55a18fdff37377659510f9cea0b6b424fb0`) | classes, then the faithful-port test ("Changes are limited to Tide syntax and std requirements, including the smallest type/import scaffolding") | BOTH a named TD code on the faithful buggy port AND admission of the capability-passing port. An unrelated diagnostic does not count; an incomplete check stays unresolved |
| P2 B09.003 | fx-zu4f.10's protocol (`44490b7ddf5b73844d2f7057bd950bfa15fe54321a29fdf5809a8a7ef82b73af`; registered at `63eb32e`, on main through merge `f85b3d3`) | E1-E5 in order (§4 there), with fx-zu4f.9 §§4-5 and 7's faithful mappings | the buggy model fails the frozen property on a seed in 0..999; one recording and two byte-identical driver replays of the lowest failing seed; the fixed model passes all 1,000 seeds |
| P3 bindgen arm A | fx-zu4f.10's protocol, arm A and its §7 real-crate reproduction | arm-A core location, port and frozen properties | the port fails the property; two byte-identical replays; the fault-removed control passes all 1,000 seeds; AND the failure reproduces on the real code at the pinned commit (§7 there) |
| P4 fbcbbb8 | fx-zu4f.11's protocol (`40e3e6612a219924c5f3255ddb58d4004b6ddd48de6c92188e2a4f191709753b`) with amendments A1-A2 | E1-E5 in order (§4 there); E3 was tentative in .11 and is re-judged | as P2 (§5 there) |
| UDP-only `12a36c9` | fx-zu4f.11's protocol | listed only | never counted |

- **Re-judged from E1.** Each row is judged from its first criterion, in
  the source round's order. Earlier passes are a starting point, not a
  result. Where the source record marked a pass tentative (fbcbbb8's E3) or
  left something uncounted (the path unions of P2-P4), that check is done
  in full before any later criterion is relied on.
- **The only change: the capability ban.** The source rounds' E4 bans every
  subprocess and network operation; .10 and .11 call it a frozen
  experiment criterion. This round lifts that ban only for the operations
  the landed capability provides, through the mapping in §5. Everything
  else still holds:
  - the 400-line cap, counted over the full union;
  - the bans on external crates and on async;
  - the OS-trigger, memory-unsafety and unresolved-dispatch exclusions;
  - the reconstruction ban and the historical-fidelity rule;
  - every network or subprocess operation outside §5.
  Nothing else is relaxed.
- **Rulings carried over.** Where a source round issued rulings, they apply
  to its rows. fx-zu4f.8's RULING-0039 and RULING-0175 bind only .8's rows,
  and no .8 row is in the pool. fx-zu4f.5's text has no ruling on
  conversions; see §4.2.

### 4.2 No Result adaptation

fx-zu4f.5's faithful-port text allows changes for "Tide syntax and std
requirements". In this round, that text never covers adding a conversion
the source did not perform: no `.map_err(..)`, no `let _ =`, no added `?`
conversion. This follows the owner's decision of 2026-11-19 (§2.1). If a
table form's `?` fails at R only because a conversion is missing, the row is
**not caught (std conversion gap)**, and the failure is quoted. This is a
narrowing.

### 4.3 Order of work after go

1. Rows in pool order (P1-P4). The UDP-only row is only checked for its
   operation: does the record's path use UDP, and is no other blocker
   named? It is not ported.
2. Freeze each row's property, trigger and child-model data (§5.3) in the
   property log, with SHA-256 and time, before any run, as the source
   round requires.
3. Every compile or run goes through heavy.sh (AGENTS.md, Heavy runs), with
   the assigned TARGET, a timeout and closed stdin.

### 4.4 chrono_anchor (P1): what is known before go

Under the bead's rule, fx-zu4f.5's row is in the pool. fx-zu4f.8 is a
separate committed record, whose own chrono_anchor row failed for a
non-capability reason, so it is not in the pool. The record discloses what
was known before go:
- fx-zu4f.8's `E0277` at `972aba0`;
- whether fx-qlek.18, the std fix for `TaskError: From<Cancelled>`, is an
  ancestor of R. At registration it already is on main (`4c5b8c0`, an
  ancestor of `bcc758c`), so it will be in R. The record confirms this
  against R.

If it is, P1's outcome and fx-zu4f.15's are not independent tests. The
record says so, and a catch is counted once, under whichever registration
the fx-qlek.7 reader accepts. P1 is ported from the Socket archive copy
(socket-3), so §6's archive condition applies.

## 5. Capability mappings (the only new mapping rows)

### 5.1 Proc::command (fx-ayh2.1, at R)

| Source operation | Allowed mapping |
| --- | --- |
| `std::process::Command::new(p)` | `proc.command(p)`, on a `&Proc` passed in from `main` or the test. The added parameter is capability-passing scaffolding |
| `.arg`, `.args`, `.stdin`/`.stdout`/`.stderr` with `Stdio::piped()`/`null()`/`inherit()`, `.spawn()`, `.output()`, `.status()` | the same method at R (companion §20; `crates/tide-std/src/process.rs`) |
| `Child::wait`, `try_wait`, `kill`; reading `child.stdout` and writing `child.stdin` | the same operation at R |
| anything else, such as platform `CommandExt` extensions (`creation_flags`, `pre_exec`), `ExitStatusExt::signal`, inherited stdin under simulation, or pipe chaining between children | **no mapping**: E4 fails (unmapped operation) |

### 5.2 Net::get and Net::resolve (at R)

- **Net::get.** A std-only HTTP GET does not exist; HTTP in Rust code goes
  through a crate, and crates stay banned. So Net::get maps nothing in this
  round's pool, and no row needs it.
- **Net::resolve.** `ToSocketAddrs` or `lookup_host` maps to
  `net.resolve(host, deadline)`. The added deadline is the table form's
  argument, and the source's result shape must convert without an
  adaptation (§4.2). No pool row uses it.

### 5.3 The simulated child

The external program (clang, rustfmt, PowerShell) is not Tide code. Under
simulation it is a registered program model of fx-ayh2.1's simulated
backend (research/40 §6):
- a finite pipe buffer, with its capacity recorded in the manifest;
- backpressure;
- statuses.

The model's data may state only what the row's evidence states, such as
"writes more than the pipe holds before reading stdin" or "never exits".
The data is frozen with the property before any run and listed as an
adaptation. It may not be tuned after a run. An effect the v0 model
cannot declare is a coverage gap: E5 fails, with the unmodelled-effect
report quoted. The pipe capacity is the backend's default at R, recorded,
never chosen per row.

## 6. Disclosure and privacy (as fx-zu4f.10 and fx-qlek.7)

- **Private directory.** `~/.cache/eon-agents/tide-lead/fx-zu4f.13-private/`
  holds every finding about a bug still unfixed upstream, from its first
  write. That covers any arm-A result for P3 and anything new found on
  P2-P4's paths, together with their models, properties, traces,
  reproduction sources and draft texts. Nothing undisclosed is ever written
  to WORK, the worktree or a commit, not even staged and later deleted.
- **Danger classification.** Each find is classified dangerous or not,
  with reasons, in the private record, as in fx-zu4f.11's "Disclosure and
  privacy". When in doubt, it is dangerous.
- **Reports.** Agents draft; Lucca alone sends. A dangerous find goes
  through SECURITY.md, then GitHub private vulnerability reporting, then a
  maintainer email. The record states each pool repository's AI-report
  policy, with its URL and date.
- **What the public record says.** It names only fixed or already-public
  bugs. P1 is a public RustSec advisory, and P2 and P4 are fixed upstream.
  For private finds it gives a count only. Only a public catch counts
  toward the RC.
- **Archive condition (P1).** tide-desk's conditions on the four Socket
  archive copies apply. Ports use inert placeholders for hosts, paths,
  programs and literal arguments. Archive-derived ports stay in WORK and are
  never committed. A port whose affected function makes a network operation
  under that condition is **unresolved (archive condition)**.
- **Owner-affiliated repositories** (Yazelix, luccahuguet, or any org the
  owner adds) are excluded up front and never backfilled. The pool has
  none. At registration, the pool and UDP-only rows come from
  rust-lang/rust-bindgen, confeden/Antigravity and the chrono_anchor
  RustSec crate.
- **Neutral framing.** State the selection; never judge the authors.

## 7. Record

- The post-hoc pool statement (§1) comes first.
- Negatives come next: every pool row with its outcome and the criterion
  that decided it, then any catch.
- Then the UDP-only list (row, source, operation, and whether it would
  otherwise meet its caught rule, the UDP trigger) and the resolve-only
  list, even when empty.
- Counts describe this selection only, never a rate.
- The record gives R, the capability set at R, each landing commit, and
  whether fx-qlek.18 is in R.
- The branch lands by `git merge --no-ff` after GOVERNANCE.md's Merge Rule.
