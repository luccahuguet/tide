# Ferox Memory: Ownership and Origins

Status: draft 1. This chapter elaborates D2 of `design/01-design.md` (draft
2.1) and the memory parts of D5, D7, D8 and D9, and it is the definition the
checker implements. Section 13 lists the edits it needs in `01-design.md`.

Notation. Ferox code is in `ferox` blocks and Rust in `rust` blocks. In rules,
`ρ`, `σ` and `τ` are origins, `ℓ` is a loan, and `p` and `q` are places.
`ρ ⊆ σ` means "σ may carry every loan of ρ"; in Rust this is `'ρ: 'σ`.

## 1. Core concepts

### 1.1 Places

```
place ::= x            a local variable or parameter
        | place.f      a named or tuple field
        | *place       a dereference of a reference or Box
        | place[e]     an index; for borrowing, the whole indexed place
```

- A place is **owned** by the function when its path dereferences no
  reference. Dereferencing a `Box` keeps it owned, as in Rust. Every other
  place is **behind a reference**.
- `p` is a **prefix** of `p.f`, `*p` and `p[e]`.
- Two places **overlap** unless they diverge at different fields of the same
  struct or tuple. `v[i]` and `v[j]` overlap; `split_at_mut`, `chunks_mut`
  and slice patterns give disjoint element borrows.

### 1.2 Moves and copies

- Using a place by value **moves** out of it unless its type is `Copy`, with
  Rust's meaning: never `&mut T` or a type with a destructor.
- A moved-out place is uninitialized. Reading or borrowing it is an error
  until it is assigned. Initialization is tracked per place and flow by flow,
  at field granularity for owned places (partial moves), as in Rust.
- Nothing moves out of a place behind a reference, or out of a field of a
  type with a destructor. `mem::take`, `mem::replace` and `Option::take`
  cover those cases.
- A closure captures each place path it uses (Rust 2021 disjoint captures) in
  the weakest mode its body needs: shared borrow, mutable borrow, or move.
  `move` forces by-value capture.

### 1.3 Drops

- At the end of a scope, every initialized local is dropped, in reverse
  declaration order. A value's fields drop in declaration order. Conditionally
  moved locals use drop flags. Assigning to an initialized place drops the old
  value first. Temporaries follow Rust 2024's rules.
- `mem::forget` and `Box::leak` are safe: scopes, `lock` and `par` take
  closures, not guards, so no guarantee depends on a destructor running.

### 1.4 Borrows and loans

Evaluating `&p` or `&mut p` at program point `P` creates a **loan**
`ℓ = (p, mode, P)`, and the new reference's origin contains `ℓ` (§2.1).

**Reborrows.** If `p` dereferences references, the checker walks them from
the one nearest `p` inward. The new reference's origin includes each one's
origin, and the walk stops after the first shared reference (Rust's
supporting-prefixes rule). `&self.src[a..b]` inside a `&mut self` method
therefore carries `src`'s loans and not the `&mut self` borrow, so it can
outlive that borrow.

**Accesses** are reads, shared borrows, writes, mutable borrows, moves,
storage death (end of scope) and drops. An access of kind `k` to `q`
**conflicts** with `ℓ = (p, m)` if `p` and `q` overlap and either `m` is
mutable or `k` is not a read or a shared borrow. Storage death of `x`
conflicts only with loans of places that `x` owns. Data behind a reference
does not die with the reference, which is why a method may return
`&self.src[..]`.

```
  access of kind k to q at P      ℓ live at P      conflicts(ℓ, q, k)
  -------------------------------------------------------------------- [Loan]
                               error
```

A loan is **live** at `P` if it belongs to an origin that is live at `P`. An
origin is live at `P` if a variable that may still be used, or may still be
dropped with that origin in its drop set (§6), has a type that mentions it.
Assigning to a reference variable `r` **kills** the loans of places reached
through `*r`.

**Implicit reborrow.** Where a `&mut T` value is used and a `&mut T` is
expected (a call argument, a method receiver, an annotated `let`), it is
reborrowed as `&mut *r` rather than moved.

**Two-phase borrows** apply in exactly Rust's three cases:

- the autoref `&mut` of a method receiver;
- a mutable reborrow in a call argument;
- the implicit `&mut` of a compound assignment on a non-primitive type.

Such a borrow acts as a shared borrow until the call activates it, so
`v.push(v.len())` is accepted.

## 2. Origins

### 2.1 What an origin denotes

An **origin** is a set of loans: the loans that a reference, or a value of a
borrowing type, may have been derived from. This is Polonius's reading of a
Rust lifetime.

- When a value of type `&ρ1 T` flows into a place of type `&ρ2 T`, the
  checker requires `ρ1 ⊆ ρ2`. Rust's outlives relation `'a: 'b` is the subset
  relation `ρa ⊆ ρb`.
- `'static` holds no loans and is a subset of every origin.
- Inside a body, origins are unknowns that the checker solves.
- In a signature, origins are universally quantified: the caller chooses
  them. The body treats each signature origin as holding an unknown
  placeholder loan. It may rely on `σ ⊆ τ` between two signature origins only
  when the signature implies it, through `from` or through well-formedness.

### 2.2 Origins of a type and of a place

`orig(T)` is the set of origins that a value of type `T` holds at top level:

```
orig(&ρ T) = orig(&ρ mut T) = {ρ}
orig(S<ρ1..ρn; T1..Tm)      = {ρ1..ρn} ∪ orig(T1) ∪ … ∪ orig(Tm)   (struct, enum)
orig((T1, …, Tn))           = orig(T1) ∪ … ∪ orig(Tn); arrays and slices likewise
orig(dyn Trait + ρ) = {ρ}        orig(dyn Trait + Owned) = ∅
orig(closure)               = the union of orig over its captured types
orig(T), T a type parameter = an abstract set
orig(scalars, str, String, fn pointers, 'static references to Owned data) = ∅
```

A reference lists only its outer origin. Well-formedness of `&ρ T` requires
`σ ⊆ ρ` for every `σ` in `orig(T)`, so `ρ` already covers everything the
pointee borrows.

**The origins of a place** are `orig(p) = orig(type of p)`: the loans held by
the value stored at `p`. This is what a place means after `from`:

| Place in `from` | Type at the place | Denotes |
| --- | --- | --- |
| `x`, with `x: &Lexer` | `&ρ Lexer` | `ρ`: the borrow `x` holds, which covers everything reachable through it |
| `self.src`, with `self: &mut Lexer` | `&σ str` | `σ`: the loans in the field, not the `&mut self` borrow |
| `*words`, with `words: &[&str]` | `[&τ str]` | `τ`: the elements' origin |
| `lx`, with `lx: Ctx` passed by value | `Ctx` | all of `Ctx`'s origins |

### 2.3 Grammar of `from`

`from` is a contextual keyword. It is recognized after a type in signatures
(parameter types, return types and where clauses), in field declarations, in
`dyn` and `impl` types, and in associated-type definitions.

```
TypeFrom   ::= Type 'from' Origins
Origins    ::= Origin
             | '(' Origin (',' Origin)+ ')'               several origins
             | '{' Name ':' Origins (',' Name ':' Origins)* '}'   per-origin binding
ReturnType ::= Type ['from' Origin (',' Origin)*]         a bare list only here
Origin     ::= "'static" | Place | ('*')* 'Self' ('.' Field)+
Place      ::= ('*')* Root ('.' Field)*
Root       ::= a parameter name | 'self'
```

- `from` binds more loosely than `&`, `&mut` and `dyn`, so `&str from a` is
  `(&str) from a`. It applies to **every elided origin position** in the type
  it follows. Use parentheses to target an inner one:
  `&mut (&str from value)`.
- A bare list `from a, b` is allowed only at the end of a return type. In
  every other position, several origins are parenthesized:
  `Vec<&str from (a, b)>`.
- A place auto-dereferences like a field-access expression. It cannot contain
  method calls or indexing.
- `Self.f` names the origin of field `f` of the impl's `Self` type (§3.5).
- It is an error if `orig(P)` is empty. For `from self.pos` with
  `pos: usize`, the fix is `from self`, which covers a borrow of the field. A
  `from` may not name the parameter whose own type contains it.

### 2.4 What `from` means in each position

Every position reads the same way: the value at this position borrows from
`P`. How that is enforced depends on who produces the value.

1. **Output positions** are return types, the argument types of closure
   bounds, and annotations in a body. Each affected position gets an origin
   `ρ` with `σ ⊆ ρ` for every `σ` in `orig(P1) ∪ … ∪ orig(Pn)`: the value may
   carry any loan of `P1` to `Pn`. In Rust this is a fresh `'r` with `'p: 'r`.
   In a closure bound's argument type the origin stays higher-ranked, and the
   closure may additionally assume the inclusion (§5.4).
2. **Parameter types.** The affected positions **are** the origin of `P`,
   which must denote exactly one origin. In Rust this reuses `P`'s lifetime
   parameter. Arguments are coerced at the call site, so at a covariant
   position this means "the argument outlives `P`'s origin", and at an
   invariant position it means "is exactly `P`'s origin".
3. **Type-parameter parameters** (`f: F from P`, or `impl Trait from P`)
   impose `F: 'p` for each `p` in `orig(P)`. For an `Fn*` bound this also
   covers its output type. In Rust: `F: 'p, F::Output: 'p`.
4. **Field types.** The affected positions share the named sibling's origin
   (§3.3), which must denote exactly one origin.
5. **Per-origin binding** (`Match from { needle: a, line: b }`) applies the
   rule for the position to each named origin of the type separately. Origins
   it does not name stay elided.

### 2.5 Elision

Elision follows Rust's rules exactly.

- **Inputs.** Each elided origin position in the parameter types is a
  distinct origin. Elided positions are `&T` and `&mut T` without `from`,
  each origin of a borrowing type written without `from` (Rust's `S<'_>`),
  and `dyn Trait + '_` or `impl Trait + '_`. Origins hidden inside type
  parameters are never counted.
- **Rule 1.** If the parameters use exactly one origin (counted after `from`
  identifications), every elided output position gets it.
- **Rule 2.** If the receiver is `&self` or `&mut self`, every elided output
  position gets the receiver reference's origin.
- **Otherwise** an elided output origin is an error. The fix lists the
  candidate `from` clauses and `&'static`.
- **Closure-bound sugar.** Inside `Fn(&A) -> &B`, the same rules apply, and
  the argument origins are higher-ranked.
- **Consts and statics.** Elided origins are `'static`.
- **Fields.** Each elided position is a new origin of the struct (§3.1).
- **Associated types in impls** have no elision, as in Rust. Write
  `from Self.f`.
- **Default `dyn` bounds** are in §5.1.

### 2.6 Legacy lifetime spelling

D1 accepts Rust named lifetimes that map exactly onto origins, and the
formatter rewrites them. A lifetime `'a` maps exactly when each use can be
rewritten as follows:

- a later parameter use becomes `from` the first place that carries `'a`;
- a return-type use becomes `from` every place that carries `'a`;
- a struct use becomes `from` the first field that carries `'a`;
- an impl use becomes `Self.f`, where `f` is the field that introduces `'a`;
- `F: 'a` becomes `f: F from <place>`.

The result is at least as general as the Rust signature: a shared `'a` over
two inputs and the output becomes the union `from a, b`. These are rejected,
with suggested origins: outlives bounds beyond what well-formedness implies,
explicit `for<'a>`, lifetime parameters on traits, and lifetimes that no
parameter or field carries.

### 2.7 Modular checking

- **Interfaces.** A module's interface artifact (D12) records each
  signature's origin positions, identifications, inclusions, implied bounds
  and field-borrow sets (§7). For each public type it records the named
  origins with their variances, the auto traits (`Owned`, `Send`, `Sync`,
  `Pure`, `Frozen`) and a drop-check summary. A change to any of these is an
  API change and appears in the trust diff.
- **Calls.** The callee's origins are instantiated freshly; argument
  coercion adds subset constraints by variance (§4); the signature's
  relations become constraints on the fresh origins. The callee's body is
  never consulted.
- **Bodies.** Signature origins are placeholders, and they are live at every
  point of the body. Implied bounds come from the well-formedness of the
  parameter and return types. Any other inclusion the body needs between
  placeholders is an error, which names the `from` clause to add.
- **A local loan reaching a placeholder origin** (a returned reference to a
  local) conflicts with the local's storage death: "returns a value borrowed
  from local `x`".
- **Trait impls.** An impl's method signature must be at least as general as
  the trait's, checked by subtyping on the instantiated signatures, as rustc
  does.

### 2.8 Precision

v1 accepts a body exactly when Rust's NLL borrow checker accepts its
translation (§11). The field-borrow declarations of §7 translate exactly.

In NLL terms, subset constraints hold at every point of the body, and a
loan is live where its origin is live and reachable from the borrow without a
kill. Polonius makes subset constraints location-sensitive, which only
accepts more programs. D12's origin cross-check holds the checker to this
definition.

## 3. Borrowing types

### 3.1 Origin parameters

A struct or enum has one origin parameter for each origin position in its
field types that `from` or `'static` does not fix. The positions are:

- each `&` and `&mut`;
- each origin of a borrowing type used as a field type (a field `lx: Lexer`
  contributes `Lexer`'s origin);
- each `dyn Trait + '_` object bound.

Two further rules apply:

- Origins carried by the type's own type parameters are not origin
  parameters. They ride with the type arguments (§3.4).
- A recursive occurrence of the type being declared, or of a type in its
  recursive group, reuses the declaration's own origins. `struct Node { val:
  i32, next: Option<&Node> }` has one origin, `next`, which is Rust's
  `Node<'a> { next: Option<&'a Node<'a>> }`.

A type is a **borrowing type** if it has an origin parameter or a type
argument with origins. Otherwise it is `Owned`.

### 3.2 Names

An origin is named by the path to its position:

| Field | Origin names |
| --- | --- |
| `src: &str` | `src` |
| `words: Vec<&str>` | `words` |
| `out: &mut Vec<&str>` | `out` (the outer borrow), `*out` (the elements) |
| `lx: Lexer` | `lx.src` |
| `pair: (&str, &str)` | `pair.0`, `pair.1` |
| variant `Ident(&str)` | `Ident.0` |

When one path holds several origins, as in `m: HashMap<&str, &str>`,
diagnostics add the position (`m` key, `m` value), and `from m` names both.
Signatures name places (§2.2); origin names appear in diagnostics.

### 3.3 Sharing an origin

A field declared `b: &str from a` uses `a`'s origin, so the struct has one
origin where it would otherwise have two. The named place may be any sibling
path that holds exactly one origin: `from lx.src`, `from *out`. Order does not
matter, and mutual references collapse into one origin.

```ferox
struct Collector { src: &str, out: &mut Vec<&str from src> }   // origins: src, out
```

Rust: `struct Collector<'s, 'o> { src: &'s str, out: &'o mut Vec<&'s str> }`.

### 3.4 Origins from type arguments

`orig(Vec<&str>) = orig(&str)`: a container borrows what its elements borrow.
`struct Tagged<T> { value: T, tag: &str }` has its own origin, `tag`, plus
`orig(T)`. A field `r: &T` implies `orig(T) ⊆ r` (Rust infers `T: 'r`, RFC
2093).

### 3.5 Impls and `Self`

`impl Lexer { … }` is implicitly generic over `Lexer`'s origins, like Rust's
`impl<'src> Lexer<'src>`. `Self` denotes `Lexer` with exactly those origins.

- `Self.src` names the impl's origin. In a method that has a receiver, the
  place `self.src` denotes the same origin.
- `-> Self` carries the impl's origins. `-> Lexer` is an elided type, and the
  elision rules resolve its origin.
- Associated types must name their origins:
  `type Item = &str from Self.src`.

### 3.6 Constructing and returning borrowing types

- In a body, origins are inferred: in `Lexer { src: text, pos: 0 }`,
  `text`'s origin flows into `src`.
- `fn lex(text: &str) -> Lexer`: by rule 1, `Lexer`'s origin is `text`'s.
- In an impl, a constructor is written `fn new(src: &str) -> Lexer` (elision)
  or `fn new(src: &str from Self.src) -> Self` (identification). Rust users
  write `fn new(src: &'a str) -> Self`. Writing `fn new(src: &str) -> Self`
  is an error, with both fixes offered.
- A returned type with several origins takes either a union (`-> Match from
  a, b`) or a per-origin binding (`-> Match from { needle: a, line: b }`).

### 3.7 `Owned` and `'static`

- `T: Owned` holds when `orig(T)` is empty after substitution, counting
  `'static` as empty. This is Rust's `T: 'static`. The bound `T: 'static` is
  accepted, and the formatter rewrites it to `T: Owned`.
- `&'static T` is `Owned` when `T` is. String literals have type
  `&'static str`, so `struct ParseError { msg: &'static str }` is `Owned`.
- The types of consts and statics are `Owned`. Statics are also `Sync` and
  `Frozen`, since D7 allows no mutable globals.
- `Owned` is required by `Box<dyn Trait>` contents (§5.1), `dyn Any`, values
  handed to tasks that outlive their creator (§8.2), and global
  registrations.

## 4. Variance and subtyping

### 4.1 Rules

Ferox subtyping comes only from origins and from instantiating higher-ranked
origins.

```
  ρ1 ⊆ ρ2    T1 <: T2              ρ1 ⊆ ρ2    T1 ≡ T2
  --------------------           -------------------------
  &ρ1 T1 <: &ρ2 T2               &ρ1 mut T1 <: &ρ2 mut T2
```

Each origin parameter and type parameter of a struct or enum gets a variance,
computed as in Rust: the meet of the variances of its occurrences in the
field types. Recursive types start from bivariant and take the fixed point.

| Constructor | Variance |
| --- | --- |
| `&ρ T` | covariant in `ρ` and `T` |
| `&ρ mut T` | covariant in `ρ`, invariant in `T` |
| `Cell`, `RefCell`, `OnceCell`, `UnsafeCell`, `Shared`, `*mut T` | invariant |
| `fn(A) -> B` | contravariant in `A`, covariant in `B` |
| `dyn Trait<A> + ρ` | covariant in `ρ`, invariant in `A` |
| `Box`, `Vec`, `Rc`, `Arc`, `Option`, arrays, slices, tuples, `*const T` | covariant |

`S<ρ̄1> <: S<ρ̄2>` holds per parameter: `ρ1 ⊆ ρ2` when covariant, `ρ2 ⊆ ρ1`
when contravariant, and equality when invariant. Subtyping applies at Rust's
coercion sites. Unsizing to `dyn` may shorten the object's origin even behind
`&mut`, which is sound because an unsized place cannot be overwritten (Rust
allows the same).

### 4.2 Why these variances

- **Covariance through `&`.** A shared reference only reads, so one that
  carries fewer loans can stand in where more are allowed.
- **Invariance through `&mut`.** A mutable reference also writes. If
  `&mut Vec<&ρ1 str>` could be used as `&mut Vec<&ρ2 str>` with `ρ1 ⊆ ρ2`,
  the callee could push a string carrying `ρ2`'s loans, such as a borrow of a
  short-lived local. The owner would still believe its elements carry only
  `ρ1`, and once that local died the owner would hold a dangling reference.
- **Cells and `Shared`** write through a shared reference: the same
  argument. Function arguments are contravariant by the dual argument.

### 4.3 Why draft 1 was unsound

Draft 1 gave each borrowing type one implicit region and left its variance
unstated. Both readings fail (`reviews/round1-soundness.md` B1):

- **Invariant.** `&mut V` becomes Rust's `&'a mut V<'a>`. A view stays
  mutably borrowed for its whole life, so every `&mut self` method becomes
  one-shot.
- **Covariant.** The single region can shrink, so
  `let r: &mut Vec<&str> = &mut names; r.push(&tmp);` stores a reference to a
  dying local into `names`.

With one origin per position and Rust's variances, `&mut Vec<&str>` has two
origins: a covariant outer one and an invariant element one. The borrow of
the `Vec` can end while its elements' origin lives on.

## 5. Trait objects, `impl Trait`, closures

### 5.1 `dyn Trait`

D2 sets `dyn Trait` to mean `dyn Trait + Owned`. It also says Rust's elision
rules apply unchanged, and Rust's elision rules include default object
bounds. This chapter resolves the two as follows:

- **Directly behind a reference** (`&dyn Trait`, `&mut dyn Trait`), the
  object's origin defaults to the reference's, which is Rust's
  `&'a (dyn Trait + 'a)`. `&mut dyn FnMut(&Node)` callbacks that capture
  locals depend on it.
- **Everywhere else**, including `Box`, `Rc`, `Arc`, fields, type arguments
  and annotations on `let`, `dyn Trait` is `dyn Trait + Owned`.
- `dyn Trait from P` borrows from `P` (§2.4).
- `dyn Trait + '_` is an elided origin: in signatures it follows the elision
  rules, and in a field it is a new origin named by the field.
- `dyn Trait + 'static` is `Owned`.
- Rust's rule for containers with declared `T: 'a` bounds does not arise;
  those positions default to `Owned`, which is stricter.
- A `dyn` type has no auto traits unless it names them:
  `dyn Fn() + Send + Pure`.

### 5.2 `impl Trait`

- **Argument position.** `impl Trait` is an anonymous type parameter, as in
  Rust.
- **Return position.** The opaque type's origins are exactly those its
  `from` clause names plus the elided positions inside its bounds; with
  neither, it is `Owned`. `fn iter(&self) -> impl Iterator<Item = &u32>`
  captures `self` through the elided `&u32`, while
  `fn evens(xs: &[u32]) -> impl Iterator<Item = u32> from xs` must name `xs`,
  which Rust 2024 captures implicitly.
- **Type parameters** are captured as in Rust.
- **Auto traits** of an opaque type are only those its bounds state. Rust's
  auto-trait leakage would make dependents depend on the function's body,
  which D12 forbids.

### 5.3 Closures

- A closure's origins are the union of its captures' origins. A by-reference
  capture of `p` contributes a reference with a loan on `p`. A by-value
  capture of a value of type `T` contributes `orig(T)`.
- A closure that captures a borrow is a borrowing value: it may be used
  locally, passed as an argument, or stored inside a borrowing type such as
  `Map<I, F>`.
- It may be returned, or stored in a field, only where its origins are named:
  `-> impl Fn() -> usize from xs`, `-> Box<dyn Fn() from xs>`,
  `pred: Box<dyn Fn(&Row) -> bool + '_>`, or a field whose type is a type
  parameter.
- `move` changes the capture mode, not whether the closure borrows. A `move`
  closure that captures a `&str` still carries that string's origin. A
  closure is `Owned` exactly when every captured value is `Owned`.
- Where `Owned` is required (`Box<dyn Fn()>`, a task that outlives its
  creator, a global registration), the usual fix is `move` plus owning the
  data (`to_string()`, `clone()`, `Arc`).

### 5.4 Higher-ranked origins

- Elided origins in the argument types of `Fn`, `FnMut` and `FnOnce` sugar,
  and of `fn` pointer types, are higher-ranked as in Rust.
  `F: Fn(&T) -> bool` is `for<'x> Fn(&'x T) -> bool`. D4's "no rank-N types"
  restricts types, not origins.
- A closure literal takes its signature from its expected type. If a closure
  returns a borrow of its argument and has no expected type, that is an
  error. The fix annotates the closure's parameter and return types.
- `from P` in a closure bound's argument type keeps the origin higher-ranked
  with a lower bound: the closure must work for every `ρ` with
  `orig(P) ⊆ ρ`. Scoped tasks use this (§8.2).
- When a function item is coerced to a `fn` pointer type, the bounds implied
  by its signature are checked again at the target instantiation. This closes
  the class of rustc bug rust-lang/rust#25860, where implied bounds are lost
  through `fn` pointer variance.

## 6. Drop check

**Rule DC.** When a value of type `T` is dropped at point `P`, by scope end,
reassignment or temporary cleanup, every origin in `dropck(T)` must be live
at `P`. In words, data borrowed by a value with a destructor must strictly
outlive that value: its loans are still valid when the destructor runs.

```
dropck(T)            = orig(T)    T has a user `impl Drop`, is `dyn`, or is a type parameter
dropck(&ρ T)         = ∅          references have no destructor
dropck(S)            = the union of dropck over the field types   (no `impl Drop`; closures alike)
dropck(C<T1..Tn>)    = dropck(T1) ∪ … ∪ dropck(Tn)                 audited std containers
dropck(scalars, String, …) = ∅
```

**Strictness.** Locals drop in reverse declaration order. A value with a
destructor can therefore hold borrows only of places that outlive it, and two
values with destructors cannot point at each other. Rule DC rejects
`reviews/round1-soundness.md` B4's `Node` pair, as Rust does: `b` is dropped
first, and `a`'s destructor then uses a loan on `b`.

**The audited exemption.** Some std types have destructors that only drop
their elements and free memory: `Box`, `Vec`, `VecDeque`, `HashMap`,
`HashSet`, `BTreeMap`, `BTreeSet`, `Rc`, `Arc`, both `Weak`s, `Arena`,
`Shared` and the channel ends. These use the container rule above, which is
the equivalent of Rust's `#[may_dangle]`. Each exemption is std-internal,
counted in std's trust surface, and covered by tests under a Miri-style
checker. Users cannot write one.

A user type with `impl Drop` gets the conservative rule; the fix is to keep
the destructor on an inner owned type. Thanks to the exemption, a
`Vec<&str>` does not keep its elements' loans alive until it is dropped.

## 7. Field-borrow declarations

**Syntax.** A method receiver may name the fields it borrows:

```ferox
fn advance(&mut self.{pos})                  // mutable borrow of `pos`
fn peek(&self.{src, pos}) -> Option<char>    // shared borrow of `src` and `pos`
fn step(&mut self.{pos, &src})               // mutable `pos`, shared `src`
```

**In the body**, only the listed fields are places. `self` cannot be passed,
moved or borrowed as a whole. A method called on `self` must declare a subset
of these fields, with compatible modes.

**At a call site**, `x.advance()` borrows only `x.pos`, through autoref and
autoderef as usual: `(*x).pos` when `x: &mut T`.

**Elision.** For rule 2, the receiver counts as one borrow whose origin is the
union of its field borrows. `from self.f` keeps its meaning from §2.2.

**API stability.**

- The declared set is part of the signature and of the package interface.
- A method visible outside its package may name only fields that are also
  visible outside it, so private field names never enter a public API.
- For an exported method, removing a field from the set, or weakening
  `&mut` to `&`, is compatible. Adding a field or strengthening a mode is a
  breaking change.
- Trait methods cannot declare field borrows, because traits know no fields.
- If a type has an `invariant` that spans fields (D10), a field-borrow method
  may not mutate any field that the invariant mentions
  (`reviews/round1-soundness.md` B10(d)).

**Translation (exact).** The method becomes an associated function taking
one reference per field, and the call site passes `&mut x.pos, &x.src`, which
rustc accepts as disjoint borrows.

## 8. Tasks

### 8.1 `Send`, `Sync` and task isolation

`Send` means a value may move to another task. `Sync` means `&T` may be
shared with another task; `T: Sync` exactly when `&T: Send`. Both are derived
structurally, as in Rust:

| Type | `Send` | `Sync` |
| --- | --- | --- |
| `Rc<T>`, `rc::Weak<T>`, raw pointers, `ThreadBound` types | no | no |
| `Cell<T>`, `RefCell<T>`, `OnceCell<T>` | if `T: Send` | no |
| `&T` | if `T: Sync` | if `T: Sync` |
| `&mut T` | if `T: Send` | if `T: Sync` |
| `Arc<T>` | if `T: Send + Sync` | if `T: Send + Sync` |
| `Shared<T>` | yes | yes |
| `Sender<T>`; `Receiver<T>` | if `T: Send` | `Sender` if `T: Send`; `Receiver` no |
| capabilities, `Scope` | yes | yes |

`unsafe impl Send` is allowed in packages granted `unsafe`, and each one is a
trust hole.

**Task boundaries** require `Send` (by value) or `Sync` (by reference): a
spawned child's captures, so the closure is `Send`; channel messages;
`Shared` payloads; a child's result; and `par` branches.

**Task isolation (D8).** A task may migrate between OS threads at a
suspension point while it holds values that are not `Send`. The argument:

- Every way to share or transfer a value between tasks requires `Send` or
  `Sync`, so a value that is not `Send` is reachable from one task only.
- The scheduler's hand-off synchronizes (release on suspend, acquire on
  resume), so the task's own accesses stay ordered on any thread.

It depends on four conditions:

1. Safe code can reach no thread-local state (D7).
2. A task is pinned while a `ThreadBound` value is live across a suspension
   point, which the compiler knows from liveness.
3. Foreign code that relies on thread identity is wrapped in `ThreadBound`
   types.
4. `par` branches run simultaneously on several threads, so they require
   `Send` rather than isolation.

Rust has no counterpart, since tokio requires `Send` futures. This is the main
construct that §11 cannot translate.

### 8.2 Scopes

API names are illustrative. The origin structure is normative.

```ferox
impl Tasks {
    pub fn scope<R, F>(&self, body: F) -> Result<R, ScopeError>
        where F: FnOnce(Scope from self) -> R;
}
impl Scope {                                  // Copy; one invariant origin σ
    pub fn spawn<T, F>(self, f: F from self) -> Handle<T>
        where F: FnOnce() -> T + Send, T: Send;
}
```

- **`Scope`** is a `Copy` handle with one origin `σ`, the scope's origin.
  Inside `body`, `σ` is higher-ranked with `orig(self) ⊆ σ` (§5.4): `body`
  must work for every `σ` that carries the loans of the `&Tasks` borrow.
  Because the caller chooses that borrow, children can borrow anything that
  outlives the `scope` call.
- **`f: F from self`** imposes `F: σ` and `T: σ`. Everything a child borrows
  stays borrowed until the scope ends:
  - inside the body, `σ` is a placeholder, so it is live at every point;
  - scope exit joins every child and cannot be cancelled (D8);
  - no guard object exists, so `mem::forget` has nothing to skip.

  The parent body therefore cannot mutate, move or free a place that a child
  borrows.
- **`Handle<T>`** has origin `σ` (rule 1, from `self: Scope`). Because `σ` is
  higher-ranked, neither a `Scope` nor a `Handle` can escape through `R`.
- **Grandchildren.** A child may capture the `Scope` (`Scope: σ` holds).
  `σ` is invariant, so it cannot shrink to admit shorter borrows.
- **A scope passed down** (`fn serve(bg: Scope, conn: Conn)`) is a signature
  origin in `serve`. A child spawned there must outlive it. That means it
  owns its data, or it borrows what the signature ties to the scope, as in
  `cfg: &Config from bg`. This is the precise form of D2's "detached tasks
  require `Owned`": a function's own locals never outlive a scope it
  received, so from that function's point of view its children are detached.

Translation: the shape of `std::thread::scope`, with `'env` taken to be the
`&Tasks` borrow. `Scope` maps to `&'σ std::thread::Scope<'σ, 'env>`.

### 8.3 Child failure

D5's moved-out rule is defined as follows. Let `M` be the set of places with
a mutable loan in `orig(body)`, that is, every place lent mutably into the
scope, directly or through a captured `&mut`. On the edge where `scope`
returns `Err`:

- **Owned places in `M`** are uninitialized. Reading or borrowing one is an
  error until it is assigned. Dropping it is allowed, because the value is
  valid memory.
- **Places in `M` behind a reference parameter** cannot be read or borrowed
  for the rest of the function. They may be assigned, and the function may
  return.
- **Cells a failing child held locked** are poisoned at run time.

A panic in `body` itself abandons the parent task, after its children are
cancelled and joined.

Memory safety does not depend on this rule. Abandoning a task leaves every
value valid memory, as unwinding does in Rust. The rule protects invariants,
but only within the function: the second case lets a half-updated value reach
the caller of a function that returns after a child failure (§12).

### 8.4 `Shared<T>` and `lock`

- **`Shared<T>`** requires `T: Send + Pure` (D8). It is a counted handle,
  invariant in `T`, with `orig(Shared<T>) = orig(T)`. It is `Send` and
  `Sync`, and neither `Pure` nor `Frozen`.
- **`lock (e1, …, en) |x1, …, xn| body`**, where `ei: &Shared<Ti>`, is
  checked like a closure body in which `xi: &σ mut Ti` for a fresh
  higher-ranked `σ`. No value carrying `σ` escapes into the result or into
  captured places.
- **The body** captures only `Pure` values and contains no `lock`; with
  `Pure` payloads it reaches no capability and no `Shared` handle (D8). The
  result is `Result<R, Poisoned>`.
- **Distinctness.** The runtime checks that the handles are pairwise
  distinct before creating `x1` to `xn`; a duplicate panics, since two
  `&mut` to one payload would be undefined behavior.
- **Poisoning.** A panic in the body poisons the cells; their payloads are
  then reachable only through std's explicit recovery API.
- **Translation.** Mutex guards turned into `&'σ mut T` inside
  `for<'σ> FnOnce(&'σ mut T1, …) -> R`.

### 8.5 `par`

`par { a, b }` and `join(f, g)` take closures that are authority-free and
`Send`. Their shared captures must be `Frozen`; mutable captures must be
disjoint `&mut` borrows, which the borrow checker verifies as usual. The
branches may borrow the enclosing frame, because `par` joins before it
continues. In Rust this is the shape of `rayon::join`. `Frozen` is stricter
than `Sync`; that serves determinism, not memory safety.

## 9. `Pure` and `Frozen`

Both are compiler-known auto traits. Outside std, they can be implemented
only by `unsafe impl` in a package granted `unsafe`, and each such impl is a
trust hole. Both are **deep**:

- they look through every field, private ones included, and through `Box`,
  `Rc` and `Arc`, with type arguments substituted;
- recursive types take the greatest fixed point;
- opaque and `dyn` types have exactly the traits their bounds state;
- a type parameter has a trait only if it is bounded by it.

### `Pure`: no authority reachable

These are **not `Pure`**:

- the capability types `Root`, `Fs`, `Net`, `Clock`, `Rand`, `Env`, `Stdin`,
  `Proc`, `Tasks` and `Accel`, and their attenuated forms;
- the synchronization handles `Shared<T>`, `Sender<T>`, `Receiver<T>`,
  `Scope`, `Handle<T>`, `sync::Weak<T>` and every atomic type;
- the foreign handles: raw pointers, `extern` types and foreign function
  pointers;
- `ThreadBound` types;
- `dyn Trait` unless `+ Pure` is written;
- an unbounded type parameter.

**Everything else is `Pure` exactly when its components are**, including
references, std collections, `Rc`, `rc::Weak`, `Arc`, `Cell`, `RefCell`,
`Arena`, structs, enums and closures. Ferox function pointers are `Pure`:
any authority the pointee uses arrives through its arguments.

**What `Pure` guarantees.** A function whose parameter and capture types are
all `Pure` is authority-free: no I/O, time, randomness, synchronization with
other tasks, or suspension. Its result is a function of the state of memory
reachable from its arguments. `Cell`, `RefCell` and `Rc` are `Pure` because
that state is private to the task.

**What std must provide in return.** No operation on a `Pure` type may return
a result that depends on other tasks. Ferox's `Arc` therefore omits
`strong_count`, `weak_count`, `get_mut`, `try_unwrap` and `into_inner`.
`Arc::make_mut` stays, because its observable result does not depend on
timing. `sync::Weak` is a synchronization handle, because `upgrade` observes
other tasks' drops.

### `Frozen`: no interior mutability reachable through shared references

**Not `Frozen`**: `Cell`, `RefCell`, `OnceCell`, `UnsafeCell`, atomics,
`Rc` and `rc::Weak` (whose counts change through shared references),
`Shared`, channel ends, `Scope`, `Handle`, capabilities, raw pointers,
`dyn Trait` without `+ Frozen`, and unbounded type parameters.

**Everything else is `Frozen` exactly when its components are.** That covers
scalars, `str`, `&T`, `&mut T` (through `& &mut T` only reads are possible),
`Box`, arrays, collections, `Option`, `Result`, structs, enums, closures and
`Arc<T>`. `Arc` qualifies because its count is atomic and, under the rule
above, invisible to safe code.

Unlike Rust's shallow `Freeze`, `Frozen` looks behind pointers, so an `Rc`
inside a `Box` disqualifies a type (`reviews/round1-soundness.md` B6(e)).

### Where each trait is used

| Use | Requires |
| --- | --- |
| `par` | authority-free and `Send` closures, `Frozen` shared captures |
| memoization and `const fn` | `Pure` and `Frozen` parameters, no `&mut` parameters (D7) |
| statics | `Owned`, `Sync` and `Frozen` |
| `lock` payloads | `Send` and `Pure` |

## 10. Worked examples

Each example shows Ferox, the Rust it corresponds to, and the verdict.

### 10.1 Iterators over a `Vec`

```ferox
fn total_len(words: &Vec<String>) -> usize {
    words.iter().map(|w| w.len()).sum()
}

fn names_over(people: &[Person], min: u32) -> Vec<&str> {
    people.iter().filter(|p| p.score > min).map(|p| p.name.as_str()).collect()
}

fn adults(people: &[Person], min_age: u32) -> impl Iterator<Item = &Person> {
    people.iter().filter(move |p| p.age >= min_age)
}

// std
pub struct Iter<T> { rest: &[T] }
impl<T> Iterator for Iter<T> {
    type Item = &T from Self.rest;
    fn next(&mut self) -> Option<Self::Item> { … }
}
```

```rust
fn names_over(people: &[Person], min: u32) -> Vec<&str> { /* same */ }
fn adults(people: &[Person], min_age: u32) -> impl Iterator<Item = &Person> + '_ { /* same */ }
impl<'a, T> Iterator for Iter<'a, T> { type Item = &'a T; /* … */ }
```

**Accepted.** By rule 1, every elided `&` in a result gets `people`'s origin,
and `adults` captures `people` through `Item`. Without `move`, its closure
would borrow `min_age` and be rejected, as in Rust.

### 10.2 `HashMap` get and entry

```ferox
fn count_words(text: &str) -> HashMap<&str, usize> {
    let mut counts = HashMap::new();
    for w in text.split_whitespace() {
        *counts.entry(w).or_insert(0) += 1;
    }
    counts
}

fn add_word(counts: &mut HashMap<&str from w, usize>, w: &str) {
    *counts.entry(w).or_insert(0) += 1;
}

impl Index {
    fn get(&self, key: &str) -> Option<&Entry> { self.map.get(key) }
}
fn find(cache: &HashMap<String, Entry>, key: &str) -> Option<&Entry> from cache {
    cache.get(key)
}
```

```rust
fn add_word<'w>(counts: &mut HashMap<&'w str, usize>, w: &'w str) { /* same */ }
fn find<'c>(cache: &'c HashMap<String, Entry>, key: &str) -> Option<&'c Entry> { /* same */ }
```

**Accepted.** `entry(&mut self, k: K) -> Entry<K, V>` and
`Entry::or_insert(self, v: V) -> &mut V` elide as in Rust. `Entry` has one
origin of its own; the origins of `K` and `V` are not counted.

**Rejected in v1**, as in stable Rust, is the conditional return of a lookup
(NLL problem case #3):

```ferox
fn get_default(map: &mut HashMap<String, String>, k: &str) -> &mut String from map {
    if let Some(v) = map.get_mut(k) { return v; }   // error: `*map` still mutably borrowed
    map.insert(k.to_string(), String::new());
    map.get_mut(k).unwrap()
}
```

The idiom is `map.entry(k.to_string()).or_default()`.

### 10.3 A lexer holding `&str`

```ferox
pub struct Lexer { src: &str, pos: usize }

impl Lexer {
    pub fn new(src: &str) -> Lexer { Lexer { src, pos: 0 } }
    pub fn next_word(&mut self) -> Option<&str from self.src> { … }
}

impl Iterator for Lexer {
    type Item = &str from Self.src;
    fn next(&mut self) -> Option<Self::Item> { self.next_word() }
}

fn first_two(text: &str) -> (Option<&str>, Option<&str>) {
    let mut lx = Lexer::new(text);
    let a = lx.next_word();
    let b = lx.next_word();      // fine: `a` borrows `text`, not `lx`
    (a, b)
}
```

```rust
struct Lexer<'a> { src: &'a str, pos: usize }
impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self { /* … */ }
    fn next_word(&mut self) -> Option<&'a str> { /* … */ }
}
impl<'a> Iterator for Lexer<'a> { type Item = &'a str; /* … */ }
```

**Accepted.** Without `from self.src`, rule 2 ties the token to the
`&mut self` borrow, and the second call is rejected (Rust behaves the same).
The diagnostic suggests `from self.src`.

### 10.4 A parser returning slices of its input

```ferox
pub struct Parser { lx: Lexer, peeked: Option<&str from lx.src> }   // one origin: lx.src

impl Parser {
    pub fn new(text: &str) -> Parser { Parser { lx: Lexer::new(text), peeked: None } }

    fn peek(&mut self) -> Option<&str from self.lx.src> {
        if self.peeked.is_none() { self.peeked = self.lx.next_word(); }
        self.peeked
    }

    pub fn pair(&mut self) -> Result<(&str, &str) from self.lx.src, ParseError> { … }
}

pub fn split_header(line: &str) -> Option<(&str, &str)> {
    let (k, v) = line.split_once(':')?;
    Some((k.trim(), v.trim()))
}
```

```rust
struct Parser<'a> { lx: Lexer<'a>, peeked: Option<&'a str> }
impl<'a> Parser<'a> {
    fn pair(&mut self) -> Result<(&'a str, &'a str), ParseError> { /* … */ }
}
```

**Accepted.** The tokens outlive every `&mut self` call and live as long as
the text.

### 10.5 Builders

```ferox
#[derive(Default)]
pub struct Request { url: String, headers: Vec<(String, String)>, timeout: Option<Duration> }

impl Request {
    pub fn header(mut self, k: &str, v: &str) -> Request {         // by value
        self.headers.push((k.to_string(), v.to_string()));
        self
    }
    pub fn timeout(&mut self, d: Duration) -> &mut Request {        // by reference
        self.timeout = Some(d);
        self
    }
}

pub struct ArgList { args: Vec<&str> }
impl ArgList {
    pub fn arg(&mut self, a: &str from self.args) -> &mut Self {
        self.args.push(a);
        self
    }
}
```

```rust
impl<'a> ArgList<'a> {
    fn arg(&mut self, a: &'a str) -> &mut Self { self.args.push(a); self }
}
```

**Accepted.** `ArgList::arg` needs `from self.args`. Without it, `a` has its
own origin and the push is rejected ("`a` must outlive the elements of
`self.args`"). Write `-> &mut Self`. Writing `-> &mut ArgList` would give the
inner origin to the `&mut self` borrow by rule 2, as `ArgList<'_>` does in
Rust, and the body would be rejected.

### 10.6 `longest`

```ferox
fn longest(a: &str, b: &str) -> &str from a, b {
    if a.len() >= b.len() { a } else { b }
}

fn longest_word(words: &[&str]) -> &str from *words {
    words.iter().copied().max_by_key(|w| w.len()).unwrap_or("")
}

fn title(text: &str) -> &str {
    let words: Vec<&str> = text.split(' ').collect();
    longest_word(&words)              // fine: the result borrows `text`, not `words`
}
```

```rust
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str { /* … */ }
fn longest_word<'w>(words: &[&'w str]) -> &'w str { /* … */ }
```

**Accepted.** This is `reviews/round1-soundness.md` B2.1. Draft 1 rejected it.

### 10.7 A struct holding references from two sources

```ferox
pub struct Match { needle: &str, line: &str }       // two origins: needle, line

pub fn find(needle: &str, text: &str) -> Option<Match from { needle: needle, line: text }> {
    let line = text.lines().find(|l| l.contains(needle))?;
    Some(Match { needle, line })
}

fn first_error(text: &str) -> Option<&str> {
    let pat = String::from("error");
    let m = find(&pat, text)?;
    Some(m.line)                      // fine: `line` borrows only `text`
}
```

```rust
struct Match<'n, 'l> { needle: &'n str, line: &'l str }
fn find<'n, 'l>(needle: &'n str, text: &'l str) -> Option<Match<'n, 'l>> { /* … */ }
```

**Accepted.** With the union `-> Option<Match> from needle, text`, both
origins would carry both loans. `Some(m.line)` would then be rejected,
because `pat` dies at the end of `first_error`.

### 10.8 A struct whose fields share an origin

```ferox
struct Collector { src: &str, out: &mut Vec<&str from src> }

impl Collector {
    fn emit(&mut self, start: usize, end: usize) {
        self.out.push(&self.src[start..end]);
    }
}

fn tokenize(text: &str) -> Vec<&str> {
    let mut toks = Vec::new();
    let mut c = Collector { src: text, out: &mut toks };
    c.emit(0, 1);
    toks                              // fine: the borrow `out` ended at `c`'s last use
}
```

```rust
struct Collector<'s, 'o> { src: &'s str, out: &'o mut Vec<&'s str> }
```

**Accepted.** Without `from src`, `out`'s elements have their own origin,
`*out`, and `emit` is rejected ("`self.src` must outlive the elements of
`self.out`; add `from src`"). Draft 1's single region made this example
either unsound or unusable (B1).

### 10.9 A graph through `Arena` handles

```ferox
struct Node { name: String, edges: Vec<Id<Node>> }

fn build() -> (Arena<Node>, Id<Node>) {
    let mut g = Arena::new();
    let a = g.insert(Node { name: "a".to_string(), edges: Vec::new() });
    let b = g.insert(Node { name: "b".to_string(), edges: vec![a] });
    g[a].edges.push(b);                              // a cycle: a -> b -> a
    (g, a)
}

fn reachable(g: &Arena<Node>, start: Id<Node>) -> Vec<&str> {
    let (mut seen, mut stack, mut out) = (HashSet::new(), vec![start], Vec::new());
    while let Some(id) = stack.pop() {
        if !seen.insert(id) { continue; }
        out.push(g[id].name.as_str());
        stack.extend(g[id].edges.iter().copied());
    }
    out
}
```

Rust: the same code with `slotmap::SlotMap` or `generational_arena`.

**Accepted.** `Id<T>` (a `u32` index and a `u32` generation) is `Copy`,
`Owned`, `Pure`, `Frozen`, `Send` and `Sync`, so it can be stored anywhere. A
stale handle panics on indexing; `get` returns `None`. Two nodes are mutated
together with `g.get_disjoint_mut([a, b])`, checked at run time.

### 10.10 `Rc<RefCell<T>>`

```ferox
struct TreeNode { val: i32, children: Vec<Rc<RefCell<TreeNode>>>, parent: rc::Weak<RefCell<TreeNode>> }

let leaf = Rc::new(RefCell::new(TreeNode { val: 3, children: vec![], parent: rc::Weak::new() }));
let root = Rc::new(RefCell::new(TreeNode { val: 5, children: vec![leaf.clone()], parent: rc::Weak::new() }));
leaf.borrow_mut().parent = Rc::downgrade(&root);
root.borrow_mut().val += leaf.borrow().val;
```

Rust: identical.

**Accepted.** The type is `Owned` and `Pure`, but not `Frozen`, `Send` or
`Sync`. It may be held across suspensions within one task (§8.1) but cannot
enter another task or `par`; use `Arc` with `Shared` there. A failed
`borrow_mut` panics.

### 10.11 A trait object holding a reference

```ferox
fn words(text: &str) -> Box<dyn Iterator<Item = &str> from text> {
    Box::new(text.split_whitespace())
}

struct Filter { pred: Box<dyn Fn(&Row) -> bool + '_> }        // one origin: pred

fn count_leaves(tree: &Tree) -> usize {
    let mut n = 0;
    tree.visit(&mut |node| if node.is_leaf() { n += 1 });     // &mut dyn FnMut(&Node)
    n
}

struct Job { next: Box<dyn Iterator<Item = u8>> }             // Owned object
fn job(text: &String) -> Job {
    Job { next: Box::new(text.bytes()) }                       // error
}
```

```rust
fn words<'a>(text: &'a str) -> Box<dyn Iterator<Item = &'a str> + 'a> { /* … */ }
struct Filter<'p> { pred: Box<dyn Fn(&Row) -> bool + 'p> }
```

**`words`, `Filter` and `count_leaves` are accepted.** `count_leaves` relies
on §5.1's default bound behind `&mut`.

**`job` is rejected, as in Rust.** `Bytes` borrows `text`, but this
`dyn Iterator` is `Owned`. There are two fixes:

- `struct Job { next: Box<dyn Iterator<Item = u8> + '_> }`, after which
  `fn job(text: &String) -> Job` elides by rule 1;
- owning the data: `text.clone().into_bytes().into_iter()`.

### 10.12 Scoped tasks borrowing a slice

```ferox
fn parallel_sum(tasks: &Tasks, xs: &[u64]) -> Result<u64, ScopeError> {
    let mid = xs.len() / 2;
    let mut sums = [0u64; 2];
    let [lo, hi] = &mut sums;
    tasks.scope(|s| {
        s.spawn(|| *lo = xs[..mid].iter().sum());
        s.spawn(|| *hi = xs[mid..].iter().sum());
    })?;
    Ok(sums[0] + sums[1])             // fine: every child has finished
}

fn after_failure(tasks: &Tasks, xs: &[u64]) -> u64 {
    let mut sums = [0u64; 2];
    let [lo, _] = &mut sums;
    match tasks.scope(|s| { s.spawn(|| *lo = xs.iter().sum()); }) {
        Ok(()) => sums[0],
        Err(_) => sums[0],            // error: `sums` was lent mutably to a failed scope
    }
}

fn serve(bg: Scope, conn: Conn) {
    let buf = [0u8; 4096];
    bg.spawn(|| conn.read_into(&buf));                                     // error
    bg.spawn(move || { let mut buf = [0u8; 4096]; conn.read_into(&mut buf) });  // fine
}
```

```rust
fn parallel_sum(xs: &[u64]) -> u64 {
    let (mut lo, mut hi) = (0, 0);
    std::thread::scope(|s| {
        s.spawn(|| lo = xs[..xs.len() / 2].iter().sum());
        s.spawn(|| hi = xs[xs.len() / 2..].iter().sum());
    });
    lo + hi
}
```

**`parallel_sum` is accepted**: `xs` and the slots outlive the `&Tasks`
borrow, whose loans `σ` carries.

**`after_failure` is rejected** by §8.3.

**In `serve`, the first spawn is rejected**: `buf` does not outlive `bg`'s
scope (`reviews/round1-soundness.md` B2.4). The second spawn is accepted,
because its closure owns everything it uses.

Rust's scope re-raises a child's panic; Ferox returns `Err` and applies
§8.3. For pure work the Ferox idiom is
`let (a, b) = par { sum(&xs[..mid]), sum(&xs[mid..]) };`.

## 11. Soundness sketch: translation to Rust

**Claim.** For every program `P` that the Ferox checker accepts, the
translation `τ(P)`, Rust with explicit lifetimes, is accepted by rustc's NLL
borrow checker, except at the constructs listed as non-exact below.

Rust's soundness covers the rest: RustBelt for the core type system, plus
std's audited unsafe code. That gives memory safety and data-race freedom
for `P`. `τ` is a proof device and the CI oracle of D12's origin cross-check;
it is not a backend. The cross-check runs on the test corpus and on every
change to std. A program that Ferox accepts and rustc rejects blocks the
release.

### 11.1 The translation

| Ferox | Rust | Exact? |
| --- | --- | --- |
| A borrowing struct or enum | one lifetime parameter per origin, in name order; `from` in a field reuses the named lifetime; recursive occurrences reuse the declaration's lifetimes; rustc derives the same variances from the same fields | yes |
| Elided origins in signatures | the same elided positions; output elision under identical rules | yes |
| `from P` in a parameter type | reuse `P`'s lifetime | yes |
| `-> T from P̄` | a fresh `'r` per affected position, with `'p: 'r` for each `p` in `orig(P̄)` | yes |
| `f: F from P` | `F: 'p, F::Output: 'p` | yes |
| `Self.f`, `self.f` in impls | `impl<'f, …> Ty<'f, …>`, with `'f` | yes |
| `dyn`, the default `Owned` | `+ 'static`; behind `&'a`, `+ 'a` | yes |
| Return-position `impl Trait` | `impl Trait + use<'p…, T…>`, with auto traits stated | yes |
| `Owned` | `'static` | yes |
| Closures, `Fn` sugar, higher-ranked origins | the same | yes |
| `from` in a closure-bound argument | the implied-bound encoding that `std::thread::scope` uses (a witness type mentioning `&'s &'p ()`) | encoded |
| Field-borrow methods | associated functions with split borrows | yes |
| Two-phase borrows, NLL, drop check | the same | yes |
| The std eyepatch | `#[may_dangle]` (unsafe, audited) | audited |
| `Tasks::scope`, `Scope::spawn` | `std::thread::scope` with `'env` = the `&Tasks` borrow | yes |
| The `Err` edge of a scope | `catch_unwind(AssertUnwindSafe(…))`; the moved-out rule only rejects more programs | yes |
| `lock` | `for<'σ> FnOnce(&'σ mut T1, …)` over Mutex guards, plus a run-time distinctness check in std | audited |
| `par` | `rayon::join` or `thread::scope`; `Frozen` is stricter than `Sync` | yes |
| A non-`Send` value inside a task that migrates | none: Rust requires `Send` futures | **no** |

### 11.2 Where the translation is not exact

- **R1. Task isolation (§8.1): medium risk.** Rust cannot express it.
  Soundness rests on the argument in §8.1, which needs a written proof in the
  style of Swift's SE-0414 and an audit of std, the runtime and FFI wrappers
  for thread-affine state.
- **R2. Bounded higher-ranked origins (§5.4): low risk.** This is the
  encoding that `std::thread::scope` relies on.
- **R3. Implied bounds through `fn` pointers: low risk once implemented.**
  Reducing to Rust inherits Rust's holes, such as rust-lang/rust#25860, which
  the oracle cannot catch because rustc accepts them. §5.4's re-check closes
  it; known exploits belong in the negative test corpus.
- **R4. The audited std set** (`Arena`, `Shared` and `lock`, channels,
  `Scope`, the eyepatch containers) has Rust std's trust model and is counted
  in the trust surface.
- **R5. `Pure` and `Frozen`: medium risk.** They promise determinism, which
  lies outside Rust's model and depends on classifying std correctly
  (`Arc`'s counts were nearly missed).
- **R6. The child-failure rule (§8.3)** is sound for memory but protects
  invariants only partially.
- **R7. The evented lowering of scoped children** needs non-moving frames
  and uncancellable joins in the C backend, outside `τ`.

## 12. Known limitations and open questions

1. **NLL precision.** NLL problem case #3 and lending iterators are rejected,
   as in stable Rust. The idioms are the entry API and index loops.
   Polonius-level precision is a compatible later relaxation.
2. **Self-referential structs** need `unsafe` (a cost D2 accepts). There is
   no `Pin`, since there is no `async`.
3. **`&'arena T` graphs** in the style of rustc's `TyCtxt` need an
   append-only arena (`alloc(&self, v: T) -> &mut T`) with an eyepatch. v1
   std ships only handle-based `Arena<T>`.
4. **Exported field groups.** Named borrow sets that a type exports without
   exposing its fields, Rust's "abstract fields", are open.
5. **Implicit origin propagation.** A new reference field adds origins to
   every containing type, possibly far from the edit. The interface diff
   reports such changes in public types as breaking. E1 data should decide
   whether a visible marker such as `view struct` is worth its cost.
6. **Lifetimes Ferox cannot express.** These are explicit outlives bounds
   between unrelated origins, lifetime parameters on traits, lending traits
   that need GATs, and `for<'a>` beyond elision. Interop with lifetime-heavy
   Rust APIs is out of scope (Positioning).
7. **`Arc` count observers.** Something must replace `Arc::try_unwrap` and
   `get_mut` in the "unwrap after the workers finish" pattern, probably
   scope results.
8. **Invariants after a child failure.** A half-updated referent behind a
   `&mut` parameter can reach the caller (§8.3). Either D10 stops assuming
   type invariants of `&mut` arguments after such a call, or such failures
   must be propagated; this is open.
9. **Agent-facing naming.** E1 should measure whether agents find the right
   `from` from diagnostics that name origins such as `*out` and `lx.src`.

## 13. Changes this chapter asks of `01-design.md`

1. **D2, `dyn`.** `&dyn Trait` and `&mut dyn Trait` should default to the
   reference's origin, which is Rust's default object bound; `dyn` elsewhere
   stays `Owned` (§5.1). Without this, `&mut dyn FnMut` callbacks that
   capture locals are rejected.
2. **D2, origins per field.** "One origin per reference-typed field" should
   read "one origin per origin position in each field's type, named by its
   path (`out`, `*out`)". The literal reading brings back the
   `&'a mut T<'a>` trap for `out: &mut Vec<&str>`.
3. **D2, `from`.** Add the per-origin binding
   `T from { name: place }` (§10.7) and `from` in closure-bound argument
   types, which `Tasks::scope` needs (§8.2).
4. **D3, `impl Trait`.** Opaque return types should not leak auto traits
   (§5.2), as D12's "a body edit never re-checks dependents" requires.
5. **D7, std types.** Classify `Rc`, `Arc` and `sync::Weak` as §9 does, and
   remove `Arc`'s count observers from std.
6. **D5, "moved out".** Define it for places behind references, and note the
   invariant gap (§8.3).
7. **D4, rank-N.** State that "no rank-N types" does not exclude
   higher-ranked origins.
