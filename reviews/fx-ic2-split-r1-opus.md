Model: claude-opus-5-5 · Date: 2026-11-13

# Opus review: fx-ic2 split into fx-ic2.1-.7 (planning-only)

## Verdict

**request_changes.** Five findings block (1-5) and four do not (6-9).

The split itself works. The ranges 1-3, 4-40, 41-80 and 81-120 cover all 120 positions once each.
The thresholds (0.8x, >=, 1.5x), the budget (20 runs, 40 minutes), the sample, the §2 held-out
rules and D0's "failure is reported, the gate stays open" all match the protocol and fx-ic2's
Acceptance. The spending go is correctly left to the owner, outside the auto-decide relay. Every
§10.2 item appears in fx-ic2.1. There are no cycles. fx-u65 is the only open direct blocker of
fx-ic2 today, so fx-ic2.1's "last open one" is true. Because br treats an epic's open blockers as
blocking its children, no child is ready yet, which is correct.

The blocking problems are about order, falsifiability and mechanism, not the split. Some of the
gate-operation work changes harness code. The machine's actual mechanics (coordinator.py fixes
jobs, model and identity at `init`; writer builds run inside the jail, not through heavy.sh) are
not reflected in the beads. Two Acceptance sections would pass with the outcome unmet.

Evidence read: `br --no-db show` for fx-ic2 (body, Acceptance, all comments to 2026-11-13 20:35Z),
for fx-ic2.1-.7, and for the grading and probe parts of fx-rpwp; protocol.md §2, §3, §4.2 and
§6-§10; `trials/e1-checked/coordinator.py` lines 28-34, 349, 544-560, 640-770 (order, init, batch,
probe, CLI). I opened no tasks, suites, references or writer material.

## Findings

### 1. Blocking: fx-ic2.2 changes the harness, but fx-ic2.1 can record sandbox and identity evidence before those changes, and neither bead runs `init` in a defined place

fx-ic2.2 asks for three changes to fx-rpwp's landed machinery, each carried over from fx-rpwp's r4
finding 5: the Nix-store closure bind (instead of all of /nix/store), jail memory caps
(`bwrap --size` plus a cgroup or ulimit), and a pinned, recorded build-jobs setting for both
writing and grading. These are code changes. They also change exactly what fx-ic2.1 must record as
§10.2 item 5 (the sandbox and canaries) and item 6 (identities: `g.identity(configs, cli, model,
jobs)` at `init`). Right now .1 and .2 have no edge between them; both only block .3. That allows
two bad orders:

- .1 records canaries and identities, then .2 changes the jail. AP-PROOF-001 says proof is never
  reused after a material change.
- .2's code lands without the Codex pass and the Opus converge round that GOVERNANCE.md's Merge
  Rule requires for code, because .2 says nothing about review and its Acceptance is only a
  comment plus a booking.

Two more ordering facts come from coordinator.py:

- `probe` (the live tool-surface probe, an item in .2) refuses to run without an initialized
  state ("probe requires the recorded real CLI").
- `init` permanently fixes the order, `order_hash`, `jobs`, model, CLI, configs and identity.

So `init` has to come after the harness changes and before the probe. Today no bead owns `init`.

**Fix.** Split fx-ic2.2 in two:

- **(a) Harness hardening.** Closure bind, plus at least a flag on store paths that contain
  trials/e1-checked; memory caps; build jobs pinned and recorded for writer and grading. It lands
  as a reviewed code change (Codex pass, then the Opus converge round), with harness_test, both
  canaries and the four-slot dry run rerun on the landed tree.
- **(b) Machine operations.** tmpfs TMPDIR, cargo-sweep, the probe, the N3 rule, the booking.

Order the work as (a) → fx-ic2.1 → (b) → fx-ic2.3. fx-ic2.1's Scope should then say it creates
the pinned gate worktree at its recorded revision and runs `coordinator.py init` there, choosing
`--jobs` (see finding 4). That puts the order, its hash and the identities in `state.json` before
anything records them. If (a) and (b) stay in one bead, at least add a `fx-ic2.2 blocks fx-ic2.1`
edge, give .2's Scope the review requirement, and move the probe to after `init`.

### 2. Blocking: fx-ic2.2's Acceptance passes on any observed state

The current Acceptance is "a gate-operations comment naming each item above with its observed
state". An item observed as failing still meets it. Compare fx-ic2.1, which says a failing item
keeps the bead open.

**Fix.** Make the Acceptance require each item in a passing state, shown by a check:

- the Rust arm's fixtures resolve under a tmpfs TMPDIR (shown in a dry-run slot), or a measured
  proof that the fsync exposure is gone;
- `systemctl list-timers` shows cargo-sweep.timer not due before the booked slot ends;
- a canary cannot read a store path that holds trials/e1-checked content;
- a jail process over its memory cap is killed;
- the identity record shows the pinned build jobs;
- the probe denies all three outside requests, and, per fx-rpwp r4 finding 2d, ends with a normal
  "done" under load;
- the booking appears in eon-overseer's list.

A failing item keeps the bead open.

The N3 line also needs work. fx-rpwp r5's comment asked whether a try lost to a cancellation
should stay uncounted. The bead quietly adopts the current behaviour ("counts as one try"). That is
a legitimate technical choice for the coordinator, but record it on fx-ic2 with its reason:
liveness only, never scoring, and harness code stays unchanged after review. Otherwise it reads as
an unanswered question.

### 3. Blocking: fx-ic2.3's Acceptance closes on a no-go, which unblocks agent spending in fx-ic2.4; the pilot trials also lack the per-trial Acceptance

- **The no-go problem.** The Acceptance is "the owner's recorded go or no-go". If it records a
  no-go and closes, br makes fx-ic2.4 ready, and an autonomous or session-runner pass could start
  trials 4-40: the spending decision passes to an agent by default. **Fix:** the Acceptance is a
  recorded go with its batch schedule. On a no-go or no answer, follow AGENTS.md's Autonomous runs
  rule: comment, set `--status blocked --add-label owner`, end the run. The bead never closes on a
  no-go. As defence in depth, add a line to fx-ic2.4's Scope: "start only if fx-ic2 records the
  owner's go".
- **The missing trial Acceptance.** Trials 1-3 are real gate trials, but .3 has none of the trial
  rules that .4-.6 carry: a fresh isolated session, the §4.2 budget, a booked slot, and the
  infrastructure-fault rerun. Its Acceptance also does not require the first snapshot, final
  snapshot, run log and grading inputs on disk. **Fix:** copy the batch Scope and the per-trial
  Acceptance into .3.
- **What the go can change.** At `init`, coordinator.py fixes `jobs`, model, CLI and session limit
  ("jobs fixed at init", "writer identity is fixed at init"). §3 lets the model or effort differ
  from the default only if fx-ic2 records it before the first session. So the owner's post-pilot
  go can set only the calendar and the batch split. It cannot cut cost through a cheaper effort or
  a different parallelism. **Fix:** say this in .3's Scope and in the message to the owner. If a
  different effort or parallelism might be wanted for cost reasons, it has to be asked before
  `init`, in fx-ic2.1. Also say what a no-go means: the three trials stand, and E1 can report
  nothing until the run resumes or a fresh task set is used.
- **The name.** "Pilot" is the name protocol §9 gives to sessions that are metadata-only, labelled
  "pilot" everywhere, and never counted. Calling counted gate trials a pilot invites a grader or
  the report to mislabel or drop them. Rename the bead to something like "E1 gate trials 1-3 and
  cost checkpoint", and say it is not §9's pilot.

### 4. Blocking: "every session's build goes through heavy.sh" cannot work as written, and batches of 37-40 trials do not fit one booked slot

Writer `./check` and `./test` run inside fx-rpwp's jail, called through the broker.
coordinator.py has no heavy.sh path, and it sets `CARGO_BUILD_JOBS=4` in the writer environment
(line 349). Sending each writer build through heavy.sh would take a harness change after review.
It would also let heavy.sh hold a writer's build during a quiet window, or under `one-slot-until`,
while that writer's 40 minutes of wall time keep running (§4.2). That is an infrastructure effect
that does not hit both arms equally, because the arms' build times differ. heavy.sh also sets its
own `CARGO_BUILD_JOBS=6`, and grading inherits that environment (fx-rpwp r4 finding 5(b)).

On scheduling, positions 4-40 run up to about 25 hours of writer time at 40 minutes each, divided
by `jobs`. AGENTS.md and the heavy-run agreement forbid any run still going when a booked window
starts. The batch beads say "stops before any booked Eon window" but not how. A run killed in the
middle of a trial (a heavy.sh `timeout` or an operator Ctrl-C) is an operator cancellation. It is
not in fx-rpwp's closed fault list (decisions 3 and 9), and it uses up grading tries (N3).

**Fix**, in fx-ic2.3-.6:

- Each slot runs one `coordinator.py run --batch N` on the pinned worktree. The whole command goes
  through heavy.sh in AGENTS.md's form, in an E1-exclusive slot booked with eon-overseer, with
  `CARGO_BUILD_JOBS` equal to the value the identity record pins.
- Choose N so that ceil(N / jobs) × (40 minutes + measured grading time) ends before the slot
  does, and set heavy.sh's timeout above that. Never stop a run mid-trial on purpose.
- One batch bead may span several slot runs. Say so, and allow the coordinator to re-split the
  ranges from the times measured in the first three trials.
- Before each slot, rerun fx-ic2.2's preflight: load below 12, no concurrent heavy I/O,
  cargo-sweep not due, tmpfs TMPDIR. Record its output in the batch comment. Today only the pilot
  slot gets these checks, but the Sunday 03:00 timer and Eon load recur across the weeks the run
  spans.
- Decide `--jobs` before `init` (finding 1) against the two-agent cap and Eon's priority, and
  record whether writer sessions count toward the cap. eon-overseer has to agree.

### 5. Blocking: the batch beads omit the audit reporting fx-ic2's comments require

fx-ic2's comment of 2026-11-12 13:31 (fx-rpwp research §4) sets these rules:

- Item 4: only the closed fault list counts as an infrastructure fault. Budget refusal, a deadline
  kill, an early stop and wrong or empty code do not.
- Item 5: a tool outside the allowed set is an infrastructure fault with one fresh rerun. A denied
  outside-path request is counted information per arm. An undenied outside access is a
  confinement breach that stops the whole run.
- The comment ends "Report each on fx-ic2."

§3 also says a second fault in the same slot counts as a failure. The batch Acceptance today asks
only for "counts, reruns and the quota drawn". coordinator.py refuses to continue after a breach
(`ConfinementBreach`, "operator action required"), and no bead says what happens next.

**Fix**, in fx-ic2.3-.6: add the closed-list rule to the Scope. Make the batch comment list each
fault with its class, each rerun, each second-fault failure, each tool outside the set, and the
denied outside-path requests per arm. On a confinement breach, stop, record it on fx-ic2, and set
the bead to blocked with the `owner` label: a breach can invalidate the gate.

### 6. Minor (but large): fx-ic2.7 leaves out report contents §6-§8 require, names an incomplete review, and is too big for one session

**Missing from its Scope and Acceptance:**

- §8's information-only 95% intervals: 10,000 task-level bootstrap resamples, percentile, using
  `random.Random(20261031).choices`, with a statement of how wide they are;
- every trial's §7.4 record published;
- the commands that reproduce grading from the stored snapshots;
- §7.3's details: the mean over passing trials only; threshold 3 not met if either arm has no
  passing trial; the all-60 mean with each failure counted as 20, as information;
- §7.4's classification of every first-check and final failure with its evidence, with a
  toolchain defect flagged and never excluded;
- the per-trial concurrency review, with cited lines;
- structure checks met only by a comment;
- any §6 suite-test removal, which needs a disputed result and must apply to both arms;
- the counts of infrastructure reruns and audit findings (finding 5).

As written, "measures, the gate verdict and reproduction identities" would accept a report missing
all of these.

**Review.** "Passes a review round" should name the Merge Rule's sequence: a Codex pass, then an
Opus round, because a report is a docs change, not planning-only. Any new script for measures or
the bootstrap also gets the code converge round (AP-PROOF-001).

**Operations.** Any re-grading run inherits the gate-operation conditions (tmpfs TMPDIR, no
concurrent I/O, pinned jobs): the 10-second test limit applies at grading.

**Size.** This bead covers aggregating 120 trials, reviewing every integrity flag, about 28
concurrency reviews (C1-C6 and C9, 4 trials each), classifying up to about 120 failures, the
bootstrap, writing the report, and a two-stage review with fixes. That will not fit one fresh
session under fx-ic2's ~100k coordinator bound. Split it, for example:

- (a) measures, integrity-flag review and concurrency reviews, with per-trial subagents and raw
  logs kept out of the coordinator's context;
- (b) §7.4 classification;
- (c) the report, the bootstrap, reviews and publishing.

### 7. Minor: fx-ic2.1's trial-order and identity items are looser than what records them

- Order: the bead says "shuffled with random.Random(20261031)". It leaves out §3's sort and the
  research item 1 requirement to record "all 120 triples and the full hash from state.json". The
  code already sorts, so this is a wording gap only. Fix: cite `coordinator.py init` and record
  `state.json`'s `order` and `order_hash`.
- Identities: "Identities from fx-rpwp" should name §10.2 item 6 in full: the §5 toolchain
  identities, the exact model and harness version, and the effort, explicitly
  (§3: `claude-opus-5-5` at the harness's default effort unless fx-ic2 records otherwise). Since
  .3 exists because of cost, record the effort choice here, before `init`.
- FX exclusion: say that item 3 records "excluded by owner decision 2026-11-09" for FX0920, FX0921
  and FX0516, as the 2026-11-09 comment requires.

### 8. Minor: fx-ic2.1 could freeze tasks.md

§2 forbids task-text changes once the first writer starts, and requires any earlier precision fix
to be recorded on fx-ic2. Recording tasks.md's SHA-256 at the readiness revision, and confirming
that every change since the fx-gfq freeze has a recorded reason, makes that rule checkable. A nit;
§6's tree hash may already cover it.

### 9. Minor: fx-ic2.2's booking only covers the pilot

That is fine as long as finding 4's per-slot booking and preflight live in each batch bead. Say
"pilot" in .2 to mean trials 1-3, consistent with finding 3's rename.

DONE

## Re-check

Model: claude-opus-5-5 · Date: 2026-11-13

**Verdict: approve.** Findings 1-9 are all resolved. Two nits follow; neither blocks or needs a
re-check.

Re-checked against `br --no-db show fx-ic2.1` to `fx-ic2.11` (bodies, Acceptance, edges) and
drafts/DONE.

1. **Resolved.** fx-ic2.8 holds the harness code: closure bind, store-path flag, memory caps,
   pinned build jobs, TMPDIR. It lands under the Merge Rule, with harness_test, the canaries and the
   dry run on the landed tree. The edges are now fx-ic2.8 → .1 → .2 → .3. fx-ic2.1 pins
   `.worktrees/fx-ic2` at R and runs `init`. The probe sits in .2, after `init`. A later harness fix
   redoes .1 with a fresh run directory. .1 item 5 reuses .8's proof only when the diff is empty.
2. **Resolved.** fx-ic2.2's Acceptance now requires each item to pass, each with a named check, and
   a failing item keeps the bead open. The probe has to end "done" under load. The N3 decision is
   recorded with its reason. The canary, memory-cap and jobs checks moved to .8's Acceptance, next
   to the code they test.
3. **Resolved.** The bead is renamed and marked as not §9's pilot. It closes only on a recorded go;
   a no-go or no reply sets it blocked with the owner label. It now carries the per-trial Scope and
   Acceptance, and says what the go can and cannot change and what a no-go means. .4 checks for the
   go, and .5 and .6 run only within its quota weeks. The effort and J question moved into .1,
   before `init`.
4. **Resolved.** Every slot runs `coordinator.py run --batch N` once, through heavy.sh in AGENTS.md's
   form, with the pinned jobs value. Writer builds stay in the jail. The beads give a sizing formula
   with a rerun margin and a timeout no later than the slot's end. No run is stopped mid-trial, and
   a run still queued is cancelled before any trial starts. The preflight reruns before every slot.
   Batches may span several slots and be re-split. .1 records J with eon-overseer's agreement and
   whether writers count toward the cap.
5. **Resolved.** .3-.6 carry the closed-fault list, the rerun and second-fault rules, tools outside
   the set, denied paths per arm, and a breach (blocked, owner label, end the run). .6 adds the
   post-run store scan to its Acceptance.
6. **Resolved.** The old .7 is split into .7 (integrity flags, concurrency reviews, suite disputes,
   and regrading under gate conditions), .9 (a reviewed script for measures and the bootstrap, with
   §7.3's details and a fixture with known answers), .10 (§7.4 classification, toolchain defects
   flagged) and .11 (every report item, durable snapshots, reproduction commands with a check, and
   the Codex-then-Opus rounds for the docs and for any script).
7. **Resolved.** .1 records the order from `init`'s state.json: all 120 triples and the full hash.
   Item 6 lists every identity field, including the effort. Item 3 records the FX0920, FX0921 and
   FX0516 exclusion.
8. **Resolved.** .1 records tasks.md's SHA-256 and checks its change log since fx-gfq without
   reading the file.
9. **Resolved.** .2 books only the slot for trials 1-3, and each batch books its own slots and
   reruns the preflight.

### Nits

- **N1. Placeholders left in bodies.** These still need the real IDs, as the author's note (c) says:
  - fx-ic2.1 lines 17, 32, 42 and fx-ic2.2 line 10: "the harness-hardening bead" → fx-ic2.8;
  - fx-ic2.1 line 19: "the report bead" → fx-ic2.11;
  - fx-ic2.11 line 17: "the measures bead" → fx-ic2.9;
  - fx-ic2.8 line 30, fx-ic2.9 line 15, fx-ic2.11 line 15: `.worktrees/<this id>` → the bead's own
    ID.
- **N2. The trials 1-3 slot is booked twice.** fx-ic2.2 books it, and fx-ic2.3's run procedure says
  "Book" again. Have .3 say it uses the slot .2 booked for its first run, and books again only if
  that slot is lost or a rerun spills over.

DONE
