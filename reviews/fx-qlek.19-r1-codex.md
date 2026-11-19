Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate 0d34c99 (code)

## Verdict

changes required. Two blocking findings. Confidence: high in the source analysis; the additional failure cases below were not compiled, as instructed.

## Blocking findings

### B1 — A user's local type can have the matched printed path

**Location:** `crates/tide-admit/src/diag/labels.rs:174` (matching at lines 175–177); `crates/tide-admit/tests/build.rs:5832` (insufficient collision case).

The recorded crate name does not make a diagnostic's printed path a crate identity. With production std injected, this unrelated program can print the same `tide_std::task::Scope<_>` prefix:

```rust
pub mod tide_std {
    pub mod task {
        pub struct Scope<E>(pub E);
    }
}
pub fn make<E: Default>() -> tide_std::task::Scope<E> {
    tide_std::task::Scope(E::default())
}
pub fn f() { let _s = make(); }
```

Its E0283 concerns `E: Default`, with no `Tasks::scope` call. The pinned printer does not trim non-unique names, and prints local paths without the local crate prefix. Consequently the local module path is indistinguishable to this substring check from the external Tide std path. The existing root-level `struct Scope` negative misses that collision. The user receives task-specific help for their own type, violating the bead's no-other-E0283 contract and `docs/driver.md:31`.

**Minimum fix:** corroborate the printed path with actual type/call provenance, or fail closed when a local definition can print the same path. Add the program above to the negative cases while retaining the real-scope positive. Merely requiring the full printed path, a backtick, lifetimes, or an underscore is not an identity check.

### B2 — The unconditional explanation asserts a cause the matcher does not establish

**Location:** `crates/tide-admit/src/diag/labels.rs:156`, `crates/tide-admit/src/diag/labels.rs:175`; related contract at `docs/driver.md:28`.

The matcher checks neither a `?` nor a `TaskError` result context. For example:

```rust
use std::task::Tasks;
pub fn f(tasks: &Tasks) {
    let _ = tasks.scope(|s| { s.spawn(|| Ok(())); Ok(()) });
}
```

The scope's `E: From<TaskError> + Send` is unconstrained, producing the same E0283 subject, so it receives the help. There is no `?` and no enclosing `TaskError` result, yet the help says “`?` reaches `TaskError` from more than one type”. The builder's deliberate “any context” choice also covers custom error contexts, for which that asserted cause need not apply. “For example” qualifies the turbofish, not the subsequent causal assertion. `docs/driver.md` describes a narrower trigger than the implementation.

**Minimum fix:** make the help conditional or general enough to be true for all matched scope ambiguities, and describe that trigger accurately in the docs; alternatively establish the narrower TaskError/`?` context before emitting this specific explanation. Add the no-`?` case with its annotated counterpart to the existing build test. No change to `From<Cancelled>` or task semantics is needed.

## Non-blocking findings

- `design/reference-companion.md:49`: update the E0283 cell to mention tide-admit's turbofish help when the holder/coordinator integrates this candidate. This is already explicitly handed off in the builder report and was excluded from the builder's writes. `design/reference.md` needs no change: this adds supplementary diagnostic guidance, with no language/API change; the companion already owns this habit and its correction.
- The positive currently checks the source line but not the column or ordering (`crates/tide-admit/tests/build.rs:5819`). While updating the test for the blockers, the cheapest stronger output check is that the help immediately follows an E0283 at the same full location. The present source preserves both, so this is not independently blocking.

## Reasoning

**Contract and scope reviewed.** A Tide consumer encountering the target scope-inference E0283 should retain rustc's rejection and receive accurate, actionable help citing design/03 §6.1; unrelated diagnostics must receive no task hint. Read AGENTS.md, GOVERNANCE.md, the bead and comments first, then the five-file diff, the builder report/brief, the emitter and its callers, injected-std identity, the task API, build-test helpers, diagnostic-format parsing, relevant design/reference text, and conformance diagnostic extraction. This is one driver invocation per crate, not a service workload. No new dependency, orphaned caller, export/config change, or dead code was found.

**Frozen identity.** HEAD was `0d34c998aa4b61cafbc14fae2c0a431abc1a8269`, with a clean tracked working tree. Independently computed SHA-256 of `git diff 64af5e9..0d34c99`: `5a31b93d876f4cd7556d012ccef3958f85459b9cb0b336512c4382b81f93124e`. Five files, +103/-3. `git diff --check 64af5e9..0d34c99` passed.

**Matching.** The numeric `Some(283)` gate excludes E0282 and other error codes. Ordinary types in another crate with a different printed prefix, and the tested root-local `Scope`, do not match. This is compiler-message matching rather than source-text matching, but B1 shows why it is not fully robust identity evidence. No error-type argument or enclosing-return-type check exists, so non-TaskError contexts can match (B2). The hint path is recorded before the walk requests typeck; preceding expansion/synthesis/config collection does not request body typeck. Method chains and nested scopes have no syntactic exclusion: they match when rustc chooses the same unannotated Scope parameter as its inference subject. Likewise `?` versus `.map_err` is not distinguished. A diagnostic that instead chooses a generic argument, closure, alias, or another type as its subject will not match; universal coverage of equivalent inference shapes is not proven. Trusted macro expansions are not excluded by the matcher; the inner emitter performs rustc's existing macro-span adjustment. User-defined macros are already rejected by Tide. No extra shape/macro executions were performed.

**Output and verdicts.** `Labels::emit_diagnostic` forwards the original `DiagInner` unchanged before emitting a separate top-level `Level::Help` at its primary span, deduplicated per span. E0283 has no matching TD rule here, so its help immediately follows the first error for the site. The original code, primary span, relative ordering of original errors, and original abort/error count remain owned by rustc. The added help bypasses the diagnostic context and has no TD code; it does not increment rejection counts or change an exit status. The existing short-format emitter remains installed. JSON/human formats are already unsupported by the operator (`invocation.rs:325`), not newly lost. The conformance runner extracts error lines and TD error/warning lines (`crates/tide-conformance/src/main.rs:745,772`), so this uncoded help cannot change its expected code sets. The TaskError turbofish is correct for the tested positive, and the help includes the requested design citation. Accuracy beyond that context is B2.

**Pinned-source evidence for the new cases.** Read the installed `nightly-2026-08-09` compiler source under `/home/lucca/.rustup/toolchains/nightly-2026-08-09-x86_64-unknown-linux-gnu/lib/rustlib/rustc-src/rust/compiler/`. In `rustc_middle/src/ty/print/pretty.rs`, local paths omit the crate prefix (2397–2409), local definitions participate in name uniqueness (3470–3482), and duplicate names prevent trimming (3550–3584). `rustc_trait_selection/src/error_reporting/infer/need_type_info.rs` selects let bindings/closure parameters as inference subjects (1302–1381) and formats their types in the diagnostic (667–693). `error_reporting/traits/ambiguity.rs:288` routes ambiguous trait obligations to E0283. These are source-derived failure scenarios, not claimed runtime reproductions.

**Existing tests and proof.** The build test has the required failing positive, exactly one hint, citation/location, E0283/exit-1 assertion, a compiling turbofish counterpart, and unrelated sum/root-local-Scope negatives. The builder's `target/work/without-change.log` shows the positive failing at the hint assertion with two original E0283s and no help. `target/work/focused-0d34c99.log` records the restored candidate passing build (118), invocation (18), and code_prefix (2) tests; the builder report supplies candidate/environment/commands. This is inspected builder evidence, not an independent test run. The cheapest missing checks are the collision and no-`?` variants above, in the existing integration test; no new test harness is necessary.

**Required follow-up execution, not run here.** After adding those cases and fixes, the coordinator should run the focused build test through the required heavy wrapper (load checked first):

```sh
bash ~/.cache/eon-agents/eon-overseer/heavy.sh env CARGO_BUILD_JOBS=4 timeout 600 cargo test -p tide-admit --test build scope_ambiguity_gets_the_turbofish_hint -- --test-threads=4 </dev/null
```

Full workspace/conformance results remain pending for this review, as instructed. A changed compiled candidate needs proof for its changed inputs. Re-check B1 and B2 after fixes; a full restarted review is unnecessary unless a fix exceeds its finding or crosses a trust boundary. Opus review remains required on the Codex-clean tree.

Only this report was written. No build, test, compiler, Tide binary, Nix command, commit, or .beads write was performed. No override.

DONE
