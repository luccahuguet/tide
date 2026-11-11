Model: gpt-6.1-sol · Effort: xhigh · Date: 2026-11-10

Verdict: request_changes

1. **blocking — The public runner reuses mutable captures after a panic.** Evidence: `crates/tide-std/src/lib.rs:64`, `crates/tide-std/src/test/mod.rs:14`, `crates/tide-std/src/test/runner.rs:99`, `crates/tide-std/src/test/runner.rs:119`, `crates/tide-std/src/test/runner.rs:177`. `run` is available in ordinary production builds, accepts an arbitrary `FnMut`, catches its panic with `AssertUnwindSafe`, then invokes that same closure during shrinking. This lets admitted code observe data the panicking callback was mutating. For example:

   ```rust
   let mut partial = 0;
   std::test::run("demo::partial", |g| {
       let _ = g.index(2);
       if partial == 0 {
           partial = 1;
           panic!("operation failed halfway through");
       }
       println!("observed partial state: {partial}");
   });
   ```

   The first call panics; the first deletion candidate calls the closure again and prints the partially updated state. Eventually resuming the original panic does not undo that observation. This conflicts with D5 (`design/01-design.md:370`–`382`), which allows recovery only through isolation and forbids observing a panicking task's mutable state. `#[doc(hidden)]` does not restrict callers. **Fix wanted:** restrict the public runner to a noncapturing callback, such as `fn(&mut Gen) -> R`; the macro's generated callback already captures nothing. Keep any mutable callback machinery needed by internal tests private. Add a driver check that ordinary code cannot pass a closure borrowing mutable caller state, while generated property entries still execute. The chosen public-runner design does not require exposing unwind-unsafe captures.

2. **blocking — An exhausted replay can hang inside one shrink evaluation.** Evidence: `crates/tide-std/src/test/mod.rs:80`–`93`, `crates/tide-std/src/test/runner.rs:172`–`177`, `crates/tide-std/src/test/runner.rs:216`–`223`. A missing or mismatching choice supplies zero forever and appends every such draw to `choices`. A normal user generator that retries zero therefore never returns when the first deletion candidate removes its entire choice sequence:

   ```rust
   #[derive(Debug)]
   struct Positive(u8);
   impl std::test::Arbitrary for Positive {
       fn arbitrary(g: &mut std::test::Gen) -> Self {
           loop {
               let n = <u8 as std::test::Arbitrary>::arbitrary(g);
               if n != 0 { return Self(n); }
           }
       }
   }
   #[test]
   fn positive(p: Positive) { assert_eq!(p.0, 0); }
   ```

   Once the initial generation returns a positive value, the property fails. Shrinking then evaluates an empty replay; every subsequent draw is zero, so it loops and grows the choice vector until timeout or allocation failure. The 1,000-evaluation and 128-stall limits cannot help: they are checked outside this one evaluation. The seed is printed, but the promised shrunk input never is. **Fix wanted:** make replay exhaustion/default expansion finite and deterministic, and reject an overrun as an invalid shrink candidate rather than a property failure. An internal replay-exhaustion outcome is sufficient; no wall-clock cutoff is needed. Add a regression using a retrying custom generator and retain the last valid counterexample.

3. **blocking — Existing implementations are traversed by type arguments rather than their actual generator requirements.** Evidence: `crates/tide-admit/src/synthesis.rs:167`–`197`. This produces both missed synthesis and false paired-library errors:

   - For a local `struct Wrap<T>(T)` with `impl<T: Arbitrary> Arbitrary for Wrap<T>`, discovery returns immediately at `has_impl`. A property over `Wrap<Leaf>`, where `Leaf` is a local `#[derive(Debug)] struct Leaf(u8)` without an explicit generator, never discovers `Leaf`. The generated call consequently requires a missing `Leaf: Arbitrary`, although every field can be generated.
   - For a paired library's `Marker<T>(PhantomData<T>)` with an unconditional `impl<T> Arbitrary for Marker<T>`, discovery follows every type argument. A property over `Marker<Opaque>`, where the same library defines a Debug `Opaque` without `Arbitrary`, gets the new paired-library diagnostic even though generating `Marker<Opaque>` does not require generating `Opaque` at all. The suite already has a valid generator and needs no library synthesis.

   **Fix wanted:** preserve user ownership of the existing impl, but follow its instantiated `Arbitrary` requirements instead of stopping at every local impl or recursing through every foreign type argument. The existing trait-selection/instantiated-clause machinery in `crates/tide-admit/src/walk.rs:1606` and `:1630` provides the surrounding pattern. Test both cases through the driver, including the paired path. This keeps the legitimate diagnostic for a genuinely required library type without an impl.

4. **blocking — Borrowed parameter support depends on spelling instead of resolved type.** Evidence: `crates/tide-macros/src/test.rs:60`–`79`, `crates/tide-std/src/test/arbitrary.rs:19`–`27`, `crates/tide-admit/src/synthesis.rs:107`–`113`. The macro allocates borrowed storage only for a direct `syn::Type::Reference`. These equivalent parameter types therefore behave differently:

   ```rust
   type Text<'a> = &'a str;
   #[test] fn direct(s: &str) { let _ = s.len(); }
   #[test] fn alias(s: Text<'_>) { let _ = s.len(); }
   #[test] fn parenthesized(s: (&str)) { let _ = s.len(); }
   ```

   The first uses generated String storage. The others expand to an `Arbitrary` call on the reference itself, for which there is no impl. HIR discovery resolves and strips the reference but never corrects that lowering. The same problem affects aliases for mutable references and borrowed slices. Reference §10.2 (`design/reference.md:909`) promises generation by parameter type, and the Rust surface gives aliases and parentheses the same type meaning. **Fix wanted:** select the owned-storage lowering from the resolved parameter type, using the existing driver planning seam or equivalent type-directed dispatch. Merely peeling syntax groups fixes only the parenthesized case. Add driver coverage for aliased shared/mutable references and retain the existing direct-reference cases.

5. **non-blocking — Availability and equivalence documentation retains the replaced test surface.** Evidence: `README.md:225`–`227`, `conformance/README.md:140`–`145`, versus `conformance/rustc/std/src/lib.rs:45` and `:57`. The root README still says the stub uses a different oracle test attribute, and the conformance README still says property generation is unavailable in production. Both are false on this candidate: the macro and runtime sources are shared. **Fix wanted:** update those two descriptions to the delivered property-test slice and its remaining planned features. Also keep the disclosed inline-insertion column limitation visible when describing span preservation; the regression proves original line numbers, not original columns.

What this change does: it lowers parameterized tests into libtest entries, implements deterministic generation and choice-based shrinking in production std, and synthesizes structural generators through a second admitted compilation. The direction is consistent with the recorded design decisions; the implementation needs the four correctness fixes above.

Confidence: high in the source-derived counterexamples and admission-flow assessment. The counterexamples were not executed under this review's machine gates.

Reviewed candidate: `ec531965dd7f1b47812973aae6e0e696b07d95b8` on `3ccbccdd9ce549cdbf9587a0ff89b26a9dbc8029`; verified diff SHA-256 `878583ab57f4b92f93d9ba1d1abb782bde606e752d48a1ea6a8271f003cfc858`. The worktree was clean. `git diff --check 3ccbccd ec53196` passed.

Coverage and remaining limits:

- Read AGENTS.md, GOVERNANCE.md, the requested beads and binding comments, both author reports, the complete diff, the runner/generator/macro/synthesis sources, their build tests and callers, invocation/cache/source-loader/admission code, the testing specs, D5/D14, and the added research report. The Opus report was not read.
- The overlay path preserves the immutable source snapshot and the paired-library compiler arguments. Its final compilation still runs `check_expanded`, `check_crate`, and `check_mono_items`, recomputing the paired crate identity; inserted impls receive ordinary admission. I found no admission check skipped or weakened by the recompile. Cache inputs include the driver executable, trusted compiler identity, std/dependencies, flags and snapshot; the paired path builds fresh artifacts. One negative admission regression that forces an overlay would strengthen the tests when these fixes are made.
- Integer/float/char encodings, built-in collection bounds, shortlex acceptance, sign-preserving minimization, deterministic case-seed override, and the count/stall accounting are coherent by inspection apart from finding 2. Panic identity compares location and payload type; the thread-local hook delegates panics on other threads and restores nested capture state. The existing integration test checks distinct panic locations, but does not directly test concurrent hook delegation.
- Inline insertions really do shift the columns of following user tokens on the same physical line (`synthesis.rs:43`–`50`); the author correctly discloses this. The current line-preservation spike passes the chosen line-only observation. Exact diagnostic/clause spans remain an unresolved limitation for downstream consumers, rather than a guarantee established by that test.
- Production and conformance use the same property runtime and test macro. The public runner and HIR overlay are usable extension points for fx-m72.2/.3 and fx-mhd, after fixing their current generator-discovery and callback boundaries. Requires guidance, simulator seeds, contract generation and structured contract identities remain correctly identified as downstream work.
- No files under `trials/e1-checked/` were opened or copied. No repository files, Beads state, staging or commits were changed. Only this requested report was written.

Minimal patch for approval: fix 1–4 at their existing owners, add the corresponding focused regressions, and update the two stale documentation paragraphs. No new dependency or alternative runner architecture is required.

Not checked: no builds or executable probes were run. The initial one-minute load was above 12; a quiet window was active when it later fell below 12. The reported frozen-tree workspace result (104 passed) and conformance result (215 PASS / 24 AWAITS / 0 FAIL) are author proof, not independently rerun evidence. External research links were read as supplied in the report, not independently revalidated. DONE
