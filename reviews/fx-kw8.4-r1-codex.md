Model: gpt-6.1-sol. Effort: xhigh. Date: 2026-10-31.

## Verdict

**request_changes — high confidence.** The backstop’s direction is sound, but source inspection identifies two false rejections and one gap in its independent `Pure` coverage. The reproducers below were not executed because the sandbox blocked compilation.

## Scope

Read AGENTS.md, GOVERNANCE.md’s supervisor brief, `br --no-db show fx-kw8.4` including its scope extension, the candidate diff, affected code and tests, policy and `Pure` helpers, relevant design text, and earlier review evidence.

Checked the pinned rustc’s collector, partitioning, instance sharing, symbol exports, shims, and clause inheritance.

Candidate: HEAD `5472a9336661e2cbe9e16f436f8d842d2d76b868` plus working tree.

- Start SHA-256: `25642709b784fbb8da710812b3098aff120be30e34b82213406f3da82af39db7`
- End SHA-256: `25642709b784fbb8da710812b3098aff120be30e34b82213406f3da82af39db7`

**Unchanged.** No repository files or Beads state were changed. `git diff HEAD --check` passed. `cargo build -p tide-admit` failed before compilation because `target/debug/.cargo-build-lock` was read-only.

## Blocking

1. **Generic parallel closures are checked using unsubstituted capture types.**

   Location: [mono.rs:26](/home/lucca/pjs/lab/tide/crates/tide-admit/src/walk/mono.rs:26), interacting with [walk.rs:659](/home/lucca/pjs/lab/tide/crates/tide-admit/src/walk.rs:659).

   Concrete trigger against Tide’s stub std:

   ```rust
   fn paired<T: Pure + Sync>(t: &T) -> (usize, ()) {
       std::par::join(|| std::mem::size_of_val(t), || ())
   }

   pub fn run() {
       let _ = paired(&1u32);
   }
   ```

   The HIR check can prove the capture is `Pure + Sync` using `T`’s bounds. The backstop checks the concrete `join` instance in `fully_monomorphized()`, but `parallel_closure` discards the closure’s arguments and reads `cap.place.ty()` from generic HIR. That leaves `T` in an environment without its bounds and produces FX0611 for valid code.

   **Smallest fix:** instantiate capture-place types with the concrete closure arguments before checking them. Preserve the generic arguments used by the HIR path. Add this positive regression case; it must pass with the normal walk and with `--tide-test-backstop`.

2. **Eager collection turns an unavailable conditional method into a failed use.**

   Location: [invocation.rs:158](/home/lucca/pjs/lab/tide/crates/tide-admit/src/invocation.rs:158), [mono.rs:51](/home/lucca/pjs/lab/tide/crates/tide-admit/src/walk/mono.rs:51).

   Concrete trigger:

   ```rust
   pub trait View {
       fn pure_only(&self) where Self: Pure {}
   }

   impl View for std::time::Clock {}
   ```

   This implementation need not establish `Clock: Pure`: that condition belongs to calls of `pure_only`. Nevertheless, rustc’s eager `create_mono_items_for_default_impls` creates `pure_only::<Clock>`. Its instantiability check accepts the stub’s blanket `Pure` impl. The backstop then rejects the instance with FX0517 despite there being no call.

   **Risk:** merely implementing a trait with an optional `Pure`-bounded method rejects legitimate authority-bearing types.

   **Smallest fix:** distinguish actual uses from eager-only instantiations with unsatisfied Tide preconditions. Such roots—and instances reachable only through them—must not become unconditional uses. Add this acceptance case alongside a call that must still fail.

3. **An overridden trait method loses the trait method’s own `Pure` clauses.**

   Location: [mono.rs:64](/home/lucca/pjs/lab/tide/crates/tide-admit/src/walk/mono.rs:64).

   Concrete trigger with `--tide-test-backstop`:

   ```rust
   pub trait View {
       fn check(&self) where Self: Pure;
   }

   impl View for std::time::Clock {
       fn check(&self) {}
   }

   pub fn run(c: &std::time::Clock) {
       <std::time::Clock as View>::check(c);
   }
   ```

   The collected instance names the impl method. Its clauses contain no `Pure` bound. The extra check visits `View`’s clauses, which also omit the method-level condition. Rustc’s clause inheritance does not copy that condition into the impl method’s clauses.

   **Risk:** the backstop alone accepts a runtime call requiring `Clock: Pure`. The normal HIR walk protects this example; this is an independent-coverage failure, not a demonstrated normal-mode bypass.

   **Smallest fix:** find the corresponding trait method, rebase the instance arguments onto its trait arguments, and check that method’s clauses. Their parent traversal also covers the trait-level clauses. Add a backstop-only regression expecting FX0517.

## Non-blocking

4. **Static-initializer edges are missing from diagnostic provenance.**

   [mono.rs:34](/home/lucca/pjs/lab/tide/crates/tide-admit/src/walk/mono.rs:34) builds reverse edges only for functions. With obligations disabled:

   ```rust
   fn need<T: Pure>() {}
   static F: fn() = need::<std::time::Clock>;
   ```

   Rustc collects the function through the static allocation, but the backstop cannot trace it to `F`; it falls back to `need`’s declaration. Rejection survives, but the promised nearest local reporting site does not. Include static allocation edges when constructing provenance. Allocation-derived `DUMMY_SP` edges also need the local-item fallback.

5. **The upstream-instantiation description is too absolute.**

   [design/03-authority-concurrency.md:576](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:576) says the flag makes the crate instantiate upstream generics itself. The pinned `Instance::upstream_monomorphization` explicitly retains upstream reuse for `#[inline(never)]`, even with sharing disabled.

   Qualify that statement. This does not invalidate coverage of the current denied impls inspected here.

All findings concern v0 implementation or edited ADMIT text. None requires an `fx-djp`-only disposition.

## Alternatives

Keep the HIR walk as the admission owner and retain the narrow mono backstop. A broader MIR allowlist would incorrectly apply user-facing restrictions to std internals.

For finding 2, lazy collection with an explicit metadata-build root policy is an alternative to eager collection. Either approach must preserve conditional `Pure` preconditions rather than treating every rustc-instantiable body as a valid Tide use.

## Reasoning

The pinned source supports the main collector choices: metadata-only builds have no ordinary exported roots; eager collection supplies roots; vtables, function-pointer allocations, drop glue, and ordinary shims feed runtime instances into collection. Mentioned-only items are separate from emitted runtime items, so omitting them is not itself a demonstrated runtime hole.

The principal weakness is reusing HIR-oriented checks under a different environment and interpreting rustc’s blanket-`Pure` instantiability as Tide callability. The new driver test exercises denied impls against Rust std, where `walk.pure` is absent. It therefore cannot detect these `Pure` failures. Approval needs the three focused regressions above and their fixes.

Not run: workspace tests, conformance, runtime probes, or performance measurements; the read-only sandbox blocked the build.
Risk: the reproducers are source-derived, so their exact diagnostics remain unverified.