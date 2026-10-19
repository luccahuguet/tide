# 02: Memory-safety and aliasing models

Research note, 2026-10-19. Tags: **[E]** marks a claim backed by the numbered source; **[I]** marks my own inference. The session's web-search quota ran out partway through, so a few systems (OxCaml's papers, Verona's 2025 follow-ups, Scala capture checking) are covered from documentation pages and abstracts rather than full papers, and some secondary claims are flagged as unverified.

## 1. Question

Can a new language keep Rust's guarantees (memory safety, data-race freedom (DRF), no GC pauses, predictable performance) while meeting three more goals?

- (a) Much less annotation and cognitive load.
- (b) Fast checking.
- (c) LLM agents write correct code on the first or second try.

Which existing models come closest, what did each one pay, and what should the new language adopt?

## 2. Findings

### 2.1 Rust: where the baseline is heading

- The core discipline is "mutation xor sharing", which gives both memory safety and DRF [1]. The 2026 roadmap "The Borrow Checker Within" says the remaining problem is false rejections: "it's the *compiler* that's limited, not their mental model". Some patterns have no workaround at all: lending iterators are "simply inexpressible without Polonius" [1]. [E]
- Polonius Alpha (flow-sensitive outlives checking) became the nightly default in August 2026, and stabilization is targeted before the end of 2026 [2]. [E]
  - It accepts conditional returns of borrows (the `get_mut_or_default` case that NLL rejects).
  - It still rejects some linked-list traversals that "we *want* to compile".
  - It "does strictly equal or more work compared to NLL", and the worst case seen is a 2–3x regression on borrow-heavy crates.
- Several features were still unshipped in 2026 [1]: [E]
  - view types (disjoint field borrows across method calls), which get a rustc experiment in 2026, with "strong updates" still an open question;
  - place-based lifetimes (`'self.text`);
  - internal (self-referential) references, which are formalized only for a simplified Rust.
- Ref-counting ergonomics became an accepted 2026 goal: a `Share` trait for handle-like clones and `move($expr)` captures. The motivation is that cloning several `Arc`s before every `spawn` is a recurring pain. The earlier `.use` proposal (RFC 3680) drew mixed feedback [3]. [E]
- Defining what unsafe code may do with aliases took years. Tree Borrows (PLDI 2025, distinguished paper) rejects 54% fewer test cases than Stacked Borrows across the 30,000 most-used crates [4]. [E]
- **Pain data.**
  - The 2024 State of Rust survey puts compile times at the top of productivity complaints and "too complex" among the top worries. The borrow checker is not singled out [5]. [E]
  - The 2025 compiler-performance survey (3,700+ responses) found that about 55% of respondents wait more than 10 s for a rebuild, and about 45% of former users cite compile times as a reason they left. The bottlenecks it names are workspace rebuilds, linking, and uncached proc-macro expansion; borrow checking does not appear [6]. Borrow checking runs on MIR [7]. [E]
  - **[I]** Borrow checking is not where Rust's compile time goes. The cost of Rust's model is cognitive and iterative (extra edit–compile loops), not CPU time.
- The Rust roadmap claims that explicit lifetime syntax does not make Rust harder to learn ("Counterintuitively, no."), and that `'map` reads better than "the lifetime of the borrow created on line 12" [1]. [E] This is the Rust team's position, not a measured result.

**LLM evidence.** Rust is the only language in this survey with real data.

- **ISSRE 2026 study [8].** The study covers 86,726 failing samples from seven models (Qwen2.5 7–14B, DeepSeek-Coder-v2, GPT-4.1-mini, Gemini-2.0-Flash, gpt-oss-120b) on CodeNet contest problems. [E]
  - Rust had the most compile failures: 23,416, against 9,953 for C++.
  - Under baseline prompting, "ownership and lifetime" errors were 16.7% of Rust compile errors. Ahead of them were incompatible parameter types (43.4%) and missing imports (20.5%).
  - With compiler feedback, only 12.9% of ownership/lifetime errors were fully fixed (53.5% partially). That rate is aggregated across languages, but the category is essentially Rust-only.
- **RustAssistant (Microsoft) [9].** With GPT-4 and up to six compiler round trips, it fixed roughly 74% of real-world compile errors from popular repositories. Its Stack Overflow set was chosen for memory- and thread-safety questions, and 94% of those errors involved lifetimes, ownership or traits. That 94% reflects how the set was picked, not how often these errors occur in general. [E]
- **SafeTrans (C→Rust) [10].** Guided repair resolves borrow-checker violations (E0499, E0502) 74.2% of the time on average across models, and smaller models show high rates of "lifetime issues" and "borrow conflicts". [E]
- **RustRepoTrans [11].** The best model reached 51.5% Pass@1 on repository-level translation into Rust. [E]
- **Low-resource languages [12].** Code LLMs do markedly worse on low-resource languages such as OCaml and Racket. [E] **[I]** Every new language starts out low-resource, so staying close to Rust and Swift, in both surface syntax and semantics, directly improves LLM accuracy.
- **Gap.** I found no rigorous 2025–26 study of frontier (Opus-class) models and borrow errors. **[I]** The data above comes mostly from small and mid-size models, and frontier models probably do better. The shape will likely persist: ownership errors are a minority behind type and API errors, but they take several compiler round trips to fix.

### 2.2 Hylo: mutable value semantics, second-class references

- **Second-class references.** In the strict form of mutable value semantics (MVS), "references become second-class citizens: they are only created implicitly, at function boundaries, and cannot be stored in variables or object fields". Variables therefore never share mutable state, while part-wise in-place mutation still works [13]. [E]
- **Single binding.** "Hylo allows access to a mutable value through exactly one binding at any given time". An `inout` binding projects a value and "has full ownership over the value it projects" [14]. [E]
- **Subscripts yield.** A subscript, "rather than returning a value to its caller, … temporarily yields control". Parameter conventions are `let`, `inout`, `sink` and `set`, and a `yielded` placeholder lets one declaration serve all three uses [15]. [E]
- **Graphs.** Whole/part relationships "always form a tree". Arbitrary graphs become adjacency lists of indices, and "traversing an arbitrary graph requires access to the whole graph at each step" [13]. [E]
- **Runtime cost.** Fixed-size values live on the stack, and dynamically sized containers use copy-on-write (CoW) [13]. [E]
  - Known limitation: an `inout` callee cannot tell whether its pointer is inside a shared buffer, so "the caller is compelled to copy non-unique storage defensively".
  - In the paper's synthetic benchmarks, Swift was fastest in most cases, and C++ did poorly on read-heavy programs because of copying.
- **Maturity.** The project site says Hylo "is not ready to be used yet", while claiming that ordinary code is "memory safe, typesafe, and data-race-free" by construction [16]. A 2025 talk says the design "has solidified into a coherent system" after about 4.5 years, and names the difficulty of keeping the project and its community alive [17]. [E]
- **Checker cost.** No data. **[I]** References cannot escape, so the checker needs no lifetime inference: exclusivity is a local, path-based check within each function.
- **LLM.** No data. **[I]** The vocabulary (`inout`, `sink`, `let`) is familiar from Swift and C#. The error "you stored a reference" disappears, and "use an index" replaces it.

### 2.3 Swift 6.x: retrofitted ownership, non-escapable types, regions

- **Exclusivity.** Swift enforces it statically where it can, and dynamically for class properties, globals and escaping closures. Since Swift 5 the dynamic checks also run in release builds and trap on violation; Apple says "the impact should be small in most cases" [18]. Closures have been non-escaping by default since Swift 3 [19]. [E]
- **`~Copyable`.** Since Swift 5.9 this gives unique ownership with `borrowing`, `consuming` and `inout` conventions, and borrowing is the default for methods [20]. The original proposal barred noncopyable types from generics, protocols and existentials; SE-0427 lifted that in Swift 6.0 [21]. [E]
- **`~Escapable` and lifetimes.** [E]
  - SE-0446 accepted non-escapable types but deliberately left out the ability to return them [22].
  - The standard library's Span-returning APIs then shipped using an unofficial `@lifetime`. The Language Steering Group was wary of formalizing that design, so in 6.2 it made `@lifetime` a "supported experimental feature" [23].
  - In February 2026, "Pitch #3" still proposes `@lifetime(borrow self)`, `@lifetime(copy x)` and `@lifetime(immortal)`, with defaults only where unambiguous, and leaves placement and inference open [24].
  - A PLDI 2026 talk covers "Adding Lifetimes and Ownership to Swift" [25].
  - Span itself is checked at compile time with no runtime cost [28].
  - **[I]** Even a language built on second-class references needed lifetime dependencies for views, and adding them late has taken more than two years.
- **DRF.** [E]
  - Region-based isolation (SE-0414, Swift 6.0) groups non-Sendable values into flow-sensitive regions and only reports a use after transfer. It is not field-sensitive [26].
  - `sending` (SE-0430) marks parameters and results that must be disconnected from other regions [27].
- **The concurrency rollout hurt.** [E]
  - Swift 6.2's "approachable concurrency" adds a main-actor-by-default option and changes `nonisolated async` semantics. It lets programmers "write data-race free code with less annotation overhead", because under the old rules it was "difficult to write async methods for class types without data-race safety errors" [28].
  - A July 2026 forum thread on default isolation says isolation "can permeate across a code base", and calls the resulting dialect split "a very high cost to pay" [29].

### 2.4 Mojo: origins

- Mojo renamed lifetimes to "origins" in 24.6 (January 2025) [32]. An origin names "the variable a reference borrows from" plus its mutability. The compiler uses origins to keep references from outliving values, to enforce mutable exclusivity, and to destroy values as soon as possible [30][31]. [E]
- "Most of the time, origins are handled automatically by the compiler" [31]. Explicit origins appear in a few places [31]: [E]
  - `ref` arguments and returns;
  - parametric types such as `Pointer` and `Span`;
  - union origins: a function that returns one of two strings uses the union origin `[a, b]`.
- The wildcard origin and `UnsafeAnyOrigin` escape hatches disable checking. The first is discouraged and the second is slated for removal [30][31]. [E]
- Mojo 1.0 entered beta in May 2026 [33]. [E]
- **[I]** Origins are Rust lifetimes named by places rather than by generic parameters, plus inference. In effect, Mojo has shipped the "place-based lifetimes" item from Rust's roadmap [1].

### 2.5 Vale, Valen, group borrowing; Carbon

- **Generational references (Vale).** [E]
  - Each reference stores its target's generation number and checks it on every dereference.
  - On a terrain benchmark, unsafe code took 43.82 s; reference counting added 25.29% and generational references 10.84% [34].
  - "Regions" (`pure` functions freeze their arguments) remove the checks: "Adding the pure keyword eliminates every single generation check in this function". But "Regions are still a work-in-progress" [35].
  - Vale is at 0.2 alpha [36].
- **Group borrowing.** Nick Smith drafted this for Mojo in August 2025; Mojo did not adopt it [37]. [E]
  - References belong to groups. Mutating a group invalidates references into its child groups only.
  - It allows mutable aliases, as in `attack(entities[0], entities[1])`, and it pushes data toward tree shapes.
  - It has no unique references, so code like `p1[] = p2[]` needs aliasing escape hatches, and it may lose `noalias` optimizations.
  - The syntax is "verbose", and the design is unimplemented.
- **Valen.** Evan Ovadia's new language (October 2026) builds on group borrowing [38]. [E]
  - It relaxes shared-xor-mutable to "no use-after-free".
  - Function signatures must state `mut` and `in` paths.
  - It may need LLVM patches to recover `noalias`, and it is a prototype.
  - Ovadia claims that Carbon "pivoted completely to group borrowing" [38]. I could not verify this.
- **Carbon's official strategy** makes "safe, shared mutation" its biggest safety change [39]. [E]
  - It adds pointer kinds for shared mutable and exclusive mutable access, building on Ante's ideas.
  - It plans incremental DRF enforcement, as Swift did.
  - Its 0.1 milestone moved to "at the soonest, the end of 2026" [40].
- **[I]** A visible 2025–26 trend is relaxing exclusivity to "no invalidation", motivated by C++ interop. None of these designs is implemented at scale.

### 2.6 Austral: linear types

- The type system has two rules, the Linear Universe rule and the Use-Once rule, and its checker is "less than 600 lines of code" [41]. [E]
- Borrows take explicit region parameters (`&[T, R]`, `&![T, R]`) [41]. [E]
- There are no destructors; cleanup is an explicit call [41]. [E]
- The author admits: "Returning tuples from every function and threading linear values through the code is very verbose" [41]. [E]
- **[I]** Strict linearity (must-use) is excellent for resources and proofs, but too heavy as the default for memory.

### 2.7 Koka, Lean 4, Morphic, Lobster: reference counting made cheap

- **Koka (Perceus).** [E]
  - Perceus emits precise reference counting (RC) so that "(cycle-free) programs are garbage free".
  - Reuse analysis turns updates of unique values into in-place updates ("functional but in-place", FBIP). On red-black tree insertion, pure Koka is "within 10%" of C++ `std::map` [42].
  - Atomic RC is used only for objects marked thread-shared; the paper cites slowdowns of up to 50% for always-atomic RC.
  - There is no cycle collector. Mutable references are the main way to create cycles [42].
- **Koka (FP²).** The `fip` keyword statically checks that a function runs with no allocation and constant stack space when its arguments are unique [43]. A later report finds reuse analysis "fragile with respect to small program transformations", and notes it can raise peak heap use without bound [44]. [E]
- **Lean 4.** [E]
  - Exact RC enables in-place updates when the count is 1, and borrow annotations are inferred heuristically [45].
  - "Lean cannot create cyclic data" [46].
  - Accidental sharing silently forces copies; the documentation provides `dbgTraceIfShared` to find them [46].
- **Morphic (OOPSLA 2026).** [E]
  - It infers borrows with lifetimes fully automatically for a pure language, with no annotations.
  - Where a program cannot be typed as written, inference inserts a few RC operations.
  - Against Perceus, RC increments drop 75–100% on every benchmark that had any, with a 1.48x geometric-mean speedup. Soundness is proved [47].
- **Lobster.** [E]
  - It picks one owner per allocation and treats every other use as a borrow. Where Rust would report an error, it inserts an RC increment instead.
  - This removes about 95% of RC operations, and porting "several dozen programs" needed only two changes.
  - Cycles are reported at program exit, and functions are specialized by ownership kind [48].
- **[I]** A borrow checker that turns errors into RC operations works. But specializing functions by ownership kind multiplies code, which costs compile time, and the cost model becomes invisible.

### 2.8 OxCaml modes

- **Modes.** Modes describe how a value is used, separately from its type [49]. [E]
  - Main axes: locality (local/global), contention (contended/shared/uncontended), portability, uniqueness (unique/aliased) and linearity (once/many).
  - Further axes: forkable, yielding, visibility and statefulness.
  - Each default is the legacy-compatible value, so unannotated OCaml keeps its old behaviour.
  - "Mode crossing" lets some types ignore an axis; for example, deeply immutable types ignore contention.
- **Locality is inferred.** "Even without explicit mode annotations, the compiler can statically determine which variables may escape". Returning a local value needs the `exclave` keyword [52]. [E]
- **DRF.** Portability and contention together give DRF. To cut false rejections, the ICFP 2026 "Mode Crossing" paper adds modal kinds. It is proved sound in Rocq/Iris, inferred by a lattice solver, and deployed in a large industrial codebase [53]. The underlying papers ("Oxidizing OCaml with Modal Memory Management", ICFP 2024; "Data Race Freedom à la Mode", POPL 2025) are listed in [54]. [E]
- **Uniqueness pitfalls** [50][51]: [E]
  - inference works "per module", so moving a function to another module can break its callers;
  - for-loops "may not close over unique values at all";
  - values must be threaded through return values;
  - combining uniqueness with pattern matching is currently unsound and needs a workaround;
  - in-place overwriting "hasn't been implemented yet".
- **[I]** OxCaml is the best example of opt-in precision layered on a safe default (here, GC). Nine axes is too many concepts to put in front of users or LLMs. The mechanism itself (lattice-valued annotations, defaults, mode crossing) is a clean way to implement a few qualifiers.

### 2.9 Pony, Inko, Verona/Reggio

- **Pony.** Six capabilities (`iso`, `trn`, `ref`, `val`, `box`, `tag`) rest on one rule: "By sharing only immutable data and exchanging only isolated data we can have safe concurrent programs without locks". The tutorial concedes that getting this right "is very difficult" [55][56]. [E]
- **Inko.** [E]
  - Single ownership, with `ref` and `mut` borrows that may coexist.
  - Borrows are counted at runtime with non-atomic counters. Dropping an owner while borrows are live panics, but moving an owner while it is borrowed is allowed.
  - Values cross between processes as unique `uni` values, created with `recover` [57].
  - **[I]** This moves the borrow checker's work into runtime panics: easy to write, weaker static guarantees.
- **Verona/Reggio (OOPSLA 2023).** [E]
  - Objects are grouped into isolated regions, and each region has its own memory-management strategy.
  - A thread may mutate only one open region at a time, its "window of mutability".
  - Reference capabilities enforce isolation, so data access needs no atomic operations [58].
  - Verona is research-only and "under going a massive refactoring" [59].

### 2.10 Cyclone

- Every pointer carries a region annotation, but defaults and intraprocedural inference fill most of them in [60]. [E]
  - Porting C required altering about 8% of the code, and only 6% of those changes were region annotations.
  - In 110,000 lines of code, the authors wrote one non-default effect annotation.
- The design integrates with stack allocation and a garbage collector [60]. [E]
- Cyclone "is no longer supported", and "Several of Cyclone's ideas have made their way into Rust" [61]. [E]
- **[I]** Defaults plus local inference can make region annotations rare. Cyclone's problem was adoption and its dependence on a GC, not annotation burden.

### 2.11 Summary matrix

Cells condense §2. Cells that state "no data" or carry no source in §2 are **[I]**.

| System | Safety | Runtime cost | Annotation burden | Gaps (graphs, cycles, self-reference) | Checker cost | Maturity (Oct 2026) | LLM evidence |
|---|---|---|---|---|---|---|---|
| Rust | Memory + DRF, static | ~0 (Rc/RefCell opt-in) | Lifetimes in signatures and structs | Self-reference (Pin/unsafe); graphs via indices/Rc | Per-MIR-body; Polonius ≤2–3x worst case | Production | 16.7% of compile errors; 74% fixed with iteration |
| Hylo | Memory + DRF by construction | CoW; defensive copies | Conventions only | Graphs via indices; no stored references | No data | Not usable | None |
| Swift 6 | Memory + DRF (Swift 6 mode) | ARC; dynamic exclusivity traps | Low, but lifetimes and isolation are growing | Classes allow cycles (ARC leaks) | No data | Production; lifetimes experimental | None |
| Mojo | Memory; exclusivity | ~0 | Mostly inferred origins | Escape hatches being removed | No data | 1.0 beta | None |
| Vale/Valen | Memory via runtime generation checks | +10.8% (before regions) | `pure`, region and path annotations | Mutable aliasing allowed | No data | Stalled / prototype | None |
| Austral | Memory + resources (linear) | 0 | High (threading values) | No destructors | <600-line checker | Hobby-scale | None |
| Koka / Lean | Memory; FBIP | RC, elided, with reuse | None | Cycles leak (Koka); impossible (Lean) | No data | Research / production (Lean) | None |
| OxCaml | Memory (GC) + DRF via modes | GC; stack allocation for `local` | Opt-in, nine axes | Uniqueness unfinished | Lattice solver | Industrial use | None |
| Pony | Memory + DRF (capabilities) | GC per actor | Six capabilities | Hard to learn | No data | Niche | None |
| Inko | Memory; runtime borrow panics | Borrow counts | Low | Panics at runtime | No data | Niche | None |
| Lobster | Memory (RC) | ~5% of RC operations remain | None | Cycles found at exit | Specialization | Niche | None |

## 3. What works, what failed, and why

### Works

1. **Non-escaping references by default.** This covers Swift closures (since 2016), Hylo references, OxCaml `local` and Mojo's inferred origins. When the default is "this reference cannot outlive the call", most code needs no lifetime annotations and checking stays local [13][19][31][52]. [E/I]
2. **Defaults and local inference instead of explicit polymorphism.** Examples: Cyclone's 6%-of-8% figure [60], Mojo's mostly inferred origins [31], OxCaml's legacy-compatible defaults [49] and Swift's "default only where unambiguous" rule [24]. [E]
3. **Cheap reference counting** [E]:
   - uniqueness enables in-place reuse of RC'd memory (Perceus, Lean, FP²) [42][43][45];
   - inferred borrows elide most RC (75–100% fewer increments in Morphic [47], about 95% fewer operations in Lobster [48]).
4. **Flow sensitivity buys acceptance.** Polonius Alpha [2] and Swift regions [26] both accept more correct programs. The price is checker time: 2–3x in Polonius's worst case [2]. [E]
5. **One DRF rule underneath everything: share immutable data, transfer isolated data, synchronize explicitly.** Pony [55], Inko's `uni` [57] and Swift's `sending` [27] all reduce to this. [E/I]

### Failed or struggled

1. **Retrofitting is the real killer.** [E]
   - Swift's lifetimes are still a pitch in 2026 [24].
   - Swift 6 strict concurrency forced a course correction in 6.2 and a dialect split [28][29].
   - Rust's view types and partial borrows are still listed as in "design exploration" [1].
   - **[I]** Pick the reference model up front. Adding a second reference discipline later costs years.
2. **Too many qualifiers.** Pony's six capabilities are "very difficult" by the tutorial's own account [55]. OxCaml has nine axes and needed a research paper to cut false positives [49][53]. [E]
3. **Strict linearity as the default is verbose.** See Austral [41] and OxCaml's unique-value threading [51]. [E]
4. **Runtime-checked schemes deliver safety but not Rust's static guarantees or zero cost.** [E]
   - Generational references cost 10.84% before regions, and regions were never finished [34][35].
   - Inko panics at runtime [57].
   - Swift traps on dynamic exclusivity violations [18].
5. **RC with hidden costs.** Reuse is fragile [44], accidental sharing forces copies in Lean [46], and cycles leak in Koka and Lobster [42][48]. Predictability suffers. [E]
6. **Unsafe aliasing rules defined after the fact.** Rust went from Stacked Borrows to Tree Borrows over several years [4]. [E]
7. **Rust's specific false rejections persist.** Conditional borrow returns, disjoint field borrows across methods, self-referential structs and lending iterators are all still open or only being fixed now, in late 2026 [1][2]. [E]

## 4. Candidate memory models

All three candidates share a common base:

- single ownership with deterministic destruction, and moves;
- no tracing GC;
- `unsafe` blocks with a defined aliasing model;
- Send/Sync-like thread-safety properties.

They differ in what a reference is.

### Model A: values with second-class borrows and declared views (recommended base)

**Ownership and copies.**

- Every type is a value with one owner.
- Copying is implicit only for bit-copyable types; any other copy is written out (`x.copy()`).
- There is no implicit CoW in the core. CoW collections are an opt-in library type.

**Parameter conventions.**

- `let`: shared borrow (the default).
- `inout`: exclusive borrow.
- `take`: move.
- `out`: initialize.
- Borrows cannot be stored in ordinary structs, returned, or captured by escaping closures. Closures do not escape by default, as in Swift [19].

**Returning access.**

1. *Projections.* Accessors `yield` a borrow, as in Hylo subscripts [15]. This covers `map[k]`, `vec[i]`, lending iterators and conditional borrows.
2. *Non-escaping view types* such as `Span`, `StrView`, `Iter` or `Parser`.
   - A view's dependency is written as a parameter name: `fn first_word(s: let String) -> StrView from s`.
   - Elision: with one borrowed input, the dependency is inferred, following Swift's rule [24]. With several, write the union `from a, b`, like Mojo's `[a, b]` [31].
   - Views may contain views.
   - A view cannot escape the scope of its sources.
   - There are no lifetime generics and no `'a`.

**Exclusivity checking.**

- The checker works per function, on places (paths).
- It is flow-sensitive and field-sensitive.
- Methods may declare the fields they touch (`inout self.{pos, buf}`), as in Rust's view-types proposal [1], so calls on disjoint fields compose.

**Non-tree data.**

1. `Arena<T>` with generational `Id<T>` handles: a checked index that pays only on access [34].
2. Region-scoped references:
   - Values allocated in an arena may hold `ref T from arena` to each other, with interior mutability through `Cell`-like types.
   - This follows Cyclone and Verona regions [58][60]: Rust's `&'arena` pattern without lifetime parameters.
3. `Rc`/`Arc` with `Weak`:
   - The compiler elides count traffic with Perceus/Morphic-style borrow inference [42][47].
   - Copies of these handles are explicit, `Share`-style [3].

**Data-race freedom.**

- Tasks take values by `take` (owned, isolated) or, inside structured scopes, as `let` borrows of `Sync` data.
- Shared mutable state lives only in `Mutex`, atomics or actors.
- Thread-safety is computed structurally from fields; plain and deeply immutable data crosses automatically, in the spirit of OxCaml's mode crossing [49][53].
- `spawn` signatures have no lifetimes.

**Costs.**

- *Runtime.* Zero for borrows and views; a check per access for arena handles; RC only where the programmer chose `Rc`.
- *Annotation.* Parameter conventions (defaults cover most cases) and an occasional `from x`.
- *Checker* **[I]**. Intraprocedural: signatures carry everything a caller needs, so checking is parallel and incremental.
- *LLMs* **[I]**. Fewer error classes. "You stored a borrow" becomes one rule with three fixes: a view type, an index, or `Rc`.

**What gets harder than in Rust** **[I]**:

- **Long-lived borrowed references inside heap collections** (`Vec<&'a Node>` kept in a long-lived context). These need views (which cannot escape), handles or `Rc`.
- **`TyCtxt<'tcx>`-style interning arenas compared by pointer.** These are possible through region-scoped references, but the arena must outlive all its users within one lexical scope.
- **Self-referential structs and intrusive lists.** These need `unsafe`; Rust also needs `Pin` or `unsafe` for them.
- **Generic containers of views** (`HashMap<StrView, V>`). Containers must be allowed to be non-escaping when their elements are. Swift is still working through this in 2026 for non-escapable elements in `Span` [24]. The generics design must handle this from day one.

### Model B: owned values, inferred borrows, RC fallback (Lobster/Morphic style)

**Design.**

- The surface is the same as Model A, but heap types can carry a reference count.
- The compiler never rejects safe code for aliasing. Where it cannot prove a borrow, it inserts an RC increment, as Lobster [48] and Morphic [47] do.
- Mutating a shared heap value triggers a copy (value semantics through CoW, as in Swift and Lean [45]).
- Uniqueness gives in-place updates and memory reuse (Perceus and FP² [42][43]).
- A `strict` attribute on a function or module turns inserted RC operations and CoW copies into compile errors that explain the cause. A `fip`-like check [43] proves code allocation-free.

**Pros.**

- Near-zero rejections, and so the highest first-try compile rate for LLMs **[I]**.
- Graphs work through `Rc` and `Weak`.

**Cons.**

- Hidden costs: a header word per heap object, count traffic wherever inference fails, and copy cliffs [46].
- Cycles leak [42][48].
- Values shared across threads need atomic counts; Koka keeps those off unshared objects with a thread-shared bit [42].
- Performance changes under small refactors [44].
- Specializing functions by ownership kind adds compile time [48] **[I]**.
- Predictability, one of the stated goals, is weaker.

**What gets harder than in Rust.** Nothing becomes impossible; precise control does. Guaranteeing that a hot loop has no RC requires `strict`, which locally turns Model B into Model A.

### Model C: modernized Rust core

**Design.** Keep first-class references with lifetimes, but add the improvements the Rust project is pursuing [1][2][3]:

- lifetimes named by places (`&'self.items T`);
- Polonius Alpha flow sensitivity;
- view types and two-phase borrows;
- `Share` handles and `move(expr)` captures.

**Pros.**

- Full Rust expressiveness: references inside structs, `&'arena` and so on.
- The easiest mechanical translation from Rust, and the most reuse of what LLMs already know about Rust [12] **[I]**.

**Cons.**

- It keeps the error classes behind most of Rust's ownership failures (E0499, E0502, lifetime errors) [8][10].
- Lifetime parameters in types remain.
- The Rust project itself has not finished these features [1].
- Checker cost grows with precision (2–3x in the worst case) [2].

### Comparison

| | A: values + views | B: inferred borrows + RC | C: modernized Rust |
|---|---|---|---|
| Static guarantees | Memory + DRF; no runtime panics except handle checks | Memory + DRF; cycles may leak | Memory + DRF |
| Runtime cost model | Visible (only `Id`/`Rc` cost) | Hidden; `strict` makes it visible | Visible |
| Annotation | Conventions + rare `from x` | Almost none | Lifetimes in structs and signatures |
| Graphs and cycles | Arena handles, region refs, Rc+Weak | Rc+Weak, leaks possible | Indices, Rc, `&'arena` |
| Checker | Intraprocedural, simple | Inference + specialization | Polonius-class |
| LLM first-try success **[I]** | High | Highest | Rust-like (known to be iterative) |
| Rust porting | Moderate rewrite where structs store references | Easy | Easiest |

### Recommendation [I]

- Use Model A as the base.
- Apply Model B's inference only inside the explicit `Rc`/`Arc` and CoW library types, never as a silent fallback for plain values.
- Use Model C's place-named dependencies as the syntax for Model A's views.

This keeps the cost model visible: an LLM or a human can tell from the types where counts or checks happen. It keeps checking modular, and the remaining errors are few and mechanical to fix.

## 5. Design implications (ranked)

1. **References are second-class by default; the only first-class borrowed values are views, and views cannot escape.**
   - *Confidence:* high that this removes most lifetime annotation; medium that it covers ≥95% of Rust idioms without handles.
   - *Evidence:* [13][14][15][19][31][52]. Counter-evidence that views cannot be avoided: [22][23][24].
2. **Ship the "return a borrow" story in v0: projections plus `from param` dependencies with elision, and no lifetime generics.**
   - *Confidence:* high.
   - *Evidence:* Swift shows that retrofitting this costs years [23][24], and Mojo shows that place-named origins can mostly be inferred [31].
3. **Keep exclusivity (mutation XOR sharing) as the core rule.** Check it per function, on places, with Polonius Alpha-level precision and declared field-access sets.
   - *Confidence:* high.
   - Do not adopt group borrowing or "safe shared mutation" in v1: neither is implemented at scale, both need path annotations, and both put `noalias` optimizations at risk [37][38][39].
   - Revisit in v2 as a narrow relaxation, such as mutating a field without changing the value's shape. Medium-low confidence.
4. **Keep checking modular.** Function signatures fully determine ownership effects at call sites: no cross-function ownership inference and no specialization by ownership kind.
   - *Confidence:* high, for compile speed.
   - *Evidence:* OxCaml's per-module inference pitfall [51] and Lobster's specialization [48]. Rust's borrow checker works on one MIR body at a time [7] and does not show up as a compile-time hotspot [6] **[I]**.
5. **Handle non-tree data with three standard-library tools:** `Arena` with generational `Id`; region-scoped references inside arenas; and `Rc`/`Arc`/`Weak`, with compiler borrow inference eliding count traffic.
   - *Confidence:* medium-high.
   - *Evidence:* [34][42][47][48][58][60].
6. **Derive DRF from one rule: share immutable data, transfer isolated data, synchronize explicitly.** Infer thread-safety properties structurally, and track "transfer" flow-sensitively only for `take` arguments.
   - *Confidence:* medium.
   - Avoid exposing Pony-style capabilities or OxCaml-style axes [49][55], and avoid Swift's late rollout that flipped defaults [28][29].
7. **Make ownership diagnostics machine-repairable.** Every ownership error names the conflicting places and offers ranked mechanical fixes: reorder, `copy()`, `take`, convert to an `Id`, or wrap in `Rc`.
   - *Confidence:* medium.
   - *Evidence:* LLMs fix borrow errors by iterating [9][10]. Small models fail to fully fix them 87% of the time even with feedback [8].
8. **Define the unsafe aliasing model together with the language, and run a Miri-like checker in tests.** It can be simpler than Tree Borrows because safe code produces fewer reference shapes.
   - *Confidence:* medium.
   - *Evidence:* [4].
9. **Use Rust and Swift vocabulary** (`let`/`inout`/`take`, `Rc`, `Span`, `Mutex`).
   - *Confidence:* medium.
   - *Evidence:* the low-resource-language penalty [12].
10. **Offer linear ("must-use") types as a separate, opt-in kind** for resources and protocol states, which also serves the optional-proofs goal. Do not make linearity the default for memory.
    - *Confidence:* medium-low.
    - *Evidence:* [41][49].

## 6. Open questions

1. Can generic containers of non-escaping views be made ergonomic? Swift is still pitching its design in 2026 [24].
2. How much precision can flow-sensitive place checking reach before it hits Polonius-like worst cases [2]? Answering this needs a benchmark on a prototype.
3. Are region-scoped references inside arenas sound together with interior mutability and arena reset? This needs a formalization (Iris-style, like [53]).
4. Async: are futures that borrow locals simply views? How do cancellation and structured task scopes interact with `take`?
5. What fraction of real Rust code stores references in structs that outlive one call tree? Measuring this on crates.io would validate or refute Model A.
6. LLM pilot: translate a Rust benchmark subset into Model A syntax and compare frontier-model first-pass compile rates against Rust. Today no frontier-model data exists (§2.1).
7. Should the core collections use CoW, as Swift's do, or should CoW stay opt-in for predictability?
8. Will Valen or Carbon produce real evidence for group borrowing by 2027 [38][39]?

## 7. Sources

1. Rust Project Goals 2026, "The Borrow Checker Within": https://goals.rust-lang.org/2026/roadmap-borrow-checker-within.html
2. Rust Blog, "Enabling the next iteration of the borrow checker on nightly" (2026-08-04): https://blog.rust-lang.org/2026/08/04/enabling-polonius-alpha-on-nightly
3. Rust Project Goals 2026, "Ergonomic ref-counting": https://goals.rust-lang.org/2026/ergonomic-rc.html
4. Villani, Hostert, Dreyer, Jung, "Tree Borrows", PLDI 2025: https://plf.inf.ethz.ch/research/pldi25-tree-borrows.html
5. Rust Blog, "2024 State of Rust Survey Results": https://blog.rust-lang.org/2025/02/13/2024-State-Of-Rust-Survey-results/
6. Rust Blog, "Rust compiler performance survey 2025 results": https://blog.rust-lang.org/2025/09/10/rust-compiler-performance-survey-2025-results/
7. rustc-dev-guide, "MIR borrow check": https://rustc-dev-guide.rust-lang.org/borrow-check.html
8. Nogueira, Vieira, Campos, "Unreliable in Practice? A Comprehensive Study of Errors in LLM-Generated Code", ISSRE 2026: https://arxiv.org/abs/2608.00661 (tables: https://arxiv.org/html/2608.00661)
9. Deligiannis et al., "RustAssistant: Using LLMs to Fix Compilation Errors in Rust Code": https://arxiv.org/abs/2308.05177 ; PDF: https://www.microsoft.com/en-us/research/wp-content/uploads/2024/08/paper.pdf
10. "SafeTrans: LLM-assisted Transpilation from C to Rust": https://arxiv.org/abs/2505.10708
11. "RustRepoTrans: Repository-level Code Translation Benchmark Targeting Rust": https://arxiv.org/abs/2411.13990
12. Cassano et al., "Knowledge Transfer from High-Resource to Low-Resource Programming Languages for Code LLMs": https://arxiv.org/abs/2308.09895
13. Racordon, Shabalin, Zheng, Abrahams, Saeta, "Implementation Strategies for Mutable Value Semantics", JOT 21(2), 2022: https://www.jot.fm/issues/issue_2022_02/article2.pdf
14. Hylo language tour, "Bindings": https://docs.hylo-lang.org/language-tour/bindings
15. Hylo language tour, "Subscripts": https://docs.hylo-lang.org/language-tour/subscripts
16. Hylo website source: https://github.com/hylo-lang/hylo-lang.github.io/blob/main/index.md
17. Racordon, "Designing Hylo, a programming language for safe systems programming", ECOOP PLSS 2025: https://2025.ecoop.org/details/plss-2025-papers/12/Designing-Hylo-a-programming-language-for-safe-systems-programming
18. Swift.org, "Swift 5 Exclusivity Enforcement": https://www.swift.org/blog/swift-5-exclusivity/
19. SE-0103, "Make non-escaping closures the default": https://github.com/swiftlang/swift-evolution/blob/main/proposals/0103-make-noescape-default.md
20. SE-0390, "Noncopyable structs and enums": https://github.com/swiftlang/swift-evolution/blob/main/proposals/0390-noncopyable-structs-and-enums.md
21. SE-0427, "Noncopyable Generics": https://github.com/swiftlang/swift-evolution/blob/main/proposals/0427-noncopyable-generics.md
22. Swift Forums, "[Accepted with modifications] SE-0446: Nonescapable Types": https://forums.swift.org/t/accepted-with-modifications-se-0446-nonescapable-types/75504
23. Swift Forums, "Experimental support for lifetime dependencies in Swift 6.2 and beyond": https://forums.swift.org/t/experimental-support-for-lifetime-dependencies-in-swift-6-2-and-beyond/78638
24. Swift Forums, "[Pitch #3] Compile-time lifetime dependency annotations" (2026-02-25): https://forums.swift.org/t/pitch-3-compile-time-lifetime-dependency-annotations/84968
25. PLDI 2026, "Adding Lifetimes and Ownership to Swift": https://pldi26.sigplan.org/details/pldi-2026-pldi-events/1/Adding-Lifetimes-and-Ownership-to-Swift
26. SE-0414, "Region-based Isolation": https://github.com/swiftlang/swift-evolution/blob/main/proposals/0414-region-based-isolation.md
27. SE-0430, "`sending` parameter and result values": https://github.com/swiftlang/swift-evolution/blob/main/proposals/0430-transferring-parameters-and-results.md
28. Swift.org, "Swift 6.2 Released": https://swift.org/blog/swift-6.2-released/
29. Swift Forums, "Contemplating Default Isolation Control" (2026-07-10): https://forums.swift.org/t/contemplating-default-isolation-control/88205
30. Mojo stdlib docs, `origin`: https://mojolang.org/docs/std/origin/
31. Mojo manual, "Lifetimes, origins, and references": https://mojolang.org/docs/manual/values/lifetimes/
32. Modular, "Hands-on with Mojo 24.6": https://www.modular.com/blog/hands-on-with-mojo-24-6
33. InfoWorld, "First look at Mojo 1.0" (2026-05-20): https://www.infoworld.com/article/4173158/first-look-at-mojo-1-0.html
34. Ovadia, "Generational References": https://verdagon.dev/blog/generational-references
35. Ovadia, "Zero-cost memory safety with Vale regions": https://verdagon.dev/blog/zero-cost-memory-safety-regions-overview
36. Vale download page (version 0.2, alpha): https://vale.dev/download
37. Ovadia (with Nick Smith), "Group Borrowing" (2025-08-28): https://verdagon.dev/blog/group-borrowing
38. Ovadia, "Valen's Memory Safety: A New Kind of Borrow Checking" (2026-10-11): https://verdagon.dev/blog/valen-group-borrowing
39. Carbon proposal #5914, "Updating Carbon's safety strategy": https://docs.carbon-lang.dev/proposals/p005914-updating-carbon-s-safety-strategy.html
40. Carbon proposal #4880, "Safety milestones and a 2025 roadmap": https://docs.carbon-lang.dev/proposals/p004880-safety-milestones-and-a-2025-roadmap.html
41. Borretti, "Introducing Austral": https://borretti.me/article/introducing-austral
42. Reinking, Xie, de Moura, Leijen, "Perceus: Garbage Free Reference Counting with Reuse", PLDI 2021 (TR v4): https://www.microsoft.com/en-us/research/uploads/prod/2020/11/perceus-tr-v4.pdf
43. Lorenzen, Leijen, Swierstra, "FP²: Fully in-Place Functional Programming", ICFP 2023: https://www.microsoft.com/en-us/research/publication/fp2-fully-in-place-functional-programming-2/ ; MSR blog: https://www.microsoft.com/en-us/research/blog/fp2-fully-in-place-functional-programming-provides-memory-reuse-for-pure-functional-programs/
44. "Reference Counting with Frame Limited Reuse" (extended version): https://www.microsoft.com/en-us/research/publication/reference-counting-with-frame-limited-reuse-extended-version/
45. Ullrich, de Moura, "Counting Immutable Beans", IFL 2019: https://arxiv.org/abs/1908.05647
46. Lean Language Reference, "Reference Counting": https://lean-lang.org/doc/reference/4.22.0-rc3/Run-Time-Code/Reference-Counting/
47. Brandon, Driscoll, Dai, Ragan-Kelley, Milano, Aiken, "Fully-Automatic Type Inference for Borrows with Lifetimes", OOPSLA 2026: https://2026.splashcon.org/details/oopsla-2026/22/Fully-Automatic-Type-Inference-for-Borrows-with-Lifetimes
48. van Oortmerssen, Lobster "Memory Management": https://aardappel.github.io/lobster/memory_management.html
49. OxCaml docs, "Modes: Intro": https://oxcaml.org/documentation/modes/intro/
50. OxCaml docs, "Uniqueness: Intro": https://oxcaml.org/documentation/uniqueness/intro/
51. OxCaml docs, "Uniqueness: Pitfalls": https://oxcaml.org/documentation/uniqueness/pitfalls/
52. Jane Street Tech Blog, "Oxidizing OCaml: Locality": https://blog.janestreet.com/oxidizing-ocaml-locality/
53. Peters, Jacobs, Kalinichenko, Stevenson, Smith, Dreyer, Eisenberg, "Mode Crossing", ICFP 2026: https://icfp26.sigplan.org/details/icfp-2026-icfp-papers/9/Mode-Crossing
54. OxCaml documentation index and bibliography: https://oxcaml.org/documentation/
55. Pony Tutorial, "Reference Capabilities": https://tutorial.ponylang.io/reference-capabilities/reference-capabilities.html
56. Clebsch, Drossopoulou, Blessing, McNeil, "Deny Capabilities for Safe, Fast Actors", AGERE 2015: https://www.ponylang.io/media/papers/fast-cheap.pdf
57. Inko manual, "Memory management": https://docs.inko-lang.org/manual/main/getting-started/memory-management
58. Arvidsson et al., "Reference Capabilities for Flexible Memory Management", OOPSLA 2023: https://www.microsoft.com/en-us/research/publication/reference-capabilities-for-flexible-memory-management/
59. Project Verona repository: https://github.com/microsoft/verona
60. Grossman, Morrisett, Jim, Hicks, Wang, Cheney, "Region-Based Memory Management in Cyclone", PLDI 2002: https://www.cs.umd.edu/projects/cyclone/papers/cyclone-regions.pdf
61. Cyclone homepage: http://cyclone.thelanguage.org/
