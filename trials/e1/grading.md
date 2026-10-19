# E1 grading: trial 1

Inputs: the ten programs in `trials/e1/programs/` (1,614 lines) and
`trials/e1/notes.md`. The writer saw only `design/reference.md`. There is no
task file; each task is taken from the program's header comment.

The grader acts as `ferox check`, `ferox verify`, `ferox test`, and a senior
reviewer, applying the normative design: `design/01-design.md` (draft 2.1,
which wins on conflict), `design/02-memory.md`, `design/03-authority-concurrency.md`,
and `design/04-contracts.md`. Chapter 03 appeared while this grading was in
progress and was still being edited; this grading uses its text as of 04:46.
It fixes many API names the reference marks provisional, and several verdicts
below depend on it.

## Method

- **Error classes.**
  - *Rust habit leaked*: Rust spelling whose Ferox meaning differs.
  - *Reference gap*: a normative chapter decides it, but the reference omits
    it or says something else.
  - *Design gap*: no chapter decides it, or the chapters conflict.
  - *Ordinary bug*.
- **Definite or contingent.** A *definite* error is one the normative design
  rejects, or a failing test or proof. A *contingent* item is a std detail no
  document decides; the writer's guess is plausible and is graded as passing,
  but a std that chose differently would reject it.
- **Severity.**
  - 0: compiles and is correct.
  - 1: one or two trivial fixes.
  - 2: a few real errors.
  - 3: fundamentally wrong, or blocked by a design gap.
- **Iterations** are `ferox check` runs to a green check, then `ferox test`
  and `ferox verify` runs to green. They assume diagnostics that carry the
  fixes shown here.
- **Diagnostic codes** in the compiler messages are placeholders, like the
  reference's own. `FX08xx` marks a code that does not exist yet.

## 1. Results at a glance

| Program | First `ferox check` | Definite errors | Contingent | Severity | Check runs | All runs |
| --- | --- | --- | --- | --- | --- | --- |
| wc | pass | none | none | 0 | 1 | 1 |
| csv_line | pass | none | none | 0 | 1 | 1 |
| lexer | pass | none | none | 0 | 1 | 1 |
| bsearch | **fail** | S1, S2 (check); S3 (verify) | C7 | 2 | 2 | 4 |
| fetch_all | **fail** | F1, F2 (check); F3 (test, logic) | C1 | 2 | 2 | 4 |
| bank | **fail** | B1 (check) | C1 | 1 | 2 | 2 |
| bfs | pass | none | C2 | 0 | 1 | 1 |
| kv_log | **fail** | L1, L2, L3 (check); L4 (logic) | C1, C6 | 3 | blocked | blocked |
| word_freq | pass | W1 (test) | none | 1 | 1 | 2 |
| app_errors | pass | none | C3, C4, C5 | 0 | 1 | 1 |

## 2. Aggregate

### 2.1 Pass rates

- **First-check pass rate: 6/10 (60%).** The six that pass are wc, csv_line,
  lexer, bfs, word_freq, and app_errors. The likely range is 5–6: bfs passes
  only if `Handle<T>` is `Hash + Eq + Debug` (C2).
- **Final pass rate with today's design: 9/10.** kv_log cannot be finished,
  because nothing in the design lets a program open a file for appending (L1).
- Every first-check failure traces to the reference lagging the normative
  chapters, or to that one design gap. None comes from a Rust habit.

### 2.2 Definite errors by class

| Class | Count | Errors | Distinct root causes |
| --- | --- | --- | --- |
| Rust habit leaked | **0** | none | none |
| Reference gap | **9** | S1, S2, S3, F1, F2, F3, B1, L2, L3 | 6 (see below) |
| Design gap or contradiction | **2** | L1, L4 | 2: append mode; durability of directory entries |
| Ordinary bug | **1** | W1 | 1 |

The six reference root causes:

1. A private `spec fn` appears in a `pub` contract (S1).
2. Specs call trait methods (S2).
3. Proofs assume `Ord` laws are ambient (S3).
4. `lock`'s error type (F1, B1).
5. The simulator's fault and crash API (F2, L2, L3).
6. What a passed scope deadline does (F3).

Errors by stage:

| Stage | Errors |
| --- | --- |
| `ferox check` | 8: S1, S2, F1, F2, B1, L1, L2, L3. S2 alone has 9 sites; S1 has 2. |
| `ferox verify` | S3 |
| `ferox test` | F3, W1 |
| Review only | L4 |

### 2.3 Contingent items

| ID | Program | Writer's assumption | Status |
| --- | --- | --- | --- |
| C1 | fetch_all, bank, kv_log | Import paths: `std::sim::{SimRoot, Faults}`. Once F1 is fixed, also `std::sync::LockError`. | Likely right for `SimRoot` and `LockError` (03 ROOT-3, §6.5). `Faults` is wrong (F2). |
| C2 | bfs | `Handle<T>: Copy + Eq + Hash + Debug` | `Eq + Hash` is implied by 02 §10.9, which puts arena ids in a `HashSet`. `Debug` is a guess. |
| C3 | app_errors | `Net::get` returns `Err` for an HTTP 503 | Not stated anywhere. It is the only reading that fits a signature returning only the body. |
| C4 | app_errors | `#[from]` implies `#[source]`, as in thiserror | Not stated. |
| C5 | app_errors | `Fs::exists` returns `io::Result<bool>`, as Rust's `std::fs::exists` does | Not stated. |
| C6 | kv_log | `File::set_len` and `File::sync_data` exist | 03 §1.1 lists only `sync_all` among `File`'s main operations. |
| C7 | bsearch | A generic `spec fn` needs no `Pure` bound | **Design contradiction.** 04 §1.5 rule 3 and 03 §3.5 require `Pure` parameters, but 04's own `sorted_by<T: Ord>` and `le<T: Ord>` omit the bound. |

Four guesses the writer flagged are already settled by the normative
chapters, all in the writer's favor:

- `read_only()` is infallible: 03 §12.1 does exactly what wc does,
  `root.fs.read_only()`.
- `par_reduce` on an empty slice returns `M::empty()` (03 §8.3).
- Test helpers may take `&SimRoot` (03 ROOT-3).
- `lock` acquisition still works in a cancelled scope (03 LOCK-3).

### 2.4 Iterations

Means over the nine programs that are not blocked:

- `ferox check` runs to green: **1.3** (12/9).
- All runs, including test and verify: **1.9** (17/9).

kv_log is blocked on L1. If std gained an append API, kv_log would need
about three check runs, for L1–L3 and the move into a simulated node, plus
one or two test runs.

### 2.5 Comparison with the same writer in Rust

| Program | Rust first check (est.) | Note |
| --- | --- | --- |
| wc | pass | |
| csv_line | pass | `Cow<'a, str>` |
| lexer | pass | `Token<'a>`, `impl<'a> Iterator for Lexer<'a>`: routine for this writer |
| bsearch | pass | No contracts, so nothing comparable to S1–S3. With Prusti or Creusot, likely worse than Ferox. |
| fetch_all | about 50% | tokio: `'static` spawn bounds forcing `Arc` clones, `Send` across `.await`, semaphore permits |
| bank | pass | The writer must order the two locks by id by hand |
| bfs | pass | `Vec` with indices |
| kv_log | pass | `OpenOptions` exists. The crash test cannot be written without a hand-built fault-injecting filesystem, so the Rust test would check less. |
| word_freq | pass | rayon |
| app_errors | pass | thiserror and `reqwest::blocking` |

The Rust estimates are about 9/10 at first check and about 1.1–1.2 runs per
program. Against D0's E1 thresholds:

| Threshold | Result | Verdict |
| --- | --- | --- |
| First check at least 0.8 of Rust's rate | 0.6 / 0.9 = 0.67 | miss |
| Final pass at least Rust's | 9/10 against about 10/10 | miss |
| Iterations within 1.5× of Rust's | about 1.6× | miss |

All three misses come from the reference and one design gap, not from the
surface. Two things support that reading:

- **No Rust habit leaked** anywhere in 1,614 lines. There is no lifetime
  syntax, no ambient I/O, no `Arc<Mutex>`, no async, no macros, and no lossy
  `as`.
- **Origins were used correctly in every borrowing case.** That includes
  three cases the reference never documents:
  - lexer's enum `Token` with borrowed fields;
  - `fn text(&self) -> &str from *self`;
  - `from Self.src` inside a generic argument that is not the last one.

  The writer never saw `02-memory.md`, inferred all three from the
  reference's general rules, and matched 02 each time. That is good evidence
  for D2.

If the reference were synced with chapters 03 and 04 (changes R1–R4 below),
bsearch, fetch_all, and bank would very likely pass on the first check. The
estimated rate would then be 9/10, and 10/10 once D-1 adds append mode.

The pass rate also hides three costs:

- The writer avoided `Rand` and `Stdin` entirely, because their receivers
  are unspecified. That narrowed two designs: kv_log derives keys from loop
  counters, and wc does not read stdin.
- The writer attenuated less than possible in fetch_all, because `only`
  appeared to take only a literal list.
- The writer invented three APIs: `open_with`, `world.crash()`, and the
  `Faults` struct literal.

In return, the Ferox versions give things the Rust versions do not without
extra crates (cap-std, shuttle or loom, Prusti):

- every effectful function's authority is in its signature;
- app_errors and kv_log test their attenuation;
- the fetch_all and bank tests replay by seed;
- bsearch can be proved.

## 3. Per-program grading

### 3.1 wc — severity 0

**Verdict.** Passes check and tests.

**Errors.** None.

**Logic.** Correct.

- The five table cases and the simulated test's expected output check out
  by hand. Byte counts include multi-byte UTF-8, and each column is
  right-aligned to width 8.
- `in_word` carries word state across chunk boundaries, and the property
  test `chunking_does_not_change_counts` checks exactly that.
- Nit (ordinary): `is_ascii_whitespace` (line 40) omits vertical tab
  (0x0B), which POSIX `isspace` includes.
- By design, not a bug: there is no stdin mode, and the total row appears
  even for a single file. The task text asks for neither.

**Authority and contracts.**

- Least authority within the API: `main` attenuates to `read_only()`; `run`
  and `count_file` take `&Fs`; `count_bytes` is `#[pure]`.
- Attenuating to the working directory, as reference §15.1 does, would
  break absolute paths. 03 §12.1 makes the same choice the writer did.
- Collecting output into a `Report` so a simulated test can observe the
  failure flag is a sound workaround, since `main` cannot be called from
  tests.
- No contracts, and none needed.

**Iterations.** 1.

### 3.2 csv_line — severity 0

**Verdict.** Passes check and tests.

**Errors.** None. Each of the writer's guesses matches 02:

- `Cow<str>` counts as an elided origin position, like Rust's `S<'_>`
  (02 §2.5), and rule 1 binds it to `line`.
- Elided origins in `const` types are `'static` (02 §2.5).

**Logic.** Correct.

- All 14 table cases trace correctly, and RFC 4180 quoting is handled.
- Every slice index sits next to an ASCII byte, so it is always a char
  boundary; non-ASCII input cannot panic.
- The round-trip property is sound: any field survives encode-then-split,
  including commas, quotes, and newlines. The early return on empty input is
  right, because `""` is one empty field.
- An early `return` in a property test counts as a pass. The reference does
  not say whether such tests count toward a vacuity floor; 04 §2.4 defines a
  floor only for generated contract tests (see R9).

**Authority and contracts.**

- No authority, and `#[pure]` on the public function is good practice.
- No contracts. A cheap, meaningful one was available:
  `ensures match result { Ok(v) => v.len() >= 1, Err(_) => true }`.

**Iterations.** 1.

### 3.3 lexer — severity 0

**Verdict.** Passes check and tests.

**Errors.** None. The writer settled three points the reference does not
cover, each as 02 decides them:

- `enum Token { Ident(&str), Int(&str), Punct(&str) }` has three origins,
  named `Ident.0`, `Int.0`, and `Punct.0` (02 §3.1–3.2). `Token from
  Self.src` and elision bind all of them, because `from` "applies to every
  elided origin position" (02 §2.3).
- `type Item = Result<Token from Self.src, LexError>` parses as intended.
  Outside the tail of a return type, `from` takes a single origin, and the
  comma ends it (02 §2.3 grammar).
- `fn text(&self) -> &str from *self` names the token's origins, not the
  `&self` borrow (02 §2.2, the `*words` row). `tokens_outlive_the_lexer` and
  `prefers_two_char_operators` depend on this, and both are accepted.

**Logic.** Correct.

- All tests trace correctly; `position()` after `"a $ b"` is 5.
- The coverage property holds: tokens never contain whitespace, each error
  consumes exactly one non-whitespace char, and `trim_start` agrees with
  `char::is_whitespace`.
- A design choice, not a bug: `42abc` lexes as `Int("42")` followed by
  `Ident("abc")`.

**Authority and contracts.** No authority. No contracts; the coverage
property test does that job.

**Iterations.** 1.

### 3.4 bsearch — severity 2

**Verdict.** Fails check with two errors at 11 sites. After those are fixed,
it fails `ferox verify` until the proofs call the `Ord` laws.

| # | Line / snippet | Class | Compiler would say | Fix |
| --- | --- | --- | --- | --- |
| S1 | 14 `spec fn sorted`, used in the `requires` of `pub fn binary_search` (22) and `pub fn contains` (62) | Reference gap. Reference §15.3 shows exactly this pattern, and 04 §1.4 forbids it. | `error[FX08xx]: private spec fn 'sorted' in the contract of public fn 'binary_search'`, with `note: a public contract may mention only public spec fns` and `help (safe): pub spec fn sorted` | `pub spec fn sorted` |
| S2 | `.cmp(` inside specs at lines 15, 24, 26, 27, 34, 35, 43, 49, 63 | Reference gap (04 §1.6, §5.1) | `error[FX08xx]: spec calls exec function 'Ord::cmp'`, with `note: trait methods cannot be #[spec]; Ord pairs cmp with the spec fn cmp_spec` and `help: xs[i].cmp_spec(&xs[j])` | Use `cmp_spec` in every clause and proof. The exec call on line 39 keeps `cmp`. |
| S3 | Proof blocks at 42–44 and 48–50 | Reference gap (04 §5.3: laws are not ambient) | `verify: binary_search failed proof_incomplete at 43:17`, with `hint: missing lemma; call T::transitive, T::reverse` | Call the laws as lemmas (below). |

The verify diagnostic in S3 should also name the missing laws. 04 §1.6
gives the S2 error two fixes, "declare a spec fn" or "mark it `#[spec]`".
Neither applies to a trait method. The diagnostic should offer the paired
spec fn instead.

**Contingent.** C7: if 04 §1.5 rule 3 is applied literally, `sorted<T: Ord>`
also needs `T: Pure` (see D-4).

The corrected loop body, after S1 and S2:

```ferox
match xs[mid].cmp(target) {
    Ordering::Less => {
        proof {
            T::reverse(&xs[mid], target);                // target.cmp_spec(xs[mid]) == Greater
            assert forall |j: usize| j < mid ==> xs[j].cmp_spec(target) == Ordering::Less by {
                if j < mid && xs[j].cmp_spec(target) != Ordering::Less {
                    T::reverse(&xs[j], target);           // le(target, xs[j])
                    T::transitive(target, &xs[j], &xs[mid]);   // le(target, xs[mid]): contradiction
                }
            };
        }
        lo = mid + 1;
    }
    Ordering::Greater => {
        proof {
            assert forall |j: usize| mid < j && j < xs.len() ==> xs[j].cmp_spec(target) == Ordering::Greater by {
                if mid < j && j < xs.len() && xs[j].cmp_spec(target) != Ordering::Greater {
                    T::transitive(&xs[mid], &xs[j], target);   // le(xs[mid], target): contradiction
                }
            };
        }
        hi = mid;
    }
    Ordering::Equal => return Ok(mid),
}
```

**Logic.** Correct.

- The contract matches `slice::binary_search`: `Err(i)` is the unique
  insertion point.
- The invariants are right, and they suffice once the laws are invoked.
- `lo + (hi - lo) / 2` cannot overflow, and `decreases hi - lo` is right
  (04 §3.3 would also infer it).
- The string tests trace correctly: `grape` gives `Err(2)`, `aardvark`
  `Err(0)`, and `zucchini` `Err(4)`.
- The writer's worry about `==` versus `cmp` in specs is answered by
  04 §5.1. `agrees_with_eq` ties `cmp_spec == Equal` to view equality, and a
  type with coarse equality must normalize its `View`. Either spelling is
  correct.

**Authority and contracts.**

- No authority.
- The contracts are meaningful and strong: the postcondition fixes the
  result up to the choice among equal elements, and `contains` is exact and
  provable from `binary_search`'s contract alone. Neither is vacuous, and
  04 §10's satisfiability and non-triviality checks pass.

**Iterations.** 2 check runs, then 2 verify runs: about 4.

### 3.5 fetch_all — severity 2

**Verdict.** Fails check with two errors. After those, two tests fail,
because a passed scope deadline fails the whole scope.

| # | Line / snippet | Class | Compiler or test would say | Fix |
| --- | --- | --- | --- | --- |
| F1 | 7 `use std::sync::{Poisoned, Shared}`; 24 `Poisoned(#[from] Poisoned)` | Reference gap. Reference §7.3 says `lock` returns `Result<R, Poisoned>` and that aliased handles panic; 01 D8 and 03 LOCK-2 and LOCK-5 say `Result<R, LockError>`. | `error: unresolved import 'std::sync::Poisoned'`, with `note: lock evaluates to Result<R, LockError>; Poisoned is a variant of LockError` | `use std::sync::{LockError, Shared};` and `Lock(#[from] LockError)` |
| F2 | 144 `sim.world.faults(Faults { net_drop: 0.5, ..Faults::none() })`; 116 imports `Faults` | Reference gap. Reference §6.8 contradicts 03 §9.2 and §9.4. | `error: no method named 'faults' found for 'World'`, with `help: sim.faults.net_drop(0.5)` | `sim.faults.net_drop(0.5);` and drop `Faults` from the import, which `FX0913` would otherwise reject as unused |
| F3 | 38 `s.set_deadline(deadline)`, with the tests at 149 and 168 | Reference gap. Reference §7.1 and §15.2 imply that a deadline is not a failure; 03 TASK-4 and §12.2 say `scope` returns `Err(DeadlineExceeded)`. | `test everything_fails_by_the_deadline_when_the_network_is_down FAILED: called Result::unwrap() on Err(Task(DeadlineExceeded))`. `half_the_packets_dropped` fails the same way on any seed where a request is still running at 10 s. | Remove the scope deadline and rely on the per-operation deadline in `net.get`. Bound the backoff with `clock.sleep_until(min(clock.now() + backoff, deadline))` and stop retrying at the deadline; otherwise the deadline test fails by up to one backoff (200 ms). |

F3 is also a logic bug against the program's own specification ("prints one
result per URL"). Under 03, any run that reaches the deadline prints only
`Error: Task(DeadlineExceeded)` and exits 1, discarding the results it had
already fetched.

**Contingent.** C1 (the `LockError` path, after F1).

**Logic, apart from F3.**

- Correct: at most four requests in flight, through `min(4, n)` workers that
  claim indices from a `Shared<usize>`; results go back in input order.
- The `expect` is justified: every index below `n` is claimed exactly once,
  and every claim pushes a result. An empty input spawns no workers.
- Minor (ordinary, caused by a reference gap): every error except a timeout
  or cancellation is retried, including permanent ones such as an `only`
  denial or a 4xx status. That costs up to 300 ms per bad URL. The reference
  gives no `NetError` taxonomy and no `is_transient()`.
- Simplification: 03 §1.1 offers `tasks.limit(n)` ("live children"). With
  `root.tasks.limit(4)` and one spawn per URL, the worker pool and the
  shared counter disappear, provided `spawn` waits for a free slot, which
  03 does not say.
- The test's timing bounds (2 s ≤ elapsed < 3 s) assume small
  latencies per operation. 03 §9.3 draws latencies from the seed; at
  millisecond scale the bounds hold.

**Authority and contracts.**

- `fetch_all`, `worker`, and `fetch_one` each take exactly what they use.
- `main` passes `root.net` unattenuated. Least authority would be
  `root.net.only(<hosts of the URLs>)`, but the reference shows `only` taking
  a literal array and documents no `Url` host accessors (R8). A reviewer
  should ask for it once those are documented.
- No contracts, none expected.

**Iterations.** 2 check runs (F1 and F2 together, then confirmation), then 2
test runs (F3, then the backoff overrun): about 4.

### 3.6 bank — severity 1

**Verdict.** Fails check with one error. The fix is two lines.

| # | Line / snippet | Class | Compiler would say | Fix |
| --- | --- | --- | --- | --- |
| B1 | 7 `use std::sync::{Poisoned, Shared}`; 23 `Poisoned(#[from] Poisoned)` | Reference gap (as F1) | `error: unresolved import 'std::sync::Poisoned'`, with the same note as F1 | `use std::sync::{LockError, Shared};` and `Lock(#[from] LockError)` |

**Logic.** Correct.

- The overdraft check happens inside the lock, and the error returns from
  the body before any write, so a transfer moves everything or nothing.
- `Bank::new` rejects totals above `u64::MAX`, so a credit can never
  overflow.
- The same-account refusal comes before `lock`. That was required under
  the reference's panic semantics. Under 03 LOCK-2, `lock` returns
  `Err(Aliased)`, so the check is now just a nicer error. 03 §12.4 does the
  same with `Shared::same`.
- The concurrent property test is meaningful: money is conserved over
  seed-chosen interleavings, and an overdraft bug would trap on `u64`
  underflow and surface as `TaskError::Panicked`.
- Nit: `transfer(9, 9, _)` reports `SameAccount` before `NoSuchAccount`.
- `total()` is not a consistent snapshot while transfers run, and the
  writer documents this. 03 LOCK-8's `sync::lock_all` would make it exact.

**Authority and contracts.**

- No capability is needed, and `Shared::new` takes none.
- No contracts. `Shared` payloads have opaque views (04 §4.4), so `Bank`'s
  methods cannot state balances in a contract; the property test is the
  right tool.

**Iterations.** 2.

### 3.7 bfs — severity 0

**Verdict.** Passes check and tests, provided C2 holds.

**Naming.** The reference calls the arena handle `Handle<T>`; 02 §10.9
calls it `Id<T>`. See D-7.

**Logic.** Correct.

- This is standard BFS with a parent map, so the path is shortest by edge
  count. A path from a node to itself is one node.
- Stale edges are skipped, and a stale start or goal returns `None`.
- All four tests trace correctly (for example, b to d gives b, c, e, a, d).

**Authority and contracts.** No authority, no contracts.

**Iterations.** 1.

### 3.8 kv_log — severity 3

**Verdict.** Fails check. It is blocked by a design gap (L1). Its crash test
must also be rebuilt around simulated nodes.

| # | Line / snippet | Class | Compiler would say | Fix |
| --- | --- | --- | --- | --- |
| L1 | 128 `dir.open_with(LOG_FILE, OpenOptions::new().create(true).append(true))`, and the `OpenOptions` import at 14 | **Design gap.** No chapter gives a way to append. 03 §1.1 lists `open` and `create`, which are Rust's read-only and truncating opens, and Rust's `OpenOptions::open` is ambient (`FX0501`). | `error: no method named 'open_with' found for '&Fs'`, with `help: Fs opens files with open (read) or create (truncate)` | None in the current design; see D-1 and R5. The only normative durable write, `Fs::write_synced` (03 §12.3), rewrites a whole file, so the task's append-only log cannot be built. |
| L2 | 272 `sim.world.faults(Faults { disk_error: 0.05, ..Faults::none() })` | Reference gap (as F2) | `error: no method named 'faults' found for 'World'` | `sim.faults.disk_error("/data", 0.05, io::ErrorKind::Other);` |
| L3 | 304 `sim.world.crash()` | Reference gap. 03 §9.4–9.5 crash and restart *nodes*; the reference has no crash at all. | `error: no method named 'crash' found for 'World'`, with `note: crashes apply to nodes: sim.faults.crash(&node)` | Run the store inside `sim.node("kv").start(..)` and log each acknowledgement with `info!`. Crash with `sim.faults.at(t, Fault::crash(&node))`, then reopen through `node.fs()` and check every acknowledged key from `sim.records()`. Sketch under R2. |
| L4 | 128, 136: no directory sync after `create(true)` makes the log | **Design gap.** 03 §9.5 says "only synced file data survives" but not whether a never-synced directory entry does, and std has no `sync_dir`. | none (review finding) | `dir.sync_dir(".")?` after the first create (proposed in D-1). On POSIX filesystems, a fresh store's first acknowledged writes can vanish with the directory entry after a power loss. |

**Contingent.**

- C1: `std::sim::SimRoot` (03 ROOT-3 makes `std::sim` likely).
- C6: `File::set_len` and `File::sync_data`. 03 §1.1 lists only `sync_all`;
  `sync_all` would do in both places.

**Logic, apart from the errors.** Careful and correct.

- The record format carries a length and an FNV-1a checksum, computed with
  wrapping arithmetic as reference §4.1 requires.
- `decode` is total: all slicing goes through `get`, with checked additions.
- Replay stops at the first invalid record, and `open` truncates that tail
  before appending.
- A write is acknowledged only after `write_all` and `sync_data`.
- After a failed append, the `broken` flag refuses further writes until the
  store is reopened. That prevents appending after a torn record.
- The tests cover a torn tail, a flipped bit, a clean restart, and a
  crash-and-restart loop that accounts correctly for the one write in doubt.
- A limitation the writer documents: a corrupt record in the middle of the
  log truncates everything after it, including acknowledged records.

**Authority and contracts.**

- Exemplary authority. `main` uses `root.fs` only for `create_dir_all`, then
  hands `KvStore` an `Fs` attenuated to the data directory. `encode`,
  `decode`, and `replay` are authority-free.
- No contracts. A `decode(encode(k, v))` round-trip property test is the
  natural missing check.

**Iterations.** Blocked. With L1 resolved: about 3 check runs and 1–2 test
runs.

### 3.9 word_freq — severity 1

**Verdict.** Passes check. One generated law test fails, and only under the
`test` contract mode.

| # | Line / snippet | Class | What fails | Fix |
| --- | --- | --- | --- | --- |
| W1 | 47 `*out.entry(word.clone()).or_insert(0) += n` in `Combine::combine` | Ordinary bug | The comment at lines 53–54 relies on `test` mode, which property-tests `impl Monoid for Counts` on generated values (04 §2.4, §5.3). Counts near `u64::MAX` make `combine` trap, and the shrunk input is `a = {"": 18446744073709551615}`, `b = {"": 1}`, giving `law Monoid::associative failed: attempt to add with overflow at 47:13`. Under the default `assert` mode the laws are only assumed. | `*slot = slot.saturating_add(*n)`. Saturating addition of unsigned integers is associative with identity 0. |

**Logic, apart from W1.** Correct.

- Words are maximal alphanumeric runs, lowercased.
- `top` sorts by count descending, then by word ascending.
- All tests trace correctly.
- `word_freq(&[])` returning `Counts::empty()` is right by 03 §8.3.

**Authority and contracts.**

- `main` attenuates with `read_only()`; reading is sequential and
  authority-bearing, and counting is authority-free in `par_map`.
- `impl Monoid for Counts {}` is meaningful: it is what makes `par_reduce`
  equal the sequential fold.
- The writer's design complaint is confirmed: `Combine::combine(&self,
  &Self)` forces a full map clone at every merge (D-5).

**Iterations.** 1 check run, then 1 test run: 2.

### 3.10 app_errors — severity 0

**Verdict.** Passes check. The tests pass, provided C3–C5 hold.

**Logic.** Correct.

- The temporary file plus `rename` means readers never see a partial
  document.
- It is not crash-durable, since there is no `sync_all` before `rename`,
  but the task did not ask for durability.

**Authority and contracts.**

- Exemplary. `main` attenuates `Net` to one host and `Fs` to one directory
  before anything else runs, never passes `Root`, and borrows only fields.
- The tests check both attenuations: another host is unreachable, and `..`
  cannot escape the directory, which 03 §1.1's `RESOLVE_BENEATH` makes an
  error.
- Choosing `main -> ExitCode` so the `source()` chain is logged with
  `Display` is reasonable. 03 §1.3 confirms that `main -> Result` prints
  only `Error: {e:?}`.
- No contracts. Errors are exercised through `?` conversion tests.

**Iterations.** 1.

## 4. Where `reference.md` disagrees with the normative chapters

The reference says it derives from 01 and that 01 wins on conflict. These
are the disagreements found while grading. The first five caused errors in
this trial; the sixth shaped the programs' designs.

| # | Topic | `reference.md` | Normative text | Effect here |
| --- | --- | --- | --- | --- |
| 1 | `lock` result and aliased handles | §7.3: `Result<R, Poisoned>`; aliasing panics | 01 D8 and 03 LOCK-2, LOCK-5: `Result<R, LockError>`; aliasing returns `Err(LockError::Aliased)`. 02 §8.4 still says "panics" (03 §13.2 asks for that to change). | F1, B1 |
| 2 | Scope deadline outcome | §7.1, and the §15.2 test that unwraps after a deadline | 03 TASK-4 and §12.2: `Err(TaskError::DeadlineExceeded)`; `TaskError` has three variants | F3 |
| 3 | Simulator API | §6.8: `sim.world.faults(Faults { .. })`; no crash, nodes, `#[sim_test]`, or `sim.records()` | 03 §9.2–9.5: `sim.faults.net_drop(p)`, `disk_error(prefix, p, kind)`, `torn_writes`, `crash(&node)` and `restart(&node)`, `sim.node`, `run_for`, `run_until`, `node.fs()`, `#[sim_test(seeds = N)]` | F2, L2, L3 |
| 4 | A private spec fn in a public contract | §15.3's example does it | 04 §1.4 forbids it | S1 |
| 5 | Trait methods in specs; which laws `Ord` has | §10.2 is silent; §5.3 says only "order consistent with Eq" | 04 §1.6, §5.1, §5.3: trait methods are not callable in specs; `cmp_spec`, `le`; laws `reverse`, `transitive`, `agrees_with_eq`, called as lemmas | S2, S3 |
| 6 | Capability and handle surface | §6.1 lacks several items | 03 §1.1–1.2 adds `Fs::write_synced`, the `File` operations, `Tasks::limit`, `shield`, `checkpoint`, `Rand::rng()`, `Clock::monotonic_only`, `Env::args_only`, `Net::no_bind`, `Shared::same`, `sync::lock_all`, and the `RESOLVE_BENEATH` path rules | fetch_all's design; notes gaps |
| 7 | Closure specs | §10.2: `f.requires((args,))` | 04 §6: `f.requires(a, b)`; the tuple form is rejected | none |
| 8 | Nonlinear proof blocks | `by(nonlinear)` | 04 §1.2: `by(nonlinear_arith)` | none |
| 9 | Choosing a contract use | §12.4: `contracts = "test"` under `[package]` | 04 §2.1: `[contracts] use = ".."`, plus a module-level `#![contracts(..)]` | none |
| 10 | `for` loops under `prove` | §10.3: loops over ranges and slices infer `decreases` | 04 §3.3: under `prove`, `for` may iterate only over integer ranges | none |
| 11 | Lazy statics | §6.4: no `LazyLock` or `OnceLock` | 03 GLOBAL-2: `static T: Lazy<..>` is allowed with an authority-free initializer | none |
| 12 | Borrowing `impl Trait` returns | §3.5: without `from` is `FX0204` | 02 §5.2, §10.1: elided `&` positions in the bounds capture, so `-> impl Iterator<Item = &Person>` is accepted | none |
| 13 | One expression per clause | not mentioned | 04 §1.2 rule 1 | none |
| 14 | `fn new(src: &str) -> Self` in a borrowing type's impl | not mentioned | 02 §3.6: an error | none (lexer used `-> Lexer`) |
| 15 | Handle names | arena `Handle<T>`; task `Task<U, E>` | 02 §10.9 arena `Id<T>`; 02 §8.2 task `Handle<T>`; 03 task `Task<U, E>` | none |
| 16 | `#[pure]` | §6.6: provisional | 03 §3.2: settled, with `FX0511` | none |

Within the normative set there are two further conflicts:

- C7: 04 §1.5 rule 3 and 03 §3.5 require `Pure` parameters on spec fns, and
  04's own generic examples omit the bound.
- 02 §8.4 contradicts 01 D8 on aliased `lock` operands.

## 5. Changes to `reference.md`, prioritized

Order: definite errors prevented first, then the number of notes gaps
closed. Most items copy decisions chapters 02–04 already make: 39 of the 55
gaps in `notes.md` have an answer in a normative chapter that the reference
has not caught up with (see §7).

### R1. §7.1–7.3 and §15.2: tasks and `lock` as chapter 03 fixes them

Prevents F1, B1, and F3. Closes notes fetch_all 1–4 and bank 1–3 and 5.

Missing facts:

- `lock`'s result type and its aliasing behavior;
- `Shared::same` and `sync::lock_all`;
- `TaskError::DeadlineExceeded`, and what a passed deadline does;
- `join` is not a checkpoint;
- `Scope<E>` is passed by value;
- `Tasks::limit(n)`, including whether `spawn` waits for a free slot (03
  does not say yet; the text below proposes that it does).

Proposed text, replacing rows of the §7.1 table:

```
| `Tasks::scope` | `fn scope<T, E: From<TaskError> + Send>(&self, body: impl FnOnce(Scope<E>) -> Result<T, E>) -> Result<T, E>`; `Scope<E>` is `Copy` |
| `Tasks::limit` | `fn limit(&self, n: usize) -> Tasks`: attenuation; scopes from it run at most `n` children at once, and further `spawn`s wait for a slot |
| `TaskError` | `enum TaskError { Panicked(String), Cancelled, DeadlineExceeded }` |
```

Add to the §7.1 semantics:

> - **A scope deadline is a failure.** When it passes, the scope is
>   cancelled and `scope` returns `Err(TaskError::DeadlineExceeded.into())`,
>   even if the body returns `Ok`; values already joined are dropped. To get
>   one outcome per item within a time budget, give each operation the
>   deadline (`net.get(url, deadline)`, `clock.sleep_until(t)`) and set no
>   scope deadline.
> - `join` is not a checkpoint. It waits for the child and returns its `Ok`
>   value, or `Err(TaskError::Cancelled.into())` if the child failed; the
>   scope reports the actual failure.
> - To bound concurrency, attenuate: `let tasks = root.tasks.limit(4);`.

Replace the §7.3 bullets on the result and on aliasing with:

> - `lock (c1, .., cn) |x1, .., xn| body` evaluates to `Result<R, LockError>`
>   (`use std::sync::{LockError, Shared};`). `LockError` is `Aliased`, when
>   two operands name the same cell and the body does not run, or
>   `Poisoned`. Test identity first with `Shared::same(&a, &b)`.
> - Operands are any expressions of type `&Shared<T>`. One cell:
>   `let b = lock (acct) |a| { a.balance }?;`. Without braces the body
>   extends as far as a closure body would, so write
>   `(lock (acct) |a| a.balance)?`. A dynamic set:
>   `sync::lock_all(&cells, |xs: &mut [&mut T]| ..)`.

Fix the §7.3 example comment (`#[from] LockError`), the §14 rows for
`FX0604` and `FX0606`, and §16 items 15 and 17.

In §15.2, remove `s.set_deadline(deadline)`; `fetch_one` already turns
`NetError::Timeout` into `Outcome::TimedOut`, so the test stands. Then point
to 03 §12.2 for the failing-deadline variant.

### R2. §6.8: the simulator as chapter 03 §9 fixes it

Prevents F2, L2, and L3. Closes notes kv_log 3, fetch_all 6, bank 4, and
wc 4.

Proposed replacement for the §6.8 table:

```
use std::sim::{SimRoot, NodeRoot, Reply, Fault};     // test code only (FX0504)

| Item | Effect |
| `#[sim_test(seeds = N)] fn t(sim: SimRoot)` | runs the test once per seed; `Root::sim(seed)` builds one by hand |
| `sim.world.file(path, s)`, `args([..])`, `env_var(k, v)`, `stdin(b)`, `http("host:port", f)` | world setup; `Reply::ok(body).after(d)`, `Reply::status(code)`, `Reply::hang()` |
| `sim.faults.net_drop(p)`, `net_delay(min, max)`, `net_duplicate(p)`, `partition(&[&a], &[&b])`, `heal()` | network faults |
| `sim.faults.disk_error(path_prefix, p, io::ErrorKind::Other)`, `torn_writes(true)` | disk faults |
| `sim.node("kv")`, `node.start(entry)`, `sim.faults.crash(&node)`, `restart(&node)`, `sim.faults.at(t, Fault::..)` | nodes, crash and restart; `entry: FnOnce(NodeRoot) -> Result<(), E>` captures only `Frozen + Pure` data |
| `sim.run_for(d)`, `sim.run_until(&h)`, `node.fs()`, `sim.elapsed()`, `sim.records()`, `sim.trace()` | drive and inspect |
```

Add these facts:

- **Crashes.** A crash stops the node without running destructors. Only
  data made durable by `File::sync_all` or `Fs::write_synced` survives;
  with `torn_writes(true)`, so does a seed-chosen prefix of unsynced writes.
  Directory entries follow D-1 and D-2.
- **Network faults.** Under `net_drop`, an operation is delayed and returns
  `NetError::Timeout` at its deadline.
- **Time and order.** Operations take latencies drawn from the seed, and
  computation takes no time. The seed orders every capability operation,
  `spawn`, `join`, channel operation, and `lock` acquisition.
- **Errors.** Simulated `io::Error`s have the host's `kind()` and message.

Add one example, a crash test, so agents see the node pattern:

```ferox
#[sim_test(seeds = 200)]
fn acked_writes_survive_a_crash(sim: SimRoot) {
    sim.faults.disk_error("/data", 0.05, io::ErrorKind::Other);
    sim.faults.torn_writes(true);
    let node = sim.node("kv");
    node.start(|r| {
        r.fs.create_dir_all("/data")?;
        let mut store = KvStore::open(&r.fs.dir("/data", Mode::ReadWrite)?)?;
        for i in 0..50 {
            let (k, v) = (format!("k{}", i % 7), format!("v{i}"));
            if store.put(&k, &v).is_ok() { info!("acked {k}={v}"); }
        }
        Ok::<(), KvError>(())
    });
    sim.faults.at(Duration::from_millis(30), Fault::crash(&node));
    sim.run_for(Duration::from_secs(1));
    let store = KvStore::open(&node.fs().dir("/data", Mode::ReadWrite).unwrap()).unwrap();
    for (k, v) in last_acked(&sim.records()) {          // parses the "acked" records
        assert_eq!(store.get(&k), Some(v.as_str()));
    }
}
```

### R3. §10.2 and §15.3: specs over generic types

Prevents S1 and S2. Closes notes bsearch 1, 2, and 6.

Proposed text for §10.2:

> - **Visibility.** A `pub` item's contract may mention only `pub` spec
>   fns, by Rust's private-in-public rule.
> - **Trait methods.** Specs cannot call trait methods: they are exec code
>   and cannot be `#[spec]`. Std traits pair each method with a spec fn:
>   `a.eq_spec(&b)` for `==`, `a.cmp_spec(&b)` for `cmp`, and
>   `le(&a, &b)` for `<=` on `T: Ord`. On primitive types, `<`, `<=`, and
>   `==` in specs mean what they mean in code.
> - **Equality.** `==` in specs is view equality. The `Eq` and `Ord` laws
>   tie `eq_spec` and `cmp_spec == Equal` to it, so a type whose
>   `PartialEq` ignores part of its structure, such as case-insensitive keys,
>   must define a normalizing `View`.
> - **Generic spec fns** are written as expected:
>   `pub spec fn sorted<T: Ord>(xs: &[T]) -> bool`. (Bound per D-4.)
> - **Quantifiers.** A quantifier body extends as far right as possible:
>   write `result == (exists |i: usize| ..)`.
> - **Clauses.** Each clause holds one expression; split `requires a, b`
>   into two clauses.

In §15.3, change `spec fn sorted` to `pub spec fn sorted`, and add the
corrected generic `binary_search` from §3.4 above as a second example.

### R4. §5.3 and §10.2: laws, and how proofs use them

Prevents S3. Closes notes bsearch 3, 4, and 6, and word_freq 3 (laws and
model `==`).

Proposed text:

> Std laws (from `04-contracts.md` §5.1):
>
> - `Eq::eq_is_view_eq(a, b)`: `a.eq_spec(b) <==> a@ == b@`.
> - `Ord::reverse(a, b)`: `a.cmp_spec(b) == b.cmp_spec(a).reverse()`.
> - `Ord::transitive(a, b, c)`: requires `le(a, b) && le(b, c)`, ensures
>   `le(a, c)`.
> - `Ord::agrees_with_eq(a, b)`: `cmp_spec == Equal <==> a@ == b@`.
> - `Hash`: equal views feed the hasher equal bytes.
> - `Clone::clone` ensures `result@ == self@`.
>
> **Laws are lemmas, not ambient facts.** The solver knows a law only where
> a proof calls it: `proof { T::transitive(&a, &b, &c); }`. To prove a
> quantified fact, write
> `assert forall |j: usize| P(j) ==> Q(j) by { ..lemma calls.. };`. Proof
> blocks may contain `if` and `match`.
>
> A call through a bound (`x.cmp(y)` with `T: Ord`) uses the trait's
> contract. The impl's panic-freedom and laws are settled at each concrete
> instantiation, and the caller's status shows them (`proved, assuming 2`).
>
> Under `test`, a hand-written impl's laws are property-tested on generated
> values, so its arithmetic must not trap on them (see R9).

### R5. §6.1 and §6.3: the `Fs` and `File` surface

Prevents L1 once D-1 decides it. Resolves C5 and C6. Closes notes wc 1, 2,
and 5, kv_log 1, 2, and part of 4, and app_errors 3 and 4.

Proposed signature block (names from 03 §1.1; the lines marked "D-1" need
that decision):

```ferox
impl Fs {
    fn read(&self, p: impl AsRef<Path>) -> io::Result<Vec<u8>>;
    fn read_to_string(&self, p) -> io::Result<String>;
    fn write(&self, p, data: impl AsRef<[u8]>) -> io::Result<()>;          // create or truncate
    fn write_synced(&self, p, data) -> io::Result<()>;                      // durable when it returns
    fn open(&self, p) -> io::Result<File>;                                  // read only
    fn create(&self, p) -> io::Result<File>;                                // write; create or truncate
    fn open_with(&self, p, opts: &OpenOptions) -> io::Result<File>;         // D-1: append, create_new, ..
    fn sync_dir(&self, p) -> io::Result<()>;                                // D-1: make directory entries durable
    fn create_dir_all(&self, p) -> io::Result<()>;
    fn exists(&self, p) -> io::Result<bool>;
    // read_dir, metadata, rename, remove_file: as std::fs
    fn dir(&self, p, mode: Mode) -> io::Result<Fs>;     // fails if p is missing or not a directory
    fn read_only(&self) -> Fs;                          // cannot fail
}
impl File { /* io::Read, io::Write, io::Seek; sync_all, close; D-1: sync_data, set_len */ }
```

Add these facts:

- **Paths.** An attenuated `Fs` resolves paths beneath its directory. An
  absolute path, or a `..` that would leave the directory, fails with
  `PermissionDenied`. `root.fs`, and `read_only()` taken from it, accept
  absolute paths and resolve relative ones against the starting working
  directory.
- **`OpenOptions`.** `OpenOptions` is plain data; its Rust `open` method is
  ambient (`FX0501`), so pass the options to `fs.open_with`.
- **Authority of a `File`.** A `File` carries the authority of the `Fs`
  that opened it. Its methods need no other capability, and exercised
  authority attributes them to that `Fs`.
- **Return types.** `dir` returns `io::Result` and `read_only` and `only` do
  not, because only `dir` can fail.

### R6. §12.3: one import table

Resolves C1. Closes notes cross-cutting 2.

| Items | Path |
| --- | --- |
| `Fs`, `Mode`, `File`, `OpenOptions` | `std::fs` |
| `Net`, `NetError`, `Url`, `UrlError` | `std::net` |
| `Clock`, `Duration`, `Instant`, `SystemTime` | `std::time` |
| `Tasks`, `Scope`, `Task`, `TaskError`, `Cancelled` | `std::task` |
| `Shared`, `LockError`, `lock_all`, `mpsc` | `std::sync` |
| `SimRoot`, `NodeRoot`, `Reply`, `Fault` | `std::sim` (test code only) |
| `Arena`, `Handle` | `std::arena` |
| `Combine`, `Monoid`, `Sum`, `Product`, `Min`, `Max`, `join` | `std::par` |
| `Pcg64`; `RandomState` | `std::rand`; `std::collections::hash_map` |
| `ExitCode`; `Cow` | `std::process`; `std::borrow` |
| `le`, `Seq`, `Set`, `Map` | `std::spec` (proposed) |

### R7. §3.3: origins in enums, generic arguments, `*self`, std types, and consts

Closes notes csv 1–2, lexer 1–4, and kv_log 4 (the `Op` enum). The writer
guessed all of these right, but only by inference.

Proposed text:

> - **Enums** work like structs. Each borrowed position in each variant is
>   an origin, named `Variant.field` or `Variant.0`, so
>   `enum Token { Ident(&str), Int(&str) }` has two origins. Elision and
>   `-> Token from x` bind all of them. Share one origin with
>   `Int(&str from Ident.0)`.
> - **Inside generic arguments**, `from` takes one origin, and a comma ends
>   it: `Result<Token from Self.src, LexError>`. Several origins:
>   `Vec<&str from (a, b)>`.
> - **Returning what `self` borrows.** `fn text(&self) -> &str from *self`
>   borrows the token's source; plain `-> &str` borrows the `&self`
>   reference.
> - **Std borrowing types** are written without lifetimes: `Cow<str>`,
>   `Chars`, `std::slice::Iter<T>`. Elision treats them as it treats `&str`.
> - **Consts and statics.** Elided origins are `'static`:
>   `const CASES: [(&str, &[&str]); 2]`.
> - **Trait impls.** `fn next(&mut self) -> Option<Self::Item>` is accepted
>   in an impl.

### R8. §6.1: capability calling conventions and `Net`

Resolves C3. Closes notes cross-cutting 3 and 5, fetch_all 5 and 7, and
app_errors 2 and 6.

Proposed text:

> - **Receivers.** Every capability method takes `&self`: `rand.gen_range`,
>   `stdin.read_line(&mut s)`. For many draws, take `let mut rng =
>   rand.rng();` once and pass the `Pcg64`.
> - **Send and Sync.** Capabilities, `Shared<T>`, and `Scope<E>` are
>   `Send + Sync` (02 §8.1), so children may borrow them.
> - **`get`.** `Net::get(&self, &Url, deadline) -> Result<Vec<u8>,
>   NetError>` returns `Err(NetError::Status(code))` for a non-2xx response.
> - **`NetError`** has the variants `Timeout`, `Cancelled`, `Denied` (a host
>   outside `only`), `Status(u16)`, `Refused`, `Reset`, `Dns`, and
>   `Io(io::Error)`. `is_cancelled()` and `is_transient()` classify them.
> - **`only`** takes any iterable of `"host:port"` strings, so `main` can
>   attenuate to hosts computed from its arguments:
>   `root.net.only(urls.iter().map(Url::authority))`.
> - **`Url::parse`** returns `Result<Url, UrlError>`. `Url` has the `url`
>   crate's accessors (`host_str`, `port_or_known_default`).

### R9. §6.8 and §5.3: property tests and law tests

Prevents W1. Closes notes cross-cutting 4 and word_freq 3.

Proposed text:

> - **Property tests.** A `#[test]` function with parameters is a property
>   test.
> - **Generated types.** Integers, floats, `bool`, `char`, `String`,
>   `Vec`, arrays, tuples, `Option`, `Result`, maps, sets, and any type whose
>   fields can all be generated, or that derives `Arbitrary`.
> - **Seeds.** A `seed: u64` parameter is an ordinary generated input that
>   combines with the others; for one fresh simulator per seed, use
>   `#[sim_test(seeds = N)]`.
> - **Filtering.** A `requires` clause on a test filters and guides
>   generation, and a run with fewer than 100 accepted cases fails as
>   `vacuous`. An early `return` counts as a pass, so prefer `requires`.
> - **Law tests** run on arbitrary values, including integers near their
>   maximum. Use `saturating_*` or `wrapping_*` in a law-bearing impl, or the
>   law test traps.

### R10. §2.2: `#[from]` and the std error types

Resolves C4. Closes notes app_errors 1.

> `#[from]` implies `#[source]`, as in thiserror. `io::Error`, `NetError`,
> `TaskError`, `LockError`, `UrlError`, and `json::Error` implement
> `Error + Send + Sync`.

### R11. §3.7: `Arena` handles

Resolves C2. Closes notes bfs 1–2.

> `Handle<T>` is `Copy`, `Eq`, `Ord`, `Hash`, and `Debug`. `Arena` also has
> `contains(h) -> bool`, `iter() -> impl Iterator<Item = (Handle<T>, &T)>`,
> and `get_disjoint_mut([a, b])`. Settle the name against 02's `Id<T>`
> (D-7).

### R12. §8: parallelism details

Closes notes word_freq 1–2.

> - `par_map`, `par_for`, `par_chunks_mut`, `par_reduce`, and `par_sort`
>   are inherent slice methods and need no import.
> - `par_reduce()` on an empty slice returns `M::empty()`.
> - Shared captures must be `Frozen + Sync`.
> - A checked integer `Sum` can trap on a partial sum that the sequential
>   fold would not reach (03 §8.3).

### R13. Remaining syncs

These caused no error in this trial. They are rows 7–14 and 16 of the §4
table, plus what `main` prints after `Err(e)` (03 §1.3):

- `f.requires(a, b)`;
- `by(nonlinear_arith)`;
- `[contracts] use`;
- `for` loops under `prove`;
- lazy statics (GLOBAL-2);
- the capture rule for `impl Trait` returns;
- one expression per clause;
- `-> Self` in borrowing impls;
- `#[pure]` and `FX0511`;
- what `main` prints after `Err(e)`.

### R14. §12.1: single-file programs (low priority)

Closes notes cross-cutting 1.

> `ferox check path.fx` treats one file as a crate root.

### Budget

The reference is roughly 19,000–23,000 tokens by rough estimate, against
D14's 25,000. R1–R12 add roughly 2,500. Two ways to make room:

- Move the per-type signature blocks in R5, R6, and R8 into the std card
  that D14 already plans (under 8 KB), and give that card to E1 writers.
- Shorten §16, which repeats the provisional markers.

## 6. Changes to `01-design.md`

### D-1. D7: file open modes and durability

Nothing in 01–04 lets a program append to a file. v0 targets storage
engines, which need it.

Proposed addition to D7:

> `Fs` covers Rust's `OpenOptions` modes through
> `fs.open_with(path, &OpenOptions)` (append, create, create-new, truncate,
> read-write) and makes directory entries durable with `fs.sync_dir(path)`.
> A `File` carries the authority of the `Fs` that opened it and offers
> `sync_all`, `sync_data`, and `set_len`.

### D-2. D15: what survives a crash

03 §9.5 keeps "only synced file data". It should also say:

- whether a new file's directory entry, or a `rename`, survives without
  `sync_dir`;
- whether `write_synced` covers both the file and its directory entry.

Optionally, add a crash of the test's own root, for example
`sim.faults.power_loss()`: discard unsynced data, invalidate open `File`s,
and keep the capabilities. Library crash tests could then skip the node
setup, whose entry must be `Owned + Send`, capture only `Frozen + Pure`
data, and report back through `run_until` or `records()`.

### D-3. D8: deadline outcome and bounded concurrency

D8 says only that cancellation is "level-triggered with deadlines". 03
TASK-4 makes a passed scope deadline a failure. Two independent authors,
reference §15.2 and the E1 writer, assumed the opposite.

- State the outcome in D8.
- Add a form that does not fail, for collect-what-finished work, such as
  `Scope::cancel_at(t)`: cancel at `t`, and let the body's result stand.
- Name `Tasks::limit(n)` in D8 as the bounded-concurrency tool, and say
  that `spawn` waits for a free slot.

### D-4. D10: specs over generic types

D10 should state what 04 decides: specs reach trait behavior only through
abstract spec fns (`cmp_spec`), and laws are lemmas.

D10 should also resolve C7, preferably as: "spec and proof fns may take
generic parameters without a `Pure` bound; instantiating one with a type
that is not `Pure` gives an opaque view". That keeps 04's own examples
legal. 04 §1.5 rule 3 and 03 §3.5 then follow.

### D-5. D9: `Combine::combine` by value

Change it to `fn combine(self, other: Self) -> Self`, or add
`combine_into(&mut self, other: Self)`, so that reductions over map-valued
monoids do not clone a whole map at every merge (word_freq).

### D-6. D0 and D14: the E1 protocol

- Regenerate the reference whenever a normative chapter lands, and check
  its examples against the chapters. §15.2 and §15.3 each contradict a
  chapter, and the writer copied both patterns.
- Give E1 writers the std card, or budget the std paths and signatures the
  tasks need into the reference.
- Rerun E1 after R1–R6. This trial mostly measured reference lag, not the
  language surface.

### D-7. D2 and D7: one name per concept

Arena handles are `Handle<T>` in the reference and `Id<T>` in 02 §10.9.
Task handles are `Handle<T>` in 02 §8.2 and `Task<U, E>` in 03. Choose
`Id<T>` or `Handle<T>` for arenas and `Task<U, E>` for tasks, everywhere.

### D-8. D14: property tests

Decide `#[test]` functions with parameters (the reference) and
`#[sim_test(seeds = N)]` (03) together: generation (`Arbitrary`), seeds,
and filtering with `requires`.

### Outside 01

These belong to other chapters:

- 02 §8.4 should say that aliased `lock` operands return `Err`, not panic,
  and 02 §8.2 should rename the task handle `Task`.
- 04 §1.6's diagnostic for a trait method called in a spec should suggest
  the paired spec fn.
- 03 §1.1 should say again how `root.fs` resolves relative paths. An
  earlier draft said "against the working directory at process start"; the
  current text says only that `root.fs` is rooted at `/`.
- 03 is still being edited. Its §13.2 already lists R1's edits.

## 7. Coverage of `notes.md`

The table lists each gap the writer reported, where a normative chapter
already decides it (blank means nowhere), and the change that closes it.

| Notes item | Decided in | Change |
| --- | --- | --- |
| X1 one file per program | | R14 |
| X2 import paths (`Poisoned`, `SimRoot`, `Faults`, `ExitCode`, `Cow`, `OpenOptions`) | 03 §6.5, ROOT-3 (in part) | R6 |
| X3 `Send`/`Sync` of capabilities and `Shared` | 02 §8.1 | R8 |
| X4 property-test inputs, seeds, filtering | | R9, D-8 |
| X5 receivers of `Rand` and `Stdin` | 03 §1.1 (`rng()`; receivers not stated) | R8 |
| wc: `read_only()` signature | 03 §1.1, §12.1 | R5 |
| wc: absolute paths on an attenuated `Fs` | 03 §1.1 | R5 |
| wc: `ExitCode` path and `from` | Rust | R6 |
| wc: simulated `NotFound` message | | R2 |
| wc: `File` implements `Read` | 03 §1.1 | R5 |
| csv: origins of std borrowing types | 02 §2.5 | R7 |
| csv: `'static` in consts | 02 §2.5 | R7 |
| lexer: enums with borrowed fields | 02 §3.1–3.2 | R7 |
| lexer: `from` in a generic argument that is not last | 02 §2.3 | R7 |
| lexer: `from *self` | 02 §2.2 | R7 |
| lexer: `-> Option<Self::Item>` | 02 §10.3 | R7 |
| bsearch: `==` versus `Ord` in specs | 04 §1.3, §5.1 | R3 |
| bsearch: trait methods in specs | 04 §1.6 | R3 |
| bsearch: which laws `Ord` carries | 04 §5.1 | R4 |
| bsearch: can a generic `T::cmp` panic under `prove` | 04 §3.1, §5.3 | R4 |
| bsearch: authority-freedom of a generic spec fn | conflicting (C7) | D-4 |
| bsearch: generic spec fns, `assert forall`, quantifier extent | 04 §1.2 | R3, R4 |
| fetch_all: no way to cap concurrency | 03 §1.1 `limit(n)` | R1, D-3 |
| fetch_all: deadline versus `scope`'s result | 03 TASK-4 | R1, D-3 |
| fetch_all: `lock` after cancellation | 03 LOCK-3 | R1 |
| fetch_all: `Poisoned` | 03 LOCK-5 | R1 |
| fetch_all: which errors to retry | | R8 |
| fetch_all: what `net_drop` does; observing in-flight work | 03 §9.4, `sim.trace()` (in part) | R2 |
| fetch_all: attenuating to a dynamic host list | | R8 |
| bank: same-account detection | 03 LOCK-2, `Shared::same` | R1 |
| bank: snapshot of many cells | 03 LOCK-8 | R1 |
| bank: `lock` forms | 03 §6.5 | R1 |
| bank: interleaving of `lock` and tasks under the simulator | 03 §9.3 | R2 |
| bank: is `Poisoned` `PartialEq` | | R1 (derive list for `LockError`) |
| bfs: `Handle` traits | 02 §10.9 (implied) | R11 |
| bfs: `Arena` API | | R11 |
| kv_log: append mode | | **D-1**, R5 |
| kv_log: durability calls and their authority | 03 §0, §1.1 | R5 |
| kv_log: directory fsync | | **D-1, D-2** |
| kv_log: crash and restart | 03 §9.4–9.5 | R2 |
| kv_log: torn writes; synced versus unsynced | 03 §9.4–9.5 | R2, D-2 |
| kv_log: `SimRoot` path and passing it to helpers | 03 ROOT-3 | R6 |
| kv_log: `Op` enum origins | 02 §3.1–3.2 | R7 |
| kv_log: `NotFound`; `dir` needs the directory to exist | | R5 |
| word_freq: `Combine` and `Monoid` paths; `par_*` are inherent | 03 §8.1 | R6, R12 |
| word_freq: `par_reduce` on an empty slice | 03 §8.3 | R12 |
| word_freq: generating values for law tests | 04 §2.4 | R9 |
| word_freq: laws use model `==` | 04 §5.1 | R4 |
| word_freq: `combine` by value | | D-5 |
| app_errors: `#[from]` implies `#[source]` | | R10 |
| app_errors: errors from an attenuated `Net`; the 503 status | | R8 |
| app_errors: `only` versus `dir` return types | reference §16 item 9 | R5 |
| app_errors: does `..` fail or clamp | 03 §1.1 | R5 |
| app_errors: what `main` prints after `Err` | 03 §1.3 | R13 |
| app_errors: `Url::parse`'s error type | | R8 |

Of these 55 items, 39 already have an answer in 02, 03, or 04: three only
in part, and one (C7) in two conflicting ways. Another 2 follow Rust or the
reference's own §16. The remaining 14 are decided nowhere.
