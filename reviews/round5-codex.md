# Tide second-model design review, round 5

## Verdict

**request_changes**

Both round-4 blockers are resolved. One new v0 contradiction remains:
invocation-count sampling makes an authority-free function's panic depend
on hidden execution history, contrary to its determinism contract.

Reviewed draft 3.3 at HEAD
`4d236fb14cfeb442a5b37a92abbc72022eb01db5` on 2026-10-19. Read all six
requested files; none had a working-tree diff. Scope follows
[01, Tiers](../design/01-design.md#tiers): later-tier features and wording
nits are excluded. The counterexample below is a deduction from the
specification, not an observed implementation failure.

## Round-4 objection status

| Round-4 objection | Status | Evidence |
| --- | --- | --- |
| **B1. Replay identity** | **Resolved** | [01, D15](../design/01-design.md#d15-deterministic-simulation-as-a-first-class-mode) now requires source and dependency content hashes, a semantic build identity, setup, and injected inputs, and says replay refuses a mismatched identity. [03, §9.6, REPLAY-1/REPLAY-2](../design/03-authority-concurrency.md#96-tide-sim) explicitly binds these to D14's snapshot hashes, includes uncommitted edits, profile and `cfg`, and checks identity before execution. It also distinguishes a fresh `--seed` run from replay. [Reference, §6.8](../design/reference.md#68-simulation) agrees. This closes the dirty-source/build-option scenario in [round 4, B1](round4-codex.md#b1-a-replay-manifest-does-not-identify-the-executed-program). |
| **B2. Observable NaN nondeterminism** | **Resolved** | [01, D3](../design/01-design.md#d3-types-small-decidable-explicit-at-boundaries) now makes every sign/bit observer treat any NaN as one canonical NaN, explicitly including `copysign`, `signum`, `total_cmp`, hashing, and formatting. [03, §8.2, PAR-3](../design/03-authority-concurrency.md#82-semantics) requires this in every engine, including `const fn` evaluation; [reference, §4.3](../design/reference.md#43-floats) matches. Thus both `total_cmp` and `copysign` in [round 4, B2](round4-codex.md#b2-canonicalizing-only-to_bits-leaves-observable-nan-nondeterminism) observe the canonical NaN rather than a native arithmetic NaN's varying sign. |

## New v0 blocking objections

### B1. Sampled checks introduce hidden history into `Pure` functions

**Evidence.** [01, D7](../design/01-design.md#d7-authority-capabilities-as-values-no-ambient-authority)
classifies a function with only `Pure` parameters and captures as
authority-free and says its results depend only on its arguments.
[03, §3.3](../design/03-authority-concurrency.md#33-what-authority-freedom-guarantees)
expressly includes **the panic, if any**, in that guarantee.
However, [04, §1.8, CLAUSE-5/6/7](../design/04-contracts.md#18-v0-clause-semantics)
classes a user-function postcondition as `sampled`, applies §2.4's schedule,
and makes a false evaluation panic. [04, §2.4](../design/04-contracts.md#24-test)
checks the first 64 calls of each function, then calls 128, 256, and so on;
[reference, §10.1](../design/reference.md#101-clauses) repeats this rule.

**Scenario.** In a single `tide test` run with the default sampling schedule
and enough clause steps to evaluate this constant predicate:

```tide
fn rejects(_: u8) -> bool { false }

fn echo(x: u8) -> u8
    ensures rejects(result)
{
    x
}
```

Starting with `echo`'s call count at zero, invoke `echo(7)` 65 times, each
inside `sim.tasks.spawn_isolated(|| echo(7))`. The first 64 invocations
panic on the postcondition and yield `Err(Panicked)`; invocation 65 skips
the clause and yields `Ok(7)`. Isolation is the documented way to contain
each panic and continue ([03, §6.2, TASK-1](../design/03-authority-concurrency.md#62-failure-cancellation-and-deadlines)).
`echo` and `rejects` have only scalar parameters, satisfying the
authority-free and clause argument rules ([03, §3.2](../design/03-authority-concurrency.md#32-authority-free-functions),
[04, CLAUSE-3](../design/04-contracts.md#18-v0-clause-semantics)). The build
and input memory are unchanged; the outcome depends on an invocation
counter outside that memory. This is a contradiction within one test
profile, not merely debug-versus-release clause erasure. It does not by
itself establish that a fixed simulation seed fails to replay.

**Smallest fix preserving the stated determinism guarantee.** Remove
invocation-count skipping of sampled function-boundary clauses in v0:
under `tide test`, evaluate them on every call with the existing per-clause
step budget and unchecked outcome. Keep the debug/release distinction and
generated property tests. Record that decision in D10 and align CLAUSE-6,
§2.4, and the reference. Alternatively, explicitly exempt sampled-check
instrumentation from D7/§3.3's panic guarantee; that is a weaker semantic
contract requiring an explicit decision.

## Ready to implement v0?

**No, for the checker and reference interpreter as an implementation of the
complete stated v0 contract.** Resolve B1's choice between history-free
boundary checks and an instrumentation exception before making the
interpreter the semantic oracle ([01, D12](../design/01-design.md#d12-compilation-and-backends)).
Replay identity and NaN observation need no further design change for the
round-4 scenarios. No other new v0 blocker was established in the requested
documents.

Validation was document comparison and scenario analysis. No Tide checker,
interpreter, or runtime was run; implementation behavior remains unproven.
Only this review file was written.
