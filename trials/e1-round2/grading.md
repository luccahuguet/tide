# E1 grading: round 2

Inputs: the ten programs in `trials/e1-round2/programs/` (1,314 lines) and
`trials/e1-round2/notes.md`. The writer saw only `design/reference.md`, which
teaches the v0 core of `design/01-design.md` draft 3. The tasks are the same
as round 1's (`trials/e1/grading.md`); each is taken from the program's
header comment.

The grader acts as `ferox check`, `ferox test`, and a senior reviewer. The
normative design is `01-design.md` (its Tiers table defines v0, and 01 wins
conflicts), then `02-memory.md`, `03-authority-concurrency.md`, and
`04-contracts.md`. v0 has no `prove` mode, so there is no `ferox verify`
stage this round.

**The documents moved during grading.** 01, 02, 03, 04, and the reference
were all edited between 06:28 and 06:35, after the programs were written
(06:22). Programs are graded against the reference text the writer used.
Normative verdicts use 01 as it stood throughout (its D7 rule on
synchronization handles predates the programs) and 02–04 as read between
06:25 and 06:35. Two mid-grading edits matter and are noted where they
apply: reference §7.4 now spells `Shared::new(tasks, v)`, and 01 D10 gained a
bullet on v0 clause semantics.

## Method

Same rubric as round 1.

- **Error classes.**
  - *Rust habit leaked*: Rust spelling whose Ferox meaning differs.
  - *Reference gap*: a normative document decides it, but the reference
    omits it or says something else.
  - *Design gap or contradiction*: no normative document decides it, or they
    conflict.
  - *Ordinary bug*.
- **Definite or contingent.** A *definite* error is one the normative design
  rejects, or a failing test. A *contingent* item is a detail no normative
  document decides; the writer's guess is plausible and graded as passing,
  but a std that chose differently would reject it.
- **Severity.** 0: compiles and is correct. 1: one or two trivial fixes.
  2: a few real errors. 3: fundamentally wrong, or blocked by a design gap.
- **Iterations** are `ferox check` runs to a green check (the green run
  counts), plus each failing `ferox test` run. They assume diagnostics that
  carry the fixes shown here.
- **Diagnostic codes** are placeholders, like the reference's own. `FX08xx`
  marks a code that does not exist yet.

## 1. Results at a glance

| Program | First `ferox check` | Definite errors | Contingent | Severity | Check runs | All runs |
| --- | --- | --- | --- | --- | --- | --- |
| wc | pass | none | C10 | 0 | 1 | 1 |
| csv_line | pass | none | none | 0 | 1 | 1 |
| lexer | pass | none | none | 0 | 1 | 1 |
| bsearch | pass | none | C11, C12 | 0 | 1 | 1 |
| fetch_all | pass | none | C1, C4, C5, C6 | 0 | 1 | 1 |
| bank | **fail** | B1 (check) | C3 | 1 | 2 | 2 |
| bfs | pass | none | C2, C11, C12 | 0 | 1 | 1 |
| kv_log | pass | none | C8, C9, C10 | 0 | 1 | 1 |
| word_freq | pass | none | C10 | 0 | 1 | 1 |
| app_errors | pass | none | C1, C4, C7, C10 | 0 | 1 | 1 |

## 2. Aggregate

### 2.1 Pass rates

- **First-check pass rate: 9/10 (90%).** Only bank fails, on one error with
  one root cause (B1). The plausible range is 7–9: if the `Error` trait were
  not in the prelude (C1), fetch_all and app_errors would also fail. That
  risk is low, because 01's own example ("The Feel") writes
  `Box<dyn Error>` without an import.
- **Final pass rate: 10/10.** B1 has a machine-applicable fix
  (`ferox fix --thread Tasks --to accounts`, 03 §4.4), and no test fails.
- No program is blocked. In round 1, kv_log was blocked on a design gap.

### 2.2 Definite errors by class

| Class | Round 1 | Round 2 | Round 2 errors |
| --- | --- | --- | --- |
| Rust habit leaked | 0 | **0** | none |
| Reference gap | 9 | **1** | B1 |
| Design gap or contradiction | 2 | **0** | none definite (C5, C6, C8, C10, C11 are open) |
| Ordinary bug | 1 | **0** | none definite (review nits in wc, bank, kv_log, bfs) |

By stage, round 2 has one `ferox check` error and no failing test. Round 1
had eight check errors, one verify error, two test failures, and one
review-only error.

B1 is a reference lag, not a learnability failure. 01 D7 says creating a
synchronization handle takes `&Tasks`, and 02 §8.4 and 03 §1.2/§6.5 agree.
The reference the writer used said `Shared::new(v)` needs "no capability".
01's rule landed (commit "Require Tasks to create synchronization handles")
after the reference was regenerated and before the programs were written.
That is the failure round 1's D-6 predicted: a reference not regenerated
when the normative design changes.

### 2.3 Contingent items

| ID | Program | Writer's assumption | Status |
| --- | --- | --- | --- |
| C1 | fetch_all, app_errors | The `Error` trait, not only the derive, is in the prelude: `Box<dyn Error>` and `e.source()` with no import | Implied by 01's "The Feel" and reference §15.2; stated nowhere. Reference §12.3 names only the derive. |
| C2 | bfs | `Id<T>: Debug`, for `assert_eq!` on `Option<Vec<Id<Node>>>` | Reference §3.6 says Debug (provisional); 02 §10.9 lists `Copy`, `Owned`, `Pure`, `Send`, `Sync`. |
| C3 | bank | `Shared::with_all` exists | 03 §6.5 and 02 §8.4 have it under a provisional name; 01 D8 says cells are "accessed only through" `with` and `with2`. |
| C4 | fetch_all, app_errors | Simulator setup argument types: `world.http(&String, ..)`, `Reply::ok(String)` or `Reply::ok(&str)` | Undecided. |
| C5 | fetch_all | A `spawn` that waits for a `limit` slot until after the scope deadline still runs its child, and the child sees cancellation at its first checkpoint | Undecided. JOIN-2 ("in a cancelled scope the child is itself cancelled") suggests the child runs. If `join` returned `Err(Cancelled)` without running it, the body would return `Err` after the deadline, `scope` would report `DeadlineExceeded`, and both fetch_all tests would fail. Reference §15.2 has the same exposure. |
| C6 | fetch_all | Under `net_drop(0.5)`, `get` fails with `Timeout` at its deadline, or `Cancelled` when the scope deadline coincides | Reference §6.8 says so (provisional); 03 §9.4 calls `net_drop` a per-message fault without saying how it surfaces. |
| C7 | app_errors | `NetError::Denied` is a unit variant | Reference §6.7 names it (provisional), no shape. |
| C8 | kv_log | `restart` reruns a `fn` item entry; crashing a node whose entry has returned is a harmless power cut; records from crashed runs stay in `sim.records()`; `Record.node == "kv"` compiles; dropping the `NodeTask` is fine | Reference §6.8 and 03 §9.5 both type the entry `FnOnce` and say `restart` reruns it (a contradiction, D-4). 03 SINK-6 puts every record in the event trace, which supports the records assumption. The rest is undecided. |
| C9 | kv_log | `File::set_len` works on a file opened `read + append`; reading starts at offset 0 | 03 §1.1 lists `set_len`; the rest is POSIX behavior. |
| C10 | wc, word_freq, kv_log, app_errors | `root.fs` and its `read_only()` resolve relative paths against the starting working directory | Reference §6.3 says so (provisional). **03 §1.1 says "`root.fs` is rooted at `/`."** Under 03's reading, every relative CLI path (`a.txt`, `kv-data`, `appdata`) would break at run time. Tests do not catch it, since they use simulated roots. Round 1 raised the same point. |
| C11 | bsearch, bfs | Auto-generated contract tests (reference §10.2, "provisional for v0") pass | No rule says which `T` instantiates the generic `lower_bound` and `bsearch`, or whether `requires is_sorted(xs)` is generated constructively (04 §2.4 rule 2 lists a `sorted` shape) or filtered toward `vacuous`. `shortest_path` needs a generated `Graph`; per 04 §2.4 rule 1, a type with private fields is built from public calls, and `add_edge` panics on a stale `from`. |
| C12 | bsearch, bfs | v0 clauses may call any authority-free function and use closures: `is_sorted(xs)`, `path.windows(2).all(\|w\| self.has_edge(..))` | Reference §10.1 says so. 01 D10's "Spec language" bullet allows only `spec fn`s and `#[spec]` functions "proved total", which do not exist in v0. The Tiers table makes D10's spec language v1, so the reference's reading stands, but 01 never says what v0 clauses may call (D-2). The new D10 bullet covers integers and `==` only. |

The normative chapters settle several items the writer flagged, all in the
writer's favor:

- `Root::sim(seed)` in a plain or property test runs the body as the
  simulation's main task (03 §9.2), so bank's concurrent property test and
  kv_log's plain tests are fine.
- A named function may take `NodeRoot` (03 §12.3's `fn node(r: NodeRoot, ..)`).
- `Net::only` takes `"host:port"` strings (03 §1.1).
- `Arena<T>` is `Pure` when `T` is (02 §9), so bfs's `ensures` is
  authority-free.
- Repeated `invariant` clauses conjoin in source order (04 §1.2 rule 2).
- When a scope deadline and an operation deadline coincide, the operation
  sees cancellation (03 TASK-4). fetch_one handles that case.
- A trap inside a `with2` closure poisons both cells (03 LOCK-6). bank's
  `checked_add` is the right response, and it is better than reference §7.4's
  own example.
- `File::set_len` exists (03 §1.1).
- Attenuations chain, because each returns the same capability type
  (03 §1.1).

### 2.4 Iterations

- `ferox check` runs to green: **1.1** (11/10). Round 1: 1.3 over nine
  programs.
- All runs, including test: **1.1** (11/10). Round 1: 1.9.
- If C5, C6, or C11 resolve against the writer, up to three more test runs
  follow: fetch_all, and the generated tests of bsearch and bfs.

### 2.5 Rust baseline and the D0 thresholds

The Rust estimate is round 1's, since the tasks are unchanged:

| Program | Rust first check (est.) | Note |
| --- | --- | --- |
| wc | pass | |
| csv_line | pass | `Cow<'a, str>` |
| lexer | pass | `impl<'a> Iterator for Lexer<'a>` is routine |
| bsearch | pass | No contracts, so less is checked |
| fetch_all | about 50% | tokio semaphore, timeout, and `JoinSet`, or threads with a blocking client; partial results at a deadline are fiddly |
| bank | pass | Lock ordering by hand |
| bfs | pass | |
| kv_log | pass | `OpenOptions` and `sync_all` are routine; the crash test needs a hand-built fault-injecting filesystem, so Rust tests less |
| word_freq | pass | rayon |
| app_errors | pass | thiserror, serde, `reqwest::blocking` |

As in round 1, that is about 9/10 for Rust at first check (0.85–0.95) and
about 1.1–1.2 runs per program.

| Threshold (D0, E1) | Result | Verdict |
| --- | --- | --- |
| First check at least 0.8 of Rust's rate | 0.9 / 0.9 ≈ **1.0** (0.95–1.06 across the Rust range; 0.74–0.82 if C1 went against the writer) | met |
| Final pass at least Rust's | 10/10 against about 10/10 | met |
| Iterations within 1.5× of Rust's | 1.1 against about 1.1–1.2 (about 0.95×) | met |

### 2.6 Round 1 against round 2

| Measure | Round 1 | Round 2 |
| --- | --- | --- |
| Lines | 1,614 | 1,314 |
| First-check pass | 6/10 | **9/10** |
| Definite errors | 12 (0 habit, 9 reference, 2 design, 1 ordinary) | **1** (1 reference) |
| Final pass | 9/10 (kv_log blocked) | **10/10** |
| Check runs to green | 1.3 | 1.1 |
| All runs | 1.9 | 1.1 |
| First-check ratio against Rust | 0.67 (miss) | about 1.0 (met) |
| APIs the writer invented | 3 (`open_with`, `world.crash()`, `Faults { .. }`) | 0 |
| Gaps listed in notes | 55 | 64, almost all second-order (§7) |

Where round 1's twelve errors went:

| Round 1 errors | Why they cannot recur in round 2 |
| --- | --- |
| S1, S2, S3 (spec fns, trait methods in specs, laws) | v0 has no `prove` mode; contracts are executable (Tiers table) |
| W1 (law test of `Monoid::combine`) | `par_reduce` and `Monoid` moved to v1; word_freq folds sequentially |
| F3 (a passed scope deadline fails the scope) | **D8 changed** to what agents assumed: a deadline fails the scope only if a child returns the cancellation |
| L1, L4 (no append; no directory sync) | Design gap filled: `Fs::open_with`, `sync_dir`, `File::sync` (D7, 03 §1.1) |
| F1, B1, F2, L2, L3 (`lock` result type; simulator API) | The reference was synced with 03 |

Round 2's gains come from syncing the reference (5 errors), cutting
v1 features out of v0 (4), and changing or completing three decisions
toward what agents expected (F3, L1, L4). None comes from the writer
unlearning Rust: no Rust habit leaked in either round. That confirms round
1's reading. The surface was never the problem; the reference's lag and
scope were.

### 2.7 Why "met" is provisional

1. **The tasks are no longer held out.** The revised reference contains a
   near-verbatim model for every task: §15.1 for wc, §3.1's `split_csv`
   signature for csv_line, §3.3 for lexer, §15.3 for bsearch, §15.2 for
   fetch_all (fetch_all.fx is §15.2 with `Outcome` replaced by `Result`),
   §7.4 for bank, §3.6 for bfs, §6.3 plus the §6.8 crash test for kv_log, §8
   for word_freq, and §2.2 plus §6.2 for app_errors. Round 2 shows that the
   reference covers these tasks. It cannot show that an agent generalizes
   from the reference.
2. **Ten tasks, not D0's thirty**, and one writer.
3. **The Rust rate is estimated**, not measured.
4. **The grader is a model**, not a compiler. Contingent items C1, C5, and
   C10 could each move a program.
5. **The design moved during the trial.** The one definite error came from
   a 01 change the reference had not absorbed. Normative documents changed
   again while this grading ran.

E1 should count as passed only after a held-out round (§6, D-1).

## 3. Per-program grading

### 3.1 wc — severity 0

**Verdict.** Passes check and tests.

**Errors.** None.

**Logic.** Correct, with one nit.

- Lines count `\n` bytes and bytes count `data.len()`. Each column is
  right-aligned to width 8; `row("x")` traces to `"       1        2        3 x"`.
- Unreadable files print `wc: path: cause`, the rest are still counted, the
  exit code is 1, and no arguments gives usage with exit code 2, as the
  header specifies.
- The property test is sound: lines and bytes add over concatenation, and
  words can only merge at the join.
- Nit (ordinary): the doc comment at line 19 promises POSIX `wc` in the C
  locale, but `u8::is_ascii_whitespace` (line 23) omits vertical tab
  (0x0B), which C `isspace` includes. Round 1 had the same nit.
- Design choice: the `total` row prints even for one file.

**Authority and contracts.** `main` attenuates with `read_only()`;
`count_file` takes `&Fs`; `Counts::of` is authority-free and could carry
`#[pure]`. No contracts, and none needed.

**Iterations.** 1.

### 3.2 csv_line — severity 0

**Verdict.** Passes check and tests.

**Errors.** None.

**Logic.** Correct.

- All ten valid and four malformed cases trace correctly, including
  `"say ""hi""",2`, `""""`, and a trailing comma.
- Every slice index sits next to an ASCII byte, so non-ASCII input cannot
  panic. The `owned` buffer is allocated only at the first `""`, so plain
  and simply quoted fields borrow (`borrows_unless_unescaping` checks it).
- Every comparison type-checks: `Vec<Cow<str>>` against `&[&str]` and
  against `Vec<String>`, and `Cow::Owned(ref s) if s == ".."`.
- The round-trip property is sound: the encoder quotes exactly the fields
  that contain `,` or `"`. `requires !fields.is_empty()` accepts almost
  every generated case, so the test is not vacuous.
- Strict RFC 4180 (a quote inside an unquoted field is an error) is a
  documented design choice.

**Authority and contracts.**

- `#[pure]` on the public function.
- `ensures match &result { Ok(fields) => !fields.is_empty(), .. }` is weak
  but true and cheap. Matching on `&result` is right whether `result` is a
  place or a value (notes cross-cutting 6).

**Iterations.** 1.

### 3.3 lexer — severity 0

**Verdict.** Passes check and tests.

**Errors.** None. The origin choices are all correct:

- `Token` is a struct with one origin, `text`.
- `Option<Result<Token from self.src, LexError>>` puts `from` inside a
  nested generic argument; the comma ends it (02 §2.3).
- `let src = self.src;` copies the `&str` out, so `&src[a..b]` has origin
  `self.src` rather than the `&mut self` borrow. Rust does the same.
- `type Item = Result<Token from Self.src, LexError>` matches `next_token`'s
  `from self.src`, exactly as reference §3.3 pairs them.
- `fn new(src: &str) -> Lexer`, not `-> Self`.

**Logic.** Correct.

- Two-character punctuation comes first in `PUNCT`, so `==`, `->`, and `::`
  win.
- `"a $ b"` gives `a`, `Err('$' at 2)`, then `b` at 4; `"x€1"` gives
  `Err('€' at 1)`.
- The property test holds: every token is non-empty and equals the source
  at its offset.
- Design choices: `12ab` lexes as `Int` then `Ident`, and an unexpected
  character is reported once and skipped.

**Authority and contracts.** No authority. No contracts; the property test
does that job.

**Iterations.** 1.

### 3.4 bsearch — severity 0

**Verdict.** Passes check and tests, given C11 and C12.

**Errors.** None. Round 1's S1–S3 cannot occur in v0.

**Logic.** Correct.

- `lower_bound` keeps the invariant `xs[..lo] < target <= xs[hi..]`, and
  `mid = lo + (hi - lo) / 2` cannot overflow.
- `bsearch` returns the first equal element, as its `ensures` states
  (`i == 0 || xs[i - 1] < *target`).
- `small_cases` traces correctly: `Some(1)` for 3, `Some(5)` for 9, `None`
  for 4, `lower_bound` 4 and 6, and `Some(1)` for `"fig"`.
- The `#[should_panic]` test relies on the `requires` check, which runs in
  debug and test builds.
- Both property tests construct sorted input (`xs.sort()`), as reference
  §10.2 advises.

**Authority and contracts.**

- No authority.
- Strong, precise contracts. `lower_bound`'s two quantified `ensures` fix
  the result uniquely. The four loop invariants are exactly the proof
  skeleton a v1 `prove` run would need. All quantifiers are bounded.
- Cost: each loop iteration checks two O(n) invariants, over O(log n)
  iterations. That is fine in debug; under 04 §2.3 `assert` checks loop
  invariants only at entry and exit anyway.

**Contingent.**

- C11: the generated contract tests for generic `lower_bound` and `bsearch`
  need an instantiation, and `is_sorted(xs)` must be recognized as the
  `sorted` shape, or acceptance may fall below 04 §2.4's floor.
- C12: clauses call the user function `is_sorted` and `Vec::contains`.

**Iterations.** 1.

### 3.5 fetch_all — severity 0

**Verdict.** Passes check and tests, given C5 and C6.

**Errors.** None. The program is reference §15.2 adapted to return
`Result<Vec<u8>, NetError>` per URL.

**Logic.** Correct against D8 and 03 TASK-4.

- `tasks.limit(4).scope(..)` bounds concurrency.
- `set_deadline(deadline)` cancels stragglers. Every child returns `Ok`
  (fetch_one turns cancellation into `NetError::Timeout`), so `scope`
  returns partial results rather than `DeadlineExceeded`.
- Results come back in input order through `join`.
- `hung_hosts_hold_every_slot` traces: `fast` takes the first slot and
  finishes; the four hung requests then hold every slot; `fast/late` spawns
  only after the 10 s deadline, and its `get` returns cancellation at once
  (C5).
- `half_the_packets_dropped` allows 1 s of slack for modeled latencies.
  That is a reasonable hedge, since 03 §9.3 does not bound them.

**Authority and contracts.**

- Improved on round 1: `main` attenuates `Net` to the hosts named on the
  command line and adds `no_bind()`.
- `fetch_all` takes exactly `Tasks`, `Net`, and `Clock`.
- Nit: `root.clock.monotonic_only()` would also do.
- A URL without a host or port is silently left out of `only`, so it fails
  with `Denied`. That is acceptable, since it would fail anyway.

**Contingent.** C1 (`Box<dyn Error>`), C4 (`http(&format!(..))`,
`Reply::ok(String)`), C5, and C6.

**Iterations.** 1.

### 3.6 bank — severity 1

**Verdict.** Fails check with one error, at one site used from two tests.
The fix is mechanical.

| # | Line / snippet | Class | Compiler would say | Fix |
| --- | --- | --- | --- | --- |
| B1 | 78 `Shared::new(Account { id, balance })` in the test helper `accounts`, called at 88 and 106 | Reference gap. The reference the writer used said `fn new(v: T) -> Shared<T>; // no capability needed`. 01 D7, 02 §8.4, and 03 §1.2 and §6.5 say creating a synchronization handle takes `&Tasks`. | `error[FX05xx]: Shared::new takes 2 arguments but 1 was supplied`, with `note: creating a synchronization handle takes &Tasks (D7)` and `help: ferox fix --thread Tasks --to accounts` | Give `accounts` a `tasks: &Tasks` parameter and call `Shared::new(Account { id, balance }, tasks)` (03 §6.5 order). In `overdrafts_are_refused_and_change_nothing`, add `let sim = Root::sim(0);` and pass `&sim.tasks`, which 03 §4.4 rule 3 makes part of the `--thread` fix. In `total_is_conserved`, pass `&sim.tasks`. |

The spelling of the fix is itself unsettled. 03 §6.5 has
`new(v: T, tasks: &Tasks)` ("argument position provisional"); the reference,
edited during this grading, now has `new(tasks: &Tasks, v: T)`. See R1.

**Logic.** Correct.

- Both checks (`checked_sub`, `checked_add`) run before either write, so a
  refused transfer changes nothing and nothing traps inside `with2`. That
  avoids poisoning (LOCK-6), which reference §7.4's own example does not.
- `Shared::same` turns the aliased case into `SameAccount` before `with2`
  could return `Err(Aliased)`.
- `total` is one consistent snapshot through `with_all` (C3).
- `run_transfers` spawns one task per transfer.
  - The scope's error type has `Task(#[from] TaskError)`, as `scope`
    requires.
  - `task.join()?` unwraps the task layer, and the match separates refusals
    from failures.
  - `Overdraft | SameAccount` as a nested or-pattern is fine.
- The property test is meaningful: random balances and plans, every
  transfer concurrent under a seed-chosen schedule, total conserved. Totals
  are sums of `u32` values in a `u64`, so `Overflow` cannot fire.
- Nit: the doc comment says refused transfers are expected outcomes, but
  `Overflow`, which also leaves balances unchanged, is treated as a failure
  that cancels the scope.

**Authority and contracts.**

- `run_transfers` takes `&Tasks`, which it needs to spawn.
- No contracts. The writer read reference §10.1 correctly: a clause such as
  `requires plan.iter().all(|&(f, t, _)| f < accounts.len() && ..)` calls
  `len` on `&[Shared<Account>]`, which is not `Pure`, so it is forbidden.
  That costs a useful precondition for no safety gain (D-7).

**Contingent.** C3 (`with_all`).

**Iterations.** 2 check runs.

### 3.7 bfs — severity 0

**Verdict.** Passes check and tests, given C2.

**Errors.** None.

**Logic.** Correct.

- Standard BFS with a parent map, so the path is shortest by edge count;
  `[start]` when `start == goal`.
- Stale ids are skipped as neighbors and give `None` as endpoints.
- `self.nodes[id]` is only indexed with live ids.
- Traces: `a` to `d` gives `a, e, d`; `d` to `c` gives `d, a, b, c`; after
  `remove_node(b)`, `a` to `c` is `None`.
- The property test compares BFS distances with Bellman-Ford relaxation
  from node 0 on random graphs; `requires n > 0` accepts 255/256 of cases.

**Authority and contracts.**

- No authority.
- The `ensures` on `shortest_path` checks that a returned path starts at
  `start`, ends at `goal`, and follows edges. Minimality is left to the
  property test. That is a good division between a cheap checked clause
  and a sampled test.
- Review: `add_edge` panics on a stale `from` (documented) instead of
  stating `requires self.node(from).is_some()`. The clause would document
  the precondition in the trust surface and would let a generated-test
  harness that builds `Graph`s from public calls respect it (C11).

**Contingent.** C2 (`Id: Debug`), C11, and C12 (a closure calling
`self.has_edge` inside `ensures`).

**Iterations.** 1.

### 3.8 kv_log — severity 0

**Verdict.** Passes check and tests, given C8 and C9. In round 1 this
program was blocked.

**Errors.** None. `open_with` with a chained `&mut OpenOptions` temporary,
`sync_dir`, `File::sync`, `set_len`, `json::encode`/`decode`, the
slice-pattern CLI, and the `NodeRoot` entry are all as the reference and 03
define them.

**Logic.** Careful and correct.

- **Durability protocol.**
  - The log's directory entry is synced after `create` (`dir.sync_dir(".")`),
    and the data directory's entry too (`sync_dir("/")` in the node,
    `sync_dir(".")` in `main`).
  - A write is acknowledged only after `write_all` and `sync`.
  - JSON escapes newlines, so the only `\n` in a record is its terminator.
- **Torn tails.**
  - `open` replays complete lines and truncates a torn tail with
    `set_len` and `sync` before any append.
  - This fixes a real bug in reference §6.3's `Log` example. That example
    ignores a torn last line when reading but keeps appending after it, so
    the next acknowledged record is glued onto the torn bytes and lost on
    the next read (R2).
- **The crash test.**
  - It crashes and restarts the node three times at seed-chosen moments,
    lets the last run finish, and crashes again.
  - It checks that every acknowledged key survived and that keys are
    contiguous.
  - Contiguity holds because each `set` is synced before the next begins,
    so at most the last record is in doubt. A record that survived without
    its `acked` log line is allowed.
- **Nits.**
  - `kv get` creates the data directory and the log, so a read command
    writes.
  - After a failed `commit`, nothing stops reuse of the store in the same
    process; the doc comment says to reopen. `run` opens a fresh store per
    command, so there is no live bug.
  - A corrupt line that is not the last fails `open` with `InvalidData`.
    That is strict, and crashes alone cannot cause it.

**Authority and contracts.**

- Exemplary. `main` uses `root.fs` only to create and attenuate the data
  directory, and `run` gets the attenuated `Fs`.
- Record encoding and replay are authority-free.
- No contracts. A `json::decode(json::encode(r)) == r` property would be
  the natural addition.

**Contingent.**

- C8: the restart-entry contradiction, a crash after the entry finished,
  records after crashes, and `Record` field types. `acked >= 40` also
  assumes 40 synced writes fit in 10 virtual seconds, which holds for
  millisecond-scale latencies.
- C9: `set_len` on an append-mode file.
- C10: `kv-data` is relative to the working directory.

**Iterations.** 1.

### 3.9 word_freq — severity 0

**Verdict.** Passes check and tests.

**Errors.** None.

**Logic.** Correct.

- Words are maximal alphanumeric runs, lowercased.
- `par_map` counts each document; a sequential fold merges into a
  `BTreeMap`, so order and schedule cannot matter.
- `top` sorts by count descending, then word ascending.
- The temporary map in `for .. in top(&word_freq(&docs), 20)` lives for the
  whole loop.
- `counts_across_documents` traces: `the` 3, `a` 2, `cat` 2, so the top
  three are `the`, `a`, `cat`.

**Authority and contracts.**

- `#[pure]` on `word_freq`; `main` attenuates with `read_only()`.
- I/O stays sequential because `par_map` closures must be `Pure`. That is
  correct; a `tasks.limit(n)` scope would parallelize reading.
- The two property tests (parallel equals sequential; document order does
  not matter) are the right checks.

**Iterations.** 1.

### 3.10 app_errors — severity 0

**Verdict.** Passes check and tests, given C1 and C7.

**Errors.** None.

**Logic.** Correct.

- Two inline modules stand in for libraries, each with its own error type.
- `AppError` wraps them with `#[from]` and wraps `io::Error` twice with
  explicit constructors.
- `#[from]` implies `#[source]` (reference §2.2), so messages name only
  their own layer and `fail` prints the `source()` chain without
  repetition.
- `run` writes with `write_synced`, which is durable. It is not atomic for
  concurrent readers, which the task does not require.
- Traces: the body `{"up":true}` is 11 bytes. The denied host gives
  `Client(Net(Denied))`. A missing config gives `"cannot load
  configuration"`, then `"cannot read config.json"`, then `NotFound`.

**Authority and contracts.**

- Exemplary, and more thorough than round 1. `main` attenuates `Fs` to one
  directory, `Net` to one host with `no_bind()`, and `Clock` to
  `monotonic_only()`. `run` reads the config through a further
  `read_only()`.
- The tests check the network attenuation and the error chain.
- `#[codec(default)]` on an `Option` field is harmless. Whether it is needed
  is undecided (R12).

**Contingent.** C1 (`e.source()` with no import), C4, C7
(`NetError::Denied`), and C10.

**Iterations.** 1.

## 4. Where the documents still disagree

### 4.1 Reference against the normative documents

| # | Topic | Reference (as the writer saw it) | Normative text | Effect here |
| --- | --- | --- | --- | --- |
| 1 | Creating synchronization handles | §7.4: `Shared::new(v)`, "no capability needed"; §7.3 `channel()`, `sync_channel(n)`; §6.4 says nothing on `Arc::new` | 01 D7; 02 §8.4; 03 §1.2: every constructor takes `&Tasks` (`Shared::new(v, tasks)`, `channel(tasks)`, `Arc::new(v, tasks)`, `AtomicU64::new(v, tasks)`); clones are free | B1. Mid-grading, reference §7.4 changed to `new(tasks, v)`, the opposite argument order to 03 §6.5; §7.3 still has `channel()`. |
| 2 | `root.fs` and relative paths | §6.3: resolved against the starting working directory | 03 §1.1: "`root.fs` is rooted at `/`" | C10, at run time only |
| 3 | `net_drop` | §6.8: the operation is delayed and fails with `Timeout` at its deadline | 03 §9.4: a per-message fault, outcome unstated | C6 |
| 4 | Node entry result | §6.8: `FnOnce(NodeRoot) -> Result<T, E>` | 03 §9.5: `-> Result<(), E>` (already reference §16.2 item 12) | none; kv_log returns `io::Result<()>` |
| 5 | When loop invariants are checked | §10.1: before the first iteration, after each, and at exit | 04 §2.3: under `assert`, entry and exit only; every iteration only under `test` | none |
| 6 | `with_all` | §7.4: provisional | 01 D8: cells are accessed "only through" `with` and `with2`; 03 §6.5 and 02 §8.4 list `with_all` as provisional | C3 |
| 7 | What v0 clauses may call | §10.1: any function whose arguments are `Pure`, including closures | 01 D10: only `spec fn`s and `#[spec]` functions "proved total"; v1 by the Tiers table | C12 |

### 4.2 Within the normative documents

- 03 §9.5 (and reference §6.8) type the node entry `FnOnce`, yet `restart`
  "reruns the entry". An `FnOnce` cannot run twice.
- 03 §9.5 still says the entry "captures only `Frozen + Pure` data", though
  `Frozen` is removed (01 Tiers, 03's own header).
- 03 §6.5's `Shared::new(v, tasks)` and the reference's new
  `Shared::new(tasks, v)` disagree. 03 §1.2 marks the position provisional.
  Value-first is the better choice: it keeps Rust's `Arc::new(v)` shape with
  one trailing argument, and it matches 03's `Arc::new(v, tasks)` and
  `AtomicU64::new(v, tasks)`.
- 01 D10's "Spec language" bullet reads as binding in v0, but it requires
  `spec fn` or `#[spec]` functions "proved total", which only exist in v1.
  The new v0 bullet in D10 settles integer and equality semantics but not
  which functions a v0 clause may call.

## 5. Changes to `reference.md`, prioritized

Order: definite errors prevented first, then latent bugs in examples agents
copy, then the number of notes gaps closed. Budget: the reference is about
76 KB, roughly 19,000–21,000 tokens against D14's 25,000; R1–R6 add about
900.

### R1. §7.3, §7.4, §6.4, §14: handles are created with `&Tasks`

Prevents B1. Partly done mid-grading (§7.4 only).

- One spelling everywhere, agreed with 03: `Shared::new(v, tasks)`,
  `channel(tasks)`, `sync_channel(n, tasks)`, `Arc::new(v, tasks)`,
  `Arc::downgrade(&a, tasks)`, `AtomicU64::new(v, tasks)`. Cloning an
  existing handle needs nothing.
- One sentence of why: handles let tasks observe each other (D7), so
  authority-free code can neither create nor receive one.
- Test pattern: a test that builds cells needs a simulated root,
  `let sim = Root::sim(0);` and `Shared::new(acct, &sim.tasks)`.
- A §14 habit row: `Shared::new(v)` or `Arc::new(v)` without `&Tasks` →
  `FX05xx`, fixed by `ferox fix --thread Tasks`.
- In the §7.4 `transfer` example, check both amounts before either write,
  as 03 §12.4 does. The current `b.balance += amount` traps inside `with2`
  and poisons both cells, which the writer noticed and avoided.

### R2. §6.3: the `Log` example must truncate a torn tail

A latent data-loss bug in an example that kv-store tasks will copy. After a
crash leaves a torn last line, `Log::open` keeps appending after it; the
next acknowledged line is glued to the torn bytes and lost on the next read.
Proposed `open`:

```ferox
pub fn open(dir: &Fs) -> io::Result<Log> {
    let mut file = dir.open_with("log", OpenOptions::new().read(true).append(true).create(true))?;
    dir.sync_dir(".")?;                           // the new entry survives a crash
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let done = bytes.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
    if done < bytes.len() {
        file.set_len(done as u64)?;               // drop the torn, never-acknowledged tail
        file.sync()?;
    }
    Ok(Log { file })
}
```

Also drop "(provisional)" from `set_len`, which 03 §1.1 lists, and say that
`set_len` works on an append-mode file and that reads start at offset 0.

### R3. §6.8: simulator details agents need for crash and fault tests

Closes notes cross-cutting 3–5, fetch_all 1 and 5, and kv_log 1, 3, 4, and 9.

- The node entry is rerun by `restart`, so it is
  `Fn(NodeRoot) -> Result<(), E> + Owned + Send` (or a `fn` item), after
  D-4.
- `restart` returns `()`. Say whether an earlier `NodeTask` and
  `run_until` follow the restarted run, whether dropping a `NodeTask` stops
  the node, and that crashing a node whose entry has returned discards its
  unsynced state.
- Sink records, including `println!` output (03 SINK-6), survive crashes in
  `sim.records()`. Give the field types: `node: String`, `level`,
  `message: String`.
- A plain `#[test]` that calls `Root::sim(seed)` runs as the simulation's
  main task (03 §9.2), and can take generated parameters. Say whether
  `#[sim_test]` can too.
- Setup signatures: `world.file(path: impl AsRef<Path>, contents: impl
  AsRef<[u8]>)`, `world.http(addr: &str, f: impl Fn(&Request) -> Reply)`,
  `Reply::ok(body: impl Into<Vec<u8>>)`. Name the `Request` accessors.
- What `net_drop(p)` does to `Net::get` (after D-4), and how large modeled
  latencies are, so tests can bound `sim.elapsed()`.

### R4. §10.2: generated contract tests and contract panics

Closes notes bsearch 3, csv 3, and cross-cutting 7. Prevents C11 failures.

- Which instantiation a generic function with `ensures` is tested at (or
  that it is skipped and reported as unchecked).
- Which `requires` shapes are generated constructively (04 §2.4 rule 2),
  naming the predicates recognized, such as `is_sorted(xs)` or
  `xs.is_sorted()`.
- How parameters of types with private fields are built (04 §2.4 rule 1:
  public call sequences, which respect the callees' `requires`).
- Whether an auto-generated test below the vacuity floor fails `ferox test`
  or is reported as unchecked (D-6).
- The panic message of a violated contract, such as
  `requires violated: is_sorted(xs)`, so `#[should_panic(expected = ..)]`
  can target it.

### R5. §12.3 and §6.3: prelude and paths

Closes notes cross-cutting 2 and wc 1. Resolves C1 and, after D-5, C10.

- "The prelude includes the `Error` trait as well as the derive, so
  `Box<dyn Error>` and `e.source()` need no import."
- State how `root.fs` resolves relative paths, once D-5 decides, and that
  `read_only()` and `dir()` keep the base.

### R6. §7.1–7.2: a spawn waiting for a `limit` slot when the scope is cancelled

Closes notes fetch_all 2. Resolves C5. Reference §15.2 depends on it.

> A `spawn` that is waiting for a slot when its scope is cancelled still
> starts the child, which observes the cancellation at its first
> checkpoint; `join` then returns whatever the child returns.

(Or the opposite, with §15.2 and its prose fixed to match.)

### R7. §10.1: clauses over collections of handles

Closes notes bank 1, after D-7. Allow shape-only operations (`len`,
`is_empty`, `Shared::same`) on slices and `Vec`s of non-`Pure` elements in
clauses, so `requires from < accounts.len()` is legal.

### R8. §10.1: `result` and quantifier extent

Closes notes cross-cutting 6 and bsearch 2.

- `result` is a place: `match &result`, `result.as_ref()`, and
  `match result` on a `Copy` value all work.
- A quantifier body inside a `match` arm ends at the arm's `,`.

### R9. Small std facts

Each closes one notes item.

- `NetError::Denied` is a unit variant. `NetError` is not `PartialEq`, since
  it holds `io::Error`; test it with `matches!`.
- `#[from]` implies `#[source]` (drop "provisional"). When `main` prints the
  `source()` chain, a message should not repeat `{0}`.
- `TaskError` derives `Debug, Clone, Copy, PartialEq, Eq, Error`.
- `with_all` on an empty slice runs `f` with an empty slice.
- `write_synced(&self, p, data: impl AsRef<[u8]>)`.
- The `par_map` signature: `fn par_map<U: Send>(&self, f: impl Fn(&T) -> U
  + Pure + Send + Sync) -> Vec<U>`.
- `Arena` has `Default`, and `contains` is no longer provisional. `Id<T>`
  is `Debug` (C2).

### R10. §2.2: codec defaults and enum shape

Closes notes kv_log 7 and app_errors 5.

- A missing `Option` field decodes as `None` without `#[codec(default)]`,
  as in serde.
- Enums are externally tagged: `{"Set":{"key":..,"value":..}}`.

### R11. §12.1: single-file programs

Closes notes cross-cutting 1. Say that a file with `main` is a binary root
and one without is a library root (so `println!` is `FX0508`), and that
`ferox test path.fx` exists.

## 6. Design fixes

### D-1. D0 and D14: the E1 protocol

- **Held-out tasks.** The reference now has a near-verbatim model for each
  of the ten tasks (§2.7). Round 3 should use tasks the reference was not
  revised against, drawn from D0's thirty. They should cover shapes the
  reference shows less of: streaming `Stdin`, a channel pipeline or worker
  pool, a TCP server under partition, a struct borrowing from two sources,
  trait objects and associated types, `Rand`-driven simulation, and a JSON
  round trip with nested enums.
- **Measure Rust** with the same writer instead of estimating it.
- **Freeze the inputs.** Regenerate the reference and check its examples
  whenever 01 changes, and freeze 01–04 and the reference while a trial is
  written and graded. The one error this round came from a 01 change made
  after the reference was regenerated; this grading saw five documents
  change underneath it.

### D-2. D10 and the Tiers table: the v0 clause language

D10's "Spec language" bullet allows only `spec fn`s and `#[spec]` functions
"proved total", neither of which v0 has. Read literally, it would reject
bsearch (`is_sorted`), bfs (`has_edge` in a closure), and reference §15.3.
Round 1's largest error class came from the same place. Add to D10's new v0
bullet:

> In v0, a clause is an executable Rust boolean expression, plus bounded
> `forall`/`exists` and `==>`. It may call any function or closure whose
> arguments and captures are `Pure`. The spec-fn restriction applies under
> `prove` (v1).

### D-3. D7 and D8: handle constructors and `with_all`

- Name the constructor form in D7 (`Shared::new(v, &tasks)`, `channel(&tasks)`,
  `Arc::new(v, &tasks)`), say clones are free, and settle the argument
  order, which 03 and the reference now disagree on.
- D8 says cells are accessed "only through" `with` and `with2`. List
  `with_all`, since 02, 03, and the reference all have it and bank's
  consistent snapshot needs it.

### D-4. D15 and 03 §9: nodes, restarts, and network faults

- The node entry must be rerunnable: `Fn(NodeRoot) -> Result<(), E>`,
  `Owned + Send`, capturing only `Pure` data. Replace 03 §9.5's leftover
  `Frozen + Pure`.
- Define a crash of a node whose entry has returned (unsynced state is
  lost; nothing restarts), and whether `run_until` follows restarts.
- Say how a per-message drop surfaces in `Net::get`: a `Timeout` at the
  operation's deadline (the reference's reading), a `Reset`, or a
  transparent retransmission.

### D-5. D7 and 03 §1.1: `root.fs` and relative paths

03 says `root.fs` "is rooted at `/`"; the reference resolves relative paths
against the starting working directory. Every CLI task here (wc, word_freq,
kv_log, app_errors) depends on the working-directory reading, and round 1
raised the same point. Decide it in D7, preferably for the working
directory, with `/` as the working directory under the simulator.

### D-6. D10 and D14: generated contract tests in v0

The reference makes `ferox test` call every authority-free function with an
`ensures` on generated inputs, and 04 §2.4 fails a test below the vacuity
floor. Together these make every function with a selective `requires` a
potential `vacuous` failure the author never wrote. Decide:

- instantiation of generic functions (a std default per bound, such as
  `i64` for `Ord`, or skip and report);
- constructive generation for recognized predicates, including user
  `is_sorted`-style functions;
- types that cannot be generated;
- that an auto-generated test below the floor is reported as unchecked,
  while a hand-written property test below it fails.

### D-7. 04 §1.6–§1.7 and reference §10.1: shape-only operations on handle collections

`len`, `is_empty`, and identity on a `&[Shared<T>]` observe nothing another
task can change: a cell's view is an opaque identity token (04 §1.4). Allow
them in v0 clauses and in `#[spec]` calls at non-`Pure` element types, so
index preconditions like bank's can be stated.

## 7. Coverage of `notes.md`

The writer listed 64 items, against 55 in round 1. They are smaller: round
1's gaps were core APIs (append, the `lock` result, the simulator), while
round 2's are mostly argument types and edge semantics. None of the 64
caused a check error. The one check error (B1) came from something the
writer had no reason to question, because the reference stated the opposite
of 01.

| Status | Count | Items |
| --- | --- | --- |
| Already answered by 01–04 or Rust; the reference should state it | 23 | cross-cutting 3; lexer 1, 2, 3, 5; csv 4; bsearch 1, 4; fetch_all 3, 4, 7; bank 2, 3, 5; bfs 2, 3; kv_log 2, 6; word_freq 2, 3; app_errors 1, 3, 7 |
| Partly answered | 7 | csv 3; bsearch 2, 3; wc 3; kv_log 9; word_freq 1, 4 |
| The documents conflict | 4 | wc 1 (D-5); fetch_all 1 (D-4); kv_log 1 (D-4); bank 1 (D-7) |
| Decided nowhere | 23 | cross-cutting 1, 2, 4, 5, 6, 7; wc 2, 4; fetch_all 2, 5, 6; bank 4, 6; bfs 1, 4; kv_log 3, 4, 5, 7, 8; app_errors 2, 4, 5 |
| Not a gap (task or design choice) | 7 | wc 5; csv 1, 2, 5; lexer 4, 6; app_errors 6 |

Changes R1–R11 and D-2 to D-7 close every item in the conflict and
decided-nowhere rows except wc 2 (non-UTF-8 arguments), wc 4 (error message
text), and fetch_all 5 (latency size, partly R3), which are low priority.
One item, kv_log 5, is a real bug in a reference example (R2).
