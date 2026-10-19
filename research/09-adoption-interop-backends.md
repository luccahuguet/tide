# 09: Adoption, ecosystem interop, and backend/bootstrapping strategy

Research date: 2026-10-19. Sources are mostly 2024–2026 primary posts and papers. Every factual claim carries a link. Lines marked **Inference** are my reasoning, not something a source states.

Method note: the shared web-search budget ran out partway through. Later facts came from fetching primary URLs directly. Package-ecosystem history (npm, crates.io) and a few smaller languages (Crystal, Nim) therefore rest on fewer sources than the core topics, and they are flagged where it matters.

---

## Question

A new language starts with no ecosystem, no users, and no training data. Ferox aims to be:

- as safe and fast as Rust;
- much lighter and much faster to compile;
- strongly typed, with optional machine-checked proofs;
- a good fit for Opus-class coding agents to write, check, and maintain.

How should it:

1. gain real users within 2–3 years;
2. interoperate with existing ecosystems, especially Rust crates and C;
3. choose backends for dev and release builds;
4. bootstrap with a small team that leans heavily on AI agents?

---

## Findings (cited)

### F1. What actually drove adoption

| Language | What got it adopted | Ecosystem bridge | Time scale |
|---|---|---|---|
| **Rust** | Mozilla sponsorship (2009) and Servo/Firefox as the first proving ground, then platform owners. Android support came in 2021, and the Linux kernel declared the Rust experiment concluded in Dec 2025 [rust-wiki][lwn-rust][devclass-rust]. Android reports a 1000x lower memory-safety vulnerability density than its C/C++, a 4x lower rollback rate, and 25% less review time [android-2025]. | C ABI only. Cargo and crates.io were its own ecosystem. | 2006 start, 1.0 in May 2015, kernel non-experimental in 2025 [rust-wiki]. |
| **Go** | Google backing and a compile-speed origin story. Go began during a 45-minute build, and its design (precise imports, export data) targeted build speed [go-splash]. Docker and Kubernetes became its flagship apps [go-wiki]. The Go 1 compatibility promise has held since 2012 [go-wiki]. | cgo; mostly its own ecosystem. | 2009 announcement, 1.0 in 2012, self-hosted in 2015 [go-wiki]. |
| **Swift** | Apple platform mandate [swift-wiki]. | Objective-C runtime interop from day one. C++ interop only from 5.9 (2023) [swift-wiki][swift-cxx]. | Announced 2014. |
| **Kotlin** | Community enthusiasm, then Google's 2017 I/O announcement, then "Kotlin-first" in 2019. JetBrains says its two biggest jumps in new users came in May 2017 and May 2019 [kotlin-5y]. | Calls Java seamlessly. Unannotated Java types become "platform types" with unchecked nullability [kotlin-java]. | |
| **TypeScript** | A JavaScript superset with gradual typing and Microsoft tooling. Type-definition files (DefinitelyTyped) describe untyped JS libraries [ts-wiki]. In August 2025 it became GitHub's #1 language by monthly contributors, and GitHub explicitly credits AI-assisted coding plus static types [octoverse]. | Any JS is valid TS, and its output is plain JS. | 2012 release [ts-wiki]. |
| **Zig** | `zig cc` as a hermetic cross-compiling C toolchain: Uber built all C/C++ in its Go monorepo with it from Jan 2023 [uber-zig]. Bun as a flagship app [bun-rust]. | Direct C header import, moved into the build system in 0.16 [zig-016-cimport]. | Still pre-1.0 (0.16) in 2026 [zig-016-cimport]. |
| **Elixir** | Productivity layer on the BEAM, with full Erlang interop and Mix tooling [elixir-wiki]. | Erlang/BEAM. | 2012. Gradual typing arrived in 1.20 (June 2026) [elixir-wiki]. |
| **Gleam** | Small, consistent language. v1 (March 2024) shipped compiler, build tool, package manager, formatter, and LSP together, with a semver stability pledge [gleam-v1]. Ranked #2 most-admired (70%) in the 2025 Stack Overflow survey [so-2025]. | BEAM and JS targets. Typed `@external` declarations; the foreign side is unchecked, with "no additional performance cost" [gleam-ext]. | Admired but still small in usage [so-2025]. |
| **Mojo** | Chris Lattner and Modular. The Python-superset promise was walked back; Mojo now calls into CPython instead [infoworld-mojo]. | Python interop. | 1.0 in Aug 2026. Compiler open-sourced Aug 18, 2026, three weeks after Qualcomm acquired Modular [modular-oss]. |
| **Carbon** | Google-backed C++ successor whose strategy is interop. | Bidirectional C++. | 0.1 (MVP) is "at the soonest" end of 2026 and "may not be possible" [carbon-roadmap]. |
| **Nim / Crystal** | Good technology with no platform owner and no flagship app. Crystal is used in Kagi's backend [crystal-wiki]. Nim (C backend, 2008) reached 1.0 in 2019 and 2.0 in 2023 [nim-wiki]. A rewrite (Nimony, "Nim 3") is in progress [nimony-medium]. | C (Nim), LLVM (Crystal). | Both have stayed niche for 10+ years (my judgment; no usage data gathered). |
| **V** | Pre-release hype. An independent 2022 review reportedly verified fewer than 25% of claimed features. "Autofree" became GC-by-default [v-critique]. | Emits C. | Credibility damage outlived the fixes. |
| **Dart** | Planned VM in Chrome dropped in 2015 amid fragmentation criticism; revived as Flutter's language [dart-wiki]. | Compile-to-JS. | A platform rescued it. |
| **Elm** | 0.19 (2018) banned user native/kernel code; no release since 0.19.1 (Oct 2019); maintainer burnout. Users report that trust eroded [elm-derw][elm-native]. | Ports only. | A cautionary tale. |

### F2. The AI shift in 2025–2026

- **Building a compiler has become cheap.**
  - 16 parallel Claude agents wrote a 100K-line C compiler in Rust in two weeks for about $20K. It boots Linux on three architectures. The author notes "the task verifier is nearly perfect, otherwise Claude will solve the wrong problem," with GCC used as the oracle [anthropic-ccc].
  - Steve Klabnik's Rue (affine types with mutable value semantics, native x86-64/Arm64) reached roughly 70–100K lines of Rust in about two weeks, with Claude writing most of the code [infoq-rue].
  - Bend 2 says its compiler is "99% AI-written and not yet fully audited" [bend].
- **Porting a codebase between languages has become cheap.**
  - Bun rewrote 535,496 lines of Zig into Rust in 11 days: about 64 Claude agents at peak, about $165K in API cost, 6,502 commits, no tests skipped [bun-rust].
  - The oracle was Bun's TypeScript test suite, which is independent of the implementation language.
  - Each task ran implementer → two diff-only adversarial reviewers → fixer, guided by shared `PORTING.md` and `LIFETIMES.tsv` files.
  - The stated reason for choosing Rust: "Compiler errors are a better feedback loop than a style guide." Most of Bun's memory bugs become compile errors in safe Rust [bun-rust].
  - Roc went the other way: a manual Rust→Zig rewrite took 487 days. Its reasons were compile speed, allocator control, and LLVM bitcode tooling [roc-zig].
  - **Inference:** both projects cost roughly $0.20–0.30 per line in API spend. Self-hosting or switching implementation languages is now an operational decision, not a multi-year one, provided a language-independent test suite exists.
- **Typed languages benefit from AI.** GitHub cites a 2025 study finding that 94% of LLM compilation errors were type-check failures [octoverse]. Type-constrained decoding cuts compile errors by more than half [mundler-pldi25].

### F3. The training-data problem for new languages

- **Direct generation in an unfamiliar language mostly fails.**
  - On CangjieBench, five of six models score under 7% pass@1 with direct prompting. About 83% of failures are syntax borrowed from other languages [cangjiebench].
  - A **2,146-token syntax reference** lifts GPT-5 from 4.8% to 50.4% on text-to-code, the best cost/accuracy trade-off.
  - Documentation RAG does worse (25%), because models that don't know the language write poor search queries.
  - Agent loops reach the highest scores (67–77%) at 10–120x the tokens [cangjiebench].
- **Fine-tuning teaches syntax, not semantics.**
  - With the invented language PyLang, fine-tuning drove syntax errors below 5%, but Python still beat PyLang by up to 19 points [pylang].
  - Code snippets were the biggest lever: Sonnet 4.5 went from 16.5% to 40.3%. Explicit rules alone helped little.
  - About 75–80% of failures used the right algorithm with a buggy implementation [pylang].
- **Strong agents adapt by metaprogramming.** Claude Opus 4.6 and GPT-5.4 wrote Python that generates esoteric-language code, and forbidding this caused large drops [metaprog].
- **Less-represented languages cost more tokens.** Agents spend significantly more tokens in OCaml, and often in Rust and Java, than in Python on the same problems, while accuracy stays mostly flat [tokenmaxx].
- **Semi-synthetic corpora work.** MultiPL-T translates Python code that has tests into a low-resource language and keeps only translations that pass. This doubled Racket and OCaml scores for fine-tuned models [multipl-t].
- **Docs plus execution beat retrieval.** An agent that explores official docs and runs code beats RAG baselines on Cangjie [ila-agent].
- **Usage share is not destiny.** Elixir has the best *pooled* AutoCodeBench score of 20 languages, but each language has only about 196 LLM-generated problems [autocodebench][codemyspec].
- **AI-designed languages exist.** MoonBit:
  - requires explicit types on all top-level definitions, which it says helps generation [deepeng-moonbit];
  - treats MCP as a first-class integration;
  - entered beta in June 2025 with a 1.0 target in 2026 [moonbit-beta].
- **Hallucinated dependencies are a supply-chain vector.** Across 16 models, 19.7% of recommended packages did not exist (5.2% for commercial models, 21.7% for open models). An empty `huggingface-cli` package got over 30K downloads [slopsquat-arxiv][csa-slopsquat].

### F4. Interop mechanisms

**C**
- Zig imports C headers directly. In 0.16 this moved from the language (`@cImport`) into the build system, an Aro-based `translate-c` package, to reduce coupling to LLVM [zig-016-cimport].
- Swift embeds Clang to import C and C++ through module maps [swift-cxx].

**C++** has proven to be a multi-year sink.
- Carbon, after years of Google investment, scoped 2025 to "most non-template C++ APIs" and still has no 0.1 [carbon-roadmap].
- Swift's C++ interop (5.9, 2023) imports only explicit template *specializations*, treats returned references as unsafe, and copies C++ value types [swift-cxx].
- Crubit's Rust→C++ direction reached Chromium only in July 2026. C++→Rust is still pending [crubit-chromium].
- The Rust Foundation's problem statement says interop is limited to "interfaces expressible in the C type system" and the boundary "is typically unsafe" [rf-problem].

**Rust**
- Rust's default layout makes "no other guarantees" beyond soundness [rust-layout].
- crABI, a richer ABI and a strict superset of the C ABI, is an experimental feature gate. Its tracking issue is open with no RFC, and the latest comment is from July 2026 [crabi].
- Workable tools today all generate C-ABI shims:
  - **cxx:** a bridge module with static assertions; zero or negligible overhead [cxx].
  - **Diplomat:** one Rust bridge definition generates C, C++, JS/TS, and Dart. ICU4X uses it [diplomat-icu].
  - **UniFFI:** Kotlin, Swift, and Python bindings from proc macros or a UDL file [uniffi].
  - **Zngur:** exposes Rust types, including generics such as `Vec<T>` and `Box<dyn Fn>`, to C++ [zngur].
  - **Crubit `cc_bindings_from_rs`:** generates C++ bindings from a Rust crate without hand-written declarations, including `Option`/`Result` and by-value Rust objects [crubit-chromium].
- stabby and abi_stable provide stable-ABI subsets for plugins. stabby warns of trait-object performance problems since Rust 1.78 and higher compile times [stabby].
- Machine-readable crate APIs (rustdoc `--output-format json`) are still nightly-only [rustdoc-json].
- **Inference:** every approach to crossing into Rust uses *explicit instantiation* of generics into a closed set of C-ABI shims. The crABI discussion also requires generics bound to concrete types [crabi].

**JVM, JS, BEAM, WASM**
- Kotlin's seamless Java interop and TypeScript's superset design both make *incremental, file-by-file adoption inside an existing codebase* possible [kotlin-java][ts-wiki].
- Gleam targets two runtimes with per-target `@external` declarations [gleam-ext].
- The WebAssembly Component Model lists C/C++, C#, Go, JS, Python, Rust, and MoonBit as component languages [wasm-cm].

### F5. Backends and bootstrapping

- **LLVM** is the default optimizer for Rust, Swift, Zig release builds, Crystal, and others. Its API churn is a real maintenance cost: Roc avoids the C++ API by writing LLVM bitcode directly, reusing Zig's serializer [roc-zig].
- **Cranelift** compiles about an order of magnitude faster than an LLVM-based Wasm engine, with code about 14% slower. It is differentially fuzzed against V8 and the Wasm spec interpreter [cranelift].
  - In rustc, Cranelift cuts codegen time by about 20%, which is only about 5% of clean-build time, because rustc's front end dominates.
  - Debug info is still missing [rcgc-goal].
- **Zig's own x86_64 backend** made Debug compiles "around a 5x decrease compared to LLVM in most cases" and is the default since 0.15 [zig-0151]. Debugger support regressed as a result [ziggit-lldb].
- **Emitting C** works in production:
  - Nim (default backend) [nim-wiki].
  - Lean ships a bundled Clang (`leanc`) [lean-build].
  - Zig uses its C backend in its bootstrap chain [zig-goodbye-cpp].
  - HACL\*, verified in F\*/Low\*, compiles to "readable, standalone C" via KaRaMeL [hacl], and that code now ships in CPython's `hashlib` [cpython-hacl].
  - Known costs: inherited undefined behavior, debug info only through `#line`, no portable guaranteed tail calls (Clang `musttail` only), and C-compiler speed and bugs [c-backend-notes].
- **Emitting Rust:**
  - In one anecdote, rustc caught 5 real bugs in a Rails→Rust transpiler's output [railcar].
  - Practitioners call the borrow checker the main obstacle to targeting Rust and suggest C or JS as first backends [railcar-search].
  - rustc compile speed is Rust's top pain point: 55% of 3,700+ respondents wait more than 10 s for an incremental rebuild, and about 45% of people who quit Rust cite compile times [rust-perf-2025].
- **MLIR/GPU:** Mojo is the MLIR-era bet and reached 1.0 only in Aug 2026 [modular-oss]. **HVM/Bend** is explicitly immature ("BEND IS YOUNG. EXPECT BUGS"; slow native compilation) [bend].
- **Self-hosting timelines:**
  - Rust: first compiler in about 38K lines of OCaml, later self-hosted on LLVM [rust-wiki].
  - Go: self-hosted at 1.5 (2015) by *automatic translation* of its C compiler, plus "custom tools created for the job" [go15].
  - Zig: replaced about 80K lines of C++ with a 637 KB compressed `zig1.wasm` seed plus a 4K-line C `wasm2c`, after years of maintaining two compilers [zig-goodbye-cpp].
  - TypeScript went the other way: from self-hosted to a Go port (TS 7.0, July 2026), about 10x faster. Go was chosen because a *port* needed structural similarity and a GC [ts-go-why][ts-wiki].

### F6. Package management and supply chain

- **Go's model:**
  - `go.mod` fully determines builds, with minimal version selection and no automatic upgrades;
  - `go.sum` hashes plus a global append-only checksum database;
  - VCS as the source of truth, with "no such thing as a package repository account";
  - "neither fetching nor building code will let that code execute" [go-supply].
- **npm:**
  - left-pad (2016) broke thousands of builds and led to unpublish limits [leftpad].
  - The self-replicating Shai-Hulud worm (Sept 2025) spread through stolen tokens and **post-install scripts**. GitHub removed 500+ packages and announced 7-day granular tokens, deprecation of classic tokens, FIDO 2FA, and trusted publishing [gh-npm-plan].
- **The xz backdoor (2024)** hid in a release-tarball build script that was not in git [xz].
- **crates.io:**
  - Added OIDC trusted publishing in July 2025 [crates-tp].
  - Removed the typosquats `faster_log` and `async_println`, which stole crypto keys at runtime [crates-malware].
- **Capability tooling:**
  - Capslock reports which privileged stdlib operations a Go package can transitively reach [capslock].
  - cackle enforces per-crate API ACLs and sandboxes build scripts and proc macros [cackle].
  - Austral makes I/O require unforgeable capability values derived from a root capability passed to `main` [austral].
- **llms.txt** (2024) is the de facto convention for agent-facing docs [llmstxt].

---

## What works / what failed and why

**What works** (pattern across F1–F6; **inference** where not directly stated)

1. **Borrow an existing ecosystem from day one.**
   - Kotlin→Java, TypeScript→JS, Elixir and Gleam→BEAM, Swift→Objective-C, and Zig→C all launched with immediate access to a large library base.
   - Rust is the counterexample that proves the cost: without a borrowed ecosystem, it needed about a decade and a platform-level champion.
2. **Allow incremental adoption inside existing codebases.** One file or one module at a time: TS, Kotlin, Rust-in-Android, Rust-in-kernel. Nobody rewrites to adopt a language.
3. **Get a platform champion or a flagship app.** Examples: Apple, Google Android, Docker/Kubernetes, Phoenix, Bun, Flutter. Dart was technically sound and failed until Flutter.
4. **Ship one integrated toolchain at launch.** Cargo, the `go` tool, Gleam v1, and MoonBit all bundle build tool, package manager, formatter, and LSP.
5. **Promise stability and keep it.** Go 1, Gleam semver, and Mojo 1.0 "source stability" all did. Breaking churn (Elm 0.19, Zig pre-1.0) costs trust. For LLMs it also makes training data stale, though that second effect is inference.
6. **Make the check loop fast.** Go's raison d'être, Zig's 5x Debug speedup, and TypeScript 7's 10x port all target it. Rust's compile time is its most persistent complaint and a reason people quit.
7. **In 2025–26, static checking helps you win with agents.** TypeScript's rise and Bun's rewrite both cite compiler feedback as what makes AI output converge.

**What failed**

1. **Overpromising (V).** Credibility is a one-shot resource.
2. **Closing the FFI escape hatch (Elm)**, which turned a purity win into an adoption wall.
3. **Interop with a language too complex to model (Carbon↔C++).** Even Google-scale effort hasn't reached an MVP. Swift needed nine years to add C++ interop.
4. **Superset promises that can't be kept (Mojo↔Python)**, which create expectation debt.
5. **Depending on another party's platform decision (Dart-in-Chrome).**
6. **No flagship and no champion (Nim, Crystal)**, which means good languages stay niche for a decade.
7. **Self-hosting as a vanity milestone.** Zig carried two compilers for years. TypeScript left self-hosting for 10x speed. Go only self-hosted by mechanical translation.

---

## Recommended path

### 1. The most credible route to real users in 2–3 years

**Set honest expectations.** Gleam, Zig, and Elixir took 5–10 years to reach visible but niche use. "Real users in 2–3 years" realistically means hundreds of production users and a handful of libraries others depend on, not mainstream adoption.

**Inference: the credible wedge is "the Rust ecosystem, a lighter language, an agent-grade check loop, proofs when you want them."** Concretely:

1. **Position Ferox as a Kotlin-to-Java for Rust, not a Rust replacement.**
   - Any crates.io crate should be callable from Ferox in its first public release.
   - Ferox libraries should be consumable from Rust as ordinary crates.
   - Teams then adopt Ferox one module at a time inside Rust workspaces, which is the Kotlin/TS adoption shape.
2. **Make "eject to readable Rust" a promise.** If Ferox can emit idiomatic-enough Rust (see §2), choosing Ferox is reversible, which cuts adoption risk the way "TS is just JS" did. (Inference; no direct precedent measured.)
3. **Sell measurable agent economics, not taste.** Publish a reproducible benchmark that compares agents writing Ferox against agents writing Rust on the same tasks. Report pass@1, tokens per solved task (the method from [tokenmaxx]), and wall-clock to green. The thesis is falsifiable: if Ferox doesn't beat Rust on agent cost for equivalent safety, the pitch fails.
4. **Pick a flagship niche where proofs and agents pay off.** Good candidates are parsers, codecs, protocol state machines, and crypto-adjacent libraries. Write them in Ferox with proofs and ship them to Rust and C users as plain crates and C libraries. HACL\* shows verified code gets adopted when it arrives as plain code in an existing ecosystem (CPython's `hashlib`) [cpython-hacl].
5. **Integrate with agent harnesses as the new IDE.** Kotlin rode JetBrains' IDE. MoonBit ships an agent and treats MCP as first-class [deepeng-moonbit]. Ferox should ship a compiler-served agent card, MCP/LSP endpoints, and Claude Code/Codex skills on day one.
6. **Find a champion.** Kotlin's growth jumped at Google's announcements [kotlin-5y]. The 2026 analog may be an agent-platform vendor or a large Rust shop that wants faster agent loops. (Speculative.)

### 2. Interop and backends

**Overall shape (medium confidence).** One front end with its own ownership/type/proof checker produces a typed Ferox IR. Three consumers sit behind it:

| Consumer | Purpose | Why |
|---|---|---|
| **Reference interpreter** | Executable semantics, test oracle, comptime evaluator | Agents need a near-perfect oracle [anthropic-ccc]. Cranelift's own reliability comes from differential fuzzing against a spec interpreter [cranelift]. |
| **Cranelift backend** | Dev builds, tests, JIT; also a "no Rust toolchain" release mode | About 10x faster compiles than LLVM for about 14% slower code [cranelift]. It is a Rust library, so it embeds in a Rust-written compiler. Zig shows an owned fast backend yields about 5x [zig-0151]. |
| **Rust-source backend** | Release builds that need peak speed or Rust deps; crate export; "eject" | Calls Rust generics natively with no ABI work. Gets LLVM, every rustc target including wasm, LTO with Rust deps, and a distribution channel on crates.io. |

**Rust-source emission: the honest trade-off.**

- **Borrow checking "for free" is a trap as the *primary* checker.** rustc errors in generated code are unreadable to Ferox users, and Rust compile time is the very problem Ferox should solve [rust-perf-2025].
- **As a *second* checker it is valuable.** Ferox's checker is authoritative. The emitter should produce safe Rust references wherever Ferox's model maps cleanly, so rustc independently re-verifies most of the program. Any rustc rejection is then a *Ferox compiler bug* caught for free. (Inference.)
- **Track the share of emitted code that is in `unsafe` blocks** as a design-health metric. Bun's mechanical port landed at about 4% [bun-rust]. Ferox should aim lower, because its model is designed to map.
- **Debuggability risk.** To my knowledge Rust has no `#line` equivalent, so panics and debug info point at generated `.rs` files. I did not verify this in this session. Plan a source-map plus a `ferox symbolize` tool, and keep emission structurally 1:1.
- **Kill criteria.** Switch release codegen to direct LLVM bitcode (Roc/Zig style, avoiding the C++ API [roc-zig]) if:
  - emitted code needs `unsafe` in more than about 10% of lines, or
  - release builds of generated Rust are more than 2x slower than equivalent hand-written Rust.

**Calling Rust crates in dev builds (glue mode).** Emit only a *boundary* crate, so the same emitter serves both modes:

- After type checking, Ferox knows every concrete generic instantiation it uses. This is the Swift-C++ / Zngur / crABI "explicit instantiation" pattern [swift-cxx][zngur][crabi].
- For each `(item, instantiation)` pair, emit a `#[no_mangle] extern "C"` wrapper, plus `size_of`/`align_of` probes and drop functions.
- Build the boundary crate with cargo. Cache it content-addressed by (crate version, instantiation set, rustc version). Link it into the Cranelift objects.
- Read crate APIs from rustdoc JSON on a pinned nightly. Its instability is a known risk [rustdoc-json].
- Map types as follows:
  - Rust values become opaque, sized Ferox values with move semantics;
  - `&T` and `&mut T` become borrows;
  - `Option` and `Result` cross through `repr(C)` mirrors;
  - closures cross as callback plus context;
  - panics are caught at the boundary.
- Defer async, macros/derive, and lifetime-heavy APIs. The escape hatch is a small hand-written Rust shim, which agents write well.
- Don't depend on crABI, stabby, or abi_stable. Keep the boundary layer swappable for when crABI stabilizes [crabi][stabby].

**C interop:**

- Import headers at build-system level, as Zig 0.16 does, rather than in the language [zig-016-cimport]. Use libclang through bindgen-style tooling.
- `extern` declarations carry an explicit trust marker, and proofs never cross them. Gleam is a model of honest, typed-but-unchecked externals [gleam-ext].
- Don't replicate `zig cc` early. It is a large maintenance surface. (Inference.)

**Deliberately skip for 3 years:** C++ interop (see Carbon and Swift [carbon-roadmap][swift-cxx]), MLIR and GPU [modular-oss], HVM [bend], and a C backend.

**When a C backend would earn its place.** Only if you need a bootstrap seed, C consumers without a Rust toolchain, or exotic targets. Then follow Zig/Nim/HACL\*: emit `#line` everywhere, use explicit wrapping or checked arithmetic helpers, and add no UB-dependent idioms [c-backend-notes][hacl].

### 3. Bootstrapping plan for a small, agent-heavy team

**Implementation language: Rust. Do not self-host before 1.0, and possibly never.**

- Cranelift, bindgen, rustdoc-JSON types, and the rustc ecosystem are all Rust.
- Agents produce strong Rust when compiler errors guide them [bun-rust][anthropic-ccc][infoq-rue].
- Mitigate Rust's compile-time drag, the reason Roc left [roc-zig]: many small crates, few proc-macro-heavy dependencies, Cranelift for the compiler's own dev builds, and a fast linker.
- If self-hosting ever makes sense, do it as a Bun- or Go-1.5-style *mechanical port* [bun-rust][go15]. That only works if, from day one, **the conformance suite is written in Ferox and is independent of the compiler's implementation language**.

**Build order** (months are rough for 2–4 humans with agent fleets):

| Phase | Months | Deliverables |
|---|---|---|
| 0 | 0–2 | Core semantics as a **small reference interpreter**. Conformance-test format (Ferox source plus expected output and diagnostics). `ferox check --json` with machine-applicable fixes. Formatter. A generated **agent card** of 2–3K tokens (syntax plus idioms plus examples), sized from CangjieBench's 2,146-token result [cangjiebench]. |
| 1 | 2–5 | Type and ownership checker; incremental check daemon (target under 200 ms for single-file edits); test runner with doctests; minimal LSP; Cranelift JIT for tests; C FFI. |
| 2 | 5–9 | Package manager (design in §Implications); Rust glue mode; first real crates exercised (e.g. `regex`, `serde_json` value API); random program generator plus differential testing (interpreter vs Cranelift). |
| 3 | 9–15 | Rust-source release backend; crate export; standard library; optional contracts and proofs (SMT-backed); **corpus pipeline**: MultiPL-T-style translation of tested Python and Rust tasks, keeping only those that pass [multipl-t], published openly; Ferox ports of HumanEval/MBPP as benchmarks. |
| 4 | 15–24 | Flagship verified libraries shipped to Rust users; the agent-economics benchmark; 1.0 stability promise with editions and `ferox migrate`. |

**Agent workflow rules, taken from the projects that worked:**

- Every task needs a near-perfect verifier: the interpreter oracle, conformance tests, and differential fuzzing [anthropic-ccc][cranelift].
- Keep shared guide files (`PORTING.md`-style idiom maps).
- Use separate-context adversarial reviewers that see only the diff, and reject workarounds that need a paragraph of justification [bun-rust].
- Make test output terse and grep-able, with a sampling `--fast` mode [anthropic-ccc].
- Fix the process, not the output [bun-rust].

---

## Design implications (ranked, with confidence)

1. **Executable reference semantics first, used as the oracle for everything** (interpreter, Cranelift, Rust emission, and later a self-host port). *High.* Every AI-built compiler that worked had a near-perfect oracle [anthropic-ccc][bun-rust]. A new language has none unless it builds one.

2. **Rust-familiar surface syntax wherever semantics match; deliberately different syntax only where semantics differ; machine-applicable fixes for common Rust/TS-isms.** *High.* Borrowed foreign syntax causes about 83% of failures in an unfamiliar language [cangjiebench]. Fine-tuning fixes syntax but not semantics [pylang], so the cheapest win is making the syntax models already produce correct, or auto-fixed.

3. **The compiler serves version-matched, agent-sized docs.** Concretely: `ferox docs --agent` (a 2–3K-token card with many short examples), `llms.txt`, an MCP endpoint, and `ferox explain <diag>`. *High.* A small syntax reference gave about a 10x lift; examples beat rules [cangjiebench][pylang][llmstxt]. Shipping docs inside the compiler prevents stale-version drift.

4. **Crates.io access in the first public release through explicit-instantiation C-ABI glue, then Rust-source emission for release builds and crate export.** *Medium.* Every working Rust interop tool uses generated shims and closed instantiation sets [zngur][crubit-chromium][diplomat-icu][uniffi]. crABI is not usable [crabi]. Borrowing an ecosystem is the most consistent adoption driver (F1).

5. **Two-tier codegen: Cranelift for dev (and an optional no-Rust release), rustc for peak-performance release builds.** Measure check and rebuild latency as product KPIs. *Medium-high.* Cranelift gives about 10x faster compiles than LLVM for about 14% slower code [cranelift]. Zig's owned backend gave about 5x [zig-0151]. Rust's pain is incremental latency [rust-perf-2025]. Note that inside rustc Cranelift saves only about 5%, because the front end dominates [rcgc-goal]. **Ferox's front end must be designed for speed too.** That means no proc macros, restricted trait solving, and separate compilation with export data, as Go does [go-splash].

6. **No code execution at fetch or build time.** No `build.rs` and no install hooks. Compile-time code runs only in the sandboxed, deterministic interpreter. Rust dependencies' build scripts run inside a sandbox and are flagged as a capability. *High.* This follows Go's model [go-supply]. Shai-Hulud used post-install scripts [gh-npm-plan], and xz hid in a build script [xz]. cackle shows sandboxing Rust build scripts is practical [cackle].

7. **Supply-chain defaults built in from day one:**
   - lockfile with hashes;
   - append-only transparency log;
   - minimal version selection;
   - OIDC trusted publishing only, with no long-lived tokens;
   - namespaced package names;
   - registry-side blocking of names close to popular packages by edit distance;
   - `ferox add` verifies that a package exists and shows publisher and provenance before editing the manifest.

   *High* for the first four [go-supply][gh-npm-plan][crates-tp], *medium* for the anti-slopsquatting measures [slopsquat-arxiv]. An agent-first language must assume agents will propose hallucinated dependencies.

8. **Package-level capability manifests (`net`, `fs`, `env`, `process`, `ffi`, `unsafe`) checked by the compiler, with transitive capability diffs shown on upgrade.** Language-level capability values (Austral-style) for std I/O are optional. *Medium.* This draws on Capslock, cackle, and Austral [capslock][cackle][austral]. The ergonomic cost of full capability-passing is the open risk; see research note 07.

9. **Stability machinery before 1.0:** editions, `ferox migrate`, and a published compatibility promise. Never ship breaking changes without an automated migration. *High.* Go 1, Gleam, and Mojo kept such promises [go-wiki][gleam-v1][modular-oss]. Elm broke one [elm-derw]. For agents, churn also turns the existing corpus into wrong answers (inference).

10. **The test and conformance corpus is written in Ferox, not in the host language.** *High.* This is what makes a later AI-driven self-host port or backend swap cheap [bun-rust][go15]. It doubles as training and eval data.

11. **Publish a semi-synthetic, test-validated corpus openly** (MultiPL-T method), plus Ferox ports of standard benchmarks. *Medium.* Proven for fine-tuning [multipl-t]. Whether it moves frontier pretraining is unknown.

12. **Honest, reproducible benchmarks in every claim.** Never advertise unimplemented features. *High.* V's credibility collapse is the cautionary tale [v-critique].

13. **No C++ interop, MLIR, GPU, or HVM before 1.0.** *High.* Carbon, Swift, Mojo, and Bend show these are multi-year efforts [carbon-roadmap][swift-cxx][modular-oss][bend].

14. **Expose a prefix or incremental type-check API** usable for constrained decoding by open models and tools. *Low-medium.* This halves compile errors in research settings [mundler-pldi25], but API-hosted Opus-class models can't use logit constraints today.

---

## Open questions

1. **Can Ferox's ownership model map to *safe* Rust for most programs?** If its model is simpler (for example second-class references or mutable value semantics, as in Rue [infoq-rue]), mapping should be easy. If it adds features Rust lacks (effects, refinements), emission falls back to `unsafe`. This needs a prototype measuring the `unsafe` share.
2. **Rust has no `#line`.** How bad is debugging release builds through generated Rust in practice? Is a source-map plus symbolizer enough?
3. **How do we read crate APIs stably?** rustdoc JSON is nightly-only [rustdoc-json]. The alternatives, rust-analyzer's library crates or a custom rustc driver, are also unstable. What pinning and caching policy keeps this from breaking monthly?
4. **What is the actual agent-economics delta vs Rust?** The pitch depends on Ferox beating Rust on tokens and latency at equal safety. No benchmark exists yet, so build it early (Phase 1) and let it kill bad design ideas.
5. **Does a published synthetic corpus influence future frontier-model pretraining?** Or is in-context documentation the whole game for Opus-class agents? The evidence so far supports in-context documentation [cangjiebench][metaprog].
6. **Which flagship niche?** Verified libraries, agent tooling, or something else? This needs user interviews, not desk research.
7. **Community stance on AI-written code.** Zig's community, for example, is skeptical, while Bend and Rue embrace it. How Ferox positions an "AI-first" language without alienating human contributors is unresolved. (Not researched in depth here because the search budget ran out.)

---

## Sources

- [android-2025] https://blog.google/security/rust-in-android-move-fast-fix-things/
- [anthropic-ccc] https://anthropic.com/engineering/building-c-compiler
- [austral] https://borretti.me/article/introducing-austral
- [autocodebench] https://arxiv.org/abs/2508.09101
- [bend] https://github.com/HigherOrderCO/Bend
- [bun-rust] https://bun.com/blog/bun-in-rust
- [c-backend-notes] https://compilers.iecc.com/comparch/article/09-02-058 ; https://learn.microsoft.com/en-us/archive/blogs/jmstall/compiling-a-language-to-c ; https://homes.cs.washington.edu/~akcheung/papers/apsys12.html ; https://discourse.llvm.org/t/experience-with-clang-musttail/89085
- [cackle] https://github.com/cackle-rs/cackle
- [cangjiebench] https://arxiv.org/html/2603.14501
- [capslock] https://github.com/google/capslock
- [carbon-roadmap] https://docs.carbon-lang.dev/docs/project/roadmap.html
- [codemyspec] https://codemyspec.com/blog/autocodebench-explained
- [crabi] https://github.com/rust-lang/rust/issues/111423
- [cranelift] https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/README.md
- [crates-malware] https://blog.rust-lang.org/2025/09/24/crates.io-malicious-crates-fasterlog-and-asyncprintln/
- [crates-tp] https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07/
- [crubit-chromium] https://groups.google.com/a/chromium.org/g/rust-dev/c/Kixvw8bhfME
- [crystal-wiki] https://en.wikipedia.org/wiki/Crystal_(programming_language)
- [csa-slopsquat] https://labs.cloudsecurityalliance.org/research/csa-research-note-slopsquatting-ai-supply-chain-20260419/
- [cpython-hacl] https://github.com/python/cpython/tree/main/Modules/_hacl
- [cxx] https://cxx.rs/
- [dart-wiki] https://en.wikipedia.org/wiki/Dart_(programming_language)
- [deepeng-moonbit] https://deepengineering.net/p/designing-for-ai-and-humans-with
- [devclass-rust] https://devclass.com/2025/12/15/rust-boosted-by-permanent-adoption-for-linux-kernel-code/
- [diplomat-icu] https://unicode.org/events/utw/2024/documents/slides/UTW-Diplomat.pdf ; https://searchfox.org/mozilla-central/source/intl/icu_capi/src/lib.rs
- [elixir-wiki] https://en.wikipedia.org/wiki/Elixir_(programming_language)
- [elm-derw] https://derw.substack.com/p/whatever-happened-to-elm-anyway
- [elm-native] https://discourse.elm-lang.org/t/clear-answer-to-the-question-of-native-modules-in-0-19-and-beyond/822
- [gh-npm-plan] https://github.blog/security/supply-chain-security/our-plan-for-a-more-secure-npm-supply-chain/
- [gleam-ext] https://gleam.run/documentation/externals/
- [gleam-v1] https://gleam.run/news/gleam-version-1/
- [go-splash] https://go.dev/talks/2012/splash.article
- [go-supply] https://go.dev/blog/supply-chain
- [go-wiki] https://en.wikipedia.org/wiki/Go_(programming_language)
- [go15] https://go.dev/doc/go1.5 ; https://go.googlesource.com/proposal/+/master/design/go15bootstrap.md
- [hacl] https://github.com/hacl-star/hacl-star
- [ila-agent] https://arxiv.org/abs/2602.06976
- [infoq-rue] https://infoq.com/news/2026/01/steve-klabnik-rue-language-ai/
- [infoworld-mojo] https://www.infoworld.com/article/4081105/revisiting-mojo-a-faster-python.html
- [kotlin-5y] https://android-developers.googleblog.com/2022/08/celebrating-5-years-of-kotlin-on-android.html
- [kotlin-java] https://kotlinlang.org/docs/java-interop.html
- [lean-build] https://lean-lang.org/doc/reference/latest/Build-Tools-and-Distribution/
- [leftpad] https://en.wikipedia.org/wiki/Npm_left-pad_incident
- [llmstxt] https://llmstxt.org/
- [lwn-rust] https://lwn.net/Articles/1049831/
- [metaprog] https://arxiv.org/abs/2606.10933
- [modular-oss] https://modular.com/blog/mojo-open-source
- [moonbit-beta] https://www.moonbitlang.com/blog/beta-release
- [multipl-t] https://arxiv.org/html/2308.09895v6
- [mundler-pldi25] https://arxiv.org/abs/2504.09246v1 ; https://pldi25.sigplan.org/details/pldi-2025-papers/25/Type-Aware-Constraining-for-LLM-Code-Generation
- [nim-wiki] https://en.wikipedia.org/wiki/Nim_(programming_language)
- [nimony-medium] https://medium.com/@s.quddin/nim-3-0-a-deep-dive-into-the-next-generation-of-nim-6cd2c92cba5c
- [octoverse] https://github.blog/news-insights/octoverse/octoverse-a-new-developer-joins-github-every-second-as-ai-leads-typescript-to-1/
- [pylang] https://arxiv.org/html/2605.15607v1
- [railcar] https://intertwingly.net/blog/2026/04/16/Railcar-Rust.html
- [railcar-search] https://lemmy.brendan.ie/comment/1874376
- [rcgc-goal] https://goals.rust-lang.org/2025h2/production-ready-cranelift.html
- [rf-problem] https://github.com/rustfoundation/interop-initiative/blob/main/problem-statement.md
- [roc-zig] https://rtfeldman.com/rust-to-zig ; https://weeklyrust.substack.com/p/why-roc-is-moving-away-from-rust
- [rust-layout] https://doc.rust-lang.org/reference/type-layout.html
- [rust-perf-2025] https://blog.rust-lang.org/2025/09/10/rust-compiler-performance-survey-2025-results
- [rust-wiki] https://en.wikipedia.org/wiki/Rust_(programming_language)
- [rustdoc-json] https://doc.rust-lang.org/nightly/rustdoc/unstable-features.html
- [slopsquat-arxiv] https://arxiv.org/abs/2406.10279v3
- [so-2025] https://survey.stackoverflow.co/2025/technology/
- [stabby] https://docs.rs/crate/stabby/5.1.0 ; https://docs.rs/crate/xabi/0.1.1
- [swift-cxx] https://www.swift.org/documentation/cxx-interop/
- [swift-wiki] https://en.wikipedia.org/wiki/Swift_(programming_language)
- [tokenmaxx] https://arxiv.org/html/2607.22807v1
- [ts-go-why] https://github.com/microsoft/typescript-go/discussions/411
- [ts-wiki] https://en.wikipedia.org/wiki/TypeScript
- [uber-zig] https://www.uber.com/blog/bootstrapping-ubers-infrastructure-on-arm64-with-zig/
- [uniffi] https://mozilla.github.io/uniffi-rs/latest/
- [v-critique] https://compiler-meetup.cs.illinois.edu/slides/New-Systems-Languages.pdf ; https://www.eevblog.com/forum/programming/v-lang-(programming-language)/msg4844048/
- [wasm-cm] https://component-model.bytecodealliance.org/
- [xz] https://en.wikipedia.org/wiki/XZ_Utils_backdoor
- [zig-0151] https://ziglang.org/download/0.15.1/release-notes.html
- [zig-016-cimport] https://ziglang.org/download/0.16.0/release-notes.html ; https://ziggit.dev/t/link-against-static-c-library-with-zig-0-16/15234/2
- [zig-goodbye-cpp] https://ziglang.org/news/goodbye-cpp/
- [ziggit-lldb] https://ziggit.dev/t/no-symbol-in-lldb-debugger/12511
- [zngur] https://github.com/HKalbasi/zngur
