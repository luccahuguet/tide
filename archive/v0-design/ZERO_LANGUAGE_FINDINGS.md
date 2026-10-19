# Zero Language Findings

This document captures a first-pass review of Zerolang / Zero as of
2026-06-14.

Zero is relevant to Ferox because it is explicitly positioned as an
agent-first programming language. The most important lesson is not its surface
syntax. The important lesson is its compiler-facing edit contract for agents.

## Sources Reviewed

Primary sources:

* https://zerolang.ai/
* https://zerolang.ai/concepts/graph-architecture
* https://zerolang.ai/diagnostics
* https://github.com/vercel-labs/zerolang
* https://github.com/vercel-labs/zerolang/releases/tag/v0.3.4

Repository files reviewed:

* `README.md`
* `CHANGELOG.md`
* `package.json`
* `AGENTS.md`
* `examples/README.md`
* `docs/articles/learn-zero.md`
* `docs/articles/diagnostics.md`
* `skills/zero/SKILL.md`

Local limitation:

* `zero` is not installed on this machine, so this review is based on official
  documentation and repository source, not hands-on execution.

## Short Summary

Zero is an experimental systems language where the program database is a
semantic graph store. Text files with the `.0` extension are human-readable
projections. Agents are expected to inspect and edit the graph through compiler
commands such as `zero query`, `zero patch`, `zero check`, `zero test`, and
`zero run`.

The core claim is that agents should not primarily manipulate source text.
Instead, agents should query compiler-owned semantic state and submit checked
graph patches.

## What Exists Today

The public repository reports version `0.3.4`.

The current system includes:

* graph-first package stores through `zero.graph`
* readable `.0` source projections
* explicit import, export, and projection verification commands
* graph query, patch, check, test, run, inspect, diff, and fix-plan workflows
* structured diagnostics intended to guide both humans and agents
* version-matched agent skills served by the compiler
* examples covering CLI programs, filesystem use, JSON, HTTP, packages, and
  native builds
* a native compiler implementation under `native/zero-c/`

The example `.graph` files are binary graph stores rather than ordinary UTF-8
source files. That matches the stated design: the graph is the compiler input,
and text is a projection for review or intentional import.

## The Agentic Idea

The strongest idea in Zero is the edit surface.

Traditional agent coding usually works like this:

* read text files
* infer intent from syntax and spans
* edit text
* run a checker
* parse diagnostics
* repeat

Zero tries to replace that with:

* query the semantic graph
* patch compiler-owned graph state
* reject stale or invalid patches before store mutation
* validate behavior
* export readable projection text when human review needs it

The graph can expose facts that are awkward to recover from text alone:

* symbol identities
* node identities
* graph hashes
* type facts
* effect facts
* ownership facts
* capability facts
* import relationships
* call edges
* target and build facts

This is directly aligned with Ferox's machine-contract goals. Ferox already
plans structured diagnostics and stable machine-readable outputs. Zero pushes
the same idea further by making structured semantic edits a normal programming
workflow.

## Why It Matters For Ferox

Zero is evidence that "agent-friendly tooling" should mean more than JSON
diagnostics.

Useful Ferox takeaways:

* Treat stable semantic identity as a product feature, not only an internal
  compiler convenience.
* Define read-side contracts for symbols, declarations, modules, type facts,
  ownership facts, and diagnostics early enough that tools can rely on them.
* Consider a write-side contract for safe automated edits once the compiler has
  enough semantic structure to check them.
* Prefer checked edit operations over line-range patches for high-value
  refactors and repairs.
* Make stale-state detection explicit. A tool should know when its view of the
  program is outdated.
* Keep human-readable source as the primary authoring surface for Ferox v1
  unless there is a very strong reason to make a graph store primary.

Recommended Ferox interpretation:

* copy the compiler contract idea
* do not copy the binary graph store as the primary source of truth in v1
* keep `.ferox` files canonical
* expose semantic graph inspection as tooling
* add checked mutation gradually after parser, resolver, type checker, and
  ownership analysis are solid

## Strengths

Zero has a sharp and coherent agent-facing thesis.

The most promising strengths are:

* a compiler-native interface for agents
* explicit protection against stale graph edits
* source projections that remain reviewable by humans
* diagnostics that include repair-oriented information
* version-matched skills so agents do not guess against stale language docs
* a standard-library direction shaped around explicit capabilities and
  caller-owned buffers

The project is also honest about its experimental state. Its own repository
warns users to expect breaking changes, rough edges, and security issues.

## Risks And Open Questions

The risk profile is high.

Main concerns:

* The binary graph store adds adoption friction and trust cost.
* Projection import and export create a second consistency problem that a
  source-first language avoids.
* Agent-first workflows may be hard for humans to debug when graph state and
  projection text diverge.
* The project is pre-1 and explicitly permits breaking changes.
* The security posture is experimental, so production use is inappropriate.
* A new systems language already faces a large adoption barrier; a new
  graph-first source model compounds that barrier.

Open questions for Ferox:

* What semantic identities should be stable across edits?
* Which diagnostics should carry machine-applicable fix plans?
* Which edits are safe enough to expose as structured operations?
* Can Ferox provide a semantic graph for tools without making it the canonical
  source artifact?
* How much of the compiler's internal representation can become public contract
  without freezing bad early architecture?

## Ferox Action Items

Near-term:

* Keep the existing machine-contract direction in `spec/tooling/machine_contract.md`.
* Extend the tooling plan with symbol/declaration identity once resolver design
  is concrete.
* Ensure diagnostic codes and spans are designed for both humans and agents.
* Avoid promising structured mutation before the compiler has real semantic
  authority.

Later:

* Add `ferox inspect` or equivalent for semantic project facts.
* Add a stable symbol graph output tier.
* Add checked refactor operations for narrow cases such as rename, extract
  helper, update import, and apply diagnostic fix.
* Evaluate whether an internal graph snapshot should be cached for incremental
  tooling while `.ferox` remains canonical source.

## Bottom Line

Zero is not mainly useful to Ferox as a language to imitate. It is useful as a
clear example of an agent-native compiler interface.

Ferox should stay source-first for v1, but the tooling architecture should leave
room for the stronger idea: agents should eventually edit through compiler-checked
semantic operations rather than brittle text patches whenever the compiler can
offer that safely.
