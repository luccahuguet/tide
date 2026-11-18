# fx-zu4f.11: agent-marker repository protocol

Registered by the commit containing this file and fx-zu4f.11-pool/, before
manual repository examination beyond the permitted mechanical counts and
scans. Examination remains gated on tide-lead stamping/upgrading the
committed artifacts and sending go.

The frozen criteria in `br --no-db show fx-zu4f.11` and its comments own the
contract. This protocol supplies operations and explicit narrowings only;
it never widens selection, eligibility, faithful-model, property or caught
rules. The bead prevails over any inconsistency, which must be disclosed.
The unit is an entire pool repository at its pinned commit, including every
workspace member. A package in that same repository is not an external crate.

WORK = `/mnt/build/eon-agents/worktrees/tide-fx-zu4f.11/target/work`.
TARGET = `/mnt/build/tide-agents/fx-zu4f.11-target`.
The builder starts no subagents, writes no Beads and posts nothing upstream.
Only tide-lead conducts governance reviews, stamps the registration and lands.

## 0. Registration boundary and prior knowledge

The snapshot was recorded ONCE as **2026-11-18T18:01:51Z**. The window is
**[2025-11-18T18:01:51Z, 2026-11-18T18:01:51Z)**, twelve calendar months.
The source worktree base is `ea7bb313eac1a66f37b643393bbd5f7dfdeb91e2`.
Acquisition observations span their actual recorded times; stars and cloned
HEADs are not represented as an atomic measurement at snapshot time.

Before go, the permitted upstream observations are only search responses,
repository metadata, the default-branch activity/marker message counts,
clone identities/ref inventories, and scan.py's mechanical source/dependency
filter. No human/manual inspection of upstream messages, history, issues,
PRs, documentation, policies or source occurs. Search and log message bodies
are consumed programmatically, never printed for assessment. Read-only
blob-less clones and materialized source remain under TARGET. Nothing
upstream is built, run, sourced, installed or executed in step 1.

The pre-registration method reading is the instructions, relevant Beads and
comments, fx-zu4f.10's registered protocol and scanner from its branch,
fx-zu4f.9's protocol, research/41 and the r1/r2 reviews, plus official GitHub
commit-search documentation and Git v2.43 promisor-fetch source for the
mechanical blob-acquisition command. Earlier records contain known examples; they
do not authorize any new per-repository assessment. Scanner tests use only
synthetic fixture source. Python standard-library scripts are used because
this brief explicitly requests scan.py/admission-obs.py and reuse of .10's
Python implementation; no dependency or compile is needed.

Step 1 ends only with the complete protocol and pool directory committed on
fx-zu4f.11, byte-identical to WORK. WORK/PROTOCOL-READY lists the commit,
every committed artifact's SHA-256, collection times and all selection counts.
Then STOP. Tide-lead adds and upgrades OpenTimestamps proofs and records
hashes before sending go. No catch hunting or admission observation on a
pool repository happens before go. Git dates and logs remain self-attested;
a timestamp proves existence by its attested time, not absence of earlier
private screening. Publication alone cannot establish screening order.
The later record reports actual proof identities and first examination time.

## 1. Candidate acquisition and repository filter

The candidate source was amended by tide-lead BEFORE registration, with no
repository manually examined. The original per-marker commit search met five
secondary HTTP 403 refusals in 39 requests despite three multi-minute
backoffs, and was stopped. Two complete leaves covered only 65 minutes of
the year, returning 1,825 commits and 828 repository names: an INCOMPLETE
aborted candidate list, never the pool. Raw responses, queries.jsonl,
error logs, failure evidence and the old collector remain at
TARGET/aborted-commit-search/ and TARGET/registration/api/. The committed
candidate-source-change.json records the amendment and why. The final
research record MUST disclose this aborted attempt and the source change.
Neither initial diagnostic probe was adopted as candidate selection.

The new source is `gh api --method GET search/repositories`, with query
`language:Rust stars:RANGE pushed:>=2025-11-18T18:01:51Z is:public`, sorted
by stars descending, `per_page=100&page=N`. The public qualifier applies
the frozen public-only criterion before collecting names. No topics,
repository names, marker observations or source properties narrow discovery.
The same activity, marker, expressibility and star criteria still decide the
pool. Repository search is a superset of the former eligible repository
source; a marker-share survivor necessarily has marker commits and meets
the language/star/window conditions. No author/code-quality inference is
made from the candidate source.

Start with stars >=1,000. At total_count >=1,000, split a finite interval
at its midpoint; split [lo,infinity) into [lo,2*lo-1] and [2*lo,infinity).
Fetch EVERY page of every complete leaf (<1,000), without a top-N cutoff.
An indivisible capped star value, incomplete response, refusal or missing
required metadata never silently becomes a complete candidate list.
Deduplicate by GitHub repository ID and retain canonical names. The actual
complete leaf ranges were 1,000..1,999 (924), 2,000..3,999 (557), and >=4,000
(658): **2,139 distinct candidate repositories**. Pagination is a recorded
sequence of observations, not an atomic GitHub transaction.

`pool-build.py` records every URL, start/end UTC, HTTP status, exit status,
body and full-response SHA-256, byte count and any reported search totals/
incomplete flag in queries.jsonl. All raw responses/headers/stderr and
successful immutable body caches remain under TARGET/registration/repository-api/.
A manifest is committed instead of the large raw responses. On resumption,
reuse only a successful response for the exact same URL.

For every distinct candidate, request `gh api repos/OWNER/REPO`, following
redirects. Record requested/canonical names, ID, primary language, stars,
default branch, public/private, archived/fork flags, observation time and
outcome in candidates.tsv; the URL and response digest are linked through
queries.jsonl. Keep ONLY primary language Rust, stars >=1,000, not archived,
not a fork, and public. All exclusions and their first reason remain in the
candidate table. Canonical duplicates must be resolved before a final pool,
never double-counted. An unavailable response is unresolved, not excluded.

Space API request starts by at least TWO seconds, with one in flight. On
ANY HTTP 403 or 429 wait >=300 seconds before retry, then >=600 before a
second retry; honor a longer Retry-After or exhausted primary reset time.
Other transient errors wait 30 then 60 seconds. Preserve every failed
attempt. After persistent refusal/incompleteness, write WORK/BLOCKED with
evidence and STOP without a partial registration. Git transfer refusals
use the same >=5-minute backoff where they report 403/429. No rate-limit
policy file is edited and no alternate identity/IP bypass is used.

## 2. Activity, marker share, expressibility, and fixed pool

For each metadata survivor, make ONE read-only commits-only clone at a time:

```
git -c core.hooksPath=/dev/null clone --filter=tree:0 --no-checkout \
  --single-branch --shallow-since=2025-11-18T18:01:51Z \
  https://github.com/OWNER/REPO.git TARGET/repositories/REPOSITORY_ID
```

No hooks, submodules or upstream executable runs. Record URL/command,
start/end UTC, default branch, HEAD commit and ref/object-ID digest. The
commit is pinned at that clone, even if upstream later changes. Count using
only commit objects and messages. Delete the task-owned clone immediately
if it fails activity or the lowest authorized marker share (20%). Retain
only those that might survive a fallback, then fetch source for their scan.
If Git refuses a date-shallow clone because no commits were selected, or reports a shallow-information error, the
technical fallback removes only shallow-since, retaining tree:0 and a single
default branch, and counts the SAME window from complete commit history.
This changes no activity or marker criterion. Log the failed and effective
commands; retry transient Git failures politely rather than record a false
activity negative.

Only after activity and T20 pass, record the remote head/tag object-ID
inventory for later B history. Materialize every Cargo.toml and its validated
in-tree targets FIRST. If a forbidden normal/target dependency already
falsifies the keep rule, record that exclusion without fetching Rust source.
Otherwise materialize the Rust source and apply the lexical source checks.
This is the same conjunction as .10, with a cheaper first falsifier for
whole repositories. All earlier scan rows are refreshed in this diagnostic
order; zero source_files on a dependency exclusion means not assessed, not
that the repository has no source. Switch that clone's
promisor filter to blob:none when tree objects become needed. Read its
pinned tree and fetch ONLY Cargo.toml and .rs blob IDs in one batch using
Git's promisor-fetch negotiation mode; write those blobs as ordinary files,
never using smudge filters or checkout hooks. No README, issue, policy,
script or other artifact is opened, and no upstream executable runs.
Resolve file symlink chains using only the pinned Git tree. Recreate an
in-tree file link only after validating its entire chain and materializing
its target; reject escaping, missing or cyclic targets. Source files are
canonicalized inside the checkout for scanning; linked Cargo.toml files
still contribute their normal/target dependencies. Source fetch logs and the tree digest are
retained. Git's [v2.43 promisor-fetch implementation](https://github.com/git/git/blob/v2.43.0/promisor-remote.c)
supplies the batch-fetch negotiation mode. This is a checkout for the scanner, not a build.

`count-activity-markers.py` consumes a NUL-delimited full-message export:

```
git -C CLONE log PIN --since-as-filter=2025-11-18T18:01:51Z \
  --until=2026-11-18T18:01:51Z \
  --format='%H%x00%ct%x00%B%x00' --no-show-signature
```

Count every commit reachable from the pinned default branch whose committer
second is >= start and < end, including merges, not only first parents.
`--since-as-filter` avoids prematurely pruning traversal at an old-dated
ancestor; the final timestamp comparison enforces the exact half-open window.
Save export digest, command, total and marker count, without publishing or
manually assessing messages. A root commit counts toward activity. Keep
activity >=50. Count a commit at most once if any marker matches.
The exact Python regex (full-message search; aider only at subject start) is:

```
(?im:Co-Authored-By:[ \t]*Claude|Generated with \[?Claude Code|Co-Authored-By:[^\n]*(Codex|Copilot|Cursor|Aider|Devin|OpenHands|Jules|Amp|Gemini)|Generated (with|by) (Codex|Copilot|Cursor|Aider|Devin))|(?i:\Aaider:)
```

The marker share is marked/default-branch window commits, not a measured
fraction of source written by AI. The frozen expression uses name substrings
(e.g. Amp can match inside a longer name); incidental matches are possible.
No author-identity inference or manual relabeling is made.
activity-markers.tsv retains every metadata survivor's pin, total, marked,
share, activity outcome and outcomes for T = 50/40/30/25/20%.

Run the committed scan.py on all activity survivors with marker share >=20%
so every possible fallback count is available, not just the first 30. The
mechanical scan is fx-zu4f.10's lexical expressibility filter, adapted to
repository checkouts, with its mask/cfg/import/module functions reused.
For every Cargo.toml, inspect normal and target normal dependencies,
including optional/inactive ones, respecting package renames and inherited
workspace dependency aliases. Build/dev dependencies do not count; unused
workspace dependency declarations do not count. Reject exact tokio, futures,
async-std, flume, loom or prefixes crossbeam*, parking_lot*, rayon*.

Include non-test library AND binary source of every package, hence every
workspace member. The lexical package inventory also includes non-workspace packages in
the same repository. This enlarges forbidden-use evidence, but is not a
verified Cargo membership graph; unconventional fixture packages are not
semantically classified. Positive evidence still requires a reached
production entry. Honor explicit
lib/bin paths and conventional auto entries, including src/bin. Ignore
build.rs, tests/test/benches/examples and definitely test-only cfg items;
retain production platform/feature branches. Follow ordinary/inline mod and
literal path attributes to identify source reachable from production entries;
only reached files provide positive std evidence. Scan the larger lexical
source superset for forbidden uses. The .10 scanner's masking retains
newlines and offsets and removes nested comments/literals and definitely
test-only items. It handles ordinary aliases and nested imports; imports
count as uses. No macro expansion, complete name resolution, generated
source or compiler graph is claimed. Custom test-framework attributes and
cfg_attr path rewrites are not interpreted. A custom bare Mutex is not assumed
std; core-only atomics are not positive std evidence.

Keep only a repository with positive std thread spawn/scope, mpsc, Mutex,
RwLock or atomic use, and no lexical Condvar, select!, async fn (including
async unsafe fn), .await, Waker or unsafe impl Send/Sync. Every Cargo.toml's
normal and target dependencies are checked. Qualified names and directly
aliased std/core Send/Sync traits are covered. **Scannability narrowing:** every Cargo.toml must be UTF-8 and parse under
Python tomllib. If any does not, exclude the repository as
`unscannable-manifest:<path>:<parser reason>` and assess no Rust source.
Do not silently omit it, guess a repair, or claim an upstream build failure;
the file may be an intentional fixture. This is an experiment/input limit,
not a Tide-language restriction or bug finding. Other parser/invariant
errors still fail visibly rather than silently omit a member or override an
exclusion.
When a resolved forbidden dependency already falsifies the conjunction,
unresolved aliases elsewhere do not override that valid first failure. A
potential survivor with unresolved inheritance still fails visibly; no
unknown dependency is silently accepted.
A synthetic regression fixes an inherited test-item removal bug: a final
test-only parameter must not consume its enclosing closing delimiter or
the production function body. All earlier source scans are recomputed under
the corrected scanner digest before registration; activity counts do not
depend on this source-masking change.

scan-output.tsv gives every scanned repository its pin, source-file count,
positive std evidence, outcome and first reason. First reason order is unscannable manifest, no
manifest, forbidden normal/target dependencies, no source, no std evidence,
Condvar, select!, async fn, .await, Waker, unsafe Send/Sync. Use sorted source paths
and manifest/package tuples for deterministic ties. No manual override or
post-output pool substitution is allowed.

Count both marker survivors and scan survivors at EVERY T. Initially T=50%.
If fewer than ten pass the scan there, take the highest of 40/30/25/20%
that yields >=10; if none does, take 20%. This choice uses counts alone,
never source assessment. Keep the 1,000-star bar unchanged.
Sort the chosen-T scan survivors by descending recorded stars, then full
canonical name ascending. pool.tsv is the first 30 (all if fewer), with
pool rank, full name, stars, default branch and pinned commit. No repository
is added, removed or replaced after registration. Arm B targets every pool
row; arm A takes the first ten. The registration summary must give actual
counts: distinct search repositories, metadata survivors, activity survivors,
marker survivors and scan survivors at every T, chosen T and pool size.

Registered selection outcome:

| Marker T | After activity and markers | After scan |
| --- | ---: | ---: |
| 50% | 66 | 5 |
| 40% | 98 | 5 |
| 30% | 137 | 5 |
| 25% | 164 | 5 |
| 20% | 208 | 8 |

**2,139** distinct search candidates; **2,105** after the repository filter;
**1,521** after activity. No T yields ten scan survivors, so the frozen
fallback is **T = 20%**, with **208** marker survivors and **8** scan
survivors. All eight form the pool because fewer than 30 survive. Arm B
covers all eight; arm A takes all eight because fewer than ten are available.
Nothing is substituted. Neither arm nor admission observations has run.

Repository search: 2026-11-18T18:21:38.922946+00:00 through
2026-11-18T18:23:36.087378+00:00. API/clone/scan acquisition completed
2026-11-18T20:34:15.052773+00:00; the last API response ended
2026-11-18T20:34:11.384518+00:00. Cached provenance normalization completed
2026-11-18T20:37:03.339010+00:00 without another API request or changes to
counts, pins, activity/scan bytes, directory inventory or pool bytes.
All **2164** new-source API responses are HTTP 200; the five original
commit-search refusals belong to the separate, disclosed aborted attempt.
Per-request times/digests are in queries.jsonl. activity-markers.tsv records
clone/count interval bounds: clone_ended_utc is the end of the combined
clone-and-count observation, not a bare transfer timestamp. clone-refs.tsv
freezes the cloned and remote head/tag inventories for B.


## Disclosure and privacy

Owner decision relayed by Lucca via tide-desk and recorded by tide-lead on
the bead at 2026-11-18 16:52 UTC, with the operational addition sent to
this builder: the repository will be public with its FULL history. Every
arm-A candidate and every arm-B finding about a bug still unfixed upstream
is written ONLY to
`~/.cache/eon-agents/tide-lead/fx-zu4f.11-private/`, outside every worktree,
never to WORK, a tracked/untracked worktree file, or a commit. Do not stage
and later delete an undisclosed find: history preserves it. Keep all A
per-repository working records, properties, model sources, traces, reproduction
sources and draft texts private from their first write, so a later failing
run cannot retroactively make a worktree copy private. Use the private
record and property log instead of WORK's logs for these cases. Public
upstream inputs and ephemeral compiler products still use TARGET; candidate
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
gives only a count of private candidates, no repository identity or revealing
detail. Non-dangerous findings become commit-eligible only AFTER Lucca
has filed the public issue; dangerous findings only AFTER disclosure
(the policy allows disclosure after a fix or 90 days, through Lucca).
Do not claim a report was sent merely because its draft exists. Launch
texts can say N bugs were found and reported privately only after that
report actually happened. A privately reviewed finding remains an extra and DOES NOT meet the RC
external-catch bar until PUBLIC at launch time. This superseding owner
decision was relayed and recorded at 2026-11-18 16:56 UTC. Its record stays
outside the repository until disclosure. When Lucca sends a private report,
tide-lead OpenTimestamps-stamps its SHA-256; both hash and proof remain
outside the repository until disclosure. This section changes no pool,
eligibility, property or caught criterion.

At classification time, for EACH pool repository, record whether CONTRIBUTING,
issue templates or its stated policy reject AI-generated reports, with the
URL and date read. Do not inspect those policies during step 1. A
non-dangerous find in such a project gets NO upstream issue draft: it is
listed publicly with a cited policy note instead, through the coordinator's
disclosure workflow. A dangerous find still gets a private security-channel
draft. The cited-note route is the exception to the public-issue prerequisite
above; agents still send nothing. This addition is the bead's second
disclosure comment, relayed by tide-lead to this builder on 2026-11-18.

## 3. Work after go: order and arm B candidate enumeration

Arm B runs first on every pool repository, then arm A on the first ten pool
repositories in descending star order (all if fewer than ten). No replacing
a repository that has no useful history or no portable core. Every pool
repository gets its own coverage row.

After go, acquire a full-history, read-only blob-less clone under TARGET
using ONLY the recorded head/tag object IDs, including the pool's pinned
default-branch commit. The step-1 clone is intentionally shallow and must
not be treated as full history. Fetch/unshallow by those identities without
moving the registered default-branch pin or introducing later refs. Record
commands, times and fetched object identities. Enumerate every commit
reachable from all those registered refs, not just the default branch,
window or newest release. Unavailable/deleted objects remain unresolved;
unreachable/deleted history is not claimed. No incomplete clone can become
a completed negative. Lazy historical blob acquisition is read-only and
must not move refs.

Use git log on that frozen clone to export commit SHA, all parents, UTC
commit time and the full message (`%B`, subject AND body, NUL-delimited).
Apply Python `re.search` with `re.IGNORECASE` to the full message, exactly:

```
\b(race|races|racy|racing|deadlock|deadlocks|deadlocked|dead-lock|hang|hangs|hanging|hung|livelock|lost wakeup|data race|stuck|freeze|freezes|frozen|starvation|starve|starved)\b
```

This is the coordinator-directed **whole-word narrowing** of the bead's
substring regex, registered before examination. tide-lead reports that .10's
unbounded matcher produced 5,076 rows, including `change` and `trace`;
whole-word boundaries remove those accidental embedded matches. Use the
EXACT listed expression above, case-insensitive over full messages, including
its explicitly listed inflections. Do not add words or silently intersect it
with the former matcher.

Apply the second narrowing: a candidate must touch a registered non-test
source directory. `b-source-dirs.tsv` freezes those directories at each pool
pin before go, derived mechanically from source_texts' REACHABLE production
library/binary/module files whose masked non-test text is nonempty. Take each
file's parent directory and collapse descendant directories covered by an
ancestor; include every workspace package. This is the same lexical module
walk used for positive scan evidence, not a manual bug-path selection. A `.`
root means the repository root. No directory is added after registration.

For each keyword match, obtain changed paths against its FIRST PARENT with
`git diff-tree --no-commit-id --name-only --no-renames -r -z PARENT COMMIT`;
for a root commit use the same command with `--root COMMIT` instead of the
pair. Disabling rename detection exposes both the old deletion path and the
new addition path. Keep a match only if at least one changed path lies in or
beneath a registered source directory, with no path component tests, test,
benches, examples, .git or target, and basename not build.rs. The gate is on
directories, not inferred hunk intent; E1 still rejects test-only/doc-only
changes that happen to touch a production directory. Historical directories
absent from the pinned directory inventory do not get substituted.

No date bound, top-N or match-count cutoff. Save the full keyword export and
its digest, keyword-match count, path-gate exclusions and resulting candidate
count. Every match that passes BOTH registered gates gets a candidate row.
Assess pool order, then matched SHA in lexical order for reproducibility. A shared fix is assessed once per distinct bug
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
repository's non-test code by its own message or linked PR/issue. A test-only,
doc-only, added-test-only or skip/delete/retry-test-only change fails.
Any workspace member's non-test fix is in scope: the unit is the whole pinned repository. A publicly
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
The .10 protocol records raw Mutex/RwLock and SC atomic APIs as planned/
unshipped at its own base, with weaker orderings behind a planned unsafe
grant. The eventual record cites actual tool/std identities. These plans
provide no weak-memory simulation. The 400-line cap, unsafe and
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
the pinned A commit.

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

At each of the first ten pool repositories' pinned commits, use the verified
checkout as source. After go only, read every workspace member's library
and binary entries and follow the non-test module graphs. Starting from every non-test
use of thread spawn/scope, channel, lock or atomic operations, follow calls,
callbacks, trait dispatch and shared-state reads/writes that form the
concurrent core. Include producer, consumer, initialization, shutdown/drop
and error paths that can interact with that state; include their required
same-repository helpers. Record a source call/state map, roots, included
functions and excluded setup, and the union's nonblank/non-comment line
count. Do not cherry-pick a short suspected bug path or omit another
interacting root to fit 400 lines. Unresolved dispatch, missing generated
source, an external dependency on the core path, unsupported behavior, or a
union over 400 lines makes that repository's core unportable; keep its pool row
and explain. No replacement repository and no partial port presented as the
whole concurrent core. Independent components may have separate tests,
but their source union must fit the same 400-line budget (**narrowing**).

Port faithfully into Tide v0 using only §§4-5's maps and narrowings, recording
every gap. Apply the E3 concurrency and E4/E5 expressibility checks to the
core and property; E1/E2 retrospective-fix requirements do not apply to A.
The pinned checkout's mechanical eligibility does not promise core
portability. No bug is inferred from the scan or from inability to port.

Properties come ONLY from this pinned commit's documentation (including
README/rustdoc), tests, or invariants the code asserts or documents.
For each, record a short exact quote, repository path/lines and pinned commit,
and the property sentence, trigger, assertion and applicability/preconditions.
If pinned repository material refers to external documentation, use a version-pinned
copy provably matching this commit and record its identity; otherwise do
not use it as a property. Tests can supply a property; they do not replace
the production code as the port's basis. No inferred wishlist invariant or
new promise. Write and hash the complete property set, with its source
quotes and test triggers, BEFORE any model build or run for this repository.
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
every frozen property; AND the same failure reproduces on the real repository code
at the pinned repository commit under §7. Unrelated failures do not count.
No candidate is called a bug without this final reproduction. A model-only
candidate is reported as such, never as a new-bug catch.

## 7. Arm A real-crate reproduction, the sole upstream execution exception

Only after the model/replay/control conditions above pass, prepare a small
external harness using the actual repository at the pool's exact pinned
commit. Its source fixture and harness source, manifest/lockfile and outcome
records live in the private directory from their first write; build products
stay under TARGET. A pristine read-only upstream checkout stays under TARGET.
Use pinned path dependencies or git rev dependencies for its actual packages;
verify every package source, repository HEAD/tree, lock identity and features.
Record the harness, lockfile, compiler/target/environment, public trigger,
property assertion and budgets before its first reproduction run. Do not patch
production source, substitute synchronization or use a hand-reimplemented core.
If a current toolchain cannot build it, use a recorded compatible toolchain;
never change source or commit to force success.

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

Loom is allowed ONLY when that crate at the pinned commit already supports
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
  CARGO_TARGET_DIR=/mnt/build/tide-agents/fx-zu4f.11-target \
  TMPDIR=/mnt/build/tide-agents/fx-zu4f.11-target/tmp \
  timeout <secs> cargo test --offline --locked --manifest-path <harness/Cargo.toml> \
  -- --test-threads=4 </dev/null
```

Every other compilation/Tide run also uses that wrapper. Start only with
1-minute load below 12; coordinate any >15-minute run and booked quiet
windows with tide-lead. Leave wrapper holds queued; report a >30-minute
hold. No full workspace suites here; tide-lead controls final proof.

Compare real and model failures against the SAME property, trigger and
version. If only an older/newer release reproduces, disclose the version
gap but A's pinned-commit condition fails. If real behavior contradicts
the model, return to triage and invalidate dependent model/replay/control
proof after any correction. Record unexplained outcomes as unresolved;
never relabel them into a catch. Do not use B's old fixed/buggy pair as an
A reproduction at a different commit.

## 8. Record, negatives first, counts and delivery

Use the next free research number at the time of writing, without overwriting another record. Lead with what this experiment
does not show: supplied properties, finite SC seed sampling, model gaps,
self-attested examination chronology, and retrospective known/fixed bugs in
B versus an independently reproduced pinned-commit failure in A. State
caught/not caught separately per arm. An unfinished screen is incomplete,
not a completed negative. No discovery rate or claim that Tide found
anything unaided; no predicted seed failure without a run.

Include the committed protocol/pool paths and digests, base and registration
commits, metadata/clone acquisition times and hashes, timestamp proofs
and go time, tooling/environment, first examination time, scan's limitations
and actual selection counts at every threshold. List EVERY pool repository (including those
outside A), every B candidate passing BOTH registered gates with commit and first parent, duplicate
links, first failing E criterion and evidence, E2 quote if passed, later
criteria unassessed/additional, and unresolved evidence. A rows give core
scope/count, property quotes, first portability failure or models and
outcome. Never erase a negative when another candidate is caught.

Per modeled candidate include the property freeze and source quotes, source
mapping, adaptations/version gaps/all model gaps, corrections and earlier
results, precise run identities/results, lowest failing seed and all replay
hashes/comparisons, fixed/control runs, and real reproduction where required.
Count separately: distinct search repositories, metadata survivors, activity survivors,
marker survivors at each T, scan survivors at each T, chosen T and pool;
per arm targeted/screened/unresolved/unportable/eligible-or-portable/modeled,
properties, caught/not caught; for B matches and unique commits/bug sites
and E1-E5 FIRST failures. Report raw keyword matches and path-gate
exclusions separately from candidates. Counts describe this selection only.

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

## 9. Admission observations and neutral framing

Only after go, run the committed `admission-obs.py --repo <pinned-checkout>`
for EVERY pool repository, with the same source scope and comment/string/test
masking as scan.py. Preserve one TSV row per lexical observation with original
repository-relative file:line. Report counts by repository and category and
cite the pinned commit. Keep ambient imports separate from references, and
report `std::fs`, `std::net`, `std::process`, `std::env`, and the clock
(`std::time::SystemTime`/`Instant`) separately. Imports themselves count as
imports only; they are not counted as executed calls.

The heuristic expands same-file ordinary nested use trees and direct aliases,
then resolves lexical paths and bare imported identifiers. It does not do type
resolution, cross-module re-exports, macro expansion or cfg evaluation beyond
definitely test-only source. Globs, shadowing, aliased re-exports and generated
source can produce omissions or false positives. An import or reference is
an observation of syntax that would require examination for Tide admission;
it proves neither execution nor a particular compiler diagnostic.

For canonical `std::thread::spawn` calls (including direct same-file aliases),
count a discarded JoinHandle only for a standalone `spawn(...);`,
`let _ = spawn(...);` or direct `drop(spawn(...));`. A direct `.join()` is
recorded separately; an assigned, returned or transformed handle remains
unresolved. These are restricted syntactic patterns, not a semantic lower bound: alias
shadowing can create false positives. There is no lifetime/escape inference,
Builder::spawn result analysis, wrapper, scope-spawn or interprocedural claim.
Generic-qualified calls and macro-generated calls can be omitted.
Record unresolved spawns separately, never silently treating them as detached.

Admission observations are DESCRIPTIVE, NEVER bugs, properties, simulator
findings or catches in either arm. Publish them separately from catch counts.
They describe what Tide would make explicit; no fault in a repository follows.
The report states the agent-marker selection and its limits plainly, never
judges authors, never equates marker share with a verified fraction of code
written by an agent, and never claims AI-written code is bad. Agent-written
code is Tide's target; marker-associated repositories are this experiment's
operational selection. Every pool repository's AI-report policy, URL and date
is recorded after go under the disclosure section.

## Amendment A1: owner-affiliated repositories (before go and before any examination)

Added by tide-lead on 2026-11-18, after the pool was registered in 759a196 and before any
pool repository was examined, stamped or given go. It changes no pool, rank, eligibility,
property or caught criterion. It only changes how a catch counts.

Pool rank 8, Yazelix/nova, belongs to the Yazelix GitHub organization, which also hosts the
owner's own projects (for example Yazelix/starcompass, the source of this repository's
agent protocols). A catch in a repository owned or maintained by the Tide owner is not an
external catch. Such a repository stays in the pool and is examined and reported exactly
like the others. A catch there is reported, in its own row labelled owner-affiliated, and
does NOT meet the RC's external-catch bar. Pool denominators count it; external-catch
counts do not. The same rule applies to any other pool repository found to be owned or
maintained by the owner; the report names the evidence.

## Amendment A2: owner-affiliated repository removed (supersedes A1; before go and before any examination)

Relayed owner decision (Lucca, through tide-desk, 2026-11-18), recorded by tide-lead:
"nah, remove nova / we dont want to shit the bed twice / eon is already enough self-own,
plus eon is replacing nova". It was decided before any pool repository was examined and
before go.

Yazelix/nova (registered pool rank 8) is EXCLUDED from the pool because it is
owner-affiliated: it is the owner's own project, in the Yazelix organization. This is an
owner-affiliation exclusion and says nothing about the repository's code. A2 supersedes A1:
there is no owner-affiliated row left to report.

The working pool is research/protocols/fx-zu4f.11-pool/pool-amended.tsv: the registered
pool.tsv minus that one row, 7 repositories, with ranks 1-7 unchanged. pool.tsv stays as
registered, with its proof, as the record of what the mechanical selection produced. There is
NO backfill. The selection rule stays mechanical, and the next repository by that rule
would lie below the registered T floor, which the rule does not allow. Every denominator
uses 7, and the report states the exclusion and this reason.

The registration-time proof of this file (with A1) is fx-zu4f.11-protocol.md.a1.ots. It
verifies against the file at commit 9d797ef. This file, with A2, is stamped anew before go.
