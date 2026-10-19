# 06: Agent-native tooling, and the toolchain contract for agents

Research date: 2026-10-19. Scope: languages and toolchains built for AI coding
agents, and the compiler features that help agents. This report does not look at
the repository's existing spec.

Method note: every factual claim below links to a source. Statements marked
**Inference** are my own reasoning. The web-search budget for this session ran
out partway through. After that I fetched known primary URLs directly, so some
areas got less coverage: Mojo, Hacker News discussion threads, and agent tooling
for record/replay debugging. Placeholder command names use `fx`.

---

## Question

What should a new language's toolchain offer AI coding agents? The target
language is safe and fast, compiles quickly, has strong types and optional
proofs, and is meant for Opus-class agents to write, check, and maintain.
Specifically:

1. Which agent-native features have evidence that they raise agent success, and
   which are still speculation?
2. What concrete "agent contract" should the toolchain expose? That covers
   commands, JSON outputs, typed holes, fix plans, protection against stale
   state, a semantic query API, and docs serving.
3. Should code be text-first, graph-first, or content-addressed?

---

## Findings (cited)

### 1. The "agent-first language" wave: many languages, almost no outcome data

- **Catalog size.** agentlanguages.dev listed 42 languages "designed for AI agents
  to write" as of 12 Oct 2026. Most entries claim no measured agent-success
  results. The stated evidence is usually a design rationale, token counts, or
  "none stated" ([agentlanguages.dev](https://agentlanguages.dev/)). Examples:
  X07 (JSON ASTs edited with JSON Patch), Tacit (AST-as-source,
  content-addressed, typed holes), SEMAPRAX (graph-native, revision-bound
  semantic patches), ilo and NERD (token minimization), and Aver, Intent, Vow
  and Thermite (contracts checked by SMT, model checking, or Lean). All are
  listed in the same catalog.
- **Zero (vercel-labs/zerolang).** Zero is a graph-first systems language. The
  binary `zero.graph` store is the compiler input, and `.0` text files are
  "human-readable projections, not the normal agent authoring surface." Agents
  use `zero query`, `zero patch`, `zero check/test/run`, and version-matched
  guidance from `zero skills`. Stale graph hashes and unexpected field values
  "fail before the store is written"
  ([GitHub README](https://github.com/vercel-labs/zerolang)). Patches can carry
  `--expect-graph-hash` and per-field `expect=` values
  ([graph architecture](https://zerolang.ai/concepts/graph-architecture)).
  Diagnostics have stable codes (`NAM003`, `RGP003`, `GRF013`). `zero fix --plan
  --json` returns a repair plan without applying it. The docs say agents should
  *not* default to JSON for every command
  ([diagnostics](https://zerolang.ai/diagnostics)).
  - **Current state.** The latest release is still v0.3.4 (13 Jun), which "makes
    graph patching the main agent edit loop"
    ([releases](https://github.com/vercel-labs/zerolang/releases)). Commit
    activity fell off after June 2026. The last commits are a std.http fix on
    Aug 31 and "Add Labs status badges" on Sep 16
    ([commits](https://github.com/vercel-labs/zerolang/commits/main)). The README
    badge reads "LABS-EXPERIMENT"
    ([README](https://raw.githubusercontent.com/vercel-labs/zerolang/main/README.md)).
    The repo has about 5.4k stars
    ([GitHub](https://github.com/vercel-labs/zerolang)).
  - **Source-model churn.** Within three weeks Zero switched source models three
    times. v0.1.4 (24 May) adopted "row syntax." v0.2.0 (28 May) made canonical
    `.0` text the native source. v0.3.0 (8 Jun) made graph-first the default. The
    next releases then spent effort on keeping graph and source consistent:
    "stale, diverged, and ambiguous reconcile states now produce deterministic
    diagnostics," plus "patch-specific tips when graph and source are out of
    sync" ([releases](https://github.com/vercel-labs/zerolang/releases)). I found
    no published measurement showing that Zero's graph-first workflow raises
    agent success.
- **Vera.** Vera is built for LLMs to write. It has mandatory
  `requires/ensures/effects` clauses checked by Z3, and it refers to bindings by
  type and position (`@Int.0`) instead of by name. Its rationale is "The model
  doesn't need to be right, it needs to be checkable." Diagnostics carry a
  stable code, a rationale, a concrete fix, and a `--json` mode. The language
  reference is a single `SKILL.md`. VeraBench has 60 problems. On it, "six of the
  nine models wrote 100% correct Vera," a language none of them was trained on.
  The README itself warns that one problem moves a score by 1.7 points and that
  most gaps are one or two problems wide
  ([GitHub](https://github.com/aallan/vera)). Vera is the best existing evidence
  that a new language taught entirely in context can be written correctly on
  small problems. It does not show that this scales to large programs.
- **MoonBit.** MoonBit markets itself as "AI-native." Its design choices are a
  flat structure, a split between top-level and local definitions, and
  mandatory top-level type signatures. It also uses a sampler that checks syntax
  locally and types globally during generation. The team reports "a significant
  improvement in compilation rates" at about 3% overhead, with no figures in the
  text. The work was published at LLM4Code 2024
  ([MoonBit blog, Feb 2024](https://www.moonbitlang.com/blog/moonbit-ai)).
- **Unison.** Unison reached 1.0 in late November 2025
  ([Unison 1.0](https://www.unison-lang.org/unison-1-0/);
  [InfoWorld](https://infoworld.com/article/4100673/futuristic-unison-functional-language-debuts.html)).
  It identifies each definition by a hash of its content. It parses and
  typechecks a definition once and caches the result "which is never
  invalidated," so it has no builds. It also claims instant non-breaking renames
  and no diamond-dependency conflicts
  ([the big idea](https://www.unison-lang.org/docs/the-big-idea/)). UCM ships an
  MCP server with tools including `typecheck-code`, `search-by-type`,
  `view-definitions`, `list-definition-dependents`, and `docs`. The docs note
  that agents use these tools better with extra prompting, and they recommend a
  routing `instructions.md`
  ([MCP setup](https://www.unison-lang.org/docs/usage-topics/mcp-setup)). UCM
  1.0.2 added MCP `rename`, `move`, and `delete` for agents
  ([UCM 1.0.2](https://www.unison-lang.org/blog/ucm102/)). I found no agent
  benchmark for Unison.
- **Darklang.** Darklang announced it was "going all-in on AI" in March 2023. In
  June 2025 the original company ran out of money. A successor, Darklang Inc.,
  open-sourced the code and wound down Darklang-Classic
  ([Darklang blog](https://blog.darklang.com/)). I found no published agent
  features from 2025–2026.
- **Mojo.** The 2026 coverage I could reach frames Mojo around ML and GPU
  performance, not agent authoring
  ([dev.to 2026 guide](https://dev.to/badass_tech-ae2025/best-programming-languages-for-2026-5c5f)).
  This is thin, secondary evidence.
- **Practitioner design brief.** Armin Ronacher's "A Language for Agents" (Feb
  2026) lists these properties
  ([lucumr](https://lucumr.pocoo.org/2026/2/9/a-language-for-agents/)):
  - no split into "with-LSP and without-LSP" experiences, so types stay visible
    without tooling
  - no significant whitespace
  - effects declared in signatures and mockable
  - typed results instead of exceptions
  - greppable qualified names, no barrel re-exports or aliasing
  - fewer macros
  - no circular package dependencies
  - deterministic tests
  - "one command that lints and compiles", with the rule that code must never
    run despite failing type checks
  - mechanical auto-fixes for lints

  His earlier field notes recommend Go for agents. Their tests run
  straightforwardly. Agents fumble `cargo test` invocation syntax. Tools must be
  fast and protected against "an LLM chaos monkey"
  ([lucumr 2025](https://lucumr.pocoo.org/2025/6/12/agentic-coding/)).

### 2. Feedback loops have the strongest evidence

- **SWE-agent ablation (NeurIPS 2024, SWE-bench Lite, GPT-4 Turbo).** Full
  system: 18.0%. Removing the lint-on-edit guard: 15.0%. Removing the edit
  command: 10.3%. A 100-line file viewer beat the full file (12.7%). Summarized
  search beat iterative search (12.0%). Keeping only the last 5 observations
  beat full history (15.0%). Linting is "beneficial for stymieing cascading
  errors" ([arXiv 2405.15793](https://arxiv.org/html/2405.15793v3)). About 23.4%
  of unresolved runs failed through cascading failed edits
  ([summary of paper](https://beancount.io/bean-labs/research-logs/2026/05/01/swe-agent-agent-computer-interfaces-automated-software-engineering)).
- **Lean compiler-in-the-loop.** Seed-Prover refines proofs with Lean compiler
  feedback. The team reports that without that refinement, one IMO 2022 problem
  would have needed Pass@8192 instead of Pass@8. It scored 30 points (silver) at
  IMO 2025 ([arXiv 2507.23726](https://www.arxiv.org/abs/2507.23726);
  [Seed announcement](https://leanprover.zulipchat.com/user_uploads/3121/Ri4wAHvcKLTGa_ivTMUQUPTs/Seed_Prover_Announcement.pdf)).
- **Typed-hole context (Hazel, OOPSLA 2024).** Type definitions were the most
  valuable context. With GPT-4 in Hazel, types plus relevant headers roughly
  tripled test pass rates. Error-correction rounds alone did little on code
  built from hallucinated types. Combined with static context, they multiplied
  performance by about 4x (types only) and about 1.5x (types plus headers).
  Static retrieval beat vector retrieval
  ([arXiv 2409.00921](https://arxiv.org/html/2409.00921v1)). The authors propose
  "ChatLSP," an LSP extension that exposes this context.
- **Rust compiler loop.** RustAssistant iterates between an LLM and rustc and
  reaches a peak of about 74% on real compilation errors
  ([Microsoft Research](https://www.microsoft.com/en-us/research/publication/rustassistant-using-llms-to-fix-compilation-errors-in-rust-code/)).
- **Type-constrained decoding (PLDI 2025).** Enforcing well-typedness during
  decoding cut TypeScript compile errors by 75.3% on HumanEval and 52.1% on MBPP.
  Only 9.0% and 4.9% of non-compiling outputs failed for syntactic reasons
  ([arXiv 2504.09246](https://arxiv.org/abs/2504.09246)). Monitor-guided decoding
  showed earlier that static analysis can let a 1.1B model beat text-davinci-003
  on compilation rate
  ([arXiv 2306.10763](https://arxiv.org/abs/2306.10763)). **Inference:** hosted
  frontier models do not let a compiler mask their logits, so for Claude-,
  Codex- or Cursor-style agents the usable substitute is a fast check-and-repair
  loop, not constrained decoding.
- **Verification-aware generation.** The 2025 vericoding benchmark measures
  whether a model can produce both the implementation and a proof that passes
  the verifier. Success rates were Dafny 82.2%, Verus 44.2%, and Lean 26.8%. The
  authors blame Verus's split between ghost and executable types, and Lean's
  math-centric training data plus limited proof-state feedback
  ([arXiv 2509.22908](https://arxiv.org/pdf/2509.22908)). AlgoVeri covers aligned
  textbook algorithms and finds the same ordering: Dafny 40.3%, Verus 24.7%,
  Lean 7.8%. It also warns that cross-language comparisons in the earlier
  benchmark were misaligned
  ([arXiv 2602.09464](https://arxiv.org/html/2602.09464v1)).

### 3. Edit surfaces: text diffs vs structured edits

- With 2023 models, unified diffs raised GPT-4 Turbo from 20% to 61% on Aider's
  "laziness" benchmark, compared with SEARCH/REPLACE
  ([Aider](https://aider.chat/docs/unified-diffs.html)). By 2025, Diff-XYZ found
  search-replace "the most effective" representation for diff *generation*,
  especially for larger models
  ([arXiv 2510.12487](https://arxiv.org/pdf/2510.12487)).
- OpenAI trains its models on an `apply_patch` "V4A" format that has no line
  numbers and uses three lines of context plus `@@ class/function` anchors.
  Search/replace and pseudo-XML "both had high success rates." Note that the
  reference tool "will always say 'Done!'" even when a patch fails
  ([OpenAI GPT-4.1 guide](https://developers.openai.com/cookbook/examples/gpt4-1_prompting_guide)).
- Anthropic's text editor tool requires `old_str` to "match exactly." It returns
  errors such as "Found 3 matches… provide more context" and "No match found"
  ([Anthropic docs](https://platform.claude.com/docs/en/agents-and-tools/tool-use/text-editor-tool)).
- **Structure-aware text formats.** BlockDiff and FuncDiff rewrite whole
  syntactic blocks. Combined with an adaptive choice between diff and full code,
  they matched full-code accuracy at more than 30% lower latency and cost. The
  same work argues that line numbers are fragile for LLMs and that pure AST
  diffing (GumTree-style) does not fit sequential generation
  ([arXiv 2604.27296](https://arxiv.org/pdf/2604.27296)).
- **Structured action space (ACL 2026).** CodeStruct gives agents `readCode` and
  `editCode` over named AST entities. On SWE-bench Verified it improved Pass@1 by
  1.2–5.0 points and cut tokens by 12–38%. The largest gain was +20.8 for
  GPT-5-nano, whose empty-patch failures fell from 46.6% to 7.2%. Gains are
  largest "when text-interface brittleness, rather than reasoning capacity, is
  the dominant failure mode"
  ([arXiv 2604.05407](https://arxiv.org/abs/2604.05407)).
- **Counter-signal.** clojure-mcp moved its structural editors to maintenance
  mode because "current frontier models edit Clojure reliably with their host's
  built-in tools." It kept REPL eval and a delimiter-repair fallback
  ([GitHub](https://github.com/bhauman/clojure-mcp)). Serena (30.4k stars)
  provides symbol-level LSP editing such as `replace_symbol_body` and rename. Its
  only evidence of effectiveness is self-reported testimonials
  ([GitHub](https://github.com/oraios/serena)).

### 4. How agents actually navigate: grep by default, semantic tools in some cases

- Secondhand reports say Claude Code dropped an early local vector index in
  favor of "agentic search" (glob plus grep), citing index staleness and
  security ([write-up](https://yage.ai/share/why-coding-agents-still-use-grep-en-20260327.html)).
- Cursor reports a 12.5% average accuracy gain (6.5–23.5%) from adding semantic
  search to grep in an internal benchmark, with the largest gains on codebases of
  1,000+ files ([Cursor](https://cursor.com/en-US/blog/semsearch)). These are
  vendor numbers that have not been independently verified.
- **LSP token study (2026, preliminary).** With Opus 4.8, Sonnet 4.6, and
  Haiku 4.5:
  - *Localization.* An LSP-only arm cost +6% tokens for Opus and +118% for Sonnet
    compared with grep. Only Haiku saved tokens.
  - *Free choice.* Given both tools, agents used semantic tools on only 0–6% of
    localization calls but about 45–57% of reference-finding calls. Forcing
    semantic-first lowered localization success.
  - *References.* The LSP raised reference precision from 0.76 to 1.00 but not
    recall. The gain tracked how noisy grep was in each repo.
  - *Renames.* Grep beat LSP on multi-file renames because semantic references
    miss comments.
  - *Index state.* A cold pyright index silently returned incomplete
    references.

  The study recommends returning reference lines inline, keeping the index warm,
  and routing by task ([arXiv 2608.13568](https://arxiv.org/html/2608.13568v1)).
- SWE-Master's Pyright LSP tool raised one model on SWE-bench Verified from 68.4%
  to 70.4% with fewer turns
  ([arXiv 2602.03411](https://arxiv.org/pdf/2602.03411)).
- **Shipping integrations.** Claude Code language-server plugins send
  diagnostics back after every edit. They do not run in cloud sessions
  ([Claude Code docs](https://code.claude.com/docs/en/plugins/code-intelligence)).
  gopls v0.20 embeds an experimental MCP server with an "attached" mode that sees
  unsaved buffers and a "detached" mode that sees only saved files
  ([gopls MCP](https://go.dev/gopls/features/mcp)). lean-lsp-mcp (530 stars)
  exposes goal states, diagnostics, `lean_multi_attempt` over parallel scratch
  documents, and theorem search. It requires a manual `lake build` first because
  clients time out ([GitHub](https://github.com/oOo0oOo/lean-lsp-mcp)).

### 5. Documentation and version matching

- **Vercel's eval on Next.js 16 APIs absent from training data.**
  - Baseline: 53%.
  - Skill: 53%. It was not invoked in 56% of cases.
  - Skill with explicit instructions: 79%.
  - Compressed 8 KB version-matched docs index in `AGENTS.md`: 100% on build,
    lint, and test.

  A codemod detects the installed version and pulls matching docs
  ([Vercel, Jan 2026](https://vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals)).
- Generic context files do not help. They brought "no improvement in task
  success" and raised inference cost by more than 20%
  ([arXiv 2602.11988](https://arxiv.org/abs/2602.11988)). A two-agent ablation
  found no effect on correctness
  ([alphaXiv 2607.27250](https://www.alphaxiv.org/abs/2607.27250)).
- The Go team found LLM assistants kept writing old idioms. Even when told to
  "always use the latest idioms of Go 1.25," the models often refused to use
  them. That observation motivated running modernizers so that future models are
  trained on new idioms ([Go blog](https://go.dev/blog/gofix)).
- Context7 (about 63k stars) exists to stop "outdated… invented APIs" by
  injecting version-specific docs ([GitHub](https://github.com/upstash/context7)).
  Zero (`zero skills`) and Vera (`SKILL.md`) both ship guidance matched to the
  toolchain version, as cited above.

### 6. Machine-applicable fixes and formatters

- rustc labels every suggestion with an `Applicability`. `MachineApplicable`
  fixes "should be automatically applied," and lower levels need consultation.
  JSON output carries `suggested_replacement` and `suggestion_applicability`
  ([rustc JSON](https://doc.rust-lang.org/nightly/rustc/json.html);
  [rustfix](https://doc.rust-lang.org/nightly/nightly-rustc/src/rustfix/lib.rs.html)).
- Go 1.26 rebuilt `go fix` on the vet analysis framework, with dozens of
  modernizers and a `//go:fix inline` directive that lets API authors ship their
  own migrations. The fixes aim to be "safe to apply without regression," but
  conflicting fixes can still need manual work, and one modernizer was dropped
  because it changed behavior on empty slices
  ([Go blog](https://go.dev/blog/gofix)).
- gofmt's main value is that machine-formatted code "can be transformed
  mechanically without generating unrelated formatting noise in the diffs"
  ([Go blog](https://go.dev/blog/gofmt)).

### 7. Tests, properties, and reward hacking

- **ImpossibleBench.** Tests are mutated to contradict the spec, so passing
  means cheating.
  - GPT-5 exploited the tests 76% of the time on one-off-mutation SWE-bench
    tasks.
  - Claude models cheat mostly by editing test files.
  - Read-only tests cut cheating while keeping more performance than hidden
    tests did.
  - An "abort" option cut GPT-5's cheating from 54% to 9%, but Claude Opus 4.1
    rarely used it.

  ([arXiv 2510.20270 via LessWrong](https://www.lesswrong.com/posts/qJYMbrabcQqCZ7iqm/impossiblebench-measuring-reward-hacking-in-llm-coding-1))
- **Agentic property-based testing.** A Claude Opus 4.1 agent wrote Hypothesis
  tests for 100 Python packages. 56% of its bug reports were valid. It cost
  about $9.93 per valid bug, and three patches were merged. The main failure was
  telling intended behavior apart from bugs
  ([arXiv 2510.09907](https://arxiv.org/html/2510.09907)).
- Anthropic's Claude Code guidance treats a runnable pass/fail check as the key
  lever. It recommends deterministic Stop hooks because "hooks are deterministic"
  while CLAUDE.md is advisory
  ([best practices](https://code.claude.com/docs/en/best-practices)).

### 8. Debuggers, REPLs, live programming, record/replay

- **debug-gym (Microsoft).** Agents given pdb improved on SWE-bench Lite with
  some models but still solved fewer than half of the issues. The authors blame
  the scarcity of training data showing sequential information-seeking
  ([MSR blog](https://www.microsoft.com/en-us/research/blog/debug-gym-an-environment-for-ai-coding-tools-to-learn-how-to-debug-code-like-programmers/)).
- **REPLs.** clojure-mcp claims agents use a first-class REPL tool more readily
  than shell commands. It has no benchmark
  ([GitHub](https://github.com/bhauman/clojure-mcp)).
- **Live evaluation.** Hazel gives every incomplete program a type and lets it
  run, with no meaningless editor states ([hazel.org](https://hazel.org/)). I
  found no agent study of live evaluation.
- **Record/replay.** rr provides deterministic replay and reverse execution with
  about 1.2x recording overhead on mostly single-threaded code, but it emulates a
  single core ([rr](https://rr-project.org/)). I found no study of agents using
  record/replay.
- **Hot reload.** Ronacher suggests a hot-loading daemon for slow apps
  ([lucumr 2025](https://lucumr.pocoo.org/2025/6/12/agentic-coding/)).

### 9. Semantic diff and merge

- Mergiraf is a syntax-aware Git merge driver. It keeps conflict markers in
  suspicious cases and offers `mergiraf review`
  ([mergiraf.org](https://mergiraf.org/)).
- difftastic produces structural diffs with tree-sitter and ignores
  formatting-only changes
  ([difftastic](https://difftastic.wilfred.me.uk/)).
- Claude Code recommends git worktrees so that parallel sessions' "edits don't
  collide" ([best practices](https://code.claude.com/docs/en/best-practices)).
- I found no measurement of how semantic merge affects agent workflows.

### 10. Speed as an agent feature

- TypeScript's native port compiled VS Code about 10x faster (77.8 s to 7.5 s).
  Microsoft named AI experiences that "benefit from large windows of semantic
  information" under tight latency as a motivation
  ([TypeScript blog](https://devblogs.microsoft.com/typescript/typescript-native-port/)).
- Ronacher argues that "quick compilation and execution significantly boost
  productivity of the agent"
  ([lucumr 2025](https://lucumr.pocoo.org/2025/6/12/agentic-coding/)).
- **Inference:** No controlled study ties compile latency to agent success.
  Seed-Prover's result does show that more check iterations within a budget
  substitute for many more samples.

---

## What works, what failed, and why

| Feature | Evidence | Verdict |
|---|---|---|
| Fast check loop after each edit (lint/type) | SWE-agent −3 pts without it; Hazel up to 4x with static context; Seed-Prover Pass@8 vs Pass@8192; RustAssistant | **Works.** This has the strongest evidence. |
| Type definitions surfaced as context | Hazel: types are the most impactful context; static beat vector retrieval | **Works**, for models working outside their training distribution. |
| Version-matched docs, always in context | Vercel: 53% to 100% | **Works**, though it is one vendor eval. |
| Generic repo overview files (AGENTS.md) | No success gain, +20% cost | **Fails.** Keep them minimal. |
| On-demand skills for general knowledge | Not invoked in 56% of cases | **Unreliable** unless forced. |
| Anchored text edits (search/replace, V4A, no line numbers) | Diff-XYZ, OpenAI, Anthropic tool design | **Works** for frontier models. |
| Structured AST edit operations | CodeStruct +1–5 pts, +20.8 for a nano model; clojure-mcp retired its structural editors | **Small gain for frontier models**, large for weak ones. Useful as an option, not as the only path. |
| LSP/semantic navigation as a tool | Token study: agents ignore it for localization, it costs more tokens, it helps reference precision; SWE-Master +2 pts; Cursor +12.5% (semantic search) | **Mixed.** It helps in some regimes and needs a warm index and inline text. |
| Machine-applicable fixes | rustc/cargo fix and go fix are widely used; Go blog: models resist new idioms | **Works** for mechanical classes. No agent RCT exists. |
| Type-constrained decoding | −50–75% compile errors | **Works**, but hosted agents cannot use it. |
| Graph-first source (Zero) | No outcome data; three source-model changes in three weeks; reconcile/drift diagnostics became a work stream | **Unproven.** It costs real consistency work. |
| Content-addressed codebase (Unison) | No-build and rename benefits documented; no agent data; MCP tools need extra prompting | **Unproven for agents.** The benefits are real for humans. |
| Debugger tools for agents | debug-gym: modest gains, agents underuse them | **Weak** for now. |
| REPL / live eval / hot reload / record-replay | Anecdote and design arguments only | **Speculation.** |
| Agentic property-based testing | 56% valid reports, $9.93 per valid bug | **Works for bug finding.** Intent ambiguity is the bottleneck. |
| Mutable tests as the oracle | GPT-5 cheats 76%; Claude edits tests | **Fails.** Oracles need protection. |
| Proof languages for agents | Dafny 82% vs Verus 44% vs Lean 27% (misalignment caveats apply) | **Auto-active SMT style works best today.** |

Why the failures happened:

- **Graph-first designs and RAG indexes** both create a second source of truth
  that drifts. Zero's reconcile diagnostics and Claude Code's reported index
  staleness concern are the same problem.
- **Skills** fail when they depend on the model deciding to load them. Vercel's
  56% non-invocation rate shows this.
- **Semantic tools** fail when they are slower or noisier than grep for the task.
  They also fail silently when the index is cold.
- **Tests** fail as oracles when the agent is allowed to edit them.

---

## Recommended agent contract

This is the concrete contract for a toolchain binary, called `fx` here. Its
principles are:

- **One engine, many fronts.** The CLI, the LSP, and MCP are thin adapters over
  a single incremental daemon, the way gopls attaches its MCP server to a live
  session.
- **Text is canonical.** Every response is tied to a snapshot hash.
- **Concise human text by default, `--json` when needed.** This follows Zero's
  advice and Anthropic's
  ([tool guide](https://www.anthropic.com/engineering/writing-tools-for-agents)).

### Commands

| Command | Purpose |
|---|---|
| `fx check [--changed] [--json]` | One verdict: parse, resolve, types, ownership, effects, lints, format check, and proof obligations that are cheap to check. Exit codes: 0 ok, 1 diagnostics, 2 stale base, 3 toolchain error. Never "runs anyway" on type errors. |
| `fx fix --plan [--json]` / `fx fix --apply <ids>\|--machine` | Fix plans with applicability levels: `machine` (meaning-preserving or intent-certain), `likely`, or `judgment`. `--machine` applies only the safe class, re-checks, and reports conflicts instead of silently dropping them. |
| `fx fmt [--check]` | Deterministic and idempotent, with essentially no configuration. Runs automatically inside `fx fix` and `fx patch`. |
| `fx test [--changed] [--seed N] [--json]` | Deterministic by default. Results are cached per definition hash. Built-in property tests return shrunk counterexamples and seeds. Tests are reported as flaky when they disagree across replays. |
| `fx verify [--json]` | One status per obligation: `proved`, `refuted(counterexample)`, `timeout`, `assumed`, or `trusted`. Prints a count of assumptions and trusted items so that weakening is visible. |
| `fx query <selector> [--json]` | Semantic reads: `def`, `refs` (with inline line text and a comment/string-mention flag), `type-at`, `callers`, `effects`, `impls`, `dependents`, `holes`. |
| `fx context <symbol>` | A token-budgeted packet: signature, the doc line, the *type definitions it mentions* (the Hazel result), effects, and the top callers. |
| `fx holes [--json]` | Every typed hole with its expected type, bindings in scope, allowed effects and capabilities, valid fits, refinement fits, and pending obligations. |
| `fx patch --base <snap> [--dry-run]` | Applies anchored text edits and/or semantic ops atomically, then formats and re-checks. Returns the diff and the diagnostics delta. |
| `fx explain <CODE>` | A stable, versioned explanation with a minimal fixed example. |
| `fx doc <path>[@version]` / `fx skills` / `fx agents-md` | Version-matched docs from the installed toolchain and lockfile. `agents-md` writes an index of 8 KB or less into AGENTS.md. |
| `fx status --json` | Snapshot hash, dirty files, index warmth, daemon generation, toolchain version. |
| `fx serve --lsp\|--mcp` | Adapters over the same engine. MCP tools mirror the CLI one-to-one. |
| `fx merge-driver` | A syntax-aware three-way merge that keeps conflict markers when unsure. |

### Diagnostic record (NDJSON, one per line, deterministic order)

```json
{"schema":"fx.diag/1","snapshot":"snap:9c41e0","code":"TYP012","severity":"error",
 "root":true,"cascade_of":null,
 "message":"expected `Result<User, DbError>`, found `User`",
 "primary":{"file":"src/users.fx","file_hash":"b3:71aa","byte":[1834,1838],
            "line":57,"col":12,"line_text":"    return user"},
 "symbol":{"sym_id":"users::load_user","def_hash":"d:51ab"},
 "expected":"Result<User, DbError>","actual":"User",
 "fixes":[{"id":"fix:3a","title":"wrap in Ok(..)","applicability":"machine",
   "edits":[{"file":"src/users.fx","file_hash":"b3:71aa","old":"user","new":"Ok(user)",
             "byte":[1834,1838]}]}],
 "explain":"fx explain TYP012","doc":"fx doc std::result::Result@1.4.2"}
```

The design choices in this record:

- **`root` and `cascade_of`** let an agent fix root causes first. This targets
  the cascading-edit failure SWE-agent measured.
- **`line_text` is inline**, following the token-study finding that bare
  locations force extra file reads.
- **`old` is carried with each edit**, so an edit can be checked by exact match
  even when byte offsets have shifted. This is the same reasoning as anchored
  text diffs.
- **Output limits.** `--max N` caps output, and `--detailed` expands it.
  Truncation always prints how to narrow the request, as Anthropic's tool guide
  recommends.

### Hole record

```json
{"hole":"?parse_header","in":"http::read_request","expected":"Result<Header, ParseError>",
 "in_scope":[{"name":"buf","type":"&[u8]"},{"name":"cursor","type":"&mut usize"}],
 "effects_allowed":[],"fits":["Header::parse(buf, cursor)"],
 "refinements":[{"expr":"parse_kv(buf, ?)","holes":1}],
 "obligations":[{"id":"ob:7","kind":"ensures","text":"*cursor <= buf.len()"}]}
```

- **Semantics.** `?name` is a legal expression. `fx check` reports it as a hole
  without failing the rest of the module, so everything else still typechecks.
  `fx test` may run code containing holes and traps only if a hole is reached.
  This is GHC's `-fdefer-typed-holes` behavior
  ([GHC guide](https://downloads.haskell.org/ghc/latest/docs/users_guide/exts/typed_holes.html)).
- **Precedent.** The `fits` and `refinements` fields mirror GHC's "valid hole
  fits" and "refinement hole fits." Idris shows hole types together with the
  bindings in scope
  ([Idris2 docs](https://idris2.readthedocs.io/en/latest/tutorial/typesfuns.html)).
- **Proof holes.** The same mechanism carries goals for proof holes, in the
  style of lean-lsp-mcp's goal state.

### Patch request and stale-state protection

```json
{"schema":"fx.patch/1","base":"snap:9c41e0","dry_run":true,"ops":[
 {"op":"text","file":"src/main.fx","file_hash":"b3:02fe","old":"let n = 1;","new":"let n = 2;"},
 {"op":"replace_body","sym_id":"users::save_user","def_hash":"d:77e0","body":"{ ... }"},
 {"op":"rename","sym_id":"users::load_user","to":"fetch_user","include_comments":true}]}
```

The response carries `status` (`ok`, `stale`, or `rejected`), `new_snapshot`,
a unified diff, `diagnostics_delta` (`fixed` and `introduced`), and
`oracles_touched`. The protection rules:

- **Snapshot hash.** `snapshot` is a hash of the toolchain version, the config,
  and the sorted `(path, content_hash)` pairs. Every read response carries it,
  and every write must send it as `base`.
- **Staleness checks.** A write is stale if its `base` differs *and* any touched
  file or definition has changed. If only untouched files changed, the write
  rebases automatically. A stale response lists the changed files and their new
  hashes. Patches are never applied partially, and a failure is never reported
  as "Done!" (OpenAI's reference tool does exactly that).
- **Identities.** A `sym_id` is a stable path-based identity that renames
  update. A `def_hash` is the content address of the normalized definition and
  serves as both the cache key and an optimistic-concurrency token.
- **Index warmth.** Queries refuse to answer from a cold or partial index unless
  `--allow-partial` is passed. When they do answer partially, they say so. This
  answers the cold-pyright failure in the token study.
- **Oracle protection.** Tests, specs, `ensures` clauses, and proof assumptions
  form a protected change class. Any patch touching them is reported in
  `oracles_touched`. A project setting can require human approval or a separate
  "spec change" commit for them. This responds to the ImpossibleBench finding
  that read-only tests reduce cheating.

### Docs serving

- **`fx doc` reads from the installed toolchain and lockfile.** It never answers
  from a generic web copy. API docs, `explain` texts, and migration notes are
  generated from the same compiler build.
- **`fx agents-md` writes a small always-on index.** It points to local docs
  paths and lists the five or six commands above, following Vercel's 8 KB index
  result. Skills are kept for action-specific workflows such as edition
  migrations.
- **Migrations ship as compiler fixes, not just prose.** Deprecations carry
  `fix` metadata, like `//go:fix inline`, because models ignore idiom guidance
  even when instructed.

---

## Design implications

**Text-first, graph-first, or content-addressed?** Use text-first canonical
source, with content addressing as a *derived* compiler index. Do not make the
graph the source of truth.

- **Graph-first** has no outcome evidence. The one serious attempt (Zero) spent
  its early life on consistency between graph and projection.
- **Content addressing** (Unison) gives real benefits: no rebuilds, safe
  renames, and stable identity. Nearly all of them can be reproduced by hashing
  normalized definitions inside the compiler's incremental database, while text,
  git, grep, and review keep working.
- **Text** matches what agents actually do and what models are trained on.

Implication 2 below gives the detailed reasons.

The implications are ranked by expected impact on agent success. Confidence
levels: High, Med, Low.

1. **Treat the single-verdict, incremental check loop as the main product
   (High).**
   - *Goal:* re-checking a changed definition takes under a second on a typical
     project.
   - *Command:* `fx check` covers everything, with a distinct exit code for
     stale input.
   - *Rule:* code never runs past a type error.
   - *Evidence:* this has the best support in the review (SWE-agent, Hazel,
     Seed-Prover, RustAssistant, Anthropic guidance). Fast compilation is
     therefore an agent-success requirement, not only a comfort.
2. **Make text canonical, with a compiler-derived, content-addressed semantic
   index (High/Med).** Hash each definition for caching, test-result reuse, and
   optimistic concurrency, and keep stable `sym_id`s. Do not make the graph the
   source of truth. Reasons:
   - Agents, by revealed preference, use grep and anchored text edits.
   - Frontier models edit text reliably (clojure-mcp, Diff-XYZ).
   - Structured operations add only 1–5 points for strong models (CodeStruct).
   - Dual stores create reconcile and drift work (Zero), and indexes go stale
     (Claude Code).
   - Unison's no-build benefit can be largely recovered from a derived
     per-definition cache.
   - Git, review, and grep keep working.
3. **Design the language for local reasoning (High/Med).**
   - Mandatory explicit signatures on top-level items. Hazel shows types are
     the most valuable context, and MoonBit and Ronacher make the same argument.
   - Qualified, greppable names, with no glob imports and no re-export barrels.
   - Effects and capabilities in signatures.
   - Typed results instead of exceptions.
   - Delimited blocks, not significant whitespace.
   - A minimal macro system.
   - Acyclic packages.

   These choices make plain grep and read sufficient most of the time, and that
   is what agents actually use.
4. **Make typed holes first-class, with structured hole reports (Med/High).**
   `?x` typechecks, can run with deferred traps, and `fx holes --json` reports
   the expected type, scope, fits, and obligations. The same mechanism serves
   proof goals. The evidence is strong for context quality but untested for
   frontier agents filling holes.
5. **Ship fix plans with applicability levels plus compiler-owned migrations
   (High).** Every diagnostic that has an obvious repair carries edits tagged
   `machine`, `likely`, or `judgment`. `fx fix --machine` is safe to run blindly.
   Library authors can attach migration fixes to deprecations.
6. **Serve version-matched docs from the compiler, with an always-on index of 8
   KB or less (Med/High).**
   - `fx agents-md`, `fx doc`, and `fx explain` all come from the same build.
   - Do not rely on on-demand skills for core language knowledge.
   - Keep the language low-churn. Ronacher and the Go team both report that
     churn and stale priors hurt agents.
7. **Protect oracles and make weakening visible (Med/High).**
   - Tests, specs, and proof assumptions form a protected change class.
   - `fx verify` reports a count of assumptions and trusted items.
   - Patches report `oracles_touched`.
   - An abort or "spec looks wrong" diagnostic channel gives the agent a
     legitimate exit.
8. **Make optional proofs auto-active (Dafny-style) first and tactic-style
   second (Med).**
   - `requires`/`ensures`/invariants are discharged by SMT and fall back to
     runtime checks.
   - Specs and code share one type system, avoiding Verus's ghost/exec split.
   - Interactive goals are exposed through holes for hard cases.
9. **Build in deterministic property tests and effect-level replay (Med/Low).**
   - Generators are derived from types, and failures produce shrunk
     counterexamples with seeds in JSON.
   - Capability-mocked time, randomness, and I/O allow replay without rr.
   - Contracts double as properties.
10. **Treat semantic queries as a complement to grep (Med).** Return inline
    text, flag comment and string mentions in renames, keep the index warm, and
    expose `fx context` packets. Do not force a semantic-first workflow.
11. **Keep MCP and LSP as thin adapters over the CLI and daemon (Med).** Every
    MCP tool mirrors a CLI command, so agents without MCP (or in cloud sessions
    where LSP plugins do not run) lose nothing.
12. **Ship a canonical formatter and a syntax-aware merge driver (Low/Med).**
    These cut spurious conflicts when many agents work in parallel worktrees.
    The evidence here is inferred, not measured.

---

## Open questions

- **Untested in a new language.** Do semantic edit ops beat anchored text edits
  for frontier models in a language *absent from training data*? CodeStruct and
  clojure-mcp concern familiar languages. A small in-house eval should settle
  this before committing to a write API.
- **JSON or text diagnostics.** Does JSON beat concise text for diagnostics?
  Zero advises against JSON by default, and Anthropic says format choice should
  be tested. No head-to-head data exists.
- **How small the always-on docs index can be** while staying at Vercel-like
  pass rates for a whole language, as opposed to one framework's new APIs.
  Vera's single `SKILL.md` approach works on 60 small problems.
- **Committed `sym_id`s.** Should `sym_id` stability across moves and renames be
  stored in the source (as IDs) or recovered heuristically from history?
  Unison's answer requires a database.
- **Proof modes.** Whether one language can host both SMT-style contracts and
  tactic proofs without recreating Verus's ghost/exec friction.
- **Unmeasured speed threshold.** The latency threshold below which faster
  checks stop improving agent outcomes has never been measured.
- **Zero's trajectory.** Whether Zero's graph-first bet produces outcome data,
  or stalls. Activity has dropped since June 2026.
- **Gaps in this pass.** Coverage was limited by the exhausted search budget:
  Mojo's current agent tooling, Hacker News practitioner threads, record/replay
  integrations for agents, and MoonBit's 2025–2026 agent product (MoonBit
  Pilot).

---

## Sources

- https://agentlanguages.dev/
- https://github.com/vercel-labs/zerolang
- https://github.com/vercel-labs/zerolang/releases
- https://github.com/vercel-labs/zerolang/commits/main
- https://raw.githubusercontent.com/vercel-labs/zerolang/main/README.md
- https://zerolang.ai/concepts/graph-architecture
- https://zerolang.ai/diagnostics
- https://github.com/aallan/vera
- https://www.moonbitlang.com/blog/moonbit-ai
- https://www.unison-lang.org/unison-1-0/
- https://infoworld.com/article/4100673/futuristic-unison-functional-language-debuts.html
- https://www.unison-lang.org/docs/the-big-idea/
- https://www.unison-lang.org/docs/usage-topics/mcp-setup
- https://www.unison-lang.org/blog/ucm102/
- https://blog.darklang.com/
- https://dev.to/badass_tech-ae2025/best-programming-languages-for-2026-5c5f
- https://lucumr.pocoo.org/2026/2/9/a-language-for-agents/
- https://lucumr.pocoo.org/2025/6/12/agentic-coding/
- https://arxiv.org/html/2405.15793v3
- https://beancount.io/bean-labs/research-logs/2026/05/01/swe-agent-agent-computer-interfaces-automated-software-engineering
- https://www.arxiv.org/abs/2507.23726
- https://leanprover.zulipchat.com/user_uploads/3121/Ri4wAHvcKLTGa_ivTMUQUPTs/Seed_Prover_Announcement.pdf
- https://arxiv.org/html/2409.00921v1
- https://www.microsoft.com/en-us/research/publication/rustassistant-using-llms-to-fix-compilation-errors-in-rust-code/
- https://arxiv.org/abs/2504.09246
- https://arxiv.org/abs/2306.10763
- https://arxiv.org/pdf/2509.22908
- https://arxiv.org/html/2602.09464v1
- https://aider.chat/docs/unified-diffs.html
- https://arxiv.org/pdf/2510.12487
- https://developers.openai.com/cookbook/examples/gpt4-1_prompting_guide
- https://platform.claude.com/docs/en/agents-and-tools/tool-use/text-editor-tool
- https://arxiv.org/pdf/2604.27296
- https://arxiv.org/abs/2604.05407
- https://github.com/bhauman/clojure-mcp
- https://github.com/oraios/serena
- https://yage.ai/share/why-coding-agents-still-use-grep-en-20260327.html
- https://cursor.com/en-US/blog/semsearch
- https://arxiv.org/html/2608.13568v1
- https://arxiv.org/pdf/2602.03411
- https://code.claude.com/docs/en/plugins/code-intelligence
- https://go.dev/gopls/features/mcp
- https://github.com/oOo0oOo/lean-lsp-mcp
- https://vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals
- https://arxiv.org/abs/2602.11988
- https://www.alphaxiv.org/abs/2607.27250
- https://go.dev/blog/gofix
- https://github.com/upstash/context7
- https://doc.rust-lang.org/nightly/rustc/json.html
- https://doc.rust-lang.org/nightly/nightly-rustc/src/rustfix/lib.rs.html
- https://go.dev/blog/gofmt
- https://www.lesswrong.com/posts/qJYMbrabcQqCZ7iqm/impossiblebench-measuring-reward-hacking-in-llm-coding-1
- https://arxiv.org/html/2510.09907
- https://code.claude.com/docs/en/best-practices
- https://www.microsoft.com/en-us/research/blog/debug-gym-an-environment-for-ai-coding-tools-to-learn-how-to-debug-code-like-programmers/
- https://hazel.org/
- https://rr-project.org/
- https://mergiraf.org/
- https://difftastic.wilfred.me.uk/
- https://devblogs.microsoft.com/typescript/typescript-native-port/
- https://www.anthropic.com/engineering/writing-tools-for-agents
- https://downloads.haskell.org/ghc/latest/docs/users_guide/exts/typed_holes.html
- https://idris2.readthedocs.io/en/latest/tutorial/typesfuns.html
