# fx-zu4f.14: post-quantum crypto Rust crates, protocol

Registered on branch `fx-zu4f.14`, based on main
`27881b6a4365fd3c2af81d9716cce01dfc117969`, before any candidate repository's
history, issues or code was examined beyond the mechanical search, pin and
scan below. The frozen criteria in `br --no-db show fx-zu4f.14` own the
contract; this file adds operations and marked **narrowings**. It never
widens the repositories, the classes, the pool or a caught rule. Where this
file and the bead disagree, the bead prevails and the record discloses the
disagreement.

## Amendment R1 (2026-11-19, re-registration before any run)

This file supersedes the registration at commit
`e1c04a4a212a68ff2d25ed1b0453a8ffe3208790` (proofs `edd3804`, `bef9bcb`,
kept in history). The Codex supervisor pass on that registration
(`gpt-6.1-sol`, high, 2026-11-19, request_changes) found two blocking
issues; both changed registered rules, so the round re-registers:

1. **PQ eligibility.** The registered gate admitted a package on a lexical
   discovery token alone (any of twelve tokens anywhere in a name,
   description, dependency name or identifier), which over-approximates the
   frozen criterion "implementing a NIST post-quantum algorithm or a
   post-quantum protocol"; ambiguous tokens such as `falcon` admitted
   packages with no recorded post-quantum implementation. **Change:** §3's
   gate now has a second, required step: recorded implementation evidence
   with file and line (`own-op` or `dep-use`, strict disambiguated names;
   `classify.py` `pq_impl_evidence`), evaluated only on packages that pass
   the old gate, so it can only narrow. `classify.py --check-narrowing`
   checks this against the registration's `classes.tsv` row by row; it
   passed. Classes C and P, ordering and caps are unchanged. classify.py
   (SHA-256 `9470eaf0cff43ede6cf59657206afc74dcb36261dc8ef183881f8cffe7f01384`)
   and the rule text of §3's gate, §7's reproduction and Safety were fixed at
   2026-11-19T20:33:01Z, before the regeneration at 20:33:04Z to 20:34:07Z;
   after it only result counts, table-column descriptions, §8's count
   units, §2's names for
   the drop reasons classify.py already emitted, and this amendment were
   written (self-attested, like every local time).
   Regenerated from the recorded inputs (the pinned clones, candidates.tsv,
   clone-refs.tsv): `classes.tsv`, `repositories.tsv`, `pool.tsv`. No row was
   dropped or added by hand. Counts, registration -> R1: repositories after
   the gate 16 -> 9; packages passing the gate 38 -> 14; class C 3 -> 0;
   class P 9 -> 3; pool 12 -> 3 (arm A all 3; arm B no target). The search,
   repository filter, pins and their files are unchanged.
2. **Network boundary.** §7 allowed upstream compilation and execution with
   `--offline` and proxy variables when `unshare -rn` was unavailable, which
   does not stop direct sockets. **Change:** that fallback is removed. Every
   upstream compilation and execution runs only inside an `unshare -rn`
   namespace whose own shell first proves it has no route; otherwise the
   reproduction is blocked and the candidate stays model-only (fail
   closed). The read-only fetch stage is kept separate and runs no upstream
   code (§7, Safety).
3. Non-blocking, applied: units. `classes.tsv` holds 751 package rows plus
   1 `no-Cargo-package` placeholder row, reported as such (§3).

Observed after the R1 scan (classify.py's) and disclosed here, rule
unchanged: `op_name` is lexical, so a fn whose name only contains an
operation word (for example a key accessor such as `encapsulation_key`)
counts as an operation; the pool's step-2 evidence column shows each
admitted package's first evidence.

Between the registrations nothing else happened: no repository content
beyond the registered mechanical scans (classify.py over the pinned sparse
checkouts) was examined, no history, issue, PR, documentation, source or
policy was read, no admission-observation run, model, property, build or
upstream execution took place, and no network request was made. The R1
author read the Codex report, the registration's committed files and
classify.py's outputs.

## Amendment R2 (2026-11-19, re-registration before any run)

This file supersedes R1 at commit
`6ae022a940eaf53d2f7e743045bf38ada421b69e` (proofs `95d074a`, kept in
history) as well as the registration. The Codex finder re-check of R1
(`gpt-6.1-sol`, high, 2026-11-19, request_changes) found R1's network fix
resolved and its eligibility fix **not resolved**: R1's step 2 admitted a
function by its name plus an enclosing header or file path, so an empty
body, a trait or extern declaration or an accessor (such as
`encapsulation_key`) passed, and `dep-use` admitted a bare import. A name
or an import is not evidence that a crate implements a listed algorithm or
a post-quantum protocol step.

**New evidence standard (tide-lead's decision, applied as §3 step 3).** A
package is admitted only among R1's step-2 survivors, and only when its
production source holds, inside a function definition with a code body
(not empty, not a declaration, not an accessor), either a published
constant of a listed algorithm (`impl`: ML-KEM q 3329, ML-DSA q 8380417,
FN-DSA q 12289, SLH-DSA signature lengths, HQC and Classic McEliece code
lengths, each cited in §3) as a literal or a named constant with that
value, or a call to an operation of a dedicated post-quantum implementation
crate through its path or a name imported from it (`delegate`). File and
line of the constant or call and of the enclosing fn are recorded in
`pq_code_evidence`. Steps 1 and 2 are unchanged and stay as pre-filters.
The self-test now rejects empty bodies, declarations, accessors, bare
imports, type mentions, literals inside strings and calls into a
general-purpose library, and accepts one fixture per evidence kind (a
literal, a named constant, a hex literal; a path call, an aliased call, a
bare imported call, a turbofish call). §3 ("What step 3 cannot see") states
plainly what the standard misses: implementations reached only through
macros, `include!`, generated code, constant expressions, other crates of
the repository, method calls on values, trait-qualified calls, re-exports,
glob imports, general-purpose libraries or differently named operations;
and a listed value can occur in unrelated code.

Also applied (non-blocking from the re-check): the isolation probe tests
`/proc/net/ipv6_route` by content (`! grep -q .`), not by procfs size (§7).

classify.py (SHA-256
`4f450078da10bb98fef6a97385c8fbbd13f7266c79cc46f10ac5576138cadd4f`) and the
rule text of §3's step 3 and §7's probe were fixed at
2026-11-19T20:46:36Z, before the regeneration at 20:46:40Z to 20:47:47Z;
after it only result counts, table-column descriptions, §2's drop-reason
names and repository counts, §8's units and this amendment were written
(self-attested). Regenerated from the recorded inputs: `classes.tsv`,
`repositories.tsv`, `pool.tsv`; nothing was dropped or added by hand.
`classify.py --check-narrowing` passed against R1's `classes.tsv` and
against the registration's (0 changed fields, 0 widened rows).

Counts, registration -> R1 -> R2: repositories kept by the gate 16 -> 9 ->
6; packages past the gate's last step 38 -> 14 -> 6 (R2: 1 `impl`, 5
`delegate`); class C 3 -> 0 -> 0; class P 9 -> 3 -> 2; pool 12 -> 3 -> 2,
both class P, both by `delegate` evidence. Arm A takes both; arm B has no
target. The gate is not loosened to refill the pool.

Between R1 and R2 nothing else happened: no repository content beyond the
registered mechanical scans (classify.py over the pinned sparse checkouts)
was examined, no history, issue, PR, documentation, source or policy was
read, no admission-observation run, model, property, build or upstream
execution took place, and no network request was made. The R2 author read
the Codex re-check, R1's committed files and classify.py's outputs.

## Amendment R3 (2026-11-20, text only, before any run)

R3 changes only this file's text. It supersedes R2's text of this file at
commit `5b9298b497e86239deb5efbd4002f598b724b0a2` (proof pending at
`e620d56`, upgraded at `5edef08`, both kept in history). R2's pool files,
classify.py, the PQ gate, the classes, the pool, the eligibility rules and
the caught rule are unchanged. The Opus review of R2 (Claude Opus 5.5,
2026-11-19, changes required) found two blocking issues, both in the text:

1. **B1, public signal.** Arm A has two crates, and the committed
   `pool.tsv` names both with their repositories, pinned commits and PQ call
   sites. The Disclosure section let the ledger and commit text give arm
   totals and the public record give a count of private candidates, and §8
   required every pool crate's outcome, so a withheld rank would stand out.
   Any of these would tell a reader of the public history that a private,
   dangerous-by-default find sits in one of two named functions.
   **Change:** Disclosure now holds a public-signal rule for this round
   ("Public signal for arm A"). From go until every arm-A pool crate has
   its final outcome and every arm-A candidate is classed non-dangerous and
   reviewed, or disclosed, the ledger, commit text and
   the public record give no arm-A outcome, total or private-candidate
   count; their only arm-A line is the fixed text `arm A: in progress`, the
   same in every case; and the record states the residual timing leak. The
   Read-first disclosure note, the method-reference note, the Committing and
   Ledger rules, §7 step 5 and §8 defer to it.
2. **B2, what the pool can test.** §3 said the gate exists "so that the
   pool holds post-quantum code", and nothing said that R2's pool puts no
   post-quantum implementation under test. Both pool crates were admitted on
   `delegate` evidence into `libcrux-ml-kem` (`pool.tsv`, step-3 column),
   and §7 step 2 makes a row unportable when an external crate's code is on
   its path. **Change:** §3's gate sentence says the gate admits packages
   that implement, or call into a dedicated implementation of, a listed
   algorithm. "Read first" and §8's "does not show" list state the
   delegation and its consequence: every row that reaches a PQ operation is
   unportable, so no ML-KEM implementation (or any other listed
   algorithm's) is tested; arm A reaches only each crate's own non-PQ code;
   and the randomness angle is answered only descriptively, by §5. §8
   requires the record to state the re-registration history next to the
   pool list.

Non-blocking notes, applied as text: N1, §0 and this amendment name the
registration set the go certifies (below); N2, R1's note now says "R1 scan"
where it meant classify.py's scan; N3, §8 says that the per-crate listing
puts negatives first; N4, §3 says that a later round reusing this gate
treats parameter-size constants as weaker evidence than arithmetic
constants.

**Registration set the go certifies (N1).**
- The 8 files unchanged since the registration at commit
  `e1c04a4a212a68ff2d25ed1b0453a8ffe3208790`, all in POOLDIR:
  `admission-obs.py`, `candidates.tsv`, `clone-refs.tsv`,
  `crates-io-hits.tsv`, `queries.jsonl`, `search-hits.tsv`, `search.py`,
  `star-slices.json`; proofs pending at `edd3804`, upgraded at `bef9bcb`.
- R2's five files at commit `5b9298b497e86239deb5efbd4002f598b724b0a2`:
  POOLDIR's `classify.py`, `classes.tsv`, `repositories.tsv` and
  `pool.tsv`, plus R2's text of this file, which R3 supersedes and history
  keeps; proofs pending at `e620d56`, upgraded at `5edef08`.
- This file as R3 amends it, at the commit that adds this amendment, with
  its OpenTimestamps proof once tide-lead stamps it and upgrades the proof.
  R3a (2026-11-20, before any run) supersedes that text: the finder
  re-check of R3 found that the public-signal hold had no start and that
  its end condition held while arm A had no candidate yet, so interim
  arm-A totals stayed permitted during the run. R3a states that the hold
  starts at go and ends no earlier than arm A's last outcome. The go
  certifies this file at the R3a commit, with its own proof; R3's pending
  proof is kept as `fx-zu4f.14-protocol.md.r3.ots`.

Between R2 and R3 nothing else happened. The R3 author read the Opus
review, the fixer briefs, the bead and its comments, and this round's
committed files, including `pool.tsv`'s step-3 evidence; nothing beyond that
committed evidence was examined. No repository content, history, issue, PR,
documentation, source or policy was read; no scan, admission-observation
run, model, property, build or upstream execution took place; and no
network request was made.

WORK = `/mnt/build/eon-agents/worktrees/tide-fx-zu4f.14/target/work`.
TARGET = `/mnt/build/tide-agents/fx-zu4f.14-target`.
PRIVATE = `~/.cache/eon-agents/tide-lead/fx-zu4f.14-private/`.
POOLDIR = `research/protocols/fx-zu4f.14-pool/`.
The builder is Claude Opus (bead comment, 2026-11-18). Only tide-lead writes
beads, runs governance reviews, stamps registration and lands the branch.
The builder starts no subagents and posts nothing upstream.

## Read first: what this round cannot see, and what it expects

- **Out of class.** Constant-time and side-channel bugs (for example
  KyberSlash's secret-dependent division), timing leaks, cache and power
  leaks, and fault attacks are outside this round. Tide's model sees values,
  not time, cache state, power or injected faults, so a port that passes
  every seed says nothing about them. No find of that kind is claimed, and
  no negative result is read as evidence against one.
- **Low expected yield.** The bead says so, and the record says so first.
  Published post-quantum implementations are tested against the standards'
  known-answer tests (KATs), many are small and heavily reviewed, and their
  arithmetic usually runs through hashing (SHA-3/SHAKE) from an external
  crate, which makes a property unportable under §7 step 2. Many pool crates
  are expected to end as unportable or not assertable. A documented
  negative is the expected outcome.
- **Tide's angle: randomness as a capability.** In Tide, code draws
  randomness only from a `Rand` capability it is handed; simulated draws come
  from the test's seed (design/reference.md §6.1). The round asks whether
  that model exposes keygen, encapsulation or signing that reach ambient OS
  randomness, RNG reuse, or determinism bugs that a seeded replay shows. The
  admission observations (§5) count where ambient randomness enters such
  code; they are descriptive and never a catch.
- **What this pool can test (R3).** Both pool crates delegate to
  `libcrux-ml-kem`: each was admitted on `delegate` evidence, a call to one
  of its ML-KEM operations, and neither shows `impl` evidence (`pool.tsv`,
  step-3 column). By §7 step 2, every row whose path reaches a PQ
  operation is unportable, so no ML-KEM implementation, or any other listed algorithm's,
  is tested. Arm A reaches only each crate's own non-PQ code, and the
  randomness angle above is answered only descriptively, by §5's admission
  observations.
- **The forward hunt (§7)** writes every property before any run: the
  crate's own KATs as oracles, plus properties quoted from its docs and
  tests. The property list is hashed and timestamped in PRIVATE before any
  port, build or run, and nothing is added after a run.
- **The caught rule is fx-zu4f.12's** (§7 step 6): a failing seed, the lowest
  failing seed replayed twice byte-identically, a control model that removes
  only the suspected fault and passes every frozen property on all 1,000
  seeds, AND reproduction on the real crate at its pinned commit. A
  model-only candidate is never reported as a catch.
- **Safety** (below): published test vectors only, no live keys, no network,
  nothing that can sign or broadcast anything real.
- **Disclosure** (below): crypto finds are dangerous by default; drafts stay
  in PRIVATE; Lucca alone sends; a private find counts toward the RC only
  once it is fixed and disclosed. For this round, the public-signal rule
  (R3) holds back every arm-A outcome, total and count until it releases
  them.

Method references, read-only, with SHA-256 (paths on main at the base):
- fx-zu4f.12 protocol, `research/protocols/fx-zu4f.12-protocol.md`,
  `09a4f5b957a1496f464e5dc93f65c0c623edbdd690b0217a8f77ff8d10361dc6`
  (called P12 below; "P12 §n" is its section n);
- its pool tools: `scan.py`
  `a7a983011884e1ac2df3d0d63e6b187ca2a098c6d4b41a03de2c6af870c696c4`,
  `admission-obs.py`
  `5823c3440408412ee5d25b3ac712141ffd67f1fdc7906a39080c8949a23b5fc9`,
  `search.py`
  `524314e73c74b0feeb98c42b2a8f314fafc3d0528f804e754b258bf579141570`;
- fx-zu4f.10 protocol, `research/protocols/fx-zu4f.10-protocol.md`,
  `44490b7ddf5b73844d2f7057bd950bfa15fe54321a29fdf5809a8a7ef82b73af`
  (P10; P12 adopts its arm-B judging and caught rule);
- `research/52-fx-zu4f.12-crypto-crates-round.md`
  `4ae63eecfc2e47d7855cc0831bed44c83ecd33b022d672ab5f1d1f6c9936ff8e` and
  reviews/fx-zu4f.12-r1-codex.md, -r1-recheck-codex.md, -r2-opus.md and
  -r2-recheck-opus.md, read for the errors they caught. This protocol fixes
  them in advance: arm B reads every row's complete evidence, E1 included
  (§6); property sources that disagree each give their own row, with no
  conflict exclusion (§7 step 1); KAT checks and fixed-input rows are
  registered and reported apart from generated-input tests (§7 step 5);
  policy sources carry URLs (Disclosure); the record opens with disclosure,
  then limits (§8); tracker comments carry arm totals only, and for arm A
  in this round nothing beyond the public-signal rule's fixed line
  (Disclosure, R3).

## 0. Registration boundary

Step 1 ended with one commit (`e1c04a4`) holding this file and POOLDIR,
byte-identical to WORK, and WORK/PROTOCOL-READY named the commit, every
committed file's SHA-256, the times and the counts. Re-registrations R1 and
R2 and the text amendment R3 replaced files as their amendments say. The go
certifies exactly the registration set that Amendment R3 lists: the 8 files
unchanged since `e1c04a4` (proofs upgraded at `bef9bcb`), R2's four pool
files at `5b9298b` (proofs upgraded at `5edef08`), and this file as R3
amends it, with its own proof. Before tide-lead stamps the files with
OpenTimestamps, records their hashes on the bead, upgrades the proofs and
sends "go", nothing else happens: no history listing, issue, PR,
documentation or source reading beyond classify.py's mechanical pass, no
security-policy reading, no admission-observation run, no model, no
property, no build and no upstream execution. Git dates and local logs are
self-attested; a timestamp shows a file existed by its attested time, not
that no earlier private screening happened. The record states the first
per-crate examination time and the proof identities, without backdating.

Pre-registration reading was limited to: AGENTS.md, GOVERNANCE.md, the
builder rules and search-round template, the bead and its comment, the
disclosure comments on fx-zu4f.10 and fx-qlek.7, the method references above,
design/reference.md §6.1 and §10.2 (to state the `Rand` capability and name
the seeded test form), GitHub repository-search and repository metadata,
crates.io search results, and classify.py's outputs. Earlier rounds' records
name some known bugs, and public writing names some known post-quantum
implementation bugs; no pool crate is assessed from that knowledge.
classify.py alone opened the checkouts; the builder read no per-crate
source, documentation, history, issue or PR. Besides the committed outputs, the
builder read only request metadata (HTTP statuses, totals, the crates.io
error body) to diagnose the superseded first run (§1).

## Disclosure and privacy

The disclosure policy recorded on fx-zu4f.10 (2026-11-18 16:52, 16:54 and
16:56 UTC, relayed owner decisions), fx-qlek.7 and P12 applies unchanged,
with the bead's crypto rules:

- **Full public history.** The repository goes public with its full history,
  so nothing undisclosed is ever committed. Every arm-A candidate and every
  arm-B record of a still-unfixed bug is written ONLY to PRIVATE, outside
  every worktree, from its first write: per-crate working notes, KAT and
  property lists, model sources, traces, reproduction sources and draft
  texts. Never stage and later delete such a file; history keeps it. Build
  products of private models go under TARGET; no provisional copy enters the
  worktree. Earlier versions of private models, property lists and compile
  logs are kept in PRIVATE, never overwritten.
- **Dangerous or not.** A find is dangerous if untrusted input or a remote
  peer can trigger it, or it causes memory unsafety, data corruption or loss,
  or denial of service in a library used by servers. **Crypto finds are
  dangerous by default**: anything that could affect keys or their
  generation, randomness, shared secrets, ciphertexts, signatures or their
  verification, or a protocol handshake is dangerous unless the private
  record shows why not. When in doubt, dangerous. The private record states
  the class with its reasons; the reviewers check it on the private copy.
- **Security and AI-report policies.** For each pool repository, after "go"
  and before any report draft for it, the private record lists its security
  program (SECURITY.md, GitHub private vulnerability reporting, a bounty
  platform, a security contact page) and whether CONTRIBUTING, issue
  templates or a stated policy reject AI-generated reports. Each entry
  gives the source URL (a pinned-commit file URL where the source is in the
  repository), the UTC date and time read, and whether the document was read
  or only named as a destination and not fetched. The program's rules bind
  the draft (scope, channel, embargo, proof-of-concept limits). No policy is
  read in step 1.
- **Drafts.** Agents only draft, into PRIVATE. Lucca alone sends or files;
  tide-desk hands him each draft. A dangerous draft uses the program's or
  SECURITY.md's channel, else GitHub private vulnerability reporting, else a
  maintainer email; nobody contacts any of them from this run. A
  non-dangerous find gets a public-issue draft, or, in a project that rejects
  AI-generated reports, no issue and a public listing with the cited policy.
- **Committing.** A non-dangerous record is committed after Lucca files the
  issue (or, for a project that rejects AI reports, once classified and
  reviewed). A dangerous record is committed only after disclosure (a fix is
  out, or 90 days pass, through Lucca). Until then the public record gives
  only a count of private candidates, with no project, crate or revealing
  detail; in this round the public-signal rule below replaces that count,
  and the public record gives none. Never say a report was sent because its
  draft exists.
- **Counting.** A private find counts toward the RC only once it is fixed and
  disclosed (bead; fx-zu4f.10's 16:56 UTC decision). It must meet the same
  caught rule. When Lucca sends a private report, tide-lead stamps its
  SHA-256; stamp and hash stay outside the repository until disclosure.
- **Ledger and commit text.** Tracker comments, commit subjects and messages
  and tracked status lines give arm totals only ("arm A: N modeled, no
  catch"), never a per-rank or per-crate count, class or property
  identifier, because such counts, differenced against a later record, can
  identify a private candidate (reviews/fx-zu4f.12-r2-opus.md, B1). The
  public record follows the same rule for any rank that holds a private
  candidate. For arm A in this round, the public-signal rule below applies
  in place of arm totals. Reading observations go only to WORK/REQUEST and
  PRIVATE (builder rules).
- **Public signal for arm A (R3; binds this round).** Arm A has two crates,
  and the committed `pool.tsv` names both with their PQ call sites, so an
  arm total, a count of private candidates or one withheld rank would point
  at one of two named functions. The rule reads the same whether or not a
  private candidate exists. The hold starts at go. It ends no earlier than
  when every arm-A pool crate has its final outcome and every arm-A
  candidate is either classed non-dangerous and reviewed, or disclosed
  (Committing, above). Until then, tracker comments, commit subjects and
  messages, tracked status lines and the public record give no arm-A
  outcome, no arm-A total and no count of private candidates, for any rank.
  Every arm-A progress line in that period, interim ones included, is the
  fixed text `arm A: in progress`, used in every case. The record states
  the residual timing leak: the hold ends once arm A's last outcome is in
  and it has no undisclosed candidate, and can last until disclosure (a fix, or 90 days)
  when it has a dangerous one, so the date when arm-A outcomes appear can
  hint that a private candidate existed. The arm-B line (no target) and
  §5's admission observations are not arm-A outcomes and stay unaffected.

This section changes no pool, eligibility, property or caught criterion.

## Safety

- Never contact a live network, mainnet, testnet or any peer: no server,
  relay, key server, RPC endpoint, bootnode or handshake partner, from a
  model, a harness or a test. Everything is local or simulated. The only
  network uses after "go" are read-only GitHub/crates.io GETs for evidence
  and the single recorded `cargo fetch --locked` per reproduction (§7), in a
  fetch stage that compiles and executes no upstream code. Every upstream
  compilation and execution runs only inside a network namespace shown to
  have no route (§7); when that cannot be established, nothing upstream
  runs (fail closed, R1).
- **No live keys.** The only stored key material is published test vectors:
  a crate's own committed KAT files and test vectors, or a standard's
  published vectors that the crate's tests name, each with its source path
  or URL and SHA-256. Keys that a seeded test derives from its case seed are
  throwaway test values: they exist only inside the test process, are never
  written outside TARGET or PRIVATE, never exported, imported or reused, and
  never belong to a real person, service or chain.
- **Nothing real is signed or sent.** Never run code that could sign or
  broadcast anything real: signing runs only on vector or seed-derived test
  keys over test messages, offline. A reproduction that needs a network
  peer, a live service, a daemon, a key store or a real key cannot be run;
  that candidate fails the reproduction condition and is recorded as
  model-only.
- Reproduction runs execute only the added reproduction test, by exact name,
  never a crate's whole test suite, inside the isolated stage (§7).

## Neutral framing

The record states the selection and what the method did and saw. It never
judges a project or its authors, never calls a crate buggy or unsafe from a
scan, a failed port or a model, and gives no discovery rate. A scan class or
a PQ-evidence label is a mechanical label, not a quality claim. "Excluded"
means outside this round's classes. An admission observation is a lexical
count, not a defect.

## 1. Snapshot and search

Snapshot time: **2026-11-19T16:48:17Z**, recorded once. Tools: Python 3.14.10,
git 2.43.0, gh 2.104.0 (authenticated read-only GETs), Linux x86_64.

`search.py search` sends read-only GETs only:

- **GitHub REPOSITORY search** (never commit search), through
  `gh api --include`:
  `search/repositories?q=<query>&sort=stars&order=desc&per_page=100&page=N`.
  Terms, covering every algorithm and protocol the bead lists and their
  common spellings: `ML-KEM`, `MLKEM`, `Kyber`, `ML-DSA`, `MLDSA`,
  `Dilithium`, `SLH-DSA`, `SLHDSA`, `SPHINCS`, `FN-DSA`, `FNDSA`, `Falcon`,
  `HQC`, `McEliece`, `post-quantum`, `postquantum`, `PQC`, `rosenpass`,
  `quantum-resistant`, `quantum-safe` (hyphenated terms quoted). Each term
  runs in two forms, in this order: `<term> language:Rust stars:>=1000
  archived:false fork:false` (GitHub's default fields) and the same with
  `in:readme`. The terms are discovery only: what implements a listed
  algorithm or protocol is decided by §3's PQ gate, so a broad term (such
  as `Falcon` or `Kyber`, which also name other things) widens nothing.
- **crates.io search**, through urllib with a generic User-Agent:
  `https://crates.io/api/v1/crates?q=<term>&per_page=100&page=N` for the same
  terms, paged to each query's `meta.total`. Each crate's `repository`
  field is mapped to a GitHub `owner/repo` (host github.com, first two path
  segments, `.git` stripped); a crate with no repository URL or a non-GitHub
  one cannot meet the GitHub star bar and is recorded with that outcome in
  `crates-io-hits.tsv`. A crate counts by its repository's stars (bead).

Slicing: when a GitHub query's first page reports `total_count` over 1,000,
its `stars:` term is split into two ranges (bisection, recursively) and the
capped page is recorded but not used; otherwise the whole `stars:>=1000`
range is one slice, paged to its total. A GitHub response with
`incomplete_results: true` is retried once after 30 s; a second one blocks
the run. Request starts are at least 2 s apart per host; a 403 or 429 waits
300 s, then 300 s, then 600 s, before the run blocks with evidence in
WORK/BLOCKED. A retried request keeps one manifest row whose `attempt`
field counts its tries; earlier attempts' bodies are overwritten by the
retry. Raw responses stay in TARGET/registration; `queries.jsonl` records
each URL, source, term, query, page, start and end UTC, attempt, HTTP
status, bytes, body SHA-256, `total_count` and item count.
`star-slices.json` records each GitHub slice. `search-hits.tsv` is the union
of distinct repository names with the queries and crates that returned
each.

A first run (16:48:22Z to 16:56:16Z, 87 logged requests plus the failing one) is **superseded**. Its GitHub part hit
the search API's 30-requests-a-minute limit once at 2 s spacing (one 403,
one 300 s back-off, then success), and its crates.io part stopped with
HTTP 400 at `post-quantum` page 11: crates.io refuses relevance-sorted pages
beyond the first 1,000 results. Its outputs were never used; its raw
responses and manifest stay in TARGET/registration-superseded-run1. The fix
changed only mechanics: crates.io terms are paged in alphabetical order
(`sort=alpha`) by following `meta.next_page` until it is empty, and the run
blocks unless the crates paged for a term equal its `meta.total`; GitHub
search requests are spaced 2.5 s (core requests stay at 2 s). Terms, forms
and filters are unchanged.

Search (second run): 40 GitHub requests, 16:56:50Z to 16:58:29Z, all HTTP
200 on the first attempt, no query near the cap (largest total: 11), 33
items; 60 crates.io requests, 16:58:29Z to 17:00:28Z, all HTTP 200 on the
first attempt, 1,752 distinct crates (1,564 with a GitHub repository URL,
111 with none, 77 with a non-GitHub URL). Result: **912** distinct
repository names (23 from GitHub search, 895 from crates.io, 6 from both).

## 2. Repository filter and pinning

`search.py confirm` drops owner-affiliated repositories first, before any
request (owner login `Yazelix` or `luccahuguet`, case-insensitive, reason
`owner-affiliated`; never backfilled), and a name that differs from an
earlier one only in letter case (`duplicate-name-case`, no request). It then
GETs `repos/<full_name>` for each other repository, following redirects,
and drops, with the first reason in this order: not found (404),
owner-affiliated (canonical owner), language not Rust, under 1,000 stars,
archived, fork, not public, duplicate of an earlier canonical id.
`candidates.tsv` has one row per requested name with the canonical name, id,
stars, language, default branch, flags, observation UTC, response URL and
SHA-256, outcome and first drop reason. Stars are those of this GET, used
for all ordering. **Narrowing** (P12 §1's filter, kept): not archived, not a
fork, public.

`search.py pin` makes one clone per kept repository under
TARGET/clones/OWNER__REPO: `git clone --filter=blob:none --no-checkout
--single-branch --branch <default branch>`, then a non-cone sparse checkout of
only `*.rs` and `Cargo.toml` at the default branch, with hooks disabled
(`core.hooksPath=/dev/null`), no submodules and Git LFS smudging skipped.
`clone-refs.tsv` records each clone URL, default branch, HEAD commit (the
pinned commit) and clone interval. Nothing is built or run. The clones are
frozen: no later fetch of new history. Arm B reads commit objects and trees
from them; blobs at pinned or historical commits are fetched lazily by the
blobless clone from the same GitHub repository, the same read-only source,
never a mirror.

The bead's last repository condition, "implements a listed algorithm or
protocol", is decided mechanically at package level by §3's three-step PQ
gate: a repository stays only when at least one of its packages passes all
three steps, else it is dropped with `no-Cargo-package`,
`no-pq-evidence-in-any-package` (no package passes step 1),
`no-pq-implementation-evidence-in-any-package` (some pass step 1, none step
2) or `no-pq-code-evidence-in-any-package` (some pass step 2, none step 3)
in `repositories.tsv`.

Confirmation: 912 requests, 17:00:43Z to 17:31:06Z (no name was
owner-affiliated or a letter-case duplicate, so every name got a GET).
Result: **26** kept; 886 dropped (first reasons: 722 under 1,000 stars, 105
not found, 57 language not Rust, 2 duplicates of an earlier canonical id).
Clones: 2026-11-19T17:31:20Z to 17:32:48Z, **26** pinned. After the PQ gate
(§3, R2): **6** repositories kept; 20 dropped (1 `no-Cargo-package`, 9
`no-pq-evidence-in-any-package`, 7
`no-pq-implementation-evidence-in-any-package`, 3
`no-pq-code-evidence-in-any-package`). The registration's one-step gate
kept 16, R1's two steps 9.

## 3. Crate-level classes (classify.py)

`classify.py` is P12's `scan.py` with one added gate in front; Python
standard-library code run with `python3 -I`. Python keeps it next to P12's
audited scanner, whose lexical helpers and class rules it reuses unchanged;
it is a one-off step-1 tool. `--self-test` (synthetic fixtures only) runs
first. It checks each checkout's HEAD against clone-refs.tsv. Packages,
workspace membership, source scope, dependency resolution, the mechanical
limits and the outputs are P12 §3's, word for word in effect: every
`Cargo.toml` outside `.git/` and `target/`, packages outside `tests/`,
`test/`, `benches/`, `examples/`; Cargo's workspace-membership rule
(`not-a-workspace-member`); library and binary targets only (no target:
`no-production-source`); the non-test superset and the module walk; masked
comments and literals; removed test-only items; `unscannable` on non-UTF-8,
unbalanced delimiters or unresolved `workspace = true` dependencies; normal
and target dependencies only, with renames and `workspace = true` resolved.
No macro expansion, cfg_attr path rewrites, `include!`, feature or platform
selection, or cross-module name resolution; lexical names can be false
positives. No manual override, post-output exclusion or substitution.

**PQ gate (narrowing), three steps, all required (amendments R1, R2).** It
applies the bead's repository condition, "implementing a NIST post-quantum
algorithm or a post-quantum protocol", to each package, before the classes.
It admits packages that implement, or call into a dedicated implementation
of, a listed algorithm (R3); a large repository's unrelated packages stay
out. A package is classified only when it passes step 1 AND
step 2 AND step 3; each step is evaluated only for packages that pass the
one before, so R1 can only remove packages that the registration's gate
admitted, and R2 only packages that R1 admitted. Steps 1 and 2 are
pre-filters; step 3 is the implementation evidence.

*Step 1, discovery token (the registration's gate, unchanged).* One of
these tokens: `mlkem`, `kyber`, `mldsa`, `dilithium`, `slhdsa`, `sphincs`,
`fndsa`, `falcon`, `hqc`, `mceliece`, `postquantum`, `rosenpass` occurs, in
this order of evidence, in (1) its package name, (2) a whitespace-separated
word of its `description` or one of its `keywords`, (3) a normal or target
dependency's package name (or the dependency is `oqs`, `oqs-sys` or
`pqcrypto`), or (4) an identifier of a reached non-test file, comments and
literals masked. Every name, word and identifier is compared after
lowercasing and deleting every non-alphanumeric character (`ML-KEM-768`
gives `mlkem768`, `SPHINCS+` gives `sphincs`). The first evidence is
recorded as `pq_evidence`. A package that fails gets `no-pq-evidence` as its
first reason for both classes. Step 1 is discovery only: on its own it
admits nothing.

*Step 2, implementation evidence (R1).* The package's own reached non-test
code, comments and literals masked and test-only items removed, must show
one of two kinds of evidence, the first found recorded as
`pq_impl_evidence` with file and line (`classify.py`, `pq_impl_evidence`):

- **Strict names** (`PQ_STRICT`). A listed algorithm by its standard name,
  `ML-KEM`, `ML-DSA`, `SLH-DSA`, `FN-DSA`, `SPHINCS` or `McEliece` (with
  `-` or `_` allowed inside, any case); an ambiguous family name only with
  its parameter set: `Kyber512/768/1024`, `Dilithium2/3/5`,
  `Falcon512/1024` (or `Falcon-padded-512/1024`), `HQC128/192/256`. So
  `falcon`, `hqc`, `kyber` and `dilithium` alone never suffice, nor do
  other schemes (`frodo`, `saber`). A strict name counts only where it
  starts a word: at the start, after a non-letter, or at a lowercase-to-
  uppercase camel-case boundary (`X25519MLKEM768`, `HybridMlKem` count;
  `XmlDsa`, `HTMLDSA`, `falcon_h1` do not). Protocol names (`rosenpass`) and
  `postquantum` are discovery tokens only: a protocol implementation passes
  through the algorithms it runs.
- **Operation names** (`op_name`). A `fn` name, split into words at `_`,
  case changes and digits, names an operation when its joined words contain
  `keygen`, `keypair`, `encap` or `decap`, or its words include `sign` or
  `verify`, or include `key`/`keys` together with `gen`/`generate` (so
  `signal`, `assign`, `signature`, `key_size` do not count).
- **Kind 1, `own-op` (the package runs a listed algorithm's operation in its
  own code).** A `fn` definition with an operation name whose context holds a
  strict name: the fn name itself, the header (text from `impl`, `trait` or
  `mod` to its `{`) of any block enclosing it, or a component of the file's
  path inside the package directory. Recorded as
  `own-op:FILE:LINE:CONTEXT:FN`. This covers a crate that implements an
  algorithm and a crate that implements a protocol's step over one (for
  example a hybrid KEM's `encapsulate` on an ML-KEM type).
- **Kind 2, `dep-use` (the package's code uses a dedicated post-quantum
  implementation crate).** A normal or target dependency (renames and
  `workspace = true` resolved) is a dedicated post-quantum implementation
  crate: its package name holds a strict name, or it is one of `oqs`,
  `oqs-sys`, `pqcrypto`, `pqcrypto-kyber`, `pqcrypto-dilithium`,
  `pqcrypto-falcon`, `pqcrypto-hqc`, `pqc_kyber`, `pqc_dilithium`,
  `crystals-dilithium`, `fips203`, `fips204`, `fips205` (`PQ_DEDICATED`,
  fixed now); AND a reached non-test file references it: a path whose first
  segment is the dependency's Rust name (`use`, inline, or `extern crate`;
  never a later segment, so `crate::ml_kem` does not count). Recorded as
  `dep-use:FILE:LINE:PACKAGE`. A general-purpose library that also offers a
  post-quantum algorithm (for example a TLS or general crypto library) is
  not dedicated, and selecting its post-quantum item does not pass.

Kind 1 is checked over all reached files (path order, then position) before
kind 2. A package that passes step 1 and fails step 2 gets
`no-pq-implementation-evidence` as its first reason for both classes. No
discovery token alone (a package name, description, keyword, dependency
entry or identifier) admits a package. Step 2 is lexical and narrows: a
crate whose post-quantum code is reached only through macros, `include!`,
trait methods with other names (for example a key exchange's `start` and
`complete`), or a general-purpose library is excluded, and the record
states this limit; a wrapper whose operation fn delegates to another
library passes at step 2. Step 2 records names only: it is a pre-filter,
and step 3 decides.

*Step 3, code evidence (R2).* A name, a header, a path or an import is not
evidence that a package implements anything. Step 3 admits a package only
when its reached non-test production source (comments and literals masked,
test-only items removed, the package's library and binary targets only, so
no example, bench or test file) holds one of two kinds of code evidence
inside a **function definition with a code body** (`classify.py`
`fn_bodies`, `pq_code_evidence`). A `fn` ending in `;` (a trait or extern
declaration) has no body and never counts; an empty body never counts; an
**accessor** never counts: a body that only returns or borrows one field,
path, constant or literal (optionally with `return`, `&`, `&mut`, a unary
minus, or one `.clone()`, `.as_ref()`, `.as_slice()`, `.as_bytes()`,
`.to_owned()` or `.into()`). Evidence counts only in the innermost function
body that encloses it.

- **Kind 1, `impl` (own implementation).** The innermost code body holds a
  published constant that defines a listed algorithm's arithmetic or
  parameter set, as an integer literal in any base, with `_` separators and
  type suffixes allowed, or as a reference to a `const` or `static` of the
  package whose initializer is exactly such a literal (a name that any
  reached file also binds to another initializer is ignored). The constants
  (`PQ_CONSTANTS`), fixed now:
  - ML-KEM / Kyber: q = 3329 (FIPS 203, §8 "Parameter Sets");
  - ML-DSA / Dilithium: q = 8380417 (FIPS 204, §4 "Parameter Sets");
  - FN-DSA / Falcon: q = 12289 (Falcon specification v1.2, parameter sets;
    FN-DSA keeps this modulus);
  - SLH-DSA / SPHINCS+: the signature lengths in bytes of the six parameter
    families, 7856, 17088, 16224, 35664, 29792, 49856 (FIPS 205, §11
    "Parameter Sets", Table 2). SLH-DSA has no modulus; these sizes follow
    only from its parameter sets;
  - HQC: the code lengths n = 17669, 35851, 57637 (HQC specification,
    parameter-set table for HQC-128/192/256);
  - Classic McEliece: the code lengths n = 3488, 6688, 6960 (Classic
    McEliece cryptosystem specification, selected parameter sets
    mceliece348864, mceliece6688128, mceliece6960119; n = 4608 and 8192 are
    left out as too common in unrelated code).
  The FIPS sections are cited from the published standards; the Falcon, HQC
  and McEliece documents are cited by title and table because this step had
  no network to re-check their section numbers. Recorded as
  `impl:CONSTFILE:LINE:VALUE[:NAME]:fn FN@FNFILE:LINE` (the constant's
  file:line, its value, the constant's name when referenced by name, and the
  enclosing fn's name and file:line).
- **Kind 2, `delegate` (delegation).** The innermost code body holds a
  **call expression** `a::b::op(` (a turbofish allowed) whose first segment
  is the Rust name of a normal or target dependency that is a dedicated
  post-quantum implementation crate (step 2's definition: a strict name in
  its package name, or `PQ_DEDICATED`), or a name that the same file
  imports from such a crate by `use` (a glob import gives no name); or a
  bare call `op(` of such an imported name (never a `fn op(` definition).
  The called last segment must be an operation: `op_name` (key generation,
  keypair, encapsulation, decapsulation, sign, verify) or `generate`. A
  `use` item, a type mention, a method call on a value (`x.encapsulate()`)
  and a declaration are not calls on a crate path. Recorded as
  `delegate:FILE:LINE:PACKAGE:PATH@fn FN`.

All files are searched for kind 1 (path order, then position) before any
for kind 2. A package that passes step 2 and fails step 3 gets
`no-pq-code-evidence` as its first reason for both classes; a repository
whose packages all stop there is dropped with
`no-pq-code-evidence-in-any-package`. Step 3 is the evidence column
`pq_code_evidence` of `classes.tsv` and `pool.tsv`.

**What step 3 cannot see (disclosed, narrowing).** It is lexical. It
misses an implementation whose constants come from an expression (for
example `(1 << 23) - (1 << 13) + 1`), from a macro, `include!`, generated
code, a build script, a feature-gated or platform-gated file outside the
reached module walk, or another crate in the repository; constants held only
in an array or table without a listed value; an implementation of SLH-DSA,
HQC or McEliece written for parameter sets whose listed values never occur
in a function body; delegation through method calls on values, trait
methods (`<T as Kem>::encapsulate`), re-exports from the crate's own
modules, glob imports, or general-purpose libraries; and operation names
other than the listed ones. A listed value can also occur in unrelated code
(for example 12289 in another lattice scheme, or a buffer size equal to a
listed length); such a row is admitted, and the record says the evidence is
lexical. No row is changed by hand, no excluded row is backfilled, and a
pool that shrinks below 10 (or to none) gives arm A all that remains. A
later round that reuses this gate treats parameter-size constants (the
SLH-DSA signature lengths and the HQC and McEliece code lengths, which a
wrapper's buffer check can also hold) as weaker `impl` evidence than
arithmetic constants such as the moduli (R3, N4).

**Narrowing check (R1, R2).** `classify.py --check-narrowing OLD` compares
the regenerated `classes.tsv` with an earlier registration's row by row; R2
runs it against R1's (6ae022a) and the registration's (e1c04a4): the same
package rows; the same stars, pinned commit, binary flag, file counts and
evidence of every gate step the earlier table has, on every row; every row
that passes step 3 passed the earlier table's last step with the same
class, reasons, line and test counts and class evidence; every row that
failed the earlier gate is unchanged; and no row that fails step 3 has a
class. It fails unless all hold.

**Class C (std concurrency)** and **class P (deterministic logic)**: exactly
P12 §3's rules and first-failure orders (C: positive std concurrency use in
a reached file; no `Condvar`, `select!`, `async fn`, `.await`, `Waker`; no
forbidden normal dependency (`tokio`, `futures`, `async-std`, `flume`,
`loom`, any `crossbeam*`, `parking_lot*`, `rayon*`); no `unsafe impl`
Send/Sync. P: no forbidden normal dependency; no `async` token or `.await`;
no `std::net`, `std::process`, `std::fs` or `std::thread` path; no `unsafe`
block or fn; 200 to 5,000 non-test lines over reached files; at least one own
`#[test]`). A package that passes C is class C (`also_p` says whether it
also passes P); else one that passes P is class P; else it is excluded, and
`first_reason` gives both classes' first failures as `C:<reason> |
P:<reason>` (one reason when they are the same).

`classes.tsv` has one row per package, ordered as the pool is, with stars,
pinned commit, crate, crate path, class, both reasons, binary flag,
superset and reached file counts, non-test lines, `#[test]` count, step-1
evidence (`pq_evidence`), step-2 evidence with file and line
(`pq_impl_evidence`), step-3 code evidence with file and line
(`pq_code_evidence`, the committed per-crate evidence table) and the class
evidence. `repositories.tsv` has one row per pinned repository with its
package count, packages passing step 1 (`pq_packages`), step 2
(`pq_impl_packages`) and step 3 (`pq_code_packages`), class C or P
packages, and the outcome.

Scan (R2): 2026-11-19T20:46:40Z to 20:47:47Z, after `--self-test` passed,
with classify.py and this section's rule text frozen at 20:46:36Z (Amendment
R2). Result: 26 repositories; 25 hold packages; **751** package rows plus 1
`no-Cargo-package` placeholder row (752 rows in `classes.tsv`); **38** pass
step 1 (in 16 repositories); **14** pass step 2 (in 9 repositories); **6**
pass step 3 (in 6 repositories): 1 `impl`, 5 `delegate`; **0** class C;
**2** class P; **750** rows excluded. First reasons of the excluded: 700
no-pq-evidence; 24 no-pq-implementation-evidence; 8 no-pq-code-evidence; 7
no-production-source; 4 not-a-workspace-member; 3 forbidden-source |
forbidden-normal-dependency; 2 unscannable; 1 no-std-concurrency-use | std
path; 1 no-Cargo-package (the placeholder). Narrowing checks: both passed,
0 changed fields and 0 widened rows on 752 rows; against R1 (6ae022a) the
last gate step goes 14 -> 6 and class C or P 3 -> 2; against the
registration (e1c04a4) 38 -> 6 and 12 -> 2. R1's scan (20:33:04Z to
20:34:07Z) had 14 packages past its last step, 0 class C and 3 class P.

## 4. Pool

Kept crates (class C or P) are ordered by repository stars descending, then
repository full name (case-insensitive), then crate name, then crate path. At
most 3 per repository, the first 3 in that order. The pool is the first 30,
or all kept crates when fewer than 30 are kept: `pool.tsv` with pool rank,
class, also_p, arm-A flag (ranks 1 to 10), repository, stars, crate, crate
path, pinned commit, non-test lines, step-1, step-2 and step-3 PQ evidence
and class evidence. The pool
is immutable after registration: no substitution when a crate has no useful
history, no portable core, no KAT or no property, and no backfill from
excluded rows.

Result (R2): 2 kept crates in 2 repositories, fewer than 30, so the **pool
is all 2** (the cap of 3 per repository dropped none): **0 class C** and **2
class P**, both admitted by `delegate` evidence. The pool is below 10, so
**arm A takes both** (ranks 1 and 2); the gate is not loosened to refill
it, and no excluded row is backfilled. **Arm B has no target** (no class C
pool crate; §6). Superseded pools: the registration's 12 (3 class C, 9
class P, from 8 repositories), whose other crates fail step 2 or step 3;
R1's 3 (3 class P), whose first crate fails step 3.

## 5. Admission observations (descriptive, never a catch)

`admission-obs.py` is committed now and runs only after "go" (its real run
requires `--after-go`), as in P12. Targets: every pool crate, and every
class-excluded package with a binary target in a pool repository. Source:
classify.py's non-test superset for that package, comments, literals and
test-only items masked. Its docstring states the rules, summarized:

- P12's counts, unchanged: each ambient `std::env`, `std::fs`, `std::net` and
  `std::process` import path and each other path reference resolved through
  same-file aliases, with file:line; spawns of `std::thread::spawn`,
  `tokio::spawn`, `tokio::task::spawn` and `tokio::task::spawn_blocking`
  labelled detached, joined or unresolved.
- **Ambient randomness** (the bead's sources), as imports and references,
  with file:line: `getrandom` (a path whose first segment is `getrandom`),
  `OsRng` (a path with that segment), `thread_rng` (a path with segment
  `thread_rng` or `ThreadRng`, or the path `rand::rng`, rand 0.9's name for
  it), `rand::random` (that path, through aliases), and `/dev/urandom` (that
  text in code with comments masked and string literals kept).
- **Where randomness enters keygen, encapsulation or signing.** Each
  observation carries operation labels: `function_op` for the innermost
  `fn` containing it, and `callee_op` for the callee of the innermost
  unclosed call whose argument list contains it. A name is `keygen` when its
  joined lowercase words contain `keygen` or `keypair` or hold key/keys with
  gen/generate; `encaps` when they contain `encap`; `sign` when its words
  hold sign, signs, signing, signer, signature or signatures (so `signal`
  and `assign` do not count). No call graph: randomness that reaches keygen
  through other functions or stored RNG values is not traced.
- P12's **key flag**: the innermost `fn` also contains an identifier with a
  key-material word (key(s), keypair(s), seed(s), mnemonic(s), secret(s),
  privkey(s), privatekey, secretkey, private key(s)).

All of it is lexical and over-approximates (a local item reusing a source's
name counts; `key` is a common word). Output: `admission-obs.tsv` and a
separate record section with totals by kind, source, operation label and key
flag; never counted as a catch, never described as a defect.

## 6. Arm B (retrospective), class C pool crates

Exactly P12 §6. After "go", for each class C pool crate, from its frozen
clone at the pinned HEAD (no fetch of later history):

```
git -C TARGET/clones/OWNER__REPO log --no-color \
  --format='%H%x1f%P%x1f%cI%x1f%B%x1e' <pinned HEAD>
```

lists every commit reachable from the pinned HEAD (merges included, full
message). Python `re.search(PATTERN, message, re.IGNORECASE)` applies the
template's word-bounded pattern, exactly:

```
\b(race|races|racy|racing|deadlock|deadlocks|deadlocked|dead-lock|hang|hangs|hanging|hung|livelock|lost wakeup|data race|stuck|freeze|freezes|frozen|starvation|starve|starved)\b
```

For each match, the changed paths are
`git diff-tree -r --no-commit-id --name-only <first parent> <sha>` (a root
commit: `git diff-tree -r --root --no-commit-id --name-only <sha>`). A match
is a candidate for a crate when a changed path lies in the crate's own
directory (`crate_path/`, or anywhere for a root package), not inside a nested
package directory of the pinned layout, and with no `tests/`, `test/`,
`benches/`, `examples/` or `target/` component below the crate directory
(P12's narrowing). Renames are not followed (a disclosed limit). Save the
complete export digest and match list per repository; a commit shared by
several pool crates gets a row per crate, linked as a duplicate.

Judging and models are exactly P10's, as P12 adopts them: §3's evidence rule
for **every** row, E1 included: the first-parent pair, the associated PRs,
their complete comments, reviews and review comments, and linked issues are
fetched (read-only GETs at least 2 s apart) and read in full, with no
filtered or truncated viewer; unavailable evidence is unresolved, never a
negative; record URLs, UTC and digests, and list every source read per row.
§4's E1-E5 in order with only the first failure counted and later criteria
marked not assessed; §4's mapping table and restrictions; §5's faithful
model, property-before-run, correction and caught rule (buggy model fails;
lowest failing seed recorded and replayed twice byte-identically through the
driver; fixed model passes all 1,000 seeds `#[sim_test(seeds = 1000)]`).
"The crate" in P10 means the pool crate at its repository's pinned HEAD;
historical source is the matched commit's first parent. Network, subprocess
and external-crate operations on the path fail E4 as in P10. With no class
C pool crate, arm B has no target and the record says so.

## 7. Arm A (forward hunt), pool ranks 1 to 10

Source is the crate at the pinned commit. After "go" only, the builder may
check out the full tree of that commit (all files, not only `*.rs` and
`Cargo.toml`, including test-vector files) in a separate read-only,
hook-free checkout under TARGET to read its docs, README, tests and KATs.
Every choice this section leaves open is recorded in PRIVATE before use.

**Class C ranks:** P12 §7's class C rule exactly (P10 §6: locate the whole
concurrent core, its call/state map and nonblank non-comment line count;
over 400 lines, an external dependency on the core path, or an unsupported
operation makes it unportable; properties frozen and hashed before any
build; 1,000 seeds 0..999 with `#[sim_test(seeds = 1000)]`).

**Class P ranks:** the core is the crate's logic as its frozen properties
exercise it.

1. **Property list, written before any run.** Rows come only from this
   commit's documentation (rustdoc, README in the crate directory), its own
   tests, invariants the code asserts or documents, and **its own KATs**:
   - **KAT rows.** Each known-answer vector set that the crate's own tests
     check at the pinned commit (committed vector files, or vectors embedded
     in its tests) is one row per tested function: "for every vector, the
     function maps the vector's inputs to its expected outputs". The row
     records the vector source path, its SHA-256, the vector count N, the
     test that reads it, and the standard the crate names for it, if any.
     The oracle is the vector file, never a value computed by the model or
     another implementation. **Narrowing:** only the crate's own vectors; no
     outside vectors are added.
   - **Quoted rows.** Each other property is recorded with a short exact
     quote, path and lines at the pinned commit, the property sentence, the
     trigger (public entry points and input domain), the Rust assertion and
     its preconditions. Rules fixed now: a doc quote is a property of the
     item it documents (trait docs bind every hand-written in-crate impl);
     an ambiguous quote asserts only the weaker common claim; the crate's
     own asserted error arms are their own rows; `//` comments with
     checkable claims count; state fields take only values the crate's own
     writers store; **sources that disagree each give their own row** (no
     conflict-based exclusion). Rows that state no checkable property ("no
     property") or whose trigger is outside the crate ("no trigger") are
     counted together as not assertable.
   - The complete list is written in source order (crate docs, then README,
     then source files by path and line, then test files and vector files by
     path and line), hashed and timestamped in PRIVATE before any port,
     build or run. No row is added after a run.
2. For each row, list the non-test functions and state its trigger reaches
   (same crate and same-repository crates; an external crate's code on the
   path makes that row unportable unless it is std/core/alloc), with the
   union's nonblank non-comment line count. An external hash, XOF or RNG
   crate on the path (for example a SHA-3/SHAKE implementation) makes the row
   unportable: the model never substitutes its own version of another
   crate's code. Code an external derive or macro generates on the path also
   makes it unportable; an external type whose code does not execute on the
   path is a recorded gap. The 400-line union is counted mechanically, in
   list order, by a script, before any model is written.
3. The model covers rows in list order while the union of their paths stays
   within 400 lines; a row whose own path exceeds 400 lines, or that would
   push the union past 400, is recorded `over budget` and not ported (P12's
   narrowing, decided mechanically before any run, never by expected
   outcome). No cherry-picked path.
4. **Port faithfully into Tide v0**, recording every gap (integer widths and
   overflow behavior, collections and iteration order, serialization, error
   types, omitted branches). **Randomness as a capability:** every draw the
   ported path makes, from an ambient source (§5's list) or from a
   caller-supplied RNG, becomes a draw from a `Rand` capability passed as a
   parameter, in the same order, width and count; where the crate takes
   caller-supplied seed bytes or a deterministic RNG, the model keeps them
   as values. Each such replacement is a recorded gap. A path that needs an
   operation Tide v0 cannot express faithfully is unportable, with the
   missing operation named.
5. **Tests, registered by kind.** Every test is `#[sim_test(seeds = 1000)]`,
   seeds 0..999, and the record reports each kind apart:
   - **Generated-input tests** (quoted rows whose preconditions admit a
     domain): inputs come from the case seed, through driver-generated
     parameters, a deterministic generator written in the test from the
     seed (frozen with the row, before any run), or draws from the
     simulator's `Rand`. Inputs stay inside the row's quoted preconditions.
   - **KAT checks**: case seed s checks vectors s, s + 1000, s + 2000, ...
     below N, so the 1,000 seeds check every vector once; seeds at or above
     N with N < 1,000 check none, and the record gives N, not 1,000, as the
     number of cases. A KAT check is a finite fixed-domain oracle check, not
     seeded exploration.
   - **Fixed-input rows** (a crate test's own fixed inputs, or a constant):
     one case, repeated under the seed labels, reported as a fixed check,
     not seeded exploration.
   The record gives the count of rows and tests of each kind beside the
   first arm-A result and in the counts table, once the public-signal rule
   (Disclosure, R3) releases arm-A outcomes.
6. **Triage and the caught rule** (P12 §7 step 6, P10 §6): a failing seed;
   the lowest failing seed recorded and replayed twice through the driver
   with byte-identical traces; a control model that removes only the
   suspected fault and passes all 1,000 seeds for every frozen row (KAT
   checks included); AND reproduction on the real crate (below). A KAT
   check that fails in the model while the crate's own KAT test passes on
   the same vector is a port error until shown otherwise; it is corrected
   as a compile-or-port correction (logged, re-hashed before the next run,
   earlier copy kept) and never reported as a catch. A model-only
   candidate is reported as such, never as a new-bug catch.

**Reproduction on the real crate (the only upstream execution).** Only after
the model, replay and control conditions pass. In a separate full worktree
of the pinned commit under TARGET/reproduction/NAME, add one new test file
(`tests/tide_repro.rs` in the crate's package) and change no production
source, dependency or feature. Record the test, features, toolchain,
`Cargo.lock` digest, trigger, assertion and budgets before its first run.
Class C: a real-thread stress test through the crate's public API with a
predeclared worker count, iteration count and per-process timeout, or the
crate's existing loom support only if the pinned source has it (P10 §7).
Class P: the crate's own test harness running the same property on the
same seeds' inputs, with any randomness supplied as the same bytes the
model drew (a seeded RNG or the crate's deterministic entry point), never
ambient OS randomness. Then:

Two stages, kept apart (R1):

1. **Fetch stage (read-only, network allowed, no upstream execution).**
   `cargo fetch --locked` downloads the locked dependencies; it compiles
   nothing and runs no build script, proc macro or test. It is the only
   dependency acquisition (UTC, output, lockfile digest and registry
   checksums recorded). Evidence GETs (§6, Disclosure) are also this
   stage's, never the isolated stage's.
2. **Isolated stage (compile and execute, no network route).** Every
   compilation and execution of upstream code (build scripts, proc macros,
   the crate, the test) runs only inside a fresh network namespace made by
   `unshare -rn`, in the same `sh` that first proves the namespace has no
   route: the only interface in `/proc/net/dev` is `lo`, `/proc/net/route`
   holds no route (its header line only) and `/proc/net/ipv6_route` holds
   no line (tested by reading its content, not its procfs size; R2). Only
   when all three hold does that shell `exec` cargo, offline and
   locked; the probe's output is recorded with the run.

```
bash ~/.cache/eon-agents/eon-overseer/heavy.sh env CARGO_BUILD_JOBS=4 \
  CARGO_TARGET_DIR=/mnt/build/tide-agents/fx-zu4f.14-target \
  TMPDIR=/mnt/build/tide-agents/fx-zu4f.14-target/tmp \
  timeout 1800 cargo fetch --locked --manifest-path <worktree>/Cargo.toml </dev/null
bash ~/.cache/eon-agents/eon-overseer/heavy.sh env CARGO_BUILD_JOBS=4 \
  CARGO_TARGET_DIR=/mnt/build/tide-agents/fx-zu4f.14-target \
  TMPDIR=/mnt/build/tide-agents/fx-zu4f.14-target/tmp \
  timeout <secs> unshare -rn sh -c '
    cat /proc/net/dev /proc/net/route /proc/net/ipv6_route
    test "$(tail -n +3 /proc/net/dev | cut -d: -f1 | tr -d " ")" = lo || exit 97
    test "$(wc -l < /proc/net/route)" -le 1 || exit 97
    ! grep -q . /proc/net/ipv6_route || exit 97
    exec cargo test --offline --locked --manifest-path <worktree>/Cargo.toml \
      -p <crate> --test tide_repro -- --exact <test name> --test-threads=4' </dev/null
```

**Fail closed.** If `unshare -rn` fails (for example unprivileged user
namespaces are unavailable) or the probe exits 97, network isolation is not
established: the reproduction is recorded as **blocked**, the candidate
stays **model-only**, and nothing upstream is compiled or run. No fallback
exists: never `--offline` alone, never proxy variables, never another
mechanism in place of the probed namespace. (`unshare -rn true` succeeded on
this machine at registration; that is not evidence for a later run, which
relies only on its own probe.) A crate without a
`Cargo.lock` at the pinned commit cannot run `--locked`; that reproduction is
recorded as blocked, not unlocked. If the pinned commit's own toolchain file
names a toolchain that is not installed, the run is recorded as blocked, not
switched. Start only with the 1-minute load below 12; coordinate any run
over 15 minutes and booked quiet windows with tide-lead. A timeout alone is
not a reproduced hang. A passing budget means the condition was not met;
budgets are never enlarged after a run. Real and model failures are compared
on the same property, trigger and commit.

## 8. Record

Use the next free `research/NN-` number at writing time. Order: a short
disclosure and privacy notice first; then the out-of-class limits and the
low expected yield; then what the round does not show (supplied properties,
finite seed sampling, KAT checks as finite fixed-domain checks, fixed-input
rows, model gaps including every `Rand` replacement, self-attested
chronology, lexical scan and PQ-gate limits, and the pool's PQ limit (R3):
both pool crates delegate to `libcrux-ml-kem`; by §7 step 2 every row that
reaches a PQ operation is unportable, so no ML-KEM implementation, or any
other listed algorithm's, is tested; arm A reaches only each crate's own
non-PQ code; and the randomness angle is answered only descriptively, by
§5); then caught or not caught per arm. Arm-A outcomes, totals, per-crate
rows and counts enter the record only once the public-signal rule
(Disclosure, R3) releases them; until then the record's only arm-A text is
the fixed line `arm A: in progress`, and the record states the residual
timing leak. List every pool crate (rank, class, outcome, first failure and
reason), negatives first: every crate and candidate without a catch, each
with its first failure, comes before any catch. Next to the pool list, state
the re-registration history: the registration's pool of 12 crates
(`e1c04a4`), R1's 3 (`6ae022a`, after the Codex supervisor pass found that
the PQ gate admitted a package on a lexical token alone), and R2's 2
(`5b9298b`, after the Codex re-check found that names and imports still
passed as evidence), each a mechanical narrowing, all before any run; and
R3, a text amendment after the Opus review of R2. List every candidate
repository and excluded package by its first reason (in the
committed TSVs, cited), every arm-B candidate (commit, first parent, first
failing E criterion, reason, sources read, E2 quote if passed, later
criteria not assessed), and counts with their units kept apart:
repositories, package rows (751 packages and the placeholder kept apart),
step-1, step-2 and step-3 PQ-gated, class C and P, pool; per arm
targeted, screened, unresolved, unportable, modeled, rows and tests by kind
(generated-input, KAT check, fixed-input), caught; for arm B matched
commits, candidate rows per crate, unique commits, duplicates and E1-E5
first failures. Name every deviation from this protocol, its time and
reason. Admission observations are a separate section, with totals by kind,
randomness source, operation label and key flag, never counted as a catch.
A catch becomes a case study under the disclosure rules; no catch means a
documented negative. Nothing is posted upstream.
