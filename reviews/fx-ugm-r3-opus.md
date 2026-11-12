Model: claude-opus-5-5 · Date: 2026-11-12

**Verdict: approve.** I found no blocking findings. The candidate is 62b7141 on ac0533b, and `git diff ac0533b HEAD | sha256sum` gives `cc671c39…d603cd9`, which matches the brief. The worktree is clean.

I ran no build or probe. A quiet window is active until 1794456000 and it was 1794452990 when I checked. The code and the frozen-run logs settled every question: workspace exit 0 with 187 passed and 0 failed, and conformance exit 0 with 285 PASS, 0 FAIL and `tasks/shared_sets.rs` PASS. I did not open or search `trials/e1-checked/`.

## Finding 1 (one-cell `with` allocations): fixed

- **One context read per set, no ids on the real runtime.**
  - `SimCell::context` (task.rs:529) reads `CURRENT` once and filters on `task.is_some()` before cloning. A real scope's context has no task id, so the real runtime clones no `Arc`.
  - `begin_set`, `record_set` and `end_set` take that context. Only the simulated branch consumes the ids iterator, and only `begin_set` collects it into a vector, for `cell_set`.
  - On the real path, the only vectors left are `ordered` and `held`, which is the (c) that was skipped as directed.
- **Domain checks are equivalent to the old ones.** A simulated operand still needs the current task's runtime by `ptr_eq`, with the same message as `Context::simulated`. A real operand still needs no simulated task, with the same message as `outside_simulation`. Every operand is checked before the sort, the alias check and any ticket, so `set_domain_checks_precede_tickets_and_aliases` still holds.
- **`release_set` order is correct** (sync.rs:151-168):
  1. It advances `serving` on every held guard.
  2. It then drains `held` in descending id order, dropping each guard and calling `notify_all` on its own cell.

  So every cell is advanced before any guard drops, and each cell is notified after its own guard drops.
- **fx-nkv.4's rebuttal holds on every exit path:**
  - Normal and `f` panic: `run_set` catches the panic, poisons the held cells, then calls `release_set`, then `resume_unwind`.
  - Poisoned: the poison record is written, then `release_set` runs, then it returns `Poisoned`.
  - Aliased: it returns before `begin_set` and before any ticket. The context it computed is just dropped.
- **One new site sits inside the ticket window: the release `record_set`.** It now runs before the advance step.
  - It runs only under the simulator. There `sim_lock` aborts rather than unwinds.
  - `baton.unwrap()` can't fail there: nothing sets the baton to `None`, and the acquire record has already gone through the same unwrap.
  - Writing to a `String` can't fail.
  - It takes no lock that the acquire record didn't already take while holding guards, so the lock order is unchanged.
- **Simulator traces are unchanged in content and order:**
  - `begin_set` gets the same ascending ids.
  - The acquire record iterates `held` before its index sort, which is the same ascending order as the old `ids`.
  - The release record lists the same descending ids. It moved from after the notifications to before the advance, but no other trace write can come in between, and it adds no scheduling point.
  - `recorded` is the same test as the old `!ids.is_empty()`, so each non-empty batch still takes one step.
  - The trace tests check structure (shared step, order, determinism), not exact bytes. The code still produces the same bytes.
- **The new test is acceptable.** `real_sets_do_not_collect_simulation_ids` (sync.rs:349) proves the helpers never consume the ids on the real path, using a panicking iterator. It doesn't drive `acquire_set` itself, which is fine because the brief said not to add allocation-counting machinery.

## Nit 3 (stored order key): keeps decision 2

- There is one private `Cell::order_key`, which returns `self.id` (sync.rs:39).
- `Shared::new` stores either `SimCell::id()` (fx-nkv.5's `allocate_cell` value) or `NEXT_CELL.fetch_add`. That is the same key as before, so trace ids are unchanged.
- `SimCell::id()` is still used at construction, so it isn't dead code.
- `same` still compares `RustArc::ptr_eq`, not the id.

## Rebase over fx-m72.2: correct

- After removing `@@`/`index` lines, the two fx-ugm patch bodies (`a5552c3..315b5fe` and `ac0533b..d5c14e7`) differ only in context lines:
  - lib.rs: the "Not provided yet" context now lacks fx-m72.2's removed items.
  - design/03 §9: the context now has fx-m72.2's new sentence.
- Both removals are kept in lib.rs:138-145, and the design/03 paragraph at :1818-1824 reads correctly. The file stats are identical.
- fx-m72.2 doesn't touch `sync.rs`, `task.rs` or the scheduler. It changes the sim runner and the `sim_test` macro, which matter here only through `#[sim_test(seeds = 8)]` in `shared_sets`.
- That case takes no extra parameters, so fx-m72.2's generation and shrinking don't apply to it. Its sim cells use the per-runtime scheduler ids, with no global state to leak between reruns.
- I see no interaction the suites would miss.

## New findings

1. **Nit (ownership): `SimCell::context` (crates/tide-std/src/task.rs:529-548) duplicates the domain check.** It re-implements the condition and both panic messages of `Context::simulated` and `Context::outside_simulation`. The comment at task.rs:287 still calls `outside_simulation` "the one check" for that rule.
   - It is behaviourally equivalent today. It exists so the set path does a single thread-local read.
   - Optional fix: have `Context::outside_simulation` and `Context::simulated` call a shared predicate over a borrowed `Option<&Context>`, or adjust the comment.
   - This is text or structure only and needs no re-check.

No other new defects. `cargo test` showed no orphaned helpers and no warnings.
