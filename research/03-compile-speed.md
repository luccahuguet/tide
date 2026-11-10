# 03 — Compile speed: what makes compilers fast or slow, and which language decisions set the ceiling

Status: research note, 2026-10-19. Every factual claim has a URL. Lines marked **Inference** are my reasoning from the evidence, not measured facts. The session's shared web-search budget ran out partway through, so later sources were fetched directly from known primary URLs. The Jai, Carbon per-line, and wild-vs-mold figures are secondary and marked that way. Section 11 was added on 2026-11-09 (bead `fx-zmr`) from primary sources fetched that day. It marks as self-reported every figure a project measured on itself.

## Question

Which language-design decisions set the upper limit on compile and check speed, and which are only implementation quality? What compiler architecture would let a Rust-class language (memory-safe, natively fast, strongly typed, with optional proofs) reach about **100 ms incremental type-check** and about **1 s debug builds** on a 100k-LOC project, with a separate optimizing release path? AI coding agents run edit→check→fix loops, so check latency multiplies agent throughput directly. This note is about that loop.

## Findings (cited)

### 1. Where rustc's time goes, and why it is hard to fix

**User-visible pain.** The 2025 Rust compiler performance survey drew more than 3,700 responses. 55% of respondents wait more than 10 seconds for a rebuild of their most troublesome project. Incremental rebuilds after small changes were the most common complaint. About 45% of former Rust users named compile times as at least one reason they stopped. About 87% read errors through editor annotations, and a third of those called waiting for annotations a big blocker ([survey results](https://blog.rust-lang.org/2025/09/10/rust-compiler-performance-survey-2025-results)).

**No single bottleneck.** No authoritative, representative frontend/backend split exists. The pieces of evidence that do exist each point at a different stage:

- **Frontend.** In the parallel-frontend announcement's example, a *release* build of Cargo spent 10.2 s in the serial frontend and 6.2 s in the backend ([Rust blog, 2023](https://blog.rust-lang.org/2023/11/09/parallel-rustc/)). Nethercote wrote that "rustc's current front-end is completely serial" ([2023](https://nnethercote.github.io/2023/07/11/back-end-parallelism-in-the-rust-compiler.html)). The parallel frontend reached nightly in 2023 with claimed savings of up to 50% ([Rust blog](https://blog.rust-lang.org/2023/11/09/parallel-rustc/)). It was still not stable in 2026, with correctness work (deadlocks, inconsistent query-cycle errors) outstanding ([2026 goal](https://goals.rust-lang.org/2026/parallel-front-end.html)). One independent 16-core test of a debug build saw no gain from `-Zthreads` ([neosmart](https://neosmart.net/blog/benchmarking-rust-compilation-speedups-and-slowdowns-from-sccache-and-zthreads/)).
- **LLVM.** Popov's LLVM keynote lists three structural causes: the "compilation unit is crate," "generics produce huge amounts of IR," and "LLVM is slow." The LLVM 19 upgrade alone cut 11–16% from rustc's best-improved benchmarks, most of them *debug* builds, which shows how much of debug-build time is LLVM ([Popov, LLVM Dev Mtg 2024](https://llvm.org/devmtg/2024-10/slides/keynote/Popov-Rust_Heart_LLVM.pdf)). The 2026 TPDE goal says plainly that LLVM "has poor compile-time performance in debug mode" ([goal](https://goals.rust-lang.org/2026/tpde.html)).
- **Linking.** Before LLD became the default, "roughly half" of a ripgrep debug build's time on Linux went to the linker. LLD cut link time 7x and end-to-end incremental rebuild time by 40% ([Rust blog](https://blog.rust-lang.org/2025/09/01/rust-lld-on-1.90.0-stable/)).
- **Monomorphization.** Generics are instantiated per crate, so sibling crates duplicate instances. matklad calls this "accidentally-quadratic compilation times across many crates" and advises "do not put generic code at the boundaries between the crates" ([Fast Rust Builds](https://matklad.github.io/2021/09/04/fast-rust-builds.html)). The 2026 polymorphization goal projects a 30%–2x speedup of "post-mono trait solving, codegen, and linking" and says it could remove "up to half of generated instances" ([goal](https://goals.rust-lang.org/2026/polymorphization.html)). The earlier polymorphization attempt only erased *unused* type parameters. It stalled because `TypeId` "broke parametricity" and closures complicated the analysis (same source).
- **Proc macros.** In one ~100k-line project, proc-macro expansion "more than tripled" code size. Trimming macros cut compile time by about 20%. A single `#[tracing::instrument]` expanded to 232 lines ([Nethercote, 2025](https://nnethercote.github.io/2025/06/26/how-much-code-does-that-proc-macro-generate.html)). Proc macros also block pipelining, because dependents wait until the macro crate is fully compiled, and `syn` sits on the critical path ([matklad](https://matklad.github.io/2021/09/04/fast-rust-builds.html)).
- **Borrow checking.** It is not the big-ticket item. NLL "shipped with very little hit to compile times" after its memory problems were fixed ([Nethercote, 2020](https://blog.mozilla.org/nnethercote/2020/09/08/how-to-speed-up-the-rust-compiler-one-last-time/)). The more precise Polonius alpha does more work and slows some crates, such as serde, by single-digit percentages ([Nethercote, Sept 2026](https://nnethercote.github.io/2026/09/30/how-to-speed-up-the-rust-compiler-in-september-2026.html)). **Inference:** an intraprocedural, signature-bounded ownership check is affordable. Whole-program or interprocedural alias analysis would not be.
- **Rebuild granularity.** Edits that leave the public interface unchanged still rebuild every dependent crate: comments, `dbg!` in a non-inlinable function, reordered `use`. That is why "Relink, don't rebuild" exists ([2025 goal](https://goals.rust-lang.org/2025h2/relink-dont-rebuild.html)). The 2026 Fast Builds roadmap adds that incremental compilation misses early phases such as name resolution, and that `check`, `build`, and `test` each redo shared work ([roadmap](https://goals.rust-lang.org/2026/roadmap-fast-builds.html)). The check→build→test loop rebuilds the workspace three times ([goal](https://goals.rust-lang.org/2026/incremental-system-rethought.html)).

**Tuning has diminishing returns.** rustc got about 1.77x faster on one clean-build benchmark between 2022 and 2025 (26.1 s → 14.7 s) ([Kobzol](https://kobzol.github.io/rust/rustc/2025/06/09/why-doesnt-rust-care-more-about-compiler-performance.html)). Mean wall-time gains are now incremental: 6.4% over the year to March 2025 and 4.6% over July–September 2026 ([Mar 2025](https://nnethercote.github.io/2025/03/19/how-to-speed-up-the-rust-compiler-in-march-2025.html), [Sept 2026](https://nnethercote.github.io/2026/09/30/how-to-speed-up-the-rust-compiler-in-september-2026.html)). Nethercote: "profiles are much flatter and obvious improvements are harder... to find" ([2020](https://blog.mozilla.org/nnethercote/2020/09/08/how-to-speed-up-the-rust-compiler-one-last-time/)). Codegen-unit partitioning tuning was a "tar pit" ([2023](https://nnethercote.github.io/2023/07/11/back-end-parallelism-in-the-rust-compiler.html)). Kobzol argues that a data-oriented rewrite of rustc is impractical and that rebuilds should become O(changes), not O(codebase) ([Kobzol](https://kobzol.github.io/rust/rustc/2025/06/09/why-doesnt-rust-care-more-about-compiler-performance.html)). Brian Anderson's retrospective names the root causes: monomorphization, coherence making decomposition into small parallel units hard, macros, build scripts, unwinding bookkeeping, LLVM, and the crate as compilation unit. It concludes that Rust "nearly always" chose run-time over compile-time ([PingCAP](https://www.pingcap.com/blog/rust-compilation-model-calamity/)).

**The most direct head-to-head.** Roc rewrote its ~300k-line Rust compiler in Zig. On the same Intel desktop, Rust 1.97.0 took 25.4 s cold and 3.4 s to rebuild after a parser edit (354k LOC). Zig 0.17 with `--watch -fincremental` took 32.1 s cold and **0.035 s** incremental (464k LOC). Feldman calls 35 ms "a different performance category." He also reports that `-fincremental` was broken on their code in Zig 0.16.0 and works only on x86-64 ([Feldman, July 2026](https://rtfeldman.com/rust-to-zig)).

### 2. Go: dependency discipline as the main speed lever

In 2007, a large Google C++ binary's 4.2 MB of source expanded to more than 8 GB of compiler input after `#include` processing, "a blow-up of 2000 bytes for every C++ source byte." Go's response was mostly about the dependency graph, not the code generator. Unused imports are compile errors. Import cycles are forbidden. Each object file starts with export data, so compiling package A opens "exactly one file" per *direct* import and never reads transitive dependencies. Pike calls this "the single biggest reason why Go compilations are faster than C or C++ compilations." The grammar parses without a symbol table ([Pike, Go at Google](https://go.dev/talks/2012/splash.article)).

Go 1.18 added generics through **GC-shape stenciling plus dictionaries**. One body is compiled per GC shape (types with the same underlying type share one), and a compile-time dictionary carries the type-specific details ([design doc](https://go.googlesource.com/proposal/+/master/design/generics-implementation-dictionaries-go1.18.md)). Generics made the 1.18 compiler about 15–18% slower at first (geomean about 7.7% on later benchmarks). That was back "in line with Go 1.17" by Go 1.20 ([golang/go#49569](https://github.com/golang/go/issues/49569)).

### 3. Zig: own backend, threaded codegen, incremental compilation by patching

- **Own backend.** The self-hosted x86_64 backend became the Debug default in June 2025. `hello.zig` went from 918 ms to 275 ms, and the Zig compiler build from 75 s to 20 s ([devlog 2025](https://ziglang.org/devlog/2025/)). The release notes summarize it as "Debug compilation is 5 times faster" ([0.15.1 notes](https://ziglang.org/download/0.15.1/release-notes.html)). Threading codegen (one semantic-analysis thread, many codegen threads, one linker thread) cut another 27% from the compiler build ([devlog 2025](https://ziglang.org/devlog/2025/)). In Ghostty, a one-line change to libghostty-vt (self-hosted backend) rebuilt in 975 ms, while the LLVM-backed executable still took 16 s ([Hashimoto](https://mitchellh.com/writing/zig-builds-getting-faster)).
- **Incremental compilation.** By 2026 it works with LLVM too, but "can't do anything to speed up the dreaded LLVM Emit Object." Its main win there is getting errors in milliseconds. A March 2026 30k-line type-resolution rework made analysis lazier to reduce "over-analysis" ([devlog 2026](https://ziglang.org/devlog/2026/)). With the new ELF linker, a Tetris clone rebuilds in "around 30ms," and the Zig compiler rebuilds in 228–288 ms after a 36 s initial build (same source).
- **Bounded comptime.** Compile-time evaluation runs under a budget. `@setEvalBranchQuota` defaults to 1000 backward branches before evaluation gives up ([Zig docs](https://ziglang.org/documentation/master/)).

### 4. Jai, Odin, D, Carbon

- **Jai.** The community wiki states the goal "to compile 1M lines of code in <1s (from scratch)" and cites "250,000 lines per second using the x64 backend" from a beta changelog. Jai does full rebuilds with no incremental step, and LLVM is the slower optimizing alternative ([Jai community wiki](https://github.com/Jai-Community/Jai-Community-Library/wiki/Overview)). These are secondary figures for an unreleased compiler.
- **Odin.** It lists "speed of compilation" as a design goal and parses "without a symbol table," but uses LLVM as its backend ([FAQ](https://odin-lang.org/docs/faq/)).
- **D.** It ships a fast-compiling reference compiler (DMD, "Very fast compilation speeds") alongside LLVM- and GCC-based compilers for "Strong optimization" ([dlang downloads](https://dlang.org/download.html)). This is the dev/release backend split.
- **Carbon.** It ranks compilation performance among its top toolchain priorities. It uses vectorized storage with integer handles instead of pointer-linked heap nodes, for allocation and cache reasons ([toolchain docs](https://docs.carbon-lang.dev/toolchain/docs/)). Carruth's 2023 talk argued that conventional compiler data structures are "rooted in 50 year old computer architectures" ([CppNow](https://schedule.cppnow.org/session/2023/modernizing-compiler-design-for-carbons-toolchain/)). The per-line targets usually attributed to that talk (100 ns/line lex+parse, 1 µs/line semantic analysis) appear in a third-party project that cites "an earlier talk" ([do-compiler](https://github.com/v-pan/do-compiler)). Treat them as plausible, unverified targets.

### 5. Type-system decisions: Swift, OCaml, Sorbet, TypeScript, ty

- **Swift.** Type-directed overloading plus bidirectional inference plus polymorphic literals means that "in the worst case, there is no better approach... except to attempt each combination of disjunction choices." The Swift team concedes "it will always be possible to write down a short program" that is too slow to check. They rule out removing overloading or bidirectional inference as too disruptive ([Swift roadmap, 2025](https://forums.swift.org/t/roadmap-for-improving-the-type-checker/82952)). One measured case: a 12-line string expression containing a type error took 42 s before the checker gave up, and the corrected version took 0.19 s ([Hooper](https://danielchasehooper.com/posts/why-swift-is-slow/)).
- **OCaml.** "Type inference doesn't scale to very large codebases that depend on separate compilation." Explicit `.mli` interfaces compile to `.cmi` files, and "recompiling a `mli` signature is much faster than a full compilation" ([Real World OCaml](https://dev.realworldocaml.org/compiler-frontend.html)).
- **Sorbet.** Stripe's Ruby checker covers more than 15M lines and returns errors in milliseconds for 80% of edits. It does this by doing only *local* inference: checking one method never affects another. That makes it "embarrassingly parallel" ([Stripe](https://stripe.dev/blog/sorbet-stripes-type-checker-for-ruby)).
- **TypeScript native port.** Checking VS Code (1.5M LOC) dropped from 77.8 s to 7.5 s, and date-fns (104k LOC) from 6.5 s to 0.7 s. That is about 10x from the implementation alone, with the same language ([Microsoft](https://devblogs.microsoft.com/typescript/typescript-native-port/)).
- **ty (Python, Rust, incremental by design).** After an edit to a central PyTorch file it recomputes diagnostics in 4.7 ms, against 386 ms for Pyright ([Astral](https://astral.sh/blog/ty)).

### 6. Generics implementation strategies

| Strategy | Example | Compile-time effect | Runtime effect |
|---|---|---|---|
| Full monomorphization | Rust, C++ | IR scales with instantiations. Exponential blow-up is possible. | Best |
| Shape/layout sharing plus dictionary | Go 1.18; Rust polymorphization goal | Fewer instances | Indirect calls through the dictionary |
| Pure dictionary passing | Haskell default; Oxford "dict" translation | Smallest, polynomial | Slower |
| Dictionary by default plus opt-in specialization | GHC `SPECIALIZE`/`INLINABLE`; Swift witness tables plus SIL specialization | Pay only where requested | Near-mono where specialized |

A controlled Go study measured this on real-world benchmarks. Dictionary passing produced the smallest code (577 instructions geomean, against 651–703 for the monomorphizing translators) at 28–43% slower execution (2.45 s against 1.71–1.91 s). Compile times on these small benchmarks were about the same. On a pathological program, monomorphizing translators, Go 1.18 included, grew *exponentially* in code size and compile time while dictionary passing stayed polynomial. The monomorphized version also became slower at run time once its binary reached 6.3 MB, from cache misses ([Generic Go to Go, OOPSLA 2022](https://ar5iv.labs.arxiv.org/html/2208.06810)). GHC specializes overloaded functions automatically within a module, and across modules only for `INLINABLE` definitions, whose bodies it stores in the interface file ([GHC guide](https://downloads.haskell.org/ghc/latest/docs/users_guide/exts/pragmas.html)). Swift shows the cost of the opposite default. Unspecialized generic code across resilient boundaries hits "an obnoxious performance cliff," and `@inlinable` puts bodies into module interfaces to recover speed ([Swift evolution pitch](https://lists.swift.org/pipermail/swift-evolution/Week-of-Mon-20171002/040074.html), [swift-dev](https://lists.swift.org/pipermail/swift-dev/Week-of-Mon-20170731/005039.html)). .NET generics share code by representation and specialize just in time, "achieving performance almost matching hand-specialized code" ([Kennedy & Syme, PLDI 2001](https://www.microsoft.com/en-us/research/publication/design-and-implementation-of-generics-for-the-net-common-language-runtime/)).

### 7. Backends

- **Cranelift (`rustc_codegen_cranelift`).** In February 2026 on Zed's incremental builds, it was **5% faster than LLVM with no debuginfo and 12% faster with line tables**. Across the benchmark suite the average was 10–15% per crate. The project now targets "a 2x speedup inside" the backend, through an in-memory daemon of per-function machine code, JIT, and *interpreting* rarely called functions ([2026 goal](https://goals.rust-lang.org/2026/improve-cg_clif-performance.html)). In 2025, codegen time fell about 20%, but clean builds only about 5% ([2025 goal](https://goals.rust-lang.org/2025h2/production-ready-cranelift.html)). On its home turf, Wasm, Cranelift compiles "approximately an order of magnitude faster" than LLVM-based WAVM, with code about 14% slower ([Cranelift README](https://github.com/bytecodealliance/wasmtime/tree/main/cranelift)).
- **TPDE.** A single analysis pass plus one combined pass for instruction selection, register allocation, and encoding compiles LLVM-IR **8–24x faster than LLVM -O0** with code within ±9% of -O0. Code size is 43–49% larger. The decisive caveat: on SPECint via Clang, the backend was only 15% of total time with LLVM and 2% with TPDE, so the **end-to-end speedup was 17%** ([TPDE paper](https://arxiv.org/pdf/2505.22610)). The Rust 2026 TPDE goal hopes for only "in the order of 50%" debug-build gains on backend-heavy builds ([goal](https://goals.rust-lang.org/2026/tpde.html)).
- **Copy-and-patch.** Stitching precompiled stencils generates code two orders of magnitude faster than LLVM -O0, with code 14% faster than -O0 for query compilation ([Xu & Kjolstad](https://arxiv.org/abs/2011.13127)). As a general LLVM-IR backend it was 18.6x faster than -O0, but its code was 2.38x slower and 4.44x larger ([TPDE paper](https://arxiv.org/pdf/2505.22610)).
- **QBE.** It aims for "70% of the performance of industrial optimizing compilers in 10% of the code." This is a goal, not a benchmark ([QBE](https://c9x.me/compile)).
- **Single-pass baselines.** V8's Liftoff skips IR entirely and produces code 50–70% slower than TurboFan on Unity benchmarks ([V8](https://v8.dev/blog/liftoff)). This is the model for a tiered dev/release split.

### 8. Incremental and query architectures

- **rustc** saves a query DAG and colors queries red or green on the next run. "Try-mark-green" proves a query unchanged without re-running it, and result hashes give early cutoff ([rustc dev guide](https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation.html)).
- **rust-analyzer** (Salsa) found that "laziness — the ability to skip huge swaths of code altogether" matters more than incrementality. Its lessons for language design: map each file to fully-qualified names, forbid or restrict macros that create top-level items, handle conditional compilation in name resolution rather than parsing, and avoid impls that "can be roughly anywhere." Rust's crate is "still too big" as a unit of laziness ([three architectures](https://rust-analyzer.github.io/blog/2020/07/20/three-architectures-for-responsive-ide.html)). Merely *validating* the stdlib's queries after each edit cost ~300 ms until **durability levels** let whole subgraphs be skipped ([durable incrementality](https://rust-analyzer.github.io/blog/2023/07/24/durable-incrementality.html)).
- **Roslyn's** red-green trees hold immutable, parent-free "green" nodes that store widths rather than positions. They are reused across edits, roughly O(log n) of the tree is rebuilt, and an on-demand "red" facade adds parents and positions. The cost is up to twice as many objects ([Lippert](https://ericlippert.com/2012/06/08/red-green-trees/)).
- **Roc** loads cached compiler data with zero parsing, at roughly memcpy speed. Feldman: that "only works because" the data uses indices rather than pointers ([Feldman](https://rtfeldman.com/rust-to-zig)).

### 9. Linking, hot reload, separate compilation

- **Linkers.** mold links a 1.01 GiB Godot debug binary in 0.46 s against 1.77 s for lld, and a 4.51 GiB Chromium in 1.65 s against 16.64 s on a Threadripper 7980X ([mold README](https://github.com/rui314/mold)). wild aims at incremental linking, but that is "not yet implemented" ([wild](https://github.com/davidlattimore/wild)); section 11 has its November 2026 state and link times. Matching benchmark setups changes the ranking ([zeli summary of Lattimore](https://zeli.app/es/story/49774000)).
- **Hot patching.** Dioxus's `subsecond` implements "a form of incremental linking / binary patching" at explicit hot entry points. Struct field changes need the framework to dispose of or migrate state ([Dioxus 0.7](https://dioxuslabs.com/blog/release-070)). Roc's new compiler hot-loads code during development by default ([Feldman](https://rtfeldman.com/rust-to-zig)).
- **C++ modules.** `import std;` is faster than `#include <vector>` alone, because a compiled `.ifc` interface replaces reprocessing headers in every file ([Microsoft Learn](https://learn.microsoft.com/en-us/cpp/cpp/tutorial-import-stl-named-module?view=msvc-170)).

### 10. Proof checking

Verus checks proofs per function with SMT. Its authors report verification "orders of magnitude faster than prior automated tools." That speed let them re-verify the whole project on each iteration instead of one function at a time ([Verus, SOSP 2024](https://www.andrew.cmu.edu/user/bparno/papers/verus-sys.pdf)). The Verus authors note that IronFleet (Dafny) split simple tasks across several functions, "presumably to keep verification times manageable." Their single-function port verified faster than any of the three originals (same source).

### 11. Build caches and linkers under a rustc backend (added 2026-11-09)

Sections 1–10 predate D18. Under D18 the backend is a pinned rustc with LLVM, and the driver "owns every rustc invocation and keeps a content-addressed artifact cache instead of Cargo's build directory" (`design/01-design.md`, D12 and D18). The four finds below are prior art for that cache and for E4's target of under 1 s from edit to running tests.

- **mbx (Mr. Boxington).** A build cache for Cargo projects that works "at the compiler boundary": a rustc shim set as `RUSTC_WRAPPER` derives a content-addressed action key for each invocation, restores the outputs on a hit, and compiles and publishes on a miss ([how it works](https://mr-boxington.jdx.dev/how-it-works)). Four mechanisms matter here.
  - *Keys that survive a path change.* Workspace, target, registry, toolchain, and sysroot paths become placeholders before they enter a key, so "equivalent worktrees share an action even though their absolute paths differ" (same source).
  - *Inputs learned after the fact.* The key depends on the files a compilation reads, which mbx learns from the dep-info of an earlier build, so "a cold compilation may therefore have no key to look up yet" (same source).
  - *Restore without copying.* A hit is restored by reflink, else by a hard link to a read-only store object, else by copy; ext4 has no reflinks, so most Linux CI runners get hard links (same source).
  - *One budget per machine.* Every compiler takes a permit from a pool shared by all builds that use the same cache directory, with CPU and predicted memory budgeted separately, and a compilation identical to one already running waits for it and restores its result (same source).
- **mbx's numbers** (self-reported: the project's own harness on a GitHub Actions runner, three trials per row, median and range, Linux x86-64, Rust 1.94.0; the page links the CI runs, which this note did not open) ([benchmarks](https://mr-boxington.jdx.dev/benchmarks)):
  - *Six overlapping check, Clippy, and test-compile jobs from an empty store*, on aube, a 15-crate workspace, with mbx 1.19.0: 2m12s run in turn; 1m7s (66.4–68.1 s) in parallel without the scheduler, peaking at 172 compilers; **54.7 s** (52.5–54.8 s) with it, peaking at 31. Cache hits over the batch were 23 without the scheduler and 3,550 with it. On hk, with mbx 1.17.0: 50.7 s, 48.2 s with 162 compilers, 39.2 s with 32. Every row runs mbx; there is no plain Cargo row.
  - *Next commit, cache warmed at the parent.* hk: mbx 9.2 s, kache 9.6 s, uncached Cargo 18.9 s. aube: mbx 23.2 s, kache 25.5 s, Cargo 33.2 s.
  - *Same commit, empty `target/`.* hk: 1.3 s (mbx), 1.2 s (kache). aube: 3.3 s, 3.6 s.
  - *One-line edit, rebuilt in place.* hk: kache 2.3 s, mbx 2.4 s, Cargo incremental 2.4 s. aube, editing a crate most of the workspace depends on: Cargo 7.9 s, mbx 8.2 s, kache 8.3 s. The first edit after a full build costs more under a cache: 24.9 s (mbx) and 27.5 s (kache) against Cargo's 8.0 s on aube.
  - The page's own caveats: two pinned workloads, ranges that "are not confidence intervals," Linux only.
- **mbx's limits** ([caching limits](https://mr-boxington.jdx.dev/limits)). What it cannot model exactly it runs uncached. Incremental compilations are never cached, and incremental state stays private to a checkout. Native links are cached only where the linker, startup objects, and libc can be put in the key. Restored artifacts are "equivalent, not always identical": rustc records absolute source paths in metadata and debug information, and Cargo runs rustc in the crate's own directory, which rustc also stores. A cacheable workspace crate can still be restored byte for byte in another checkout. When a checkout-specific key forces it to compile there instead, its output bytes differ and its dependents miss. The opt-in fix maps the workspace root to a placeholder, at the cost of placeholder paths in debug information, `file!()`, and panic locations.
- **kache.** A compiler cache for Rust, C/C++, and CUDA that mbx says "directly inspired its design" ([mbx README](https://github.com/jdx/mr-boxington)). It runs as `RUSTC_WRAPPER`, computes a BLAKE3 key, and keeps outputs as content-addressed blobs; a restore first tries a copy-on-write clone, then a hard link for immutable artifacts, then a copy; a miss "joins a machine-wide flight, takes a memory-weighted permit and a per-key lock" ([architecture](https://kunobi.ninja/docs/kache/how-it-works/architecture)). Version 2.0.0 (2026-11-02) splits it into a client, an engine, and a daemon (same source; [docs.rs](https://docs.rs/crate/kache)). "Zero-copy, content-addressed" is the description of the [repository](https://github.com/kunobi-ninja/kache) and of the 0.5.0 crate; the 2.0.0 crate description drops it. Zero-copy holds only where the filesystem has reflinks (APFS, btrfs, XFS with reflink enabled). Elsewhere restores are hard links and the store is filled by copying ([storage report](https://kunobi.ninja/blog/kache-storage-worktrees)).
- **kache's numbers** (self-reported, one run; the post calls itself "a vendor making a vendor claim"): building Firefox 151 in two worktrees on APFS with kache 0.7.0, the second worktree restored 13.5 GB by reflink with no bytes copied and added about 3 GB of new data, where its build directory is 16.6 GB on paper. The warm build was 10.75x faster at a 98.5% hit rate (same source). mbx's benchmark page above is the only measurement of kache (0.26.3) found that kache's vendor did not run, and a competitor ran it.
- **Cargo's cross-workspace cache.** An accepted 2026 project goal to share build units across workspaces. In 2025 Cargo split `target-dir` into `artifact-dir` and `build-dir` and began regrouping the build directory by build unit; the 2026 plan is to stabilize that layout, then put "an initial shared cache on nightly that only supports basic crates (no build scripts, proc macros, etc)," and stabilization "will likely not be completed during the goal period" ([goal](https://goals.rust-lang.org/2026/cargo-cross-workspace-cache.html)). The goal page gives no measurements. The tracking issue's minimum design caches "only immutable items (registry, git, not path)" ([cargo#5931](https://github.com/rust-lang/cargo/issues/5931)). Status from the goal's updates: the new layout was stabilized, rolled back because the extra `-L` arguments hit OS command-length limits, and stabilized again, with release expected in Rust 1.100.0; as of 14 October 2026 the cache itself is a design document and prototypes built on a shared content-addressed blob store, garbage collection is the hard open part because `cargo clean` does not free a shared store, and absolute paths in rustc's output still limit what can be cached ([rust-lang/goals#626](https://github.com/rust-lang/goals/issues/626)).
- **wild.** Its README still says "the plan is to eventually make it incremental, however that isn't yet implemented," as it did when section 9 was written ([wild](https://github.com/wild-linker/wild), v0.11.0, 2026-10-24; the repository moved from `davidlattimore/wild`). The README lists ELF on Linux for five architectures and wasm32 as supported, and Mach-O and Windows as future work, but also links [preliminary Mach-O benchmarks](https://github.com/wild-linker/wild/blob/main/benchmarks/mac-mini-m4.md) that omit unsupported features, including debug information. Rust's Fast Builds roadmap lists "Incremental, efficient linking with Wild" with the timespan "2026 and beyond" ([roadmap](https://goals.rust-lang.org/2026/roadmap-fast-builds.html)). The project's charts (self-reported; link step only, Ryzen 9 9955HX, output on tmpfs) give each linker's time relative to wild 0.11.0; the absolute times here are read off the bars and are approximate ([benchmarks](https://github.com/wild-linker/wild/blob/main/benchmarks/ryzen-9955hx.md)):

  | Binary | wild 0.11.0 | mold 3.0.0 | LLD 22.1.2 |
  |---|---|---|---|
  | ripgrep | about 26 ms | +95% | +209% |
  | rust-analyzer | about 0.10 s | +136% | +687% |
  | Zed, debug | about 0.6 s | +107% | +688% |

  Section 9's caveat still applies: matching benchmark setups changes the ranking.

**Inference, on D18's cache.**

1. *Two tools converged on one shape.* mbx and kache both use a content-addressed store, a key derived from normalized inputs, restore by reflink or hard link from read-only objects, in-flight deduplication, and a machine-wide permit pool weighted by memory. D18's cache can take that shape as known to work.
2. *The driver starts where the wrappers struggle.* A wrapper infers a compilation's inputs from a command line and learns the rest from dep-info, so a cold build has no key and anything unmodeled is a bypass. D18's driver constructs every rustc command and snapshots the manifest with the sources; [RUN-2](../design/03-authority-concurrency.md) routes rustc's source reads through the driver's file loader, so the driver can derive a key from the snapshot before the first compile. Tide crates have no build scripts, avoiding their own scripts' unsupported search paths and link arguments, and checkout paths embedded in generated files, which cause mbx bypasses or restrict sharing ([limits](https://mr-boxington.jdx.dev/limits)). Imported Rust crates' build scripts (D13) keep those risks.
3. *Path independence is the part D18 does not yet state.* Normalized action keys permit cross-worktree restores even when the restored artifact embeds the original checkout path. Normalizing embedded output paths can also preserve dependent hits when a checkout-specific key forces an upstream crate to rebuild in another checkout ([mbx limits](https://mr-boxington.jdx.dev/limits#a-rebuilt-workspace-crate-records-its-checkout)). The driver owns the flags, so it can remap paths or compile from a fixed root for every build. The cost mbx names applies: debug information, `file!()`, and panic locations then carry the mapped path.
4. *Cargo's proposed shared storage does not replace the driver's policy.* The Cargo-integrated proposal separates shared content-addressed storage from the workspace `build-dir` where compilation happens ([proposed layout](https://github.com/rust-lang/cargo/issues/5931#issuecomment-5498506395)); direct reuse by Tide has not been established. Its minimum design excludes path crates, which is what a Tide package's own code is. As of the 14 October update it remains design work and prototypes, with stabilization not expected in 2026. Tide would still need driver-owned keys and admission policy covering `tide.toml` and the admission verdict D18 requires before rustc writes any artifact. The goal's blob-store layout and garbage-collection work is a reference for D18's cache, and its work on absolute paths in rustc output helps Tide, since Tide runs the same compiler. Under v1's exported `Cargo.toml`, a Rust consumer's Cargo could cache a Tide crate fetched from a registry, if the cache ships and its scope admits the renamed `std` dependency and toolchain proc macros.

**Inference, on E4's edit-to-tests target.**

1. *A whole-crate cache does not shorten the edit loop.* On their own benchmark both caches are level with or behind Cargo's incremental rebuild after a one-line edit (2.3–2.4 s on hk, 7.9–8.3 s on aube), and all of those are above 1 s. The cache answers clean builds, branch switches, new worktrees, and fleets. E4's edit-to-tests number has to come from rustc's incremental compile of small crates, the link, and the test run.
2. *The linker decides only for large test binaries.* At ripgrep's size wild, mold and LLD all link in under 0.1 s ([chart](https://github.com/wild-linker/wild/blob/main/benchmarks/images/ryzen-9955hx/ripgrep-time.svg)). At Zed's size LLD, which Rust uses by default on `x86_64-unknown-linux-gnu` ([Rust announcement](https://blog.rust-lang.org/2025/09/01/rust-lld-on-1.90.0-stable/)), takes several seconds (+688% of about 0.6 s) and wild about 0.6 s. Wild's incremental mode does not exist yet, and its FAQ says mold's author does not intend to add one ([wild FAQ](https://github.com/wild-linker/wild/blob/main/FAQ.md)). If Tide's test binaries grow that large, a faster linker alone leaves little of the second for compiling and running. The levers the driver controls are the linker it selects (the branch cannot choose it, D13) and how much code each test binary links. D12 already names the fallback: an interpreter, built only if E4's edit-to-test target needs it.
3. *E4 should report the split.* Compile, link, and run times for the edit-to-tests case, with the default LLD and at least one of mold and wild, would show which term to attack.

## What works / what failed and why

**What works**
1. **Compiled interfaces that read only direct dependencies** (Go export data, OCaml `.cmi`, C++ `.ifc`). This bounds fan-in by design, and C++ had a 2000x counterexample.
2. **Separate dev and release backends** (Zig self-hosted vs LLVM, DMD vs LDC/GDC, Jai x64 vs LLVM). Zig's 5x debug speedup came from owning the backend.
3. **Function-granular incremental compilation down to the binary.** Zig's 35 ms on 464k LOC is the clearest proof that O(change) rebuilds are achievable in a systems language.
4. **Local inference and required signatures** (Sorbet, OCaml `.mli`). Checking becomes embarrassingly parallel and incremental.
5. **Query engines with early cutoff and durability** (Salsa/ty: 4.7 ms on PyTorch).
6. **Fast parallel linkers** (LLD: 7x link speedup and a 40% faster incremental ripgrep build; mold roughly 2–16x faster than LLD on its README's x86 benchmarks).
7. **Data-oriented, index-based compiler data** (Carbon, Roc, Zig), plus native implementation (tsgo: 10x).

**What failed or underdelivered, and why**
1. **Retrofitting speed onto rustc.** The parallel frontend has been "almost there" since 2023, because a global, dynamically tracked query system shared across threads makes deadlocks and cycle-error nondeterminism hard to remove. Polymorphization v1 was blocked by non-parametric features (`TypeId`). Relink-don't-rebuild has to retrofit an interface/body distinction the language never had.
2. **Swapping in a fast backend without changing the language** (Cranelift in rustc: 5–15% end to end. TPDE in Clang: 17% end to end). A fast backend only helps the share of time spent in the backend. Rust's frontend, monomorphization volume, and linker remain.
3. **Global inference with type-directed overloading** (Swift). Years of solver work cannot remove the exponential worst case. The fix would need language changes the team rules out as too disruptive.
4. **Codegen-unit partitioning heuristics.** Size estimates are off "by a factor of two or more," so tuning went nowhere ([Nethercote](https://nnethercote.github.io/2023/07/11/back-end-parallelism-in-the-rust-compiler.html)).
5. **Unrestricted metaprogramming.** Proc macros and build scripts are opaque to caching and to pipelining, and they inflate code 3x.
6. **Full monomorphization of generics** (Go 1.18 hybrid still exponential in the worst case. Rust's per-crate duplication).

## Design implications (ranked, concrete, with confidence)

Ranking is by expected effect on check/build latency, weighted by how hard the decision is to reverse after 1.0.

1. **Interfaces are compiled artifacts, and bodies never leak into dependents in dev builds.** Each module produces a signature artifact holding types, function signatures, trait/impl headers, and exported constants, with a stable *interface hash*. Dependents read only the signature artifacts of their direct imports. Function bodies, private items, and comments never change the interface hash. Cross-module inlining and cross-module specialization are **release-only**, or limited to items explicitly marked `inline`, whose bodies are then part of the interface (like GHC `INLINABLE` or Swift `@inlinable`). Import cycles between modules are forbidden, and unused imports are errors. *Rationale:* Go's "single biggest reason," the 2000x C++ blowup, and Rust's relink-don't-rebuild retrofit. *Confidence: high.*

2. **Required signatures everywhere at item level; inference only inside bodies; no type-directed overloading.** Function parameter and return types, struct fields, constants, and statics need annotations. Inference stays inside one function body, so a body can be checked from its own text plus signatures (Sorbet's property). Overloading is by trait/interface only, with no ad-hoc same-name overloads that the solver must choose between. Integer and float literals default deterministically, with no search over literal types. *Rationale:* Swift's exponential solver, OCaml's statement that inference does not scale, Sorbet's millisecond rechecks. **AI-agent note (inference):** mandatory signatures also give agents precise, local error messages, so this costs little ergonomically for machine writers. *Confidence: high.*

3. **Generics compile to one polymorphic body plus dictionaries by default; specialization is a separate, semantics-free optimization.** Dev builds use layout-based sharing, where types with the same size, alignment, and pointer map share a body (Go GC shapes, Rust's 2026 "monomorphic data, polymorphic code" plan), with hidden dictionary arguments for trait methods, drop, and size. Release builds specialize hot or annotated instantiations (`#[specialize]` or PGO-driven) under an instantiation budget. Two language rules make this possible. (a) Generic code must be **parametric**: no `TypeId`-style type inspection inside unconstrained generics, which is exactly what blocked Rust polymorphization v1. Type inspection must go through an explicit bound. (b) No specialization-dependent semantics, so overlapping-impl specialization is out. *Rationale:* Oxford study (dictionary: polynomial size vs exponential), Rust's 30%–2x projection, GHC/.NET precedent. *Risk:* about 1.3–1.4x slower dev-build runtime on generic-heavy code (Oxford numbers). That is acceptable for dev, and release recovers it. *Confidence: medium-high* (high on compile-time benefit, medium on keeping release runtime at Rust parity without full monomorphization, see Open questions).

4. **Metaprogramming is a deterministic, budgeted, memoizable compile-time interpreter, not arbitrary native plugins.** Allow `comptime` functions running on the compiler's IR interpreter, under a fuel budget (Zig's eval quota) and with no I/O except declared inputs, so results can be cached by input hash. Ship a fixed set of built-in derives (eq, hash, debug, serialize) implemented in the compiler. Code generation that adds top-level items must be declared in the module's signature (the item names are visible without running it), or be confined to a file-scoped generator whose output is checked in. No build scripts with undeclared side effects. *Rationale:* 3x expansion and 20% compile-time cost of proc macros, rust-analyzer's macro lessons, Anderson's build-script critique. *Confidence: high.*

5. **Name resolution and impl lookup must be indexable per file.** Module path = file path. Only explicit imports, no glob imports (or globs restricted to a prelude). Impls/instances may live only in the module that defines the type or the trait (a strict orphan rule), so finding "the impl for T: Tr" is a lookup in two known modules, never a crate-wide search. Conditional compilation acts at item level after parsing and is resolved during name resolution. *Rationale:* rust-analyzer "three architectures." *Confidence: high.*

6. **The compilation and caching unit is the item (function, type), not the crate, and the package is not a compilation barrier.** Packages exist for versioning and visibility. The compiler's dependency graph tracks items, which enables Zig-style function-granular rebuilds. *Rationale:* Rust's crate-as-unit is the most cited structural cause, Zig and Roc reach 35 ms. *Confidence: medium-high* (a question of engineering effort, not feasibility).

7. **Zero-cost abstractions must be cheap *before* the optimizer.** Iterators, closures, and error propagation should lower to reasonable code without inlining-heavy cleanup (for example closures as direct calls with environment structs, and `?` as a branch). Default `panic = abort` for dev, with no unwinding tables. Use line-tables-only debuginfo by default (2–30% fewer cycles in Rust's benchmarks, per the [survey](https://blog.rust-lang.org/2025/09/10/rust-compiler-performance-survey-2025-results)). *Rationale:* "generics produce huge amounts of IR" that LLVM must clean up; Anderson's unwinding cost. *Confidence: medium.*

8. **Ownership and borrow checking stay intraprocedural, using signatures and lifetimes at boundaries, and run in parallel per function.** Proofs are optional, per-function, cached by (spec, body, callee-spec) hash, and run asynchronously off the `check` path unless requested. *Rationale:* NLL's small time cost, Verus's per-function speed making whole-project iteration practical. *Confidence: high* for borrow checking, *medium* for proofs.

9. **Tests do not force a second full compile.** Tests are ordinary items in the same incremental graph, discovered by attribute and run through the dev backend or a JIT. *Rationale:* Anderson's "tests next to code" cost, Rust's check/build/test triple rebuild. *Confidence: medium.*

## Reference architecture

```
                 ┌───────────── persistent compiler daemon (one per workspace) ─────────────┐
 editor/LSP ───► │  VFS (in-memory files, durability tags: stdlib/deps = durable, user = volatile)│
 agent CLI  ───► │                                                                               │
                 │  [1] lex+parse per file  ──► lossless green tree (Roslyn-style), index-based   │
                 │        (parallel, ~10M LOC/s/core target)                                      │
                 │  [2] item tree per file: names + signatures only (stub, Sorbet/IntelliJ-style) │
                 │        └─► module interface artifact + interface hash (early cutoff point)     │
                 │  [3] name resolution / impl index (per module, FQN-keyed)                      │
                 │  [4] body check per function: local inference, trait resolution, ownership     │
                 │        (parallel; query = (fn, interface hashes of deps))                      │
                 │  [5] comptime interpreter (fuel-budgeted, memoized on inputs)                  │
                 │  [6] optional proof check per fn (async, cached by spec/body/callee hashes)    │
                 │  [7] polymorphic mid-IR (dictionary-passing, layout-shared), per function      │
                 │         │                                     │                                │
                 │   DEV ▼                              RELEASE ▼                                 │
                 │  [8a] single-pass backend             [8b] specialize (budgeted/PGO)           │
                 │       (TPDE/Liftoff-style direct            → LLVM -O2/-O3, ThinLTO/LTO        │
                 │       emission, per-fn object cache)        (full rebuild acceptable)          │
                 │  [9a] incremental linker / in-place       [9b] mold/lld                        │
                 │       binary patching via per-fn                                               │
                 │       indirection; hot reload                                                  │
                 │  [10] IR interpreter/JIT for tests & comptime (no link needed)                 │
                 │  Cache: on-disk, index-based, zero-parse (Roc), shared by check/build/test     │
                 └───────────────────────────────────────────────────────────────────────────────┘
```

**Key properties**

- **One daemon serves the editor, CLI, and agents.** `check`, `build`, and `test` share one incremental base. This avoids Rust's triple rebuild and IDE/Cargo lock contention, which more than 35% of survey respondents called a big problem.
- **Two early-cutoff walls.** (a) A body edit that leaves the interface hash unchanged stops at [2], so dependents keep their check results and only that function goes through [4]→[8a]→[9a]. (b) A signature change invalidates only modules that import the changed item and use it, tracked per item.
- **Durability tiers.** The standard library and third-party packages are "durable," so their subgraphs are skipped wholesale after a volatile edit. This fixes rust-analyzer's 300 ms validation problem by design.
- **Parallelism by construction, not retrofit.** Stages [1], [4], [6], [7], and [8a] are embarrassingly parallel per file or per function, because each needs only immutable signature artifacts. This is the property rustc lacks.
- **Two backends with one semantics.** The dev path never specializes or inlines across modules. The release path may. Because generics are parametric and specialization has no semantic effect, both must produce identical observable behavior. A differential-testing harness that runs the test suite under both is mandatory (**inference**).
- **Hot reload.** In dev builds every function is called through a patchable slot (GOT- or jump-table-like), so [9a] can swap a changed function in place. Changes to type layout invalidate live state and fall back to a restart (the subsecond limitation).

### Latency budget and realistic numeric targets (100k LOC project, 8-core laptop, stdlib/deps already cached)

| Operation | Target | Evidence that it is achievable |
|---|---|---|
| Edit one function body → diagnostics | **p50 ≤ 30 ms, p95 ≤ 100 ms** | ty 4.7 ms on PyTorch central-file edit; Sorbet ms for 80% of edits on 15M LOC; Zig incremental errors in "milliseconds" |
| Edit signature used by ~20 modules → diagnostics | ≤ 150 ms | Inference: recheck is ~20 modules' bodies at ≥1M LOC/s/core |
| Body edit → runnable debug binary (patch in place) | **≤ 100 ms**, stretch 50 ms | Zig: Roc 464k LOC in 35 ms, Tetris ~30 ms, Zig compiler 228–288 ms |
| Clean debug build (user code only) | **≤ 1 s** | Jai 250k LOC/s x64 (secondary); Zig `hello` 244 ms; date-fns 104k LOC type-check 0.7 s in tsgo; mold links 1 GiB debug in 0.46 s |
| Clean `check` (no codegen) | ≤ 300 ms | Inference from 10M LOC/s parse + 1M LOC/s check targets with 8-way parallelism |
| Release build | No hard target; ≤ Rust for equal code; 10–50x slower than dev is acceptable | LLVM -O2 plus specialization dominates (inference) |
| Dev-build runtime speed | within ~1.5–3x of release on typical code | TPDE ±9% of LLVM -O0; dictionary passing 1.3–1.4x slower than mono; Liftoff 1.5–1.7x slower than optimized |

**Clean 1 s debug build budget for 100k LOC (inference from the throughput figures above):** parse about 10 ms (10M LOC/s), signature collection and resolution about 10 ms, body check plus ownership about 100 ms single-core or about 20–30 ms on 8 cores (1M LOC/s/core), mid-IR lowering about 30 ms, dev codegen about 400 ms single-core at 250k LOC/s or about 60–100 ms on 8 cores, link about 100–200 ms. That totals roughly 0.25–0.5 s, which leaves 2x headroom against the 1 s target. **What blows this budget, in order:** monomorphization fan-out, macro expansion volume, full debuginfo, and a slow linker.

**Throughput targets per stage (per core):** lex+parse ≥ 5–10M LOC/s, signature/resolve ≥ 5M LOC/s, body check ≥ 1M LOC/s, dev codegen ≥ 250k LOC/s, release unconstrained. These are design targets anchored on Carbon's (unverified) per-line goals and Jai's reported backend speed, not measured results for this language.

## Open questions

1. **Release-mode parity without default monomorphization.** Can budgeted, PGO-guided specialization match Rust's runtime performance on generic-heavy code (collections, iterators, numeric kernels)? Swift's "performance cliff" says the selection policy matters a great deal. One option: specialize every instantiation in release builds (full mono) and use dictionaries only in dev builds. That keeps release parity at release-build cost.
2. **Ownership checking with polymorphic code.** Ownership checking happens before lowering, so it is unaffected. But drop glue and move semantics through dictionaries need a precise ABI (size/align/drop in the dictionary, as in Rust's polymorphization plan). Unsized or `!Sized` cases need design work.
3. **Can the TPDE/Liftoff-class dev backend be built in-house at reasonable cost?** TPDE's Cranelift-IR adapter was about 4k lines on top of the framework ([TPDE paper](https://arxiv.org/pdf/2505.22610)). Reusing TPDE (via LLVM-IR) is faster to ship but inherits LLVM IR construction cost.
4. **Incremental linking and binary patching outside Linux ELF.** Zig's and wild's incremental linking work is ELF-first. Mach-O and COFF need their own work, and Zig's x86 default excluded Windows for linker reasons ([devlog 2025](https://ziglang.org/devlog/2025/)).
5. **Comptime power vs cacheability.** How much reflection (type introspection) can comptime have before it breaks parametricity (implication 3) or item-level indexability (implication 5)?
6. **Proof-checking latency budget.** Should proofs be in the agent's inner loop (a blocking `check --proofs` on changed functions only) or a CI tier? That depends on per-function SMT time distributions, which need measuring on representative code.
7. **Daemon memory and robustness.** Persistent daemons trade RAM for latency. The parallel rustc frontend raised memory up to 35% ([Rust blog](https://blog.rust-lang.org/2023/11/09/parallel-rustc/)). There also needs to be a deterministic batch mode that reproduces daemon results exactly, for CI and reproducible builds.

## Sources

- Rust compiler performance survey 2025 results — https://blog.rust-lang.org/2025/09/10/rust-compiler-performance-survey-2025-results
- Faster compilation with the parallel front-end in nightly — https://blog.rust-lang.org/2023/11/09/parallel-rustc/
- Rust project goal 2026: parallel front end — https://goals.rust-lang.org/2026/parallel-front-end.html
- Benchmarking sccache and -Zthreads — https://neosmart.net/blog/benchmarking-rust-compilation-speedups-and-slowdowns-from-sccache-and-zthreads/
- Popov, "Rust ❤ LLVM", LLVM Dev Mtg 2024 — https://llvm.org/devmtg/2024-10/slides/keynote/Popov-Rust_Heart_LLVM.pdf
- LLD default on x86_64 Linux (Rust 1.90) — https://blog.rust-lang.org/2025/09/01/rust-lld-on-1.90.0-stable/
- matklad, Fast Rust Builds — https://matklad.github.io/2021/09/04/fast-rust-builds.html
- Rust 2026 goal: polymorphization — https://goals.rust-lang.org/2026/polymorphization.html
- Nethercote, How much code does that proc macro generate? — https://nnethercote.github.io/2025/06/26/how-much-code-does-that-proc-macro-generate.html
- Nethercote, Back-end parallelism in the Rust compiler — https://nnethercote.github.io/2023/07/11/back-end-parallelism-in-the-rust-compiler.html
- Nethercote, How to speed up the Rust compiler one last time (2020) — https://blog.mozilla.org/nnethercote/2020/09/08/how-to-speed-up-the-rust-compiler-one-last-time/
- Nethercote, March 2025 update — https://nnethercote.github.io/2025/03/19/how-to-speed-up-the-rust-compiler-in-march-2025.html
- Nethercote, September 2026 update — https://nnethercote.github.io/2026/09/30/how-to-speed-up-the-rust-compiler-in-september-2026.html
- Rust 2025 goal: Relink, don't rebuild — https://goals.rust-lang.org/2025h2/relink-dont-rebuild.html
- Rust 2026 Fast Builds roadmap — https://goals.rust-lang.org/2026/roadmap-fast-builds.html
- Rust 2026 goal: Incremental systems rethought — https://goals.rust-lang.org/2026/incremental-system-rethought.html
- Rust 2026 goal: TPDE — https://goals.rust-lang.org/2026/tpde.html
- Rust 2026 goal: cg_clif performance — https://goals.rust-lang.org/2026/improve-cg_clif-performance.html
- Rust 2025 goal: production-ready Cranelift — https://goals.rust-lang.org/2025h2/production-ready-cranelift.html
- Kobzol, Why doesn't Rust care more about compiler performance? — https://kobzol.github.io/rust/rustc/2025/06/09/why-doesnt-rust-care-more-about-compiler-performance.html
- Anderson, The Rust compilation model calamity — https://www.pingcap.com/blog/rust-compilation-model-calamity/
- Feldman, How our Rust-to-Zig rewrite is going — https://rtfeldman.com/rust-to-zig
- Pike, Go at Google — https://go.dev/talks/2012/splash.article
- Go 1.18 generics via dictionaries and GC-shape stenciling — https://go.googlesource.com/proposal/+/master/design/generics-implementation-dictionaries-go1.18.md
- golang/go#49569 (1.18 compile-time regression) — https://github.com/golang/go/issues/49569
- Generic Go to Go (OOPSLA 2022) — https://ar5iv.labs.arxiv.org/html/2208.06810
- Zig devlog 2025 — https://ziglang.org/devlog/2025/
- Zig devlog 2026 — https://ziglang.org/devlog/2026/
- Zig 0.15.1 release notes — https://ziglang.org/download/0.15.1/release-notes.html
- Zig language reference (@setEvalBranchQuota) — https://ziglang.org/documentation/master/
- Hashimoto, Zig builds are getting faster — https://mitchellh.com/writing/zig-builds-getting-faster
- Jai community wiki, Overview — https://github.com/Jai-Community/Jai-Community-Library/wiki/Overview
- Odin FAQ — https://odin-lang.org/docs/faq/
- D downloads (DMD/LDC/GDC) — https://dlang.org/download.html
- Carbon toolchain architecture — https://docs.carbon-lang.dev/toolchain/docs/
- Carruth, Modernizing compiler design for Carbon's toolchain (CppNow 2023) — https://schedule.cppnow.org/session/2023/modernizing-compiler-design-for-carbons-toolchain/
- do-compiler (cites per-line targets) — https://github.com/v-pan/do-compiler
- Swift type checker roadmap — https://forums.swift.org/t/roadmap-for-improving-the-type-checker/82952
- Hooper, Why Swift's type checker is so slow — https://danielchasehooper.com/posts/why-swift-is-slow/
- Swift cross-module inlining pitch — https://lists.swift.org/pipermail/swift-evolution/Week-of-Mon-20171002/040074.html
- swift-dev, metadata/conformance uniqueness — https://lists.swift.org/pipermail/swift-dev/Week-of-Mon-20170731/005039.html
- Real World OCaml, compiler frontend — https://dev.realworldocaml.org/compiler-frontend.html
- Stripe, Sorbet — https://stripe.dev/blog/sorbet-stripes-type-checker-for-ruby
- Microsoft, A 10x faster TypeScript — https://devblogs.microsoft.com/typescript/typescript-native-port/
- Astral, ty — https://astral.sh/blog/ty
- GHC user guide, pragmas — https://downloads.haskell.org/ghc/latest/docs/users_guide/exts/pragmas.html
- Kennedy & Syme, .NET generics (PLDI 2001) — https://www.microsoft.com/en-us/research/publication/design-and-implementation-of-generics-for-the-net-common-language-runtime/
- Cranelift README — https://github.com/bytecodealliance/wasmtime/tree/main/cranelift
- TPDE paper (arXiv 2505.22610) — https://arxiv.org/pdf/2505.22610
- Xu & Kjolstad, Copy-and-patch compilation — https://arxiv.org/abs/2011.13127
- QBE — https://c9x.me/compile
- V8 Liftoff — https://v8.dev/blog/liftoff
- rustc dev guide, incremental compilation — https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation.html
- rust-analyzer, Three architectures for a responsive IDE — https://rust-analyzer.github.io/blog/2020/07/20/three-architectures-for-responsive-ide.html
- rust-analyzer, Durable incrementality — https://rust-analyzer.github.io/blog/2023/07/24/durable-incrementality.html
- Lippert, Red-green trees — https://ericlippert.com/2012/06/08/red-green-trees/
- mold — https://github.com/rui314/mold
- wild — https://github.com/davidlattimore/wild
- Wild vs mold benchmark discrepancy (secondary) — https://zeli.app/es/story/49774000
- Dioxus 0.7 (subsecond hot-patching) — https://dioxuslabs.com/blog/release-070
- Microsoft Learn, import std — https://learn.microsoft.com/en-us/cpp/cpp/tutorial-import-stl-named-module?view=msvc-170
- Verus: A Practical Foundation for Systems Verification (SOSP 2024) — https://www.andrew.cmu.edu/user/bparno/papers/verus-sys.pdf
- mbx (Mr. Boxington) README — https://github.com/jdx/mr-boxington
- mbx, How it works — https://mr-boxington.jdx.dev/how-it-works
- mbx, Benchmarks (self-reported) — https://mr-boxington.jdx.dev/benchmarks
- mbx, Caching limits — https://mr-boxington.jdx.dev/limits
- kache on docs.rs — https://docs.rs/crate/kache
- kache repository — https://github.com/kunobi-ninja/kache
- kache, Architecture — https://kunobi.ninja/docs/kache/how-it-works/architecture
- kache storage report, Firefox in two worktrees (self-reported) — https://kunobi.ninja/blog/kache-storage-worktrees
- Rust 2026 goal: Cargo cross workspace cache — https://goals.rust-lang.org/2026/cargo-cross-workspace-cache.html
- rust-lang/goals#626 (status updates for that goal) — https://github.com/rust-lang/goals/issues/626
- rust-lang/cargo#5931, Per-user compiled artifact cache — https://github.com/rust-lang/cargo/issues/5931
- Cargo shared-cache storage proposal, 1 Sep 2026 — https://github.com/rust-lang/cargo/issues/5931#issuecomment-5498506395
- wild, current repository — https://github.com/wild-linker/wild
- wild benchmarks, Ryzen 9 9955HX (self-reported) — https://github.com/wild-linker/wild/blob/main/benchmarks/ryzen-9955hx.md
- wild preliminary Mach-O benchmarks, Mac Mini M4 (self-reported) — https://github.com/wild-linker/wild/blob/main/benchmarks/mac-mini-m4.md
- wild FAQ — https://github.com/wild-linker/wild/blob/main/FAQ.md
