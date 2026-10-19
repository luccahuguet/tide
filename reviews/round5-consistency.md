# Round 5 Review: Confirmation of Cross-Document Consistency and Soundness

Reviewer persona: a programming-language researcher who has contributed to the
Rust borrow checker.

Scope: `design/01-design.md` (draft 3.3 and its later additions; the Tiers
table defines the v0 core), `design/02-memory.md`,
`design/03-authority-concurrency.md`, `design/04-contracts.md`, and
`design/reference.md`, checked against `reviews/round4-consistency.md` (R1–R6)
and `reviews/round4-codex.md` (B1 replay identity, B2 NaN). Only v0-core
problems are raised. Later tiers, style, and wording are out of scope.

Citations: `01 D8` is a decision, `03 TASK-2` a rule ID, `04 §2.4` a section,
`ref 7.1` a section of `reference.md`, and `01:435–444` a line range. The
scenarios are deductions from the text, not observed failures of an
implementation.

## Verdict

**request_changes**

Draft 3.3 closes nearly all of round 4. R1, R2, R4, R5, and R6 are resolved,
and so are both Codex items. R2 is resolved with an alternative: a snapshot
panic is a violation, and the docs give the `Option` idiom. R3 is partial:
D8 now makes the body's own `Err` produce echoes, but TASK-2 and ref 7.1 were
not updated, and 03 §13.2 still lists the question as open.

Three new v0 holes remain. All three break the determinism claim, and each is
fixed by one sentence:

- **N1.** `par_sort`'s comparison structure is left to the runtime, so
  PAR-1 fails for it.
- **N2.** Whether a sampled clause runs is not a function of the seed, which
  breaks replay and the determinism self-check.
- **N3.** `min` and `max` of signed zeros differ by target and engine.

None of them changes the architecture.

## Round-4 status

| Item | Status | Evidence | Residual |
| --- | --- | --- | --- |
| R1 Parallel element types admit sync, blocking, I/O | resolved | 01 D9 element-type rule (01:469–472); 02 §8.5 (02:766–771) and §9 table (02:846); 03 §1.3 (03:157–159), §3.4 (03:384–386), §3.5 (03:393), PAR-4 and `FX0612` (03:832–838); ref 6.6 (ref:654), ref 8 signatures (ref:1077–1080) | None for R1. `par_sort` has a separate problem (N1) |
| R2 `old` of non-`Copy` or borrowed values | resolved (alternative) | 01 D10 (01:516–520); 04 CLAUSE-4 (04:374–390): `Copy` is copied, anything else is cloned, a reference is snapshotted as its referent, the snapshot is borrow-checked as a local from entry to return, and neither-`Copy`-nor-`Clone` is an error. `Clone` falls under CLAUSE-3 (04:363–364), which excludes `Rc` refcount effects. A snapshot panic is a violation (CLAUSE-7, 04:406–409), and the docs give the `old(v.get(10).cloned())` idiom in place of round 4's deferred panic. Ref 10.1 agrees (ref:1131–1137) | none |
| R3 Scope result reports a cancellation echo | **partial** | Sibling echoes are fixed: TASK-2 has the per-child bit (03:636–642), and D8 and ref 7.1 agree (01:440–442; ref:870–872). Body echoes are not. D8 says that when the body returns `Err`, the children it cancels are echoes (01:443–444). TASK-2 and ref 7.1 define an echo only as a cancellation "a sibling's failure caused", and 03 §13.2 (03:1397–1400) still lists this as open. D8 still lacks TASK-1's body-panic exception | See below |
| R4 Clause callees; generic skip | resolved | 01 D10 (01:510–513, 524–525); 04 CLAUSE-3 (04:358–373); 03 §3.5 now defers to CLAUSE-3 (03:398); ref 10.1 (ref:1141–1148). The `all`/`any` prohibition is gone everywhere. All four documents give one generic rule: the definition needs `Pure + Sync`, and no instantiation skips a clause | none |
| R5 Panic during unwinding; stack overflow | resolved | 01 D5 (01:300–302); 03 PANIC-1, PANIC-2 (03:237–245); 02 §8.3 (02:720–726); ref 9 (ref:1105–1108) | What an abort means under the simulator (note 1) |
| R6 NaN sign and payload | resolved | 01 D3 (01:266–270); 03 PAR-3, including `const fn` evaluation (03:858–863); ref 4.3 (ref:387–390). "Every operation that reads a float's sign or bits" also covers `to_*_bytes` | Signed zero is a separate hole (N3) |
| Codex B1 Replay identity | resolved | 01 D15 (01:693–697); 03 REPLAY-1 and REPLAY-2 (03:1014–1028): source and dependency hashes, semantic build identity, refusal on mismatch, and `--seed` declared not a replay; ref 6.8 (ref:719–723, 781–785); §9.3's claim is now qualified (03:943–946) | Sampling counters are a replay input the manifest does not fix (N2) |
| Codex B2 NaN observers | resolved | as R6 | none |

**R3 scenario.** The body returns `Err(Invalid)` while child `x` is inside
`net.get`. `x`'s `get` returns `Cancelled`, and `x` returns `Err`. Under
TASK-2 and ref 7.1 the bit stays clear, because a body is not a sibling. So
`x` is a failing child, and rule 2 returns `Cancelled`. Under D8 the scope
returns `Invalid`.

**R3 fix.**

- TASK-2 and ref 7.1: set the bit for a cancellation "that a sibling's
  failure or the body's `Err` caused".
- D8: end the sentence at "are echoes". As written, its tail "returns the
  body's error unless a child panicked" overrides rule 2 for a child that
  fails without ever observing the cancellation.
- D8: add "unless the body panicked".
- Delete 03 §13.2's bullet.

## New v0 blocking items

### N1. `par_sort`'s comparison structure is left to the runtime, so PAR-1 fails for it

**Where.**

- 01 D9 says "Granularity is the runtime's job" (01:487–488).
- 03 §8.4 uses lazy binary splitting and says "scheduling is unobservable, so
  the policy can change freely" (03:899–904).
- PAR-1 says values, writes, and sink output are identical "on any core
  count, runtime, or schedule" (03:842–846).
- PAR-4 admits `par_sort` elements that are `T: Pure + Send` (03:835).
- Ref 8 (ref:1075–1079).

**Problem.** For `par_map` and `par_for`, granularity is invisible because
each element is visited exactly once. For a sort, the splits and merges
decide which pairs are compared, how many times, and in what order. Nothing
fixes that structure, and D9 hands granularity to the runtime. The
comparison runs authority-free user code, which v0 lets observe its own call
sequence:

```tide
struct Job { key: u32, probes: Cell<u32> }       // Pure + Send: PAR-4 admits it
impl Ord for Job {                               // PartialOrd, Eq, PartialEq elided
    fn cmp(&self, o: &Self) -> Ordering {
        self.probes.set(self.probes.get() + 1);  // `&self` reaches the Cell
        self.key.cmp(&o.key)
    }
}
jobs.par_sort();
println!("{}", jobs.iter().map(|j| j.probes.get()).sum::<u32>());
```

The order is consistent, so the sorted result is unique. The printed total,
however, is the number of comparisons. It differs between `par-threads = 1`
and `par-threads = 64`, and between runs, because lazy splitting depends on
deque state. The same holds without any `Cell`:

- A `trace!` in `cmp` changes the sink output that PAR-1 promises.
- A `cmp` that panics on one particular pair panics or not depending on
  whether that pair is ever compared.

This falsifies D7's "results depend only on its arguments" in production.
The simulator never sees it, because it runs regions sequentially (§8.4).

**Smallest fix.** Add one sentence to D9 and PAR-1: `par_sort`'s split
points, leaf sort, and merge order are a fixed function of the input (its
length and the comparison results), set by the language version as G(n) is.
PAR-1's "sequential evaluation" is that structure run left to right. Lazy
splitting only decides which thread runs a subtree.

An alternative is to move `par_sort` to v1 beside `par_reduce`. The Tiers
table already names only `join`, `par_map`, and `par_for` (01:73).

### N2. Whether a sampled clause runs is not a function of the seed

**Where.**

- 04 §2.4 evaluates sampled clauses "on the first 64 calls of each function
  in a test run" (04:473–477).
- 04 CLAUSE-6 and CLAUSE-7 (04:399–412); ref 10.1 (ref:1162–1165).
- 03 REPLAY-1 and REPLAY-2 (03:1014–1028); the self-check (03:1035–1036);
  the trace kinds `sink` and `assert` (03:1010–1012); 03 §9.1's one trace
  per seed (03:915–918).

**Problem.** Two inputs decide whether a sampled clause is evaluated on a
call: the function's call counter and the step budget. Neither is tied to the
simulated run:

- "In a test run" does not say whether counters restart per seed, per test,
  or per `tide test` process.
- No document defines a step. Round 4 flagged this as non-blocking.

Evaluation shows up in the trace in three ways: a violation panics, a clause
callee or an `old` `Clone` may log (CLAUSE-3 allows sinks), and the trace
records `assert` events.

**Scenario.** Take `#[sim_test(seeds = 100)]`, where each seed calls
`insert_sorted` 40 times and `insert_sorted` has the sampled clause
`ensures is_sorted(v)`. Suppose counters persist across seeds, the natural
reading of "test run":

- **The self-check fails.** Seed 1 checks calls 1–40. The self-check reruns
  seed 1 and now checks only calls 41–64 of the global count, so the trace
  hashes differ. §9.7 then reports a determinism failure in a deterministic
  program.
- **Replay is not bit for bit.** Seed 3 covers global calls 81–120, so it
  checks none of them (the next scheduled call is 128). `tide sim --replay`
  of seed 3 checks all 40 calls. It can then hit a violation the original
  run skipped, and it emits records the original never did.

With per-process counters and tests on parallel threads, the evaluated set
also depends on thread timing. An engine that counts steps differently
reports a clause as unchecked where another panics, which breaks §9.1.

**Smallest fix.** One sentence in 04 §2.4 (CLAUSE-6) and ref 10.1:

- Sampling counters start at zero in every simulated run: each seed of each
  test, and each self-check rerun.
- A step is one mid-IR call or one loop back-edge, in every engine.
- The schedule and budget settings are semantics-affecting options in
  REPLAY-1.

### N3. `min` and `max` of signed zeros differ by target and engine

**Where.**

- 01 D3: "observable results match across backends and targets while
  arithmetic stays native" (01:266–270).
- 03 PAR-3 (03:858–863).
- Ref 1, scalars "with their Rust API" (ref:64), and ref 4.3 (ref:387–393).
- 03 §9.1, one trace across engines (03:915–918).

**Problem.** R6's fix canonicalizes NaN wherever a float's sign or bits are
read. A signed zero, though, is an ordinary value that no rule
canonicalizes. Rust's contract for `f32`/`f64` `min` and `max` lets them
return either input when the inputs compare equal, and hardware differs:

- x86 `maxsd` returns its second operand for two zeros.
- AArch64 `fmaxnm` orders −0.0 below +0.0.
- An interpreter that calls the host's `f64::max` need not agree with
  Cranelift-generated code or with `const fn` evaluation.

**Scenario.** `let z: f64 = "-0".parse()?; println!("{}", 1.0 / z.max(0.0));`
prints `inf` on one engine or target and `-inf` on another.
`is_sign_negative` and `total_cmp` see the same difference. So does
`fold(f64::NEG_INFINITY, f64::max)` over data containing both zeros.

**Smallest fix.** One clause in D3, PAR-3, and ref 4.3: `min` and `max` order
−0.0 below +0.0, as IEEE 754-2019 `minimumNumber` and `maximumNumber` do, in
every engine, `const fn` evaluation included. On x86 this costs one extra
instruction.

## Non-blocking notes

1. **PANIC-2 and node panics under the simulator.** PANIC-2 "aborts the
   process", but under `tide sim` and `#[sim_test]` one process hosts several
   nodes and the harness. Nothing says whether an abort ends the run or downs
   only that node. Nothing says what an ordinary panic in a node's main task
   does either: ref 6.8's `run_until` covers only `Result` and down nodes.
   Engines that choose differently produce different traces. Fix: a
   simulated PANIC-2 ends the run with an `abort` failure record, and a
   node's main-task panic downs the node like exit status 101.
2. **The echo bit misses two paths.**
   - A `spawn_isolated` task inherits the child's cancellation (TASK-1), but
     its checkpoints are neither "in the child" nor "in a scope nested in
     it".
   - A `join` that returns `Cancelled` because the joined task failed is not
     a checkpoint (JOIN-2).

   As a result, ref 7.1's server pattern, a scoped child that wraps a
   request in `spawn_isolated`, reports its echo as a failure. Fix: also set
   the bit for checkpoints in tasks the child isolates and for joins that
   report a failed task.
3. **Sink output during clause evaluation** (round 4, still open). CLAUSE-3
   says an erased build "differs only by the missing checks" (04:366–367),
   but clause callees and `old`'s `Clone` impls may log. Either discard sink
   records emitted during clause evaluation or state the exception. After
   N2's fix the output is at least deterministic per seed.
4. **Sink-handle classification** (round 4, still open). Ref 6.6's extra
   clause (ref:634–635) makes any `Sync` type whose `&self` API can cause
   mutation not `Pure`. 02 §9 has no such clause. Under it, `Stdout` is not
   `Pure`, so a `with` closure that calls `stdout()` locally breaks LOCK-4's
   "every call in it is authority-free" (03:723–725). Fix: classify std's
   sink handles in 02 §9, for example as `Pure + Send` and not `Sync`, then
   move ref 6.6's general clause there or drop it.
5. **The `const fn` parameter rule.** Ref 2.3 and ref 6.6 (ref:161–162, 658)
   require authority-freedom and no `&mut`. 02 §9 and 03 §3.5 (02:847;
   03:394) also require `Pure + Sync` parameters, because they share a row
   with memoization. The two sets of documents therefore accept different
   `const fn` signatures. Fix: give `const fn` its own row, and state that v0
   engines never memoize (round 4's note). That also settles SINK-7.

## Ready to implement v0

**Yes.** Two teams could build the same of each of these from the text
today:

- The ownership and origin checker, with rustc as the oracle.
- Auto-trait classification, including PAR-4's element bounds.
- Capability typing and `Root`.
- Coherence.
- Clause typing (CLAUSE-1 to CLAUSE-5).
- Scope typing.
- A sequential reference interpreter with precise traps, PANIC-1 and PANIC-2,
  the simulator scheduler, and the replay manifest.

Four one-sentence decisions must land before the conformance tests for these
rule IDs are frozen. They do not need to land before work starts:

1. N1, for PAR-1.
2. N2, for CLAUSE-6 and REPLAY-2.
3. N3, for D3 and PAR-3.
4. The R3 sync, for TASK-2.
