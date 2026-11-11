gpt-6.1-sol · high · 2026-11-11

**Verdict: approve — round-2 re-check clean.** Confidence: high.

1. **Finding 1 resolved.** Original projection reproducers pass. Normalized growth, including `Vec<T::A>` where `T::A = T`, receives FX0407. Unresolved projections remain unlabelled; parameters outside unresolved aliases still establish growth. No new defect found in normalization or the growth predicate (`traits.rs:196,257`).

2. **Finding 2 resolved.** Original uncalled dot-call and qualified-call reproducers now receive FX0407. Fixed recursion passes. Additional probes cover multiple impls, nested impl heads, where-clause-dependent selection, generic blanket impls, parameter-dependent impl choice, generic methods, and trait objects. No new false positive or false negative found in selected-method resolution (`traits.rs:204`).

3. **Finding 3 resolved.** Finite-chain reproducers retain rustc’s instance-limit failure without FX0407; the higher-limit chain passes. The exact source from the renamed test (`tests/build.rs:1858`) retains rustc’s failure, adds no FX0407, and produces no executable. Proven growing local cycles still receive FX0407 from the definition graph. Removing the unconditional label (`diag/labels.rs:118`) satisfies the precision decision.

**New findings: none.**

Proof: 28/28 original invocations and 53 additional valid adversarial invocations matched expectations against the candidate’s existing artifacts. Runs were serial, under `nice -n 19`, `CARGO_BUILD_JOBS=4`, with 45-second timeouts and closed stdin. Resource gates passed before each invocation.

HEAD remains `c69662c`; the supplied diff SHA-256 matches and the checkout remains clean. No repository or tracker writes; no held-out files opened.

Not checked: full suites were not rerun; this verdict covers only the three fixes. Remaining limitation: genuinely unresolved dispatch may omit FX0407 under the approved precision policy.