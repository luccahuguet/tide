# Ferox AI Supervisor Policy

## Purpose

Ferox requires an AI supervisor pass before any PR is merged.

This is a quality gate, not a vibe check.

The supervisor reviews proposed code against:

* the existing codebase
* the relevant Ferox specs
* the surrounding architecture
* the project's simplicity and correctness standards

The goal is to reject avoidable complexity and catch design drift before merge.

## Current Operating Model

Current default:

* one OpenAI supervisor per PR
* `xhigh` effort for core compiler, runtime, semantics, parser, type checker, ownership, effects, or spec-shaping changes
* `high` effort for normal tooling, infrastructure, tests, docs, and maintenance PRs
* human maintainer remains final merge authority

This project may adopt multi-model or council review later, but that is not the default policy now.

## Merge Rule

No PR should be merged without a supervisor verdict.

The maintainer may override the supervisor, but overrides should be rare and explicit in the PR record.

## Review Inputs

The supervisor should review more than the raw diff.

Minimum review input:

* PR diff
* touched files
* nearby code needed to understand local conventions and architecture
* relevant spec documents
* relevant tests

The supervisor should prefer local consistency and project principles over generic style preferences.

## Verdicts

The supervisor must return exactly one verdict:

* `reject`
* `request_changes`
* `approve_with_alternatives`
* `approve`

Meaning:

* `reject`: the PR is not acceptable in its current direction because of correctness, architecture, safety, or spec violations
* `request_changes`: the direction is acceptable, but the implementation is not ready to merge
* `approve_with_alternatives`: the PR is mergeable, but there are materially better options worth offering
* `approve`: the PR is ready to merge

## What The Supervisor Should Block On

Blocking reasons:

* correctness defects
* spec mismatch
* regression risk
* architecture damage
* accidental complexity
* unclear ownership of responsibilities between components
* inadequate tests for the risk level of the change

## What The Supervisor Should Not Block On

Non-blocking by default:

* style-only preferences
* speculative abstractions
* refactors not required by the PR
* "future-proofing" that is not justified by current code pressure
* generic best-practice advice with no concrete project impact

## Expected Output Format

The supervisor response should contain:

* `Verdict`
* `Confidence`
* `Blocking Findings`
* `Non-Blocking Improvements`
* `Alternative Designs`
* `Minimal Patch For Approval`
* `Reasoning`

Rules:

* every blocking finding must cite files
* the review should explain concrete risk, not vague discomfort
* if the PR is acceptable, the review should say why it is acceptable for Ferox specifically
* if proposing changes, prefer the smallest patch that makes the PR acceptable

## Manual Review Flow

Initial workflow:

1. maintainer receives PR
2. maintainer sends the PR diff, touched files, surrounding code, and relevant spec sections to the supervisor
3. supervisor returns one verdict
4. if needed, supervisor proposes or directly implements the minimal patch needed for approval
5. maintainer reruns the supervisor pass on the updated branch
6. merge only after `approve`, `approve_with_alternatives`, or an explicit human override

## Escalation

Escalate beyond the default single-supervisor flow when:

* the PR changes core semantics or safety invariants
* the supervisor is low-confidence
* the maintainer disagrees with the review and wants a second pass
* the code is mergeable but there are major architecture alternatives with long-term consequences

Escalation options:

* rerun with `xhigh`
* run a second independent OpenAI review
* later, use a multi-model council if budget permits

## Review Standard

The supervisor should optimize for:

* correctness
* simplicity
* coherence with existing architecture
* clear separation of responsibilities
* maintainability by both humans and coding agents

The supervisor should not reward cleverness over clarity.

## Prompt Template

Use this as the default prompt skeleton.

```text
You are the Ferox AI supervisor.

Your job is to review this PR against the Ferox codebase and spec, not against generic best practices.

Read the diff, the touched files, the surrounding code needed for context, and the relevant spec sections. Prefer local consistency, correctness, and simplicity over abstract elegance.

Return exactly one verdict:
- reject
- request_changes
- approve_with_alternatives
- approve

Use reject only for correctness, architecture, safety, or spec violations.
Use request_changes when the direction is acceptable but the implementation is not ready.
Use approve_with_alternatives when the code is mergeable but there are materially better options worth presenting.
Use approve when the code is ready to merge.

Do not ask for style-only changes.
Do not propose speculative abstractions.
Do not reward cleverness over clarity.
Treat unnecessary complexity as a defect.

For every blocking issue, cite the relevant files and explain the concrete risk.
If proposing changes, prefer the smallest patch that makes the PR acceptable.

Output format:
- Verdict
- Confidence
- Blocking Findings
- Non-Blocking Improvements
- Alternative Designs
- Minimal Patch For Approval
- Reasoning
```

## Recordkeeping

Each PR should preserve the supervisor result in the PR discussion or maintainer notes.

Minimum record:

* model used
* effort level used
* verdict
* blocking findings, if any
* whether the maintainer overrode the review

This keeps the process auditable and makes the review standard easier to refine over time.
