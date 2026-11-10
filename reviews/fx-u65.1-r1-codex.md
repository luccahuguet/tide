Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-09

## Verdict

**request_changes — high confidence.** The severity plumbing and conformance matcher work for the intended cases. Two ordinary, statically identifiable adaptations bypass the new warnings: coercing the channel constructor to a function pointer, and boxing the supplied lock callback. Fix B1 and B2 before approval. Neither fix requires new language semantics or whole-program effect analysis.

## Scope

Reviewed `git diff e6fa043 baee486`, candidate `baee48690fa6a5cbb3847d125896e722a0d430aa`, against base `e6fa043c38c72effed33ff65fb2c7597c21491c7`. Read AGENTS.md, GOVERNANCE.md, README.md, both beads through `br --no-db show`, the implementer's report, all 16 changed files, the admission walk and its mono-item backstop, crate identity/export resolution, invocation/profile/cache handling, relevant Pure/callable handling and integration-test helpers, the existing sink and lock cases, and conformance format/rule documentation. Read reference sections naming all four codes, D7, D18 and the tier table, SINK/LOCK/channel rules, and E1 protocol §§2 and 10. Ignored the implementer's unofficial review verdicts.

All execution used the private archive `/tmp/tide-fx-u65-r1-codex.cAL6Ts`, created with the brief's `git archive baee486` command. No repository/worktree edits or Beads writes were made. Repository status was clean before and after inspection.

Independent checks:

- `git diff --check e6fa043 baee486`: passed. `bash -n conformance/run.sh`: passed. Private-copy `conformance/run.sh --trace`: SINK-8, LOCK-4 and CHANNEL-1 name the intended cases.
- Exercised the candidate runner's actual `check()` body with mocked compiler output: 12 assertions passed. Correct warning, error, and mixed marks passed; missing warning, unmarked warning, wrong code, wrong severity in either direction, warning-only rejection, bare WARN, repeated WARN, and WARN followed by a rustc error mark failed as required. No Cargo setup or compiler ran in these matcher probes.
- Exercised the same matcher with the real copied driver on all ten targeted cases, in normal and `--test` modes: ten PASS, exit 0. This driver-only harness skipped Cargo setup and the stable-rustc oracle; it is not a full conformance rerun.
- Ran separate single-file probes for channel aliases, callback adapters, re-exported macros in debug/release, and aliases that swap stdout/stderr macro spellings. Compiler invocations ran serially under `nice -n 19 timeout 45`, each gated by the specified quiet-until check. Relevant source and results appear below.

Executable probes used existing build artifacts copied from the candidate worktree, not a fresh build: driver SHA-256 `ff513828d439005e4f104516b906dcea3d9c74cb32a2c77d006287fa5f91a22a`; stub std SHA-256 `753daf95c3d4ed713206a5cddd9d08cef81087490c60b7ee643ef8599be0b0df`. Their build-to-commit provenance was not independently rebuilt. The findings also follow directly from the candidate's explicit type matches and early returns.

## Blocking

### B1. A function-pointer alias loses FX0608

**Location:** `crates/tide-admit/src/walk/sinks.rs:39–45`, channel check at `:57`; `conformance/cases/tasks/channel_library.rs:8–9`.

The call handler returns unless the adjusted callee type is `FnDef`. A typed function-pointer alias is `FnPtr`, so this library creates an unbounded channel without FX0608:

```rust
use std::sync::mpsc::{channel, Sender, Receiver};
use std::task::Tasks;
pub fn queues(tasks: &Tasks) {
    let direct = channel::<u8>;
    let _ = direct(tasks);
    let pointer: fn(&Tasks) -> (Sender<u8>, Receiver<u8>) = channel::<u8>;
    let _ = pointer(tasks);
}
```

**Observed:** `probe-src/channel_alias.rs` exited 0 and emitted FX0608 only at line 5, the `direct` call. Neither the coercion at line 6 nor the call at line 7 warned. Reference §7.3 and CHANNEL-1 do not exempt channels created through this alias. Decision 2's function-pointer limitation concerns lock callbacks, not channel constructors.

**Smallest fix:** retain the resolved channel identity when it is coerced to a function pointer, and emit FX0608 at that known use or its statically traceable local call. This needs no inference of arbitrary runtime pointer targets. Add a conformance case for the typed alias, plus its accepted binary counterpart; preserve warning severity and exit 0. The existing untyped alias case never exercises `FnPtr`.

### B2. Boxing a known callback suppresses FX0607

**Location:** `crates/tide-admit/src/walk/sinks.rs:73–80`; existing adapter handling in `crates/tide-admit/src/walk.rs:747` onward (`parallel_closure`).

The callback resolver unwraps references, but returns on `Box<F>`. Consequently these equivalent ways of supplying a local closure get different diagnostics:

```rust
use std::sync::{LockError, Shared};
pub fn report(a: &Shared<u32>) -> Result<(), LockError> {
    a.with(|n| { eprintln!("direct {n}"); })?;
    let show = |n: &mut u32| { eprintln!("ref {n}"); };
    a.with(&show)?;
    let boxed = Box::new(|n: &mut u32| { eprintln!("boxed {n}"); });
    a.with(boxed)?;
    Ok(())
}
```

**Observed:** the equivalent `probe-src/lock_callbacks.rs` exited 0 with FX0607 at lines 3 and 4 only; the boxed callback's print at line 6 was silent. Its concrete `Box<Closure>` type identifies the local body. This is neither a dynamic callback nor transitive analysis of arbitrary callees. The callback still prints while the cell is held, the situation reference §7.4 and LOCK-4 warn about.

**Smallest fix:** unwrap `Box<F>` as well as references before matching `Closure`/`FnDef`, using the `def.is_box()`/first-type-argument pattern already present in `parallel_closure`. Add a boxed-callback warning case while keeping uncalled nested bodies and logging accepted. The existing callback tests cover direct closures, references, and fn items, but no box.

## Non-blocking

**N1. The lexical limitation also skips an immediately invoked nested closure.** At `crates/tide-admit/src/walk/sinks.rs:90–116`, the visitor's default nested-body filter skips `a.with(|n| { (|| eprintln!("{n}"))(); })`. The callback probe confirmed no FX0607 there. This follows the recorded choice to avoid following callees, so I am not making it a separate merge blocker. Describe this actual boundary when recording Decision 2; “uncalled nested bodies” alone understates it. A later extension can distinguish known invoked local bodies from merely constructed closures without walking every nested body indiscriminately.

No findings require only v1/Later-tier text changes; there is nothing to route to fx-djp in this report.

## Decisions

1. **Library printing: agree with stdout only.** D7 explicitly owns “printing to stdout from a library package,” and reference §6.5 names the stdout lint. Reference §2.1's broader parenthesis is shorthand, not a reason to warn on stderr/logging. Keep the implementation's distinction. Renaming `eprintln!` to `println!` remained accepted, while renaming `println!` to another spelling warned in my probe. Direct write-only-handle analysis is not needed to deliver the specified macro cases in this slice.

2. **FX0607 reach: agree with lexical analysis instead of arbitrary transitive effects, with B2 required.** Sinks remain allowed and authority-free; this warning must not become a new purity restriction or rejection. Local closure bindings, references, and fn items are useful minimum coverage, and uncalled nested bodies must not warn merely because they are declared. A concrete boxed closure belongs in that same statically known coverage. Arbitrary function-pointer/dynamic targets may remain uninferred under the recorded narrow choice. Record the immediately invoked nested-closure limitation from N1 accurately. Opus remains the decision authority for this reading.

3. **FX0507 profile detection: agree.** `Invocation::rustc_args` fixes debug assertions on for debug and off for release; raw user flags cannot override it, and profiles participate in cache identity. The session setting therefore represents the existing operator profile reliably. Re-exported `dbg!` was accepted in debug and rejected with FX0507 in release. Do not add a CI profile to satisfy a release-only Acceptance condition.

4. **Warning mark grammar: agree.** Bare FX marks keep their error meaning; WARN switches subsequent FX marks on that line. The matcher retains severity and source line, requires warning-only success, and still requires expected diagnostics to exist. My 12 matcher assertions covered malformed marks, missing/wrong diagnostics, and mixed error/warning output. Passing the intended crate type explicitly under `--test` is necessary for the library/binary distinction and matches invocation.rs's interface.

5. **Fixed warning severity: agree for this slice.** Code owns severity once, and the three warnings do not increment rustc's error count. No current contract assigns these provisional FX codes registered lint names or requires an allow/deny interface. Keep the prescribed severities rather than expanding the operator interface. The existing success cache stores and replays compiler stderr, so successful warnings are not lost on cache hits.

6. **CHANNEL-1 trace: agree.** Naming the existing sentence introduces no new requirement, and the rule inventory, prefix table, and three cases are connected. Keep the existing sentence's meaning and fix its enforcement under B1.

## Reasoning

The direction fits the existing architecture: diagnostics own severity, the HIR walk uses rustc's resolved identities, macro checks reuse the expansion walk, and profile/library context comes from the controlled invocation. Renamed and re-exported macro definitions retain the right classification. The new runner does not pass warning cases vacuously: missing expected warnings, unexpected warnings, severity mismatches, and incorrect exit status all failed independent probes. The committed targeted cases exercise each requested code and its principal accepted alternative.

The reported Cargo and full-corpus successes remain the implementer's evidence. My independent driver-only checks support the narrow Acceptance examples, but B1 and B2 show the implementation is incomplete on ordinary source forms that preserve the same operation. Both are localized detection gaps, not reasons to replace the architecture. Keep the fixes at the resolved-use/callback boundary and extend the existing cases.

The changed cases and diagnostic text are small generic examples consistent with the cited rules. I saw no indication of held-out contamination. I read only the authorized E1 protocol sections and did not open held-out tasks, checked cases, solutions, or writer programs to compare them. Production channel/Shared execution remains outside this compile-time stub slice.

Not rerun: Cargo builds/tests or the complete conformance suite, as prohibited by the brief. Remaining risk: the two demonstrated forms bypass required warnings until B1 and B2 are fixed. DONE
