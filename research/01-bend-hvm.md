# 01 — Bend, HVM, interaction nets, and HigherOrderCO's direction

Research date: 2026-10-19. Scope: Bend 1 and 2, HVM1 through HVM5, Kind, interaction combinators, superpositions and SupGen, and Higher Order Company (HOC) strategy. Numbers in brackets refer to the Sources list. Text marked **Inference** is my reasoning, not something a source states.

**Correction — 2026-11-12.** Bend's canonical home is
[bendlang/bend](https://github.com/bendlang/bend), and HVM's former repository
is now [HigherOrderCO/HVM1](https://github.com/HigherOrderCO/HVM1).
[2.0.40, published November 6](https://github.com/bendlang/bend/releases/tag/v2.0.40)
is the latest Bend release as of this correction; 2.0.37 in F1 belongs to
this report's 2026-10-19 snapshot. The native library target (#813; the issue
itself is closed as `not_planned`) remains
[“planned, not scheduled”](https://github.com/bendlang/bend/blob/f311527776559ce40095108ad6a7514a6c34aaa1/WONTFIX.txt#L152-L154).
See [report 11's dated correction](11-bend-interop.md#correction--2026-11-12)
for the release cadence and unchanged interop staging. Findings below are
preserved as dated evidence.

## Question

What are Bend and HVM, and what do they deliver against native code? Which of their ideas are new and worth keeping? And which of five options should a Rust-like, agent-first language with optional proofs take?

- (a) inherit surface ideas
- (b) compile a pure subset to HVM
- (c) use superposition-based search
- (d) interoperate with Bend
- (e) ignore them

**Bottom line.** In September 2026 HOC itself dropped interaction nets as Bend's runtime. Bend 2 is an affine, dependently typed language that compiles to one C file, with fork-join parallelism and no garbage collector. Taelin wrote that he "could not make [interaction nets] as fast as lower-order variants on everyday hardware" [10]. The ideas that survived (ownership-driven memory, pure fork-join, proof-gated builds for AI agents) are close to what a Rust-like agent-first language would build anyway. So Bend 2 is the closest peer and competitor to this project, not a backend for it. HVM now continues as the engine for SupGen, HOC's unreleased program-synthesis system. SupGen is interesting but has not been demonstrated publicly.

## Findings

### F1. Lineage and current state (as of 2026-10-19)

| Artifact | What it is | Status |
|---|---|---|
| Interaction combinators | Lafont's graph-rewriting model: three node types, local and confluent rewrites [35] | Theory, 1997 |
| HVM1 (2022) | Lazy evaluator for interaction nets, in Rust, with near-"optimal" sharing [32] | Superseded |
| HVM2 (2024) | Strict, massively parallel interaction-combinator evaluator, compiled to C and CUDA [27][28] | Repo kept, no longer Bend's runtime [26] |
| Bend 1 (May 2024) | Python-like surface over HVM2, with `fold`/`bend` [33] | "Superseded" [26] |
| HVM3 (2024–25) | Haskell prototype of the lazy, optimal Interaction Calculus. Adds superpositions and "unordered" superpositions [29][21][24] | Work in progress, MIT [29] |
| SupGen (2025–) | Program search by superposed enumeration on HVM [19][15] | Proprietary. Cut from Bend 2's launch [13][46] |
| HVM4 (2025–26) | HVM rewritten in C with a type system on interaction nets and SupGen built in [15][30] | Public, "before launch" [30] |
| HVM5 (Jun 2026) | Reported about 5x faster than HVM4 [14] | Not public as far as I found |
| Kind | HOC's earlier minimal proof checker [34] | Repo last updated Jan 2025 [26] |
| Bend 2 (17 Sep 2026) | Affine dependent types, compiles to C, Metal, CUDA and JS, with laws and proofs. No interaction nets at runtime [1][2][11][47] | v2.0.37 on 13 Oct 2026 [7]. Apache-2.0, about 23.8k stars, 3,299 commits [2] |
| BendAI / "Bender" | Paid proving agent. Currently "a thin harness around models from Anthropic, OpenAI and others" [25] | API "within the coming days" [25] |

Taelin's November 2025 retrospective on Bend 1 and HVM2 [12] listed these problems:
- interpretation overhead still significant
- a "full RTX 4090 [was needed] to beat 1-core OCaml / JavaScript"
- int24, no IO, no packages
- "most devs can't think recursively"
- HVM2's strict mode was "not β-optimal"

He then "[threw] HVM2 away" and returned to the lazy, optimal line (HVM3, then HVM4) to research program synthesis [12]. At that point Bend 2 was promised to have a "lazy/optimal mode", 16/32/64-bit numbers and "SupGen integration" [12]. What actually shipped ten months later had none of those [2][13]. The pre-launch post says: "Bend2 doesn't have interaction nets ... designed to be practical, not idealistic ... closer to C than it is to Haskell" [10].

### F2. Execution models

**HVM (interaction nets).** A program is a graph. Each rewrite touches only two connected nodes and the system is confluent, so any number of threads can reduce independent pairs at once with no scheduling decisions [28][35]. This is where the "automatic parallelism" comes from.

DUP nodes copy values lazily and incrementally, so shared subterms (including work under lambdas) are reduced once. This is the "optimal" (Lévy/Lamping-style) sharing [31][32]. SUP nodes are the dual: two values stored in one location. A computation applied to a SUP runs once on the shared parts and forks only where the two values differ [22][31].

The costs:
- HVM's bookkeeping-free "light lambdas" do not cover the full λ-calculus. HVM1's docs say the fix (Lamping bookkeeping) "affects the performance of beta-reduction by about 10x" [32].
- The HVM2 paper states that a source language must enforce "a higher-order lambda that clones its variable can not be cloned". Otherwise reduction is unsound; Church 2^2 is its example [28].
- HVM2 is "ultra-eager". It evaluates both branches unless code is split into lazily unfolded definitions, and infinite structures hang [28].
- Optimal reduction in general has non-elementary worst-case cost [36], and HVM inherits these bounds [32].

**BendRT (Bend 2's runtime)** [4]. The compiler emits one C file. The same code runs on one thread, on CPU threads, or on the GPU through Metal or CUDA, over one shared 64-bit heap.

- **Memory.** Affinity from the type checker gives each value one owner. A `match` frees the node it consumes. Reference counts exist only for "hot" types that whole-program share inference finds. Borrow inference passes read-only arguments without count traffic. There is no tracing GC [4].
- **No C stack.** Every function becomes a segment of one worklist state machine, which is what lets the same C text run as a GPU shader [4].
- **Parallelism.** Only two marks create it:
  - the parallel let `l r = f(x) g(y)`, which promises the calls are independent and of roughly equal cost
  - `f!(x)`, which runs a call on the GPU [3][4]

  The scheduler is a fixed 128×128 grid of rings that alternates grow and work phases. It has no work stealing: "balance is the program's job", and "a skewed program silently loses its parallelism" [4].

The paper puts the trade plainly: "BendRT keeps the goals and drops the mechanism ... Lost is optimal reduction of shared redexes; gained are native-speed sequential code, flat memory and a cost model a programmer can read" [4]. On Hacker News Taelin said HVM remains asymptotically better for λ-calculus-heavy workloads, while Bend 2 wins on raw speed [46].

**BendTT (Bend 2's type theory)** [5].
- One universe with `Type : Type`, impredicative function types, and no positivity check.
- Consistency comes from splitting every term in two:
  - **Live** (running) code must be affine. It may copy only `Data`, meaning first-order values with no functions. Recursion must be on structurally smaller arguments.
  - **Dead** code (types, erased arguments, proofs about equations) is exempt.
- Confluence, subject reduction, progress, termination of live code, and consistency are proved in a roughly 4,000-line Lean 4 file whose checker is the one `bend --verdict` runs [5].
- The TypeScript checker and the translation into the kernel are not verified [2][5].
- The costs [5][3]:
  - a closure may be called at most once, so the usual `map(f, xs)` is rejected
  - workarounds are compile-time `~` templates or a separate definition per loop
  - no mutual recursion
  - server loops need a fuel counter
  - "almost no inference"
  - matching is allowed only on variables

### F3. Performance evidence

**HVM2 and Bend 1 (2024): HOC's own figures**
- 400 MIPS on one M3 Max thread, 5,200 MIPS on 16 threads, 74,000 MIPS on an RTX 4090 [28].
- Bitonic sort: 12.33 s on one thread, 0.96 s on 16 threads, 0.24 s on the 4090 [33].
- The paper itself says single-thread HVM2 is "about 5x slower than GHC", growing "to 100x on programs that involve loops and mutable arrays" [28].
- HVM1 docs said single-thread performance ranged from "50% faster to 3x slower" than GHC [32].

**HVM2 and Bend 1 (2024): third parties**
- **SSCAD 2024** (Mackenzie University) [40]: n-body (12 bodies, 50k iterations) on an i5-1235U.
  - Bend 0.2.36: 255.28 s elapsed, 352,224 KB peak RSS
  - Python multiprocessing: 971.87 s, 44,364 KB
  - C+OpenMP: 0.82 s, 2,804 KB
  - So Bend was about 311x slower than OpenMP and used about 125x its memory.
- **Speedfox** [39]: 11×11 cofactor determinant.
  - Bend: about 2 h single-threaded, 20 min multi-core
  - Go: 22 s single-threaded, 4 s with sensible parallelism
  - The author blames linked-list "arrays" (O(i) indexing) and 24-bit integers.
- **HN launch thread** [37]: pypy ran the recursive-sum example in 4.5 s, while a single-thread Bend run was still going after 42 minutes at 6 GB. Taelin conceded that "the codegen is still abysmal", that there was no tail-call optimization, and that ints were 24-bit.
- **Lobsters** [38]: rising "interactions per second" shows the GPU is kept busy, not that the code is efficient.

**Bend 2 (2026): HOC's own figures** [4]
- Apple M4 Max, 12 benchmarks.
- Sequential: 0.8–1.5x the time of a "hand-written C twin, one C function per Bend definition". Example: radix sort 3.89 s vs 2.67 s; game of life 5.46 s vs 6.82 s.
- 16 threads: 8.8–12.1x over one thread.
- Integrated GPU: 52–67x on uniform work. Divergent work loses to 16 CPU threads; n-queens takes 1.24 s on the GPU vs 0.46 s on the CPU.
- The CUDA lane "is not measured" [4].
- **Inference:** the C twin follows the same algorithm and data shapes as the Bend program, not idiomatic optimized C. So "near-C" means near-C for tree-shaped recursive code.

**Bend 2 (2026): third parties**
- **Akita** ported two Rust CLI tools on launch day [41][42].
  - The ports were correct; ghpending's output was "byte-identical" to the Rust version.
  - The workloads were network- or idle-bound, so they are not compute benchmarks. The author: "No number here shows Bend faster than Rust on the same work."
  - Binaries were much smaller (1.4 MB vs 6.8 MB). The clean build was slower (62 s vs 57 s, almost all of it clang on one C file), with "no incremental builds".
- **Astropulse** had Claude port a numeric image tool [43][54].
  - Outputs were identical to Rust on 18/18 images.
  - It ran "about 4x slower than Rust" (Rust used rayon, rustfft and f64). Fork-join gave about 2x on 6 threads.
  - The first list-based port took 3.6 min per image; Bend arrays brought that to 4.5 s.
  - Two of three real bugs were silent U32 wraparounds.

**Checker speed.** Akita measured 0.1 s for his program [42]. The site compares checking 3,200 generic instantiations: 0.38 s for Bend vs 19.2 s for Lean and 6.04 s for Rocq [41]. Nezk argues the comparison is misleading: Bend does no elaboration, unification, implicit arguments or type classes, and explicit code costs LLM tokens and context [45].

### F4. Known limitations of Bend 2 (from its own docs unless noted)

- **Numbers.** Only `Nat`, `U32` and `F32`; no U64 or F64, partly because Metal has no f64 [2][10]. F32 is "axiomatic", so nothing about floats can be proven [2]. U64 and F64 are listed as "SOON" [6].
- **Data.** Strings are linked lists. `Array<T>` is a single-owner tree that the backends store flat. A read "hands the array back" next to the element [3][4]. Shared atomic arrays are experimental [2].
- **Control.** No `if` syntax (branch with `match`), no mutual recursion, termination required, `@unsafe` escapes [2][3]. Unbalanced forks silently serialize [4].
- **IO and FFI.** A single-threaded event loop. Foreign effects need a C file plus a JS twin. Effects have "no ABI promise" across releases [3][42].
  - No DNS, TLS, HTTP or crypto in Base [6]. No stdin at launch [42].
  - Output is a whole program only. A native library target is "planned, not scheduled" [6]. "The proven core cannot be linked into a Rust application today" [42].
- **Build and tooling.**
  - Clang only, one C file per program, no separate or incremental compilation [2][6].
  - No debugger, profiler, REPL or test framework. Terse errors. Editor support is formatting only [2].
  - No Windows [2].
- **Semantic divergence between targets.** `bend file.bend` runs `main` lazily while compiled targets are strict. F32 NaN bits differ on JS [6].
- **Trust.** "The compiler (not kernel) is 99% AI-written and not yet fully audited" [2]. Taelin also warned that "the kernel drifted, so early days exploits are not impossible" [10]. Both papers disclose they were written by Claude models [4][5].
- **Ecosystem.** "We need a mathlib": Base ships without basic Nat lemmas [41][46].

### F5. Laws and proofs as guardrails for AI agents

**The workflow** [1][2][3]:
- Humans write `LAWS.bend`, which states rules as dependent types. The README calls it "`AGENTS.md` backed by **proof**".
- Agents write `PROOF.bend`.
- `bend PROOF.bend` acts as a commit gate. There are no tactics and no proof search.

**Positive evidence**
- In Akita's port, an off-by-one mutation passed all sample tests but was rejected by the proof gate [41][42].
- Astropulse found that "the proofs and laws genuinely made this conversion easy" [43].
- The repo ships graded proof tasks for agents (`cake`, `easy`, `firm`, `hard`, `hell`) [9].

**Negative evidence**
- Akita needed about 93 proof lines and about 30 check rounds for about 30 lines of guarded code. The proofs reached "a few percent" of his programs [41][42].
- Liam Powell counted 58 lines of laws and 442 lines of proof for the demo. An LLM-written SPARK version discharged the same properties automatically: "all checks proved (12 checks)". He concluded the design is "decades behind the state of the art" [44].
- HN commenters reported:
  - laws were underspecified, and an agent satisfied the letter but broke the intent
  - agents edited the laws themselves
  - about half a million tokens were spent on one small feature

  Taelin replied that "AI time is cheap, while bugs take human time" [46].

### F6. Superpositions and SupGen

**Mechanism.** Build one superposition of all candidate programs. Apply it to test inputs, so shared sub-computations run once. Then collapse to the candidates whose results match [22]. A third-party note puts it this way: the speedup "comes from structural redundancy between programs, not from smarter scheduling" [55].

**Evidence (all self-reported)**
- ADD-CARRY with 2^16 candidate bit-fills: about 36k interactions vs an estimated 262M for brute force (about 7,277x) [22]. The author notes that "the general problem is still exponential" [22].
- Superposed λ-term enumerator: 32M → 76k interactions on one problem. Wall-clock 0.992 s for a Haskell loop vs 0.0011 s superposed [23].
- "Unordered superpositions" cut a logic-programming emulation from quadratic to linear [21][24]. The caveats: the implementation was "not a faithful implementation", and the change "might break strong confluence" [24].
- Taelin claimed the system "now beats all published alternatives in efficiency and capabilities" [12].
- Engine speed: HVM3 160M interactions/s, HVM4 130M, then 190M after Claude ported HVM3's optimizations [16]. HVM5 reported about 5x HVM4 [14].
- I found no independent benchmark. The one third-party write-up has no measurements [56].

**Status**
- SupGen was cut from Bend 2's launch [13]. It is "proprietary" [46].
- BendAI promises SupGen "in future updates" [25].
- Taelin says SupGen is "meant to be a component of an actual AI architecture", not a productivity tool [20].

### F7. How agents cope with exotic semantics

Taelin's 24/7 agent experiment failed [17]. In his words, the models were "completely unable to even *grasp* SupGen" and kept trying to add `console.log` tracing to memory that is "in a 'superposition' of 1000's of candidate programs" [18]. They did best on the familiar JS compiler [17].

Bend 2's affine, no-loop, no-`if` style forced the porting agent to write "a kernel generator and a topological sorter just to keep compiling". A 3×3 median became "nine chained continuation functions" [43].

## What works / what failed and why

**Worked**
1. **Purity plus ownership makes parallelism safe and nearly free to express.** Both Bend generations achieve real multi-core scaling with no locks [4][28][33]. Bend 2 gets there without graph rewriting: affinity means a forked task owns its arguments, so "a fork moves memory and never shares it" [4]. Rust's ownership model already contains this idea (HN commenters compared Bend 2 to `rayon::join` [46][53]).
2. **Compiled frees instead of a GC.** Static ownership plus selective reference counting gives near-C sequential speed on allocation-heavy tree code [4]. This continues work on Perceus and Lean reference counting [50].
3. **One source text for CPU and GPU.** No stack, flat memory, and the C file doubles as the shader [4]. It is genuinely new for a general-purpose language with recursion and algebraic data. The payoff is limited to uniform workloads [4].
4. **Proof gate as agent contract.** It catches what tests miss [42] and gives agents a precise target [43].
5. **A small Lean-verified kernel, checked again independently (`--verdict`)** [5][8]. The repo also caps each core file's size in tokens (`bend.ts` 48k, `comp.ts` 66k), presumably so the whole file fits in an agent's context [8].

**Failed, or not shown**
1. **Interaction nets as a general runtime.** Single-thread overhead stayed at 5x to over 300x slower than native in every independent test [28][39][40]. HOC abandoned them for Bend [10]. Why: every operation allocates graph nodes and moves pointers. Numbers and arrays fit poorly (24-bit unboxed numbers, λ-encoded constructors costing 2x–5x memory [28]). And beta-optimality only pays off for higher-order sharing patterns that ordinary programs rarely contain. **Inference:** HOC's exponential-speedup examples are Church numerals and function fusion [32], not systems code.
2. **"Write recursion, get parallelism" as a mass-market surface.** "Most devs can't think recursively" [12]. Bend 2 dropped `fold`/`bend` from its guide in favor of the explicit parallel let [3].
3. **Manual term-style proofs at scale.** They are verbose, with no automation, no lemma library, and heavy token use [41][44][46]. Mature auto-active tools such as SPARK discharge comparable properties automatically [44].
4. **Backend-driven restrictions.** Metal's lack of f64 shaped the numeric types [10]. The consistency argument for `Type : Type` forces closures to be called at most once [5]. Users and agents pay daily for both [43][46].
5. **Adoption basics.** No library target, unstable FFI, thin standard library, slow whole-program builds [6][42].

## Design implications (ranked)

Confidence is the probability that the recommendation is right for this project, given the evidence above.

**1. Do not use HVM or interaction nets as the runtime or a backend.** Confidence: **high (≈90%)**.
- HOC itself left [10].
- HVM2 is legacy and HVM4/5 are pre-release or private [26][30][14].
- Every independent measurement shows large single-thread and memory overheads [39][40].
- If an interaction-net target is ever wanted for research, Vine/Ivy is an independent, Rust-influenced alternative [48], but it lacks performance evidence.

**2. Treat Bend 2 as a validation of a Rust-like core, and build its parallel story natively.** Confidence: **high for CPU (≈80%), low–medium for GPU (≈40%)**.
- Make `par` (fork-join over owned or immutably borrowed arguments) a checked primitive, with structured joins and no shared mutation.
- Unlike BendRT, use work stealing or MPL-style automatic granularity control. These handle irregular parallelism and need no "equal parts" promise from the programmer [4][49].
- Defer GPU work. If pursued, prefer an array-flattening subset in the style of Futhark [51] over Bend's whole-language device runtime, whose wins are bimodal [4].

**3. Adopt the proof-gate workflow, not Bend's proof style.** Confidence: **medium-high (≈70%)**.
- Keep laws in a human-owned spec file that the build pins. Changes to laws need explicit human approval, which addresses agents editing their own laws [46].
- Agents write implementations and proofs. The build refuses to emit artifacts when a law fails.
- Discharge proofs automatically first (SMT or auto-active verification, as in SPARK [44]). Fall back to explicit proof terms.
- Copy BendTT's live/dead split: ghost code (specs, proofs, erased arguments) is exempt from ownership rules [5]. This keeps the proof layer expressive without weakening runtime safety.
- Ship a standard lemma library from day one [41].

**4. Design for a "pure core, thin effect shell" shape.** Confidence: **medium-high (≈70%)**.
- Bend's proofs reach pure code only, and the sweet spot reported is allocators, parsers, state machines and CRDT merges [41][42].
- Ferox should make effects visible in signatures (capabilities or effect rows), so agents and tools know which functions they can prove, parallelize, memoize or search over.

**5. Do not let a backend or the logic dictate the surface language.** Confidence: **high (≈85%)**.
- Keep `if`, loops, mutable owned locals, mutual recursion, and Rust's `Fn`/`FnMut`/`FnOnce` closure kinds.
- Provide full-width integers and floats. Check arithmetic by default: silent wraparound caused two of three bugs in the Bend port [43].
- Termination checking belongs only in the proof or total fragment, not in all code. Bend makes server loops carry a fuel counter [5].

**6. Check fast with local inference, not with none.** Confidence: **medium (≈60%)**.
- Bend's 0.1 s checks show the value of instant feedback for agents [42].
- But explicitness costs tokens, which are dearer than CPU time [45]. And "zero inference" made code longer and harder for agents to write [43].
- Target sub-second incremental checks with bidirectional, local inference. Require signatures at item boundaries. Avoid global, search-heavy features.

**7. Treat superposition search as a pluggable external engine, not part of the language's semantics.** Confidence: **medium on the recommendation (≈65%); low on SupGen's eventual payoff (≈25–40%)**.
- Define a pure, total, first-order fragment with typed holes, plus a hole-filling and counterexample-search protocol.
- Implement that protocol first with conventional methods: enumerative synthesis with deduplication, e-graphs [52], and SMT.
- Revisit an HVM-style engine only if SupGen or HVM5 ships publicly with independent benchmarks.

**8. Separate compilation and a C ABI library target from the start.** Confidence: **high (≈85%)**. Bend's whole-program-only output and unstable effect ABI are among the most cited blockers to adoption [6][41][42].

**9. Copy the repo hygiene.**
- A small, human-audited, formally specified kernel, with AI-written outer layers checked independently against it [2][5][8].
- Token-size caps on core files [8].
- Graded agent eval suites [9].

Confidence: **medium (≈60%)**.

### The five options

| Option | Cost | Risk | Payoff | Verdict (confidence) |
|---|---|---|---|---|
| (a) Inherit surface ideas | Low | Low | Medium | Yes, selectively. Take purity by default, parallel let / `par`, a protected laws file, ghost erasure. Do not take `fold`/`bend` as syntax; offer them as library combinators with parallel variants (≈75%) |
| (b) Compile a pure subset to HVM | High | Very high: moving target, private engine, poor constant factors | Low | No (≈90%) |
| (c) HVM-style superposition for search, synthesis and tests | High if built in-house | High: unpublished, undebuggable [18], gains depend on redundancy among candidates | Speculative, potentially high | Research spike only, behind a stable "pure fragment + holes" interface (≈65%) |
| (d) Interoperate with Bend | Medium | Medium: no library target, no stable ABI [6] | Low now | Defer. Revisit if Bend ships its native library target (#813) [6]. A JS-module bridge is possible today [4] but not useful (≈70%) |
| (e) Ignore | None | Missing the closest peer's lessons | — | No. Track Bend 2 as the competitor to beat on proofs-for-agents (≈85%) |

## Open questions

1. Do BendRT's near-C sequential numbers hold against idiomatic optimized C or Rust, and on CUDA, in independent tests [4]?
2. Is BendTT sound *as implemented*? The TypeScript checker and translation are unverified, and the kernel "drifted" [5][10][45].
3. What can SupGen actually do against LLM sampling, enumerative synthesizers, SyGuS solvers or e-graph search on public benchmarks, and will it ever be available outside HOC's paid products [13][25][46]?
4. How many tokens does agent-written proof cost per verified line, compared with auto-active verification? Only anecdotes exist [44][46].
5. Can a type or effect system check BendRT's "equal parts" promise, or is work stealing simply better for the irregular code agents write [4][49]?
6. Can a Rust-style borrow checker reproduce Bend's share and borrow *inference* (no annotations) while keeping diagnostics agent-readable [4]?
7. Is a `Type : Type` style ghost layer worth adopting for spec flexibility, given the consistency controversy [5][45]?

## Sources

1. Bend 2 site — https://bend-lang.com
2. Bend 2 README — https://github.com/bendlang/bend
3. Bend 2 GUIDE — https://github.com/bendlang/bend/blob/main/guide/GUIDE.md
4. Taelin et al., *BendRT: A Parallel Runtime for CPUs and GPUs* (2026) — https://github.com/bendlang/bend/blob/main/paper/BendRT.pdf
5. Taelin et al., *BendTT: An Affine Dependent Type Theory* (2026) — https://github.com/bendlang/bend/blob/main/paper/BendTT.pdf
6. Bend WONTFIX — https://github.com/bendlang/bend/blob/main/WONTFIX.txt
7. Bend CHANGELOG — https://github.com/bendlang/bend/blob/main/CHANGELOG.md
8. Bend AGENTS.md — https://github.com/bendlang/bend/blob/main/AGENTS.md
9. Bend evals — https://github.com/bendlang/bend/tree/main/evals
10. Taelin, pre-launch post (17 Sep 2026) — https://x.com/VictorTaelin/status/2100374221671051472
11. Taelin, launch post — https://x.com/VictorTaelin/status/2100681226143092875
12. Taelin, HOC state overview (3 Nov 2025) — https://x.com/VictorTaelin/status/1985423273304473788
13. Taelin, features cut from Bend 2 (4 Sep 2026) — https://x.com/VictorTaelin/status/2095876946080968838
14. Taelin, HVM5 (6 Jun 2026) — https://x.com/VictorTaelin/status/2063392469195338129
15. Taelin, HVM4 with SupGen built in (26 Sep 2025) — https://x.com/VictorTaelin/status/1971591584916393984
16. Taelin, HVM3/HVM4 throughput (28 Nov 2025) — https://x.com/VictorTaelin/status/1994370663516115444
17. Taelin, 24/7 agent experiment (27 Feb 2026) — https://x.com/victortaelin/status/2027214947193679932
18. Taelin, agents failing on SupGen (27 Feb 2026) — https://x.com/VictorTaelin/status/2027371199223722265
19. Taelin, SupGen parser (11 Jan 2025) — https://x.com/VictorTaelin/status/1877944934059180497
20. Taelin, SupGen purpose (29 Jan 2025) — https://x.com/VictorTaelin/status/1884464731617919079
21. Taelin, unordered superpositions (16 Nov 2024) — https://x.com/VictorTaelin/status/1857840782662619252
22. Gist: discrete program search with SUP nodes (ADD-CARRY) — https://gist.github.com/VictorTaelin/d5c318348aaee7033eb3d18b0b0ace34
23. Gist: superposed λ-calculus enumerator — https://gist.github.com/VictorTaelin/7c4c69a1f07b5c668be613f1032e7d4e
24. Gist: truly optimal evaluation with unordered superpositions — https://gist.github.com/VictorTaelin/93c327e5b4e752b744d7798687977f8a
25. BendAI / Bender — https://bend-lang.com/bender
26. HigherOrderCO GitHub organization — https://github.com/HigherOrderCO
27. HVM2 repository — https://github.com/HigherOrderCO/HVM2
28. Taelin, *HVM2: A Parallel Evaluator for Interaction Combinators* — https://github.com/HigherOrderCO/HVM2/blob/main/paper/HVM2.pdf
29. HVM3 repository — https://github.com/HigherOrderCO/HVM3
30. HVM4 repository — https://github.com/HigherOrderCO/HVM4
31. Interaction Calculus — https://github.com/VictorTaelin/Interaction-Calculus
32. HVM1 documentation (hvm 1.0.4) — https://docs.rs/hvm/1.0.4
33. Bend 1 GUIDE — https://github.com/HigherOrderCO/Bend1/blob/main/GUIDE.md
34. Kind repository — https://github.com/HigherOrderCO/Kind
35. Lafont, *Interaction Combinators*, Information and Computation 137(1), 1997 — https://doi.org/10.1006/inco.1997.2643
36. Asperti & Mairson, *Parallel Beta Reduction Is Not Elementary Recursive* — https://scholarworks.brandeis.edu/esploro/outputs/journalArticle/Parallel-Beta-Reduction-Is-Not-Elementary/9924086529701921
37. Hacker News, Bend 1 launch thread (May 2024) — https://news.ycombinator.com/item?id=40390287
38. Lobsters, Bend 1 thread — https://lobste.rs/s/2cbh68/bend_high_level_language_runs_on_gpus_via
39. Speedfox, "Breaking Bend: Benchmarking the HVM" (3 Sep 2024) — https://blog.speedfox.co.uk/
40. Passos, Passos & Bianchini, "Benchmark da linguagem Bend em comparativo com Python e OpenMP", SSCAD 2024 — https://sol.sbc.org.br/index.php/sscad_estendido/article/view/30969
41. Akita, "New AI-Focused Language Just Released: Bend 2" (19 Sep 2026) — https://akitaonrails.com/en/2026/09/19/new-ai-language-just-released-bend-2/
42. Akita, bend-tests findings — https://github.com/akitaonrails/bend-tests/blob/main/docs/findings.md
43. Astropulse, pixel-fixer port (19 Sep 2026) — https://x.com/RealAstropulse/status/2101132976704622908
44. Liam Powell, Bend vs SPARK — https://blog.liampwll.com/posts/bend_vibe_coding/
45. Nezk, critique of Bend's checker benchmarks — https://gist.github.com/Nezk/dda0511c492cf9bd673885f0341dca0e
46. Hacker News, Bend 2 thread (Sep 2026) — https://news.ycombinator.com/item?id=49746163
47. gihyo.jp, Bend 2 announcement — https://gihyo.jp/article/2026/09/bend-2
48. Vine (interaction-net language) — https://github.com/VineLang/vine
49. Westrick, Fluet, Rainey & Acar, *Automatic Parallelism Management*, POPL 2024 — https://popl24.sigplan.org/details/POPL-2024-popl-research-papers/40/Automatic-Parallelism-Management
50. Reinking et al., *Perceus*, PLDI 2021 — https://www.microsoft.com/en-us/research/publication/perceus-garbage-free-reference-counting-with-reuse/
51. Futhark performance notes — https://futhark-lang.org/performance.html
52. Willsey et al., *egg: Fast and Extensible Equality Saturation* — https://arxiv.org/abs/2004.03082
53. rayon `join` — https://docs.rs/rayon/latest/rayon/fn.join.html
54. Digg summary of the Astropulse port — https://digg.com/ai/yt56pevk
55. mwolf.dev note on SupGen — https://mwolf.dev/general/supgen
56. Molina, HVM superpositions toy synthesis (Aug 2025) — https://dmolinad.com/projects/sym_knack/hvm_knack_circc/
