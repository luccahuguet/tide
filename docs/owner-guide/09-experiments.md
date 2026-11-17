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

### E3: in review, not yet run

E1 suggests models can write Tide at little extra cost, on small tasks
with one model. It doesn't show Tide is worth adopting.
That's E3's job, and the alpha leads with it.

The pilot's protocol is **in review** (bead fx-1vwf.1). As drafted:

- **Codebase:** about 1,900 lines of Tide that already existed (the demo
  service and E1's reference solutions), so nobody wrote it with the test in
  mind.
- **12 pairs of changes.** In each pair, a **twin** does exactly what its
  description says, and a **plant** does the same plus quietly takes
  authority the description doesn't call for. A separate agent writes them
  and never reviews.
- **Six ways to plant**, two pairs each. Three the v0 trust diff can show (a
  new capability parameter, a capability field in a type, a new function that
  holds one). Three it **can't**: less attenuation, new use of authority
  already held, and leaking data through printed output. The design
  deliberately includes plants the tool is blind to.
- **Two arms:** reviewers get either the code diff alone, or the code diff
  plus the trust diff. Reviewers are fresh Opus and Sol sessions that see
  only their prompt.
- **The bar is D0's:** the trust-diff arm catches at least 1.5 times as many
  plants as the code-diff arm. False positives (flagging a clean twin) are
  counted too.
- **The result is published whatever it is.** The owner decided on
  2026-11-17 that if E3 misses the bar, the alpha still launches and states
  the negative result.

The plants (fx-1vwf.2) and the review runs (fx-1vwf.3) come after the
protocol lands.

<!-- TODO(tide-lead, fx-1vwf.1): when the protocol lands, link
trials/e3-pilot/protocol.md, check the numbers above against it, and update as
fx-1vwf.2 and fx-1vwf.3 land (E3's result). -->

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

## What to look at

- `E1 measures reproduced exactly`: every number in the table above is
  recomputed from the 120 stored sessions.
- In the report, the sentence "It is not a confidence-bound claim". That is
  the honest framing to use on a call.

## The hardest question

> "E1 shows models write Tide as well as Rust. Parity isn't a reason to
> switch. Where's the benefit?"

**Agree with the premise.** E1 is the cost line: switching to Tide costs a
model little on these tasks. It was never the reason to switch. The reason is
what Tide gives a reviewer and a tester: authority you can read in a
signature, replay of concurrency bugs, and the trust diff. E3 is the first
measurement of that benefit, and it is designed to be hard to pass: half its
plants use mechanisms the v0 trust diff isn't expected to show, and the result is published
either way.

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

## Going deeper

- `design/01-design.md` D0 (the validation program and thresholds).
- `trials/e1-checked/gate-report.md` (E1, with its limitations section).
- `br show fx-1vwf` (E3's plan) and `br show fx-1vwf.1` (its protocol, in
  review).
- `reviews/2026-11-15-achilles-opus.md` W2 (the critique E3 answers).
