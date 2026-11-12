Model: gpt-6.1-sol · Effort: high · Date: 2026-11-12

Verdict: request_changes

Confidence: high in the finding; the full workspace and corpus checks remain the coordinator's responsibility.

Reviewed candidate `b3eda121b1534c853fc2147434c4b8d0a5c311aa` against parent `42dd813d5f7876a88ed085b0e7d659e987ca4704`. Candidate tree: `6b82938b2a571a071c9c80932b2b6ef7e094c336`. SHA-256 of `git diff 42dd813..b3eda12`: `5ae9b7b2aa74b48fe93ea95de0a5dfac36d0c505bcca97bbc332994158d071f0`.

What this change does: It adds error diagnostics for the six concurrency and error-handling rules, routes denied std APIs through their rule codes, and adds conformance cases and driver checks. The direction is sound, but the unresolved Tokio fallback loses imported module identities.

1. **Blocking — Tokio module aliases bypass FX0602.**

   Evidence: `crates/tide-admit/src/walk/habits.rs:81`, especially the literal two-segment match at lines 89–95; `crates/tide-admit/src/walk.rs:1826`; `conformance/cases/tasks/spawn_reject.rs:8`.

   This input produces no FX0602:
   ```rust
   use tokio as runtime;
   pub fn f() { let _ = runtime::spawn(|| 1); }
   ```
   The focused driver probe exited 1 with only rustc E0432 for the import. The unavailable runtime call is the same prohibited operation as the fixture's literal `tokio::spawn`, but renaming the module defeats its rule-specific diagnostic. In contrast, `use tokio::spawn as launch;` gets FX0602 at its import because that import happens to retain the literal two-segment spelling. The fallback does not follow imports to classify the later call.

   Rustc still rejects this particular input, so this is a missing required FX diagnostic rather than an observed admitted executable. Alias evasions are explicitly part of this review's contract. The resolved-external-Tokio branch cannot repair this case because no Tokio definition resolved. The tests cover renamed std functions and literal unavailable Tokio calls, leaving this boundary unprotected.

   Fix: preserve unresolved Tokio source identity through module aliases and re-exports. Reuse or narrowly extend the import/re-export recovery owned by `walk/authority.rs:327` and its helpers instead of growing a second spelling-reconstruction mechanism in `habit_path`. Keep resolved same-spelled local functions accepted. Add a regression for the example above and its re-export equivalent, asserting FX0602 at error severity.

Other review conclusions:

- The six new codes emit errors, independently of allow attributes. Each has an accepted and violating conformance case; every rule ID across `conformance/cases/` exists in `conformance/rules.md`.
- The policy keeps core task machinery under FX0520 and assigns future/pin APIs FX0601. The shared impl change preserves the rejecting trait/head's code and help without changing its allow/deny condition; I found no additional blocker in that change.
- The select check covers definition names and invocation names and reuses the existing macro provenance parser. Probes confirmed a select macro renamed away from `select`, and a nested user-written select invocation, still receive FX0605.
- The Result owner uses diagnostic identity rather than spelling or general must-use status. Probes confirmed `r.ok();`, `r?;`, explicit macro discard, and a handled macro result are accepted; ignored `writeln!` is rejected with FX0702. The containing-block provenance check avoids treating ordinary trusted macro internals as caller statements. No additional blocker found.
- Other implementation choices in Sol's report are reasonable for the recorded contract. Documentation and registry changes reflect those choices. There is no substantial unrelated policy/registry restructuring requiring coordination with fx-yg5.5.

Validation and limits:

- Read AGENTS.md, GOVERNANCE.md, both requested beads and their decisions, Sol's report, the diff, surrounding owners and callers, the relevant research, reference §§7/9, and decision-record D5/D8.
- Ran a 13-case serial probe batch against the existing candidate driver binary, through the required machine-wide queue:
  `(ulimit -c 0; CARGO_BUILD_JOBS=4 timeout 180 bash /home/lucca/.cache/eon-agents/eon-overseer/heavy.sh nice -n 19 python3 -c '<serial probe harness>' </dev/null)`.
  The harness created temporary source packages outside the repository and invoked `target/debug/tide-admit --edition=2024 --crate-type=lib --emit=metadata --error-format=short --out-dir <outside-package-output> <source>`, with closed stdin and a 15-second timeout per case. The batch completed normally on 2026-11-12 around 20:29Z, before the prohibited window. In addition to the cases described above, it exercised Future impls, Mutex method/path calls, std spawn re-exports, and local Tokio name resolution.
- An initial probe submission was cancelled while queued to correct its temporary output layout. A further boundary-probe batch was cancelled after roughly two minutes queued; it never executed. No results are claimed for those submissions.
- `git diff --check 42dd813..b3eda12` passed. HEAD and the clean repository status remained unchanged. No source, Git, or Beads writes were made.
- No held-out file was opened or searched.

Not checked: full workspace/corpus suites, a rebuild, or a real external Tokio crate; those are not claimed as review proof.
Risk: unresolved Tokio aliases currently lose the required FX0602 diagnostic. DONE
