# Round 2 Review: Simplicity

Reviewer: language minimalist (Go and Gleam school) who runs coding agents
daily. Every feature must pay rent to the thesis.
Date: 2026-10-19.
Read in full: `design/00-first-principles.md`, `design/01-design.md`
(draft 2.1), `trials/e1/notes.md`. Skimmed for size, structure, and the
sections on each feature named below: `design/02-memory.md`,
`design/03-authority-concurrency.md`, `design/04-contracts.md`,
`design/reference.md`, `reviews/round1-resolution.md`,
`reviews/round1-strategy.md`, and `trials/e1/programs/bank.fx`.

Token figures are characters divided by 4. For code-heavy text that
undercounts, so the real counts are somewhat higher.

## Verdict

**request_changes**

The thesis is right, and most of the everyday surface is right. Authority in
signatures, whole-program deterministic simulation, a computed trust diff,
Rust spelling, no async, no user macros, and no build scripts are a strong
core. Rust cannot copy that core, because each part is a subtraction.

The specified v1 is not that core. Review round 1 closed every hole by adding
a rule and never by removing a feature. As a result, draft 2.1 is Rust, plus
a Verus-class verifier, plus a second borrow-annotation language, plus a
lock calculus, plus a parallel-reduction specification with hard-coded
constants, plus a Bend RPC protocol, plus a Rust symbol scanner. On concept
count it is heavier than Rust, not lighter. The cuts below remove about half
of the normative text and about half of the questions the E1 agent could not
answer, and they leave all five thesis pillars standing.

Short answers:

1. **Has complexity betrayed "lighter than Rust"? Yes.** By my count Ferox
   adds about 39 concept clusters to Rust and removes about 12, and only 4 of
   the 12 removals are large (see the ledger). Draft 2.1 is lighter only on
   the three axes where Rust hurts most: async, macros, and build scripts.
   Lifetimes are respelled, not removed.
2. **What to cut:** the `prove` mode and everything that exists only for it
   (`final`, views, spec types, ghost code, laws). Also `Frozen`, the `lock`
   keyword, `par {}`, `par_reduce` and its grouping constants, field-borrow
   declarations, `Shape`, the Bend boundary, exercised-authority tracking,
   `#[trusted_pure]`, `#[pure]`, the moved-out rule, and the evented
   runtime. Shrink Encode/Decode to JSON. Make `from` an E1 experiment arm
   instead of the default.
3. **Minimal v0:** Rust's surface and ownership, the capability set, one
   new auto trait (`Pure`), scopes, a `Shared<T>` method API, library
   fork-join, a simulator with crashes, executable `requires`/`ensures`, a
   declared-authority trust diff, per-dependency grants, and the agent
   toolchain. The lists are below.
4. **Missing for "tremendous":** make the simulator the debugger. That means
   production record and replay through the capability boundary, traces that
   agents can query, snapshot-and-branch state exploration, authority
   policies on module paths, and source porting of Rust crates.

## How the complexity got in

- **No subtraction step.** `reviews/round1-resolution.md` grades each
  objection `resolved`, `partial`, `deferred`, or `unaddressed`. It has no
  status for "feature removed". Each specialist reviewer (soundness,
  verification, performance, usability) closed a hole by adding a rule, and
  no reviewer was paid to delete anything.
- **Round 1 asked for these cuts, and draft 2.1 expanded the features
  instead.** `round1-strategy.md` "Cuts for v0" deferred `prove`, the `lock`
  primitive, field-borrow methods, and deterministic float-reduction trees.
  Each of these now has a normative chapter section: LOCK-1 to LOCK-8 plus
  `lock_all`; `par_reduce` with B = 2048 and 8 strided lanes fixed by the
  language version; `final` prophecies under RustHorn semantics.
- **The budget does not bite.** The 25,000-token budget covers only
  `reference.md`, so complexity moves into the chapters. Sizes:

  | Document | ~Tokens |
  | --- | ---: |
  | `01-design.md` | 9.4k |
  | `02-memory.md` | 14.0k |
  | `03-authority-concurrency.md` | 15.6k |
  | `04-contracts.md` | 14.3k |
  | Normative semantics beyond Rust | **~53k** |
  | `reference.md` (the delta from Rust) | 18.6k, 74% of budget |

  By my count, E1 then found about 55 open questions and unspecified APIs,
  including persistence, crash simulation, std paths, origins on enums, and
  contracts on generic code. The reference cannot absorb those answers
  within budget unless something leaves.
- **The principle drifted.** `00-first-principles.md` says the language must
  be "closer to Go, Lua, or Gleam in size than to Rust". It is now "all of
  Rust plus a 25k-token delta". Borrowing Rust's priors is a legitimate
  strategy. It means "lighter" can be measured only as concepts removed
  minus concepts added, and that number is negative.
- **Drift already shows.** The single `lock` primitive has three
  incompatible specifications across four documents written on the same
  day:
  - the reference says aliased handles panic and `lock` returns
    `Result<R, Poisoned>`;
  - the memory chapter (§8.4) says `Result<R, Poisoned>`;
  - D8 and the authority chapter (LOCK-2, LOCK-5) say `Err(Aliased)` and
    `Result<R, LockError>`.

  `Handle<T>` names the task handle in the memory chapter (§8.2) and the
  arena handle in the reference (§3.7). The memory chapter's §9 then lists
  `Handle<T>` as neither `Pure` nor `Frozen`, so a reader can conclude that
  arena code is not authority-free. Spawn has two different signatures. Too
  many interacting rules produce exactly this kind of drift, and agents read
  every one of these documents.

## Complexity ledger

### Removed relative to stable Rust

| Removed | Weight for agents |
| --- | --- |
| `async`/`.await`, `Future`, `Pin`, executor choice, `Send`-future bounds, `select!` | **large**: Rust's worst agent failure class |
| `macro_rules!` and proc macros, both writing them and calling crate macros | **large**, though 23 built-in `!` forms keep the common spellings |
| Ambient I/O, mutable statics, `thread_local!`, `OnceLock`/`lazy_static` globals | **large**, and the thesis needs it |
| `build.rs` and install hooks | **large** for supply chain, small for agents |
| Lifetime syntax (`<'a>`, `'a: 'b`, `T: 'a`, `for<'a>`) | **respelled**, not removed (see `from` below) |
| GATs; HRTB beyond elision | small: rare in application code |
| Debug versus release overflow divergence | small: one behavior instead of two |
| Lossy `as` | small: replaced by named methods |
| `catch_unwind` | small |
| `Mutex`/`RwLock` guards and manual lock ordering | medium: replaced, not removed |
| `mod foo;` declarations, `#[path]` | small |
| `Pin` outside async | small |

D3 and D4 also ban specialization, HKTs, negative impls, user auto traits,
and overloading. None of these is in stable Rust (Rust has no
overloading either), so banning them removes nothing an agent knows.

### Added: needed for ordinary code

1. `Root`, nine capability types (`Fs`, `Net`, `Clock`, `Rand`, `Env`,
   `Stdin`, `Proc`, `Tasks`, `Accel`), the `Root` naming rule, attenuation
   methods
2. Capability threading, the app-context pattern, and lint `FX0510`
3. `Root::sim`, `SimRoot`, `sim.world`, `Faults`, `Reply`, property tests
   with a `seed` parameter, and `#[sim_test]`
4. The `from` grammar, which has about eight positional forms: parameter,
   several parameters, place path, `*p`, `Self.f`, inside generic arguments
   with parentheses, on fields, and on `dyn`/`impl`. Also per-position
   origin names (`*out`), `T from { name: place }`, and `Owned`
5. Field-borrow declarations, `&mut self.{a, &b}`
6. Structured scopes: `Scope<E>`, `E: From<TaskError>`, deadlines,
   level-triggered cancellation
7. `Shared<T>`, the `lock (a, b) |x, y| {}` syntax, `LockError`, body
   restrictions, `reset`, and `lock_all`
8. `Pure`
9. `Frozen`
10. Write-only sink rules and `root.log`
11. Overflow traps in release; `truncate()`, `round_from`, and
    `cast_unsigned`
12. Impl placement, no module cycles, and unused imports as errors
13. An opaque `impl Trait` exposes only the auto traits it names
14. The closed derive set, `#[derive(Error)]`, `Encode`/`Decode`, and
    `#[codec(...)]` with 7 keys
15. The moved-out rule after a child failure (`FX0703`)
16. `Arena`/`Handle` as the graph idiom
17. Deterministic `HashMap` iteration
18. `ThreadBound` and the runtime choice

### Added: opt-in or advanced

19. `par {}`, `join`, `par_map`, `par_for`, `par_reduce`, `Combine`,
    `Monoid`, and the canonical grouping G
20. `requires`, `ensures`, `invariant`, `decreases`, `diverges`
21. `result`, `old`, `final` (a prophecy)
22. Spec types `int`, `nat`, `Seq`, `Set`, `Map`, `Multiset`, `spec_fn`;
    `==>`, `<==>`, `forall`, `exists`, `choose`; model `==`
23. `spec fn`, `proof fn`, `law fn`, `#[spec]`, `ghost let`, `proof {}`, and
    the proof statements `assert`, `assume`, `reveal`
24. The `View` trait, `x@`, inferred closed spec fns
25. Three contract modes and three clause classes
26. Type invariants
27. Trait laws, abstract spec fns (`eq_spec`, `cmp_spec`), and refinement
28. Closure contracts (`f.requires`, `f.ensures`)
29. `#[trigger]`, `by(nonlinear)`, `by(bit_vector)`, `#[opaque]`
30. `#[trusted]`, `#[trusted_pure]`, `#[pure]`, `#[protected]`,
    `#[spec_test]`
31. Proof status, trust closure, and signed approvals
32. `Shape`
33. `Secret<T>`
34. Trust holes and the trust diff
35. Per-dependency grants and lockfile authority
36. Rust crate import: `authority`, symbol scan, `build = true`
37. Bend: `[bend-dependencies]`, `#[bend]`, `Accel`, `BendArray`,
    `BendNat`, `BendFault`, `by bend_law`
38. Typed holes with `_?name`
39. `ferox fix --thread`

### Counts that matter

| Kind | Draft 2.1 | After the cuts below |
| --- | ---: | ---: |
| New keywords, contextual keywords, and operators | 21 keywords (`from`, `lock`, `par`, and 18 contract words) plus `==>`, `<==>`, `@`, `_?` | 4 (`requires`, `ensures`, `result`, `old`) |
| New attributes | 15 | 5 (`error`, `from`, `source`, `codec`, `protected`; the first three are thiserror's) |
| New auto traits | 3 (`Pure`, `Frozen`, `Owned`) | 1 (`Pure`) |
| Capability types, including `Root` | 10 | 9 |
| Normative text beyond Rust | ~53k tokens | ~25k (estimate) |
| Reference | 18.6k tokens, before filling E1's gaps | ~14k before filling E1's gaps (estimate) |

**Bottom line.** For ordinary code the delta is about 18 concepts and pays
for the four large removals, so it is a fair trade. For loading, reviewing,
and building the language it is not lighter than Rust: the agent loads all
of it, the reviewer reads proof statuses and trust closures, and the
implementers build several research systems. The pillars do not need most
of the opt-in half.

## Cut or defer list

Each entry gives a verdict, the reason, and what is lost. "Cut" means it
leaves the design. "Defer" means it leaves v0 and returns only if an
experiment asks for it.

### Memory

**`from` origins: defer as the default, and run them as an E1 arm.** Use
the origin-equivalent subset of Rust's lifetime syntax in v0.
- *Reason:* `02-memory.md` §11.1 shows that origins translate exactly to
  Rust lifetimes in every position. D1's own rule is "Rust syntax is Ferox
  syntax for every construct with the same meaning", and by that rule Ferox
  should spell them `'a`. The subset is lifetime parameters on structs,
  enums, fns, and impls, plus elision, with no outlives bounds, no trait
  lifetime parameters, and no `for<'a>` beyond elision. That is exactly what
  `ferox fmt` already accepts as a "legacy spelling".
- *Evidence from E1:* the agent's third-ranked gap is origins, with three
  separate questions: tuple variants have no field names to serve as origin
  names; `from` is ambiguous inside a generic argument list; `from self` and
  `from *self` mean different things. Each of these is a non-question in
  Rust syntax (`Token<'a>`, `Result<Token<'a>, E>`, `&'a str`).
- *Locality cost:* `02-memory.md` §12, item 5, admits that a new reference
  field silently adds origins to every containing type, far from the edit.
  Rust's explicit `<'a>` is local.
- *Lost:* writing `struct Lexer { src: &str }` without `<'a>`, and the
  place-path sugar. The reference's 2.7k-token §3 shrinks to about 0.6k.
  `Owned` goes too, because `'static` covers it. The origin cross-check
  becomes an identity translation.

**Field-borrow declarations (`&mut self.{pos}`): defer.**
- *Reason:* new syntax with no model priors. It affects API stability
  (adding a field is a breaking change), interacts with invariants (02 §7),
  and adds frame conditions to contracts (04 §4.1). Rust is exploring view
  types, so the wait costs nothing. Open Question 2 in `01-design.md`
  already says the design is unsure.
- *Lost:* disjoint-field method calls. The workarounds (split the struct,
  pass fields to free functions) are what Rust agents already write.

**The moved-out rule after a child failure (`FX0703`): cut.** Use Rust's
semantics instead.
- *Reason:* it is a new borrow-check rule that exists only because a panic
  in a scoped child becomes a value. Rust's `thread::scope` re-raises the
  panic in the parent, so the parent never sees half-updated borrowed data.
- *v0 rule:* a panic in a scoped child propagates. For isolation (servers),
  `s.spawn_owned(f)` takes only `'static` captures and its `join` returns
  `Result<T, Panicked>`, like `thread::spawn`. With no borrowed data, there
  is nothing half-updated to observe.
- *Lost:* catching panics of borrowing children as values.

### Authority and purity

**`Frozen`: cut.** Fold it into `Pure` plus Rust's `Sync`.
- *Reason:* `par` already requires `Send` closures, and capturing `&T`
  requires `T: Sync`. `Cell`, `RefCell`, `Rc`, and `OnceCell` are not
  `Sync`. Atomics are `Sync` but already not `Pure`.
- *One rule replaces the trait:* a `Sync` type whose shared-reference API
  can observe or cause mutation (atomics, `OnceLock`, raw locks,
  `sync::Weak`) is a synchronization handle and is therefore not `Pure`.
  Then `Pure + Send` is exactly what `par` needs, and statics need
  `'static + Sync + Pure`.
- Memoization has no user-facing feature in the design, and Rust's
  `const fn` already forbids interior mutation.
- *Lost:* nothing I can find, provided std classifies its `Sync` cells.

**`#[pure]`: cut.**
- *Reason:* a function is authority-free when its parameter types are
  `Pure`, and the signature already shows that. Adding authority means
  adding a capability parameter, which is already a signature change and
  already in the trust diff. E1 shows the cost: the agent put `#[pure]` on
  two programs because it was in the reference, which adds entropy for no
  checked benefit.
- *Lost:* a marker for generic code whose purity depends on its
  instantiation. Write `F: Pure` instead.

**`#[trusted_pure]`: cut.** Compute the fact instead.
- *Reason:* a safe wrapper around foreign code already needs an `unsafe`
  block, the `ffi` grant, and a `// SAFETY:` comment, and the unsafe block
  is already a trust hole. The trust surface can list "contains unsafe,
  takes no capability, so it claims to be authority-free" automatically.
- *Lost:* one explicit `reason =` string. The SAFETY comment carries the
  reason.

**Exercised-authority tracking and `FX0510`: defer.** v0 tracks declared
authority only.
- *Reason:* declared authority (the capability types reachable from the
  parameters) is the thesis, "read the envelope from the signature". The
  exercised analysis (03 §4.1) has six attribution rules, closure and `dyn`
  attribution sites, fixpoints over recursion, and sink effects. It refines
  the envelope, and E6 is the experiment that shows whether bundles defeat
  review. Build it if E6 says so.
- *Lost:* the least-authority lint, and visibility when a method begins
  using one field of an app bundle it already held.

**`Secret<T>`: defer.** It is a library type. Add it after v0.

**`Shape`: cut from v1.**
- *Reason:* nothing in the five pillars uses it. `Debug` covers printing,
  and `Encode`/`Decode` cover serialization. It already needed a soundness
  patch (forging capabilities, round-1 S-B5), so it has cost before having
  any user.
- *Lost:* generic inspection and diffing tools. Add a `Shape` derive later
  if a tool needs one.

### Concurrency and parallelism

**The `lock` keyword: cut.** Use a method API on `Shared<T>`.
- *Mechanism:* `cell.with(|x| ..)`, `Shared::with2(&a, &b, |x, y| ..)`, and
  `sync::with_all(&cells, |xs| ..)`, where each closure is bounded by
  `FnOnce(..) -> R + Pure`.
- *Every guarantee survives as a bound.* A `Pure` closure cannot capture a
  capability or a `Shared` handle, and payloads are `Pure`, so the body
  cannot do I/O, suspend, or nest another lock. Global acquisition order
  stays in the library. `return` and `?` already leave a closure in Rust,
  so the "the body behaves like a closure" rule becomes simply true.
- *Errors:* aliased handles are a bug, so they panic, as an out-of-bounds
  index does. The result is `Result<R, Poisoned>`. That gives one error,
  not a `LockError` enum, and it ends the three-way drift.
- *Lost:* a contextual keyword and arbitrary arity in surface syntax.
  `with2` plus `with_all` cover the uses in E1 (bank needed `with_all` for a
  consistent snapshot, and the reference did not offer one).

**`par { }` block syntax: cut.** `join(|| a, || b)` does the same thing
with no new syntax.

**`par_reduce`, `Combine`/`Monoid`, and the canonical grouping G: defer.**
- *Reason:* the language version fixes B = 2048 and an 8-lane strided
  fold, and the feature pulls in laws, `Sum<f64>` caveats, and overflow
  differences against a sequential fold. E1's word_freq raised six
  questions about it.
- *v0 replacement:* determinism with zero rules:
  `xs.par_chunks(n).map(f).collect::<Vec<_>>()` and then a sequential fold.
  The user picks `n`, so the grouping is explicit and reproducible.
- *Lost:* a one-call parallel reduce. This is also Bend's ground, and the
  owner does not want to compete there.

**Evented runtime, stackless lowering, the "may suspend" analysis,
`ThreadBound`, and the `blocking` runtime: defer.** The Positioning section
of `01-design.md` already says the evented runtime follows. Write that down
as the v0 boundary, and with it remove the task-isolation soundness risk
(memory chapter §11.2, R1) from v0. v0 ships `threads` and `sim`.

**Three simulation engines with conformance tests: defer.** v0 has one
engine, baton-passed threads or the interpreter, whichever ships first.

### Contracts

**The `prove` mode and everything that exists only for it: defer to
post-v0.** If proofs return, the route is emitting Verus
(`round1-strategy.md` Bold 5), not a native verifier.
- *What goes:* SMT; total correctness; `decreases`/`diverges`; `final`;
  `View` and `@`; the spec types; `spec`/`proof`/`law fn`; `#[spec]`; ghost
  code; `forall`/`exists`/`choose`/`==>`; triggers and `by(..)`; laws and
  abstract spec fns; refinement obligations; closure contracts;
  `#[trusted]` and `#[opaque]`; solver discipline; trust closure; proof
  caches; and proof-based deletion of checks.
- *Reason:* the thesis names authority, simulation, and the trust surface.
  The thesis section of `01-design.md` says contracts "support the thesis;
  they are not the headline", and D16 gives proofs to Bend. This is the
  largest single block of text (04 is 14.3k tokens) and the largest
  implementation effort (Verus took years at MSR).
- *Evidence from E1:* the agent's second-ranked gap (eight questions in
  bsearch alone) was the semantics of `==`, `<`, laws, and panics in specs
  over generic `T: Ord`. None of those questions exists if a spec is an
  ordinary executable Rust boolean expression.
- *Lost:* level 4 of the trust ladder, "proved on all inputs". That is
  real, but no pillar depends on it, and adding it later is
  backward-compatible.

**Three contract modes and three clause classes: collapse to one
behavior.** `requires` and `ensures` are ordinary boolean expressions over
authority-free calls. They are checked in debug and test builds, erased in
release, and `requires` filters property-test inputs. Quantifiers are
`(0..n).all(|i| ..)` and implication is `!a || b`, both of which models
already write. No manifest setting is needed.
- *Lost:* the `checked` and `sampled` distinction, and per-package mode
  policy.

**`final`: cut together with `prove`.** Without prophecies, `*x` in
`ensures` simply means the value at return, as a Rust reader expects, and
`old(*x)` means the value at entry. Error `FX0801` goes away.

**Laws: defer.** Their only consumers are `prove`, `par_reduce`, and
generic proofs, and all three are deferred. A law tested under `test` mode
is just a property test, which users can already write.

**Type and loop `invariant`, spec mutation testing, the spec-quality checks
`FX0810`–`FX0812`, and signed approvals: defer.**
- In v0, approval is a CI rule over the trust diff, in the style of
  CODEOWNERS.
- `#[protected]` stays, because it costs one attribute and is the cheapest
  anti-gaming rule there is.

### Surface, interop, and toolchain

**`Encode`/`Decode`: keep, and shrink.**
- *Reason:* without macros there is no serde, and CLI and infrastructure
  code reads JSON and config files on day one. Without it, E2's comparison
  with Rust plus serde would be unfair.
- *Shrink:* support JSON only, with `rename`, `rename_all`, `default`, and
  `skip`. Defer `flatten`, `deny_unknown_fields`, `try_from`, and the
  format plug-in API.
- *Lost:* other formats until the plug-in API settles.

**The Bend boundary (D16, 03 §10, `Accel`, `by bend_law`): cut from the
design.** Keep one positioning sentence. A Bend kernel can run today through
`Proc` as a child process, with no language support.
- *Reason:* a typed RPC protocol, a type map, a capability type, and
  imported proof claims, all for a pre-1.0 external language. Not competing
  with Bend means staying out of proofs and massive parallelism. It does not
  mean building a bridge to Bend.
- *Lost:* typed kernel calls and claim import. Add them back when Bend
  ships a library target.

**Rust crate import (generated shims, symbol scan, sandboxed `build.rs`,
libc interposition): defer.** v0 has C FFI under the `ffi` grant. The
better ecosystem path is source porting (see Missing big ideas).
- *Lost:* calling synchronous Rust crates on day one.

**Typed holes spelled `_?name`: cut.** `todo!()` already works as a typed
hole and agents already write it.

**No import cycles between modules: relax to no cycles between packages.**
Rust allows module cycles within a crate, and so does Go within a package,
so file-level acyclicity is a Rust-habit tax that no other language
imposes. Incremental checking can work on strongly connected components.
- *Lost:* a small amount of build parallelism.

**The origin cross-check: keep.** It costs agents nothing, and it is
insurance on the riskiest component. With Rust lifetime syntax it becomes
nearly free.

### Keep: these pay rent

- capabilities and `Root`, with attenuation
- no globals and no ambient std I/O
- write-only sinks
- deterministic `HashMap` iteration
- overflow traps and lossless `as`
- the closed derive set and `#[derive(Error)]`
- the fixed `!` forms and no build scripts
- impl placement
- the non-leaking opaque `impl Trait` (it serves fast builds)
- must-use `Result`
- structured scopes and cancellation
- channels
- `Arena`
- the simulator chapter (03 §9), which is the best part of the design
- the trust diff over declared authority and holes
- per-dependency grants
- NDJSON diagnostics with safe fixes for Rust habits
- `ferox fix --thread` (it directly reduces the E6 plumbing risk)
- `ferox eval` and `ferox fmt`

## Minimal v0

The goal is the smallest language that can run E1, E2, E3, and E6 on the
interpreter and then E4 on one native development backend.

### In v0

1. **Surface.**
   - Rust 2024 syntax for items, expressions, patterns, closures,
     iterators, traits, generics, and modules, with D4's power ceiling.
   - The fixed `!` forms; no user macros.
   - Impl placement; module cycles allowed within a package.
2. **Memory.**
   - Rust ownership, borrowing, NLL, drop check, and elision.
   - Rust lifetime syntax limited to the origin-equivalent subset.
   - `Box`, `Rc`, `Arc` (without count observers), `Cell`, `RefCell`, and
     `Arena<T>` with generational handles.
3. **Types.**
   - Overflow traps in every build.
   - Lossless `as` with safe fixes; canonical NaN bits.
   - Must-use `Result`.
   - `#[derive(Error)]`; the std derives; JSON-only `Encode`/`Decode`.
4. **Authority.**
   - `main(root: Root)` and `Root` usable only in `main`.
   - `Fs`, `Net`, `Clock`, `Rand`, `Env`, `Stdin`, `Proc`, `Tasks`.
   - Attenuation with `Fs::dir`, `Fs::read_only`, and `Net::only`.
   - No ambient std I/O, no mutable globals, no thread-locals.
   - `println!` and the logging macros as write-only sinks.
5. **Purity.** One auto trait, `Pure`, with the rule that `Sync` interior
   mutability makes a type a synchronization handle.
6. **Concurrency.**
   - `Tasks::scope`, `spawn`, `join`, `spawn_owned`, deadlines, and
     level-triggered cancellation.
   - `mpsc` channels plus a bounded multi-consumer work queue (E1's
     fetch_all had no way to cap concurrency).
   - `Shared<T>` with `with`, `with2`, and `with_all` over `Pure` closures,
     and poisoning.
7. **Parallelism.** `join`, `par_map`, `par_for`, and
   `par_chunks(_mut)` as library functions with `Pure + Send` closures.
8. **Runtimes.** `threads` and `sim`.
9. **Simulation.**
   - `Root::sim(seed)`, virtual time, and deadlock reports.
   - Network faults and partitions, disk errors, and crash and restart with
     the line between synced and unsynced data and torn writes (03 §9.4–9.5;
     the reference omits all of this, which is why E1's kv_log had to
     invent it).
   - Nodes; `ferox test` over seeds; `ferox sim`; replay from seed and
     commit; the determinism self-check; the list of unmodeled effects.
   - Fs append and open options, plus `sync_all` as an `Fs` operation.
10. **Contracts.**
    - `requires` and `ensures`, with `result` and `old()`, as executable
      checks.
    - `#[test]` functions with generated, shrunk parameters, and `requires`
      filtering those inputs.
11. **Trust surface.**
    - Per public item: the signature, declared authority, contract text,
      and holes (`unsafe`, `extern`, grants).
    - `#[protected]` tests.
    - `ferox review <base>`.
    - A lockfile that records public authority and holes; any widening
      fails CI until a human approves it.
12. **Packages.** Per-dependency `unsafe` and `ffi` grants, no build
    scripts, C FFI only, and a lockfile with hashes.
13. **Toolchain.**
    - `ferox check` with NDJSON output and safe fixes for Rust habits.
    - `todo!()` holes.
    - `ferox fix --thread`, `fmt`, `eval`, `explain`, `test`, `sim`,
      `review`.
    - Snapshot hashes.
    - The interpreter first, then one Cranelift-class development backend
      for E4.

### Explicitly deferred

- `from` origins (an E1 arm), field-borrow declarations, `Owned`, and the
  moved-out rule
- `Frozen`, `#[pure]`, `#[trusted_pure]`, `Shape`, and `Secret<T>`
- the `lock` keyword, `LockError::Aliased`, `par {}`, `par_reduce`,
  `Combine`/`Monoid`, and the grouping G
- the `prove` mode with all ghost and spec machinery, `final`, laws, type
  and loop invariants, `decreases`/`diverges`, the three modes, the clause
  classes, and the spec-quality and mutation checks
- exercised authority, `FX0510`, sink-effect tracking, and signed approvals
- the evented and blocking runtimes, stackless lowering, `ThreadBound`,
  and the second and third simulation engines
- the PCT scheduling strategy and `--minimize`
- Rust crate import and the Bend boundary with `Accel`
- the C release backend, the witness-table fallback, and the
  cross-package instantiation cache
- `_?name` holes
- the codec plug-in API and the formats beyond JSON

**What this v0 still proves.**
- *Authority in signatures:* E3 and E6.
- *Deterministic simulation:* the replicated key-value demo that
  `round1-strategy` proposed.
- *Trust surface:* E3.
- *Rust-familiar surface:* E1. There is nothing new to learn about
  lifetimes, and four contract words.
- *Fast builds:* E4 on the checker and the Cranelift backend.

None of the deferred items is needed to falsify any of these.

## Missing big ideas

"Rust but lighter, with fast builds" is incremental, though valuable. The
potential becomes tremendous where Ferox can do something no mainstream
language can. The capability boundary is a complete record of a program's
nondeterminism, and the design uses it only for tests.

1. **Production record and replay.** Add a `record` runtime that logs every
   capability result and the order of every synchronization operation (the
   scheduling points in 03 §9.3, with real values). Then
   `ferox replay <trace>` reproduces a production incident bit for bit in
   the interpreter. Today that takes rr, Linux, and kernel support. In Ferox
   it falls out of a design decision already made. This is the feature that
   makes infrastructure teams switch: every bug becomes a deterministic
   repro an agent can work on.
2. **Traces agents can query.** Because replay is deterministic, agents can
   ask questions such as `ferox replay --at <event> --eval <expr>` or
   `ferox why <assertion>` (which capability results and writes flowed into
   this value). Agents drive interactive debuggers badly but run queries
   well, so this takes the "signal quality" parameter of the agent loop
   (`00-first-principles.md`) to its limit.
3. **Snapshot-and-branch exploration.** The interpreter can snapshot a whole
   simulated world cheaply, because no OS state is involved. A
   coverage-guided explorer can branch from interesting states and choose
   faults, schedules, and inputs. This is what Antithesis sells at the
   hypervisor level, and Ferox gets it from the language. Random seeds find
   shallow bugs; branching finds the deep ones.
4. **Authority policies on module paths.** A project file such as
   `src/core/**: authority-free` or `src/store/**: fs only` is checked by the
   compiler against declared authority. Architecture becomes a checked
   artifact, and reviewers approve changes to the policy instead of reading
   code. It is cheap, because declared authority already exists. It turns
   the trust surface from a diff into an invariant.
5. **Source porting as the ecosystem.** Round 1 proposed agent porting
   (`round1-strategy.md` Bold 4). Keeping Rust's lifetime syntax makes it
   mostly mechanical. A `ferox port` command would apply the habit table and
   leave agents only the ambient-I/O edits. A ported crate gets checked
   authority. A shimmed crate is an opaque trust hole. This replaces the
   symbol-scan machinery with something that strengthens the thesis.

## Process recommendation

- **Rent rule.** Each feature in `01-design.md` names the pillar it serves
  and the experiment (E1–E6) that would show it is unnecessary. A feature
  that names neither goes to a deferred appendix.
- **A budget on semantics, not only on the reference.** Cap the normative
  text beyond Rust, for example at 30k tokens for v0, so complexity cannot
  hide in the chapters.
- **A `cut` status in resolution audits.** Future rounds may close an
  objection by deleting the feature that causes it. In round 1, that would
  have closed several soundness and verification rows outright: `Shape`,
  `final`, the `lock` payload holes, and memoization.
