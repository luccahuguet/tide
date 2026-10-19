# Round 1 Review: Contracts, Proofs, and the Trust Surface

Reviewer stance: adversarial. I review as a formal-verification researcher who
has built and used Dafny, Verus, F\*, and Creusot, and who has studied how LLMs
write and game proofs. My job is to break D10 and D13 and the places where they
meet D2, D4, D5, D7, D8, D9, and D11. Praise is limited to the last section.

Read in full: `design/00-first-principles.md`, `design/01-design.md`,
`research/04-verification.md`. Consulted: `research/00`, `research/06`,
`research/07`, `research/10`. Prior art was re-checked against current primary
documentation (Verus guide, Creusot guide, Dafny reference manual, IronSpec,
SPARK). Sources are listed at the end.

## Verdict

**request_changes.**

The direction is right, and the evidence in `research/04` supports it:
auto-active SMT contracts, Dafny and Verus clause names, contracts in the
grammar rather than a macro DSL, mathematical types in specs, machine integers
in executable code, opaque bodies, a pinned solver, and counted trust holes. I
would not change the strategy.

D10 as written is a list of features, not a semantics. It does not say what
`prove` proves, what a spec expression means, what may be called from a spec,
how `&mut` is specified, how trait contracts bind impls, or what a spec hash
covers. Each gap either admits an unsound proof or lets an agent pass every
check with a vacuous spec. The examples below are short and realistic. Several
of them are exactly the cheats measured in VeruSAGE and the vericoding
benchmark (`research/04` F5). The fixes are small, and most of them copy
decisions Verus and Creusot have already made and paid for.

## Blocking Objections

### B1. D10 + D5 + D8: `prove` does not define what is proved

**Problem.** D10 never says whether `prove` means partial or total correctness,
whether panics are obligations, whether executable code must terminate, or what
the runtime `assert` built-in (D6) means under `prove`. Under partial
correctness every one of these verifies:

```ferox
fn sort(v: Vec<i64>) -> Vec<i64>
    ensures is_sorted(result@)
{ panic("todo") }                    // never returns, so the ensures is vacuous

fn sort(v: Vec<i64>) -> Vec<i64>
    ensures is_sorted(result@)
{ loop {} }                          // Creusot documents this exact `nonsense()` case
```

`assert` is ambiguous. In Rust and in D6 it is a runtime check. In Dafny it is
a proof obligation. If under `prove` it is "check at run time, then assume", it
is Dafny's `expect`: an uncounted `assume`. An agent that writes
`assert(is_sorted(&out))` before returning gets the ensures for free.

Unwinding also breaks type invariants. D5 lets a structured task boundary
recover a panic as a value, and D8 lets children borrow from the parent's
frames. Take a type invariant `lo <= hi` on `Pair` and a child running
`p.lo += d; p.hi += d;`. If the second add traps, the parent recovers and then
reads a `Pair` that violates its invariant, while the verifier assumes every
`Pair` satisfies it. Verus's guide states that its only unwind-safety mechanism
is to "bluntly ensure that no unwinding happens at all" (`no_unwind`).

**Smallest fix.**

1. Under `prove`, every function with a contract is checked for **total
   correctness**. Every panic site is an obligation: overflow, bounds,
   division, `unwrap`, explicit `panic`, and the runtime `assert(e)`. A runtime
   `assert` never becomes an assumption, and there is no `expect`.
2. Loops and recursion need `decreases`. The compiler infers it for range loops
   and structural recursion, as Dafny does for simple `while` loops.
3. A signature-level `diverges` clause opts a function out of termination. It
   is part of the spec surface.
4. `spec fn` and `proof fn` must always terminate. That is a soundness
   requirement, not a lint: Creusot's `falso()` proves `false` otherwise.
5. A recovering task boundary may not capture `&mut` borrows from the parent.
   This is Rust's `UnwindSafe` rule, enforced. Scopes that do borrow `&mut`
   propagate the panic instead of recovering it.

### B2. D10 + D3: spec arithmetic and evaluation are undefined, so the three uses disagree

**Problem.** "One spec, three uses" works only if the spec means the same thing
under the runtime evaluator, the property tester, and the SMT encoding. D10
mixes machine integers with unbounded `int` and gives no lifting rule.

- `requires i - 1 < xs.len()` with `i: usize = 0`. In SMT over `int` the
  precondition holds (`-1 < len`). The runtime check evaluates `0usize - 1`
  and traps (D3) inside the precondition.
- `ensures result == (lo + hi) / 2`. This is the classic binary-search
  overflow. SMT over `int` is fine; the `assert`-mode evaluator over `usize`
  traps inside the spec.
- `ensures result == a / b` with `a = -7` and `b = 2`. Executable `/`
  truncates to `-3`. SMT-LIB `div` is Euclidean and gives `-4`. Verus documents
  that in ghost code `/` and `%` are Euclidean, "rather than Rust's truncating
  division". The result: a correct implementation fails its proof, or the proof
  and the runtime check give different verdicts.
- Spec `==` is undefined for floats, where `NaN != NaN`, and for types with a
  hand-written `Eq`.

**Smallest fix.** Specs are evaluated in mathematical types in all three uses.

- Machine-integer reads widen to `int`, and spec arithmetic never overflows.
- `/` and `%` in specs use Rust's truncating semantics, defined over SMT's
  operators, so the same expression text means the same thing in code and in
  spec wherever the code does not trap.
- Division by zero in a spec is a well-formedness error, not a value.
- The runtime evaluator uses `i128` with a bignum fallback.
- Spec `==` is model equality: structural, and extensional on
  `Seq`/`Set`/`Map`. It never calls user `Eq`.
- Floats compare bitwise in specs. Float arithmetic is uninterpreted in v1.

### B3. D10 + D2: `&mut` cannot be specified, though D2 lets it be returned and stored in views

**Problem.** D10 has no `old`, no final-value operator, and no rule for what
`*x` means in `ensures`. Yet D2 allows `-> &mut T from p` and view types with
`&mut` fields, and std will need `get_mut`, `iter_mut`, `entry`, and D9's
`par_chunks_mut`. Consider:

```ferox
fn get_mut(&mut self, k: &K) -> Option<&mut V> from self
```

Without a prophecy there is no way to say that a later `*r = v` updates
`self`'s map, so callers can prove nothing about the map after using `r`.
Verus now handles this with `old(x)` and `final(x)`. It also *removed* the
earlier rule that bare `*x` in `ensures` means the post-state, because that
rule "turned out to be incompatible with the formal theory that Verus later
adopted". Old Ferox's own example
(`ensures account.balance == old(account.balance) - amount` on a `&mut`
parameter, quoted in `research/00`) uses exactly the form Verus removed.

**Smallest fix.** Adopt Verus's current rule from day one.

- In `requires`, `*x` is the value at entry.
- In `ensures`, a bare `*x` of a `&mut` parameter is a compile error with two
  machine-applicable fixes, `old(x)` and `final(x)`.
- `final(r)` is allowed on `&mut` parameters, `&mut` results, and `&mut` fields
  of views.
- Type invariants follow Creusot. The invariant of `final(r)` is assumed, and
  the invariant of the borrowed value is proved when the borrow ends.

D2's second-class references keep this small, because prophecies only arise at
`from` returns and in views.

### B4. D10 + D7: what may be called from a spec is undefined, and D7 purity is not spec purity

**Problem.** D7's "pure" allows panics, divergence, allocation, and `dbg`
(`research/07`). A spec function must be total and deterministic. The design's
own example calls executable `xs.len()` and `Index::index` (which panics) inside
`ensures`. VeriStruct saw LLMs call executable functions inside specs
(`research/04` F7). Three concrete failures:

- `ensures result == crc(data)`, where `crc` is a D7-pure `fn` that loops
  forever on some inputs. The runtime check hangs. SMT gets either an
  unconstrained term or, if the body is inlined, an unsound definition.
- Higher-order specs: `ensures forall |i| ... result[i] == f(xs[i])`, where
  `f` is a borrowed closure. D7's contextual polymorphism lets `f` capture
  `&Net`. The SMT encoding treats `f` as a mathematical function and proves
  `f(a) == f(a)` across two network calls, which is false about the program.
- Generic specs: if `x@` or `old(x)` needs a `T: Model` or `T: Clone` bound,
  the claim that "specs never affect typing" is false.

**Smallest fix.**

- Specs may call only `spec fn`s, model accessors (`@`), and a fixed list of
  std methods that carry declared spec aliases (`len`, indexing). This is
  Verus's `when_used_as_spec`, restricted to std in v1.
- A `spec fn` whose body classifies as executable may also be called from
  executable code, so users write the function once. This is Dafny 4's compiled
  `function`.
- Closures and function-typed parameters appear in specs only through
  `f.requires(args)` and `f.ensures(args, out)` (Verus's
  `call_requires`/`call_ensures`). A closure literal may carry
  `requires`/`ensures`. In v1, `FnMut` closures get no spec about their
  captured state.
- Every type has a model. The default is structural identity, and
  `impl Model` overrides it. Specs therefore need no bounds.
- An executable call inside a spec is a diagnostic with the fix "declare it
  `spec fn`".

### B5. D10: "one spec, three uses" is false for a large class of specs, and the reports would lie about it

**Problem.** Some specs cannot run at all:

- `forall |x: int|` over an unbounded domain, and unbounded `exists`;
- `final(..)`, whose value lies in the future;
- `f.ensures(..)` for a closure with no executable contract;
- `old(*v)` on a non-`Clone` type, since a runtime snapshot needs a clone.

Others run but change asymptotic cost. The design's own `max_index` re-checks
the loop invariant `forall j < i ==> xs[best] >= xs[j]` every iteration. That is
O(n²) in debug builds, about 5·10¹¹ comparisons at n = 10⁶. A
`requires sorted(xs)` turns every O(log n) binary search into O(n). Teams will
switch `assert` mode off, or, worse, the evaluator will quietly skip clauses
while the report still says "checked". Generating code for runtime checks also
contradicts "specs never affect code generation".

**Smallest fix.**

- The compiler classifies every clause statically as `checked`, `sampled`, or
  `unchecked`. A clause is executable when all quantifiers are bounded by a
  syntactic range or by collection membership, it uses no `final`, and every
  `old` applies to a snapshot-able value.
- `assert` mode checks `requires`, `ensures`, and type invariants at function
  boundaries.
- `test` mode also checks loop invariants on every iteration.
- `ferox check` and the trust report list the `unchecked` clauses per mode.
- Reword D10 to: "specs never affect typing or release code generation, and
  ghost values never flow into executable values (mode-checked)."

### B6. D10 + D4 + D11: trait contracts, impl obligations, and laws are missing, so generic proofs rest on unchecked assumptions

**Problem.** Take a generic proof such as
`fn sort<T: Ord>(v: &mut Vec<T>) ensures is_sorted(final(v)@)`. It is sound
only if `Ord` is a total order. Rust-style traits carry no laws. A hand-written,
non-transitive `impl Ord for Version` in an `assert`-mode package makes `sort`
"proved" and its output unsorted. With D11's dictionary passing, the generic
proof is checked once and the impl is linked later, so nothing ever re-checks
the combination. The same applies to:

- `Clone`: proved code needs `y@ == x@` after `let y = x.clone()`;
- `Eq`;
- `Hash`, which must agree with `Eq` for `HashMap`'s spec.

Verus guards std traits with `obeys_eq_spec()` and `obeys_ord_spec()`, and its
guide states order laws as `proof fn transitive` and similar items inside the
trait.

**Smallest fix.**

- Trait method declarations carry contracts. Impls inherit them and may not
  change them in v1, so behavioral subtyping holds automatically.
- **Laws are bodiless `proof fn` items in the trait.** Every impl must supply
  their proofs.
- The closed derives of D6 get compiler-generated law proofs.
- A manual impl whose laws are unproven (for example, one in an `assert`-mode
  package) is a counted trust hole, and so is every proof that instantiates
  through it.
- std states laws for `Eq`, `Ord`, `Hash`, and `Clone`.

D4's coherence rules (impl lives with the type or the trait, no overlap, no
specialization) put each law proof next to its impl, so this stays modular.

### B7. D13 + D10 + D11: the spec hash covers the wrong thing, the hole list is incomplete, and approval enforces nothing

**Problem 1: the hash covers clause text, not meaning.**
`ensures is_sorted(result@)` stays byte-identical while an agent edits
`spec fn is_sorted` to return `true`. The same trick works through any of these
without touching an `ensures`:

- a type invariant;
- an `impl Model`;
- a trait law;
- a `const` used in a spec;
- the enum variants matched in a spec.

**Problem 2: the hole list misses most real holes.** D10 counts `assume`,
`#[trusted]`, and `unsafe`. Not counted:

- **Proved code calling an unproved contract.** Per-package modes mean a
  `prove` package assumes every `ensures` of its `assert`-mode dependencies.
  This is the biggest hole in the design.
- Contracts on `extern` functions, Rust crates (D12), and Bend kernels (D14).
  Verus has `assume_specification` for this case.
- A bodiless `proof fn`, which is an axiom. "A lemma proved elsewhere" is also
  an axiom.
- `diverges`.
- Unproven trait laws (B6).
- Mode downgrades in the manifest, and per-function verification skips.
- Values constructed through D6 `Shape` for types with invariants. D6's
  opt-out can be forgotten, and the verifier assumes the invariant holds for
  every value.

**Problem 3: the agent can write the lockfile.** "Require approval" is not a
mechanism. An agent with commit access can update the lockfile. A proof cache
"by content hash" (D11) that the agent can write is also forgeable: a forged
`proved` entry is an unchecked `assume`.

**Problem 4: approving every spec edit causes fatigue.** Humans will
rubber-stamp, and no study shows that humans review specs reliably
(`research/04` open question 3).

**Smallest fix.**

1. **Spec-closure hash.** Hash the normalized contract together with the
   transitive closure of everything its meaning depends on: spec function
   bodies, models, type invariants, trait laws, constants, and type
   definitions.
2. **Complete, transitive hole accounting.** Count every item in the list
   above, per function and transitively through calls. The lockfile records
   each dependency's holes and verification mode. As with authority, an
   upgrade that adds holes fails until a human approves it.
3. **Enforcement outside the agent.** The lockfile is a change detector. The
   gate is a required human reviewer at merge (CODEOWNERS or a signed
   approval). CI verdicts come only from fresh runs or CI-written caches.
4. **Approve only root specs.** Root specs are public API contracts, the
   invariants of public types, and `#[protected]` tests. Under `prove`,
   internal helper specs are proof scaffolding and need no approval, because
   their only job is to make root proofs go through.
5. **Machine-classified diffs.** `ferox review` asks the solver whether each
   root-spec change is a refinement (old requires ⇒ new requires, and new
   ensures ⇒ old ensures). It marks refinements as auto-approvable, marks
   weakened or incomparable changes for human review, and attaches a
   solver-generated distinguishing example to each. People judge examples far
   better than formulas.

### B8. D10 + D13: no vacuity or spec-strength checks, so a weak spec passes every gate

**Problem.** Every one of these passes the hole checks:

- an unsatisfiable `requires` (`xs.len() > 0 && xs.is_empty()`), or a
  contradictory type invariant;
- an `ensures` already implied by the `requires`;
- an incomplete spec: `sort` ensures `is_sorted` but not "is a permutation", so
  `Vec::new()` verifies;
- a quantified `ensures` whose trigger never fires at call sites.

Vericoding found about 9% of *successful* specs too weak (`research/04` F4).
IronSpec found 10 spec bugs across all six real-world verified systems it
examined. Dafny ships `--warn-contradictory-assumptions` and analysis of
redundant assumptions because this failure is common even among experts.

**Smallest fix.** Root specs must pass these checks, and all are reported in
`ferox check`:

1. `requires` together with the parameter invariants is satisfiable.
2. No `ensures` is implied by `requires` alone.
3. Some output violates each `ensures`.
4. Every proof gets contradictory-assumption warnings, as in Dafny.
5. **Spec examples.** Protected tests evaluate `f.ensures((args,), out)` on
   good and bad outputs. This reuses B4's predicate form; see the grammar
   section.
6. A nightly mutation run mutates implementation bodies. Any mutant that still
   proves is reported as a spec gap.

### B9. D7 + D8 + D10: specs over capabilities and shared state are undefined, and D7 contradicts D10

**Problem.** D7 says pure functions are "eligible for ... proofs", which
implies capability-using functions are not. D10 makes `prove` a per-package
setting. So can `fetch_feed(net: &Net, ...)` have an `ensures`, and what does
the verifier know about `net.get`? Suppose std gives `Dir` a world model, such
as `Map<Path, Bytes>`, so that code can prove "after `write`, `read` returns
`data`". That proof is unsound: D8 children share `&Dir`, and so do other
processes. `lock (a, b)` cells change between locks, so reading one twice in a
spec is meaningless.

**Smallest fix.** State the v1 rule.

- Capabilities have no spec model. Capability methods have contracts only on
  their return values, and these are usually empty.
- A `spec fn` cannot take a capability.
- **Effects are specified by authority (D7), not by contracts.**
  Capability-using functions *can* be proved for panic-freedom and for
  properties of the values they compute.
- **Shared cells.** The lock invariant is the type invariant of the cell's
  payload type. It is assumed on lock entry and proved when the body's
  `&mut` borrow ends (B3). D8 forbids nested locks and capabilities inside lock
  bodies, so this classic concurrent-separation-logic rule is sound here with
  no ordering or reentrancy reasoning. Channel payloads keep their type
  invariants.

## Non-Blocking Issues

**N1. D10 + D11: "opaque by default" conflates two things, and D11's
"a body edit never re-checks dependents" is false for spec functions.**

- Executable bodies should *never* be visible to callers, with no `reveal`.
  This keeps D11's claim true.
- Spec function bodies should be transparent with fuel 1, with `opaque` plus
  `reveal f` as the opt-in. This matches the Dafny default under which the
  82% vericoding result was measured. Verus's `open` and `closed` modifiers
  are the alternative.
- Public spec-function bodies, models, type invariants, and laws belong in the
  D11 interface artifact and in the spec hash (B7).
- Define the proof cache key as the hash of all of the following:
  - the function's normalized body, including ghost code;
  - its own contract;
  - its callees' interface contracts;
  - the reachable transparent or revealed spec bodies;
  - the invariants, models, and laws of the types it mentions;
  - the solver build, the rlimit, and the encoding version.

**N2. D13 + D11: proofs cannot fit the 100 ms check target.** SMT queries take
from 0.1 s to tens of seconds. `ferox check` should report a `pending` status
for each obligation, and `ferox verify` should stream results. The verdict must
say which obligations are still pending, never "ok".

**N3. D10: agents need more than a pinned solver to iterate on proofs.**

- Use deterministic resource limits (rlimit), never wall-clock timeouts.
  Otherwise the D11 cache and CI disagree across machines.
- Record each obligation's rlimit use, and warn when it exceeds about half the
  limit. That margin is an early brittleness signal. Dafny has a brittleness
  measurement mode for this.
- Prune each query to the facts reachable from the function's interface
  imports.
- Give one diagnostic per failed obligation, with a hint category: missing
  invariant, needs `reveal`, nonlinear, or trigger.
- **Confirm counterexamples in the reference interpreter (D12).** Report
  `confirmed_bug` when a concrete run violates the spec, and
  `proof_incomplete` when it does not. The SMT model is often spurious under
  quantifiers, and this distinction is the single most useful signal for a
  repair loop (ExVerus, `research/04` F8).
- Extend D13's typed holes to proof holes. `proof { _?goal }` should print the
  goal, the relevant facts, and any counterexample.
- Add lemma search by type, and a small standard lemma library like Verus's
  `vstd`. It is served by `ferox doc` and does not count against the 25K-token
  core reference.

**N4. D10: triggers.** Infer triggers and print the choice in diagnostics.
Require `#[trigger]` only when inference fails, and reject matching loops. If
explicit triggers are mandatory everywhere, agents will scatter them to silence
errors. Drop the word "decidable" from the design's claims. Default obligations
(overflow, bounds) are mostly quantifier-free linear integer arithmetic, which
is decidable. User quantifiers are not. Promise determinism through rlimit, not
decidability.

**N5. D10: `for` loop semantics.** The design's `max_index` relies on knowing
`i == xs.len()` at loop exit, and `best < i` must hold at entry even when the
range is empty. Define Dafny's rule: the loop variable is in scope in the
invariant, ranges over `lo..=hi` at invariant points, and equals `hi` at exit.
In v1, allow `for` under `prove` only over ranges, plus `while` and `loop` with
explicit invariants. Iterator loops need a prophetic `produced` sequence, which
Verus has. Defer that to v2.

**N6. D10: per-package mode is too coarse.** Use per-module modes with a
package default, so a team can prove one core module. Record each module's mode
in the trust surface. Lower the entry cost with SPARK-style contextual analysis:
calls to private, non-recursive helpers that have no contract are inlined for
proof instead of requiring contracts. GNATprove does exactly this for local
subprograms without contracts.

**N7. D10 + D3 + D12: do not let proofs remove runtime safety checks in v1.**
SPARK Silver and `research/04` implication 9 suggest letting proven facts drop
bounds and overflow checks. That puts the solver and the encoding into the
memory-safety TCB. Verus's guide says plainly that "the memory safety of a
verified program is conditional on verification." Keep the checks in v1 and let
LLVM remove the ones it can. Revisit once proof certificates exist.

**N8. D10 release erasure: keep `requires` checks at package boundaries when
the caller is unproven.** A proved library called by unproved code in a release
build loses its guarantees silently when a caller violates a precondition. One
compare per public call is cheap insurance, in the Eiffel and Racket tradition.

**N9. D9: "results equal sequential execution" is false for `par_reduce` with
non-associative operators.** A fixed reduction tree over `f64` addition does not
equal a sequential left fold. If std's spec says `par_reduce == fold`, proofs
over floats are unsound. Specify `par_reduce` as `tree_fold(shape(n), op, xs)`,
and provide a lemma that `tree_fold == fold` when `op` carries an associativity
law (a `proof fn`, as in B6). `par { a(), b() }` over pure closures really is
sequential composition, and the verifier can ignore it.

**N10. D10 + D4: termination through trait dispatch.** A generic lemma can call
`T::law()`, and an impl's law proof can call the generic lemma at its own type.
That cycle is invisible per function and proves `false`. D11 forbids import
cycles, which confines recursion to within a module plus impl edges. Run one
package-level cycle check over the call graph including impl edges, and key its
cache on that graph.

**N11. D6 + D10: `Shape` construction must be automatically unavailable for
any type with a declared invariant or a private field.** An opt-out that an
agent can forget is a soundness hole (B7). Property-test generators (`test`
mode) must build such types through their public constructors.

**N12. D10 `test` mode: generators must respect preconditions.** Rejection
sampling almost never produces inputs satisfying `sorted(xs)`. Derive
generators from simple `requires` shapes, or seed them from SMT models. Report
how many generated inputs satisfied the precondition, and treat a low count as
a vacuous test. Capability-using functions are tested under `Root::sim` seeds.
D7 makes this work.

**N13. Naming: D2 "view types" collide with the Verus `View`/`@` model
abstraction that models already know.** Call the spec abstraction a **model**
(`x@` reads "the model of x"). Otherwise agents will map D2 views onto Verus
`View` and write nonsense.

**N14. Interactive proofs and Lean.** Leaving interactive proofs out of v1 is
right, because program proofs are weakest in Lean (`research/04` F3). But "a
lemma proved elsewhere" in D10 is an unverifiable claim, so call it `axiom` and
count it. The right later escape hatch is **Lean export of spec-level lemma
statements only**, never program export:

- These lemmas are pure statements over `int`, `Seq`, `Set`, `Map`, and
  algebraic data types, which is where RL-trained Lean provers are strongest.
- The trusted base is the statement translator plus Lean's kernel.
- The lockfile records the Lean proof's hash, and the item counts as an axiom
  until that proof checks.

Keep the spec sublanguage total, first-order, and free of executable calls now
so the door stays open. Also add `assert e by(compute)` through the reference
interpreter, as Verus does. It is a cheap escape hatch inside the language for
ground facts such as tables and small enumerations.

**N15. Budget.** The contract layer below costs roughly 2–3K tokens of
reference. That fits D1's 25K budget only if the lemma library and the std
spec catalog stay out of the core reference.

## Proposed Contract Grammar

This is the smallest layer I would accept. It reuses the executable expression
grammar and adds no macros. Keywords: `requires ensures invariant decreases
diverges spec proof ghost old final result forall exists assume axiom trusted
reveal opaque`.

```ebnf
(* items *)
fn_item     ::= vis? "trusted"? "fn" name generics? "(" params ")" ret? from?
                fn_spec* block
fn_spec     ::= "requires" sexpr | "ensures" sexpr
              | "decreases" sexpr ("," sexpr)*          (* lexicographic *)
              | "diverges"                               (* exec only; on the spec surface *)
spec_fn     ::= vis? "opaque"? "spec" "fn" name generics? "(" params ")" "->" type
                ("requires" sexpr)* ("decreases" slist)? (block | ";")
                                                         (* ";" = uninterpreted; no axioms *)
proof_fn    ::= vis? "axiom"? "proof" "fn" name generics? "(" params ")"
                ("requires" sexpr)* ("ensures" sexpr)* ("decreases" slist)?
                (block | ";")                            (* ";" legal only with axiom or in a trait *)
type_inv    ::= struct_or_enum_decl "invariant" sexpr    (* `self` in scope *)
model_impl  ::= "impl" "Model" "for" type "{" "type" "M" "=" type ";"
                "spec" "fn" "model" "(" "&" "self" ")" "->" "Self::M" block "}"
trait_item  ::= fn_sig fn_spec* ";"                      (* contract inherited, fixed in v1 *)
              | "spec" "fn" ... ";"                      (* abstract model operation *)
              | "proof" "fn" ... fn_spec* ";"            (* law: every impl proves it *)

(* statements *)
loop_spec   ::= "invariant" sexpr | "decreases" slist    (* on while / for-over-range / loop *)
ghost_block ::= "proof" "{" pstmt* "}"                   (* erased; mode-checked *)
pstmt       ::= "assert" sexpr ("by" "(" ("nonlinear" | "bit_vector" | "compute") ")"
                           | "by" "{" pstmt* "}")? ";"
              | "assert" "forall" binders sexpr "by" "{" pstmt* "}" ";"
              | "reveal" path ";"
              | "let" "ghost" pat "=" sexpr ";"
              | path "(" args ")" ";"                    (* lemma call *)
              | "assume" sexpr ";"                       (* hole, counted *)

(* spec expressions = executable expressions without exec calls, plus: *)
sexpr       ::= expr
              | ("forall" | "exists") binders sexpr      (* binders ::= "|" x ":" T ("," x ":" T)* "|" *)
              | sexpr ("==>" | "<==>") sexpr
              | "old" "(" place ")" | "final" "(" place ")"
              | sexpr "@"                                (* model *)
              | path "." ("requires" | "ensures") "(" tuple ("," sexpr)? ")"
              | "#[trigger]" sexpr                       (* only inside a quantifier *)
              | "result"                                 (* only in ensures *)
spec types  ::= int | nat | Seq<T> | Set<T> | Map<K,V> | any capability-free type
```

Rules. Each is one line in the reference.

1. **Modes.**
   - Spec code calls only spec functions, `@`, and std spec aliases.
   - Proof code also calls proof functions.
   - Executable code may call spec functions that classify as executable.
   - Ghost values never flow into executable values.
2. **Termination.** `spec fn` and `proof fn` must terminate; this is required
   for soundness. Under `prove`, executable functions must terminate unless
   they declare `diverges`. `decreases` is inferred for ranges and structural
   recursion.
3. **Panics.** Under `prove`, every panic site is an obligation, including the
   runtime `assert(e)`. Nothing is ever assumed from a runtime check.
4. **`&mut`.**
   - In `requires`, `*x` is the value at entry.
   - In `ensures`, a bare `*x` of a `&mut` parameter is an error; write
     `old(x)` or `final(x)`.
   - `final` is allowed on `&mut` parameters, results, and view fields.
   - Invariants are assumed of `final` values and proved when the borrow ends.
5. **Arithmetic.**
   - Machine integers widen to `int` on read.
   - Spec arithmetic never overflows.
   - `/` and `%` truncate, as in Rust.
   - Division by zero in a spec is a well-formedness error.
6. **Well-formedness.** Clauses are checked left to right, and each may assume
   the earlier ones. So `xs[result]` is legal after `result < xs.len()`.
7. **Equality.** Spec `==` is model equality, extensional on collections.
   Floats compare bitwise, and float arithmetic is uninterpreted.
8. **Models.** Every type has a model. The default is structural, and
   `impl Model` overrides it. Specs never add bounds.
9. **Opacity.**
   - Executable bodies are never visible to callers.
   - Spec bodies are visible with fuel 1 unless declared `opaque`.
   - Public spec bodies are part of the interface and of the spec hash.
10. **Triggers.** Triggers are inferred and reported. `#[trigger]` is needed
    only when inference fails. Matching loops are rejected.
11. **Executability.** A clause is `checked` only if it is executable: every
    quantifier is bounded by a syntactic range or by collection membership, it
    uses no `final`, and every `old` applies to a snapshot-able value.
    Otherwise it is reported as `unchecked` under `assert` and `test`.
12. **Holes.** These are keywords, counted per function and transitively:
    - `assume`, `axiom`, `trusted`, `unsafe`;
    - contracts on `extern` functions, Rust crates, and Bend kernels;
    - calls from proved code into unproved contracts;
    - unproven trait laws.

Worked examples:

```ferox
pub struct Stack { items: Vec<u64>, cap: usize }
    invariant self.items.len() <= self.cap

impl Model for Stack {
    type M = Seq<u64>;
    spec fn model(&self) -> Seq<u64> { self.items@ }
}

impl Stack {
    pub fn top_mut(&mut self) -> Option<&mut u64> from self
        ensures match result {
            Some(r) => old(self)@.len() > 0
                && *r == old(self)@.last()
                && final(self)@ == old(self)@.update(old(self)@.len() - 1, *final(r)),
            None => old(self)@.len() == 0 && final(self)@ == old(self)@,
        }
    { ... }
}

pub trait Ord: Eq {
    spec fn le(&self, other: &Self) -> bool;
    proof fn le_total(a: &Self, b: &Self) ensures a.le(b) || b.le(a);
    proof fn le_trans(a: &Self, b: &Self, c: &Self)
        requires a.le(b) && b.le(c) ensures a.le(c);
    proof fn le_antisym(a: &Self, b: &Self)
        requires a.le(b) && b.le(a) ensures a@ == b@;
    fn cmp(&self, other: &Self) -> Ordering
        ensures match result {
            Less => self.le(other) && self@ != other@,
            Equal => self@ == other@,
            Greater => other.le(self) && self@ != other@,
        };
}

pub fn map<T, U>(xs: &[T], f: &Fn(&T) -> U) -> Vec<U>
    requires forall |i: int| 0 <= i < xs@.len() ==> f.requires((&xs[i],))
    ensures result@.len() == xs@.len()
    ensures forall |i: int| 0 <= i < xs@.len() ==> f.ensures((&xs[i],), result[i])
{ ... }

// Lock invariant = payload type invariant; assumed on entry, proved at exit.
struct Budget { used: u64, limit: u64 } invariant self.used <= self.limit

// Spec examples: protected, executable, and they catch the "return empty" cheat.
#[protected]
#[test]
fn sort_spec_examples() {
    assert(sort.ensures((Vec::from([3, 1, 2]),), Vec::from([1, 2, 3])));
    assert(!sort.ensures((Vec::from([3, 1, 2]),), Vec::new()));
    assert(!sort.ensures((Vec::from([3, 1, 2]),), Vec::from([1, 1, 2])));
}
```

## Strengths Worth Keeping

- **Auto-active SMT as the primary layer, using Dafny and Verus clause names,
  with contracts in the grammar.** This is the best-supported choice in the
  evidence: Dafny 82%, Verus 44%, and Lean 27% on vericoding, and Verus's
  "Macro Barrier" (`research/04` F2, F7).
- **Unbounded `int`/`nat` plus `Seq`/`Set`/`Map` in specs, machine integers in
  code, and overflow as an obligation.** This keeps the property that makes
  Dafny easy for LLMs without Dafny's runtime cost.
- **Ownership with shared-xor-mutable and second-class references.**
  Verification conditions need no heap model. With no lifetimes and returns
  only through `from`, prophecies are confined to a few places, a smaller
  surface than Creusot or Verus face.
- **D2 field-level borrow declarations double as frame conditions.** Fields
  outside the declared set are unchanged, so no Dafny-style `modifies`/`reads`
  clauses are needed. Make this explicit and put the declarations in the
  hashed signature.
- **No mutable globals, no thread-locals, no ambient authority (D7).** No
  hidden state enters a verification condition, and "pure" is a type fact the
  verifier can trust for determinism.
- **D8's restricted `lock` and D9's sequential semantics.** Together they make
  lock invariants trivially sound and let the verifier ignore parallelism
  entirely. Few languages get this for free.
- **D4's power ceiling.** With no overlap, no specialization, and
  definition-checked generics, a generic proof is done once and remains valid
  under dictionary passing and monomorphization alike.
- **D11 interface artifacts that carry contracts, with per-function caching.**
  This is modular verification aligned with compilation units. Proofs (unlike
  specs) are not part of the trust surface, so agents can repair proofs after
  solver upgrades without needing human approval. This is a major
  operational win; state it explicitly.
- **Counted trust holes, a trust-surface diff, and protected specs and tests.**
  This answers the cheat rates measured by VeruSAGE, where 2–14% of agent
  solutions cheated, falling below 1.5% with a mechanical checker.
- **A pinned solver and a CI stability check under seed changes**, which
  follow Mariposa.
- **A reference interpreter as the semantic oracle (D12).** It enables
  executable specs, counterexample confirmation (N3), and `by(compute)` (N14).
- **No interactive proofs and no dependent types in v1.**

## Sources

- Verus guide, full text (`old`/`final`, the removal of bare `*x` in `ensures`,
  `f.requires`/`f.ensures`, `no_unwind`, `obeys_eq_spec`/`obeys_ord_spec`,
  trait laws as `proof fn`, `open`/`closed`, `when_used_as_spec`,
  `by(compute)`, Euclidean `/` in ghost code, `assume_specification`, and
  "memory safety of a verified program is conditional on verification"):
  https://verus-lang.github.io/verus/guide/print.html
- Creusot README and guide (absence of panics by default, partial correctness
  with `#[check(terminates)]` opt-in, termination of logic and ghost code as a
  soundness requirement, type invariants proved at the end of `&mut`
  lifetimes): https://github.com/creusot-rs/creusot (`guide/src/termination.md`,
  `type_invariants.md`, `trusted.md`, `limitations.md`)
- Dafny reference manual (`--warn-contradictory-assumptions`, analysis of
  redundant assumptions and proof dependencies, `opaque`/`reveal`/`hide`,
  brittleness measurement): https://dafny.org/latest/DafnyRef/DafnyRef
- Goldweber et al., "IronSpec: Increasing the Reliability of Formal
  Specifications," OSDI 2024 (ten spec bugs across six real-world verified
  systems): https://www.usenix.org/conference/osdi24/presentation/goldweber
- SPARK contextual analysis of subprograms without contracts:
  https://docs.adacore.com/spark2014-docs/html/ug/en/source/how_to_write_subprogram_contracts.html
  and https://blog.adacore.com/contextual-analysis-of-subprograms-without-contracts
- Benchmark numbers (vericoding, AlgoVeri, VeruSAGE, VERINA, Mariposa) are as
  cited in `research/04-verification.md`.
