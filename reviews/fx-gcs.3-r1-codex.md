Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-21.

## Verdict

**request_changes** — high confidence. The direction is sound, but several cases encode incomplete or incorrect expected verdicts.

## Scope

Reviewed clean HEAD `19bc1a2ec2cdcfe6972739c7c791232694d6dfdc` against `e2a7063`, including all 71 new cases, runner and stub changes, governance, the three bead records, inventory findings, and relevant design/reference sections.

Ran `bash -n conformance/run.sh`, `conformance/run.sh --trace`, `git diff --check`, and an in-memory coverage audit. All passed. **Did not run the full corpus:** it writes build artifacts, which this sandbox prohibits.

## Blocking

1. **An accepting `Pure` case contains forbidden narrowing casts.**  
   [conformance/cases/authority/pure_accept.rs:21](/home/lucca/pjs/lab/tide-fx-gcs.3/conformance/cases/authority/pure_accept.rs:21) converts two `usize` lengths to `u32` using `as`. D3 and reference §4.2 reject these with `FX0301`. A conforming checker must therefore reject this unmarked accepting case.  
   **Minimal patch:** replace both casts with checked `u32::try_from(...).unwrap()` conversions, preserving this case’s accepting verdict.

2. **The ROOT-4 probes interfere with ROOT-1.**  
   [root_fields.rs:5](/home/lucca/pjs/lab/tide-fx-gcs.3/conformance/cases/authority/root_fields.rs:5) expects acceptance while explicitly naming `Root` in a pattern. ROOT-1 and reference §6.2 permit that name only in `main`’s signature. [root_forge.rs:12](/home/lucca/pjs/lab/tide-fx-gcs.3/conformance/cases/authority/root_forge.rs:12) also names it outside `main`, but expects only the subsequent rustc construction error. These cases introduce unstated exceptions or require suppressing another rejection.  
   **Minimal patch:** move `root.fs` and `root.clock` individually in the accepting case; test private construction on `Fs` or another capability in the rejecting case.

3. **The Waker case leaves another forbidden operation unmarked.**  
   [conformance/cases/authority/admit_waker.rs:4](/home/lucca/pjs/lab/tide-fx-gcs.3/conformance/cases/authority/admit_waker.rs:4) calls `Waker::wake_by_ref` without an `FX0520` mark. ADMIT-1 excludes `core::task` and checks method calls as well as type paths. Reporting this independent violation would produce an unexpected diagnostic.  
   **Minimal patch:** remove the method call and rename the parameter `_w`, isolating the marked type-path rejection.

4. **The `const fn` fix remains incomplete in the core reference.**  
   [design/reference.md:99](/home/lucca/pjs/lab/tide-fx-gcs.3/design/reference.md:99) lists authority-freedom and no `&mut` parameters, but omits AFREE-2’s `Sync` requirement and `FX0518`. Consequently, `afree_const_cell.rs` satisfies the core reference’s stated restrictions while expecting rejection. The Pure-code summaries at [03:790](/home/lucca/pjs/lab/tide-fx-gcs.3/design/03-authority-concurrency.md:790) and [01:1010](/home/lucca/pjs/lab/tide-fx-gcs.3/design/01-design.md:1010) also omit the new code.  
   **Minimal patch:** state `Pure + Sync` parameters and `FX0518` in reference §2.3, refresh those summaries, and retain the token budget.

## Non-blocking

None. No `fx-djp` findings.

## Reasoning

All **25 in-scope rule IDs** have files labeled with accepting and rejecting verdicts; no entire in-scope ID is missing. Semantic acceptance is incomplete because the accepting cases above violate other rules.

The macro-code reconciliation, new IDs, and memoization clarification fit the design. `FX0518` has no competing assignment. The runner’s feature restriction matches ADMIT-2, preserves both compilation modes, and leaves the five seeds unchanged. Static inspection found no additional blocking runner or stub defect.

Not checked: the full corpus or token counter. The reported 32 PASS / 44 AWAITS CHECKER result and 24,996-token count remain independently unverified.