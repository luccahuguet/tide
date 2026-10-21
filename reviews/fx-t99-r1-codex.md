Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-21.

## Verdict

**approve** — high confidence (95%). The classifications are justified, consistent with the existing rules, and within fx-t99’s scope.

## Scope

- HEAD: `76dfe0634a41c314112760e2e44cfef0893a35af`.
- Recomputed `git diff | sha256sum`: `880258673cf4d58dc7437523ec068c850ced39f22a92b2c9b3192ea3ec21d2f4` — **matches**. Rechecked at completion.
- Read `GOVERNANCE.md` first; then repository instructions, README, `br --no-db show fx-t99`, the complete diff, and round 32’s originating finding.
- Reviewed 01’s tiers, D7 and D15; 02 §9 and relevant closure/parallel rules; 03 §0, §1.3, ROOT-3/4, sinks, purity, §9.2–9.5 and simulator examples; reference §6, §8 and the std table; companion §15.4 and §17. Searched throughout `design/` for affected objects and captures.
- `git diff --check` passed. No files changed.

## Blocking

None. No patch required.

## Non-blocking

None identified, including no later-tier finding to route to **fx-djp**.

## Reasoning

Each classification follows D7’s distinction between ordinary data and reachable authority:

| Objects | Why they are not `Pure` |
|---|---|
| `SimRoot`, `NodeRoot` | They contain capability fields; simulated authority remains authority. See [03 §9.2](/home/lucca/pjs/lab/tide-fx-t99/design/03-authority-concurrency.md:1532) and [§9.5](/home/lucca/pjs/lab/tide-fx-t99/design/03-authority-concurrency.md:1597). |
| `sim.world`, `Faults`, `SimNode` | They configure or operate the simulated world without another capability argument: setup, fault injection, node execution and disk access. Their test-only availability does not make them authority-free. |
| `NodeTask` | It is a task handle whose completion `run_until` observes, fitting D7’s synchronization-handle category. See [companion §17](/home/lucca/pjs/lab/tide-fx-t99/design/reference-companion.md:464). |
| `LogConfig` | It can hold a non-`Pure` `File` supplied to `to_file`. The classification applies to the type, including configurations without a file. [SINK-2](/home/lucca/pjs/lab/tide-fx-t99/design/03-authority-concurrency.md:382) supports this rationale; the exemption for write-only sink operations does not make their configuration object `Pure`. |

The closure examples remain consistent. A closure’s `Pure` classification concerns its captures; receiving a non-`Pure` `NodeRoot` parameter is permitted and makes the invocation authority-bearing. The entries in companion §15.4 and 03 §12.3 capture none of these objects. HTTP peer callbacks have no `Pure` bound, while ordinary spawned tasks may capture authority.

The added cross-references resolve correctly. ROOT-3’s test restriction, ROOT-4’s construction rules, the reference’s summaries and std paths, and the companion agree with the owner table. The companion’s capture prohibition follows the existing node-entry bounds. The patch adds no unnecessary simulator API or lifecycle guarantees.

No executable checks or learnability trials were run; implementation conformance remains unproven by this textual review.