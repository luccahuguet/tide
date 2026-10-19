# Ferox Contracts

Status: draft 1. This chapter is the normative definition of D10 (contracts)
and D11 (trust surface) in `design/01-design.md`, draft 2.1. It elaborates those
decisions and must not contradict them. Where it needs a decision changed, it
says so in the last section; until `01-design.md` changes, the decision there
wins.

Inputs: the round-1 reviews in `reviews/`, chiefly
`round1-verification.md`, and `research/04-verification.md`. Prior art is
cited by name: Verus, Dafny, Creusot, Prusti, and SPARK.

## 0. Terms

- **Contract**: the clauses attached to an item. Functions take `requires`,
  `ensures`, `decreases`, and `diverges`; types and loops take `invariant`;
  traits declare `law fn`s.
- **Use**: `assert`, `test`, or `prove`, as in D10.
- **Mode**: the kind of code. *Spec code* is clause expressions and `spec fn`
  bodies. *Proof code* is `proof fn` bodies, law proofs, `proof { }` blocks,
  and `ghost let`. *Exec code* is everything else. Spec and proof code are
  *ghost*: no build emits code for them, except the runtime checks of §2.
- **View**: the spec value of an executable value, written `x@`.
- **Obligation**: one fact to establish at one program point, such as a
  callee's precondition or the absence of overflow.
- **Trust closure**: the set of unproved facts that a verdict depends on (§8).
- **Root spec**: a spec that needs human approval to weaken (§11).

## 1. Syntax

### 1.1 Words

Every contract word is a contextual keyword, so Rust code that uses `result`,
`old`, or `proof` as an identifier keeps compiling.

| Word | Recognized |
| --- | --- |
| `requires`, `ensures`, `decreases`, `diverges` | after a function signature or closure parameter list |
| `invariant` | after a struct, enum, or loop header |
| `spec`, `proof`, `law` | before `fn` |
| `proof` | before `{` at statement start |
| `ghost` | before `let` |
| `result`, `old`, `final`, `forall`, `exists`, `choose` | inside spec expressions |
| `assert`, `assume`, `reveal` | as statements inside proof blocks |

Attributes: `#[spec]` (§1.6), `#[trusted(reason = "...")]` (§8.3),
`#[opaque]` (§8.1), `#[protected]` and `#[spec_test]` (§10, §11), and
`#[contracts(assert | test | prove)]` (§2.1). `final` is already reserved in
Rust, so it never collides. The runtime
`assert!` keeps its Rust meaning; a bare `assert e;` statement exists only in
proof code.

### 1.2 Grammar

The grammar extends the Rust item and expression grammar that D1 adopts.
`sexpr` is a spec expression (§1.3). Braces `{ }` denote repetition, brackets
`[ ]` an option.

```ebnf
(* Functions *)
fn_item     = attrs vis? "fn" ident generics? "(" params ")" ret? from? where?
              { fn_clause } block ;
fn_clause   = "requires" sexpr
            | "ensures" sexpr
            | "decreases" sexpr { "," sexpr }        (* lexicographic *)
            | "diverges" ;
trait_fn    = attrs "fn" ident generics? "(" params ")" ret? from? where?
              { fn_clause } ( block | ";" ) ;
impl_fn     = attrs vis? "fn" ident generics? "(" params ")" ret? from? where?
              { "requires" sexpr } { "ensures" sexpr }
              [ "decreases" sexpr { "," sexpr } ] block ;     (* refines, §5.2 *)

(* Ghost functions *)
spec_fn     = attrs vis? "spec" "fn" ident generics? "(" params ")" "->" type
              where? { "requires" sexpr } [ "decreases" sexpr { "," sexpr } ]
              ( "{" sexpr "}" | ";" ) ;
proof_fn    = attrs vis? "proof" "fn" ident generics? "(" params ")" where?
              { "requires" sexpr } { "ensures" sexpr }
              [ "decreases" sexpr { "," sexpr } ] ( pblock | ";" ) ;
law_decl    = attrs "law" "fn" ident generics? "(" params ")"
              { "requires" sexpr } { "ensures" sexpr } ";" ;       (* in a trait *)
law_proof   = attrs "law" "fn" ident generics? "(" params ")" pblock ;  (* in an impl *)

(* Types: invariant clauses sit where Rust puts a where clause *)
struct_item = attrs vis? "struct" ident generics?
              ( where? { type_inv } "{" fields "}"
              | "(" tuple_fields ")" where? { type_inv } ";" ) ;
enum_item   = attrs vis? "enum" ident generics? where? { type_inv }
              "{" variants "}" ;
type_inv    = "invariant" sexpr ;

(* Loops *)
while_expr  = "while" expr { loop_clause } block ;
for_expr    = "for" pat "in" expr { loop_clause } block ;
loop_expr   = "loop" { loop_clause } block ;
loop_clause = "invariant" sexpr | "decreases" sexpr { "," sexpr } ;

(* Closures: clauses force a block body *)
closure     = "move"? "|" cparams "|" [ "->" type ] { fn_clause } ( expr | block ) ;

(* Ghost statements *)
ghost_let   = "ghost" "let" "mut"? pat [ ":" type ] "=" sexpr ";" ;
proof_block = "proof" pblock ;
pblock      = "{" { pstmt } "}" ;
pstmt       = "assert" sexpr [ by ] ";"
            | "assert" "forall" binders sexpr "by" pblock ";"
            | "assume" sexpr ";"                      (* trust hole *)
            | "reveal" path ";"
            | ghost_let
            | ident "=" sexpr ";"                     (* ghost mut only *)
            | path [ generic_args ] "(" args ")" ";"  (* lemma or law call *)
            | "if" sexpr pblock [ "else" ( pblock | pstmt ) ]
            | "match" sexpr "{" { pat "=>" pblock } "}" ;
by          = "by" "(" ( "nonlinear_arith" | "bit_vector" | "compute" ) ")"
              { "requires" sexpr }
            | "by" pblock ;

(* Spec expressions *)
sexpr       = equiv ;
equiv       = implies { "<==>" implies } ;
implies     = rust_or [ "==>" implies ] ;           (* right associative *)
rust_or     = (* Rust's ||, &&, comparison, arithmetic, unary, postfix levels,
                 with sprimary as the primary and "@" as a postfix operator *) ;
sprimary    = rust_primary
            | ( "forall" | "exists" ) binders { trigger_set } sexpr
            | "choose" binders sexpr
            | "|" ident ":" type { "," ident ":" type } "|" sexpr   (* spec lambda *)
            | "old" "(" splace ")" | "final" "(" splace ")"
            | "result"
            | postfix "." ( "requires" | "ensures" ) "(" args ")"
            | "#[trigger]" postfix ;                (* inside a quantifier body *)
binders     = "|" ident ":" type { "," ident ":" type } "|" ;
trigger_set = "#![trigger" sexpr { "," sexpr } "]" ;
splace      = "*" ident { "." ( ident | int ) }           (* rooted at a &mut, §4.1 *)
            | ident { "." ( ident | int ) } ;
```

Rules that the grammar does not show:

1. **One expression per clause.** A top-level comma in a clause is an error
   whose safe fix splits it into repeated clauses. Verus's comma lists are
   rejected so that each clause gets its own classification and diagnostic.
2. **Order.** Clauses of one kind are conjoined in source order. For
   well-formedness (§1.3), each `requires` may assume the earlier ones; each
   `ensures` may assume every `requires` and the earlier `ensures`.
3. **`result`** is valid only in `ensures`. A function with `ensures` cannot
   have a parameter named `result`; the safe fix renames it.
4. **Parameters in `ensures`.** A by-value or `&T` parameter denotes its value
   at entry. A bare `*x` of a `&mut` parameter `x` is an error with two
   machine-applicable fixes, `old(*x)` and `final(*x)` (§4.1).
5. **`decreases`** on a function bounds recursion; on a loop it bounds
   iterations (§3.3). `diverges` is legal only on exec functions.
6. **Impls refine trait contracts.** An impl method inherits its trait
   method's contract and may weaken the precondition and strengthen the
   postcondition, never the reverse (§5.2).
7. **Bodiless ghost functions.** A bodiless `spec fn` is uninterpreted, and its
   result type must be inhabited. A bodiless `proof fn` is an axiom and must
   carry `#[trusted(reason = "...")]`.
8. **Type invariants** see the value as `self` and may mention only private
   fields (§4.2).
9. **Quantifier, `choose`, and lambda bodies** extend as far right as
   possible, as in Verus and Dafny.

### 1.3 Spec expressions

A spec expression is an executable expression without assignment, `&mut`,
loops, `return`, `break`, `?`, closures (spec lambdas excepted), or `!` forms
other than `vec![...]` used as a literal. It calls only `spec fn`s, built-in
spec operations, and `#[spec]` functions (§1.6). It adds the forms in §1.2.
Its meaning is the same in all three uses:

- **Integers.** Reads of machine integers widen to `int`. `+ - * / %` and
  unary `-` compute in `int` and never overflow. `/` and `%` truncate toward
  zero, as in Rust, so `-7 / 2 == -3`; `div_euclid` and `rem_euclid` give
  Euclidean results. Verus's ghost `/` is Euclidean; Ferox chooses Rust's
  rule so that the same text means the same thing in code and in specs.
  Comparisons compare as `int`. Bitwise operators and shifts take operands of
  one machine type, compute at that width with Rust's meaning, then widen.
  `e as u32` on an `int` is well-formed only when the value fits.
- **Equality.** Spec `==` is structural equality of spec values: fieldwise on
  structs, enums, and tuples; extensional on `Seq`, `Set`, `Map`, and
  `Multiset`; on floats it compares D3's canonical `to_bits()`, so it is
  reflexive even for NaN. It never calls a user `PartialEq`. For abstract
  equality, write `a@ == b@`.
- **References.** `&e` and `*e` are transparent: a `&T` denotes the `T`.
- **Floats** are IEEE 754 values, not reals: `a + b` in a spec is the rounded
  IEEE sum, and `<` is the IEEE comparison. Default queries treat float
  operations as uninterpreted, which is sound because it only forgets facts;
  `by(bit_vector)` and `by(compute)` interpret them exactly (§9).
- **Well-formedness.** Some expressions are meaningful only under a
  condition: `s[i]` needs `0 <= i < s.len()`; `x / y` needs `y != 0`; a call
  to a spec fn needs its `requires`; `choose |x: T| p` needs `exists |x: T| p`;
  `x@` needs the invariant of `x` (§4.2); a value passed where `nat` is
  expected needs to be non-negative. `&&`, `||`, `==>`, `if`, and `match`
  supply the context for their right-hand parts. Under `prove`, the condition
  is an obligation discharged once where the expression is written. Under
  `assert` and `test`, a violated condition stops evaluation and is reported
  as `spec_error` against the clause's author. Spec evaluation never panics
  the program and never changes program state.

### 1.4 Spec types and views

The spec types are `bool`, `char`, the machine integers, `int`, `nat`,
`f32`/`f64` (IEEE values), tuples, `Option`, `Result`, user structs and
enums, `Seq<T>`, `Set<T>`, `Map<K, V>`, `Multiset<T>`, and `spec_fn(A) -> B`
for spec lambdas. They exist only in ghost code; an exec signature or exec
local cannot have a spec-only type.

Every type has a view. The standard library defines the `View` trait with
Verus's spelling:

```ferox
pub trait View {
    type V;
    spec fn view(&self) -> Self::V;
}
```

`x@` means `x.view()`. A type without a `View` impl is its own view:
`x@ == x`. The standard library's views:

| Executable type | `x@` |
| --- | --- |
| `bool`, `char`, machine integers, floats, tuples, `Option`, `Result` | the value itself |
| `&T`, `Box<T>`, `Rc<T>`, `Arc<T>` | `(*x)@` |
| `[T; N]`, `[T]`, `Vec<T>`, `VecDeque<T>` | `Seq<T>` |
| `str`, `String` | `Seq<char>` |
| `HashMap<K, V>`, `BTreeMap<K, V>` | `Map<K, V>` |
| `HashSet<T>`, `BTreeSet<T>` | `Set<T>` |
| capabilities, `Shared<T>`, channel ends, task handles, `Cell<T>`, `RefCell<T>`, atomics | opaque identity token |

Opaque values can be compared for identity but not read through, so a spec
never observes interior-mutable or shared state (soundness review B10b).

Inside `view`, the invariant of `self` is a precondition, so `x@` is
well-formed only where the invariant of `x` is known (§4.2). The view of a
standard collection is also its spec encoding, which is why `==` on two
`Vec`s is extensional on their elements. The word "model" in D10 means this
view.

Visibility follows Rust's private-in-public rule: a public contract may
mention only public spec fns. A public spec fn whose body reads private
fields is *closed*: its body is part of the interface hash and the
documentation, but other modules' proofs cannot unfold it. That is Verus's
`closed spec fn`, inferred instead of declared.

### 1.5 Mode rules

1. Spec code calls spec fns and `#[spec]` functions. Proof code may also
   call `proof fn`s and laws. Exec code calls neither, except in clauses and
   `proof { }` blocks.
2. A `ghost let` may read exec locals; exec code cannot read ghost locals. A
   `ghost let mut` is assigned only in proof blocks. Ghost values never flow
   into exec values, and the mode checker rejects any path that would let
   them, including through control flow.
3. Parameters of spec and proof functions must be `Pure` (D7). A spec fn
   cannot take a capability or a `Shared` handle.
4. Spec and proof functions must terminate (§3.4).

### 1.6 `#[spec]` functions

D10 lets specs call exec functions marked `#[spec]`, so one function serves
both worlds and `xs.len()` means the same in a clause as in code. A
`#[spec]` function must satisfy all of the following, checked by the compiler:

1. It is an inherent function or method. Trait methods cannot be `#[spec]` in
   v1; traits give methods a spec meaning through abstract spec fns (§5.1).
2. It is authority-free; its parameters are by value or `&T`, never `&mut`;
   every parameter and result type is `Pure`.
3. Its body calls only `#[spec]` functions and built-in operators. This makes
   the result a function of the arguments' spec values: operations that see
   more than the spec value, such as `Vec::capacity`, `HashMap` iteration
   order, `Rc::ptr_eq`, `Rc::strong_count`, or `Cell::get`, are not
   `#[spec]`, so they cannot be reached.
4. It is not `diverges`, and it is verified under `prove` whatever use its
   module selects: panic-free under its `requires` and terminating. A
   `#[spec]` function that does not verify is a compile error, not a trust
   hole.

In spec code, a call `f(a)` denotes the value `f` returns on `a`. Its
`requires` are well-formedness conditions, and its `ensures` describe the
value; its body stays opaque (§8.1). Under `assert` and `test` the evaluator
runs the function. The standard library marks its pure accessors (`len`,
indexing, `get`, `contains`, `is_some`, `unwrap`, `first`, `last`, and
similar) with exact `ensures`, such as `ensures result == self@.len()`; these
are part of the toolchain base (§8.3), conditional on the laws of their type
arguments.

Calling any other exec function in a spec is an error with two fixes: declare
a `spec fn`, or mark the function `#[spec]` if it qualifies.

## 2. The three uses

### 2.1 Selecting a use

The manifest sets the package default (`[contracts] use = "assert"` when
absent). A module overrides it with
`#![contracts(test)]` and an item with `#[contracts(prove)]`. Uses are
cumulative: `test` does everything `assert` does, and `prove` keeps `test`
behavior for whatever it has not proved. The trust surface records each item's
use. Setting a module or item below its package's use, or lowering the
package's use, is a downgrade and needs approval (D11, §11).

### 2.2 Clause classes

The compiler gives every clause one class, the same in every use, and
`ferox check` reports it per clause.

- **`proof-only`** if the clause contains an unbounded quantifier, `choose`, a
  call to an uninterpreted spec fn, `final` of a prophetic borrow (§4.1),
  `.requires` or `.ensures` on a function value other than a named function,
  `Set::new` or `Map::new` over a predicate, or a call to a spec fn whose body
  is proof-only. A quantifier is *bounded* when its body has the form
  `lo <= x && x < hi ==> p` (`&&` for `exists`), with `lo` and `hi` free of
  `x`, or `c.contains(x) ==> p` for a finite collection `c`. An unsigned
  machine-integer binder carries the implicit bound `0 <= x`, so the Feel's
  `forall |i: usize| i < xs.len() ==> ...` is bounded.
- **`sampled`** if it is executable but its cost can grow with the size of a
  value: a bounded quantifier, a recursive spec fn, an `old` that must
  snapshot a collection, or a whole-collection operation such as `==` on
  sequences, `to_multiset`, `take`, `skip`, or `+`.
- **`checked`** otherwise: its cost does not depend on input sizes.

`old(e)` snapshots only what it needs. At entry the evaluator computes each
maximal subexpression of an `old` argument that mentions no `result`,
`final`, or bound variable, so `old(*v).len()` costs one word. Snapshots copy
views inside the evaluator and never call `Clone`, so `old` adds no trait
bound. When a use does not evaluate a clause, the report lists it as
unchecked in that use; nothing is skipped silently.

### 2.3 `assert`

Debug and test builds evaluate `checked` clauses:

- `requires` at callee entry; a failure blames the call site;
- `ensures` at each return; a failure blames the callee;
- type invariants at the exits of functions in the type's defining module that
  produce a value of the type or hold `&mut` to one (§4.2);
- loop invariants at loop entry and at loop exit.

The evaluator uses unbounded integers and persistent sequences, linked into
debug builds. Any pending deferred trap (§3.2) fires before a check runs.

### 2.4 `test`

`test` adds three things.

**Sampled clauses.** At function boundaries, `sampled` clauses are evaluated
on the first 64 calls of each function in a test run, then on calls 128, 256,
512, and so on. Each evaluation has a step budget, and one that runs out is
counted as `skipped`, never as checked. Packages can change both defaults.

**Loop invariants and measures.** Executable loop invariants are evaluated
before the first iteration, after every iteration, and at exit. `sampled`
invariants follow the geometric schedule over iterations, which turns
`max_index`'s quadratic invariant into linear total work. Loop `decreases`
measures are evaluated every iteration and must stay non-negative and
strictly decrease. In `for x in a..b`, an invariant sees `x` as the index of
the next iteration: it is checked with `x == a` before the first iteration,
even when the range is empty, and with `x == max(a, b)` at exit. This is
Dafny's rule, and `max_index` needs it to conclude `best < xs.len()`.

**Generated tests.** Every function with an `ensures`, and every law (§5),
gets a property test.

1. Structural types derive a generator; a type implementing std's
   `Arbitrary` supplies its own. A type with an invariant or private fields is
   built only through random sequences of public constructor and method calls.
   Capabilities come from `Root::sim(seed)` with fault injection. Function
   parameters are not generated in v1.
2. The `requires` conjuncts are matched against generator shapes: scalar
   ranges, length bounds, an index below a length (generated after the
   length), `sorted`, `distinct`, `contains`, `is_some`, and equalities that
   fix a value. Matched conjuncts are produced constructively; unmatched ones
   filter.
3. When the solver is available, the corpus is seeded with solver models of
   the `requires`, made distinct by blocking clauses.
4. Shrinking keeps the `requires` true.
5. The report counts generated inputs and inputs that satisfied the
   `requires`. Below a floor (by default 100 accepted cases or 1% acceptance)
   the test fails as `vacuous`, so a filtered test cannot pass on zero cases.
6. A failure carries the seed, shrunk input, failing clause, and replay
   command (D14). `ferox test --fuzz` adds coverage-guided mutation.

### 2.5 `prove`

The solver discharges every clause and every panic site (§3). Debug and test
builds keep the `test` checks for any clause whose obligations are not proved
with an empty trust closure (§8.2); clauses proved that way are not checked
again at run time.

### 2.6 Release builds and check deletion

Release builds erase clauses and ghost code. The manifest can keep checks
with `release_checks = "none" | "pub_requires" | "checked"`; the default is
`none`, as D10 says.

D3 lets a proof delete a runtime check only if every fact it relies on is
proved in the same build or still checked at run time. A check for overflow,
division, shifts, `unwrap`, or `assert!` is therefore deleted only when both
of the following hold:

1. its obligation is proved with an empty trust closure, by proofs this
   build's verifier checked;
2. every `requires` the proof used is guaranteed. For a private function, every
   caller in the package is proved. For a `pub` function, each such `requires`
   stays checked at entry in every profile. Only a `checked` clause can stay
   checked, so a check that depends on a `sampled` or `proof-only`
   precondition is never deleted.

Bounds checks follow the same rule but are deleted only under
`release_delete_bounds_checks = true`, off by default in v1. A wrongly deleted
bounds check is memory unsafety, not a wrong value, so deleting one puts the
solver and encoding in the memory-safety base (verification review N7).

Deleting checks is the only way specs affect code generation. Clauses never
change types, layouts, or which code runs.

## 3. Total correctness under `prove`

`prove` verifies every function in scope, including ones without contracts,
whose implicit contract is `requires true`. A function verifies when, for
every input that satisfies its `requires`, it reaches no panic site,
terminates unless it is `diverges`, and at each return satisfies its
`ensures` and the invariants it owes (§4.2). Allocation failure, which
aborts, and stack overflow are outside the guarantee. `#[test]` functions are
not verified.

### 3.1 Panic sites

| Site | Obligation |
| --- | --- |
| `+ - *`, unary `-` | the result is in range |
| `/`, `%` | the divisor is non-zero, and the operation is not `MIN / -1` |
| `<<`, `>>` | the shift is below the bit width |
| `xs[i]`, `xs[a..b]`, `str` slicing | in bounds, and on char boundaries for `str` |
| a call | the callee's `requires`, including argument invariants (§4.2) |
| `unwrap`, `expect` | `is_some()` or `is_ok()` (std `requires`) |
| `assert!(e)`, `assert_eq!`, `debug_assert!` | `e` |
| `panic!`, `unreachable!`, `todo!`, `unimplemented!` | the point is unreachable |
| `RefCell::borrow_mut` and similar | unprovable in v1, which has no borrow-flag model; use `try_borrow_mut` |

`checked_*`, `wrapping_*`, `saturating_*`, `try_from`, and lossless `as`
carry no obligation. Channel, `lock`, and capability operations report
failure through `Result`. A runtime `assert!` is never an assumption: its
condition must be proved, and only then is it known. There is no Dafny-style
`expect`. When a callee was not proved in this build, the caller's verdict
assumes the callee's `ensures`, panic-freedom, and termination, and lists them
in its trust closure.

### 3.2 Arithmetic and the deferred-trap rule

D3 lets an overflow trap be reported late; design open question 1 asks how
that interacts with proofs. For contracts the answer is three rules.

1. **Proofs use precise traps.** Each arithmetic operation is an obligation at
   its own site, whatever the code generator batches. A proved function never
   reaches a trap, so deferral cannot change verified behavior.
2. **Deferral is unobservable except in abandoned memory.** A program with a
   deferred trap behaves like the precise program except for the contents of
   places the abandoned task wrote after the overflow, which D5 makes
   unobservable (§4.3), and for the moment within the window when the trap
   fires. The compiler may defer only across code whose control flow and
   addresses do not depend on the overflowed value; a loop-exit test that
   reads the value is a flush point. Differential runs against the reference
   interpreter check this compiler obligation.
3. **Contract checks are flush points.** Under `assert` and `test`, every
   clause evaluation, including each loop-invariant check, first fires any
   pending trap, so a check never sees a wrapped value.

### 3.3 Termination

- **Loops.** Every loop needs a measure. `for x in a..b` infers `b - x`;
  writing it is allowed, as `max_index` does. `while a < b` infers `b - a`,
  and `while a > b` infers `a - b`. Any other loop needs `decreases`: an
  `int` expression or a lexicographic tuple of them, non-negative whenever an
  iteration begins and strictly smaller after it. In v1, `for` under `prove`
  iterates only over integer ranges; other `for` loops are errors whose safe
  fix rewrites them as index loops. Iterator loops need a prophetic sequence
  of produced items, as in Verus, and are deferred. A variable that a loop
  never assigns or mutably borrows keeps its facts across the loop without an
  invariant.
- **Recursion.** Recursive exec, spec, and proof functions need `decreases`,
  except structural recursion on an enum argument, where each call passes a
  value matched out of it. Mutually recursive functions in one module compare
  measures lexicographically at each call.
- **Function values.** A closure's calls count as calls by the function that
  creates it, and naming a function as a value adds an edge to it. A call
  through a function value adds none. A call in a closure body must decrease
  the creating function's measure, which stays sound when the closure
  escapes: each activation reached that way has a smaller measure than the
  one that created the closure.
- **`diverges`** declares partial correctness: `ensures` holds if the function
  returns. A function that is not `diverges` cannot call one that is. D11
  counts `diverges` as a trust hole that needs approval. Capability operations
  are assumed to return; a peer that never answers is outside contracts.

### 3.4 Soundness of ghost code

Careless ghost code proves `false`, so these are errors, not lints.

1. `spec fn`s and `proof fn`s always terminate. `diverges` is illegal on them,
   and recursion needs a measure (Creusot's `falso()` shows why).
2. Termination is checked over a package-level call graph that includes impl
   edges. Each interface exports, for each generic function, the trait
   methods and laws of its type parameters that it calls. A cycle such as a
   generic lemma calling `T::law` while an impl's law proof calls that lemma
   is then caught by the downstream build, even across packages.
3. A bodiless `spec fn` must return an inhabited type. A bodiless `proof fn`
   is an axiom and must be `#[trusted]`.
4. A spec fn's body must be well-formed under its `requires`, checked at the
   definition; every use must establish those `requires`.

## 4. Mutable borrows, invariants, and shared cells

### 4.1 `old` and `final`

For a `&mut` parameter `x`:

- In `requires`, `x` and `*x` denote the value at entry. `old` and `final` are
  errors there.
- In `ensures`, `old(*x)` is the value at entry. `old(x)` means the same,
  because references are transparent in specs; the formatter writes `old(*x)`.
- If the result does not borrow from `x`, `final(*x)` is the value `*x` holds
  when the function returns. It is executable.
- If the result borrows from `x` (`-> &mut V from x`, or a borrowing result
  with a `&mut` field), `final(*x)` is a **prophecy**: the value `*x` will hold
  when the caller's borrow ends. `final(*r)` for a `&mut` `r` inside the result
  (`final(*result)`, a binding `Some(r)`, or a field path `final(*result.buf)`)
  is likewise the value at the end of that borrow. Prophetic `final` is
  `proof-only`.

This is Verus's current rule and the RustHorn encoding that Creusot uses and
RustHornBelt proved sound. In the caller, when a returned borrow `r` ends at
its last use, the verifier learns that `final(*r)` equals the current `*r`,
and the callee's `ensures` then determines `final(*x)`. D2 confines
prophecies to `from` returns and to borrowing types with `&mut` fields;
ordinary `&mut` arguments need none.

```ferox
impl<K: Eq + Hash, V> Table<K, V> {
    pub fn get_mut(&mut self, k: &K) -> Option<&mut V>
        ensures match result {
            Some(r) => old(*self)@.contains_key(k) && *r == old(*self)@[k]
                && final(*self)@ == old(*self)@.insert(k, final(*r)),
            None => !old(*self)@.contains_key(k) && final(*self)@ == old(*self)@,
        }
}
```

Inside the `match`, `*r` is the value when `get_mut` returns. A method with
declared field borrows, `fn advance(&mut self.{pos})`, gets a free frame
condition: every field outside the set has `final == old`. The field-borrow
declaration is part of the hashed signature (§11).

### 4.2 Type invariants

```ferox
pub struct Span
    invariant self.lo <= self.hi
{
    lo: usize,
    hi: usize,
}
```

A type may carry several `invariant` clauses. A clause's *footprint* is the
set of fields it reads, and those fields must be private, so only the defining
module can break an invariant. A type with an invariant cannot opt in to
`Shape` construction (D6). The invariant `inv(T)` is the conjunction of the
clauses and the invariants of the field types; for std containers it includes
the invariants of their elements, as in Creusot.

The verifier **assumes** `inv` of: each parameter at entry, including the entry
value of a `&mut`; each call result; the value a `&mut` argument holds after
the call; every prophetic `final`; and a lock payload at body entry. It
**owes** `inv` for: each argument at a call; each return value; the final
value of each `&mut` parameter at return; and a lock payload at body exit.
Between those points, code in the defining module may break the invariant
freely. Because `inv(self)` is a precondition of `view`, a spec that mentions
`x@` while `x` is broken fails well-formedness instead of reading garbage.

Under `assert` and `test`, invariant clauses are checked at the exits of the
defining module's functions that return a value of the type or hold `&mut` to
one. Elsewhere the fields are private, so no check is needed.

A method that borrows `&mut self.{fields}` assumes and owes each clause whose
footprint lies inside its field set, ignores each clause disjoint from it,
and is rejected if it splits a clause's footprint (soundness review B10(d)).
Derived impls owe the invariant too: `Default` for its value, `Clone` for its
result, discharged by a fixed schema when every field's view is structural;
`Decode` checks every clause at run time and returns `Err` on violation, and
the derive is rejected if any clause is `proof-only`.

### 4.3 Panics and abandoned values

D5 treats every place a panicking child borrowed mutably as moved out on the
scope's error path. D5 drops the abandoned value, as Rust's unwinding does,
so resources are released. Contracts therefore need one more fact:
`Drop::drop` is verified without assuming `Self`'s invariant, since it may
run on a possibly broken value. No other verified code assumes the invariant
of an abandoned value, and the writes a deferred trap leaves behind (§3.2) land only
in such places.

### 4.4 `Shared<T>` and `lock`

A `Shared<T>` handle has an opaque view, so specs can compare handles but
cannot read the payload, and spec fns cannot take handles (they are not
`Pure`). In `lock (a, b) |x, y| { body }`, the body is verified as a closure
with `x: &mut T` and `y: &mut U`. It assumes `inv(*x)` and `inv(*y)`, and that
the cells are distinct, because the run-time distinctness check comes first.
It owes total correctness and `inv(final(*x))`, `inv(final(*y))`. This is the
lock-invariant rule of concurrent separation logic, and it needs no
interference reasoning here: D8 bodies call only authority-free functions,
cannot lock or suspend, and hold `Pure` payloads, and a panicking body poisons
its cells.

Callers learn only about the `R` in `Ok(r)`, constrained by the body's
`ensures` with the payload's entry and final values existentially
quantified. They learn nothing about the payload afterwards, because another
task may lock it next. A poisoned cell returns `Err(Poisoned)` from every
later `lock`. The standard library offers no way to read a poisoned payload;
`reset(v)` replaces it, and `inv(v)` is owed at that call. Channel payloads
keep their invariants because sending is a call.

## 5. Traits and generics

### 5.1 Trait contracts and abstract spec fns

Trait methods carry contracts. A trait may declare bodiless `spec fn`s that
each impl defines, and a method's contract can tie the method to one of them.
The standard library states its core traits this way:

```ferox
pub trait PartialEq {
    spec fn eq_spec(&self, other: &Self) -> bool;
    fn eq(&self, other: &Self) -> bool
        ensures result == self.eq_spec(other);
}
pub trait Eq: PartialEq {
    law fn eq_is_view_eq(a: &Self, b: &Self)
        ensures a.eq_spec(b) <==> a@ == b@;
}
pub trait Ord: Eq + PartialOrd {
    spec fn cmp_spec(&self, other: &Self) -> Ordering;
    fn cmp(&self, other: &Self) -> Ordering
        ensures result == self.cmp_spec(other);
    law fn reverse(a: &Self, b: &Self)
        ensures a.cmp_spec(b) == b.cmp_spec(a).reverse();
    law fn transitive(a: &Self, b: &Self, c: &Self)
        requires le(a, b) && le(b, c)
        ensures le(a, c);
    law fn agrees_with_eq(a: &Self, b: &Self)
        ensures (a.cmp_spec(b) == Ordering::Equal) <==> a@ == b@;
}
pub spec fn le<T: Ord>(a: &T, b: &T) -> bool { a.cmp_spec(b) != Ordering::Greater }
```

`Eq`'s single law says equality is view equality. A type whose equality is
coarser than its structure, such as case-insensitive names, expresses that by
giving the type a `View` that normalizes, which keeps `HashMap`'s `Map` view
honest. `Hash` has one law: values with equal views feed the hasher equal
bytes. `Clone::clone` has the contract `ensures result@ == self@`.

An impl must define each abstract spec fn. Under `assert` and `test` an impl
may omit one; it is then uninterpreted for that impl, and clauses that
mention it are `proof-only` there. That keeps Rust-style
`impl Ord for Version { fn cmp(..) }` compiling. `test` still checks such an
impl's laws *operationally*: it replaces each application `a.cmp_spec(b)`
with a run of the paired method `a.cmp(b)`, the method whose contract is
`ensures result == self.cmp_spec(other)`. That is a test, not a definition,
so the impl's laws are `tested`, never `proved`. Under `prove`, omitting an
abstract spec fn is an error.

### 5.2 Refinement

An impl method inherits its trait method's contract and may restate it. Its
`requires` must be implied by the trait's, and its `ensures` are conjoined with
the trait's, so the impl can only weaken the precondition and strengthen the
postcondition (D10). Under `prove`, `trait.requires ==> impl.requires` is an
obligation. Under `assert` and `test`, when both are executable, the entry
check evaluates the trait's `requires` first; if it holds and the impl's
fails, the failure is a `refinement_violation` against the impl. Calls on a
concrete type use the impl's contract, and calls through a bound use the
trait's.

### 5.3 Laws

A `law fn` is a bodiless proof obligation in a trait. An impl discharges it
with `law fn name(..) { proof }` or omits it, in which case the verifier tries
an empty proof. Each impl's laws get a status: `proved` under `prove`;
`tested` when `test` runs them as property tests; `assumed` under `assert`, or
when the law proof is `#[trusted]`.

Generic code may rely on the laws of its bounds, called as lemmas:
`T::transitive(&a, &b, &c);`. Laws are not ambient quantified axioms, for
solver stability. A generic proof is parametric and checked once, so
monomorphization and witness tables share it. Its reliance is settled at
each *concrete instantiation*: a call with concrete type arguments, a `dyn`
coercion, or the use of a default method by an impl. There the caller's trust
closure gains every law of the instantiated impls that is not `proved`. A
generic caller passes the reliance up to its own instantiation sites. This is
how the unsound `max_of::<Weird>` example of the soundness review (B10a)
becomes a counted assumption.

Derived impls carry generated proofs: the standard library proves, once per
derive kind, lemmas such as "a lexicographic order over total orders is
total", and a derived impl instantiates them with its fields' laws. Its status
is the weakest among its fields' impls.

`par_reduce` is specified as `tree_fold(canonical_shape(xs.len()), op, xs)`.
A std lemma equates it with `fold` given the operation's
`law fn associative`; used with a `tested` law, the lemma's conclusion is a
counted assumption.

## 6. Closures and function values

A closure literal may carry clauses, which force a block body:

```ferox
let scale = |x: &i64| -> i64 requires *x < 1_000_000 ensures result == *x * 1000 { *x * 1000 };
```

In specs a function value `f` appears only as `f.requires(a, b)` and
`f.ensures(a, b, r)`, with arguments in `f`'s parameter order and, for
`ensures`, the result last. A `&mut` parameter position takes a pair
`(entry, final)`. Named functions work the same way:
`binary_search.ensures(xs, key, r)`, `sort::<i64>.ensures((v0, v1), ())`.
Verus's tuple form `f.requires((x,))` is rejected with a safe fix.

- A closure literal with clauses means its clauses.
- A closure literal without clauses whose body is one expression built from
  operators, field access, and `#[spec]` calls gets an inferred contract:
  `requires` is the expression's panic-freedom conditions and its callees'
  `requires`, and `ensures result == body`. So `|x: &i64| *x * 2` requires
  that the product fits. This is SPARK's contextual analysis applied to the
  commonest closures.
- Any other closure without clauses has `requires true` and `ensures true`.
- For a function-typed parameter, `f.requires` and `f.ensures` are
  uninterpreted predicates.

They are relations, not functions: `f.ensures(a, r1) && f.ensures(a, r2)` does
not give `r1 == r2`. Specs never call function values, so a closure that
captures `&Net` cannot be mistaken for a mathematical function (verification
review B4). For `FnMut` closures in v1, clauses may not mention captured
mutable state, and a caller learns nothing about captures the closure
mutated; proved code uses loops for that. At run time each closure checks its
own clauses on every call, so `f.requires(..)` on a function value is
`proof-only` at the higher-order function's boundary without losing a check.

```ferox
pub fn map<T, U>(xs: &[T], f: &dyn Fn(&T) -> U) -> Vec<U>
    requires forall |i: int| 0 <= i && i < xs.len() ==> f.requires(&xs[i])
    ensures result.len() == xs.len()
    ensures forall |i: int| 0 <= i && i < xs.len() ==> f.ensures(&xs[i], result[i])
```

## 7. Capabilities

Capabilities have no spec model in v1 (D10). Capability values and handles
have opaque views, and spec and `#[spec]` functions cannot take them.
Capability methods carry contracts only on their arguments and results, such
as `ensures match result { Ok(b) => b.len() <= max, Err(_) => true }` on a
bounded read. The verifier treats each capability call's result as an
arbitrary value that satisfies that method's `ensures`, so a postcondition of
an effectful function holds for every possible response of the world.

Under `prove`, an effectful function can be proved panic-free, terminating
(given that capability calls return), and correct about the values it
computes. What it does to the world is described by authority (D7), not by
contracts. Under `test`, effectful
functions run under `Root::sim` seeds with fault injection.

## 8. Modularity, status, and caching

### 8.1 What callers see

- **Exec bodies are never visible** to callers' proofs, and there is no way to
  reveal one. The one exception is SPARK's: a private helper with no clauses,
  no recursion, and no loops is inlined into its callers in the same module.
- **Spec fn bodies are transparent** with fuel 1. `#[opaque]` hides one, and
  `reveal f;` in a proof block unfolds it for the rest of that block. A closed
  spec fn (§1.4) is opaque outside its module. D10's "bodies are opaque to
  callers by default" is read as exec bodies.
- The interface artifact (D12) carries contracts, public spec fn bodies,
  views, invariants, laws, and generic trait-call summaries (§3.4). A body edit
  never re-checks dependents' proofs, because proofs see only contracts and
  spec fns; editing a public spec fn body is an interface change.

### 8.2 Status

`ferox verify` streams NDJSON verdicts per obligation: `pending`, `proved`,
`failed` with `confirmed_bug` or `proof_incomplete`, `unknown` (resource limit
reached), or `unstable` (§9). A solver counterexample is replayed in the
reference interpreter; `confirmed_bug` means a concrete run violates the
contract. `ferox check` never runs the solver, to keep its 100 ms budget: it
reports classes and cached verdicts, marks the rest `pending`, and never says
`ok` while obligations are pending.

Each item exports one D11 status, `proved`, `tested`, or `assumed`, together
with its **trust closure**: the trust holes its verdict depends on,
transitively. `proved` with an empty closure is shown as `proved`; otherwise
it is `proved, assuming N`, and the review lists the N.

### 8.3 Trust holes

Counted per function and transitively: `assume`; `#[trusted]` functions, law
proofs, and bodiless `proof fn`s; `#[trusted_pure]`; `unsafe` blocks;
contracts of `extern` functions, Rust shims, and Bend kernels (Bend claims
carry the `external` label of D16); dependency contracts a proof relies on that
were not proved in this build, including panic-freedom and termination; laws
that are not `proved` at a concrete instantiation; `diverges`; and use
downgrades. Each hole carries its reason string.

Not counted per use, but listed once per toolchain version: the verification
condition generator and its encoding, the pinned solver, the reference
interpreter (for `by(compute)` and spec tests), and the standard library's
contracts and `#[spec]` accessors.

### 8.4 Proof cache

A function's proof is cached under the hash of: its normalized body,
including ghost code; its contract; its callees' interface contracts; the
bodies of every transparent or revealed spec fn it reaches; the invariants,
views, and laws of the types and bounds it mentions; inlined helper bodies;
and the solver build, options, resource limit, and encoding version. CI
rebuilds proofs into its own cache; a cache an agent can write is advisory
only (D11).

## 9. Solver discipline

- **Small queries.** One query per function, split per loop and per group of
  obligations, with context pruned to facts reachable from interface imports,
  revealed spec fns, and mentioned types.
- **Triggers** are inferred, and each choice is printed as a note.
  `#[trigger]` or `#![trigger ...]` is required only when inference finds none,
  and may override it. Matching loops are rejected.
- **Linear by default.** Default queries treat a product of two non-constant
  terms, division or remainder by a non-constant, bitwise operators, and float
  operations as uninterpreted. `assert e by(nonlinear_arith) requires p;` and
  `by(bit_vector)` run isolated queries that see only the listed `requires`;
  `by(compute)` evaluates a ground expression in the reference interpreter
  under fuel.
- **Deterministic limits.** Each obligation has a resource limit (rlimit),
  never a wall-clock timeout; `#[rlimit(n)]` raises it up to a toolchain cap.
  Using more than half the limit is a warning, an early brittleness signal.
- **Pinned solver.** Each toolchain release pins one solver build. Upgrades
  rerun all proofs. Proofs are not on the trust surface, so agents repair them
  without approval.
- **Stability CI.** `ferox verify --stability N` reruns each obligation under
  N variants (solver seeds, shuffled declarations, renamed symbols), following
  Mariposa. An obligation that fails under any variant is `unstable`, which
  fails CI when a root spec depends on it and warns otherwise.
- **Diagnostics.** One per failed obligation, with its kind, span, and a hint
  category: missing invariant, needs `reveal`, nonlinear, trigger, or weak
  callee contract. A proof hole `proof { _?goal }` prints the goal, the facts
  in scope, and any counterexample.

## 10. Spec quality

Root specs (§11) must pass checks 1–3; all results appear in `ferox check`
and next to each contract in `ferox review`.

1. **Satisfiability.** The `requires` together with the parameter invariants
   must be satisfiable, and each type invariant must have a model. The
   solver's witness is shown.
2. **Non-triviality.** For each `ensures`, the solver must find an output that
   violates it while the `requires` hold. A clause implied by the `requires`
   alone is an error for root specs (`#[allow(trivial_ensures, reason = "..")]`
   overrides it) and a warning elsewhere.
3. **No contradictory assumptions.** For each obligation, the facts assumed on
   the way to it must be satisfiable, as Dafny's
   `--warn-contradictory-assumptions` checks. An obligation proved only from a
   contradiction fails.
4. **Trivial bodies.** The contract is evaluated against synthesized trivial
   implementations: return `Default`, an empty collection, the first argument,
   or leave a `&mut` unchanged. A contract that accepts one is flagged in
   review ("admits: return empty"). This is a signal, not an error.
5. **Mutation testing** runs in CI on implementation bodies (operators,
   constants, conditions, deleted statements). A mutant that still verifies
   under `prove`, or still passes under `test`, is reported as a spec gap, and
   the share of mutants each contract rejects is its strength score.
6. **Spec tests.** A `#[spec_test]` function holds ground spec assertions,
   evaluated under `test` and decided by `by(compute)` under `prove`:

   ```ferox
   #[protected]
   #[spec_test]
   fn sort_examples() {
       assert sort::<i64>.ensures((vec![3, 1, 2], vec![1, 2, 3]), ());
       assert !sort::<i64>.ensures((vec![3, 1, 2], vec![]), ());
       assert !sort::<i64>.ensures((vec![3, 1, 2], vec![1, 1, 2]), ());
   }
   ```

## 11. Approval and anti-gaming

**Root specs** are the contracts of public items, the invariants of types
reachable from the public API, the laws of public traits, `#[protected]` tests
and spec tests, and the use settings. Under `prove`, internal specs are proof
scaffolding and need no approval.

**What is hashed.** Each root spec has a *spec-closure hash* over the
normalized clauses, the signature (types, `from` origins, field-borrow sets),
and, transitively, everything its meaning depends on: spec fn bodies (opaque
and closed ones included), the bodies and contracts of `#[spec]` functions
(their bodies define their spec meaning), views, invariants, type
definitions, trait laws and trait method contracts, constants, and the
standard library's spec catalog version. Names resolve to the content hashes
of D14, so a rename leaves the hash unchanged and an edit to
`spec fn is_sorted` changes it. Proofs, `proof fn` bodies, and other exec
bodies are excluded.

**Classifying changes.** For a changed root spec, the solver checks
refinement between the old and new closures, renamed apart:
`old.requires ==> new.requires`, and
`old.requires && new.ensures ==> old.ensures`. A refinement, or a contract on
a previously unspecified item, is listed in review without a gate. A
weakened, incomparable, or undecided change needs approval and comes with a
distinguishing example, a solver model replayed in the interpreter. As D11 lists, approval is needed
for weakening or removing a contract, widening authority, adding a trust hole
(including `diverges`), downgrading a use, and changing or unprotecting a
`#[protected]` test.

**Mechanism.** An approval record holds the item's stable ID, the old and new
closure hashes, the change class, the approver's key ID, and a signature.
Records live in the repository; the allowlist of approver keys lives outside
it, in CI configuration the agent cannot write. The CI gate compares the
candidate with the protected base branch and requires, for every gated
change, a valid record whose new hash matches exactly. Hashes, lockfiles, and
caches written in the same change never count. Locally, `ferox check` reports
`pending_approval`. CI runs a protected test only if its hash matches an
approved one. Agent loops run `ferox verify --deny-new-holes`, the mechanical
cheat checker that brought VeruSAGE's cheating rates below 1.5%.

## 12. Worked examples

### 12.1 Binary search, proved

```ferox
pub spec fn sorted(s: Seq<i64>) -> bool {
    forall |i: int, j: int| 0 <= i && i <= j && j < s.len() ==> s[i] <= s[j]
}

#[contracts(prove)]
pub fn binary_search(xs: &[i64], key: i64) -> Option<usize>
    requires sorted(xs@)
    ensures match result {
        Some(i) => i < xs.len() && xs[i] == key,
        None => forall |i: int| 0 <= i && i < xs.len() ==> xs[i] != key,
    }
{
    let mut lo: usize = 0;
    let mut hi: usize = xs.len();
    while lo < hi
        invariant hi <= xs.len()
        invariant forall |i: int| 0 <= i && i < lo ==> xs[i] < key
        invariant forall |i: int| hi <= i && i < xs.len() ==> xs[i] > key
    {
        let mid = lo + (hi - lo) / 2;
        if xs[mid] < key {
            lo = mid + 1;
        } else if xs[mid] > key {
            hi = mid;
        } else {
            return Some(mid);
        }
    }
    None
}
```

The obligations: `hi - lo` does not underflow (the loop condition);
`lo + (hi - lo) / 2 <= hi` and `mid + 1 <= hi` do not overflow; `mid < xs.len()`;
the invariants hold initially and after each branch, where the solver
instantiates `sorted` at `(i, mid)` through the inferred trigger
`{s[i], s[j]}`; the inferred measure `hi - lo` decreases; and both returns meet
the `ensures`. Under `test`, the quadratic `requires` is `sampled` (§2.4),
and the generator builds sorted inputs from the `sorted` shape.

### 12.2 A bounded queue with a type invariant

```ferox
spec fn phys(head: int, k: int, cap: int) -> int {     // slot of logical index k
    if head + k < cap { head + k } else { head + k - cap }
}
spec fn offset(head: int, s: int, cap: int) -> int {   // logical index of slot s
    if s >= head { s - head } else { s + cap - head }
}

pub struct Ring<T>
    invariant self.slots.len() > 0
    invariant self.head < self.slots.len()
    invariant self.len <= self.slots.len()
    invariant forall |s: int| 0 <= s && s < self.slots.len() ==>
        (#[trigger] self.slots[s]).is_some() == (offset(self.head, s, self.slots.len()) < self.len)
{
    slots: Vec<Option<T>>,
    head: usize,
    len: usize,
}

impl<T> View for Ring<T> {
    type V = Seq<T>;
    spec fn view(&self) -> Seq<T> {
        Seq::new(self.len, |k: int| self.slots[phys(self.head, k, self.slots.len())].unwrap())
    }
}

impl<T> Ring<T> {
    pub spec fn cap(&self) -> nat { self.slots.len() }   // closed: reads a private field

    pub fn push(&mut self, x: T) -> Result<(), T>
        ensures final(*self).cap() == old(*self).cap()
        ensures match result {
            Ok(()) => old(*self)@.len() < old(*self).cap()
                && final(*self)@ == old(*self)@.push(x),
            Err(y) => old(*self)@.len() == old(*self).cap()
                && final(*self)@ == old(*self)@ && y == x,
        }
    {
        let cap = self.slots.len();
        if self.len == cap {
            return Err(x);
        }
        let tail = wrap(self.head, self.len, cap);
        proof { assert offset(self.head, tail, cap) == self.len; }   // so the slot is free
        self.slots[tail] = Some(x);
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T>
        ensures final(*self).cap() == old(*self).cap()
        ensures match result {
            Some(x) => old(*self)@.len() > 0 && x == old(*self)@[0]
                && final(*self)@ == old(*self)@.skip(1),
            None => old(*self)@.len() == 0 && final(*self)@ == old(*self)@,
        }
    {
        if self.len == 0 {
            return None;
        }
        let x = self.slots[self.head].take();
        self.head = if self.head + 1 == self.slots.len() { 0 } else { self.head + 1 };
        self.len -= 1;
        x
    }
}

fn wrap(head: usize, k: usize, cap: usize) -> usize
    requires head < cap && k < cap
    ensures result == phys(head, k, cap)
{
    if k < cap - head { head + k } else { k - (cap - head) }
}

pub fn submit(q: &Shared<Ring<Job>>, job: Job) -> Result<(), SubmitError>
    ensures match result { Err(SubmitError::Full(j)) => j == job, _ => true }
{
    match lock (q) |r| { r.push(job) } {
        Ok(Ok(())) => Ok(()),
        Ok(Err(j)) => Err(SubmitError::Full(j)),
        Err(e) => Err(SubmitError::Lock(e)),
    }
}
```

The view's `unwrap` is well-formed because of the fourth invariant, which is
why `view` may assume `inv(self)`. Inside `push`, the invariant is broken
between the write and `self.len += 1` and owed again at return. `pop` gets
its `&mut` element through `IndexMut`, whose std contract uses `final` (§4.1),
and the `self.head + 1` obligation holds because `head < cap`. In `submit`,
`lock` assumes `Ring`'s invariant for `*r` and owes it at exit, which
`push`'s postcondition supplies. The caller learns that a rejected job comes
back unchanged; it learns nothing about the queue's contents afterwards,
because other tasks share the cell (§4.4).

### 12.3 A generic sort relying on `Ord` laws

```ferox
pub spec fn sorted_by<T: Ord>(s: Seq<T>) -> bool {
    forall |a: int, b: int| 0 <= a && a < b && b < s.len() ==> le(&s[a], &s[b])
}

#[contracts(prove)]
pub fn selection_sort<T: Ord>(v: &mut Vec<T>)
    ensures sorted_by(final(*v)@)
    ensures final(*v)@.to_multiset() == old(*v)@.to_multiset()
{
    let n = v.len();
    for i in 0..n
        invariant v.len() == n
        invariant v@.to_multiset() == old(*v)@.to_multiset()
        invariant sorted_by(v@.take(i))
        invariant forall |a: int, b: int| 0 <= a && a < i && i <= b && b < n
            ==> le(&v[a], &v[b])
    {
        let mut m = i;
        proof { T::reverse(&v[i], &v[i]); }              // gives le(v[i], v[i])
        for k in i + 1..n
            invariant i <= m && m < k
            invariant forall |t: int| i <= t && t < k ==> le(&v[m], &v[t])
        {
            if v[k].cmp(&v[m]) == Ordering::Less {
                proof {
                    assert forall |t: int| i <= t && t < k ==> le(&v[k], &v[t]) by {
                        T::transitive(&v[k], &v[m], &v[t]);
                    }
                    T::reverse(&v[k], &v[k]);
                }
                m = k;
            } else {
                proof { T::reverse(&v[k], &v[m]); }      // gives le(v[m], v[k])
            }
        }
        v.swap(i, m);
    }
}
```

The inner loop does not touch `v`, so its facts about `v` carry through
(§3.3). The outer invariants after `swap` follow from std's `swap` contract,
which states the update and that the multiset is unchanged, by cases on
whether `b == m`. The proof is generic and runs once. Its trait-call summary
records that it calls `T::reverse`, `T::transitive`, and `T::cmp`.

For `selection_sort::<Version>`, where `Version` has a hand-written
`impl Ord` with only `fn cmp` in an `assert` package, the caller's verdict is
`proved, assuming 3`: `Version: Ord::reverse` (tested operationally under
`test`, otherwise assumed), `Version: Ord::transitive` (same), and the
`Version::cmp` contract (assumed, since `cmp_spec` is uninterpreted for that
impl). With `#[derive(PartialEq, Eq, PartialOrd, Ord)]` on a struct of
integers, the laws come from the std lexicographic lemmas and the verdict is
`proved` with an empty trust closure.

## 13. Open questions

1. Ghost fields and linear ghost permissions (Verus's `Ghost` and `Tracked`)
   for structures whose view cannot be computed from their fields, and for
   verifying `unsafe` code.
2. Iterator loops under `prove`, which need a prophetic sequence of produced
   items.
3. Checking prophetic `final` at run time, at the caller's borrow end.
4. Specs for interior mutability: `Cell`, `RefCell`, and reference counts.
5. Leaking versus dropping abandoned values (§4.3).
6. Proof certificates, which would let bounds checks be deleted by default.
7. Lean export of spec-level lemma statements (verification review N14).
8. Generator shapes beyond the fixed list, and the defaults for the vacuity
   floor and sampling schedule.
9. Whether reviewers judge distinguishing examples reliably; E3 should
   measure it.

## Amendments needed in `01-design.md`

1. D8: a failed distinctness check in `lock` must return `Err`, not panic;
   otherwise proved code cannot lock two handles.
2. D5: say whether abandoned values are leaked or dropped (§4.3).
3. D10: "bodies are opaque to callers by default" should say executable
   bodies; spec fn bodies are transparent at fuel 1 (§8.1).
4. D12: interface artifacts also carry public spec fn bodies, views,
   invariants, laws, and generic trait-call summaries; editing a public spec
   fn body is an interface change.
5. D11: the exported status carries its trust closure (`proved, assuming N`).
