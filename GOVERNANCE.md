# Ferox Governance

## Development Rule - Agent-Produced Code

Ferox development follows an **agent-produced code rule**.

Primary rule:

> No production Ferox code should originate from direct human coding.

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

This rule is project governance, not part of the Ferox language specification.

## Merge Rule - AI Supervisor Pass

Ferox also follows an **AI supervisor merge rule**.

Primary rule:

> No PR should be merged without an AI supervisor pass.

Current operating model:

* one OpenAI supervisor per PR
* `xhigh` effort for core compiler, semantics, parser, type checker, ownership, effects, runtime, or spec-shaping changes
* `high` effort for normal tooling, tests, infrastructure, docs, and maintenance PRs
* human maintainer remains final merge authority

The supervisor reviews proposed changes against:

* the existing codebase
* the relevant Ferox specs
* surrounding architecture
* project simplicity and correctness standards

Required verdicts:

* `reject`
* `request_changes`
* `approve_with_alternatives`
* `approve`

The supervisor should block on correctness, spec mismatch, regression risk, architecture damage, and unnecessary complexity.

The supervisor should not block on style-only preferences or speculative abstractions.

Detailed workflow and prompt guidance live in [AI_SUPERVISOR.md](./AI_SUPERVISOR.md).

## Operational Work Tracking

Ferox uses `br` as the canonical tracker for open work.

Canonical open-task state lives in the committed `.beads/issues.jsonl` ledger and its associated `br` metadata.

Canonical task meaning and completed-task archive still live in [TASKS.md](./TASKS.md) and `tasks/T###.md`.

The detailed `br` workflow is documented in [BR_WORKFLOW.md](./BR_WORKFLOW.md).

## Design Admission Rule

Ferox should apply an explicit low-entropy admission filter to major language and toolchain changes.

Core rule:

> A feature should be rejected by default if it increases semantic entropy more than it improves safe change.

Before accepting a substantial design addition, Ferox should ask:

* does this improve mechanically safer change?
* does this reduce or increase the number of overlapping ways to solve the same problem?
* does this preserve or damage local reasoning?
* does this support or hinder fast incremental feedback?
* does this help humans and agents converge on one obvious code shape?
* can this live in tooling or libraries instead of widening the core language?

If the answers are weak or mixed, the default action should be defer or reject rather than expand the core casually.
