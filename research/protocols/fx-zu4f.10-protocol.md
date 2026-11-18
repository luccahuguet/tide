# fx-zu4f.10: expressibility-first protocol

Registered on branch `fx-zu4f.10`, based on
`a28b4df0b49731a20e9f181ab788fc5ce498a53a`, before examining any candidate
crate's history, issues, or source beyond the mechanical archive scan.
The frozen criteria in `br --no-db show fx-zu4f.10` and its comments own the
contract. This protocol supplies operations and marked narrowings; it never
widens the pool, eligibility, faithful-model requirements, or caught rules.
The bead prevails over an inconsistency, which must be disclosed.

WORK = `/mnt/build/eon-agents/worktrees/tide-fx-zu4f.10/target/work`.
TARGET = `/mnt/build/tide-agents/fx-zu4f.10-target`.
Only the coordinator writes beads, conducts the governance reviews, stamps
registration, and lands the branch. The builder starts no subagents and
posts nothing upstream.

## 0. Registration boundary and prior knowledge

Step 1 ends with a commit containing this file and
`research/protocols/fx-zu4f.10-pool/`. Copies are byte-identical to WORK.
WORK/PROTOCOL-READY names the commit, every committed file's SHA-256, the
collection times, and the four selection counts. Then the builder stops.
No clone, history search, issue, PR, source reading beyond the mechanical
scan, model, property assessment, or upstream execution occurs before
tide-lead stamps the files with OpenTimestamps, records the hashes and
proofs on the bead, and sends `go`. The stamp and its upgrade belong to
tide-lead, not this step. Git dates and local collection logs are
self-attested; a timestamp proves existence by its attested time, not that
no earlier private screening happened. Publication alone cannot prove the
screening order. The later record discloses the first per-crate examination
time and the actual timestamp/proof identities, without backdating claims.

Pre-registration reading is limited to instructions, the bead and comments,
the fx-zu4f.9 protocol, research/41, its r1 Codex and r2 Opus reviews,
research/39's method/limitations map, crates.io policy pages, registry
metadata, GitHub repository metadata, and the mechanical scan's outputs.
The earlier records contain already-known bug examples; no new pool crate
is manually assessed from that knowledge. Raw archives are opened only by
scan.py; the builder reads no per-crate source, documentation, history,
issue or PR in step 1. Synthetic scanner fixtures are not upstream code.

## Disclosure and privacy

Owner decision relayed by Lucca via tide-desk and recorded by tide-lead on
the bead at 2026-11-18 16:52 UTC, with the operational addition sent to
this builder: the repository will be public with its FULL history. Every
arm-A candidate and every arm-B finding about a bug still unfixed upstream
is written ONLY to
`~/.cache/eon-agents/tide-lead/fx-zu4f.10-private/`, outside every worktree,
never to WORK, a tracked/untracked worktree file, or a commit. Do not stage
and later delete an undisclosed find: history preserves it. Keep all A
per-crate working records, properties, model sources, traces, reproduction
sources and draft texts private from their first write, so a later failing
run cannot retroactively make a worktree copy private. Use the private
record and property log instead of WORK's logs for these cases. Public
archive inputs and ephemeral compiler products still use TARGET; candidate
records and source artifacts are not copied there or into the worktree.
Private models use the driver's existing compilation/test/replay entrypoints
at private source paths, with build products under TARGET; no provisional
case-study copy is placed in the worktree. These privacy rules override
the generic record and case-study locations below wherever needed.

Classify each find dangerous or not, with reasons in its private record.
It is dangerous if untrusted input or a remote peer can trigger it, or it
causes memory unsafety, data corruption/loss, or denial of service in a
library used by servers. Otherwise it can be non-dangerous (for example,
a CLI hang under a local race). When in doubt, classify dangerous. The
reviewers check the classification on the private copy. Draft the report
and upstream issue texts only in that directory, and NEVER send or post
them. Lucca alone files/sends; tide-desk hands him the drafts. A dangerous
draft uses SECURITY.md's channel, else GitHub private vulnerability
reporting, else a maintainer email, without contacting any of them.

The committed/public record names only fixed or already-public bugs and
gives only a count of private candidates, no crate identity or revealing
detail. Non-dangerous findings become commit-eligible only AFTER Lucca
has filed the public issue; dangerous findings only AFTER disclosure
(the policy allows disclosure after a fix or 90 days, through Lucca).
Do not claim a report was sent merely because its draft exists. Launch
texts can say N bugs were found and reported privately only after that
report actually happened. A privately reviewed finding counts as an
external catch only when it meets the SAME caught rule; its record stays
outside the repository until disclosure. This section changes no pool,
eligibility, property or caught criterion.

At classification time, for EACH pool crate, record whether CONTRIBUTING,
issue templates or its stated policy reject AI-generated reports, with the
URL and date read. Do not inspect those policies during step 1. A
non-dangerous find in such a project gets NO upstream issue draft: it is
listed publicly with a cited policy note instead, through the coordinator's
disclosure workflow. A dangerous find still gets a private security-channel
draft. The cited-note route is the exception to the public-issue prerequisite
above; agents still send nothing. This addition is the bead's second
disclosure comment, relayed by tide-lead to this builder on 2026-11-18.

## 1. Snapshot and popularity

The snapshot is ten sequential read-only GETs of
`https://crates.io/api/v1/crates?page=N&per_page=100&sort=downloads`, N=1..10,
in page and response order. Each page contributes 100 entries. The raw
bodies are `snapshot/page-NN.json`; `snapshot-manifest.json` records each
URL, start/end UTC, HTTP status, bytes, and SHA-256. Request starts are at
least 1.1 seconds apart, using the generic, non-personal User-Agent
`tide-research-snapshot/1.0 (public registry metadata research)`. It follows
the crates.io crawler restriction of at most one request per second. The
snapshot is one pagination pass, not an atomic server transaction. Verify
1,000 distinct crate IDs and nonincreasing all-time `downloads`; preserve
the API's order for ties. If these checks fail, stop rather than splice a
replacement snapshot into this one.

`newest_version` in that snapshot pins the newest published archive; do not
substitute `max_version`, a stable-only release, HEAD, or a later release.
Repository URLs are interpreted as GitHub only for http(s) github.com
(including www), git://github.com, or git@github.com forms with an owner and
repository. Strip `.git` and ignore a trailing URL path. Missing or other
hosts fail popularity. For each distinct GitHub repository URL, use only
`gh api repos/OWNER/REPO`; follow GitHub's repository redirects and record
its canonical `full_name`, `stargazers_count`, request-start UTC in
`observed_utc`, and response
SHA-256. Reuse a response for crates with the same repository URL: stars
are a repository measurement. `stars.tsv` has one row per snapshot crate,
in original order, including non-GitHub and below-threshold exclusions.
Keep only counts >=1,000. These observations occur over the recorded
collection interval, not at one fictional instant. Raw GitHub responses
remain in TARGET/registration/github for auditing; their digests and the
measurements are committed in stars.tsv. No history or issue endpoints are
queried during this step.

For a refusal, rate limit or transient failure, retain the URL, UTC, exit
or HTTP error and retry politely (up to three attempts, waiting at least
30 then 60 seconds; honor a longer Retry-After or rate-reset delay). If it
still fails, write WORK/BLOCKED with the evidence and stop. An unavailable
GitHub metadata response is not silently treated as a low star count.

## 2. Mechanical expressibility scan and the fixed pool

`scan.py` is Python standard-library code, as explicitly requested for this
step; it adds no dependency and compiles nothing. Run its synthetic
`--self-test` first. With `--fetch`, it downloads every popularity survivor's
pinned `.crate` from
`https://static.crates.io/crates/NAME/NAME-VERSION.crate`, using the same UA
and >=1.1-second request-start spacing. `archive-hashes.tsv` records crate,
version, URL, start/end UTC, byte count, filename and SHA-256. Archive bytes
stay under TARGET/archives. Extraction is under TARGET/unpacked, accepting
only regular files and directories through Python tarfile's `data` safety
filter. Never build, run, source or install an archive member. Default
invocation is offline: verify every archive digest before extracting and
scanning again. A parse, archive or invariant error fails the run visibly;
it is never converted into a survivor or silently omitted row.

### Source scope and lexical method

The library entry is published Cargo.toml's `[lib].path`, defaulting to
`src/lib.rs`. A missing library entry fails `no-library-source`. Scan sorted
`.rs` files beneath the entry's parent directory; omit path components
`tests`, `test`, `benches`, `examples`, `bin`, `.git`, `target`, and files
named `build.rs`. This is a static superset of ordinary library modules,
not compiler reachability: unused source and alternative platform/feature
branches can conservatively exclude a crate. Binary source outside those
conventional directories can remain in the superset. No platform or
feature branch is selected by building it.

Mask line comments, nested block comments, ordinary/raw/byte/C strings and
character literals, retaining offsets and newlines. Lifetimes remain.
An inner `#![cfg(...)]` that is definitely test-only blanks the whole file.
Remove an annotated item through its balanced body or semicolon for
`#[test]` or a `#[cfg(...)]` expression that is definitely false with
`test=false` (all other flags unknown). This includes `#[cfg(test)]` inline
modules and items and `all(test, ...)`, but retains `not(test)` and mixed
`any(test, feature=...)` source. Comma-delimited test-only fields/variants
are also removed, preserving the containing item's closing delimiter.
For removed external `mod NAME;`, omit its
conventional NAME.rs or NAME/ subtree relative to the declaring source
file's module directory. Test helpers beneath nested inline modules or
custom test paths can remain in the conservative forbidden-use superset.
This also removes test-only helper modules not named tests. For positive
std-use evidence, follow ordinary `mod NAME;` and inline `mod NAME { ... }`
declarations from the library entry, skipping other item/macro bodies in
this module walk. Resolve conventional NAME.rs/NAME/mod.rs and literal
`#[path="..."]` attributes; include such declared files outside the entry's
parent if still inside the archive. Reachable production files are not
discarded merely because a test declaration elsewhere shares their name.
Only these reached files may supply positive std evidence (**narrowing**);
the larger source superset still supplies forbidden-use evidence.
Macro-generated modules, cfg_attr path rewrites, include-generated code,
unusual test framework attributes and cross-module name resolution are
not interpreted. Other files outside the entry's parent are not followed. These are
mechanical limits, disclosed in the record; the scan does not prove an
entire Rust program free of a forbidden operation. No manual overrides,
post-output exclusions, or pool substitutions are permitted. A later
unportable crate remains in the pool with its recorded failure.

Concurrency detection recognizes explicit `std::thread::spawn` or `scope`,
`std::sync::mpsc`, `Mutex`, `RwLock` and `atomic` paths, including ordinary
nested `use` trees, `as` aliases and simple std glob imports. Imports count
as source uses. A bare custom `Mutex` is not assumed to be std; a core-only
atomic import is not used as positive std evidence (**narrowing**).
The script records the matching file and canonical std path for survivors.
It is lexical, not type resolution or macro expansion.

On the remaining non-test text, reject the identifier `Condvar`, `select`
followed by `!`, `async fn` (including `async unsafe fn`), `.await`, `Waker`,
and an `unsafe impl ... Send for` or `Sync for`, with qualified trait paths,
generic impl parameters and direct std/core marker-trait aliases recognized.
These lexical names can be
conservative false positives for unrelated local names. Qualified macro
calls are covered; macros that synthesize a forbidden spelling and indirect
re-exports are not expanded. This limitation never justifies manual pool
editing.

Parse published Cargo.toml with tomllib: all `[dependencies]` and every
`[target.<selector>.dependencies]`, including optional and inactive-target
normal dependencies, using `package` instead of the alias where supplied.
Never include dev-dependencies or build-dependencies. Forbidden package
names are exactly `tokio`, `futures`, `async-std`, `flume`, `loom`, and the
prefixes `crossbeam`, `parking_lot`, `rayon` followed by anything. Do not
extend exact `futures` or `tokio` to wildcard families.

One scan-output.tsv row is emitted for every snapshot crate, with its
pinned version, rank, repository, downloads, stars, outcome, first reason,
source-file count and positive std evidence. First failure order is:
popularity; library entry; positive std use; Condvar; select!; async fn;
.await; Waker; normal dependencies (package-name sort); unsafe Send/Sync
impl. Within a source reason, take the first sorted file. Later reasons do
not replace the first. The first 30 `kept` rows in original download order
form pool.tsv, adding a pool rank; if fewer survive, use all of them and say
so. Keep scanning every popularity survivor even after obtaining 30.
The pool is immutable after this registration commit.

Registered selection outcome: **1,000** snapshot crates; **369** after the
GitHub star threshold; **24** after the mechanical scan; **24** in the
pool. Fewer than 30 survive, so ALL 24 are retained, without substitutions.
Arm B therefore targets 24 crates and arm A the first ten. Full measurements
and first-failure rows are in the committed pool directory. A dash in
std_evidence means no positive evidence is reported for an excluded row.

## 3. Work after go: order and arm B candidate enumeration

Arm B runs first on every pool crate, then arm A on the first ten pool
crates in download order (all if fewer than ten). No replacing a crate that
has no useful history or no portable core. Shared repositories are cloned
and enumerated once but every pool crate gets its own coverage row.

After go, make one read-only, non-shallow clone per distinct repository
under TARGET/upstream/OWNER__REPO, with no checkout, hooks or submodules
executed. Record the clone URL, UTC, HEAD commit, fetched refs and object
IDs. Freeze them and do not fetch later history. Enumerate every commit
reachable from all those refs, not only the newest release or main branch;
record the limits (unreachable/deleted history is not claimed). Repositories
that cannot be fully cloned are unresolved and the arm is incomplete.

Use git log on that frozen clone to export commit SHA, all parents, UTC
commit time and the full message (`%B`, subject AND body, NUL-delimited).
Apply Python `re.search` with `re.IGNORECASE` to the full message, exactly:

```
race|deadlock|dead-lock|hang|hung|livelock|lost wakeup|data race|stuck|freeze|starv
```

No word boundaries, path filter, date bound, keyword additions, top-N or
match-count cutoff. Save the complete match list and the export digest.
All matches get a record row, even unrelated workspace commits and textual
false matches such as `change`; those fail E1 for this crate if they fix no
bug in its non-test code. Assess pool order, then matched SHA in lexical
order for reproducibility. A shared fix is assessed once per distinct bug
site, with duplicate rows linked and counted separately from unique fixes.

The fix is the matched commit's diff against its first parent; that parent
is the buggy revision. A merge uses its first parent, not a chosen branch
intermediate. A root commit with no buggy parent cannot supply this pair
(**narrowing**). Never substitute a different fix or buggy revision from a
linked PR. Read the matched commit, its associated PR(s), the complete
comments/reviews and linked issue(s) that report or explain this bug, with
all pagination. Follow explicit source links needed to establish E1/E2;
record URLs, UTC, responses and digests. Resolve issue references in their
own repository, not one inferred from a coincidentally equal number.
Save a per-candidate evidence inventory before any failure at E2 or later;
read the inventoried sources in full, so a failure is never explained by a
skipped thread. Unavailable required evidence stays unresolved, with the
failed attempts; it does not become a completed negative.

## 4. Arm B judging, in order E1-E5

Record only the first failure as the counted criterion; later concerns may
be labelled additional, and later unexamined criteria say not assessed.
Eligibility requires all five, before any model is attempted.

**E1: a non-test bug fix.** The matched commit fixes a bug in the pool
crate's non-test code by its own message or linked PR/issue. A test-only,
doc-only, added-test-only or skip/delete/retry-test-only change fails.
A workspace-neighbor-only fix is not a fix in this crate. A publicly
resolvable source/first-parent pair is required, not an inferred repair.

**E2: this bug's stated symptom.** Quote an observable wrong behavior in the
commit, PR or linked issue with URL and revision/comment identity: deadlock,
hang, freeze, blocked/stalled named operation, error, panic/assertion,
wrong value/output, lost/duplicated data, crash, failing test or sanitizer
report. A bare race/cause label or a description of the code change is not
a symptom. Apply fx-zu4f.9 r2 exactly: a stated failure the evidence ties to
THIS row's bug, even tentatively, passes; uncertainty about the diagnosis
does not erase an observed failure. A failure attributed to another defect,
a PR-only revision or new code, or nothing identified as this bug, does
not pass. Retain such quotes as context instead of relabelling them as
hypothetical or using them as this row's symptom. A merely possible failure
with no stated occurrence/reproduction anywhere in the allowed evidence
fails the inherited observation narrowing. A stated deadlock itself is an
observable symptom; an unqualified word `race` alone is not. Memory
unsafety is not an E2 exclusion; the model restriction is E4.

**E3: concurrency-shaped.** The evidence/fix identifies a race, lost update,
lost wakeup, ordering fault, deadlock or hang, or timing-dependent flake in
shared state/blocking between concurrent activities. A self-relocking
thread's blocking cycle qualifies. Single-thread logic faults and
performance/contention with neither wrong behavior nor hang fail.

**E4: small and faithfully expressible.** Identify functions changed by the
fix, the operations they race or wait against, and the evidence's trigger.
Read source at the fixed/first-parent identities from the frozen clone or
read-only GET; never run it. List exact files, functions and nonblank,
non-comment source-line counts (count the union once, not a handpicked
slice). The affected non-test path, including same-repository helpers it
requires, must fit 400 lines. No network, subprocess or external crate
operation anywhere on it. std/core/alloc are not external, and crates of
the same repository are not external, but their behavior still needs the
registered mapping below. Test setup outside the path may be replaced only
by an explicit starting-state construction.

Adopt fx-zu4f.9 protocol §§4-5 and 7's faithful mappings/narrowings, including:

| Source operation | Allowed mapping and mandatory gap |
| --- | --- |
| std spawn/scope/join, same-repository task helper | scoped tasks/Scope::spawn/Task::join; formerly detached thread becomes a child of the trigger scope |
| Mutex/RwLock guard covering pure computation only | one Shared::with per guard; FIFO, serialized readers; no concurrent-reader-dependent bug |
| each atomic access/RMW/CAS | one Shared::with per source atomic operation; SC, no weak-CAS spurious failure; ordering arguments ignored, never fuse distinct source operations |
| std mpsc | corresponding Tide mpsc operation; a zero-capacity channel is portable only if its rendezvous semantics agree |
| park/unpark/park_timeout | per-task capacity-one unit-token channel, try_send ignoring Full, recv/recv_deadline; no spurious return |
| sleep/Instant::now | virtual clock sleep/now |
| yield/spin hint, same-repository Backoff | tasks.yield_now; no real spin timing |
| synchronized UnsafeCell/MaybeUninit slot | payload field, one with per source access; no uninitialized memory |
| pointer to own linked node | Vec index, never free nodes; no reclamation |
| Arc for sharing | scoped borrow or Tide Arc; no reference-count observations |
| anything else | unportable, with the missing mapping named |

No external operation may be replaced with a similar Tide primitive. No
async/Future/Waker/Context/poll-notify path. No guard across another
acquisition (same lock included), blocking, signaling or effectful action,
spawn/join, channel, sleep or I/O; no reentrancy. No Condvar or selection
reconstruction. No Once/call_once/lazy initialization whose ordering or
blocking matters. No futex/OS wait queues/kernel mechanisms, hardware,
interrupts, FFI, thread-local destructors, thread/process exit ordering,
stack-size/CPU-count/affinity dependence. No unsafe beyond the two slot/node
mappings above; no manual allocation, freeing/reclamation/drop_in_place.
No path whose stated symptom is memory unsafety (UAF, double free, segfault,
uninitialized read, sanitizer data race or soundness advisory), rather than
a source panic, wrong/lost/duplicated value or hang. The models take no
unsafe grant. This preserves the earlier protocol rather than converting a
memory bug into a different, invented value race.

**E5: explored by the simulator.** For each E4 pass, explicitly answer:
(a) could the bad schedule occur under SC? (b) does it require a switch
inside an indivisible mapped step? (c) does it require hardware, real OS
preemption/time/fairness/starvation, spurious wakeups or weak CAS failure?
(d) can the simulator report the hang? Fail for weaker-memory-only behavior,
a required intra-step switch, a platform dependency or an unreportable spin.
An ordering-only/fence-only fix fails (a); a logic-plus-ordering fix passes
only with a documented SC failing schedule before any run. The simulator
samples task interleavings at capability operations, spawns/joins, channel
entry, Shared entry/release and explicit yield/checkpoint points. It does
not explore weak memory or arbitrary instruction preemption. Deadlock is no
runnable task with no timer; a timer-related stall is reported after its
point budget. A timer-free spin merely ended by host timeout is not a catch.

### How failures are described

Keep the design/planned/budget map from research/39 and corrected research/41:
Shared's Pure, non-nesting/non-suspending closure is a safe-design limit.
Async/Waker are rejected by design. Condvar, select, sync once/lazy cells,
and Arc count/get_mut/try_unwrap operations are deliberate omissions with
named replacements (channels, one enum channel, initialization in main or
const/static, and scoped sharing); they are not promised missing features.
Raw Mutex/RwLock and SC atomic APIs are planned/unshipped at this base;
raw weaker orderings are behind the planned unsafe grant, and their future
existence promises no weak-memory simulation. The 400-line cap, unsafe and
symptom narrowings, reconstruction ban and historical-fidelity restrictions
are protocol budgets/restrictions. Network, subprocess and external-crate
bans are frozen experiment criteria, not claims that Tide has no Net or
Cargo dependencies. Attribute a first failure to its actual restriction;
report which constraints decided no rows, instead of calling all failures
missing v0 primitives. Admission rejection is never counted as a seed catch.

## 5. Version gaps and faithful arm B models

Historical source is modeled at the fixed commit's first parent, never at
current HEAD. List every old std/crate API and exact source-to-model map;
syntax/edition differences may preserve computation, listed explicitly.
A bug depending on historical std/crate implementation internals rather
than documented behavior at that version is unportable at E4: do not
rebuild it or silently replace it with current behavior. Same-repository
helpers may be inlined only with source mapping and within the cap. All
model gaps (including omitted branches, ownership/drop/fairness behavior
and each atomic point under SC) are explicit. An unsafe mapping must not
add or hide the source bug. A modern passing version never substitutes for
the historical fixed model, and a historical B catch is not a new bug at
the published A version.

For each eligible B candidate, BEFORE any build or run, write the expected
behavior from its own commit/PR/issue, with a short exact quote, URL,
revision/comment identity and the property sentence, trigger and Rust
assertion. Hash the property-bearing record, timestamp it in
WORK/property-log.txt, and preserve the pre-run copy. The property and
trigger never change after a run. For an observed hang/deadlock the property
is `the trigger completes`. No property is reverse-engineered from a run
or taken from the patch author's reproduction alone.

Map every modeled function, shared state and scheduling point to upstream
file/lines at the pinned SHA. Keep the affected path's structure and shared
state; adapt only syntax/std as mapped. Add no scheduling point other than
one authorized by the mapping and remove no source synchronization. The
fixed model is that exact buggy model plus the matched fix's change, no
other repair. Show the buggy/fixed diff and map every changed model line to
the fix; disclose fix hunks outside the path. Use byte-identical test code
and `#[sim_test(seeds = 1000)]` in both.

Public fixed-bug models go through the existing examples-runner convention with declared
expected outcomes, not a new runner. Run seeds 0..999 for every property.
B is caught only if the buggy model fails that property, one recording and
TWO replays of the LOWEST failing seed give byte-identical traces, and the
fixed model passes every one of the 1,000 seeds. Use the production driver's
`tide-admit ... --tide-run-test tests::<test> --seed N`, then twice the same
build arguments with `--replay <run.manifest.json>`. Record tool/std/source
identities, command, environment, exit status, failure, manifest trace_hash,
trace.ndjson SHA-256/event count and byte cmp for all three traces. Each
replay must report `replay reproduced`, reproduce the property's failure
and create exactly one run directory. Replay is through the driver, not a
second arbitrary test invocation. For non-hang properties, an unrelated
panic, deadlock/stall or compile failure is not property failure. Host
timeout without a simulator deadlock/stall report never counts.

Model corrections cite the exact source discrepancy and lines BEFORE any
rerun, preserve every earlier result and increment a correction count.
Never change property, trigger or seed budget. Rerun all seeds after each
correction, including replays/control or fixed proof invalidated by it.
A correction after a passing full run requires an independently read source
discrepancy and is flagged for review; it is not made to force a failure.
If faithfulness cannot be preserved, say unportable and stop that candidate.
A portable but uncaught model says not caught, with the failed condition.

## 6. Arm A: locate and port the concurrent core

At each of the first ten pool crates' pinned published versions, use the
verified extracted archive as source. After go only, read Cargo.toml's lib
entry and follow the non-test module graph. Starting from every non-test
use of thread spawn/scope, channel, lock or atomic operations, follow calls,
callbacks, trait dispatch and shared-state reads/writes that form the
concurrent core. Include producer, consumer, initialization, shutdown/drop
and error paths that can interact with that state; include their required
same-repository helpers. Record a source call/state map, roots, included
functions and excluded setup, and the union's nonblank/non-comment line
count. Do not cherry-pick a short suspected bug path or omit another
interacting root to fit 400 lines. Unresolved dispatch, missing generated
source, an external dependency on the core path, unsupported behavior, or a
union over 400 lines makes that crate's core unportable; keep its pool row
and explain. No replacement crate and no partial port presented as the
whole concurrent core. Independent components may have separate tests,
but their source union must fit the same 400-line budget (**narrowing**).

Port faithfully into Tide v0 using only §§4-5's maps and narrowings, recording
every gap. Apply the E3 concurrency and E4/E5 expressibility checks to the
core and property; E1/E2 retrospective-fix requirements do not apply to A.
The newest archive's mechanical eligibility does not promise core
portability. No bug is inferred from the scan or from inability to port.

Properties come ONLY from this published version's documentation (including
packaged README/rustdoc), tests, or invariants the code asserts or documents.
For each, record a short exact quote, archive path/lines and archive hash,
and the property sentence, trigger, assertion and applicability/preconditions.
If packaged material refers to external documentation, use a version-pinned
copy provably matching this release and record its identity; otherwise do
not use it as a property. Tests can supply a property; they do not replace
the production code as the port's basis. No inferred wishlist invariant or
new promise. Write and hash the complete property set, with its source
quotes and test triggers, BEFORE any model build or run for this crate.
Freeze it in the record/property log and retain the copy. No after-run
adjustment, cherry-picking or extra properties prompted by a result. If no
quotable testable property exists, record that negative.

Run 1,000 seeds 0..999 per frozen property, with an identical test harness
where models are compared. Triage each failing seed FIRST against pinned
source and the mapping: if the model introduced/omitted a step, state,
ordering, lifetime or behavior, it is a model error. Correct only toward
the source under §5's correction rule, disclose it BEFORE rerunning, then
rerun every seed for every property. Keep earlier traces/results. If
correcting it needs an unsupported operation, mark unportable. If property
or trigger cannot be tested as frozen, mark not caught; do not repair the
property from the observation.

Only a remaining source-faithful failure is a candidate. State its suspected
fault and an SC schedule; construct a control model by removing ONLY that
fault (the smallest repair of the suspected production path), preserving
all other behavior, properties and test inputs. Record a line diff and the
reason this is a fault-removal control, not a weakened property or
serialized workaround. No control deletes tasks, adds scheduling points,
changes the trigger, or unconditionally suppresses the failing assertion.
If no faithful local control is possible, the candidate is not caught.

A is caught only when ALL hold: the port fails the frozen property on a
seed in 0..999; the lowest failing seed has a recording and two byte-identical
driver replays per §5; the fault-removed control passes all 1,000 seeds for
every frozen property; AND the same failure reproduces on the real crate
at the pinned published version under §7. Unrelated failures do not count.
No candidate is called a bug without this final reproduction. A model-only
candidate is reported as such, never as a new-bug catch.

## 7. Arm A real-crate reproduction, the sole upstream execution exception

Only after the model/replay/control conditions above pass, prepare a small
external harness under TARGET/reproduction/NAME using the actual published
crate pinned with `=VERSION` and the scanned archive's hash. Record the
harness, Cargo manifest/lockfile, features, compiler/target/environment,
source checksums, public trigger, property assertion and budgets before its
first reproduction run. Do not patch the production crate, replace its
synchronization, or test a hand-reimplemented core. Verify Cargo's registry
checksum against archive-hashes.tsv and that package/lock identifies the
pinned release. If a current toolchain cannot build it, use a compatible
recorded toolchain; do not change source or version to force success.

Default method: a stress test with actual std threads operating the crate's
real API, the source's synchronization, and the frozen trigger/property.
Use repeated synchronized starts/barriers outside the production path to
increase overlap without adding synchronization inside it. Predeclare a
finite worker count, iteration/attempt count and per-process timeout in the
reproduction record. Preserve commands, elapsed time and actual outcomes;
a timeout alone is not a reproduced hang without evidence of the same
blocked-state failure. A passing stress budget is not proof of no bug and
means this caught condition was not met. Do not enlarge a failed budget
silently or substitute a model for real execution.

Loom is allowed ONLY when that crate at the pinned version already supports
loom: use its existing cfg/features, loom synchronization adapter and test
conventions. Verify that support in the pinned source before choosing it.
Do not add a loom port to a crate that lacks one. Identify the actual crate
code exercised, assertions, exploration bounds and output. It must
reproduce the same source failure, not only the Tide model.

A recorded `cargo fetch --locked` is the only dependency acquisition stage;
save UTC, complete command/output, lockfile digest, registry checksums and
the resulting dependency identities. Subsequent build and execution use
`--offline --locked`, with networking absent from the harness and no
fallback fetch, entirely within TARGET. Fetch, build and runs all go
through the repository's heavy wrapper with closed stdin, timeouts and
CARGO_TARGET_DIR/TMPDIR under TARGET. Example form (not run in step 1):

```
bash ~/.cache/eon-agents/eon-overseer/heavy.sh env CARGO_BUILD_JOBS=4 \
  CARGO_TARGET_DIR=/mnt/build/tide-agents/fx-zu4f.10-target \
  TMPDIR=/mnt/build/tide-agents/fx-zu4f.10-target/tmp \
  timeout <secs> cargo test --offline --locked --manifest-path <harness/Cargo.toml> \
  -- --test-threads=4 </dev/null
```

Every other compilation/Tide run also uses that wrapper. Start only with
1-minute load below 12; coordinate any >15-minute run and booked quiet
windows with tide-lead. Leave wrapper holds queued; report a >30-minute
hold. No full workspace suites here; tide-lead controls final proof.

Compare real and model failures against the SAME property, trigger and
version. If only an older/newer release reproduces, disclose the version
gap but A's pinned-version condition fails. If real behavior contradicts
the model, return to triage and invalidate dependent model/replay/control
proof after any correction. Record unexplained outcomes as unresolved;
never relabel them into a catch. Do not use B's old fixed/buggy pair as an
A reproduction at a different version.

## 8. Record, negatives first, counts and delivery

Use `research/43-...` if free at the time of writing, otherwise the next free
number, without overwriting another record. Lead with what this experiment
does not show: supplied properties, finite SC seed sampling, model gaps,
self-attested examination chronology, and retrospective known/fixed bugs in
B versus an independently reproduced pinned-version failure in A. State
caught/not caught separately per arm. An unfinished screen is incomplete,
not a completed negative. No discovery rate or claim that Tide found
anything unaided; no predicted seed failure without a run.

Include the committed protocol/pool paths and digests, base and registration
commits, metadata/archive acquisition times and hashes, timestamp proofs
and go time, tooling/environment, first examination time, scan's limitations
and actual four selection counts. List EVERY pool crate (including those
outside A), every B keyword match with commit and first parent, duplicate
links, first failing E criterion and evidence, E2 quote if passed, later
criteria unassessed/additional, and unresolved evidence. A rows give core
scope/count, property quotes, first portability failure or models and
outcome. Never erase a negative when another candidate is caught.

Per modeled candidate include the property freeze and source quotes, source
mapping, adaptations/version gaps/all model gaps, corrections and earlier
results, precise run identities/results, lowest failing seed and all replay
hashes/comparisons, fixed/control runs, and real reproduction where required.
Count separately: snapshot, GitHub/star survivors, scan survivors, pool;
per arm targeted/screened/unresolved/unportable/eligible-or-portable/modeled,
properties, caught/not caught; for B matches and unique commits/bug sites
and E1-E5 FIRST failures. Counts describe this selection only.

Continue every target in each arm even after a catch. A catch creates a
case study in examples/case-studies following eon-discovery-race and the
existing runner conventions, citing upstream revision/source/license with
short attributed quotes, the property and model gaps, seed/replay evidence,
and A's real reproduction. Explain what Rust's existing tests/loom or other
tools can do for the same property without claiming uniqueness. If several
are caught, choose the shortest faithful model for the main exposition,
then clearest evidence, then pool/candidate order, and list all catches.
No catch means a documented negative, no case study. Nothing is posted
upstream: Lucca owns disclosure; a security-relevant candidate is reported
to tide-desk first for private handling. No out-of-scope driver/std/reference
changes, new pool, stretched model or widened caught rule is authorized.
