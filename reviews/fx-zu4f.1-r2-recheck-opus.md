Model: claude-opus-5-5 · Date: 2026-11-15

## Verdict

request_changes, on one new blocking defect: the fix for B1 calls #2892
"caught", although its diagnosis is unconfirmed. Every original finding
(B1-B5, N1-N11) is resolved. The counts add up. The fix is one sentence.

Scope: re-checked `aa1b7a2` against `640bd98`, limited to my findings and
the lines they changed. `git diff --check` passes. Read-only, no builds,
nothing written to Eon. The author's account is in report.md, "## Opus r2
fixes".

## Counts

All reconcile.

- **Eon by class** (all / v0):
  - class 1: 6 / 2 (ff1, 8ehhn yes; orb-h1f, g3k, qbf, k0okt no);
  - class 2: 6 / 5 (hhzy, u979, 23y9.8, yx8.14, 7df0 yes; a9g0.1.4 no);
  - class 3: 3 / 3;
  - class 4: 2 / 0.

  That gives 17 / 10.
- **Eon tiers.** F = 8wh7h and pimct (2). M = the other 8. 2 + 8 = 10.
- **Eon shares.** 17/309 = 5.5%, written as 6%; 10/309 = 3.2%, written
  as 3%. Both round correctly.
- **hyper by row.** #2595 A, #2808 A, #2892 A, #3030 M, #2643 M,
  RUSTSEC-2021-0079 M. That makes A 3, M 3, and 6 matches.
- **hyper by class:** 1 + 2 + 0 + 2 + 1 = 6. Non-matches: 68 + 4
  downgraded = 72, and 72 + 6 = 78. 6/78 = 7.7%, written as 8%. Open: #2808
  and #2892, so 2.

## New defects in the changed lines

### R1 (blocking): #2892 is called "caught" while its diagnosis is unconfirmed

`:545-547`, `:327-331`, against `:322` and `:429-430`

- §7 says "Only the 3 tier-A hyper rows may be described as caught". One
  of the three is #2892. But its row (:322) says "Reported diagnosis,
  unconfirmed", and rank 2 drops it (:429-430) because "that issue's
  diagnosis is unconfirmed".
- A bug cannot be "caught" by rejecting a spelling that may not be its
  cause. #2892 is also an open issue, so under the outreach rule it is a
  comment target.
- §1 also says "no row is 'caught' yet" (:75-76), which conflicts with
  §7's licence.

Fix:
- Limit "caught" in §7 to #2595 and #2808.
- Mark #2892 as tier A on the reported diagnosis, and not to be called
  caught until that diagnosis is confirmed.
- Make :75-76 agree, for example "no case study has landed; only tier-A
  rows may be called caught".

The tier counts do not change.

### R2 (non-blocking): the rejection is described wrongly for tokio's timer, as built

`:329-331`, `:438-441`

- The note says `FX0501` rejects "an ambient timer (`tokio::time`)", and
  that a reaper reading "tokio's clock" is rejected under `FX0602` and
  `FX0501`.
- In `crates/tide-admit`, the only tokio entry is `tokio::spawn`, which is
  `FX0602` (`policy.rs:188`). The FX0501 table lists std items such as
  `std::time::Instant::now` (`policy.rs:135`), not tokio.
- The driver takes one `.rs` entry and no `--extern`, so `tokio::time` does
  not resolve and fails with E0433. The conformance case
  `cases/tasks/spawn_reject.rs:8` expects `E0433 FX0602` for
  `tokio::spawn`.

Fix: attribute the ambient-timer rejection to `Instant::now` (`FX0501`)
and to tokio not resolving at all (E0433). Keep `FX0602` for
`tokio::spawn`. Tier A still holds.

### R3 (nit): two small text slips

- `:311` says "its 70 original non-matches". The subagent's own
  non-match rows sum to 68: 70 was the round-1 total, which included 2 of
  this agent's downgrades. My r2 N7 carried the same slip.
- `:373` says "with or without load". eon-8wh7h's record has the
  14-terminal timeout under stress-ng and under load 20 without stress-ng,
  never at no load. Write "with or without stress-ng".

## Original findings

- **B1. Resolved.**
  - Retitled; the tiers A, M and F are defined at :79-85.
  - Each row carries a tier, with counts in §3.1, §4.2 and §7, and the
    rank headings show their tiers.
  - The tier assignments are sound, apart from R1.
- **B2. Resolved.**
  - #2730 and #2787 are downgraded, with the maintainers' words quoted
    accurately and the same reason the Eon exclusions use (:339-346).
  - Recounted consistently, to 6 of 78 with 2 open.
- **B3. Resolved.**
  - The figures match eon-8wh7h's records 01-03: 6992 ms and 7079 ms, the
    test that fails at 3.78 s, and strace on the start wait.
  - "What Tide adds" says Tide would not reject the buggy loop and
    mentions tokio's paused clock.
  - eon-pimct is called a known, filed bug, and the delay is labelled
    chosen, not observed. See R3 for one wording slip.
- **B4. Resolved.**
  - The hyper-util `legacy/pool.rs` claim is accurate, as is the opt-in
    `pool_timer`.
  - The required-`&Clock` contrast is fair. hyper-util is MIT, which I
    confirmed through `gh`.
- **B5. Resolved.**
  - The pool task now owns its connections. `PoolMsg` payloads need only
    be `Send` (§7.3).
  - `recv_deadline` expires idle entries, and cancellation is handled at
    `recv_deadline` with the join at scope exit (§7.2).
  - `&'s Clock` is right. The citation `FX0606, §6.6, §7.4` is correct;
    §7.4 is the `Shared` section, where my r2 wrongly said §8.
  - #2892 is out of rank 2's claims.
  - Nit for the child: with no idle entries the loop needs a plain
    `recv()`, since there is no oldest deadline to wait on.
- **N1. Resolved.** `task/simulation_tests.rs` and
  `simulation_runner.rs` exist, and fx-nkv.8 now "plans" its demo.
- **N2. Resolved.** #2595 is "closed as `not_planned`, tracked in #2857",
  PR #2975 closes #2857, and the loaded wording is gone.
- **N3. Resolved.** The PR #2646 qualifier is quoted exactly.
- **N4. Resolved.**
  - The endpoint is created first, with `sim.world.file` (which exists in
    `sim.rs:106`).
  - The assertion moved to after the join, with the right reason
    (§10.1).
  - The `retire_dead_run` branch is listed under "cannot show", and Eon's
    own test is acknowledged.
- **N5. Resolved.** The fixed 14 × 500 ms worst case makes the buggy run
  fail on every seed. The difference between Eon's fix shape and the
  model's is stated.
- **N6. Resolved.** `tokio::spawn` and `&'s Clock` are both used. See R2
  for the timer half.
- **N7. Resolved.** "In depth" is gone, the shares are lower bounds, and
  the need for a second reader is stated. See R3 for the 68.
- **N8. Resolved.** 23y9.8 is now class 2, tier M, with its invariant.
- **N9. Resolved.** The pimct row labels the inference.
- **N10. Resolved.** The availability caveat names `spawn_isolated`.
- **N11. Resolved.** sqlx is cited as `FX0301`, reference §4.2.

DONE
