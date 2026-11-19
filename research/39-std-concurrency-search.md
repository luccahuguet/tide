# Std-level concurrency bug search for a Tide simulation catch (fx-zu4f.7)

This is the second pre-registered search (see "What 'pre-registered' means here") for a
real, external concurrency bug that a Tide `sim_test` catches on some seed and the driver
replays, after [fx-zu4f.6](38-external-concurrency-search.md) found none in a broad GitHub
sample. The owner chose a std-level pool on 2026-11-18, so that the code paths would sit in
std and its neighbours rather than behind external crates. tide-lead froze the criteria in
bead fx-zu4f.7 before any candidate was examined, and the builder registered the operational
protocol below (SHA-256 `5038d09ba8b6441870ab56768f3b016370c61db4933a33ef36503f63410d39e4`,
recorded on the bead by tide-lead, who accepted its three mapping adaptations) before
sending any request.

**Result: a documented negative.** All 56 queries ran once, each answered 200 with complete
results. Of 162 unique issues, 43 fell outside the scope filter and 119 were screened. None
met all five eligibility criteria, so no property was written, no model was built and
nothing ran in the simulator. No case study follows from this search.

What this record does not say:

- It says nothing about whether Tide's simulator would find any of these bugs. It was never
  asked to, because no candidate was eligible.
- The counts describe this registered sample only. They are not a rate for std-level Rust
  concurrency bugs, or for any population.
- The criteria were frozen to keep the search from being stretched. Several excluded bugs are
  real, observed and fixed concurrency bugs; they are excluded because Tide v0 cannot express
  their code paths faithfully, not because they are not bugs.

## Counts

| | Count |
| --- | ---: |
| Queries (7 repositories × 8 terms) | 56 |
| Queries with no results | 18 |
| Raw results (at most 10 per query) | 215 |
| Unique issues | 162 |
| Duplicate rows (an issue already seen in an earlier query; 50 issues appear more than once) | 53 |
| Not candidates (scope): rust-lang/rust 27, crossbeam 5, rayon 11 | 43 |
| Screened (unique issues past the scope filter) | 119 |
| Not eligible, first failure E1 (no public linked fix) | 74 |
| Not eligible, first failure E2 (no observed symptom) | 11 |
| Not eligible, first failure E3 (not concurrency-shaped) | 8 |
| Not eligible, first failure E4 (not portable to Tide v0) | 26 |
| Not eligible, first failure E5 (outside what the simulator explores) | 0 |
| Eligible | 0 |
| Caught | 0 |
| Not caught | 0 |
| Unportable (eligible, but the model needed more than the faithful rule) | 0 |
| Unresolved | 0 |

Screened per repository: rust-lang/rust 11, crossbeam 25, parking_lot 22, dashmap 21,
flume 7, once_cell 13, rayon 20.

## Method

**What "pre-registered" means here.** The frozen criteria in the bead body reached git in
ledger commit afb17ea at 13:27:20Z, before the first search request at 13:34:14Z. The bead
body requires the operational protocol to be committed as well, before any candidate is
examined. It was not: tide-lead recorded its SHA-256 on the bead (br timestamp 13:33:52Z),
and the file stayed uncommitted in the work directory. The hash first reached git in ledger
commit 951e6c3 at 13:41:44Z, after the searches. The bytes embedded below match that hash.
Every timestamp here, git or br, is self-attested. Publishing the repository will show only
that these commits existed by the publication date, not the order before it. From fx-zu4f.8
on, a protocol is committed on the bead branch before any candidate is examined.

**Searches.** The 56 requests were sent once each, in the registered order, on 2026-11-18
between 13:34:14Z and 13:37:40Z, at least 3 seconds apart. Each went through `gh api
--method GET search/issues --include` (gh 2.104.0), with the frozen repository qualifier
and term followed by `is:issue is:closed created:<=2026-11-18`, `-F per_page=10 -F page=1`,
and no `sort` or `order` parameter. Every response was HTTP 200 with `incomplete_results`
false, so no query failed, none was retried and none was rerun. A shell script saved each
query's request, raw response with headers, and body (`search/RiTj.request.json`, `.http`,
`.json` in the builder's WORK directory), creating every file exclusively. All 56 were
saved before any result was opened. No item's repository differed from its qualifier, so
no rename was resolved.

**Repository metadata**, read at 13:37:53Z-13:37:57Z (`repos/`):

| Repository | Default branch | Stars | License |
| --- | --- | ---: | --- |
| rust-lang/rust | main | 122,234 | Apache-2.0 |
| crossbeam-rs/crossbeam | main | 8,645 | Apache-2.0 |
| Amanieu/parking_lot | master | 3,440 | Apache-2.0 |
| xacrimon/dashmap | master | 4,131 | MIT |
| zesterer/flume | master | 3,089 | Apache-2.0 |
| matklad/once_cell | master | 2,089 | Apache-2.0 |
| rayon-rs/rayon | main | 13,476 | Apache-2.0 |

(The SPDX field shows the first license GitHub detects; all but dashmap are dual
MIT/Apache-2.0.)

**Screening.** For each of the 162 unique issues, the issue, every page of its comments and
every page of its timeline were read by GET between 13:38:16Z and 13:44:05Z
(`issues/OWNER__REPO__N/`). Then each candidate fix was read: the pull request's metadata,
body and changed files (`pr-*.json`, `pr-*.files.json`), or the commit. Upstream source was
read only where an E4 or E5 judgment turned on it, through the contents API at the buggy
revision (crossbeam's `crossbeam-channel/src/waker.rs` at `4efd6ad` for crossbeam#1102,
and rayon-core's `sleep/mod.rs` at rayon#348's base for rayon#343). Nothing was cloned,
built or run. No request commented, reacted, starred or posted.

**Re-audit (round r1).** The Codex supervisor pass found E1 and E2 judgments that the
threads contradict: fix attributions buried in long threads (flume#74, rayon#938), an E1
pass resting on a mere mention (parking_lot#306), E2 failures on issues that report a Miri
error, a crash or a wrong value (rust#125526, parking_lot#258 and #346, dashmap#201), and
E2 passes recorded without the symptom quote the protocol requires. A fresh Opus subagent
then re-read every comment of all 120 screened threads against the registered E1 and E2
rules, and checked every fix it cited by GET; the builder applied its readings, kept the
E3-E5 judgments, and continued each promoted row in order. Every quoted symptom below is
an exact substring of the saved issue body or of the comment it links, posted before the
fix merged where it is a comment; a script checked each against the saved JSON and its
date. The subagent sent one read-only GraphQL query (a POST, against the GET-only rule)
to find rust#96706's connected PR; nothing was written. In round r2, rust#96706 moved to
the scope filter (the panic machinery), so no counted row rests on that request.

The checks follow the protocol in order: the scope filter, then E1 to E5. Each assessment
records the first one that fails; "✓" marks a criterion that was met before it, and later
criteria were not assessed. Where a later criterion would plainly fail too, the reason says
so, but the count uses only the first failure. Each row that passes E2 ends with its symptom,
quoted. "Doubtful" marks a reading the evidence leaves open; the reason gives the other
reading and where it would lead.

## Assessments

One row per unique issue, in order of first appearance. "First seen" is the query and
position where it first appeared; its rows in later queries link back here.

| First seen | Issue | Closed | Outcome | Reason |
| --- | --- | --- | --- | --- |
| R1T1.1 | [rust-lang/rust#161018](https://github.com/rust-lang/rust/issues/161018) | 2026-08-14 completed | not a candidate (scope) | `process::exit` and signal alt-stacks: not one of the listed sync or thread items. |
| R1T1.2 | [rust-lang/rust#93740](https://github.com/rust-lang/rust/issues/93740) | 2023-01-20 completed | not eligible (E1) | A tracking issue for new `Mutex`/`RwLock`/`Condvar` implementations; its 18 linked PRs implement that plan, not a fix of a reported bug. |
| R1T1.3 | [rust-lang/rust#110317](https://github.com/rust-lang/rust/issues/110317) | 2026-01-26 completed | not a candidate (scope) | `process::Command` (`pre_exec`, `SIGCHLD`). |
| R1T1.4 | [rust-lang/rust#124105](https://github.com/rust-lang/rust/issues/124105) | 2024-06-14 completed | not a candidate (scope) | `fs` (`closedir`). |
| R1T1.5 | [rust-lang/rust#141726](https://github.com/rust-lang/rust/issues/141726) | 2026-02-16 completed | not a candidate (scope) | Float tests. |
| R1T1.6 | [rust-lang/rust#29497](https://github.com/rust-lang/rust/issues/29497) | 2025-06-01 completed | not a candidate (scope) | `fs::remove_dir_all` on Windows. |
| R1T1.7 | [rust-lang/rust#136669](https://github.com/rust-lang/rust/issues/136669) | 2025-03-08 completed | not a candidate (scope) | Documentation-only (wording of the atomics docs). |
| R1T1.8 | [rust-lang/rust#124466](https://github.com/rust-lang/rust/issues/124466) | 2025-02-28 completed | not a candidate (scope) | `JoinHandle::join` of a spawned thread, outside `thread::scope` (the list is read as exhaustive); the thread stops at process exit. |
| R1T1.9 | [rust-lang/rust#27970](https://github.com/rust-lang/rust/issues/27970) | 2024-05-30 completed | not a candidate (scope) | `env::set_var`. |
| R1T1.10 | [rust-lang/rust#127576](https://github.com/rust-lang/rust/issues/127576) | 2024-08-23 completed | not a candidate (scope) | `fs::remove_dir_all`. |
| R1T2.1 | [rust-lang/rust#165950](https://github.com/rust-lang/rust/issues/165950) | 2026-11-16 not_planned | not a candidate (scope) | `Vec::capacity`. |
| R1T2.2 | [rust-lang/rust#156334](https://github.com/rust-lang/rust/issues/156334) | 2026-08-20 completed | not a candidate (scope) | Release-notes tracking for `thread_local!` destructors on Windows. |
| R1T2.4 | [rust-lang/rust#157390](https://github.com/rust-lang/rust/issues/157390) | 2026-06-07 completed | not a candidate (scope) | `File::lock` on Solaris. |
| R1T2.5 | [rust-lang/rust#145816](https://github.com/rust-lang/rust/issues/145816) | 2025-09-12 completed | not eligible (E1) | The fix it names, rust#145895 ("thread parking: fix docs and examples", "Fixes #145816"), changes the `Thread::unpark` doc example, a comment in `sys/sync/once/queue.rs` and tests, and no library code. The deadlock was in a doc example under Miri on aarch64-apple-darwin, inside the platform condvar (`sys/pal/unix/sync/condvar.rs:47`). |
| R1T2.6 | [rust-lang/rust#114851](https://github.com/rust-lang/rust/issues/114851) | 2025-01-23 not_planned | not eligible (E1) | Closed `not_planned`; no fix linked (a crossbeam branch was offered for testing, and triage asked for a reproduction). |
| R1T2.7 | [rust-lang/rust#139034](https://github.com/rust-lang/rust/issues/139034) | 2025-04-10 completed | not a candidate (scope) | Documentation-only (`get_mut` docs). |
| R1T2.8 | [rust-lang/rust#96706](https://github.com/rust-lang/rust/issues/96706) | 2025-01-03 completed | not a candidate (scope) | The panic machinery, which the R1 filter excludes: the interleaved messages come from std's panic hook writing to stderr, the thread says it "also happens for non-scoped threads", and the fix rust#122565 changes `library/std/src/panicking.rs` and test expectations, nothing under the listed sync or thread directories. |
| R1T2.9 | [rust-lang/rust#116179](https://github.com/rust-lang/rust/issues/116179) | 2023-10-18 completed | not eligible (E1) | No fix linked; folded into rust#116237 (TLS destructors after `thread::scope`'s join). |
| R1T2.10 | [rust-lang/rust#123458](https://github.com/rust-lang/rust/issues/123458) | 2024-07-18 completed | not eligible (E4) | E1 ✓ (rust#124881), E2 ✓, E3 ✓. `ReentrantLock` identified its owner by a TLS address that another thread reuses after the owner exits; the fix switches to `ThreadId`. The path depends on `thread_local!` layout and thread exit, which the E4 narrowing excludes. Symptom (issue body, [2024-04-04](https://github.com/rust-lang/rust/issues/123458)): “In practice, in some executions the assertion fails:” |
| R1T5.3 | [rust-lang/rust#125526](https://github.com/rust-lang/rust/issues/125526) | 2024-05-25 completed | not eligible (E4) | E1 ✓ (rust#125527). E2 ✓. E3 ✓ (two threads race through a shared guard). E4 fails: the observed wrong behavior is a Miri data-race report, memory unsafety, which the E4 narrowing excludes. The cause is a `Sync` impl that safe Tide code could not write. Symptom (issue body, [2024-05-24](https://github.com/rust-lang/rust/issues/125526)): “Instead, this happened: compiled successfully and miri reported a data race” |
| R1T5.5 | [rust-lang/rust#114949](https://github.com/rust-lang/rust/issues/114949) | 2023-08-21 completed | not a candidate (scope) | `env::var`. |
| R1T6.1 | [rust-lang/rust#155518](https://github.com/rust-lang/rust/issues/155518) | 2026-10-19 not_planned | not a candidate (scope) | Float-parse test. |
| R1T6.2 | [rust-lang/rust#148408](https://github.com/rust-lang/rust/issues/148408) | 2026-04-12 completed | not a candidate (scope) | `fs` metadata test. |
| R1T6.4 | [rust-lang/rust#147885](https://github.com/rust-lang/rust/issues/147885) | 2025-10-21 completed | not eligible (E1) | The fix rust#147910 ("handle spurious returns of `wait_timeout` in test") changes only the test. |
| R1T6.5 | [rust-lang/rust#146024](https://github.com/rust-lang/rust/issues/146024) | 2025-08-31 completed | not a candidate (scope) | `sys::process` on Windows. |
| R1T6.6 | [rust-lang/rust#132416](https://github.com/rust-lang/rust/issues/132416) | 2025-07-28 completed | not a candidate (scope) | WASI unwinding (toolchain). |
| R1T6.7 | [rust-lang/rust#133421](https://github.com/rust-lang/rust/issues/133421) | 2024-11-27 completed | not eligible (E1) | The closing fix rust#133435 disables the test on macOS under Miri; the other linked PR, rust#135162, is a release PR. The symptom was a Miri borrow-stack error (rust#121950). |
| R1T6.8 | [rust-lang/rust#89571](https://github.com/rust-lang/rust/issues/89571) | 2023-04-12 completed | not eligible (E2) | A safety hazard found by reading (`Once` relies on `park` not unwinding); no observed failure. It was resolved by rust#102412. |
| R1T6.9 | [rust-lang/rust#108147](https://github.com/rust-lang/rust/issues/108147) | 2023-03-02 completed | not eligible (E1) | No fix; the thread concludes the behavior is as intended. |
| R1T6.10 | [rust-lang/rust#99280](https://github.com/rust-lang/rust/issues/99280) | 2022-07-26 completed | not a candidate (scope) | `fs::create_dir_all` on OpenBSD. |
| R1T7.2 | [rust-lang/rust#146183](https://github.com/rust-lang/rust/issues/146183) | 2025-09-04 completed | not a candidate (scope) | `Path` equality. |
| R1T7.3 | [rust-lang/rust#115209](https://github.com/rust-lang/rust/issues/115209) | 2025-11-29 completed | not a candidate (scope) | `thread::current` in a global allocator (not a listed item). |
| R1T7.6 | [rust-lang/rust#120147](https://github.com/rust-lang/rust/issues/120147) | 2024-02-09 completed | not eligible (E3) | E1 ✓ (rust#120238), E2 ✓. Unchecked `pthread_mutex_lock` return codes: error handling, not an interleaving. Symptom (issue body, [2024-01-19](https://github.com/rust-lang/rust/issues/120147)): “On Solaris-derived kernels it will instead succeed.” |
| R1T7.7 | [rust-lang/rust#75446](https://github.com/rust-lang/rust/issues/75446) | 2020-09-05 completed | not a candidate (scope) | `fs::copy`. |
| R1T7.8 | [rust-lang/rust#79480](https://github.com/rust-lang/rust/issues/79480) | 2020-12-01 completed | not a candidate (scope) | `io::BufReader`. |
| R1T7.9 | [rust-lang/rust#76600](https://github.com/rust-lang/rust/issues/76600) | 2020-09-19 completed | not a candidate (scope) | `fork` in `process`. |
| R1T8.1 | [rust-lang/rust#165901](https://github.com/rust-lang/rust/issues/165901) | 2026-11-17 completed | not a candidate (scope) | `alloc` tests. |
| R1T8.2 | [rust-lang/rust#142411](https://github.com/rust-lang/rust/issues/142411) | 2025-06-25 completed | not a candidate (scope) | `fs::ReadDir` on Windows. |
| R2T1.1 | [crossbeam-rs/crossbeam#663](https://github.com/crossbeam-rs/crossbeam/issues/663) | 2021-02-20 completed | not eligible (E4) | E1 ✓ (crossbeam#998, named in the thread), E2 ✓ (a ThreadSanitizer report), E3 ✓. The path is crossbeam-epoch's lazily initialized global collector (`lazy_static`, an external crate, and a `thread_local!` handle) and its reclamation, and the fix makes epoch's fences visible to TSan: excluded by the E4 narrowings (external crate, `thread_local!`, reclamation logic). Symptom (issue body, [2021-02-19](https://github.com/crossbeam-rs/crossbeam/issues/663)): “ThreadSanitizer: reported 4 warnings” |
| R2T1.2 | [crossbeam-rs/crossbeam#1269](https://github.com/crossbeam-rs/crossbeam/issues/1269) | 2026-06-20 duplicate | not eligible (E1) | Closed as a duplicate; the linked PR crossbeam#1219 is not merged. |
| R2T1.3 | [crossbeam-rs/crossbeam#646](https://github.com/crossbeam-rs/crossbeam/issues/646) | 2021-01-12 duplicate | not eligible (E1) | Closed as a duplicate of #644; no fix linked. |
| R2T1.4 | [crossbeam-rs/crossbeam#1207](https://github.com/crossbeam-rs/crossbeam/issues/1207) | 2026-07-06 completed | not eligible (E4) | E1 ✓ (crossbeam#998), E2 ✓ (TSan), E3 ✓. The race is in epoch's deferred `free` during collection: reclamation logic, excluded by E4. Symptom (issue body, [2025-10-30](https://github.com/crossbeam-rs/crossbeam/issues/1207)): “WARNING: ThreadSanitizer: data race (pid=1852202)” |
| R2T1.5 | [crossbeam-rs/crossbeam#1108](https://github.com/crossbeam-rs/crossbeam/issues/1108) | 2024-12-15 completed | not eligible (E4) | E1 ✓: the maintainer names rust#128778 (merged 2024-09-28), a Miri change in another repository (E1 has no same-repository restriction). E2 ✓. E3 ✓ (a data race between threads, as Miri reports it). E4 fails: the observed wrong behavior is a Miri undefined-behavior report, memory unsafety under the E4 narrowing. Symptom (issue body, [2024-05-08](https://github.com/crossbeam-rs/crossbeam/issues/1108)): “error: Undefined Behavior: Data race detected between (1) atomic load on thread `unnamed-4` and (2) retag read of type `std::sync::atomic::AtomicUsize`” |
| R2T1.6 | [crossbeam-rs/crossbeam#1136](https://github.com/crossbeam-rs/crossbeam/issues/1136) | 2024-09-19 not_planned | not a candidate (scope) | An event announcement, not about a crate. |
| R2T1.7 | [crossbeam-rs/crossbeam#315](https://github.com/crossbeam-rs/crossbeam/issues/315) | 2024-12-09 completed | not eligible (E2) | A question about the soundness of `compare_exchange` over padding bytes, from reading; no observed failure. |
| R2T1.8 | [crossbeam-rs/crossbeam#1117](https://github.com/crossbeam-rs/crossbeam/issues/1117) | 2024-06-07 completed | not eligible (E1) | Feature request; no fix (the requested API already exists). |
| R2T1.9 | [crossbeam-rs/crossbeam#1027](https://github.com/crossbeam-rs/crossbeam/issues/1027) | 2023-09-23 completed | not eligible (E1) | Usage question (`into_owned`); no fix. |
| R2T1.10 | [crossbeam-rs/crossbeam#993](https://github.com/crossbeam-rs/crossbeam/issues/993) | 2023-06-17 completed | not eligible (E4) | E1 ✓ (crossbeam#995). E2 ✓. E3 ✓. E4 fails: the observed wrong behavior is a Miri data-race report, memory unsafety, which the E4 narrowing excludes; the repro also uses epoch guards, pointer dereferences and deferred destruction. (Its fix changes only the ordering of `load_consume`, which would also fail E5(a); E5 is not assessed.) Symptom (issue body, [2023-06-17](https://github.com/crossbeam-rs/crossbeam/issues/993)): “error: Undefined Behavior: Data race detected between (1) Write on thread `thread-1` and (2) Read on thread `thread-2` at alloc106668. (2) just happened here” |
| R2T2.1 | [crossbeam-rs/crossbeam#1102](https://github.com/crossbeam-rs/crossbeam/issues/1102) | 2026-06-07 completed | not eligible (E4) | E1 ✓ (crossbeam#1121, "fix #1102", merged 2026-06-07), E2 ✓ (a reproduction that hangs), E3 ✓ (the caller waits forever in `recv` on a zero-capacity channel whose sender sits in a buffer the dropped receiver never discards). E4 fails: at the buggy revision `4efd6ad`, `SyncWaker::disconnect` holds its `Mutex<Waker>` (`crossbeam-channel/src/waker.rs:265-266`) while `Waker::disconnect` calls `entry.cx.unpark()` (`waker.rs:157-165`), a lock guard held across `unpark`. The path also spans the array and zero flavors, the counter, `Context` and the waker; its length was not counted, and the outcome rests on the guard narrowing. That structure is incidental to the bug, which is that the array flavor's `disconnect_receivers` did not discard buffered messages (#1121 ports rust#108164 into `flavors/array.rs`); the protocol excludes on the path's structure, not the bug's mechanism. Symptom (issue body, [2024-04-16](https://github.com/crossbeam-rs/crossbeam/issues/1102)): “I have encountered a situation where the code gets stuck indefinitely and the content in the channel's buffer is not dropped when the receiver is dropped. ” |
| R2T2.2 | [crossbeam-rs/crossbeam#1203](https://github.com/crossbeam-rs/crossbeam/issues/1203) | 2025-10-08 completed | not eligible (E3) | E1 ✓ (crossbeam#1015, named). The Miri errors come from integer-pointer casts (provenance), and the deadlocks from API misuse, per the maintainer: not interleaving causes. Symptom (issue body, [2025-10-05](https://github.com/crossbeam-rs/crossbeam/issues/1203)): “error: Undefined Behavior: constructing invalid value: encountered a dangling reference (0x214d0[noalloc] has no provenance)” |
| R2T2.3 | [crossbeam-rs/crossbeam#196](https://github.com/crossbeam-rs/crossbeam/issues/196) | 2023-12-02 not_planned | not eligible (E1) | Feature request, closed `not_planned`. |
| R2T2.4 | [crossbeam-rs/crossbeam#534](https://github.com/crossbeam-rs/crossbeam/issues/534) | 2020-06-27 completed | not eligible (E1) | A design question (use two channels); no fix. |
| R2T2.5 | [crossbeam-rs/crossbeam#424](https://github.com/crossbeam-rs/crossbeam/issues/424) | 2020-01-24 completed | not eligible (E1) | The cause was std's thread-id counter being duplicated across DLLs, deemed unsupported; no fix. |
| R2T2.6 | [crossbeam-rs/crossbeam#264](https://github.com/crossbeam-rs/crossbeam/issues/264) | 2019-02-21 completed | not eligible (E1) | Feature request (Future/Stream). |
| R2T2.7 | [crossbeam-rs/crossbeam#388](https://github.com/crossbeam-rs/crossbeam/issues/388) | 2019-06-13 completed | not eligible (E3) | E1 ✓ (crossbeam#391). `AtomicCell` compare-exchange fails on garbage padding bytes: not an interleaving. Symptom (issue body, [2019-06-05](https://github.com/crossbeam-rs/crossbeam/issues/388)): “But I see no further output after `[src/main.rs:22] data_i = 150` when running on Windows. It works fine on Linux.” |
| R2T2.8 | [crossbeam-rs/crossbeam#314](https://github.com/crossbeam-rs/crossbeam/issues/314) | 2019-04-22 completed | not eligible (E1) | API design discussion. |
| R2T2.9 | [crossbeam-rs/crossbeam#180](https://github.com/crossbeam-rs/crossbeam/issues/180) | 2018-07-05 completed | not eligible (E1) | User error (indicatif's join), per the thread; no fix. |
| R2T5.9 | [crossbeam-rs/crossbeam#1020](https://github.com/crossbeam-rs/crossbeam/issues/1020) | 2023-09-12 completed | not eligible (E2) | E1 ✓ (crossbeam#1026). E2 fails: a false-sharing performance report (cache-line contention), with no error, wrong value, hang or crash. |
| R2T5.10 | [crossbeam-rs/crossbeam#838](https://github.com/crossbeam-rs/crossbeam/issues/838) | 2022-06-04 completed | not eligible (E1) | No crossbeam fix linked (the only merged cross-reference is rust#93563, std's import of crossbeam-channel); the thread treats the missing happens-before on disconnect as a question of specification. It is also weak-memory only (a `Relaxed` load), which would fail E5(a). |
| R2T6.2 | [crossbeam-rs/crossbeam#609](https://github.com/crossbeam-rs/crossbeam/issues/609) | 2021-11-23 completed | not eligible (E1) | No fix linked; an ARM hang in epoch's registration loop, tied to rust#53670. |
| R2T6.3 | [crossbeam-rs/crossbeam#482](https://github.com/crossbeam-rs/crossbeam/issues/482) | 2020-11-29 completed | not eligible (E2) | A question about `Parker`'s spurious wakeups; no observed failure (crossbeam#563 later reworked `Parker`). |
| R2T6.4 | [crossbeam-rs/crossbeam#451](https://github.com/crossbeam-rs/crossbeam/issues/451) | 2020-06-12 completed | not eligible (E2) | From reading ("the following sequence may happen"), with no observed failure; the maintainer notes double notifications cost only performance. Fixed by crossbeam#478. |
| R2T6.5 | [crossbeam-rs/crossbeam#97](https://github.com/crossbeam-rs/crossbeam/issues/97) | 2017-01-01 completed | not a candidate (scope) | The pre-split umbrella crate's `src/sync/ms_queue.rs`, not one of the five crates; its fix crossbeam#98 changes orderings, so it would fail E5(a) in any case. |
| R2T6.6 | [crossbeam-rs/crossbeam#133](https://github.com/crossbeam-rs/crossbeam/issues/133) | 2017-04-07 completed | not a candidate (scope) | The umbrella crate's `src/sync/seg_queue.rs`; an ordering report. |
| R2T6.7 | [crossbeam-rs/crossbeam#76](https://github.com/crossbeam-rs/crossbeam/issues/76) | 2017-04-07 completed | not a candidate (scope) | The umbrella crate's `chase_lev`; an API request. |
| R2T7.2 | [crossbeam-rs/crossbeam#778](https://github.com/crossbeam-rs/crossbeam/issues/778) | 2022-02-17 completed | not eligible (E1) | Feature request. |
| R2T7.4 | [crossbeam-rs/crossbeam#397](https://github.com/crossbeam-rs/crossbeam/issues/397) | 2019-07-03 completed | not eligible (E1) | The thread says "fixed in master" and links only issue #388; it names no commit or PR as the fix (the fix, crossbeam#391, is linked only from #388). A mention is not a link to a fix. If "this is this issue: #388" counted as naming #391, E3 would fail exactly as for #388 (padding bytes, not an interleaving). |
| R2T7.6 | [crossbeam-rs/crossbeam#124](https://github.com/crossbeam-rs/crossbeam/issues/124) | 2017-07-05 completed | not a candidate (scope) | Contribution guidelines. |
| R3T1.1 | [Amanieu/parking_lot#535](https://github.com/Amanieu/parking_lot/issues/535) | 2026-10-02 completed | not eligible (E4) | E1 ✓ (parking_lot#533 lists it), E2 ✓ (a PoC), E3 ✓. `ReentrantMutexGuard::bump` unlocks and relocks a raw `lock_api` lock whose hold spans user code and a panic; a raw lock held across arbitrary code does not map to one `Shared::with`, and the lock's slow path runs through `parking_lot_core` (futex `ThreadParker`, a bucket lock held while callbacks run). Symptom (issue body, [2026-09-14](https://github.com/Amanieu/parking_lot/issues/535)): “Thread B: acquires the reentrant lock successfully even though A still holds a guard.” |
| R3T1.2 | [Amanieu/parking_lot#212](https://github.com/Amanieu/parking_lot/issues/212) | 2026-10-02 completed | not eligible (E4) | E1 ✓ (parking_lot#533 lists it), E2 ✓ (the program deadlocks), E3 ✓. The path is `src/raw_rwlock.rs` and `parking_lot_core::park`, which holds a bucket `WordLock` while the caller's `validate` and `before_sleep` callbacks run and then waits on a futex: a guard held across effects, and an OS primitive. (Its length was not counted; the outcome rests on those two narrowings.) Symptom (issue body, [2020-01-28](https://github.com/Amanieu/parking_lot/issues/212)): “This program deadlocks:” |
| R3T1.3 | [Amanieu/parking_lot#463](https://github.com/Amanieu/parking_lot/issues/463) | 2025-05-16 not_planned | not eligible (E1) | Closed `not_planned`; the reporter withdrew it (the release-ordered operation exists). |
| R3T1.4 | [Amanieu/parking_lot#465](https://github.com/Amanieu/parking_lot/issues/465) | 2025-05-27 completed | not eligible (E4) | E1 ✓ (parking_lot#466), E2 ✓ (a production lockup), E3 ✓ (a timed-out writer racing an unpark). Same path as #212, through `park` with a timeout: excluded as there. Symptom (issue body, [2025-05-23](https://github.com/Amanieu/parking_lot/issues/465)): “We have been investigating a weird lockup in our production system that uses `RwLock` from `parking_lot` this week.” |
| R3T1.5 | [Amanieu/parking_lot#346](https://github.com/Amanieu/parking_lot/issues/346) | 2022-08-27 completed | not eligible (E4) | E1 ✓ (parking_lot#349). E2 ✓. E3 ✓ (a race between threads sharing an `ArcMutexGuard`). E4 fails: the observed wrong behavior is a Miri race report, memory unsafety under the E4 narrowing. The cause is a `Sync` impl on the guard that safe Tide code could not write. Symptom (issue body, [2022-08-18](https://github.com/Amanieu/parking_lot/issues/346)): “This code causes a race condition: (tested with miri)” |
| R3T1.6 | [Amanieu/parking_lot#306](https://github.com/Amanieu/parking_lot/issues/306) | 2021-12-19 completed | not eligible (E1) | The only commit mentioned, c866ba80, predates the issue and fixed an earlier upgrade problem; the comment that mentions it says the current report could not be reproduced, and the reporter then withdrew it as a possible ThreadSanitizer false positive. A mention is not a fix of this issue, and the closing event has no commit. |
| R3T1.7 | [Amanieu/parking_lot#323](https://github.com/Amanieu/parking_lot/issues/323) | 2022-05-31 completed | not eligible (E4) | E1 ✓ (parking_lot#344), E2 ✓ (TSan), E3 ✓. `raw_rwlock` path as #212; the fix changes only orderings (E5(a)). Symptom (issue body, [2022-02-25](https://github.com/Amanieu/parking_lot/issues/323)): “running the following results in failures about half the time with parking_lot, while never failing with the std RwLock.” |
| R3T1.8 | [Amanieu/parking_lot#294](https://github.com/Amanieu/parking_lot/issues/294) | 2021-08-25 completed | not eligible (E4) | E1 ✓ (commit c866ba80, "Fixes #294"), E2 ✓ (Miri), E3 ✓. `raw_rwlock` path as #212; ordering-only fix (E5(a)). Symptom (issue body, [2021-08-24](https://github.com/Amanieu/parking_lot/issues/294)): “finds a bug in the test: the line `*lock = -1` is reported as a data race” |
| R3T1.9 | [Amanieu/parking_lot#258](https://github.com/Amanieu/parking_lot/issues/258) | 2020-11-17 completed | not eligible (E4) | E1 ✓ (parking_lot#262). E2 ✓. E3 ✓ (a data race through a `Cell` reached from two threads). E4 fails: the observed wrong behavior is a segfault, memory unsafety under the E4 narrowing. The cause is a `Send` bound on mapped guards that safe Tide code could not write. Symptom (issue body, [2020-11-08](https://github.com/Amanieu/parking_lot/issues/258)): “Return Code: -11 (SIGSEGV)” |
| R3T1.10 | [Amanieu/parking_lot#257](https://github.com/Amanieu/parking_lot/issues/257) | 2020-11-17 completed | not eligible (E4) | E1 ✓ (parking_lot#260), E2 ✓ (TSan), E3 ✓. `raw_rwlock` path as #212; the fix adds an `Acquire` (E5(a)). Symptom (issue body, [2020-11-02](https://github.com/Amanieu/parking_lot/issues/257)): “WARNING: ThreadSanitizer: data race (pid=4254)” |
| R3T2.1 | [Amanieu/parking_lot#489](https://github.com/Amanieu/parking_lot/issues/489) | 2026-10-02 completed | not eligible (E4) | E1 ✓ (parking_lot#533), E2 ✓ (observed deadlocks), E3 ✓. `raw_rwlock` upgradable path as #212. Symptom (issue body, [2025-09-03](https://github.com/Amanieu/parking_lot/issues/489)): “I am observing deadlocks when mixing `upgradable_read` locks with `read` locks on different thread.” |
| R3T2.3 | [Amanieu/parking_lot#505](https://github.com/Amanieu/parking_lot/issues/505) | 2026-10-02 completed | not eligible (E4) | E1 ✓ (parking_lot#533), E2 ✓ (a hang in a game-engine workload), E3 ✓ (a missed unpark). `raw_rwlock` path as #212. Symptom (issue body, [2025-11-28](https://github.com/Amanieu/parking_lot/issues/505)): “When testing this code under a high-concurrency scenario (6+ threads), from standard synchronous threads, I see a deadlock after roughly 1-2 minutes.” |
| R3T2.4 | [Amanieu/parking_lot#438](https://github.com/Amanieu/parking_lot/issues/438) | 2026-08-18 completed | not eligible (E1) | Feature request. |
| R3T2.5 | [Amanieu/parking_lot#515](https://github.com/Amanieu/parking_lot/issues/515) | 2026-02-03 completed | not eligible (E1) | `fork` in a multithreaded process; the reporter withdrew it; no fix. |
| R3T2.6 | [Amanieu/parking_lot#498](https://github.com/Amanieu/parking_lot/issues/498) | 2025-10-30 completed | not eligible (E1) | The wasm parker works as designed, per the maintainer; no fix. |
| R3T2.8 | [Amanieu/parking_lot#453](https://github.com/Amanieu/parking_lot/issues/453) | 2024-12-27 completed | not eligible (E1) | Usage question. |
| R3T2.9 | [Amanieu/parking_lot#165](https://github.com/Amanieu/parking_lot/issues/165) | 2019-08-08 completed | not eligible (E1) | Feature question (a Condvar for `RwLock`). |
| R3T2.10 | [Amanieu/parking_lot#447](https://github.com/Amanieu/parking_lot/issues/447) | 2024-07-21 completed | not eligible (E1) | Feature request. |
| R3T6.2 | [Amanieu/parking_lot#318](https://github.com/Amanieu/parking_lot/issues/318) | 2022-04-23 completed | not eligible (E1) | User code notified the condvar without holding the lock, per the maintainer; no fix. |
| R3T6.3 | [Amanieu/parking_lot#152](https://github.com/Amanieu/parking_lot/issues/152) | 2019-08-08 completed | not eligible (E1) | The commit that closes it (a1a35fd, "Fix Condvar example") changes only documentation. |
| R3T7.2 | [Amanieu/parking_lot#361](https://github.com/Amanieu/parking_lot/issues/361) | 2022-10-28 completed | not eligible (E1) | No fix linked (a hang in `futex_wait`). |
| R3T7.3 | [Amanieu/parking_lot#130](https://github.com/Amanieu/parking_lot/issues/130) | 2019-04-12 completed | not eligible (E1) | No fix linked (armhf on armv8). |
| R4T1.1 | [xacrimon/dashmap#78](https://github.com/xacrimon/dashmap/issues/78) | 2020-04-25 completed | not eligible (E1) | Feature request. |
| R4T1.2 | [xacrimon/dashmap#213](https://github.com/xacrimon/dashmap/issues/213) | 2022-05-28 completed | not eligible (E4) | E1 ✓ (dashmap#214, "Fixes #213"), E2 ✓ (Miri), E3 ✓. dashmap's lock implements `lock_api::RawRwLock`, an external crate on the path. Symptom (issue body, [2022-05-27](https://github.com/xacrimon/dashmap/issues/213)): “error: Undefined Behavior: Data race detected between Read on Thread(id = 2) and Write on Thread(id = 1) at alloc60701+0x738” |
| R4T1.3 | [xacrimon/dashmap#201](https://github.com/xacrimon/dashmap/issues/201) | 2022-04-29 completed | not eligible (E4) | E1 ✓ (dashmap#202). E2 ✓ (the wrong value is stated in the reproducer's comment rather than pasted as output; marked doubtful). E3 ✓ (concurrent increments through a `Ref` that should not be `Send`). E4 fails: at the buggy revision `82c2323` the shards are `parking_lot` locks over `hashbrown` tables, both external crates on the path; and the race is on plain memory through an unsound `Send`, which safe Tide code cannot express. Symptom (issue body, [2022-04-29](https://github.com/xacrimon/dashmap/issues/201)): “NOTE how the two cells are out of sync and not equal to `1_000_000`, which shows the data race in action” |
| R4T1.4 | [xacrimon/dashmap#196](https://github.com/xacrimon/dashmap/issues/196) | 2022-04-27 completed | not eligible (E1) | Usage question. |
| R4T1.5 | [xacrimon/dashmap#139](https://github.com/xacrimon/dashmap/issues/139) | 2022-02-21 completed | not eligible (E1) | No fix linked (the reporter was asked to try v4). |
| R4T1.6 | [xacrimon/dashmap#143](https://github.com/xacrimon/dashmap/issues/143) | 2021-05-16 completed | not eligible (E1) | Feature request. |
| R4T1.7 | [xacrimon/dashmap#121](https://github.com/xacrimon/dashmap/issues/121) | 2020-12-25 completed | not eligible (E1) | No fix linked. |
| R4T1.8 | [xacrimon/dashmap#90](https://github.com/xacrimon/dashmap/issues/90) | 2020-07-16 completed | not eligible (E1) | Usage question. |
| R4T1.9 | [xacrimon/dashmap#22](https://github.com/xacrimon/dashmap/issues/22) | 2020-01-16 completed | not eligible (E1) | Feature request. |
| R4T2.1 | [xacrimon/dashmap#363](https://github.com/xacrimon/dashmap/issues/363) | 2026-10-05 completed | not eligible (E1) | Documentation wording. |
| R4T2.2 | [xacrimon/dashmap#369](https://github.com/xacrimon/dashmap/issues/369) | 2026-05-17 completed | not eligible (E1) | Documented behavior (`insert` while holding a `Ref`); no fix. |
| R4T2.3 | [xacrimon/dashmap#356](https://github.com/xacrimon/dashmap/issues/356) | 2026-05-17 completed | not eligible (E1) | Feature request. |
| R4T2.4 | [xacrimon/dashmap#319](https://github.com/xacrimon/dashmap/issues/319) | 2025-02-01 completed | not eligible (E1) | Not a dashmap bug, per the maintainer; no fix. |
| R4T2.5 | [xacrimon/dashmap#284](https://github.com/xacrimon/dashmap/issues/284) | 2025-02-01 completed | not eligible (E1) | Usage question. |
| R4T2.6 | [xacrimon/dashmap#316](https://github.com/xacrimon/dashmap/issues/316) | 2025-02-01 completed | not eligible (E1) | User code with tokio's oneshot; no fix. |
| R4T2.7 | [xacrimon/dashmap#79](https://github.com/xacrimon/dashmap/issues/79) | 2020-05-24 completed | not eligible (E1) | A `Ref` held across an `.await`; no fix linked. |
| R4T2.8 | [xacrimon/dashmap#304](https://github.com/xacrimon/dashmap/issues/304) | 2024-06-24 completed | not eligible (E3) | E1 ✓ (dashmap#305), E2 ✓. One thread holds a shard's write lock and calls `self.len()`, which read-locks every shard: a self-deadlock with no second concurrent activity. Symptom (issue body, [2024-06-19](https://github.com/xacrimon/dashmap/issues/304)): “When I update to the lates `6.0.0` version, the `Dashmap::shrink_to_fit()` method deadlocks.” |
| R4T2.10 | [xacrimon/dashmap#239](https://github.com/xacrimon/dashmap/issues/239) | 2022-11-29 completed | not eligible (E1) | Feature request. |
| R4T4.1 | [xacrimon/dashmap#47](https://github.com/xacrimon/dashmap/issues/47) | 2020-02-03 completed | not eligible (E1) | No fix linked; the maintainer explains that the shard locks guard it. |
| R4T6.2 | [xacrimon/dashmap#6](https://github.com/xacrimon/dashmap/issues/6) | 2020-01-15 completed | not eligible (E1) | Feature request. |
| R4T8.1 | [xacrimon/dashmap#351](https://github.com/xacrimon/dashmap/issues/351) | 2026-05-17 completed | not eligible (E1) | No fix linked (an assertion in `lock.rs`). |
| R5T1.1 | [zesterer/flume#120](https://github.com/zesterer/flume/issues/120) | 2023-04-06 completed | not eligible (E4) | E1 taken as ✓ (flume#121, "Fix a race condition in the async code"; the outcome does not depend on it), E2 ✓, E3 ✓. The path is `src/async.rs`: futures and wakers, excluded by E4. Symptom (issue body, [2023-03-08](https://github.com/zesterer/flume/issues/120)): “When using the `async` code and the `Sender` sends and gets dropped at the wrong time, the `Receiver` won't receive the sent value.” |
| R5T1.2 | [zesterer/flume#78](https://github.com/zesterer/flume/issues/78) | 2021-04-12 completed | not eligible (E4) | E1 ✓ (commit 45edd4, named), E2 ✓, E3 ✓. The path is `recv_async`: async, excluded by E4. Symptom (issue body, [2021-04-11](https://github.com/zesterer/flume/issues/78)): “On a channel, I can verify I've received an `Ok(())` from a sender, and receive a disconnect error from flume on the corresponding receiver.” |
| R5T1.3 | [zesterer/flume#44](https://github.com/zesterer/flume/issues/44) | 2021-02-08 completed | not eligible (E3) | E1 ✓ (flume#69, whose text says "This may also be the cause of #44"; flume#70, "Closes #44", only re-enables the tests), E2 ✓ (hangs in CI). The cause the fix names is `Arc::ptr_eq` on trait-object pointers comparing vtables unpredictably (rust#46139), a codegen property, not an interleaving. Symptom (comment, [2020-09-15](https://github.com/zesterer/flume/issues/44#issuecomment-692840769)): “This hangs on Linux too according to CI” |
| R5T2.1 | [zesterer/flume#2](https://github.com/zesterer/flume/issues/2) | 2020-09-02 completed | not eligible (E2) | E1 ✓ ("addressed by #24", a merged PR). E2 fails: a feature request (futures support), with no observed wrong behavior. |
| R5T6.1 | [zesterer/flume#38](https://github.com/zesterer/flume/issues/38) | 2020-09-03 completed | not eligible (E1) | CI duration. |
| R5T7.1 | [zesterer/flume#176](https://github.com/zesterer/flume/issues/176) | 2026-04-16 completed | not eligible (E3) | E1 ✓ (flume#179), E2 ✓. `wait_timeout` on an empty `Selector` hangs in one thread; no concurrent activity. Symptom (issue body, [2026-02-27](https://github.com/zesterer/flume/issues/176)): “This test passes on flume 0.11.1, but hangs forever on 0.12.0.” |
| R5T7.2 | [zesterer/flume#74](https://github.com/zesterer/flume/issues/74) | 2021-04-12 completed | not eligible (E2) | E1 ✓: the owner names commit 3f8a925 ("Capped exponential backoff", 2021-03-09) as the fix. E2 fails: the report comes from reading the spinlock code (the reporter does not use flume), with no observed failure. |
| R6T1.1 | [matklad/once_cell#299](https://github.com/matklad/once_cell/issues/299) | 2026-11-02 completed | not eligible (E1) | By design, per the maintainer. |
| R6T1.2 | [matklad/once_cell#291](https://github.com/matklad/once_cell/issues/291) | 2025-10-09 completed | not eligible (E1) | Feature idea. |
| R6T1.3 | [matklad/once_cell#271](https://github.com/matklad/once_cell/issues/271) | 2025-03-12 completed | not eligible (E1) | Feature request. |
| R6T1.4 | [matklad/once_cell#220](https://github.com/matklad/once_cell/issues/220) | 2025-03-27 completed | not eligible (E2) | A proposal to weaken orderings; no wrong behavior. |
| R6T1.5 | [matklad/once_cell#279](https://github.com/matklad/once_cell/issues/279) | 2025-03-28 completed | not eligible (E1) | Proposal. |
| R6T1.6 | [matklad/once_cell#263](https://github.com/matklad/once_cell/issues/263) | 2025-03-28 completed | not eligible (E1) | Refactoring. |
| R6T1.7 | [matklad/once_cell#266](https://github.com/matklad/once_cell/issues/266) | 2024-09-30 not_planned | not eligible (E1) | An event announcement, not a bug. |
| R6T1.8 | [matklad/once_cell#264](https://github.com/matklad/once_cell/issues/264) | 2024-09-29 completed | not eligible (E3) | E1 ✓ (once_cell#265). A build-feature regression with portable-atomic: not concurrency. Symptom (issue body, [2024-09-16](https://github.com/matklad/once_cell/issues/264)): “This breaks our use case.” |
| R6T1.9 | [matklad/once_cell#223](https://github.com/matklad/once_cell/issues/223) | 2023-02-22 completed | not eligible (E1) | Feature request. |
| R6T1.10 | [matklad/once_cell#1](https://github.com/matklad/once_cell/issues/1) | 2022-10-22 completed | not eligible (E2) | E1 ✓: the thread names commit 0af68fb ("Use stronger memory ordering in sync::OnceCell", 2018-08-02) as the fix. E2 fails: only the body, an audit request, predates that commit, and it reports no observed failure. |
| R6T2.1 | [matklad/once_cell#65](https://github.com/matklad/once_cell/issues/65) | 2024-10-31 completed | not eligible (E1) | A design question; the linked PR once_cell#71 is not merged. |
| R6T2.2 | [matklad/once_cell#93](https://github.com/matklad/once_cell/issues/93) | 2020-02-13 completed | not eligible (E1) | UB in user code (dropping a replaced `Mutex` with its guard); no fix. |
| R6T6.1 | [matklad/once_cell#186](https://github.com/matklad/once_cell/issues/186) | 2022-07-04 completed | not eligible (E3) | E1 ✓ (once_cell#187), E2 ✓ (Miri test failures). The cause is a spurious failure of `compare_exchange_weak` tripping a debug assertion, which happens in one thread; not an interleaving (and E5(c) names spurious failures). Symptom (issue body, [2022-07-02](https://github.com/matklad/once_cell/issues/186)): “By default, Miri makes most calls fail so this panics when running tests” |
| R7T1.1 | [rayon-rs/rayon#1275](https://github.com/rayon-rs/rayon/issues/1275) | 2026-07-06 completed | not eligible (E4) | E1 ✓ (fixed by crossbeam#998, as the thread says), E2 ✓ (TSan), E3 ✓. The race is in crossbeam-epoch, an external crate to rayon-core. Symptom (issue body, [2025-10-30](https://github.com/rayon-rs/rayon/issues/1275)): “On my machine, it reliably triggers TSAN warnings about data races, all related to `crossbeam_epoch`” |
| R7T1.2 | [rayon-rs/rayon#1267](https://github.com/rayon-rs/rayon/issues/1267) | 2025-08-17 duplicate | not eligible (E1) | Closed as a duplicate; no fix linked. |
| R7T1.3 | [rayon-rs/rayon#1211](https://github.com/rayon-rs/rayon/issues/1211) | 2024-11-18 not_planned | not a candidate (scope) | ThreadSanitizer report in the test harness's backtrace symbolization, not rayon-core, per the maintainer. |
| R7T1.4 | [rayon-rs/rayon#1210](https://github.com/rayon-rs/rayon/issues/1210) | 2024-11-18 completed | not a candidate (scope) | As #1211. |
| R7T1.5 | [rayon-rs/rayon#1198](https://github.com/rayon-rs/rayon/issues/1198) | 2024-10-08 completed | not a candidate (scope) | An event announcement. |
| R7T1.6 | [rayon-rs/rayon#1017](https://github.com/rayon-rs/rayon/issues/1017) | 2023-03-12 completed | not a candidate (scope) | Parallel iterators over indices. |
| R7T1.7 | [rayon-rs/rayon#938](https://github.com/rayon-rs/rayon/issues/938) | 2022-05-31 completed | not eligible (E4) | In scope (rayon-core's `mutual_install` test). E1 ✓: the maintainer names rayon#740 (merged 2020-06-15) as the fix the rustc-rayon fork lacked; rust#97472 landed the rebased fork. #740 predates the issue, so this is marked doubtful. E2 ✓. E3 ✓. E4 fails: the observed wrong behavior is a Miri data-race report (deallocate against read), memory unsafety under the E4 narrowing. Symptom (issue body, [2022-05-27](https://github.com/rayon-rs/rayon/issues/938)): “error: Undefined Behavior: Data race detected between Deallocate on Thread(id = 2) and Read on Thread(id = 1) at alloc167539” |
| R7T1.8 | [rayon-rs/rayon#947](https://github.com/rayon-rs/rayon/issues/947) | 2022-06-22 completed | not a candidate (scope) | `chain` (parallel iterators). |
| R7T1.9 | [rayon-rs/rayon#827](https://github.com/rayon-rs/rayon/issues/827) | 2022-05-12 completed | not eligible (E1) | rayon#800 is named as the fix of #797, a different overflow, and the crash persisted on rayon-core 1.8.1; the reporter credits a crossbeam-deque advisory, which is not a PR or commit. No fix of this issue is linked. |
| R7T1.10 | [rayon-rs/rayon#738](https://github.com/rayon-rs/rayon/issues/738) | 2022-05-13 completed | not eligible (E1) | The maintainer says #740 fixes "the other crash"; the issue closed with the release of #934 and "please reopen" if it recurs, which is a hope, not a naming, and #934's own text fixes #913 and #929. No fix of this issue is named (doubtful; if the close counted, E1 and E2 would pass and the segfault would fail E4 as memory unsafety). |
| R7T2.1 | [rayon-rs/rayon#1258](https://github.com/rayon-rs/rayon/issues/1258) | 2025-08-12 completed | not eligible (E1) | Usage question. |
| R7T2.2 | [rayon-rs/rayon#1251](https://github.com/rayon-rs/rayon/issues/1251) | 2025-06-12 completed | not eligible (E1) | Work-stealing reentrancy (rayon#592), explained; no fix. |
| R7T2.3 | [rayon-rs/rayon#1205](https://github.com/rayon-rs/rayon/issues/1205) | 2025-02-25 completed | not eligible (E1) | As #1251; no fix. |
| R7T2.4 | [rayon-rs/rayon#1181](https://github.com/rayon-rs/rayon/issues/1181) | 2024-07-12 completed | not eligible (E1) | Closed as a duplicate feature request. |
| R7T2.5 | [rayon-rs/rayon#861](https://github.com/rayon-rs/rayon/issues/861) | 2023-02-22 completed | not eligible (E2) | Feature request (a fallback without threads), implemented by rayon#1019; no wrong behavior. |
| R7T2.6 | [rayon-rs/rayon#1100](https://github.com/rayon-rs/rayon/issues/1100) | 2023-11-01 completed | not eligible (E1) | Usage question. |
| R7T2.7 | [rayon-rs/rayon#690](https://github.com/rayon-rs/rayon/issues/690) | 2023-06-25 completed | not a candidate (scope) | `par_bridge`; the fix rayon#996 changes `src/iter/par_bridge.rs`, not rayon-core. |
| R7T2.8 | [rayon-rs/rayon#1083](https://github.com/rayon-rs/rayon/issues/1083) | 2023-09-21 not_planned | not eligible (E1) | Closed `not_planned`; no fix. |
| R7T2.9 | [rayon-rs/rayon#1068](https://github.com/rayon-rs/rayon/issues/1068) | 2023-07-18 completed | not a candidate (scope) | `par_bridge` memory use. |
| R7T2.10 | [rayon-rs/rayon#1052](https://github.com/rayon-rs/rayon/issues/1052) | 2023-09-20 completed | not eligible (E2) | Feature request (use the current thread in a new pool), implemented by commit 9461f7b; no wrong behavior. |
| R7T5.7 | [rayon-rs/rayon#879](https://github.com/rayon-rs/rayon/issues/879) | 2021-08-23 completed | not a candidate (scope) | A crossbeam-deque advisory (a dependency upgrade), not rayon-core code. |
| R7T5.8 | [rayon-rs/rayon#919](https://github.com/rayon-rs/rayon/issues/919) | 2022-04-27 completed | not eligible (E1) | No fix linked (a segfault and an index panic in `sleep`). |
| R7T5.9 | [rayon-rs/rayon#768](https://github.com/rayon-rs/rayon/issues/768) | 2020-06-15 completed | not eligible (E4) | E1 ✓ (rayon#740), E2 ✓ (SIGSEGV), E3 ✓ (cross-pool dispatch). The symptom is memory unsafety: `TickleLatch::set` reads its registry after the latch's owner may have freed it. Symptom (issue body, [2020-06-15](https://github.com/rayon-rs/rayon/issues/768)): “We're getting a `SIGSEGV` in rayon under load with cross-`ThreadPool` dispatch.” |
| R7T6.1 | [rayon-rs/rayon#267](https://github.com/rayon-rs/rayon/issues/267) | 2017-03-19 completed | not a candidate (scope) | A build dependency on Windows. |
| R7T7.2 | [rayon-rs/rayon#967](https://github.com/rayon-rs/rayon/issues/967) | 2022-12-15 not_planned | not eligible (E1) | Closed `not_planned`; no fix. |
| R7T7.3 | [rayon-rs/rayon#396](https://github.com/rayon-rs/rayon/issues/396) | 2023-02-24 not_planned | not a candidate (scope) | `rayon_wait` and `spawn_future` (rayon-futures), not rayon-core. |
| R7T7.4 | [rayon-rs/rayon#820](https://github.com/rayon-rs/rayon/issues/820) | 2021-05-19 completed | not eligible (E1) | No fix linked; forwarded to crossbeam#609. |
| R7T7.5 | [rayon-rs/rayon#776](https://github.com/rayon-rs/rayon/issues/776) | 2020-07-24 completed | not eligible (E1) | No fix (dropping a pool does not wait for its threads, as documented). |
| R7T7.6 | [rayon-rs/rayon#10](https://github.com/rayon-rs/rayon/issues/10) | 2016-02-17 completed | not eligible (E4) | E1 ✓ (rayon#37), E2 ✓, E3 ✓. A worker that panics dies without setting the latch its joiner waits on. In Tide v0 a panicking child cancels its scope and the panic is re-raised (reference §7.1), so a thread that dies silently cannot keep its structure. Symptom (issue body, [2015-12-30](https://github.com/rayon-rs/rayon/issues/10)): “Instead, it just hangs waiting for a thread that will never finish.” |
| R7T8.2 | [rayon-rs/rayon#667](https://github.com/rayon-rs/rayon/issues/667) | 2019-07-18 completed | not a candidate (scope) | A parallel-iterator test (`iter_panic_fuse`). |
| R7T8.3 | [rayon-rs/rayon#343](https://github.com/rayon-rs/rayon/issues/343) | 2017-05-30 completed | not eligible (E4) | E1 ✓ (rayon#348, "Fix #343"). E2 ✓ (a comment posted before the fix merged on 2017-05-30 reports CI failures). E3 ✓ (a worker spins while others wait to be woken). E4 fails: rayon-core's `Sleep` at the buggy revision waits on a `Mutex<()>` and a `Condvar` (`rayon-core/src/sleep/mod.rs:6-27`), excluded by E4; the reported spin without a yield is also a livelock (E5(d)). Symptom (comment, [2017-05-24](https://github.com/rayon-rs/rayon/issues/343#issuecomment-303666382)): “more tests timing out or having intermittent failures on CI” |

## What a catch would need, and what this search found

- **The pool's fixed concurrency bugs live inside lock and channel implementations.** The
  26 E4 failures are all real, observed, fixed bugs (rayon#938's E1 reading is marked
  doubtful): crossbeam-channel's lost
  messages on disconnect (crossbeam#1102), parking_lot's `RwLock` deadlocks and the
  timed-out writer (#212, #465, #489, #505), rayon-core's latch and sleep (#343, #768). Their
  paths are lock or channel implementations: atomics and parking, a lock guard held while
  threads are woken, and futex waits. Tide v0 has `Shared` cells, whose
  closures cannot suspend, wake a thread or nest, and channels. It has no raw lock, no
  condition variable and no futex, so a faithful model of these paths would have to rebuild
  those primitives. That is exactly where a model could add or hide the bug, so the
  protocol excludes it.
- **Which limits are Tide's design, which are planned, and which decided rows.**
  `async` and `Waker` are rejected by design (`TD0601`, reference §7). The `Pure`,
  non-nesting `Shared::with` closure is permanent (design/03 LOCK-4). Safe Tide has no
  condition variable by design ("Condition waits are channels", design/03 §6.5). Raw locks
  and weaker orderings are planned in `std::sync::raw` behind the `unsafe` grant (design/03
  CAP-3; reference §7.4 and §11), which this protocol's models do not take, and the design
  does not say the simulator would explore weak memory. Safe Tide cannot produce
  memory-unsafety symptoms, by design. External crates are not in v0. The 400-line cap is a
  budget choice of this protocol, not a Tide limit. In the rows, the cap decided nothing:
  it was never counted, and the rows that mention path length (crossbeam#1102,
  parking_lot#212 and the rows "as #212") also fail the guard or `parking_lot_core`
  narrowings. The Condvar narrowing decided only rayon#343, which is also a livelock
  (E5(d)). The guard-across-effects narrowing is what excludes the nearest miss,
  crossbeam#1102.
- **Many are memory-unsafety or weak-memory reports.** Several E4 rows were observed only as
  a Miri or sanitizer report or a segfault (for example crossbeam#993, rust#125526,
  parking_lot#258 and #346, rayon#938), which the registered E4 narrowing excludes; several
  of them are type-level `Send`/`Sync` mistakes that safe Tide code could not write. Several
  fixes change only memory orderings or fences (crossbeam#993, parking_lot#257, #294, #323),
  which Tide's sequentially consistent simulator would not explore either (E5(a)), though
  each is counted at its earlier first failure.
- **The nearest miss is crossbeam#1102.** It has an observed hang, a reproduction and a
  merged fix that names it, and its shape (a caller blocked in `recv` on a reply channel
  whose sender sits in a buffer the dropped receiver never empties) is one a `sim_test`
  could express. It fails E4 because the source wakes blocked threads while holding the
  waker's mutex, which `Shared::with` cannot express; the path also spans two channel
  flavors and the waker machinery. Modelling it would mean restructuring the source. That
  structure is incidental to the bug itself, which is that the array flavor's
  `disconnect_receivers` did not discard buffered messages (#1121 ports rust#108164 into
  `flavors/array.rs`): the protocol excludes on the path's structure, not the bug's
  mechanism.
- **Upstream already tests at this level.** crossbeam's CI runs Miri and ThreadSanitizer
  (`ci/miri.sh` and `ci/tsan`, touched by crossbeam#1121 and #998), and std's sync and
  thread tests run under Miri (rust#145816, #133421). Several issues here were found by Miri
  or ThreadSanitizer (parking_lot#257, #294, #323; dashmap#213; rayon#938). Those tools
  explore weak memory and detect undefined behavior, which Tide's simulator does not
  attempt. crossbeam's CI also runs loom (`ci/crossbeam-epoch-loom.sh` on `main`; not
  checked at the buggy revisions), the closest analogue to Tide's simulator. This record
  did not check the other projects' CI.
- **What did not happen.** No property was written and no model was built or run, because no
  candidate was eligible. As with fx-zu4f.6, the Eon case studies
  (`examples/case-studies/eon-discovery-race`, `eon-slow-close`) remain the only
  demonstrations of a seeded catch and a replay, and both are on the owner's own project.

## Results by query

#### R1T1: `repo:rust-lang/rust label:T-libs race is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:34:14.701Z; HTTP 200; `total_count` 23; `incomplete_results` false; 10 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [rust-lang/rust#161018](https://github.com/rust-lang/rust/issues/161018) | `process::exit` in a child thread unmaps the main thread's sigalt stack | not a candidate (scope) |
| 2 | [rust-lang/rust#93740](https://github.com/rust-lang/rust/issues/93740) | Tracking issue for improving std::sync::{Mutex, RwLock, Condvar} | not eligible (E1) |
| 3 | [rust-lang/rust#110317](https://github.com/rust-lang/rust/issues/110317) | std::process:Command::spawn panics when pre_exec hook fails if SIGCHLD is ignored | not a candidate (scope) |
| 4 | [rust-lang/rust#124105](https://github.com/rust-lang/rust/issues/124105) | panic: unexpected error during closedir "Bad file descriptor" for unix::fs::Dir | not a candidate (scope) |
| 5 | [rust-lang/rust#141726](https://github.com/rust-lang/rust/issues/141726) | Unify and deduplicate float tests | not a candidate (scope) |
| 6 | [rust-lang/rust#29497](https://github.com/rust-lang/rust/issues/29497) | fs::remove_dir_all rarely succeeds for large directories on windows | not a candidate (scope) |
| 7 | [rust-lang/rust#136669](https://github.com/rust-lang/rust/issues/136669) | atomics documentation for conflicting access uses undefined term "write" leading to ambiguity about data races | not a candidate (scope) |
| 8 | [rust-lang/rust#124466](https://github.com/rust-lang/rust/issues/124466) | A thread can stop execution without dropping its packet | not a candidate (scope) |
| 9 | [rust-lang/rust#27970](https://github.com/rust-lang/rust/issues/27970) | set_var/remove_var are unsound in combination with C code that reads the environment | not a candidate (scope) |
| 10 | [rust-lang/rust#127576](https://github.com/rust-lang/rust/issues/127576) | `std::fs::remove_dir_all` fails if any of the intermediate file deletions fails with `ENOENT` | not a candidate (scope) |

#### R1T2: `repo:rust-lang/rust label:T-libs deadlock is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:34:18.699Z; HTTP 200; `total_count` 24; `incomplete_results` false; 10 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [rust-lang/rust#165950](https://github.com/rust-lang/rust/issues/165950) | Footgun for calling capacity on a `Vec<()>` | not a candidate (scope) |
| 2 | [rust-lang/rust#156334](https://github.com/rust-lang/rust/issues/156334) | Tracking issue for release notes of #148799: Switch the destructors implementation for thread locals on Windows to use FLS | not a candidate (scope) |
| 3 | [rust-lang/rust#93740](https://github.com/rust-lang/rust/issues/93740) | Tracking issue for improving std::sync::{Mutex, RwLock, Condvar} | not eligible (E1) (assessed at R1T1.2) |
| 4 | [rust-lang/rust#157390](https://github.com/rust-lang/rust/issues/157390) | `File::lock` (probably) behaves incorrectly on Solaris | not a candidate (scope) |
| 5 | [rust-lang/rust#145816](https://github.com/rust-lang/rust/issues/145816) | std `thread::Thread::unpark` test deadlocked when run under miri | not eligible (E1) |
| 6 | [rust-lang/rust#114851](https://github.com/rust-lang/rust/issues/114851) | Channel sending/receiving ends up spinlooping and deadlocking | not eligible (E1) |
| 7 | [rust-lang/rust#139034](https://github.com/rust-lang/rust/issues/139034) | `{Mutex,RwLock}::get_mut()` documentation does not consider if a guard is forgotten | not a candidate (scope) |
| 8 | [rust-lang/rust#96706](https://github.com/rust-lang/rust/issues/96706) | Scoped thread panic messages unexpectedly interleaving in stderr output | not a candidate (scope) |
| 9 | [rust-lang/rust#116179](https://github.com/rust-lang/rust/issues/116179) | Scoped thread implicit join completes before thread-locals are dropped | not eligible (E1) |
| 10 | [rust-lang/rust#123458](https://github.com/rust-lang/rust/issues/123458) | ReentrantLock can be held by multiple threads at the same time | not eligible (E4) |

#### R1T3: `repo:rust-lang/rust label:T-libs "lost wakeup" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:34:22.602Z; HTTP 200; `total_count` 0; `incomplete_results` false; 0 rows.

#### R1T4: `repo:rust-lang/rust label:T-libs "lost update" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:34:26.226Z; HTTP 200; `total_count` 0; `incomplete_results` false; 0 rows.

#### R1T5: `repo:rust-lang/rust label:T-libs "data race" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:34:29.902Z; HTTP 200; `total_count` 5; `incomplete_results` false; 5 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [rust-lang/rust#136669](https://github.com/rust-lang/rust/issues/136669) | atomics documentation for conflicting access uses undefined term "write" leading to ambiguity about data races | not a candidate (scope) (assessed at R1T1.7) |
| 2 | [rust-lang/rust#27970](https://github.com/rust-lang/rust/issues/27970) | set_var/remove_var are unsound in combination with C code that reads the environment | not a candidate (scope) (assessed at R1T1.9) |
| 3 | [rust-lang/rust#125526](https://github.com/rust-lang/rust/issues/125526) | ReentrantLockGuard's Sync impl is unsound | not eligible (E4) |
| 4 | [rust-lang/rust#123458](https://github.com/rust-lang/rust/issues/123458) | ReentrantLock can be held by multiple threads at the same time | not eligible (E4) (assessed at R1T2.10) |
| 5 | [rust-lang/rust#114949](https://github.com/rust-lang/rust/issues/114949) | Possible data race & use-after-free in std::env::var for unix implementation | not a candidate (scope) |

#### R1T6: `repo:rust-lang/rust label:T-libs spurious is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:34:33.680Z; HTTP 200; `total_count` 10; `incomplete_results` false; 10 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [rust-lang/rust#155518](https://github.com/rust-lang/rust/issues/155518) | spurious? failure in test-float-parse | not a candidate (scope) |
| 2 | [rust-lang/rust#148408](https://github.com/rust-lang/rust/issues/148408) | Spurious test failure in metadata_access_times | not a candidate (scope) |
| 3 | [rust-lang/rust#93740](https://github.com/rust-lang/rust/issues/93740) | Tracking issue for improving std::sync::{Mutex, RwLock, Condvar} | not eligible (E1) (assessed at R1T1.2) |
| 4 | [rust-lang/rust#147885](https://github.com/rust-lang/rust/issues/147885) | spurious test failures `condvar::nonpoison_timeout_nanoseconds` | not eligible (E1) |
| 5 | [rust-lang/rust#146024](https://github.com/rust-lang/rust/issues/146024) | `sys::process::windows::tests::test_thread_handle` spuriously failed on aarch64-pc-windows-msvc | not a candidate (scope) |
| 6 | [rust-lang/rust#132416](https://github.com/rust-lang/rust/issues/132416) | WASI unwinding is broken in release | not a candidate (scope) |
| 7 | [rust-lang/rust#133421](https://github.com/rust-lang/rust/issues/133421) | Spurious miri library rwlock test failure? | not eligible (E1) |
| 8 | [rust-lang/rust#89571](https://github.com/rust-lang/rust/issues/89571) | Safety hazard: thread::park() doesn't guarantee being unwind-free but sync::Once implicitly relies on it being unwind-free | not eligible (E2) |
| 9 | [rust-lang/rust#108147](https://github.com/rust-lang/rust/issues/108147) | Condvar missing wake ups. | not eligible (E1) |
| 10 | [rust-lang/rust#99280](https://github.com/rust-lang/rust/issues/99280) | std::fs::create_dir_all can spuriously fail on OpenBSD | not a candidate (scope) |

#### R1T7: `repo:rust-lang/rust label:T-libs hang is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:34:37.480Z; HTTP 200; `total_count` 9; `incomplete_results` false; 9 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [rust-lang/rust#165950](https://github.com/rust-lang/rust/issues/165950) | Footgun for calling capacity on a `Vec<()>` | not a candidate (scope) (assessed at R1T2.1) |
| 2 | [rust-lang/rust#146183](https://github.com/rust-lang/rust/issues/146183) | Checking a `&PathBuf` and a `&str` for equality leads to a stack overflow | not a candidate (scope) |
| 3 | [rust-lang/rust#115209](https://github.com/rust-lang/rust/issues/115209) | Calling `std::thread::current()`  in global allocator results in non-obvious error | not a candidate (scope) |
| 4 | [rust-lang/rust#114851](https://github.com/rust-lang/rust/issues/114851) | Channel sending/receiving ends up spinlooping and deadlocking | not eligible (E1) (assessed at R1T2.6) |
| 5 | [rust-lang/rust#116179](https://github.com/rust-lang/rust/issues/116179) | Scoped thread implicit join completes before thread-locals are dropped | not eligible (E1) (assessed at R1T2.9) |
| 6 | [rust-lang/rust#120147](https://github.com/rust-lang/rust/issues/120147) | Unix PAL POSIX Mutex lock should check its return codes for soundness | not eligible (E3) |
| 7 | [rust-lang/rust#75446](https://github.com/rust-lang/rust/issues/75446) | fs::copy hangs on docker (Linux) | not a candidate (scope) |
| 8 | [rust-lang/rust#79480](https://github.com/rust-lang/rust/issues/79480) | BufReader reads limited by the size of the internal buffer | not a candidate (scope) |
| 9 | [rust-lang/rust#76600](https://github.com/rust-lang/rust/issues/76600) | Deadlocks on NetBSD around fork() | not a candidate (scope) |

#### R1T8: `repo:rust-lang/rust label:T-libs intermittent is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:34:41.384Z; HTTP 200; `total_count` 3; `incomplete_results` false; 3 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [rust-lang/rust#165901](https://github.com/rust-lang/rust/issues/165901) | alloctests::c_str_alloc_error is flaky: global ARMED flag lets libtest's main thread take the injected allocation failure | not a candidate (scope) |
| 2 | [rust-lang/rust#142411](https://github.com/rust-lang/rust/issues/142411) | ReadDir on Windows can return error infinitely when run on removable path | not a candidate (scope) |
| 3 | [rust-lang/rust#99280](https://github.com/rust-lang/rust/issues/99280) | std::fs::create_dir_all can spuriously fail on OpenBSD | not a candidate (scope) (assessed at R1T6.10) |

#### R2T1: `repo:crossbeam-rs/crossbeam race is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:34:45.077Z; HTTP 200; `total_count` 26; `incomplete_results` false; 10 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [crossbeam-rs/crossbeam#663](https://github.com/crossbeam-rs/crossbeam/issues/663) | crossbeam-epoch: ThreadAnalyzer reports data race when using epoch::pin() | not eligible (E4) |
| 2 | [crossbeam-rs/crossbeam#1269](https://github.com/crossbeam-rs/crossbeam/issues/1269) | Soundness: Unsynchronized data races and aliasing violations in AtomicCell fallback path | not eligible (E1) |
| 3 | [crossbeam-rs/crossbeam#646](https://github.com/crossbeam-rs/crossbeam/issues/646) | ThreadSanitizer data race alert in read_volatile | not eligible (E1) |
| 4 | [crossbeam-rs/crossbeam#1207](https://github.com/crossbeam-rs/crossbeam/issues/1207) | `crossbeam_epoch` - TSAN warnings when running multiple user-defined Rayon thread pools concurrently | not eligible (E4) |
| 5 | [crossbeam-rs/crossbeam#1108](https://github.com/crossbeam-rs/crossbeam/issues/1108) | A question regarding Miri error running the “sanitize" example under crossbeam-epoch.  | not eligible (E4) |
| 6 | [crossbeam-rs/crossbeam#1136](https://github.com/crossbeam-rs/crossbeam/issues/1136) | RoSctober 2024: The Open Source Developer Challenge is about to start! | not a candidate (scope) |
| 7 | [crossbeam-rs/crossbeam#315](https://github.com/crossbeam-rs/crossbeam/issues/315) | Soundness of AtomicCell::compare_exchange is dubious | not eligible (E2) |
| 8 | [crossbeam-rs/crossbeam#1117](https://github.com/crossbeam-rs/crossbeam/issues/1117) | Channel timeouts based on deadlines instead of durations | not eligible (E1) |
| 9 | [crossbeam-rs/crossbeam#1027](https://github.com/crossbeam-rs/crossbeam/issues/1027) | epoch `fetch_update` not working as expected | not eligible (E1) |
| 10 | [crossbeam-rs/crossbeam#993](https://github.com/crossbeam-rs/crossbeam/issues/993) | Question (epoch): How to avoid Miri to report data race on an `Atomic` pointer? | not eligible (E4) |

#### R2T2: `repo:crossbeam-rs/crossbeam deadlock is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:34:48.939Z; HTTP 200; `total_count` 9; `incomplete_results` false; 9 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [crossbeam-rs/crossbeam#1102](https://github.com/crossbeam-rs/crossbeam/issues/1102) | Potential deadlock and resource leak when `Receiver` is dropped in `crossbeam_channel`'s bounded channel | not eligible (E4) |
| 2 | [crossbeam-rs/crossbeam#1203](https://github.com/crossbeam-rs/crossbeam/issues/1203) | `crossbeam_utils`: Potential Undefined Behavior Reported by Miri | not eligible (E3) |
| 3 | [crossbeam-rs/crossbeam#196](https://github.com/crossbeam-rs/crossbeam/issues/196) | A method to query the scope if anything panicked | not eligible (E1) |
| 4 | [crossbeam-rs/crossbeam#534](https://github.com/crossbeam-rs/crossbeam/issues/534) | Deadlock when using channel | not eligible (E1) |
| 5 | [crossbeam-rs/crossbeam#424](https://github.com/crossbeam-rs/crossbeam/issues/424) | crossbeam-channel deadlock when used in separate dlls | not eligible (E1) |
| 6 | [crossbeam-rs/crossbeam#264](https://github.com/crossbeam-rs/crossbeam/issues/264) | Future/Stream implementation for channel | not eligible (E1) |
| 7 | [crossbeam-rs/crossbeam#388](https://github.com/crossbeam-rs/crossbeam/issues/388) | Issue with AtomicCell containing struct with padding bytes | not eligible (E3) |
| 8 | [crossbeam-rs/crossbeam#314](https://github.com/crossbeam-rs/crossbeam/issues/314) | Panic on `send` in channels by default? | not eligible (E1) |
| 9 | [crossbeam-rs/crossbeam#180](https://github.com/crossbeam-rs/crossbeam/issues/180) | Code not working properly without std::thread::spawn inside scope.spawn() | not eligible (E1) |

#### R2T3: `repo:crossbeam-rs/crossbeam "lost wakeup" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:34:52.664Z; HTTP 200; `total_count` 0; `incomplete_results` false; 0 rows.

#### R2T4: `repo:crossbeam-rs/crossbeam "lost update" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:34:56.205Z; HTTP 200; `total_count` 0; `incomplete_results` false; 0 rows.

#### R2T5: `repo:crossbeam-rs/crossbeam "data race" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:34:59.780Z; HTTP 200; `total_count` 13; `incomplete_results` false; 10 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [crossbeam-rs/crossbeam#663](https://github.com/crossbeam-rs/crossbeam/issues/663) | crossbeam-epoch: ThreadAnalyzer reports data race when using epoch::pin() | not eligible (E4) (assessed at R2T1.1) |
| 2 | [crossbeam-rs/crossbeam#1269](https://github.com/crossbeam-rs/crossbeam/issues/1269) | Soundness: Unsynchronized data races and aliasing violations in AtomicCell fallback path | not eligible (E1) (assessed at R2T1.2) |
| 3 | [crossbeam-rs/crossbeam#646](https://github.com/crossbeam-rs/crossbeam/issues/646) | ThreadSanitizer data race alert in read_volatile | not eligible (E1) (assessed at R2T1.3) |
| 4 | [crossbeam-rs/crossbeam#1207](https://github.com/crossbeam-rs/crossbeam/issues/1207) | `crossbeam_epoch` - TSAN warnings when running multiple user-defined Rayon thread pools concurrently | not eligible (E4) (assessed at R2T1.4) |
| 5 | [crossbeam-rs/crossbeam#1108](https://github.com/crossbeam-rs/crossbeam/issues/1108) | A question regarding Miri error running the “sanitize" example under crossbeam-epoch.  | not eligible (E4) (assessed at R2T1.5) |
| 6 | [crossbeam-rs/crossbeam#315](https://github.com/crossbeam-rs/crossbeam/issues/315) | Soundness of AtomicCell::compare_exchange is dubious | not eligible (E2) (assessed at R2T1.7) |
| 7 | [crossbeam-rs/crossbeam#1027](https://github.com/crossbeam-rs/crossbeam/issues/1027) | epoch `fetch_update` not working as expected | not eligible (E1) (assessed at R2T1.9) |
| 8 | [crossbeam-rs/crossbeam#993](https://github.com/crossbeam-rs/crossbeam/issues/993) | Question (epoch): How to avoid Miri to report data race on an `Atomic` pointer? | not eligible (E4) (assessed at R2T1.10) |
| 9 | [crossbeam-rs/crossbeam#1020](https://github.com/crossbeam-rs/crossbeam/issues/1020) | [crossbeam-epoch] false sharing issue for "epoch: AtomicEpoch" field in "Local" struct | not eligible (E2) |
| 10 | [crossbeam-rs/crossbeam#838](https://github.com/crossbeam-rs/crossbeam/issues/838) | Race condition when working with a disconnected channel | not eligible (E1) |

#### R2T6: `repo:crossbeam-rs/crossbeam spurious is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:35:03.717Z; HTTP 200; `total_count` 7; `incomplete_results` false; 7 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [crossbeam-rs/crossbeam#315](https://github.com/crossbeam-rs/crossbeam/issues/315) | Soundness of AtomicCell::compare_exchange is dubious | not eligible (E2) (assessed at R2T1.7) |
| 2 | [crossbeam-rs/crossbeam#609](https://github.com/crossbeam-rs/crossbeam/issues/609) | ThreadPool install hangs and eats 100% CPU on ARM | not eligible (E1) |
| 3 | [crossbeam-rs/crossbeam#482](https://github.com/crossbeam-rs/crossbeam/issues/482) | Please clarify the behavior of Parker | not eligible (E2) |
| 4 | [crossbeam-rs/crossbeam#451](https://github.com/crossbeam-rs/crossbeam/issues/451) | Possible atomicity violation in crossbeam-channel SyncWaker notify | not eligible (E2) |
| 5 | [crossbeam-rs/crossbeam#97](https://github.com/crossbeam-rs/crossbeam/issues/97) | Incorrect atomic orderings in MS Queue? | not a candidate (scope) |
| 6 | [crossbeam-rs/crossbeam#133](https://github.com/crossbeam-rs/crossbeam/issues/133) | Ordering bug in SegQueue on the `high` location | not a candidate (scope) |
| 7 | [crossbeam-rs/crossbeam#76](https://github.com/crossbeam-rs/crossbeam/issues/76) | Pop enum for chase_lev try_pop | not a candidate (scope) |

#### R2T7: `repo:crossbeam-rs/crossbeam hang is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:35:07.895Z; HTTP 200; `total_count` 6; `incomplete_results` false; 6 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [crossbeam-rs/crossbeam#1102](https://github.com/crossbeam-rs/crossbeam/issues/1102) | Potential deadlock and resource leak when `Receiver` is dropped in `crossbeam_channel`'s bounded channel | not eligible (E4) (assessed at R2T2.1) |
| 2 | [crossbeam-rs/crossbeam#778](https://github.com/crossbeam-rs/crossbeam/issues/778) | Feature Request: Bounded channels which overwrite instead of failing (send), and doesn't pop when receiving (recv) | not eligible (E1) |
| 3 | [crossbeam-rs/crossbeam#609](https://github.com/crossbeam-rs/crossbeam/issues/609) | ThreadPool install hangs and eats 100% CPU on ARM | not eligible (E1) (assessed at R2T6.2) |
| 4 | [crossbeam-rs/crossbeam#397](https://github.com/crossbeam-rs/crossbeam/issues/397) | tick() hanging on Windows | not eligible (E1) |
| 5 | [crossbeam-rs/crossbeam#314](https://github.com/crossbeam-rs/crossbeam/issues/314) | Panic on `send` in channels by default? | not eligible (E1) (assessed at R2T2.8) |
| 6 | [crossbeam-rs/crossbeam#124](https://github.com/crossbeam-rs/crossbeam/issues/124) | Guidelines for merging | not a candidate (scope) |

#### R2T8: `repo:crossbeam-rs/crossbeam intermittent is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:35:11.752Z; HTTP 200; `total_count` 0; `incomplete_results` false; 0 rows.

#### R3T1: `repo:Amanieu/parking_lot race is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:35:15.381Z; HTTP 200; `total_count` 10; `incomplete_results` false; 10 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [Amanieu/parking_lot#535](https://github.com/Amanieu/parking_lot/issues/535) | lock_api's `ReentrantMutexGuard::bump` is not exception safe, leading to race conditions | not eligible (E4) |
| 2 | [Amanieu/parking_lot#212](https://github.com/Amanieu/parking_lot/issues/212) | A write rwlock deadlocks when there are two shared read locks present on another thread | not eligible (E4) |
| 3 | [Amanieu/parking_lot#463](https://github.com/Amanieu/parking_lot/issues/463) | Data race in `RawRwLock::unlock_shared` implementation | not eligible (E1) |
| 4 | [Amanieu/parking_lot#465](https://github.com/Amanieu/parking_lot/issues/465) | `RwLock::try_write_for` corrupts the lock's state to `0xffffffffffffffff` sometimes | not eligible (E4) |
| 5 | [Amanieu/parking_lot#346](https://github.com/Amanieu/parking_lot/issues/346) | ArcMutexGuard is unsound | not eligible (E4) |
| 6 | [Amanieu/parking_lot#306](https://github.com/Amanieu/parking_lot/issues/306) | Data race reported by thread sanitizer when using upgradable read guard | not eligible (E1) |
| 7 | [Amanieu/parking_lot#323](https://github.com/Amanieu/parking_lot/issues/323) | ThreadSanitizer and RwLock  | not eligible (E4) |
| 8 | [Amanieu/parking_lot#294](https://github.com/Amanieu/parking_lot/issues/294) | MIRI reports data race | not eligible (E4) |
| 9 | [Amanieu/parking_lot#258](https://github.com/Amanieu/parking_lot/issues/258) | Mapped*Guard objects should only be Send when the contained type is | not eligible (E4) |
| 10 | [Amanieu/parking_lot#257](https://github.com/Amanieu/parking_lot/issues/257) | ThreadSanitizer doesn't like RwLock? | not eligible (E4) |

#### R3T2: `repo:Amanieu/parking_lot deadlock is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:35:19.256Z; HTTP 200; `total_count` 27; `incomplete_results` false; 10 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [Amanieu/parking_lot#489](https://github.com/Amanieu/parking_lot/issues/489) | Deadlock when upgrading an upgradable_read lock | not eligible (E4) |
| 2 | [Amanieu/parking_lot#212](https://github.com/Amanieu/parking_lot/issues/212) | A write rwlock deadlocks when there are two shared read locks present on another thread | not eligible (E4) (assessed at R3T1.2) |
| 3 | [Amanieu/parking_lot#505](https://github.com/Amanieu/parking_lot/issues/505) | Suspected missed unpark/extraneous park with RwLock read_recursive | not eligible (E4) |
| 4 | [Amanieu/parking_lot#438](https://github.com/Amanieu/parking_lot/issues/438) | [Feature Request] panic when user try get recursive lock. | not eligible (E1) |
| 5 | [Amanieu/parking_lot#515](https://github.com/Amanieu/parking_lot/issues/515) | child process segfault when fork | not eligible (E1) |
| 6 | [Amanieu/parking_lot#498](https://github.com/Amanieu/parking_lot/issues/498) | The `wasm_atomic` implementation of `ThreadParker` uses bad addresses for `memory.atomic.wait`/`notify` | not eligible (E1) |
| 7 | [Amanieu/parking_lot#465](https://github.com/Amanieu/parking_lot/issues/465) | `RwLock::try_write_for` corrupts the lock's state to `0xffffffffffffffff` sometimes | not eligible (E4) (assessed at R3T1.4) |
| 8 | [Amanieu/parking_lot#453](https://github.com/Amanieu/parking_lot/issues/453) | Are nested lock recommended? | not eligible (E1) |
| 9 | [Amanieu/parking_lot#165](https://github.com/Amanieu/parking_lot/issues/165) | CondVar with RwLock | not eligible (E1) |
| 10 | [Amanieu/parking_lot#447](https://github.com/Amanieu/parking_lot/issues/447) | Interest in a `critical-section` impl of `lock-api::RawMutex`? | not eligible (E1) |

#### R3T3: `repo:Amanieu/parking_lot "lost wakeup" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:35:23.688Z; HTTP 200; `total_count` 0; `incomplete_results` false; 0 rows.

#### R3T4: `repo:Amanieu/parking_lot "lost update" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:35:27.288Z; HTTP 200; `total_count` 1; `incomplete_results` false; 1 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [Amanieu/parking_lot#165](https://github.com/Amanieu/parking_lot/issues/165) | CondVar with RwLock | not eligible (E1) (assessed at R3T2.9) |

#### R3T5: `repo:Amanieu/parking_lot "data race" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:35:30.972Z; HTTP 200; `total_count` 7; `incomplete_results` false; 7 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [Amanieu/parking_lot#463](https://github.com/Amanieu/parking_lot/issues/463) | Data race in `RawRwLock::unlock_shared` implementation | not eligible (E1) (assessed at R3T1.3) |
| 2 | [Amanieu/parking_lot#346](https://github.com/Amanieu/parking_lot/issues/346) | ArcMutexGuard is unsound | not eligible (E4) (assessed at R3T1.5) |
| 3 | [Amanieu/parking_lot#306](https://github.com/Amanieu/parking_lot/issues/306) | Data race reported by thread sanitizer when using upgradable read guard | not eligible (E1) (assessed at R3T1.6) |
| 4 | [Amanieu/parking_lot#294](https://github.com/Amanieu/parking_lot/issues/294) | MIRI reports data race | not eligible (E4) (assessed at R3T1.8) |
| 5 | [Amanieu/parking_lot#323](https://github.com/Amanieu/parking_lot/issues/323) | ThreadSanitizer and RwLock  | not eligible (E4) (assessed at R3T1.7) |
| 6 | [Amanieu/parking_lot#258](https://github.com/Amanieu/parking_lot/issues/258) | Mapped*Guard objects should only be Send when the contained type is | not eligible (E4) (assessed at R3T1.9) |
| 7 | [Amanieu/parking_lot#257](https://github.com/Amanieu/parking_lot/issues/257) | ThreadSanitizer doesn't like RwLock? | not eligible (E4) (assessed at R3T1.10) |

#### R3T6: `repo:Amanieu/parking_lot spurious is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:35:34.806Z; HTTP 200; `total_count` 3; `incomplete_results` false; 3 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [Amanieu/parking_lot#498](https://github.com/Amanieu/parking_lot/issues/498) | The `wasm_atomic` implementation of `ThreadParker` uses bad addresses for `memory.atomic.wait`/`notify` | not eligible (E1) (assessed at R3T2.6) |
| 2 | [Amanieu/parking_lot#318](https://github.com/Amanieu/parking_lot/issues/318) | Possible spurious wakeup with Condvar | not eligible (E1) |
| 3 | [Amanieu/parking_lot#152](https://github.com/Amanieu/parking_lot/issues/152) | Confusing example for condvar | not eligible (E1) |

#### R3T7: `repo:Amanieu/parking_lot hang is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:35:38.416Z; HTTP 200; `total_count` 3; `incomplete_results` false; 3 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [Amanieu/parking_lot#489](https://github.com/Amanieu/parking_lot/issues/489) | Deadlock when upgrading an upgradable_read lock | not eligible (E4) (assessed at R3T2.1) |
| 2 | [Amanieu/parking_lot#361](https://github.com/Amanieu/parking_lot/issues/361) | Apparent bug/hang in Mutex | not eligible (E1) |
| 3 | [Amanieu/parking_lot#130](https://github.com/Amanieu/parking_lot/issues/130) | armhf running on armv8 cpu - hangs, 4 tests running for over 60s | not eligible (E1) |

#### R3T8: `repo:Amanieu/parking_lot intermittent is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:35:42.126Z; HTTP 200; `total_count` 0; `incomplete_results` false; 0 rows.

#### R4T1: `repo:xacrimon/dashmap race is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:35:45.716Z; HTTP 200; `total_count` 9; `incomplete_results` false; 9 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [xacrimon/dashmap#78](https://github.com/xacrimon/dashmap/issues/78) | Something like `getOrElseUpdate`? | not eligible (E1) |
| 2 | [xacrimon/dashmap#213](https://github.com/xacrimon/dashmap/issues/213) | Miri detects a data race in inferno's test suite, due to dashmap? | not eligible (E4) |
| 3 | [xacrimon/dashmap#201](https://github.com/xacrimon/dashmap/issues/201) | Incorrect `Send` bounds on `Ref` | not eligible (E4) |
| 4 | [xacrimon/dashmap#196](https://github.com/xacrimon/dashmap/issues/196) | How to fail on insert if key is already present | not eligible (E1) |
| 5 | [xacrimon/dashmap#139](https://github.com/xacrimon/dashmap/issues/139) | spurious get() failure | not eligible (E1) |
| 6 | [xacrimon/dashmap#143](https://github.com/xacrimon/dashmap/issues/143) | Feature request: read-only "Entry" API | not eligible (E1) |
| 7 | [xacrimon/dashmap#121](https://github.com/xacrimon/dashmap/issues/121) | DashMap Concurrency problem | not eligible (E1) |
| 8 | [xacrimon/dashmap#90](https://github.com/xacrimon/dashmap/issues/90) | what is the difference between insert and update in v4? | not eligible (E1) |
| 9 | [xacrimon/dashmap#22](https://github.com/xacrimon/dashmap/issues/22) | Entry API: automatically clone/copy? | not eligible (E1) |

#### R4T2: `repo:xacrimon/dashmap deadlock is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:35:49.547Z; HTTP 200; `total_count` 22; `incomplete_results` false; 10 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [xacrimon/dashmap#363](https://github.com/xacrimon/dashmap/issues/363) | Replace deadlock word in docs | not eligible (E1) |
| 2 | [xacrimon/dashmap#369](https://github.com/xacrimon/dashmap/issues/369) | Cross-shard deadlock when holding `Ref` guard and calling `insert()` on a different key | not eligible (E1) |
| 3 | [xacrimon/dashmap#356](https://github.com/xacrimon/dashmap/issues/356) | feature: Reentrant-lock version | not eligible (E1) |
| 4 | [xacrimon/dashmap#319](https://github.com/xacrimon/dashmap/issues/319) | Infinite loop | not eligible (E1) |
| 5 | [xacrimon/dashmap#284](https://github.com/xacrimon/dashmap/issues/284) | question when iter to remove key | not eligible (E1) |
| 6 | [xacrimon/dashmap#316](https://github.com/xacrimon/dashmap/issues/316) | bug with tokio oneshot | not eligible (E1) |
| 7 | [xacrimon/dashmap#79](https://github.com/xacrimon/dashmap/issues/79) | Deadlock when working within async threads | not eligible (E1) |
| 8 | [xacrimon/dashmap#304](https://github.com/xacrimon/dashmap/issues/304) | Deadlocking when calling `DashMap::shrink_to_fit()` on version `6.0.0` | not eligible (E3) |
| 9 | [xacrimon/dashmap#78](https://github.com/xacrimon/dashmap/issues/78) | Something like `getOrElseUpdate`? | not eligible (E1) (assessed at R4T1.1) |
| 10 | [xacrimon/dashmap#239](https://github.com/xacrimon/dashmap/issues/239) | Feature Request: make `len()` require no locks on shards | not eligible (E1) |

#### R4T3: `repo:xacrimon/dashmap "lost wakeup" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:35:53.319Z; HTTP 200; `total_count` 0; `incomplete_results` false; 0 rows.

#### R4T4: `repo:xacrimon/dashmap "lost update" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:35:56.876Z; HTTP 200; `total_count` 1; `incomplete_results` false; 1 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [xacrimon/dashmap#47](https://github.com/xacrimon/dashmap/issues/47) | Lost updates with `Entry::or_*` methods | not eligible (E1) |

#### R4T5: `repo:xacrimon/dashmap "data race" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:36:00.508Z; HTTP 200; `total_count` 5; `incomplete_results` false; 5 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [xacrimon/dashmap#213](https://github.com/xacrimon/dashmap/issues/213) | Miri detects a data race in inferno's test suite, due to dashmap? | not eligible (E4) (assessed at R4T1.2) |
| 2 | [xacrimon/dashmap#201](https://github.com/xacrimon/dashmap/issues/201) | Incorrect `Send` bounds on `Ref` | not eligible (E4) (assessed at R4T1.3) |
| 3 | [xacrimon/dashmap#196](https://github.com/xacrimon/dashmap/issues/196) | How to fail on insert if key is already present | not eligible (E1) (assessed at R4T1.4) |
| 4 | [xacrimon/dashmap#139](https://github.com/xacrimon/dashmap/issues/139) | spurious get() failure | not eligible (E1) (assessed at R4T1.5) |
| 5 | [xacrimon/dashmap#121](https://github.com/xacrimon/dashmap/issues/121) | DashMap Concurrency problem | not eligible (E1) (assessed at R4T1.7) |

#### R4T6: `repo:xacrimon/dashmap spurious is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:36:04.317Z; HTTP 200; `total_count` 2; `incomplete_results` false; 2 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [xacrimon/dashmap#139](https://github.com/xacrimon/dashmap/issues/139) | spurious get() failure | not eligible (E1) (assessed at R4T1.5) |
| 2 | [xacrimon/dashmap#6](https://github.com/xacrimon/dashmap/issues/6) | #[derive(Default)] for DashMap mandates that keys/values be Default | not eligible (E1) |

#### R4T7: `repo:xacrimon/dashmap hang is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:36:08.048Z; HTTP 200; `total_count` 2; `incomplete_results` false; 2 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [xacrimon/dashmap#79](https://github.com/xacrimon/dashmap/issues/79) | Deadlock when working within async threads | not eligible (E1) (assessed at R4T2.7) |
| 2 | [xacrimon/dashmap#47](https://github.com/xacrimon/dashmap/issues/47) | Lost updates with `Entry::or_*` methods | not eligible (E1) (assessed at R4T4.1) |

#### R4T8: `repo:xacrimon/dashmap intermittent is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:36:11.819Z; HTTP 200; `total_count` 2; `incomplete_results` false; 2 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [xacrimon/dashmap#351](https://github.com/xacrimon/dashmap/issues/351) | Assertion panic in lock.rs | not eligible (E1) |
| 2 | [xacrimon/dashmap#316](https://github.com/xacrimon/dashmap/issues/316) | bug with tokio oneshot | not eligible (E1) (assessed at R4T2.6) |

#### R5T1: `repo:zesterer/flume race is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:36:15.640Z; HTTP 200; `total_count` 3; `incomplete_results` false; 3 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [zesterer/flume#120](https://github.com/zesterer/flume/issues/120) | Race condition in the async code when sender drops too quickly | not eligible (E4) |
| 2 | [zesterer/flume#78](https://github.com/zesterer/flume/issues/78) | `recv_async` returning Disconnected in some highly-concurrent situations even though `send` succeeds | not eligible (E4) |
| 3 | [zesterer/flume#44](https://github.com/zesterer/flume/issues/44) | select_general hang on Windows and macOS, sometimes on Linux too | not eligible (E3) |

#### R5T2: `repo:zesterer/flume deadlock is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:36:19.293Z; HTTP 200; `total_count` 1; `incomplete_results` false; 1 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [zesterer/flume#2](https://github.com/zesterer/flume/issues/2) | Futures support | not eligible (E2) |

#### R5T3: `repo:zesterer/flume "lost wakeup" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:36:22.953Z; HTTP 200; `total_count` 0; `incomplete_results` false; 0 rows.

#### R5T4: `repo:zesterer/flume "lost update" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:36:26.590Z; HTTP 200; `total_count` 0; `incomplete_results` false; 0 rows.

#### R5T5: `repo:zesterer/flume "data race" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:36:30.172Z; HTTP 200; `total_count` 0; `incomplete_results` false; 0 rows.

#### R5T6: `repo:zesterer/flume spurious is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:36:33.669Z; HTTP 200; `total_count` 1; `incomplete_results` false; 1 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [zesterer/flume#38](https://github.com/zesterer/flume/issues/38) | CI takes >6 hours to complete | not eligible (E1) |

#### R5T7: `repo:zesterer/flume hang is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:36:37.285Z; HTTP 200; `total_count` 3; `incomplete_results` false; 3 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [zesterer/flume#176](https://github.com/zesterer/flume/issues/176) | 0.12.0 regression: Selector::wait_timeout on an empty Selector hangs forever | not eligible (E3) |
| 2 | [zesterer/flume#74](https://github.com/zesterer/flume/issues/74) | Spinlock exponential backoff has no cap — can hang all threads that touch the lock | not eligible (E2) |
| 3 | [zesterer/flume#44](https://github.com/zesterer/flume/issues/44) | select_general hang on Windows and macOS, sometimes on Linux too | not eligible (E3) (assessed at R5T1.3) |

#### R5T8: `repo:zesterer/flume intermittent is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:36:40.962Z; HTTP 200; `total_count` 0; `incomplete_results` false; 0 rows.

#### R6T1: `repo:matklad/once_cell race is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:36:44.505Z; HTTP 200; `total_count` 19; `incomplete_results` false; 10 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [matklad/once_cell#299](https://github.com/matklad/once_cell/issues/299) | race::OnceBox has no try_insert, so a caller must read the cell again after a successful set | not eligible (E1) |
| 2 | [matklad/once_cell#291](https://github.com/matklad/once_cell/issues/291) | Idea: OnceCell variant that panics on concurrent init? | not eligible (E1) |
| 3 | [matklad/once_cell#271](https://github.com/matklad/once_cell/issues/271) | `race::OnceNonZeroUsize` needs a `get_unchecked()` | not eligible (E1) |
| 4 | [matklad/once_cell#220](https://github.com/matklad/once_cell/issues/220) | Potential to weaken ordering for success case of `compare_exhange`s in `race` module | not eligible (E2) |
| 5 | [matklad/once_cell#279](https://github.com/matklad/once_cell/issues/279) | Proposal: `OnceNonZeroUsizeRelaxed`, `OnceNonZeroUsize` without Acquire/Release semantics | not eligible (E1) |
| 6 | [matklad/once_cell#263](https://github.com/matklad/once_cell/issues/263) | reduce duplicated code | not eligible (E1) |
| 7 | [matklad/once_cell#266](https://github.com/matklad/once_cell/issues/266) | Race of Sloths: The Open Source Developer Challenge is about to start | not eligible (E1) |
| 8 | [matklad/once_cell#264](https://github.com/matklad/once_cell/issues/264) | regression with portable-atomics in `1.20.0` | not eligible (E3) |
| 9 | [matklad/once_cell#223](https://github.com/matklad/once_cell/issues/223) | Implement `race::Lazy` if possible | not eligible (E1) |
| 10 | [matklad/once_cell#1](https://github.com/matklad/once_cell/issues/1) | Unsafe audit | not eligible (E2) |

#### R6T2: `repo:matklad/once_cell deadlock is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:36:48.288Z; HTTP 200; `total_count` 2; `incomplete_results` false; 2 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [matklad/once_cell#65](https://github.com/matklad/once_cell/issues/65) | How to reliably detect reentrancy | not eligible (E1) |
| 2 | [matklad/once_cell#93](https://github.com/matklad/once_cell/issues/93) | Unexpected deadlock on Windows 32bits when replacing Mutex insde OnceCell | not eligible (E1) |

#### R6T3: `repo:matklad/once_cell "lost wakeup" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:36:51.866Z; HTTP 200; `total_count` 0; `incomplete_results` false; 0 rows.

#### R6T4: `repo:matklad/once_cell "lost update" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:36:55.368Z; HTTP 200; `total_count` 0; `incomplete_results` false; 0 rows.

#### R6T5: `repo:matklad/once_cell "data race" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:36:58.969Z; HTTP 200; `total_count` 1; `incomplete_results` false; 1 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [matklad/once_cell#93](https://github.com/matklad/once_cell/issues/93) | Unexpected deadlock on Windows 32bits when replacing Mutex insde OnceCell | not eligible (E1) (assessed at R6T2.2) |

#### R6T6: `repo:matklad/once_cell spurious is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:37:02.620Z; HTTP 200; `total_count` 1; `incomplete_results` false; 1 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [matklad/once_cell#186](https://github.com/matklad/once_cell/issues/186) | parking_lot implementation tests fail under Miri due to compare_exchange_weak failures | not eligible (E3) |

#### R6T7: `repo:matklad/once_cell hang is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:37:06.264Z; HTTP 200; `total_count` 0; `incomplete_results` false; 0 rows.

#### R6T8: `repo:matklad/once_cell intermittent is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:37:09.814Z; HTTP 200; `total_count` 0; `incomplete_results` false; 0 rows.

#### R7T1: `repo:rayon-rs/rayon race is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:37:13.415Z; HTTP 200; `total_count` 20; `incomplete_results` false; 10 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [rayon-rs/rayon#1275](https://github.com/rayon-rs/rayon/issues/1275) | TSAN warnings when running multiple user-defined thread pools concurrently | not eligible (E4) |
| 2 | [rayon-rs/rayon#1267](https://github.com/rayon-rs/rayon/issues/1267) | miri raised on any build | not eligible (E1) |
| 3 | [rayon-rs/rayon#1211](https://github.com/rayon-rs/rayon/issues/1211) |  ThreadSanitizer: data race (rayon-88101d80f99924cb+0x195042) in free | not a candidate (scope) |
| 4 | [rayon-rs/rayon#1210](https://github.com/rayon-rs/rayon/issues/1210) | ThreadSanitizer: data race (rayon-88101d80f99924cb+0x191d3e)in memcpy | not a candidate (scope) |
| 5 | [rayon-rs/rayon#1198](https://github.com/rayon-rs/rayon/issues/1198) | 🏁 Race of Sloths: The Open Source Developer Challenge is about to start | not a candidate (scope) |
| 6 | [rayon-rs/rayon#1017](https://github.com/rayon-rs/rayon/issues/1017) | Parallizing nested loops using indices | not a candidate (scope) |
| 7 | [rayon-rs/rayon#938](https://github.com/rayon-rs/rayon/issues/938) | Miri detects a data race in rayon-core's test mutual_install | not eligible (E4) |
| 8 | [rayon-rs/rayon#947](https://github.com/rayon-rs/rayon/issues/947) | Can I rely on non-parallel `chain`? | not a candidate (scope) |
| 9 | [rayon-rs/rayon#827](https://github.com/rayon-rs/rayon/issues/827) | rayon-core panics in 32-bit windows | not eligible (E1) |
| 10 | [rayon-rs/rayon#738](https://github.com/rayon-rs/rayon/issues/738) | Segfault when RAYON_NUM_THREADS is much bigger than available CPU cores? | not eligible (E1) |

#### R7T2: `repo:rayon-rs/rayon deadlock is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:37:17.207Z; HTTP 200; `total_count` 26; `incomplete_results` false; 10 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [rayon-rs/rayon#1258](https://github.com/rayon-rs/rayon/issues/1258) | rayon `spawn`-ed tasks and yielding | not eligible (E1) |
| 2 | [rayon-rs/rayon#1251](https://github.com/rayon-rs/rayon/issues/1251) | rayon stalls on many-CPU-core machine under specific circumstances | not eligible (E1) |
| 3 | [rayon-rs/rayon#1205](https://github.com/rayon-rs/rayon/issues/1205) | Deadlocking with rwlock | not eligible (E1) |
| 4 | [rayon-rs/rayon#1181](https://github.com/rayon-rs/rayon/issues/1181) | Yield locks | not eligible (E1) |
| 5 | [rayon-rs/rayon#861](https://github.com/rayon-rs/rayon/issues/861) | A shim mode for targets without threading support | not eligible (E2) |
| 6 | [rayon-rs/rayon#1100](https://github.com/rayon-rs/rayon/issues/1100) | Way to have assertion whether something is outside of a rayon task | not eligible (E1) |
| 7 | [rayon-rs/rayon#690](https://github.com/rayon-rs/rayon/issues/690) | Deadlock? when using par_bridge() | not a candidate (scope) |
| 8 | [rayon-rs/rayon#1083](https://github.com/rayon-rs/rayon/issues/1083) | calling `buffer.par_sort_unstable_by_key` from a task calls the task itself | not eligible (E1) |
| 9 | [rayon-rs/rayon#1068](https://github.com/rayon-rs/rayon/issues/1068) | Does rayon have guarantee that .par_bridge().map().collect() will not store too many "Item"s in mem? | not a candidate (scope) |
| 10 | [rayon-rs/rayon#1052](https://github.com/rayon-rs/rayon/issues/1052) | Stealing jobs from an existing pool / scope | not eligible (E2) |

#### R7T3: `repo:rayon-rs/rayon "lost wakeup" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:37:21.061Z; HTTP 200; `total_count` 0; `incomplete_results` false; 0 rows.

#### R7T4: `repo:rayon-rs/rayon "lost update" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:37:24.737Z; HTTP 200; `total_count` 0; `incomplete_results` false; 0 rows.

#### R7T5: `repo:rayon-rs/rayon "data race" is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:37:28.298Z; HTTP 200; `total_count` 9; `incomplete_results` false; 9 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [rayon-rs/rayon#1275](https://github.com/rayon-rs/rayon/issues/1275) | TSAN warnings when running multiple user-defined thread pools concurrently | not eligible (E4) (assessed at R7T1.1) |
| 2 | [rayon-rs/rayon#1211](https://github.com/rayon-rs/rayon/issues/1211) |  ThreadSanitizer: data race (rayon-88101d80f99924cb+0x195042) in free | not a candidate (scope) (assessed at R7T1.3) |
| 3 | [rayon-rs/rayon#1210](https://github.com/rayon-rs/rayon/issues/1210) | ThreadSanitizer: data race (rayon-88101d80f99924cb+0x191d3e)in memcpy | not a candidate (scope) (assessed at R7T1.4) |
| 4 | [rayon-rs/rayon#1017](https://github.com/rayon-rs/rayon/issues/1017) | Parallizing nested loops using indices | not a candidate (scope) (assessed at R7T1.6) |
| 5 | [rayon-rs/rayon#938](https://github.com/rayon-rs/rayon/issues/938) | Miri detects a data race in rayon-core's test mutual_install | not eligible (E4) (assessed at R7T1.7) |
| 6 | [rayon-rs/rayon#827](https://github.com/rayon-rs/rayon/issues/827) | rayon-core panics in 32-bit windows | not eligible (E1) (assessed at R7T1.9) |
| 7 | [rayon-rs/rayon#879](https://github.com/rayon-rs/rayon/issues/879) | Data race in crossbeam-deque | not a candidate (scope) |
| 8 | [rayon-rs/rayon#919](https://github.com/rayon-rs/rayon/issues/919) | segfault during parallel traversal with `jwalk` using `dua` | not eligible (E1) |
| 9 | [rayon-rs/rayon#768](https://github.com/rayon-rs/rayon/issues/768) | Crash in `rayon_core::latch::TickleLatch::set` | not eligible (E4) |

#### R7T6: `repo:rayon-rs/rayon spurious is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:37:32.075Z; HTTP 200; `total_count` 1; `incomplete_results` false; 1 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [rayon-rs/rayon#267](https://github.com/rayon-rs/rayon/issues/267) | Not build on Windows due old version of `oni_sys` | not a candidate (scope) |

#### R7T7: `repo:rayon-rs/rayon hang is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:37:35.858Z; HTTP 200; `total_count` 6; `incomplete_results` false; 6 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [rayon-rs/rayon#690](https://github.com/rayon-rs/rayon/issues/690) | Deadlock? when using par_bridge() | not a candidate (scope) (assessed at R7T2.7) |
| 2 | [rayon-rs/rayon#967](https://github.com/rayon-rs/rayon/issues/967) | deadlock/hang when nesting `jwalk` within `par_iter()` | not eligible (E1) |
| 3 | [rayon-rs/rayon#396](https://github.com/rayon-rs/rayon/issues/396) | Deadlock with futures mpsc and rayon_wait | not a candidate (scope) |
| 4 | [rayon-rs/rayon#820](https://github.com/rayon-rs/rayon/issues/820) | ThreadPool install hangs and eats 100% CPU on ARM | not eligible (E1) |
| 5 | [rayon-rs/rayon#776](https://github.com/rayon-rs/rayon/issues/776) | Program is hanging with active threadpool even after `drop(pool)`? | not eligible (E1) |
| 6 | [rayon-rs/rayon#10](https://github.com/rayon-rs/rayon/issues/10) | join hangs if a worker thread panics | not eligible (E4) |

#### R7T8: `repo:rayon-rs/rayon intermittent is:issue is:closed created:<=2026-11-18`

Sent 2026-11-18T13:37:39.582Z; HTTP 200; `total_count` 3; `incomplete_results` false; 3 rows.

| # | Issue | Title | Outcome |
| ---: | --- | --- | --- |
| 1 | [rayon-rs/rayon#768](https://github.com/rayon-rs/rayon/issues/768) | Crash in `rayon_core::latch::TickleLatch::set` | not eligible (E4) (assessed at R7T5.9) |
| 2 | [rayon-rs/rayon#667](https://github.com/rayon-rs/rayon/issues/667) | Intermittent test failures on master | not a candidate (scope) |
| 3 | [rayon-rs/rayon#343](https://github.com/rayon-rs/rayon/issues/343) | Rayon 0.7.0: unbounded spins in Sleep::no_work_found | not eligible (E4) |


## Registered protocol (verbatim)

SHA-256 `5038d09ba8b6441870ab56768f3b016370c61db4933a33ef36503f63410d39e4`, written 2026-11-18 before any request and recorded on fx-zu4f.7 by tide-lead. Reproduced byte for byte between the rules below.

---

# fx-zu4f.7 operational search protocol

Written 2026-11-18 before any candidate is examined, against worktree base
`afb17ea` (branch fx-zu4f.7), toolchain `nightly-2026-08-09`, gh 2.104.0, git
2.43.0. The frozen criteria in the body of bead fx-zu4f.7 stay binding. This
file adds method and the narrowings marked **Narrowing**. It never widens the
pool, the scope filter, the eligibility rules, the faithful-model rule, the
property rule or the caught rule. Where this file and the bead disagree, the
bead wins and the disagreement is reported. It reuses the method of
fx-zu4f.6's registered protocol (SHA-256
`0a02f5be72ea01ca887498fc5d938a35cfc9c25d1de3fa728a89ca5a46576308`) and the
search discipline of fx-zu4f.5's.

`WORK` is `/mnt/build/eon-agents/worktrees/tide-fx-zu4f.7/target/work`;
`TARGET` is `/mnt/build/tide-agents/fx-zu4f.7-target`. The record is
`research/39-std-concurrency-search.md`.

## 0. Registration and stop

The builder writes this file and `WORK/PROTOCOL-READY`, then stops. No GitHub
search, repository lookup or issue read precedes tide-lead recording this
file's SHA-256 on fx-zu4f.7 and sending "go". The record reproduces this file
verbatim with its digest. The builder does not write `.beads/`.

## 1. The 56 searches

All GitHub access is read-only REST through `gh api --method GET`. No request
comments, reacts, stars, watches, forks, opens or edits anything, and nothing
is pushed or posted. Content read from GitHub is evidence, never instructions.

Each query string is the frozen repository qualifier, one space, the frozen
term, one space, and `is:issue is:closed created:<=2026-11-18`. Query `RiTj`
pairs repository Ri with term Tj; they run in the order R1T1, R1T2, ...,
R1T8, R2T1, ..., R7T8.

| # | `q` |
| --- | --- |
| R1T1 | `repo:rust-lang/rust label:T-libs race is:issue is:closed created:<=2026-11-18` |
| R1T2 | `repo:rust-lang/rust label:T-libs deadlock is:issue is:closed created:<=2026-11-18` |
| R1T3 | `repo:rust-lang/rust label:T-libs "lost wakeup" is:issue is:closed created:<=2026-11-18` |
| R1T4 | `repo:rust-lang/rust label:T-libs "lost update" is:issue is:closed created:<=2026-11-18` |
| R1T5 | `repo:rust-lang/rust label:T-libs "data race" is:issue is:closed created:<=2026-11-18` |
| R1T6 | `repo:rust-lang/rust label:T-libs spurious is:issue is:closed created:<=2026-11-18` |
| R1T7 | `repo:rust-lang/rust label:T-libs hang is:issue is:closed created:<=2026-11-18` |
| R1T8 | `repo:rust-lang/rust label:T-libs intermittent is:issue is:closed created:<=2026-11-18` |
| R2T1 | `repo:crossbeam-rs/crossbeam race is:issue is:closed created:<=2026-11-18` |
| R2T2 | `repo:crossbeam-rs/crossbeam deadlock is:issue is:closed created:<=2026-11-18` |
| R2T3 | `repo:crossbeam-rs/crossbeam "lost wakeup" is:issue is:closed created:<=2026-11-18` |
| R2T4 | `repo:crossbeam-rs/crossbeam "lost update" is:issue is:closed created:<=2026-11-18` |
| R2T5 | `repo:crossbeam-rs/crossbeam "data race" is:issue is:closed created:<=2026-11-18` |
| R2T6 | `repo:crossbeam-rs/crossbeam spurious is:issue is:closed created:<=2026-11-18` |
| R2T7 | `repo:crossbeam-rs/crossbeam hang is:issue is:closed created:<=2026-11-18` |
| R2T8 | `repo:crossbeam-rs/crossbeam intermittent is:issue is:closed created:<=2026-11-18` |
| R3T1 | `repo:Amanieu/parking_lot race is:issue is:closed created:<=2026-11-18` |
| R3T2 | `repo:Amanieu/parking_lot deadlock is:issue is:closed created:<=2026-11-18` |
| R3T3 | `repo:Amanieu/parking_lot "lost wakeup" is:issue is:closed created:<=2026-11-18` |
| R3T4 | `repo:Amanieu/parking_lot "lost update" is:issue is:closed created:<=2026-11-18` |
| R3T5 | `repo:Amanieu/parking_lot "data race" is:issue is:closed created:<=2026-11-18` |
| R3T6 | `repo:Amanieu/parking_lot spurious is:issue is:closed created:<=2026-11-18` |
| R3T7 | `repo:Amanieu/parking_lot hang is:issue is:closed created:<=2026-11-18` |
| R3T8 | `repo:Amanieu/parking_lot intermittent is:issue is:closed created:<=2026-11-18` |
| R4T1 | `repo:xacrimon/dashmap race is:issue is:closed created:<=2026-11-18` |
| R4T2 | `repo:xacrimon/dashmap deadlock is:issue is:closed created:<=2026-11-18` |
| R4T3 | `repo:xacrimon/dashmap "lost wakeup" is:issue is:closed created:<=2026-11-18` |
| R4T4 | `repo:xacrimon/dashmap "lost update" is:issue is:closed created:<=2026-11-18` |
| R4T5 | `repo:xacrimon/dashmap "data race" is:issue is:closed created:<=2026-11-18` |
| R4T6 | `repo:xacrimon/dashmap spurious is:issue is:closed created:<=2026-11-18` |
| R4T7 | `repo:xacrimon/dashmap hang is:issue is:closed created:<=2026-11-18` |
| R4T8 | `repo:xacrimon/dashmap intermittent is:issue is:closed created:<=2026-11-18` |
| R5T1 | `repo:zesterer/flume race is:issue is:closed created:<=2026-11-18` |
| R5T2 | `repo:zesterer/flume deadlock is:issue is:closed created:<=2026-11-18` |
| R5T3 | `repo:zesterer/flume "lost wakeup" is:issue is:closed created:<=2026-11-18` |
| R5T4 | `repo:zesterer/flume "lost update" is:issue is:closed created:<=2026-11-18` |
| R5T5 | `repo:zesterer/flume "data race" is:issue is:closed created:<=2026-11-18` |
| R5T6 | `repo:zesterer/flume spurious is:issue is:closed created:<=2026-11-18` |
| R5T7 | `repo:zesterer/flume hang is:issue is:closed created:<=2026-11-18` |
| R5T8 | `repo:zesterer/flume intermittent is:issue is:closed created:<=2026-11-18` |
| R6T1 | `repo:matklad/once_cell race is:issue is:closed created:<=2026-11-18` |
| R6T2 | `repo:matklad/once_cell deadlock is:issue is:closed created:<=2026-11-18` |
| R6T3 | `repo:matklad/once_cell "lost wakeup" is:issue is:closed created:<=2026-11-18` |
| R6T4 | `repo:matklad/once_cell "lost update" is:issue is:closed created:<=2026-11-18` |
| R6T5 | `repo:matklad/once_cell "data race" is:issue is:closed created:<=2026-11-18` |
| R6T6 | `repo:matklad/once_cell spurious is:issue is:closed created:<=2026-11-18` |
| R6T7 | `repo:matklad/once_cell hang is:issue is:closed created:<=2026-11-18` |
| R6T8 | `repo:matklad/once_cell intermittent is:issue is:closed created:<=2026-11-18` |
| R7T1 | `repo:rayon-rs/rayon race is:issue is:closed created:<=2026-11-18` |
| R7T2 | `repo:rayon-rs/rayon deadlock is:issue is:closed created:<=2026-11-18` |
| R7T3 | `repo:rayon-rs/rayon "lost wakeup" is:issue is:closed created:<=2026-11-18` |
| R7T4 | `repo:rayon-rs/rayon "lost update" is:issue is:closed created:<=2026-11-18` |
| R7T5 | `repo:rayon-rs/rayon "data race" is:issue is:closed created:<=2026-11-18` |
| R7T6 | `repo:rayon-rs/rayon spurious is:issue is:closed created:<=2026-11-18` |
| R7T7 | `repo:rayon-rs/rayon hang is:issue is:closed created:<=2026-11-18` |
| R7T8 | `repo:rayon-rs/rayon intermittent is:issue is:closed created:<=2026-11-18` |

The request for R1T1, with the query in shell single quotes (the others
differ only in `q`):

```sh
gh api --method GET search/issues --include \
  -f q='repo:rust-lang/rust label:T-libs race is:issue is:closed created:<=2026-11-18' \
  -F per_page=10 -F page=1 > WORK/search/R1T1.http
```

No `sort`, `order` or other parameter is sent, so GitHub's default
(best-match) sort applies. The 56 requests run once each on 2026-11-18 (UTC),
after "go" and before any result is opened, in the order above, with a pause
of at least 3 seconds between requests so that they stay under the search
rate limit (30 a minute). A small shell script sends them; it creates every
saved file exclusively (`set -o noclobber`), so no response is overwritten.
It is a one-off builder tool kept in WORK, not a tool users run, so it is
not written in Rust.

Saved per query, before any result is investigated:

- `WORK/search/RiTj.request.json`: the exact `q`, `per_page`, `page`, the gh
  command line, the UTC start and end times, and `gh --version`;
- `WORK/search/RiTj.http`: the raw response, status line and headers
  included;
- `WORK/search/RiTj.json`: the body alone, from which the record takes
  `total_count`, `incomplete_results` and the ordered item list (position,
  `html_url`, title, `state`, `state_reason`, `created_at`, `closed_at`,
  labels).

The first 10 items of each response are the pool: all of them, in the
returned order, fewer only if the response has fewer (a query with
`total_count` 0 contributes none, and is recorded). There is no second page,
refill, rerank, synonym or substitute query.

Failures: a response with a non-200 status is not a run. The identical request
is retried, at most twice, after the rate-limit reset if the headers give one;
each attempt is saved under its own name (`RiTj.attempt2.http`, ...). A query
that never returns 200 is recorded as failed, and the search is then disclosed
as incomplete and cannot support a completed-search negative. A 200 response
with `incomplete_results: true` counts as the run; it is recorded and
disclosed, never rerun.

Moved repositories: if an item's `repository_url` names a different
owner/name than the qualifier (GitHub resolved a rename), the record gives
the resolved address. Nothing substitutes another project.

**Repository metadata.** After the 56 searches, and before any result is
opened, the builder reads, for each of the seven repositories,

```sh
gh api --method GET repos/OWNER/REPO > WORK/repos/OWNER__REPO.json
```

recording the lookup time, `full_name`, `default_branch`,
`stargazers_count`, `archived` and `license.spdx_id`. These are descriptive
(for the license and the case study); the bead sets no star or language
filter, and this protocol adds none.

**Duplicates.** The same issue URL in several queries keeps a row in each
query's table, is assessed once, and each row links to the one assessment.

## 2. Reading a result

For each unique result, read, by GET only, and save under
`WORK/issues/OWNER__REPO__N/`:

- `repos/O/R/issues/N` (body, labels, `state_reason`, `closed_at`);
- every page of `repos/O/R/issues/N/comments?per_page=100`;
- every page of `repos/O/R/issues/N/timeline?per_page=100`;
- for each candidate fix: `repos/O/R/pulls/P` (its `merged_at`, body and
  `files`), or `repos/O/R/commits/SHA` (its parents and diff).

Paging through one issue's thread is reading that result, not paginating the
search. **Narrowing:** for a result that fails the scope filter on the
issue's own text (title, body, labels), the comments and timeline may be read
only as far as needed to confirm that; the record says what was read.

The checks are applied in order: the scope filter, then E1, E2, E3, E4, E5.
The record gives the first one that fails, with its reason and evidence;
checks after a failure are marked "not assessed". A result is eligible only
when all five criteria hold.

## 3. The scope filter

A result that fails it is recorded as "not a candidate (scope)", with the
reason. It is judged from the issue (title, body, labels, thread) and, where
the issue leaves it open, the files the fix changes.

- **R1** (`rust-lang/rust`, label `T-libs`). In scope when the issue is about
  the behavior of one of the items the bead lists: `mpsc` and channels,
  `Once`, `OnceLock`, `LazyLock`, `Mutex`, `RwLock`, `Condvar`, `Barrier`,
  `thread::scope`, `park` and `unpark`, or a std type built on atomics (the
  `core::sync::atomic` types, `Arc`, `sync::Weak`). **Narrowing:** the list
  is read as exhaustive, so other items of the thread module (`spawn` and
  `JoinHandle` outside `thread::scope`, `Builder`, `thread_local!`,
  `available_parallelism`) are not in scope. The fix's
  files, where needed, are under `library/std/src/sync/`,
  `library/std/src/thread/`, `library/std/src/sys/sync/` (or the older
  `sys_common` and `sys/*/` lock, condvar, parker and thread files),
  `library/core/src/sync/` or `library/alloc/src/sync.rs`. **Not in scope**:
  compiler, codegen, MIR, LLVM, linker, platform-ABI and toolchain bugs
  (including miscompilations, lints, diagnostics and const evaluation),
  documentation-only issues, and issues about other std modules (`process`,
  `fs`, `io`, `net`, `env`, `alloc` other than `Arc` and `Weak`, the `panic`
  machinery, the test harness).
- **R2** (`crossbeam-rs/crossbeam`). In scope when the issue is about the
  `crossbeam-channel`, `crossbeam-deque`, `crossbeam-epoch`,
  `crossbeam-queue` or `crossbeam-utils` crate (named in the issue, or
  changed by the fix). Not in scope: `crossbeam-skiplist`, the umbrella
  crate's re-exports alone, CI, docs and tooling.
- **R3-R6** (`parking_lot`, `dashmap`, `flume`, `once_cell`). No scope
  filter beyond the query.
- **R7** (`rayon-rs/rayon`). In scope when the issue is about `rayon-core`:
  `join`, `scope`, `spawn`, `ThreadPool` and its builder, the registry,
  sleep, latch or job code (named in the issue, or changed by the fix under
  `rayon-core/`). Not in scope: rayon's parallel-iterator adaptors and
  collections, even when they deadlock through rayon-core, unless the fix
  changes `rayon-core/`.

## 4. Eligibility E1-E3

**E1, a public linked fix.** One of these must exist and be public:

1. a `closed` timeline event whose `commit_id` names a commit that
   `repos/O/R/commits/SHA` returns;
2. a pull request, merged, that is linked from the issue or its timeline
   (`cross-referenced` or `connected` events, or a link in the body or a
   comment) and that either closes the issue by a closing keyword, or is named
   as the fix of this issue by its own text or by a comment in the issue
   thread. "Merged" means `merged_at` is not null, or (for `rust-lang/rust`
   PRs landed through bors or a rollup, which GitHub sometimes shows only as
   closed) its head commit is reachable from the default branch, checked by
   `repos/O/R/compare/HEAD_SHA...DEFAULT_BRANCH` returning `behind_by: 0`;
3. a commit linked from the body or a comment that the thread names as the
   fix ("fixed in abc1234", "fixed by #123", and the like).

**Narrowing:** a mention alone is not a link to a fix. A change that only
ignores, skips, deletes or retries the failing test, only quarantines it in
CI, only documents the behavior, or only adds a test, is not a fix of the bug
and fails E1. If several changes qualify, the fix is the one the thread names;
otherwise the last one merged before the issue closed. An issue closed as
`not_planned` or as a duplicate passes E1 only if it still meets one of 1-3
for this issue.

**E2, the symptom.** **Narrowing:** the issue body, or a comment in its thread
posted before the fix merged, reports a wrong behavior that was observed: an
error, panic or assertion message, a wrong value or output, a hang or timeout,
lost data, a crash, or a failing test run (in CI, production, a
reproduction, or a model checker such as loom or Miri run against the code). A
report that a failure could happen, from reading code alone, with no observed
failure, fails E2. The recorded symptom is a quote.

**E3, concurrency-shaped.** The issue, its thread or the fix names a cause in
the interleaving or timing of two or more concurrent activities (threads,
tasks, futures, signal handlers) over shared state: a race, a lost update, a
lost wakeup, an ordering fault, a deadlock (on locks or on waits such as a
channel `recv`, a `park` or a `Condvar` wait), or a flake whose outcome
depends on relative timing. **Narrowing:** a flake whose named cause is not
concurrency (platform, randomness, a non-concurrent logic error, a compiler
bug) fails E3, and so does one whose cause neither the thread nor the fix
names. A performance or contention report with no wrong result and no hang
fails E3.

## 5. What Tide v0 offers, how the simulator runs it, and E4-E5

This section is written from `design/reference.md` (§§6.8, 6.9, 7.1-7.4, 11)
and `design/03-authority-concurrency.md` (§1.2 CAP-2 and CAP-3, §1.3, §6.4,
§6.5 LOCK-1 to LOCK-6, §9.3), and the std at base `afb17ea`
(`crates/tide-std/src/sync.rs`, `sync/mpsc.rs`, `task.rs`,
`task/scheduler.rs`).

**What v0 offers for the operations these paths use.**

| Operation | In Tide v0 at `afb17ea` |
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
point before acquiring the set and one after release), `reset`, `Weak::upgrade`,
`yield_now` and checkpoints. Between two points a task runs alone. At each
point with several runnable tasks, a seeded PRNG picks one uniformly; the
seed fixes the whole run, and a replay manifest reproduces it. The search is
random sampling, not exhaustive enumeration (unlike loom). A run where no
task can run and no timer is pending ends with a deadlock report; a task that
polls while a timer waits ends, after 1,000,000 points, with a stall report.
A spin loop with no timer pending is never reported; it runs until the seed's
process times out.

**Which orderings it explores.** Only sequentially consistent interleavings
of the model's steps: every step sees every earlier step's writes. It never
reorders a load before a store, never delays a store's visibility, and never
splits a step between two points.

**Mappings (E4).** Each Rust operation on the path is plain computation or
one of these; each mapping used is listed as an adaptation in the record.

| Rust on the path | Tide v0 in the model | Gap listed |
| --- | --- | --- |
| `std::thread::spawn` + `join`; `thread::scope`; `crossbeam_utils::thread::scope` | `Scope::spawn`, `Task::join` in `tasks.scope` | a never-joined thread becomes a child of the trigger's scope |
| a lock (`std`, `parking_lot`, `lock_api`, crossbeam's `Spinlock`) whose guard covers only pure computation | one `Shared::with` per guard | lock fairness: FIFO per cell |
| `RwLock` read and write guards, same condition | one `Shared::with` per guard | readers serialize; the bug must not need two concurrent readers |
| an atomic operation (`load`, `store`, `swap`, `fetch_*`, `compare_exchange`, `compare_exchange_weak`) on an atomic field | one `Shared::with` per operation over the cell holding the structure's atomic fields, under sequential consistency | ordering argument ignored; `compare_exchange_weak` never fails spuriously |
| `thread::park`, `Thread::unpark`, `park_timeout` | a per-thread `sync_channel::<()>(1, tasks)` token: `unpark` is `try_send(())` with `Full` ignored, `park` is `recv()`, `park_timeout(d)` is `recv_deadline(clock.now() + d)` | `park` never returns spuriously |
| `mpsc` (std) | Tide's `mpsc` | as fx-zu4f.6 |
| `thread::sleep`, `Instant::now` | `clock.sleep`, `clock.now` | virtual time |
| `thread::yield_now`, `spin_loop`, crossbeam `Backoff::{spin, snooze}` | `tasks.yield_now()` | no real spin timing |
| an `UnsafeCell` or `MaybeUninit` slot whose every access the path's own synchronization orders | a field of the structure's `Shared` payload, one `with` per source access | a value, never uninitialized memory |
| a raw pointer to a node of the path's own linked structure | an index into a `Vec` of nodes in the same payload; nodes are never freed | no memory reclamation |
| `Arc<T>` sharing data | a borrow into the scope, or Tide's `Arc` | no reference counts |
| anything else | unportable, with the reason | |

**E4, portable to Tide v0.** The builder first identifies the affected code
path at the buggy revision (the fix commit's first parent, or the PR's merge
base): the functions the fix changes, the code they race against, and the
trigger the issue reports. It reads that source only by GET
(`repos/O/R/contents/PATH?ref=SHA`, commit diffs) or from a read-only clone
under `TARGET/upstream/OWNER__REPO` (`git clone --filter=blob:none
--no-checkout`, then `git show SHA:PATH`). Upstream code is never built, run,
tested or installed.

The path is portable when all of these hold:

- Every operation on it is plain computation or one of the mappings above.
- It uses no network, no subprocess and no external crate. Crates of the same
  repository are not external (`crossbeam-utils` for crossbeam's channel,
  `lock_api` and `parking_lot_core` for parking_lot); for `rust-lang/rust`,
  `core`, `alloc` and `std` are one repository, but `sys` code that calls the
  operating system is not portable (below). **Narrowing:** an external
  crate's operation anywhere on the path, including its interleaving points,
  makes it unportable (for example `crossbeam-deque` in `rayon-core`,
  `lock_api` or `hashbrown` in `dashmap`, `spin` or `futures` in `flume`).
  Test scaffolding outside the path that only builds the starting state may
  be replaced by setup, listed as an adaptation.
- **Narrowing:** no `async fn`, `.await`, `Future`, `Waker` or `Context` on
  the path (Tide v0 has none).
- **Narrowing:** no lock guard held across a blocking or effectful operation
  (`park`, `unpark`, a channel operation, a sleep, a spawn or join, I/O) or
  across another lock's acquisition; no reentrant locking. `Shared::with`
  takes a `Pure` closure that cannot suspend, signal or nest, and `with2`
  cannot express nested acquisition in a chosen order, so such a path cannot
  keep its structure.
- **Narrowing:** no `Condvar` wait or notify. Tide v0 has no condition
  variable; its `wait` atomically releases a guard and blocks, and modelling
  it needs a waiter registry built from channels, model code standing in for
  a primitive, which is where a model could add or hide the bug. Condvar
  paths are "not eligible (E4)".
- **Narrowing:** no futex, OS wait queue, `pthread`, `WaitOnAddress`, `SRW`
  lock or other operating-system synchronization call on the path (std's
  `sys` primitives); these are platform behavior (E5) with no v0 operation.
- **Narrowing:** no `unsafe` beyond the two mapped forms (an ordered
  `UnsafeCell`/`MaybeUninit` slot, a pointer into the path's own linked
  structure); no allocation, deallocation or reclamation logic (epoch
  garbage collection, `Box::from_raw`, manual `drop_in_place`) on the path,
  and no bug whose observed wrong behavior is memory unsafety (use after
  free, double free, a segfault, uninitialized reads, a Miri or sanitizer
  report) rather than a wrong value, a lost or duplicated item, a panic the
  source raises, or a hang.
- **Narrowing:** no `thread_local!` destructors, thread exit or process exit
  ordering, signal handlers, stack-size, CPU-count or affinity dependence.
- **Narrowing:** the modeled non-test path is at most 400 source lines at the
  buggy revision, comments and blank lines excluded.

**E5, within what the simulator explores.** Judged for each result that
passes E4, from the issue, its thread and the fix. The record states the
answers to (a)-(d) and the candidate's model gaps, including every atomic
operation modeled as one interleaving point under sequential consistency.
It fails E5 when any of these holds:

- (a) **Weak memory.** The bug needs an ordering weaker than sequential
  consistency to appear: the issue, its thread or the fix names a missing or
  wrong `Ordering`, a fence, a reordering, store buffering, or a weakly
  ordered architecture (Arm, AArch64, POWER, RISC-V) as the cause; or the fix
  changes only `Ordering` arguments, fences or `compiler_fence`; or it is
  reproduced only under weak-memory checking (loom's or Miri's weak-memory
  emulation, a relaxed-memory model checker). A fix that changes orderings
  together with logic passes (a) only if the failing interleaving the issue
  describes is possible under sequential consistency, shown by a schedule in
  the record before any run.
- (b) **A switch inside a step.** The failing interleaving needs another
  thread to run between two operations that the model has inside one step:
  two plain (non-atomic) accesses outside the mapped slot form, a torn
  multi-word read or write, or two accesses within one lock guard.
- (c) **Operating-system and hardware behavior.** It needs real preemption
  timing, a timeout against real time, scheduler fairness or starvation
  (a thread that never runs while others do, priority inversion), a
  spurious wakeup (of `park`, a `Condvar`, a futex or
  `compare_exchange_weak`), futex or kernel semantics, signals,
  `thread_local!` destructors or thread-exit order, `fork`, CPU count, cache
  or false-sharing effects, or a specific operating system's primitive.
- (d) **A hang the simulator cannot report.** The symptom is a hang whose
  faithful model would spin without a pending timer (a livelock), since the
  simulator reports only deadlocks and timer stalls; a hang passes (d) when
  the faithful model's hang is a deadlock (no runnable task) or a stall.

Results that fail E4 are "not eligible (E4)"; results that fail E5 are "not
eligible (E5)". An eligible candidate whose model, once attempted, turns out
to need more than the faithful rule allows (section 6, step 7) is
**unportable**, with the reason. The record counts the three separately.

## 6. The faithful model

For each eligible candidate, in query order (R1T1 position 1 first):

1. **Property first.** Before any model of that candidate is built or run, the
   record gets: the issue's own statement of the expected behavior, quoted,
   with its link (body or comment URL); the property in one sentence; and the
   test's assertion in Rust form, with the trigger it runs. For a hang or
   deadlock symptom the property is "the trigger completes". Then the builder
   appends the record's SHA-256 and the UTC time to `WORK/property-log.txt`.
   The property is never changed after a run. If it cannot be tested as
   written, the candidate is "not caught", with the reason.
2. **Mapping.** The record gives a source-to-model table: each model item
   (function, state, interleaving point) against its source file and lines at
   the buggy revision SHA. It lists every adaptation (a mapping from section
   5) and every gap (source behavior the model leaves out), as the Eon
   studies do.
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

## 7. Runs, the caught rule and replay

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
   candidate whose reported symptom is a hang or a deadlock, the property is
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

## 8. The record, counts and selection

The record is kept as the work goes, in
`research/39-std-concurrency-search.md`. It holds: this protocol verbatim
with its digest; the search date and method; per query, the request,
`total_count`, `incomplete_results` and the ordered rows (up to 10), each
with its outcome and reason; the repository metadata; one assessment per
unique issue; and, for each eligible candidate, its property (written before
its runs), mapping, adaptations, gaps, runs and verdict.

Counts, reported separately: raw results; unique issues; duplicates; not
candidates (scope), by repository; screened (unique issues past the scope
filter); not eligible, by first failing criterion E1-E5; eligible; caught;
not caught; unportable; unresolved. They describe this sample only and are
never presented as a rate.

Every eligible candidate is assessed even after a catch. If several are
caught, the case study uses the one with the shortest buggy model, counted as
the non-blank, non-comment lines of `buggy.rs` before `#[cfg(test)]`; ties go
to the clearest evidence (an issue with an observed failure and a
reproduction, and a fix that names the issue), then to query order. The
others are listed in the record.

If one is caught, the case study goes in
`examples/case-studies/<repo>-<short-name>/` with `buggy.rs`, `fixed.rs` and a
README in the shape of eon-discovery-race: negatives first (Tide found nothing
unaided, the property was supplied, the bug was already fixed upstream, what
the model leaves out), then the issue, the fix commit, the license (from the
repository metadata and its license file), the property with its quote, the
model and its gaps, the seed and replay evidence, and what Rust needs for the
same test (loom, Shuttle, Miri and so on), saying where the project already
tests with loom or a similar tool. The models are written fresh; quotes from
upstream are short and attributed.

If none is caught, the record lands as a documented negative, no case study
is written, and the report tells tide-lead, for fx-qlek.7. An incomplete
search is disclosed as incomplete and is not called a completed negative. No
widening, substitute search or stretched model is used to avoid a negative.

---
