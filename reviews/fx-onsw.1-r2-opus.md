Model: Claude Opus 5.5 · Date: 2026-11-18 · Candidate c291b19 (research note)

## Verdict

**Changes required.** The note is accurate, scoped and well qualified, and Codex r1's citation checks hold. Two findings block, both because the note does not yet carry the owner's direction for this round (items 1 and 2 of the brief). The author can apply the Decisions text below as written. Once it is in, the finder re-checks only those insertions. The other items are non-blocking.

## Scope

Read in full: research/45 (400 lines); AGENTS.md; GOVERNANCE.md (Merge Rule); the bead, parent and comments (`br --no-db`); fx-onsw.2–.7 and fx-qlek.7 from `main:.beads/issues.jsonl`; target/work/r1-codex.md and opus-notes.md. Read against Tide's contracts: design/01 D8, D15 and D17; design/03 §9.6 (REPLAY-1/2, `--strategy`, `--faults swarm`), §9.5's `#[sim_test]` runner text (lines 2114–2130) and SINK-4 (line 754); research/16 §1.7 (sub-streams); research/30 §13.1. Read-only code: `crates/tide-std/src/sim.rs`, `replay.rs`, `sim/manifest.rs:205–235`, `task.rs:61–206` (baton) and `task/scheduler.rs:654–681, 776–787, 894–906` (pick and draws). Checked the saved DPOR text (target/work/dpor2005.txt) at §2.1 and footnote 1 and at §3's backtracking paragraph. No builds, edits other than this file, commits, `.beads/` writes or subagents.

I did not repeat Codex r1's citation spot-checks. Its one non-blocking item (expand SUP, IC, DPOR and HVM at first use) still stands, and the text below applies it where it touches §2 and §3.

## Decisions

### 1. Reframe §2: is SUP worth anything for Tide?

**Decision.** Yes, a little. What is worth something is a classical idea, not a construct: keep alternatives together, share everything they have in common, and pay separately only where they differ. Labels record which choices are correlated. For Tide that idea belongs in tooling. Its best use is exploring alternative executions in `tide sim` (item 2), and its second is search tooling (fx-onsw.7, icebox). It is worth nothing as Tide syntax or semantics, and nothing as a speed claim. The quantum question becomes a side note.

**Reasons.** The IC rules the note already cites (SUP application distributes and inserts a DUP; same-label DUP-SUP projects; different labels commute) describe sharing work across alternatives. That is the mechanism any Tide use would borrow. Tide already has a recorded home for exploring alternative executions: D17's **Snapshot and branch (later)**, which reads "the simulator can snapshot a run and explore alternative schedules and faults from that point" (design/01-design.md:1073). So the idea attaches to an existing Later decision and adds no new surface. The search use is already filed behind a conventional baseline (report 01 R7, report 31, fx-onsw.7).

**Text for the note.** Replace the heading at line 106 and lines 122–143 as follows. Keep lines 108–120 (Evidence), but expand SUP at its first use there as "a superposition node (SUP)".

```markdown
## 2. SUP: is it worth anything for Tide?
```

(Evidence, lines 108–120, unchanged except the SUP expansion.)

```markdown
### What Tide could take from it (inference)

Under the cited rules, SUP's working idea is classical: hold several
alternatives in one place, share every part they have in common, and pay
separately only where they differ. Labels record which choices are correlated.
[H2], [H3] That idea is worth something to Tide, but in tooling, not in the
language:

1. **Exploring alternative executions in `tide sim`.** The simulator could
   share one execution's prefix and branch only at its choice points. D17
   already records this direction as Later ("Snapshot and branch").
   §3 rates the concrete form, superposed seed exploration. [T1] (D17)
2. **Search over candidate values or programs.** This applies only behind a
   conventional interface and only for a real workload (fx-onsw.7).

It is not worth adding to Tide itself. First-class SUP, DUP and collapse would
give every reader a second evaluation model, and the number of alternatives a
representation holds says nothing about speed.

**Side note: the quantum analogy.** The shared name is a name-level analogy.
Quantum states use complex amplitudes and measurement probabilities, and
relative phase can change outcomes through interference. [Q2] The cited SUP
rules specify none of these. [H3] One resemblance is structural: a linear
operator distributes over a sum of states, much as application distributes
over a SUP's branches. That resemblance supplies no amplitudes and no
interference. A classical evaluator could encode quantum mathematics only
with an added state, gate and measurement model, which SUP does not provide.

### Options and recommendation (inference)

| Option | Recommendation and reason | Fit | Cost | Alpha relevance |
| --- | --- | --- | --- | --- |
| Share execution prefixes across alternative runs in `tide sim` | **Later, under D17's snapshot and branch.** The best use of the idea; rated in §3 with DPOR. No language surface. | Medium | Large | No |
| Investigate shared alternatives in an external pure search tool | **Later, conditional on a real search problem.** Compare a normal enumerator with deduplication before an HVM engine, and keep the engine outside Tide semantics. | Medium | Medium | No |
| Add first-class SUP/DUP and collapse to Tide expressions | **Reject for Tide's core.** Labels, distribution and collapse add another evaluation model for all readers. | Low | Large | No |
| Treat SUP as quantum superposition or a quantum speedup | **Reject (side note).** The cited semantics supplies neither interference nor measurement. | Low | Large | No |
```

Then keep lines 140–143 (the search comparison's falsifier) unchanged.

Also replace the summary's lines 6–8, from "SUP has substantive" to "limited structural resemblance.", with:

```markdown
SUP is worth something to Tide as a classical idea, sharing work across
alternatives until they differ. It applies to simulator exploration and to
search tooling, not to the language. Its quantum name is a name-level analogy.
```

### 2. Superposed seed exploration in `tide sim`, next to DPOR

**Decision.** Rate it **Later**: fit **Medium**, cost **Large**, alpha relevance **No**. It **complements** DPOR (dynamic partial-order reduction). Neither subsumes the other. Fold it into **fx-onsw.4**, retitled "Schedule-space exploration: DPOR and/or shared-prefix forking". Do not file a separate bead.

**Reasons, checked against Tide's contracts and code.**

- *Where it fits.* D15 has `tide sim` run `main` under many seeds and replay a failing one bit for bit. D17's Later "Snapshot and branch" is the decision that owns exploring from a mid-run point. 03 §9.6 already treats exploration as a simulator option (`--strategy random|pct`, `--faults swarm|none|<plan>`). A new strategy adds no language surface and changes no v0 contract.
- *Choice points are discrete and visible.* The scheduler draws only when two or more tasks are runnable (`scheduler.rs:654–681`) and records `runnable` and `chosen` in the trace. Network and process draws use separate sub-streams (`scheduler.rs:780–787`; research/16 §1, sub-streams).
- *The literal form (many seeds, each replayable from its own seed) is sound but yields little.* Each seed drives its own splitmix64 stream (`scheduler.rs:894–906`, `replay.rs:9`), and `#[sim_test]` runs seeds 0, 1, 2… (03, line 2121). Draws at a choice point are therefore effectively independent across seeds. Two seeds agree at a point with b runnable alternatives with probability 1/b. Beyond the prefix before the first choice point, N seeds share only about log_b N levels before most run alone. A `Rand` draw with a wide bound, or a generated test input, ends sharing at once. **Inference:** savings are about (P + log_b N)/(P + L) of a run, where P is the choice-free prefix and L the steps after it. They are large only for tests whose costly setup comes before any concurrency.
- *The productive form changes how a branch is named.* Branching on every alternative at a choice point (bounded exhaustive, or DPOR-guided) shares real work. But its branches are choice vectors, not seeds: no seed can be chosen to produce a given vector. Tide's replay reruns a run from its seed under a pinned scheduler version. It compares the recorded trace (`manifest.rs:213–223`) and records no schedule to force (REPLAY-1/2; `replay.rs` Manifest). So "every branch replayable from its own seed" holds only in the low-yield form. The productive form needs a replay format that records forced choices, which REPLAY-1/2's owner would have to decide.
- *Cost.* The simulator runs each task on its own OS thread, passed a baton by condition variables (`task.rs:61–206`; `"scheduler":"seeded-baton/1"` in `replay.rs` configuration), and `Root::sim` runs in-process as task 0. POSIX `fork` copies only the calling thread, so this simulator cannot be forked at a choice point as built. Sharing a prefix therefore needs one of three things: single-threaded stackful tasks plus process forking, checkpoint and restore, or re-executing the prefix (which shares nothing). Each is a runtime-architecture change.
- *Relation to DPOR.* DPOR decides **which** interleavings to run. Shared-prefix forking decides **how** a chosen set runs: it continues from a shared state instead of re-executing. Flanagan and Godefroid's own algorithm is stateless and names this mechanism: backtracking "by re-executing the program from its initial state, or 'forking' new processes at each backtracking point, or … checkpointing" (P1 §3). Forking alone still runs every equivalent interleaving. DPOR alone pays a re-executed prefix per backtrack. DPOR's presentation also leaves out nondeterministic operations (P1 §2.1, footnote 1), while forking treats fault and random-draw choices the same way as scheduler picks. The two combine; they do not compete.

**Text for the note.** Replace lines 276–285 (from "Opus also recommends" to "supplies a concrete need.") with:

```markdown
#### Exploring the schedule space: DPOR and shared-prefix forking (inference)

The independent Opus partner recommended **Later** for dynamic partial-order
reduction (DPOR) inside `tide sim`. DPOR explores representative
interleavings using an independence relation between transitions. [P1] It
could avoid redundant exploration without adding Tide syntax (fit **high**,
cost **large**, alpha **no**). Named channels, cells and capabilities help
identify candidate dependencies. But separate handles can share a world
resource, and cancellation can affect siblings. An implementation must justify
commutation and enabledness preservation; handle inequality alone is
insufficient. DPOR's presentation also leaves out nondeterministic operations
(P1 §2.1, footnote 1), so random draws and fault choices need their own
treatment.

A second candidate borrows §2's sharing idea: **superposed seed exploration**.
Many seeds run as one shared execution that forks only at nondeterministic
choice points (a scheduler pick, a fault injection, a random draw). Everything
before a fork is shared, and every branch stays replayable. This is a strategy
for D17's Later "Snapshot and branch". Tide's simulator makes the choice points
discrete: the scheduler draws only when two or more tasks are runnable and
traces `runnable` and `chosen`, and network and process draws use their own
sub-streams. [T1] (D15, D17), [T4]

In the literal form, each branch stays a seed, and sharing is small. Each
seed drives an effectively independent stream, so two seeds agree at a point
with b alternatives with probability 1/b. Beyond the choice-free prefix, N
seeds share only about log_b N levels, and a wide random draw or a generated
test input ends sharing at once. Branching on every alternative at each choice
point shares more. But its branches are choice vectors, not seeds. Tide's
replay reruns a seed under a pinned scheduler version and compares the
recorded trace; it records no schedule to force. That form therefore needs a
replay manifest that records forced choices, which REPLAY-1/2's owner must
decide. Both forms also need a simulator that can branch: today each task runs
on its own OS thread, and POSIX `fork` copies only the calling thread. The
options are single-threaded stackful tasks plus process forking, checkpoint
and restore, or re-executing the prefix, which shares nothing. [T4], [X1]

The two ideas **complement** each other. DPOR decides which interleavings to
run; forking decides how a chosen set runs, continuing from shared states
instead of re-executing them. DPOR's own authors list forking new processes at
backtracking points, alongside re-execution and checkpointing, as ways to
implement its stateless search. [P1] (§3) Forking alone still runs every
equivalent interleaving, and DPOR alone re-executes a prefix per backtrack.

| Option | Recommendation and reason | Fit | Cost | Alpha relevance |
| --- | --- | --- | --- | --- |
| DPOR in `tide sim` | **Later.** Prunes equivalent schedules; needs a justified independence relation for Tide's actions. | High | Large | No |
| Shared-prefix forking (superposed seed exploration) | **Later, with DPOR, under D17's snapshot and branch.** Seed-named branches share little; choice-named branches need a replay format decision. Both need a forkable simulator. | Medium | Large | No |

The cheapest falsifiers need no simulator change. First, build the trie of
recorded `sched` choices from N seeds' existing traces of current sim tests,
and measure the shared fraction of steps; a small fraction ends the literal
form. Second, any later branch must produce a trace byte-identical to an
independent replay of the same seed or choice vector; the existing REPLAY-2
trace comparison is that check. Compare either technique with exhaustive
schedules of tiny modelled programs, faults included, before claiming that
bug coverage is preserved. Defer both until simulation cost or missed schedule
coverage supplies a concrete need.
```

Add to the Sources list:

```markdown
- **T4.** Tide simulator and replay contracts: [chapter 03](../design/03-authority-concurrency.md)
  §9.6 (REPLAY-1/2, `--strategy`, `--faults`) and SINK-4; the
  [scheduler](../crates/tide-std/src/task/scheduler.rs) (`pick`, sub-stream draws),
  [task runtime](../crates/tide-std/src/task.rs) (one OS thread per task under a
  baton), [replay manifest](../crates/tide-std/src/sim/manifest.rs) and
  [replay wire format](../crates/tide-std/src/replay.rs).
- **X1.** The Open Group, POSIX.1-2024, [`fork()`](https://pubs.opengroup.org/onlinepubs/9799919799/functions/fork.html):
  a multithreaded process's child contains a replica of the calling thread only.
```

Extend P1's entry with "§2.1 and footnote 1 (deterministic processes; nondeterministic operations set aside); §3, backtracking by re-execution, forking or checkpointing". The author must open and pin X1 before citing it. I give the clause from the POSIX `fork` specification as I know it and have not re-read it in this round.

### 3. The recorded D3-b disagreement

**Decision.** The researcher is right, and the note's qualification at lines 249–257 stands, with three corrections. A deterministic task scope earns a **by-construction guarantee**: for an admitted pipeline, successful outputs cannot depend on scheduling, so readers and agents need not reason about interleavings for that property. It earns **no cut in simulation or replay**. Seed counts stay as they are, and failure, cancellation, sink order and liveness still need exploration.

**Reasons.**

- *Replay saves nothing.* The partner's claimed payoff, that replay "need not record the interleaving", is already true of all Tide replay. The manifest records the seed and pinned scheduler version, re-derives the schedule, and keeps the trace only as a check (`replay.rs:9, 57–72`; `manifest.rs:213–223`; REPLAY-1/2). A deterministic scope could only shorten that check.
- *Failures and cancellation stay schedule-dependent.* D8 cancels siblings on the first failure and reports the earliest-spawned failing child, with echoes excluded (design/01 D8). Which children fail before cancellation reaches them depends on timing, so the reported error can differ between schedules. The partner conceded this.
- *Sinks are a further observable the note misses.* Across tasks, the simulator orders sink records by scheduler order (03 SINK-4). The SPSC candidate excludes clock, random and I/O capabilities but not sinks. A scope must either buffer sinks per task and append them in a fixed order, as parallel regions do, or treat sink output as schedule-dependent.
- *Capacity is a liveness question, not a schedule question.* Line 253 lists "capacity" among things that "expose schedules". That is not shown. Bounded write-blocking changes whether a pipeline completes, and the note gives no source that it makes the outcome schedule-dependent. It belongs with liveness, which simulation still checks.

**Text for the note.** Replace lines 249–257 with:

```markdown
**Decision on the partner's proposed payoff** (Opus review round, 2026-11-18):
a deterministic task scope gives a by-construction guarantee. For an admitted
pipeline, successful outputs do not depend on scheduling. It gives no cut in
simulation or replay. Replay already re-derives the schedule from the seed and
uses the trace only as a check. [T4] Under D8, the first failure cancels
siblings, so which error a scope reports can depend on timing. Across tasks,
the simulator writes sink records in scheduler order (SINK-4), so a scope must
buffer sinks per task or treat sink output as schedule-dependent. Kahn's model
assumes unbounded channels; a bounded Tide pipeline can block on writes, so
whether it completes depends on its capacities and is a liveness question.
The initial contract should therefore claim only deterministic successful
outputs under stated progress assumptions, and keep simulation, at unchanged
seed counts, for failure, cancellation, sink order and liveness. No finite seed
sample proves the general claim. [K1], [T1] (D8/D15), [T2] (§§6.8, 7), [T4]
```

In the falsifier paragraph (line 261), add "interleaved sink output from two stages" after "a failed child". At line 229, write "The independent Opus partner recommended" in place of "Opus recommends".

### 4. Conclusions changed, and the beads to update

No bead closes, and every one of the note's borrow, reject and later dispositions stands. The coordinator applies these through `br` (I made no writes):

- **fx-onsw.4: retitle and extend.** New title: "Schedule-space exploration: DPOR and/or shared-prefix forking (research/45 §3)". Append to Scope: "Includes superposed seed exploration (shared-prefix forking at scheduler, fault and random-draw choice points), as a strategy under D17's Later 'Snapshot and branch'. The two complement each other: DPOR chooses schedules, and forking reuses prefixes. Deciding whether branches are named by seed or by choice vector is part of the work; a choice-vector replay needs a REPLAY-1/2 format decision." Append to Acceptance: "(a) Before any build: the shared-step fraction measured from the `sched` choice trie of N seeds' existing traces of current sim tests. (b) Any branch's trace is byte-identical to an independent replay of its seed or choice vector, through the existing REPLAY-2 trace comparison." Keep it deferred, P3 and post-alpha.
- **fx-onsw.2: comment, no scope change.** "Opus review round (research/45 D3-b): the payoff is a by-construction guarantee on successful outputs. It gives no cut in seeds or replay data, since replay already re-derives schedules from seeds. Add interleaved sink output across stages (03 SINK-4) to the probes: the scope either buffers sinks per task in a fixed order or declares sink order schedule-dependent." It stays P2 and deferred.
- **fx-onsw.7: comment only.** "§2's reframe names the simulator as SUP's better use for Tide (fx-onsw.4). This bead stays icebox." The coordinator may skip this one.
- **fx-onsw.3, .5, .6:** unchanged.
- **fx-qlek.7 fold-ins:** fold-in 1 (copying data, duplicating authority, sharing a borrow) is kept as filed. Fold-in 2 is kept but needs its wording qualified. Safe Rust already rules out data races, so "race-free by construction" undersells the claim and also overstates its proof. Append: "Phrase the paper claim as: D7/D9 parallel regions give schedule-independent successful results by admission (D9's sequential equivalence). This is a checked contract, not a formal confluence theorem; D9's panic and trust-hole qualifications apply; the parallel runtime is partial (par.rs; reference §6.8) (research/45 §3)."

## Blocking findings

1. **research/45-bend-hvm-ideas-for-tide.md:106–143 (and 6–8). §2 does not ask whether SUP is worth anything for Tide.** *Evidence:* the heading and body center on "the superposition analogy". The only Tide-facing option is the external search tool, and the simulator use is missing. The relayed owner direction (fx-onsw.1 comment, 2026-11-18 22:48Z, quoting Lucca: "what matter is inspiration for tide. if worth anything. not if it is quantum or not") requires the reframe. *Minimum fix:* apply Decision 1's text.
2. **research/45-bend-hvm-ideas-for-tide.md:276–285. Superposed seed exploration is not rated, and D17's "Snapshot and branch", the decision that owns it, is not cited.** *Evidence:* the DPOR paragraph is the note's only schedule-space option. design/01-design.md:1073 already records "explore alternative schedules and faults from that point" as Later. The same relayed direction requires a fit, cost and alpha rating next to DPOR. *Minimum fix:* apply Decision 2's text, with the T4 and X1 sources and the P1 extension.

## Non-blocking findings

1. **:249–257. The D3-b qualification has one unsupported item and two gaps.** "Capacity … can expose schedules" is not shown. The note also misses the sink-order observable (SINK-4) and implies that replay records schedules today ("or that replay may omit schedules"), which it does not. Decision 3's text fixes all three. These are folded into the required edit because the decision must land anyway.
2. **:12–14. Stale status.** "The coordinator's Merge Rule reviews remain pending" is wrong once the note lands. Replace it with "Reviewed under the Merge Rule (Codex r1; Opus review round 2026-11-18, which decided the §3 D3-b disagreement)."
3. **:229, :276. "Opus recommends" is ambiguous.** It means the partner, but Opus also decides in this round. Write "The independent Opus partner recommended" (applied in the Decision 2 and 3 texts).
4. **Codex r1's abbreviation item** (SUP, IC, DPOR, HVM at first use) is still open. Decisions 1 and 2 cover SUP and DPOR. Expand IC at :51 ("Interaction Calculus (IC)") and HVM at :5 ("HVM, the Higher-order Virtual Machine").
5. **:375–383. Sources.** T1–T3 do not cover chapter 03 or the simulator files that §3's D15 claims and the new text rely on. T4 (Decision 2) covers them.

Tone check: no hype, and no judgment of other projects' authors. Scope check: every recommendation, including the new text above, is Later or Reject research and authorizes no v0 language, reference, std or simulator change. D16 stays unchanged.

## Reasoning

The note's claims match their sources. Codex r1 spot-checked 21 of them, and I re-checked the DPOR passages that the new text relies on (P1 §2.1 and footnote 1, §3's backtracking paragraph). The research is careful. What it lacks is the owner's question: what can Tide use? For SUP, the honest answer is a mechanism, sharing across alternatives, and Tide already has a Later decision that wants that mechanism (D17 snapshot and branch). That makes item 1 and item 2 one finding seen from two sides. §2 should point at the simulator, and §3 should rate the simulator use.

The rating rests on reading the simulator rather than the idea's name. Seeds drive independent PRNG streams, so "many seeds as one execution" shares little beyond setup. That conclusion falls out of `scheduler.rs`'s splitmix64 draws, and a trie of existing traces can measure it cheaply. The version that shares real work branches on choice values, so it leaves seed identity behind and runs into REPLAY-1/2's seed-based replay. Either version runs into the one-OS-thread-per-task baton runtime, which cannot be forked mid-run. Hence fit Medium (it has an owning decision and no surface, but it strains the replay identity) and cost Large (a runtime-architecture change). It is not alpha work. DPOR's paper settles the relation itself: forking is one of the backtracking mechanisms it lists, so the two are layers of one design, which supports the combined fx-onsw.4 title.

On D3-b, the partner's payoff fails on Tide's own replay design, which never recorded schedules. The real value of a deterministic scope is the guarantee it gives readers, not a saving in simulator time. That keeps fx-onsw.2 alive at its current priority with a corrected rationale.

DONE
