# Round 2 Review: Cross-Chapter Consistency and Soundness

Reviewer persona: a programming-language researcher who has contributed to the
Rust borrow checker and trait solver.

Scope: I read all of `design/01-design.md` (draft 2.1 with its later
corrections), `design/02-memory.md`, `design/03-authority-concurrency.md`, and
`design/04-contracts.md`, and skimmed `reviews/round1-resolution.md`.
`design/reference.md` is being revised separately and is ignored here.
Round-1 objections that are now resolved are not raised again. Where an item
extends a round-1 objection that is still partial, the item says so.

Citations: `01 D5` is decision D5 in `01-design.md`; `02 §8.3` is a section of
`02-memory.md`; `03 TASK-3` is a numbered rule in
`03-authority-concurrency.md`; and so on.

## Verdict

**request_changes**

The chapters turn draft 2's outline into rules that can be checked. Most of
round 1's soundness objections now have concrete answers (see the last
section). What remains is a small set of places where a guarantee in the
decision record is contradicted by a chapter rule, or where a rule fails on a
short program:

- **Determinism of authority-free code fails in two independent ways.** This
  determinism is the basis of the thesis's `par` and simulation claims.
  Authority-free code can create synchronization handles (B1). The standard
  library runs user code (destructors, `Clone`, lazy initializers) at points
  that depend on timing (B2).
- **The deferred-trap rule is stated twice, differently,** and both versions
  are observable (B3).
- **Four holes in proof soundness:** check deletion (B4), stored `&mut`
  arguments and `final` (B5), half-updated values after a child failure (B6),
  and termination through trait objects (B7).
- **A memory-safety gap in task isolation** (B8).
- **A completeness bug in the origin calculus** that contradicts D1 and
  02 §2.6 (B9).

Each item has a fix of one or two sentences. None needs a new direction.

## Blocking

### B1. Authority-free code can create synchronization handles and observe other tasks

**Where.**

- 01 D7: "Synchronization handles … are created without a capability".
- 01 D7: "`sync::Weak` is a synchronization handle, so shared ownership cannot
  leak timing into authority-free code".
- 01 D8: "A function can suspend only by using a capability; an
  authority-free function never suspends".
- 03 §0: "Only capability and synchronization operations suspend".
- 03 §3.2–§3.3, the definition of authority-free code and its argument.
- 03 §6.4 (`recv` is a checkpoint) and 03 TASK-3.
- 03 §8.1: "A `par` branch … has no checkpoints".
- 03 SUSP-1 and §7.2.

**Problem.** Authority-freedom is defined by parameter and capture types only.
Handles are not `Pure`, so an authority-free function cannot *receive* one.
Nothing stops it from *creating* one, and 03 §3.3's argument considers only
what arguments reach. Two operations on locally created handles observe other
tasks:

```ferox
// Authority-free: the only parameter, Arc<Config>, is Pure.
fn released_last(a: Arc<Config>) -> bool {
    let w = Arc::downgrade(&a);      // a sync::Weak, created without a capability
    drop(a);
    w.upgrade().is_none()            // true iff no other task or branch still holds a clone
}

// Authority-free: no parameters at all.
fn cancelled() -> bool {
    let (tx, rx) = channel::<()>();
    tx.send(()).unwrap();            // unbounded send: never waits
    rx.recv().is_err()               // recv is a checkpoint (03 §6.4): Err(Cancelled)
}                                    // once the enclosing scope is cancelled (TASK-3)
```

- **`par` stops being deterministic.**
  `par { released_last(a1), released_last(a2) }`, with `a1` and `a2` clones of
  one `Arc`, passes FX0610: both branches are authority-free and capture only
  `Send` values. Sequential evaluation gives `(false, true)`. On real cores it
  can give `(true, false)` or `(true, true)`. This breaks 01 D9 and 03 PAR-1.
- **Replay breaks.** 01 D15 runs such a region on real cores under the
  simulator, so its result is not reproducible from the seed.
- **Authority-free code observes cancellation.** `cancelled()` inside a child
  task reports whether a sibling has failed yet. Inside a `par` branch, it is a
  checkpoint in a construct that 03 §8.1 says has none.
- **The evented lowering is inconsistent.** SUSP-1 compiles `cancelled` as
  non-suspending, on the native stack (03 §7.2). Yet `recv` on an empty queue
  would have to suspend there. Likewise, SUSP-1 says a call through a
  non-`Pure` `dyn` value "may suspend", but a native-stack caller has nowhere
  to return `Pending` to.

**Fix.** Creating a synchronization handle takes `&Tasks`: `tasks.channel()`,
`tasks.sync_channel(n)`, `tasks.shared(v)`, `Arc::downgrade(&a, &tasks)`.
Atomics may stay free, because a local atomic cannot reach another task
without a spawn. After this change:

- no value reachable from authority-free code is or contains a handle;
- D8's sentence becomes true;
- 03 §7.2 can state that a non-suspending caller drives a may-suspend `dyn`
  call to completion, because that call can never return `Pending`.

A weaker alternative: remove `sync::Weak`, and specify that handle operations
in a non-suspending instance ignore cancellation and panic on any wait. Such a
wait is always a self-deadlock.

### B2. The standard library runs user code at timing-dependent points

**Where.**

- 01 D7: `Arc` exposes no count readers, "so shared ownership cannot leak
  timing into authority-free code".
- 02 §9 and 03 GLOBAL-6: "`make_mut` stays: its observable result does not
  depend on timing".
- 03 GLOBAL-2 (lazy statics).
- 03 §3.3 item 1: an authority-free function's result and panic are a function
  of its arguments' memory.
- 03 PAR-1, PAR-2, SINK-4, and §8.4.
- 01 D15 and 03 §9.7.

**Problem.** 03 §3.3 argues that every type that is `Sync` and mutable through
`&` is a synchronization handle. `Arc` (its count) and `Lazy` (its
initialization) are both `Sync`, both mutable through `&`, and both `Pure`.
Hiding their state is not enough. The standard library runs user code when
that state changes, and which task or branch runs it depends on timing.

1. **`make_mut` runs the user's `Clone` only if another clone exists.**

   ```ferox
   impl Clone for Cache {
       fn clone(&self) -> Cache { Cache { hits: 0, map: self.map.clone() } }   // clones start cold
   }
   fn touch(c: &mut Arc<Cache>) -> u64 {        // authority-free
       let c = Arc::make_mut(c);
       c.hits += 1;
       c.hits                                   // old + 1, or 1 if another task holds a clone
   }
   ```

   The result depends on other tasks. 04 §5.1's `Clone` contract
   (`result@ == self@`) forbids this `Clone`, but that contract is checked
   only under `test` (sampled) and `prove`. D7's guarantee is unconditional.

2. **The last `Arc` release runs `T::drop` in whichever branch or task
   releases last.**

   ```ferox
   struct Noisy(u32);
   impl Drop for Noisy { fn drop(&mut self) { info!("freed {}", self.0); } }

   let a = Arc::new(Noisy(1));
   let b = a.clone();
   par { { drop(a); info!("left"); }, { drop(b); info!("right"); } };
   ```

   Sequential evaluation prints `left`, `freed 1`, `right`. If the left branch
   releases last on real cores, `freed 1` lands in its buffer and prints first.
   PAR-1 says sink output matches sequential evaluation. If `Noisy::drop`
   panics, PAR-2 may report a different panic. Outside `par`, an
   authority-free `fn f(a: Arc<Noisy>)` panics or not depending on other
   tasks, which 03 §3.3 item 1 rules out.

3. **A lazy static is initialized by whichever branch forces it first.**
   `build`'s sink records land in that branch's buffer. A panicking `build`
   gives one branch its own message and the others a "poisoned" message, so
   the payload that PAR-2 reports as "first in sequential order" depends on
   timing. Because `par` runs on real cores under the simulator (D15), traces
   differ between runs, and 03 §9.7's self-check reports a false divergence.

**Fix.**

- Remove `Arc::make_mut` by adding it to GLOBAL-6's list. 03 §13.1 Q4 already
  tracks the porting cost.
- Inside a `par` branch, record an `Arc` release instead of performing it. At
  the join, apply the recorded releases in sequential order: branch order,
  then program order. Destructors then run in the sequential program's order,
  at the join. Define PAR-1's reference semantics the same way.
- Outside `par`, a panic in a destructor run by an `Arc` release aborts, as a
  panic during unwinding does.
- Require a lazy `build` to have no exercised effects (03 §4.1 already
  computes them), and make poison re-raise the original panic payload and span
  at every access.

### B3. The deferred-trap rule is stated twice, differently, and both versions are observable

**Where.**

- 01 D3: a trap is reported "before the overflowed value influences any
  observable effect or leaves the enclosing loop or function".
- 01 Open Question 1, which is still open.
- 04 §3.2 rule 2: the compiler "may defer only across code whose control flow
  and addresses do not depend on the overflowed value", and deferral is
  unobservable except in abandoned memory, "which D5 makes unobservable".
- 01 D5 and 04 §4.3: abandoned values are dropped.
- 03 §9.1: one trace across three engines.
- `TaskError::Panicked(String)` in 03 §6.1.

**Problem.** The two texts disagree:

- 04 allows the overflowed value to be stored anywhere, sent, printed, or
  written to an atomic, as long as no branch or address depends on it.
- D3 forbids observable effects but allows deferral across loop iterations.

Neither bounds the deferral window or says which panic is reported. Three
differences from precise trapping are observable:

```ferox
struct Out { buf: Vec<i32>, tx: Sender<Vec<i32>> }
impl Drop for Out {
    fn drop(&mut self) { let _ = self.tx.send(mem::take(&mut self.buf)); }
}

fn triple(out: &mut Out, xs: &[i32]) {
    for (o, x) in out.buf.iter_mut().zip(xs) { *o = x * 3; }   // checks batched to loop exit
}
```

- **Destructors publish abandoned memory.**
  - Precise trapping fires at the first overflowing element, and later
    elements keep their old values.
  - Batching, which both texts allow, overwrites every element with a wrapped
    product and traps at loop exit. Unwinding then drops `Out` (D5), and its
    destructor sends the wrapped buffer to another task.

  04's claim that D5 makes abandoned memory unobservable is false, because D5
  drops abandoned values instead of leaking them.
- **Panic identity changes.** Take
  `for i in 0..n { s += a[i] * k; t = b[i]; }`, where `b` is shorter than `a`.
  - Precise trapping reports overflow at iteration 3.
  - Batching reports the bounds failure at iteration 5.

  The message reaches the parent as `TaskError::Panicked(String)`. The
  interpreter traps precisely and compiled engines batch, so their traces
  differ, which contradicts 03 §9.1.
- **A panic becomes a hang.** In `loop { n += 1; step(); }` with no exit test
  that reads `n`, precise trapping panics. Deferring "until it leaves the
  loop" never reports, and the task hangs.

**Fix.** Replace both texts with one list. A pending trap fires at the first
of these flush points:

- a store to memory (anything other than a register-allocated local);
- a call;
- any other panic site;
- a sink, handle, or capability operation;
- a function return;
- a loop back-edge, except in loops whose trip count is fixed at entry.

The reported trap is the earliest overflowing operation in program order. This
keeps the two optimizations D3 wants: checking a vector's lanes before its
store, and keeping reduction accumulators in registers across a counted loop.
It also makes every engine report the same panic. Close Open Question 1 in 01.

### B4. Check deletion in 04 is weaker than D3

**Where.**

- 01 D3: a proof may delete a check "only if every fact it relies on is either
  proved inside the same build or still checked at run time".
- 04 §2.6 condition 2: "For a private function, every caller in the package is
  proved. For a `pub` function, each such `requires` stays checked at entry".
- 04 §3.1: a caller's verdict assumes the contracts of callees not proved in
  this build.
- 04 §5.3: reliance on laws is settled per concrete instantiation.
- 01 D12: one instantiation cache across builds, plus a witness-table fallback.
- 04 §2.3: invariant checks exist only in debug and test builds.

**Problem.** Condition 2 counts a caller as "proved" even when its proof
assumes unproved facts. It also equates "private" with "reachable only from
proved code in this package". Four cases delete a check that rests on a fact
neither proved in the build nor checked at run time:

```ferox
// package p, #[contracts(prove)], release_delete_bounds_checks = true
fn idx(xs: &[u8; 16], i: usize) -> u8 requires i < 16 { xs[i] }   // private: bounds check deleted
pub fn get(xs: &[u8; 16]) -> u8 { idx(xs, dep::small()) }        // "proved, assuming 1":
                                                                 // dep::small() ensures result < 16 (assert-mode dependency)
```

1. **A caller proved under assumptions.** `get` counts as proved, so `idx`'s
   bounds check is deleted. If `dep::small()` returns 20, safe code reads out
   of bounds. Without the bounds flag, the same pattern silently wraps an
   integer that D3 says traps in every profile.
2. **A private function that is reachable from outside.** Two examples:
   - a private function escapes as a value
     (`pub fn handler() -> fn(u32) -> u32 { inc }`);
   - a method in a private type's impl of a public trait is called through a
     `Box<dyn Trait>`.

   Either way, other packages call it with no entry check. Whether a
   trait-impl method counts as `pub` is not stated, and implementers will
   disagree.
3. **Laws at an instantiation.** A check deleted in a generic proof relies on
   the laws of the instantiated impls. D12's cross-build instantiation cache is
   not keyed on those laws' proof status. Witness-table code is shared by
   instances whose laws differ.
4. **Type invariants.** A function proved inside an `assert` module (one item
   marked `#[contracts(prove)]`) can delete a check using a type invariant.
   An unproved sibling function in the same module can break that invariant,
   and release builds check no invariants. A `pub` entry check covers
   `requires`, not assumed invariants.

**Fix.**

- Rewrite condition 2 as follows. Every path into the function is a direct
  call in this package, and each call establishes every `requires` the
  deletion uses, proved with an empty trust closure. The function is not
  `pub`, not a trait-impl method, and never named as a value or coerced to
  `dyn`. Otherwise each `requires` the deletion uses stays checked at entry.
- Count a type invariant as a fact the deletion relies on, unless every
  function of the invariant's defining module is proved.
- Decide deletion per instantiation. Put the proof status of instantiated laws
  in the instantiation-cache key, and keep law-dependent checks in
  witness-table code.

### B5. `final(*x)` is executable even when the callee stores the `&mut` argument

**Where.**

- 04 §4.1: "If the result does not borrow from `x`, `final(*x)` is the value
  `*x` holds when the function returns"; "ordinary `&mut` arguments need
  none".
- 01 D10: `final(x)` is "the value a returned or stored `&mut` holds when its
  borrow ends".
- 02 §2.4 item 2 (`from` in a parameter type).

**Problem.** Under 02, a `&mut` parameter's origin can be identified with a
place under another `&mut` parameter, which lets the callee store the borrow.
The borrow then outlives the call, so its value at return says nothing about
its value when the borrow ends:

```ferox
pub struct Slot { r: Option<&mut i32> }

#[contracts(prove)]
fn stash(s: &mut Slot, x: &mut i32 from s.r)
    ensures final(*x) == old(*x)          // classified executable: proved trivially at return
{
    s.r = Some(x);
}

#[contracts(prove)]
fn caller() -> i32
    ensures result == 0                   // proved: after the call, v == final(*x) == 0
{
    let mut v = 0;
    let mut s = Slot { r: None };
    stash(&mut s, &mut v);
    if let Some(r) = s.r { *r = 5; }      // writes v through the stored borrow
    v                                     // returns 5
}
```

The Rust equivalent is `fn stash<'a>(s: &mut Slot<'a>, x: &'a mut i32)`, and
borrowck accepts both functions. This is a soundness hole in `prove`, not a
missing feature. RustHorn's encoding is sound only if `final(*x)` is a
prophecy here.

**Fix.** In 04 §4.1, `final(*x)` is executable only if no origin of `x`'s
outer reference occurs in the result type. It must also not occur, through the
signature's `from` identifications, in a position under another `&mut`
parameter. Otherwise `final(*x)` is prophetic (`proof-only`), and the callee
must relate it to the final value of the place where it stored `x`. 01 D10
already says "stored"; 04 must say how storing is detected.

### B6. D5 and 02 §8.3 disagree on whether a half-updated value can reach a caller

**Where.**

- 01 D5: "A place behind a reference cannot be left moved out, so the function
  must reassign it or itself panic; a half-updated value can never reach a
  caller".
- 02 §8.3: such places "may be assigned, and the function may return"; "the
  second case lets a half-updated value reach the caller".
- 02 §11.2 R6, and 02 §12 item 8, which calls the question open.
- 04 §4.2: the verifier assumes `inv` of "the value a `&mut` argument holds
  after the call".
- 04 §4.3, whose argument relies on D5.

**Problem.** 02 calls itself "the definition the checker implements", yet it
implements a weaker rule than the decision it elaborates. Under 02, this code
returns a broken `Span` to its caller:

```ferox
// Span has `invariant self.lo <= self.hi`; Span::set writes lo, then hi (assert-mode module).
pub fn widen(tasks: &Tasks, s: &mut Span) -> Result<(), TaskError> {
    tasks.scope(|sc| { sc.spawn(|| { s.set(10, 20); Ok(()) }); Ok(()) })
}   // the child panics inside `set` after writing lo; 02 lets `widen` return without assigning *s
```

04 §4.2's assumption on the caller's side, and 04 §4.3's argument, are sound
only under D5. A verifier that follows 02 literally reasons "the place cannot
be read, so nothing is owed for it" and proves `s.lo <= s.hi` after `widen`
returns.

**Fix.** 02 §8.3 adopts D5. On every path from the `Err` edge to a return,
each place in `M` behind a reference must be definitely assigned. This is a
forward must-assign check, like the existing initialization analysis. If the
place is not assigned, the function must not return normally. Delete 02 §12
item 8 and the "partially" in R6.

### B7. Termination checking under `prove` misses recursion through trait objects and instantiations

**Where.**

- 01 D10: `prove` means total correctness; loops and recursion need
  `decreases` or `diverges`.
- 01 D11: `diverges` needs approval.
- 04 §3.3: "Mutually recursive functions in one module …", and "A call
  through a function value adds none".
- 04 §3.4 item 2: impl edges and trait-call summaries, stated for ghost code
  only.

**Problem.** Recursion in executable code is detected only within one module
and through named function values. A cycle through `dyn` dispatch, or through
a generic function instantiated in another package, is invisible:

```ferox
// package a, #[contracts(prove)]
pub trait Step { fn step(&self); }
pub fn drive(t: &dyn Step) { t.step() }          // no loop and no visible recursion: proved

// package b, #[contracts(prove)]
struct S;
impl Step for S { fn step(&self) { a::drive(self) } }
pub fn run() { S.step() }                        // reported `proved`, with no `diverges`
```

`run` recurses forever. Both calls are in tail position, so the release
backend's C compiler can turn them into jumps, and the loop then runs without
growing the stack. "Stack overflow is outside the guarantee" (04 §3)
therefore does not cover this case. An agent can use the pattern to write an
unbounded loop without the `diverges` approval D11 requires.

**Fix.** Under `prove`, add call edges in two places:

- a coercion to `dyn Trait` adds edges from the coercion site to the impl's
  methods, mirroring 03 §4.1 rule 4 for authority;
- a concrete instantiation adds edges to the instantiated impls' methods,
  through trait-call summaries exported for executable generics as they are
  for ghost ones (04 §3.4 item 2).

A cycle through these edges needs a measure or `diverges`.

### B8. `dyn` hides `ThreadBound`, so pinning misses it

**Where.**

- 01 D8: "A task holding a `ThreadBound` value is pinned to its thread while
  it holds it".
- 02 §8.1 condition 2: pinning is computed "from liveness".
- 03 §5.4.
- 02 §5.1: "A `dyn` type has no auto traits unless it names them".
- 02 §11.2 R1.

**Problem.** Pinning is computed from the liveness of values whose type is
`ThreadBound`. The chapters do not say that `ThreadBound` is structural, and a
`dyn` type hides it:

```ferox
// `gl` is a safe wrapper package; GlContext is #[thread_bound].
fn render(net: &Net, ctx: GlContext, url: &Url, t: Instant) -> Result<(), NetError> {
    let held: Box<dyn Debug> = Box::new(ctx);   // no ThreadBound-typed value is live from here on
    let frame = net.get(url, t)?;               // suspends; the evented scheduler migrates the task
    drop(held);                                 // GlContext's foreign destructor runs on another OS thread
    Ok(())
}
```

Rust rules this out because the handle is `!Send`, and a `!Send` value held
across an `.await` makes the future `!Send`. Ferox deliberately drops that
requirement (02 §8.1), which leaves pinning as the only defence. Pinning must
therefore see through erasure.

**Fix.** Make `ThreadBound` a compiler-known structural property: a type is
`ThreadBound` if any component is. Forbid coercing a `ThreadBound` type to
`dyn` unless the object type names `+ ThreadBound`; such values pin like the
concrete type. Add this counterexample to the proof that R1 requires.

### B9. An output `from` cannot name an origin at an invariant position, which contradicts D1 and 02 §2.6

**Where.**

- 02 §2.4 item 1: an output position gets a fresh `ρ` with `orig(P) ⊆ ρ`.
- 02 §11.1: `-> T from P̄` becomes a fresh `'r` with `'p: 'r`, while `Self.f`
  in impls becomes `'f`.
- 02 §2.6: "The result is at least as general as the Rust signature".
- 01 D1 (named lifetimes that map onto origins are accepted) and 01 D2.

**Problem.** Inclusion is the right reading at covariant positions. Under
`&mut`, however, no body can satisfy an inclusion, because the caller may
choose a `ρ` larger than the field's origin. This common Rust accessor
therefore has no Ferox spelling:

```rust
impl<'a> Parser<'a> {
    fn toks_mut(&mut self) -> &mut Vec<&'a str> { &mut self.toks }
}
```

```ferox
pub struct Parser { lx: Lexer, toks: Vec<&str from lx.src> }
impl Parser {
    pub fn toks_mut(&mut self) -> &mut Vec<&str from Self.lx.src> { &mut self.toks }
}   // rejected: the element position is invariant, and the body cannot show ρ == lx.src
```

- **No other spelling works.** Elision ties the inner origin to the
  `&mut self` borrow, which 02 §10.5 shows is rejected. Every output `from`
  is an inclusion.
- **02 §2.6 rewrites the Rust signature into exactly this rejected form**
  while calling the rewrite "at least as general". Here generality runs the
  wrong way: a more general signature makes the body harder to accept.
- **Two checkers will differ.** The translation table maps `-> T from P̄` to a
  fresh `'r` but maps `Self.f` to `'f`, so it is unclear which applies to
  `-> … from Self.f`. 02's own worked examples translate `from self.src` as
  `'a` (§10.3, §10.4), not as a fresh `'r`.
- **The same gap blocks `fn lexer_mut(&mut self) -> &mut Lexer`**, which Rust
  writes as `-> &mut Lexer<'a>`.

**Fix.** When `P` denotes exactly one origin, an output `from P` identifies
the position with `P`'s origin, as parameter types do (02 §2.4 item 2). At
covariant positions this is equivalent to inclusion; at invariant positions it
is the only reading a body can satisfy. A union (`from a, b`) at an invariant
position is an error with a diagnostic. The §2.6 rewrite and the translation
τ then reuse `'p`. This is a completeness bug, not a soundness bug, but it
breaks a stated property of the normative calculus, and E1 tasks will hit it.

## Non-blocking

### Contradictions to reconcile

- **N1. `lock` aliasing.** 01 D8 and 03 LOCK-2 and LOCK-5 say a duplicate
  operand gives `Err(Aliased)` and the result is `Result<R, LockError>`.
  02 §8.4 still says a duplicate panics and the result is
  `Result<R, Poisoned>`, and 02 §11.1's `lock` row describes the panic.
  03 §13.2 records this; 02 has not been updated.
- **N2. Statics.** 02 §3.7 and the 02 §9 table require `Owned`, `Sync`, and
  `Frozen`; 03 GLOBAL-1 also requires `Pure`. 03 is right: a foreign function
  pointer is `Owned + Sync + Frozen` but not `Pure`.
- **N3. Field-borrow methods and invariants.** 02 §7 forbids a field-borrow
  method from mutating any field that an invariant mentions. 04 §4.2 lets it
  assume and owe each clause whose footprint lies inside the declared set.
  04's rule is sound and more useful; 02 should defer to it.
- **N4. What specs may read.** 03 §3.5 says specs "read only owned and Frozen
  state (`04-contracts.md`)". 04 §1.6 instead requires `Pure` parameters and
  relies on the opaque views of `Cell` and `RefCell`. Choose one statement.
- **N5. `par_reduce` and the fold.** 01 D9 says a proved associativity law
  makes `par_reduce` equal the sequential fold. 03 §8.3 notes that a checked
  integer `Sum` can trap under G and not under the fold. Two consequences for
  04 §5.3:
  - its lemma may equate the two values only when neither side traps;
  - `par_reduce`'s panic obligation must range over G's partial results, not
    the fold's prefixes.

  Otherwise a proof that the fold of `[MAX, -MAX, MAX, -MAX, …]` cannot
  overflow transfers to `par_reduce`, whose strided accumulator 0 adds
  `MAX + MAX`.
- **N6. Scope API.** 02 §8.2 has `Result<R, ScopeError>` and `Handle<T>`, and
  its children return `T`. 03 §6.1 has `Result<T, E>` and `Task<U, E>`, and
  its children return `Result<U, E>`. The difference is in failure semantics,
  not only in names. 02 should adopt 03's signatures; its origin argument is
  unaffected.
- **N7. Unwinding on cancellation.** 01 D8 says "suspended work unwinds and
  runs destructors". Under 03 TASK-3, cancellation is an `Err` value at
  checkpoints, and nothing unwinds.
- **N8. What long-lived tasks may capture.** 01 D2 says "tasks spawned into a
  long-lived scope capture `Owned` values". 02 §8.2 lets them borrow whatever
  the signature ties to the scope (`cfg: &Config from bg`).
- **N9. Stale bookkeeping.**
  - 04 §3.2 answers 01 Open Question 1, and 03 §3.2 answers Open Question 4.
  - 04's "Amendments needed" 1, 3, 4, and 5 are already applied but still
    listed.
  - D5 decides 04 §13 item 5 (leak versus drop).
  - 03 §13.2's notes on D9's heartbeat and D7's `Accel` are already done.
- **N10. Gaps in grammar and tables.**
  - 02 §2.3's grammar has `Root ::= a parameter name | self`, but §3.3 uses
    sibling-field roots (`from src`, `from *out`).
  - §2.4 allows `from` in body annotations, which §2.3's list of positions
    omits.
  - 02 §8.1's `Send`/`Sync` table lacks `Task`/`Handle`, `SyncSender`, and
    atomics.

### Underspecified rules

- **N11. The sink configuration is an ambient input.**
  - **Lazy formatting.** 03 SINK-3 evaluates format arguments only when the
    record's level is enabled. `main` sets the level (SINK-2), and it differs
    by profile (SINK-8). A `Display` impl that fills a `OnceCell` cache (a
    `Pure` type) therefore makes an authority-free function's later result
    depend on the log level. If `main` may change levels after spawning, the
    result also depends on timing.
  - **Caught panics.** SINK-3 catches formatting panics. This is a catch point
    that neither D5's moved-out rule nor 04 §3.2 accounts for.

  Fix: state that the sink configuration is fixed before the first spawn and
  recorded in the sim trace. Either evaluate format arguments whenever the
  form is compiled in, or require them to be `Frozen`. 04 should also say how
  sink forms are verified: formatting calls carry termination obligations,
  and their panics are caught.
- **N12. Spec evaluation runs user code.** 04 §1.6 marks std's `contains`,
  `get`, and similar accessors `#[spec]`, and under `assert` and `test` "the
  evaluator runs the function". `requires xs.contains(&k)` therefore calls the
  user's `PartialEq::eq`, which may write a `Cell`, panic, or diverge. This
  contradicts 04 §1.3: "spec evaluation never panics the program and never
  changes program state". Fix: evaluate std accessors at the view level
  (`xs@.contains(k@)`), which is also their spec meaning.
- **N13. The sign of NaN.** 01 D3 promises observable float results that
  match across targets, via `to_bits()` canonicalization. But `copysign`,
  `is_sign_negative`, `total_cmp`, `to_le_bytes`, `Hash`, and `Encode` all
  read a NaN's sign or payload. For `0.0 / 0.0` computed at run time, the
  sign differs between x86 and Arm, so `1.0f64.copysign(0.0 / 0.0)` returns
  `-1.0` on one and `1.0` on the other. This extends round-1 S-B9. Route every
  operation that observes bits through the canonical bits.
- **N14. Float `==` in specs.** Spec `==` on floats is bitwise (04 §1.3). As a
  result, `ensures result == 0.0` fails for `-0.0` while executable `==`
  holds. This goes against 04's own reason for choosing Rust's `/`: the same
  text should mean the same thing in code and specs. Use IEEE `==` and give
  bitwise equality its own name, or report the difference next to each clause
  that compares floats.
- **N15. Destructors and exercised authority.** 03 §4.1 counts calls,
  closures, and `dyn` coercions, but not destructors. A `Drop` impl that uses
  a held `Net` exercises `net` at every drop site, including generic drop
  glue. Treat drop glue like a call: it contributes the destructor's summary
  for exercised authority, may-suspend classification, and termination.
- **N16. `par` inside a `lock` body.** LOCK-4 allows authority-free calls, so
  a lock body may run `par`. 03 §8.4 shares the `par` pool with the evented
  workers. If a `par` join helps by running evented tasks on its worker, a
  helped task that blocks on the held cell deadlocks that worker. State that a
  `par` join helps only with `par` branches.
- **N17. `Shared::reset`.** It must acquire the cell in the global order and
  be a scheduling point. 03 §1.2 and LOCK-6 say neither.
- **N18. Stack overflow.** 03 §2.3 says stack overflow "abandons the task".
  Abandoning the task means unwinding and joining children that borrow the
  frame. A guard-page handler that tears down the stack would leave those
  borrows dangling. Specify prologue checks that panic, or specify an abort.
- **N19. The replay record.**
  - 03 §9.6's record and replay command omit the engine. The default engine
    differs between `ferox test` and long `ferox sim` runs, and engines differ
    in stack depth (and, until B3 is fixed, in trap reporting).
  - D15 should carry PAR-2's divergence carve-out, because `par` runs on real
    cores under the simulator.
  - Under simulation, a Bend call without a `reference` runs "the real kernel
    outside virtual time" (03 §10), and its float results can differ between
    CPU and GPU. Pin the CPU backend, or require the reference, under
    simulation.
- **N20. Coherence and auto traits.** Ferox has no overlapping impls and no
  negative bounds, so state that overlap checking never relies on the absence
  of `Owned`, `Pure`, `Frozen`, `Send`, or `Sync`. For example,
  `impl<T: Owned> K for T` and `impl K for &str` overlap at `&'static str`.
  `Owned` depends on origins, which Rust's coherence never inspects.
- **N21. R3 covers only rust-lang/rust#25860.** The rustc oracle also inherits
  rustc's other implied-bound holes, such as those through impl headers and
  supertrait bounds (for example #100051 and #84591). §5.4's re-check should
  say that implied bounds from an impl header are available in a method body
  only where the trait signature also implies them. The negative corpus
  should include these cases.
- **N22. Facts kept across loops.** 04 §3.3 says "A variable that a loop never
  assigns or mutably borrows keeps its facts". It should say that no
  overlapping place is assigned or mutably borrowed, so that borrowing `s`
  invalidates facts about `s.x`.
- **N23. Lock-entry invariants.** 04 §4.4 assumes `inv(*x)` when a lock body
  starts. That assumption holds only if every lock body on the cell, in every
  package, preserves the invariant. If the payload type's module has
  unproved functions, the assumption belongs in the trust closure (see also
  B4, case 4).

## What improved since round 1

- **The origin calculus is written down** (02 §1–§7), with variance, elision,
  origins per position, drop check in the right direction, and a translation
  to Rust that again serves as a CI oracle. This addresses round-1 S-B1,
  S-B2, S-B3, and S-B4's note, and new problems 3, 5, 6, and 13. Closure
  escape is now decided by origins, not by `move`.
- **Check deletion is now conditioned on proof status.** `pub` preconditions
  stay checked at entry, and bounds-check deletion is off by default (new
  problem 1). B4 concerns the remaining wording, not the design.
- **`Pure` is stricter.** It excludes `dyn` without `+ Pure`, raw pointers,
  and `sync::Weak`, and memoization forbids `&mut` parameters (S-B6, C-B1).
- **`lock` is tightened.** Payloads are `Pure`, acquisition is defined as not
  a checkpoint, and aliasing returns `Err` (S-B7).
- **`par_reduce` and `par` panics are defined.** `par_reduce` is defined by
  G, and `Combine` versus `Monoid` separates float sums from the
  associativity law (new problem 2, C-B6). PAR-2 says which panic wins.
- **Contracts are defined in detail** (V-B1 to V-B9, A-B5, new problems 10
  and 11):
  - spec semantics: spec termination, spec arithmetic and floats,
    well-formedness obligations, and rejection of a bare `*x`;
  - specification forms: `final`, `#[spec]`, trait contracts with refinement,
    laws settled per instantiation, type invariants with footprints, and lock
    invariants;
  - spec quality: a vacuity floor and contradictory-assumption checks;
  - approval: root specs, protected tests, `diverges` and use downgrades
    counted as holes, and proof caches rebuilt only in CI.
- **Authority is tightened** (C-B2, St-B2, new problems 8 and 9):
  - `Root` cannot be passed;
  - declared and exercised authority are computed per access path;
  - sinks are recorded as effects in the lockfile;
  - FFI wrappers carry explicit claims;
  - unmodeled foreign effects block `ferox sim` by default;
  - Rust crates get symbol scans and sandboxed build scripts, including proc
    macros.
- **Suspension is specified.** May-suspend classification covers closures and
  `dyn` (new problem 12), the evented lowering has a stated frame model, and
  FFI callbacks cannot suspend (C-B3).
- **The validation gates are measurable.** D0 now has numeric E1 and E3
  thresholds (St-B6, A-B7).
