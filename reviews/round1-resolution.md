# Round 1 Resolution Audit

Audit of `design/01-design.md` draft 2 against every blocking objection in
the six round-1 reviews. Each objection has one status:

- `resolved`: draft 2 states a concrete rule that closes the problem the
  review describes, including its counterexamples.
- `partial`: some of the problem is closed, and a named sub-problem or
  counterexample from the review still stands.
- `deferred`: draft 2 explicitly moves the whole objection to a chapter or an
  open question.
- `unaddressed`: draft 2 does not respond.

The bar is strict. A concrete soundness hole counts as closed only when a
stated rule rejects the review's example. Fix suggestions that draft 2 did not
adopt are noted, but they downgrade a row only when the review's stated
problem depends on them. Where part of an objection was moved to a chapter or
open question, the row says so.

## Summary

| Status | Count |
| --- | ---: |
| resolved | 19 |
| partial | 28 |
| deferred | 0 |
| unaddressed | 0 |
| **total** | **47** |

| Review | Resolved | Partial |
| --- | ---: | ---: |
| Soundness (S) | 3 | 7 |
| Agent usability (A) | 3 | 4 |
| Performance (P) | 7 | 0 |
| Verification (V) | 1 | 8 |
| Strategy (St) | 2 | 4 |
| Codex (C) | 3 | 5 |

No objection was ignored outright. The partial rows cluster in three places:

- the origin calculus (D2), which is decided in outline but whose normative
  form is deferred to an unwritten memory chapter;
- the contract semantics (D10), which adopted the headline rules of the
  verification review but left out the details that make them sound;
- the purity and `lock` rules (D7, D8), where some counterexamples from the
  reviews are still accepted.

## Resolution table

| Review | Objection | Status | Where / What is missing |
| --- | --- | --- | --- |
| Soundness | B1. Region model unspecified; literal reading unsound or unusable | partial | **Where:** D2 "Types carry origins per reference field … Variance follows Rust: shared references are covariant, `&mut T` and cells are invariant in `T`"; "Rust's lifetime-elision rules apply unchanged". **Missing:** (1) D2 gives "one origin per reference-typed field", so a field that nests references, such as `out: &mut Vec<&str>`, gets a single origin and reproduces the `&'a mut Vec<&'a str>` trap unless the author writes `from`. The default for nested positions is not stated. (2) The normative calculus is deferred ("lives in the memory chapter"), and that chapter does not exist; the review asked that it exist before freeze. (3) The review's soundness argument was translation to Rust, re-checked by rustc. D12 drops Rust emission and names no replacement oracle. |
| Soundness | B2. `from` cannot name an inner region, relate two parameters, or type scoped spawn | partial | **Where:** D2 "An origin names where a borrow comes from: a parameter, a place path, or `'static`"; `-> Option<&str from self.src>` in D2 and The Feel. This covers shape 1 (`from *words` as a place path) and shape 2 (lexer tokens). **Missing:** shape 3: `from` in parameter types, needed to relate two parameters (`add_word(counts: &mut HashMap<&str from w, u32>, w: &str)`). D2 says "`from` is a keyword only in this position". Shape 4: there is no signature for `Scope::spawn` and no rule that rejects a child outliving its function when a scope is passed down (`fn handle(bg: &Scope, conn: Conn)`). Also missing: a spelling for a trait impl's associated type that names the implementing type's origin (`impl Iterator for Lexer { type Item = &str from ?; }`). |
| Soundness | B3. No `'static`/owned region; `dyn` and `move` closures erase regions | partial | **Where:** D2 "A type with no borrowed origins is `Owned` … `dyn Trait` means `dyn Trait + Owned` unless written `dyn Trait from x` … a closure that captures a borrow is itself borrowing." This closes the review's `make`, `Job`, and literal examples. **Missing:** escape is still decided by the `move` keyword ("Escaping closures (`move` closures stored or returned) … require `Owned`"), not by a region judgment as the review asked. That contradicts the rule that a borrowing closure is a borrowing type, and contradicts `dyn Fn() from x`. It also rejects Rust-valid stored `move` closures that borrow, such as `flat_map(move …)` over a borrowed collection. |
| Soundness | B4. No drop check | resolved | D2 "Drop check uses Rust's conservative rule … standard containers carry an audited exemption." Note: the explanatory clause states the outlives relation backwards (New problem 3). The std exemption is not listed as a trust hole in D11. |
| Soundness | B5. `Shape` reflection forges capabilities and bypasses invariants | resolved | D6 "Constructing a value through `Shape` is allowed only for types that opt in and whose fields are all visible; capability types can never be constructed this way." |
| Soundness | B6. `Pure` does not imply determinism, non-suspension, or race freedom | partial | **Where:** D7 `Pure` excludes any "capability, synchronization handle (channel ends, `Shared<T>` handles, task handles), or foreign handle"; D7 `Frozen`; D9 requires `Send` and `Frozen` captures; D7 `#[trusted_pure]` counts as a trust hole. This fixes (a), (c), and (e). **Missing:** (b) no rule makes `dyn Trait` or `Box<dyn Any>` non-`Pure` unless spelled `+ Pure`, so erased captures stay invisible. (d) Memoization needs only `Pure` and `Frozen`, so `fn bump(v: &mut Vec<i32>)` is still memoizable; the review's "no `&mut` parameters" condition is absent. Raw pointers are not excluded from `Pure`. |
| Soundness | B7. `lock` neither sound nor deadlock-free; scope join cancellable | partial | **Where:** D8 "checks at run time that the handles are distinct"; "Its body may call only authority-free functions"; "Joining a scope cannot be cancelled". This fixes (a), (d), and nesting through calls. **Missing:** (b) the payload bound is `T: Send`, not `T: Pure`, so the body can reach `Shared` handles or capabilities through its `&mut T` binding (`lock (bank) \|r\| { lock (r.accounts[i]) … }`, or `Shared<Conn>` where `Conn` holds `Net`). The stated reason, "`Shared` handles are not `Pure`", constrains calls, not data reached through the payload. (c) Nothing says what a cancelled task waiting on a contended `lock` gets; `Result` is described only for poisoning. |
| Soundness | B8. Recovery at task boundaries exposes broken invariants | resolved | D5 "On that error path, every place the child borrowed mutably is treated as moved out … and every cell the child held locked is poisoned". |
| Soundness | B9. Development and release builds not observationally equivalent | partial | **Where:** D3 "No polymorphic recursion"; D3 "`to_bits()` canonicalizes NaN"; D12 monomorphization in every profile, with a witness fallback that "must behave identically"; the `Rc` count-elision promise is gone (D2 "No hidden reference counting"). **Missing:** type-dependent constants (`4096 / size_of::<T>()` with a zero-sized `T`) and const-fuel exhaustion still fail per instantiation, which contradicts D4's "instantiation never produces a type error". NaN canonicalization does not cover transmutes or hashing. Layout is not defined as a function of the type alone. Equivalence "modulo resource exhaustion", with stack overflow as a coded task failure, is not defined; the witness fallback needs it. |
| Soundness | B10. Proofs unsound across modes and under aliasing; `&mut` results unspecifiable | partial | **Where:** D10 trait laws ("Under `prove` each impl proves them"); D11 holes include "unproved impls of laws, and unproved dependency contracts that a proof relies on"; D10 `final(x)`; D10 "Specs call only `spec fn`s", which keeps `RefCell::borrow` out of specs. **Missing:** (d) field-declared borrows versus invariants that span fields are not addressed, and draft 2 does not define type invariants at all. Closures without specs that are passed to proved higher-order functions are not counted as holes. |
| Agent usability | B1. Rust-habit tax; respellings buy nothing | resolved | D1 "Rust syntax is Ferox syntax for every construct with the same meaning, including `where` clauses, `pub(crate)`, inline `mod` blocks, `#[derive(...)]` with Rust's trait names, `#[cfg(test)]` …"; the fixed set of `!` forms; D3 accepts lossless `as`. Notes: lossy int-to-float conversion (`len() as f64`, the review's example) still has no named spelling, and `truncate()` still collides with `Vec::truncate`. |
| Agent usability | B2. Elision stricter than Rust; `from` collides; closures and iterator chains open | partial | **Where:** D2 "Rust's lifetime-elision rules apply unchanged"; "`from` is a keyword only in this position"; D5 `#[from]`. **Missing:** D1 rejects Rust `'a` signatures that fit the model, although their meaning is identical; accepting and normalizing them would follow D1's own "reject only where meaning differs". Problem 3 (iterator chains): D4 has no return-position `impl Trait`, so returning a borrowing adapter chain can only be spelled `Box<dyn … from x>`. The `move`-escape rule (S-B3) rejects some Rust chains. No iterator-chain examples are committed to the reference. |
| Agent usability | B3. Logging has no home; `print` contradicts D7; logging destroys purity | resolved | D7 "Write-only diagnostic sinks are not authority … they do not affect purity … `dbg!` is an error in release builds". Not adopted: a per-package `emits: log` record in the lockfile and trust diff, and `Secret<T>`. The `log` spelling is still Open Question 1 (New problem 7). |
| Agent usability | B4. Reviewability bypasses (`&Root`, bundles, authority-bearing `self`, `Root::sim`) | resolved | D7 "`Root` can be named only in `main`; a simulated root (`Root::sim(seed)`) only in tests and in `ferox sim`"; "Tooling reports declared and exercised authority for every public item and lints over-granted capabilities. `ferox fix` can thread a capability through a call chain." Note: exercised authority covers public items only, not every function's interface entry. |
| Agent usability | B5. Weak specs pass every mode | partial | **Where:** D10 classifies clauses as `checked`, `sampled`, or `proof-only` and "reports unchecked clauses"; "Spec quality … preconditions are satisfiable and postconditions are not trivially true, offers mutation testing … a weakened spec … needs approval"; `old(x)`; "Capabilities have no spec model in v1". **Missing:** gap 3. The only rule is "`requires` filters and guides generation". There are no derived generators for sorted, distinct, or membership preconditions, no count of accepted cases, and no floor, so a property test can still pass on zero cases. Trigger inference is not adopted ("quantifiers need triggers" reads as manual). Spec strength is not shown next to the spec in `ferox review`. |
| Agent usability | B6. Toolchain contract omits the execution loop | partial | **Where:** D14 "`ferox test` and `ferox sim` return structured results: each failure has its seed, a replay command, and a capability event trace"; "`ferox eval` …; coverage and spec coverage are reported". **Missing:** an output contract for `ferox verify` with a status per obligation (`proved`, `refuted` with a counterexample, `timeout`, `assumed`, `trusted`); expected-versus-actual diffs and shrunk counterexamples in test results; latency targets that hold for a fresh CLI process; `ferox run`. |
| Agent usability | B7. Reference budget unmeasured; learnability target has no number | partial | **Where:** D1 "The reference is written as a delta from Rust and token-counted in CI"; D14 card under 8 KB, core reference 25,000 tokens, std reference separate; D0 E1. **Missing:** E1 has no numeric thresholds ("compared with the same agent writing Rust"), yet D0 gates the syntax freeze on "its targets". The 25k core budget is unchanged against the review's 33–36k estimate, with no evidence it fits. Reference paragraphs are not delivered with diagnostics. There is no explicit rule to use Rust std names. |
| Performance | B1. No v1 backend meets the build targets; dev code 20–100x slower | resolved | D12 "a Cranelift-class development backend with mid-IR inlining and an incremental linker, in v1"; targets "edit to running tests under 1 s; clean debug build a few seconds; development-build runtime within 3x of release". Note: there is no release-build-time target, and the one-liner still says "Go's build speed" (see St-B3). |
| Performance | B2. Generics slow in dev, unimplementable in release | resolved | D12 "Generics are monomorphized in every profile … Witness-table code is a budgeted fallback"; D3 "No polymorphic recursion." |
| Performance | B3. Stackful tasks have no sound lowering and miss memory targets | resolved | D8 "under the evented runtime it compiles them to stackless state machines with non-moving heap frames"; Rejected: "stackful tasks in compiled code". Note: "knows from signatures which functions may suspend" does not hold for closures and `dyn` (New problem 12). |
| Performance | B4. Borrow model does not lower to Rust rustc accepts | resolved | Moot. D12 rejects Rust emission as the release backend, and D2 sets "NLL precision plus declared field borrows". |
| Performance | B5. Always-on overflow traps conflict with Rust-class speed | resolved | D3 "The trap may be reported late, but before the overflowed value influences any observable effect or leaves the enclosing loop or function … Proved contracts delete checks." The exact rule is deferred to Open Question 2. Check deletion creates a new soundness hole (New problem 1). |
| Performance | B6. `Shape` serialization slow; construction unsound | resolved | D6 closed derives include `Encode` and `Decode`, "compiler-generated serialization … so serialization does not depend on reflection speed"; the `Shape` construction rule as in S-B5. |
| Performance | B7. crates.io interop does not reach the target market's APIs | resolved | Positioning "Async Rust ecosystems (tokio, axum) and lifetime-heavy APIs (serde's `'de`) are out of scope for interop"; D13 synchronous C-ABI shims. Not adopted: serde impls generated for exported types. |
| Verification | B1. `prove` does not define what is proved | partial | **Where:** D10 "`prove`: an SMT solver proves total correctness. Every panic site, including arithmetic, indexing, `unwrap`, and assertions, is a proof obligation, and every loop and recursion needs `decreases` or an explicit `diverges` marker. Lemmas must terminate."; D5's moved-out rule for unwinding. **Missing:** termination of `spec fn`s. Only lemmas are required to terminate, and a non-terminating recursive `spec fn` is an unsound axiom; the review called this a soundness requirement. Inferred `decreases` for range loops and structural recursion is not stated. |
| Verification | B2. Spec arithmetic and evaluation undefined | partial | **Where:** D10 "Spec integers are mathematical; `/` and `%` truncate as in Rust; `==` is model equality". **Missing:** equality and arithmetic on floats in specs (NaN), which the review raised explicitly; division by zero in a spec; how `assert` mode evaluates mathematical integers (i128 or bignum). "Model" is used but never defined. |
| Verification | B3. `&mut` cannot be specified | partial | **Where:** D10 "`old(x)` the entry value of a `&mut` parameter; `final(x)` the value a returned or stored `&mut` holds when its borrow ends". **Missing:** what a bare `*x` or `x` of a `&mut` parameter means in `ensures`, so the form Verus removed is not ruled out; whether `final` applies to `&mut` parameters; rules for type invariants at borrow end. Type invariants are undefined. |
| Verification | B4. What specs may call is undefined; D7 purity is not spec purity | partial | **Where:** D10 "Specs call only `spec fn`s and the built-in spec types … Closures appear in specs only through `f.requires(args)` and `f.ensures(args, r)`." **Missing:** spec aliases for std methods such as `len` and indexing. The Feel's `max_index` contracts call `xs.len()` and `xs[i]`, which this rule forbids as written. Models (`@`, `impl Model`, the rule that specs add no bounds) are absent. Draft 2 does not say whether a `spec fn` can be called from executable code. |
| Verification | B5. "One spec, three uses" false for a class of specs | resolved | D10 "The compiler classifies each clause as `checked` …, `sampled` …, or `proof-only` …, and reports unchecked clauses"; `test` mode checks loop invariants. Note: "Specs never affect typing or code generation" was not reworded, although `assert` mode generates checks. |
| Verification | B6. Trait contracts, impl obligations, and laws missing | partial | **Where:** D10 "Traits declare bodiless `law fn`s … Under `prove` each impl proves them … Derived impls come with generated proofs"; D11 "unproved impls of laws". **Missing:** contracts on trait method declarations, and their inheritance by impls (behavioral subtyping); std laws for `Eq`, `Hash` (agreement with `Eq`), and `Clone`. |
| Verification | B7. Spec hash covers the wrong thing; hole list incomplete; approval enforces nothing | partial | **Where:** D11 "Hashes cover everything a spec's meaning depends on, including the bodies of `spec fn`s it calls"; "Approval lives outside the agent's write scope … signed by a human-held key and verified by CI"; transitive hole list. **Missing:** `diverges`, verification-mode downgrades (`prove` to `assert`), and per-function verification skips are not holes. The proof cache (D12 "cached by content hash") is not restricted to entries CI wrote, so a forged `proved` entry is still possible. The approval scope is undecided: D10 and D11 disagree (New problem 10), and the review's root-specs-only scope is not adopted. |
| Verification | B8. No vacuity or spec-strength checks | partial | **Where:** D10 "checks that preconditions are satisfiable and postconditions are not trivially true, offers mutation testing of specs". **Missing:** contradictory-assumption warnings in proofs; detection of a quantified `ensures` whose trigger never fires at call sites; protected spec-example tests that evaluate `f.ensures` on good and bad outputs. Mutation testing is optional ("offers"), so the incomplete `sort` spec (no permutation clause) passes by default. |
| Verification | B9. Specs over capabilities and shared state undefined | partial | **Where:** D10 "Capabilities have no spec model in v1. A function that uses a capability may state postconditions about its results that hold for every possible response of the world." **Missing:** the lock invariant for `Shared` cells: the payload invariant is assumed on entry and proved at exit. No rule explicitly bars a `spec fn` from taking a capability. |
| Strategy | B1. Competes with Bend on the headline; tandem unbuildable | resolved | Thesis "Proof-first, massively parallel pure computation is Bend's territory (D16)"; the one-liner no longer says "Dafny's proofs"; D16 Stage 0 runs kernels as a child process, Stage 1 is conditional ("Ferox never depends on this stage"), and a contract can be exported as a Bend law. |
| Strategy | B2. Trust surface lies at the Rust boundary | partial | **Where:** D13 "Each imported crate declares its authority in the manifest; the shim requires those capabilities; a symbol scan of the compiled crate checks the declaration; its build script, if any, runs sandboxed". **Missing:** simulation is not gated. D15 only "lists any reachable foreign effect with no simulator model", and Open Question 4 leaves the model open, yet Thesis point 2 promises replay for "any Ferox program, dependencies included". Proc macros of Rust dependencies are not covered. An attenuated capability (`net.only(...)`) passed to a shim is not enforced on the Rust side and is not labelled "declared, scan-verified". `ferox review` has no separate band for Rust dependencies. |
| Strategy | B3. Pitch leads with what Rust is absorbing | partial | **Where:** the Thesis leads with authority, simulation, and the trust surface; contracts "are not the headline"; simulation is decision D15. **Missing:** the one-liner still claims "Go's build speed", which is unmeasured, and release builds go through an optimizing C compiler. There is no single quantified, falsifiable claim. |
| Strategy | B4. Services are where interop works worst; evented runtime is costly | resolved | Positioning "v0 target: command-line tools, developer tooling, and infrastructure components … Blocking and thread runtimes plus the simulator; the evented runtime follows." Not adopted: a kill criterion for the evented runtime. |
| Strategy | B5. Authority signal and specs decay into noise | partial | **Where:** D7 declared and exercised authority and the over-grant lint; D7 decides logging; D10 vacuity checks and mutation testing. **Missing:** mutation or strength results beside each changed contract in the trust diff (D11 reports only "spec coverage"; presentation is Open Question 7); the capability-plumbing experiment before D7 freezes (D0 gates only D1 syntax); a logging decision justified against `faster_log`. |
| Strategy | B6. No plan to falsify the thesis; no v0 scope | partial | **Where:** D0 E1–E5 and "Gates at months 3, 6, and 9 decide continue, adjust, or stop"; the Positioning v0 scope; D13 C-ABI shims. **Missing:** the gates have no thresholds and no baseline (for E3, Rust with cargo-acl and cargo-semver-checks), so no gate can fail; no commitment to publish negative results; no statement that the dev loop links prebuilt shim libraries and never runs rustc. |
| Codex | B1. Capability freedom does not establish mathematical purity | partial | **Where:** D7 split into `Pure` and `Frozen`; synchronization handles are not `Pure`; D8 lock body "may call only authority-free functions". **Missing:** the review's own counterexample is still accepted: memoizing `fn increment(x: &mut i32) -> i32` passes, because memoization needs only `Pure` and `Frozen`. Propagation through trait objects (`dyn`) is unspecified. |
| Codex | B2. Authority guarantee has executable escape routes | partial | **Where:** D7 "Foreign calls carry authority … wrapped by a Ferox signature that takes the capabilities it may use, or is marked `#[trusted_pure]` … counts as a trust hole"; "`dbg!` is an error in release builds"; D15 lists unmodeled foreign effects. **Missing:** diagnostic sinks stay ambient by design (D7 rejected the trace-capability fix), and nothing in the trust surface records which packages emit. A capability-free dependency can therefore run `log::info!("{secret}")` without a reviewer seeing it. Capability lists on C `extern` wrappers are declared but never verified; only Rust crates get a symbol scan. Thesis point 1 ("in any dependency") is not qualified by this trusted boundary. |
| Codex | B3. Task isolation does not make suspended Rust frames transferable | resolved | D8 "Foreign callbacks cannot suspend. A task holding a `ThreadBound` value is pinned to its thread while it holds it." plus stackless lowering. Not stated: unknown blocking foreign calls run on blocking workers under the evented runtime. |
| Codex | B4. Optional proof modes lack a composition rule | partial | **Where:** D11 exports proof status ("proved, tested, or assumed") and counts "unproved dependency contracts that a proof relies on". **Missing:** the review required that memory safety not depend on erased, unproved preconditions. Draft 2 does the opposite: D3 and D10 say proved facts delete checks, and "Release builds erase specs" (New problem 1). Proof status is not bound to the implementation and toolchain identity. There is no `refuted` or `unknown` (resource-limit) status. |
| Codex | B5. Spec language cannot have a universal runtime fallback | resolved | D10 classification into `checked`, `sampled`, and `proof-only`, with unchecked clauses reported. Not stated: failed proofs versus solver resource limits, and whether `ferox check` and its 30 ms target include proof obligations. |
| Codex | B6. Reproducible reductions do not imply sequential equivalence | partial | **Where:** D9 "defined by a canonical balanced grouping that depends only on input length … For non-associative operations such as float addition, the canonical grouping is the definition". **Missing:** the same bullet requires "an operation whose trait declares associativity as a law", which float addition cannot satisfy (New problem 2). Nothing says which failure wins when several `par` branches panic or one diverges. |
| Codex | B7. Trust diff cannot replace behavior review | resolved | D11 "Items without contracts still need ordinary review and tests; the trust diff narrows review, it does not replace it", plus spec coverage. |
| Codex | B8. Hashing an editable oracle does not establish approval | partial | **Where:** D11 "Approval lives outside the agent's write scope … A hash written in the same change is never approval." **Missing:** proof-mode and configuration downgrades are not in the approval set, which covers only "spec changes, authority widening, and new trust holes". Protected tests no longer exist in draft 2, so editing or deleting a test needs no approval (New problem 11). |

## New problems in draft 2

Ordered by severity.

1. **Check deletion makes memory safety depend on erased, unproved
   preconditions (D3, D10, D11).** D3 says "Proved contracts delete checks".
   D10 says "proved facts may delete runtime checks. Release builds erase
   specs." Take a `prove`-mode `pub fn at(xs: &[u8], i: usize) -> u8 requires
   i < xs.len() { xs[i] }`. Its bounds check is deleted on the strength of its
   own `requires`. In a release build, an `assert`-mode caller that passes
   `i = 10` then reads out of bounds from safe code. Codex B4 asked for exactly
   the opposite rule, and verification N7 and N8 warned about this case. One
   of three fixes is needed:
   - delete a check only on facts proved without a caller-supplied
     precondition;
   - keep `requires` checks on public functions whose bodies lost checks;
   - never delete memory-safety checks in v1.

2. **`par_reduce`'s associativity law contradicts float addition (D9, D10).**
   D9 requires "an operation whose trait declares associativity as a law" and
   in the same bullet allows float addition. Under D10, `prove` makes each
   impl prove its laws, which is impossible for `f64` addition, and `test`
   turns laws into property tests that `f64` addition fails. Either drop the
   law requirement for the tree-defined reduction and keep it only for proving
   `par_reduce == fold`, or provide a separate reduction with no law.

3. **Drop check is stated backwards (D2).** "a value with a destructor must
   strictly outlive the borrows it holds" is the reverse of Rust's rule. It
   should read "the borrows it holds must strictly outlive the value". The
   memory chapter will implement this sentence, so the direction matters.

4. **The closure escape rule contradicts origins (D2, D8).** "Escaping closures
   (`move` closures stored or returned) … require `Owned`" conflicts with "a
   closure that captures a borrow is itself borrowing" and with
   `dyn Trait from x`, which exists to return borrowing trait objects. It also
   rejects Rust code that stores borrowing `move` closures in iterator
   adapters. The same sentence mentions "detached tasks", which D8 does not
   have ("unstructured spawn" is rejected).

5. **The `from` grammar is inconsistent and ambiguous (D2).** D2 says "`from`
   is a keyword only in this position", but draft 2 uses it in four positions:
   - after a return type (`-> &str from a, b`);
   - inside type arguments (`Option<&str from self.src>`);
   - on struct fields (`b: &str from a`);
   - in trait objects (`dyn Trait from x`).

   The comma form `from a, b` collides with generic-argument commas: is
   `HashMap<&str from a, b>` one origin and one type argument, or two origins?
   Pick one placement rule and give it a grammar.

6. **The Feel's lexer cannot implement `Iterator` (D2, D4).** There is no
   spelling for `impl Iterator for Lexer { type Item = &str from ?; }`. An
   associated type has no `self` place to name, and D2 says nothing about
   naming the implementing type's origin (`src`) there. This is the canonical
   zero-copy pattern, and it ties to S-B2.

7. **The Feel uses forms that the rules forbid or leave open.**
   - `log::info!` is not in D1's closed set of `!` forms (Open Question 1), yet
     The Feel and D7 rely on it.
   - `max_index`'s contracts call executable `xs.len()` and index `xs[i]`, and
     quantify over `usize`. D10 says "Specs call only `spec fn`s" and
     "quantifiers range over spec types". A spec-alias rule and a rule that
     maps machine integers to spec integers are needed.

8. **Code that runs at build time contradicts itself (D6, D7, D13, D16).**
   D13 says "No build scripts, install hooks, or code that runs at fetch or
   build time", and D7 says "There are no build scripts". Yet:
   - D13 runs a Rust crate's "build script, if any … sandboxed";
   - D6 says "Code generators are build steps";
   - D16 builds and checks Bend kernels as part of the build.

   State the rule as "no unsandboxed or unrecorded build-time code", or move
   generators out of the build (codex N8).

9. **Thesis point 2 overclaims (Thesis, D13, D15, D16).** "any Ferox program,
   dependencies included … replays bit for bit". But Rust crates with declared
   `net` or `fs` authority, `#[trusted_pure]` FFI, and Bend child processes
   have no simulator model. D15 only lists them, and Open Question 4 is open.
   Either refuse simulation builds that contain unmodeled foreign effects, or
   qualify the claim.

10. **D10 and D11 disagree on what needs approval.** D10 gates only "a weakened
    spec". D11 requires signed "Approvals of spec changes", which means every
    change. Agent-usability B5 and verification B7 both asked to gate only
    weakening, and only for root specs.

11. **Protected tests were silently dropped.** Draft 1 had protected specs and
    tests. The strategy and agent-usability reviews praised them, and
    verification B8 and codex B8 relied on them. Draft 2 never mentions them,
    and its approval set leaves out tests. An agent can now edit a failing test
    to pass, which is the ImpossibleBench failure.

12. **"Knows from signatures which functions may suspend" is false for
    higher-order code (D7, D8, D12).** A generic `fn retry<F: Fn() -> R>(f: F)`
    or a `&dyn Fn()` suspends only when the closure captures a capability,
    which no signature shows. D7 also does not say whether a generic `F` or a
    `dyn Fn` counts as `Pure`. The witness-table fallback (D12) and `dyn` calls
    under the evented runtime need a rule, such as boxed frames for
    possibly-suspending indirect calls, or `dyn … + Pure` to forbid them.

13. **Dropping Rust emission removed the soundness oracle and the eject path
    (D12).** S-B1's soundness argument was "every accepted Ferox program maps to
    a Rust program with explicit lifetimes", re-checked by rustc in CI. The
    strategy review treated emission as the exit door. Draft 2 names no
    replacement for either. A CI-only Rust emitter used as a borrow oracle
    would keep the first.

14. **Minor naming and scoping inconsistencies.**
    - `Shared` appears in D7's capability list and is also D8's cell handle
      type, `Shared<T>`. It is unclear whether creating a cell needs a
      capability.
    - D16's `Accel` capability is missing from D7's list.
    - D3's "one method per name per type" must exempt trait methods: The
      Feel's `#[derive(Debug, Error)]` gives `FetchError` both `Debug::fmt` and
      `Display::fmt`.
