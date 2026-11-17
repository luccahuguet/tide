Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 7632f00

# fx-zu4f.2 round 2: Opus converge review (reports only)

Reviewer: a fresh Claude Opus 5.5 agent, read-only. I never opened `trials/e1-checked/`. I read Eon
only through `git log/show/branch/ls-tree/grep` on its refs and `br --no-db show` for eon-8wh7h and
eon-pimct. I changed nothing in Eon or in the candidate, ran no builds or tests, and started no
subagents.

Read: AGENTS.md, GOVERNANCE.md (Merge Rule), `br --no-db show` for fx-zu4f.2 (with comments),
fx-zu4f and fx-qlek (the A1 framing), research/33 §1 and §5, design/reference.md §6.8 and §7.2–§7.3,
`git diff main...7632f00`, the runner as a whole at 7632f00 and its `support/admitted.rs::package`,
the builder's report.md, and r1-codex.md.

## Verdict

**request_changes**, confidence high. One blocking finding. Its fix is text-only (it adds the source
and a caveat, and changes no example's meaning), so under the Merge Rule it needs no re-check. The
model, the tests, and the runner change are sound.

## Blocking findings

### B1. The 15 s start timeout is cited as Eon's but exists only on Eon's `edge` branch, and the open_start failure depends on it

- **Where:** `examples/case-studies/eon-slow-close/README.md:37-38` ("up to Eon's 15 s
  `TERMINAL_START_TIMEOUT`"), `README.md:44` ("Both commits are on Eon's public `main`"),
  `README.md:68-70`, and `open_start.rs:4-6,15` ("as Eon still runs them", "(Eon's
  `TERMINAL_START_TIMEOUT`)").
- **Evidence:**
  - Eon's `origin/main` is at `d41f99ea` ("Release v0.1.16", 2026-11-05). On that branch,
    `runtime/crates/eon-workspace-protocol/src/lib.rs` has no `TERMINAL_START_TIMEOUT`. The start
    wait there is `SESSION_START_TIMEOUT = 5 s` (`runtime/crates/eon-runtime/src/supervisor.rs:51`).
  - The 15 s constant was added in `283bf76a` ("…give starts more time (eon-g3vdz, eon-l4e7s)",
    2026-11-11). eon-pimct cites it at `dcaef1c`. Both commits are on `edge` only (`git branch -a
    --contains dcaef1c` lists `origin/edge` and no `main`).
- **Failing case:** a launch reader checks the case study against Eon's public `main`, which the
  README points them to at line 44. They find no `TERMINAL_START_TIMEOUT`, and a 5 s start bound.
  With `START_TIMEOUT = 5 s`, open_start's single held Ready blocks the loop for only 5 s. The
  Inspect sent at t = 0 is answered at 5 s, inside Venus's 6 s, so
  `inspect_is_answered_while_a_terminal_starts` would pass. The declared failure therefore depends
  on an edge-only value that the README attributes to "Eon" with no source.
  - This does not make the model wrong. eon-pimct's own falsifier ("before that start is released
    or times out") fails under either bound.
  - It does make one launch-evidence statement unsourced, and checkable against `main` to a
    different number. A1 forbids that kind of overclaim.
- **Minimum fix (text only):**
  1. At `README.md:37-38`, source the figure. For example: "up to the 15 s `TERMINAL_START_TIMEOUT`
     on Eon's development branch `edge` (commit `283bf76a`; the v0.1.16 release on `main` waits up
     to 5 s)".
  2. At `README.md:68-72`, add one sentence: under `main`'s 5 s bound, one held start stays under
     Venus's 6 s, which is why eon-pimct names several queued starts.
  3. Make `open_start.rs:15` (and "as Eon still runs them" at :4-5) say `edge`.
  4. `README.md:44` can stay as it is: it is about `d431fcab` and `1766afd1`, which are on `main`.
     But the License and source paragraph (`README.md:138-139`) should add `283bf76a`/`dcaef1c` to
     what the figures come from.

## Non-blocking findings

None of these is required. Each is a wording or simplicity point.

1. **README.md:7-10, the lead's "What Tide adds".**
   - research/33 §5 puts "tokio's paused clock gives Rust virtual time too" right beside the
     negative. The lead names virtual time and "no clock injection" as Tide's additions, and
     mentions tokio only in the last section.
   - Suggest adding to the lead: "(async Rust gets the same from tokio's paused clock; Eon's
     thread-based supervisor would need a clock it is handed)". The model's `serve` does take a
     `&Clock`, so "needs no clock injection written for it" reads better as "needs no fake clock
     written for it".
   - "Gives the same result on every seed" is true but adds nothing here. The model has no race,
     and the builder's report says the outcome does not depend on the interleaving. "Is
     deterministic" says the same without suggesting that seed exploration found something.
2. **README.md:113-118, "a second wait cannot start a fresh five seconds".** This is true only for
   waits inside the one scope (§7.2: `set_deadline` lowers, nested scopes take the earliest). A
   reconcile written as a second scope after the first gets its own five seconds. That is the shape
   of Eon's record-04 defect: a pass after a failed batch. Eon's fix was the same idea in Rust: one
   shared deadline across the pre-check, stop and reconcile. Suggest: "…cannot start a fresh five
   seconds while it sits inside that scope". That way the record-04 sentence does not read as
   "Tide would have prevented it".
3. **README.md:119-121, "Keep the unended by construction".** What keeps them is the fix's
   `filter(!ended)`, not a Tide property. A Rust wait with a timeout also returns an error. Suggest
   dropping "by construction", or reframing: "cancellation arrives as an ordinary `Err` at each
   wait, so the fix keeps a terminal exactly when its wait did not return `Ok`".
4. **README.md:69-70, "so the loop waits the full 15 s".** In the test, the Inspect fails at 6 s,
   and `held` is dropped as the body unwinds, so the loop never waits 15 s. The sentence describes
   the input rather than the run. Wording such as "a start whose Ready never comes, which would hold
   the loop up to 15 s" avoids the mismatch.
5. **Simplicity, open_start.rs.**
   - `CLOSE_BOUND`, `close()`, `Request::Close`, the `Tasks` parameter of `serve` and the
     `TaskError` import are never exercised by its only test. About 30 lines could go.
   - Keeping them does buy a clean `diff fixed.rs open_start.rs` that shows only the Start
     addition, and the README frames the file that way. Either choice is defensible. If they stay,
     nothing needs to change.
6. **Simplicity, buggy.rs:45-53.** Without a deadline, `clock.sleep` never errs, so the `kept`
   branch of the serial `close` is dead in this model. It mirrors Eon's "keeps any it could not
   confirm", which is a reason to keep it. Optional.
7. **Runner test, examples.rs `case_studies_is_a_group_of_examples_one_level_deep`.** It does not
   cover "`examples/` holds no example". That is the one top-level rule whose message changed from
   main ("holds no variant"). It is equivalent, since every example needs a variant, so this is
   minor.
8. **Scope note.** The bead's Scope lists "changes to the runner" as out of scope. tide-lead's
   2026-11-17 21:08 comment records the decision to extend it, with a reason. I agree with the
   decision. I note it only so the landing record cites that comment.

## Reasoning, by lens

### 1. Claims

The negative comes first and is plain: README.md:3 "Tide's admission would not have rejected this
bug", then "What catches the bug is a test" and "Rust can write the same test". The "Would not"
list (lines 98-104) restates it. Nothing says Tide prevents the bug. What the simulator catches is
described exactly: a test whose property someone has to state (line 102-103), run in virtual time.
This matches research/33 §5 (tier F, "Tide would not have rejected the buggy loop") and A1.

I checked every figure and quote against Eon:

| Claim | Source | Result |
| --- | --- | --- |
| one loop thread; Venus Inspect every 250 ms, 6 s timeout | eon-8wh7h record 01 and body | true |
| Orbit's 500 ms grace | `d431fcab` message (ORB-C12) | true |
| 14 terminals, 6992 ms, Inspect TIMEOUT at 6011 ms, "Venus's exact failure" | record 01 table | true |
| `d431fcab` subject and date 2026-11-04; the quoted lines | `git show d431fcab` | verbatim; `[...]` elides one parenthetical |
| `1766afd1` is "say plainly when Eon is slow" | `git log` | true |
| both on public `main` | `git branch -a --contains` | true (`origin/main`) |
| Eon's test name; it fails at 3.78 s on the serial loop | record 03 | true |
| first fix reconciled twice with a fresh 5 s | record 04 | true |
| eon-pimct is open; 3.8 s (3758 ms) observed; the falsifier quote | eon-pimct | true, verbatim |
| 15 s `TERMINAL_START_TIMEOUT` | eon-pimct at `dcaef1c` | edge only (B1) |
| Eon and Tide are Apache-2.0 | Eon `LICENSE`; Tide `Cargo.toml` and `LICENSE` | true |
| D8, §7.1–§7.3, §6.8 | 01-design.md:592; reference.md | correct sections |

The timing statements in the README (7 s serial; first Inspect times out at 6.25 s; 500 ms
together; 4.75 s with one unended terminal) all follow from the code, as computed below.

### 2. Test validity

- **buggy.rs.** Close is queued at t = 0, before Venus's first Inspect at t = 0.25, so the loop
  sleeps 14 × 0.5 = 7 s first. `recv_deadline(0.25 + 6)` = 6.25 s < 7 s, so it times out. The only
  path to the declared message is a `Timeout` on the first Inspect sent 250 ms in, and the
  `//@ reports:` lines pin that text, `Timeout` included, plus `0 passed; 20 failed (20 seeds)`. A
  `Disconnected`, `Cancelled`, a deadlock report, or a later Inspect would not match. The failure
  is the modelled stall: 7 s, the record's 6992 ms, against Venus's 6 s.
- **fixed.rs.** The same `close_while_inspecting` and the same `slow_tab()` give the same load.
  `inspect_is_answered_while_a_tab_closes` also asserts that the close kept `[]`. A serial loop
  under the 5 s bound would keep 4 terminals and fail, so the test checks concurrency, not just
  the bound. `a_close_keeps_only_the_unended` (kept `== [4]`) fails a close that drops everything.
  Neither test can pass vacuously. Per §7.2, the cancelled child's `sleep` returns `Err(Cancelled)`
  and is mapped to `Ok(false)`. Joins are not checkpoints, so the scope returns `Ok` with the
  partial results, and the `expect` holds.
- **open_start.rs.** The Start is queued before the Inspect by the same task. Ready is held, the
  loop waits until 15 s, and the Inspect deadline at 6 s fails with `Timeout`, as the reports pin.
  The test is stricter than eon-pimct's falsifier (6 s rather than "before release or 15 s"), so it
  fails whenever the falsifier fails. Only B1's sourcing is at issue.
- **Seeds.** The outcome does not depend on the seed: each queue order is fixed by one sender, and
  virtual time is deterministic (§6.8). `seeds = 20` shows that.
- **Fidelity.** The model claims the request loop's structure, and states what it omits: Orbit,
  the transport, OS scheduling, and the shape of Eon's fix (lines 80-90). That is accurate and
  stays within fx-dett's rule.

### 3. Runner

- **Direct examples are discovered exactly as on main.**
  - Every top-level entry must be a directory with a `README.md` and at least one `*.rs`.
  - Their subdirectories are still ignored, which was Codex's r1 finding, now fixed with
    `member: bool`.
  - "Holds no example" is equivalent to main's "holds no variant".
  - Read errors still fail.
- **Group rule.** Only an entry named `case-studies` that is a directory is a group. An empty group,
  a file directly in it, and any directory inside a member (empty or not) are errors. That is
  strict and one level deep.
- **Labels.** `case-studies/<name>/<variant>` nests under `CARGO_TARGET_TMPDIR/build/examples/…`
  in `package()` without colliding with direct examples.
- **Regression test.** It lays out each case, including the direct-example `assets/` and `more/b.rs`
  that must stay ignored, and asserts each error message.
- **No weakened check.** `declared`, `check`, `tested` and the outcome rules are unchanged.

### 4. Rewrite, not copy

No Eon code is copied. The model is a fresh channel-and-clock sketch. The quotes are three attributed
lines of `d431fcab`'s message and one sentence of eon-pimct, and the license is stated.

### 5. Simplicity

This is near the smallest model that shows the class: one loop, one channel, a stop modelled as a
clock wait, one Venus loop, and the record's fixed workload with no generated cases. The concrete
deletions I would consider are in non-blocking items 5 and 6. Neither is needed.

DONE
