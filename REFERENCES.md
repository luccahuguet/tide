# References

This file lists the languages, projects, papers, and measurements Tide draws
on, with the decisions (D0–D17 in `design/01-design.md`) each one informed. It
is a bibliography, not the language reference, which is `design/reference.md`.
Each report in `research/` holds the full citations and context; reports
written before the rename call the language Ferox.

## Influences by Language and Project

The Report column names files in `research/` by number. "perf" is
`reviews/round1-performance.md` and "usability" is
`reviews/round1-agent-usability.md`. A dash means the research records nothing
in that column.

| Language or project | What Tide takes | What Tide rejects | Decisions | Report |
| --- | --- | --- | --- | --- |
| [Rust](https://doc.rust-lang.org/book/ch10-02-traits.html) | Spelling wherever the meaning matches; ownership and shared-xor-mutable; lifetime elision; `Send`/`Sync`; traits; `Result` and `?`; derives; NLL-precision borrow checking; `thread::scope` panic propagation | Lifetime parameters, user and proc macros, `async`/`.await`, GATs and specialization, build scripts, wrapping release arithmetic, lossy `as`, ambient I/O | D1–D6, D8, D13 | 02, 03, 05, 08, 10 |
| [Zero (zerolang)](https://github.com/vercel-labs/zerolang) | Stable diagnostic codes; repair plans returned without applying them; expected-hash checks so stale writes fail; version-matched agent guidance | Graph-first source store: three source-model switches in three weeks and no outcome data; Tide keeps text canonical | D14 | 06 |
| [Vera](https://github.com/aallan/vera) | Evidence that a language taught only in context can be written correctly on small problems; diagnostics with code, rationale, and fix | Bindings named by type and position (`@Int.0`) | D14 | 06 |
| [Koka](https://koka-lang.github.io/koka/doc/index.html) | Evidence-passing result that tail-resumptive handlers are just passed implementations; Perceus data on reference-counting costs | Row-polymorphic effects, user handlers, hidden reference counting of ordinary values | D2, D7 | 02, 07 |
| [Effekt](https://effekt-lang.org/) | Capabilities as lexically scoped values ("no builtin effects, only captures") | Handlers and continuations; boxes for first-class functions | D7, D8 | 07, 10 |
| [Austral](https://borretti.me/article/introducing-austral) | Root capability that arrives only as `main`'s argument; narrower capabilities derived from broader ones | Linear types as the memory default; explicit region parameters; no type inference | D2, D3, D7 | 02, 07, 08, 09 |
| [Pony](https://tutorial.ponylang.io/reference-capabilities/reference-capabilities.html) | Derived authority from `AmbientAuth`; "share immutable, transfer isolated" as the race-freedom rule; the warning that passing root everywhere is too permissive, hence `Root` only in `main` | Six reference capabilities | D7, D8 | 02, 07, 10 |
| E / [Joe-E](https://www2.eecs.berkeley.edu/Pubs/TechRpts/2008/EECS-2008-91.html) | Joe-E: a method whose argument types are all immutable is pure (the shape of the `Pure` auto trait); no mutable statics; "reviewably secure" as a goal. E itself has no source in `research/` | – | D7, D11 | 07 |
| [WASI](https://wasi.dev/) and [cap-std](https://docs.rs/cap-std/2.0.1/cap_std) | WASI: start with no ambient authority, deny or virtualize what the host grants (the shape of the simulated root); cap-std: directory capabilities and attenuation | Capability libraries over an ambient std ("not a sandbox") | D7, D15 | 07 |
| [Deno](https://docs.deno.com/runtime/fundamentals/security/) and [Java SecurityManager](https://openjdk.org/jeps/411) | Lessons only | Process-wide permissions with no per-dependency tier; stack-inspection ACLs that ended in `AllPermission` | D7, D13 | 07 |
| [Hylo](https://docs.hylo-lang.org/language-tour/bindings) | References that cannot escape by default, so most code needs no annotation | Second-class-only references with copy-on-write defensive copies; its `inout`/`sink` vocabulary | D2 | 02 |
| [Swift](https://swift.org/blog/swift-6.2-released/) | Overflow traps by default; non-escaping closures; spelled-out existentials (`any P`, here `dyn`); scoped child tasks | Type-directed overloading with global inference; witness-table generics by default; lifetimes and isolation retrofitted late; reentrant actors | D3, D4, D8, D12 | 02, 03, 08, 10 |
| [Mojo](https://mojolang.org/docs/manual/values/lifetimes/) | Origins: a borrow named by where it comes from, including union origins (`from a, b`) | Square-bracket generics; MLIR and GPU work before 1.0 | D1, D2, D12 | 02, 08, 09 |
| [Zig](https://ziglang.org/documentation/master/) | I/O passed by the caller like `Allocator`; an owned fast debug backend; function-level incremental rebuilds; fuel-limited compile-time evaluation; checked overflow with explicit wrapping | Duck-typed comptime generics with no definition-site checking | D3, D4, D6, D7, D12 | 03, 07, 08, 09, 10 |
| [Go](https://go.dev/doc/faq) | Export data read only for direct imports; unused imports and cycles as errors; no implicit numeric conversion; no code runs at fetch or build time; `go fix` and option-free gofmt; `synctest` virtual time | Shared-memory goroutines (data races); GC-shape dictionaries that block inlining | D3, D12–D15 | 03, 06, 08, 09, 10 |
| [Trio](https://vorpus.org/blog/notes-on-structured-concurrency-or-go-statement-considered-harmful/) | Nurseries (no task outlives its scope); level-triggered cancel scopes with deadlines | – | D8 | 10 |
| [Kotlin coroutines](https://kotlinlang.org/docs/coroutines-basics.html) | Parent and child tasks with linked lifecycles | Implicit context parameters, which later needed explicit arguments to resolve overload ambiguity | D7, D8 | 07, 10 |
| [Java Loom](https://openjdk.org/jeps/444) and [.NET green threads](https://github.com/dotnet/runtimelab/issues/2398) | Lessons only | Stackful tasks in compiled code (pinning, slow foreign calls, thread-local surprises) | D8 | 10, perf |
| [OCaml 5 and Eio](https://icfp23.sigplan.org/details/ocaml-2023-papers/5/Eio-1-0-Effects-based-IO-for-OCaml-5) | Direct-style I/O with structured cancellation | Untracked effects that hide suspension points | D8 | 07, 10 |
| [Scala capture checking](https://scala-lang.org/api/3.x/docs/experimental/capture-checking/overview.html) and [Flix](https://doc.flix.dev/effect-oriented-programming.html) | Lessons only | Per-value capture sets and Boolean effect algebras (years of research, NP-hard inference) | D7 | 07 |
| [Rayon](https://github.com/rayon-rs/rayon) | `join`; parallel results equal to sequential results; lazy adaptive splitting | – | D9 | 01, 10 |
| [MPL / Parallel ML](https://github.com/MPLLang/mpl) | Automatic granularity control; heartbeat scheduling only if benchmarks justify it | – | D9 | 01, 10 |
| [Futhark](https://futhark-lang.org/) | GPU data parallelism belongs in a separate kernel language | GPU work in Tide before 1.0 | D12, D16 | 01, 10 |
| [Bend 2](https://bend-lang.com) ([repo](https://github.com/bendlang/bend)) and Bend 1 | Laws that impls must satisfy, with agent-written proofs gated by human approval; erased ghost code; the tandem for proved parallel kernels through a process boundary | Leading with proofs (Bend's pitch); once-only closures, no `if`, 32-bit numbers that wrap; whole-program builds; emitting Bend | D10, D11, D16 | 01, 11 |
| [HVM](https://github.com/HigherOrderCO/HVM) | Lessons only; research/01 leaves superposition search as a possible external engine | Interaction nets as a runtime or backend | D9, D16 | 01, 10 |
| [FoundationDB](https://apple.github.io/foundationdb/testing.html) | A whole system simulated in one process, with simulated time, network, and disks | – | D15 | 10 |
| [TigerBeetle](https://docs.tigerbeetle.com/about/vopr) | VOPR's seed-plus-commit replay; always-on assertions | – | D15 | 07, 10 |
| [Antithesis](https://antithesis.com/blog/deterministic_hypervisor/) | Whole-machine determinism as the bar a simulator must meet | A hypervisor as the enforcement point; Tide gets determinism from capabilities | D15 | 10 |
| [Dafny](https://dafny.org/) | Clause names (`requires`, `ensures`, `invariant`, `decreases`); spec well-formedness obligations; opaque bodies by default | Managed heap; mathematical integers in executable code | D10 | 04, 05 |
| [Verus](https://verus-lang.github.io/verus/guide/print.html) | SMT contracts on Rust-like code, per function; erased ghost code; fuel limits; nonlinear and bit-vector reasoning in marked blocks | Separate ghost and executable types, blamed for lower LLM success | D10, D12 | 03, 04 |
| [Creusot](https://github.com/creusot-rs/creusot) | Prophecies for `&mut` (the model behind `final(x)`) | – | D10 | 04 |
| [Prusti](https://www.pm.inf.ethz.ch/research/prusti.html) | Evidence that ownership supplies separation-logic framing for free | Annotation load up to 24% of code size | D10 | 04 |
| [Flux](https://flux-rs.github.io/flux/) | Evidence that ownership enables cheap SMT verification (half Prusti's spec lines, a tenth of its time) | Refinements in the type system; Tide's specs never affect typing | D10 | 04, 08 |
| [Liquid Haskell](https://ucsd-progsys.github.io/liquidhaskell/) | Decidable refinements and termination metrics as evidence for SMT automation | – | D10 | 04, 08 |
| [Lean](https://lean-lang.org/papers/lean4.pdf) | Kernel-checked proofs as a later export target; reference counting with in-place updates | Interactive proofs in v1 (LLM program-proof success of 4.9–20%) | D10 | 02, 04 |
| Agda / [Idris 2](https://idris2.readthedocs.io/en/latest/tutorial/typesfuns.html) | Idris: hole types shown with the bindings in scope; erasure in types. Agda has no source in `research/` | Full dependent types | D10, D14 | 04, 06, 08 |
| [SPARK](https://docs.adacore.com/spark2014-docs/html/ug/en/usage_scenarios.html) | Graded assurance; proved absence of runtime errors lets checks go | – | D3, D10 | 01, 04 |
| [Hazel](https://hazel.org/) | Typed holes; type definitions as the context agents use most | – | D14 | 06 |
| [Roc](https://roc-lang.org/docs/main/langref/functions/) | Built-in `Encode`/`Decode`; index-based cached compiler data; a pure/effectful split | A single coarse effect color | D6, D7, D12 | 03, 07, 08, perf |
| [Gleam](https://gleam.run/frequently-asked-questions/) | Typed externals with an unchecked foreign side; the whole toolchain shipped at 1.0 | Refusing type classes outright (Tide keeps traits under a ceiling) | D4, D7, D14 | 05, 08, 09 |
| [Elm](https://elm-lang.org/news/compiler-errors-for-humans) | Diagnostics that show the user's own code | Closing the FFI escape hatch; breaking churn | D7, D14 | 08, 09 |
| [Unison](https://www.unison-lang.org/docs/the-big-idea/) | Content-hashed definitions with cached checks | A content-addressed store in place of source text | D12, D14 | 06, 07 |
| [Vale](https://vale.dev/download) | Generational handles, in `Arena` only | Generation checks on every dereference; group borrowing | D2 | 02, perf |
| [Lobster](https://aardappel.github.io/lobster/memory_management.html) | Evidence that borrow inference removes about 95% of RC operations | Silent RC fallback; specialization by ownership kind | D2 | 02 |
| [Morphic](https://2026.splashcon.org/details/oopsla-2026/22/Fully-Automatic-Type-Inference-for-Borrows-with-Lifetimes) | Evidence that fully automatic borrow inference works in a pure language | Inference across signatures; hidden RC | D2 | 02 |
| [OxCaml](https://oxcaml.org/documentation/) | Non-escaping `local` values; mode crossing as an implementation technique | Nine mode axes in front of users | D2, D8 | 02, 07 |
| [Cyclone](http://cyclone.thelanguage.org/) / [Verona](https://github.com/microsoft/verona) | Cyclone: region defaults plus local inference make annotations rare. Verona: cowns, which shaped draft 1's shared cells (now `Shared<T>`) | Cyclone's GC dependence; Verona is research-only | D2, D8 | 02, 10, perf |
| [Carbon](https://docs.carbon-lang.dev/toolchain/docs/) | Data-oriented compiler with integer handles; checked generics; the coherence rationale | C++ interop; "safe shared mutation" | D4, D12 | 02, 03, 08, 09 |
| [MoonBit](https://www.moonbitlang.com/blog/moonbit-ai) | Mandatory top-level signatures; evidence that Rust-like signatures transfer to models | Syntax of its own | D1, D3 | 05, 06, 09 |
| [TypeScript](https://devblogs.microsoft.com/typescript/typescript-native-port/) | The 10x native port as proof that checker speed is implementation work; typed languages gaining from AI | `any`-style escape hatches, which agents overuse (here counted trust holes) | D11, D12 | 03, 05, 09 |
| [Midori](https://joeduffyblog.com/2016/02/07/the-error-model/) | Bugs abandon the task; recoverable errors are values | Exceptions, including Java-style checked exceptions | D5 | 08 |
| [Capslock](https://github.com/google/capslock) | The authority diff on upgrade as a high-signal review event | After-the-fact analysis that over-approximates | D7, D11, D13 | 07, 09 |
| [cargo-cackle](https://github.com/cackle-rs/cackle) | Per-package grants for APIs and `unsafe`; sandboxed build scripts | ACLs layered on an ambient language | D7, D13 | 07, 09 |
| [Cranelift](https://github.com/bytecodealliance/wasmtime/tree/main/cranelift) | The development backend: about 10x faster compiles than LLVM for about 14% slower code; embeds in a Rust compiler | – | D12 | 03, 09, perf |
| [TPDE](https://arxiv.org/pdf/2505.22610) | Data showing a faster backend alone buys about 17% end to end | The backend as the main speed lever | D12 | 03 |
| [Salsa / rust-analyzer](https://rust-analyzer.github.io/blog/2020/07/20/three-architectures-for-responsive-ide.html) | Query engine with early cutoff and durability; laziness; impls in known modules | Macros that create items | D4, D6, D12 | 03, 08 |
| [Polonius](https://blog.rust-lang.org/2026/08/04/enabling-polonius-alpha-on-nightly) | A later, compatible relaxation of borrow precision | Polonius precision in v0 (2–3x worst-case cost) | D2 | 02, 03, perf |
| [facet](https://github.com/facet-rs/facet) / [serde](https://github.com/serde-rs/serde/issues/2538) | facet: read-only `Shape` reflection. serde: derive-generated monomorphic code, the model for `Encode`/`Decode` | Reflection-driven serialization by default; proc macros; precompiled derive binaries | D6, D13 | 08, perf |

## Papers and Studies

Grouped by topic. Each entry gives the source as the report cites it and what
Tide used it for.

### Memory and Ownership

- [The Borrow Checker Within](https://goals.rust-lang.org/2026/roadmap-borrow-checker-within.html), Rust Project Goals, 2026. What Rust still cannot express (view types, place-based lifetimes); origins and field borrows answer it (D2).
- [Tree Borrows](https://plf.inf.ethz.ch/research/pldi25-tree-borrows.html), Villani, Hostert, Dreyer, Jung, PLDI 2025. Defining unsafe aliasing late took years; define it with the language.
- [Implementation Strategies for Mutable Value Semantics](https://www.jot.fm/issues/issue_2022_02/article2.pdf), Racordon et al., JOT 2022. Second-class references and their copying costs (D2).
- [Perceus](https://www.microsoft.com/en-us/research/uploads/prod/2020/11/perceus-tr-v4.pdf), Reinking, Xie, de Moura, Leijen, PLDI 2021. Reference counting made cheap, and the cost of always-atomic counts.
- [Counting Immutable Beans](https://arxiv.org/abs/1908.05647), Ullrich and de Moura, IFL 2019. Lean's RC with in-place updates.
- [Fully-Automatic Type Inference for Borrows with Lifetimes](https://2026.splashcon.org/details/oopsla-2026/22/Fully-Automatic-Type-Inference-for-Borrows-with-Lifetimes), Brandon et al., OOPSLA 2026. Morphic's annotation-free borrow inference.
- [Region-Based Memory Management in Cyclone](https://www.cs.umd.edu/projects/cyclone/papers/cyclone-regions.pdf), Grossman et al., PLDI 2002. Region defaults kept annotations to 6% of an 8% port diff.
- [Reference Capabilities for Flexible Memory Management](https://www.microsoft.com/en-us/research/publication/reference-capabilities-for-flexible-memory-management/), Arvidsson et al., OOPSLA 2023. Verona's isolated regions.
- [Mode Crossing](https://icfp26.sigplan.org/details/icfp-2026-icfp-papers/9/Mode-Crossing), Peters et al., ICFP 2026. How OxCaml cut false rejections from its modes.
- [Generational References](https://verdagon.dev/blog/generational-references), Ovadia. Vale's 10.84% checking cost (D2).

### Compile Speed

- [Rust compiler performance survey 2025 results](https://blog.rust-lang.org/2025/09/10/rust-compiler-performance-survey-2025-results), Rust Project, 2025. Rebuild latency is Rust's top complaint (D12).
- [Go at Google](https://go.dev/talks/2012/splash.article), Pike, 2012. Export data and direct-import reading as the main speed lever (D12).
- [The Rust compilation model calamity](https://www.pingcap.com/blog/rust-compilation-model-calamity/), Anderson (PingCAP). Root causes of slow Rust builds: monomorphization, macros, build scripts, the crate as unit.
- [How our Rust-to-Zig rewrite is going](https://rtfeldman.com/rust-to-zig), Feldman, 2026. Roc's head-to-head rebuild numbers.
- [Generic Go to Go](https://ar5iv.labs.arxiv.org/html/2208.06810), OOPSLA 2022. Dictionary passing: smaller code, 28–43% slower.
- [Design and Implementation of Generics for the .NET CLR](https://www.microsoft.com/en-us/research/publication/design-and-implementation-of-generics-for-the-net-common-language-runtime/), Kennedy and Syme, PLDI 2001. Shared code plus specialization as precedent.
- [TPDE](https://arxiv.org/pdf/2505.22610), arXiv 2505.22610. A backend 8–24x faster than LLVM -O0 yields 17% end to end.
- [Three architectures for a responsive IDE](https://rust-analyzer.github.io/blog/2020/07/20/three-architectures-for-responsive-ide.html), rust-analyzer, 2020. Language rules that make laziness possible (D4, D12).
- [How much code does that proc macro generate?](https://nnethercote.github.io/2025/06/26/how-much-code-does-that-proc-macro-generate.html), Nethercote, 2025. Proc macros tripled code size (D6).
- [A 10x faster TypeScript](https://devblogs.microsoft.com/typescript/typescript-native-port/), Microsoft. Checker speed from implementation alone.

### Verification and LLM Provers

- [A benchmark for vericoding](https://arxiv.org/abs/2509.22908), Bursuc et al., 2025. Dafny, Verus, and Lean success rates (D10).
- [AlgoVeri](https://arxiv.org/abs/2602.09464), Zhao et al., ICML 2026. The same ranking on identical contracts.
- [VeruSAGE](https://arxiv.org/abs/2512.18436), Yang et al., 2025–26. Agent proof rates and verifier cheating (D11).
- [VERINA](https://arxiv.org/abs/2505.23135), Ye et al. Specs sound and complete 52.3% of the time, proofs 4.9% (D10 spec-quality checks).
- [AutoVerus](https://arxiv.org/abs/2409.13082), Yang et al., OOPSLA 2025. Verus-Bench proof generation.
- [Verus: A Practical Foundation for Systems Verification](https://www.microsoft.com/en-us/research/publication/verus-a-practical-foundation-for-systems-verification/), Lattuada et al., SOSP 2024. Per-function SMT fast enough to rerun every iteration (D12).
- [Mariposa](https://www.andrew.cmu.edu/user/bparno/papers/mariposa.pdf), Zhou et al., FMCAD 2023. SMT instability and how design controls it (D10 solver rules).
- [Flux: Liquid Types for Rust](https://arxiv.org/abs/2207.04034), Lehmann et al., PLDI 2023. Ownership makes verification cheaper.
- [RustHornBelt](https://people.mpi-sws.org/~dreyer/papers/rusthornbelt/paper.pdf), Matsushita et al. Soundness of the prophecy model behind `final(x)`.
- [How We Built Cedar](https://arxiv.org/abs/2407.01688), Disselkoen et al., FSE 2024. Proofs plus differential testing against an executable model.
- [Clover](https://arxiv.org/abs/2310.17807), Sun, Sheng, Padon, Barrett. Consistency checks catch wrong specs.
- [AI will make formal verification go mainstream](https://martin.kleppmann.com/2025/12/08/ai-formal-verification.html), Kleppmann, 2025. The challenge moves to specifications.

### LLM Coding Empirics

- [MultiPL-E](https://arxiv.org/abs/2208.08227v4), Cassano et al. Popularity predicts model performance.
- [No-resource languages (Gleam, MoonBit)](https://arxiv.org/abs/2606.16827), Giagnorio, Martin-Lopez, Bavota, 2026. Syntax dominates failures in unseen languages (D1).
- [ILA-agent / Cangjie-bench](https://arxiv.org/abs/2602.06976), 2026. Execution feedback is the strongest lever (D14).
- [CangjieBench](https://arxiv.org/html/2603.14501). A 2,146-token syntax reference beats documentation retrieval (D14 reference budget).
- [MultiPL-T](https://arxiv.org/abs/2308.09895), Cassano et al. Test-validated translated corpora lift low-resource languages.
- [Type-Aware Constraining for LLM Code Generation](https://arxiv.org/abs/2504.09246), Mündler et al., PLDI 2025. Most compile errors are type errors (D3 required signatures).
- [Unreliable in Practice?](https://arxiv.org/abs/2608.00661), Nogueira, Vieira, Campos, ISSRE 2026. Rust error classes from LLMs (D2, D3).
- [RustAssistant](https://arxiv.org/abs/2308.05177), Deligiannis et al., ICSE 2025. Localized diagnostics drive repair (D14).
- [SafeTrans](https://arxiv.org/abs/2505.10708). Trait errors resist repair (D4).
- [SWE-bench Multilingual](https://www.swebench.com/multilingual.html) and [Claw-SWE-Bench](https://arxiv.org/html/2606.12344v2), 2026. Agents do best in Rust (thesis).
- [Mining Type Constructs in AI-generated code](https://arxiv.org/abs/2602.17955). Agents use `any` 9x more than humans (D11).
- [Package hallucinations](https://arxiv.org/abs/2406.10279), Spracklen et al., USENIX Security 2025. Agents invent dependencies (D13).

### Agent Tooling

- [SWE-agent](https://arxiv.org/html/2405.15793v3), NeurIPS 2024. Lint-on-edit stops cascading failures (D14).
- [Typed-hole context in Hazel](https://arxiv.org/html/2409.00921v1), OOPSLA 2024. Types as the most valuable agent context (D14).
- [CodeStruct](https://arxiv.org/abs/2604.05407), ACL 2026. Structured edits help weak models more than frontier ones.
- [LSP token study](https://arxiv.org/html/2608.13568v1), 2026. Agents prefer grep for localization; text stays primary.
- [AGENTS.md outperforms skills](https://vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals), Vercel, 2026. An 8 KB version-matched index (D14).
- [ImpossibleBench](https://www.lesswrong.com/posts/qJYMbrabcQqCZ7iqm/impossiblebench-measuring-reward-hacking-in-llm-coding-1). Agents edit tests; protect oracles (D11, D14).
- [Go fix](https://go.dev/blog/gofix), Go blog. Compiler-owned migrations, because models resist new idioms (D14).
- [rustc JSON diagnostics](https://doc.rust-lang.org/nightly/rustc/json.html). Machine-applicable fix labels (D14).
- [A language for agents](https://lucumr.pocoo.org/2026/2/9/a-language-for-agents/), Ronacher, 2026. Practitioner brief: explicit effects, results, one check command.

### Effects and Capabilities

- [Effect handlers, evidently](https://www.microsoft.com/en-us/research/publication/effect-handlers-evidently/), Xie and Leijen, ICFP 2020. Authority effects need no continuations (D7).
- [Effects as capabilities](https://ps.informatik.uni-tuebingen.de/publications/brachthaeuser20effects/), Brachthäuser et al., OOPSLA 2020. Capability passing as effect typing.
- [Retrofitting effect handlers onto OCaml](https://arxiv.org/pdf/2104.00250), Sivaramakrishnan et al., PLDI 2021. Cost of untyped one-shot effects.
- [Boolean unification for Flix](https://users-cs.au.dk/magnusm/papers/oopsla20/paper_b.pdf), Madsen and van de Pol, OOPSLA 2020. The inference cost of a rich effect algebra.
- [What's in the box](https://www.arxiv.org/abs/2509.07609), Xu et al., OOPSLA 2025. Capture sets in generic containers need new theory.
- [Soundly handling linearity](https://arxiv.org/pdf/2307.09383), Tang et al., POPL 2024. Multi-shot handlers break linear types (D7 rejects handlers).
- [How capabilities work in Austral](https://borretti.me/article/how-capabilities-work-austral), Borretti. Root capability from `main` (D7).
- [Joe-E purity](https://people.eecs.berkeley.edu/%7Edaw/papers/pure-ccs08.pdf), Finifter et al., CCS 2008. Purity from immutable argument types (`Pure`).
- [Wyvern](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ECOOP.2017.20), Melicher et al., ECOOP 2017. Authority reviewable from interfaces (D11).
- [JEP 411](https://openjdk.org/jeps/411), OpenJDK. Why the Java SecurityManager failed.
- [CaMeL](https://arxiv.org/abs/2503.18813), Debenedetti et al. Run-time capability tagging against prompt injection.
- [cargo-scan](https://arxiv.org/abs/2602.06466v1), Zoghbi et al. Risky operations concentrate in about 3% of crates.

### Type Systems

- [Implementing Swift Generics](https://llvm.org/devmtg/2017-10/slides/Pestov-McCall-ImplementingGenerics.pdf), Pestov and McCall, LLVM Dev Meeting 2017. Witness tables and their cliff (D12).
- [Modular Type Classes](https://www.cs.cmu.edu/~rwh/papers/mtc/short.pdf), Dreyer, Harper, Chakravarty. Traits versus modules (D4).
- [Bidirectional typing](https://arxiv.org/abs/1908.05839), Dunfield and Krishnaswami. Local inference with good error locality (D3).
- [Rust blog on the next trait solver](https://blog.rust-lang.org/2026/08/21/enabling-next-solver-on-nightly), Rust Project, 2026. Four years of solver work (D4 ceiling).
- [The Error Model](https://joeduffyblog.com/2016/02/07/the-error-model/), Duffy, 2016. Abandonment for bugs (D5).
- [RFC 560: integer overflow](https://rust-lang.github.io/rfcs/0560-integer-overflow.html), Rust. Overflow is a program error (D3).
- [Deser](https://lucumr.pocoo.org/2026/9/29/deser/), Ronacher, 2026. Dynamic-dispatch serialization trade-offs (D6).
- [IDEs and macros](https://rust-analyzer.github.io/blog/2021/11/21/ides-and-macros.html), rust-analyzer, 2021. Macros cost about X² in an IDE (D6).
- [Things Zig comptime won't do](https://matklad.github.io/2025/04/19/things-zig-comptime-wont-do.html), matklad, 2025. Hermetic, cacheable compile-time evaluation (D6).
- [Trojan Source](https://trojansource.codes/). Bidi controls in source text.

### Concurrency and Simulation

- [Notes on structured concurrency](https://vorpus.org/blog/notes-on-structured-concurrency-or-go-statement-considered-harmful/) and [Timeouts and cancellation for humans](https://vorpus.org/blog/timeouts-and-cancellation-for-humans/), Smith. Nurseries and level-triggered cancellation (D8).
- [Why async Rust](https://without.boats/blog/why-async-rust/), without.boats. Pin as a shipping compromise; why Rust dropped green threads.
- [Cancelling async Rust](https://sunshowers.io/posts/cancelling-async-rust/), sunshowers. Cancellation by drop as a correctness hazard (D8).
- [Rust concurrency bug study](https://cseweb.ucsd.edu/~yiying/RustStudy-PLDI20.pdf), Qin et al., PLDI 2020. Double locking dominates blocking bugs (D8 `Shared::with`).
- [Go data races at Uber](https://arxiv.org/abs/2204.00764), Chabbi and Ramanathan, PLDI 2022. Uncolored but racy.
- [Behaviour-oriented concurrency](https://www.microsoft.com/en-us/research/publication/when-concurrency-matters-behaviour-oriented-concurrency/), Cheeseman et al., OOPSLA 2023. Atomic multi-resource access.
- [Parallel Programming Must Be Deterministic by Default](https://static.usenix.org/event/hotpar09/tech/full_papers/bocchino/bocchino_html/), Bocchino et al., HotPar 2009. The principle behind D9.
- [Heartbeat scheduling](https://www.chargueraud.org/research/2018/heartbeat/heartbeat.pdf), Acar et al., PLDI 2018. Granularity without tuning (D9, deferred).
- [FoundationDB testing](https://apple.github.io/foundationdb/testing.html), Apple. Whole-cluster simulation (D15).
- [VOPR](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/internals/vopr.md), TigerBeetle. Seed and commit replay (D15).
- [Deterministic simulation testing at S2](https://s2.dev/blog/dst), S2. Library-level simulation leaks through dependencies (D7, D15).
- [Go synctest](https://go.dev/blog/synctest), Go blog. Virtual time that advances when every task blocks (D15).

### Adoption and Interop

- [Building a C compiler](https://anthropic.com/engineering/building-c-compiler), Anthropic. A near-perfect oracle makes agent-built compilers work (D12 interpreter).
- [Bun in Rust](https://bun.com/blog/bun-in-rust), Bun. A 535k-line agent port; compiler errors as the feedback loop.
- [Rust in Android: move fast, fix things](https://blog.google/security/rust-in-android-move-fast-fix-things/), Google, 2025. Memory-safety payoff at platform scale.
- [Go supply-chain security](https://go.dev/blog/supply-chain), Go blog. No code runs at fetch or build (D13).
- [Our plan for a more secure npm supply chain](https://github.blog/security/supply-chain-security/our-plan-for-a-more-secure-npm-supply-chain/), GitHub. Install scripts as the worm vector; trusted publishing (D13).
- [Interop problem statement](https://github.com/rustfoundation/interop-initiative/blob/main/problem-statement.md), Rust Foundation. Interop stops at the C type system (D13 shims).
- [crABI tracking issue](https://github.com/rust-lang/rust/issues/111423), Rust. No stable richer ABI yet.
- [Zngur](https://github.com/HKalbasi/zngur) and [cxx](https://cxx.rs/). Explicit instantiation into C-ABI shims (D13).
- [HACL\*](https://github.com/hacl-star/hacl-star). Verified code shipped as emitted C (D12 release backend).
- [New systems languages critique](https://compiler-meetup.cs.illinois.edu/slides/New-Systems-Languages.pdf). V's overpromising as the cautionary tale.

### Bend

- [BendRT: A Parallel Runtime for CPUs and GPUs](https://github.com/bendlang/bend/blob/main/paper/BendRT.pdf), Taelin et al., 2026. Bend 2 dropped interaction nets at run time.
- [BendTT: An Affine Dependent Type Theory](https://github.com/bendlang/bend/blob/main/paper/BendTT.pdf), Taelin et al., 2026. The live/dead split and once-only closures.
- [HVM2: A Parallel Evaluator for Interaction Combinators](https://github.com/HigherOrderCO/HVM2/blob/main/paper/HVM2.pdf), Taelin. HOC's own single-thread overhead figures.
- [Interaction Combinators](https://doi.org/10.1006/inco.1997.2643), Lafont, Information and Computation, 1997. The underlying model.
- [Benchmark da linguagem Bend](https://sol.sbc.org.br/index.php/sscad_estendido/article/view/30969), Passos, Passos, Bianchini, SSCAD 2024. Independent Bend 1 slowdown.
- [Breaking Bend: Benchmarking the HVM](https://blog.speedfox.co.uk/), Speedfox, 2024. Independent Bend 1 slowdown.
- [New AI-Focused Language Just Released: Bend 2](https://akitaonrails.com/en/2026/09/19/new-ai-language-just-released-bend-2/) and [bend-tests findings](https://github.com/akitaonrails/bend-tests/blob/main/docs/findings.md), Akita, 2026. Porting Rust tools to Bend 2.
- [Bend vs SPARK](https://blog.liampwll.com/posts/bend_vibe_coding/), Powell. Auto-active proof discharged what Bend proved by hand.
- [Issue #813](https://github.com/bendlang/bend/issues/813), Bend. The missing native library target that shapes D16's two stages.
- [Bend EFFECTS guide](https://github.com/bendlang/bend/blob/main/guide/EFFECTS.md). Bend's only FFI and its lack of an ABI promise.

## Measurements That Changed Decisions

Decision IDs are draft 3.1's. `reviews/round1-performance.md` numbered its
targets against draft 1, so its decision numbers differ from these.

| Measurement | Number | Source | Decision it affected |
| --- | --- | --- | --- |
| Vericoding success, union of 9 models over 12,504 specs | Dafny 82.2%, Verus 44.2%, Lean 26.8% | research/04, 05 | D10: SMT contracts with Dafny and Verus clause names; no interactive proofs in v1 |
| Verifier cheating without a cheat checker | 2–14% of tasks; under 1.5% with a checker | research/04 (VeruSAGE) | D11: trust holes counted transitively; approval outside the agent's write scope |
| Spec quality | About 9% of specs too weak even when solved; spec generation 52.3% sound and complete (VERINA) | research/04 | D10: satisfiability, vacuity, and mutation checks on specs |
| RustAssistant repair rate by prompt format (GPT-3.5) | 10.74% to 73.70% with localization and fix descriptions | research/05 | D14: exact spans, related spans, fix plans |
| Execution feedback on an unfamiliar language | Sonnet 4.5 on Cangjie 45.2% to 81.9% | research/05, usability | D12, D14: interpreter first; `tide test` and `tide eval` |
| Syntax reference in context | GPT-5 4.8% to 50.4% with 2,146 tokens; 83% of failures borrowed syntax | research/09 | D1; D14's 25,000-token reference budget |
| Rust-habit census, 912 crates, 6.08M lines | 76.8% of files use a same-meaning construct draft 1 respelled; lifetimes in 44.2%, `as` in 26.9% | usability | D1: Rust spelling, built-in `!` forms, legacy lifetimes accepted; D3: lossless `as` |
| Agents on real repositories | Rust best on SWE-bench Multilingual (58.1%) and for 7 of 11 models on Claw-SWE-Bench | research/05 | Thesis: keep Rust's semantics and spelling |
| LLM Rust errors | Ownership and lifetime 16.7% of compile errors; overflow checks caught 43.9% of runtime errors | research/02, 05 | D2 origins; D3 traps in every profile |
| Guided repair by error class | Borrow 74.2%, trait 58.7% | research/05, usability | D4: trait power ceiling |
| Emitted Rust, 100k-line crate | Check 13.1 s; clean debug 53.2 s; one-function edit 17.2 s; release 136.9 s | perf | D12: Rust emission rejected as release backend; Cranelift-class dev backend; C-emitting release backend |
| Witness tables vs monomorphization | 5.6–23.9x slower (sum); 3.3–8.0x (iterator pipeline) | perf | D12: monomorphize in every profile, overriding research/03's dictionaries-by-default |
| Overflow traps vs wrapping | `i32` sum 5.3x; `u8` add 10.8–27.9x cache-resident; `.text` +18.4%; deferred trap 0.8–1.4x on `u8` | perf | D3: precise traps in v0, deferred traps a later tier, proved facts may delete checks |
| Stackful task stacks | 2 VMAs and 4 KiB RSS per task; default `vm.max_map_count` caps a process near 32k tasks | perf | D8: stackless state machines in compiled code |
| Lowering origins to Rust | rustc rejects three accepted patterns (E0499, E0502) | perf | D2: NLL precision plus field borrows; Polonius later |
| Reflection serialization | facet-json 5–7x slower than serde_json | perf | D6: `Encode`/`Decode` derives |
| Roc rebuilds | Rust 3.4 s after an edit; Zig 0.035 s | research/03 | D12: 30 ms edit-to-diagnostics target; per-function caching |
| Fast backend inside an unchanged language | Cranelift 5–15% end to end in rustc; TPDE 17% | research/03 | D12: front-end rules matter more than the backend |
| Proc-macro expansion | Code size more than tripled; trimming saved about 20% | research/03 | D6: no user macros |
| Bend 1 / HVM2 | n-body 255.28 s vs 0.82 s for C+OpenMP; HVM2 5x to 100x slower than GHC single-threaded | research/01 | D16: no HVM backend; D9: native fork-join |
| Bend 2 port of a Rust tool | About 4x slower than Rust; 2 of 3 real bugs were silent U32 wraparound | research/01 | D3 checked arithmetic; D16 closed type map |
| Vale generation checks | +10.84% (RC +25.29%) | research/02 | D2: no runtime-checked references |
| Capability changes in upgrades | Fewer than 2% of Go updates add a capability | research/07, usability | D7, D13: lockfile authority diff fails until approved |
| Version-matched docs | Next.js eval 53% to 100% with an 8 KB index; skills not invoked 56% of the time | research/06 | D14: card under 8 KB |
| Multi-file edits | Aider 17.7% single-file vs 7.9% at five or more files | research/05, usability | D7: write-only sinks are not capabilities |
| Test tampering | GPT-5 exploited mutated tests 76% of the time | research/06 | D11, D14: `#[protected]` tests |

## Source Quality

Reports 02 through 10 say the shared web-search budget ran out partway
through, after which sources were fetched directly from known primary URLs.
Claims that rest on summaries or unverified prior knowledge are flagged in
place: "unverified" in 02, "secondary" in 03, 06, 08, and 10, "Not verified
here" in 04, and `[background]` in 07; 05 and 09 name the threads left thin.
Treat flagged claims as secondary or background evidence, not as findings.
