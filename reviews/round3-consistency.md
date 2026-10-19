# Round 3 Review: Cross-Chapter Consistency and Soundness

Reviewer persona: a programming-language researcher who has contributed to the
Rust borrow checker and trait solver.

Scope: I read all of `design/01-design.md` (draft 3, with the Tiers table),
`design/02-memory.md`, `design/03-authority-concurrency.md`, and
`design/04-contracts.md`, and the three round-2 reviews
(`round2-consistency.md`, `round2-codex.md`, `round2-simplicity.md`).
`design/reference.md` is being revised and is ignored, except where a chapter
delegates a v0 rule to it (B3). The v0 core is reviewed first; later tiers are
checked only for rules that are unsound or that v0 would block.

Citations: `01 D5` is decision D5 in `01-design.md`; `02 §8.3` is a section of
`02-memory.md`; `03 TASK-1` is a numbered rule in
`03-authority-concurrency.md`. `C-B4` is round-2 consistency blocking item 4,
`X-B7` is round-2 codex blocking item 7, and `C-N13` is a round-2 consistency
non-blocking item. Items that draft 3 resolves are not raised again.

## Verdict

**request_changes**

Draft 3 is a large improvement. The Tiers table removed most of what round 2
attacked: `prove`, `final`, check deletion, field borrows, the evented runtime,
and FFI are out of v0, and `Frozen`, `lock`, `par { }`, and the moved-out rule
are gone. Of the 18 round-2 blocking items, 14 are resolved, 2 are partial, and
2 are open (tables below). The ownership and origin chapter is now precise
enough to implement against rustc as an oracle.

What remains in v0 is small but real:

- **One memory-safety gap** in the stated admission rule for `par_map` and
  `par_for` (B1).
- **Three rules that two conforming implementations would read differently:**
  the outcome of `scope` (B2), the meaning of a v0 contract clause (B3), and
  overlap checking with auto traits (B9).
- **Four determinism gaps** that falsify "authority-free results depend only on
  the arguments" or "replays bit for bit from seed and commit": lazy statics
  (B4), panicking parallel regions (B5, round-2 X-B7 still open), the bounded
  thread pool (B6), and platform-dependent float and pointer results together
  with an incomplete replay record (B7).
- **One anti-gaming hole** in v0 approval (B8).

Each fix takes one to three sentences, and none changes direction.

## Round-2 status

### Round-2 consistency review

| Item | Status | Evidence in draft 3 | Residual |
| --- | --- | --- | --- |
| C-B1: authority-free code creates handles | resolved | 01 D7 ("creating one takes `&Tasks`"); 03 §1.2 ("Every constructor of a synchronization handle takes `&Tasks`", atomics included); 03 §3.2, last sentence; 02 §9 | `Lazy` is an ungated `Sync` value that is mutated through `&` (B4); SUSP-1 and callable arguments (L2) |
| C-B2: std runs user code at timing-dependent points | partial | `Arc` and `sync::Weak` are handles and not `Pure` (01 D7, 02 §9, 03 GLOBAL-6), which closes `make_mut` and the last-release destructor | Item 3, lazy statics, is unchanged: 03 GLOBAL-2 still lets `build` write sinks and still poisons later forcers with a different payload (B4) |
| C-B3: deferred trap stated twice | resolved for v0 | 01 D3 ("v0 traps are precise"); 04 §3.2 items 1–3; 03 §9.1 | The candidate rule is parked in 04 §3.2 item 3 for the later tier |
| C-B4: check deletion weaker than D3 | resolved (v1) | 04 §2.6 conditions 1–4 adopt the round-2 wording | 01 D12 does not yet key the instantiation cache on law status (04 "Amendments" item 1); non-blocking |
| C-B5: `final` with a stored `&mut` | resolved (v1) | 04 §4.1, third and fourth bullets | none |
| C-B6: half-updated value reaches a caller | resolved | 01 D5 propagates child panics; 02 §8.3; 02 §12 item 8 closed; 04 §4.3 | Precedence between a panic and an `Err` is now the open question (B2) |
| C-B7: termination through `dyn` and instantiations | resolved (v1) | 04 §3.3, "Hidden call edges" | Drop glue is still not a call edge (L4) |
| C-B8: `dyn` hides `ThreadBound` | resolved (v1) | 03 §5.4; 02 §8.1 condition 2; 02 §11.2 R1 | none |
| C-B9: output `from` at an invariant position | resolved | 02 §2.4 item 1 (identification for one origin, and a union at an invariant position is an error); 02 §2.6; 02 §11.1 rows | none |

### Round-2 codex review

| Item | Status | Evidence in draft 3 | Residual |
| --- | --- | --- | --- |
| X-B1: locally created handles; callable arguments | partial | Local creation is closed, as for C-B1 | SUSP-1 still says a call through a closure or `dyn` value may suspend "exactly when that value's type is not `Pure`" (01 D7, 03 SUSP-1). No v0 effect; it breaks the v1 lowering (L2) |
| X-B2: `Arc` reveals counts | resolved | As for C-B2 | none |
| X-B3: `ThreadBound` values created on the blocking pool | open (v1) | 03 FFI-5 and §5.4 still run inline only calls that take a `ThreadBound` argument | L1 |
| X-B4: opposite child-failure rules | resolved | As for C-B6 | none |
| X-B5: log configuration changes results | resolved | 03 SINK-3 evaluates arguments and formats records at every level and in every profile; 01 D7; SINK-8 | none |
| X-B6: destructors observe deferred writes | resolved | Precise traps in v0 (01 D3, 04 §3.2) | none |
| X-B7: panicking parallel regions | open (v0) | 03 PAR-1 still promises identical writes, qualified only by exhaustion and divergence; 01 D15 still runs regions on real cores under simulation | B5 |
| X-B8: `const fn` bodies missing from interfaces | resolved | 01 D12 ("every `const fn` body evaluated at compile time is part of its consumers' cache keys"); 04 §8.1 and §8.4 | none |
| X-B9: contradictory deadline rules | resolved | 03 TASK-4 matches 01 D8; 03 §12.2 returns partial results | How `scope` recognizes "the cancellation" inside a user `E` is unspecified (B2) |
| X-N5: thread-pool progress (non-blocking in round 2) | open | 03 §1.3 is unchanged | Raised to blocking as B6, because `threads` is the v0 default runtime |

### Round-2 simplicity review

| Recommendation | Status | Evidence |
| --- | --- | --- |
| Defer `prove` and everything that exists only for it | adopted (v1) | 01 Tiers; 04 tier header |
| Cut `Frozen`, `lock`, `par { }`, and the moved-out rule | adopted | 01, the paragraph after the Tiers table; 02 and 03 headers |
| Defer `par_reduce`, field borrows, the evented runtime, exercised authority, Rust import, and FFI | adopted (v1) | 01 Tiers |
| Cut the Bend boundary and `Shape` | deferred to Later | 01 Tiers; D6 still describes `Shape` construction (non-blocking) |
| JSON-only `Encode`/`Decode` | adopted | 01 Tiers |
| `from` as an E1 arm rather than the default | not adopted (owner's choice) | 01 D2 |
| Cut `#[pure]` | not adopted, and now inconsistent | 03 §3.2 defines `#[pure]` and FX0511 inside its v0 tier, while 01 Open Question 4 still asks whether to have it |
| A budget on normative semantics beyond Rust | not adopted | Only the reference has a token budget |

Round-2 consistency non-blocking items: N1–N6, N8, N10, N11, N17, and N23 are
fixed. N7, N9 (partly), N12, N14, N15, N16, N18, N19, N20, N21, and N22
remain. N12 and N14 now sit in v0 and become part of B3; N13 and N19 become
part of B7; N20 becomes B9; N15 and N16 are later-tier blockers (L3, L4).

## Blocking (v0)

### B1. The admission rule for `par_map` and `par_for` admits a data race

**Where.** 01 D9 ("They run `Pure + Send` closures that capture shared data
only if it is `Pure + Sync`"); 02 §8.5; 03 §8.1 (FX0610, FX0611); 03 §3.5.

**Problem.** The rule is written for `join`, whose two closures each run once
on one thread. `par_map`, `par_for`, `par_chunks_mut`, and `par_sort`'s
comparator are one closure called from many branches at once, so the closure
itself is shared. A `move` closure that owns a `Cell` or `RefCell` is `Pure`
(02 §9) and `Send` (02 §8.1: `Cell<T>` is `Send` if `T` is), and it has no
shared captures. FX0610 and FX0611 both pass:

```tide
fn tag_all(xs: &[u32]) -> Vec<u32> {
    let next = Cell::new(0u32);
    xs.par_map(move |x| { next.set(next.get() + 1); x + next.get() })
}
```

Two branches call `next.set` concurrently. That is a data race on a
non-atomic cell (with `RefCell`, a race on the borrow flag, which is undefined
behavior), and the result depends on scheduling. A std that implements
`par_map` with `join` would be forced by FX0611 to require `F: Sync`; a
runtime-intrinsic implementation that follows the stated rule would not.

**Smallest fix.** State the bounds. `join` takes `FnOnce() -> R + Pure + Send`
closures. `par_map`, `par_for`, `par_chunks_mut`, and `par_sort` take
`Fn(..) + Pure + Send + Sync`. With `Pure + Sync` on the closure, no captured
state can change while branches run, so per-element results are independent.

### B2. The outcome of `scope` is underdetermined

**Where.** 03 TASK-1, TASK-2, TASK-4, and JOIN-2; 01 D5 and D8; 02 §8.3.

**Problem.** Three rules each say what `scope` returns, and they overlap.

1. **Panic versus `Err`.** TASK-2: "The first failure ... cancels the scope ...
   `scope` returns ... with that failure converted into `E`; later failures go
   to one `debug!` record." TASK-1 defines a failure as returning `Err`, and
   says that after a child's panic "`scope` panics in the calling task". The
   common case hits both: the body returns `Err(Timeout)`, which cancels the
   scope; a child's checkpoint then returns `Cancelled`, and the child calls
   `.unwrap()`.
   - Reading A (TASK-2): `scope` returns `Err(Timeout)`, and the panic becomes
     a `debug!` record.
   - Reading B (TASK-1, Rust's `thread::scope`): `scope` panics.

   The readings decide whether the caller's code after `scope` runs at all,
   and whether an enclosing `spawn_isolated` returns `Err(Panicked)`.
2. **Several panics.** TASK-1 says `scope` panics "with the child's message"
   without saying which child.
3. **Recognizing cancellation.** TASK-4: "The scope fails only if a child (or
   the body) returns the cancellation as its error; `scope` then reports
   `TaskError::DeadlineExceeded`." The error is a user `E`, and the runtime
   cannot see whether it wraps a cancellation. In 03 §12.3, `put`'s body
   returns the `connect` error through `?` after the deadline. Implementation A
   returns that error unchanged; implementation B returns
   `ServeError::from(TaskError::DeadlineExceeded)`. A test that matches on the
   variant passes under one and fails under the other.

**Smallest fix.** One rule in TASK-2, applied after every child has finished:

- If the body panicked, `scope` resumes the body's panic. Otherwise, if any
  child panicked, it panics with the first child panic in real-time order (in
  scheduler order under the simulator). Every `Err` goes to the `debug!`
  record.
- Otherwise, if a failure occurred, it returns the first failure's value
  unchanged. The exception: if the scope's deadline or `cancel()` fired before
  that failure, it returns `E::from(TaskError::DeadlineExceeded)` or
  `E::from(TaskError::Cancelled)`.
- Otherwise it returns the body's `Ok`.

This matches 03 §12.2 and §12.3 as written and needs no inspection of `E`.

### B3. Three sources give v0 contract clauses different meanings

**Where.**

- 01 D10: the spec-language bullet (untiered), the bare-`*x` sentence, and "In
  v0, executable clauses use code semantics".
- 01 Tiers, rows v0 and v1.
- 04 tier header: v0 includes "the parts of §1.1–§1.3 that cover them", and
  "v0 clauses call authority-free functions directly (`reference.md` §10.1)".
- 04 §1.2 rule 4; 04 §1.3; 04 §2.2–§2.4; 03 §3.5.

**Problem.** Each point below is a direct conflict that two implementers would
resolve differently.

1. **Integers.** D10 says "machine integers that trap". 04 §1.3, which 04
   places in v0, says integers "widen to `int` ... and never overflow", and 04
   §2.3 says "The evaluator uses unbounded integers". Under D10,
   `fn widen(a: u8, b: u8) -> u16 ensures result == a + b` is a type error
   (`u16 == u8`), and `requires a + b <= 255` traps at `a = 200, b = 100`.
   Under 04 the first is well-typed and holds, and the second is a clean
   precondition failure.
2. **Equality.** D10 says "`==` through `PartialEq`". 04 §1.3 says spec `==` is
   model equality that "never calls a user `PartialEq`" and compares floats
   bitwise. `ensures result == 0.0` with `result = -0.0` holds under D10 and
   fails under 04. `ensures *v == old(*v)` on a `Vec<Box<dyn Fn()>>` is a type
   error under D10 and holds under 04 (C-N14).
3. **What a clause may call.** D10 (untiered) says "Specs call only `spec fn`s,
   the built-in spec types, and functions marked `#[spec]`", all of which are
   v1. 04 and 03 §3.5 let v0 clauses call any authority-free function.
   - Under the second reading, a clause can mutate a `Cell` or `RefCell`
     reachable through `&`, print, panic, or diverge, so debug and release
     builds differ even when every contract holds. This contradicts 04 §1.3:
     spec evaluation "never changes program state" (C-N12).
   - Example: `requires self.index.lookup(k).is_some()`, where `lookup` fills
     a `RefCell` cache, changes later behavior in debug builds only. If the
     caller holds a `borrow_mut` on that cache, the clause panics in debug
     builds and not in release.
4. **Bare `*x` in `ensures`.** D10 says it "is an error". 04 §1.2 rule 4 says
   that in v0 it "means its value at return".
5. **Tier drift.**
   - The Tiers table puts `decreases` in v1. The Feel's example (01) uses it,
     and 04 §2.4 (v0) checks loop measures.
   - 04 §2.3 and §2.4 (both v0) evaluate only `checked` clauses and sample
     `sampled` ones. The classes, and the definition of a bounded quantifier
     that v0 needs, are in §2.2 (v1).
   - Uses (§2.1) are v1, so nothing says which v0 command behaves as `assert`
     and which as `test`. D10 checks loop invariants under `test`; 04 §2.3
     also checks them under `assert`.
6. **`old` snapshots.** 04 §2.2 snapshots "views", which are v1. A v0 compiled
   backend has no rule for `old(*v)` when `*v` is not `Copy`.

**Smallest fix.** Mark D10's spec-language bullets v1, and give 04 a normative
"v0 clause semantics" subsection instead of citing `reference.md` §10.1:

- A clause is a Rust boolean expression, typed and evaluated as code: machine
  integers that trap, and `==` through `PartialEq`. The added forms are
  `result`, `old(e)`, `==>`, and `forall`/`exists` over integer ranges in
  §2.2's bounded form, which moves into the subsection.
- A clause may call authority-free functions whose parameters are passed by
  value or `&` and are `Pure + Sync`, so no interior mutability is reachable.
- A panic while evaluating a clause is a failure of that clause, blamed by the
  clause's kind.
- Each maximal `old(..)` subexpression is evaluated at entry and must be
  `Copy`.
- A bare `x` or `*x` of a `&mut` parameter in `ensures` is its value at return.
- Debug builds check every clause at its check points, and `tide test` adds
  generated tests and per-iteration loop invariants. Either import §2.2's
  `checked`/`sampled` split into v0 or check every clause everywhere; state
  which. Loop `decreases` moves into v0 as a run-time check, since 04 §2.4
  already specifies it.

### B4. Lazy statics break the authority-free argument

**Where.** 03 GLOBAL-1, GLOBAL-2, §3.3 (the argument), §3.5; 03 §9.5; 01 D7
and D8; round-2 C-B2 item 3.

**Problem.** 03 §3.3's argument rests on "every type that is `Sync` and
mutable through `&` is a synchronization handle and therefore not `Pure`".
`Lazy<Table>` is `Sync`, is mutated through `&` when forced, and must be `Pure`
to be a static (GLOBAL-1). The exception is unlisted, and it is observable in
three ways.

1. **Parallel output and panics differ from sequential evaluation.** `build`'s
   sink records land in the buffer of whichever branch forces first:

   ```tide
   join(|| { let _t = &*TABLE; info!("L"); },
        || { info!("R"); let _t = &*TABLE; });
   ```

   Sequential output is `build`'s records, `L`, `R`. If the right branch
   forces first, the output is `L`, `R`, then `build`'s records. If `build`
   panics, the first forcer reports `build`'s message and the other branch
   gets the poison message, so PAR-2's "first in sequential order" depends on
   timing. Under simulation `join` runs on real cores, so the determinism
   self-check (03 §9.7) reports a false divergence.
2. **Authority-free code can block, and the simulator cannot see it.** Suppose
   `build_a` reads `B` and `build_b` reads `A`. When one task forces `A`, the
   inner force of `A` is reentrant, and 03 does not say what that does. When
   two tasks force `A` and `B` at once under `threads`, each waits for the
   other forever, inside code that "never suspends" (01 D8), and `with`
   closures that force either static block too. Under the simulator, no
   scheduling point exists inside `build`, so one task always runs both
   initializers, and the cross-task hang never appears.
3. **Simulated nodes share statics.** Nodes are separate processes in
   production and one process under `tide sim` (03 §9.5). A static forced on
   node `a` is already initialized on node `b`, a crash and restart does not
   reset it, and a poisoned static poisons every node.

**Smallest fix.** Remove GLOBAL-2 from v0. Statics are `const`-initialized,
and runtime-built tables are values built in `main`, which is already the
"global configuration" row of 03 §2.2. If lazy statics must stay, force every
reachable lazy static in the main task before `main` runs, in dependency
order, once per simulated node; a cycle is a deterministic startup panic.

### B5. A panicking parallel region does not match sequential evaluation (X-B7, still open)

**Where.** 01 D9 and D15 ("Authority-free `par` regions run on real cores
under simulation, because their results are deterministic"); 03 PAR-1, PAR-2,
§8.4, and §9.1; 01 D5 (abandoned values are dropped).

**Problem.** PAR-1 promises that a parallel call's "values, its writes through
`&mut`, and its sink output are identical to sequential evaluation", qualified
only by exhaustion and divergence. A branch to the right of a panicking branch
may write through its `&mut` split before it sees the stop flag, and D5 then
runs destructors that read the write:

```tide
struct Report { value: u32 }
impl Drop for Report { fn drop(&mut self) { println!("{}", self.value); } }

fn f() {
    let mut r = Report { value: 0 };
    let v = &mut r.value;
    join(|| -> () { panic!("left") }, move || { *v = 1; });
}   // sequential: unwinding prints 0; parallel: prints 0 or 1
```

The admission rules accept this: the `&mut` capture is disjoint, and both
closures are `Pure + Send`. Under `tide sim` the region runs on real cores, so
one seed yields two traces, the self-check fires, and the replay is not exact.
The same happens when the written place belongs to a parent task whose frame
unwinds after the child's panic propagates.

**Smallest fix.**

1. Qualify PAR-1 and D9. Results are identical when no branch panics. When one
   panics, the reported panic and the kept sink output are as PAR-2 says, and
   writes by branches to its right are unspecified.
2. Under simulation and in the interpreter, run branches left to right
   (`par-threads = 1`), and delete D15's real-cores sentence, so replay stays
   exact. Production keeps real parallelism; simulating `par`-heavy code gets
   slower.

### B6. The `threads` runtime can deadlock where the simulator cannot

**Where.** 03 §1.3: under `threads`, tasks run on "OS threads from a pool; a
task stays on one thread for life", `workers = "cores"`, and "No runtime runs
children inline ... they differ in cost, not meaning". 01 D8; round-2 X-N5.

**Problem.** A wait blocks its OS thread, and `workers` sizes the pool. Take
`workers = 4` and a scope that spawns five children. Children 1–4 each call
`recv()` on their own channel; child 5 holds the four senders and sends one
message to each. If children 1–4 take every pool thread first, child 5 is
runnable but never runs, and the program hangs. The simulator has no worker
bound, so every seed passes and no deadlock report appears. `threads` is the
v0 default, so the simulator does not test the program that ships, and
"they differ in cost, not meaning" is false.

**Smallest fix.** The `threads` runtime never leaves a runnable task queued
while every pool thread is blocked; it starts another OS thread. `workers`
bounds the threads kept idle, not the threads in use. Equivalently, `threads`
is `blocking` with thread reuse.

### B7. Seed and commit do not determine a run

**Where.** 01 Thesis item 2 and D15 ("replays bit for bit from seed and
commit"); 01 D3 ("`to_bits()` canonicalizes NaN so observable results match
across backends and targets"); 03 §3.3 item 4; 03 GLOBAL-4; 03 §9.1, §9.6,
§9.8; round-2 C-N13, C-N19, and X-N1.

**Problem.** Authority-free code still observes the platform, and the replay
record omits inputs that change the run.

1. **NaN sign and payload.** Only `to_bits` canonicalizes. For `x / y` with
   `x = y = 0.0` at run time, `is_sign_negative()` is `true` on x86-64 and
   `false` on AArch64. `copysign`, `signum`, `total_cmp`, `to_le_bytes`,
   `Hash`, and `Encode` read the same bits.
2. **Float functions.** With Rust's meaning (D1), `sin`, `exp`, `ln`, `powf`,
   and `powi` call the platform libm, whose results differ in the last bit
   between glibc and macOS and between glibc releases. A CI failure on Linux
   that depends on one ulp does not replay on a macOS laptop.
3. **Raw pointers.** Creating one is safe (`&x as *const T`); GLOBAL-4 forbids
   only integer casts and `{:p}`. `PartialOrd`, `Hash`, and `Debug` on raw
   pointers still expose addresses, which differ between engines and, under
   ASLR, between runs. 02 §9 makes raw pointers non-`Pure`, so authority-free
   code cannot receive one, but it can create one. That is the gap round-2 C-B1
   found for handles.
4. **The record.** The replay command in 03 §9.6 omits `--faults <plan.toml>`,
   `--strategy`, and `<bin> -- args`. The record omits the engine, the target
   triple, and a working-tree hash. A run under a fault-plan file replays
   without the plan. Engines differ in stack depth ("Not simulated: stack
   depth", §9.8), so a stack-overflow failure under baton-passed threads may
   not recur in the interpreter.

**Smallest fix.**

- Every std operation that reads a float's sign or bits treats any NaN as the
  canonical NaN.
- std's float functions beyond IEEE's basic operations are Tide code, one
  portable libm, never the platform's.
- Raw-pointer types may be named only in packages granted `unsafe`.
- The record and the replay command carry the seed, the commit plus a
  working-tree hash, the toolchain, the target triple, the engine, the
  strategy, the fault-plan hash, and the arguments. D15 says "on the same
  target".

### B8. v0 approval misses edits to the functions a contract calls

**Where.** 01 D11 ("Hashes cover everything a spec's meaning depends on,
including the bodies of `spec fn`s it calls"); 04 §11 ("Proofs, `proof fn`
bodies, and other exec bodies are excluded"); the 04 tier header (v0 has
"approval of weakened contracts (§11, without its solver checks)", and "v0
clauses call authority-free functions directly").

**Problem.** v0 clauses call ordinary functions, and §11 excludes their bodies
from the spec-closure hash. An agent can weaken a precondition without
touching it:

```tide
pub fn withdraw(&mut self, amt: u64) requires self.covers(amt) { ... }
pub fn covers(&self, amt: u64) -> bool { true }   // was: self.balance >= amt
```

The clause text, the signature, and the hash are unchanged, so `tide review`
shows no contract change and CI asks for no approval. This is the gaming
pattern D11 exists to catch, in the tier where E3 runs. Separately, without a
solver v0 cannot classify a change as a refinement, and 04 does not say what
it does instead.

**Smallest fix.** In v0, the spec-closure hash includes, transitively, the
bodies of every function a root clause calls, and review lists those functions
as spec dependencies. Without a solver, every changed root-spec hash is
`undecided` and needs approval, except a contract added to a previously
unspecified item.

### B9. Overlap checking with auto traits is unspecified

**Where.** 01 D4 ("No overlapping impls ... negative bounds, or user auto
traits"); 02 §9 (`Pure` is deep and structural, and it looks through private
fields); 02 §3.7 (`Owned` depends on origins); round-2 C-N20.

**Problem.** Whether two impls overlap depends on whether the checker may use
the absence of a compiler-known auto trait:

```tide
trait Describe { fn describe(&self) -> String; }
impl<T: Debug + Pure> Describe for T { ... }
impl Describe for Net { ... }               // Net is not Pure
```

A checker that uses the closed definition of `Pure` accepts these impls as
disjoint. A checker that follows Rust's coherence, which never reasons from
the absence of an auto trait, rejects them. The same split occurs for
`impl<T: Owned> K for T` and `impl K for &str`, which overlap at
`&'static str` only if the checker ignores origins as Rust ignores lifetimes.
If disjointness is allowed, a private-field edit, such as adding an `Rc` to a
type, can turn two accepted impls into an overlap error.

**Smallest fix.** One sentence in D4: overlap checking treats every auto-trait
bound (`Send`, `Sync`, `Pure`, `Owned`, and in v1 `ThreadBound`) as possibly
satisfied, so no impl is disjoint from another because a type lacks an auto
trait.

## Blocking (later tiers)

### L1. `ThreadBound` values created on the blocking pool (X-B3, open; v1)

03 FFI-5 runs capability-taking foreign calls on the blocking pool unless they
are `#[nonblocking]`. 03 §5.4 runs inline only calls that take a `ThreadBound`
argument. A constructor `Session::open(fs: &Fs) -> Session`, with
`Session: ThreadBound`, runs on blocking thread B and returns to evented
worker A. The task is then pinned to A, and every later method and the foreign
destructor run on the wrong thread.

**Fix.** Run inline every foreign call whose argument or result type is
`ThreadBound`, and pin from creation to destruction.

### L2. SUSP-1 ignores the argument types of an indirect call (X-B1 remainder; v1)

01 D7 and 03 SUSP-1 say a call through a closure or `dyn` value "may suspend
exactly when that value's type is not `Pure`". Tide function pointers are
`Pure` (02 §9), and so is a closure with no captures. So for
`fn fetch(net: &Net, u: &Url) -> ..`, a call `f(net, u)` through
`f: fn(&Net, &Url) -> ..` is classified as non-suspending, although `fetch`
suspends. Under the evented lowering (03 §7.2), the caller makes it an
ordinary native-stack call, and the callee returns `Pending` to nowhere.

**Fix.** An indirect call may suspend when the callee value's type or any
argument type is not `Pure`. Change 01 D7 to match.

### L3. A `join` that helps evented tasks can deadlock inside `Shared::with` (C-N16; v1)

03 §8.4 shares the `par` pool with the evented workers. A `with` closure may
call `join`, because `join` is `Pure`. Suppose the waiting worker helps by
running another evented task, and that task waits for the same cell. It cannot
suspend from beneath the holder's native frames, so the worker deadlocks.

**Fix.** A `join` waits by running only `par` branches.

### L4. Drop glue is missing from the call graph (C-N15, and C-B7's class; v1)

04 §3.3 adds call edges for `dyn` coercions and instantiations but not for
implicit drops, and 03 §4.1 does not count destructors as exercised authority.

- `struct S; impl Drop for S { fn drop(&mut self) { let _t = S; } }` recurses
  through drop glue forever, yet it is reported as terminating without
  `diverges`.
- A `Drop` impl that uses a held `Net` exercises `net` at every drop site
  without appearing in the exercised summary.

**Fix.** For termination and exercised authority, an implicit drop of a value
of type `T` is a call to `T`'s drop glue.

### L5. Production replay records capability responses but not synchronization order (D17; v1)

01 D17 says the recording root "logs every capability response", and that
replaying the trace "reproduces the run bit for bit". With several tasks, the
order in which tasks acquire a cell, which `recv` gets which message, and
which child fails first (TASK-2) are not capability responses. Two tasks that
each run `cell.with(|c| { *c += 1; *c })` and print the result can replay in
the other order.

**Fix.** The recording runtime also logs the order of every scheduling point
listed in 03 §9.3, and replay enforces that order.

## Non-blocking

### Contradictions to reconcile

- **Backend tier.** 01 D12 places the Cranelift-class backend "in v1". The
  Tiers table puts it in v0 ("then the Cranelift-class backend"), and E4
  needs it.
- **`#[pure]`.** 01 Open Question 4 asks whether to have `#[pure]`. 03 §3.2
  defines it with FX0511 inside its v0 tier, and 03 §13.2 calls the question
  answered. Decide it in 01.
- **Cancellation wording.** 01 D8 says "suspended work unwinds and runs
  destructors" (C-N7). Under 03 TASK-3, cancellation is an `Err` at a
  checkpoint, and code returns normally.
- **`Shape`.** 01 D6 still describes construction through `Shape` for types
  that opt in, and 03 ROOT-4 relies on it. The Tiers table defers only
  read-only `Shape`, to Later.
- **Returned borrows.** 01 D2 says "a returned `impl Trait` or `dyn Trait`
  that borrows must name its origins". 02 §5.2 and §10.1 accept
  `-> impl Iterator<Item = &Person>`, whose elided `&` carries the origin. Add
  "or an elided reference in its bounds carries them".
- **What a panic hides.** 01 D5 says "no code ever observes data a panicking
  task was mutating". Destructors do, as 04 §4.3 says. Write "no code other
  than destructors".
- **Instantiation cache.** 01 D12 omits the law-status key that 04 §2.6 item 4
  and 04 "Amendments" item 1 require (v1).
- **Node restart.** 03 §9.5's `node.start(entry)` takes an `FnOnce`, but
  `restart` "reruns the entry". Make it `Fn`.
- **Memoization.** 01 D7, 02 §9, 03 §3.5, and SINK-7 ("may appear zero or one
  times under memoization") mention compiler memoization, but nothing defines
  it. An engine that memoizes produces a different sink trace from the
  interpreter, which 03 §9.1 forbids. Remove it from v0 or define when it
  applies.
- **Migration.** `tide fix` rewrites a v0 bare `*x` to `final(*x)` (04 §1.2
  rule 4). For a `&mut` that the callee stores, `final` is prophetic in v1, so
  the rewrite changes the clause's meaning. Rewrite only where `final` is
  executable.

### Underspecified, lower priority

- **Panic during unwinding.** Rust aborts when a destructor panics while
  unwinding. Tide says nothing, so the interpreter and the compiled backend
  will differ.
- **Stack overflow (C-N18).** 03 §2.3 says it "abandons the task". If that
  means a panic, compiled code needs a stack-limit check in each prologue, and
  the interpreter a depth limit. If it means an abort, say so. A guard-page
  handler that kills only the thread would free a frame that children borrow.
- **`Tasks::limit(n)`.** Nothing says what `spawn` does at the limit: whether
  it blocks, and whether it is a checkpoint. `spawn` returns `Task`, not a
  `Result`.
- **`Root::sim` in a plain `#[test]`.** Nothing says which thread becomes the
  simulation's main task (03 §9.2 says only that "the test body" does), or
  what two `SimRoot`s in one test mean.
- **Crash ordering.** "May be lost or torn, as the seed decides" (03 §9.5)
  does not say whether the lost operations are a prefix in issue order or any
  subset, per file and per directory, so two simulators will explore different
  crash states. Suggested rule: a seed-chosen prefix, in issue order, of each
  file's and each directory's unsynced operations, plus at most one torn write.
- **Closed stdout.** Under SINK-5, a closed stdout makes `println!` in
  authority-free code exit the process with status 141. List it beside
  exhaustion in 03 §2.3 and §3.3.
- **`as` and pointer-width integers.** `u64 as usize` is lossless only on
  64-bit targets, so whether a program is accepted depends on the target. Pick
  one rule.
- **`ptr::eq` on zero-sized pointees** is unspecified in Rust, and the
  interpreter and compiled code can disagree.
- **Origins on enums.** 02 §3.3's sibling-field `from` has no spelling that
  lets `enum Token { Ident(&str), Str(&str) }` share one origin, and §2.6's
  rewrite of `Token<'a>` is undefined for enums. A signature-level `from`
  covers most uses, so this is E1 data, not a soundness issue.
- **Generated tests.** 04 §2.4 builds types with private fields from "random
  sequences of public constructor and method calls" without defining a
  constructor.
- **Still open from round 2, all v1:** C-N21 (implied bounds through impl
  headers: rust-lang/rust#100051, #84591) and C-N22 (facts kept across loops
  must exclude overlapping places).

## Ready to implement a v0 checker and interpreter?

**No.** The design does not need new ideas for that, but nine rules (B1–B9)
either give two conforming implementations different observable behavior or
admit a data race. Settling each takes one to three sentences.

**What can start now:**

- The ownership and origin checker (02 §1–§6, §8.2, §10, §11), with the rustc
  translation as the shared oracle; it is precise enough for two teams to
  agree.
- `Send`, `Sync`, `Pure`, and `Owned` classification (02 §8.1, §9), once the
  std table is complete.
- Capability typing and the `Root` rules (03 §2.1).
- A sequential interpreter core with precise traps.

**What blocks agreement, most important first:**

1. The outcome of `scope`: panic versus `Err`, several panics, and how a
   deadline failure is recognized (B2).
2. v0 clause semantics: integers, `==`, callable functions, panics inside
   clauses, `old`, bare `*x`, `decreases`, and which clauses run when (B3).
   The chapters should own this rule rather than cite `reference.md` §10.1.
3. The bounds of `join`, `par_map`, `par_for`, `par_chunks_mut`, and
   `par_sort` (B1).
4. Overlap checking with auto traits (B9).
5. Lazy statics: remove them or make them eager (B4).
6. Parallel regions with a panic, and how the interpreter and simulator run
   them (B5).
7. Float and pointer operations that read platform bits, and which libm std
   uses (B7).
8. A panic during unwinding, and stack overflow as a panic or an abort.
9. The progress guarantee of the `threads` runtime (B6).
10. The complete auto-trait table for every v0 std type. 02 §9 does not
    classify `Lazy`, the sink handles (`stdout()`, trace span guards), or the
    simulator's types.
11. Simulator API details: `Tasks::limit(n)` at the limit, `Root::sim` in a
    plain test, `node.start` and `restart`, and the crash-ordering model.
12. v0 approval without a solver, and what the spec hash covers (B8).

Two inputs also live outside these chapters: the standard-library signatures
("The standard library reference holds the full signatures", 03 §1.1) and the
revised `reference.md`. With B1–B9 settled and those signatures fixed, the
answer becomes yes; items 10–12 are details that conformance tests can pin
down.
