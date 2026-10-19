# 08: Type system core and abstraction mechanisms

Research date: 2026-10-19. Scope: the static semantics of Ferox (data types, generics, traits, inference, refinements, metaprogramming, errors, nulls, integers, strings, diagnostics, surface regularity). Ownership, concurrency and effects belong to other reports and appear here only where they constrain the type system.

Method: claims with an inline link are evidence from that source. Paragraphs marked **Inference** are my reasoning from that evidence. The shared web-search budget ran out partway through, so later sources were fetched directly. Claims that rest on search summaries of pages I could not open are marked *(secondary)*.

---

## 1. Question

Which type-system core and abstraction mechanisms let Ferox:

- match Rust's safety and runtime speed;
- compile far faster and stay lighter;
- keep checking decidable, local and fast, with excellent errors;
- offer machine-checked proofs as an option;
- give coding agents one obvious way to write things?

The report has three deliverables: a coherent core with explicit omissions (§4.1, §4.7), a generics and abstraction recommendation with costs (§4.2), and a metaprogramming recommendation (§4.3).

---

## 2. Findings (cited)

### 2.1 Generics implementation strategies and their costs

- **Monomorphization (Rust, C++, Mojo).**
  - In Rust, "monomorphization happens per crate", so "generics in Rust can lead to accidentally-quadratic compilation times across many crates". The standard workaround is a thin generic layer that delegates at once to non-generic code ([matklad, Fast Rust Builds](https://matklad.github.io/2021/09/04/fast-rust-builds.html)).
  - Mojo likewise "creates a concrete version … for each unique parameter value" ([Mojo manual](https://mojolang.org/docs/manual/parameters)).
  - Users notice. The 2025 Rust compiler survey had 3,700+ responses: 55% wait more than ten seconds per rebuild, and about 45% of former users cite compile times as a reason they left ([Rust blog](https://blog.rust-lang.org/2025/09/10/rust-compiler-performance-survey-2025-results)).
  - Retrofitting sharing failed. rustc's experimental *polymorphization* pass was deleted in December 2024: it "breaks that mental model" of monomorphized bodies, had no known users, and cost maintenance ([MCP #810](https://github.com/rust-lang/compiler-team/issues/810); [PR #133883](https://github.com/rust-lang/rust/pull/133883)).
- **Witness passing with optional specialization (Swift).**
  - Generic code is compiled once. Callers pass *value witness tables* (size, alignment, copy, move, destroy) and protocol witness tables, and "specialization can still be done as an optimization".
  - Generics are reified, so `Pair<Int>` and `Pair<Bool>` get distinct unboxed layouts ([Pestov & McCall, LLVM 2017](https://llvm.org/devmtg/2017-10/slides/Pestov-McCall-ImplementingGenerics.pdf)).
  - Costs: performance falls off a cliff where specialization is impossible, such as across modules without `@inlinable`. One developer measured specializations at about 7% of an app binary *(secondary: [forum](https://forums.swift.org/t/generic-specialization-when-satisfying-a-concrete-protocol-requirement-with-generics/61577), [forum](https://forums.swift.org/t/generic-specialization-binary-size-cost/69096))*.
- **Shape stenciling plus dictionaries (Go).**
  - Go emits one body per GC shape (size, alignment, pointer map), and each body takes a dictionary argument ([design doc](https://go.googlesource.com/proposal/+/master/design/generics-implementation-gcshape.md)).
  - All pointers share one shape, so method calls go through the dictionary. That blocks inlining and devirtualization: "DO NOT attempt to use Generics to de-virtualize or inline method calls". Value types showed no regression ([PlanetScale](https://planetscale.com/blog/generics-can-make-your-go-code-slower)).
  - .NET shares one copy across reference types and specializes per value type ([Microsoft Learn](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/generics/generics-in-the-run-time)).
- **A 2026 counter-intuitive data point.** Deser is a Rust serializer built on dynamic dispatch instead of monomorphization. Release builds of derived code compile about 2.3× faster than with serde. JSON reads are about 10% slower, and binary bloat is "quite a bit worse" ([Ronacher, Deser](https://lucumr.pocoo.org/2026/9/29/deser/)).

**Inference.** Generics vary on two independent axes:

- *when* bodies are checked: at the definition (Rust, Swift, Carbon) or at each instantiation (C++, Zig);
- *how* code is generated: specialized or shared.

Swift shows that definition-checked semantics leave code generation free. Go shows that sharing alone gives up the inlining a Rust-class target needs. Rust shows that sharing cannot be bolted on after promising full monomorphization.

### 2.2 Traits, modules, comptime, parameters

- **Coherence.**
  - Rust allows an impl only where the trait or the type is local, which "ensures that other people's code can't break your code" ([Rust book ch10-02](https://doc.rust-lang.org/book/ch10-02-traits.html)).
  - Carbon states the trade-off: "coherence is a desirable property, but to get that property we need an orphan rule, and that rule has a cost". The motivation is the hashtable problem (insert and lookup must use one `Hash`).
  - Carbon rejects incoherence because "the interpretation of source code changes based on imports". It serves alternative impls through compatible types the user casts between ([Carbon appendix](https://docs.carbon-lang.dev/docs/design/generics/appendix-coherence.html)).
- **Orphan-rule cost.**
  - Ronacher doubts any serde successor can win because "the orphan rule" forces every type-defining crate to add impls ([Deser](https://lucumr.pocoo.org/2026/9/29/deser/)). Bridge crates are a recurring tax ([DZone](https://dzone.com/articles/architectural-cost-rust-orphan-rule)).
  - A March 2026 proposal by a Rust compiler contributor to drop coherence for named impls drew heated debate *(secondary: [wikidocs](https://wikidocs.net/blog/@jaehong/9865/))*. The standing objection is that library code such as `HashMap` calls traits on your behalf, so call-site disambiguation is impossible ([Ixrec/rust-orphan-rules](https://github.com/Ixrec/rust-orphan-rules)).
- **Modules versus type classes.** Type classes are "a particular mode of use of modules": modules stress explicit configuration, classes implicit construction ([Dreyer, Harper, Chakravarty](https://www.cs.cmu.edu/~rwh/papers/mtc/short.pdf); see also [modular implicits](https://arxiv.org/abs/1512.01895)).
- **Scala 3** rebuilt Scala 2 implicits, which were "easily over-used and mis-used", produced "inscrutable type errors" fixed by imports, and hurt tooling ([Scala 3 reference](https://docs.scala-lang.org/scala3/reference/contextual/index.html)).
- **Refusing type classes.**
  - Gleam says they "tend to have confusing error messages", "have a high compile time cost", and carry runtime cost "unless the compiler performs full-program compilation and expensive monomorphisation". Gleam also excludes overloading, implicit and optional arguments, exceptions and untagged unions ([Gleam FAQ](https://gleam.run/frequently-asked-questions/)).
  - Roc's only ad hoc polymorphism is static method dispatch. It derives a closed set of six methods and will never add higher-kinded polymorphism ([Roc reference](https://roc-lang.org/docs/main/langref/types/); [Roc FAQ](https://git.joshthomas.dev/language-servers/roc/src/commit/1e20ff1a9a74f262ee07ca6acd3de6571f42aa98/FAQ.md)).
- **Zig comptime.**
  - Generics are duck-typed: "Zig doesn't have declaration-site type checking of comptime code".
  - Deliberate limits (no IO, no host leakage, no syntax extension, no methods on generated types) make it "hermetic, reproducible, safe, and cacheable" ([matklad](https://matklad.github.io/2025/04/19/things-zig-comptime-wont-do.html)).
  - Tooling still suffers. ZLS is "100% AST based", and without comptime its guesses "would make ZLS nearly perfectly accurate" *(secondary: [Ziggit](https://ziggit.dev/t/what-is-holding-back-zls/1471))*. Global comptime state is banned because it "makes incremental compilation virtually impossible" *(secondary: [Ziggit](https://ziggit.dev/t/comptime-and-incremental-compilation/5389))*.
- **Carbon and Mojo.**
  - Carbon's checked generics are "typechecked once" against interfaces and preferred at API boundaries. Templates are mainly for C++ interop ([Carbon overview](https://docs.carbon-lang.dev/docs/design/generics/overview.html)).
  - Mojo parameters are compile-time values that allow limited "dependent type" expressions. The manual warns that inference "sometimes reports a misleading error" ([Mojo](https://mojolang.org/docs/manual/parameters)).

### 2.3 Rust and Swift trait-system pain

- **The new trait solver.** It took "nearly 4 years" and is "the largest single change to the Rust compiler since its initial release". It reached nightly by default in August 2026.
  - It had "many cases where it performed quadratically or even exponentially slower".
  - It makes `datafusion` compile 8× faster. A chess engine written in the type system now finishes in a minute instead of hanging.
  - "We have not yet spent too much time on error messages" ([Rust blog, 2026-08-21](https://blog.rust-lang.org/2026/08/21/enabling-next-solver-on-nightly)).
- **GATs** shipped "over six and a half years after the original RFC", with known limitations ([Rust blog](https://blog.rust-lang.org/2022/10/28/gats-stabilization/)).
- **Async fn in traits** shipped without a way to require `Send` futures. The fix (return-type notation) had its stabilization PR closed and is blocked on the new solver ([RTN goal](https://rust-lang.github.io/goals/2026/rtn.html); [PR #138424](https://github.com/rust-lang/rust/pull/138424)).
- **Three spellings of one bound:** argument-position `impl Trait` ("syntax sugar"), inline bounds, and `where` ([Rust book](https://doc.rust-lang.org/book/ch10-02-traits.html)).
- **Hand-patched diagnostics.** Trait authors needed `#[diagnostic::on_unimplemented]` to make missing-impl errors usable ([Rust 1.78](https://blog.rust-lang.org/2024/05/02/Rust-1.78.0/)).
- **Swift undecidability.** Recursive protocol constraints plus same-type requirements made Swift generics undecidable (a word-problem reduction). They were reimplemented as a term-rewriting "requirement machine" *(secondary: [Swift forums](https://forums.swift.org/t/formalizing-swift-generics-as-a-term-rewriting-system/45175); [mjtsai](https://mjtsai.com/blog/2022/02/28/swift-type-checking-is-undecidable))*.
- **Termination can be guaranteed by design.** GHC's Paterson conditions require constraints smaller than the instance head, so "each reduction step makes the problem smaller" ([GHC guide](https://downloads.haskell.org/ghc/latest/docs/users_guide/exts/instances.html)).

### 2.4 Inference scope

- **Signatures.** Rust requires typed signatures as "a deliberate decision", which also yields "more-helpful error messages" ([Rust book ch03-03](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html)).
- **Bidirectional typing** avoids heavy annotation, improves error locality, and supports features where full inference is undecidable ([Dunfield & Krishnaswami](https://arxiv.org/abs/1908.05839)).
- **Hindley–Milner.** It is DEXPTIME-complete in the worst case ([Mairson 1990](https://dl.acm.org/doi/10.1145/96709.96748)) but close to linear in practice ([Wikipedia](https://en.wikipedia.org/wiki/Hindley%E2%80%93Milner_type_system)). Roc gives up higher-kinded and rank-N types to keep principal inference ([Roc FAQ](https://git.joshthomas.dev/language-servers/roc/src/commit/1e20ff1a9a74f262ee07ca6acd3de6571f42aa98/FAQ.md)).
- **Overloading plus literals.** In Swift's solver this searches "an n-dimensional Cartesian product". `0 + 1 + -(2)` needed 1,190 guesses, and one expression took 20 s in Swift 3.1 ([Cocoa with Love, 2016](https://www.cocoawithlove.com/blog/2016/07/12/type-checker-issues.html)).
- **No inference.** Austral has none at all and "no implicit type conversions anywhere" ([Austral](https://borretti.me/article/introducing-austral)).
- **Types steering LLM output.** Type-constrained decoding "reduces compilation errors by more than half" and raises functional correctness ([Mündler et al., PLDI 2025](https://arxiv.org/abs/2504.09246)).

### 2.5 Refinement types and dependent types lite

- **Flux** (liquid types for Rust) needs half the specification lines Prusti does and verifies an order of magnitude faster. Prusti's annotation overhead was up to 24% of code size ([Lehmann et al.](https://arxiv.org/abs/2207.04034)). TickTock used Flux to verify isolation in the production Tock kernel, found real bugs, and verifies "in half a minute" *(secondary: [paper](https://cseweb.ucsd.edu/~dstefan/pubs/rindisbacher:2025:ticktock.pdf))*.
- **Verus** writes specs and proofs in Rust, with linear ghost types and a mode system that keeps specifications unrestricted ([Lattuada et al.](https://arxiv.org/abs/2303.05491)).
- **LLMs with verifier feedback already do well.**
  - AutoVerus proved more than 90% of 150 tasks *(secondary: [arXiv](https://arxiv.org/abs/2409.13082v1))*.
  - On DafnyBench the best model verified 68%. Error feedback helped, and longer programs hurt ([DafnyBench](https://arxiv.org/abs/2406.08467)).
- **Other systems.**
  - Liquid Haskell is a GHC plugin ([site](https://ucsd-progsys.github.io/liquidhaskell/)).
  - Idris 2's quantitative type theory expresses erasure and resource use in types ([Brady](https://arxiv.org/abs/2104.00480)).
  - Lean 4 unifies hygienic macros with elaboration ([Ullrich & de Moura](https://arxiv.org/abs/2001.10490)).

### 2.6 Metaprogramming costs

- **IDE cost.** rust-analyzer estimates that macros cost `X` in a batch compiler but about `X²` in an IDE ([rust-analyzer](https://rust-analyzer.github.io/blog/2021/11/21/ides-and-macros.html)).
  - Proc macros are opaque, entangled with name resolution, and must run in a killable process.
  - Their advice: macros should "not change the meaning of existing code", be recognizable before name resolution, and be local, deterministic and bounded. Prefer code generation where possible.
- **Compile cost.** Derive expansion "is not currently cached" ([survey](https://blog.rust-lang.org/2025/09/10/rust-compiler-performance-survey-2025-results)). Proc-macro users are "blocked until the proc macro is fully compiled", and `syn` is slow to build ([matklad](https://matklad.github.io/2021/09/04/fast-rust-builds.html)).
- **Supply-chain cost.** In 2023 `serde_derive` shipped a precompiled binary to save build time. After backlash, v1.0.184 restored source builds ([serde#2538](https://github.com/serde-rs/serde/issues/2538); [release](https://github.com/serde-rs/serde/releases/tag/v1.0.184)).
- **Swift and Haskell repeat the pattern.**
  - Swift macros pull in swift-syntax, adding minutes to CI builds, so SwiftPM now experiments with prebuilt binaries ([Swift forums](https://forums.swift.org/t/preview-swift-syntax-prebuilts-for-macros/80202)).
  - For Template Haskell, "the slow part is compiling the resulting code": 1,500 lines expanded to about 200 kloc ([Parsons](https://www.parsonsmatt.org/2019/11/27/keeping_compilation_fast.html)).
- **Reflection instead of codegen.** Rust's `facet` gives each type a `SHAPE` constant and builds JSON, TOML, YAML and other formats on that one layer: "the last proc macro you should need" ([facet](https://github.com/facet-rs/facet)). Gleam is open to metaprogramming only if it is "not detrimental to … readability and fast compilation" ([Gleam FAQ](https://gleam.run/frequently-asked-questions/)).

### 2.7 Error handling

- **Midori (Duffy).** Bugs cause *abandonment*, and recoverable errors are checked and annotated at call sites. "All unannotated over/underflows were considered bugs and led to abandonment". Java checked exceptions are "the worst of both worlds" ([Duffy](https://joeduffyblog.com/2016/02/07/the-error-model/)).
- **Hejlsberg** objects to checked exceptions on versioning and scalability grounds, which push developers to `throws Exception` and empty catches ([Artima](https://www.artima.com/articles/the-trouble-with-checked-exceptions)).
- **Swift.** Untyped `throws` "remains the better default". Typed throws serves exhaustive handling, pass-through generics, and Embedded Swift ([SE-0413](https://github.com/apple/swift-evolution/blob/main/proposals/0413-typed-throws.md)).
- **Rust `?`** returns early and calls `From::from` implicitly ([Rust book ch09-02](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)).
- **Zig error sets** coerce to supersets, and `try` is `catch |err| return err`. Inferred sets complicate function pointers and recursion *(secondary: [zig.guide](https://zig.guide/language-basics/errors))*.
- **Kotlin rich errors** (KEEP-0441, in design review) add Zig-inspired error unions (`User | NotFound | PermissionDenied`). Explicit goals include "keeping type inference polynomial", and checked exceptions are a non-goal ([KEEP-0441](https://github.com/Kotlin/KEEP/blob/main/proposals/KEEP-0441-rich-errors-motivation.md)).
- **Go** stopped pursuing error syntax in June 2025. Error handling is still the top complaint, but no proposal reached consensus, and a late second way conflicts with Go's principles ([Go blog](https://go.dev/blog/error-syntax)).
- **Koka**'s effect handlers are "not ready for production use" ([Koka](https://koka-lang.github.io/koka/doc/book.html)).

### 2.8 Nulls, integers, strings

- **Null.** Hoare: "my billion-dollar mistake" ([InfoQ](https://www.infoq.com/presentations/Null-References-The-Billion-Dollar-Mistake-Tony-Hoare/)). Dart has enforced sound null safety since Dart 3 and reports "fewer bugs, smaller binaries, and faster execution" ([dart.dev](https://dart.dev/null-safety)).
- **Overflow.**
  - Rust (RFC 560) treats overflow as a program error, checked under debug assertions and wrapping in release ([RFC 560](https://rust-lang.github.io/rfcs/0560-integer-overflow.html); [reference](https://doc.rust-lang.org/reference/expressions/operator-expr.html)). The cost of release checks is argued with anecdotes, from "barely measurable" to 10× in vectorizable loops *(secondary: [LWN](https://lwn.net/Articles/960822/))*.
  - Swift "reports an error" by default and offers `&+` ([Swift book](https://raw.githubusercontent.com/swiftlang/swift-book/main/TSPL.docc/LanguageGuide/AdvancedOperators.md)).
  - Zig makes overflow detectable illegal behavior, with `+%` for wrapping, `+|` for saturating, and lossless widening coercion ([zig.guide](https://zig.guide/language-basics/integer-rules); [reference](https://ziglang.org/documentation/master/)).
- **Conversions.** Go bans implicit numeric conversion because of "the confusion it causes" ([Go FAQ](https://go.dev/doc/faq)). Rust's `as` truncates silently ([reference](https://doc.rust-lang.org/reference/expressions/operator-expr.html)).
- **Strings.**
  - Rust strings are UTF-8 with no indexing, because indexing should be O(1). Std provides bytes and scalars but not graphemes ([Rust book ch08-02](https://doc.rust-lang.org/book/ch08-02-strings.html)).
  - Swift's `Character` is a grapheme cluster, with no integer indexing and canonical-equivalence `==` ([Swift book](https://raw.githubusercontent.com/swiftlang/swift-book/main/TSPL.docc/LanguageGuide/StringsAndCharacters.md)). Native UTF-8 gave SwiftNIO 20% ([swift.org](https://www.swift.org/blog/utf8-string/)).
- **Source text.** Trojan Source (CVE-2021-42574) hides logic with bidi controls, and specs should "formally disallow" them ([trojansource.codes](https://trojansource.codes/)).

### 2.9 Diagnostics, syntax, LLMs

- **Diagnostics.** Elm moved to errors that show the user's code with hints *(secondary: [Elm](https://elm-lang.org/news/compiler-errors-for-humans))*. Rust followed with primary and secondary labels, because "errors should focus on the code you wrote" ([Rust blog 2016](https://blog.rust-lang.org/2016/08/10/Shape-of-errors-to-come/)).
- **LLMs and errors.** LLMs fix Rust compile errors with about 74% peak accuracy ([RustAssistant](https://arxiv.org/abs/2308.05177)). They perform much worse in low-resource languages ([MultiPL-T](https://arxiv.org/abs/2308.09895)).
- **Generic syntax.** Rust needs the turbofish "to avoid ambiguity with the less-than operator" ([reference](https://doc.rust-lang.org/reference/paths.html)). Go and Mojo use square brackets, and Go bans method-level type parameters ([Go design](https://go.googlesource.com/proposal/+/refs/heads/master/design/43651-type-parameters.md)).
- **Equivalent spellings.** Kotlin's scope functions are "interchangeable in many cases" ([Kotlin](https://kotlinlang.org/docs/scope-functions.html)).
- **Visible costs.** Swift made existentials `any P` because the old spelling hid costs and was confused with generics ([SE-0335](https://github.com/apple/swift-evolution/blob/main/proposals/0335-existential-any.md)).
- **Agents.** Ronacher finds agents do best with explicit, simple code: "agents struggle with Python's magic", and "simple code significantly outperforms complex code in agentic contexts" ([2025](https://lucumr.pocoo.org/2025/6/12/agentic-coding/)). He also sees agents lowering the cost of unfamiliar languages ([2026](https://lucumr.pocoo.org/2026/8/22/fast-hard-code/)).

---

## 3. What works, what failed, and why

**Works.**

- ADTs with exhaustive matching (Rust, Swift, Elm, Gleam, Roc, and Kotlin's coming error unions). Go left variants out ([FAQ](https://go.dev/doc/faq)), and error handling is now its top complaint.
- Mandatory signatures with local inference.
- Definition-checked generics.
- An error/bug split with explicit propagation markers (`?`, `try`).
- Non-null by default.
- Checked arithmetic with explicit wrapping operators.
- Hermetic compile-time evaluation.
- Closed derive sets and reflection-driven libraries.
- Elm-style diagnostics.

**Inference:** the common thread is that behavior is visible at the site where an agent reads it.

**Failed or proved costly.**

- Per-crate monomorphization without dedup, and failed retrofits of code sharing.
- Unbounded solvers (Swift undecidability, Rust's four-year rewrite, GAT and RTN delays).
- Orphan-rule lock-in. Note that it centres on *derivable* traits whose behavior follows from structure.
- Token macros (IDE X², uncached expansion, supply-chain incidents).
- Wide-scope implicits (Scala 2).
- Overloading plus literal polymorphism (Swift).
- Checked exceptions.
- Late error-syntax changes (Go).
- Multiple equivalent spellings.
- Sharing-only generics (Go: lost inlining).

**Inference:** each feature that adds solver power or user-defined expansion has cost years of compiler work and degraded diagnostics.

---

## 4. Recommended core

### 4.1 Types

| Category | Included | Notes |
|---|---|---|
| Scalars | `Bool`; `i8…i128`, `u8…u128`; `isize`/`usize` (indices only); `f32`, `f64`; `Char` (Unicode scalar) | No implicit conversions. Literals take the expected type, else default to `i64`/`f64`. |
| Products | Nominal `struct`; small tuples | One construction syntax. |
| Sums | Nominal `enum` with payloads | Exhaustive `match` required. `open enum` forces a wildcard arm in other packages (as Rust [`non_exhaustive`](https://doc.rust-lang.org/reference/attributes/type_system.html)). |
| Optionality | `T?`, the only spelling of `Option[T]` | No null. |
| Errors | `T ! E` | §4.5 |
| Functions | First-class functions, closures | Closure parameter types come from the expected type, else they must be annotated. |
| Generics | Type and const parameters (integers, `Bool`, fieldless enums) in `[...]` | No arithmetic on const parameters inside types in v1. |
| Abstraction | `trait`: methods, flat associated types and consts, default methods | §4.2 |
| Dynamic dispatch | `dyn Trait`, always spelled | Cost is visible (SE-0335). |
| Text | `Str`, `String` (UTF-8), `Bytes` | §4.5 |

### 4.2 Generics and abstraction (deliverable 2)

**Semantics: definition-checked generics, coherent traits.**

- A generic body is checked once, against its declared bounds, and instantiation never yields a type error. Only resource limits can fail later, and they get their own error.
- Traits resolve implicitly and coherently: one impl per (trait, type) in the program.
- The orphan rule allows impls in the trait's package or the type's package.
- The coherence unit is the **published package**, not the compilation unit. Splitting a package for build parallelism never trips the orphan rule.
- Alternative behaviors (sort by title, a custom hash) use explicit record-of-functions values, or zero-cost newtypes with delegated impls (`newtype ByTitle = Song via Song`). This is Carbon's answer to the hashtable problem: the type selects the impl.
- Decidability, enforced syntactically:
  1. No overlapping impls and no specialization. Selection is a head match with no backtracking.
  2. Paterson-style termination: impl where-clauses must be structurally smaller than the head, with no escape hatch.
  3. Flat associated types, with no recursive same-type constraints (the Swift undecidability source).
  4. No HKT, GAT, rank-N types, negative bounds or user auto traits. Only compiler-known markers (`Copy`, `Send`, `Sync`) are auto traits.
  5. One bound syntax, inside the bracket list only: `fn sort[T: Ord](xs: &mut [T])`.
- Methods cannot add their own type parameters (as in Go). Use free functions.
- Modules are namespaces only, with no functors. Traits are the implicit mode of modules (Dreyer et al.), and explicit record values cover the remaining functor use cases.

**Implementation: semantics independent of codegen, two backends.**

- *Development builds* use shared code with witness passing, as Swift does. Each generic function compiles once, with a witness table per bound and a layout witness (size, alignment, move, drop) per type parameter. Codegen then scales with generic *definitions*, not *uses*.
- *Release builds* use full monomorphization through a **content-addressed instantiation cache shared across the package graph**. Each `(function, type-args)` pair is compiled once per build, which removes per-crate duplication. Cold, large instantiations may fall back to the shared version.
- No overlap, no specialization and no type-identity dispatch together make the two strategies observationally equivalent. CI runs tests in both modes. `dyn Trait` reuses the witness tables.

**Costs.**

| Cost | Magnitude | Mitigation |
|---|---|---|
| Two codegen paths | High. Swift's value-witness runtime is complex. | Resolve layouts at compile time where possible. Ship monomorphization first, but design the IR for both from day one (polymorphization shows retrofit fails). |
| Slower development-build runtime on generic hot loops | Go-like ([PlanetScale](https://planetscale.com/blog/generics-can-make-your-go-code-slower)) | Per-dependency "specialize in dev" opt-in. Benchmark only in release. |
| Release binary size | Swift-like specialization share *(secondary)* | Global dedup, size heuristic, linker identical-code folding. |
| Orphan friction | Real (serde) | Package-level coherence, reflection (§4.3), cheap newtype delegation. |
| Lost expressiveness (HKT, GAT, specialization) | No generic monad stacks, lending iterators, or specialized fast paths | Concrete types and explicit dictionaries. Revisit GATs only with corpus evidence. |

### 4.3 Metaprogramming (deliverable 3)

No user-defined syntax-level macros. Five mechanisms, each analyzable without running user code during name resolution:

1. **Closed built-in `derive`** (`Eq`, `Ord`, `Hash`, `Clone`, `Copy`, `Debug`, `Default`). Compiler-implemented, cached and documented, like Roc's list.
2. **A compiler-provided `Shape` constant on every nominal type** (kind, fields, variants, layout, names), opt-out for types with invariants. Serializers, printers, ORMs and diff tools become ordinary generic library code, as with facet and Zig std. This removes the main driver of both proc macros and orphan-rule pain. Cost: per-type specialization (code size) or interpretation (Deser: about 10% slower reads). Hot paths can hand-write impls.
3. **`const fn` evaluation.** Pure, fuel-limited, deterministic, with no IO or host information (Zig's rules), cached by content hash. It evaluates constants and const arguments, and never creates types or declarations.
4. **Codegen as a visible build step.** Generators emit ordinary `.fx` files that the IDE and agents read like any other code, per rust-analyzer's advice.
5. **Attributes are inert data** (`#[test]`, `#[inline]`, diagnostic hints).

Deferred: *typed derive templates*, which are `const fn`s over `Shape` that may only emit impls of one named trait for the annotated type. They would be append-only, local and deterministic. Add them only if reflection proves insufficient.

**Inference:** agents read source. Every mechanism above keeps the real program in source or in fixed, documented semantics. Token macros hide it behind an expansion the agent cannot see.

### 4.4 Inference discipline

- **Signatures.** All item signatures are annotated: functions, fields, constants, trait items.
- **Bodies.** Bidirectional local inference. Unification works only within one function, and there is no signature inference.
- **Names.** No overloading: one function per name per scope, one method per name per type. A clash between two in-scope traits is an error with a fix-it.
- **Operators.** They map to fixed core traits, with no custom operator symbols. Literals are checked against the expected type, never searched over.
- **Result.** One pass per function with no backtracking. Checking is polynomial, parallel per function, and works on prefixes, which constrained decoding needs.

### 4.5 Errors, nulls, integers, strings

- **Errors.**
  - `fn f() -> T ! E`, where `E` is a nominal error type or a flat union: `T ! (NotFound | Denied)`.
  - `?` propagates and only *widens* unions. Converting into a different nominal type is explicit (`.map_err(Wrap)`), with no hidden `From`.
  - Results are must-use.
  - No exceptions, checked or unchecked.
  - Bugs (overflow, out-of-bounds, failed asserts, `panic`) abandon the task or process. Recovery happens only at supervised boundaries.
- **Null.** None. `T?` gets a niche layout.
- **Integers.**
  - `+ - * / % << >>` trap on overflow, division by zero and over-wide shifts in every profile.
  - Explicit forms: wrapping `+%`, saturating `+|`, checked `a.checked_add(b) -> T?`, and a `Wrapping[T]` type.
  - Conversions: `widen[i64]()` (lossless only), `narrow[u8]() -> u8?`, `truncate[u8]()`. There is no `as`.
  - Refinements and the optimizer delete provably redundant checks.
  - No numeric tower. Big integers and decimals live in the standard library. Floats get IEEE operators and `total_cmp`, but no `Ord`/`Hash`.
- **Strings.**
  - `Str`/`String` are always UTF-8, indexed by opaque `StrIdx` byte offsets.
  - Iteration is explicit: `.bytes()`, `.chars()`, and `.graphemes()` in the standard library with a pinned Unicode version.
  - `==` is byte equality. Canonical equivalence is a library call. **Inference:** Swift's grapheme `==` is principled but makes a basic operator depend on Unicode tables.
- **Source text.** ASCII identifiers. Bidi controls are rejected everywhere.

### 4.6 Optional verification layer

- **What can be written.** `requires`/`ensures` clauses and parameter refinements (`i: usize where i < xs.len()`), restricted to a decidable liquid fragment: linear arithmetic, uninterpreted functions, arrays and enum tags. Invariants are inferred where possible (Flux).
- **Modes per package.**
  - `off`: specs become runtime asserts (Midori contracts).
  - `check`: SMT-verified, with counterexample diagnostics.
  - `prove`: adds ghost code and lemmas (Verus).
- **Erasure.** Specs are erased and never affect typing or codegen, so unverified code pays nothing.
- **No full dependent types.** Types never compute.

### 4.7 Explicit omissions

| Omitted | Reason |
|---|---|
| HKT, rank-N types | Break local or principal inference and add solver load ([Roc FAQ](https://git.joshthomas.dev/language-servers/roc/src/commit/1e20ff1a9a74f262ee07ca6acd3de6571f42aa98/FAQ.md); [Gleam #1276](https://github.com/gleam-lang/gleam/discussions/1276)) |
| GATs (v1) | 6.5 years in Rust, shipped with limits ([Rust blog](https://blog.rust-lang.org/2022/10/28/gats-stabilization/)) |
| Specialization, overlapping or negative impls | Break dev/release equivalence and coherence; Carbon calls scoped impls "particularly complex … due to … specialization" ([Carbon](https://docs.carbon-lang.dev/docs/design/generics/appendix-coherence.html)) |
| Recursive associated-type constraints | Swift's undecidability source *(secondary)* |
| Overloading, custom operators | Exponential checking, many meanings per name ([Cocoa with Love](https://www.cocoawithlove.com/blog/2016/07/12/type-checker-issues.html)) |
| Implicit parameters and conversions | Import-dependent meaning ([Scala 3](https://docs.scala-lang.org/scala3/reference/contextual/index.html); [Go FAQ](https://go.dev/doc/faq)) |
| Exceptions, checked exceptions | Hidden control flow, versioning ([Hejlsberg](https://www.artima.com/articles/the-trouble-with-checked-exceptions); [Duffy](https://joeduffyblog.com/2016/02/07/the-error-model/)) |
| Null | [Hoare](https://www.infoq.com/presentations/Null-References-The-Billion-Dollar-Mistake-Tony-Hoare/); [Dart](https://dart.dev/null-safety) |
| Token macros, Template Haskell-style splices, unrestricted comptime | §2.6 |
| Full dependent types | Evaluation in the checker. Refinements give most of the value (inference). |
| User-defined effects and handlers (v1) | Still research-stage ([Koka](https://koka-lang.github.io/koka/doc/book.html)) |
| Subtyping and inheritance (beyond union widening and lifetimes) | Complicate inference and coherence (inference) |
| Untagged unions outside FFI | A second, unsafe way to write sums ([Gleam FAQ](https://gleam.run/frequently-asked-questions/)) |
| Signature inference | Loses local reasoning and prefix typing ([Rust book](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html); [Mündler](https://arxiv.org/abs/2504.09246)) |
| Method-level type parameters | Object-safety complexity ([Go design](https://go.googlesource.com/proposal/+/refs/heads/master/design/43651-type-parameters.md)) |

---

## 5. Design implications (ranked)

1. **Cap trait-system power now, syntactically.** No overlap or specialization, no HKT, GAT or rank-N, Paterson termination, flat associated types, one bound syntax, head-match resolution. Every Rust and Swift trait crisis (solver rewrite, undecidability, GAT and RTN delays) came from power added later. *Confidence: high.*
2. **Mandatory signatures, bidirectional local inference, no overloading.** This gives local reasoning, error locality, polynomial and parallel checking, and prefix typing for constrained decoding, which halves compile errors. *High.*
3. **No user syntax macros.** Use closed derives, a built-in `Shape` reflection constant, hermetic `const fn`, and visible codegen. Macros are the recurring source of IDE cost, uncached builds and supply-chain incidents in Rust and Swift, and reflection also defuses orphan-rule lock-in. *High (no token macros); medium (reflection-based serialization, which needs benchmarks).*
4. **Make generic semantics independent of codegen.** Design the IR for both witness passing (dev) and globally deduplicated monomorphization (release). This is the biggest structural compile-speed lever that keeps Rust-class runtime. *High that the IR must allow both; medium that the dev backend repays its cost before 1.0.*
5. **Errors are values: `T ! E`, widening-only `?`, abandonment for bugs, no exceptions. Freeze this before 1.0.** Midori, Rust, Zig and Swift converge on it, Kotlin is moving toward it, and Go shows it cannot be fixed later. *High; medium for unions over plain enums.*
6. **Checked integer arithmetic in every profile, explicit wrapping and saturating forms, no lossy `as`.** Release-mode wrapping hides bugs in shipped builds. Benchmark vectorized loops early. *Medium-high.*
7. **Coherence per published package, with newtype delegation as the sanctioned escape hatch.** *Medium.*
8. **Treat diagnostics as a data product.** JSON output, stable codes, spans in user code, machine-applicable fixes, trait-author hints, and a regression suite that gates releases. Agents live on this feedback, and Rust's solver work shows diagnostics regress unless someone owns them. *High.*
9. **Optional, erased, decidable liquid verification with assert, check and prove modes.** Flux and TickTock show low annotation cost and fast checks, and LLMs reach 68–90% proof success with verifier feedback. *Medium (SMT determinism open).*
10. **Familiar, regular syntax.** Square-bracket generics, mandatory `dyn`, single spellings (`T?`, `T ! E`), ASCII identifiers, no bidi controls, Rust and Swift tokens where semantics match. LLMs underperform in low-resource languages, so transfer matters. *Medium-high.*
11. **UTF-8 strings with opaque indices and byte `==`; graphemes in the standard library.** *Medium-high.*

**Surprising findings.**

- Rust's orphan-rule pain centres on *derivable* traits. A built-in reflection constant attacks both the orphan rule and the proc-macro problem.
- A 2026 dynamic-dispatch serializer compiled release builds about 2.3× faster but had *larger* binaries.
- Rust deleted its code-sharing pass for maintainability and lack of users, not for soundness. The lesson is architectural.

---

## 6. Open questions

1. How much of Swift's runtime generic metadata is needed if dev builds never cross a stable ABI? Can every layout be resolved at compile time, passing only witness tables?
2. Can unsafe code or FFI observe the layout differences between the two backends? Should unsafe generic functions opt out of sharing?
3. What are the size and throughput numbers for `Shape`-driven serialization and hashing, specialized versus interpreted?
4. How are error unions represented (global IDs as in Zig, or per-signature enums), and how do they interact with generic `E`?
5. Should an unconstrained literal default to `i64`, or be an error?
6. Can trait methods be `async` in v1? Rust's RTN saga argues for coordinating with the concurrency report.
7. How do we keep verification results stable across SMT solver versions: pin the solver, emit certificates, or limit the fragment to a built-in decision procedure?
8. Without GATs, do lending iterators and zero-copy parsing push users toward `unsafe`?
9. How does package-level coherence behave when two semver-incompatible versions of one package coexist in a build?

---

## 7. Sources

Generics and compile time:
- https://matklad.github.io/2021/09/04/fast-rust-builds.html
- https://blog.rust-lang.org/2025/09/10/rust-compiler-performance-survey-2025-results
- https://github.com/rust-lang/compiler-team/issues/810
- https://github.com/rust-lang/rust/pull/133883
- https://llvm.org/devmtg/2017-10/slides/Pestov-McCall-ImplementingGenerics.pdf
- https://forums.swift.org/t/generic-specialization-when-satisfying-a-concrete-protocol-requirement-with-generics/61577
- https://forums.swift.org/t/generic-specialization-binary-size-cost/69096
- https://go.googlesource.com/proposal/+/master/design/generics-implementation-gcshape.md
- https://planetscale.com/blog/generics-can-make-your-go-code-slower
- https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/generics/generics-in-the-run-time
- https://mojolang.org/docs/manual/parameters
- https://lucumr.pocoo.org/2026/9/29/deser/

Traits, coherence, modules, comptime:
- https://doc.rust-lang.org/book/ch10-02-traits.html
- https://docs.carbon-lang.dev/docs/design/generics/appendix-coherence.html
- https://docs.carbon-lang.dev/docs/design/generics/overview.html
- https://dzone.com/articles/architectural-cost-rust-orphan-rule
- https://wikidocs.net/blog/@jaehong/9865/
- https://github.com/Ixrec/rust-orphan-rules
- https://www.cs.cmu.edu/~rwh/papers/mtc/short.pdf
- https://arxiv.org/abs/1512.01895
- https://docs.scala-lang.org/scala3/reference/contextual/index.html
- https://gleam.run/frequently-asked-questions/
- https://github.com/gleam-lang/gleam/discussions/1276
- https://roc-lang.org/docs/main/langref/types/
- https://git.joshthomas.dev/language-servers/roc/src/commit/1e20ff1a9a74f262ee07ca6acd3de6571f42aa98/FAQ.md
- https://matklad.github.io/2025/04/19/things-zig-comptime-wont-do.html
- https://ziggit.dev/t/what-is-holding-back-zls/1471
- https://ziggit.dev/t/comptime-and-incremental-compilation/5389

Trait-system pain:
- https://blog.rust-lang.org/2026/08/21/enabling-next-solver-on-nightly
- https://blog.rust-lang.org/2022/10/28/gats-stabilization/
- https://rust-lang.github.io/goals/2026/rtn.html
- https://github.com/rust-lang/rust/pull/138424
- https://blog.rust-lang.org/2024/05/02/Rust-1.78.0/
- https://forums.swift.org/t/formalizing-swift-generics-as-a-term-rewriting-system/45175
- https://mjtsai.com/blog/2022/02/28/swift-type-checking-is-undecidable
- https://downloads.haskell.org/ghc/latest/docs/users_guide/exts/instances.html

Inference:
- https://doc.rust-lang.org/book/ch03-03-how-functions-work.html
- https://arxiv.org/abs/1908.05839
- https://dl.acm.org/doi/10.1145/96709.96748
- https://en.wikipedia.org/wiki/Hindley%E2%80%93Milner_type_system
- https://www.cocoawithlove.com/blog/2016/07/12/type-checker-issues.html
- https://borretti.me/article/introducing-austral
- https://arxiv.org/abs/2504.09246

Verification:
- https://arxiv.org/abs/2207.04034
- https://cseweb.ucsd.edu/~dstefan/pubs/rindisbacher:2025:ticktock.pdf
- https://arxiv.org/abs/2303.05491
- https://arxiv.org/abs/2409.13082v1
- https://arxiv.org/abs/2406.08467
- https://ucsd-progsys.github.io/liquidhaskell/
- https://arxiv.org/abs/2104.00480
- https://arxiv.org/abs/2001.10490

Metaprogramming:
- https://rust-analyzer.github.io/blog/2021/11/21/ides-and-macros.html
- https://github.com/serde-rs/serde/issues/2538
- https://github.com/serde-rs/serde/releases/tag/v1.0.184
- https://forums.swift.org/t/preview-swift-syntax-prebuilts-for-macros/80202
- https://www.parsonsmatt.org/2019/11/27/keeping_compilation_fast.html
- https://github.com/facet-rs/facet

Errors, null, integers, strings:
- https://joeduffyblog.com/2016/02/07/the-error-model/
- https://www.artima.com/articles/the-trouble-with-checked-exceptions
- https://github.com/apple/swift-evolution/blob/main/proposals/0413-typed-throws.md
- https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
- https://zig.guide/language-basics/errors
- https://zig.guide/language-basics/integer-rules
- https://ziglang.org/documentation/master/
- https://github.com/Kotlin/KEEP/blob/main/proposals/KEEP-0441-rich-errors-motivation.md
- https://go.dev/blog/error-syntax
- https://go.dev/doc/faq
- https://koka-lang.github.io/koka/doc/book.html
- https://www.infoq.com/presentations/Null-References-The-Billion-Dollar-Mistake-Tony-Hoare/
- https://dart.dev/null-safety
- https://rust-lang.github.io/rfcs/0560-integer-overflow.html
- https://doc.rust-lang.org/reference/expressions/operator-expr.html
- https://lwn.net/Articles/960822/
- https://raw.githubusercontent.com/swiftlang/swift-book/main/TSPL.docc/LanguageGuide/AdvancedOperators.md
- https://raw.githubusercontent.com/swiftlang/swift-book/main/TSPL.docc/LanguageGuide/StringsAndCharacters.md
- https://www.swift.org/blog/utf8-string/
- https://doc.rust-lang.org/book/ch08-02-strings.html
- https://trojansource.codes/
- https://doc.rust-lang.org/reference/attributes/type_system.html

Diagnostics, syntax, LLMs:
- https://elm-lang.org/news/compiler-errors-for-humans
- https://blog.rust-lang.org/2016/08/10/Shape-of-errors-to-come/
- https://arxiv.org/abs/2308.05177
- https://arxiv.org/abs/2308.09895
- https://doc.rust-lang.org/reference/paths.html
- https://go.googlesource.com/proposal/+/refs/heads/master/design/43651-type-parameters.md
- https://kotlinlang.org/docs/scope-functions.html
- https://github.com/apple/swift-evolution/blob/main/proposals/0335-existential-any.md
- https://lucumr.pocoo.org/2025/6/12/agentic-coding/
- https://lucumr.pocoo.org/2026/8/22/fast-hard-code/
