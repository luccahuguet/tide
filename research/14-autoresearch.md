# 14: Karpathy's autoresearch loop and where it fits Tide

Research date: 2026-10-19. Scope: what Andrej Karpathy's `autoresearch`
does, what its results and critics report, and which parts apply to Tide
(`design/01-design.md`, draft 3.4). **Inference** marks reasoning that no
source states.

Method: the repository README and `program.md` were read directly [A1, A2];
results and critiques come from secondary write-ups [A3–A6], each claim
checked against the page cited, and every figure in them is the author's
claim. Karpathy's own posts were not reopened.

---

## Question

Autoresearch lets an agent run hundreds of experiments overnight with no
human in the loop. Could Tide's design or toolchain work run that way, and if
not, which parts of the method should Tide take?

---

## What it is

Released on 7 March 2026 [A4] under the MIT license [A3]. Three files [A1]:

- `prepare.py`: data download, tokenizer, data loader, and the evaluation
  function `evaluate_bpb`. Read-only; `program.md` calls it "the ground truth
  metric" and forbids changing the evaluation harness or adding dependencies
  [A2].
- `train.py`: the GPT model, optimizer, and training loop. The only file the
  agent edits; "everything is fair game" [A1].
- `program.md`: the human's instructions, "a super lightweight skill" [A1].
  The human iterates on this file, not on the Python.

The loop [A2]:

1. Create a fresh branch `autoresearch/<tag>`.
2. Edit `train.py`, commit, train for a fixed 5 minutes of wall time.
3. If `val_bpb` (validation bits per byte, vocabulary-independent) improved,
   keep the commit; if equal or worse, `git reset` to the start.
4. Log every attempt to an untracked `results.tsv`
   (`commit val_bpb memory_gb status description`, status
   `keep | discard | crash`).
5. Fix trivial crashes and rerun; log broken ideas as `crash` and move on;
   kill runs over 10 minutes.
6. "Do NOT pause to ask the human if you should continue."

A simplicity rule weighs every change: "A small improvement that adds ugly
complexity is not worth it", and deleting code at equal or better score is a
win [A2].

## What it found

- Karpathy: about 700 experiments over two days and about 20 genuine
  improvements; time to GPT-2 quality fell from 2.02 h to 1.80 h [A5]. One
  find was a latent bug, a missing scalar multiplier in QK-Norm [A5].
- Other runs: a community session of 126 experiments, validation loss
  0.9979 to 0.9697 [A3]; 198 experiments with no human intervention on
  OpenShift [A6]; Tobi Lütke reports a 0.8B model scoring 19% higher than
  his hand-tuned 1.6B baseline [A5].
- Critiques: many runs scored on one evaluation carry "an overfitting risk:
  some improvements may be specific to that eval rather than genuine gains"
  [A3]; critics invoke Goodhart's law [A4]; the search bets on the model's
  general knowledge, and its training makes it conservative [A3].
  `program.md` itself says nothing about metric gaming beyond making the
  evaluator read-only [A2].

## Why the loop does not fit Tide's design work

1. **No cheap, objective metric.** Tide judges design by review rounds and
   held-out learnability trials (D0). A trial arm is a fresh writer agent
   plus an LLM grader; the last one could not rank its arms because it had
   one writer per arm (`trials/e1-origins/grading.md`). That is neither
   5 minutes nor a comparable scalar.
2. **Optimizing against held-out tasks spends them.** **Inference.** A loop
   that edits `design/reference.md` until the trial tasks pass teaches to
   those tasks, and D0's evidence depends on their being held out. This is
   [A3]'s overfitting critique with a smaller task set and a costlier
   replacement.
3. **It conflicts with the imported protocols.** "Never stop, never ask"
   meets AP-SCOPE-001 (the owner chooses outcomes), AP-PLAN-001 (one issue
   per writing run), AP-GIT-001 (no branch without authority), and
   `GOVERNANCE.md` (a supervisor verdict before merge). Running it would
   need a recorded AP-EXCEPTION-001 exception bounded to its branch.

## Where it fits, once the checker exists

**Inference** for each; none is tested.

- **Checker speed (D12, E4).** The closest match: a frozen corpus and timing
  harness as `prepare.py`, one checker module as `train.py`, the test and
  conformance suite as a pass/fail gate, median edit-to-diagnostics time as
  the metric (target about 30 ms, D12). The gate stops the loop from buying
  speed with wrong answers. ImpossibleBench's finding that agents edit
  checks they can reach (`REFERENCES.md`, Agent Tooling) argues for keeping
  the corpus and harness outside the agent's write set.
- **Diagnostic text (D14).** A frozen corpus of broken programs; the agent
  edits only message and fix-suggestion text; the metric is a fresh agent's
  first-attempt repair rate. Faster and less noisy than an E1 round, but the
  repair agent is itself noisy, so a kept change needs a margin over
  repeated baseline runs.
- **Reference size (fx-jv9, `research/13-agent-fleets.md` need 6).** Tokens
  as the metric, trial pass rate as a gate. Riskiest: it only stays honest
  with a sealed second task set the loop never sees, so it is not
  recommended before the other two.

## What Tide can take now

- **A frozen grader.** Fix a trial's tasks and rubric before any arm runs,
  as `prepare.py` is fixed. The E1 trials already do this in practice.
- **A log of every attempt**, discards included, so a later session does not
  retry a dead direction. Review rounds record findings; nothing records
  abandoned alternatives in one place.
- **Simpler at equal score wins**, which AP-MINIMAL-001 and the D0
  admission filter already state.

## Sources

- [A1] [karpathy/autoresearch README](https://github.com/karpathy/autoresearch)
- [A2] [autoresearch `program.md`](https://raw.githubusercontent.com/karpathy/autoresearch/master/program.md)
- [A3] [DataCamp, A Guide to Andrej Karpathy's AutoResearch](https://www.datacamp.com/tutorial/guide-to-autoresearch)
- [A4] [NYU Shanghai RITS, Karpathy open-sources autoresearch](https://rits.shanghai.nyu.edu/ai/karpathy-open-sources-autoresearch-100-ai-experiments-overnight-on-one-gpu/)
- [A5] [Data Science Dojo, Karpathy autoresearch explained](https://datasciencedojo.com/blog/karpathy-autoresearch-explained/)
- [A6] [Red Hat Developer, 198 experiments on OpenShift AI](https://developers.redhat.com/articles/2026/04/07/autoresearch-on-red-hat-openshift-ai-198-experiments-zero-intervention)
