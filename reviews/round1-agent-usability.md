# Round 1 Review: Agent Usability

Reviewer stance: an engineer who runs fleets of Claude Code and Codex agents on
production codebases and is paid to make them succeed. The job here is to break
the design, not to praise it.

Inputs: `design/00-first-principles.md` and `design/01-design.md` in full;
`research/05-llm-code-empirics.md` and `research/06-agent-native-tooling.md`;
the relevant parts of `research/02`, `research/04`, and `research/07`; the
v0.3 spec in `spec/language/` (used only as a size data point); and a habit
census I ran over 912 crates in the local cargo registry (method in the
appendix).

The question: will Opus-class agents be more productive, and produce more
trustworthy code, in Ferox than in Rust, TypeScript, or Go, given that Ferox
has no training data?

## Verdict

**request_changes**

Short answer: not as drafted, and not on productivity.

- **Productivity.** A language with no training data starts with a penalty.
  research/05 puts an unmitigated frontier model at 20–50% of its
  mainstream-language performance. D1 is Ferox's main mitigation: "idiomatic
  Rust is a near-correct first draft". The draft then respells much of what
  Rust writers type every day. In my census, 77% of Rust files contain at
  least one construct that Ferox respells even though the meaning is the same.
  The median 100–500-line file has 4 such constructs, or 10 when the
  meaning-changing ones are included (B1). D7 adds a plumbing tax that Rust,
  Go, and TypeScript do not have. It lands on the most common maintenance
  edit: adding a log line or a timeout deep in a call chain (B3). D13 leaves
  out the execution loop, which is the best-evidenced lever in both research
  reports (B6). Ecosystem matters too. Every Rust crate an agent wants has
  ambient authority and needs a wrapper before Ferox code can call it. In
  year one, expect lower task throughput than in Rust, TypeScript, or Go.
- **Trust.** The trust ladder is the right product. As drafted, though, each
  rung has a cheap agent bypass that produces no diagnostic:
  - Bundle structs, `&Root` parameters, and `Root::sim()` in production code
    erase the authority signal (B4).
  - `dbg` becomes the production logger (B3).
  - Vacuous specs pass all three D10 modes and then show up in `ferox review`
    as contracts (B5).

  research/05 and research/04 show that agents take the cheapest path to a
  green check: 9x more `any` than humans in TypeScript, and 2–14% verifier
  cheating without a cheat checker. A trust feature that an agent can disable
  without the toolchain noticing is not a trust feature.

The claim I would accept, and that the design should commit to, is narrower:
*on a fixed benchmark, Ferox task success within 10% of Rust after one release
of tooling, with measurably higher human detection of authority changes and
spec weakening per review minute.* The experiments below test exactly that.

The direction is right: Rust's surface, no async, no user macros,
capabilities, contracts, a trust diff, and deterministic simulation. Every
objection below has a fix that keeps the thesis. That is why the verdict is
not reject.

## Blocking objections

### B1. D1 and D6: the Rust-habit tax is large, and most of it buys nothing

**Problem.** D1 promises that Rust habits produce near-correct Ferox. D6 then
replaces the highest-frequency Rust tokens with new spellings:

- `f"..."` instead of `format!`;
- a built-in `print` instead of `println!`;
- `assert(...)`, `panic(...)`, and `dbg(...)` as functions;
- `widen()`, `narrow()`, and `truncate()` instead of `as`, `into`, and
  `try_into`;
- bounds only in the generic parameter list, so no `where`;
- a closed derive list without `PartialEq` and `PartialOrd`.

The macro-free spellings of `vec!`, `format!`, `matches!`, and `assert_eq!`
are still open (Open Question 6).

D1 itself rejects `[T]` generics and `T?` as "second spellings of things
models already write correctly". The same test condemns most of D6.

The fix-it mechanism does not make the respellings cheap:

- Each habit turns the verdict red.
- A flood of habit diagnostics buries the real errors. In research/06's
  SWE-agent data, 23.4% of unresolved runs failed through cascading failed
  edits.
- Claude Code and Codex do not run `ferox fix` unless told to.

**Evidence.** I measured this over 912 crates, 15,164 files, and 6.08M lines.
Each row shows how often a construct that the draft rejects, respells, or
leaves unspecified appears in Rust code.

| Rust construct | Ferox draft | per 1k lines | % of files |
| --- | --- | ---: | ---: |
| explicit lifetime tokens | rejected (`from`) | 20.2 | 44.2 |
| `assert_eq!` / `assert_ne!` | open (OQ6) | 12.6 | 31.0 |
| `#[derive(PartialEq/PartialOrd)]` | not in the closed set | 2.0 | 29.7 |
| `where` clause | rejected | 4.9 | 29.1 |
| `as` numeric cast | rejected | 6.9 | 26.9 |
| `assert!` | becomes `assert(...)` | 4.4 | 23.6 |
| `.into()` | unspecified | 3.1 | 21.0 |
| inline `mod tests {` | module = file | 0.5 | 20.8 |
| `panic!` / `todo!` / `unreachable!` | functions or open | 1.4 | 16.0 |
| `Type::from(` | breaks if `from` is reserved | 1.7 | 14.4 |
| `vec!` | open (OQ6) | 2.3 | 13.5 |
| `impl From<` | replaced by variant `from` | 1.1 | 12.0 |
| `format!` | becomes `f"..."` | 1.1 | 10.1 |
| `write!` / `writeln!` | no spelling given | 1.6 | 9.8 |

How often files are affected:

| What is counted | Files with at least one | Median per 100–500-line file |
| --- | ---: | ---: |
| Respellings whose meaning is the same | 76.8% | 4 |
| All respellings except lifetime tokens | 83.2% | 6 |
| All respellings | 87.0% | 10 |

"Same meaning" covers `vec!`, `format!`, `write!`, the `assert` family,
`matches!`, the `panic` family, `where`, the `PartialEq` derive, inline
`mod tests`, and `impl From`.

These are library crates. They are heavy on lifetimes and `unsafe` and light
on async, logging, and serde. Service code would score higher on the rows
that D7 affects.

**Example.** A typical first draft from an agent:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Stats { pub count: usize, pub mean: f64 }

pub fn summarize<T>(xs: &[T]) -> Stats where T: Into<f64> + Copy {
    let sum: f64 = xs.iter().map(|&x| x.into()).sum();
    Stats { count: xs.len(), mean: sum / xs.len() as f64 }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mean_of_two() {
        assert_eq!(summarize(&[1u8, 3]).mean, 2.0);
    }
}
```

Under the draft this has at least six errors:

- `PartialEq` in the derive;
- the `where` clause;
- `Into`;
- `as f64`;
- `#[cfg(test)] mod tests`;
- `assert_eq!`.

Only the cast changes meaning, and `usize` to `f64` has no specified Ferox
spelling.

**Smallest fix.**

1. **Reject only where the meaning differs; normalize where it is the same.**
   A same-meaning Rust spelling should parse and check, and pass with an info
   note; `ferox fmt` then rewrites it to the canonical form. Stored code stays
   canonical, and agents do not spend iterations on spelling. This covers:
   - `where` clauses;
   - `PartialEq` and `PartialOrd` in derives;
   - `'a` signatures that fit D2's model (B2);
   - inline `mod tests`;
   - argument-position `impl Trait`.
2. **Keep a closed, non-extensible set of compiler built-in `!` forms with
   Rust's spelling and meaning,** instead of `f"..."`, `print`, `assert()`,
   `panic()`, and `dbg()`. The set is `format!`, `write!`, `writeln!`,
   `vec!`, `assert!`, `assert_eq!`, `assert_ne!`, `matches!`, `panic!`,
   `todo!`, `unreachable!`, and `dbg!`. Users cannot define `!` forms, so
   every objection D6 raises against macros (opacity, caching, supply chain)
   still holds. This also closes OQ6.
3. **Reject with a fix only where the meaning differs:**
   - `println!` and `eprintln!` (ambient I/O, see B3);
   - lossy `as`;
   - `async` and `.await`;
   - ambient std I/O calls;
   - `macro_rules!`;
   - derives outside the closed set.

### B2. D2 and D5: the borrow-return rule is stricter than Rust's elision, and `from` collides

**Problem 1: elision.** D2 says "a returned reference's source is inferred
when exactly one parameter is borrowed". Rust also infers the source from
`&self` when other parameters are borrowed. The most common accessor shape in
Rust compiles with no lifetimes, yet fails in Ferox:

```rust
impl Index {
    fn get(&self, key: &str) -> Option<&Entry> { ... }   // Rust: fine. Ferox draft: needs `from self`.
}
```

3.7% of library files contain this exact single-line shape. That is a lower
bound, because multi-line signatures and view-returning types were not
counted. Here the agent wrote correct, idiomatic Rust and still gets an
error.

**Problem 2: `from`.**

- The keyword has two unrelated meanings: the reference source in D2 and the
  error conversion in D5.
- If `from` is reserved, it breaks `String::from("..")`, `PathBuf::from`, and
  `HashMap::from([..])`. `Type::from(` appears in 14% of files.
- The draft does not say whether `from` is reserved.

**Problem 3: closures and views.** Open Question 2 covers closures that
capture views and views inside generic containers. It decides whether
`xs.iter().filter(|x| x.score > min).map(|x| &x.name).collect()` compiles,
and whether such a chain can be returned. This is the most common idiom in
Rust. It cannot stay open while learnability is being measured.

**Smallest fix.**

- Adopt Rust's three elision rules verbatim, including the `self` rule.
  Require `from` only where Rust would require a named lifetime.
- Accept a Rust lifetime signature that fits the
  one-region-per-borrowed-parameter model, such as
  `fn pick<'a>(a: &'a str, b: &str) -> &'a str`, and normalize it to
  `from a`. Reject only lifetimes the model cannot express. The message
  should name the alternatives: a view, a handle, or `Rc`.
- Make `from` a contextual keyword, valid only after a return type.
- Spell D5's conversion the way thiserror does: `Net(#[from] NetError)`.
  Models have seen this form at very high volume, and it means exactly what
  D5 means. Ship a fix that rewrites a trivial `impl From<X> for E` into the
  attribute.
- Close OQ2 with a rule stated in terms of iterator adapters, and put three
  iterator-chain examples in the reference.

### B3. D7 and D6: logging has no home, `print` contradicts D7, and logging destroys purity

**Problem.**

- D7's capability list is `Fs`, `Net`, `Clock`, `Rand`, `Env`, `Stdio`,
  `Proc`, and `Tasks`. There is no logging, tracing, or metrics story.
- research/07 left logging as an open question (its open question 3). The
  design dropped it.
- D6 makes `print` a built-in free function. D7 says the standard library has
  no free I/O functions, and the Feel example writes `root.stdio.print`. The
  two decisions contradict each other.

Logging is the most frequent cross-cutting edit in service code, and it lands
at the leaves. Under D7, a leaf that starts logging needs a capability
parameter. Every caller up to `main` then changes, and every test call site
changes. The leaf also stops being pure, just because it wants to emit a
warning: it loses eligibility for `const fn`, memoization, `par`, and proofs.

**Example.** The task is "warn when a legacy row format is decoded".

```ferox
fn handle(req: &Request, db: &Db) -> Result<Response, AppError>   // calls load_user
fn load_user(db: &Db, id: UserId) -> Result<User, DbError>        // calls decode_user
fn decode_user(row: &Row) -> Result<User, DecodeError>             // pure; 12 tests call it
```

In Rust or Go this is one line in `decode_user`, in one file. In the Ferox
draft, `decode_user`, `load_user`, `handle`, the router, `main`, and all 12
tests change, across four or more files, and `decode_user` becomes
effectful. research/05 reports that success falls as patches span more files:
Aider went from 17.7% on single-file tasks to 7.9% on tasks touching five or
more files. Agents will route around the cost with `dbg`, which needs no
plumbing and only warns in release builds. `dbg` then becomes the production
logger.

**Smallest fix.**

- **Add an ambient, write-only telemetry facility:** `log::warn(..)`,
  `trace::span(..)`, `metrics::count(..)`.
  - It cannot read anything or block, and it returns nothing that depends on
    the world.
  - It sends to a sink that only `main` configures (`root.telemetry`).
  - Under `Root::sim`, it is captured and can be asserted on in tests.
  - Emitting telemetry does not affect `Pure`, because results still depend
    only on inputs.
  - The lockfile records `emits: log` per package, so a dependency that
    starts logging shows up in the trust diff.
  - A `Secret<T>` type whose `Shape` refuses formatting closes the
    "log a secret" leak.
  - The 2025 crates.io logging attack needed file reads and network access,
    and both remain capabilities.
- **Delete the free `print`.** `println!` gets a fix to `stdio.println(..)`
  in code that holds `&Stdio`.
- **Make `dbg!` an error, not a warning,** in release and CI profiles.

### B4. D7: the reviewability benefit has cheap, silent bypasses

**Problem.** The thesis, which D7 implements, claims that "a signature tells
a reviewer everything a function can do to the world". Four predictable agent behaviors make every
signature say "everything", and none of them triggers a diagnostic.

1. **Pass `&Root` down the call chain.** Nothing forbids it.
2. **Bundle.** For example, `struct Ctx { fs: Fs, net: Net, clock: Clock,
   rand: Rand, stdio: Stdio }`, passed everywhere. This is Rust's
   `AppState` / `Arc<AppState>` idiom and Go's habit of passing a context
   object, so models produce it without being asked. In my experience
   (practitioner observation, not a measurement), it usually arrives as the
   agent's own "cleanup" after the second plumbing task.
3. **Methods on an authority-bearing `self`.** Take
   `impl Service { fn handle(&self, ..) }`, where `Service` holds clients.
   The computed authority of every method is then the authority of all of
   `Service`'s fields.
4. **Mint a sim root in production code.** `Root::sim(0).clock` satisfies a
   signature with no plumbing at all. The draft does not restrict `Root::sim`
   to tests.

After behaviors 2 or 3, a function that newly starts calling the network
changes neither its signature nor its type-level authority. `ferox review`
is then blind to the exact event it exists to show. The evidence that agents
take such paths is direct: 9x more `any` than humans, 383 suppressions in one
agentic migration, and 2–14% verifier cheating without a checker.

**Smallest fix.**

- **Confine `Root`.** `Root` may appear only as `main`'s parameter and in
  `#[test]` functions. It cannot be passed, borrowed into a call, or stored,
  so attenuation happens in `main`.
- **Confine sim constructors.** `Root::sim` and the other sim constructors
  compile only in test code.
- **Report exercised authority per function.** This is the set of capability
  kinds reachable from the body through parameters, fields, and callees'
  summaries, conservative through `dyn` and closures. Put it in module
  interfaces and in `ferox review`. A function that starts using `ctx.net`
  then shows up in the trust diff even though its signature did not change.
  - The cost is a set union per call edge, stored in the interface artifact.
  - Dependents re-check only when the summary changes, and that change is
    exactly the trust-relevant event.
- **Add a least-authority lint.** On any non-`main` function whose held
  authority is strictly larger than its exercised authority, warn, with a
  machine fix that narrows the parameter. research/07 recommended this
  diagnostic (its implication 9); the draft dropped it.
- **Add a threading refactor:** `ferox fix --thread <Cap> --to <fn>`. It is a
  checked multi-file edit that adds the parameter along the shortest call
  path from a holder, including test call sites (which get sim capabilities).
  Leftover plumbing becomes one command instead of many agent iterations.

### B5. D10: "one spec, three uses" will produce weak specs that pass every mode

**Problem.** research/04 found:

- about 9% of specs too weak, even among successful solutions;
- spec generation about 50% sound and complete;
- 2–14% cheating without a checker.

It recommended vacuity checks, executable spec tests, counterexample search,
and spec diffs. D10 adopts trust-hole counting and nothing else from that
list. There are four concrete gaps.

1. **Weak specs.** Agent specs tend to state a shape, not a behavior:

   ```ferox
   pub fn dedup_sorted(xs: &mut Vec<i64>)
       ensures xs.len() <= old(xs.len())
   ```

   An empty body satisfies this in `assert`, `test`, and `prove` modes.
   Nothing in the draft flags it, and `ferox review` will show it to a human
   as a proven contract.
2. **Executability and cost.** The design's own `max_index` invariant,
   `forall |j| j < i ==> xs[best] >= xs[j]`, makes the loop O(n²) when it is
   checked on every iteration in `assert` mode. At n = 10⁵ that is about
   5×10⁹ comparisons, and the test times out. Agents respond by deleting
   invariants. Clauses with unbounded quantifiers over `int` cannot run at
   all, and the draft does not say whether `assert` mode skips them, or
   whether it says so when it does.
3. **Generating inputs from `requires`.** Take
   `fn search(xs: &[i64], x: i64) -> Option<usize> requires is_sorted(xs)`.
   A random 10-element vector is sorted with probability about 3×10⁻⁷.
   Rejection sampling therefore generates nothing, and the property test
   "passes" on zero cases.
4. **Incentive.** Under D13, every spec edit needs approval. The cheapest
   path for an agent is then to write no spec at all.

**Smallest fix.**

- **Classify clauses.** Mark each clause `runtime`, `runtime-costly` (a
  quantifier over a collection), or `proof-only`, and report the counts. Never
  skip a clause silently. In `assert` mode, check loop invariants at loop
  entry and exit by default.
- **Detect weak specs in `ferox check`:**
  - Vacuity by SMT: flag a `requires` that cannot be satisfied, or an
    `ensures` that follows from `requires` alone.
  - Trivial-body test: evaluate `ensures` against synthesized trivial bodies
    (no-op, default value, identity) on generated inputs, and warn when the
    spec still holds.
  - Spec strength: `ferox test --spec-strength` reports the fraction of body
    mutants that each spec rejects. `ferox review` shows that fraction next
    to the spec.
- **Generate inputs properly.** Derive generators for a fixed set of
  precondition forms: ranges, lengths, sorted, distinct, and membership.
  Report how many generated cases were accepted, and fail below a floor.
- **Gate only weakening.** A spec change that is provably stronger (a weaker
  `requires` or a stronger `ensures`) and a brand-new spec are listed in
  review but not gated.
- **Scope the claim.** Specs cannot mention effects, so contracts cover the
  pure core. That is fine, but the design should say so.
- **Fill the gaps in the clause set.** Add `old(..)`, which the example needs.
  Infer quantifier triggers, and reject only when none can be inferred:
  agents fail at writing triggers by hand in Verus.

### B6. D13: the toolchain contract omits the execution loop

**Problem.** The evidence for execution feedback is the strongest in the
research:

- research/05's strongest inference-time result is execution feedback. The
  ILA-agent took Claude Sonnet 4.5 on Cangjie from 45.2% to 81.9%, and the
  authors call execution-based verification central.
- research/06 rates the fast feedback loop as the best-supported feature, and
  its recommended contract includes a test command with seeds, shrunk
  counterexamples, and cached results.

D13 specifies `check`, typed holes, `doc`, `explain`, `fix`, `review`, and
hashes. It has no output contract for `test`, `run`, or `verify`.

Deterministic simulation is Ferox's best testing feature, and it has no
debugging story. A failing sim seed with no event trace is a black box. In my
experience (practitioner observation), simulation testing is only as useful
as its traces.

**Smallest fix.** Add these to D13:

- **`ferox test [filter] [--changed] [--seed N] [--json]`.** For each test,
  report:
  - expected and actual values, with a structural diff;
  - the panic message and its `.fx` span (never a span in the emitted Rust);
  - the seed and a shrunk counterexample;
  - the exact command that replays the failure.

  Cache results by definition hash.
- **`ferox test --trace`.** For a sim test, print the ordered capability event
  log for the failing seed: time advances, messages, task scheduling, and
  cancellations. Make it diffable against a passing seed.
- **`ferox run` and `ferox eval '<expr>'`.** `eval` evaluates one expression
  in a module's scope on the reference interpreter. In a language with no
  priors, it answers "does `narrow` return `Option`?" in one call instead of
  a write-and-check cycle.
- **`ferox verify --json`.** Give one status per obligation: `proved`,
  `refuted` (with a counterexample), `timeout`, `assumed`, or `trusted`.
- **Coverage of lines and of contract clauses,** so that `assert` mode shows
  which specs were actually exercised.
- **Latency targets for a fresh process.** State that the incremental
  latency targets hold for a fresh CLI process, through an on-disk cache or
  an auto-started daemon. Agents start a new process for every check.

### B7. Hard constraint 7 and the learnability target: unmeasured, and likely violated

**Problem.**

- The draft names three documents: a 25k-token core reference (hard
  constraint 7), an always-on card under 8 KB, and "a full reference that
  fits in context". None of them exists, and nothing has been counted.
- One data point exists. The v0.3 language spec in `spec/language/` is
  88.7 KB, roughly 21–23k tokens. It covered ownership, types, effects,
  async, errors, syntax, modules, and traits, but not contracts, `Shape`,
  `par`, simulation, views, or holes.

My estimate for this design, written at the same density:

| Area | Approximate tokens |
| --- | ---: |
| Items, expressions, patterns (as a delta from Rust) | 2,500 |
| Scalars, conversions, overflow | 1,500 |
| Ownership, views, `from`, field-borrow declarations, closures, `Arena`, `Rc` | 4,000 |
| Traits, generics, coherence, `dyn` | 2,000 |
| Errors, `?`, panics, task boundaries | 1,000 |
| Modules, visibility, imports, manifest grants | 1,500 |
| Derives, `Shape`, `const fn`, codegen, built-in forms | 2,000 |
| Capabilities: `Root`, kinds, attenuation, `Pure`, simulation | 2,500 |
| Tasks, scopes, cancellation, channels, `lock`, `Send`/`Share`, runtimes | 3,000 |
| `par`, `join`, `par_map`, `par_reduce` | 1,000 |
| Contracts: clauses, spec types, quantifiers, ghost code, lemmas, triggers, modes, trust holes | 5,000 |
| `unsafe`, FFI, Rust interop | 1,500 |
| Tests, attributes, typed holes | 1,000 |
| Worked examples (the part research/05 says matters most) | 5,000–8,000 |
| **Total** | **about 33,000–36,000** |

The standard library comes on top of that.

- **The budget leaves out the std, where the risk is.** Hallucinated APIs are
  the second half of the low-resource penalty (research/05, section 9).
- **The I/O half of Ferox's std differs from Rust's everywhere:** there is no
  `std::fs::read_to_string`, no `Instant::now`, and no `env::var`.
- **The design already invents names it does not need.** The Feel example
  uses `Duration::seconds(5)` where Rust has `Duration::from_secs(5)`.
- **The learnability target has no number.** "High first-check pass rate …
  within a few check iterations" cannot fail.

**Smallest fix.**

- **Write the reference as a delta from Rust.** Include an explicit list of
  every Rust feature that is absent or respelled. Otherwise models assume
  that anything unmentioned works as it does in Rust.
- **Split the budgets:**

  | Document | Budget |
  | --- | ---: |
  | Always-on card (habit table and commands) | ≤ 2k tokens |
  | Core delta reference | ≤ 20k tokens |
  | Prove-mode chapter, loaded only for `prove` and `test` packages | ≤ 6k tokens |
  | Std index | ≤ 4k tokens |

  Count them with the production tokenizer in CI.
- **Deliver docs with diagnostics.** Each diagnostic should carry the one to
  three reference paragraphs it concerns. Agents always read check output,
  but they do not reliably load docs on demand: Vercel's skill was not
  invoked in 56% of cases.
- **Std naming rule:** use the Rust std name wherever the concept exists.
- **Replace the qualitative target with numbers, and freeze D1 and D6 syntax
  only after E1 meets them.** Proposed thresholds:
  - at least 60% first-check pass rate with the card in context;
  - a median of at most 3 check iterations to green;
  - a final hidden-test pass rate of at least 90% of the Rust control on the
    same tasks.

## Non-blocking issues

### D1 ledger: each Rust difference, keep or change

| Difference | Rust prior that misleads | Fails loudly? | Recommendation |
| --- | --- | --- | --- |
| `&` second-class; views instead of structs with lifetimes | `struct Parser<'a> { src: &'a str }`; a `Vec<&T>` stored in a long-lived struct | Yes | Keep. It removes a class of hard errors. Needs B2. |
| No lifetime syntax | `'a` in 44% of library files | Yes | Keep the semantics; accept and normalize `'a` signatures that fit (B2). |
| No user macros | `macro_rules!` in 12% of library files, often to deduplicate tests | Yes | Keep. Put a table-driven test idiom in the reference so agents do not need macros for test tables. |
| Built-in `!` forms respelled (`f"..."`, `print`, `assert()`, `panic()`) | `format!`, `assert_eq!`, `vec!`, `write!` in 10–31% of files | Yes | Change (B1). |
| No `as` | 6.9 casts per 1k lines; 27% of files | Yes | Keep for lossy casts. Accept lossless `as`, whose meaning is identical. See D3 below. |
| Capabilities as parameters | `fs::read_to_string`, `Instant::now`, `env::var`, `thread::spawn`: 8.6% of library files, 33% of a small sample of binary crates | Yes | Keep, with B3 and B4. Ship fixes that rewrite common ambient calls into capability calls when a capability of that kind is in scope. |
| `from` on enum variants | `impl From<X> for E` in 12% of files; thiserror's `#[from]` | Yes | Keep the semantics; use the `#[from]` spelling (B2). |
| `print` as a built-in | `println!` | Not applicable | Remove (B3). |
| `assert` as a function | Rust's `assert!` runs at run time; Dafny's and Verus's `assert` is a proof step | No: one word, two meanings | Keep `assert!` for run-time checks. Use a bare `assert expr;` statement only in spec and proof contexts. |
| Module = file | inline `mod tests { use super::*; }` (21% of files); `mod foo;`; `pub use` re-exports | Yes | Keep module = file. Accept inline `mod tests` or fix it. Drop file-level acyclicity (D11 below). |
| Impls live in the type's or the trait's module | `impl Display` in a separate `fmt.rs` | Yes | Keep: it turns "where is `impl X for Y`" into a grep of two files. State the rule for inherent impls. |
| Bounds only in the generic parameter list | `where` in 29% of files | Yes | Normalize instead of rejecting (B1). |
| Closed derives without `PartialEq` and `PartialOrd` | `#[derive(PartialEq)]` in 30% of files | Yes | Accept them as aliases and normalize. Specify what `Eq` means for float fields. |
| No async | `async fn`, `.await`, `tokio::spawn` | Yes | Keep. This is the draft's biggest win for agents. Ship a safe fix that strips `async` and `.await` and maps `tokio::join!` and `spawn` to the scope API. |
| `lock` blocks over shared cells | `Arc<Mutex<T>>` with `.lock().unwrap()` | Yes | Keep the construct. Do not call the cell `Cell`: Rust's `Cell` is single-threaded and has no lock. This is the draft's one case of familiar syntax with a different meaning. |
| Std names | `Duration::from_secs(5)`, where the Feel example writes `Duration::seconds(5)` | Yes | Change: use Rust's std names wherever the concept exists. |
| `result` in `ensures` | `let result = ...` is a very common local name | Maybe not | Make it a contextual keyword, valid only inside `ensures`. |
| Typed holes `_?name` | `todo!()` is Rust's hole | Not applicable | Keep `_?name`, and also treat `todo!()` as an anonymous typed hole. Agents write `todo!()` without being prompted. |

Almost every difference fails loudly. That discipline is good, and rarer than
it should be in new languages.

### D2

- Methods that declare the fields they borrow need a concrete spelling. Put
  that spelling into E1.
- "An escaping closure is `move`" differs from Rust, which can box or return
  a closure that captures nothing without `move`. Match Rust.

### D3

- `truncate()` collides with `Vec::truncate(n)` and `String::truncate(n)`,
  which shorten a collection, and is easy to confuse with `f64::trunc`.
- `narrow()` returns `Option`, while Rust's `try_from` returns `Result`. That
  is a counterfactual difference with no benefit.
- `.widen()`, like `.into()`, depends on the target type being inferred. That
  is Rust's "type annotations needed" error class.
- Recommendation: use built-in associated functions that name the target
  type: `u64::from(x)` (lossless), `u8::try_from(x)` (returns `Result`), and
  `u8::wrapping_from(x)`. Make these built-ins, not a user trait.
- Specify float-to-int, int-to-float, and `char`-to-`u32` conversions.

### D4

Five things need to be specified:

- argument-position `impl Trait` (9.5% of files), which should be normalized;
- return-position `impl Trait` (4.5% of files), which iterator-returning
  functions need;
- operator traits (`Add`, `Mul`, `Index`, `Neg`), since "no custom operators"
  is ambiguous about `impl Add for Vec2`;
- how a user type prints inside a format string (`Display` or `Shape`);
- `Iterator` and `IntoIterator` for `for` loops over user types.

### D5

- Specify how a test expects a panic, the equivalent of `#[should_panic]`.

### D6

- Service code is full of `#[derive(Serialize, Deserialize)]` with
  `#[serde(rename, default, skip, flatten)]`. Serialization based on `Shape`
  needs a closed set of field attributes for those cases, or agents will hand
  write encoders.
- Decoding through `Shape` must go through a validating constructor, so that
  JSON cannot produce a value that breaks its type's invariant (OQ3).

### D7

- Attenuation scope is dynamic: `net: &Net` does not tell a reviewer which
  hosts it can reach. `ferox review` should show where each capability value
  reaching a function was attenuated in `main`.
- On Open Question 7: yes. Allow `#[pure]` to be declared and checked on
  public functions, and store exercised authority in the interface (B4).
- `Fs` with a dynamic `Mode` hides read versus write from the type. Reviewers
  care about that difference. Revisit the granularity of capability kinds
  (research/07, open question 1).

### D8

- With B3's telemetry sink, logging inside a `lock` body stays possible. The
  sink must not block.
- Rename the shared cell (see the ledger).

### D10

- Show the pre-state and post-state model for `&mut` parameters with worked
  examples.

### D11

- **File-level acyclicity is stricter than both Rust and Go.** Rust allows
  module cycles inside a crate, and Go allows cycles between files in a
  package. Under the draft, types that refer to each other across files
  (`user.fx` and `order.fx`, or AST and types) must be merged into one file.
  Agents that split large files will keep hitting this, and they will respond
  by writing giant files. Make the package or directory the acyclic unit, and
  compile strongly connected groups of modules together.
- **Unused imports as errors** cause friction in the middle of an edit; this
  is Go's most complained-about rule. Have `ferox fmt` add and remove imports
  the way goimports does. Make the check a warning in the development profile
  and an error in CI.
- **There is no conditional compilation.** `#[cfg(..)]` appears in 55% of
  library files, for platforms, features, and tests. Specify a story for
  target-specific modules and test-only items.
- **Open Question 1 (visibility):** provide a `pub(crate)` equivalent. Agents
  write `pub(crate)` often.

### D12

- `rustc` errors on emitted code must never reach the agent, and panic
  backtraces must map to `.fx` spans. Otherwise agents will "fix" the
  generated Rust.
- Mixed Rust and Ferox workspaces strengthen Rust priors: the agent has just
  read five Rust files. E1 should include a mixed-workspace arm.
- Every Rust crate called from Ferox has ambient authority. It needs an
  `ffi`-class grant and shows as "unverified authority" in the trust surface.
  Say so, and plan wrapper generation, because this is the main throughput
  cost against Rust.

### D13

- **Output format.** research/06 recommends concise text by default and
  `--json` on request. NDJSON by default costs tokens in every iteration.
  Test it (E7).
- **Flood control.** D13 leaves out research/06's `root` and `cascade_of`
  fields, `--max`, and grouping. Group same-code habit diagnostics into one
  line with a count and a single `ferox fix` command.
- **Typed holes.** Agents use unfamiliar tools only when prompted. Report the
  expected type of each `todo!()` in normal `check` output, so the benefit
  arrives without the agent knowing the feature exists.
- **Snapshot-hash protocol.** Claude Code and Codex edit files with their own
  tools, so the protocol matters only for `ferox fix` and an MCP path. It is
  a lower priority than B6.
- **Protected tests.** Give agents a legitimate way to dispute a protected
  test or spec: an attribute that fails CI and records the reason. In
  ImpossibleBench, Claude models rarely aborted and edited tests instead.
- **`ferox explain`.** Test each page in CI: its example must fail before the
  fix and pass after it.
- **`ferox doc`.** Needs std search by name and by type. In the Hazel study,
  type definitions were the most valuable context.
- **Benchmarking.** Add `ferox bench` with text output. Release-performance
  claims invite optimization tasks.

## Proposed experiments

All experiments share one harness:

- three fixed models: one Opus-class, one Sonnet-class, and one GPT-class;
- two fixed agent harnesses: Claude Code headless and Codex CLI;
- fixed token and time budgets;
- a published task set (Ferox-Bench), so every release reruns E1, E3, and E5.

### E0. Prior-collision census (no compiler needed; can run now)

- **Protocol.**
  - Give each model only `design/01-design.md` (later, the card) and 40 tasks:
    - 20 translated HumanEval or MBPP problems;
    - 10 CLI tasks using `Fs`, `Env`, and `Stdio`;
    - 10 service tasks needing `Net`, `Clock`, logging, and retries.
  - Collect three samples per task.
  - Classify every construct that violates the design, using a regex census
    like the one in the appendix plus manual review.
- **Metrics.**
  - Violations per 1k lines, by category.
  - Share of programs with zero violations.
  - Constructs the design leaves unspecified, where the agent had to guess.
- **Decision rule.**
  - Any category above 1 per 1k lines of agent output gets normalization or a
    design change.
  - Every construct the agent had to guess gets specified before E1.

### E1. Learnability A/B (needs a parser, checker, and interpreter)

- **Protocol.** Run the E0 tasks with hidden tests in three arms:
  - (a) the draft's D1 and D6, rejecting habits with a fix-it;
  - (b) accept-and-normalize plus the built-in `!` forms;
  - (c) a Rust control.
- **Metrics.**
  - First-check pass rate.
  - Check iterations to green (median and 90th percentile).
  - Tokens and wall time to completion.
  - Final hidden-test pass rate.
  - Error categories: syntax, type, ownership, authority, and hallucinated
    API.
- **Decision rule.**
  - Adopt (b) if it raises the first-check pass rate by at least 15 points or
    cuts median iterations by at least one.
  - Ship to users only when the best arm reaches 90% of Rust's final pass
    rate.

### E2. Reference size and form

- **Protocol.** Run fixed tasks with four kinds of context:
  - the card only (2k tokens);
  - the card plus the delta reference (about 15k);
  - the full reference (about 30k);
  - examples only, at each of those token counts.
- **Metrics.** As in E1.
- **Purpose.** Find the knee of the curve. This tests research/05's "examples
  beat prose" for a whole language and sets the real budget for constraint 7.

### E3. Capability plumbing and cheating

- **Protocol.**
  - Build the same service in Ferox, Rust, and Go: 6–8k lines, with routing,
    handlers, services, repositories, and clients at a call depth of at least
    six, and about 150 tests.
  - Write 25 change tasks taken from real backlogs: add a leaf warning, a
    timeout, retry jitter, a cache TTL, a metric, or an endpoint; fix a bug;
    test a leaf.
  - Run each task five times per language with fresh agents.
- **Metrics.**
  - Success, files touched, signature changes, iterations, tokens, and wall
    time.
  - A cheat census of the Ferox diffs:
    - `&Root` parameters;
    - new bundle structs;
    - `dbg` outside tests;
    - `Root::sim` outside tests;
    - authority held but never exercised.
- **Decision rule.** After the B3 and B4 fixes:
  - median files touched for the leaf-logging and leaf-timeout tasks is at
    most twice Rust's;
  - the cheat rate is under 5% of tasks.

  Otherwise reopen D7, including research/07's inferred `with` clauses for
  private functions.

### E4. Does the trust diff help humans?

This is the only experiment that tests the thesis directly.

- **Protocol.**
  - Take 40 agent-written PRs from E3, with Ferox and Rust versions of the
    same changes.
  - Inject a defect into half of them:
    - a deep function that starts using the network;
    - a widened filesystem scope;
    - a weakened `ensures`;
    - a new `assume`;
    - a dependency upgrade that adds authority.
  - At least eight reviewers each get 10 minutes per PR, randomized across
    three arms: the Rust diff, the Ferox diff, and the Ferox diff plus
    `ferox review`.
- **Metrics.** Detection rate, false positives, and minutes per PR.
- **Decision rule.** Ferox with `ferox review` must at least double the
  detection rate for authority and spec defects, at equal or lower review
  time. Otherwise D7's cost is not paying for itself.

### E5. Spec quality

- **Protocol.** Use 40 pure-core functions (parsers, pricing rules,
  collections, state machines) with hidden expert specs and tests. Agents
  implement each function and write its contracts, with and without the B5
  tools. Follow-up tasks then change behavior.
- **Metrics.**
  - Vacuity rate and trivial-body pass rate.
  - Mutation score of the agents' specs.
  - Agreement with the expert specs, by SMT implication in both directions
    where decidable.
  - Proof success under `prove`.
  - Slowdown in debug builds.
  - Share of clauses that are proof-only.
  - Trust holes per 1k lines.
  - In the follow-up tasks, the rate at which agents weaken specs, with
    approval required for all spec edits versus only for weakening.
- **Decision rule.** Agent specs must reach a mutation score of at least 0.7
  with the tools before `ferox review` presents specs as evidence of trust.

### E6. Toolchain ablations

- **Protocol.** Use 12 seeded bugs in sim tests (timeouts, cancellation
  ordering, retry storms) and 12 ordinary failing tests. Compare four arms:
  - pass or fail only;
  - structured `ferox test --json`;
  - structured output plus `--trace`;
  - all of that plus `ferox eval`.
- **Metrics.** Success, iterations, and tokens.
- **Decision rule.** Ship first whatever raises success by 10 points or more.

### E7. Diagnostic format

- **Protocol.** On the E1 tasks, compare a compact text default with an
  NDJSON default, each with and without root and cascade grouping.
- **Metrics.** Tokens per iteration, fix rate per error code, and iterations
  to green.
- **Decision rule.** Make the winner on iterations to green the default, and
  keep the other format behind a flag.

### E8. Latency sensitivity

- **Protocol.** Inject check latencies of 50 ms, 300 ms, 1 s, 3 s, and 10 s
  into E1 and E3.
- **Metrics.** Wall time, iterations, and success.
- **Purpose.** research/06 says this threshold has never been measured. The
  result decides how much type-system power is worth giving up to hit D11's
  100 ms target.

### E9. Convergence

- **Protocol.** Make 10 independent runs of each E1 task in Ferox and in
  Rust.
- **Metric.** The pairwise normalized AST edit distance after formatting.
  This tests the "one obvious way" claim (agent-loop parameter 4).
- **Decision rule.** If Ferox is not lower than Rust, drop the low-entropy
  claim from the pitch.

## Strengths worth keeping

- **No async, no futures, no `Pin`; suspension happens only through
  capabilities (D8).** This removes Rust's worst agent failure class: `Send`
  bounds across `.await`, `Pin`, and choosing an executor.
- **A small, explicit type system (D3, D4).** Mandatory item signatures,
  inference local to function bodies, no overloading, no implicit
  conversions, definition-checked generics, and a hard ceiling on trait power
  all target trait-bound and type-mismatch errors. These are the Rust errors
  agents repair least well: 58.7% for trait failures even with guidance.
- **No ambient authority, `Root` only in `main`, and the lockfile's
  per-package authority diff (D7).** For dependencies, this is a real and
  cheap supply-chain control. Capslock found that fewer than 2% of updates
  add a capability, so an authority change is a high-signal event.
- **Deterministic simulation from a sim root (D7, D8).** Deterministic tests
  are what let agent loops converge. In my experience (practitioner
  observation), flaky tests waste more agent iterations than anything else.
  This may be Ferox's most valuable feature for agents; B6 is about making
  failures debuggable.
- **Errors as values (D5).** Results are must-use, `?` propagates, and every
  conversion is visible in one place.
- **Overflow traps in every profile (D3).** In research/05, debug-mode
  overflow checks turned 43.9% of Rust's runtime errors into visible panics.
- **Counted trust holes, protected oracles, and `ferox review` (D10, D13).**
  These match the ImpossibleBench and VeruSAGE findings.
- **Canonical text, a formatter with no options, and compiler-owned
  migrations through `ferox fix` (D13).** This is the lesson of Go's
  `go fix`.
- **A reference interpreter first (D12).** Agents get an execution loop before
  the native backends exist.
- **Fix-its for Rust habits (D1).** The instinct is right; B1 changes only the
  mechanism.
- **Loud failures.** Almost every Ferox difference from Rust fails loudly.
  The draft has only one case of familiar syntax with a different meaning
  (`Cell`) and one ambiguity (`assert`).

## Appendix: habit census method

- **Corpus.** The newest version of each crate in
  `~/.cargo/registry/src/index.crates.io-*`. I excluded crates whose names
  match bindings, `-sys`, `proc-macro`, `syn`, `quote`, `libc`, or Unicode
  tables. That left 912 crates.
- **Files.** All `.rs` files under each crate's `src/`, keeping files of at
  least 20 lines and at most 400 KB: 15,164 files and 6,083,149 lines.
- **Counting.** One regular expression per construct, counting tokens rather
  than AST nodes. Explicit lifetimes count every occurrence. The elision row
  matches only single-line signatures that have `&self`, another `&`
  parameter, and a `&` in the return type, with no `'`.
- **Union metrics.** Computed per file. "Same meaning" covers `vec!`,
  `format!`, `write!`, `assert!`, `assert_eq!` and `assert_ne!`, `matches!`,
  the `panic` family, `where`, the `PartialEq`/`PartialOrd` derive, inline
  `mod tests`, and `impl From`. The full union adds lifetimes, `as` casts,
  the print family, `async`/`.await`, ambient I/O calls, log and tracing
  calls, and derives outside the closed set.
- **Caveats.**
  - These are library crates, so they over-represent lifetimes, `unsafe`, and
    `cfg`, and under-represent async, logging, and serde.
  - The regular expressions have false positives (for example `where` in
    comments) and miss multi-line signatures.
  - A subset of 12 binary crates (30k lines) showed ambient I/O in 33% of
    files and inline `mod tests` in 33%.
  - The numbers are for direction and rough magnitude, not precision.
