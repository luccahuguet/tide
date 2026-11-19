# fx-zu4f.12: crypto and blockchain Rust crates, protocol

Registered on branch `fx-zu4f.12`, based on main
`a4ab41c975442fe360363677d038aca71d9d4c10`, before any candidate repository's
history, issues or code was examined beyond the mechanical search, pin and
scan below. The frozen criteria in `br --no-db show fx-zu4f.12` own the
contract; this file adds operations and marked **narrowings**. It never
widens the repositories, the classes, the pool or a caught rule. Where this
file and the bead disagree, the bead prevails and the record discloses the
disagreement.

WORK = `/mnt/build/eon-agents/worktrees/tide-fx-zu4f.12/target/work`.
TARGET = `/mnt/build/tide-agents/fx-zu4f.12-target`.
PRIVATE = `~/.cache/eon-agents/tide-lead/fx-zu4f.12-private/`.
POOLDIR = `research/protocols/fx-zu4f.12-pool/`.
The builder is Claude Opus (bead comment, 2026-11-18). Only tide-lead writes
beads, runs governance reviews, stamps registration and lands the branch.
The builder starts no subagents and posts nothing upstream.

Method references, read-only, with SHA-256:
- fx-zu4f.10 protocol, `git show 707ab781af3bfc76b65cbfc35e122c698a416b9c:research/protocols/fx-zu4f.10-protocol.md`,
  `44490b7ddf5b73844d2f7057bd950bfa15fe54321a29fdf5809a8a7ef82b73af`
  (called P10 below; "P10 §n" is its section n);
- fx-zu4f.10 scan.py at that commit, `8f2afde9707f49a9821093680e36cb1fe7827acec13ae40e7d2158256a0a7898`;
- fx-zu4f.11 protocol on main, `40e3e6612a219924c5f3255ddb58d4004b6ddd48de6c92188e2a4f191709753b`;
  its scan.py `3b1d1334b09afbfc250951f156478e9794f330cd869a51e82bb8df87fb6a1bff`
  and admission-obs.py `f36cef2e4572aea0b9a2602407526f02af2b49a5e54d66f655779ff87c38458a`;
- reviews/fx-zu4f.11-r1-codex.md, -r2-opus.md and reviews/fx-zu4f.8-r1-codex.md,
  -r2-opus.md, read for the errors they caught (unregistered restrictions,
  unbounded regexes, a mirror fetch, a miscounted diagnostic, unread
  inventoried sources, missing per-row evidence, units of counts).

## 0. Registration boundary

Step 1 ends with one commit holding this file and POOLDIR, byte-identical to
WORK. WORK/PROTOCOL-READY names the commit, every committed file's SHA-256,
the times and the counts. Then the builder stops. Before tide-lead stamps the
files with OpenTimestamps, records their hashes on the bead, upgrades the
proofs and sends "go", nothing else happens: no history listing, issue, PR,
documentation or source reading beyond scan.py's mechanical pass, no
security-policy reading, no model, no property, no build and no upstream
execution. Git dates and local logs are self-attested; a timestamp shows a
file existed by its attested time, not that no earlier private screening
happened. The record states the first per-crate examination time and the
proof identities, without backdating.

Pre-registration reading was limited to: AGENTS.md, GOVERNANCE.md, the
builder rules and search-round template, the bead and its comment, the
disclosure comments on fx-zu4f.10 and fx-qlek.7, the method references above,
design/reference.md's test-generation section (to name class P's test form),
GitHub repository-search and repository metadata, and scan.py's outputs.
Earlier rounds' records name some known bugs; no pool crate is assessed from
that knowledge. scan.py alone opened the checkouts; the builder read no
per-crate source, documentation, history, issue or PR. The builder looked at
one repository's tracked path list (MinerProxyPro/TCMinerProxy, `git ls-tree`)
to confirm it has no Cargo.toml, and at three manifests' `members` and
`[workspace.dependencies]` lines (unionlabs/union, starcoinorg/starcoin) to
diagnose "unresolved workspace dependency" rows, which led to §3's
workspace-membership rule. No source file was read. Neither repository's
outcome was chosen by hand: the rule is mechanical and applies to every
repository.

## Disclosure and privacy

The disclosure policy recorded on fx-zu4f.10 (2026-11-18 16:52, 16:54 and
16:56 UTC, relayed owner decisions) and fx-qlek.7 applies unchanged, with the
bead's crypto additions:

- **Full public history.** The repository goes public with its full history,
  so nothing undisclosed is ever committed. Every arm-A candidate and every
  arm-B record of a still-unfixed bug is written ONLY to PRIVATE, outside
  every worktree, from its first write: per-crate working notes, properties,
  model sources, traces, reproduction sources and draft texts. Never stage and
  later delete such a file; history keeps it. Build products of private
  models go under TARGET; no provisional copy enters the worktree.
- **Dangerous or not.** A find is dangerous if untrusted input or a remote
  peer can trigger it, or it causes memory unsafety, data corruption or loss,
  or denial of service in a library used by servers. **Crypto addition:** it
  is dangerous by default when funds, consensus or keys could be affected
  (custody, balances, fees, transfers, signatures, key or seed derivation,
  block or state validity, fork choice, finality, peer messages). When in
  doubt, dangerous. The private record states the class with its reasons; the
  reviewers check it on the private copy.
- **Security programs.** For each pool repository, after "go" and before any
  report draft for it, the record lists its security program (Immunefi,
  HackerOne, other bounty platform, SECURITY.md, GitHub private vulnerability
  reporting) with URL and date read, and whether CONTRIBUTING, issue templates
  or a stated policy reject AI-generated reports (URL, date). The program's
  rules bind the draft (scope, channel, embargo, proof-of-concept limits). No
  policy is read in step 1.
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
  disclosed (bead; fx-qlek.7's public-catch decision). It must meet the same
  caught rule. When Lucca sends a private report, tide-lead stamps its
  SHA-256; stamp and hash stay outside the repository until disclosure.

This section changes no pool, eligibility, property or caught criterion.

## Safety

- Never contact a live network, mainnet or testnet: no RPC endpoint, peer,
  faucet, explorer or bootnode, from a model, a harness or a test. Everything
  is local or simulated. The only network uses after "go" are read-only
  GitHub/crates.io GETs for evidence and the single recorded
  `cargo fetch --locked` of §7.
- No key material beyond published test vectors (a crate's own committed test
  vectors, or a standard's published vectors, with source and hash). Never
  generate, import or derive a key for a real chain.
- Never run code that could sign or broadcast a transaction or message to any
  network. A reproduction that needs a validator, node, RPC client, wallet or
  live key cannot be run; that candidate fails the reproduction condition and
  is recorded as model-only.
- Reproduction runs execute only the added reproduction test, by exact name,
  never a crate's whole test suite, with networking absent (§7).

## Neutral framing

The record states the selection and what the method did and saw. It never
judges a project or its authors, never calls a crate buggy or unsafe from a
scan, a failed port or a model, and gives no discovery rate. A scan class is
a mechanical label, not a quality claim. "Excluded" means outside this
round's classes.

## 1. Snapshot and repository search

Snapshot time: **2026-11-19T04:57:46Z**, recorded once. Tools: Python 3.14.10,
git 2.43.0, gh 2.104.0 (authenticated read-only GETs), Linux x86_64.

`search.py search` sends GitHub REPOSITORY search GETs only (never commit
search), through `gh api --include`:
`search/repositories?q=<query>&sort=stars&order=desc&per_page=100&page=N`,
with the bead's query `language:Rust stars:>=1000 archived:false fork:false
topic:<t>` for each of the twelve topics, in the bead's order. Slicing: when a
query's first page reports `total_count` over 1,000, its `stars:` term is
split into two ranges (bisection, recursively) and the capped page is
recorded but not used; otherwise the whole `stars:>=1000` range is one slice
and is paged to its total. No topic reached the cap (largest: blockchain,
33), so each query is a single slice. A response with
`incomplete_results: true` is retried once after 30 s; a second one blocks
the run. Request starts are at least 2 s apart; a 403 or 429 waits 300 s,
then 300 s, then 600 s, before the run blocks with evidence in WORK/BLOCKED.
Raw responses (headers and body) stay in TARGET/registration/search;
`queries.jsonl` records each URL, topic, query, page, start and end UTC, HTTP
status, bytes, body SHA-256, `total_count` and item count. `star-slices.json`
records each slice. `search-hits.tsv` is the union of distinct
`full_name`s with the topics and pages that returned each.

`search.py confirm` drops owner-affiliated repositories first, before any
request (owner login `Yazelix` or `luccahuguet`, case-insensitive, reason
`owner-affiliated`; never backfilled), then GETs `repos/<full_name>` for each
other repository, following redirects, and drops, with the first reason in
this order: owner-affiliated (canonical owner), language not Rust, under 1,000
stars, archived, fork, not public, duplicate of an earlier canonical id.
`candidates.tsv` has one row per distinct repository with the canonical name,
id, stars, language, default branch, flags, observation UTC, response URL and
SHA-256, outcome and drop reason. Stars are those of this GET, used for all
ordering.

Search: 12 requests, 2026-11-19T04:58:31Z to 04:58:55Z. Confirmation: 76
requests, 04:59:03Z to 05:01:33Z. Result: **76** distinct repositories;
**0** dropped; **76** kept.

## 2. Pinning

`search.py pin` makes one clone per kept repository under
TARGET/clones/OWNER__REPO: `git clone --filter=blob:none --no-checkout
--single-branch --branch <default branch>`, then a non-cone sparse checkout of
only `*.rs` and `Cargo.toml` at the default branch, with hooks disabled
(`core.hooksPath=/dev/null`), no submodules and Git LFS smudging skipped.
`clone-refs.tsv` records each clone URL, default branch, HEAD commit (the
pinned commit) and clone interval. Nothing is built or run. Clones:
2026-11-19T05:02:01Z to 05:06:09Z, **76** pinned. The clones are frozen: no
later fetch of new history. Arm B reads commit objects and trees from them;
blobs at pinned or historical commits are fetched lazily by the blobless
clone from the same GitHub repository, which is the same read-only source,
never a mirror.

## 3. Crate-level scan (scan.py)

`scan.py` is Python standard-library code run with `python3 -I`. It was
requested as a one-off step-1 tool; Python keeps it next to fx-zu4f.10 and
fx-zu4f.11's audited scanners, whose lexical helpers it reuses unchanged
except where noted. Run `--self-test` (synthetic fixtures only) first. Scan
interval: 2026-11-19T05:10:45Z to 05:13:59Z. An earlier run (ended 05:09:33Z),
before the workspace-membership rule below, gave the same 30-crate pool; its
outputs were overwritten and are superseded. `unshare -rn true` succeeded on
this machine at registration (used in §7). It checks each checkout's HEAD against clone-refs.tsv first.

**Packages.** Every `Cargo.toml` in the checkout outside `.git/` and `target/`
is parsed with `tomllib`. A package is a manifest with `[package]` (or one
that fails to parse, reported `unscannable-manifest`), outside any `tests/`,
`test/`, `benches/` or `examples/` directory component. The bead scans "every
Cargo package in each repository's workspace"; a repository can hold several
workspaces, so scan.py applies Cargo's membership rule
(**narrowing**): a workspace root's own package, its `members` globs, and path
dependencies (any dependency table, including `workspace = true` entries that
resolve to a path) of members inside the root's directory, transitively, minus
`exclude`. A package with no ancestor `[workspace]`, or one its nearest
ancestor workspace excludes, is its own root. Any other package fails first
with `not-a-workspace-member`. A repository with no package gets one row,
`no-Cargo-package`.

**Source scope.** Targets are the library (`[lib].path`, default
`src/lib.rs`, unless `autolib = false` without `[lib]`) and binaries
(`src/main.rs`, `src/bin/*.rs`, `src/bin/*/main.rs`, `[[bin]]` paths).
Test, bench and example targets and `build.rs` are never sources. No target:
`no-production-source`. The non-test source superset is every `.rs` file
under each target's parent directory that the package owns (no `tests/`,
`test/`, `benches/`, `examples/`, `target/` component, not inside a nested
package), minus external modules declared test-only, exactly as P10 §2 and
fx-zu4f.11. Comments and literals are masked; items under `#[test]`, a
definitely test-only `#[cfg(...)]` (with `test` false and other flags
unknown) or an inner test-only `#![cfg]` are removed. The module walk from
each target (ordinary `mod`, inline modules, `#[path]`) gives the reached
files. Reached files supply positive evidence and line counts; the superset
supplies every ban (conservative). A non-UTF-8 file, an unbalanced delimiter
or an unresolved `workspace = true` dependency makes the package
`unscannable`. The mechanical limits of P10 §2 apply and are disclosed: no
macro expansion, cfg_attr path rewrites, `include!`, feature or platform
selection, or cross-module name resolution; lexical names can be false
positives. No manual override, post-output exclusion or substitution.

**Dependencies.** Normal dependencies are `[dependencies]` and every
`[target.<selector>.dependencies]`, including optional ones, with `package`
renames and `workspace = true` resolved through the package's workspace root
(`package.workspace`, else the nearest ancestor `[workspace]`). Never dev- or
build-dependencies. Forbidden: exactly `tokio`, `futures`, `async-std`,
`flume`, `loom`, and any name starting `crossbeam`, `parking_lot` or `rayon`.

**Class C (std concurrency).** fx-zu4f.10's filter at crate level, in P10's
first-failure order: (1) positive std concurrency use in a reached file
(`std::thread::spawn` or `scope`, `std::sync::mpsc`, `Mutex`, `RwLock`,
`atomic`, through use trees, aliases and std globs; a bare custom `Mutex` or a
core-only atomic is not evidence) else `no-std-concurrency-use`; (2) no
`Condvar`, (3) `select!`, (4) `async fn`, (5) `.await`, (6) `Waker` anywhere in
the superset, else `forbidden-source:<label>:<first file>`; (7) no forbidden
normal dependency, else `forbidden-normal-dependency:<first name>`; (8) no
`unsafe impl ... Send/Sync for` (with std/core marker aliases), else
`forbidden-source:unsafe-impl-Send-or-Sync`.

**Class P (deterministic logic)**, in the bead's order: (1) no forbidden normal
dependency; (2) no `async` keyword anywhere (async fn, block or closure) and no
`.await` (**narrowing**: any `async` token, not only `async fn`), else
`async`; (3) no path under `std::net`, `std::process`, `std::fs` or
`std::thread` (use trees, same-file aliases, qualified paths; `std::thread_local!`
is not a `std::thread` path), else the prefix and file; (4) no `unsafe` block
or fn (`unsafe {`, `unsafe fn` including `const`/`extern "ABI"` forms, and
`unsafe extern {` blocks; `unsafe impl` and `unsafe trait` alone do not
exclude), else `unsafe-block-or-fn`; (5) 200 to 5,000 non-test source lines,
inclusive, counted over the reached files as lines with non-blank text after
removing comments and test-only items (string and character literal lines
count as code), else `nontest-lines-outside-200-5000:<n>`; (6) at least one
`#[test]` attribute in any `.rs` file the package directory owns (src and its
own tests/ alike; nested packages excluded), else `no-own-#[test]`.

A crate that passes C is class C (`also_p` says whether it also passes P);
else one that passes P is class P; else it is excluded, and `first_reason`
gives both classes' first failures as `C:<reason> | P:<reason>` (one reason
when they are the same). `scan-output.tsv` has one row per package, ordered
as the pool is, with stars, pinned commit, crate, crate path, class, both
reasons, binary flag, superset and reached file counts, non-test lines,
`#[test]` count and the positive evidence.

Result: 76 repositories; 75 hold packages; **3,921** packages scanned;
**112** class C (13 of them also pass P); **601** class P; **3,208**
excluded. First reasons of the excluded (C | P): 948 no-std-concurrency-use |
nontest-lines-outside-200-5000; 811 no-std-concurrency-use |
forbidden-normal-dependency; 419 forbidden-source | forbidden-normal-dependency;
350 no-std-concurrency-use | std path; 202 no-std-concurrency-use |
no-own-#[test]; 167 no-std-concurrency-use | unsafe-block-or-fn; 147
no-std-concurrency-use | async; 91 forbidden-normal-dependency (both); 34
not-a-workspace-member; 17 forbidden-source | async; 12 no-production-source;
7 forbidden-source | std path; 3 forbidden-source | unsafe-block-or-fn.

## 4. Pool

Kept crates (class C or P) are ordered by repository stars descending, then
repository full name (case-insensitive), then crate name, then crate path. At
most 3 per repository, the first 3 in that order. The pool is the first 30:
`pool.tsv` with pool rank, class, also_p, arm-A flag (ranks 1 to 10),
repository, stars, crate, crate path and pinned commit. The pool is
immutable after registration: no substitution when a crate has no useful
history, no portable core or no property.

Result: 713 kept crates in 57 repositories; **pool of 30** from 12
repositories: **6 class C** (ranks 6, 14, 23, 24, 25, 27) and **24 class P**.
Arm A takes ranks 1 to 10: 1 class C (rank 6, linera-witty) and 9 class P.
Arm B takes the 6 class C crates.

## 5. Admission observations (descriptive, never a catch)

`admission-obs.py` is committed now and runs only after "go" (its real run
requires `--after-go`). Targets: every pool crate, and every class-excluded
package with a binary target in a pool repository (339 at registration).
Source: scan.py's non-test superset for that package. Its docstring states the
rules, summarized: each ambient `std::env`, `std::fs`, `std::net` and
`std::process` import path and each other path reference resolved through
same-file aliases, with file:line; spawns of `std::thread::spawn`,
`tokio::spawn`, `tokio::task::spawn` and `tokio::task::spawn_blocking`
labelled detached (whole expression statement, `let _ =`, `drop(...)`),
joined (immediate `.join()` or `.await`) or unresolved. **Key flag rule:** an
observation is flagged when the innermost `fn` item containing it also
contains an identifier whose `_`/camelCase words include key, keys, keypair(s),
seed(s), mnemonic(s), secret(s), privkey(s), privatekey or secretkey, or
`private` followed by `key`/`keys`. Comments, literals and test-only items are
masked first; observations outside a fn are not flagged. `key` is a common
word (map keys), so the flag over-approximates key handling; it is reported
as a lexical count, never as a finding. Output: `admission-obs.tsv` and a
separate record section; never counted as a catch.

## 6. Arm B (retrospective), class C pool crates

After "go", for each class C pool crate, from its frozen clone at the pinned
HEAD (no fetch of later history):

```
git -C TARGET/clones/OWNER__REPO log --no-color \
  --format='%H%x1f%P%x1f%cI%x1f%B%x1e' <pinned HEAD>
```

lists every commit reachable from the pinned HEAD (merges included, full
message). Python `re.search(PATTERN, message, re.IGNORECASE)` applies the
bead's word-bounded pattern, exactly:

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
(**narrowing**: the template's "own non-test source"; a test-only change
would fail E1 anyway). Renames are not followed: commits from before the crate
moved to its pinned path are not enumerated (a disclosed limit). Save the
complete export digest and match list per repository; a commit shared by
several pool crates gets a row per crate, linked as a duplicate.

Judging and models are exactly P10's: §3's evidence rule (first-parent pair,
associated PRs, linked issues and complete comments read and inventoried;
unavailable evidence is unresolved, never a negative; record URLs, UTC and
digests, and list every source read per row), §4's E1-E5 in order with only
the first failure counted and later criteria marked not assessed, §4's
mapping table and restrictions, and §5's faithful model, property-before-run,
correction and caught rule (buggy model fails; lowest failing seed recorded
and replayed twice byte-identically through the driver; fixed model passes
all 1,000 seeds `#[sim_test(seeds = 1000)]`). "The crate" in P10 means the pool
crate at its repository's pinned HEAD; historical source is the matched
commit's first parent. Network, subprocess and external-crate operations on
the path fail E4 as in P10.

## 7. Arm A (forward hunt), pool ranks 1 to 10

Source is the crate at the pinned commit. After "go" only, the builder may
check out the full tree of that commit (all files, not only `*.rs` and
`Cargo.toml`) in a separate read-only worktree under TARGET to read its docs,
README and tests.

**Class C (rank 6):** P10 §6 exactly: locate the whole concurrent core (all
roots from every non-test thread, channel, lock or atomic use; producers,
consumers, initialization, shutdown, error paths and required same-repository
helpers), record the call/state map and the union's nonblank non-comment line
count; over 400 lines, an external dependency on the core path, or an
unsupported operation makes it unportable. Properties as P10 §6, frozen and
hashed before any build. 1,000 seeds 0..999 with `#[sim_test(seeds = 1000)]`.

**Class P (ranks 1-5, 7-10):** the core is the crate's state machine or logic
as its frozen properties exercise it:
1. Properties come only from this commit's documentation (rustdoc, README in
   the crate directory), its own tests, and invariants the code asserts or
   documents. Each is recorded with a short exact quote, path and lines at the
   pinned commit, the property sentence, the trigger (public entry points and
   input domain), the Rust assertion and its preconditions. The complete list
   of quotable properties is written in source order (crate docs, then README,
   then source files by path and line, then test files by path and line),
   hashed and timestamped in PRIVATE before any port, build or run.
2. For each property, list the non-test functions and state its trigger
   reaches (same crate and same-repository crates; an external crate's code on
   the path makes that property unportable unless it is std/core/alloc), with
   the union's nonblank non-comment line count.
3. The model covers properties in list order while the union of their paths
   stays within 400 lines; a property whose own path exceeds 400 lines, or
   that would push the union past 400, is recorded `over budget` and not
   ported (**narrowing**, decided mechanically before any run, never by
   expected outcome). No cherry-picked path; no property added after a run.
4. Port faithfully into Tide v0, recording every gap (integer widths and
   overflow behavior, collections and iteration order, serialization, error
   types, omitted branches). A path that needs an operation Tide v0 cannot
   express faithfully is unportable, with the missing operation named.
5. **Seeded property test:** each property is a Tide test that derives its
   inputs from the case seed, `#[sim_test(seeds = 1000)]`, seeds 0..999,
   with generated parameters where the driver supports them, else a
   deterministic generator written in the test from the seed (frozen with
   the property, before any run). Inputs stay inside the property's quoted
   preconditions.
6. Triage, correction, the fault-removal control and the caught rule are
   P10 §6's: a failing seed; the lowest failing seed recorded and replayed
   twice through the driver with byte-identical traces; a control model that
   removes only the suspected fault and passes all 1,000 seeds for every
   frozen property; AND reproduction on the real crate (below). A model-only
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
same seeds' inputs. Then:

```
bash ~/.cache/eon-agents/eon-overseer/heavy.sh env CARGO_BUILD_JOBS=4 \
  CARGO_TARGET_DIR=/mnt/build/tide-agents/fx-zu4f.12-target \
  TMPDIR=/mnt/build/tide-agents/fx-zu4f.12-target/tmp \
  timeout 1800 cargo fetch --locked --manifest-path <worktree>/Cargo.toml </dev/null
bash ~/.cache/eon-agents/eon-overseer/heavy.sh env CARGO_BUILD_JOBS=4 \
  CARGO_TARGET_DIR=/mnt/build/tide-agents/fx-zu4f.12-target \
  TMPDIR=/mnt/build/tide-agents/fx-zu4f.12-target/tmp \
  timeout <secs> unshare -rn cargo test --offline --locked \
  --manifest-path <worktree>/Cargo.toml -p <crate> --test tide_repro \
  -- --exact <test name> --test-threads=4 </dev/null
```

The fetch is the only dependency acquisition (UTC, output, lockfile digest
and registry checksums recorded). Build and run are offline and, through
`unshare -rn`, without a network namespace route; if unprivileged user
namespaces are unavailable, the record says so and the run uses
`--offline` with `http_proxy`/`https_proxy` set to `http://127.0.0.1:9`.
If the pinned commit's own toolchain file names a toolchain that is not
installed, the run is recorded as blocked, not switched. Start only with the
1-minute load below 12; coordinate any run over 15 minutes and booked quiet
windows with tide-lead. A timeout alone is not a reproduced hang. A passing
budget means the condition was not met; budgets are never enlarged after a
run. Real and model failures are compared on the same property, trigger and
commit.

## 8. Record

Use the next free `research/NN-` number at writing time. Negatives first:
what the round does not show (supplied properties, finite seed sampling,
model gaps, self-attested chronology, lexical scan limits), then caught or
not caught per arm. List every pool crate (rank, class, outcome, first
failure and reason), every arm-B candidate (commit, first parent, first
failing E criterion, reason, sources read, E2 quote if passed, later criteria
not assessed), and counts with their units kept apart: repositories, packages
scanned, class C and P, pool; per arm targeted, screened, unresolved,
unportable, modeled, properties, caught; for arm B matched commits, candidate
rows per crate, unique commits, duplicates and E1-E5 first failures. Name
every deviation from this protocol, its time and reason. Admission
observations are a separate section, with totals by kind and key flag, never
counted as a catch. A catch becomes a case study under the disclosure rules;
no catch means a documented negative. Nothing is posted upstream.
