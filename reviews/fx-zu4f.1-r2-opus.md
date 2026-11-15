Model: claude-opus-5-5 · Date: 2026-11-15

## Verdict

request_changes

## Scope

Read-only review round (docs-only, GOVERNANCE.md Merge Rule) of
`/home/lucca/pjs/lab/tide/.worktrees/fx-zu4f.1`, branch `fx-zu4f.1` at
`640bd98`, which adds only `research/33-case-study-candidates.md` (485
lines). Line numbers below are in that file.

Read: AGENTS.md; `br --no-db show fx-zu4f.1` and `fx-zu4f` (Scope,
Acceptance, owner directions, eon-overseer's terms, the outreach rule);
design/reference.md §§4.1-4.2, 6.6, 6.8-6.9, 7.1-7.3, 8 (`Shared`), 10;
README "Status" and the tide-std surface list; the Codex review and
re-check. I checked v0-as-built claims against `crates/` by grep only
(`tide-admit/src/policy.rs`, `walk/habits.rs`, `walk/authority.rs`,
`tide-std/src/{fs.rs,time.rs,task.rs}`).

Spot checks Codex did not name (all read-only):
- Eon, `br --no-db show` in ~/pjs/eon/eon (HEAD now `2c22b38`; the note's
  `4a6f9e48` is an ancestor): eon-8wh7h (records 01-03), eon-pimct,
  eon-10qt, eon-scroll-preview-per-frame-hhzy,
  eon-preview-answer-survives-frames-u979, eon-quick-search-parity-23y9.8,
  eon-8ehhn, eon-agent-sidebar-yx8.14,
  eon-eon-attention-working-hides-finish-7df0, eon-geigu, eon-k0okt,
  eon-yxo2y. Status and root cause match the rows, with the exceptions
  below. `git log -1` on 14 cited fix commits: all exist, all authored by
  the owner's account, so the outside-contributor term holds;
  `871c9f63` and `d968d2f` are absent, as the note's pre-merge rule says.
- hyper, via `gh api`: issue states and reasons for #2595, #2643, #2730,
  #2787, #2808, #2892, #3030, #2409, #2621 and #2457; PRs #2975, #2790,
  #3261, #2646 and #2675; the issue comments on #2595, #2730, #2787,
  #2808, #2892 and #3030; search counts (1,923 issues, 327 `C-bug`, 74
  `C-bug` in 2021-2022: all confirmed); hyper's `src/client/` tree and
  hyper-util's `src/client/legacy/pool.rs`.

No files changed, nothing built, no trials/e1-checked/ access, no writes to
Eon or the repository. No credential-like strings appear here.

## Blocking

### B1. The title says Tide "catches" these bugs, but the note's own bar is "plausibly catch or prevent", and no row has been demonstrated

`research/33-case-study-candidates.md:1`, with `:73-75`, `:129`, `:261`

**Evidence.** The title, "bugs in Eon and hyper that Tide's features
catch", is the sentence most likely to be quoted. The bar at :73 counts a
match when a feature "would plausibly catch or prevent its class". That
lumps three different things together, and v0 as built supports them to
different degrees:
- (a) **Unwritable at admission.** The Rust form is rejected (`FX0501` for
  `Instant::now`, `policy.rs:135`; `FX0602` for `thread::spawn` and
  `tokio::spawn`). This fits hyper #2595, #2808 and #2892.
- (b) **Found by a Tide test feature, but only on a hand-written model and
  a test that encodes the expected property.** This covers the sim and
  contract rows: eon-8ehhn, hhzy, u979, yx8.14, 7df0 and #2643.
- (c) **Only the fix is expressible with Tide surfaces.** Rust can say the
  same, with a timeout or join. This covers eon-8wh7h, #2730 and #2787.

No case study exists yet, so nothing is "caught" today. The counts 17 and 8
read as "Tide catches 17 Eon bugs", which a hostile reader will refute by
pointing to rows in tier (c).

**Fix.**
- Retitle, for example "Case-study survey: bugs in Eon and hyper that
  match Tide's feature classes".
- Add a column to §3.2 and §4.3 saying how Tide engages each row: rejected
  at admission, found by a sim or property test of a model, or fix
  expressible.
- Give the counts per tier in §3.1, §4.2 and §7.
- Keep "catches" for rows in tier (a), or for rows whose case study has
  landed.

### B2. hyper #2730 and #2787 fail the note's own match bar, which the Eon exclusions apply strictly

`:299-300`, `:305-308`, with `:77-78`, `:261-266`, `:285`, `:474`

**Evidence.**
- The note itself says "Tide does not force a drain to have a deadline"
  (:308). That is the same reason it excludes Eon's timeout bugs at :77-78
  ("Nothing in Tide checks that a client's and a server's bounds agree").
- **#2787.** A maintainer (davidpdrsn, MEMBER) answered that graceful
  shutdown "will stop accepting new connections and wait for all existing
  connections to close", and suggested wrapping the stream. In Tide the
  programmer must likewise choose to cancel or bound the drain. The row
  calls it an open bug, with Status **open** and Fix "none", but leaves
  out the maintainers' position. Under the outreach rule (fx-zu4f), open
  issues are the only comment targets, so this row invites a post on a
  thread whose maintainers consider the behaviour expected.
- **#2730.** seanmonstar said "currently on purpose. Whether it _should_
  be is a fair question". The bug was then fixed in 2023 (PR #3261, label
  `B-breaking-change`), so it was not simply "by design". hyper's
  graceful shutdown is a soft signal, not a cancel. Tide has no separate
  graceful signal, so a Tide port that forgot to check a shutdown flag in
  the initial state would have the same bug. Level-triggered cancellation
  (§7.2) helps only if graceful shutdown is implemented as cancellation,
  and that changes the semantics.

**Fix.** Do one of the following:
- name a concrete Tide mechanism, not a programmer's choice, that would
  have prevented each bug;
- or move both into a "fix expressible" tier (B1), or into the
  non-matches.

Then recount: §4.2 becomes 6 of 78 if both move, the "still open" count at
:474 drops from 3 to 2, and the "5 since fixed" figure the outreach rule
cites changes. Whatever stays, add the maintainers' position to the #2787
row itself, because rows get quoted without their notes.

### B3. Rank 1 misstates Eon's reproduction and does not say what Tide adds

`:347`, `:344-346`, `:356-357`

**Evidence.**
- :347 says "Eon needed stress-ng and strace to reproduce it". The record
  in eon-8wh7h (record 01) contradicts this:
  - a 14-terminal close with "no stress-ng but system load 20 from other
    builds" took 7079 ms and hit the same TIMEOUT;
  - the driver was programs slow to exit on HUP (`iso/slowclose.sh`), not
    stress;
  - strace measured the start wait, which is eon-pimct's mechanism, not
    the serial close.
- Record 03 shows that Eon's fix landed with a Rust red/green test,
  `a_close_stops_its_terminals_together_and_keeps_only_the_unended`, which
  "fails at 3.78 s when the batch is replaced by one-at-a-time stops". So
  the sentence is unfair to Eon, and it inflates Tide's contrast.
- No Tide rule rejects serial stops in a request handler. The buggy
  variant fails only because the test models Venus's 6 s deadline. What
  Tide adds is virtual time: a fast, deterministic run with no real
  sleeps. tokio's paused clock gives Rust the same, a point the outreach
  rule itself names.
- :356-357 says the case study can "report [eon-pimct] as found". Eon
  filed it as a known follow-up, and the model fails only because its
  author picks a start delay above 6 s. No such start has been observed;
  the record shows 3.8 s.

**Fix.**
- Replace :347 with what the record shows: reproduced with slow-exiting
  terminals, under load or not, and guarded in Eon by a real-time Rust
  regression test.
- Add a "What Tide adds" bullet: a virtual-time, seeded harness, so the
  7 s close runs in milliseconds, deterministically; and a scope deadline
  that states the fix. Also state that Tide would not have rejected the
  buggy loop.
- Reword the eon-pimct sentence as "a known open bug that the model
  reproduces".

### B4. The claim about hyper 1.x and #2808 is inaccurate

`:296`, `:376-380`

**Evidence.**
- hyper 1.x has no pooled client. Its `src/client/` holds only `conn`,
  `dispatch.rs`, `mod.rs` and `tests.rs`; the pool moved to hyper-util's
  `client::legacy`.
- There, `spawn_idle_interval` returns early when no timer is set
  (`hyper-util/src/client/legacy/pool.rs:437-441`): `let timer = if let
  Some(timer) = self.timer.clone() { timer } else { return; };`. A client
  built without `pool_timer` still never evicts idle connections
  proactively.
- So "hyper 1.x takes a caller-supplied `rt::Timer`" (:296), and the claim
  that this "removes the ambient runtime dependency behind ... #2808"
  (:379), are half true. The tokio dependency is gone, but the reaper is
  now opt-in, and forgetting the timer reproduces #2808's symptom on any
  executor. The links point at hyper's `rt/timer.rs`, not at the pool
  that #2808 is about.

**Fix.** Say that the pooled client moved to hyper-util, where idle reaping
runs only when the caller supplies `pool_timer`, and link hyper-util's
`pool.rs`. This also sharpens the Tide contrast: a required `&Clock`
parameter cannot be left out, while an optional builder setting can.

### B5. A rewrite child cannot start the rank-2 brief as written

`:387-406`, `:372-373`, `:384-386`

**Evidence.**
- (a) **The pool cannot be shared state.** The model shares "checkout and
  checkin of TCP connections" between callers and a separate reaper task
  that sleeps on `clock.sleep`. In v0, state shared between tasks lives in
  `Shared<T>`, whose payload must be `Send + Pure` (reference §8, the
  `Shared` section: `FX0606`, "a cell never holds a capability, channel
  end, or another `Shared`"). A `TcpStream` is a handle derived from `Net`,
  which §6.6 lists as not `Pure`. README says the driver checks `Pure` for
  `Shared`. So `Shared<Vec<TcpStream>>`, or any pool cell that holds the
  streams, is rejected. The design works only as a pool task that owns the
  connections, serves checkout and checkin over a channel, and expires
  idle entries with `recv_deadline` on its request channel. That changes
  the shutdown story at :401-405: the pool loop sees `Cancelled` on
  `recv_deadline`, not on a separate reaper's `clock.sleep`.
- (b) **#2892 is not modelled.** The brief lists #2892 in the cluster and
  says "No task outlives its owner, which is #2892's failure" (:406). But
  the model has no dispatcher task: only checkout and checkin, an idle
  timeout and a reaper. The child would have nothing that shows #2892.
  The diagnosis is also unconfirmed (:374-375), and the issue is open,
  which makes it an outreach target.

**Fix.**
- Rewrite the model as an owner-task pool. Name the channel protocol, the
  `recv_deadline` idle expiry and the cancel path.
- Either add a per-connection dispatcher task spawned into `bg`, with a
  test that a request after the scope ends gets an error, not a panic, or
  say explicitly that the case study covers #2595 and #2808 only, and
  keep #2892 out of the claims.

## Non-blocking

1. **`:328-329`: fx-nkv.8 "already covers" seeded replay of a lost
   update.**
   - Evidence: `br --no-db show fx-nkv.8` is OPEN, and its scope does not
     mention a lost update. The lost-update replay exists today in
     `crates/tide-std/src/task/simulation_tests.rs` and
     `conformance/cases/tasks/simulation_runner.rs`, and it is designed in
     research/16 §1.5.
   - Fix: cite those, or say "fx-nkv.8 plans to".

2. **`:295`: hyper #2595's status.**
   - Evidence: GitHub shows `closed` with reason `not_planned`
     (2022-06-15). seanmonstar wrote "Tracking this in #2857", and PR
     #2975's body says "Closes #2857". The row reads as if PR #2975 fixed
     it directly. This also bears on the outreach rule's "5 since fixed".
   - Fix: write "closed as tracked in #2857; PR #2975 (closing #2857)
     removed the feature". Drop "behind the caller's back": the
     `runtime` feature was documented behaviour.

3. **`:301`: hyper #2643 is missing its practical scope.**
   - Evidence: PR #2646 says this "only ever happens if a server replies
     with a full response before reading the full request, while also
     choosing to not close that connection".
   - Fix: add that qualifier, so the release-build intermixing does not
     read as a common failure.

4. **`:426-439`: the rank-3 model needs two fixes before a child can
   start it.**
   - Setup: the endpoint must exist before `record = Live` (via
     `sim.world.file`, or a host write that precedes Live). Otherwise
     seeds where discovery checks before the endpoint exists fail in the
     fixed variant too, since the reread still sees Live.
   - `#[ensures]`: a postcondition on discovery cannot refer to the
     host's "final record" (reference §10.1: checked at each return).
     Make that an assertion in the `#[sim_test]` after both tasks join.
   - Also list under "cannot show" the fix's second branch (the same Live
     record with a dead host, `retire_dead_run`). It needs crash faults,
     which v0 lacks.
   - Fairness: note that Eon's fix landed with a deterministic seam test.
     Tide's gain is finding the interleaving without the seam.

5. **`:340-351`: the rank-1 harness and fix shape.**
   - A generated `Vec<u16>` has variable length and range, so it cannot
     simply be "14 terminals, up to 500 ms". Uniform draws in 0-500 ms
     average about 3.5 s over 14 terminals and rarely pass 6 s, so
     "fails deterministically" needs a fixed worst case (14 × 500 ms =
     7 s, the record's figure), alongside any generated case.
   - Eon's fix writes every Stop first, then reads the replies under one
     shared deadline on the same thread (record 02). The Tide model
     spawns concurrent tasks instead. Say the shapes differ.

6. **`:392-397`: the rank-2 Rust-spelled variant and signature.**
   - hyper spawns through tokio, so mirror it with `tokio::spawn`, which
     is also `FX0602` (reference §7.1; `walk/authority.rs:435`), rather
     than `thread::spawn`.
   - `clock: &Clock` must be `&'s Clock` for a child spawned into `bg`
     to borrow it (§7.1, "A scope passed down"), or the child must own a
     clone.

7. **`:20`, `:89-103`, `:144-150`: the method under a hostile reader.**
   - "Eon in depth" sits beside a titles-only pass over 265 beads and
     reads of 450-650 characters for the other 40.
   - The shares are lower bounds: the title pass set aside unread test
     flakes, and eon-8ehhn was itself found through a flake.
   - One classifier per project, and hyper's 70 non-matches were not
     re-verified.
   - Fix: phrase shares as "at least", drop "in depth", and before the
     paper quotes a share, have a second reader classify a random sample
     of rows.

8. **`:166`: the class of eon-quick-search-parity-23y9.8.**
   - The bead describes an allocator ordering fault driven by user
     action: t2 commits first, and `first_reserved` picks the first
     consuming pane. Its proof is a plain integration test.
   - Fix: class 1 (simulation) is a reach. Consider class 2, with the
     invariant "t1's first pane gets the initial child", or justify why
     a seeded scheduler adds anything.

9. **`:160`: the eon-pimct row mixes inference into evidence.** "So one
   slow start can by itself outlast Venus's 6 s timeout" is an inference
   inside an [Eon evidence] row. The bead says "several queued starts can
   stack up". Label it as inference.

10. **`:447`: the RUSTSEC-2021-0079 alternate.** Say that the trap turns
    silent smuggling into a panic, which is an availability effect unless
    the case study uses `spawn_isolated`.

11. **`:452` vs `:461`: section references for sqlx.** sqlx's lossy `as`
    is cited as "(§6)" at :452 and as "§4.2" at :461. §4.2 is right.

## Reasoning

- **Mechanics.** The arithmetic holds: 309 = 4 + 265 + 40; 40 = 17 + 1 +
  22; the class tables sum to 17 and 10; hyper's subtotals sum to 70 + 8 =
  78; the GitHub counts reproduce. Every Eon row I checked matches its
  bead's status and mechanism. Every Eon fix commit cited is the owner's.
  The Eon rules hold: the fx-dett exclusions, the outside-code exclusion,
  and no `Issue:` trailers required. The Acceptance letter is met: 5 or
  more candidates, 3 chosen, two from Eon, each with a source, a license
  and a v0 verdict.
- **Where it fails.** The note will feed a paper, a launch thread and, by
  the outreach rule, comments on open hyper issues. Its weak points are
  the claims those uses would quote:
  - the title's "catch" (B1);
  - two hyper matches that the note's own bar excludes, one of them an
    open issue its maintainers consider expected behaviour (B2);
  - a rank-1 contrast with Eon that the bead's own record contradicts
    (B3);
  - a factual error about where hyper 1.x fixed the pool (B4).
- **Rank 2.** It also leads its rewrite child into an `FX0606` rejection,
  and claims an issue its model does not contain (B5).
- **Size of the fixes.** All five are text changes to the research note,
  plus a recount for B2. None needs a new survey or code.
- **Not checked.** I did not reread every Eon row that Codex read. I did
  not re-run the 309-bead title pass or rerun hyper's 70 non-match
  classifications. I verified hyper-util's pool on its current default
  branch, not at a pinned tag.

DONE
