# Tide review, fx-t99 round 2 (Opus): confirmation of round 1 fixes

## Verdict

**Approve.** The candidate is HEAD `76dfe06` plus the uncommitted diff, with
`git diff -- design | sha256sum` =
`360e7cb54d688b6fdee3aa4b64c3c18be88cc36310be46cde2282573a08a8cec`, which
matches. All three wording fixes resolve the round 1 findings and add no
contradiction or wrong cross-reference:

- **N1** is resolved. `design/02-memory.md:828` now reads "the type of
  `sim.world`". It parallels "`Faults` (the type of `sim.faults`)", so every
  entry in the list is a type, which matches the rule at the end of §9 that
  a named type is not `Pure` whatever its fields.
- **N2** is resolved, and the fix is better than the one round 1 suggested.
  The bullet now reads "the test-only simulator handles (03 ROOT-3)". So
  "test-only" describes the types, not the classification. The bullet also
  uses "handles", not "controls", so 02 does not stretch ROOT-4's term "test-only
  simulator controls (`SimRoot`'s `world` and `faults`)" to cover `SimNode`
  and `NodeTask`. The point round 1 raised about ROOT-4's narrower term no
  longer applies.
- **N3** is resolved. `design/reference-companion.md:443-445` now says "The
  simulator's handles (…)". This no longer claims to cover the whole of
  `std::sim`, so `Fault`, `Request`, `Reply`, `Record`, and `Sink` stay under
  the structural rule. "So an entry cannot capture a node or the fault
  controls" now states the full rule, the entry's own node included, and
  "fault controls" fits ROOT-4's use of "controls" for `faults`.

## Blocking

None.

## Non-blocking

N1 (nit). `design/reference-companion.md:443`: the companion's "simulator's
handles" list includes `SimRoot` and `NodeRoot`. 02 §9 puts those two in the
capability bullet as "simulated roots" and keeps "simulator handles" for
the type of `sim.world`, `Faults`, `SimNode`, and `NodeTask`. Each document
is internally consistent, and the classification (not `Pure`) is the same in
both, so this is a difference of grouping only. ROOT-4's "capability and
handle types" covers both readings. No change needed.

N2 (nit). `design/02-memory.md:828-831`: the simulator bullet chains two
"…, which …" clauses ("…, and `SimNode`, which act …, and `NodeTask`, which,
like `Task`, observes …"). It reads correctly, but it would be easier to
parse with a semicolon before "and `NodeTask`". This sentence did not change
in this round. Optional.

## Checked

- `br --no-db show fx-t99` (Scope and Acceptance), round 1's
  `reviews/fx-t99-r1-opus.md` (N1 to N6) and `reviews/fx-t99-r1-codex.md`
  (approve, no findings).
- The current `git diff -- design` (sha matched) and `git diff --check --
  design` (clean).
- 03 ROOT-3 (`Root::sim`, `SimRoot`, `NodeRoot`, and everything in `std::sim`
  are test-only, so the cross-reference "(03 ROOT-3)" is correct) and ROOT-4
  ("test-only simulator controls (`SimRoot`'s `world` and `faults`)", and
  "sink configuration (`Root::log`)"). Neither conflicts with the new
  wording.
- 03 §9.2 to §9.5 (`sim.world`, `sim.faults`, `sim.node`, `node.start`, the
  node-entry capture bounds `Pure + Sync`). The references "(03 §9.2,
  §9.5)" and "(03 §9.2–9.5)" point to the right sections.
- Companion §17 in full: the item table (`SimNode` and `NodeTask` are
  provisional), the node-entry bullet with the new sentence, and Restarts.
  No contradiction.
- reference.md:376, 513, 1013, and 1016 (the `std::sim` members, and
  `Faults` as the type of `sim.faults`). These agree with the classifications
  and with N3's narrowing to "handles". Unchanged items: the classification
  of `SimRoot`/`NodeRoot` in the capability bullet and of `LogConfig` in the
  `Stdio` bullet with "(03 SINK-2)". Both are as round 1 approved them.
