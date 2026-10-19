# Round 1 Review: Soundness and Expressiveness

Reviewer persona: a programming-language researcher who has worked on the Rust
borrow checker and trait solver. The brief was to break the design, not to
praise it.

Scope: `design/00-first-principles.md` and `design/01-design.md`, read in full,
with `research/02`, `research/07`, `research/08`, and `research/10` consulted.
Code examples use the Ferox surface from `01-design.md`. Where the design is
silent, I state which reading I assume and show what goes wrong under each
plausible reading.

## Verdict

**request_changes**

The direction is right, and most of it can be saved. Second-class references
with declared views, capabilities as values, structured stackful tasks, and
definition-checked generics are a coherent set of choices. But the core safety
claims are stated as conclusions, not as rules:

- D2 claims "Rust's lifetime system restricted to one implicit region per type
  and one region per borrowed parameter … without losing soundness". As
  written, this is either unsound or unusable, depending on how it is read. It
  also leaves out three pieces of machinery that Rust needs for soundness:
  variance, a `'static` analog, and drop check.
- D2's closure rules cannot type D8's headline feature, children borrowing
  from the parent's frames.
- D7's `Pure` is defined as "contains no capability". That does not imply
  determinism, non-suspension, or race freedom, which D7, D8, and D9 all
  assume.
- D11's equivalence claim is false in at least four concrete ways.

None of this needs a new direction. Each objection has a small fix, and
several share one: place-named origins, which `research/02` already
recommended and the design dropped. The design should not freeze until the
region calculus in B1 and B2 exists as a written, checkable chapter.

## Blocking objections

### B1. D2: the region model is unspecified, and its literal reading is unsound or unusable

**Problem.** "One implicit region per view type and one region per borrowed
parameter" has three readings. None of them matches the claim.

**Reading (a): one region per parameter, held invariant.** One region covers
the outer borrow and every region inside the pointee, and it is invariant, as
`&mut` requires. Every `&mut V` for a view `V` then becomes Rust's
`&'a mut V<'a>`: the view stays mutably borrowed for the rest of its life.
That makes every `&mut self` method on a view a one-shot method:

```ferox
struct Lexer { src: &str, pos: usize }          // a view
impl Lexer {
    fn bump(&mut self) { self.pos += 1; }       // self: &ρ mut Lexer<ρ>, one region ρ
}
fn count(text: &str) -> usize {
    let mut lx = Lexer { src: text, pos: 0 };
    lx.bump();
    lx.bump();      // error: `lx` is still mutably borrowed by the first call
    lx.pos
}
```

The same happens to every out-parameter of view type, such as
`fn collect(out: &mut Vec<&str>)`. After one call, the caller's `Vec` can no
longer be used.

**Reading (b): one region, kept covariant so that (a) stays usable.** This is
unsound, because `&mut T` must be invariant in `T`:

```ferox
fn main(root: Root) -> Result<(), AppError> {
    let mut names: Vec<&str> = Vec::new();
    {
        let tmp = String::from("temp");
        let r: &mut Vec<&str> = &mut names;   // one region for the type; shrunk to this block
        r.push(&tmp);                         // accepted: tmp outlives the shrunk region
    }                                         // tmp freed
    root.stdio.print(names[0]);               // use after free
    Ok(())
}
```

`Cell<&str>`, `RefCell<Vec<&str>>`, and `Shared<…>` inside a view have the
same hole.

**Reading (c): Rust's input elision.** Every reference and every view in a
parameter gets a fresh region, with variance computed as in Rust. This is
sound and usable, but it is not "one region per borrowed parameter", and it
leads straight into B2. Even under (c), a user-defined view that has one
region and holds a `&mut` to another view borrows it forever:

```ferox
struct Ctx { src: &str, out: &mut Vec<&str> }   // one region shared by three references
fn tokenize(text: &str) -> usize {
    let mut toks: Vec<&str> = Vec::new();
    run(Ctx { src: text, out: &mut toks });
    toks.len()      // error: toks mutably borrowed for the region of its own elements
}
```

Rust solves this with `Ctx<'s, 'o>`. Ferox has no spelling for it except a
hidden encoding through a type parameter (`struct Ctx<V> { src: &str, out: &mut V }`).

**Smallest fix.** Write the region calculus as a normative chapter before any
more surface design:

1. Every `&`, `&mut`, and view type carries its own hidden region.
2. Variance is computed as in Rust: invariant under `&mut`, `Cell`, `RefCell`,
   and `Shared`.
3. Elision is Rust's input elision (a fresh region per position) plus Rust's
   `&self` output rule.
4. A view field may name a sibling field's region (`out: &mut Vec<&str from src>`),
   which is the field-level form of B2's fix.

Argue soundness by translation: every accepted Ferox program maps to a Rust
program with explicit lifetimes. D12 already emits Rust, so the emitter can
write explicit lifetimes and let `rustc`'s borrow checker re-check every
release build in CI. That gives a free, permanent soundness oracle for the
region checker.

### B2. D2 and D8: `from` can name a parameter, but not a region inside one, and it cannot relate two parameters

**Problem.** `from` applies only to return values, and only to whole
parameters. Four common shapes are inexpressible or wrongly restricted.

**1. Returning the inner region.**

```ferox
fn longest(words: &[&str]) -> &str { … }     // inferred: from `words`, the outer slice borrow
fn title(text: &str) -> &str {
    let words: Vec<&str> = text.split(' ').collect();
    longest(&words)       // error: result borrows `words`, which dies here
}                         // Rust: fn longest<'a>(words: &[&'a str]) -> &'a str
```

**2. Zero-copy lexers and parsers.** A token is tied to the `&mut self` borrow
instead of to the source text:

```ferox
impl Lexer { fn next_token(&mut self) -> &str { … } }    // inferred: from self
let a = lx.next_token();
let b = lx.next_token();    // error: lx still mutably borrowed through `a`
```

Trait impls can avoid this through `Self::Item`, but only if the impl writes
`Self::Item` instead of `&str`. If it writes `&str`, elision ties the result
to `&mut self` and the signature no longer matches the trait's.

**3. Out-parameters and accumulators.** These need "the second argument
outlives the first argument's elements":

```ferox
fn add_word(counts: &mut HashMap<&str, u32>, w: &str) {
    *counts.entry(w).or_insert(0) += 1;   // error: no way to say w outlives the keys
}
```

The only workaround is to make the helper generic so that the region rides
inside a type parameter
(`fn add_word<K: Hash + Eq>(counts: &mut HashMap<K, u32>, w: K)`). Agents will
not find this, and it turns monomorphic code generic.

**4. Scoped spawn, which is D8's headline feature.** `Scope::spawn` must
accept a closure that borrows anything that outlives the scope and nothing
shorter. In Rust that is `F: FnOnce() + Send + 'scope` with `&'scope self`.
D2 offers two closure kinds. A non-escaping closure dies at the end of the
call to `spawn`, so `spawn` cannot keep it. A `move` closure owns everything it
captures, so it cannot borrow the parent frame. Neither kind types spawn, so
the standard library's signature cannot be written in the language. Passing a
scope down, which is the Trio background-work pattern that `research/10`
recommends, is then either impossible or unsound:

```ferox
fn handle(bg: &Scope, conn: Conn) {
    let buf = [0u8; 4096];
    bg.spawn(|| conn.read_into(&buf));   // must be rejected: the child outlives `handle`
}
```

**Smallest fix.** Use place-named origins, Mojo style, with no `'a` syntax.
`research/02` recommended this ("use Model C's place-named dependencies as the
syntax for Model A's views"). Give `from` one meaning everywhere: `T from p`
means the region of `T` is the region named by place `p`. Allow it in return
types and in parameter types, and let `p` be a path:

- `fn longest(words: &[&str]) -> &str from *words`
- `fn next_token(&mut self) -> &str from self.src`
- `fn add_word(counts: &mut HashMap<&str from w, u32>, w: &str)`
- `fn spawn<F: FnOnce() + Send>(&self, f: F from *self)`

The last one reads as "f's borrows must cover the scope's region". One keyword
handles all four shapes, it stays greppable, and it lowers directly to Rust
lifetimes for the B1 oracle.

### B3. D2: there is no `'static` or owned region, and `dyn` and `move` closures erase regions

**Problem.** Nothing distinguishes "contains no borrows" from "contains
borrows of some region". Three holes follow.

**A `move` closure over a reference.** The closure "owns everything it
captures", but one of the things it owns is a reference:

```ferox
fn make(s: &String) -> Box<dyn Fn() -> usize> {
    let r: &str = s.as_str();
    Box::new(move || r.len())    // if `move` means escaping means non-view: dangling once s dies
}
```

**A view coerced to `dyn`.** This erases its region:

```ferox
struct Job { next: Box<dyn Iterator<Item = u8>> }   // a non-view struct
fn job(text: &String) -> Job { Job { next: Box::new(text.bytes()) } }   // use after free
```

**Literals and constants.** These become views:

```ferox
struct ParseError { msg: &str }            // a view, so it cannot go in Box<dyn Error>,
let e = ParseError { msg: "eof" };         // a non-view struct, or a channel to a long-lived task
const KEYWORDS: [&str; 3] = ["fn", "let", "if"];   // a constant of view type?
```

Generic code that stores a `T` somewhere long-lived also has no bound to ask
for. That covers `Box<dyn Any>`, caches inside non-view structs, and a
background scope.

The design also does not say whether a non-`move` closure can be stored in a
struct. In Swift, a non-escaping closure cannot be stored at all. If Ferox
follows Swift, lazy iterator adapters (`Map<I, F>`) are impossible. If it does
not, a closure is simply a view, and "non-escaping" means nothing extra.

**Smallest fix.**

- Add a `static` region. String literals have type `&static str`.
- Add an auto-marker `Owned`, meaning "no non-static regions", derived
  structurally like `Send`.
- Make `dyn Trait` mean `dyn Trait + Owned`, and add `dyn Trait from p` for
  borrowing trait objects.
- Make every closure that captures a view a view, whether or not it is `move`.
  "Escaping" becomes a region judgment, not a keyword.

### B4. D2: there is no drop check

**Problem.** Views plus user destructors plus interior mutability is exactly
the combination that needs Rust's drop check (RFC 769, RFC 1238, and
`#[may_dangle]`). D2 says nothing about it.

```ferox
struct Node { name: String, peer: Cell<Option<&Node>> }    // a view
impl Drop for Node {
    fn drop(&mut self) {
        if let Some(p) = self.peer.get() { dbg(p.name.len()); }   // reads a freed Node
    }
}
fn main(root: Root) -> Result<(), AppError> {
    let a = Node { name: String::from("a"), peer: Cell::new(None) };
    let b = Node { name: String::from("b"), peer: Cell::new(None) };
    a.peer.set(Some(&b));
    b.peer.set(Some(&a));
    Ok(())
}   // b is dropped first; a.drop() then reads b.name after its buffer is freed
```

Rust rejects this program through drop check. D2's "arena-scoped references"
are the `typed_arena` case, which is sound in Rust only because of a carefully
audited `#[may_dangle]`.

**Smallest fix.** A type with an explicit `Drop` impl that may contain a
region, whether a view type or a generic type over a possible view, requires
those regions to strictly outlive the value. That is Rust's conservative rule.
`Vec`, `Arena`, and the other std containers get an audited eyepatch that is
counted in the trust surface. The simplest alternative is to forbid user
`Drop` on view types.

### B5. D6 and D7: `Shape` reflection forges capabilities and bypasses invariants

**Problem.** D6 gives every nominal type a `Shape`, with "opt-out for types
with invariants". The Feel's `json::decode` builds values through `Shape`. The
same call builds authority out of bytes, inside a function that D7 rates as
pure:

```ferox
pub struct Config { pub name: String, pub out: Fs }

pub fn parse_config(body: &[u8]) -> Result<Config, json::Error> {   // no capability in sight
    json::decode(body)               // constructs an `Fs` from bytes; or decode::<Root>(b"{}")
}
```

Opt-out is a default-allow. One forgotten opt-out on a capability type breaks
D7. One forgotten opt-out on a type whose invariant unsafe code relies on
(UTF-8 in a string-like type, sortedness used to skip bounds checks) turns
safe code into undefined behavior.

**Smallest fix.**

- Reading through `Shape` stays universal.
- Constructing through `Shape` requires that every field be visible at the use
  site, or that the type opt in.
- Capability types, and types that contain them, can never be constructed
  through `Shape`. This is a language rule, not a library convention.

### B6. D7 and D9: `Pure` does not imply determinism, non-suspension, or race freedom

**Problem.** D7 defines `Pure` as "contains no capability" and then claims
that pure functions are "deterministic, eligible for `const fn`, memoization,
`par`, and proofs". D8 adds that "a pure function never suspends". Each
property below is violated by a function that D7 classes as pure.

**(a) Shared cells and channel endpoints contain no capability.**

```ferox
fn next_id(c: &Shared<u64>) -> u64 {          // Pure by D7
    lock (c) |n| { *n += 1; *n }               // result depends on other tasks; may suspend
}
```

The same holds for `Receiver<T>::recv` if channel endpoints are not
capabilities.

**(b) `dyn` hides what it captures.** `fn run(job: &dyn Fn())` and
`Box<dyn Any>` (downcast to `Net`) hide captured authority. A structural rule
must treat `dyn` as impure unless it is spelled `+ Pure`, as Rust does for
`Send`.

**(c) FFI needs no capability to call.** A package with the `ffi` grant can
export `fn now_ns() -> u64`, and its computed authority is empty. D9's
automatic parallelization then runs non-reentrant C code on several threads.
That is a data race and undefined behavior in "safe" code.

**(d) Pure does not mean memoizable.** `fn bump(v: &mut Vec<i32>)` and
`fn peek(x: &Rc<RefCell<i32>>) -> i32` are pure under D7, but neither is
memoizable or referentially transparent.

**(e) `par` branches share one task but run on several threads.**

```ferox
fn twice(xs: &Rc<Vec<i32>>) -> usize {       // Rc holds no capability
    let (a, b) = par { xs.clone().len(), xs.clone().len() };   // two threads, one non-atomic count
    a + b
}
```

D9's "without shared interior mutability" catches this only if the check is
deep. Rust's `Freeze` is shallow: `Rc`'s count sits behind a pointer, so `Rc`
passes. And D8's "the task, not the OS thread, is the isolation unit" must not
apply to `par` branches.

**Smallest fix.** Define two structural properties and use each one only
where it is sound.

`Pure` means all of the following:

- no capability;
- no cross-task synchronization handle (`Shared`, channel endpoints, atomics,
  and task handles all count as authority-bearing);
- no raw pointers;
- `dyn` only when spelled `+ Pure`;
- `extern` functions are impure unless marked `#[trusted] pure`, which counts
  as a trust hole.

`Pure` gives determinism within a task and no suspension.

`Frozen` means `Pure`, plus no interior mutability reachable through any
indirection, plus `Share` on every captured reference. Use `Frozen` for `par`,
automatic parallelization, `const fn`, and proofs. Memoization additionally
requires no `&mut` parameters.

### B7. D8: `lock (a, b)` is neither sound nor deadlock-free as specified, and scope join must not be cancellable

**(a) Aliasing.**

```ferox
fn transfer(a: &Shared<Account>, b: &Shared<Account>, amt: u64) {
    lock (a, b) |x, y| { x.balance -= amt; y.balance += amt; }
}
transfer(&acct, &acct, 10);   // x and y are two `&mut` to one Account
```

Depending on the implementation, this self-deadlocks or produces aliased
`&mut`, which is undefined behavior and a `noalias` miscompile. Distinctness
cannot be proved statically in general.

**(b) Nesting through calls, through data, and through capabilities inside
the cell.**

```ferox
fn credit(acc: &Shared<Account>, amt: u64) { lock (acc) |x| { x.balance += amt; } }
lock (a) |x| { credit(b, 1) }          // nested acquisition hidden in a call: A→B against B→A
lock (bank) |r| { lock (r.accounts[i]) |acct| { … } }   // cells inside cells
```

"No `lock` nests" is checked lexically. Acquiring a lock needs no capability,
so "no capability in scope" does not stop nesting transitively. In the other
direction, `Shared<Conn>` where `Conn` holds `Net` puts a capability inside
the body through `x`, which allows suspension while the lock is held.

**(c) Lock waits are untyped suspensions.** A contended `lock` suspends the
task without a capability, and its syntax has no `Err(Cancelled)` path. What
happens when a task waiting on a lock is cancelled is undefined.

**(d) Scope join.** "Once a scope is cancelled, capability operations in it
return `Err(Cancelled)`." If the join at scope exit is such an operation, it
returns at once, and the parent frame is popped while children still borrow
it. That is a use after free.

**Smallest fix.**

- Check operand identity at run time; duplicate operands panic, with a
  diagnostic.
- Make `Shared<T>` authority-bearing and require `T: Pure`. The existing
  no-capability rule then forbids nesting transitively, and pure functions
  cannot lock, which agrees with B6.
- Make `lock` evaluate to `Result<R, Cancelled>`.
- Define scope-exit join as shielded and uncancellable. Accept that deadlines
  are soft for children that do not cooperate.

### B8. D5, D8, and D10: recovery at task boundaries exposes broken invariants and makes proofs unsound

**Problem.** D5 recovers from panics at task boundaries. D8 lets children
borrow the parent's frames mutably. D10 proves type invariants. Nothing stops
the parent from continuing to use data that a panicking child left half
updated:

```ferox
struct Ledger { entries: Vec<Entry>, total: i64 }   // invariant: total == sum of entry amounts
fn apply(l: &mut Ledger, e: Entry) {
    l.entries.push(e);
    l.total = l.total + e.amount;     // overflow traps after the push
}
let r = tasks.scope(|s| { s.spawn(|| apply(&mut ledger, big)); });
// r is Err(Panicked), yet the parent keeps using `ledger`; every proved
// method whose `requires` assumes the invariant now runs on a false premise.
```

A panic inside a `lock` body publishes the broken state to every task. Rust
has `UnwindSafe` and `Mutex` poisoning for this. Ferox has neither.

**Smallest fix.** Both parts are static and cheap.

- On a scope's `Err` path, the borrow checker treats every place lent `&mut`
  to the scope as moved out. The parent may only drop it or reassign it.
- A `Shared` cell is poisoned when its lock body panics, and a later `lock` on
  it returns `Err(Poisoned)`.

### B9. D4 and D11: development and release builds are not observationally equivalent

D4 says that "generic code cannot inspect concrete types, so … behave
identically". That is not enough. Four channels break the equivalence.

**(a) NaN bits.** Rust RFC 3514 makes the sign and payload of a produced NaN
nondeterministic, and says compile-time and run-time results can differ. That
covers interpreter versus LLVM, and const evaluation versus run time.
Ferox's release builds lower to Rust, so they inherit this.

```ferox
fn tag(x: f64, y: f64) -> u64 { (x * y).to_bits() }   // tag(f64::INFINITY, 0.0)
```

This breaks D11, D7's "deterministic", D9's bitwise reproducibility, and the
prover's float model.

**(b) Failures that appear only after monomorphization.**

```ferox
fn depth<T: Debug>(n: u32, x: T) -> usize {          // polymorphic recursion:
    if n == 0 { 0 } else { depth(n - 1, Some(x)) + 1 }   // fine with witnesses, infinite under monomorphization
}
impl<T> Pool<T> { const PER_PAGE: usize = 4096 / size_of::<T>(); }   // zero-sized T
```

The first is a release-only build failure. The second is a run-time trap in
development builds and a compile error in release builds. Const fuel
exhaustion also depends on the instantiation. All three contradict "instantiation never produces a type error".

**(c) `Rc` count elision is observable.** D2 promises "compiler-elided count
traffic". `strong_count`, `get_mut`, `try_unwrap`, and `make_mut` all read the
count. If the two build modes elide differently, `make_mut` copies in one and
mutates in place in the other. If elision lowers the count below the logical
count while a clone is live, `get_mut` hands out a `&mut` that aliases a live
`Rc`. That is unsound.

**(d) Stack depth and layout.** Code that uses witness tables needs more
stack, so a recursion that passes in release can overflow in development.
`Shape` exposes layout, so layout must not depend on the build mode.

**Smallest fix.**

- Canonicalize NaN at `to_bits`, at transmutes, and in hashing.
- Reject polymorphic recursion at definition time, with a check that the
  instantiation graph is finite.
- Give type-dependent constants run-time semantics in both modes, or require
  them to be total.
- Specify that elision preserves the logical count at every point where the
  count is read.
- Make layout a pure function of the type, independent of build mode.
- Define equivalence explicitly as "modulo resource exhaustion", and make
  stack overflow a task failure with its own code.

### B10. D10: proofs are unsound across mode boundaries and under aliasing, and `&mut` results cannot be specified

**(a) Proved generic code relies on trait laws that unproved impls never
established.**

```ferox
// package `algo`, mode = prove
pub fn max_of<T: Ord>(xs: &[T]) -> &T
    requires xs.len() > 0
    ensures forall |i: usize| i < xs.len() ==> *result >= xs[i]
{ … }
// package `app`, mode = assert
impl Ord for Weird { fn cmp(&self, o: &Weird) -> Ordering { Ordering::Less } }
```

The proof of `max_of` needs `Ord` to be a total order. `app` never proves
that, yet the trust surface shows zero `assume`s. Closures passed to proved
higher-order functions have the same problem.

**(b) Interior mutability.** Specs that read through `&Shared<T>` or
`&Rc<RefCell<T>>` can be changed by other code between the check and the use.
In `assert` mode, a spec that calls `RefCell::borrow` writes the borrow flag,
so evaluating the spec changes program behavior.

**(c) Returned `&mut` cannot be specified.**
`fn get_mut(v: &mut Vec<i32>, i: usize) -> &mut i32 from v` has no way to say
what `v` looks like after the caller writes through the result. That needs
prophecies (RustHorn and Creusot's `^r`).

**(d) Field-declared borrows conflict with type invariants.** A method that
borrows `&mut self.{a}` cannot assume or restore an invariant over `a` and `b`
while another method holds `&mut self.{b}`.

**Smallest fix.**

- Trait laws are proof obligations on impls. An unproved impl, or a closure
  without a spec (Verus `f.requires` and `f.ensures`), that discharges a law
  in a `prove` package counts as an `assume` in the trust surface.
- Specs may read only state reachable through owned values and `Frozen`
  references.
- Either add `final(r)` for `&mut` results, or have `prove` reject functions
  that return `&mut` or take views that contain `&mut` in v1.
- Invariants that span fields require a borrow of the whole of `self`.

## Non-blocking issues

### Memory model (D2)

1. **Elision for methods.** "Inferred when exactly one parameter is borrowed"
   forces `from self` on every `get(&self, key: &Q) -> Option<&V>`, the most
   common method shape in Rust. Adopt Rust's rule that `&self` wins. Also
   state that a view passed by value counts as a borrowed parameter, so that
   `Entry::or_insert(self) -> &mut V` elides.
2. **Union sources.** `from a, b` appears in `research/02` but not in D2.
   Specify it.
3. **"Declared views" are not declared.** The heading says declared, but the
   rule is implicit: "a struct with a reference field is a view". Viewness
   then spreads silently up the containment tree. Changing one deep field
   from `String` to `&str` turns dozens of types into views and produces
   errors far from the edit. That is the opposite of the locality principle,
   and the change is an invisible semver break. Require `view struct`. This
   also lets reviewers see mutation channels, since a by-value view holding
   `&mut` mutates the caller's data with no `&mut` in the signature.
4. **Field-declared borrows** (`&mut self.{pos}`) put private field names into
   public signatures and do not compose through traits. Add named field groups
   that a type exports without exposing the fields themselves.
5. **Higher-ranked regions.** `F: Fn(&T) -> bool`, as used by `sort_by`,
   `retain`, and `filter`, needs implicitly higher-ranked regions, because the
   callee chooses the borrow. D4's "no rank-N types" must exclude regions
   explicitly. Closures that return references from their arguments are a
   long-standing `rustc` pain point (`for<'a>` closure signatures). Specify
   that the expected type supplies the higher-ranked signature, and give a
   clear error when no expected type exists.
6. **No `impl Trait` in return position.** Closure types cannot be named, and
   D3 requires full signatures, so a function that returns an iterator
   adapter needs `Box<dyn …>`, which hides allocation and dispatch and
   conflicts with hard constraint 3. If opaque return types are added, do not
   let `Pure`, `Send`, or `Share` leak through them as Rust's auto traits do.
   Require them to be stated (`-> impl Iterator<Item = u8> + Pure`).
7. **Raw pointers and the derived auto traits.** `Send`, `Share`, and `Pure`
   are "derived structurally", and D4 forbids negative impls. Raw pointers
   must default to `!Send`, `!Share`, and `!Pure`, with `unsafe impl` to opt
   in, counted as a trust hole. Otherwise an FFI handle wrapper is
   automatically `Send`.

### Authority (D7)

8. **No mutable globals versus real programs.** The design has no answer yet
   for each of these:
   - **Logging.** A `Log` capability threaded through every caller turns one
     added log line into a signature change all the way up the call tree.
   - **Caches, interning, and lazy statics.**
   - **`HashMap` seeds.** `research/07` and `research/10` raise this; D7
     leaves it out.
   - **Pointer addresses.** Address-to-integer conversion, `{:p}` formatting,
     and hashing pointers leak ASLR into pure code.

   Recommendations:
   - Allow `static` items only of `Frozen` types.
   - Allow lazy statics whose initializer is a `Frozen` pure expression; these
     are observationally immutable.
   - Make the standard `HashMap` iterate in insertion order, so that the seed
     affects only performance and can come from the runtime.
   - Make addresses unobservable in safe code.
   - Provide a write-only `trace` sink chosen by `Root`. Show it in the
     authority summary as a coarse effect that does not break `Pure`.
9. **`dbg` as "the one untracked effect".** Its output interleaves
   nondeterministically under `par`, disappears under memoization, and gives a
   malicious dependency a covert channel to write secrets to logs. Compile it
   out of release builds instead of warning, forbid it in non-root packages,
   and route it to the simulation trace under `Root::sim`.
10. **Authority of `unsafe` and `ffi` packages is unverifiable.** Such a
    package can create a `Root` with `mem::zeroed` or make raw syscalls, and
    its computed authority would read "none". `extern` declarations should
    name the capabilities they use, and the lockfile should mark packages
    with these grants as "authority: declared, unverified".

### Concurrency and parallelism (D8, D9)

11. **The blocking runtime with `Tasks`.** If "blocking (no runtime)" runs
    spawned children inline, a library that is correct under the evented
    runtime deadlocks. One child waits on a channel that a later child fills.
    Either the blocking runtime must still multiplex, or it provides no
    `Tasks`.
12. **What `lock` rules out.** It rules out reader-writer sharing, waiting
    for a predicate (condition variables), dynamic lock sets ("lock every
    account in this `Vec`"), hand-over-hand locking, and acquisition with a
    deadline. State the replacement for each: channels, a
    `lock_all(&[cells])` with sorted acquisition and an identity check, and a
    low-level module for the rest.
13. **Cancellation cannot interrupt pure computation, lock waits, or FFI
    calls.** Deadlines are therefore soft. Destructors that need I/O during
    cancellation need shielded scopes. `research/10` included shielding, and
    the design dropped it.
14. **`ThreadBound`.** `research/10` had it; D8 does not. Objects tied to an
    OS thread, such as some FFI handles and OS mutex guards, break when a
    task migrates under work stealing.
15. **Panics in `par`.** "Results equal sequential execution" must specify
    which panic wins. The sequentially first one should, which means a right
    branch's panic has to wait for the left branch.
16. **`par_reduce` is not a fold.** With a fixed reduction tree, `par_reduce`
    and `fold` differ for non-associative operations such as float addition.
    The sequential semantics is the tree, so proofs need an associativity
    lemma to equate the two.

### Contracts (D10)

17. **Executing specs in `assert` mode.**
    - `old(e)` must be evaluated at entry. Deep snapshots such as
      `old(v@)` cost O(n) per call.
    - Specs that mention a parameter after it has been moved cannot be
      executed without `Clone`.
    - Quantifiers over unbounded `int`, `Seq`, `Set`, or `Map` cannot be
      executed at all.

    Mark such specs as prove-only, with a diagnostic, rather than silently
    weakening the claim "one spec, three uses".

### Traits and codegen (D4, D11)

18. **The orphan rule's unit.** D4 puts impls in "the module that defines the
    type or the trait", which is stricter than the package-level coherence
    that `research/08` recommended. A `Display` impl could not live in a
    sibling module. Pick one rule and justify it.
19. **`Shape` versus "generic code cannot inspect concrete types".** Generic
    code can branch on `T`'s shape or name, which is a library-level
    specialization channel. Equivalence survives as long as `Shape` values do
    not depend on the build mode, but the stated justification is wrong, and
    shape-based dispatch is a semver hazard.

## Strengths worth keeping

- **Second-class references plus views, framed as a restriction of Rust's
  model.** This is the right shape. Once B1 to B4 are written down, soundness
  can be inherited from Rust's model by translation rather than re-proved,
  and D12's Rust emission turns `rustc` into a permanent oracle.
- **Structured, stackful tasks with no cancel-by-drop.** Because the scope API
  takes a closure and runs on stackful tasks, scoped borrowing is sound
  without Rust's unfinished "guaranteed destructors" work, given the
  uncancellable join in B7(d). Returning consumed values inside errors closes
  the Tokio `send` hazard.
- **Capabilities as values, `Root` only in `main`, and `Root::sim`.** Once
  synchronization handles are classified as in B6, this gives genuinely
  type-level determinism and makes deterministic simulation testing a
  property of the language.
- **`lock` as atomic multi-acquire with no capability in the body.** It is a
  good primitive. With the B7 fixes it rules out lock-order inversion, double
  locking, and holding a lock across a suspension, which together are
  37 of the 38 lock hangs in the Rust bug study.
- **Definition-checked generics** with no overlap, no specialization,
  Paterson termination, and flat associated types. This is the right ceiling.
  Keep it, and close the post-monomorphization holes in B9.
- **`par` restricted to pure closures with fixed reduction trees.** This is
  the right way to inherit Bend's idea without its runtime.
- **`?` converting only through `from` variants declared on the destination
  type.** It makes error conversion local and visible.
- **Contracts erased from release builds, opaque bodies by default, and a
  pinned solver with seed-stability checks in CI.** This is the discipline
  Verus users learned the hard way.
- **Checked arithmetic in every profile, with no `as`.**
