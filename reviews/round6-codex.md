# Tide second-model final confirmation, round 6

## Verdict

**request_changes**

Reviewed HEAD `5d7dedfced7fda1b5626d70f5f09e23135449223` on 2026-10-19,
including commits `34f2887`, `b12b6a4`, and `5d7dedf`. Compared both round-5
reviews with the current decision record, concurrency chapter, contracts
chapter, and reference; those six files have no working-tree diff.

Four items have decision-level resolutions. R3 remains partial: the fixes
carry the body-error wording across the documents, but leave the echo-bit
definition and scope-result precedence inconsistent. This is an existing
round-5 objection, not a new blocker introduced by the fixes.

## Round-5 item status

### Codex B1: invocation-count sampling

**Resolved.** [D10, lines 523–531](../design/01-design.md#d10-contracts-one-spec-three-uses-defined-semantics)
requires sampled boundary clauses to run "on every call and within their
step budget" and makes the outcome independent of how many times the clause
has run. [CLAUSE-6/7 and §2.4, lines 399–411 and 473–478](../design/04-contracts.md#24-test)
retain the budget and unchecked outcome and specify "on every call in a
test run". [Reference §10.1, lines 1158–1166](../design/reference.md#101-clauses)
agrees. The round-5 `echo(7)` scenario therefore cannot succeed merely
because it is invocation 65. The remaining loop-invariant schedule uses the
loop's own iteration index (§2.4, lines 480–484), not earlier calls.

### Consistency R3: body-error cancellation echoes

**Partial; not fully resolved.** [TASK-2, lines 641–643](../design/03-authority-concurrency.md#62-failure-cancellation-and-deadlines)
now says that when the body's `Err` cancels the scope, the children's
resulting errors are echoes and the scope returns the body's error unless
a child panicked. [Reference §7.1, lines 870–873](../design/reference.md#71-scopes-and-tasks)
also reports the body's error in that case, and §13.2 of the concurrency
chapter no longer lists body-origin cancellation as an open question.

However, TASK-2's per-child bit still becomes set only for a cancellation
"that a sibling's failure caused" (lines 636–640). Rule 2 still selects the
earliest-spawned failing child (lines 630–634), while the following body-error
sentence says to return the body's error unless a child panicked.
[D8, lines 435–444](../design/01-design.md#d8-concurrency-structured-tasks-no-async-compiled-to-state-machines)
retains that same precedence conflict, which round 5 explicitly identified.

For example, let the body return `Err(Invalid)` while a child computes and
then returns `Err(Independent)` without observing cancellation. Cancellation
does not interrupt that computation ([TASK-5](../design/03-authority-concurrency.md#62-failure-cancellation-and-deadlines)).
The child's bit stays clear and rule 2 selects `Independent`; the body-error
sentence selects `Invalid`. An independent failure is not a cancellation
echo. Extending the observed-cancellation bit to body-caused cancellation
and preserving rule 2 for independent child failures would close the
remaining R3 issue. TASK-1 already specifies body-panic unwinding; that does
not settle this ordinary-`Err` case.

### Consistency N1: `par_sort` structure

**Resolved at the decision level.** [D9, lines 473–475](../design/01-design.md#d9-parallelism-deterministic-with-defined-grouping)
defines a "stable merge sort whose splits and merges depend only on the
input length" and makes that structure's left-to-right order the sequential
order for panics and buffered sinks. [§8.1, lines 821–824](../design/03-authority-concurrency.md#81-forms-and-admission)
repeats the length-dependent structure. Together with PAR-1/2, this separates
the logical comparison structure from §8.4's runtime promotion policy; a
different worker count cannot choose a different logical sort tree.

### Consistency N2: sampled-clause seeding and budgets

**Resolved for the reported failure.** The every-call rule cited above
removes the function invocation counters, so previous seeds, tests, and
self-check reruns cannot consume a later call's sampling allowance.
[D10, lines 527–531](../design/01-design.md#d10-contracts-one-spec-three-uses-defined-semantics)
specifies that "steps count evaluated operations", with deterministic
exhaustion. §2.4 gives each evaluation its own budget and makes the remaining
loop schedule depend on its iteration index. Package budget changes fall
under source/configuration identity; any budget build option falls under
[REPLAY-1, lines 1017–1024](../design/03-authority-concurrency.md#96-tide-sim)'s
"every option that affects semantics". No per-process sampling state remains
to reset or record for the round-5 scenario.

### Consistency N3: signed zeros

**Resolved by the owning decision.** [D3, line 270](../design/01-design.md#d3-types-small-decidable-explicit-at-boundaries)
now states: "`min` and `max` order -0.0 below +0.0." Thus `min` chooses
negative zero and `max` chooses positive zero for opposite signed zeros,
including the round-5 reciprocal example. PAR-3 and reference §4.3 do not
repeat the added sentence, but they cannot authorize a contrary engine or
target result: the decision record owns language semantics and wins on
conflicts ([README, Reading Guide](../README.md#reading-guide)). Their missing
restatement is not a new v0 blocking decision.

## New blocking problems

**none**

I found no new v0-core blocking problem introduced by these fixes. The
remaining R3 problem above is part of the requested round-5 status check.
I did not reopen the reviews' unrelated non-blocking notes or later tiers.

## Ready to implement v0

**No.** Resolve R3's scope-result selection before treating the reference
interpreter as the semantic oracle for the complete v0 core. The sampling,
sort-structure, replay-counter, and signed-zero objections no longer require
a blocking design choice.

Validation was read-only document comparison and scenario analysis at the
revision above. No checker, interpreter, or runtime was run; implementation
behavior remains unproven. Only this review file was written.
