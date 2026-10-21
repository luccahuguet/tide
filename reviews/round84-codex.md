Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-21.

## Verdict

**revise** — high confidence. The cancellation decision is sound, but §12.2 retains a partial-results guarantee that the change invalidates for limited `Tasks`.

## Scope

HEAD: `db1bebc33362d81461e90d462fc83f93ca205666`.

Diff: `git diff -- design/`.

Recomputed SHA-256: `0661355bfbd8a42c048898d6df5dbbdb059ee6286515cdf8f8f75231517b5610` — matches, including at completion. `git diff --check -- design/` passes.

Read `AGENTS.md`, `GOVERNANCE.md`, `br --no-db show fx-sz7` including every comment, rounds 80–83, the complete diff, every requested specification section, and surrounding suspension rules and examples. Searched all of `design/` for related contracts.

## Blocking

1. **B1 — §12.2 accepts limited `Tasks` but promises results that it can now lose.** [design/03-authority-concurrency.md:1816](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:1816) promises partial results, while the function accepts any `&Tasks` and sets a scope deadline.

   Pass `tasks.limit(1)` and two URLs whose requests exceed the budget. The first child occupies the slot; the body waits spawning the second. At expiry, the new rule skips the second child and records its deadline cancellation ([03:1085](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:1085)). Its join fails, and TASK-2/TASK-4 return `FetchError::Task(DeadlineExceeded)`, rather than the promised per-URL results. The explanation at [03:1844](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:1844) is therefore conditional.

   This is introduced by the reversal: previously the waiting child started and converted cancellation into data. Calling `tasks.scope` does not establish that the supplied capability is unlimited.

   **Smallest fix:** explicitly state an unlimited-`Tasks` precondition in §12.2’s function comment and surrounding prose. Its existing test remains valid.

## Non-blocking

1. **N1 — Qualify the deadlock recovery claim.** [03:1107](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:1107) says cancellation breaks the slot deadlock. Under TASK-6, shielded spawns ignore enclosing cancellation and can remain deadlocked. Say cancellation lets *unshielded* waiting spawns return. TASK-6 already determines the behavior, so this is a wording correction.

2. **N2 — The summaries omit explicit exceptions.** [reference.md:637](/home/lucca/pjs/lab/tide/design/reference.md:637) does not distinguish an unstarted child that is merely an echo in an uncancelled target. Also, [reference.md:668](/home/lucca/pjs/lab/tide/design/reference.md:668) and [01-design.md:521](/home/lucca/pjs/lab/tide/design/01-design.md:521) retain general consumed-value wording, whereas §6.1/TASK-3 explicitly drop `f`. The detailed rules resolve both cases. Any core clarification should replace or trim existing text, preserving the budget.

3. **N3 — Recorded token count is stale.** [reference-companion.md:14](/home/lucca/pjs/lab/tide/design/reference-companion.md:14) still records 24,999; the supplied measurement and rounds 82–83 report 24,995. Correcting this metadata costs no core tokens.

### Pre-existing

4. **N4 — Existing underspecification remains outside this change.** `limit(0)`, chained limits, and isolated tasks’ participation in the count remain unspecified ([reference.md:587](/home/lucca/pjs/lab/tide/design/reference.md:587)). TASK-2’s attribution of nested cancellation to an outer child’s echo bit also remains ambiguous ([03:1135](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:1135)). These do not block this pass.

## Reasoning

The central choice is coherent: a shared concurrency count can be occupied by unrelated scopes, so observing the **caller’s** cancellation avoids an otherwise unbounded wait. The target-cancelled and target-uncancelled cases are explicit; the latter’s synthetic echo prevents a request deadline from automatically failing an enclosing server scope ([03:1076](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:1076)). Propagating the returned join error can still fail the caller normally.

The suspension/checkpoint classification agrees with §0 and §1.2, and fits D7’s synchronization authority. TASK-1..4 and JOIN-1..2 preserve panic precedence, failure selection, deadline conversion, and mandatory joining. Dropping unrun captures happens in the caller; a destructor panic follows D5. No change weakens the lifetime or failure-edge guarantees in [02-memory.md:683](/home/lucca/pjs/lab/tide/design/02-memory.md:683) and [02-memory.md:707](/home/lucca/pjs/lab/tide/design/02-memory.md:707).

Shielding remains governed by TASK-6. Hold-and-wait deadlocks remain possible; pending deadlines can wake unshielded callers, while §9.3 reports only global deadlock when no future event permits progress ([03:1508](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:1508)).

The edited companion 15.2 example holds under its demonstrated inputs. Operation deadlines let late-starting children produce outcomes without cancelling queued spawns. Its test still yields `[Fetched(5), TimedOut]` at two virtual seconds ([reference-companion.md:230](/home/lucca/pjs/lab/tide/design/reference-companion.md:230)). Ancestor cancellation can still make the function return `Err`, as its signature permits. The 14.3 mappings remain useful shorthand.

No files changed. Examples were checked semantically, not executed; the supplied 24,995-token count was not independently remeasured.