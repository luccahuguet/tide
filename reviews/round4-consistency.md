# Round 4 Review: Cross-Document Consistency and Soundness

Reviewer persona: a programming-language researcher who has contributed to the
Rust borrow checker.

Scope: `design/01-design.md` (draft 3.2, with the Tiers table),
`design/02-memory.md`, `design/03-authority-concurrency.md`,
`design/04-contracts.md`, and `design/reference.md`, against
`reviews/round3-consistency.md` and `reviews/round3-codex.md`. Only the v0
core is examined for new problems. Later tiers are checked only for the
round-3 items that this review must track.

Citations: `01 D8` is a decision in `01-design.md`, `03 TASK-2` a rule ID,
`04 §1.8` a section, `ref 7.1` a section of `reference.md`, and `01:425` a
line. `C-B2` is round-3 consistency blocking item 2, `C-L1` its later-tier
item 1, and `X-V3` round-3 Codex item V3.

## Verdict

**request_changes**

Draft 3.2 closes most of round 3. Of the 22 round-3 blocking items, 15 are
resolved, 3 are partial, and 4 are open. The 4 open items are all v1 and
were already deferred. Lazy statics are gone. `par_map` and `par_for` take
`Fn + Pure + Send + Sync`. The threads runtime is one OS thread per task.
Simulation runs parallel regions sequentially, and 04 §1.8 is now a real
normative statement of v0 clause meaning. The approval hash covers called
bodies, and coherence ignores missing auto traits.

Six v0 problems remain. Each fix is one to three sentences, and none
changes direction:

- **A determinism and progress hole in parallelism (R1).** Admission checks
  the closure and its captures but not the slice element type. Branches can
  therefore lock `Shared` cells, block on channels, and do I/O.
- **A memory-safety gap in debug-build `old` (R2).** The rule for
  snapshotting non-`Copy` or borrowed values is missing.
- **A scope-result rule that hides root causes (R3).** It reports a sibling's
  cancellation echo instead of the failure that caused the cancellation,
  which contradicts D8 and ref 7.1.
- **One contradiction inside the normative clause rule (R4).** It concerns
  `Iterator::all` and the generic skip, and four documents give three
  readings of it.
- **Two undefined panic edge cases (R5).** A panic during unwinding and stack
  overflow have no defined outcome, so engines diverge, and one reading is
  unsound.
- **NaN sign and payload (R6).** These are still visible across targets and
  across compile-time and run-time evaluation.

## Round-3 status

### `round3-consistency.md`

| Item | Status | Evidence | Residual |
| --- | --- | --- | --- |
| C-B1 `par_map`/`par_for` admit a race through a shared closure | resolved | 01 D9 (01:451–455); 02 §8.5 (02:750–754); 03 §8.1 and `FX0610` (03:806–811); ref 8 (ref:1067–1072) | The element type is unconstrained, a different hole (R1) |
| C-B2 scope outcome underdetermined | resolved | 01 D8 precedence (01:425–430); 03 TASK-1, TASK-2, TASK-4 (03:603–645); ref 7.1, 7.2 | Panic vs. `Err`, several panics, and deadline recognition are all specified. The chosen "earliest-spawned failing child" rule reports cancellation echoes (R3). D8 omits TASK-2's "unless the body panicked" |
| C-B3 three meanings of a v0 clause | partial | 04 §1.8 items 1–8 (04:339–395) settle integers, `==`, read-only calls, panics as violations, bare `*x` (also 04 §1.2 rule 4), `decreases` in v1, classes, and when clauses run | `old` snapshots are still undefined for non-`Copy` values (R2). The callable-function rule contradicts itself (R4). D10's first bullet still calls a bare `*x` an error (01:480–481). The Feel uses v1 `decreases` (01:764) |
| C-B4 lazy statics | resolved | 01 D7 (01:342–344); 03 GLOBAL-2 (03:193–197); 02 §3.7 (02:386–389); ref 6.4 | none |
| C-B5 panicking parallel region | resolved | 01 D9 (01:456–460); 03 PAR-1 (03:818–825), §8.4 (03:875–877), §9.1 (03:887–890); 02 §8.5 (02:761–764) | none |
| C-B6 threads runtime can deadlock | resolved | 01 D8 (01:431–433); 03 §1.3 (03:151, 155–159); ref 7.5 | Breaks again if a parallel branch can block (R1) |
| C-B7 seed and commit do not determine a run | partial | One libm (03 PAR-3, 03:835–838; 01 D15). Raw-pointer addresses need an `unsafe` grant (03 GLOBAL-4, 03:201–205). A replay manifest exists (01 D15, 01:665–667; 03 §9.6, 03:985–990) | NaN sign and payload are still visible (R6). The manifest has a commit but no source-snapshot hash (non-blocking) |
| C-B8 approval misses called predicate bodies | resolved | 01 D11 (01:554–556); 04 §1.8 item 8; 04 §11 (04:1047–1068: called bodies, operator impls, "undecided" changes need approval); ref 10.3 | Generic-clause skips are ungated (non-blocking) |
| C-B9 overlap with auto traits | resolved | 01 D4 (01:277–278); 02 §9 (02:778–781, `Owned` included); ref 5.1 | D4's list omits `Owned` (non-blocking) |
| C-L1 `ThreadBound` created on the blocking pool (v1) | open | 03 §5.4 still inlines only calls that *take* a `ThreadBound` argument (03:563–564) | v1 |
| C-L2 SUSP-1 ignores argument types | resolved | 01 D7 (01:374–376); 03 SUSP-1 (03:746–752) | none |
| C-L3 a helping `join` deadlocks inside `with` (v1) | open | 03 §8.4 still shares the pool with evented workers (03:869–870) | v1 |
| C-L4 drop glue missing from the call graph (v1) | open | 04 §3.3 adds `dyn` and instantiation edges only (04:603–610) | v1 |
| C-L5 production replay omits synchronization order (v1) | open | 01 D17 unchanged (01:703–708) | v1 |

### `round3-codex.md`

| Item | Status | Evidence | Residual |
| --- | --- | --- | --- |
| X-V1 `join` panic cleanup depends on scheduling | resolved (alternative) | Codex asked for sequential execution unless the region is panic-free. The design instead leaves writes unspecified, observable only by destructors, and runs regions sequentially in every simulation engine (01 D9; 03 PAR-1; 03 §8.4; 03 §9.1). Tests always use the simulator (ref 7.5) | none |
| X-V2 lazy statics | resolved | as C-B4 | none |
| X-V3 executable clauses change state | resolved | 04 §1.8 item 3 (`Pure + Sync` arguments, `&mut` only to temporaries) and item 7 (a panic is a violation; a budget overrun is unchecked) | Sink output from clause callees still differs between debug and release (non-blocking) |
| X-V4 approval hash misses ordinary predicate bodies | resolved | as C-B8 | SINK-10 still says exfiltration needs `Net`, `Fs`, or `Proc` (03:292–293), but `info!("{}", s.expose())` needs none, and v0 does not report sink effects (non-blocking) |
| X-V5 thread runtime progress | resolved | as C-B6 | none |
| X-V6 replay identifies too few inputs | partial | The manifest records arguments, environment, files, strategy, fault plan, and injected inputs (03 §9.6) | No source-snapshot hash for a dirty tree. `tide test --seed` (ref 6.8, ref 13) bypasses the manifest. 03 §9.3's "reach every observable outcome" is still unqualified (03:915–917) |
| X-V7 indirect calls have two classifications | resolved | as C-L2 | none |
| X-V8 restart cannot rerun a consuming entry | resolved | 01 D15 (01:670–672); 03 §9.5 (`Fn`, `Owned + Send`, captures `Pure + Sync`; 03:943–946); ref 6.8 (ref:731–735) | none |

Codex's later-tier note on `par_reduce` is still open (v1). Strided lanes
permute the inputs (03 §8.3 step 3), so a proved associativity law alone
does not give "equals `fold`" (03:858–859; 04 §5.3). The fix needs
contiguous lanes or a commutativity law.

## Remaining v0 blocking items

### R1. Parallel slice methods admit synchronization, blocking, and I/O through the element type

**Where.** 01 D7 (01:370–376) and D9 (01:451–460); 02 §8.5 (02:750–759) and
the §9 table (02:834); 03 §8.1 (03:806–814) and PAR-1 (03:818–825); ref 8
(ref:1065–1079).

**Problem.** Every admission rule constrains the closure type (`Fn + Pure +
Send + Sync`) and its captures (`Pure + Sync`). The slice elements are passed
to the closure as arguments, and no rule requires them to be `Pure`. The
reference signature is
`fn par_map<U: Send>(&self, f: impl Fn(&T) -> U + Pure + Send + Sync)`, with
no bound on `T` beyond the implied `Sync`.

```tide
fn tickets(tasks: &Tasks) -> Vec<u64> {
    let c = Shared::new(0u64, tasks);
    let cells = vec![c.clone(), c.clone(), c.clone(), c];
    cells.par_map(|c| c.with(|n| { *n += 1; *n }).unwrap())
}
```

The closure captures nothing, so `FX0610` and `FX0611` pass, and
`Shared<u64>` is `Sync` (02 §8.1). Sequential evaluation returns
`[1, 2, 3, 4]`, but parallel evaluation returns any permutation. That
falsifies PAR-1 and D9's "equal their sequential versions". SUSP-1 itself
classifies the call `f(&elem)` as may-suspend, because its argument type is
not `Pure`.

The same gap breaks two other v0 promises:

- **Progress.** Run `receivers.par_for(|rx| { let _ = rx.recv(); })`, where
  `Receiver<T>` is `Send`, while the senders sit in a task whose own `join`
  is waiting for a `par-threads` pool thread. The blocked branches hold every
  pool thread, so production deadlocks. The simulator runs regions
  sequentially and treats `recv` as a scheduling point, so it completes. That
  contradicts D8's "production cannot deadlock where the simulator makes
  progress" (01:431–433) and 03 §8.1's "a branch ... has no checkpoints".
- **No I/O in branches.** `files.par_for(|f| f.write_all(b"x"))` does I/O,
  although ref 8 says "Branches cannot do I/O".

**Smallest fix.** One sentence in D9, mirrored in 02 §8.5, 03 §8.1 (a new
diagnostic), and the ref 8 signatures: every argument and every capture of a
parallel closure is `Pure`. Concretely, `par_map` requires `T: Pure + Sync`,
and `par_for`, `par_chunks_mut`, and `par_sort` require `T: Pure + Send`, so
every branch is authority-free. The Lean statement of D7 determinism (D0)
should carry this premise.

### R2. `old` has no rule for non-`Copy` or borrowed snapshots

**Where.** 04 §1.8 item 4 (04:371–377); 04 §2.2's paragraph on `old`
(04:429–433), which relies on v1 views; ref 10.1 (ref:1117–1120). This is the
remainder of C-B3 item 6.

**Problem.** 04 says "the evaluator snapshots each maximal subexpression of
`e` that mentions no `result` or bound variable ... and it never calls
`Clone`". Inside a quantifier, the maximal binder-free part of `old(v[i])` is
`&*v` or `*v`:

```tide
pub fn shout(v: &mut Vec<String>)
    ensures forall |i: usize| i < v.len() ==> v[i].len() == old(v[i]).len() + 1
{
    for s in v.iter_mut() { s.push('!'); }
}
```

Three implementations follow the text, and each fails:

- **(a) Keep the reborrow `&*v`.** The snapshot reads the values at return,
  so a correct function reports a violation. The reborrow also overlaps the
  body's `&mut` use, which is an aliasing violation in compiled code.
- **(b) Bitwise-copy the `Vec<String>`.** Each `push` that reallocates frees a
  buffer the snapshot still points to, so reading `old(v[i])` at return is a
  use-after-free in a debug build.
- **(c) Deep-copy inside the evaluator.** A compiled backend cannot do this
  for `Box<dyn Trait>`, `Rc`, or arbitrary user types without `Clone`. The
  interpreter and the backend then disagree.

Eager evaluation also blames correct code. With
`ensures old(v.len()) > 10 ==> v[10] == old(v[10])`, the snapshot of `v[10]`
panics at entry when `v.len() <= 10`. Item 7 then reports an `ensures`
violation against a correct callee.

**Smallest fix.** Replace item 4's snapshot sentence (and ref 10.1's) with
this rule:

- A snapshot is a value. A subexpression that reborrows through a `&mut`
  parameter is snapshotted as its referent.
- A `Copy` snapshot is copied. Any other snapshot is taken with `Clone`. That
  call falls under item 3, and the clause becomes `sampled`. A type that is
  neither `Copy` nor `Clone` is a compile error, and the fix moves the
  method call inside `old`, as in `old(v.len())`.
- A panic while taking a snapshot is recorded. It counts as a violation only
  if the clause reads that snapshot at return.

### R3. The scope result reports a cancellation echo instead of the failure that caused it

**Where.** 01 D8 (01:405–406, "The first failure cancels siblings, then
propagates"; 01:425–430); 03 TASK-2 (03:611–626), TASK-4 (03:632–645), and
JOIN-2 (03:663–668); ref 7.1 (ref:857–870, "the scope reports the real
cause").

**Problem.** Rule 2 returns "the error of the earliest-spawned failing
child". A child that fails *because* a sibling's failure cancelled the scope
is also a failing child. Take ref 7.1's own `fetch_both`:

```tide
tasks.scope(|s| {
    let x = s.spawn(|| Ok(net.get(a, deadline)?));
    let y = s.spawn(|| Ok(net.get(b, deadline)?));
    Ok((x.join()?, y.join()?))
})
```

Suppose host `b` answers `Reply::status(503)` after 3 ms and host `a` answers
after 8 ms. `y` fails first with `Status(503)` and cancels the scope.
`x`'s `get` then returns `NetError::Cancelled`, and `?` turns it into an
`Err`. Both children failed, and `x` was spawned first, so `scope` returns
`FetchError::from(NetError::Cancelled)`. The real cause goes to a `debug!`
record. That contradicts D8 ("the first failure ... propagates") and
ref 7.1.

The body is affected the same way. If the body returns `Err(Invalid)` while
an I/O child is in flight, the child's echo wins under rule 2. The body's
error is therefore replaced, despite TASK-4's "the rule never rewrites the
body's error".

The rule's stated aim, "spawn order, not timing" (03:624), is not met
either. Whether `x` echoes depends on whether its reply arrived before `y`'s
failure, and under the simulator that depends on the seed.

D8's precedence bullet also omits TASK-1's exception: when the body panics,
the body's panic continues, not a child's.

**Smallest fix.** Add this to TASK-2 rule 2, D8, and ref 7.1:

- A child's `Err` counts as a failure only if the child had not observed its
  scope's cancellation at a checkpoint before returning. A later `Err` is a
  cancellation echo and goes to the `debug!` record.
- Rule 2 reports the earliest-spawned failure.
- If there is none, rule 3 returns the body's result. If the body returned
  `Ok` and echoes remain, `scope` returns `DeadlineExceeded` (TASK-4, for
  the scope's own deadline) or else the earliest-spawned echo.

The runtime keeps one bit per child, set when a checkpoint hands that child
a cancellation error. It never inspects `E`. Also add "unless the body
panicked" to D8.

### R4. The normative clause rule gives three readings of what a clause may call

**Where.** 01 D10 (01:492–496); 04 §1.8 item 3 (04:357–370); 03 §3.5,
the contract-clause row (03:386); ref 10.1 (ref:1124–1133); ref 6.6's table
(ref:654).

**Problem.**

1. **`Iterator::all` and `any`.**
   - D10 says clauses may take `&mut` to clause-created temporaries "(so
     `xs.iter().all(...)` works)". 04 §1.8 item 3 uses the same iterator as
     its example of a permitted temporary. Ref 10.1 lists
     `xs.iter().all(|x| *x > 0)` among allowed calls.
   - The last sentence of 04 item 3, and the very next sentence of ref 10.1,
     say `all` and `any` take `&mut self`, so a clause must use `forall` or
     `exists` instead.
   - 03 §3.5 admits only callees "with no `&mut` parameters", which rejects
     every iterator adapter.

   So `ensures xs.iter().all(|x| *x > 0)` compiles under one reading and is
   an error under the other two.
2. **The generic skip.** 04 item 3 leaves a clause unevaluated when any
   argument type is not `Pure + Sync`. Ref 10.1 skips only "a clause that
   calls [the type parameters'] methods". Take
   `fn push_one<T>(v: &mut Vec<T>, x: T) ensures v.len() == old(v.len()) + 1`
   at `T = Cell<u8>`. 04 skips the clause, while the reference evaluates it
   and panics on a violation.

**Smallest fix.**

- Keep D10's temporaries rule.
- Delete the "`Iterator::all` and `any` take `&mut self`" sentences from 04
  §1.8 item 3 and ref 10.1.
- Replace 03 §3.5's row with "as 04 §1.8 item 3".
- State one generic rule in 04 and the reference. 04's is simpler.

### R5. A panic during unwinding and a stack overflow have no defined outcome

**Where.** 03 §2.3 (03:232–234, "Stack overflow abandons the task"); 03 §7.2
(03:772–774, v1: "panics as on stack overflow"); 01 D5; 03 §9.1 ("a seed
yields one trace in all of them"). Round 3 listed both cases among the
decisions blocking agreement.

**Problem.**

1. **Panic during unwinding.** A `Drop` impl can panic while its task is
   already unwinding: a failed `RefCell::borrow_mut`, an overflow trap, or a
   violated `requires` in a callee. Rust aborts in that case, but Tide says
   nothing. An interpreter that keeps unwinding and a compiled backend that
   aborts then produce different exit statuses and traces from one seed.
2. **Stack overflow.** "Abandons the task" has three implementations, and
   each has a cost:
   - **Kill the OS thread from a guard-page handler.** This frees a stack
     that scoped children still borrow, because 02 §8.2 relies on scope exit
     to join them. That is a use-after-free.
   - **Raise a panic.** Every prologue needs a stack-limit check.
   - **Abort the process.** `spawn_isolated` can no longer contain deep
     recursion on hostile input.

**Smallest fix.** Two sentences in 03 §2.3. A panic that escapes a
destructor while its task is unwinding aborts the process, as in Rust, and
the simulator records a `double_panic` failure. Stack overflow is a panic
raised by a stack-limit check at function entry (in the interpreter, by a
depth limit), so it unwinds, joins scoped children, and can be caught by
`spawn_isolated`. The depth at which it occurs stays outside the
determinism claim.

### R6. NaN sign and payload remain platform-visible

**Where.** 01 D3 (01:261–262, "observable results match across backends
and targets") and D15 (01:667–668); 03 PAR-3 (03:834–835) and §3.3 item 4
(03:354–355); ref 4.3 (ref:387–390). This is the remainder of C-B7 item 1.

**Problem.** Only `to_bits` canonicalizes NaN. Take
`let z: f64 = "0".parse()?; let q = z / z;`. On x86-64, SSE produces the
default NaN with the sign bit set, while AArch64 produces it with the sign
bit clear. As a result:

- `q.is_sign_negative()` is `true` on one target and `false` on the other.
- `v.sort_by(f64::total_cmp)`, which ref 4.3 recommends, puts `q` first on
  one target and last on the other.

On a single host, a compile-time evaluation of `0.0 / 0.0` disagrees with the
same division at run time, because APFloat-style folders return the positive
quiet NaN. The interpreter's `const fn` result and compiled code can
therefore differ, contradicting D3 and 03 §3.3 item 4.

**Smallest fix.** One sentence in D3, repeated in PAR-3 and ref 4.3: every
std operation that reads a float's sign or bits treats any NaN as the
canonical quiet NaN. That covers `to_bits`, `to_le_bytes`, `to_be_bytes`,
`to_ne_bytes`, `is_sign_positive`, `is_sign_negative`, `signum`,
`copysign`, `total_cmp`, and the `Hash` and `Encode` impls.

## Non-blocking

### Determinism and tooling

- **Unit of the step budget.** 04 §1.8 item 7 and §2.4 give each `sampled`
  evaluation a step budget but do not define a step. Engines that count
  differently can report the same clause as violated in one and unchecked in
  another, which breaks 03 §9.1. Fix: count mid-IR calls and loop back-edges
  executed during evaluation.
- **Sink output during clause evaluation.** 04 §1.8 item 3 says a build that
  erases clauses "differs only by the missing checks", but a callee's
  `info!` or `println!` appears only in debug builds. Fix: discard sink
  records emitted during clause evaluation, or state the exception.
- **Replay identity.**
  - Add D14's source-snapshot hash to the manifest, and have `--replay`
    reject a mismatch.
  - Make `tide test`'s printed replay command load the manifest.
  - Qualify 03 §9.3's "every observable outcome" claim to the simulator
    model (X-V6).
- **`par_sort` with an inconsistent comparator.** PAR-1 equality holds only
  if the sort's split and merge structure depends only on the length. 03
  §8.4 lets lazy splitting change granularity at run time. State that the
  structure is length-determined, as G(n) is for `par_reduce`.
- **Generic skips are ungated.** Adding a `Cell` field to a type makes other
  items' generic clauses unchecked at that instantiation without changing
  any spec-closure hash (04 §1.8 item 3, §11). The report lists the clause,
  but no approval is needed. Consider reporting newly unchecked root clauses
  in `tide review`.
- **Memoization.** D7, 03 §3.5, and SINK-7 mention compiler memoization, but
  nothing defines it. State that v0 engines never memoize, or 03 §9.1's
  one-trace claim fails.

### Classification gaps

- **Sink handles.** `stdout()` and `stderr()` handles and `trace::span`
  guards still have no auto-trait row in 02 §9. Ref 6.6's general clause
  ("any `Sync` type whose `&self` API can ... cause mutation" is not `Pure`)
  would make a `Sync` `Stdout` impure, contradicting "sinks keep a function
  authority-free". Classify them, for example as `Pure`, `Send`, and not
  `Sync`.
- **Where the general rule lives.** Put ref 6.6's general rule and
  `OnceLock` (if std has it) into 02 §9, which 03 calls the full table.

### Ownership and tiering

- **Authority policies.** D17 marks them v0, and ref 12.4 specifies them,
  but the Tiers table and 03 do not. Give 03 the normative rule.

### Stale wording carried from round 3

- D12 still puts the Cranelift-class backend "in v1" (01:588–589), while the
  Tiers table puts it in v0.
- D10's first bullet calls a bare `*x` an error (01:480–481).
- The Feel uses `decreases` (01:764).
- D8 says "suspended work unwinds and runs destructors" (01:410–411).
- D5 says "no code ever observes data a panicking task was mutating"
  (01:294–295).
- D2 says a returned `impl Trait` "must name its origins" (01:214–215),
  while 02 §5.2 accepts elided ones.
- D6 still describes `Shape` construction, which the Tiers table defers to
  Later.
- 01's open question 4 conflicts with 03 §3.2 (ref 16 item 3).
- D8 omits `with_all` (ref 16 item 2).
- D4's auto-trait list omits `Owned`.
- SINK-10's exfiltration sentence (X-V4) remains.

### Smaller unspecified points

- What two `Root::sim` values in one test mean.
- Which unsynced operations a crash may lose: a prefix in issue order, or
  any subset.
- Closed stdout (status 141, SINK-5) should be listed beside exhaustion in 03
  §2.3.
- `ptr::eq` on zero-sized pointees.
- What counts as a "public constructor" for generated tests.
- Whether clauses are borrow-checked at their check point, for example
  `ensures *result == v[0]` when `result` reborrows `v`.

### Later tiers, still open

- C-L1, C-L3, C-L4, and C-L5 (table above).
- `par_reduce`'s strided lanes.

## Ready to implement v0

**Yes, to start.** The following parts are precise enough for two teams to
build the same thing:

- The ownership and origin checker (02 §1–§6, §8, §10, §11, with rustc as
  the oracle).
- Auto-trait classification.
- Capability typing and the `Root` rules.
- Coherence.
- The v0 clause grammar and code semantics (04 §1.8).
- Scope typing.
- A sequential interpreter with precise traps and a simulator scheduler.

None of R1–R6 changes the architecture. They are local decisions, and they
must be written down before the conformance tests for these rule IDs are
frozen:

1. The element-type bound for `par_map`, `par_for`, `par_chunks_mut`, and
   `par_sort` (R1).
2. How `old` snapshots a value: `Copy`, else `Clone`, referents of `&mut`,
   and deferred panics (R2).
3. Cancellation echoes in TASK-2 (R3).
4. One rule for clause callees and the generic skip (R4).
5. Panic during unwinding, and stack overflow as a panic (R5).
6. Canonical NaN on every bit or sign read (R6).

Two inputs still live outside these documents: the standard-library
signatures, which 03 §1.1 defers to a std reference, and the auto-trait rows
for sink handles.
