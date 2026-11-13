Model: gpt-6.1-sol · Effort: high · Date: 2026-11-13

Verdict: request_changes

1. **Blocking — anonymous const bodies escape the preflight and erase all buffered FX diagnostics.** Evidence: `crates/tide-admit/src/walk/recovery.rs:52` inventories definitions, nested bodies and opaques, but omits ordinary anonymous constants. `crates/tide-admit/src/walk.rs:1856` then replaces the enclosing recovery state with a membership test for that omitted body owner and calls `typeck_body` at line 1858. This reproduces the original defect:

   ```rust
   pub fn a() { let _ = std::fs::read("x"); }
   pub fn b<T: Missing>(_: T) {}
   pub fn c() { let _: [u8; { b(0); 0 }]; }
   ```

   Candidate result: exit 1, E0405 on line 2, **no FX0501 on line 1 and no FX diagnostic anywhere** (`batch2/anon-call/stderr`). The s1 control retains FX0501. Array lengths in signatures and fields, repeat counts, const-generic defaults, and closure parameter types reproduce the loss. An unresolved trait directly inside the anonymous constant also reproduces it; E0404 and E0433 variants fail likewise. The Future variant `let _: [u8; { let _: &dyn std::future::Future<Output=()>; 0 }];` loses FX0601 too (`batch3/future-anon`).

   The pinned compiler confirms the inventory distinction: `rustc_middle/src/hir/map.rs:1393` adds anonymous constants only to `body_owners`; inline consts and closures additionally enter `nested_bodies`. `TyCtxt::hir_body_owners`, at `map.rs:372`, is a HIR-only inventory of all body owners. The `DefKind::AnonConst` match at recovery.rs:65 cannot help constants absent from the iteration.

   **Fix wanted:** include every body owner in dependency propagation and in the fallback set, using `tcx.hir_body_owners()` rather than the incomplete nested-body inventory. Ensure anonymous bodies inherit their enclosing/dependency recovery state. Add the direct-call and direct-Future anonymous-const regressions, asserting the unrelated code and the Future code as well as rejection. Merely checking for absence of an ICE/panic string does not detect this silent fatal unwind.

2. **Blocking — trait aliases can trigger fatal lowering through method lookup without a written reference in the caller.** Evidence: `crates/tide-admit/src/walk/recovery.rs:30` records explicit HIR path dependencies only, while the implicit-selection fallback at line 90 covers `Trait` and `Impl`, not `TraitAlias`. This file loses FX0501:

   ```rust
   pub fn a() { let _ = std::fs::read("x"); }
   pub trait Bad = Missing;
   pub fn c(x: u8) { x.absent(); }
   ```

   Candidate result: exit 1, E0405 and E0658 on line 2, **no FX diagnostics** (`batch4/alias-method/stderr`). The caller never writes `Bad`, so no dependency edge marks it. Replacing `absent()` with the valid `to_string()` call still loses all FX output. Importing the alias from a module, declaring it inside the function, and using a wrong-kind alias bound reproduce the defect. The existing test named `alias` exercises a type alias to `dyn Missing`, not a trait alias.

   This is a concrete rustc query dependency: pinned `rustc_hir_typeck/src/method/probe.rs:1136` expands in-scope trait aliases when assembling extension-method candidates, via `traits::expand_trait_aliases`. That can lower the alias's unresolved bound. The E0658 rejection does not stop Tide's callback from requesting these queries, as these probes show.

   **Fix wanted:** account for in-scope trait-alias expansion, including imports and nested scopes, before querying method-containing bodies. Include aliases wherever implicit lookup/selection hazards are classified; do not rely on an explicit reference to the alias in the caller. Add a regression using an actual trait alias and a caller with no written alias path, asserting retained FX0501. Preserve the crate's E0658/E0405 rejection.

3. **Blocking — the crate-wide HIR-only fallback drops independent diagnostics even where the required types can be obtained safely.** Evidence: `crates/tide-admit/src/walk/recovery.rs:86` through line 92 marks every definition untyped as soon as a bad trait/impl is reachable. With this input, the candidate reports only E0405:

   ```rust
   pub trait B: Missing {}
   pub fn c(x: u16) -> u8 { x as u8 }
   pub fn d() { Ok::<(), ()>(()); }
   ```

   Both FX0301 on line 2 and FX0702 on line 3 disappear (`batch3/global-trait/stderr`). The same loss occurs with an unresolved implementee, an inherent impl with a missing bound, a blanket Debug impl with a missing bound, and an impl of the bad supertrait. These are unrelated, known violations, not verdicts requiring invented types. This contradicts the brief's contract to keep diagnostics on other items/lines; recording it as a limitation does not satisfy that contract.

   A standalone callback built against the **same pinned rustc** requested `fn_sig(c)` and `typeck(c)` on each of those five error-tainted crates. Every request completed, every result was **untainted**, and the cast's recorded types were `Some(u16) -> Some(u8)` (`batch3/*/query.stderr`). The ordinary clean cast control reports FX0301. Thus a blanket ban on obtaining even these types is not necessary. This experiment proves that safe subset; it does not claim arbitrary method lookup or trait selection is safe.

   **Fix wanted:** separate guards on unsafe selection/normalization from checks that can safely use known signatures and expression types. Keep independent safe owners/checks active through their existing rule owners instead of returning all definitions as untyped. Add known casts and ignored Results beside bad trait/impl definitions to the regression tests. A replacement must also address findings 1 and 2; simply deleting the fallback without guarding implicit query dependencies would be unsafe.

4. **Blocking — the new whole-result taint gate regresses existing feedback for ordinary unresolved types.** Evidence: `crates/tide-admit/src/walk.rs:1927` returns before `surface_expr` whenever any part of the body's typeck results is tainted; `typeck_of` at line 421 similarly filters out the whole result. This affects crates with no unresolved trait and no active recovery set:

   ```rust
   pub fn c(x: u16) -> u8 {
       let _: Missing;
       x as u8
   }
   ```

   Candidate result: E0425 on line 2, no FX0301 on line 3 (`batch5/partial-cast-lines/stderr`). Removing only the erroneous declaration restores FX0301 with its normal error severity and source line. The pinned callback confirms that the partial result has `tainted=true` but the cast still has `Some(u16) -> Some(u8)`. The base walk calls `surface_expr` on this result; its existing cast check at `walk/surface.rs:168` already requires both node types to exist and contain no error before reporting. The new early return bypasses that safe check. This is additional diagnostic loss introduced outside unresolved-trait recovery.

   **Fix wanted:** retain query-free checks over independently known node types while guarding operations that select, normalize, or query dependent definitions. In particular, let the existing cast rule evaluate its known primitive inputs without enabling all of `surface_expr`'s other query paths on tainted results. Add the multiline example and its unknown-operand twin: the former must retain FX0301, the latter must not invent it.

Review scope and conclusions: frozen HEAD `7a771467f42916abcf60ed3d3d8a22c37b920bde`, base `71127b1cf5ff07fee11c626321c14fdfab6bf0ce`; diff SHA-256 `7a3d38dfe88d96adc928a397ab85b677feb9ab035cfe5768d4e0d71d4165547f`. Read the diff, touched files and connected walk/query/diagnostic paths, invocation/artifact lifecycle, AGENTS.md, GOVERNANCE.md, the bead and epic, brief, author report, source Opus finding, relevant reference/authority specs and research, and the cited pinned lowering implementation. Confidence in these findings is high. The direction of guarding fatal queries and reusing the walk is sound, but this candidate is not Codex-clean and must not advance to Opus as a clean tree.

The named s1 and ordinary Future forms work in the probes. Recovery's initial scan is gated on the diagnostic context having errors (`recovery.rs:5`); the normal error-free branch returns an empty set. No clean-source behavior difference was demonstrated, though the changed walk is not literally byte-for-byte identical to the old code. Unresolved-path diagnostic recovery remains in authority.rs and uses existing policy owners; the new file is query availability bookkeeping, not a second FX rule table. Skipping hole serialization for these rejected crates is appropriate: invocation.rs permits interface writing only after successful analysis with no errors. None of the failing probes was admitted, and no panic/ICE marker appeared; findings 1 and 2 concern silent fatal unwinds and lost feedback. The unknown-cast and unknown-receiver controls produced no invented typed FX verdict.

Probes run:

All candidate runs reused `target/debug/tide-admit` (SHA-256 `50b4bd7542a6eb62365e9952a801e311ec1f1a283cecc4d1cec77a417c6a9cf4`), built after the final source edit. Scratch root `P` is `/home/lucca/.cache/eon-agents/tide-fx-u65.7-work/codex-r1-probes`. Source files, per-case stderr and exit results remain there. Five serial batches ran **60 candidate invocations**. Each invocation used a separate package/output directory and a 25-second timeout with stdin closed. The runner recorded compiler status; batch exit 0 means the runner completed, not that rejected candidates passed admission.

Each batch used this outer command, with SCRIPT replaced by the absolute runner path listed below:

```sh
(ulimit -c 0; CARGO_BUILD_JOBS=4 timeout 600 bash /home/lucca/.cache/eon-agents/eon-overseer/heavy.sh nice -n 19 python3 SCRIPT </dev/null)
```

- `P/run.py`: 15 cases. s1, type aliases, simple anonymous/inline const controls, qualified and wrong-kind paths, a bad trait with independent typed violations, nested functions, macros and opaque returns. s1 and ordinary controls retained feedback; the global trait case dropped the cast/Result codes.
- `P/batch2/run.py`: 12 cases. Nontrivial anonymous-const bodies in array lengths, repeat counts, defaults, signatures, fields and closure parameters; direct missing dyn traits and inherited dependencies; partial typeck casts. Nine anonymous-const cases lost all FX diagnostics. The partial-result cases dropped known casts.
- `P/batch3/run.py`: 17 cases. Future impl/bound/dyn/argument/import/return/reexport/macro/anonymous-const forms, unknown receiver, five global trait/impl cases and cast control. Future runs used the author's built fixture `target/tmp/unresolved-future/tide-std/libtide_std.rlib`. The named Future forms retained error FX0601 on their usual lines; the anonymous-const form lost it. The `future-glob` case exited 0 because that fixture's Rust-2024 prelude already supplies Future; it is not evidence of unresolved-path recovery. All other candidate cases exited 1.
- `P/batch4/run.py`: 12 cases. Eight actual trait-alias shapes, E0404/E0433 anonymous constants, and closure/opaque nested-function controls. Seven trait-alias lookup shapes and both anonymous-const shapes lost all FX output. Explicit alias-bound and closure/opaque controls retained unrelated FX0501.
- `P/batch5/run.py`: four cases. Multiline partial-typeck casts, with/without a separate missing trait, unknown-operand control and known-cast control. Both known partial casts lost FX0301; the clean known cast reported it; the unknown cast reported no invented FX0301.

The standalone query probe is `P/query.rs`, a small rustc callback, not a modified Tide build. It queries only the named function `c` and prints the result's taint and recorded cast types. It was built through the same queue with:

```sh
(ulimit -c 0; CARGO_BUILD_JOBS=4 timeout 600 bash /home/lucca/.cache/eon-agents/eon-overseer/heavy.sh nice -n 19 /home/lucca/.rustup/toolchains/nightly-2026-08-09-x86_64-unknown-linux-gnu/bin/rustc --edition=2024 /home/lucca/.cache/eon-agents/tide-fx-u65.7-work/codex-r1-probes/query.rs -L native=/home/lucca/.rustup/toolchains/nightly-2026-08-09-x86_64-unknown-linux-gnu/lib -C link-arg=-Wl,-rpath,/home/lucca/.rustup/toolchains/nightly-2026-08-09-x86_64-unknown-linux-gnu/lib -o /home/lucca/.cache/eon-agents/tide-fx-u65.7-work/codex-r1-probes/query </dev/null)
```

There were two builds: the initial callback and then the same callback with cast-type reporting. Both exited 0. The initial six-case query run, one direct partial-result run, and final seven-case `P/query-run.py` run all used the queue and outer 600-second timeout. The final run proves the current probe source: five bad trait/impl crates returned untainted typeck results and known cast types, the clean control did likewise, and the partial-result case returned tainted results with the same known cast types. Error-tainted inputs still exited 1. The seven-case runner's outer command is the batch command above with SCRIPT=`P/query-run.py`; each child has a 25-second timeout. One earlier attempt to construct a combined build/runner command was blocked by the command guard before execution; using a literal compiler command and separate runner succeeded without changing guard policy.

All queued work finished by 01:02:20Z, before the cutoff. `git diff --check 71127b1..7a77146` passed; HEAD, diff hash and clean worktree were rechecked. No candidate files, staging, commits, br state or other worktrees were changed. No held-out files were opened or searched.

Not run: full workspace/corpus suites, production-std rebuilding, or the later Opus review; these remain with the coordinator. Current risk: rejected crates can still lose their FX feedback. DONE
