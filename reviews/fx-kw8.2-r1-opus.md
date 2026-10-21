Model: Opus 5.5. Date: 2026-10-21. Round 1, candidate 7f3b95a.

# fx-kw8.2 round 1 (Opus): the Pure check in tide-admit

## Verdict

**request_changes** (confidence: high on B1–B5, each reproduced with a probe).

The architecture is sound. `pure.rs` owns the structural decision, `walk.rs` finds the obligations, and the NEEDS table maps each failure to its D18 code. Every conformance case passes, and the E1 trial programs, `tally.rs` included, raise no Pure false rejections. Five defects block. Three are false accepts of programs the design rejects: Pure obligations from autoderef and `main`'s `Termination`, RPITIT default bodies, and `fmt`'s debug builders. One is a crash or hang on legal code (polymorphic recursion). One reports the wrong code (FX0610 where PAR-7 says FX0611).

## Scope

Read: AGENTS.md, GOVERNANCE.md, the bead (`br --no-db show fx-kw8.2`), and `git show 7f3b95a` in full: `pure.rs`, `walk.rs`, `crates.rs`, `diag.rs`, `main.rs`, `policy.rs`, the stub std, the two edited cases, README, and the design edits. Also read 02 §8.5 and §9 (PURE-1/2, the not-Pure table, std's types), 03 §2.1–2.2 (ROOT-4, GLOBAL-1..6), RUN-4/UNSAFE-1, §3.1–3.5 (AFREE-1/2), LOCK-1..5, PAR-4..7, §5 FFI-1 (v1), §9.5 nodes, reference §6.6, `conformance/run.sh`, and every Pure, unsafe, afree, global, lock, and par case.

Ran:
- `CARGO_BUILD_JOBS=4 cargo test --workspace`: exit 0, 2 + 4 passed.
- `conformance/run.sh`: exit 0, 172 PASS, 26 AWAITS CHECKER, 0 FAIL. This matches the author's results.
- The 11 E1 trial programs (`trials/e1-dialect/programs`) through the driver. Only FX0520 and rustc errors appear, and no Pure-family code does. `tally.rs` (par_map, Shared::new, with, with2) is admitted.
- About 20 probe files under `scratchpad/k2r1/` (a1–a9, b1–b9, c1), run with `probe.sh`, which invokes the built driver with the stub std as run.sh does. Most routes behave as the design says:
  - generics and forwarding, associated and projected types, Pure-bounded associated types at the impl;
  - PhantomData, fn pointers, arrays and tuples, nested `Rc<Cell<Shared>>`, opaque types with and without `Pure`;
  - recursion through Box (monomorphic), `dyn Any`, `Box<dyn Fn>`, casts and coercions to `dyn PureValue`;
  - bounds seen from closures in generic fns;
  - statics (`static mut`, dyn), the const fn forms (`Option<&mut>`, unbounded `impl Copy`, `&Cell`), and `#[pure]` on free fns, methods, and trait default methods;
  - unsafe in a local `macro_rules!`, in static and const initializers, in closures, and in trait default bodies, plus nested `#[unsafe(no_mangle)]`;
  - the join, par_map, par_for, Shared, and with codes;
  - 14 Fn-kind shapes with no false FX0610;
  - `format!` and `println!` inside `with` and par closures;
  - error recovery (E0425, E0308, E0631, tainted closures) with no ICE.

## Blocking

### B1. Autoderef and `main`'s `Termination` impl incur Pure obligations that are never decided (false accept)

`walk.rs:1314-1326` follows an overloaded autoderef's `Deref`/`DerefMut` impl only through `self.obligation(..)`, and `entry_point` (`walk.rs:927-944`) does the same. Neither calls `pure_obligation`, so a `T: Pure` where-clause on the selected impl goes unchecked. Every other `obligation` call site pairs it with `pure_obligation`.

Probe (`a5.rs`; all of it is accepted except the cast):
```rust
pub struct W<T>(pub T);
impl<T: Pure> Deref for W<T> { type Target = T; fn deref(&self) -> &T { &self.0 } }
pub fn auto(w: &W<Clock>) -> Instant { w.now() }      // accepted; should be FX0517
pub struct H { pub c: Clock }
pub fn field(w: &W<H>) -> &Clock { &w.c }              // accepted
fn take(c: &Clock) {}
pub fn coerce(w: &W<Clock>) { take(w) }                // accepted (deref coercion)
```
`a6.rs` does the same for `impl<T: Pure> Termination for W<T>` with `fn main() -> W<Clock>`, and it is also accepted. An explicit `*w` on a `W<Clock>` value is caught, because it is a type-dependent call.

Minimal fix: add `self.pure_obligation(tr, Need::Bound, span, 0)` beside both `obligation` calls. A better fix for AP-OWNERSHIP is one helper that follows an obligation for admission and for Pure together, called from every site, so the two lists cannot drift. Add a case: `pure_deref.rs` with `//~ FX0517` on the autoderef and field lines.

### B2. RPITIT with a default body: the hidden type is never checked against its `Pure` bound (false accept)

`opaque_bounds` (`walk.rs:680-691`) returns for every `RpitContext::Trait` opaque. Its own comment says such an opaque "has no hidden type unless its method has a default body", but the code also skips the default-body case. The `Pure` the bounds state is then trusted at every use.

Probe (`b8.rs`; accepted, exit 0):
```rust
pub trait Tr {
    fn m(&self, c: Clock) -> impl Fn() -> u64 + Pure + Send { move || { let _ = c.now(); 1 } }
}
pub struct X;
impl Tr for X {}
pub fn go(x: &X, c: Clock) -> (u64, u64) { join(x.m(c), || 2) } // a Clock-holding closure in join
```
Minimal fix: return early only when the trait method has no body, using `tcx.defaultness(fn_def).has_value()` on the opaque's parent fn. Otherwise check the hidden type as for any RPIT. Add a case.

### B3. `fmt`'s debug builders read as Pure, so a `with` closure can reach a `Formatter` (false accept). The design edit's agreement claim is false.

`DebugStruct`, `DebugTuple`, `DebugList`, `DebugSet`, and `DebugMap` (`core::fmt::builders`) have only lifetime arguments, so the type-argument rule (`pure.rs:316-326`) calls them Pure. Each one holds `&mut Formatter`. 02 §9's walk calls them not Pure, and so does the design's stated cost: "a `with` closure cannot capture a `Formatter`". `core::fmt::*` is admitted, so `f.debug_struct(..)` is reachable.

Probe (`a2.rs`): line 11 is accepted, and line 20, which captures the `Formatter` directly, gets FX0603 as it should.
```rust
impl fmt::Debug for S {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ds = f.debug_struct("S");
        let _ = self.cell.with(|v| { ds.field("v", v); });   // accepted; should be FX0603
        ds.finish()
    }
}
```
Minimal fix:
- Add `"core::fmt::builders::Debug*"` to `NOT_PURE` under "Erased payloads".
- Correct `design/02-memory.md:943-950`. It says the checker's list names the types whose erased payloads are not Pure, "That agrees with the walk", with one exception for markers. That is not true unless the list also names the std types that hold such a payload with no type argument. Name the debug builders there, or state that rule.
- Add a case (LOCK-4, FX0603) for the builder capture.

The author should re-audit the admitted std surface for other lifetime-only types that hold an erased payload, an `Arc`, or another handle. I found none besides these five and the ones already listed. `PanicMessage` is unreachable in v0.

### B4. Polymorphic recursion makes the driver hang or overflow its stack on legal code

`Pure::walk` (`pure.rs:257-340`) has no depth bound, and `assumed` only stops recursion on an exactly repeated type. A type whose recursive field changes its argument never repeats. rustc accepts these types under `--emit metadata`.

Probes:
- `a8.rs`: `pub enum Poly<T> { Nil, Cons(T, Box<Poly<(T, T)>>) }` with `needs_pure(p)` on a `&Poly<u8>` hangs. It was killed by a 30 s timeout, and the type doubles in size at each step.
- `b7.rs`: `pub enum Poly<T> { Nil, Cons(T, Box<Poly<Vec<T>>>) }` with `pub static P: Poly<u8> = Poly::Nil;` overflows the stack: "thread 'rustc' has overflowed its stack", exit 134.
- `b6.rs`: the same type with no Pure use terminates.

Minimal fix: thread a depth counter through `walk` and fail closed past a bound (`tcx.recursion_limit()` or a constant like `MAX_OBLIGATION_DEPTH`), with a `Why` such as "nests too deep to decide". Alternatively, count re-entries per ADT `DefId`. Add a case marking the expected verdict.

### B5. A closure passed by reference gets FX0610 where PAR-7 says FX0611

`parallel_closure` (`walk.rs:535`) inspects captures only when `ty` is itself `ty::Closure`. For `xs.par_map(&f)` the argument type is `&{closure}`, so the shared-capture check is skipped. The closure-level Pure check then reports FX0610. PAR-7 says a non-`Pure + Sync` shared capture is FX0611, "reported in place of PAR-5's and PAR-6's FX0610".

Probe (`a2.rs:26-29`):
```rust
pub fn by_ref(xs: &[u64], clock: &Clock) -> Vec<u64> {
    let f = |x: &u64| { let _ = clock.now(); *x };
    xs.par_map(&f)   // FX0610 "`&{closure}` is not `Pure`"; should be FX0611
}
```
Minimal fix: match on `ty.peel_refs()` for the capture loop. `reads_captures_only` can peel too. Add the line to `par_capture_reject.rs`.

## Non-blocking

1. **`pure_obligation` fails open at depth** (`walk.rs:479-485`). Past `MAX_OBLIGATION_DEPTH` it returns silently and relies on `obligation` rejecting the same chain with FX0520. The two walks differ in environment (opaques revealed or rigid), in dedup, and, as B1 shows, in their call sites. Fail closed with FX0517, or merge the two followers, which share select and where-clauses, behind one entry point (AP-OWNERSHIP).
2. **The stub std bounds only `mem::zeroed` by `Pure`.** ROOT-4 also names `transmute` and raw-pointer reads. Under the grant, `unsafe { std::mem::transmute::<(), Fs>(()) }` is still admitted. The same stub edit could cover `transmute`. A later stub bead could cover `ptr::read` and `MaybeUninit::assume_init`.
3. **`#[trusted_pure]` on a generic type makes every instantiation Pure** (`pure.rs:317`). For example, `#[trusted_pure] struct Tag<T> { p: *const u8, t: T }` makes `Tag<Clock>` Pure. Consider vouching only for the type's own fields and still requiring its type arguments to be Pure, as std types do, so the trust hole covers only what the reviewer approved. This is a design question for 02 §9 and FFI-3. Currently it is the user's false claim, under the grant.
4. **A `#[trusted_pure]` type emitted by a local `macro_rules!` is not trusted.** `std_attr_made` requires an attribute-only expansion chain, which gives a false rejection under the grant. Minor.
5. **Reporting once can hide errors.** `seen_pure` reports an obligation once per item and rule, and `Diagnostics::emit` keeps one diagnostic per (start, code). A second failing use at a different line, or a second failing argument of one call, is not shown. The crate is still rejected, and this is consistent with FX0520's policy, but a case that marks two lines would fail.
6. **Out of scope, pre-existing:** `unsafe extern "C" { fn ext(); }` gets no diagnostic. FFI-1, which allows extern blocks only under an `ffi` grant, is v1, and v0 has no `ffi` grant. File it on its own bead, or as an fx-djp comment if it is judged v1-only.
7. **Observed:** a crate with E0072 (`b2.rs`) gets no FX verdicts at all, not even for unrelated functions, although `main.rs`'s doc says a crate with rustc errors still gets its verdicts. I did not check whether this predates the change.
8. **`#[pure]` on a bodiless trait method is not checked**, because `visit_fn` sees only bodies. The design is silent on whether that form exists.
9. **Tests.** The change adds no driver tests. The new verdicts with no case are the marker rule (`PhantomData<Clock>` and `BuildHasherDefault<H>` are not Pure, the 02 §9 edit), Pure acceptance of polymorphic recursion, the Deref route, the debug builders, and the by-reference closure. B1–B5's fixes should each land with a case.
10. **Nits.**
    - `pure::implements` is a general trait query, not part of the Pure decision; it could live next to `select`.
    - `Pure::new` is a pass-through constructor.
    - `NOT_PURE`'s `core::sync::atomic::Atomic*` and `alloc::sync::*` are already denied by ADMIT-1. Harmless defense in depth.
    - Several `tide_std::` entries name items the stub lacks (`fs::File`, `net::*`, `process::*`, `log::LogConfig`, `accel::Accel`, `sync::atomic::*`, `io::Lines`, `mpsc::{Sender, Iter, IntoIter, TryIter}`), so no case exercises them.

## Reasoning

PURE-1 says the checker decides `Pure` "wherever a type meets a `Pure`-implying obligation". B1 and B2 are obligations the walk never reaches, and B3 is a std type the type-argument shortcut misreads. Each lets a capability or a `Formatter` sink into a closure that LOCK-4 or PAR-5/6 requires to be `Pure`. That is exactly the authority leak the rule exists to stop. B4 is a crash on code rustc admits, which run.sh classifies as a driver failure, and AP-MINIMAL requires boundary input to be handled. B5 is a code mismatch with PAR-7's explicit "in place of" rule. All five fixes are local: two calls, one condition, one NOT_PURE line plus the 02 sentence, one depth bound, and one `peel_refs`. Each needs a conformance case, so all five change this change's code, cases, or v0-core design text, and all five block. The non-blocking items are either documented policy (5), outside UNSAFE-1 or v1 (6, 8), design questions (3), or robustness and quality improvements (1, 2, 4, 7, 9, 10).
