# Property tests, requires-guided generation and generated contract tests (research for fx-m72.1, .2, .3)
Drafted by Fable on 2026-11-10. Decisions are recorded on fx-m72.1, fx-m72.2 and fx-m72.3.

Research report, 2026-11-10. Read-only: nothing in the repository or `br` was changed, and no
build was run. Held-out rule kept: of `trials/e1-checked/` only `protocol.md` §§4.1 and 7.2 and
the harness code that fx-m72.3 names (`harness.py`, `executable_tests` and `test_command`) were
read.

Claims marked **[repo]** come from the checkout at `b153512`. Web claims carry a URL. Most web
evidence is from search-result excerpts, not full reads; the two claims I state from memory are
marked as such.

## 0. Facts that constrain every choice

- **[repo]** `#[test]` is today Rust's built-in attribute, re-exported by the production prelude
  (`crates/tide-std/src/lib.rs:58`). Test binaries are driver-built with `--test` and use
  rustc's libtest harness. There is no `tide` CLI crate; `crates/` holds only `tide-admit`,
  `tide-macros` and `tide-std`.
- **[repo]** The E1 harness lists tests with `<bin> --list --format=terse`, then runs each one
  alone with `--exact <name> --test-threads=1 --logfile …` under a 10-second limit, inside a
  120-second run (`harness.py:325-347`; protocol §4.1). So every property test and every
  generated contract test has to be an ordinary libtest entry, and one test's generation plus
  shrinking has to finish in well under 10 seconds.
- **[repo]** For acceptance builds the harness requires the listing to equal the frozen list
  (`harness.py:333-340`). Generated contract tests in that listing would fail the comparison
  (§1.8).
- **[repo]** The admission walk treats as toolchain code only core's `#[test]` descriptors, the
  harness `main`, and the `#[tide::main]` entry (`crates/tide-admit/src/walk.rs:317`). Any new
  expansion that calls hidden std runtime items needs that rule widened, or has to call public
  std API only.
- **[repo]** `tide-std` already uses `catch_unwind` internally (`task.rs`), so the runner can
  catch a failing case in process although user code cannot (`FX0701`).
- **[repo]** `std::rand` holds only the `Rand` identity; `Pcg64` is not implemented.

## 1. Design choices

### 1.1 Shape of a parameterised `#[test]`

Options:

- **A. A std-exported `test` attribute macro** replaces the prelude's re-export of core's. With
  no parameters it re-emits the item under `#[::core::prelude::v1::test]`. With parameters it
  emits a parameterless core test whose body calls the runner with a closure that draws each
  argument and calls the original function, kept as an inner item. `#[tokio::test]` and
  proptest's `proptest!` use this shape: a macro wrapper around an ordinary test
  (https://docs.rs/crate/proptest/latest).
- **B. The driver rewrites tests** after parsing. rustc's `Callbacks::after_crate_root_parsing`
  takes `&mut ast::Crate`, but submodules are not parsed yet at that point
  (https://doc.rust-lang.org/nightly/nightly-rustc/rustc_driver_impl/trait.Callbacks.html).
- **C. A custom test framework.** Unstable, and the harness depends on libtest's flags.

**Recommendation: A.** It is the existing pattern (`tide::main` is the same kind of macro),
keeps libtest and so the harness's listing, and needs no driver change for built-in types. Three
details the implementer must settle:

1. *Clause visibility.* The test macro sees a `#[requires]` only when it expands first, that
   is, when `#[test]` is written above it. fx-mhd's `requires` macro should therefore, when it
   finds `test` or `sim_test` among the item's remaining attributes, re-emit the item with
   itself moved below that attribute. Otherwise attribute order silently turns guided
   generation off.
2. *Argument types.* Dispatch on the parameter type through the `Arbitrary` trait; `&T` and
   `&mut T` generate an owned `T` and borrow it; `&str` comes from a `String`; `&[T]` from a
   `Vec<T>`.
3. *Admission.* Either widen `generated_by_toolchain` to the std-exported `test` and
   `sim_test`, or give the runner a public std API so the expansion is ordinary admitted test
   code. The second leaves no new trust rule in the walk and is the cheaper review; prefer it.

**Structs and enums without a derive.** Reference §10.2 says structs and enums "whose fields
can all be generated" are generated; 04 §2.4 says they "derive a generator". A proc macro on
the test function sees the name `Point`, not its definition, and Rust has no reflection. The
options:

- **A1. Require `#[derive(Arbitrary)]`.** Cheapest. It reads the reference narrowly: if a
  derive were needed, "or that implement `Arbitrary`" would already cover it.
- **A2. The driver attaches a hidden derive to every struct and enum under `--test`**, by
  editing source text in its `FileLoader`. Single pass and purely syntactic. A field that
  cannot be generated then has to become a run-time "ungeneratable" result through
  autoref-style dispatch, which does not work inside generic code.
- **A3. The driver synthesises impls from types.** In `after_expansion` it finds the ADTs that
  test parameters reach and that lack an `Arbitrary` impl, stops, and recompiles with impls
  appended to the module that defines each type (a child module can name its ancestors'
  private items).

**Recommendation: A3**, built in fx-m72.1 in its smallest form. fx-m72.3 needs the same pass in
any case: private-field types built through public constructors, the `i64`-else-`String`
instantiation, and "unmet bounds" all need type information that no macro has (§1.7). One
mechanism then serves both beads. If the spike in §3 fails, fall back to A1 and record the
narrowed reading on the bead as an owner-visible deviation. This is the largest cost risk in
fx-m72.1.

### 1.2 Generation and shrinking model

| Model | How it shrinks | Evidence |
| --- | --- | --- |
| Type-based (QuickCheck, Rust `quickcheck`) | `shrink(value)` per type, independent of generation | The Rust trait's `shrink` returns a boxed iterator and defaults to no shrinking (https://docs.rs/quickcheck/1/quickcheck/trait.Arbitrary.html). MacIver: shrunk values can break the invariants the generator established, and failures get harder to read (https://hypothesis.works/articles/integrated-shrinking/). |
| Value tree (proptest), rose tree (Hedgehog) | The generator returns a value with its shrink states | proptest's README: QuickCheck shrinks statelessly from the output value, proptest "must hold on to all the intermediate states", and generating complex values can be up to an order of magnitude slower (https://docs.rs/crate/proptest/latest). Rose-tree shrinking across monadic bind can stop at a local minimum (https://well-typed.com/blog/2019/05/integrated-shrinking/). |
| Choice sequence (Hypothesis, falsify, Hegel) | Record the draws; shrink the recorded sequence; regenerate through the same generator | "Instead of applying test-case reduction externally to generated test cases, we apply it internally, to the sequence of random choices made during generation", ordered shortlex (https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ECOOP.2020.13). falsify shrinks the PRNG samples and so works across bind (https://well-typed.com/blog/2023/04/falsify/). Hegel's Rust crate is the same engine, with a pure-Rust engine still marked experimental (https://docs.rs/crate/hegeltest/0.14.9). |
| Byte stream (`arbitrary`, bolero) | A fuzzer minimises the raw bytes | `arbitrary` turns fuzzer bytes into values and has no shrinker of its own (https://docs.rs/arbitrary/); bolero drives generators from bytes and replays corpus files (https://camshaft.github.io/bolero/print.html). |

One 2026 experience report finds QuickCheck's structural shrinking usually faster and
competitive on counterexample quality (https://arxiv.org/pdf/2608.09935; I saw only the search
summary). That is a cost argument against internal shrinking, not a correctness one.

**Recommendation: an owned choice-sequence engine with typed choices.** Reasons specific to
Tide:

1. 04 §2.4 item 4 requires that "shrinking keeps the `requires` true". With a choice sequence
   every shrink candidate is regenerated through the same constrained generators and then
   rechecked against the full `requires`; a candidate that fails it is discarded. A value-tree
   design needs a constraint-preserving shrinker per shape.
2. A user's `Arbitrary` needs one method, `fn arbitrary(g: &mut Gen) -> Self`, and no shrinker.
   That is the smallest surface for a writer, and a user generator's invariants survive
   shrinking.
3. "Shrinking keeps the seed" becomes trivial: the simulator seed is outside the sequence
   (§1.6).
4. Private-field values built from call sequences (§1.7) shrink with no extra code: deleting
   choices deletes calls.

Minimum engine, about 600 to 900 lines in `tide-std` by my estimate:

- `Gen` holds a PRNG and a `Vec` of typed choices (integer in a range, bool, float bits). In
  replay mode it reads choices back and clamps each into the requested range.
- Collections draw a "one more element?" bool per element, so deleting one element's span
  leaves the rest aligned. This is Hypothesis's layout
  (https://github.com/HypothesisWorks/hypothesis/blob/master/guides/strategies-that-shrink.rst).
- Scalars cover the whole type, biased toward small values and `0`, `1`, `MIN`, `MAX`, as 04
  §2.4 item 1 fixes. Draw a bias class first, then the value, with the simplest class at
  choice 0 so shrinking moves toward it.
- Three shrink passes run to a fixpoint under an evaluation budget: delete spans, largest
  first; minimise each integer by binary search; replace a choice with its simplest value.
  Accept a candidate only if it still fails with the same failure kind and is shortlex
  smaller.

**Dependency gate (AP-DEPENDENCY-001).** The capability is generation with shrinking, which
becomes part of the std contract.

- proptest: persistence writes regression files next to the sources
  (https://proptest-rs.github.io/proptest/), but the harness sandbox hides package sources from
  the test. Its `Strategy` trait would become std API, and its shrinker would have to be
  taught to preserve `requires`.
- `quickcheck`: type-based shrinking, rejected above.
- `arbitrary` and bolero: no shrinker without a fuzz engine.
- hegeltest: its default engine needs a Python server.

Owned code wins on API stability and net complexity. Replacement cost is low, because the
user-visible surface is one trait and one method. The PRNG should be a private SplitMix64
until `std::rand`'s `Pcg64` exists (https://prng.di.unimi.it/splitmix64.c, cited from memory);
when `Pcg64` lands, `std::rand` should own the one generator.

### 1.3 Seed format, determinism and reporting

Evidence:

- Hypothesis prints a seed for `@seed` or `--hypothesis-seed`. Separately,
  `@reproduce_failure` takes an opaque blob that fails on another library version and is meant
  to be temporary (https://hypothesis.readthedocs.io/en/latest/reproducing.html). Its docs warn
  against relying on the example database for correctness and say to pin inputs with `@example`
  (https://hypothesis.readthedocs.io/en/latest/tutorial/replaying-failures.html).
- proptest takes fresh seeds per run and relies on the committed `proptest-regressions`
  directory (https://proptest-rs.github.io/proptest/).
- The design fixes the surface: `tide test [filter] [--seed N]`, "reporting each failure's
  seed and replay command" (reference §13).

Recommendation:

- **One `u64`, printed as decimal** so it pastes into `--seed N` and into a `u64` literal.
- **Deterministic by default.** The case seed is
  `mix(run_seed, hash(test path), case_index)`, and `run_seed` defaults to a constant. A fresh
  seed per run would make `./test`, and so E1's final-pass grade (protocol §7.2), vary between
  runs of one snapshot. The cost is that repeated runs explore nothing new; `--seed` varies it
  on purpose. The design does not say which default applies, so record this on the bead.
- **The reported seed is the failing case's seed.** Replay generates that one case and shrinks
  again. For the acceptance line "rerunning that seed reproduces it" to hold for the shrunk
  input, shrinking must be a pure function of the case: budget it by evaluation count, never
  by wall time, and iterate no hash-ordered container.
- **Mechanism without a CLI.** The runner reads `TIDE_TEST_SEED`; libtest rejects unknown
  flags, so the binary cannot take `--seed`. The report prints the design's command,
  `tide test <path> --seed N`, which a later CLI maps onto the variable. The toolchain tests
  set the variable.
- **No persistence file.** The sandbox gives tests scratch space only, and a regression file
  would add state that D0's replay rule does not cover.

Report for a failure, one block per test:

```
property failed: geometry::tests::area_is_positive
  input: w = 0, h = -1          (shrunk; 41 evaluations)
  cause: assertion failed: area(w, h) > 0, src/lib.rs:31
  seed: 1234567890123456789
  replay: tide test geometry::tests::area_is_positive --seed 1234567890123456789
```

Printing inputs needs `Debug` on every parameter type; proptest and quickcheck require the
same. The design is silent here: require `Debug` and report its absence as the ordinary trait
error.

### 1.4 `requires`-guided generation

Evidence:

- *Filtering alone fails quietly or loudly depending on the tool.* QuickCheck's `==>` discards
  failing cases and gives up after `maxDiscardRatio` discards per success, default 10
  (https://hackage.haskell.org/package/QuickCheck-2.14.2/src/src/Test/QuickCheck/Test.hs).
  proptest aborts after 1,024 global or 65,536 local rejects for 256 cases
  (https://docs.rs/proptest/latest/proptest/test_runner/struct.Config.html). Hypothesis fails
  its `filter_too_much` health check, for example "found 50 filtered examples but only 7 good
  ones" (https://hypothesis.readthedocs.io/en/hypothesis-python-4.57.1/healthchecks.html,
  https://gitlab.com/ifosim/finesse/finesse3/-/issues/555).
- *Syntactic rewriting of simple predicates works and is safe when the predicate still
  filters.* Hypothesis rewrites `integers().filter(lambda x: x > 9)` into
  `integers(min_value=10)` and keeps the original predicate as a filter in case its analysis
  is wrong (https://newreleases.io/project/github/HypothesisWorks/hypothesis/release/hypothesis-python-6.3.1,
  https://github.com/HypothesisWorks/hypothesis/issues/2701).
- *Deriving generators from arbitrary predicates is a research topic.* Luck needs annotations
  and a constraint solver (https://arxiv.org/abs/1607.05443). Pex uses dynamic symbolic
  execution (https://www.microsoft.com/en-us/research/publication/pex-white-box-test-generation-for-net).
  The design already excludes both: your own predicate "only filters", and the solver corpus
  is 04 §2.4 item 3, not v0.
- *Enumeration* (SmallCheck: all values up to a depth,
  https://hackage.haskell.org/package/smallcheck) conflicts with "scalars over their full
  range" and is not what the design chose.

**Recommendation: syntactic shape matching in the macro, and the full `requires` always
re-evaluated as a filter.** The recogniser can then never be unsound: a missed or misread
shape costs acceptance rate and nothing else, and shrink candidates are checked the same way.

- Split the clauses into conjuncts at top-level `&&` and across attributes, in source order.
- Match each conjunct against the reference's list: scalar ranges against a constant or
  another parameter; length bounds; `i < xs.len()`; `xs.is_sorted()`; `xs.contains(&x)`;
  `o.is_some()`; `x == c`.
- Build a generation plan: order parameters so that dependencies come first (the length before
  the index, `lo` before `hi`), narrow each generator's bounds, and apply post-steps (sort for
  `is_sorted`; for `contains`, draw `x` and insert it at a drawn position).
- A contradictory plan, such as `i < xs.len()` with `xs.len() == 0`, falls back to
  unconstrained generation and lets the vacuity floor report it.

Where the design is silent or split:

- **`distinct`.** 04 §2.4 item 2 lists it; the reference's list does not, and adds `lo <= hi`.
  The reference is the agent-loadable contract and the bead quotes it: implement the
  reference's list and report the difference for `design/04`.
- **"A tested function".** The reference says `#[requires]` "on a test or a tested function"
  filters. For a hand-written test the design does not say how the tested function is
  identified. Recommended reading: a `requires violated` panic whose blamed call site lies in
  the test function's own body discards the case and is counted against that clause; one
  raised deeper is a failure; a test with `#[should_panic]` never discards. This matches 04
  §2.4 item 1's rule that the generator, as caller, skips calls whose `requires` fails. It
  needs fx-mhd's violation panic to carry a structured payload (clause, kind, blamed location)
  that the runner can downcast. Only the test's own `#[requires]` can guide generation; a
  callee's can only filter.
- **A panic while evaluating a `requires`** during generation counts as a rejection by that
  conjunct. CLAUSE-7 calls it a violation, and for a generator a violation is a discard.

### 1.5 The vacuity floor

What the design fixes: run "until 100 cases (or `N`, for `seeds = N`) satisfy its `requires`.
If under 1% of generated cases do, it fails the test as `vacuous` and names the conjunct that
rejected most" (reference §10.2). 04 §2.4 item 5 says "below a floor (by default 100 accepted
cases or 1% acceptance)", and its open question 8 leaves the defaults open.

The two texts agree under one stopping rule: generate until `N` cases are accepted or
`100 × N` cases have been generated. If the target is reached at attempt `k ≤ 100N`, the rate
is `N/k ≥ 1%`. If it is not reached, the rate is below 1% and the test is vacuous. **Recommend
exactly this**: one cap, no second threshold, no statistics. The cap is 10,000 generations for
the default 100 cases.

- **Measurement.** Count generated and accepted cases, and per conjunct the number of cases it
  was the first to reject. Evaluation is left to right and short-circuits, because a later
  conjunct such as `xs[i] > 0` may panic when an earlier one is false. "Rejected most" is the
  conjunct with the highest first-rejection count; ties go to source order. Discards from §1.4's
  callee rule count against that callee clause.
- **No `requires`** means every case is accepted and the floor is trivially met. An early
  `return` counts as a pass, as the reference says; the runner cannot see it.
- **Report.** `vacuous: 37 of 10000 generated cases satisfied requires; most rejected by
  \`is_valid(x)\` (9321)`, and a failing test. Every passing property test should also print
  `accepted/generated`, as 04 item 5 requires of the report.
- **Shrink candidates do not count** toward the floor.

Evidence that the floor is lenient: QuickCheck's default tolerates 10 discards per success and
proptest about 4, against Tide's 99. Kani shows the same failure in a prover: an unsatisfiable
`kani::assume` makes every later check pass as unreachable, and `kani::cover` exists to detect
it (https://model-checking.github.io/kani/faq.html,
https://model-checking.github.io/kani/rfc/rfcs/0003-cover-statement.html). The floor catches
"almost never", not "seldom and unrepresentative" (§2).

Cost: 10,000 generations plus `requires` evaluations must fit the 10-second limit with room
for shrinking. Evaluate the `requires` before building any simulator.

### 1.6 `#[sim_test]` parameters and seeds

fx-nkv.6 owns `#[sim_test]` on `fn t(sim: SimRoot)`; fx-m72.2 adds parameters.

**Recommendation:** `sim_test` lowers to the same runner entry as a parameterised `#[test]`,
so there is one owner for case loops, the floor, shrinking and reports. Per case, the
simulator seed is the case seed, and the other parameters draw from a stream derived from it.
Shrinking edits only the parameters' choice sequence and builds a fresh `SimRoot` from the
same seed for each candidate. "Shrinking keeps the seed" then holds by construction, and the
acceptance test (a `Vec<u16>` shrinks while the seed stays) checks one printed number.

Ambiguity: the reference's other form is "a generated `seed: u64` parameter and
`Root::sim(seed)`". As an ordinary `u64` it would be biased toward `0`, `1` and `MAX` and
would shrink, so 100 cases would repeat a few schedules. Recommended reading: a parameter
spelled `seed: u64` on a `#[test]` receives the case seed, uniform and not shrunk. That reads
the sentence literally; it is a spelling-based rule the design does not state outright, so
record it on the bead as a reading of a provisional line.

### 1.7 Generated contract tests: which functions, which inputs

Evidence:

- *Contracts as oracle and filter.* AutoTest uses preconditions to filter inputs and
  postconditions as the oracle, with no hand-written tests
  (https://www.academia.edu/16738341/Programs_That_Test_Themselves). Plain random selection
  left some routines untested because their preconditions were never met; choosing objects
  known to satisfy them fixed that (Wei et al., ICST 2010,
  https://researchr.org/publication/WeiGMO10).
- *Objects from call sequences.* Randoop builds values by random method-call sequences,
  executes each as it is built, and classifies it as illegal, contract-violating or reusable
  (https://homes.cs.washington.edu/~mernst/pubs/feedback-testgen-icse2007-abstract.html).
- *Solver-derived tests.* Dafny's test generation works from Boogie counterexamples and is
  marked experimental
  (https://assets.amazon.science/b9/d4/eeb4fe1a47609f75956dd6fbc0c6/a-toolkit-for-automated-testing-of-dafny.pdf,
  https://dafny.org/v4.2.0/HowToFAQ/Errors). Out of reach for v0, as 04 item 3 says.

**Where the tests come from.** The `#[ensures]` macro can emit a sibling `#[cfg(test)]` test
for a free function, but not for a method: an item inside an `impl` cannot be a libtest entry,
and an item nested in the method body cannot name `Self`. The macro also cannot see trait
bounds or a type's public constructors. **Recommend that the driver owns generated contract
tests**, extending §1.1's pass A3. From the HIR it lists every function carrying an `ensures`,
decides each one's instantiation or skip reason from types, and injects into the defining
module a child test module holding one `#[test]` per function. Injected tests are ordinary
admitted test code that calls public std API.

What fx-mhd's lowering must provide for this (raise it with that bead now):

1. a marker that survives expansion on each function with an `ensures`;
2. each function's `requires` as a separately callable predicate over borrowed arguments,
   returning the first false conjunct;
3. the structured violation payload of §1.4.

**Inputs**, as the design fixes them:

- Parameters use the same generators as property tests, guided by the function's own
  `requires` (§1.4) under the same floor (§1.5).
- Scalars cover their full range.
- A type parameter is `i64` when it meets the bounds, else `String`, else the function is
  skipped.
- A type with private fields is built by one public constructor and then a drawn number of
  public `&mut self` calls. The generator evaluates each call's `requires` first and skips a
  call that fails it.
- Capabilities come from `Root::sim(seed)`.
- Closure, `dyn` and handle parameters, and unmet bounds, make the function unchecked.

**Oracle.** Call the function in the test build. An `ensures violated` panic fails the test
and names the clause. Any other panic in the body also fails it: 04 item 1's `with_capacity`
example ("a limit belongs in the `requires`") implies this.

Silent points and recommended minimal readings:

- *Which constructors.* Public associated functions with no receiver that return `Self`.
  `Result<Self, _>` and `Option<Self>` constructors are a natural extension; leave them out
  until a case needs them.
- *Sequence length.* 0 to 8 calls, drawn. This is a budget, not a contract.
- *`unchecked` and `#[allow(vacuous)]` in libtest.* libtest has only ok, failed and ignored. A
  function skipped at compile time becomes `#[ignore = "unchecked: <reason>"]`: listed, exit
  0. A test that is vacuous at run time under `#[allow(vacuous, reason)]` passes and prints
  `unchecked (vacuous, allowed: <reason>)`. Both are grep-able in the harness log.
- *Laws* (04 §5) also get generated tests in 04, but the reference does not mention them and
  the bead scopes "every function with an `ensures`". Leave them out.
- *Fault injection* (04 item 1) is outside fx-nkv.6's slice. Generated tests use a fault-free
  `SimRoot`; label the behaviour partial in the std docs.

### 1.8 Inside the harness's R stage

`./test` builds with `--test`, lists, and runs each test alone for at most 10 seconds
(protocol §4.1). With §1.1 and §1.7, plain, property, simulation and generated contract tests
are all libtest entries, so `executable_tests` needs no new runner logic. The placeholder
record goes once the binary is built with the production macros.

Three points:

1. **Acceptance listing.** Generated contract tests must not appear in the acceptance build's
   listing, which is compared with the frozen list. Recommend a reserved module name for the
   injected tests and a harness filter on that path segment when `acceptance=True`. It needs
   no driver flag and no cache-key change. A driver flag is the alternative, if building them
   proves slow. Protocol §7.2 puts the generated tests and the floor in the writer's `./test`,
   not in the acceptance suite, so this follows the protocol.
2. **Time.** Budget shrinking at about 1,000 property evaluations and print
   `shrink budget reached` when it stops early. Sim cases are the expensive ones.
3. **Exit status.** A vacuous test fails through the ordinary panic path, so the harness sees
   a non-zero exit without parsing output.

## 2. Pitfalls

- **Flaky seeds.** Any time-based cut-off in shrinking, hash-ordered iteration in a generator,
  or `usize`-dependent draw order breaks "the same seed reproduces". Budget by counts; draw
  lengths as `u64`. Hypothesis has a dedicated flaky-failure error for tests whose outcome
  changes on replay (https://hypothesis.readthedocs.io/en/latest/explanation/example-count.html).
- **Slow shrinking.** proptest's default shrink limit is effectively unbounded
  (`max_shrink_iters = u32::MAX`, Config docs above), and Hypothesis recently added a warning
  for shrinks over five minutes
  (https://hypothesis.readthedocs.io/en/latest/changelog.html, 6.151.10, per search summary).
  Under a 10-second limit an unbounded shrinker turns a real failure into a timeout with no
  input and no seed. Print the seed before shrinking starts.
- **Panic noise.** Each failing shrink candidate prints a panic message. Silence the hook on
  the runner's thread while shrinking, restore it afterwards, and rerun the final input with
  output on.
- **Shrinking into a different bug.** Accept a candidate only when its failure kind matches
  the original (the same violated clause, or the same panic location). Otherwise the report
  names a bug the seed did not find.
- **Generator bias that hides bugs.** Too much small-value bias never reaches large
  collections; too little never hits boundaries. Constructive shapes skew too: sorting after
  generation over-represents duplicates, and `contains` by insertion over-represents short
  vectors. Keep a share of cases unbiased, and never let a shape narrow a type beyond what its
  conjunct states.
- **Vacuity masking.** A 1% floor passes a test whose accepted cases are all trivial, for
  example only empty vectors satisfying a sortedness predicate. An early `return` is invisible
  to the floor. In a generated contract test, a constructor sequence in which every method
  call is skipped tests only fresh objects. The floor does not catch these; the
  `accepted/generated` line and the per-conjunct counts are the only signal, so always print
  them. QuickCheck's `cover` and `checkCoverage` address this class
  (https://www.gresearch.com/news/metatesting-your-property-based-tests/); it is outside the
  design, so report it and do not add it.
- **Attribute order** (§1.1 item 1) silently turns guidance off.
- **Callee discards** (§1.4) can hide a test that always passes bad arguments. The floor then
  reports it as vacuous, which is the correct diagnosis; check this with a test.
- **Injected source** must not move user spans. Append after the last user token of a module,
  and confirm that diagnostics and contract panic locations keep their line numbers.

## 3. Cheapest falsifiers

Each is one driver-built test binary run from `crates/tide-admit/tests`. None uses E1
material.

| Recommendation | Falsifier |
| --- | --- |
| Std `test` macro (§1.1) | `#[test] fn p(a: u8, b: u8) { assert!(a as u16 + b as u16 <= 300) }` fails; a parameterless test still runs; `--list` shows both. |
| Clause order (§1.1) | The same property with `#[requires]` above and below `#[test]` reports identical `accepted/generated`. |
| Driver-synthesised impls (§1.1 A3) | Spike before committing: a private struct in a nested inline module, used as a test parameter with no derive, generates; a compile error on a line after the type keeps its line number. If either fails, fall back to A1. |
| Choice-sequence shrinking (§1.2) | `fn p(xs: Vec<i64>) { assert!(xs.iter().sum::<i64>() < 100) }` reports a one- or two-element vector; a user `Arbitrary` producing only even numbers reports an even number. |
| Seeds (§1.3) | Two runs print the same seed and input; `TIDE_TEST_SEED=<n>` reproduces both; a different seed still passes a true property over 100 cases. |
| Shrink budget (§1.3, §2) | A property whose body takes 50 ms reports seed and input in under 10 seconds and prints `shrink budget reached`. |
| Guided shapes (§1.4) | `#[requires(i < xs.len())]` passes with acceptance near 100%; removing that shape from the recogniser makes the printed rate fall. The filter-always rule is checked by `#[requires(n <= 64 && is_prime(n))]` never running a composite `n`. |
| Shrink keeps `requires` (§1.4) | `#[requires(xs.len() >= 3)]` with a false body reports an input of length 3 or more. |
| Floor (§1.5) | `#[requires(x == 7 && y == x.wrapping_mul(31) + 1)]`, with `y`'s conjunct outside the list, fails as `vacuous` naming it; a predicate true for about 5% of inputs passes; with zero accepted cases the test fails and does not pass. |
| Callee discard (§1.4) | A test calling a function with `#[requires(n > 0)]` directly on generated `n` passes with discards counted; the same violation one call deeper fails. |
| Sim parameters (§1.6) | The bead's case: `#[sim_test] fn t(sim: SimRoot, sizes: Vec<u16>)` failing when `sizes.len() > 2` reports three elements and the seed of the unshrunk case. |
| Contract tests (§1.7) | A function with `#[ensures(result >= x)]` and body `x.wrapping_add(1)` fails its generated test with the clause text; a method on a private-field type is tested through its constructor; a function taking a closure lists as ignored with `unchecked`; `#[allow(vacuous, reason = "..")]` prints `unchecked` and passes. |
| Harness (§1.8) | The bead's acceptance: `python3 trials/e1-checked/harness_test.py`, plus a case that a candidate with an `ensures` leaves the acceptance listing equal to the frozen one. |

## 4. Where the design is silent or split

No new language rule is proposed; each item is a reading to record on its bead and, where
noted, to report against the design text.

1. Whether a struct needs a derive to be generated (§1.1): the reference implies no.
2. `Arbitrary`'s module path and method signature: propose `std::test::Arbitrary` with
   `fn arbitrary(g: &mut Gen) -> Self`, marked provisional.
3. `Debug` on parameter types (§1.3).
4. The default run seed, fixed or fresh (§1.3): fixed.
5. `distinct` in 04 but not in the reference; `lo <= hi` in the reference but not in 04
   (§1.4).
6. What "a tested function" means for a hand-written test (§1.4).
7. "100 accepted cases or 1% acceptance" in 04 against the reference's wording; one cap
   reconciles them (§1.5). Defaults are 04's open question 8.
8. Whether `seed: u64` is special by spelling (§1.6).
9. Constructor selection, sequence length, laws, and how `unchecked` appears in a libtest run
   (§1.7).
10. Fault injection in generated tests is not available from fx-nkv.6 (§1.7).
11. Whether a non-contract panic fails a generated contract test: implied yes (§1.7).

Not verified, because builds were out of scope: that `after_expansion` on the pinned
`nightly-2026-08-09` exposes the HIR needed for §1.1 A3; that source injection through the
driver's `FileLoader` keeps the cache keys in `invocation.rs` sound; and the line estimate in
§1.2.

DONE
