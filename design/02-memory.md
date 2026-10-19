# Tide Memory: Ownership and Origins

Status: draft 1, synchronized with draft 3.5. This chapter elaborates D2 of
`design/01-design.md` and the memory parts of D5, D7, D8 and D9, and it is the
definition the checker implements. Section 13 records the edits it asked of
`01-design.md`.

**Tier** (01's Tiers table). v0 core: §1–§6, §8 except task migration, §9,
§10 and §11. v1: §7 (field-borrow declarations) and task migration under the
evented runtime with `ThreadBound` pinning (§8.1, R1). Later:
Polonius-precision borrows (§2.8, §12). Draft 3 removed `Frozen` (§9 now
requires `Pure + Sync`), the `lock` keyword (§8.4, now `Shared::with`),
`par { }` (§8.5, now `join`), and the moved-out rule (§8.3).

Notation. Tide code is in `tide` blocks and Rust in `rust` blocks. In rules,
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
- `mem::forget` and `Box::leak` are safe: scopes, `Shared::with` and `join`
  take closures, not guards, so no guarantee depends on a destructor running.

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
  when the signature implies it, through a declared outlives bound or the
  well-formedness of its types (§2.4).

### 2.2 Origins of a type and of a place

`orig(T)` is the set of origins that a value of type `T` holds at top level:

```
orig(&ρ T) = orig(&ρ mut T) = {ρ}
orig(S<ρ1..ρn; T1..Tm)      = {ρ1..ρn} ∪ orig(T1) ∪ … ∪ orig(Tm)   (struct, enum)
orig((T1, …, Tn))           = orig(T1) ∪ … ∪ orig(Tn); arrays and slices likewise
orig(dyn Trait + ρ) = {ρ}        orig(dyn Trait + Owned) = ∅
orig(closure)               = the union of orig over its captured types
orig(returned impl Trait)   = ∅ if its bounds state Owned (+ 'static); {ρ} if + ρ;
                              otherwise its captured lifetimes ∪ orig of its
                              captured type parameters (§5.2)
orig(T), T a type parameter = an abstract set
orig(scalars, str, String, fn pointers, 'static references to Owned data) = ∅
```

A reference lists only its outer origin. Well-formedness of `&ρ T` requires
`σ ⊆ ρ` for every `σ` in `orig(T)`, so `ρ` already covers everything the
pointee borrows.

**The origins of a place** are `orig(p) = orig(type of p)`: the loans held by
the value stored at `p`. Diagnostics use them to say what a borrow came from.

### 2.3 Lifetime syntax

Tide's lifetime syntax is Rust 2024's, without the forms beyond D4's power
ceiling:

```
Lifetime     ::= "'" Ident | "'_" | "'static"
Generics     ::= '<' LtParam (',' LtParam)* [',' TypeParam (',' TypeParam)*] '>'
               | '<' TypeParam (',' TypeParam)* '>'
                                   lifetime parameters on fn, struct, enum, impl;
                                   never on a trait
LtParam      ::= Lifetime [':' Lifetime ('+' Lifetime)*]
RefType      ::= '&' [Lifetime] ['mut'] Type
PathType     ::= Path ['<' Lifetime (',' Lifetime)* [',' Type (',' Type)*] '>']
Bound        ::= TraitBound | Lifetime            in `T: ..`, `impl ..`, `dyn ..`
WherePred    ::= Type ':' Bound ('+' Bound)* | Lifetime ':' Lifetime ('+' Lifetime)*
Capture      ::= 'use' '<' .. '>'                 only on a return-position `impl Trait`;
                                   lists every type parameter in scope
```

- A lifetime parameter is declared on the item that uses it: `fn f<'a, T>`,
  `struct S<'a>`, `enum E<'a>`, `impl<'a> S<'a>`,
  `impl<'a> Iterator for S<'a>`. Each names one origin.
- `'_` is an elided lifetime (§2.5). In a signature, a borrowing type
  written without its lifetime arguments (`Lexer` for `Lexer<'_>`) elides
  them, as in Rust; in an impl header the bare name is an error (§2.5).
- Outlives bounds are Rust's: `'a: 'b` declares `ρa ⊆ ρb`; `T: 'a`,
  `T: Trait + 'a`, `impl Trait + 'a`, and `dyn Trait + 'a` are type-outlives
  bounds (§2.4 item 3). `T: 'static` is accepted and written `T: Owned` by
  the formatter (§3.7). `+ use<..>` lists what a returned `impl Trait`
  captures (§5.2).
- **Not accepted** (`FX0201`, `FX0403`, `FX0404`, with a suggested
  rewrite), as beyond D4's power ceiling: lifetime parameters on traits
  (`trait Parse<'a>`), explicit `for<'a>` (higher-ranked lifetimes come from
  elision in `Fn` sugar and `fn` pointer types, §5.4), and generic
  associated types (`type Item<'a>`).

### 2.4 What a named lifetime means in each position

A signature's lifetime parameters are universally quantified origins
(§2.1): the caller chooses them. Every use of one name denotes the same
origin, and variance (§4) decides what that asks of the caller, as in Rust.

1. **Parameter types.** At a covariant position, the argument must carry no
   loan outside the origin; at an invariant position, its origin must be
   exactly the origin.
2. **Output positions** are return types and annotations in a body. A result
   that may borrow from several inputs names one lifetime on all of them:
   `fn longest<'a>(a: &'a str, b: &'a str) -> &'a str`. At the call, both
   arguments coerce to a common origin that carries the loans of each. When
   one input sits at an invariant position, declared bounds give the result
   a lifetime of its own:
   `fn pick<'r, 'a: 'r, 'b: 'r>(v: &mut Vec<&'a str>, s: &'b str) -> &'r str`.
3. **Outlives bounds.** A type-outlives bound (`F: 'a`, `impl Fn() + 'a`,
   `dyn Trait + 'a`) requires `σ ⊆ ρa` for every `σ` in `orig(F)`, and a
   lifetime-outlives bound `'a: 'b` declares `ρa ⊆ ρb`, as in Rust.
4. **Field types.** Each use of a struct's lifetime parameter in a field type
   is that origin; fields that use one name share it (§3.3).
5. **Implied relations.** The well-formedness of parameter and return types
   implies relations between their lifetimes, as in Rust (RFC 2093):
   `&'m BTreeMap<K, Vec<&'t str>>` implies `ρt ⊆ ρm`, so a result
   `&'m [&'t str]` needs no bound. A struct's declared and implied bounds
   hold wherever the struct type is well formed.

### 2.5 Elision

Elision follows Rust's rules exactly.

- **Inputs.** Each elided lifetime in the parameter types is a distinct
  origin. Elided positions are `&T` and `&mut T` without a lifetime, `'_`,
  each lifetime of a borrowing type written without lifetime arguments
  (`Lexer` as `Lexer<'_>`), and `dyn Trait + '_` or `impl Trait + '_`.
  Origins hidden inside type parameters are never counted.
- **Rule 1.** If the parameters use exactly one origin, elided or named,
  every elided output position gets it.
- **Rule 2.** If the receiver is `&self` or `&mut self`, every elided output
  position gets the receiver reference's origin.
- **Otherwise** an elided output lifetime is an error (`FX0202`). The fix
  lists the candidate named lifetimes and `'static`.
- **Closure-bound sugar.** Inside `Fn(&A) -> &B`, the same rules apply, and
  the argument origins are higher-ranked.
- **Consts and statics.** Elided lifetimes are `'static`.
- **Fields** have no elision: a reference field uses a lifetime parameter of
  its struct (§3.1) or `'static`.
- **Impl headers.** Each `'_` is a fresh lifetime parameter of the impl
  (`impl Collector<'_, '_>`). The bare name of a borrowing type is an error,
  as in Rust (E0726), whose fix adds `<'_>`.
- **Associated types in impls** have no elision, as in Rust. Write the impl's
  lifetime: `type Item = &'a str`.
- **Default `dyn` bounds** are in §5.1.

### 2.6 Relation to Rust lifetimes

Every signature Tide accepts is a Rust 2024 signature with the same
meaning, so §11's translation is the identity on lifetimes, except the rows
§11.1 marks. The forms Rust has and Tide rejects (§2.3) are D4's
power-ceiling forms: lifetimes on traits, explicit higher-ranked binders,
and lifetime-generic associated types.

Draft 3 spelled origins with `from` clauses that named places
(`fn next(&mut self) -> Option<&str from self.src>`, `struct Lexer { src:
&str }`). The held-out trial in `trials/e1-origins/` found no correctness
difference between the two spellings, and D1 keeps Rust's (D2). Draft 3.4
first kept a narrower subset; round 8 found programs it left without a
spelling (`reviews/round8-consistency.md`), so D2 takes Rust 2024's rules
whole.

### 2.7 Modular checking

- **Interfaces.** A module's interface artifact (D12) records each
  signature's lifetimes and their uses, outlives bounds, implied relations
  and field-borrow sets (§7). For each public type it records the named
  origins with their variances, its declared and implied outlives relations, the auto traits (`Owned`, `Send`, `Sync`,
  `Pure`, and in v1 `ThreadBound`) and a drop-check summary. A change to any of these is an
  API change and appears in the trust diff.
- **Calls.** The callee's origins are instantiated freshly; argument
  coercion adds subset constraints by variance (§4); the signature's
  relations become constraints on the fresh origins. The callee's body is
  never consulted.
- **Bodies.** Signature origins are placeholders, and they are live at every
  point of the body. Inclusions between placeholders come from the
  signature's declared outlives bounds and from the well-formedness of its
  parameter and return types (§2.1, §2.4). Any other inclusion the body
  needs is an error; its fix suggests the bound (`'a: 'b`) or a shared
  lifetime, as rustc does.
- **A local loan reaching a placeholder origin** (a returned reference to a
  local) conflicts with the local's storage death: "returns a value borrowed
  from local `x`".
- **Trait impls.** An impl's method signature must be at least as general as
  the trait's, checked by subtyping on the instantiated signatures, as rustc
  does.

### 2.8 Precision

v0 accepts a body exactly when Rust's NLL borrow checker accepts its
translation (§11). The field-borrow declarations of §7 (v1) translate exactly.

In NLL terms, subset constraints hold at every point of the body, and a
loan is live where its origin is live and reachable from the borrow without a
kill. Polonius makes subset constraints location-sensitive, which only
accepts more programs. D12's origin cross-check holds the checker to this
definition.

## 3. Borrowing types

### 3.1 Lifetime parameters

A struct or enum declares its lifetime parameters as in Rust, and each names
one origin of the type. Every reference, borrowing type, and `dyn Trait`
object bound in a field type uses a declared lifetime or `'static`.

- Origins carried by the type's own type parameters are not lifetime
  parameters. They ride with the type arguments (§3.4).
- A recursive occurrence reuses the declaration's lifetimes, as in Rust:
  `struct Node<'a> { val: i32, next: Option<&'a Node<'a>> }`.

A type is a **borrowing type** if it has a lifetime parameter or a type
argument with origins. Otherwise it is `Owned`.

### 3.2 Names

An origin is named by its lifetime parameter: `'a` in `struct Lexer<'a>`.
Diagnostics add the fields that carry it ("`'a`, carried by `src`").

### 3.3 Sharing an origin

Fields that use one lifetime name share its origin; fields from different
sources take different names:

```tide
struct Collector<'s, 'o> { src: &'s str, out: &'o mut Vec<&'s str> }   // origins: 's, 'o
```

Writing one name for a `&mut` borrow and for the data behind it
(`out: &'a mut Vec<&'a str>`) is accepted and means what it means in Rust:
the mutable borrow then lasts as long as the data, which usually fails at
the caller. Elision never produces it (§4.3).

### 3.4 Origins from type arguments

`orig(Vec<&str>) = orig(&str)`: a container borrows what its elements borrow.
`struct Tagged<'t, T> { value: T, tag: &'t str }` has its own origin, `'t`,
plus `orig(T)`. A field `r: &'r T` implies `σ ⊆ ρr` for every `σ` in
`orig(T)` (Rust infers `T: 'r`, RFC 2093).

### 3.5 Impls and `Self`

`impl<'a> Lexer<'a> { … }` is generic over `Lexer`'s origin, and `Self`
denotes `Lexer<'a>`.

- `-> Self` carries `'a`. `-> Lexer` (or `Lexer<'_>`) is an elided type, and
  the elision rules resolve its origin.
- A method returns a borrow of what the value holds, not of the receiver,
  with the impl's lifetime: `fn next_word(&mut self) -> Option<&'a str>`.
- Associated types name the impl's lifetimes: `type Item = &'a str`.

### 3.6 Constructing and returning borrowing types

- In a body, origins are inferred: in `Lexer { src: text, pos: 0 }`,
  `text`'s origin flows into `src`.
- `fn lex(text: &str) -> Lexer<'_>`: by rule 1, `Lexer`'s origin is
  `text`'s.
- In an impl, a constructor is written `fn new(src: &'a str) -> Self`.
  `fn new(src: &str) -> Self` is an error whose fix names `'a`.
- A returned type with several origins names each one:
  `-> Match<'n, 'l>` (§10.7).

### 3.7 `Owned` and `'static`

- `T: Owned` holds when `orig(T)` is empty after substitution, counting
  `'static` as empty. This is Rust's `T: 'static`. The bound `T: 'static` is
  accepted, and the formatter rewrites it to `T: Owned`.
- `&'static T` is `Owned` when `T` is. String literals have type
  `&'static str`, so `struct ParseError { msg: &'static str }` is `Owned`.
- The types of consts and statics are `Owned`. Statics are also `Sync` and
  `Pure` (`03-authority-concurrency.md` GLOBAL-1), since D7 allows no mutable
  globals. v0 has no lazy statics: a static's value is computed at compile
  time by `const fn` (D7).
- `Owned` is required by `Box<dyn Trait>` contents in a signature or field
  (§5.1), `dyn Any`, values handed to tasks that outlive their creator
  (§8.2), and global registrations.

## 4. Variance and subtyping

### 4.1 Rules

Tide subtyping comes only from origins and from instantiating higher-ranked
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

With Rust's lifetimes and variances, an elided `&mut Vec<&str>` has two
origins: a covariant outer one and an invariant element one. The borrow of
the `Vec` can end while its elements' origin lives on.

## 5. Trait objects, `impl Trait`, closures

### 5.1 `dyn Trait`

D2 follows Rust's object-lifetime defaults: in a signature or field,
`Box<dyn Trait>` is `Owned`, while `&dyn Trait` and `&mut dyn Trait` take
the reference's origin. In detail:

- **Directly behind a reference** (`&dyn Trait`, `&mut dyn Trait`), the
  object's origin defaults to the reference's, which is Rust's
  `&'a (dyn Trait + 'a)`. `&mut dyn FnMut(&Node)` callbacks that capture
  locals depend on it.
- **Everywhere else in signatures and fields**, including `Box`, `Rc`,
  `Arc`, and type arguments, `dyn Trait` is `dyn Trait + Owned`, unless the
  containing type's own `T: 'a` bound gives the default `'a`, as in Rust.
  Inside a body (`let` annotations, type arguments), an elided object
  lifetime is inferred, as in Rust.
- `dyn Trait + 'a` borrows from `'a` (§2.4 item 3).
- `dyn Trait + '_` is an elided origin and follows the elision rules. A
  field names a declared lifetime: `Box<dyn Fn(&Row) -> bool + 'p>`.
- `dyn Trait + 'static` is `Owned`.
- A `dyn` type has no auto traits unless it names them:
  `dyn Fn() + Send + Pure`.

### 5.2 `impl Trait`

- **Argument position.** `impl Trait` is an anonymous type parameter, as in
  Rust.
- **Return position.** As in Rust 2024, the opaque type captures every
  generic parameter in scope, lifetimes included, elided ones too;
  `+ use<..>` lists the captured parameters instead. It is `Owned` when
  every captured parameter is, or when its bounds state `Owned`
  (`+ 'static`), which the hidden type must then satisfy. So
  `fn evens(xs: &[u32]) -> impl Iterator<Item = u32>` borrows `xs` with no
  annotation, and `fn len_of<'a>(xs: &'a [String], key: &str) -> impl Fn()
  -> usize + use<'a>` releases the caller's `key` borrow at the call.
- **Type parameters** are captured as in Rust.
- **Auto traits.** An opaque type is `Send`, `Sync`, `Pure`, or in v1
  `ThreadBound` only if its bounds state it. Rust's auto-trait leakage would
  make dependents depend on the function's body, which D12 forbids. `Owned`
  instead follows `orig` (§2.2, §3.7), as Rust's `'static` does.

### 5.3 Closures

- A closure's origins are the union of its captures' origins. A by-reference
  capture of `p` contributes a reference with a loan on `p`. A by-value
  capture of a value of type `T` contributes `orig(T)`.
- A closure that captures a borrow is a borrowing value: it may be used
  locally, passed as an argument, or stored inside a borrowing type such as
  `Map<I, F>`.
- It may be returned, or stored in a field, only where its origins are named
  or captured: a returned `impl Fn() -> usize` (§5.2),
  `-> Box<dyn Fn() + 'a>`, a field
  `pred: Box<dyn Fn(&Row) -> bool + 'p>`, or a field whose type is a type
  parameter.
- `move` changes the capture mode, not whether the closure borrows. A `move`
  closure that captures a `&str` still carries that string's origin. A
  closure is `Owned` exactly when every captured value is `Owned`.
- Where `Owned` is required (`Box<dyn Fn()>` in a signature or field, a
  task that outlives its creator, a global registration), the usual fix is `move` plus owning the
  data (`to_string()`, `clone()`, or an `Arc`, created with `&Tasks`).

### 5.4 Higher-ranked origins

- Elided origins in the argument types of `Fn`, `FnMut` and `FnOnce` sugar,
  and of `fn` pointer types, are higher-ranked as in Rust.
  `F: Fn(&T) -> bool` is `for<'x> Fn(&'x T) -> bool`. D4's "no rank-N types"
  restricts types, not origins.
- A closure literal takes its signature from its expected type. If a closure
  returns a borrow of its argument and has no expected type, that is an
  error. The fix annotates the closure's parameter and return types.
- A higher-ranked origin gets a lower bound only through the
  well-formedness of the argument type that carries it: in
  `FnOnce(Scope<'_, 'env, E>)`, `Scope`'s declared `'env: 's` gives
  `ρenv ⊆ ρs`, so the
  closure must work for every `ρs` that carries the loans of `'env`.
  Scoped tasks use this (§8.2), as Rust's `std::thread::scope` does.
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

## 7. Field-borrow declarations (v1)

**Syntax.** A method receiver may name the fields it borrows:

```tide
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
union of its field borrows. The impl's named lifetimes keep their meaning.

**API stability.**

- The declared set is part of the signature and of the package interface.
- A method visible outside its package may name only fields that are also
  visible outside it, so private field names never enter a public API.
- For an exported method, removing a field from the set, or weakening
  `&mut` to `&`, is compatible. Adding a field or strengthening a mode is a
  breaking change.
- Trait methods cannot declare field borrows, because traits know no fields.
- If a type has `invariant` clauses (D10, v1), a field-borrow method assumes
  and owes each clause whose footprint lies inside its declared set, ignores
  each clause disjoint from it, and is rejected if it splits a clause's
  footprint (`04-contracts.md` §4.2; `reviews/round1-soundness.md` B10(d)).

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
| `Sender<T>`, `SyncSender<T>`; `Receiver<T>` | if `T: Send` | senders if `T: Send`; `Receiver` no |
| `Task<'s, U, E>` | if `U, E: Send` | if `U, E: Send` |
| capabilities, `Scope<'s, 'env, E>`, atomics | yes | yes |

`unsafe impl Send` is allowed in packages granted `unsafe`, and each one is a
trust hole.

**Task boundaries** require `Send` (by value) or `Sync` (by reference): a
spawned child's captures, so the closure is `Send`; channel messages;
`Shared` payloads; a child's result; and the branches of `join`, `par_map`
and `par_for`, including the slice elements they share or split (§8.5).

**Task isolation (D8).** Under the evented runtime (v1), a task may migrate
between OS threads at a suspension point while it holds values that are not
`Send`; v0's blocking and thread runtimes never migrate a task. The argument:

- Every way to share or transfer a value between tasks requires `Send` or
  `Sync`, so a value that is not `Send` is reachable from one task only.
- The scheduler's hand-off synchronizes (release on suspend, acquire on
  resume), so the task's own accesses stay ordered on any thread.

It depends on four conditions:

1. Safe code can reach no thread-local state (D7).
2. A task is pinned while a `ThreadBound` value is live across a suspension
   point, which the compiler knows from liveness. `ThreadBound` is
   structural, and a `dyn` may hold it only as `dyn Trait + ThreadBound`, so
   erasure cannot hide it (`03-authority-concurrency.md` §5.4).
3. Foreign code that relies on thread identity is wrapped in `ThreadBound`
   types.
4. Parallel branches (`join`, `par_map`, `par_for`) run simultaneously on
   several threads, so they require `Send` rather than isolation.

Rust has no counterpart, since tokio requires `Send` futures. This is the main
construct that §11 cannot translate.

### 8.2 Scopes

The names and failure semantics are those of `03-authority-concurrency.md`
§6.1. The origin structure is normative here.

```tide
impl Tasks {
    pub fn scope<'env, T, E, F>(&'env self, body: F) -> Result<T, E>
        where E: From<TaskError> + Send, F: FnOnce(Scope<'_, 'env, E>) -> Result<T, E>;
    pub fn spawn_isolated<T: Send, F>(&self, f: F) -> Result<T, Panicked>
        where F: FnOnce() -> T + Send + Owned;
}
pub struct Scope<'s, 'env: 's, E> { .. }      // Copy; 's and 'env invariant, as in std
impl<'s, 'env, E> Scope<'s, 'env, E> {
    pub fn spawn<U: Send + 's, F>(self, f: F) -> Task<'s, U, E>
        where F: FnOnce() -> Result<U, E> + Send + 's;
}
```

- **`Scope<'s, 'env, E>`** is a `Copy` handle. `'s`, written `σ` below, is
  the scope's origin; `'env` is the `&Tasks` borrow. `Scope` declares
  `'env: 's`, so `ρenv ⊆ σ`, and the elided `'_` in `body`'s argument makes `σ`
  higher-ranked (§5.4): `body` must work for every `σ` that carries the
  loans of the `&Tasks` borrow. Because the caller chooses that borrow,
  children can borrow anything that outlives the `scope` call.
- **`F: .. + 's` and `U: 's`** impose `F: σ` and `U: σ`.
  Everything a child borrows stays borrowed until the scope ends:
  - inside the body, `σ` is a placeholder, so it is live at every point;
  - scope exit joins every child and cannot be cancelled (D8);
  - no guard object exists, so `mem::forget` has nothing to skip.

  The parent body therefore cannot mutate, move or free a place that a child
  borrows.
- **`Task<'s, U, E>`** has origin `σ`. Because `σ` is higher-ranked, neither
  a `Scope` nor a `Task` can escape through `T`.
- **`spawn_isolated`** takes an `Owned` closure, so it borrows nothing from
  the caller and has no origin to check.
- **Grandchildren.** A child may capture the `Scope` (`Scope: σ` holds).
  `σ` is invariant, so it cannot shrink to admit shorter borrows.
- **A scope passed down** (`fn serve(bg: Scope<'_, '_, E>, conn: Conn)`)
  carries a signature origin in `serve`. A child spawned there must outlive
  it. That means it owns its data, or it borrows what the signature ties to
  the scope, as in `fn serve<'s>(bg: Scope<'s, '_, E>, cfg: &'s Config)`.
  This is the precise form of D2's "tasks spawned into a long-lived scope
  capture `Owned` values": a function's own locals never outlive a scope it
  received, so its children cannot borrow them.

Translation: `std::thread::scope`, with `'env` taken to be the `&Tasks`
borrow. `Scope<'s, 'env, E>` maps to `&'s std::thread::Scope<'s, 'env>`.

### 8.3 Child failure

Draft 3 removes draft 2's moved-out rule (D5). A child's panic propagates to
its scope as in Rust's `std::thread::scope`: the scope cancels the other
children, joins them all, and then re-raises in the calling task the panic
of the earliest-spawned panicking child (D8, `03-authority-concurrency.md`
TASK-2), so no code after the `scope` call runs. A panic in `body` itself cancels and joins the
children, then continues unwinding. A child that returns `Err` has finished
normally, like any function that returns an error.

The borrow checker therefore needs no rule for the failure edge. When
`scope` returns, `Ok` or `Err`, every child has finished normally, the loans
in `σ` end, and every place lent into the scope is usable again, as in Rust.
A place a panicking child was mutating is reached afterwards only by
unwinding, that is, by destructors, which `04-contracts.md` §4.3 verifies
without assuming the type's invariant.

The only place a panic becomes a value is `Tasks::spawn_isolated`, whose
closure is `Owned` (§8.2): it borrows nothing, so its `Err(Panicked)` exposes
no half-updated place of the caller. A cell whose `with` closure panicked is
poisoned (§8.4).

Memory safety never depended on a failure rule: abandoning a task leaves
every value valid memory, as unwinding does in Rust. Stack overflow is a
panic too (D5, `03-authority-concurrency.md` PANIC-1), so it unwinds
through scope exit, which joins the children that borrow the overflowing
task's frames before those frames are freed. A panic during unwinding
aborts the process (PANIC-2), so no frame is freed while a child still
borrows it.

### 8.4 `Shared<T>` and `Shared::with`

- **`Shared<T>`** requires `T: Send + Pure` (D8). It is a counted handle,
  invariant in `T`, with `orig(Shared<T>) = orig(T)`. It is `Send` and
  `Sync`, not `Pure`, and creating one takes `&Tasks` (D7).
- **`Shared::with(&a, f)`** and **`Shared::with2(&a, &b, f)`** (and the
  provisional `with_all` over a slice) replace draft 2's `lock` keyword. The
  closure has type `FnOnce(&mut T1, …) -> R`, which elision makes
  `for<'σ> FnOnce(&'σ mut T1, …) -> R`: each payload
  reference gets a fresh higher-ranked `σ`, so no value carrying `σ` escapes
  into the result or into captured places.
- **The closure** is `Pure`, so it captures no capability and no `Shared`
  handle; with `Pure` payloads it can neither suspend nor call `with` again
  (D8). Both methods return `Result<R, LockError>`, where `LockError` is
  `Aliased` or `Poisoned` (`03-authority-concurrency.md` LOCK-5).
- **Distinctness.** Before acquiring anything, `with2` checks that the
  handles are distinct, since two `&mut` to one payload would be undefined
  behavior. If both name the same cell, it returns `Err(LockError::Aliased)`
  and the closure does not run (D8, LOCK-2).
- **Poisoning.** A panic in the closure poisons the cells; a later `with`
  naming one returns `Err(LockError::Poisoned)` without running its closure.
  No safe API reads a poisoned payload; `cell.reset(v)` replaces it and
  clears the poison (LOCK-6).
- **Translation.** Mutex guards turned into `&'σ mut T` inside
  `for<'σ> FnOnce(&'σ mut T1, …) -> R`.

### 8.5 `join`, `par_map`, `par_for`

`join(f, g)` takes two `FnOnce + Pure + Send` closures, each called once.
`par_map` and `par_for` call one closure from several branches at once, so
it is shared and must be `Fn + Pure + Send + Sync` (D9): without `Sync`, a
`move` closure owning a `Cell` would let two branches race on it. Their
shared captures must be `Pure + Sync`; mutable captures must be disjoint
`&mut` borrows, which the borrow checker verifies as usual. The branches may
borrow the enclosing frame, because `join` returns only after both finish. In
Rust this is the shape of `rayon::join`. `Pure + Sync` is stricter than
`Sync`; that serves determinism, not memory safety. Draft 2's `par { }`
block is removed (D9).

Element types follow the same rule (D9, `03-authority-concurrency.md`
PAR-4): every element is `Pure`, plus `Sync` when branches share it and
`Send` when branches split or move it. `par_map` requires `T: Pure + Sync`,
and `par_for`, `par_chunks_mut` and `par_sort` require `T: Pure + Send`.
Without the `Pure` bound, a slice of `Shared` handles or channel ends would
let a closure that captures nothing lock cells or block in schedule order.

If a branch panics, the panic propagates (D5), and the places the region
could write through `&mut` hold unspecified but valid values: a branch may or
may not have written before it stopped. Only destructors observe them, as
after a panicking child (§8.3).

## 9. `Pure`

`Pure` is a compiler-known auto trait. Outside std, it can be implemented
only by `unsafe impl` in a package granted `unsafe`, and each such impl is a
trust hole. It is **deep**:

- it looks through every field, private ones included, and through `Box` and
  `Rc`, with type arguments substituted;
- recursive types take the greatest fixed point;
- opaque and `dyn` types have exactly the auto traits their bounds state
  (`Owned` alone follows `orig`, §3.7);
- a type parameter is `Pure` only if it is bounded by it.

Coherence never treats a missing auto trait (`Send`, `Sync`, `Pure`, `Owned`)
as making two impls disjoint (D4): `impl<T: Debug + Pure> K for T` and
`impl K for Net` overlap, although `Net` is not `Pure`. So a private-field
edit that changes a type's auto traits cannot create or remove an overlap.

### `Pure`: no authority reachable

These are **not `Pure`**:

- the capability types `Root`, `Fs`, `Net`, `Clock`, `Rand`, `Env`, `Stdin`,
  `Proc`, `Tasks` and `Accel`, and their attenuated forms;
- the synchronization handles `Shared<T>`, `Sender<T>`, `SyncSender<T>`,
  `Receiver<T>`, `Scope<'s, 'env, E>`, `Task<'s, U, E>`, `Arc<T>`, `sync::Weak<T>` and
  every atomic type;
- the foreign handles: raw pointers, `extern` types and foreign function
  pointers;
- `ThreadBound` types;
- `dyn Trait` unless `+ Pure` is written;
- an unbounded type parameter.

**Everything else is `Pure` exactly when its components are**, including
references, std collections, `Rc`, `rc::Weak`, `Cell`, `RefCell`, `Arena`,
structs, enums and closures. Tide function pointers are `Pure`: any
authority the pointee uses arrives through its arguments.

**What `Pure` guarantees.** A function whose parameter and capture types are
all `Pure` is authority-free: no I/O, time, randomness, synchronization with
other tasks, or suspension. Its result is a function of the state of memory
reachable from its arguments. `Cell`, `RefCell` and `Rc` are `Pure` because
that state is private to the task. Creating a synchronization handle takes
`&Tasks`, which is not `Pure`, so authority-free code can neither receive
nor create one (D7).

**What std must provide in return.** No operation on a `Pure` type may return
a result that depends on other tasks. `Arc` and `sync::Weak` are therefore
synchronization handles (D7): even without count readers, the last release
runs `T::drop` in whichever task drops last, `make_mut` calls `Clone` only
when another clone exists, and `upgrade` observes other tasks' drops. Tide's
`Arc` also omits `strong_count`, `weak_count`, `get_mut`, `try_unwrap` and
`into_inner`. Authority-free and parallel code shares data by reference
inside a scope instead.

### `Pure + Sync` replaces `Frozen`

Draft 2 had a second deep trait, `Frozen`, for "no interior mutability
through `&`". Draft 3 removes it: `Sync` already excludes `Cell`, `RefCell`,
`OnceCell`, `UnsafeCell` and `Rc`, and `Pure` excludes atomics and every
other synchronization handle, so nothing reachable from a `Pure + Sync`
value can change while another task or branch reads it. Like `Pure`, `Sync`
is structural, so an `Rc` inside a `Box` disqualifies a type
(`reviews/round1-soundness.md` B6(e)).

### Where `Pure` is used

| Use | Requires |
| --- | --- |
| `join`; `par_map`, `par_for`, `par_chunks_mut`, `par_sort` | `FnOnce + Pure + Send` closures; `Fn + Pure + Send + Sync` closures; both with `Pure + Sync` shared captures; elements `Pure + Sync` (`par_map`) or `Pure + Send` (the others) |
| memoization and `const fn` | authority-free, `Pure + Sync` parameters, no `&mut` parameters (D7) |
| statics (computed at compile time) | `Owned + Sync + Pure` |
| `Shared` payloads; `with` closures | `Send + Pure`; `Pure` |

## 10. Worked examples

Tide's lifetime spelling is Rust's (§2.6), so each example is also the Rust
program, except where a `rust` block shows a difference. Each verdict
matches rustc's.

### 10.1 Iterators over a `Vec`

```tide
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
pub struct Iter<'a, T> { rest: &'a [T] }
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> { … }
}
```

**Accepted.** By rule 1, every elided `&` in a result gets `people`'s origin,
and `adults` captures `people`'s origin (§5.2). Without `move`, its closure would borrow `min_age` and be rejected,
as in Rust.

### 10.2 `HashMap` get and entry

```tide
fn count_words(text: &str) -> HashMap<&str, usize> {
    let mut counts = HashMap::new();
    for w in text.split_whitespace() {
        *counts.entry(w).or_insert(0) += 1;
    }
    counts
}

fn add_word<'w>(counts: &mut HashMap<&'w str, usize>, w: &'w str) {
    *counts.entry(w).or_insert(0) += 1;
}

impl Index {
    fn get(&self, key: &str) -> Option<&Entry> { self.map.get(key) }
}
fn find<'c>(cache: &'c HashMap<String, Entry>, key: &str) -> Option<&'c Entry> {
    cache.get(key)
}
```

**Accepted.** `entry(&mut self, k: K) -> Entry<'_, K, V>` and
`Entry::or_insert(self, v: V) -> &'a mut V` elide or name as in Rust.
`Entry` has one origin of its own; the origins of `K` and `V` are not
counted.

**Rejected**, as in stable Rust, is the conditional return of a lookup
(NLL problem case #3):

```tide
fn get_default<'m>(map: &'m mut HashMap<String, String>, k: &str) -> &'m mut String {
    if let Some(v) = map.get_mut(k) { return v; }   // error: `*map` still mutably borrowed
    map.insert(k.to_string(), String::new());
    map.get_mut(k).unwrap()
}
```

The idiom is `map.entry(k.to_string()).or_default()`.

### 10.3 A lexer holding `&str`

```tide
pub struct Lexer<'a> { src: &'a str, pos: usize }

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self { Lexer { src, pos: 0 } }
    pub fn next_word(&mut self) -> Option<&'a str> { … }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> { self.next_word() }
}

fn first_two(text: &str) -> (Option<&str>, Option<&str>) {
    let mut lx = Lexer::new(text);
    let a = lx.next_word();
    let b = lx.next_word();      // fine: `a` borrows `text`, not `lx`
    (a, b)
}
```

**Accepted.** With an elided `-> Option<&str>`, rule 2 ties the token to the
`&mut self` borrow, and the second call is rejected (Rust behaves the same).
The diagnostic suggests `&'a str`.

### 10.4 A parser returning slices of its input

```tide
pub struct Parser<'a> { lx: Lexer<'a>, peeked: Option<&'a str> }   // one origin: 'a

impl<'a> Parser<'a> {
    pub fn new(text: &'a str) -> Self { Parser { lx: Lexer::new(text), peeked: None } }

    fn peek(&mut self) -> Option<&'a str> {
        if self.peeked.is_none() { self.peeked = self.lx.next_word(); }
        self.peeked
    }

    pub fn pair(&mut self) -> Result<(&'a str, &'a str), ParseError> { … }
}

pub fn split_header(line: &str) -> Option<(&str, &str)> {
    let (k, v) = line.split_once(':')?;
    Some((k.trim(), v.trim()))
}
```

**Accepted.** The tokens outlive every `&mut self` call and live as long as
the text.

### 10.5 Builders

```tide
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

pub struct ArgList<'a> { args: Vec<&'a str> }
impl<'a> ArgList<'a> {
    pub fn arg(&mut self, a: &'a str) -> &mut Self {
        self.args.push(a);
        self
    }
}
```

**Accepted.** `ArgList::arg` needs the impl's `'a` on `a`. With an elided
`a: &str`, `a` has its own origin and the push is rejected ("`a` must
outlive `'a`"). Write `-> &mut Self`. Writing `-> &mut ArgList<'_>` would
give the inner lifetime to the `&mut self` borrow by rule 2, and the body
would be rejected.

### 10.6 `longest`

```tide
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

fn longest_word<'w>(words: &[&'w str]) -> &'w str {
    words.iter().copied().max_by_key(|w| w.len()).unwrap_or("")
}

fn title(text: &str) -> &str {
    let words: Vec<&str> = text.split(' ').collect();
    longest_word(&words)              // fine: the result borrows `text`, not `words`
}
```

**Accepted.** This is `reviews/round1-soundness.md` B2.1. Draft 1 rejected it.

### 10.7 A struct holding references from two sources

```tide
pub struct Match<'n, 'l> { needle: &'n str, line: &'l str }   // two origins

pub fn find<'n, 'l>(needle: &'n str, text: &'l str) -> Option<Match<'n, 'l>> {
    let line = text.lines().find(|l| l.contains(needle))?;
    Some(Match { needle, line })
}

fn first_error(text: &str) -> Option<&str> {
    let pat = String::from("error");
    let m = find(&pat, text)?;
    Some(m.line)                      // fine: `line` borrows only `text`
}
```

**Accepted.** With one shared lifetime,
`fn find<'a>(needle: &'a str, text: &'a str) -> Option<Match<'a, 'a>>`, both
fields would carry both loans, and `Some(m.line)` would be rejected,
because `pat` dies at the end of `first_error`.

### 10.8 A struct whose fields share an origin

```tide
struct Collector<'s, 'o> { src: &'s str, out: &'o mut Vec<&'s str> }

impl Collector<'_, '_> {
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

**Accepted.** If the elements had a lifetime of their own
(`out: &'o mut Vec<&'e str>`), `emit` would be rejected ("`'s` must outlive
`'e`"). If one name covered everything
(`struct Collector<'a> { src: &'a str, out: &'a mut Vec<&'a str> }`), the
`&mut toks` borrow would last as long as the tokens and `tokenize` would be
rejected at `toks`. Draft 1's single region made this example either
unsound or unusable (B1).

### 10.9 A graph through `Arena` handles

```tide
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
`Owned`, `Pure`, `Send` and `Sync`, so it can be stored anywhere. A
stale handle panics on indexing; `get` returns `None`. Two nodes are mutated
together with `g.get_disjoint_mut([a, b])`, checked at run time.

### 10.10 `Rc<RefCell<T>>`

```tide
struct TreeNode { val: i32, children: Vec<Rc<RefCell<TreeNode>>>, parent: rc::Weak<RefCell<TreeNode>> }

let leaf = Rc::new(RefCell::new(TreeNode { val: 3, children: vec![], parent: rc::Weak::new() }));
let root = Rc::new(RefCell::new(TreeNode { val: 5, children: vec![leaf.clone()], parent: rc::Weak::new() }));
leaf.borrow_mut().parent = Rc::downgrade(&root);
root.borrow_mut().val += leaf.borrow().val;
```

**Accepted.** The type is `Owned` and `Pure`, but not `Send` or `Sync`. It
may be held across suspensions within one task (§8.1) but cannot enter
another task or a parallel branch; use `Shared` there. A failed `borrow_mut`
panics.

### 10.11 A trait object holding a reference

```tide
fn words<'a>(text: &'a str) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(text.split_whitespace())
}

struct Filter<'p> { pred: Box<dyn Fn(&Row) -> bool + 'p> }    // one origin: 'p

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

**`words`, `Filter` and `count_leaves` are accepted.** `count_leaves` relies
on §5.1's default bound behind `&mut`.

**`job` is rejected, as in Rust.** `Bytes` borrows `text`, but this
`dyn Iterator` is `Owned`. There are two fixes:

- `struct Job<'a> { next: Box<dyn Iterator<Item = u8> + 'a> }`, after which
  `fn job(text: &String) -> Job<'_>` elides by rule 1;
- owning the data: `text.clone().into_bytes().into_iter()`.

### 10.12 Scoped tasks borrowing a slice

```tide
fn parallel_sum(tasks: &Tasks, xs: &[u64]) -> Result<u64, TaskError> {
    let mid = xs.len() / 2;
    let mut sums = [0u64; 2];
    let [lo, hi] = &mut sums;
    tasks.scope(|s| {
        s.spawn(|| { *lo = xs[..mid].iter().sum(); Ok(()) });
        s.spawn(|| { *hi = xs[mid..].iter().sum(); Ok(()) });
        Ok(())
    })?;
    Ok(sums[0] + sums[1])             // fine: every child has finished
}

fn after_failure(tasks: &Tasks, xs: &[u64]) -> u64 {   // SumError: #[from] TaskError, Overflow
    let mut sums = [0u64; 2];
    let [lo, _] = &mut sums;
    let r: Result<(), SumError> = tasks.scope(|s| {
        s.spawn(|| { *lo = checked_sum(xs).ok_or(SumError::Overflow)?; Ok(()) });
        Ok(())
    });
    match r { Ok(()) => sums[0], Err(_) => sums[0] }   // fine: the loans ended with the scope
}

fn serve(bg: Scope<'_, '_, ServeError>, conn: Conn) {  // read_into(&self, &mut [u8]) -> Result<usize, ServeError>
    let mut buf = [0u8; 4096];
    bg.spawn(|| conn.read_into(&mut buf));                                 // error
}

fn serve_owned(bg: Scope<'_, '_, ServeError>, conn: Conn) {
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

**`after_failure` is accepted.** A child that returns `Err` has finished, so
`sums` is usable on both arms. Draft 2's moved-out rule rejected it. If the
child panicked instead, `scope` would panic and neither arm would run (§8.3).

**`serve` is rejected**: its child borrows the locals `conn` and `buf`,
neither of which outlives `bg`'s scope (`reviews/round1-soundness.md`
B2.4). **`serve_owned` is accepted**, because its closure owns everything it
uses.

Like Rust's scope, Tide's re-raises a child's panic (§8.3). For pure work
the Tide idiom is
`let (a, b) = join(|| sum(&xs[..mid]), || sum(&xs[mid..]));`.

## 11. Soundness sketch: translation to Rust

**Claim.** For every program `P` that the Tide checker accepts, the
translation `τ(P)`, Rust with explicit lifetimes, is accepted by rustc's NLL
borrow checker, except at the constructs listed as non-exact below.

Rust's soundness covers the rest: RustBelt for the core type system, plus
std's audited unsafe code. That gives memory safety and data-race freedom
for `P`. `τ` is a proof device and the CI oracle of D12's origin cross-check;
it is not a backend. The cross-check runs on the test corpus and on every
change to std. A program that Tide accepts and rustc rejects blocks the
release.

### 11.1 The translation

| Tide | Rust | Exact? |
| --- | --- | --- |
| Lifetime parameters, `'_`, `'static`, outlives bounds, `use<..>`, elision, implied bounds, object-lifetime defaults | the same | yes |
| Return-position `impl Trait` | the same capture; auto traits stated, since Tide does not leak them (§5.2) | yes |
| `Owned` | `'static` | yes |
| Closures, `Fn` sugar, higher-ranked lifetimes, `Scope`'s `'env: 's` | the same | yes |
| Field-borrow methods | associated functions with split borrows | yes |
| Two-phase borrows, NLL, drop check | the same | yes |
| The std eyepatch | `#[may_dangle]` (unsafe, audited) | audited |
| `Tasks::scope`, `Scope::spawn` | `std::thread::scope` with `'env` = the `&Tasks` borrow | yes |
| A child's panic | `std::thread::scope`'s re-raise; the `Err` edge needs no extra rule | yes |
| `spawn_isolated` | `catch_unwind(AssertUnwindSafe(…))` over a `'static` closure | yes |
| `Shared::with`, `with2` | `for<'σ> FnOnce(&'σ mut T1, …)` over Mutex guards, plus a run-time distinctness check in std | audited |
| `join`, `par_map`, `par_for` | `rayon::join` or `thread::scope`; `Pure + Sync` is stricter than `Sync` | yes |
| A non-`Send` value inside a task that migrates (v1) | none: Rust requires `Send` futures | **no** |

### 11.2 Where the translation is not exact

- **R1. Task isolation (§8.1, v1): medium risk.** Rust cannot express it.
  Soundness rests on the argument in §8.1, which needs a written proof in the
  style of Swift's SE-0414 and an audit of std, the runtime and FFI wrappers
  for thread-affine state. The proof must cover erasure: a `ThreadBound`
  value boxed as `dyn Debug` must still pin (round-2 consistency review B8).
- **R2. Bounded higher-ranked origins (§5.4): low risk.** They come only
  from well-formedness, the encoding that `std::thread::scope` relies on.
- **R3. Implied bounds through `fn` pointers: low risk once implemented.**
  Reducing to Rust inherits Rust's holes, such as rust-lang/rust#25860, which
  the oracle cannot catch because rustc accepts them. §5.4's re-check closes
  it; known exploits belong in the negative test corpus.
- **R4. The audited std set** (`Arena`, `Shared` and its `with` methods,
  channels, `Scope`, the eyepatch containers) has Rust std's trust model and
  is counted in the trust surface.
- **R5. `Pure`: medium risk.** It promises determinism, which lies outside
  Rust's model and depends on classifying std correctly (`Arc` was first
  classified `Pure`; round 2 showed its drops and `make_mut` reveal timing).
- **R6. Child failure (§8.3)** follows `std::thread::scope` exactly; the only
  recovery point, `spawn_isolated`, takes `Owned` closures.
- **R7. The evented lowering of scoped children** needs non-moving frames
  and uncancellable joins in the C backend, outside `τ`.

## 12. Known limitations and open questions

1. **NLL precision.** NLL problem case #3 and lending iterators are rejected,
   as in stable Rust. The idioms are the entry API and index loops.
   Polonius-level precision is a compatible later relaxation.
2. **Self-referential structs** need `unsafe` (a cost D2 accepts). There is
   no `Pin`, since there is no `async`.
3. **`&'arena T` graphs** in the style of rustc's `TyCtxt` need an
   append-only arena (`alloc(&self, v: T) -> &mut T`) with an eyepatch. v0
   std ships only handle-based `Arena<T>`.
4. **Exported field groups.** Named borrow sets that a type exports without
   exposing its fields, Rust's "abstract fields", are open.
5. **Origin propagation.** A new reference field needs a lifetime parameter
   on its type and on every type that contains it, as in Rust. The edit is
   local and visible, and the interface diff reports it in public types as
   breaking.
6. **Lifetimes Tide cannot express.** These are lifetime parameters on
   traits, lending traits that need GATs, and `for<'a>` beyond elision (D4).
   Interop with lifetime-heavy Rust APIs is out of scope (Positioning).
7. **`Arc` count observers.** Something must replace `Arc::try_unwrap` and
   `get_mut` in the "unwrap after the workers finish" pattern, probably
   scope results.
8. **Invariants after a child failure.** Closed by draft 3: a child's panic
   propagates through its scope (§8.3), so no half-updated referent behind a
   `&mut` parameter reaches a caller.
9. **Agent-facing repair.** E1 should measure whether agents find the right
   lifetime from diagnostics; the origin trial produced no borrow errors to
   repair (`trials/e1-origins/grading.md`). It also did not exercise
   outlives bounds, `use<..>`, or `Scope<'s, 'env, E>`.

## 13. Changes this chapter asked of `01-design.md`

Draft 3 applies items 1–4 and 7. Item 5 is superseded: D7 makes `Arc` and
`sync::Weak` synchronization handles (§9). Item 6 is superseded: D5 removes
the moved-out rule (§8.3). Draft 3.4 supersedes items 2 and 3: lifetimes
keep Rust's spelling (D2, §2.6). None are outstanding.

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
