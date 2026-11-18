Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate 3411299

# fx-zu4f.4 r2: Opus converge round (reports, fixes nothing)

## Verdict

**request_changes**, high confidence. There is one blocking finding: the root README now has a stale
count. The fix is one word. The case study is faithful to Eon, its declared outcomes hold, and its
claims are sized correctly. Once the word is fixed, this is an approve.

## Scope

- Diff `git diff 33085fd 3411299` (SHA-256 `226d1b27…b9531f`, matches the brief): `README.md` and
  the four new files under `examples/case-studies/eon-discovery-race/`.
- Read: the bead fx-zu4f.4 and its comments, the parent fx-zu4f and its comments, AGENTS.md, and
  GOVERNANCE.md's Merge Rule. Also read research/33 §3 (rank 3) and the examples runner
  (`crates/tide-admit/tests/examples.rs`: declarations and `tested`). In
  `crates/tide-std/src/test/runner.rs`, I read `run_config`: the determinism self-check and the
  failure reporting. Then the sibling case-study list and `paper/tide-v0.typ`'s case-study text.
  Finally the builder's report, and Codex r1 after I had formed my own view.
- Eon, read-only with `git show`/`git branch -r --contains`, plus `br --no-db show` for eon-8ehhn and
  eon-j7zfg:
  - commit `27b39095`: its message, its `CHANGELOG.md` entry, and its `sessions.rs` diff
    (`reconcile_if_ended`, `discover_untracked_runs`, the `before_endpoint_check` hook and the
    discovery loop at 900–960);
  - the bead's body and landing record;
  - `27b39095` is on `origin/edge` and not on `main`, as the write-up says.
- Ran, through heavy.sh at load about 2:
  - `cargo test -p tide-admit --test examples -- --test-threads=4` (target
    `/mnt/build/tide-agents/fx-zu4f.4-r2-target`): exit 0, 8 passed.
  - The buggy and out_of_order test executables that run built, run directly with
    `--test-threads=1`. Both exited 101.
    - buggy: `87 passed; 13 failed (100 seeds)`, failing seeds 1 12 17 25 27 33 39 40 51 74 86 93
      97, the same as the builder's list.
    - out_of_order: `89 passed; 11 failed (100 seeds)`.
    - In both, every failing seed printed the discovery cause, and no
      `nondeterminism`/`self-check` line appeared.
    - In seed 1's trace, `fs.read` (Live) completes before the host's `fs.write` and
      `fs.remove_file`, then `fs.metadata` fails, as the README describes.
  - `git diff --check 33085fd 3411299`: clean.

## Blocking findings

1. **A stale case-study count in the root README, in a file this change edits.**
   - **Where:** `README.md:212`, which reads "**Runnable evidence**: the demo, the examples, the two
     case studies, four benchmark pairs…".
   - **Evidence:** this change adds the third case study (`examples/case-studies/` now holds
     eon-discovery-race, eon-slow-close and hyper-ambient-runtime). It updated the summary at
     `README.md:134` ("none of the bugs") but missed this line. Both the builder's grep and Codex
     r1 missed it. This is the leftover AP-MINIMAL-001 names. The line is in `README.md`, which this
     change edits, and it is launch-facing text that fx-qlek.7 will audit.
   - **Minimum fix:** change "the two case studies" to "the three case studies". "The case studies"
     also works and will not go stale again. This is a text-only fix and needs no re-check.

## Non-blocking findings

2. **The paper is now stale (out of this diff; route it to tide-lead or fx-qlek.7).**
   - `paper/tide-v0.typ:30` says "Two case studies show that admission catches none of the real bugs
     studied". `:144` says "Two case studies rewrite four real bug reports", and the section lists
     only eon-slow-close and hyper.
   - The builder flagged this too. The paper is fx-qlek.4's reviewed text and is not part of this
     diff. So it should not block, but the launch-claims audit (fx-qlek.7) should not miss it.
   - The paper's framing, "what Tide adds there is a deterministic test of the fix", is also
     narrower than this study's gain, which is finding the interleaving by seed.

3. **The "Declared outcome" column says more than the declaration checks.**
   - **Where:** `examples/case-studies/eon-discovery-race/README.md`, table row for `buggy.rs`:
     "fails on 13 of 100 seeds, each with discovery's 'cannot inspect terminal endpoint' error".
   - **What the declaration checks:** `//@ reports:` is a substring check on the test's captured
     output (`examples.rs`, `tested`). It proves the count and that at least one seed reported the
     cause. It does not prove that every one of the 13 did.
   - **Why the same is true of the rerun claims:** the README says each failing seed "reruns with an
     identical trace" (lead, and root README bullet). The runner does rerun every failing seed
     (`runner.rs`, `run_config`). But a seed that failed its self-check would still count among the
     13 failures, while other seeds printed the expected cause. So the declaration would still pass.
   - **The facts hold today:** in my run all 13 printed the cause and none was nondeterministic.
     The builder's report, however, overstates the check: "holds only when each failing seed's
     rerun matched" is not what the runner enforces.
   - **Optional fix:** in the table, drop "each" or move it out of the "Declared outcome" column.
     Do not touch the runner (it is out of scope).

4. **"Eon already had a deterministic test" in the root README can mislead.**
   - **Where:** `README.md:141`.
   - **Problem:** read alone, the phrase suggests Eon's tests predated the bug. The case-study
     README says it precisely: "Eon's fix already landed with its own deterministic test".
   - **Optional fix:** "Eon's fix landed with a deterministic test".

5. **Small modelling gaps the "does not model" list could name.** None changes a claim.
   - **Two sockets, not one.** Eon's host has two endpoints, presentation and management, and
     `reconcile_tombstone` waits for both. The model has one.
   - **A missing record on the reread.** The model's reread propagates `read_record`'s error, where
     Eon's `read_management_record` returns `None` and keeps the first endpoint error (the landing
     record: "no record" returns the first error). The model never removes the record, so this path
     is unreached.
   - **Optional fix:** one bullet for each.

6. **"Either" reads oddly.**
   - **Where:** `README.md:137`, "admission would not have rejected Eon's bug either".
   - **Problem:** the bullet comes first in the list, so "either" has nothing earlier to refer to
     except the summary line.
   - This is a nit.

## Reasoning

**Faithfulness to Eon.**
- At `27b39095`, discovery does four things in order: reads the record (`read_management_record`),
  checks host liveness (`recorded_process_is_dead`, then `retire_dead_run`), calls the new
  `before_endpoint_check` hook, then calls `validate_management_identity(.., true)`.
- On that check's error, `reconcile_if_ended` rereads once. It accepts an equal-identity tombstone,
  or the same Live record at the same inode with a dead host, and otherwise keeps the first error.
- The write-up's "The bug" section matches this, the commit message and the bead. That covers the
  consumers (Stop, final or Stack close, replacement supervisor start), the "No retry, sleep or new
  timeout" quote, the bead's acceptance quote, and the hook-based test.
- The model keeps exactly the window that matters: a stale Live read, then a failed endpoint check.
  `fixed.rs` keeps the tombstone branch of the fix.
- The write-up names every omission in the brief: socket stat, run identity, the
  `retire_dead_run` branch (crash faults planned), reconciliation, the residual races, and ORB-C13
  as an assumption. `out_of_order.rs` makes the last one concrete without claiming that Orbit keeps
  the order.
- Codex r1's hook overclaim is gone. No other "new hook" or "hook placement" claim remains. The
  claims that do remain, that Eon's test needed `before_endpoint_check` added to discovery, are
  supported by the commit.

**Declared outcomes.**
- **buggy:** `test fails` with the cause and `87 passed; 13 failed (100 seeds)`.
- **fixed:** `tests pass`, which requires at least one passing test and none failing.
- **out_of_order:** `test fails` with `89 passed; 11 failed`.
- These pin the claimed counts, so the README's "13 of 100" and "11 of 100" are checked by
  `cargo test --workspace`. The fixed variant's fail-closed test guards against a "fix" that would
  settle every missing endpoint as Ended.
- Simulation seeds are the case indices 0..99 (`case_seed`), so the counts are deterministic. They
  move only if the std scheduler changes, the same brittleness the sibling studies accept.
- The weaker point is "each" (finding 3).

**Framing.**
- The negatives come first: admission does not reject the bug, someone must state the property,
  Eon already had a deterministic test, and the model shows only what it models.
- No speedup is claimed. "Unforgeable" does not appear.
- "`Fs` is its only file access" describes std's design (D7, no ambient file access). It is not a
  forgery claim, and it is acceptable.
- The replay claim is limited to `--tide-run-test`/`--replay`. Codex r1 and the builder each ran a
  replay of seed 1 independently, and both reproduced.
- The root summary "Admission catches none of the bugs" is accurate for all three studies.

**Leftovers.**
- The root README's case-study list is updated, but the "two case studies" count at line 212 is not
  (finding 1).
- No other in-repo index lists the case studies except the paper (finding 2).
- The examples runner discovers the new directory by itself, and the group rules accept it (README
  plus variants, no subdirectories).
- Nothing else breaks: the focused examples suite passes.

**Blocking test.** Under the brief's rule, finding 1's fix changes `README.md`, a file this change
edits, and the text is wrong, not a nit. Findings 3–6 are nits or optional precision. Finding 2
lies outside the diff.

DONE
