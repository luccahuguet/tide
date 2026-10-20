# E1-dialect grading: a held-out round on the D18 reference

**Inputs** (revision `fab8e7d`):

- `tasks.md`: eleven held-out tasks, chosen to exercise what the origins
  trial did not (01 D2, "Accepted costs"): outlives bounds, `+ use<..>`,
  Rust 2024 capture, and `Scope<'s, 'env, E>`, plus capabilities, the
  simulator, and attribute contracts.
- `programs/*.rs` (11 files, 1,593 lines) and `notes.md`, by one fresh
  writer (claude-opus-5-5). It read `design/reference.md` in full, then
  `design/reference-companion.md` §15, §16, §17, and Appendix A on demand
  (not §14), and `tasks.md`, in an isolated directory with no compiler.
- Writer cost: 17 API calls, peak context 192,494 tokens, 2,193,709
  cache-read plus 174,084 cache-creation input tokens, 16.5 minutes. Its
  output (programs plus notes) measures 27,290 tokens by
  `scripts/count-tokens.sh`. The transcript's own output-token field (1,091)
  undercounts and is not used.

**Normative sources.** Tide is a checked dialect of Rust (01 D18): every
program is a Rust 2024 crate that rustc builds against Tide's std after
Tide's checker admits it, and borrow checking is rustc's (02 §2.8, §11).
`design/01-design.md` owns every decision (its Tiers table defines v0, and
01 wins conflicts), then `02-memory.md`, `03-authority-concurrency.md`, and
`04-contracts.md`. No design file changed between the writer's copy and
grading: both reference files were byte-identical to the repository's at
`fab8e7d`, and `git diff HEAD -- design` is empty.

**Protocol notes.** Writer isolation held. There is one writer, so there is
no arm comparison. The tasks are held out: unlike round 2, no companion §15
program is a near copy of a task, although 15.1 (dedupe's testable `run`),
15.2 (retry's `sim.world.http`), and 15.4 (backup's crash test) are close
models for parts of three tasks.

## Method

The rubric is that of `trials/e1-origins/grading.md` and
`trials/e1-round2/grading.md` (their Method sections):

- **Classes:** Rust habit leaked, reference gap, design gap or
  contradiction, ordinary bug.
- **Definite or contingent:** a definite error is one the normative design
  rejects, or a failing test. A contingent item is a detail no normative
  source decides; the writer's guess is graded as passing, with the
  consequence if it went the other way.
- **Severity** 0–3. **Check runs** count `tide check` runs to green; **all
  runs** add failing `tide test` runs. Both assume diagnostics carry the
  fixes.
- Every definite error and contingent item is tagged **borrow-related**
  (lifetime spelling or semantics, elision, outlives bounds, `use<..>` or
  `impl Trait` capture, borrowing types, closure captures, `Scope`
  lifetimes) or **other**. Review nits are listed but not counted.
- A question the normative chapters decide but the reference does not is a
  reference gap even when it caused no error (§3.0).
- Every test was traced by hand, including each drop test: which lifetime
  the result carries, which loans that lifetime holds, and whether the drop
  conflicts with a live loan or with drop check.

**The rustc oracle (new this round).** `trials/e1-dialect/rustc/run.sh`
reproduces it with one command. It builds a stub std
(`rustc/std/src/lib.rs`, 519 lines) whose items carry the signatures that
the reference and 02/03 give, with `unimplemented!()` bodies. It then builds
each program with rustc 1.96.0 stable as a 2024-edition crate, with the stub
injected as `std`, as `spikes/d18/scripts/env.sh` does:
`rustc --edition 2024 --extern std=<stub rlib> -L dependency=<deps> -D unused-imports --emit metadata`.
Each program is built twice, as `--crate-type lib` (dedupe as `bin`) and
with `--test`. `-D unused-imports` stands in for `FX0913`.

- **Nothing was stripped from any program**, and there is no
  preprocessing. Toolchain attributes are proc macros exported through the
  stub's prelude, as 01 D6 says Tide's are:
  - The contract attributes, `invariant!`, and `#[tide::main]` are the D18
    spike's (`spikes/d18/macros`). Clauses are type-checked inside the
    lowered body.
  - `rustc/macros/src/lib.rs` supplies `#[test]`, `#[sim_test]`,
    and `derive(Error)`. It replaces Rust's built-in `#[test]` as 01 D6
    says Tide's does. A test with parameters (a property test) and a
    `#[sim_test]` become plain functions: their bodies are type- and
    borrow-checked, but nothing generates inputs or runs them.
  - `derive(Error)` follows thiserror's surface: `Display` through
    `write!` with the message's fields bound (so `{path}` and `{0:?}` are
    checked against real fields and their `Display`/`Debug` impls),
    `source()` from `#[source]`, `#[from]`, or a field named `source`, and
    `From` for `#[from]` fields.
- **Approximations.** `par_map` is a method of a prelude trait over `[T]`,
  since a stub std cannot add inherent slice methods (reference 8).
  `HashMap` is Rust's (the fixed-key wrapper changes no type or borrow
  verdict here). The log macros format their arguments and discard them.
- **Signatures the stub had to guess** (marked `GUESS` in the stub):
  `Env::args()` returns an iterator of `String`; `Fs::read_dir` returns
  Rust's `ReadDir` of `io::Result<DirEntry>`; `Instant` keeps Rust's
  `checked_add`; `mpsc::RecvError` has `is_cancelled()`; `Id<T>`'s traits
  hold for every `T`; `sim.world.args` takes any iterable of
  `Into<String>`, and `sim.world.stdin` any `AsRef<[u8]>`;
  `sim.run_for` returns `()`. Each matches a contingent item below. The
  stub encodes 02 §8.2 exactly: `Scope<'s, 'env: 's, E>`, `Copy`, invariant
  in `'s` and `'env`. `SimNode::start` takes an
  `Fn(NodeRoot) -> Result<T, E> + Send + Sync + Pure + 'static` entry
  (companion 17, 03 §9.5).
- **Validation.** Before trusting a clean run, ten probes went through the
  same build. Nine were rejected as rustc should reject them:
  - a `get` elided to `&self` that outlives its `Dict` (E0597);
  - 02 §10.12's `serve`, whose child borrows locals through a passed-down
    scope (E0373);
  - a mutation of a place a child borrows (E0502);
  - a `Rules` that borrows a `Vec` declared after it (drop check, E0597);
  - a `use<'a>` that omits a captured `'w` (E0700);
  - a `derive(Error)` message naming a missing field (E0425);
  - an unused import;
  - type errors inside a plain test and a property test;
  - an `ensures` comparing `String` with `usize`.

  02 §10.12's `parallel_sum` was accepted as it should be. The probes were
  scratch files and are not kept.
- **A variant run** removed `'env: 's` from the stub's `Scope`, which is
  the signature as reference 7.1 printed it at `fab8e7d`. rustc then
  rejects longest_lines (2× E0521), pipeline (E0521), and tally (E0597,
  E0521). To reproduce:
  `sed -i "s/Scope<'s, 'env: 's, E>/Scope<'s, 'env, E>/"` on a copy of the
  stub. See G1.

## 1. Results at a glance

| Program | First `tide check` | Definite errors | Contingent | Severity | Check runs | All runs | rustc-checked |
| --- | --- | --- | --- | --- | --- | --- | --- |
| canon | pass | none | none | 0 | 1 | 1 | yes (lib, test) |
| rules | pass | none | none | 0 | 1 | 1 | yes (lib, test) |
| scan | pass | none | none | 0 | 1 | 1 | yes (lib, test) |
| longest_lines | pass | none | none | 0 | 1 | 1 | yes (lib, test) |
| pipeline | pass | none | C1 | 0 | 1 | 1 | yes (lib, test) |
| retry | pass | none | C2 | 0 | 1 | 1 | yes (lib, test) |
| backup | pass | none | C3, C4 | 0 | 1 | 1 | yes (lib, test) |
| ring | pass | none | C5, C6 | 0 | 1 | 1 | yes (lib, test) |
| deps | pass | none | C7 | 0 | 1 | 1 | yes (lib, test) |
| tally | pass | none | none | 0 | 1 | 1 | yes (lib, test) |
| dedupe | pass | none | C8 | 0 | 1 | 1 | yes (bin, test) |

All 22 oracle builds pass with no errors and no warnings.

## 2. Aggregate

### 2.1 Pass rates and runs

| Measure | Result |
| --- | --- |
| First-check pass rate | **11/11** |
| Final pass rate | **11/11** |
| Check runs to green (total, mean) | 11 (1.0) |
| All runs (total, mean) | 11 (1.0) |
| Failing tests | 0 |
| Drop tests ("must stay usable after X is dropped"), all passing | 7: canon ×3, scan ×2 (the `Needle`, the local `word`), longest_lines ×1, rules ×1 (`one_of` borrowing a same-block `Vec`) |

**Plausible range.** If every contingent item went against the writer, the
first check would pass 6 to 11 programs:

- pipeline fails if channel errors have no `is_cancelled()` (C1);
- deps fails if `Id<T>`'s impls bound `T` (C7);
- backup fails if `DirEntry::file_name` is not admitted or `read_dir`
  yields another type (C3);
- dedupe fails if `world.stdin` takes only bytes (C8);
- retry fails if a simulated HTTP peer must be `Send` (C2).

Test-level risks are C2 (retry timing), C4 (backup's crash test), C5 (ring's
`should_panic` text), and C6 (ring's generated tests). Each is a one-line
fix.

### 2.2 Definite errors by class

| Class | Borrow-related | Other |
| --- | --- | --- |
| Rust habit leaked | 0 | 0 |
| Reference gap | 0 | 0 |
| Design gap or contradiction | 0 | 0 |
| Ordinary bug | 0 | 0 |
| **Total** | **0** | **0** |

**Borrow-related definite errors: 0.** rustc agrees: the oracle found no
type or borrow error in any program, and no verdict where hand tracing and
rustc disagreed. One hand-traced point was refined by the oracle. The writer
annotated three closure return types (notes item 5). rustc shows that only
the scope body's annotation in tally's `count_concurrently` is needed:
without it, E0283 reports "type annotations needed for `Scope<'_, '_, _>`".
`?` converts through `From`, and std's `impl<T> From<T> for T` adds a
candidate beside `From<TaskError>`, so a scope result fed to `?` is
ambiguous unless the target error type is `TaskError` or an `Err` in the
scope names `E` (rounds 21 and 22, B2 and B1; 02 §10.12's `parallel_sum`
returns `TaskError` and needs none). The two `Shared::with*` annotations
are optional, since those calls are tail expressions (G8).

No Rust habit leaked. The programs use no ambient call, `as` cast, `Arc`,
`Mutex`, `thread::spawn`, unbounded library channel, `println!` in a
library, or user macro, and no unused import (rustc `-D unused-imports`).
No `Result` is ignored (`FX0702`).

Review nits, not counted: N1–N3 (§3), all other.

### 2.3 Contingent items

| ID | Program | Tag | Writer's assumption | Status |
| --- | --- | --- | --- | --- |
| C1 | pipeline | other | `RecvError` has `is_cancelled()`, so the summer can tell a cancellation (an echo) from a disconnect (end of input) | Reference 7.2 says "channel errors whose `is_cancelled()` is true". 03 §1.1 promises a cancellation variant only for *capability* error types. 03 §6.4 makes channels "Rust's `std::sync::mpsc`", whose `RecvError` is a unit struct, and TASK-3 says only that a checkpoint returns "its cancellation error". If the method is missing, the check fails at line 50; the fix is to treat every `recv` error as end of input, and the tests still pass (§3.5). G3. |
| C2 | retry | other | 503 is transient; three GETs to a peer that answers at once take under 600 ms of packet time, so the 300–900 ms bound holds; the `http` peer closure need not be `Send` (an `Rc<Cell<u32>>` counter) | Reference 6.7 marks `is_transient` "provisional beyond `Timeout` and `Cancelled`", and 6.8 gives 1–10 ms per packet but no packet count for `Net::get`. Reference 6.8 types the peer `impl Fn(&Request) -> Reply + 'static`; 03 §9.2 gives no type. If 5xx were not transient, `succeeds_on_the_third_attempt` fails. If peers must be `Send`, the check fails (fix: a `Shared<u32>` counter). G5, G9. |
| C3 | backup | other | `fs.read_dir(".")` yields Rust's `io::Result<DirEntry>`; `DirEntry::file_name()`, `OsString::into_string()`, and `OsStr::is_empty()` are admitted | Reference 6.3: `read_dir` "as std::fs". 03 ADMIT-1 excludes I/O methods on std types, so `DirEntry::metadata` and `file_type` are probably out (the writer avoided them), but it publishes no list. `OsStr` is not among reference 1's standard types. G6. |
| C4 | backup | other | After a crash, an unsynced `rename` leaves the target naming the old file or the new one, never neither; node records survive the crash; a node's disk is separate from `sim.fs` | 03 §9.5 and D15 say unsynced "directory entries may be lost or torn", without defining a torn rename. Companion 17 says records survive a crash and nodes have their own disks. If a crash can drop the target entry, `a_crash_leaves_old_or_new_contents` can fail at `unwrap()` on a seed between `rename` and `sync_dir`. G4. |
| C5 | ring | other | A precondition violation panics with a message containing `requires violated` | Reference 10.1 gives the text as provisional; 04 CLAUSE-7 says only that a violation panics. Under another text, `pushing_into_a_full_ring_violates_the_precondition` fails. G10. |
| C6 | ring | other | Generated contract tests build `Ring` (private fields) from public call sequences that respect each call's `requires`, and the resulting tests are not vacuous | 04 §2.4 rule 1 says only "random sequences of public constructor and method calls". If a sequence could call `push` on a full ring, the generator itself would violate a precondition. `with_capacity` draws `n` from all of `usize`, which the lazy storage makes safe (notes 14). G10. |
| C7 | deps | other | `Id<T>`'s `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, and `Debug` impls hold for every `T`, so `Handle(Id<Node>)` derives them and `HashMap<Id<Node>, usize>` works while `Node` derives only `Debug` | Reference 3.6 lists them unconditionally. It cites 02 §10.9 for `Eq` and `Hash`, but 02 §10.9 lists only `Copy`, `Owned`, `Pure`, `Send`, and `Sync`; its own `reachable` example needs `Eq + Hash` (`HashSet<Id<Node>>`). An std that derived them would add `T:` bounds, and deps would fail the check. G7. |
| C8 | dedupe | other | `sim.world.stdin("x\ny\nx\nz")` accepts a `&str`, and `sim.world.args([..])` an array of `&str` | Reference 6.8 marks setup types provisional and writes `stdin(bytes)`. 03 §9.2 names `stdin` and `args` without types. If `stdin` took `&[u8]`, the check fails (fix: `b"..."`). G9. |

No borrow-related contingent item remains. Every borrow question the
writer raised is decided by 01/02 or by rustc (§3.0).

### 2.4 Tokens

| Item | Tokens |
| --- | --- |
| Core reference (`reference.md`, recorded in the companion) | 24,954 |
| Companion, whole (`scripts/count-tokens.sh`) | 9,848 |
| Companion sections read (§15–§17, Appendix A) | about 6,950 (estimate: their 70.6% byte share of the whole) |
| Writer peak context | 192,494 |
| Cumulative input (17 calls: 2,193,709 cache-read + 174,084 cache-creation) | 2,367,793 |
| Output (programs plus notes) | 27,290 |
| Output per program (11) | about 2,480 |
| Cumulative input per program | about 215,250 |
| Output per passing program (11 of 11 at first check) | about 2,480 |
| Cumulative input per passing program | about 215,250 |

The references the writer read come to about 31,900 tokens. Most of the
cumulative input is that context re-read across 17 calls.

### 2.5 Why the result is provisional

1. **One writer and eleven tasks**, against D0's thirty.
2. **The oracle checks types and borrows only.** It runs no test. Test
   verdicts rest on hand traces and on the contingent items above. The
   stub guessed eight signatures, all listed.
3. **The rustc is stable 1.96**, not D18's pinned nightly, and the
   admission allowlist (03 ADMIT-1) does not exist yet, so `FX0520`
   verdicts are readings of the reference.
4. **Partial cover of the targeted features**: no program uses a
   quantifier, `==>`, `Net::only`, or a second capability attenuation
   besides `Fs::dir`, `read_only`, and `Tasks::limit` (§4).

## 3. Per-program findings

### 3.0 Questions the reference leaves open

Every question the writer raised (notes items 1–21 and the per-program
risks), and every one grading turned up, with its resolution. None is a
definite error.

| ID | Location | Question | Resolution and source | Class |
| --- | --- | --- | --- | --- |
| Q1 | longest_lines 29–43, pipeline 24, 63, tally 99–106 (notes 2) | Does `Scope<'s, 'env, E>` declare `'env: 's`, so a child may borrow caller data and results like `&'d str` flow out? | Yes. 01 D2: "`Scope<'s, 'env, E>` declares `'env: 's`, so the higher-ranked `'s` is bounded below by the `&Tasks` borrow". 02 §8.2 gives `pub struct Scope<'s, 'env: 's, E>`, invariant, and 02 §5.4 explains the bound. Reference 7.1 prints only the impl and says children "may borrow anything that outlives the `scope` call". The oracle's variant shows the bound is load-bearing: without it rustc rejects three programs (Method). | reference gap (G1), borrow-related, no error |
| Q2 | scan 71, 76 (notes 6) | May `+ use<'a>` on a `&self` method leave out the elided `&self` lifetime? | Yes. 02 §2.3's `Capture` rule requires every type parameter in scope and every lifetime the bounds name, nothing else; 02 §5.2's `len_of(..) -> impl Fn() -> usize + use<'a>` releases an elided input the same way. rustc accepts both methods and rejects a `use<'a>` that omits the captured `'w` (E0700). Reference 3.5 has one clause, "`+ use<'a>` captures less", and no example. | reference gap (G2), borrow-related, no error |
| Q3 | rules 9–50 (notes 1) | Does a struct of boxed closures that may borrow need a lifetime parameter and `T: 'a` on the owning predicate? | Yes. Reference 3.4: `Box<dyn ..>` in a field means `+ 'static`; write `+ 'a`. 02 §2.4 item 3 for `T: 'a`. | decided by the reference |
| Q4 | canon 31, 40 | `fn canonical_or<'r>(&self, word: &'r str) -> &'r str where 't: 'r` | Reference 3.2's "own lifetime" row, written as a where-clause. 02 §2.4 item 2. | decided by the reference |
| Q5 | longest_lines, pipeline (notes 3) | Which error does a scope report when the body fails through `join()?`? | The earliest-spawned failing child's, echoes excluded (reference 7.1; 03 TASK-2; 01 D8). | decided by the reference |
| Q6 | pipeline (notes 4) | Is a send to a dropped receiver an echo? Is `send` on a non-full `sync_channel` a checkpoint? | Neither. The echo bit is set only "when a checkpoint ... returns a cancellation error" (03 TASK-2), so a disconnect error a child returns is an independent failure. Only `send` on a *full* channel and `recv` are checkpoints (03 §6.4, reference 7.3). The parser's choice to return `Ok(sent)` on any send error is therefore right: returning `Err` on a disconnect would compete with the summer's `Overflow` and could win. Reference 7.1 does not spell out the first point. | reference gap (G3), other, no error |
| Q7 | pipeline 50 (notes 4) | Does `RecvError` have `is_cancelled()`? | Undecided (C1). | design gap (G3) |
| Q8 | tally 61, 79, 99 (notes 5) | Must a closure's return type be annotated around `?`? | Only where the scope's result is the operand of `?;`, its target error type is not `TaskError`, and no `Err` in the scope names `E`: rustc E0283 without it (§2.2; rounds 21 and 22). Tail-position calls take it from the return type. Reference 7.1 and 7.4 show only tail-position forms; 02 §10.12's `after_failure` annotates the `let`. | reference gap (G8), other, no error |
| Q9 | dedupe 96 (notes 7) | Are `Stdin::read_to_string`/`read_to_end` inherent, or `Read` methods needing `use std::io::Read`? | Undecided. Reference 6.1 and 03 §1.1 list them as `Stdin` operations "with `&self` receivers", which `Read`'s `&mut self` methods are not, so inherent is the likely reading. In Rust they are `Read` trait methods. The writer's `read_line` loop avoids the question. | design gap (G9), no error |
| Q10 | dedupe tests (notes 8) | How does a test run "the whole program"? | Tests cannot call `main` (`FX0506`) or pass `root`; reference 6.2 and companion 15.1 put the logic in `run(env, fs, stdin)`. `SimRoot` has `Root`'s capability fields, `env` and `stdin` included (03 §9.2). | decided by the reference |
| Q11 | backup (notes 9) | With the task's `fs: &Fs` signature, can `backup` "only be able to read below `src` and write below `dst`"? | Not as declared authority: 03 §4.1 computes it from parameter types, and `&Fs` is the whole filesystem. The writer attenuates at once and confines the copy loop to `fs.dir(src, Mode::Read)` and `fs.dir(dst, Mode::ReadWrite)`. That is the strongest form the given signature allows. A task that wants the guarantee must give `backup` two attenuated `Fs`. | task wording, no error |
| Q12 | backup (notes 10) | What does `read_dir` yield, and which `DirEntry` methods are admitted? | Undecided (C3). | design gap (G6) |
| Q13 | backup (notes 11) | Can a crash tear an unsynced `rename`? | Undecided (C4). | design gap (G4) |
| Q14 | ring (notes 12) | May a `pub` method's clause read private fields? | Undecided for v0. Reference 10.1's visibility rule is provisional and names functions only; 04 §1.4's private-in-public rule is about v1 spec fns. The answer decides whether ring's clauses are `checked` (fields) or `sampled` (calls to `len()`; 04 CLAUSE-5). Both run under `tide test`. | design gap (G11), no error |
| Q15 | ring (notes 13) | Where does a `while let` loop's invariant run, before or after the condition? | 01 D10 says only "at loop entry ... and at exit". 04 §2.4 fixes the next-index rule for `a..b` alone. The spike checks before each evaluation of the condition. The writer used `for moved in 0..n`, which is decided. | design gap (G12), no error |
| Q16 | ring (notes 14, 15) | Generated tests' input sizes and the violation text | C5, C6. | design gap (G10) |
| Q17 | deps (notes 16) | Do `Id<T>`'s impls bound `T`? | Undecided (C7). | reference inaccuracy and design gap (G7) |
| Q18 | retry (notes 17) | Simulated GET cost, `is_transient`, `drop_all` timing, `Send` peers | C2. `drop_all` is safe both ways: the test accepts one attempt or five. | design gap (G5, G9) |
| Q19 | retry (per-program risk) | Is `Instant::checked_add` admitted? | Reference 1 lists `Instant` among the standard types that "keep their Rust API"; only the ambient `now`/`elapsed` go (6.9). Graded as decided. | decided by the reference |
| Q20 | tally | Does a `Shared::with` closure capturing an owned `String` meet `Pure`? | Yes: a closure is `Pure` exactly when its captures are (reference 6.6, 03 §3.1). | decided by the reference |
| Q21 | all | Do the `error!` sink macro and `#[error]` derive helper coexist in one prelude? | Yes. The oracle's prelude holds both, and rustc reports no ambiguity. | decided (rustc) |

### 3.1 canon — severity 0

**Verdict.** Passes check and tests.

- **Lifetimes.** `Dict<'t> { first: HashMap<String, &'t str> }`.
  - `get(&self, ..) -> Option<&'t str>` returns the build text's lifetime,
    not `&self`'s (reference 3.2, the `text(&self) -> &'a str` row).
  - `canonical_or<'r>` and `rewrite<'r>` declare `where 't: 'r` and return
    `&'r str`. In `self.get(word).unwrap_or(word)`, `Option<&'t str>`
    coerces to `Option<&'r str>` by covariance under the bound
    (02 §2.4 items 2–3).
- **Drop tests.** All three were traced against rustc's loans; the oracle
  agrees.
  - `get_result_outlives_dict`: `got: Option<&'t str>`, with `'t` the loan
    of `text`. The `&d` autoref loan ends at the call, so dropping `d` at
    the block's end conflicts with nothing live. `HashMap`'s destructor is
    `#[may_dangle]` (02 §6, the audited exemption).
  - `canonical_or_result_outlives_dict`: `known` and `unknown` carry
    `'r ⊆ text ∩ other`. `drop(d)` moves `d`, and no loan of `d` is live.
    `other` lives to the block's end.
  - `rewrite_result_outlives_dict`: inner `Dict` and outer `line` (the
    second half), and both inner (the first half).
- **Other tests.** `"Hello World hello WORLD Rust"` gives
  `HELLO`→`Hello`, `world`→`World`, `rUsT`→`Rust`, and `missing`→`None`.
  `canonical_or("Other")` is `"Other"`. `rewrite` gives `["The", "Quick",
  "brown", "fox"]` and `["fox", "jumps"]`.
- **Authority and contracts.** None needed.

### 3.2 rules — severity 0

**Verdict.** Passes check and tests.

- **Lifetimes.** `Rules<'a, T> { preds: Vec<Box<dyn Fn(&T) -> bool + 'a>> }`
  (reference 3.4).
  - `at_least` bounds `T: PartialOrd + 'a`, since the closure owns a `T`.
  - `one_of(&mut self, allowed: &'a [T])` bounds `T: PartialEq + 'a`.
  - `custom<F: Fn(&T) -> bool + 'a>` boxes `F`. The `&T` inside `Fn` sugar
    is higher-ranked (02 §5.4), so `|row| !banned.contains(row)` takes
    `row: &&str` and `HashSet<&str>::contains::<&str>` applies.
- **Drop check.** `one_of_borrows_a_vec_from_the_same_block` declares
  `allowed` before `r`, so `r`, whose boxed trait objects may use `'a` in
  their destructors, drops first. The oracle's reversed-order probe fails
  with E0597, which confirms the writer's note.
- **Tests.**
  - `at_least(10)` plus `one_of(&[5, 10, 20, 30])`: the literal is a
    promoted `'static` constant. Results are 5 no, 10 yes, 15 no, and a
    count of 3 over `[5, 10, 15, 20, 30, 40]`.
  - The custom set counts 2 over `["ham", "spam", "", "eggs", "junk"]`.
  - The same-block `Vec` counts 3 (6, 9, 12), and `matches(&3)` is false.
  - The empty list accepts everything.

### 3.3 scan — severity 0

**Verdict.** Passes check and tests.

- **Horspool.** `skip[b] = m - 1 - i` for each byte but the last; on a
  mismatch the window advances by the skip of its last byte, and on a match
  by `m` (non-overlapping). Traces: `"aaaa"` in nine `a`s gives `[0, 4]`;
  `"abc"` in `"xxabcabcab"` gives `[2, 5]` (skips a→2, b→1); `"ab"` gives
  none; an empty needle matches nowhere.
- **Needle drop.** `positions` returns a named `Positions<'h>` that clones
  `pat` and copies the `[usize; 256]` table, so it carries only the
  haystack's loan. `positions_outlive_the_needle` drops `needle` with no
  live loan of it and yields `[0, 8]`.
- **`use<..>` (Q2).** `numbered(&self) -> impl Iterator<Item = (usize,
  &'a str)> + use<'a>` and `containing<'w>(&self, word: &'w str) -> impl
  Iterator<Item = &'a str> + use<'a, 'w>` capture the text, and the word
  where the filter closure holds it, but not the `&self` borrow. Each
  clones the `Lines` (`derive(Clone)`), so nothing is consumed. rustc
  accepts both. Here `use<..>` is not load-bearing: every test collects
  before reusing `lines`, so Rust 2024's default capture would also pass.
- **Tests.** `"first line\r\nsecond\r\nthird\r\n"` gives `(1, "first
  line")`, `(2, "second")`, `(3, "third")`. `containing("ir")` gives
  `["first line", "third"]`, then `lines.count()` is 3. `"a\nb\r\n\nc"`
  gives `["a", "b", "", "c"]`. In the local-`String` test, `hits: Vec<&'a
  str>` holds no loan of `word`, so dropping `word` is fine.

### 3.4 longest_lines — severity 0

**Verdict.** Passes check and tests.

- **Scope lifetimes (Q1).** `longest_lines<'d>(tasks, docs: &[&'d str], ..)
  -> Result<Vec<&'d str>, DocError>`.
  - Each child gets `let doc: &'d str = doc;`, a reborrow through a shared
    reference that keeps `'d`, and returns `Result<&'d str, DocError>`.
  - `U: 's` needs `'d: 's`, which follows from `'env: 's` and the caller's
    `&Tasks` borrow.
  - The results do not borrow `docs` (the `Vec` in the drop test) or the
    `limit` temporary, which lives through the tail expression.
- **Drop test.** `result_outlives_the_docs_vec`: `got: Vec<&'d str>` with
  `'d` the loans of `a` and `b`. The `&docs` loan ends when
  `longest_lines` returns, so dropping `docs` conflicts with nothing.
- **Failure.** `nul_document_fails_the_whole_call` with `limit(1)`:
  - child 0 succeeds;
  - child 1 returns `Nul(1)` and cancels the scope;
  - child 2's `spawn` waited past the cancellation, so it still starts
    (reference 7.1, provisional) and returns `Ok`, having no checkpoint;
  - the body's `collect` stops at `h1`'s `Err(Task(Cancelled))`.

  The scope reports the earliest-spawned failing child's `Nul(1)` (03
  TASK-2 rule 2). Under `limit(3)` and in `total_words` the outcome is the
  same.
- **Other tests.** The three documents give `["the longest line", "",
  "bb"]`: in `"a\r\nbb\r\ncc"`, `lines()` strips `\r\n` and `cc` loses the
  tie. `total_words` gives 6.
- **Contracts.** `#[requires(max_parallel > 0)]` keeps `limit(0)` out,
  whose meaning no document fixes. There is no `ensures`, so no generated
  test.

### 3.5 pipeline — severity 0

**Verdict.** Passes check and tests, given C1.

- **A scope passed down.** `spawn_parser<'s>(s: Scope<'s, '_, PipeError>,
  lines: &'s [String], tx: SyncSender<i64>)` is 02 §8.2's
  `serve<'s>(bg, cfg: &'s Config)` form. The child borrows only `lines`
  and owns `tx`. In `sum_lines`, the borrow `lines: &[String]` meets `'s`
  through `'env: 's` (Q1). `spawn_summer` owns `rx`.
- **Tests.**
  - *Ten lines*: `(10, 55)`. *Empty*: the parser sends nothing and drops
    `tx`; the summer's `recv` disconnects and it returns `Ok(0)`.
  - *Bad line 3*: the parser sends 1 and 2 (capacity 4), then returns
    `BadLine(3)`. The summer either observes the cancellation (an echo,
    03 TASK-2) or the disconnect (`Ok`). Either way `parser.join()?` fails
    the body, and rule 2 reports `BadLine(3)`.
  - *Overflow*: `MAX + 1` fails the summer with `Overflow`, an independent
    failure. The parser, blocked on a full channel or facing a dropped
    `rx`, gets a send error and returns `Ok(sent)` (Q6). The body fails
    through `summer.join()?`, and rule 2 reports `Overflow`.
  - *Negative values*: `-9223372036854775808` and `+3` parse with Rust's
    `i64::from_str`; partial sums are −1, −6, −3, so `(4, -3)`.
- **C1.** Only line 50 depends on `RecvError::is_cancelled()`. If every
  `recv` error ended the input instead, the summer would return `Ok` after
  a cancellation, which is not a failure, and the outcomes above would not
  change.

### 3.6 retry — severity 0

**Verdict.** Passes check and tests, given C2.

- **Logic.** The deadline is `clock.now() + overall`, passed to every
  `get`. The loop returns on success, on a cancelled or non-transient
  error, and when attempts run out. Before sleeping it gives up if `now +
  delay` would not land strictly before the deadline, which also stops a
  timed-out request (transient per reference 6.7) from being retried past
  the deadline. A cancelled `sleep_until` returns `NetError::Cancelled` at
  once. The delay doubles with `saturating_mul`.
- **Tests.**
  - *503, 503, 200*: sleeps of 100 and 200 ms; `calls == 3`; elapsed is
    300 ms plus packet time, at most 300 ms more at 10 ms per packet for
    up to ten packets per GET, inside the 900 ms bound (C2).
  - *404*: not transient, one attempt.
  - *Hanging peer*: `get` returns `Timeout` at 2 s; `2 s + 100 ms` is not
    before the deadline, so one attempt and elapsed is 2 s.
  - *`drop_all`*: one attempt at 2 s, or five quick timeouts with 1.5 s of
    sleeps; both meet the assertions.
- **Authority and contracts.** `&Net` and `&Clock` only, and
  `#[requires(policy.attempts >= 1)]`.

### 3.7 backup — severity 0

**Verdict.** Passes check and tests, given C3 and C4.

- **Attenuation (Q11).** `fs.dir(src, Mode::Read)?` comes first, so a
  missing `src` fails with `NotFound` before anything is written. Only
  `create_dir_durably` uses the full `fs`. `copy_matching` holds a
  read-only `Fs` below `src` and a read-write `Fs` below `dst`.
- **Durability** (reference 6.3; 03 §1.1):
  - each newly created ancestor of `dst` gets `sync_dir` of its parent
    (`ancestors().take_while(missing)`, with `"."` for a relative root);
  - each copy is `write_synced` to a temporary, then `rename`d over the
    target;
  - one `to.sync_dir(".")` at the end makes the renames durable before
    `Ok`.

  This is the standard atomic-replace protocol.
- **Tests.**
  - *Two matching files and one other*: `["a.log", "b.log"]`, contents
    copied, `c.txt` absent from the backup.
  - *Replacement*: `/backup` exists, so no directory is created; `a.log`
    reads `"new"`, `keep.txt` is untouched, and no temporary is left.
  - *Missing `src`*: `NotFound`, and `/backup` is absent.
  - *Crash test* (200 seeds, crash at 5 ms): if `"backup done"` was
    logged, `/dst/a.log` must read the new contents, which holds because
    `Ok` follows `sync_dir`. If only `"setup done"` was logged, the target
    was `write_synced` as `"old"` and its directory synced, so it reads old
    or new, given C4. The node entry captures nothing, so it is an `Fn`,
    `Send`, and `'static` (03 §9.5).
- **Nits** (other, not counted):
  - **N1**: the temporary name avoids only names being copied, so an
    unrelated `a.log.backup-tmp` already in `dst` would be overwritten and
    renamed over `a.log`. Disclosed in notes item 11.
  - **N2**: `from.metadata(name)` follows symlinks, and one dangling entry
    fails the whole backup. Disclosed in notes item 10.

### 3.8 ring — severity 0

**Verdict.** Passes check and tests, given C5 and C6.

- **Storage.** The invariant is that while `buf.len() < cap`,
  `head + len == buf.len()`.
  - It holds through `push`: `tail = head + len < cap`, so `tail ==
    buf.len()` and the value is pushed.
  - It holds through `pop`, because `head` cannot wrap while `head < head
    + len <= buf.len() < cap`.
  - Once `buf.len() == cap`, it is an ordinary ring with `tail < cap ==
    buf.len()`.

  `pop` indexes `buf[head]` only when `len > 0`.
- **Contracts.**
  - `with_capacity` has `requires(n >= 1)` and ensures `len() == 0` and
    `capacity() == n`.
  - `len` ensures `result <= capacity()`, and `is_full` ensures
    `result == (len() == capacity())`.
  - `push` requires `!is_full()` and ensures `len() == old(len()) + 1`.
  - `pop` ensures `result.is_none() == (old(len()) == 0)` and
    `len() == old(len()).saturating_sub(1)`.
  - `extend_from` requires the elements fit and ensures the length adds.
  - `drain_into` ensures `out.len() == old(out.len()) + old(self.len())`
    and `len() == 0`, with the loop invariants `self.len() + moved == n`
    and `out.len() == out_start + moved`. At entry `moved == 0`; after
    each iteration one element has moved; at exit `moved == n`
    (04 §2.4's next-index rule).

  Every clause calls `pub` methods (reference 10.1). Each is therefore
  `sampled` (04 CLAUSE-5) and runs on every call under `tide test`. No
  `&mut` parameter's lifetime appears elsewhere in a signature, so there
  is no `FX0803`.
- **Tests.**
  - *Wraparound*: 1, 2 pop; 4 → slot 0; 5 → slot 1; pops 3, 4, 5, then
    `None`.
  - *Extend and drain*: `head = 1` after the first pop; 4 lands in slot 0;
    the drain gives `[0, 1, 2, 3, 4]`.
  - *Property test*: `cap` in 1..=8 is a recognized scalar-range shape
    (reference 10.2); the model is `VecDeque`, compared via
    `Vec::from(model)`.
  - *Full ring*: `push(2)` on a full `with_capacity(1)` violates `requires`,
    and `tide test` evaluates sampled clauses on every call, so it panics
    (C5).

### 3.9 deps — severity 0

**Verdict.** Passes check and tests, given C7.

- **Arena.** `Handle(Id<Node>)`. `remove` drops the node, its name, its
  insertion slot, and every edge to it (disjoint field borrows of
  `inserted` and `nodes`). A stale handle reads `None` through
  generations.
- **`order`.** Kahn's algorithm with a `BTreeSet<usize>` ready set keyed
  by insertion index, so ties go to insertion order. If nodes remain, a
  walk from the earliest one along not-yet-emitted dependencies must
  revisit a node, and that loop is the reported cycle.
- **Capture.** `dependents(&self, h) -> impl Iterator<Item = &str>` relies
  on Rust 2024 capture of the elided `&self` lifetime (reference 3.5;
  02 §5.2). rustc accepts it.
- **Tests.**
  - *Diamond*: `base, left, right, app`; dependents of `base` are
    `left, right`, of `left` is `app`, and of `app` none.
  - *Independent nodes*: `a, b, c`.
  - *Cycle*: the self-loop `solo` comes first; after removing `solo`, the
    cycle is `x, y, z`.
  - *Removing `z`*: `y, x`.
  - *Stale handle*: `b != c` even if `c` reuses the slot; `name(b)` is
    `None`; `depends(a, b)`, `depends(b, a)`, and `remove(b)` are all
    `Stale`; `order` gives `a, c`; a re-added `b` differs from the old
    handle.

### 3.10 tally — severity 0

**Verdict.** Passes check and tests.

- **Parallel.** `docs.par_map(|doc| freq(doc))` captures nothing, and
  `String` is `Pure + Sync` (reference 8). The per-document maps are
  merged sequentially (`par_reduce` is v1).
- **`Shared`.**
  - `add` checks with `checked_add` before its only write.
  - `merge_into` returns early when `Shared::same`, so self-merge is a
    no-op and `with2` never sees `Aliased`. It computes every total before
    any write, then inserts and clears.
  - Both closures capture only `Pure` data (03 LOCK-4), and an `Overflow`
    leaves both cells unchanged and unpoisoned (no panic).
  - `?` maps `LockError` and `TaskError` through `#[from]`.
- **Scope.** Children capture `tally: &Tally` (`Shared` is `Sync`) and
  `doc: &String`. The scope body's return-type annotation is needed (Q8).
- **Tests.**
  - `freq` gives `the: 2`, `cat: 1`, `hat: 1`; separators alone give an
    empty map.
  - The fixed documents give `the: 3`, `hat: 2`, `x2: 3`, `trick: 1`.
  - Self-merge leaves `{a: 2}`; a merge gives `{}` and `{x: 1, y: 5}`.
  - On overflow, both `add` and `merge_into` return `Overflow("w")` and
    both maps are unchanged.
  - The sim test's totals do not depend on the interleaving.

### 3.11 dedupe — severity 0

**Verdict.** Passes check and tests, given C8.

- **Structure.** `parse_args` and `dedupe` are authority-free. `run(env,
  fs, stdin)` holds the program minus printing (Q10). `main` is
  `#[tide::main]`; it passes `root.fs.read_only()` and borrows
  `root.env` and `root.stdin`, as reference 6.2 allows, and maps errors to
  exit codes 2 and 1 through `ExitCode::from(u8)`.
- **Tests.**
  - `"b\na\nb\nc\na\n"` gives `"b\na\nc\n"`, and with counts
    `"3\tb\n2\ta\n1\tc\n"`.
  - `"x\r\nx\ny"` gives `"x\ny\n"`, and `"\n\n"` with counts gives
    `"2\t\n"`.
  - Usage errors (`--bogus`, two files, `-c`) exit with 2.
  - In the sim tests, the named file gives `"2\tb\n2\ta\n1\tc\n"`, standard
    input `"x\ny\nz\n"` (the `read_line` loop ends on 0), and a missing
    file `Read { NotFound }` with exit code 1.
- **N3** (other, not counted): a lone `-` is rejected as an unknown flag.
  This is a disclosed reading (notes item 21).

## 4. Coverage

| Feature | Where | How it fared |
| --- | --- | --- |
| `'a: 'b` | canon (`where 't: 'r`, twice) | Correct and load-bearing: it lets results mix build-text and argument borrows and outlive the `Dict`. rustc confirms all three drop tests. |
| `T: 'a`, `F: .. + 'a` | rules (`T: PartialOrd + 'a`, `T: PartialEq + 'a`, `F: Fn(&T) -> bool + 'a`) | Correct, with drop order handled. |
| `+ use<..>` | scan (`use<'a>`, `use<'a, 'w>` on `&self` methods) | Correct (Q2); the reference shows no such example (G2). Not load-bearing in the tests. |
| Rust 2024 return-position capture | deps `dependents` (implicit capture of `&self`); scan narrows it | Correct. For `positions` the writer returned a named struct instead of relying on capture rules, a cautious choice with less to learn. |
| `Scope<'s, 'env, E>`: children borrowing from the caller | longest_lines (`&'d str` in, results out), tally (`&Tally`), total_words (`&String`) | Correct, but it relies on `'env: 's`, which reference 7.1 omits and 01 D2/02 §8.2 state (G1). Without it rustc rejects all three. |
| `Scope<'s, 'env, E>`: a scope passed down | pipeline's `spawn_parser<'s>(s, lines: &'s [String], tx)` and `spawn_summer` | Correct: the 02 §8.2 form, borrowing only what the signature ties to `'s`. |
| Capabilities and attenuation | backup (`fs.dir(.., Mode::Read)`, `Mode::ReadWrite`), dedupe (`read_only()`, borrowed `Root` fields), longest_lines (`tasks.limit(n)`), retry (`&Net`, `&Clock`, `sim.net.drop_all()`), handles created with `&Tasks` (`Shared::new`, `sync_channel`) | All correct, with no ambient call. `Net::only` is not exercised. The task's "only able to read below `src`" cannot be a declared-authority guarantee with an `&Fs` parameter (Q11). |
| Simulator | `#[sim_test]` in six programs; `world.file`, `world.args`, `world.stdin`, `world.http` with `Reply::status`/`ok`/`hang`, `elapsed`, `records`, `sim.node`, `node.start`, `faults.at(Fault::crash)`, `run_for`, `node.fs()`; `Root::sim` in plain tests | Correct use throughout. Exposure is to provisional setup types and fault semantics (C2, C4, C8). |
| Attribute contracts | ring (`requires`, `ensures`, `old`, `result`, `invariant!` in a `for` range, a property test with `requires`, `should_panic` on a precondition); `requires` in longest_lines and retry | Correct and strong in ring. No program uses a quantifier or `==>`. The spike's lowering type-checks every clause. |
| Also exercised | `Shared::with`/`with2`/`same` and `par_map` (tally), `sync_channel` and echoes (pipeline), `Arena`/`Id` (deps), `derive(Error)` with `#[from]`, `transparent`, and named `source` (every error enum) | Correct. |

The origins trial left outlives bounds, `use<..>`, and `Scope` untested
(01 D2). This round exercised all three and found no borrow-related error.
The one borrow-related exposure is a reference omission the writer filled
correctly from the reference's prose (G1).

## 5. Reference and design gaps

Reference sections are `design/reference.md` unless marked companion.

- **G1. Reference 7.1, `Scope`'s outlives bound** (borrow-related; hit by
  longest_lines, pipeline, tally). Source: 01 D2; 02 §8.2, §5.4. Fix: add
  `pub struct Scope<'s, 'env: 's, E>  // Copy; 's and 'env invariant` to
  the signature block. Add: "so `'s` is bounded below by the `&Tasks`
  borrow, and a child may borrow anything that outlives it". The oracle
  shows the three programs fail without it.
- **G2. Reference 3.5, `+ use<..>` rules and a method example**
  (borrow-related; scan). Source: 02 §2.3 `Capture`, §5.2. Fix: add
  `fn numbered(&self) -> impl Iterator<Item = (usize, &'a str)> + use<'a>`
  in `impl<'a> Lines<'a>`. Add the rule: "`use<..>` must list every type
  parameter in scope and every lifetime the bounds name, so it cannot be
  written while an argument-position `impl Trait` is in scope; leaving out
  an elided `&self` lifetime releases the receiver's borrow."
- **G3. Channel error shapes and echoes** (other; pipeline). Reference
  7.2/7.3; 03 §6.4, TASK-2, TASK-3. Design question: give `RecvError`,
  `SendError<T>`, and `TrySendError<T>` a cancellation case with
  `is_cancelled()`. For example, `RecvError::{Disconnected, Cancelled}`,
  and a `SendError` that keeps the value and says why. Rust's unit
  `RecvError` and reason-less `SendError(T)` cannot say why a checkpoint
  failed. Reference fix, once decided: state in 7.1 that only an error
  returned after a *cancellation* is an echo; a disconnect is an
  independent failure.
- **G4. Crash semantics of an unsynced `rename`** (other; backup). 03 §9.5,
  D15; reference 6.3, companion 17. Design question: say whether a crash
  before `sync_dir` leaves a renamed target as old-or-new (POSIX
  atomicity, the reading every atomic-replace protocol needs) or can lose
  or tear the entry. Then add atomic replace (`write_synced` a temporary,
  `rename`, `sync_dir`) to reference 6.3 as the pattern.
- **G5. Simulated network cost and `is_transient`** (other; retry).
  Reference 6.7, 6.8; 03 §9.3. Design question: fix `NetError::is_transient`
  (5xx, timeouts, resets, refusals) and the packet count of one simulated
  `Net::get` (or a per-`get` latency range), so tests can bound
  `sim.elapsed()` without guesswork.
- **G6. `Fs::read_dir`'s item type and admitted `DirEntry` methods**
  (other; backup). Reference 6.3 ("as std::fs"); 03 §1.1, ADMIT-1. Design
  question, then reference fix: "`read_dir` yields `io::Result<DirEntry>`;
  `file_name()` is admitted, `metadata()` and `file_type()` are `FX0520`
  (use `fs.metadata(name)`)". Also say whether `OsStr`/`OsString` keep
  their Rust API.
- **G7. Reference 3.6 and 02 §10.9, `Id<T>`'s traits** (other; deps).
  The reference cites 02 §10.9 for `Eq` and `Hash`, but 02 §10.9 lists
  only `Copy`, `Owned`, `Pure`, `Send`, and `Sync`, though its own
  `reachable` example hashes `Id`s. Fix: list `Eq` and `Hash` in 02
  §10.9, and in both places say every listed trait holds for every `T`,
  so a type holding an `Id<Node>` can derive them while `Node` does not.
- **G8. Reference 7.1, scope results consumed by `?`** (other; tally).
  Rust inference, made common by `E: From<TaskError>`. Fix: add
  `tasks.scope(|s| -> Result<(), MyError> { .. })?;` and the sentence:
  "when the scope's result feeds `?` rather than the return value, annotate
  the body's return type, or `E` is ambiguous (E0283)".
- **G9. Reference 6.1 and 6.8, `Stdin` reads and simulator setup types**
  (other; dedupe, retry). 03 §1.1, §9.2. Fixes:
  - say whether `Stdin::read_to_string`/`read_to_end` are inherent
    (implied by the `&self` receivers) or need `use std::io::Read`;
  - give `world.args(impl IntoIterator<Item: Into<String>>)` and
    `world.stdin(impl AsRef<[u8]>)`;
  - say whether a `world.http` peer closure must be `Send`.

  Origins' RS1 asked for part of this.
- **G10. Generated tests over private-field types, and the violation text**
  (other; ring). 04 §2.4 rule 1, CLAUSE-7; reference 10.1, 10.2. Design
  question: do build sequences respect each call's `requires` (discarding
  a violating call), and how are sizes such as `with_capacity(n)` drawn?
  Then fix the violation message prefix that `#[should_panic(expected =
  ..)]` matches, since tests depend on it.
- **G11. Private fields in a `pub` function's clauses** (other; ring). 04
  §1.4 (v1 visibility), CLAUSE-5; reference 10.1 (provisional). Design
  question: may a `pub` method's v0 clause read a private field? The answer
  decides whether `self.len == old(self.len) + 1` is `checked` or must be
  written through `len()` and become `sampled`.
- **G12. Invariant placement in `while` and `while let` loops** (other).
  01 D10; 04 §2.4. Design question: say that a `while`/`while let`
  invariant is checked before each evaluation of the condition (entry is
  before the first test, exit after the failing one), which is what the
  D18 spike does. Add it to reference 10.1.

## 6. Follow-up (bead `fx-jv9`)

| Gap | Disposition |
| --- | --- |
| G1 | Fixed: reference 7.1 prints `pub struct Scope<'s, 'env: 's, E> { .. }`, as 02 §8.2 does. |
| G2 | No change. Reference 3 adopts Rust 2024's `use<..>` rules unchanged, 02 §2.3 and §5.2 state them, and the writer applied them correctly; the core has no room for a restatement. |
| G3, G4, G5, G6, G9 | Filed as bead `fx-i5p` (std, filesystem, network, and simulator-setup decisions in 03). |
| G7 | Fixed: 02 §10.9 lists `Eq` and `Hash`; both it and reference 3.6 say the traits hold for every `T`, and `Owned`/`'static` only when `T` is (round 21, B1). |
| G8 | Fixed in companion 14.1: a row for a scope result fed to `?` into an error type other than `TaskError` (rustc E0283). |
| G10, G11, G12 | Filed as bead `fx-eky` (v0 contract decisions in 04). |

The core reference now measures 24,989 tokens (`scripts/count-tokens.sh`,
2026-10-20), under D14's 25,000.
