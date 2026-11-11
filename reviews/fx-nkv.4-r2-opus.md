Model: claude-opus-5-5 · Effort: low · Date: 2026-11-10

# fx-nkv.4 round-2 Opus re-check

Fix commit `9663675` on `0fef60d`. I read `git diff 0fef60d 9663675` (7 files) and the current `with` body. The worktree was clean at `9663675`.

I ran no focused probe because the gate was closed: the 1-minute load was 19.09, above 12. Free memory and the quiet window were within limits. This re-check is static. It relies on the author's reported runs (`fix-r1-report.md` checks 1-6) for runtime evidence.

## Verdict

approve. All five of my findings are resolved. The fix diff introduces no blocking defect. It adds one nit.

## Per-finding status

1. **`LockError::Aliased` restored: resolved.**
   - `sync.rs` defines `Aliased` with a doc line saying only the planned multi-cell operations (fx-ugm) return it, and adds a `Display` arm for it. The type now matches reference §7.4 `enum LockError { Aliased, Poisoned }`.
   - The conformance stub still gets the type through `include!` (`conformance/rustc/std/src/lib.rs:247`). Both surfaces agree, and the stub's `with2`/`with_all` placeholders return a type that has the variant again.
   - `lock_accept.rs` adds an exhaustive match over both variants, so removing either variant would fail on both surfaces.
   - `README.md`, the `lib.rs` header, and 03 §6.5's status note say the same thing: `Poisoned` is the only error from `with`, and `Aliased` is reserved.

2. **Downstream-constraint comment: resolved.** `sync.rs:48-50` states both obligations. fx-ugm reserves each ticket only after acquiring the previous cell in ID order. In fx-nkv.5 the scheduling point comes before reservation, or the wait hands off the baton.

3. **Queued-waiter poison test: resolved.** `shared_propagates_panics_and_wakes_poisoned_waiters` now exercises the path I asked for:
   - The holder takes ticket 0 and blocks inside the callback on `held`.
   - Three clone waiters are spawned. The test waits until `next == WAITERS + 1`, so all three tickets are reserved before release. They are therefore queued during the panicking callback, not after it.
   - Each waiter must return `(Err(Poisoned), false)` within a bounded receive. This proves both the wake on the panic arm and that the callback is skipped.
   - The scope runs in a separate `thread::spawn`. A stuck join therefore fails at the bounded `is_finished` wait instead of hanging the test.
   - The original `73u32` payload is checked at the scope's caller.
   - If a regression stopped advancing `serving` on the panic arm, the waiter receives would time out and the test would fail, not hang. Dropped senders on early failure unblock the holder, so a failed run also shuts down cleanly.

4. **Poison-tolerant lock and explicit predicate loop: resolved.**
   - Both `lock()` and `wait()` recover the guard with `PoisonError::into_inner`, so nothing after `fetch_add` can panic and strand a reserved ticket.
   - The switch from `wait_while(..)` to the explicit loop is necessary, not cosmetic. std's `wait_while` is `while cond { guard = self.wait(guard)?; }`, so a poisoned wait returns early with the predicate unsatisfied. Recovering that result would have let a caller proceed out of turn.
   - With the explicit loop, a recovered guard still re-checks `serving != ticket`. FIFO holds: only the caller whose ticket equals `serving` leaves the loop. `serving` changes only under the mutex, and each advance is followed by `notify_all`.
   - No wakeup is lost. The predicate is checked under the mutex, and `wait` atomically releases the mutex and parks. An advance between check and park cannot occur because the advancer needs the mutex. Spurious wakeups only loop again.
   - The release path is unchanged: advance `serving` under the guard, drop the guard, `notify_all`, then `resume_unwind`.

5. **LOCK-3 tag and runtime registration: resolved.**
   - `shared_counter.rs` tags LOCK-3.
   - `rules.md` adds a `runtime` value to **Decided by**, widens the intro to cover runtime rules exercised against production std, and adds the row `LOCK-3 | §6.5 | Single-cell acquisition proceeds in a cancelled scope | runtime | —`.
   - `run.sh` reads only the first-column IDs (`run.sh:23`) and does not parse the Decided-by column, so the new value cannot break the runner. The author's `--trace` output lists the counter under LOCK-3.

## New findings from the fix diff

1. **Nit (non-blocking): the rest of LOCK-3 has dropped out of the deferred list.**
   - The fix deletes `- LOCK-3 (03:1312): runtime acquisition order.` from the deferred list. The new row covers only the cancelled-scope part of LOCK-3, as it says.
   - LOCK-3 (03:1494) also requires global acquisition order by cell ID, deterministic under the simulator. That part belongs to fx-ugm/fx-nkv.5 and now appears nowhere in `rules.md` as awaiting a case.
   - The new row also cites `§6.5` without the `03:<line>` reference the other rows carry.
   - Fix: keep a deferred line such as `- LOCK-3 (03:1494): multi-cell ID order and deterministic ids; awaits fx-ugm.`, and add `03:1494` to the row. This is text-only, so no re-check is needed.

DONE
