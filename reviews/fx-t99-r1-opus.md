# Tide review, fx-t99 round 1 (Opus): Pure status of simulator objects

## Verdict

**Approve.** The candidate (HEAD `76dfe06` plus the uncommitted diff,
sha256 `880258673cf4d58dc7437523ec068c850ced39f22a92b2c9b3192ea3ec21d2f4`)
puts every object the bead names in 02 §9's list of types that are not `Pure`,
and each classification follows from 02 §9's rules and D7. `SimRoot` and
`NodeRoot` hold capability fields, so the structural rule already rejects
them. `sim.world`, `Faults`, and `SimNode` change simulator state, the world
that the code under test sees, without a capability argument. `NodeTask`
stands for another task's result. `LogConfig` can hold a `File`. Placing
`NodeTask` in its own bullet and not with the synchronization handles is a
good choice: `node.start` takes no `&Tasks`, so the synchronization-handle
bullet would have contradicted D7 and 02 §9, which both say that creating a
synchronization handle takes `&Tasks`. No example or rule in `design/`
places one of these objects under a `Pure` bound, in a node entry's
captures, in a `with`/`with2` closure, in `join` or `par_*`, or in a `Shared`
payload. The entries in 03 §12.3 and companion §15.4 capture nothing, and
the `world.http` peers need only `Send + 'static`. The cross-references are
correct. The bead's Acceptance holds: each status is stated once, in 02 §9,
the companion line points back to it, and no other document contradicts it.
The non-blocking items below are wording nits and observations, and none of
them blocks the merge.

## Blocking

None.

## Non-blocking

N1. `design/02-memory.md:828`: `sim.world` is a field expression in a list
of types. In the same bullet `Faults` gets "(the type of `sim.faults`)". The
rule at the end of §9 that "a type the list above names is not `Pure`
whatever its fields" works on types. Because `design/` never names the
world's type, the meaning is still clear. Optional fix: write "the type of
`sim.world`".

N2. `design/02-memory.md:828`: "the simulator's controls, in test code only
(03 ROOT-3)". A reader could take "in test code only" as limiting the
classification, when it means the types exist only in test code. ROOT-4
(03:288) already uses the phrase "test-only simulator controls". Optional
fix: "the test-only simulator controls (03 ROOT-3)". Also note that ROOT-4
applies "simulator controls" only to `world` and `faults`, while 02 extends
the term to `SimNode` and `NodeTask`. This does not contradict ROOT-4, but
the shared term now covers more in 02.

N3. `design/reference-companion.md:443-445`: the phrase "The simulator's own
objects (…)" can be read as covering everything in `std::sim`. But `Fault`,
`Request`, `Reply`, `Record`, and `Sink` are also in `std::sim`
(reference.md:1013), and they follow the structural rule (`Reply` and
`Record` are plainly `Pure` data). Separately, "cannot capture another node"
is narrower than the rule: an entry cannot capture any `SimNode`, its own
included. Optional fix: "The simulator's handles (…) are not `Pure` (02 §9),
so an entry cannot capture a node or the fault controls."

N4. Observation, no change needed for this bead: the status of `Fault` is
left to the structural rule, and its fields are unspecified.
`Fault::crash(&node)` takes `&SimNode`. If a `Fault` kept that borrow, it
would not be `Pure`. If it names the node by identity, it is `Pure` data that
does nothing until `Faults::at` takes it, as `OpenOptions` does. The bead
does not name `Fault`, and no rule or example depends on its status.

N5. Observation: reference.md 6.6's summary of types that are not `Pure`
(reference.md:470-473) covers `SimRoot`, `NodeRoot`, and `LogConfig` only by
implication. It already omits `Stdio`, so this is the existing summary
practice, and the core is at 24,997 of 25,000 tokens. The companion line
covers the simulator objects for agents. No change needed.

N6. Observation: companion 17 marks `SimNode` and `NodeTask` provisional
(reference-companion.md:406), while 02 §9 now names them without that
marker. Renaming either type later means updating 02 §9 as well. This cost
is acceptable, and reference.md:1013 already names them unmarked.

## Checked

- AGENTS.md repository-local rules; GOVERNANCE.md Merge Rule; `br --no-db
  show fx-t99` (Why, Evidence, Scope, Acceptance, Notes);
  `reviews/round32-consistency.md` N4, the source of the bead.
- The diff (`git diff`, sha matched), `git diff --check` (clean), and the
  lengths of the added lines.
- 02 §8.5 and all of §9: the list of types that are not `Pure`, the
  structural rule, what `Pure` guarantees and what std must provide, and the
  std-types exception that a named type is not `Pure` whatever its fields.
- 01 D7 (the definition of `Pure`, synchronization handles needing `&Tasks`,
  sinks, the may-suspend rule) and the tier table (the simulator is v0
  core).
- 03 §1.1 (process handles, `Stdio`), §1.2 (synchronization handles and
  their `&Tasks` constructors), §1.3 (`Root` fields including
  `log: LogConfig`), ROOT-1 to ROOT-5, SINK-1 to SINK-4, §9.2 to §9.8, and the
  worked examples in §12.1 to §12.4 (`counts_lines`,
  `slow_host_hits_the_deadline`, `acked_writes_survive` with its
  capture-free node entries, and `money_is_conserved` with its `Shared` and
  `with2` closures).
- reference.md 6.1, 6.2, 6.5, 6.6, 6.8, 10.1 (clause expressions need
  `Pure + Sync` arguments), 10.2 (`#[sim_test]` with generated parameters),
  and the std table near line 1013 together with the `Faults` note.
- reference-companion.md §15.1 and §15.2 (`sim.world` use), §15.4 (the crash
  test, whose node entry captures nothing), §17 in full (the item table and
  the bullets on node entries and restarts), and §20 (`Stdio` and pipe ends,
  which are not `Pure`).
- A search of `design/` for every mention of `SimRoot`, `NodeRoot`,
  `SimNode`, `NodeTask`, `Faults`, `sim.world`, `sim.faults`, `sim.node`,
  `LogConfig`, and `root.log`. None places one of these objects under a
  `Pure`-implying bound or in a node entry, `with`, `join`, `par_*`, or a
  `Shared` payload.
