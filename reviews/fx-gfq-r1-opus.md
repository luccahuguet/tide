Model/Effort/Date: claude-opus-5-5, low, 2026-10-31

# fx-gfq r1 Opus review: frozen checked-toolchain E1 protocol

## Verdict

request_changes, confidence 0.8.

The direction is right. The paired arms, the fixed sample, the separate
metadata-only pilot, the executable-readiness checklist, and the prerequisite
beads with their blocker edges are what fx-gfq asks for. I recomputed every
checked numeric value (A2, A3, A4, A5, A10, B3, B4, C4, C9 Collatz 0/8/111/261,
C9 summary 119/3/1/2) and found them correct. Bead graph: 20 owners in §10.1,
all 20 block fx-ic2, `br dep cycles` reports none. Seven findings still need
fixing before the freeze. Six of them are text-only.

## Scope

Candidate: the working-tree diff against HEAD 1e0cbac. Its SHA-256 is
d1bd3f2c99d2f100ceb89a58e1094c115e546d4f369612e2bc118aafc7af8c86, and I
confirmed it matches r1.diff. Files read:
- `trials/e1-checked/protocol.md` and `tasks.md`, in full
- the 13 new beads, fx-gfq, fx-ic2, fx-nkv.1–6, and fx-kw8.9
- `design/01-design.md`: Tiers and D0
- `design/reference.md`: all of §§0–13
- `design/reference-companion.md`: §15
- the task lists of `trials/e1-dialect` and `trials/e1-origins`
- the `#[pure]` handling in `crates/tide-admit/src/walk.rs`
- `rust-toolchain.toml`
- `GOVERNANCE.md` (Merge Rule)

I did not edit anything in the worktree.

## Blocking

1. **The iteration measure dilutes D0's 1.5x threshold**
   (`protocol.md:165-169`, `:191`).
   - Risk: a trial that fails final pass counts as 20, and that constant
     goes into both arms' means. When pass rates are similar, this pulls the
     ratio toward 1. Example: both arms have P = 0.8, Rust's passing trials
     take 3 runs, and Tide's take 6. Then I(Rust) = 0.8×3 + 0.2×20 = 6.4 and
     I(Tide) = 0.8×6 + 0.2×20 = 8.8. The ratio is 1.375, which meets the
     threshold even though Tide's iterations to green are 2x Rust's.
   - D0 says "iterations to green". A failed trial is already penalized by
     threshold 2.
   - Fix: gate I on the mean runs over trials that pass final in that arm.
     Report the censored mean (failures = 20) as information only. Keep §8's
     wording otherwise unchanged.

2. **The held-out rule starts too late** (`protocol.md:33-36`).
   - Risk: the no-copy rule starts at "the first gate writer session". But
     `tasks.md` is committed now, and about 20 prerequisite beads will edit
     the std, the diagnostics, and possibly the reference and companion
     before the run. Agents doing that work can read the tasks and shape
     examples, fix plans, or diagnostic text toward them.
   - §9 already treats pilot-driven document changes as voiding the task
     set. A direct edit made with `tasks.md` in view is not covered.
   - Fix: make the rule run from the freeze (fx-gfq close). Add that any
     change to the reference, companion, diagnostic text, or examples in
     that window must not draw on `trials/e1-checked/`.

3. **§10.2 lets an incomplete run report "failed"** (`protocol.md:270` vs
   `:16-17`).
   - §1 says only a run that meets every prerequisite can report E1 as
     passed **or failed**. §10.2 says a run missing an item "cannot report
     E1 as passed", which implies it may report failed.
   - Risk: a run on a partial toolchain becomes a recorded gate failure that
     reflects missing toolchain work, not the language.
   - Fix: "cannot report E1 as passed or failed".

4. **Two required surfaces have no committed owner** (`protocol.md:229-250`;
   `tasks.md:555-556`, `:803-805`).
   - `Scope::cancel` (C6's `.cancel(`) is assigned to fx-nkv.3, but
     fx-nkv.3's Scope says "cancel() only if exposed". So it may never ship.
   - The `#[pure]` attribute is required by B7's structure check, but it is
     in no table row. It exists only as a stub proc macro in
     `conformance/rustc/macros`. fx-nkv.1's "Pure declaration" does not
     clearly cover the attribute.
   - Risk: B7 and C6 can't be done as written, and readiness item 1 can
     pass while they are missing.
   - Fix:
     - Add `Scope::cancel` to fx-zc1's Scope (or make it firm on fx-nkv.3).
     - Add a `#[pure]` row with an owner (fx-nkv.1 or fx-yg5) and put it in
       that bead's Scope.

5. **All 13 new beads cite row IDs that do not exist**
   (`.beads/issues.jsonl`, lines 1, 10, 15, 24, 55, 59, 82, 84, 104, 105,
   116, 123, 124).
   - Each bead's Why and Evidence cite "§10.1 row E1-P1…E1-P13". The
     protocol table has no such IDs; `grep E1-P trials/ design/` finds
     nothing.
   - Risk: the owner link from bead to protocol row is broken.
   - Fix: add an ID column to the §10.1 table matching the beads' numbering
     (fx-zc1 = P1 … fx-yg5 = P13), or drop the IDs from the beads.

6. **fx-yg5's Acceptance depends on work that can only happen after it
   closes** (fx-yg5 Acceptance, `issues.jsonl:123`).
   - It requires that "the driver admits every reference-admitted item that
     the E1 reference solutions use".
   - The reference solutions are written under fx-ic2 (§6, §10.2 item 2).
     fx-ic2 is blocked until fx-yg5 closes, so fx-yg5 can never check its
     own acceptance.
   - Fix: phrase it against the Tide-arm items `tasks.md` names and the
     reference teaches (or the §10.1 row). Leave the reference-solution
     check to fx-ic2's readiness list, where it already is.

7. **B7's spec contradicts its own case 4, and its replay check is weak**
   (`tasks.md:552-554`, `:560-562`).
   - The contradiction: "If any step's source is missing before the first
     rename, it returns NotFound." A swap plan's last step renames from the
     temporary name (e.g. `a.tmp`), which does not exist before the first
     rename. Read literally, `apply` fails case 4.
   - The weak check: the replay compares only name sets ("the final set must
     equal the simultaneous moves"). That accepts an empty or wrongly
     rotated plan for the 3-cycle in case 1, since the start and final
     name sets are both {a, b, c}.
   - Fix:
     - Change the rule to "a source that neither exists before the first
       rename nor is the target of an earlier step".
     - Say the replay tracks contents: each name ends up holding the
       original content of the source moved to it.

## Non-blocking

- **C5 is stricter for Tide than for Rust** (`tasks.md:788-792`). Tide must
  hit exactly 100 ms; Rust has only an upper bound. Add B10's "at least the
  stated time" lower bound for Rust.
- **A9 case 3 doesn't give its starting inventory** (`tasks.md:326-329`). It
  needs skus `a` and `b`, but case 1's inventory has only `a`. "Applies"
  states no resulting quantities. Give the inventory.
- **Some signatures are ambiguous:**
  - B4 `read_all`: a free function or `RotLog::read_all`?
  - The payload type of B6's `BadName` and `Duplicate`, and of B7's
    `DuplicateSource` and `DuplicateTarget` (String or &str).

  Whatever the grader picks, it must compile against both readings, or
  first-check will measure the guess. Simplest: state `String` payloads and
  a free `read_all`.
- **The held-out disclosure in §2 is incomplete** (`protocol.md:27-32`).
  - C7 is a near-isomorph of e1-dialect `ring` (bounded container,
    requires-not-full, length postconditions, panic on a full push).
  - A10's `with_word` closely mirrors e1-dialect `scan`'s `containing`.
  - A1's drop-the-index case mirrors e1-origins `group_words`.
  - B4 is close to companion 15.4.
  - B9 is the reference §6.3 atomic-replace snippet.

  The reference was tuned on those earlier trials. List these alongside the
  companion overlaps so the report can read their results with that in mind.
- **The Rust arm runs on an older compiler** (`protocol.md:115`). It uses
  stable 1.96.0, while the Tide arm builds on the 1.99 nightly, so Tide
  writers get three more releases of stabilized language and core APIs.
  Either name this in §11's asymmetries or move Rust to 1.99 stable.
- **The run order and bootstrap aren't reproducible from the seed alone**
  (`protocol.md:59-62`, `:195-197`). Name the shuffle and resampling
  implementations along with the seed.
- **§7.2 "all of the writer's own tests" is unclear for Tide.** State
  whether it includes Tide's generated contract tests and vacuity failures.
  §4.1 and §11 imply it does.
- **First-check can be inflated by an early probing run.** A first run on a
  `#[tide::main] fn main(){}` binary skeleton, or on `todo!()` stubs, passes.
  The Tide writer, new to the toolchain, is the more likely of the two to
  probe early. Record the first snapshot's size and hole count per trial so
  the report can show this.
- **fx-yg5 overreaches and fx-hxw has an unexplained edge.**
  - fx-yg5's Scope includes `RandomState::from_rand`, which needs `Rand`.
    `Rand` is out of coverage (§11) and has no owner.
  - fx-hxw's edge on fx-o8j has no reason in its body.
- **Seed counts are only partly specified.** They're given for C4 and C10
  but not for C3, C5, C6, or C9. The frozen suites will fix them, but the
  task text could state them.
- **Nothing owns the harness work before fx-ic2.** The harness, render
  script, sandbox, suites, and reference solutions have no owner except
  fx-ic2, which is blocked until all 20 prerequisites close. Writing the
  reference solutions is the cheapest way to find a missing prerequisite. A
  separate unblocked bead for suites, solutions, and harness would find gaps
  while the prerequisite beads are still open.
- **Structure checks are text searches**, so a mention in a comment
  satisfies them. This is acceptable since writers see the requirement, but
  the report should note it.

## Reasoning

**Gate thresholds.** D0 states first-check ≥ 0.8x Rust, final pass ≥ Rust,
and iterations ≤ 1.5x Rust. §8 copies all three exactly, and §1 and §9 keep
the metadata pilot from ever producing a verdict. The weak point is the
operational definition of iterations (finding 1): it can turn a 2x
iteration gap into a pass. That is the kind of misleading result fx-gfq's
Acceptance forbids.

**Prerequisites.** These are mostly right. The 13 new beads cover every
Tide-arm surface the tasks name, and fx-ic2 has an edge to all 20 owners.
The defects are:
- two surfaces with no committed owner (finding 4)
- the ordering loop in fx-yg5's Acceptance (finding 6)
- the dangling row IDs (finding 5)

**Tide bindings.** These match reference v0:
- `Shared::new(v, tasks)` and `with_all`
- `sync_channel(n, tasks)`
- `Clock` sleeps returning `Cancelled`
- `Fs::dir`, `write_synced`, `sync_dir`
- `spawn_isolated`/`Panicked`
- `par_map`/`join`
- the contract clauses

**Checked cases.** All the values I recomputed are right. The one
self-contradiction is B7 (finding 7).

**Held-out.** The task texts are new. The real held-out risk is the long
window before the run (finding 2), not the tasks themselves.
