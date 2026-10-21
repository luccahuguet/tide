Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-21.

## Verdict

**request_changes — high confidence.** Nine original blockers are resolved. Finding 8 remains partly unresolved, and a later ROOT-4 fix rejects an explicitly exempted operation.

## Scope

Reviewed HEAD `ce6d728b5cf372863b7be0220fbe23aeb83d2bc4`, one commit on `a8f48ab`.

Read AGENTS.md, GOVERNANCE.md, both requested beads and fx-kw8.4’s last comment, both round-1 reports, Opus rounds 2–8, the changes since round 1, affected driver code, cases, runner, stub exports, and relevant design sections.

Ran read-only stdin probes against existing pinned-nightly driver/std artifacts. Twelve focused conformance files matched their FX marks across both library/bin and `--test` modes. Additional probes reproduced both findings below. An independent borrow error prevented artifact generation. No files changed; the worktree remains clean.

**Neither `cargo test --workspace` nor `conformance/run.sh` ran:** both require filesystem writes. The reported 2 + 5 tests and 180/28/0 conformance result remain author-supplied evidence.

## Blocking

1. **P2 — A boxed mutable callable still misses FX0610.**  
   [walk.rs:683](/home/lucca/pjs/lab/tide-fx-kw8.2/crates/tide-admit/src/walk.rs:683)

   ```rust
   pub fn run(xs: &mut [u64]) {
       let mut n = 0;
       let mut f = |_: &mut u64| n += 1;
       xs.par_for(Box::new(&mut f));
   }
   ```

   Confirmed: rustc emits E0277, but the driver emits no FX0610. The same omission occurs with a boxed mutable reference to a captureless closure and with `par_for(&boxed_mutable_reference)`.

   The early mutable-reference check examines only the outer argument type. A `Box` bypasses it, and the later type-normalization guard can return before the supplemental `Fn` check.

   **Risk:** original finding 8 remains incomplete for the supported reference/Box adapter chain. Rustc rejects the program, but PAR-6’s required Tide diagnostic is missing.

   **Minimal patch:** remember mutable references while peeling the existing reference/Box chain, and apply the early `Fn` rejection when any adapter in that chain is mutable. Preserve FX0611’s capture priority. Add the boxed mutable-reference regression to `par_shared_reject.rs`.

2. **P2 — ROOT-4 rejects indexed writes and raw borrows.**  
   [walk.rs:536](/home/lucca/pjs/lab/tide-fx-kw8.2/crates/tide-admit/src/walk.rs:536), [03-authority-concurrency.md:363](/home/lucca/pjs/lab/tide-fx-kw8.2/design/03-authority-concurrency.md:363)

   Under `--tide-grant=unsafe`:

   ```rust
   use std::fs::Fs;

   pub fn address(p: *const [Fs; 1]) -> *const Fs {
       unsafe { &raw const (*p)[0] }
   }

   pub fn store(p: *mut [Fs; 1], value: Fs) {
       unsafe { (*p)[0] = value; }
   }
   ```

   Confirmed: both operations receive FX0517 for `[Fs; 1]`. The equivalent direct assignment `*p = value` stays clean.

   **Risk:** these operations neither read out nor forge an `Fs`. ROOT-4 expressly exempts assignments and raw borrows, but `raw_place` follows only field projections before recognizing those exemptions. Round 3’s classification as a conservative rejection does not match that text.

   **Minimal patch:** also follow builtin array/slice indexing when the current expression is its base. Keep overloaded indexing and evaluation of the index expression checked. Add clean indexed-write/raw-borrow regressions beside the existing ROOT-4 controls.

## Non-blocking

No new v1/Later-only finding for `fx-djp`, and no additional discharge route assigned to fx-kw8.4.

The opaque `Iterator<Item: Pure>` consumer limitation recorded in rounds 4 and 6 remains unchanged from round 1; it is outside this narrow regression verdict.

## Reasoning

| Original finding | Re-check |
|---|---|
| 1. Formatting builders | Resolved: transitive formatter holders are excluded; FX0603 reproduced. |
| 2. Changing recursive substitutions | Resolved: controlled depth rejection terminates without overflow. |
| 3. Default trait opaque results | Resolved: default bodies’ hidden types are checked. |
| 4. Declaration-only types | Resolved for the reported field, signature, and associated-type routes. |
| 5. Nominal mutable-reference wrappers | Resolved: field traversal produces FX0518. |
| 6. Bodyless declarations | Resolved: required methods and unsafe foreign declarations receive their codes. |
| 7. Nested attribute provenance | Resolved: unmarked helpers stay clean; independently marked helpers are checked. |
| 8. Callable adapters | Partly resolved: capture classification works; blocking finding 1 remains. |
| 9. Foreign functions | Resolved through the recorded distinction between checked Tide bodies, foreign declarations, and foreign-ABI pointers. |
| 10. Alternate forging paths | Resolved for the reported core/std paths; FX0517 reproduced. |

The coordinator decisions are sound and recorded. Trusted generic types retain checks on caller-supplied type arguments. ROOT-4 accurately describes finite defense in depth under an already unverified unsafe grant. The route-independent closure is explicitly assigned to fx-kw8.4, whose comment also records its reachable-library-root and shared-generics limitations. Neither blocking finding belongs to that deferred class.

`design/reference.md` is byte-identical to main `a8f48ab`, preserving its reported **24,992-token** size. No expansion is needed for either fix.

Not checked: fresh builds, full suites, or an independent token recount.  
Risk: the candidate still misses one required FX diagnostic and rejects valid ROOT-4 write/address operations.