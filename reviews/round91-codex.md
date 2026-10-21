Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-21.

## Verdict

**revise** — high confidence. The limit decisions are coherent; shielded join attribution retains one result-changing ambiguity.

## Scope

HEAD: `fceb12ea3b368122cd74094626209bf0e779cc35`.

Diff: `git diff -- design/`.

Recomputed SHA-256: `a36cba04746818daefb2488ad42a35059a8169af778bd01d0eb55fe04f7a7122` — matches at entry and completion. Only `design/03-authority-concurrency.md` changes. `git diff --check -- design/` passes.

Read `AGENTS.md`, `GOVERNANCE.md`, `br --no-db show fx-0ai` including all revisions, rounds 87–90, the complete diff, the requested specification sections, and relevant companion material. Checked the preceding commit and rounds 84–86 for settled behavior; searched all of `design/` for related contracts. An independent reader checked the shield counterexample.

## Blocking

1. **B1 — “As if no shield were in force” can suppress an independent nested failure.** The join rule at [03:1161](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:1161) permits two interpretations: an unshielded checkpoint using current cancellation state, or historical recomputation without shields. [Round 90:137](/home/lucca/pjs/lab/tide/reviews/round90-consistency.md:137) explicitly endorses the latter.

   Let `bg` spawn `H` before sibling `S`. With its echo bit clear, `H` enters a shield and opens nested scope `s`. Then:

   - `S` fails; the shield prevents its cancellation from reaching `s`.
   - `s`’s child `K` independently returns `Err(K)`.
   - Inside `s`’s body, `H` calls `K.join()?`, then propagates `s`’s result.

   Using actual causes, `s` reports `K`; `H` remains independent and, because it was spawned first, `bg` reports `K`. Historical recomputation instead treats `s` as cancelled through `bg` before `K` failed, sets `H`’s bit, and makes `bg` report `S`.

   That contradicts the actual-first-cause and nested-reach rules at [03:1154](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:1154), and D8’s independent-failure ordering at [01:535](/home/lucca/pjs/lab/tide/design/01-design.md:535). The returned error changes, so this is semantic rather than a wording nit.

   **Smallest fix:** specify that the hypothetical checkpoint ignores shielding **at the join**, using actual recorded cancellation causes and reachability; it must not replay past cancellation or replace a nested scope’s own cause. Direct failed joins inside a user’s shield still receive the intended mark. No core addition is needed.

## Non-blocking

1. **N1 — Make transitive isolation explicit.** I read “a task it waits for with `spawn_isolated`” at [03:1151](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:1151) transitively. Adding “directly or through further isolated calls” would settle `H → I1 → I2`, with no intermediate scope and only `I2` observing cancellation. A direct-only implementation could otherwise leave `H` unmarked.

### Pre-existing

2. **N2 — Suspension terminology remains inconsistent.** [D8:511](/home/lucca/pjs/lab/tide/design/01-design.md:511) permits suspension only through capabilities; [03:54](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:54) also names synchronization operations. This predates the diff and does not block it.

## Reasoning

The limit rules preserve attenuation. Clones share counts; chained limits retain every outer count and acquire new reservations together. A zero count remains usable until `spawn`, whose unconditional panic resolves cancellation and shielding precedence ([03:1112](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:1112)). This fits D5 and the core’s same-authority `Clone` note.

`spawn_isolated` substitutes for its waiting caller’s execution and takes no slot. Spawning through an owned limited clone still uses the shared counts. Existing children can nevertheless hold slots while awaiting further spawns; the chapter acknowledges that deadlock, its cancellation remedy, and shielding exception ([03:1105](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:1105)). Atomic acquisition introduces no additional partial-reservation deadlock. Passed-down scopes retain their creating capability’s counts.

Outside B1, first-cause attribution distinguishes nested independent failures from inherited echoes. Ancestor cancellation, `cancel()`, and own deadlines do not become failure-caused echoes. Ancestor deadline ties and scope-versus-operation ties have explicit precedence. Failed, echoed, and unstarted joins follow the checkpoint attribution conditions; panics retain priority, and body joins retain child-error precedence ([03:1135](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:1135), [03:1182](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:1182)).

The change preserves mandatory joining, origins, and isolated closures’ ownership requirements in 02 §8.2–8.3. The core and companion introduce no further blocking contradiction. The core is byte-identical to HEAD, preserving the recorded 24,995-token measurement.

No files changed. Runtime behavior was not executed; the unchanged core’s token count was not remeasured.