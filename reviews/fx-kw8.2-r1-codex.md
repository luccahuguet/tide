Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-21.

## Verdict

**request_changes — high confidence.** The architecture is reasonable, but the candidate has authority leaks, a reproducible checker crash, false rejections, and missing or incorrect diagnostics.

## Scope

Reviewed HEAD `7f3b95a41e7a2ec6f8d2be34dac96a87bb8e4cca`, parent `8c352065347a146569a3c223199160f80faaf997`.

Read AGENTS.md and GOVERNANCE.md fully, bead `fx-kw8.2`, the complete change and surrounding driver code, relevant conformance cases and runner, D18, the requested memory/authority rules, and the reference’s diagnostic descriptions. Inspected pinned rustc sources for visitor behavior, expansion provenance, formatting builders, and `ExprUseVisitor`.

Ran `git diff --check` and targeted stdin probes using existing driver/std artifacts. Probes included an independent borrow error to stop compilation before artifact generation. The recursive crash was also compared with pinned rustc. **Neither full suite ran:** the read-only sandbox prevents their build/output writes. Existing artifacts were not rebuilt. No files changed; the worktree remained clean.

## Blocking

1. **P1 — Formatting builders smuggle a non-Pure formatter through the std shortcut.**  
   [pure.rs:320](/home/lucca/pjs/lab/tide-fx-kw8.2/crates/tide-admit/src/pure.rs:320), [02-memory.md:943](/home/lucca/pjs/lab/tide-fx-kw8.2/design/02-memory.md:943)

   `DebugStruct`, `DebugTuple`, `DebugList`, `DebugSet`, and `DebugMap` hold a `Formatter` but have no type arguments. They therefore pass the new check. Confirmed: this produces no FX0603:

   ```rust
   pub fn f(f: &mut std::fmt::Formatter<'_>, c: &std::sync::Shared<u64>) {
       let mut builder = f.debug_struct("x");
       let _ = c.with(|_| builder.finish());
   }
   ```

   Formatting can invoke the formatter’s arbitrary writer, including synchronization, inside `Shared::with`. This violates PURE-1 and LOCK-4. The added design paragraph’s claimed equivalence to the field walk is therefore false.

   **Minimal patch:** include these transitive formatter holders in the audited std exceptions, correct the paragraph, and add a regression for a builder captured by `with`.

2. **P1 — Changing recursive substitutions overflow the checker’s stack.**  
   [pure.rs:317](/home/lucca/pjs/lab/tide-fx-kw8.2/crates/tide-admit/src/pure.rs:317)

   The visited set recognizes only repeated *instantiated types*. This valid recursive shape never repeats one:

   ```rust
   use std::marker::PhantomData;
   pub struct R<T> {
       tag: PhantomData<T>,
       next: Option<Box<R<Vec<T>>>>,
   }
   fn require<T: Pure + ?Sized>(_: &T) {}
   pub fn check(x: &R<u8>) { require(x); }
   ```

   The driver aborted with a stack overflow, exit `-6`; pinned rustc reached only the independent control error.

   **Minimal patch:** make recursive substitution analysis terminate while preserving PURE-1’s greatest-fixed-point result. Add a controlled exhaustion guard and regressions for growing substitutions with both Pure and non-Pure components. Merely remembering `DefId` would incorrectly accept some changing substitutions.

3. **P1 — Default trait methods can falsely promise a Pure opaque result.**  
   [walk.rs:680](/home/lucca/pjs/lab/tide-fx-kw8.2/crates/tide-admit/src/walk.rs:680)

   The comment distinguishes required methods from methods with default bodies, but the condition skips both:

   ```rust
   pub trait T {
       fn value(c: std::time::Clock) -> impl Pure { c }
   }
   ```

   Confirmed: no FX0517, including when an implementation inherits this method and its result enters another Pure-bounded call. This launders `Clock` through an opaque type, violating OPAQUE-2 and PURE-1.

   **Minimal patch:** skip only trait declarations without a hidden type. Check default bodies’ hidden types against their stated bounds and add this regression.

4. **P2 — Pure obligations in declaration-only type positions are missed.**  
   [walk.rs:1262](/home/lucca/pjs/lab/tide-fx-kw8.2/crates/tide-admit/src/walk.rs:1262), [walk.rs:1344](/home/lucca/pjs/lab/tide-fx-kw8.2/crates/tide-admit/src/walk.rs:1344)

   Generic arguments are checked through body type-check results. Declaration-only types lack those results:

   ```rust
   pub struct NeedsPure<T: Pure>(T);
   pub struct Bad {
       pub field: NeedsPure<std::time::Clock>,
   }
   ```

   Confirmed: no FX0517. The same omission occurs in required trait return types and associated-type definitions containing `NeedsPure<Clock>`. PURE-1 requires checking these obligations too.

   **Minimal patch:** resolve declaration types in their owning environment and feed their instantiated obligations through the existing owner, preserving the type-use span. Add a compact case covering these declaration routes.

5. **P1 — A nominal wrapper bypasses AFREE-2’s mutable-reference restriction.**  
   [walk.rs:786](/home/lucca/pjs/lab/tide-fx-kw8.2/crates/tide-admit/src/walk.rs:786)

   `Ty::walk()` traverses type arguments, not an ADT’s fields:

   ```rust
   pub struct Wrapped<'a>(&'a mut u32);
   pub const fn bump(w: Wrapped<'_>) { *w.0 += 1; }
   ```

   Confirmed: no FX0518, whereas the equivalent `Option<&mut u32>` parameter is rejected. The wrapper permits the same caller-memory mutation that the “no `&mut` anywhere” restriction excludes.

   **Minimal patch:** inspect instantiated nominal fields for mutable references with cycle protection. Clarify the AFREE-2 wording to include this route and test both wrapper forms.

6. **P2 — Bodyless function declarations bypass the function rules.**  
   [walk.rs:1208](/home/lucca/pjs/lab/tide-fx-kw8.2/crates/tide-admit/src/walk.rs:1208), [walk.rs:1226](/home/lucca/pjs/lab/tide-fx-kw8.2/crates/tide-admit/src/walk.rs:1226)

   Pinned rustc visits required trait methods through `visit_fn_decl`, which never reaches `fn_item`. Both examples omit their required diagnostic:

   ```rust
   pub trait A { unsafe fn run(); } // missing FX0901
   pub trait B {
       #[pure]
       fn run(c: &std::time::Clock); // missing FX0511
   }
   ```

   Unsafe foreign function declarations have the same header-check gap. This violates UNSAFE-1 and AFREE-1.

   **Minimal patch:** invoke the common function checks for bodyless declarations, without checking provided methods twice. Add required-method regressions.

7. **P2 — Expansion ancestry incorrectly applies `#[pure]` to nested helpers.**  
   [walk.rs:123](/home/lucca/pjs/lab/tide-fx-kw8.2/crates/tide-admit/src/walk.rs:123)

   ```rust
   #[pure]
   pub fn outer() {
       fn helper(_: &std::time::Clock) {}
   }
   ```

   Confirmed: FX0511 is emitted for `helper`, although only `outer` asserts authority-freedom. Rustc records the enclosing attribute expansion for descendant definitions too; `expn_that_defined` does not establish that this definition was the annotated target.

   **Minimal patch:** distinguish the attribute’s target from descendants sharing its expansion provenance. Test an unmarked nested helper and an independently marked nested helper.

8. **P2 — Callable adapters lose parallel checks and diagnostic ownership.**  
   [walk.rs:535](/home/lucca/pjs/lab/tide-fx-kw8.2/crates/tide-admit/src/walk.rs:535), [walk.rs:581](/home/lucca/pjs/lab/tide-fx-kw8.2/crates/tide-admit/src/walk.rs:581)

   Capture inspection and the supplemental `Fn` check handle only bare closure types:

   ```rust
   pub fn run(xs: &mut [u64]) {
       let mut n = 0;
       let mut f = |_: &mut u64| n += 1;
       xs.par_for(&mut f);
   }
   ```

   Confirmed: rustc emits E0277, but the required FX0610 is absent. Separately, passing `&job` or `Box::new(job)` where `job` captures `&Clock` produces FX0610 instead of PAR-7’s FX0611.

   **Minimal patch:** check the instantiated `Fn` obligation for adapted callables and preserve capture classification through supported reference/Box forwarding. Keep the body-use check needed for directly constrained closures. Add adapter regressions for both codes.

9. **P2 — Foreign function items and `#[pure] extern` functions receive inconsistent verdicts.**  
   [pure.rs:272](/home/lucca/pjs/lab/tide-fx-kw8.2/crates/tide-admit/src/pure.rs:272), [walk.rs:765](/home/lucca/pjs/lab/tide-fx-kw8.2/crates/tide-admit/src/walk.rs:765)

   A locally defined `extern "C" fn` passes a Pure bound as a function item; casting that same item to an `extern "C" fn()` pointer correctly fails. Also, `#[pure] pub extern "C" fn f() {}` produces no FX0511, despite 03 §3.2 explicitly excluding extern functions from authority-freedom.

   **Minimal patch:** apply the foreign-function classification consistently to function items and marked functions, using ABI as well as foreign-item identity. Add paired item/pointer and attribute cases.

10. **P2 — ROOT-4’s result bound is enforced only through the new facade wrapper.**  
    [std/src/lib.rs:73](/home/lucca/pjs/lab/tide-fx-kw8.2/conformance/rustc/std/src/lib.rs:73), [walk.rs:408](/home/lucca/pjs/lab/tide-fx-kw8.2/crates/tide-admit/src/walk.rs:408)

    With the unsafe grant, `std::mem::zeroed::<Fs>()` correctly emits FX0517, but these confirmed routes omit it:

    ```rust
    unsafe { core::mem::zeroed::<std::fs::Fs>() }
    unsafe { std::mem::transmute::<(), std::fs::Fs>(()) }
    ```

    `core::ptr::read::<Fs>` likewise bypasses the bound. ROOT-4 explicitly requires Pure results for zeroing, transmutation, and raw reads even under the grant; its separate allowance for arbitrary unsafe memory forgery does not remove those checks.

    **Minimal patch:** enforce that result-type obligation for the admitted resolved core operations as well as facade wrappers, through the existing Pure decision. Extend the forging case to cover alternate paths.

## Non-blocking

No additional findings confined to v1/Later text for `fx-djp`.

## Alternatives

Keep `pure.rs` as the single Pure decision owner and retain the rule-to-code table. Neither a new dependency nor a wholesale driver rewrite is needed.

The std type-argument shortcut can remain with audited transitive exceptions. Restoring field inspection would be a larger alternative requiring explicit handling of std’s owning raw pointers and erased payloads.

## Reasoning

The shared structural owner and code mapping are good foundations. Direct conformance cases exercise useful boundaries, but the confirmed failures show that declaration routes, adapters, expansion provenance, and changing recursion need stronger coverage.

I found no separate concrete defect in the hand-built `LateContext` or the std/derive unsafe exemption. Obligation deduplication preserves distinct types and rules, while repeated identical obligations report only their first site; I did not treat that repetition alone as a blocker.

Not checked: fresh builds, either full suite, or runtime behavior; the reported 172/26/0 result was not independently reproduced.  
Risk: the current candidate can admit authority into Pure-required code and crash on valid recursive types.