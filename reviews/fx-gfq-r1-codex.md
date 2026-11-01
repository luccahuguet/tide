Model/Effort/Date: gpt-6.1-sol, xhigh, 2026-10-31  
Verdict: **request_changes** (high confidence)

Scope: Read-only review of the diff against `1e0cbac`, verified against the supplied SHA-256. Reviewed all 30 tasks, thirteen new prerequisites, `fx-gfq`, `fx-ic2`, the requested design references, governance, and earlier trials.

Blocking:

1. **B7’s preflight rejects its required swap.** [tasks.md:552](/home/lucca/.cache/eon-agents/tide-gfq/trials/e1-checked/tasks.md:552) requires every step’s source to exist before execution. A valid swap includes `a → a.tmp, b → a, a.tmp → b`; `a.tmp` initially does not exist. Fix: preflight only sources whose contents must exist initially, accounting for names produced by earlier steps.

2. **The prerequisites omit required Tide rejection checks.** [protocol.md:250](/home/lucca/.cache/eon-agents/tide-gfq/trials/e1-checked/protocol.md:250) covers conversion APIs and allowlist entries, but no prerequisite owns `FX0301`. The current driver does not implement that rule. For example, an A2 solution can narrow an already range-checked integer using `as`, produce correct results, and pass despite violating reference §4.2. Positive reference solutions cannot detect this. Fix: add the missing enforcement prerequisite and blocker edge, with a negative readiness probe for lossy casts.

3. **B7’s plan oracle loses file identity.** [tasks.md:560](/home/lucca/.cache/eon-agents/tide-gfq/trials/e1-checked/tasks.md:560) checks names as a set. An empty plan passes that oracle for the three-cycle because the initial and final name sets are identical. Fix: replay a map from names to distinct payload tokens and compare the final mapping with simultaneous renames.

4. **A9’s checked mutation case has no defined starting inventory.** [tasks.md:326](/home/lucca/.cache/eon-agents/tide-gfq/trials/e1-checked/tasks.md:326) says restocking `b` succeeds, but the only supplied inventory contains `a`. Using that fixture correctly returns `UnknownSku("b")`; inventing another fixture changes the test. Fix: specify the initial quantities and SKUs, plus which operations share state.

5. **Required concurrency can pass without being implemented.** [protocol.md:130](/home/lucca/.cache/eon-agents/tide-gfq/trials/e1-checked/protocol.md:130) uses token searches for structure, while [tasks.md:888](/home/lucca/.cache/eon-agents/tide-gfq/trials/e1-checked/tasks.md:888) gives C9 only result checks. A serial loop calling `spawn_isolated` once per input passes every case despite never running jobs concurrently. Similar output-only gaps affect other concurrency tasks. Fix: add paired, mechanically checked witnesses for required concurrency and cancellation behavior; token presence alone does not establish it.

6. **The sandbox contract does not enforce the stated isolation.** [protocol.md:277](/home/lucca/.cache/eon-agents/tide-gfq/trials/e1-checked/protocol.md:277) restricts writes, networking, and environment, but leaves readable host paths unrestricted. A Rust test can read repository files, other trials, or grader solutions through `std::fs`, bypassing writer-tool restrictions. Fix: specify an explicit readable filesystem allowlist that excludes those sources, alongside the writable-tree restriction.

Non-blocking:

7. [protocol.md:237](/home/lucca/.cache/eon-agents/tide-gfq/trials/e1-checked/protocol.md:237) names `fx-nkv.6`, whose scope explicitly excludes replay manifests. Name replay as an unavailable toolchain feature unless `fx-nkv.7` and support for the E1 inputs become prerequisites; reference companion §17 promises replay commands and identity checks.

8. [protocol.md:33](/home/lucca/.cache/eon-agents/tide-gfq/trials/e1-checked/protocol.md:33) starts the contamination restriction at the first writer session. Extend it to readiness and reference-solution work, applying the pilot’s fresh-task requirement when those activities shape writer-facing documents.

Reasoning: The direction is sound. D0’s numeric thresholds are preserved, pilot claims are appropriately limited, all thirteen new beads block `fx-ic2`, and its blocker graph is acyclic. The task contradictions and admission/grading gaps prevent this candidate from defining a reliable executable gate.

Not checked: executable suites or writer trials; production readiness remains unproven. No files changed.