Model: gpt-6.1-sol · Effort: high · Date: 2026-11-12
Verdict: request_changes

Reviewed frozen HEAD `783345b42580973ee14a8fdaef06beff2fc87729` against main `d02ccf539f699785b4f5beb2592f2f706e74d8e6`. Diff SHA-256: `03a55700b8692b24f9c9fad4204c3d314fe67ffa1a062b1f7542803f714f1bd3`. Confidence: high in the findings below. The direction and shared-runner architecture are sound, but valid admitted source can fail its test build because of synthesized code.

1. **Blocking — copied requires text is corrupted by single-line insertion.** Evidence: `crates/tide-admit/src/synthesis/contracts.rs:386-391`, together with `crates/tide-admit/src/synthesis.rs:65-66`. The generator interpolates the original clause directly into an attribute, then `append` replaces every newline with a space. This valid source passes ordinary admission but fails `--test` with an unclosed delimiter:

   ```rust
   #[requires(n > // lower bound
   0)]
   #[ensures(result == n)]
   fn ident(n: u8) -> u8 { n }
   ```

   The comment consumes the remaining synthesized module after flattening. A newline inside a string literal can instead change the filter's meaning. This also contradicts the author's reported decision to escape requires text in metadata. **Fix:** transport requires through an escaped string attribute, using the existing `__tide_requires:` consumer, then parse it inside the macro. Preserve literal contents and comments without adding physical source lines. Add a driver regression for the comment and a multiline literal. Probe logs: `r1-probes/multiline-comment-{control,test}.log` beside this report.

2. **Blocking — capability parameters disappear from the copied precondition's scope.** Evidence: `crates/tide-admit/src/synthesis/contracts.rs:304-313`, `:380-391`, `:423`. Capability inputs are omitted from the wrapper parameters and projected only into numbered locals inside its body. The original requires is copied unchanged onto the wrapper, where the original capability name does not exist. For example:

   ```rust
   #[requires({ let _ = fs; true })]
   #[ensures(result)]
   fn check(fs: &std::fs::Fs) -> bool { true }
   ```

   Ordinary admission succeeds; `--test` fails E0425, `cannot find value fs`. **Fix:** make the projected capability available under the original binding when evaluating the copied requires, using the same case simulator for the predicate and target call. Keep ordinary-argument guidance and the rejection floor. Add a capability-referencing requires regression. Probe logs: `r1-probes/cap-requires-{control,test}.log`.

3. **Blocking — static reference inputs produce an invalid wrapper instead of an unchecked entry.** Evidence: `crates/tide-admit/src/synthesis/contracts.rs:298-329`, `:423`; `crates/tide-macros/src/test.rs:185-216`. A function such as `#[ensures(result == text.len())] fn size(text: &'static str) -> usize { text.len() }` passes normal admission. Synthesis prints a borrowed wrapper parameter and generates temporary owned storage, then calls the original static-reference function. Its test build fails E0521 because the borrow cannot satisfy `'static`. **Fix:** preserve and inspect lifetime requirements before emitting a supposedly runnable entry; emit `#[ignore = "unchecked: ..."]` when temporary generated storage cannot satisfy them. Add a driver regression proving that this signature builds and lists as unchecked. Probe logs: `r1-probes/ref-static-{control,test}.log`.

4. **Blocking — the unconditional generated glob import breaks valid lint configurations.** Evidence: `crates/tide-admit/src/synthesis/contracts.rs:49`. Every injected module contains `use super::*`, even when its target and generated types use absolute paths and it has no requires needing parent names. The probes above already emit `unused import: super::*`. Consequently `#![deny(unused_imports)]` plus `#[ensures(result == n)] pub fn identity(n: u8) -> u8 { n }` fails solely on the injected import. `#[deny]` is an admitted attribute (reference §2.3). **Fix:** narrowly allow unused imports on that generated import, or emit it only when needed. Add a regression using `deny(unused_imports)`. This finding is established by the emitted warning and source inspection; I did not start another compilation during the quiet window.

5. **Non-blocking — the harness README still describes the removed deliberate failure.** Evidence: `trials/e1-checked/README.md:187-189`. It says a plain libtest pass still leaves an explicit writer-test failure pending these prerequisites, while this patch removes precisely that failure. **Fix:** describe execution of all four test forms, visible allowed-vacuity reports, and the generated-module exclusion from acceptance listings.

The minimum patch for approval addresses findings 1–4 and adds their focused driver regressions. Finding 5 is a small documentation follow-up within this bead's stated documentation scope.

Review coverage: read the changed code, callers, contract/property runtime and synthesis boundaries, required bead decisions, author/research reports, relevant specs, and permitted E1 files/sections. The five files shared with fx-ugm/fx-zc1 retain their Shared/task-limit/deadline documentation and tests; I found no separate merge conflict. Production and conformance include the same modified test, contract and simulator implementations. The harness filters the exact reserved segment only for acceptance and keeps writer entries. The first-pass reserved-metadata rejection and byte-range provenance fit together; I found no additional admission bypass. No changed fixture or diagnostic looked copied from held-out task material; restricted E1 files were not opened for comparison.

Verification limits: no candidate edits, staging, commits, br writes, full-workspace build or heavy.sh run. Focused metadata-only driver probes used existing worktree driver/std artifacts, nice 19, CARGO_BUILD_JOBS=4, timeout 300 and closed stdin, after quiet/load/memory/next-window checks; they finished before 05:20 UTC. Their ordinary-build controls passed and the three faulty test builds failed as described. The artifacts predate the rebase; source inspection and the identical range-diff tie these diagnoses to the frozen patch, but these probes are not fresh full-candidate suite proof. No further compiling probe ran during the quiet window. Full suites and reference readiness on the final rebased candidate remain unverified and are the coordinator's scheduled work. Candidate worktree remains clean.
DONE
