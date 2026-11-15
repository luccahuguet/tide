# 33 — Case-study survey: bugs in Eon and hyper that match Tide's feature classes

Research for fx-zu4f.1, 2026-11-15, drafted by a fresh Claude Opus 5.5
research agent (uclaude) with two read-only subagents. None of them opened
`trials/e1-checked/` (E1 protocol §2). No Tide code, builds or Beads writes
belong to this bead.

**Eon publication.** On 2026-11-15 the owner approved public use of case
studies that name Eon bugs ("i dont mind it"; relayed by the coordinator,
2026-11-15). The other Eon rules still hold: Eon is read only through
`git log`/`git show` and `br --no-db`, and no case study is built on code or
text from contributors outside the Yazelix org.

**Scope change.** The owner's direction, relayed on 2026-11-15, asked for
many matching bugs in each picked project rather than single bugs: "itd be
more impressive if our case study had a lot of matching bugs (solved or
not) in the picked project", then "like 10 matches or so". This note
therefore surveys:

- Eon: every bug bead in its tracker, by title, with the likely ones'
  bodies inspected;
- one public project, hyper, as a smaller dated sample.

It keeps three ranked rewrite briefs, and the round-1 single-bug candidates
in §6.

Evidence labels apply to each following paragraph, list or table:

- **[repo evidence]**: Tide's design, README or read-only bead state at
  branch `fx-zu4f.1` (base `2509fa3`).
- **[Eon evidence]**: Eon beads and commits at HEAD
  `4a6f9e48af5c730cf7b145318bb5a0bee1ad5324`, read by this agent.
- **[subagent rows]**: rows a subagent classified by reading each item.
  This agent verified each matching row as §4 states.
- **[web evidence]**: the linked primary source, fetched 2026-11-15 through
  the GitHub API or the RustSec advisory database.
- **[inference]**: classifications, verdicts, rankings and proposed models.

## 1. Bar: feature classes and v0

**[repo evidence]** The five classes come from fx-zu4f.1's Scope:

| Class | Bug | Tide feature |
| --- | --- | --- |
| 1 | a concurrency race, lost update or ordering fault | simulation and replay (reference §6.8) |
| 2 | a violated invariant | contracts and property tests (§10) |
| 3 | a hang, leak or unbounded wait | scopes and deadlines, cancellation, limits, bounded channels (§7.1–§7.3) |
| 4 | an unwanted ambient effect | capability I/O (§6) |
| 5 | a pattern an admission rule rejects | an FX rule (§4.2, §6.4, §6.9, §7) |

**[repo evidence]** Overflow traps (§4.1) are a run-time check, not an
admission rule. A bug they catch is counted under class 2, with the
feature named "overflow traps".

**[repo evidence]** "Expressible in v0" means the affected part can be
modelled with surfaces that `design/reference.md` (the reference) states
and that the README lists as implemented in `tide-std`:

- implemented: structured tasks, `limit`, scope deadlines and cancellation,
  `Shared`, channels with `recv_deadline`, `Clock`, simulated files and TCP
  loopback, `#[sim_test]` with generated parameters, contracts and
  generated contract tests, overflow checks in every profile, and the
  admission codes `FX0301`, `FX0501`, `FX0601`, `FX0602` and `FX0604`;
- not available: simulated HTTP and randomness, replay manifests, faults
  and crash images (README: "remain planned"); `sim.world.proc` (fx-ayh2);
  Unix-domain sockets (fx-ssek); planted durability faults (fx-m0f5); FFI
  (reference §13). In `tide-std`, `Proc` is still only an identity.

**[inference]** A "yes" verdict means a model, never a port of Eon's code.
Unix sockets become TCP loopback, files or channels, and processes become
tasks. fx-dett's rule holds: "a hand-written model reproducing a race only
proves the model". So each brief says what its model cannot show.

**[inference]** A bug counts as a match only when a Tide feature would
plausibly catch or prevent its class. No case study has landed. Only
hyper #2595 and #2808 may be described as caught; #2892 is tier A on its
reported diagnosis only (§4.3). Each v0-expressible match therefore carries a **tier**
saying how Tide engages it:

| Tier | Meaning |
| --- | --- |
| **A** | The Rust form is rejected at admission (for example `FX0501`, `FX0602`). |
| **M** | A sim or property test, on a hand-written model and with a property the test author states, finds it. |
| **F** | Only the fix is writable in Tide's surfaces. Rust can write the same fix, and Tide does not reject the bug. |

"Catches" is reserved for tier A, or for rows whose case study has landed.
Fitting the topic is not enough for a match. For example:

- A timeout chosen too short across a process boundary is not a match.
  Nothing in Tide checks that a client's and a server's bounds agree.
- A loop that ends the whole supervisor through `?` is not a match.
  Tide's scope rule, "the first failure cancels the scope" (§7.1), would
  reproduce it.

## 2. Method

**Eon [Eon evidence].**
- **Population.** All beads of type `bug` in any status
  (`br --no-db list -a -t bug`): **309**, created 2026-08-05 to
  2026-11-15. That is 239 closed, 67 open, 2 deferred and 1 in progress.
- **Pass 1, titles.** This agent read every one of the 309 titles and kept
  the ones that could involve concurrency, ordering, waits, timeouts,
  queues, invariants on state, ambient environment or file access, or
  arithmetic. It set aside rendering, fonts, layout, accessibility, input
  mapping, packaging, macOS build fixes, contract-text fixes, and test
  flakes caused by processes or PTYs.
- **Pass 2, bodies.** This agent inspected 40 beads and classified each by
  hand. A row counts as a match only after this pass. Every one of the 40
  is a **partial read**: the description's opening (650 characters for 30
  beads, 450 for 10) plus the close reason.
  - For eon-8wh7h, eon-10qt, eon-agsni and eon-pimct it also read the
    first 45–120 lines of `br --no-db show`, comments included.
  - For eon-8wh7h, eon-10qt, eon-agsni and eon-8ehhn it read the fix
    commit's message and file list.
  - No bead was read to its last comment.
- **Fix commits.** Found by `git log --grep=<bead id>`. Some beads predate
  the 2026-10-14 merge of the Orbit, Venus and Eon trackers (`6eabe055`),
  and their proof commits are not in Eon's merged history. Those rows give
  the commit the bead cites.
- **Excluded by the Eon rules (4).**
  - eon-awmq.6 and eon-l4e7s belong to fx-dett.
  - eon-g3vdz shares l4e7s's fix commit `283bf76a`.
  - eon-gkeae was reported by a contributor outside the org.
- **Earlier pass.** A round-1 mining subagent proposed ten Eon candidates.
  All ten are among the 40 and were reclassified here.

**hyper.** See §4.1.

## 3. Eon

### 3.1 Counts

**[inference]**

| | Count |
| --- | ---: |
| Bug beads in the tracker | 309 |
| Excluded by the Eon rules | 4 |
| Set aside at the title pass | 265 |
| Bodies inspected (partial reads) and classified | 40 |
| **Eligible matches** | **17** |
| Of those, expressible in v0 | 10 |
| Matching but ineligible: outside contributors' code (eon-2yj90) | 1 |
| Not matching | 22 |

**Eligible matches by class:**

| Class | All | v0 yes |
| --- | ---: | ---: |
| 1, simulation and replay | 6 | 2 |
| 2, contracts | 6 | 5 |
| 3, scopes and deadlines | 3 | 3 |
| 4, capability I/O | 2 | 0 |
| 5, FX rule | 0 | 0 |

**v0-expressible matches by tier:** A 0, M 8, F 2.

**Honest share, as lower bounds.**
- At least 17 of 309 bug beads, 6%, are eligible matches, and at least 10,
  3%, can be modelled in v0.
- These are lower bounds. The title pass set aside 265 beads unread,
  including test flakes, and eon-8ehhn was itself found through a flake.
- One classifier read each project. Before a paper quotes a share, a
  second reader should classify a random sample of rows.
- Of the 40 inspected, 17 are eligible matches and 1 more matches but is
  ineligible.
- The title pass removed mostly rendering, input and platform work, which
  no Tide feature addresses.

### 3.2 Matching bugs

**[Eon evidence]** for the facts in each row; **[inference]** for class,
feature and verdict.

| ID | Status | Fix commit | Class | Tide feature | v0 (tier) | Root cause, from the bead |
| --- | --- | --- | --- | --- | --- | --- |
| eon-8wh7h | closed | `d431fcab` | 3 | scopes and deadlines | yes (F) | A tab close stopped its terminals one by one on the supervisor's only request thread. The 14-terminal close took 7 s, and Venus's 6 s read timed out. |
| eon-pimct | open | — | 3 | scopes and deadlines | yes (F) | Each terminal start still waits synchronously on that thread for Ready and the lease (at `dcaef1c`, per the bead). At the original observation the start polled every 25 ms for up to 5 s, and Venus's Inspect waited 3.8 s behind one start. The bead says "several queued starts can stack up". `TERMINAL_START_TIMEOUT` is now 15 s, after eon-g3vdz. [Inference] So one start slower than 6 s could outlast Venus's timeout, though none has been observed. |
| eon-10qt | closed | `12b1832f` | 3 (and 1) | scopes and deadlines: bounded channels; simulation | yes (M) | Preview replies between frames stopped frames from replacing each other. A full event queue then turned into `Lost`, and nothing retried. |
| eon-scroll-preview-per-frame-hhzy | closed | `777b9410` | 2 | contracts | yes (M) | `preview_pending` suppressed a resend only for the same revision, so one preview per frame went out and several were outstanding. Invariant: at most one outstanding. |
| eon-preview-answer-survives-frames-u979 | open | — | 2 | contracts | yes (M) | Orbit's output queue lets a newer frame replace an unwritten `VerticalPreview` answer, so a request can go unanswered. Invariant: every request gets exactly one answer. |
| ven-preserve-authoritative-orbit-completion-ff1 | closed | Venus `df95e345` (pre-merge) | 1 | simulation and replay | yes (M) | Orbit flushed `Exited` and closed the socket. Venus's writer saw the socket error before its reader drained the buffered message, so the final message was lost. |
| eon-8ehhn | closed | `27b39095` | 1 | simulation and replay | yes (M) | Discovery read a Live record, then the host published its tombstone and removed its socket before discovery's socket check, so discovery failed instead of treating the run as ended. |
| eon-quick-search-parity-23y9.8 | open | — | 2 | contracts | yes (M) | If a later tab commits before a pending t1, that tab's first pane takes terminal-1 and the `eon run` child meant for t1. The user's action order drives this allocator fault, so the invariant is "t1's first pane gets the initial child". |
| eon-agent-sidebar-yx8.14 | closed | `2112ff31` | 2 | contracts | yes (M) | Cached status records were matched by `id` alone, so a record re-created within one 500 ms poll inherited the old seen raise. |
| eon-eon-attention-working-hides-finish-7df0 | closed | `5a862448` | 2 | contracts | yes (M) | Normalization ranked Finished above Working, so an older unseen finish hid a newer Working on the same terminal. |
| orb-h1f | closed | Orbit `25185127` (pre-merge) | 1 | simulation and replay | no | A launcher saw no record. Orbit then published Ready, and the launcher used stale pre-Ready child-signal authority. |
| eon-g3k | closed | `871c9f63` (pre-merge) | 1 | simulation and replay | no | The consumer side of orb-h1f: it inferred the Ready boundary from record observations. |
| eon-converge-supervisor-startup-qbf | closed | `d968d2f` (pre-merge) | 1 | simulation and replay | no | Two launchers starting one generation raced on a check, then remove, of the control socket. |
| eon-k0okt | closed | `4763d295` | 1 | simulation and replay | no | A Close right after a failed stop reached Orbit before Orbit processed the old lease's EOF, and was refused as Busy. |
| eon-agent-parity-a9g0.1.4 | closed | `8ebdd69b` | 2 | contracts | no | After supervisor recovery, popup IDs were reissued, so a saved `u1` named a newer popup (EON-C10). |
| eon-geigu | closed | `f3bc10db` | 4 | capability I/O | no | Terminals inherited the launching agent's session variables, including a messaging token. |
| eon-yxo2y | closed | `3e8202fc` | 4 | capability I/O | no | Terminals inherited agent tool-runner settings, for example `GIT_EDITOR=true`, `NO_COLOR` and a forced locale. |

**[inference] Notes on the v0 verdicts.**
- **Processes, signals and Unix sockets.** The v0 "no" rows for class 1
  turn on Orbit's process lifecycle, signals or Unix-socket accept order
  (fx-ayh2, fx-ssek). A model of them would prove only itself.
- **Environment leaks.** Class 4 Eon leaks happen at child-process spawn,
  and `Proc` has no operations yet.
- **Matching but ineligible: eon-2yj90** (closed, `26105db9`, class 4).
  - Eon's Helix loaded a repository's own `.helix/*.scm` Steel code, which
    can spawn processes, with no trust check.
  - An `Fs` attenuated with `dir(config, Mode::Read)` could not read
    `<workspace>/.helix` (reference §6.3), so the loader's confinement is
    expressible.
  - The loader is Helix and Steel code by authors outside the Yazelix org,
    so the rule against outside code excludes it. It is not counted among
    the eligible matches.

### 3.3 Inspected, not matching (22)

**[inference]**

- **Fault isolation (4: eon-tab-close-kills-supervisor-7iap,
  eon-reap-error-unstopped-hfzr, eon-popup-cancel-failure-ends-supervisor-k9sx,
  eon-wt6l).** One terminal's error ended the whole supervisor through `?`.
  Tide's default cancels a scope on the first failure (§7.1). Returning
  errors as data is a documented idiom, but no check enforces it.
- **Timeout values across a process boundary (3:
  eon-workspace-action-timeout-287o,
  eon-return-committed-workspace-action-result-xfg,
  eon-eon-admission-under-load-dyfd).** One side's bound was shorter than
  the other's.
- **Error policy on a program's own output (2: eon-title-utf8-ends-terminal-atq2,
  eon-frame-budget-text-kzze).** Each was a decode or budget error made
  fatal.
- **Open, cause undiagnosed (4: eon-getih, eon-tsrto,
  eon-quick-search-parity-23y9.11, eon-7nor8).** These may be races, but
  no bead names a mechanism yet.
- **Fork or process lifecycle (2: eon-orbit-ready-claim-flake-3si2,
  eon-failed-child-blocks-creation-ax8y).**
- **Uncoordinated processes (1: eon-agsni).** Supervisors of several Eon
  generations each polled on their own.
- **Same as in Rust, or outside every class (6):**
  - orb-stale-client-readiness-e0f: an `unwrap` on stale state;
  - ven-byte-bound-orbit-server-events-gho: a byte bound, while Tide's
    channels bound count;
  - eon-codex-quota-chip-disappears-oiw: a display-age policy;
  - eon-8jdmw: an upstream format change;
  - eon-ih692: UI hysteresis;
  - eon-dbovl: superseded by k0okt.

## 4. hyper (public sample)

### 4.1 Choice and method

**[inference]** hyper is the chosen public project, for three reasons:

- It is an MIT-licensed Rust HTTP library with a large public tracker:
  1,923 issues, 327 of them labelled `C-bug`, and eight RustSec
  advisories.
- Its server, client and pool code is application-level concurrency:
  shutdown, timeouts, idle reaping, dispatch and parsing. Tide models
  that, unlike runtime internals such as tokio's schedulers and wakers,
  which v0 cannot model.
- It already supplied two round-1 candidates.

**[web evidence]** Sample: every `C-bug` issue created
2021-01-01..2022-12-31 (**74**), plus hyper's RustSec advisories dated in
that window (**4**: RUSTSEC-2021-0020, -0078, -0079 and 2022-0022), so 78
items in all.

**[subagent rows]** A subagent read each item by hand through `gh`: the
issue body, its comments and the closing PR where there was one. It
classified each against §1's classes.

**[web evidence]** This agent verified each matching row's state, title,
labels and fix PR, with its merge commit, through the GitHub API. It
downgraded four of the subagent's matches (below): two at round 1, and
#2730 and #2787 after the Opus review.

### 4.2 Counts

**[inference]**

| | Count |
| --- | ---: |
| Items reviewed (74 issues, 4 advisories) | 78 |
| **Matching a Tide feature class** | **6** |
| Of those, expressible in v0 | 6 |
| Not matching | 72 |

**Matches by class:**
- class 1: 1;
- class 2: 2, RUSTSEC-2021-0079 included through overflow traps;
- class 3: 0;
- class 4: 2;
- class 5: 1.

**By tier:** A 3, M 3, F 0.

**Non-matches by reason [subagent rows]:**

| Reason | Count |
| --- | ---: |
| HTTP semantics or protocol compliance, RUSTSEC-2021-0020 and -0078 included | 16 |
| API, ergonomics, by-design behaviour or misuse | 13 |
| Build, compile or feature flags | 11 |
| h2 internals | 6 |
| Docs or examples | 5 |
| Platform or inherent network races | 4 |
| Unreproduced, or user-side async misuse | 4 |
| Performance | 3 |
| FFI | 3 |
| Unsafe or soundness, RUSTSEC-2022-0022 included | 2 |
| TLS | 1 |
| Downgraded by this agent (below) | 4 |

**Honest share, as a lower bound.** At least 6 of 78 sampled items, 8%,
match, and all 6 can be modelled in v0. One classifier read the sample,
and its 68 original non-matches were not re-verified.

### 4.3 Matching bugs

**[web evidence]** for state, title and fix; **[subagent rows]**, checked,
for the root cause; **[inference]** for class, feature and verdict.

| ID | Status | Fix | Class | Tide feature | v0 (tier) | Root cause |
| --- | --- | --- | --- | --- | --- | --- |
| [#2595](https://github.com/hyperium/hyper/issues/2595) "`runtime` feature is not additive" | closed as `not_planned`, tracked in #2857 | [PR #2975](https://github.com/hyperium/hyper/pull/2975), `23a56005` (closes #2857; removes the `runtime` feature) | 4 | capability I/O | yes (A) | With the documented `runtime` feature on, hyper used tokio's ambient timer, and panicked ("no reactor running") under another executor. |
| [#2808](https://github.com/hyperium/hyper/issues/2808) "client does not clean up idle connections with non-tokio executor" | **open** | none in 0.14. In 1.x the pooled client moved to hyper-util, where idle reaping runs only if the caller sets `pool_timer` ([`legacy/pool.rs`](https://github.com/hyperium/hyper-util/blob/master/src/client/legacy/pool.rs), `spawn_idle_interval` returns early without a timer) | 4 | capability I/O | yes (A) | The pool runs its idle-connection cleaner only when the `runtime` feature pulls in tokio ([issue text](https://github.com/hyperium/hyper/issues/2808)), so idle connections leak under other executors. |
| [#2892](https://github.com/hyperium/hyper/issues/2892) "panicked at dispatch dropped without returning error" | **open** | related [PR #2790](https://github.com/hyperium/hyper/pull/2790), `9fa36382`, turns the panic into an error | 5 | FX rule (`FX0602`, structured tasks) | yes (A) | Reported diagnosis, unconfirmed: the dispatcher is spawned detached onto the ambient runtime, and when a test's runtime is dropped, the pooled client keeps using the dead connection. |
| [#3030](https://github.com/hyperium/hyper/issues/3030) "reproducible panic 'dispatch dropped without returning error'" | closed | PR #2790, `9fa36382` | 1 | simulation and replay | yes (M) | The dispatcher took a request's reply callback and was dropped before answering. Whether it fails depends on when the drop lands. |
| [#2643](https://github.com/hyperium/hyper/issues/2643) "Panic via reqwest when sending large body in debug mode" | closed | [PR #2646](https://github.com/hyperium/hyper/pull/2646), `c88011da` | 2 | contracts | yes (M) | The pool reused a connection that had read a full response while its request body was still flushing, so in release builds the next request's bytes mixed into the old body. Per PR #2646, this "only ever happens if a server replies with a full response before reading the full request, while also choosing to not close that connection". |
| [RUSTSEC-2021-0079](https://rustsec.org/advisories/RUSTSEC-2021-0079) (GHSA-5h46-h7hh-c6x9) | patched in 0.14.10 | [`1068b994`](https://github.com/hyperium/hyper/commit/1068b994df1aec2a0a562224965a80ddced50e60) (master), `efd9a98` (0.14.x) | 2 | overflow traps: a run-time panic in every build (reference §4.1), not admission | yes (M) | `*size *= radix; *size += ..` wrapped, so `f0000000000000003` read as 3: data loss, or smuggling behind a lenient proxy. |

**[inference] Notes.**
- **Tier A for #2595, #2808 and #2892.** Their Rust spellings do not
  admit:
  - an ambient std clock (`Instant::now`, `thread::sleep`) is `FX0501`
    (reference §6.9);
  - `tokio::spawn` is `FX0602` (§7.1);
  - tokio's timer (`tokio::time`) is not an FX0501 case. v0 admits no
    external crates (§13), so it fails as an unresolved path (rustc
    `E0433`).
  #2892 is tier A on its reported diagnosis only.
  A Tide pool must take a `&Clock`, and a required parameter cannot be
  left out, where hyper-util's `pool_timer` is an optional builder
  setting.
- **#2643.** Tide's contracts are erased in release, as Rust's debug
  assertions are (§10). The gain is a stated invariant that generated
  tests exercise, not release-time detection.
- **Downgraded (4).**
  - [#2730](https://github.com/hyperium/hyper/issues/2730), closed by PR
    #3261 (`f4b51300`, 2023), and [#2787](https://github.com/hyperium/hyper/issues/2787),
    still open. A maintainer answered #2787 that graceful shutdown "will
    stop accepting new connections and wait for all existing connections
    to close". On #2730, seanmonstar wrote that it was "currently on
    purpose". hyper's graceful shutdown is a soft signal. In Tide the
    programmer must still choose to cancel or to bound the drain, the same
    reason §1 excludes Eon's timeout bugs, and no Tide mechanism forces it.
  - [#2409](https://github.com/hyperium/hyper/issues/2409): a
    length-mismatch panic that Rust already raises.
  - [#2621](https://github.com/hyperium/hyper/issues/2621): a `Drop` that
    re-takes h2's store lock. It would count only if Tide's rule against
    nesting `Shared` closures (`FX0603`) also covers drop glue, which no
    document establishes, and it is h2 code.

## 5. The strongest three, ranked

**[inference]** The ranking weighs four things:

- how directly a Tide feature, not a test-writer's foresight, catches or
  prevents the bug;
- whether v0 can run both the buggy and the fixed model;
- the model's fidelity to the original mechanism;
- spread across classes. Seeded replay of a lost update already exists
  in `crates/tide-std/src/task/simulation_tests.rs` and
  `conformance/cases/tasks/simulation_runner.rs` (designed in research/16
  §1.5), and fx-nkv.8 plans a demo of it.

All three come from the surveyed projects.

### 1. eon-8wh7h, with open sibling eon-pimct: one slow operation blocks every request (Eon; class 3, tier F)

**Brief.**
- **Eon's record.** eon-8wh7h's record 01 reproduced the bug with
  programs slow to exit on HUP (`iso/slowclose.sh`), with or without
  stress-ng: a 14-terminal close took 6992 ms under IO+CPU stress and 7079 ms
  with "no stress-ng but system load 20 from other builds". Both hit
  Venus's timeout. strace measured the separate start wait, eon-pimct's
  mechanism.
- **Eon's own test.** The fix landed with a real-time Rust red/green test,
  `a_close_stops_its_terminals_together_and_keeps_only_the_unended`, which
  "fails at 3.78 s when the batch is replaced by one-at-a-time stops"
  (record 03).
- **Model.** Model the supervisor at `d431fcab^` in
  `examples/case-studies/eon-close-bound/`:
  - a supervisor task serves `Inspect` and `Close(tab)` requests from one
    channel;
  - each terminal is a task that answers `Stop` after its grace;
  - a Venus task sends `Inspect` every 250 ms and waits with
    `recv_deadline(now + 6 s)` (reference §7.3).
- **Graces.** Use a fixed worst case of 14 terminals × 500 ms = 7 s, the
  record's figure, so the buggy run fails on every seed. Add a generated
  case, with a `requires` bounding its length and values, only as an
  extra.
- **Buggy variant.** Serial stops inside the request handler fail the
  Venus deadline in virtual time (§6.8).
- **Fixed variant.**
  - It spawns one stop per terminal in a `tasks.scope` with
    `set_deadline(now + 5 s)` (§7.1, §7.2), prunes confirmed ends, and
    keeps the rest. A `#[sim_test]` assertion checks that every
    unconfirmed terminal stays in the tab.
  - The shape differs from Eon's fix. Eon writes every Stop first, then
    reads the replies on the same thread under one shared deadline
    (record 02). Concurrent tasks are Tide's idiom for the same bound.
- **What Tide adds.**
  - A seeded harness in virtual time, so the 7 s close runs in
    milliseconds, with the same result every run.
  - A scope deadline that states the fix.
  - Tide would **not** have rejected the buggy loop, and tokio's paused
    clock gives Rust virtual time too.
- **Open sibling.** eon-pimct is a known open bug that Eon already filed.
  - The model can reproduce it with a `Start` that waits synchronously for
    Ready under the current `TERMINAL_START_TIMEOUT` of 15 s (Eon
    `dcaef1c`; the bead's 5 s is historical).
  - It fails only when the model's start delay exceeds 6 s. Eon has
    observed 3.8 s, so the case study must present any such delay as
    chosen, not as observed.
- **What the model cannot show.** Orbit's signals, the Unix socket
  transport, OS scheduling under load, or Orbit's own grace. The claim is
  about the request loop's structure, not Eon's code.
- **Not fx-dett's work.** It is not one of fx-dett's races, and it needs
  no fx-dett surface.

### 2. hyper #2808 and #2595: a pool that reaches for an ambient runtime (hyper; class 4, tier A)

**Brief.**
- **Bugs covered.**
  - #2595: an ambient tokio timer panics under a foreign executor.
  - #2808, still open in 0.14: idle connections are reaped only when an
    ambient tokio is present.
  - #2892 is kept out of the claims. The model has no dispatcher, and that
    issue's diagnosis is unconfirmed.
- **hyper's fix.** In 1.x the pooled client moved to hyper-util's
  `client::legacy`. There, idle reaping runs only when the caller supplies
  `pool_timer` ([`legacy/pool.rs`](https://github.com/hyperium/hyper-util/blob/master/src/client/legacy/pool.rs):
  `spawn_idle_interval` returns early with no timer). The ambient tokio
  dependency is gone, but reaping is opt-in: a client built without the
  timer still never evicts idle connections.
- **The Tide contrast.** A required `&Clock` parameter cannot be left out.
- **Rust-spelled variant.** A reaper started with `tokio::spawn` is
  rejected at admission as `FX0602` (reference §7.1). One that reads
  `Instant::now()` is `FX0501` (§6.9). One that uses tokio's timer fails
  differently: v0 admits no external crates (§13), so `tokio::time` is an
  unresolved path (rustc `E0433`), not an FX0501 diagnostic. The case
  study shows each as it occurs.
- **Tide variant: a pool task that owns its connections.** `Shared`
  cannot hold a `TcpStream`, which is not `Pure` (`FX0606`, reference
  §6.6, §7.4).
  - `Pool::run<'s>(bg: Scope<'s, '_, E>, clock: &'s Clock, rx:
    Receiver<PoolMsg>, ..)` spawns one pool task into the caller's scope
    (§7.1, "A scope passed down"). Alternatively the task owns a clone of
    the `Clock`.
  - `PoolMsg` has two variants: `Checkout { reply: SyncSender<Option<TcpStream>> }`
    and `Checkin(TcpStream)`. Payloads need only be `Send` (§7.3).
  - The task keeps `Vec<(TcpStream, Instant)>` of idle entries. With no
    idle entry it waits on a plain `rx.recv()`. Otherwise it waits with
    `rx.recv_deadline(oldest_idle + idle_timeout)`. On `Timeout` it
    drops expired entries, which closes them.
  - Shutdown is explicit. The owner calls `bg.cancel()` (or sets a scope
    deadline). `recv` or `recv_deadline` then returns `Cancelled`, the task drops
    its connections and returns `Ok`, and scope exit joins it (§7.2).
- **Test.** A `#[sim_test]` over TCP loopback (README `std::net`):
  - a server task accepts connections;
  - the test checks connections out and back in, then lets them sit idle
    past the timeout in virtual time;
  - it asserts that the server reads EOF on each.
- **What the model cannot show.** hyper's async `Executor` and `Timer`
  traits, tokio, h1 dispatch, or that hyper's real pool has no other
  leak. Simulated HTTP is planned, so the pool carries raw TCP
  connections.
- **Attribution.** Cite the issues and PRs. hyper is MIT, and hyper-util
  is MIT.

### 3. eon-8ehhn: discovery races a host's exit (Eon; class 1, tier M)

**Brief.**
- **Source.** The bug and fix are at `27b39095`:
  - Discovery read a run's record and saw it Live.
  - Before its endpoint check, the run's host published a tombstone record
    and removed its socket, so discovery failed with "cannot inspect
    terminal endpoint". That error failed Stop, a final close, or a
    replacement supervisor's start.
  - The fix rereads the record once after a failed endpoint check, and
    reconciles only that same run if it has ended. "No retry, sleep or new
    timeout; ORB-C13's order makes one read enough."
  - Eon's fix landed with its own deterministic test in `sessions.rs`.
    Tide's gain is finding the interleaving by seed, with no seam written
    for it.
- **Model.**
  - Model the record and the endpoint as two files on the simulated `Fs`
    (README `std::fs`; each operation is a scheduling point).
  - Setup creates `endpoint` before `record = Live`, with
    `sim.world.file`, or a host write that comes first. Otherwise seeds
    where discovery checks before the endpoint exists would fail in the
    fixed variant too.
  - A host task later writes `record = Tombstone`, then removes
    `endpoint`, in ORB-C13's order.
  - A discovery task reads `record`, then checks `endpoint` with
    `fs.metadata`.
- **Buggy variant.** A `#[sim_test]` over 100 seeds finds the interleaving
  where the check fails after a Live read. The runner reruns the failing
  seed and checks that its trace is identical (README "Status"). Replay
  manifests themselves are still planned.
- **Fixed variant.** The reread classifies the run as ended in every seed.
  After both tasks join, a test assertion checks that discovery never
  failed for a run whose final record is a tombstone. It is not an
  `#[ensures]`, which cannot see the host's later state (reference
  §10.1).
- **What the model cannot show.**
  - The Unix socket stat.
  - The fix's second branch, a Live record with a dead host
    (`retire_dead_run`), which needs crash faults.
  - The races the commit leaves open: "Record-read, listing-to-lease and
    failed-host races remain".
  - That Orbit really keeps ORB-C13's order. The model assumes it.

**[inference] Alternates, in order:**
1. **hyper RUSTSEC-2021-0079**: overflow traps in release; a pure parser
   with a `u128` reference contract. The trap turns silent smuggling into
   a panic. That is an availability effect unless the case study isolates
   each connection with `spawn_isolated` (reference §7.1).
2. **eon-10qt**: backpressure with a bounded `sync_channel`.
3. **eon-preview-answer-survives-frames-u979**: an open protocol
   invariant, as a contract on a pure queue model.
4. **sqlx RUSTSEC-2024-0363**: a lossy `as` rejected at admission
   (`FX0301`, reference §4.2; this note's §6).

## 6. Round-1 single-bug candidates

**[web evidence]** for sources; **[inference]** for verdicts. These were
examined before the scope change and are kept for the record.

| Candidate | License | Class, feature | v0 | Status here |
| --- | --- | --- | --- | --- |
| [sqlx RUSTSEC-2024-0363](https://rustsec.org/advisories/RUSTSEC-2024-0363.html): `(self.len() - offset - 4) as i32` truncated the Postgres length prefix ([code at 6f29056](https://github.com/launchbadge/sqlx/blob/6f2905695b9606b5f51b40ce10af63ac9e696bb8/sqlx-postgres/src/arguments.rs#L163)). Fixed by [PR #3441](https://github.com/launchbadge/sqlx/pull/3441) (`0aa06763`), in 0.8.1. | MIT OR Apache-2.0 | 5, FX rule (`FX0301`, reference §4.2), with contracts | yes | Third alternate: not a surveyed project |
| [Kubernetes PR #5316](https://github.com/kubernetes/kubernetes/pull/5316): `finishRequest`'s goroutine blocked forever on an unbuffered send after a timeout ("1496 goroutines running when apiserver crashed"). Fix: capacity-1 channels (`86f97c82`). | Apache-2.0 | 3, scopes and deadlines (`FX0602`, §7.1–§7.3) | yes | Dropped: Go, and not a surveyed project |
| [rust-lang/rust#27970](https://github.com/rust-lang/rust/issues/27970): `env::set_var` races with C `getenv`. [PR #124636](https://github.com/rust-lang/rust/pull/124636) made it `unsafe` in edition 2024. | MIT OR Apache-2.0 | 4/5, capability I/O (`FX0501`) | rejection only (needs FFI) | Rejected |
| [hyper #2457](https://github.com/hyperium/hyper/issues/2457): no header read timeout (Slowloris). Added by [PR #2675](https://github.com/hyperium/hyper/pull/2675), a `feat`. | MIT | 3, scopes and deadlines | yes | Outside §4's sample: not labelled `C-bug`. It is a feature, not a defect |

## 7. Outcome against Acceptance

**[inference]**

- **Eon.** 309 bug beads were surveyed. 40 had their bodies inspected
  (partial reads), giving at least 17 eligible matches, 10 of them
  expressible in v0 (tiers A 0, M 8, F 2), plus 1 ineligible match.
- **hyper.** 78 items were sampled (2021–2022 `C-bug` issues and
  advisories), giving at least 6 matches, all 6 expressible in v0 (tiers
  A 3, M 3, F 0), 2 of them still open (#2808, #2892).
- **What "catches" covers.** No case study has landed. Only hyper #2595
  and #2808 may be described as caught, because Tide rejects their Rust
  spelling at admission. #2892 is tier A on its reported, unconfirmed
  diagnosis only. The M and F rows are matches, not catches.
- **Ranked three.** Each comes from a surveyed project and has a source, a
  license and a v0 verdict. Ranks 1 and 3 are from Eon, so the blocked
  path in fx-zu4f.1's Acceptance does not apply.
- **Licenses.** Eon's root `LICENSE` is the Apache License 2.0
  [Eon evidence]. hyper is MIT ([repository](https://github.com/hyperium/hyper)).

**Open point.** None of the ranked Eon fixes carries an `Issue:` trailer,
which the Eon rules prefer but do not require. Only two Eon bug fixes have
one, eon-jhn2w and eon-6afbi, and neither matches. The ranked beads hold
their proof in their records and named tests. The recommendation is to
accept them.
