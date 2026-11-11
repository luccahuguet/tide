Model: claude-opus-5-5 · Date: 2026-11-11

**Verdict: approve.** All five re-checked findings are resolved. I found no new false positives, false negatives or false labels in the expanding-cycle graph. There are two new non-blocking findings, both below; neither blocks landing.

**What I checked against:** HEAD 88c69c5 on base d9a02c0. `git diff d9a02c0 HEAD | sha256sum` gives `9b8b32b1…a8b`, which matches. The checkout stayed clean, and the driver's SHA-256 (`a30503b3…094`) matches `freeze-r3.json`. I made no edits, ran no git or `br` writes, and did not open `trials/e1-checked/`.

**How I ran the probes:** with `target/debug/tide-admit` against Rust std. The oracle was `rustup run stable rustc --emit=link -C link-dead-code`. For the rejected cases, I added an entry function so each one is instantiated, and rustc then has to show the growth is real. Every run was serial under `nice -n 19` with `CARGO_BUILD_JOBS=4`, stdin closed and a timeout. Before each run I checked that load was under 12, at least 6 GB was available (it was about 14.7 GB) and the quiet window had expired. Load once reached 13.4, and I paused until it dropped. Probes and logs are in `/tmp/claude-1000/r4u653/`.

## Status of the round-3 findings

1. **Unresolved dispatch edges — resolved.** `traits.rs:208-213` now returns for `None`, `Builtin` and non-local impls. `wrap_default` is accepted (driver 0, stable 0). The new `DefaultGrow` reject case covers a selected impl that uses the default body. Removing the early `callee.is_local()` return lets a local impl that overrides a foreign trait's method enter the graph.
   - No regressions: `a25` (a default body calling a fixed-shape override) is accepted. `a23` and `a26` (growth only through parameter dispatch) are accepted with no label, which is the documented fallback.
2. **Expanding-cycle check — resolved, and it holds up.** The results below show driver / stable rustc.
   - **Mutual recursion:**
     - `a01` (2 functions, 2 parameters, swap plus wrap) and `a03` (3 functions, 3 rotating parameters) get FX0407; instantiated, rustc hits its recursion limit.
     - `a02` and `a04` (rotations with wrapping into a discarded slot) are accepted and build.
   - **Selected impl methods with substituted impl arguments:**
     - Rejected, and each instantiated case hits rustc's limit: `a08` (`W<T>` to `W<Option<T>>`), `a10` (`P<A,B>` to `P<B,Vec<A>>`), `a11` (a method-level parameter), `a13` (a lifetime slot before `T`, so indexes must line up), `a14` (impl method to free function and back) and `a24` (default body to free function and back).
     - Accepted and built: `a09` (a swap) and `a12` (a lifetime slot with fixed shapes).
   - **Const generics:** `a28` (const slot, growing parameter) gets FX0407 and hits the limit; `a29` (const slot, discarded wrap) is accepted. So lifetime and const slots line up with rustc's argument indexes.
   - **Normalized projections:** `a15` (`<Wrap<T> as Tr>::A = Option<T>`) and `a17` (the same inside impl arguments) are rejected and hit the limit. `a16` (a projection that drops the wrapped `T`) is accepted.
   - **One parameter both bare and under a constructor in the same call:** `a05` (`f::<T, Vec<T>>`) is accepted and builds. `a06` (`f<U,(T,U)>`) is rejected, and stable rustc times out from exponential type growth. `a07` (`(T, u8)` in `T`'s own slot) is rejected.
   - **Closures and inherent methods:** `a18` (a call inside a closure) and `a19` (a closure type that captures `T`) are rejected; `a19` hits the limit. Of the inherent methods, `a20` and `a22` are rejected and `a21` (a permutation) is accepted. `a27` (an fn item through a binding) is rejected.
   - **Large graphs:** every case terminates. 300 functions took 4.7 s acyclic and 2.9 s cyclic (299 FX0407). 1500 acyclic functions took 114.6 s, against 0.25 s for `rustc --emit=metadata`. That cost is new finding A.
   - `reset` and `fixed_wrap` are accepted (0 / 0).
3. **FX0401 when no module can hold the impl — resolved.** `placement_foreign` and `Display for Box<Local>` get the new wording without "move the impl". Wrong-module impls (`p03`) keep "move the impl". `conformance/README.md:109-112` states that such impls are not admitted.
4. Not re-checked; it went to `fx-djp`. The bound-argument probes still fail with rustc E0275 and get no FX label, as before.
5. **research/27 — resolved.** Lines :54, :110 and :130 are corrected. C1 points to C6, and C6 has the "Adopted with changes (fx-u65.3)" note with the three required points. It describes the check accurately.
6. **Nit — resolved.** `trait_recursion_accept.rs:30-37` now covers parameter dispatch next to a default body.

**Corpus:** the PASS/FAIL/AWAITS lines in `conformance-r3.log` and `conformance-r2.log` are identical (292 lines each; diff exit 0). The workspace log sums to 165 passed with no failures.

**Docs:** `conformance/README.md:116-129`, `README.md:66-68` and the REF-0407 row match what the code does. One wording nit: the REF-0407 row (`conformance/rules.md:160`) has a comma splice ("…require selected local impls, unresolved aliases … prove no growth").

## New findings (both non-blocking)

- **A. Low severity, efficiency — `crates/tide-admit/src/walk/traits.rs:252`.** `ceiling_cycles` runs a separate reachability search for each expanding edge, so its cost is roughly the number of expanding edges times the graph size.
  - Each parameter occurrence also pushes a duplicate `Call` (`:235`).
  - With per-parameter nodes, dense generic code costs about 100× more than the old function-level graph did.
  - **Measured:** 1500 functions, each with 4 parameters and 2 calls passing crossed tuples, took 114.6 s; rustc takes 0.25 s on the same file. 300 functions took 4.7 s. The input is synthetic and the check still terminates.
  - **Fix:** compute strongly connected components once (Tarjan) and report each expanding edge whose two ends share a component. That is linear and gives the same verdicts. Remove duplicate edges as well. This can land now or go to `fx-djp`.
- **B. Low severity, label wording — `traits.rs:103`.** The no-module message also fires for an inherent impl on a local trait object, `impl dyn Local { … }`, placed in the trait's own module (probe `p01`; rustc accepts it). That impl does sit in the module that defines its type, so "no module can hold this impl" and the From/Into example mislead there.
  - The rejection itself predates this change; only the wording is new. Whether §5.2 admits `impl dyn LocalTrait` is a C8 reading.
  - **Recommendation:** add a comment on `fx-djp`. The hard-coded From/Into example is also a nit for impls like `Display for Box<Local>`.
