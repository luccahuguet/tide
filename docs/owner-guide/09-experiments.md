# 9. E1 and E3: what they measure, and what they showed

## The idea

Tide's design is a hypothesis, and decision D0 says so: before the syntax
freezes, experiments test it, each with a threshold set in advance. Two
matter most for the alpha.

- **E1, learnability: what does Tide cost?** Can an AI model write Tide as
  reliably as Rust, given only Tide's reference? This is the *cost* side.
- **E3, review: what does Tide buy?** Do reviewers catch planted
  authority-widening changes more often with the trust diff (chapter 7) than
  with the plain code diff? This is the *benefit* side.

### E1: done, passed, with honest margins

Set-up (`trials/e1-checked/`): 30 small tasks, each written twice in Tide and
twice in Rust, 120 sessions in all. Each was a fresh `claude-opus-5-5`
session with a budget of 20 check or test runs and 40 minutes. The Tide
writer got only the reference (`design/reference.md` and its companion); the
Rust writer got nothing extra. The task order and the thresholds were frozen
before the first session. Three measures:

| Measure | Tide | Rust | Threshold | Verdict |
| --- | ---: | ---: | --- | --- |
| **F**, first check passes | 56/60 (93%) | 59/60 (98%) | Tide ≥ 0.8 × Rust | met (ratio 0.95) |
| **P**, final pass | 58/60 | 58/60 | Tide ≥ Rust | met (tie) |
| **I**, iterations to green | 1.48 | 1.02 | Tide ≤ 1.5 × Rust | met (ratio 1.46) |

What it shows: **models write Tide about as reliably as Rust**, on these
tasks, with this model. What it doesn't, as the report itself says:

- Tide never beat Rust. Rust passed first check more often, and Tide's
  iteration ratio of 1.46 sits just under the 1.5 limit. The statistical
  interval for it (1.29–1.64) crosses 1.5.
- Both arms were near the ceiling, on small single-file tasks with median
  sessions of 30–40 seconds. Harder tasks might separate them.
- One writer model. The owner declined a second-model run.
- Tide sessions used about 9 times Rust's cached input tokens: the cost of
  loading a 25,000-token reference every session.
- All four final failures were literal structure-check slips, two per arm,
  not wrong behavior.

### E3: protocol frozen, no data yet

E1 suggests models can write Tide at little extra cost, on small tasks
with one model. It doesn't show Tide is worth adopting.
That's E3's job, and the alpha leads with it.

In three sentences: twelve small changes each come in two versions, an
honest one and one that quietly takes extra authority. Fresh AI reviewers
review every version, both with the trust diff and without it. If reviewers
with the trust diff catch the sneaky version at least 1.5 times as often,
D0's bar is met.

E3's pilot protocol landed on 2026-11-17 (bead fx-1vwf.1), before any
plant existed: [`trials/e3-pilot/protocol.md`](../../trials/e3-pilot/protocol.md).
It is now frozen: the arms, prompts, budget, models, sample, seed, grading
rules, threshold and statistics don't change. Before the first review
session, the only change allowed is one the run can't proceed without,
recorded on fx-1vwf.3 and drawing on no plant's content; that includes
fixing the judges' prompt if they fail a control. After the first session
nothing changes, and a change the run would need voids it. **No review has
run yet, so E3 has no data.** The design:

- **Codebase:** about 1,900 lines of Tide that the driver admitted before
  E3 existed (the demo service and E1's reference solutions), so nobody
  wrote it with the test in mind.
- **12 pairs of changes, 24 in all.** In each pair, a **twin** does exactly
  what its description says, and a **plant** does the same plus quietly
  takes authority the description doesn't call for, by exactly one
  mechanism. Both share the same description. A separate agent writes them
  and takes no part in reviewing or grading.
- **Six mechanisms, two pairs each.** Three the v0 trust diff is expected
  to show: a new capability parameter on an exported function, a capability
  field in an exported type, a new exported function that holds one. Three
  it **isn't**: less attenuation, new use of authority already held, and
  data sent out through printed output. The design deliberately includes
  plants the tool is blind to: the protocol expects any gain to come from
  the visible half, with the arms close on the rest, while all twelve
  count toward the bar. In one pair per mechanism the description itself
  asks for an authority change, so rejecting every non-empty trust diff
  would also reject clean twins.
- **Two arms.** An arm is one of the two conditions a reviewer is put in.
  Both receive the change's description, the code diff, and the complete
  program after the change, with line numbers. That alone is the **plain**
  arm. The **trust** arm also receives the trust diff and a fixed paragraph
  on what it does and doesn't show.
- **Reviewers:** Opus (`claude-opus-5-5`) and Sol (`gpt-6.1-sol`). Each
  session is fresh: one prompt, one reply, no tools, at most 15 minutes.
  Every item is reviewed in both arms by both models three times: **288
  sessions**, 72 of them on plants in each arm, in an order shuffled from a
  fixed seed. The 12 planted changes are the experimental units; the
  sessions are repeated reviews of them.
- **What counts as a catch:** the reviewer rejects the plant with a finding
  within 3 lines of the planted code, whose authority label (each finding
  carries one from a fixed list: `fs`, `net`, `clock`, and so on) is one of
  the kinds sealed for that plant, and **two judges agree** it identifies the
  widening. The judges (the protocol calls them adjudicators) are fresh
  Opus and Sol sessions; a disagreement counts as no catch. The rule is
  keyed to the plant's mechanism, never to the arm: for the three visible
  mechanisms, naming the capability the item gains is enough; for the other
  three, the finding must name the action, and the data or destination when
  those are involved. Judges never see the arm or the trust diff, and
  trust-diff wording in a finding is masked. The protocol calls this blind
  to the arm's labels, not proven blind to the arm. The report gives each
  judge's agreement by reviewing model, so a judge favouring its own
  model's reviewers would show.
- **Frozen controls:** two control changes with six findings and their
  expected answers. Before the first session both judges must answer every
  one as expected; if they don't, the run doesn't start until the judges'
  prompt is fixed and both judges pass.
- **The bar is D0's:** the trust arm catches the plant in at least 1.5
  times as many sessions as the plain arm (and in more, so zero against
  zero fails). Like E1's, it is a point estimate on a fixed sample; no
  significance test is part of it. The report adds, without gating on them:
  - an **interval**: a range for the true ratio, computed by resampling the
    12 changes, where a ratio of 1 means no difference. If its low end is 1
    or less, or it can't be estimated, the report must add "The interval
    does not rule out no benefit.";
  - a **paired test** that compares the two arms on the same change, model
    and replicate;
  - **false positives** (rejecting a clean twin). If the bar is met and the
    trust arm's false-positive rate is more than 0.10 above the plain
    arm's, the report must say the trust arm also rejected more clean
    changes.
- **Stopping rules:** all 288 sessions run; nothing stops early or is added
  for a result, and nobody computes a result until every session has its
  outcome. A model that hasn't finished 14 days after the first session
  isn't counted, and the report says so first. The judges also have 14
  days, with no stand-in. An isolation breach halts the run. No pair is
  dropped afterwards.
- **The result is published whatever it is,** in a sentence the protocol
  fixes in advance. If E3 misses the bar, the alpha still launches and
  states the negative result. That call is recorded on fx-qlek as yours,
  made by delegation: on 2026-11-17 you said "autodecide" on the alpha's
  gate calls, and tide-lead took this recommendation.
- **It's a pilot:** model reviewers only, one small codebase, and v0's
  partial trust diff. A met bar would be a result for this setting, not a
  general claim about review with Tide.

Built with the protocol:

- **The `e3` harness** (`trials/e3-pilot/harness`, Rust). It writes trust
  diffs through the driver, freezes the session order, prints any session's
  prompt exactly as sent, runs each reviewer in a fresh isolated session,
  and records every attempt. The grading commands come with fx-1vwf.3,
  which may add them without changing the review path or the prompts.
- **The isolation canary passed at landing (2026-11-17).** `e3 canary`
  plants instruction files and hooks where each CLI looks for them. In the
  run recorded in commit 9b080c3, a plain control run of each CLI picked up
  planted instructions and hooks, and the isolated attempts, launched the
  way reviews are, picked up none. It proves isolation only for what the
  controls picked up. Two of Claude's planted probes didn't fire even in
  its control, so isolation from them is unproven: a `CLAUDE.md`
  instruction file in `~/.config/claude/`, and hooks in a parent
  directory's `.claude/settings.json`. The canary runs again before the
  first session and after every breach fix.

Still to come:

- **The plants** (fx-1vwf.2, in progress): the 12 pairs, sealed before any
  review.
- **The runs, grading and report** (fx-1vwf.3): the controls, the 288
  sessions (the protocol budgets about 5 to 15 hours, one at a time), the
  judges, and the published result.

## Run it

E1's results are reproducible from the published records, with no model and
no compilation. This unpacks the archive, verifies every file's hash,
recomputes all the measures, and checks they match the published ones byte
for byte (a few seconds):

```sh
bash -c 'set -e; (cd trials/e1-checked/gate-results && sha256sum -c --quiet SHA256SUMS); python3 trials/e1-checked/gate_reproduce.py unpack target/guide/e1-records; python3 trials/e1-checked/measures.py target/guide/e1-records target/guide/e1-measures; cmp trials/e1-checked/gate-results/measures.json target/guide/e1-measures/measures.json; cmp trials/e1-checked/gate-results/per-trial.json target/guide/e1-measures/per-trial.json; echo "E1 measures reproduced exactly"'
```

Output:

```text
Verified archive and members; relocated 120 slot records
Wrote measures.json and per-trial.json
E1 measures reproduced exactly
```

The scripts refuse an existing destination. To run it again, delete
`target/guide/e1-records` and `target/guide/e1-measures` first.

Then read the top of the report, which states the verdict and its limits:

```sh
bash -c 'sed -n "1,45p" trials/e1-checked/gate-report.md'
```

E3 has no results to reproduce yet. Read what its protocol says it can
conclude, and its gate:

```sh
bash -c 'sed -n "/^## 1\./,/^## 2\./{/^## 2\./!p};/^### 10\.2/,/^### 10\.3/{/^### 10\.3/!p}" trials/e3-pilot/protocol.md'
```

## What to look at

- `E1 measures reproduced exactly`: every number in the table above is
  recomputed from the 120 stored sessions.
- In the report, the sentence "It is not a confidence-bound claim". That is
  the honest framing to use on a call.
- In E3's section 1, "A met threshold is a result for this setting, not a
  general claim about review with Tide." In 10.2, "This is a point estimate
  on a fixed sample, as E1's gate was."

## The hardest question

> "E1 shows models write Tide as well as Rust. Parity isn't a reason to
> switch. Where's the benefit?"

**Agree with the premise.** E1 is the cost line: switching to Tide costs a
model little on these tasks. It was never the reason to switch. The reason is
what Tide gives a reviewer and a tester: authority you can read in a
signature, replay of concurrency bugs, and the trust diff. E3 is the first
measurement of that benefit. Half its plants deliberately use mechanisms the
v0 trust diff isn't expected to show, so the protocol expects any gain to
come from the other half, and the result is published either way. The bar
is a point estimate; the report also gives an interval over the 12 changes
and must say when it doesn't rule out no benefit. Its protocol is frozen,
but no review has run, so it has no data yet. Don't hint at a result.

**If asked "and if E3 fails?":** then the pilot missed its preset 1.5×
target, and Tide says so, with the actual catch rates, false positives and
their uncertainty. Missing the target doesn't by itself show there's no
benefit: a 1.3× improvement misses it, and a small pilot can be
inconclusive. Report what was measured; don't round it up or down. The other benefits
(replay, the sandbox, readable authority) stand on their own demos, and the
diff's planned v1 parts (exercised authority, behavior diffs) are the next
thing to measure.

**Also be ready for:** "E1 is all-Claude." True: one writer model, and the
grading chain was Claude-reviewed too. A second writer model is recommended
before quoting E1 externally (Achilles W2).

**And for:** "In E3, AI judges grade AI reviewers." True, and the protocol
says so. Two judges from different model families must both agree, both
must pass frozen controls before the run, and the report gives each judge's
agreement by reviewing model.

## Going deeper

- `design/01-design.md` D0 (the validation program and thresholds).
- `trials/e1-checked/gate-report.md` (E1, with its limitations section).
- `trials/e3-pilot/protocol.md` (E3's frozen protocol) and
  `reviews/fx-1vwf.1-*.md` (the reviews it passed before it landed).
- `br show fx-1vwf` (E3's plan), `br show fx-1vwf.2` (the plants) and
  `br show fx-1vwf.3` (the runs, grading and report).
- `reviews/2026-11-15-achilles-opus.md` W2 (the critique E3 answers).
