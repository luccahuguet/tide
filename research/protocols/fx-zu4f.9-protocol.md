# fx-zu4f.9 operational protocol

Written 2026-11-18 before any bug of the pool is examined, against worktree
base `1e673c5` (branch fx-zu4f.9), toolchain `nightly-2026-08-09`, gh
2.104.0, git 2.43.0. The frozen criteria in the body of bead fx-zu4f.9 stay
binding. This file adds method and the narrowings marked **Narrowing**. It
never widens the pool, the eligibility rules, the faithful-model rule, the
property rule or the caught rule. Where this file and the bead disagree, the
bead wins and the disagreement is reported. It reuses the method of
fx-zu4f.7's registered protocol (SHA-256
`5038d09ba8b6441870ab56768f3b016370c61db4933a33ef36503f63410d39e4`),
sections 5-8 nearly verbatim, and fx-zu4f.6's (SHA-256
`0a02f5be72ea01ca887498fc5d938a35cfc9c25d1de3fa728a89ca5a46576308`). The
owner's instruction for this round is "the same rules as fx-zu4f.6 and .7,
plus E5" (tide-desk's notes, bead comment of 2026-11-18 14:13 UTC); where the
two earlier protocols differ, this one takes the stricter rule.

`WORK` is `/mnt/build/eon-agents/worktrees/tide-fx-zu4f.9/target/work`;
`TARGET` is `/mnt/build/tide-agents/fx-zu4f.9-target`. The record is
`research/41-qin-dataset-search.md`.

## 0. Registration and stop

This file and the pool export are committed on branch fx-zu4f.9 as
`research/protocols/fx-zu4f.9-protocol.md` and
`research/protocols/fx-zu4f.9-pool/section-6.1-blocking.csv` and
`.../section-6.2-non-blocking.csv`, byte-identical to the files in `WORK`.
The builder then writes `WORK/PROTOCOL-READY` and stops. No fix commit, pull
request, issue, advisory or reproduction of any row is read before tide-lead
records the SHA-256s on fx-zu4f.9 and sends "go". The record reproduces this
file verbatim with its digest. The builder does not write `.beads/`.

**What was read before this file was written.** Disclosed so that reviewers
can judge what the protocol's author knew. All by read-only GET, on
2026-11-18:

- `https://docs.google.com/spreadsheets/d/1GWbiPSBCIbqH2g3Vzc-o2XdxOskXyDhkJjNUjFi1pzs/htmlview`
  (14:13:53.570Z), only to find the tabs' gids: `section-6.1-blocking` is
  gid `1219539227`, `section-6.2-non-blocking` is gid `1166836418`;
- the two exports (section 1);
- the rust-study README (`repos/system-pclub/rust-study/readme`) and the
  names of the entries in `section-6-thread-safety-issues/` at
  `403b06f30809d10d2bb066d2fbcd33caf3446d17` (`reproduced_concurrency_bugs`,
  `section-6-1-blocking-bugs`, `section-6-2-non-blocking-bugs`); none was
  opened;
- `repos/OWNER/REPO` metadata for the 21 repositories named in section 2, to
  name them correctly.

Of the exports, the builder read the header records, the record counts, the
non-empty cells of columns A and B by position, the shape of column B's
values (40 hex digits, a `-N` suffix, or `redox#N`), the shape of column C's
values (`project#N`, `RUSTSEC-N-N` or empty), and the distinct values of
column D with their counts. No commit, issue, pull request, advisory,
reproduction or other per-bug evidence was read, and no row was assessed.

## 1. The pool

**Export.** Each tab was exported once, by

```sh
curl -sS -L -D <tab>.headers -o <tab>.csv \
  'https://docs.google.com/spreadsheets/d/1GWbiPSBCIbqH2g3Vzc-o2XdxOskXyDhkJjNUjFi1pzs/export?format=csv&gid=<gid>'
```

which answered with a 307 redirect to `doc-0o-88-sheets.googleusercontent.com`
and then HTTP 200, `content-type: text/csv; charset=utf-8`. The raw bodies,
the headers and the times are in `WORK/export/`.

| Tab | gid | Start (UTC) | End (UTC) | Bytes | CSV records | SHA-256 |
| --- | --- | --- | --- | ---: | ---: | --- |
| `section-6.1-blocking` | `1219539227` | 2026-11-18T14:14:14.474Z | 2026-11-18T14:14:16.539Z | 8,843 | 106 | `f7946a0d61f2478f3cf7a13ba008062c5106b45e0e7273dc15070a082ac8dac8` |
| `section-6.2-non-blocking` | `1166836418` | 2026-11-18T14:14:19.543Z | 2026-11-18T14:14:20.511Z | 5,989 | 62 | `4f5ce8902b98686804d315e44ec3fe683b1ff8844c99c4001b64506cffe9c1a0` |

**Data rows.** CSV records are counted from 0 as Python's `csv.reader`
returns them. In `section-6.1-blocking`, records 0-2 are headers (record 2
names the columns: B `commits`, C `BugID`, D `App`, E `Patching Time`); in
`section-6.2-non-blocking`, records 0-1 are headers (record 1 names the same
columns B-E) and record 2 is empty. A data row is a record after the headers
whose column B is not empty. That gives **59** blocking rows (records 4-69)
and **41** non-blocking rows (records 3-50), **100** in all, which equals the
100 concurrency bugs that the paper and the artifact README ("we studied 70
memory and 100 concurrency bugs") report. Blocking record 71 (`59` in column
A, column B empty) is the tab's own count, equal to the 59 blocking rows, and
is not a row;
the records after the data rows are the authors' summary tables and are not
rows. The export is complete by this test; if a later read shows otherwise,
the builder stops and reports.

Rows are named `B01`-`B59` and `N01`-`N41`, in file order. Each gets one
record row. The rows are assessed in that order, `B01` first, `N41` last.

**Duplicates.** Column B values compared after removing a trailing `-N`
suffix name 99 distinct commits: one commit (`5f1ce7f281…`) appears twice,
as `…-0` and `…-1`, in blocking records 58 and 69. The authors count these as
two bugs fixed by one commit. Following the bead, the commit is assessed once
and both rows link to that assessment; within it, E2-E5 are judged for each
of the two bugs (each change site of the commit that the authors' two rows
denote), and each row records its own outcome. If the evidence does not say
which change site is which row, the assessment says so and judges the
commit's change sites together for both rows.

## 2. From column B to a repository, a fix and a buggy revision

**Repositories.** Column D names the project. The fix commit is looked up in
the repository below; GitHub's rename redirects are followed and the
resolved `full_name` is recorded.

| Column D | Repository (`full_name` on 2026-11-18) | Also searched |
| --- | --- | --- |
| `parity-ethereum` | `paritytech/parity-ethereum`, which redirects to `openethereum/parity-ethereum` (archived) | `openethereum/openethereum` (archived) |
| `servo` | `servo/servo` | - |
| `servo-webrender` | `servo/webrender` | `servo/servo` |
| `tikv` | `tikv/tikv` | - |
| `redox` | the `redox-os` group on `gitlab.redox-os.org` | the `redox-os` organization on GitHub (`redox-os/redox`, `redox-os/kernel` and its other mirrors) |
| `crossbeam` | `crossbeam-rs/crossbeam` | the archived `crossbeam-rs/crossbeam-channel`, `-epoch`, `-deque`, `-utils` |
| `rayon` | `rayon-rs/rayon` | - |
| `rust-threadpool` | `rust-threadpool/rust-threadpool` | - |
| `lazy-static.rs` | `rust-lang-nursery/lazy-static.rs` (archived) | - |
| `rand` | `rust-random/rand` | - |
| `tock` | `tock/tock` | - |
| `renderdoc-rs` | `ebkalderon/renderdoc-rs` | - |

The brief also names std (`rust-lang/rust`); no row's column D names it. A
row that turns out to fix std code is looked up in `rust-lang/rust`.

**Lookup, in order, stopping at the first success.**

1. `gh api --method GET repos/O/R/commits/SHA` in the column's repository,
   then in each "also searched" repository, in table order.
2. For `redox`: `GET https://gitlab.redox-os.org/api/v4/groups/redox-os/projects`
   (every page, `per_page=100`), then
   `GET /api/v4/projects/<id>/repository/commits/SHA` for each project in the
   returned order, then step 1 against each GitHub repository of the
   `redox-os` organization (`orgs/redox-os/repos`, every page).
3. `gh api --method GET search/commits -f q='hash:SHA'`; a result counts only
   when its repository is in the project's own organization or in the fork
   network of a repository in the table (a fork's commit is accepted only if
   its patch equals the one the row names, which a fork with that SHA
   necessarily has).

The response is saved under `WORK/rows/<row>/`. The commit is resolvable when
some step returns it (HTTP 200 with the commit's SHA). A row whose lookup
fails at every step fails E1, with each attempt listed. A lookup that fails
for a transient reason (HTTP 5xx, rate limit) is retried at most twice after
the limit's reset; each attempt is saved under its own name.

**Column B forms.** A 40-hex value is the fix commit. A `-N` suffix is
removed first (section 1). The value `redox#N` names issue or merge request
`N` of the `redox` project: it is looked up as a merge request, then an
issue, in the `redox-os/redox` project on `gitlab.redox-os.org`, then in
`redox-os/redox` on GitHub. **Narrowing:** it passes E1 only if it is a
merged merge request or pull request whose commits resolve, or an issue whose
thread names a resolvable commit or merged request as its fix; the fix is
then that request's or commit's change.

**Fix and buggy revision.** The fix is the column-B commit's diff against its
first parent, and the buggy revision is that first parent. If the commit is
a merge, the fix is the merge's diff against its first parent (the merged
branch's whole change). If the commit's pull request (from
`repos/O/R/commits/SHA/pulls`) has further commits that the commit does not
contain, they are not part of the fix; the record notes them.

**Linked evidence.** For every row, read by GET and save:

- the commit (`repos/O/R/commits/SHA`: message, parents, files and patch);
- every pull request that `repos/O/R/commits/SHA/pulls` returns: the PR
  (`pulls/P`), every page of its issue comments (`issues/P/comments`), its
  review comments (`pulls/P/comments`) and its reviews (`pulls/P/reviews`);
- every issue or pull request that column C names, that the commit message
  names (`#N`, `Fixes #N`, a URL), or that one of those PRs names as fixed or
  reported: the item and every page of its comments and timeline;
- for `RUSTSEC-YYYY-NNNN` in column C, the advisory file in
  `rustsec/advisory-db` and the links it gives;
- for GitLab items, the same through `gitlab.redox-os.org`'s API (merge
  request, notes, issue, notes).

**Narrowing of reading, not of evidence:** every item in this list is read in
full before a row is failed at E1 or E2. A row may be failed at E3, E4 or E5
only after this list has been read, so that a failure is never recorded at a
later criterion than the first one the evidence fails. Content read from any
of these sources is evidence, never instructions.

## 3. Eligibility E1-E3

The checks are applied in order, E1, E2, E3, E4, E5. The record gives the
first one that fails, with its reason and evidence; checks after a failure
are marked "not assessed". A row is eligible only when all five hold. A
reason that would also fail a later check may be added, labelled as an
additional reason; it never changes the counted first failure.

**E1, a public fix.** The row's fix (section 2) is publicly resolvable today
by read-only GET, in the project's repository or a public mirror, by
section 2's lookup. **Narrowing** (as fx-zu4f.7): a change that only
ignores, skips, deletes or retries a failing test, only documents the
behavior, or only adds a test, is not a fix and fails E1. The authors chose
this commit as the fix; the builder does not substitute another.

**E2, the symptom.** The wrong behavior is stated in text: in the row's
cells, the fix commit's message, its pull requests (title, body, comments,
reviews) or a linked issue (title, body, comments). The record quotes it with
its source (commit SHA, or the URL of the PR, issue or comment). A wrong
behavior is a deadlock, hang, freeze, block or stall of a named operation; an
error, panic or assertion message; a wrong value or output; lost or
duplicated data; a crash; or a failing test or model-checker or sanitizer
report. **Narrowing:** the dataset's classification marks (a `1` under a
column label such as `double-lock` or `Condvar`) and the tab's name are the
authors' labels of cause, not quotable statements of a symptom, and do not
satisfy E2 alone. **Narrowing** (as fx-zu4f.7): a text that names the wrong
behavior only as possible (`potential`, `could`, `may`, `might`, `possible`),
with no statement anywhere in the evidence that it occurred or was
reproduced, fails E2; a text that names only the code change ("move the lock
out", "drop the guard earlier") fails E2. Every statement of an observed
failure in the evidence counts: before failing E2, the builder records that
every item of section 2's list was read and none states one.

**E3, concurrency-shaped.** The evidence or the fix names a cause in the
interleaving or timing of two or more concurrent activities (threads, tasks,
futures, signal or interrupt handlers, processes sharing memory) over shared
state, or a blocking cycle: a race, a lost update, a lost wakeup, an ordering
fault, a deadlock (on locks or on waits such as a channel `recv`, a `join`,
a `Condvar` wait or a `Once`), a hang, or a flake whose outcome depends on
relative timing. A thread that blocks on a lock it already holds (a double
lock) is a deadlock and passes E3. The dataset's placement of the row among
the authors' concurrency bugs, and its labels, may be cited as supporting
evidence of the cause. **Narrowing** (as fx-zu4f.7): a flake whose named cause
is not concurrency fails E3, and so does a performance or contention report
with no wrong result and no hang.

## 4. Old code: the version-gap rule

Most rows fix code from 2016-2019. Old versions are modeled as they are.

- The model follows the project source at the buggy revision, never later
  code, and never the authors' reproduction (section 6).
- The record lists every version gap: each old std or crate API on the path
  (for example pre-2018-edition syntax, `try!`, `std::sync::mpsc` before its
  1.67 reimplementation, `crossbeam::scope`, old `parking_lot` or
  `crossbeam-channel` APIs), and the section-5 mapping used for it.
- **Narrowing:** a path whose wrong behavior depends on the old std's or an
  old crate's implementation internals, rather than its documented
  behavior at that version, is unportable at E4, since the model would have
  to rebuild that implementation (for example a bug inside the old `mpsc`
  flavor upgrade, or a std `Mutex` poisoning quirk of one release).
- Syntax and edition differences (`try!`, `extern crate`, trait objects
  without `dyn`, `Box<FnBox>`) are not gaps when the model's Tide code
  expresses the same computation; the record lists them in one line.

## 5. What Tide v0 offers, how the simulator runs it, and E4-E5

This section is fx-zu4f.7's section 5. `crates/`, `design/reference.md` and
`design/03-authority-concurrency.md` are unchanged between fx-zu4f.7's base
`afb17ea` and this base `1e673c5` (`git log afb17ea..1e673c5 -- crates/
design/reference.md design/03-authority-concurrency.md` is empty). It is
written from `design/reference.md` (§§6.8, 6.9, 7.1-7.4, 11) and
`design/03-authority-concurrency.md` (§1.2 CAP-2 and CAP-3, §1.3, §6.4, §6.5
LOCK-1 to LOCK-6, §9.3), and the std (`crates/tide-std/src/sync.rs`,
`sync/mpsc.rs`, `task.rs`, `task/scheduler.rs`).

**What v0 offers for the operations these paths use.**

| Operation | In Tide v0 at `1e673c5` |
| --- | --- |
| threads | `tasks.scope` with `Scope::spawn` and `Task::join`; no detached threads (`thread::spawn` is `TD0602`) |
| mutual exclusion | `Shared<T>`: `with`, `with2`, `with_all`; the closure is `Pure`, so it cannot suspend, do I/O, touch a channel or another `Shared`, or nest a `with`; multi-cell calls acquire atomically in one global order and cannot deadlock; FIFO per cell; poison on panic |
| `Mutex`, `RwLock` | not shipped: planned in `std::sync::raw` behind the `unsafe` grant |
| atomics | not shipped: planned in `std::sync::atomic`, sequentially consistent only (`load`, `store`, `fetch_add`, `compare_exchange`); weaker orderings planned in `std::sync::raw`; `core::sync::atomic` is `TD0520` |
| `Condvar` | none; "condition waits are channels" (03 §6.5) |
| `park`, `unpark` | none |
| futex, OS wait queues | none |
| `Once`, `OnceLock`, `LazyLock` | none (`OnceLock`, `LazyLock` are `TD0520`); `OnceCell`, `LazyCell` (not `Sync`) only |
| channels | `std::sync::mpsc` `channel(tasks)`, `sync_channel(n, tasks)`: `send`, `try_send`, `recv`, `try_recv`, `recv_deadline`, iteration; no `select` |
| shared ownership | `Arc::new(v, tasks)`, `sync::Weak`, no `strong_count`/`get_mut`/`try_unwrap` |
| time and yielding | `clock.sleep`, `clock.now`, `tasks.yield_now`, `tasks.checkpoint` |
| parallel regions | `std::par::join`, `par_*`: run sequentially under simulation |
| `unsafe`, raw pointers, `UnsafeCell` | only with the `unsafe` grant, which a case-study model does not take |
| `async`, `Future`, `Waker` | none (`TD0601`; `Waker` is `TD0520`) |

**How the simulator schedules them.** One task runs at a time. Scheduling
points are capability operations, `spawn`, `join`, every channel call (one
point on entry, before inspecting channel state), each `Shared::with*` (one
point before acquiring the set and one after release), `reset`,
`Weak::upgrade`, `yield_now` and checkpoints. Between two points a task runs
alone. At each point with several runnable tasks, a seeded PRNG picks one
uniformly; the seed fixes the whole run, and a replay manifest reproduces it.
The search is random sampling, not exhaustive enumeration (unlike loom). A
run where no task can run and no timer is pending ends with a deadlock
report; a task that polls while a timer waits ends, after 1,000,000 points,
with a stall report. A spin loop with no timer pending is never reported; it
runs until the seed's process times out.

**Which orderings it explores.** Only sequentially consistent interleavings
of the model's steps: every step sees every earlier step's writes. It never
reorders a load before a store, never delays a store's visibility, and never
splits a step between two points.

**Mappings (E4).** Each Rust operation on the path is plain computation or
one of these; each mapping used is listed as an adaptation in the record.
"External" is defined below: an external crate's operation is never mapped.

| Rust on the path | Tide v0 in the model | Gap listed |
| --- | --- | --- |
| `std::thread::spawn` + `join`; `thread::scope`; a same-repository scoped-thread helper (`crossbeam_utils::thread::scope` in crossbeam, `rayon-core`'s own spawn in rayon) | `Scope::spawn`, `Task::join` in `tasks.scope` | a never-joined thread becomes a child of the trigger's scope |
| a std lock, or a same-repository lock, whose guard covers only pure computation | one `Shared::with` per guard | lock fairness: FIFO per cell |
| `RwLock` read and write guards, same condition | one `Shared::with` per guard | readers serialize; the bug must not need two concurrent readers |
| an atomic operation (`load`, `store`, `swap`, `fetch_*`, `compare_exchange`, `compare_exchange_weak`, the old `compare_and_swap`) on an atomic field | one `Shared::with` per operation over the cell holding the structure's atomic fields, under sequential consistency | ordering argument ignored; `compare_exchange_weak` never fails spuriously |
| `thread::park`, `Thread::unpark`, `park_timeout` | a per-thread `sync_channel::<()>(1, tasks)` token: `unpark` is `try_send(())` with `Full` ignored, `park` is `recv()`, `park_timeout(d)` is `recv_deadline(clock.now() + d)` | `park` never returns spuriously |
| `std::sync::mpsc` | Tide's `mpsc` (`channel` for `channel`, `sync_channel(n)` for `sync_channel(n)`) | as fx-zu4f.6; `sync_channel(0)` rendezvous only if Tide's `sync_channel(0, tasks)` has the same semantics, else unportable |
| `thread::sleep`, `Instant::now` | `clock.sleep`, `clock.now` | virtual time |
| `thread::yield_now`, `spin_loop_hint`, `spin_loop`, a same-repository `Backoff::{spin, snooze}` | `tasks.yield_now()` | no real spin timing |
| an `UnsafeCell` or `MaybeUninit` slot whose every access the path's own synchronization orders | a field of the structure's `Shared` payload, one `with` per source access | a value, never uninitialized memory |
| a raw pointer to a node of the path's own linked structure | an index into a `Vec` of nodes in the same payload; nodes are never freed | no memory reclamation |
| `Arc<T>` sharing data | a borrow into the scope, or Tide's `Arc` | no reference counts |
| anything else | unportable, with the reason | |

**E4, portable to Tide v0.** The builder first identifies the affected code
path at the buggy revision (section 2): the functions the fix changes, the
code they race or block against, and the trigger the evidence reports. It
reads that source only by GET (`repos/O/R/contents/PATH?ref=SHA`, commit
diffs, GitLab's `repository/files/PATH/raw?ref=SHA`) or from a read-only
clone under `TARGET/upstream/OWNER__REPO` (`git clone --filter=blob:none
--no-checkout`, then `git show SHA:PATH`). Upstream code is never built, run,
tested or installed.

The path is portable when all of these hold:

- Every operation on it is plain computation or one of the mappings above.
- It uses no network, no subprocess and no external crate. Crates of the same
  repository (workspace members, and the crates of one repository's tree)
  are not external; std, core and alloc are not external to any project, but
  only their documented behavior is mapped (section 4). **Narrowing** (as
  fx-zu4f.6, which names them): an external crate's operation anywhere on
  the path, including its interleaving points, makes it unportable: for
  example `parking_lot` in any project other than parking_lot,
  `crossbeam-channel` or `crossbeam-utils` in a project other than crossbeam,
  `ipc-channel`, `futures`, `tokio`, `mio`, `grpcio`, `rocksdb`, `spin`.
  Test scaffolding outside the path that only builds the starting state may
  be replaced by setup, listed as an adaptation.
- **Narrowing:** no `async fn`, `.await`, `Future`, `Waker`, `Context` or
  futures-0.1 `poll`/`Task::notify` on the path (Tide v0 has none).
- **Narrowing:** no lock guard held across a blocking or effectful operation
  (`park`, `unpark`, a channel operation, a sleep, a spawn or join, I/O) or
  across another lock's acquisition, the same lock's included; no reentrant
  locking. `Shared::with` takes a `Pure` closure that cannot suspend, signal
  or nest, and `with2` cannot express nested acquisition in a chosen order,
  so such a path cannot keep its structure.
- **Narrowing:** no `Condvar` wait or notify. Tide v0 has no condition
  variable; its `wait` atomically releases a guard and blocks, and modelling
  it needs a waiter registry built from channels, model code standing in for
  a primitive, which is where a model could add or hide the bug. Condvar
  paths are "not eligible (E4)".
- **Narrowing:** no `Once`, `call_once` or `lazy_static!` initialization on
  the path whose blocking or ordering the bug involves; Tide v0 has no `Sync`
  once-cell, and modelling one is model code standing in for a primitive.
- **Narrowing:** no futex, OS wait queue, `pthread`, `WaitOnAddress`, `SRW`
  lock, system call, kernel scheduler, interrupt handler or other
  operating-system or hardware mechanism on the path (std's `sys` primitives,
  Redox's or Tock's kernel internals); these are platform behavior (E5) with
  no v0 operation.
- **Narrowing:** no `unsafe` beyond the two mapped forms (an ordered
  `UnsafeCell`/`MaybeUninit` slot, a pointer into the path's own linked
  structure); no allocation, deallocation or reclamation logic (epoch
  garbage collection, `Box::from_raw`, manual `drop_in_place`) on the path,
  no FFI, and no bug whose stated wrong behavior is memory unsafety (use
  after free, double free, a segfault, uninitialized reads, a data-race
  report from Miri, TSan or another sanitizer, a soundness advisory) rather
  than a wrong value, a lost or duplicated item, a panic the source raises,
  or a hang. This symptom exclusion applies however small the affected
  operation is.
- **Narrowing:** no `thread_local!` destructors, thread exit or process exit
  ordering, signal handlers, stack-size, CPU-count or affinity dependence.
- **Narrowing:** the modeled non-test path is at most 400 source lines at the
  buggy revision, comments and blank lines excluded.

**E5, within what the simulator explores.** Judged for each row that passes
E4, from the evidence and the fix. The record states the answers to (a)-(d)
and the candidate's model gaps, including every atomic operation modeled as
one interleaving point under sequential consistency. It fails E5 when any of
these holds:

- (a) **Weak memory.** The bug needs an ordering weaker than sequential
  consistency to appear: the evidence names a missing or wrong `Ordering`, a
  fence, a reordering, store buffering, or a weakly ordered architecture
  (Arm, AArch64, POWER, RISC-V) as the cause; or the fix changes only
  `Ordering` arguments, fences or `compiler_fence`; or it is reproduced only
  under weak-memory checking. A fix that changes orderings together with
  logic passes (a) only if the failing interleaving the evidence describes is
  possible under sequential consistency, shown by a schedule in the record
  before any run.
- (b) **A switch inside a step.** The failing interleaving needs another
  thread to run between two operations that the model has inside one step:
  two plain (non-atomic) accesses outside the mapped slot form, a torn
  multi-word read or write, or two accesses within one lock guard.
- (c) **Operating-system and hardware behavior.** It needs real preemption
  timing, a timeout against real time, scheduler fairness or starvation (a
  thread that never runs while others do, priority inversion), a spurious
  wakeup (of `park`, a `Condvar`, a futex or `compare_exchange_weak`), futex
  or kernel semantics, signals or interrupts, `thread_local!` destructors or
  thread-exit order, `fork`, CPU count, cache or false-sharing effects, or a
  specific operating system's primitive.
- (d) **A hang the simulator cannot report.** The symptom is a hang whose
  faithful model would spin without a pending timer (a livelock), since the
  simulator reports only deadlocks and timer stalls; a hang passes (d) when
  the faithful model's hang is a deadlock (no runnable task) or a stall.

Rows that fail E4 are "not eligible (E4)"; rows that fail E5 are "not
eligible (E5)". An eligible candidate whose model, once attempted, turns out
to need more than the faithful rule allows (section 7, step 7) is
**unportable**, with the reason. The record counts the three separately.

## 6. The authors' reproductions

`system-pclub/rust-study` at `403b06f30809d10d2bb066d2fbcd33caf3446d17`
holds, under `section-6-thread-safety-issues/reproduced_concurrency_bugs/`,
the authors' reproductions of some bugs. For a row, the builder may read the
reproduction that names the row's project and commit or bug ID, by GET,
only after that row's section-2 evidence has been read, and only as evidence
of the trigger: which operations interleave and which schedule fails. A
reproduction never satisfies E1 or E2, never supplies the property, and
never replaces the project source as the model's basis. Where the
reproduction and the source differ, the source wins and the record states
the difference. Reproductions are never built or run. The record lists, for
each row, whether a reproduction exists and whether it was read.

## 7. The faithful model

For each eligible candidate, in row order:

1. **Property first.** Before any model of that candidate is built or run, the
   record gets: the evidence's own statement of the expected behavior (the
   dataset row, the fix commit message, the PR or a linked issue), quoted,
   with its source; the property in one sentence; and the test's assertion in
   Rust form, with the trigger it runs. For a hang or deadlock symptom the
   property is "the trigger completes". Then the builder appends the
   record's SHA-256 and the UTC time to `WORK/property-log.txt`. The property
   is never changed after a run. If it cannot be tested as written, the
   candidate is "not caught", with the reason.
2. **Mapping.** The record gives a source-to-model table: each model item
   (function, state, interleaving point) against its source file and lines at
   the buggy revision SHA. It lists every adaptation (a mapping from section
   5), every version gap (section 4) and every gap (source behavior the model
   leaves out), as the Eon studies do.
3. **Buggy model.** Only the affected path, with its structure, shared state
   and interleaving points (atomic operations, lock guards, park and unpark,
   channel operations, spawns, joins, clock operations) kept. Nothing beyond
   what Tide v0's syntax and std require changes. It adds no bug the source
   lacks: no added scheduling point beyond the mappings' own, no removed
   synchronization, no state that the source does not have.
4. **Fixed model.** The buggy model with the fix's change applied to the
   modeled lines, through the same mappings, and nothing else. The record
   shows `diff buggy.rs fixed.rs`: apart from the `//@` lines and the header
   comment, every changed line maps to a line of the fix. Fix lines outside
   the path are listed as gaps.
5. **Test.** One `#[sim_test(seeds = 1000)]` that runs the trigger and asserts
   the property. It is byte-identical in the two models.
6. **Model corrections.** If a run shows that the model departs from the
   source (the failing interleaving is not one the source allows, or a
   scheduling point is missing), the builder corrects the model toward the
   source. Before the corrected model's rerun, the record cites the source
   lines the model departed from and describes the correction. The record
   keeps every earlier run and counts the corrections per candidate. A
   correction never changes the property, the test's trigger or the seed
   budget, and is never made to turn a pass into a failure. A correction made
   after a full 1,000-seed run that passed must be justified only by a source
   discrepancy found by reading, never by the run's result, and the record
   flags it for the reviewers. The verdict uses the last model's full
   1,000-seed runs.
7. **Unportable.** If the model would need more than steps 3-4 allow, the
   candidate is "unportable", with the reason, and no further runs are made.

## 8. Runs, the caught rule and replay

Every compile or run goes through
`bash ~/.cache/eon-agents/eon-overseer/heavy.sh env CARGO_BUILD_JOBS=4
CARGO_TARGET_DIR=TARGET TMPDIR=TARGET/tmp timeout <secs> <cmd> </dev/null`,
started only while the first field of `/proc/loadavg` is below 12, with
`book-quiet.sh --list` checked before any run that may last over 15 minutes.

**The 1,000 seeds.** The candidate's two models go in a temporary,
uncommitted `examples/case-studies/zz-probe-<name>/` (with a stub README) and
run through the fx-rpsb.1 examples runner:

```sh
cargo test -p tide-admit --test examples -- --test-threads=4
```

The buggy model declares `//@ expect: test fails <test>` with `//@ reports:`
lines for the failing count `(1000 seeds)` and the property's failure; the
fixed model declares `//@ expect: tests pass`. Before the declarations are
written, a first run with a placeholder declaration only reads the outcome;
the record keeps that output (failing seeds, counts, cause). Seeds are the
case indices 0..999.

**Caught** needs all three:

1. the buggy model fails the property on at least one seed in 0..999. For a
   candidate whose stated symptom is a hang or a deadlock, the property is
   "the trigger completes", and the simulator's deadlock or stall report on
   the faithful model is the property failing (fx-zu4f.6's registered rule);
   a run that the seed's timeout ends without either report is not counted as
   failing. For every other candidate, only the property's assertion counts;
   a build error, a deadlock or stall report, or any other panic is not the
   property failing;
2. for the lowest failing seed N, one recording and two replays through the
   driver give byte-identical traces:

   ```sh
   tide-admit --crate-type=lib --test --emit=link \
     --tide-std STD/libtide_std.rlib --tide-dep-dir STD \
     --out-dir WORK/replay/<name>/out WORK/replay/<name>/package/src/buggy.rs \
     --tide-run-test tests::<test> --seed N
   tide-admit <the same build arguments> --replay <recorded run.manifest.json>   # twice
   ```

   with `STD` the production std that the examples runner built under
   `TARGET/tmp/build/std` and the driver from `TARGET`. The package source is
   a byte copy of the landed `buggy.rs` (its SHA-256 recorded beside the
   original's). The recording must fail with the property's failure, and
   each replay must fail the same way, print `replay reproduced` and add
   exactly one new run directory. The record lists, for the recording and
   both replays: the exit code, the `replay reproduced` line, the manifest's
   `trace_hash`, the SHA-256 of each `trace.ndjson`, the event count, and
   `cmp` results showing the three traces equal byte for byte;
3. the fixed model passes all 1,000 seeds.

Anything else is "not caught", with the reason. Admission alone never counts.

## 9. The record, counts and selection

The record is kept as the work goes, in `research/41-qin-dataset-search.md`.
It holds: a citation of Qin, Chen, Yu, Song and Zhang, "Understanding Memory
and Thread Safety Practices and Issues in Real-World Rust Programs", PLDI
2020, and of the artifact (`system-pclub/rust-study` at `403b06f`, its
Google Sheet, the two exported tabs with their SHA-256s), stating that the
dataset's authors chose these bugs, not Tide; this protocol's path and digest
(the committed copy, not reproduced again); the export's method and times;
one row per data row (`B01`-`N41`: column B, C and D, the resolved
repository and fix, the outcome and its reason, and the E2 quote for every
row that passes E2); one assessment per unique commit, linked from its rows;
and, for each eligible candidate, its property (written before its runs),
mapping, adaptations, version gaps, gaps, runs and verdict.

Counts, reported separately: rows (100: 59 blocking, 41 non-blocking);
unique commits; duplicates; screened (every row); not eligible, by first
failing criterion E1-E5, by tab and by project; eligible; caught; not
caught; unportable; unresolved. They describe this sample only and are never
presented as a rate.

Every eligible candidate is assessed even after a catch. If several are
caught, the case study uses the one with the shortest buggy model, counted as
the non-blank, non-comment lines of `buggy.rs` before `#[cfg(test)]`; ties go
to the clearest evidence (an observed failure with a reproduction, and a fix
that names the issue), then to row order. The others are listed in the
record.

If one is caught, the case study goes in
`examples/case-studies/<project>-<short-name>/` with `buggy.rs`, `fixed.rs`
and a README in the shape of eon-discovery-race: negatives first (Tide found
nothing unaided, the property was supplied, the bug was already fixed
upstream, the dataset's authors chose the bug, what the model leaves out),
then the citation of Qin et al. and the artifact, the issue, the fix commit,
the license (from the repository metadata and its license file), the
property with its quote, the model and its gaps, the seed and replay
evidence, and what Rust needs for the same test (loom, Shuttle, Miri, and the
paper's own detectors), saying where the project already tests with loom or
a similar tool. The models are written fresh; quotes from upstream are short
and attributed.

If none is caught, the record lands as a documented negative, no case study
is written, and the report tells tide-lead, for fx-qlek.7. A screen with an
unresolved row is disclosed as incomplete and is not called a completed
negative. No widening, substitute pool or stretched model is used to avoid a
negative.
