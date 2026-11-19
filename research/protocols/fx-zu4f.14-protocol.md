# fx-zu4f.14: post-quantum crypto Rust crates, protocol

Registered on branch `fx-zu4f.14`, based on main
`27881b6a4365fd3c2af81d9716cce01dfc117969`, before any candidate repository's
history, issues or code was examined beyond the mechanical search, pin and
scan below. The frozen criteria in `br --no-db show fx-zu4f.14` own the
contract; this file adds operations and marked **narrowings**. It never
widens the repositories, the classes, the pool or a caught rule. Where this
file and the bead disagree, the bead prevails and the record discloses the
disagreement.

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
  once it is fixed and disclosed.

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
  then limits (§8); tracker comments carry arm totals only (Disclosure).

## 0. Registration boundary

Step 1 ends with one commit holding this file and POOLDIR, byte-identical to
WORK. WORK/PROTOCOL-READY names the commit, every committed file's SHA-256,
the times and the counts. Then the builder stops. Before tide-lead stamps the
files with OpenTimestamps, records their hashes on the bead, upgrades the
proofs and sends "go", nothing else happens: no history listing, issue, PR,
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
  detail. Never say a report was sent because its draft exists.
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
  candidate. Reading observations go only to WORK/REQUEST and PRIVATE
  (builder rules).

This section changes no pool, eligibility, property or caught criterion.

## Safety

- Never contact a live network, mainnet, testnet or any peer: no server,
  relay, key server, RPC endpoint, bootnode or handshake partner, from a
  model, a harness or a test. Everything is local or simulated. The only
  network uses after "go" are read-only GitHub/crates.io GETs for evidence
  and the single recorded `cargo fetch --locked` per reproduction (§7).
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
  never a crate's whole test suite, with networking absent (§7).

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
protocol", is decided mechanically at package level by §3's PQ gate: a
repository stays only when at least one of its packages passes it, else it
is dropped with `no-pq-evidence-in-any-package` (or `no-Cargo-package`) in
`repositories.tsv`.

Confirmation: 912 requests, 17:00:43Z to 17:31:06Z (no name was
owner-affiliated or a letter-case duplicate, so every name got a GET).
Result: **26** kept; 886 dropped (first reasons: 722 under 1,000 stars, 105
not found, 57 language not Rust, 2 duplicates of an earlier canonical id).
Clones: 2026-11-19T17:31:20Z to 17:32:48Z, **26** pinned. After the PQ gate
(§3): **16** repositories kept; 10 dropped (9
`no-pq-evidence-in-any-package`, 1 `no-Cargo-package`).

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

**PQ gate (narrowing).** It applies the bead's repository condition to each
package, before the classes, so that the pool holds post-quantum code and
not a large repository's unrelated packages. A package passes when one of
these tokens: `mlkem`, `kyber`, `mldsa`, `dilithium`, `slhdsa`, `sphincs`,
`fndsa`, `falcon`, `hqc`, `mceliece`, `postquantum`, `rosenpass` occurs, in
this order of evidence, in (1) its package name, (2) a whitespace-separated
word of its `description` or one of its `keywords`, (3) a normal or target
dependency's package name (or the dependency is `oqs`, `oqs-sys` or
`pqcrypto`, post-quantum implementation crates whose names hold no token),
or (4) an identifier of a reached non-test file, comments and literals
masked. Every name, word and identifier is compared after lowercasing and
deleting every non-alphanumeric character (`ML-KEM-768` gives `mlkem768`,
`SPHINCS+` gives `sphincs`). The first evidence is recorded as
`pq_evidence`. A package that fails gets `no-pq-evidence` as its first
reason for both classes. The gate is lexical: it over-approximates
"implements" (a crate that only calls a post-quantum library passes, which
fits the bead's protocols), and a token can name something else; no row is
changed by hand.

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
superset and reached file counts, non-test lines, `#[test]` count, PQ
evidence and the class evidence. `repositories.tsv` has one row per pinned
repository with its package count, PQ-gated packages, class C or P packages,
and the repository filter's outcome.

Scan interval: 2026-11-19T17:32:56Z to 17:35:56Z, after `--self-test`
passed. Result: 26 repositories; 25 hold packages; **752** package rows;
**38** pass the PQ gate (in 16 repositories); **3** class C (none also
passes P); **9** class P; **740** excluded. First reasons of the excluded:
700 no-pq-evidence; 12 forbidden-source | forbidden-normal-dependency; 7
no-production-source; 4 no-std-concurrency-use |
nontest-lines-outside-200-5000; 4 not-a-workspace-member; 3
no-std-concurrency-use | forbidden-normal-dependency; 3
no-std-concurrency-use | std path; 2 no-std-concurrency-use |
unsafe-block-or-fn; 2 unscannable; 1 forbidden-normal-dependency (both); 1
no-std-concurrency-use | no-own-#[test]; 1 no-Cargo-package.

## 4. Pool

Kept crates (class C or P) are ordered by repository stars descending, then
repository full name (case-insensitive), then crate name, then crate path. At
most 3 per repository, the first 3 in that order. The pool is the first 30,
or all kept crates when fewer than 30 are kept: `pool.tsv` with pool rank,
class, also_p, arm-A flag (ranks 1 to 10), repository, stars, crate, crate
path, pinned commit, non-test lines, PQ evidence and class evidence. The pool
is immutable after registration: no substitution when a crate has no useful
history, no portable core, no KAT or no property, and no backfill from
excluded rows.

Result: 12 kept crates in 8 repositories, fewer than 30, so the **pool is all
12** (the cap of 3 per repository dropped none; two repositories hold 3 each):
**3 class C** (ranks 1, 2 and 8) and **9 class P**. Arm A takes ranks 1 to
10: the 3 class C crates and 7 class P. Arm B takes the 3 class C crates.

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
   first arm-A result and in the counts table.
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

```
bash ~/.cache/eon-agents/eon-overseer/heavy.sh env CARGO_BUILD_JOBS=4 \
  CARGO_TARGET_DIR=/mnt/build/tide-agents/fx-zu4f.14-target \
  TMPDIR=/mnt/build/tide-agents/fx-zu4f.14-target/tmp \
  timeout 1800 cargo fetch --locked --manifest-path <worktree>/Cargo.toml </dev/null
bash ~/.cache/eon-agents/eon-overseer/heavy.sh env CARGO_BUILD_JOBS=4 \
  CARGO_TARGET_DIR=/mnt/build/tide-agents/fx-zu4f.14-target \
  TMPDIR=/mnt/build/tide-agents/fx-zu4f.14-target/tmp \
  timeout <secs> unshare -rn cargo test --offline --locked \
  --manifest-path <worktree>/Cargo.toml -p <crate> --test tide_repro \
  -- --exact <test name> --test-threads=4 </dev/null
```

The fetch is the only dependency acquisition (UTC, output, lockfile digest
and registry checksums recorded). Build and run are offline and, through
`unshare -rn`, without a network route (`unshare -rn true` succeeded on this
machine at registration); if unprivileged user namespaces are unavailable
at run time, the record says so and the run uses `--offline` with
`http_proxy`/`https_proxy` set to `http://127.0.0.1:9`. A crate without a
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
chronology, lexical scan and PQ-gate limits); then caught or not caught per
arm. List every pool crate (rank, class, outcome, first failure and reason),
every candidate repository and excluded package by its first reason (in the
committed TSVs, cited), every arm-B candidate (commit, first parent, first
failing E criterion, reason, sources read, E2 quote if passed, later
criteria not assessed), and counts with their units kept apart:
repositories, packages scanned, PQ-gated, class C and P, pool; per arm
targeted, screened, unresolved, unportable, modeled, rows and tests by kind
(generated-input, KAT check, fixed-input), caught; for arm B matched
commits, candidate rows per crate, unique commits, duplicates and E1-E5
first failures. Name every deviation from this protocol, its time and
reason. Admission observations are a separate section, with totals by kind,
randomness source, operation label and key flag, never counted as a catch.
A catch becomes a case study under the disclosure rules; no catch means a
documented negative. Nothing is posted upstream.
