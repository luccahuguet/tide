# Tide Governance

## Development Rule - Agent-Produced Code

Tide development follows an **agent-produced code rule**.

Primary rule:

> No production Tide code should originate from direct human coding.

Human responsibilities:

* defining architecture
* writing specifications
* reviewing generated code
* validating correctness
* steering project direction

Agent responsibilities:

* implementing compiler components
* writing tooling
* generating tests
* refactoring code
* applying mechanical fixes

Human-written production code is **forbidden by default** and must be explicitly marked if it occurs as an exception.

Philosophy:

> Humans define the system.
> Agents produce the code.

This rule is project governance, not part of the Tide language specification.

## Merge Rule - AI Supervisor Pass

Tide also follows an **AI supervisor merge rule**.

Primary rule:

> No change lands on `main`, by PR or direct commit, without an AI supervisor pass.

Current operating model:

* the first review round runs two read-only reviews in parallel on one frozen candidate, the working tree at its recorded diff SHA-256: an Opus review (for code, tests, or scripts, a converge round that reports instead of fixing; for docs-only design work, a review round) and one Codex supervisor pass (`gpt-6.1-sol`, always `xhigh` effort). Nothing edits the tree until both reports are in; then the author fixes what both found. The two reviews share a round number
* Opus confirmation rounds follow until one comes clean; later converge rounds fix as usual. Then a narrow supervisor re-check reviews the changes made since the frozen candidate, unless there are none. If the re-check leads to any change, run one narrow Opus confirmation round and another narrow re-check; text-only fixes restart this loop at most twice. The supervisor pass and its re-check replace converge's own Codex round
* a finding blocks or restarts a round when its fix would change a tier the change edits or an earlier one; most changes edit the v0 core. A finding whose fix would change only text in a tier later than every tier the change edits (for a v0 change, v1 or Later in `design/01-design.md`'s tier table) is recorded as a comment on bead `fx-djp` and is not fixed in the change under review
* planning-only changes (bead filings) skip the supervisor pass
* human maintainer remains final merge authority; overrides should be rare and recorded

The supervisor reads the diff, the touched files, the surrounding code needed for context, the relevant spec sections, and the relevant tests, and reviews the change against:

* the existing codebase
* the relevant Tide specs
* surrounding architecture
* project simplicity and correctness standards

Required verdicts:

* `reject`: the direction is wrong because of correctness, architecture, safety, or spec violations
* `request_changes`: the direction is acceptable, but the change is not ready to merge
* `approve_with_alternatives`: the change is mergeable, but materially better options are worth presenting
* `approve`: the change is ready to merge

The supervisor should block on correctness, spec mismatch, regression risk, architecture damage, unclear ownership between components, unnecessary complexity, and tests too weak for the risk.

The supervisor should not block on style-only preferences, speculative abstractions, refactors the PR does not need, or future-proofing without current code pressure.

The review gives the verdict, its confidence, blocking findings, non-blocking improvements, alternative designs, the minimal patch for approval, and its reasoning. Every blocking finding cites files and states the concrete risk; a proposed change is the smallest patch that makes the PR acceptable.

Escalate, by adding a second independent review, when the PR changes core semantics or safety invariants, the supervisor has low confidence, or the maintainer disagrees with the review.

Record each pass in the PR discussion, or for a direct commit in `reviews/`, one file per review, named by bead, round within that bead, and reviewer: `<bead>-r<N>-opus.md` or `<bead>-r<N>-codex.md` (for example `fx-gcs.2-r1-codex.md`; a change without a bead uses a short kebab-case slug not yet used in `reviews/`). Each bead or slug starts at round 1; every later review takes the next unused round number within that bead, continuing across commits and handoffs, except that each change's parallel first-round pair shares one number. A Codex file records model, effort level, verdict, blocking findings, and any override. Per-bead names let parallel sessions record reviews without colliding; the `round<N>-*.md` files are the earlier, global numbering and stay as history.

Give the supervisor this section as its brief.

## Operational Work Tracking

Tide uses `br` as the canonical tracker for open work.

Canonical open-task state lives in the committed `.beads/issues.jsonl` ledger and its associated `br` metadata.

Canonical task meaning and completed-task archive still live in [TASKS.md](./TASKS.md) and `tasks/T###.md`.

The detailed `br` workflow is documented in [BR_WORKFLOW.md](./BR_WORKFLOW.md).

## Design Admission Rule

Tide should apply an explicit low-entropy admission filter to major language and toolchain changes.

Core rule:

> A feature should be rejected by default if it increases semantic entropy more than it improves safe change.

Before accepting a substantial design addition, Tide should ask:

* does this improve mechanically safer change?
* does this reduce or increase the number of overlapping ways to solve the same problem?
* does this preserve or damage local reasoning?
* does this support or hinder fast incremental feedback?
* does this help humans and agents converge on one obvious code shape?
* can this live in tooling or libraries instead of widening the core language?

If the answers are weak or mixed, the default action should be defer or reject rather than expand the core casually.
