Model: claude-opus-5-5 · Effort: medium · Date: 2026-11-10

# fx-ic2 dependency pruning (commit 8b0e134): Opus review, round 1

## Verdict

request_changes

The graph change itself is correct. The fix asked for below is two small field edits through `br`; nothing in 8b0e134 needs reverting.

## What I verified

- **Count.** A transitive walk of `.beads/issues.jsonl` at HEAD (f93c3e9) counts `blocks` edges plus open children of epic prerequisites. It finds exactly 28 open beads blocking fx-ic2: 23 direct open edges and fx-u65.2–.6. `br --no-db blocked --json` lists the same 23 direct open blockers. There are no cycles.
- **Longest chain.** The only maximal chain is `fx-ic2 → fx-m72.3 → fx-m72.2 → fx-nkv.6 → fx-nkv.5 → fx-nkv.4`, with five open prerequisites. This matches the comment.
- **Direct blockers.** Every open direct blocker carries the `e1-prereq` label, and each cites a §10.1 row or a gate need. fx-dsa covers the L marker (harness.py:262–264), fx-rpwp covers §10.2(5)/(6), and fx-m72.3 covers the R placeholder (harness.py:289–291). Every open `e1-prereq` bead is in the closure except the three pruned epics. No other open bead references E1 as a need.
- **Unlinked beads, all correctly outside E1:**
  - fx-nkv.7: protocol.md:120–123 says outright "(`fx-nkv.7`, not a prerequisite)".
  - fx-nkv.8: a demo, guide and agent card. It blocks nothing, and nothing in §10.1 maps to it.
  - fx-kw8.5: `package()` (harness.py:236–243) builds a fresh tree containing only `src/main.rs` or `src/lib.rs`, plus sibling-module files for acceptance. A writer cannot create `build.rs` there, so FX0915 cannot arise in E1.
  - fx-kw8.6: `candidate()` / `command()` (harness.py:419–437) catch an escaping path (`relative_to`) and invalid UTF-8 (`decode`) as a counted candidate failure before staging. The staged tree has no candidate subdirectories.
  - The seal chain (fx-kw8.15–.22, fx-g2z, fx-upy) is outside E1 by the 2026-11-09 owner exclusion of FX0516/0920/0921. fx-u65's Acceptance records that exclusion. fx-upy and fx-g2z feed only fx-g2z, fx-gcs and fx-kw8.17.
- **Comment claims.** fx-kw8 and fx-nkv never had a `blocks` edge from fx-ic2: none of the 133 ledger revisions has one. The claim that FX0911 and FX0915 have only driver integration fixtures (invocation.rs:166,281,292) and no child assignment is accurate. The fx-u65.2–.6 bodies don't name them, and fx-u65.6 excludes "codes the driver already reports".
- **Kept redundant edges.** Keeping the redundant direct edges (fx-mhd, fx-o8j, fx-nkv.4/.5/.6) instead of following Sol's removal list is a reasonable call. Each edge cites its own task need, and the count is unchanged.

## Findings

1. **Blocking: the graph now contradicts the frozen protocol, and the only record of the fix is a comment.**
   - **Evidence:**
     - protocol.md §10.1 (the table plus the sentence "Every bead in the table blocks `fx-ic2`") still names fx-byk (E1-P6), fx-m72 (E1-P12) and fx-yg5 (E1-P13).
     - §10.2(1) makes "every bead named in §10.1 is closed" a gate-start condition.
     - After 8b0e134, none of the three epics blocks fx-ic2, and nothing makes them close: br does not close an epic when its children close.
     - Once the 28 beads close, `br ready` will show fx-ic2 as ready while §10.2(1) still fails on three open epics. The bead then claims "28 open beads block the gate" when the gate as written needs 31 closures.
     - AP-CONTRACT-001 says to update the contract first for an intentional change. AP-PLAN-001 says acceptance and prerequisites stay honest in fields.
     - The remap was put on fx-rpwp only as a comment. fx-rpwp's Acceptance covers only the FX0516/0920/0921 edit to §10.2(3), and its Scope still says "Protocol, tasks and thresholds do not change". An implementer working from Acceptance will miss the remap.
   - **Fix:**
     - Add the remap to fx-rpwp's Acceptance in the same docs-review protocol edit as the exclusion:
       - E1-P6 → fx-byk.1–.2
       - E1-P12 → fx-m72.1–.3
       - E1-P13 → fx-yg5.1–.5
       - E1-P14 stays fx-u65
     - Rewrite the "blocks fx-ic2" sentence to match the graph.
     - Add a one-line note on fx-ic2 that, until that edit lands, §10.2(1) also needs fx-byk, fx-m72 and fx-yg5 closed.
     - fx-yg5's Notes already say "close when all children close". Adding the same line to fx-byk and fx-m72 makes the closures happen.
   - **Alternative (if you'd rather not touch the frozen protocol):** restore the three epic edges alongside the new child edges. Every open child is a real prerequisite, so the epics add no false blockers; the count becomes 31, the chain is unchanged, and the fx-rpwp remap comment can be withdrawn. I'd still take the first fix, because it follows the owner's "instead of on whole epics" decision.

2. **Non-blocking: the FX0911/FX0915 proof sits on an epic that no task owns, so fx-u65 cannot close on schedule.**
   - **Evidence:**
     - fx-u65's Acceptance requires "a conformance case violating it" for every named code. FX0911 is reference.md:978 and FX0915 is reference.md:1053.
     - fx-u65.6's Scope says "codes the driver already reports stay with their owners", and those owners (fx-kw8.x) are closed.
     - The 2026-11-10 comment on fx-u65 places the work "at fx-u65 close", which is not a schedulable owner. When fx-u65.2–.6 close, fx-u65 stays open with unscheduled work and fx-ic2 stalls with nothing ready.
     - The gap predates this change; the pruning analysis surfaced it. It is still a schedulable follow-up recorded only as a comment.
   - **Fix:** add FX0911 and FX0915 to fx-u65.6's Scope and Acceptance. It already owns the module/build-tooling neighbours FX0910 and FX0912–0914. The Acceptance should say the FX0915 case is package-level, either as a conformance check the per-line parser does not need, or by citing the invocation.rs test with an explicit code/severity/verdict mapping. This edits a field, not a new bead, so it fits the no-new-beads rule.

3. **Nit: wrong section cited in the fx-rpwp comment.**
   - **Evidence:** the comment says "§10.2(1) still names the old epic IDs as owners". The owners are named in §10.1's table; §10.2(1) only refers to it.
   - **Fix:** cite §10.1 when moving the text into Acceptance (finding 1).

4. **Nit: the remap leaves out readiness.md.**
   - **Evidence:** readiness.md:59–92 and 103–104 still attribute first failures to fx-yg5, fx-byk and fx-m72. They are a record at 31a9800 and remain true at epic level. Sol's report (§2) suggested carrying the remap into the readiness record; the fx-rpwp comment does not.
   - **Fix:** either note that readiness.md stays as a historical record, or include it in the same edit.

5. **Nit: the count in the commit message and comment omits the three epics.**
   - **Evidence:** the commit message and the fx-ic2 comment say "28 open beads block the gate". That is true of the graph, but under §10.2(1) as written it is 28 + 3 epics.
   - **Fix:** this resolves itself once finding 1 lands. Otherwise, say "28 in the graph; §10.2(1) also needs the three epics closed".

6. **Non-blocking (process): the change landed without this review.**
   - **Evidence:** 8b0e134 landed before this round, contrary to the GOVERNANCE.md Merge Rule. This review's corrections go in as a separate commit, not an amend.
   - **Fix:** record this verdict on fx-ic2 when the coordinator commits the field edits.

DONE
