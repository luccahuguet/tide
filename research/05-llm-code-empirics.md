# 05: Empirical evidence on language properties and LLM coding success

Research date: 2026-10-19. Scope: published benchmarks, papers, and practitioner reports from 2022 to 2026, weighted toward 2024 to 2026 and toward frontier (Opus-class) models where data exists. Every factual claim carries a URL. Text marked **Inference** is my reasoning from the evidence, not a measured result.

Methodology note: the shared web-search budget ran out partway through this research. Later sources were reached by fetching known paper URLs directly. As a result, a few threads (SWE-bench-java, Aider polyglot per-language splits, any controlled study of macros) are thinner than I would like. They are listed under Open questions.

---

## Question

Which programming-language properties make LLMs and coding agents more or less successful at writing, checking, and maintaining code? Specifically, how should a new language (safe and fast like Rust, lighter and faster to compile, strongly typed, with optional proofs) be designed so that Opus-class agents can use it well, given that it starts with zero training data?

---

## Findings (cited)

### 1. The low-resource penalty is real. For a brand-new language it mostly shows up as syntax errors.

**Popularity predicts performance, with notable exceptions.** The MultiPL-E study (HumanEval and MBPP translated into 18+ languages) found that "code generation performance is correlated with language popularity." Codex scored significantly lower on medium, low, and niche languages than on high-frequency ones (p=0.006, p<0.001, p=0.002). Lua was the exception: it was the 9th-best language despite "only 0.2% of GitHub activity" ([MultiPL-E, ar5iv](https://ar5iv.labs.arxiv.org/html/2208.08227)). McEval (40 languages, human-written, ICLR 2025) reports a substantial gap between closed and open models, with open models struggling most in low-resource languages ([McEval, ICLR 2025](https://www.iclr.cc/virtual/2025/poster/29445)). A benchmark summary reports steep drops on Erlang and Tcl ([benchmarklist McEval summary](https://benchmarklist.com/benchmarks/mceval/)).

**The gap widens on low-resource languages even for frontier models, but cross-language comparisons are fragile.** AutoCodeBench (Tencent, ICLR 2026; 3,920 problems across 20 languages) reports that average Pass@1 spreads across models are small for popular languages (about 50.4 to 53.8) and widen for low-resource ones (about 45.3 to 62.0) ([AutoCodeBench](https://arxiv.org/html/2508.09101v1)). The same paper has the much-repeated "Elixir is best" result: Claude Opus 4 (reasoning) scored 80.3 on Elixir against 40.3 on Python, 47.2 on TypeScript, and 38.7 on Rust. The authors explain why this ranking should not be read as a language effect. Problem difficulty was filtered with a small model (DeepSeek-Coder-V2-Lite) that "struggles with low-resource languages," so low-resource problems were easier to keep. Fourteen of the 20 languages, including Elixir, use problems translated from a shared pool, and only six languages were manually verified ([AutoCodeBench](https://arxiv.org/html/2508.09101v1)). The blog post that popularized the Elixir result admits it is one benchmark of self-contained functions ([codemyspec blog](https://codemyspec.com/blog/why-elixir-is-the-best-language-for-llms)).

**"No-resource" languages are a different regime, and syntax is the bottleneck.** Giagnorio, Martin-Lopez and Bavota (arXiv 2606.16827, 2026) built benchmarks for Gleam and MoonBit, two recent languages with almost no training data ([no-resource paper](https://arxiv.org/abs/2606.16827)). Their results ([HTML](https://arxiv.org/html/2606.16827)):
- Zero-shot pass@1 was between 0% and 20%. GPT-4o scored 7.6% on Gleam HumanEval and 12.6% on MoonBit HumanEval, and below 1% on McEval-Hard for both. The same models scored about 59 to 89% on McEval-Hard in high-resource languages and 27 to 84% in low-resource ones.
- About two-thirds of GPT-4o's failures were syntactic, and up to 90% of o3-mini's on Gleam. In high- and low-resource languages, syntax errors were typically under 10% of failures. Java was the exception at about 30%, which the authors attribute to its "syntactic verbosity, which requires the correct placement of multiple mandatory elements."

**Frontier models do better but are still penalized.** On Cangjie (Huawei's statically typed language, with little public data), Claude Sonnet 4.5 zero-shot reached 45.2% on generation, 33.9% on translation, and 65.6% on repair. DeepSeek-V3.2 reached 3.2% and 0.0% on generation and translation ([ILA-agent / Cangjie-bench](https://arxiv.org/html/2602.06976v1)). On esoteric languages, models with documentation "can now produce some correct code" but remain "far below" their mainstream-language performance ([EsoEval, NeurIPS 2025](https://neurips.cc/virtual/2025/131647)).

**Familiarity matters, not only volume.** MoonBit has "seven times less GitHub repositories than Gleam," yet it gained more from in-context learning. The authors attribute this partly to MoonBit's Rust-like function signatures and its AI-friendly design. They also note that Lua beats R despite fewer repositories because "Lua is more similar to Python than R" ([no-resource paper](https://arxiv.org/html/2606.16827)). With five examples, o3-mini reached 39.2% on MoonBit HumanEval but only 9.7% on Gleam. The converse holds too: counterfactual variants of familiar systems, such as "ThonPy," a Python with 1-based indexing, cause performance to "substantially and consistently degrade" relative to the default ([Wu et al., counterfactual tasks](https://arxiv.org/abs/2307.02477)).

### 2. What closes the gap: examples, tools, and synthetic data

**In context: examples beat documentation, and bigger models benefit more.** For Gleam and MoonBit, "few-shot is slightly more effective than RAG," winning in 7 of 12 cases for Gleam and 8 of 12 for MoonBit. Few-shot also cut syntax errors by 15.36% relative to zero-shot ([no-resource paper](https://arxiv.org/html/2606.16827)). On R and Racket, in-context learning "always helps" and becomes more effective as models grow. Fine-tuning is best for small models and can hurt very large ones ([Giagnorio et al., ICPC 2025](https://conf.researchr.org/details/icpc-2025/icpc-2025-research/8/Enhancing-Code-Generation-for-Low-Resource-Languages-No-Silver-Bullet)). LLMs are "surprisingly" good at learning new library modules from natural-language descriptions or raw implementations alone ([Patel et al., NAACL 2024](https://arxiv.org/abs/2311.09635)). Self-scaffolding, which feeds verified model solutions back as examples, raised Pyth accuracy from 16.67% to 30.82% and Minipy from 51% to 65% ([EsoEval](https://neurips.cc/virtual/2025/131647)).

The natural-language analog is striking. Given a single grammar book in context, Gemini 1.5 Pro learned to translate English to Kalamang "at a similar level to a person who learned from the same content" ([Gemini 1.5 report](https://arxiv.org/abs/2403.05530); [MTOB](https://arxiv.org/abs/2309.16575)).

**Agentic acquisition with execution feedback is the strongest inference-time lever measured.** The ILA-agent gives the model documentation navigation (structure view, semantic search, type lookup) plus code execution and test submission. It raised Claude Sonnet 4.5 on Cangjie from 45.2% zero-shot and 59.4% single-shot RAG to 81.9% on generation, from 33.9% to 76.4% on translation, and from 65.6% to 90.6% on repair. Ablating either the execution/verification tools or semantic documentation search dropped Qwen3-Max from 73.55% to 53.55%. The authors conclude that execution-based verification is central and that documentation lookup alone gives only marginal gains over iterative RAG ([ILA-agent](https://arxiv.org/html/2602.06976v1)).

**Synthetic, test-validated translation corpora work, and plain extra training on scarce native code does not.** MultiPL-T has an LLM translate Python functions into the target language and keeps only translations that pass generated tests. For StarCoderBase-15B on MultiPL-E it moved OCaml from 6.9 to 19.9, Racket from 11.8 to 21.0, R from 10.2 to 17.3, Julia from 21.1 to 35.2, and Lua from 26.6 to 31.0. Training longer on the native low-resource data "is not only inefficient, but can actually hurt performance" ([MultiPL-T](https://arxiv.org/html/2308.09895v3)). Bridge-Coder has the model draft in a high-resource language first and then generate the target. On R, D, Racket, and Bash it improved fine-tuned DeepSeek-Coder by an average of 18.71 points on M-HumanEval and 10.81 on M-MBPP, beating direct generation in every language ([Bridge-Coder](https://arxiv.org/html/2410.18957v1)). For no-resource languages, further pre-training gave the largest gains, for example lifting Qwen2.5-Coder-32B on MoonBit McEval-Hard to 25.86%. Applying it directly to instruction-tuned models harms instruction following, which the authors fix by transferring instruction weights ([no-resource paper](https://arxiv.org/html/2606.16827)). SPEAC lets the LLM write in a familiar "parent" language and compiles that into a very low-resource target (UCLID5). It produces syntactically correct programs more often than retrieval or fine-tuning baselines without losing semantic correctness ([SPEAC](https://arxiv.org/abs/2406.03636)).

### 3. Static types: small effect on first-pass generation, large effect on catching and repairing errors

**On first-pass generation the effect is neutral to slightly positive.** In the Codex era, MultiPL-E found that "static type-checking neither helps nor hinders" (no significant effect of type annotations, p=0.33 and p=0.23). For TypeScript specifically, losing precise types in the prompt cost about 2.5% (p<0.001) ([MultiPL-E](https://ar5iv.labs.arxiv.org/html/2208.08227)).

**Most compile errors in LLM code are type errors, and a type checker in the decoding loop removes many of them.** Mündler et al. (PLDI 2025) measured TypeScript generated by open models with 2B to 34B parameters. Syntax errors were about 6% of compilation errors, so about 94% were type-check failures. Type-constrained decoding cut compilation errors by 75.3% on HumanEval and 52.1% on MBPP. It improved pass@1 by a relative 3.5% for synthesis, 5.0% for translation, and 37.0% for repair of non-compiling code, at a median runtime cost of 39 to 52% in an unoptimized Python prototype ([Mündler et al., PDF](https://arxiv.org/pdf/2504.09246v2.pdf); [PLDI page](https://pldi25.sigplan.org/details/pldi-2025-papers/25/Type-Aware-Constraining-for-LLM-Code-Generation)). Grammar-only constraining (SynCode) removes 96.07% of syntax errors in Python and Go ([SynCode](https://arxiv.org/abs/2403.01632)). For mainstream languages and capable models this matters less, because syntax is rarely the failure. For a new language it matters more, because syntax is the dominant failure (Section 1).

**TypeScript tends to beat JavaScript, though not uniformly.** Claude Opus 4 scored 47.2 on TypeScript and 38.6 on JavaScript in AutoCodeBench ([AutoCodeBench](https://arxiv.org/html/2508.09101v1)). In Multi-SWE-bench the best TypeScript result was 11.61% against 5.06% for JavaScript ([Multi-SWE-bench](https://arxiv.org/html/2504.02605)). On SWE-PolyBench the result is mixed: TypeScript is ahead for Aider (13.0 vs 12.6) and SWE-agent (10.2 vs 6.5) and behind for Agentless (4.7 vs 7.2) ([SWE-PolyBench](https://arxiv.org/html/2504.08703v3)). GitHub reports that TypeScript became its most-used language in August 2025, with "correlative evidence" that AI-assisted development contributed ([GitHub blog](https://github.blog/ai-and-ml/github-copilot/why-ai-is-pushing-developers-toward-typed-languages/); [Octoverse 2025](https://github.blog/news-insights/octoverse/octoverse-a-new-developer-joins-github-every-second-as-ai-leads-typescript-to-1/)).

**Agents game type systems when they can.** In TypeScript projects, AI agents are "9x more prone to use the 'any' keyword" than humans and lean more on constructs that bypass type checking ([Mining Type Constructs](https://arxiv.org/abs/2602.17955)). An agentic JS-to-TS migration with Claude Sonnet 4.5 resolved all 633 type errors in about 20 minutes but used 383 suppressions plus 26 avoidable ones ([AgenticTyper](https://arxiv.org/html/2602.21251v1)).

### 4. Rust: strictness costs on the first attempt and pays off in the loop

**First-pass generation produces many compile errors.** One study collected 86,726 erroneous samples from seven small and mid-sized models (Qwen2.5 7 to 14B, DeepSeek-Coder-v2, GPT-4.1-mini, Gemini-2.0-Flash, gpt-oss-120b) on 1,651 CodeNet problems. Rust produced 23,416 compile errors against 6,996 for Java, 9,953 for C++, and 12,304 for C. Using the implied per-language denominator, that is about 23.6% of Rust samples failing to compile versus about 7.1% for Java; the denominator is my inference. Within Rust compile errors, incompatible types were 43.4%, missing imports 20.5%, ownership and lifetime 16.7%, and trait or type bounds 6.1%. Debug-mode overflow checks produced 43.9% of Rust's runtime errors, turning silent bugs into visible panics ([Unreliable in Practice](https://arxiv.org/html/2608.00661)).

**C-to-Rust translation: single-shot is weak and repair loops are strong.**
- In CRUST-Bench (100 C repositories with hand-written safe-Rust interfaces), OpenAI o1 solved 15 tasks single-shot ([CRUST-Bench](https://arxiv.org/abs/2504.15254)).
- The agentic ORBIT reached 100% compilation and 91.7% test success on 24 CRUST-Bench programs (most over 1,000 LOC), with near-zero `unsafe` ([ORBIT](https://arxiv.org/abs/2604.12048)).
- SafeTrans: iterative repair raised GPT-4o's end-to-end translation success from 54% to 80%, and compile-repair rates reached 93.5% for GPT-4o. E0277 (trait not implemented) and E0308 (mismatched types) were the most common errors, more than 18% for most models and up to 30% for some. The authors report that "merely providing compiler [messages]" was insufficient for persistent errors. Few-shot "guided repair" with per-error-code instructions and example fixes resolved borrow-checker violations at 74.2% and trait failures at 58.7% on average, and simpler errors such as E0384 and E0425 at above 80% ([SafeTrans](https://arxiv.org/pdf/2505.10708)).
- Earlier, the best of GPT-4, Claude 2/3, Gemini Pro, and Mixtral translated 47% of real-world benchmarks to I/O-equivalent Rust ([FLOURINE](https://arxiv.org/abs/2405.11514)). Adding repository context dropped DeepSeek-R1 from 73.7% to 51.5% on Rust translation ([RustRepoTrans](https://arxiv.org/abs/2411.13990)).

**Repairing Rust compile errors works well with frontier models, and localization is decisive.** RustAssistant (ICSE 2025) with GPT-4 fixed 92.6% of micro-benchmarks covering 270 error codes, 72 to 74% of Stack Overflow programs, and 73.6% of real commits from the top-100 crates. Prompt structure mattered enormously. On GPT-3.5, the fix rate went from 10.74% with a changelog-only format to 24.07% with line prefixes, 58.15% with localization, and 73.70% with a fix description ([RustAssistant, ar5iv](https://ar5iv.labs.arxiv.org/html/2308.05177); [ICSE 2025](https://conf.researchr.org/details/icse-2025/icse-2025-research-track/9/RustAssistant-Using-LLMs-to-Fix-Compilation-Errors-in-Rust-Code)).

**At repository scale with frontier agents, Rust is among the best languages.** On SWE-bench Multilingual (Claude 3.7 Sonnet, SWE-agent), Rust had the highest resolution rate at 58.1% (25 of 43), ahead of Java at 53.5%, JS/TS at 34.9%, Go at 31.0%, and C/C++ at 28.6%. The authors found that gold-patch size did not clearly correlate with resolution rate ([SWE-bench Multilingual](https://www.swebench.com/multilingual.html)). On Claw-SWE-Bench (2026; 350 instances, 8 languages), Rust was the best language for 7 of 11 models: GPT 5.5 scored 93.0% on Rust, and Claude Opus 4.7 scored 88.4% on Rust against its own 82.0% on Python and 81.4% on JS/TS. Go was the lowest-scoring language for all 11 models ([Claw-SWE-Bench](https://arxiv.org/html/2606.12344v2)). The older Multi-SWE-bench (2025 models) found the opposite ordering: Python 52.2%, Java 23.4%, Rust 15.9%, TypeScript 11.6%, JavaScript 5.1%. Its authors note that the agent scaffolds were "initially optimized for Python" ([Multi-SWE-bench](https://arxiv.org/html/2504.02605)).

**Inference:** Rust's strict checker raises the first-attempt failure rate, especially for weaker models. Frontier agents iterating against a strict compiler and a test suite turn that strictness into correctness: errors become explicit, localized, and repairable instead of latent. A new language can aim for Rust's "if it compiles, it probably works" property while reducing the error classes that resist repair (trait resolution, type inference failures, lifetimes).

### 5. Proofs: SMT automation and a unified spec/exec type system help

On 12,504 "vericoding" tasks (generate code plus proof from a formal spec), the union of models succeeded on 82.2% in Dafny, 44.2% in Verus/Rust, and 26.8% in Lean. Claude Opus 4.1 scored 67.5%, 24.6%, and 10.7%; GPT-5 scored 66.1%, 30.9%, and 17.9%. Pure Dafny verification success rose from 68% to 96% in one year, and adding natural-language descriptions "does not significantly improve performance." The authors attribute Verus's difficulty to its separation of ghost (spec) types from executable Rust types, which forces handling of overflow and type mapping, and to Verus being "newer, lower-resource." They attribute Lean's to training on mathematics rather than code verification and to limited proof-state feedback in their scaffold ([Vericoding](https://arxiv.org/html/2509.22908)).

### 6. Feedback loops: the quality of feedback is the bottleneck

Self-debugging improved accuracy by up to 12% when unit tests were available and matched baselines that sampled more than 10x as many programs ([Self-Debug](https://arxiv.org/abs/2304.05128)). Olausson et al. (ICLR 2024) found self-repair gains "often modest" once cost is counted. They identified the bottleneck as the model's ability to produce feedback on its own code: better feedback from a stronger model or from humans gave substantially larger gains ([Is Self-Repair a Silver Bullet?](https://arxiv.org/abs/2306.09896)). Taken together with RustAssistant's localization ablation, SafeTrans's per-error-code guidance, and ILA-agent's verification ablation, the evidence says the *content and precision* of the feedback (where, why, and how to fix) drives repair success, not merely its presence. With vanilla decoding, small models repaired only 33.5% and 25.8% of non-compiling TypeScript programs, rising to 56.4% and 58.4% with type constraints ([Mündler et al.](https://arxiv.org/pdf/2504.09246v2.pdf)).

### 7. Verbosity, token efficiency, and formatting

- **Tokens per language.** An informal RosettaCode comparison with the GPT-4 tokenizer across 19 languages found Clojure about 2.6x more token-efficient than C. Dynamic languages were generally leaner, and Haskell and F# were "barely less efficient" thanks to type inference. JavaScript was the most verbose dynamic language. Unusual glyphs (APL) cost extra tokens, while ASCII J was the most compact ([Alderson](https://martinalderson.com/posts/which-programming-languages-are-most-token-efficient/)). This is not peer-reviewed.
- **Formatting.** Stripping indentation and newlines cut input tokens by 24.5% on average with no measurable performance change across 10 LLMs and Java, Python, C++, and C# ([Hidden Cost of Readability](https://arxiv.org/abs/2508.13666)).
- **Verbosity and errors.** Java's mandatory boilerplate drives its unusually high syntax-error share (about 30%) ([no-resource paper](https://arxiv.org/html/2606.16827)). Small prompt-surface choices matter: naming Perl arguments raised Codex by about 8% ([MultiPL-E](https://ar5iv.labs.arxiv.org/html/2208.08227)).

### 8. Locality, hidden control flow, and "magic"

**Measured (correlational).** Resolution rates fall as patches span more files and get longer, and "repositories with lower entropy typically achieve higher resolved rates," so "code simplicity and consistency play a crucial role" ([Multi-SWE-bench](https://arxiv.org/html/2504.02605)). On SWE-PolyBench, Aider dropped from 17.7% on single-file tasks to 7.9% on tasks touching five or more files. Mixed class-and-function changes were hardest, and more informative issues (locations, hints) raised pass rates ([SWE-PolyBench](https://arxiv.org/html/2504.08703v3)). Long-context reliability degrades with input length and distractors across 18 models: focused prompts of about 300 tokens beat full ones of about 113k ([Chroma, Context Rot](https://www.trychroma.com/research/context-rot)). Studies of exception handling find failure modes such as "insensitive detection of fragile code" and "distorted handling solution" ([Seeker](https://arxiv.org/abs/2412.11713)).

**Practitioner reports (not controlled).** Armin Ronacher reports that agents "struggle with Python's magic," such as pytest fixture injection. He finds Go's explicit context passing, structural interfaces, fast incremental tests, and low ecosystem churn agent-friendly ([Agentic coding recommendations](https://lucumr.pocoo.org/2025/6/12/agentic-coding/)). His language-for-agents proposals:
- explicit types, so code is readable without an LSP
- explicit effect annotations
- results rather than exceptions
- greppable, package-qualified names
- no re-exports or barrel files
- few macros
- syntax that minimizes diff churn
- one command that lints, compiles, and reports pass/fail with mechanical auto-fixes

([A language for agents](https://lucumr.pocoo.org/2026/2/9/a-language-for-agents/))

MoonBit's designers claim a flat, nesting-light structure with mandatory top-level signatures enables near-linear generation and real-time "semantic sampling." They report a "significant improvement in compilation rates" at about a 3% speed cost, but publish no detailed numbers ([MoonBit blog](https://www.moonbitlang.com/blog/moonbit-ai)). I found no controlled study isolating the effect of macros or metaprogramming on LLM accuracy.

### 9. Hallucinated APIs, standard-library size, and churn

- Across 16 code LLMs and 576,000 samples in Python and JavaScript, hallucinated-package rates averaged at least 5.2% for commercial models and 21.7% for open models, producing 205,474 unique fake package names ([Spracklen et al., USENIX Security 2025](https://arxiv.org/abs/2406.10279)).
- GPT-4o produced only 38.58% valid invocations of low-frequency cloud APIs. Documentation retrieval raised that to 47.94%, but a sub-optimal retriever cut high-frequency API accuracy by 39.02 points. Retrieving only when needed gave +8.20% ([CloudAPIBench](https://arxiv.org/abs/2407.09726)).
- On Rust API evolution, success was 56.1% for APIs released before the model's cutoff and 32.5% after. Stabilized APIs scored 65.8% and behavioral changes (same signature, new semantics) 38.0%. RAG added 13.5% ([RustEvo²](https://arxiv.org/abs/2503.16922)).

---

## Evidence table

Direction is the effect on LLM or agent correctness. Strength: **Strong** means multiple independent, controlled or large-sample studies; **Moderate** means one or two controlled studies or consistent correlational data; **Weak** means practitioner reports, vendor claims, or a single informal measurement.

| Property | Direction | Strength | Key evidence |
|---|---|---|---|
| Training-data volume / popularity | + (penalty when absent; collapses to 0–20% for no-resource languages) | Strong | [MultiPL-E](https://ar5iv.labs.arxiv.org/html/2208.08227), [McEval](https://www.iclr.cc/virtual/2025/poster/29445), [no-resource](https://arxiv.org/html/2606.16827), [AutoCodeBench](https://arxiv.org/html/2508.09101v1) |
| Syntax and semantics close to a high-resource language | + (MoonBit vs Gleam; Lua vs R) | Moderate | [no-resource](https://arxiv.org/html/2606.16827), [MultiPL-E](https://ar5iv.labs.arxiv.org/html/2208.08227) |
| Familiar syntax with changed semantics (counterfactual defaults) | − (substantial degradation) | Moderate | [Wu et al.](https://arxiv.org/abs/2307.02477) |
| Static types, first-pass generation | 0 to slight + | Moderate | [MultiPL-E](https://ar5iv.labs.arxiv.org/html/2208.08227), [AutoCodeBench](https://arxiv.org/html/2508.09101v1) |
| Static types used as a checker (loop or constrained decoding) | ++ (−52 to −75% compile errors; +37% repair) | Strong (open models); Moderate (frontier) | [Mündler et al.](https://arxiv.org/pdf/2504.09246v2.pdf), [GitHub](https://github.blog/ai-and-ml/github-copilot/why-ai-is-pushing-developers-toward-typed-languages/) |
| Strict ownership/borrow checking | − first pass (Rust about 3x Java's compile errors); + after loop (Rust top on agentic SWE benchmarks) | Moderate | [Unreliable in Practice](https://arxiv.org/html/2608.00661), [SWE-bench Multilingual](https://www.swebench.com/multilingual.html), [Claw-SWE-Bench](https://arxiv.org/html/2606.12344v2) |
| Complex trait resolution and type inference | − (E0277/E0308 dominate and resist repair: 55–59%) | Moderate | [SafeTrans](https://arxiv.org/pdf/2505.10708) |
| Precise error localization in diagnostics | ++ (10.7% → 58–74% fix rate) | Moderate | [RustAssistant](https://ar5iv.labs.arxiv.org/html/2308.05177) |
| Per-error-code explanations with example fixes | + (borrow 74%, trait 59%, simple errors >80%) | Moderate | [SafeTrans](https://arxiv.org/pdf/2505.10708) |
| Fast execution and verification tools for agents | ++ (Cangjie 45% → 82% for Sonnet 4.5) | Moderate | [ILA-agent](https://arxiv.org/html/2602.06976v1), [Self-Debug](https://arxiv.org/abs/2304.05128) |
| Few-shot examples in context | + (beat docs RAG; −15% syntax errors) | Moderate | [no-resource](https://arxiv.org/html/2606.16827), [EsoEval](https://neurips.cc/virtual/2025/131647) |
| Documentation retrieval | + for rare/new APIs, − for common ones if retrieval is poor | Moderate | [CloudAPIBench](https://arxiv.org/abs/2407.09726), [RustEvo²](https://arxiv.org/abs/2503.16922) |
| Synthetic translated, test-validated corpus | ++ (OCaml 2.9x, Racket 1.8x pass@1) | Strong (open models) | [MultiPL-T](https://arxiv.org/html/2308.09895v3), [Bridge-Coder](https://arxiv.org/html/2410.18957v1) |
| Large or churning API surface | − (56% → 33% post-cutoff; 21.7% fake packages, open models) | Moderate | [RustEvo²](https://arxiv.org/abs/2503.16922), [Spracklen](https://arxiv.org/abs/2406.10279) |
| Locality (few files per change, low entropy) | + | Moderate (correlational) | [Multi-SWE-bench](https://arxiv.org/html/2504.02605), [SWE-PolyBench](https://arxiv.org/html/2504.08703v3) |
| Short, focused context | + | Moderate | [Context Rot](https://www.trychroma.com/research/context-rot) |
| Verbosity / mandatory boilerplate | − (Java syntax-error share about 30%; token cost) | Weak–Moderate | [no-resource](https://arxiv.org/html/2606.16827), [Alderson](https://martinalderson.com/posts/which-programming-languages-are-most-token-efficient/) |
| Formatting / whitespace | 0 on correctness; formatting costs about 25% of input tokens | Moderate (narrow scope) | [Hidden Cost of Readability](https://arxiv.org/abs/2508.13666) |
| Hidden control flow, injection "magic", macros | − | Weak | [Ronacher 2025](https://lucumr.pocoo.org/2025/6/12/agentic-coding/), [Ronacher 2026](https://lucumr.pocoo.org/2026/2/9/a-language-for-agents/) |
| Exceptions vs typed results | Results likely better | Weak | [Ronacher 2026](https://lucumr.pocoo.org/2026/2/9/a-language-for-agents/), [Seeker](https://arxiv.org/abs/2412.11713) |
| Type escape hatches (`any`, casts, suppressions) | − (agents use `any` 9x more than humans) | Moderate | [Mining Type Constructs](https://arxiv.org/abs/2602.17955), [AgenticTyper](https://arxiv.org/html/2602.21251v1) |
| SMT-automated proofs with unified spec/exec types | + (Dafny 82% vs Verus 44% vs Lean 27%) | Moderate (confounded by data volume) | [Vericoding](https://arxiv.org/html/2509.22908) |

---

## How a new language avoids the low-resource penalty

The evidence suggests the penalty has two parts. The first is **surface unfamiliarity**: syntax errors dominate failures for unseen languages. The second is **API and idiom ignorance**: hallucinated functions and wrong idioms. The first is largely fixable by design. The second is fixable by tooling and data. In order of evidence strength and cost-effectiveness:

1. **Be a dialect, not a new language, at the surface.** Use Rust and TypeScript spellings for every concept those languages share: `fn`, `let`, `mut`, `struct`, `enum`, `match`, `impl`, `->`, `::`/`.`, `Option`/`Result`, `?`, generics `<T>`, and `//` comments. Evidence: MoonBit gained more from in-context learning than Gleam despite 7x fewer repositories, partly because its signatures look like Rust's. Lua outperforms its data share because it resembles Python. Changing the semantics of a familiar form (ThonPy's 1-based indexing) is worse than having no familiar form, so where Ferox's semantics differ from Rust's, it should use a *different* keyword rather than reuse a Rust keyword with new meaning. **Inference:** the target is that an Opus-class model writing "Rust as it remembers it" produces Ferox that almost compiles, and the compiler explains the remainder.

2. **Make the compiler the teacher: execution plus precise, fix-carrying diagnostics.** The largest inference-time effect measured (Sonnet 4.5 on Cangjie: 45% to 82%) came from giving the agent execution, verification, and navigable documentation, with execution the most important part. Repair success depends on localization (RustAssistant: 10.7% to 73.7%) and on per-error explanations with example fixes (SafeTrans: borrow errors 74%, trait errors 59%). Ferox's compiler should therefore emit:
   - structured diagnostics with exact spans and stable error codes
   - machine-applicable fix suggestions
   - an `explain` page per code, with a minimal failing example and a correct fix

3. **Ship a compiler-served, small, example-heavy reference.** Few-shot examples beat documentation retrieval for syntax. Documentation retrieval helps rare APIs but can hurt common ones when retrieval is noisy, and long contexts degrade. So the primary artifact should be a compact "language card" of canonical idioms (written as code, not prose) that fits comfortably in context, plus on-demand symbol lookup (`doc <path>`, `examples <topic>`) instead of one huge spec dump. Gemini's Kalamang result shows a model can learn a whole language from one book in context. The card should be much smaller than a book.

4. **Keep the API surface small, stable, and greppable.** Hallucination concentrates on low-frequency and post-cutoff APIs (38.6% valid for rare cloud APIs; 56% → 33% across the Rust cutoff). A small standard library with frozen names, no aliases or re-exports, and one canonical path per item reduces the hypothesis space. A lookup tool can then answer "does X exist?" exactly.

5. **Enable constrained decoding and early checking.** For a new language, syntax errors are the majority failure mode, and grammar-constrained decoding removes about 96% of them. Type-constrained decoding removes more than half of all compile errors. **Inference:** a context-free, LL(1)-friendly grammar and local, left-to-right type checking (mandatory signatures, no whole-program inference) make both possible for open models and for the project's own tooling. Frontier API models will rely on the fast check loop instead.

6. **Build bridges: transpile in from familiar code and train on validated synthetic data.** SPEAC and Bridge-Coder show that letting the model think in a familiar language and translating mechanically or with validation works. A `ferox import` tool for a Rust subset, and a MultiPL-T-style pipeline (translate Python, Rust, or TS benchmark solutions to Ferox and keep only test-passing ones), would create a Ferox corpus for few-shot banks, open-model fine-tuning, and eventually public pre-training data. The no-resource paper shows further pre-training is the strongest lever when weights are accessible.

7. **Publish code and benchmarks early.** RustEvo² shows that knowledge arrives with retraining (56% vs 33% across the cutoff). **Inference:** every permissively licensed Ferox repository, the test-validated synthetic corpus, and a public MultiPL-E-style "Ferox-HumanEval" increase the chance that future frontier models will have seen Ferox.

Expected magnitude (**Inference**): Gleam, MoonBit, and Cangjie suggest a frontier model will start at roughly 20 to 50% of its mainstream performance on a truly unseen language. With a Rust-like surface, an in-context card, and an execution-and-diagnostics agent loop, the gap can plausibly shrink to tens of points or less, as with Cangjie at 82%. Ferox's own benchmark should measure this rather than assume it.

---

## Design implications (ranked, concrete, with confidence)

1. **Rust/TS-compatible surface syntax, with new keywords only for new semantics.** No novel glyphs or operators for familiar operations. Never reuse a Rust keyword with different meaning. Mandatory braces and semicolons as in Rust. *Confidence: high* (no-resource syntax-error data; MoonBit vs Gleam; counterfactual degradation).

2. **One fast `ferox check` command emitting structured JSON diagnostics.** Each diagnostic carries a primary span, related spans, a stable error code, a one-line cause, and machine-applicable fix-its. Every error code has an `explain` page with a failing example and a fixed example, tested in CI. Measure "agent fix rate per error code" as a release metric. *Confidence: high* (RustAssistant localization ablation; SafeTrans guided repair; Olausson feedback bottleneck; ILA-agent).

3. **Explicit types at every item boundary, local inference only inside bodies, and a decidable, prefix-checkable type system.** This makes the 94% of compile errors that are type errors catchable early, keeps code legible without an LSP, and allows type-constrained decoding. *Confidence: moderate-high* (Mündler et al.; MultiPL-E TypeScript precise-types effect; Ronacher; MoonBit's top-level signatures).

4. **A simpler ownership and trait story than Rust.** Rust's hardest-to-repair LLM errors are type mismatches and trait bounds, with ownership and lifetimes third; trait failures resolve only about 59% even with guidance. Prefer:
   - no user-written lifetime annotations (second-class or region-inferred references)
   - no orphan-rule or blanket-impl puzzles
   - no implicit trait-driven conversions
   - coherent, explicit dispatch

   Keep Rust's property that strictness converts latent bugs into compile errors, which is why Rust leads frontier agentic SWE benchmarks. *Confidence: moderate* (SafeTrans; Unreliable in Practice; SWE-bench Multilingual and Claw-SWE-Bench, both with small per-language samples).

5. **Locality and greppability by construction.** One declaration path per item, no re-exports or glob imports, qualified imports, no user-defined syntax or procedural macros (prefer comptime-style functions or generics), explicit effects or capabilities in signatures, and `Result` rather than exceptions. *Confidence: moderate-low* (correlational SWE data on cross-file edits and repository entropy; practitioner reports; no controlled macro study).

6. **No silent escape hatches.** `unsafe`, unchecked casts, and "trust me" annotations must be explicit, greppable, counted by the compiler, and deniable by project policy, because agents use `any`-style escapes 9x more than humans. *Confidence: moderate.*

7. **Small, frozen standard library and a compiler-served documentation index.** Provide `ferox doc`, `ferox examples`, and a language card that fits in a small fraction of the context window. Evolve the language through editions rather than renames. *Confidence: moderate* (CloudAPIBench; RustEvo²; package hallucination; few-shot > RAG; context rot).

8. **Optional proofs that are SMT-automated, with math types shared between spec and code.** A Dafny-style experience (requires/ensures/invariants discharged automatically, with counterexamples as diagnostics) rather than tactic proofs. Avoid a Verus-style ghost/exec type split where possible. *Confidence: moderate* (Vericoding 82/44/27, confounded by data volume).

9. **Canonical formatter and token-lean but ASCII, readable syntax.** One formatting, no significant whitespace, trailing-comma rules that minimize diff churn, and keywords that are common English words (tokenizer-friendly). *Confidence: low-moderate* (formatting is neutral to correctness; token studies informal).

10. **Bridges and a data flywheel from day one.** Build a Rust-subset importer, a MultiPL-T-style validated translation corpus, a public Ferox-HumanEval/MBPP, and an error-category telemetry harness that runs Opus-class agents on Ferox each release. *Confidence: moderate* (MultiPL-T, Bridge-Coder, SPEAC, no-resource pre-training).

11. **Agent-loop speed as a language requirement.** Incremental check in well under a second, deterministic tests, and a built-in test runner with one obvious invocation. *Confidence: moderate-low* (ILA-agent shows execution matters; latency effects reported only by practitioners).

---

## Open questions

- **How large is the zero-shot penalty for Opus-class models specifically?** The best frontier data point is one language (Cangjie, Sonnet 4.5). Ferox should measure itself with a translated benchmark under three conditions: zero-shot, with the language card, and with the full agent loop. It should also classify failures (syntax, type, ownership, API hallucination).
- **Why does Rust lead agentic SWE benchmarks but trail on single-shot generation?** Is it the compiler, Rust's test culture, or task selection (only 43 Rust tasks in SWE-bench Multilingual)? Multi-SWE-bench's older results disagree.
- **No controlled evidence exists** on macros, significant whitespace, explicit-vs-inferred local types, or effect annotations for LLM correctness. All current claims are practitioner reports.
- **Token efficiency vs correctness trade-off.** Do terser languages raise error rates for agents, or only cut cost? No study isolates this.
- **Constrained decoding for closed frontier models.** Its benefit is shown on 2B–34B open models. Whether hosted frontier APIs allow arbitrary grammar or type constraints is outside this report's verified evidence.
- **Size and shape of the in-context card.** How many tokens of examples versus reference are optimal before context-rot effects dominate? Is there a ceiling?
- **Proof layer.** Will Verus-style proofs close the gap with Dafny as data grows (Dafny went from 68% to 96% in one year)? Or is the ghost/exec split a lasting tax?
- **Benchmark validity.** Cross-language comparisons (AutoCodeBench's Elixir, Multi-SWE-bench vs SWE-bench Multilingual) are confounded by task selection and scaffold bias. Ferox's own evaluation should hold tasks fixed across languages, as MultiPL-E does.

---

## Sources

- MultiPL-E (Cassano et al.): https://ar5iv.labs.arxiv.org/html/2208.08227 ; https://arxiv.org/abs/2208.08227v4
- McEval (ICLR 2025): https://www.iclr.cc/virtual/2025/poster/29445 ; summary https://benchmarklist.com/benchmarks/mceval/
- AutoCodeBench: https://arxiv.org/html/2508.09101v1 ; blog https://codemyspec.com/blog/why-elixir-is-the-best-language-for-llms
- No-resource languages (Gleam, MoonBit): https://arxiv.org/abs/2606.16827 ; https://arxiv.org/html/2606.16827
- Low-resource "No Silver Bullet" (ICPC 2025): https://conf.researchr.org/details/icpc-2025/icpc-2025-research/8/Enhancing-Code-Generation-for-Low-Resource-Languages-No-Silver-Bullet
- ILA-agent / Cangjie-bench: https://arxiv.org/html/2602.06976v1
- EsoEval (NeurIPS 2025): https://neurips.cc/virtual/2025/131647
- Counterfactual tasks (Wu et al.): https://arxiv.org/abs/2307.02477
- In-context learning of libraries (Patel et al.): https://arxiv.org/abs/2311.09635
- Gemini 1.5 report: https://arxiv.org/abs/2403.05530 ; MTOB: https://arxiv.org/abs/2309.16575
- MultiPL-T: https://arxiv.org/html/2308.09895v3
- Bridge-Coder: https://arxiv.org/html/2410.18957v1
- SPEAC: https://arxiv.org/abs/2406.03636
- Type-constrained decoding (Mündler et al., PLDI 2025): https://arxiv.org/pdf/2504.09246v2.pdf ; https://pldi25.sigplan.org/details/pldi-2025-papers/25/Type-Aware-Constraining-for-LLM-Code-Generation
- SynCode: https://arxiv.org/abs/2403.01632
- GitHub on typed languages: https://github.blog/ai-and-ml/github-copilot/why-ai-is-pushing-developers-toward-typed-languages/ ; Octoverse 2025: https://github.blog/news-insights/octoverse/octoverse-a-new-developer-joins-github-every-second-as-ai-leads-typescript-to-1/
- Mining Type Constructs in AI-generated code: https://arxiv.org/abs/2602.17955
- AgenticTyper: https://arxiv.org/html/2602.21251v1
- Unreliable in Practice (errors in LLM code): https://arxiv.org/html/2608.00661
- SafeTrans: https://arxiv.org/pdf/2505.10708
- CRUST-Bench: https://arxiv.org/abs/2504.15254
- ORBIT: https://arxiv.org/abs/2604.12048
- FLOURINE (translating real-world code to Rust): https://arxiv.org/abs/2405.11514
- RustRepoTrans: https://arxiv.org/abs/2411.13990
- RustAssistant: https://ar5iv.labs.arxiv.org/html/2308.05177 ; https://conf.researchr.org/details/icse-2025/icse-2025-research-track/9/RustAssistant-Using-LLMs-to-Fix-Compilation-Errors-in-Rust-Code
- RustEvo²: https://arxiv.org/abs/2503.16922
- SWE-bench Multilingual: https://www.swebench.com/multilingual.html
- Claw-SWE-Bench: https://arxiv.org/html/2606.12344v2
- Multi-SWE-bench: https://arxiv.org/html/2504.02605
- SWE-PolyBench: https://arxiv.org/html/2504.08703v3
- Vericoding benchmark: https://arxiv.org/html/2509.22908
- Self-Debug: https://arxiv.org/abs/2304.05128
- Is Self-Repair a Silver Bullet?: https://arxiv.org/abs/2306.09896
- Token efficiency by language (Alderson): https://martinalderson.com/posts/which-programming-languages-are-most-token-efficient/
- Hidden Cost of Readability (formatting tokens): https://arxiv.org/abs/2508.13666
- Context Rot (Chroma): https://www.trychroma.com/research/context-rot
- Seeker (exception handling): https://arxiv.org/abs/2412.11713
- Ronacher, Agentic coding recommendations: https://lucumr.pocoo.org/2025/6/12/agentic-coding/
- Ronacher, A language for agents: https://lucumr.pocoo.org/2026/2/9/a-language-for-agents/
- MoonBit AI design: https://www.moonbitlang.com/blog/moonbit-ai
- Package hallucinations (Spracklen et al.): https://arxiv.org/abs/2406.10279
- CloudAPIBench (API hallucination and documentation): https://arxiv.org/abs/2407.09726
