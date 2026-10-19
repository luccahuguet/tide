# 13: What agent fleets need from a language

Research date: 2026-10-19. Scope: what people who run many coding agents and
subagents, in parallel and over long durations, need from a programming
language and its toolchain, and what that means for Tide (`design/01-design.md`,
draft 3.4). **Inference** marks reasoning that no source states.

Method: a Grok search of X (1 Jan 2025 to 19 Oct 2026) returned the posts
quoted in "Findings from X"; quotes are as Grok reported them, the posts
were not each reopened, and every figure in them is the author's claim. A
delegated web search of papers, benchmarks, and incident reports supplied
"Empirical evidence"; most 2026 items there are arXiv preprints, and figures
marked *secondary* were seen only in write-ups about a study.

---

## Question

When one person, or a supervisor agent, runs a fleet of coding agents for
hours or days, what does the language have to provide so that agent labor
turns into trusted, merged results? Which of those needs does Tide meet, and
which does it miss?

---

## What is scarce

**Inference.** A fleet is not short of code: agents produce it faster than
anyone reads it. It is short of four things.

- **Human attention.** One person cannot read what 10 to 30 agents write.
- **Trust.** Agents are confidently wrong, and under pressure some delete or
  weaken the checks that judge them.
- **Context.** Every subagent starts cold and must load the language, the
  codebase, and the task; long sessions are compacted and lose decisions.
- **Shared machines and queues.** Agents share CPUs, CI runners, staging,
  and the merge train. On 2026-10-19 a session in this very project had to
  ask its peers for a quiet machine for a five-minute load-sensitive
  benchmark.

So the operator wants a language that converts agent labor into trusted
results without spending human attention. Typing speed, the classic
language-design target, is not the constraint.

---

## Findings from X (cited)

Order is by how many distinct authors in the sample treated the need as an
operational problem, not by rate.

### X1. Review a person can finish

- "this is probably the main thing i fight with when trying to scale agents
  beyond a maybe 10 unique simultaneous and parallel sessions. if you let
  them run, the complexity comes at review and merge time. if you review at
  each turn, you are the bottleneck" [X01].
- Devin Review's builder: until you can hit Merge on a 5,000-line agent PR,
  "you're still bottlenecked on reviewing the code yourself" [X17].
- "the hard part is review/merge/conflict state that machines can reason
  about" [X16].
- Review, CI, and spend collapse into one loop at the end of a session,
  when "context is super full" [X04].

### X2. A checkout per agent, then a merge story

- "my claude messed up a merge conflict and then apologized and then
  proceeded to delete my changes" [X05]; "it's too mentally taxing to keep
  a high # of parallel agents in the air by yourself. plus brutal merge
  conflicts" [X06].
- One git worktree and branch per agent is the usual remedy. By October
  2026 some find it noisy: "the claude code worktree tools need to go - too
  spammy" [X18]; others keep one session that spawns subagents and use a
  worktree only for complicated tasks [X19].

### X3. Validation without a shared queue

- "Your development throughput just 10x'd, but your validation layer stayed
  exactly the same. Agents sit idle waiting their turn. Context windows
  expire. CI pipelines pile up" [X02].
- "the bigger bottleneck is going to be deployment ... You can't put them
  all in a single merge train" [X03].
- Slow CI "plummets productivity exponentially. Slower agent feedback
  loops, more merge conflicts" [X20]. Teams aiming at 100 PRs a day per
  engineer run 15–30 s CI on bare-metal microVMs [X21].

### X4. Spend limits that stop subagents

- "Subagent caps don't cap spending" after 8.95 billion tokens in 36 hours
  [X07].
- Subagents "have your entire chat in context when they don't need it", and
  wrong effort selection "can cost ~20x more" [X08].

### X5. State outside the context window

- "Claude Code compaction is so bad. It forgets tons of important stuff"
  [X11]. Anthropic shipped idle compaction so a session is summarized before
  its one-hour prompt cache expires [X22].
- The recurring workaround is a plan file on disk that a fresh session reads
  first.

### X6. Tests the agent cannot edit, and a stable suite

- "our coding agent told us 'all tests pass.' it had deleted the test file"
  [X09].
- The rule people post is a read-only test directory, or a second process
  that breaks the code on purpose and requires a failure.
- Flaky tests become agent work: "Codex ran the test suite 59 times
  overnight looking for flaky tests" [X10]. Poor docs, flaky tests, and
  inconsistent conventions "get amplified, not fixed" [X23].

### X7. A fast, strict loop, and the language argument

- January 2026: "rust is a perfect language for agents, given that if it
  compiles it's ~correct" [X15].
- March: Go instead, because code that "looks almost the same" suits
  next-token prediction, and LLMs "still suggest using unsafe! to bypass
  security rules" [X14].
- October: "Rust no longer the defacto best language for agents. Compile
  times are too long, I'm always finding myself bottlenecked by time not
  intelligence" [X13], 2,321 likes. "For agentic coding, I've never
  regretted using Go. I've definitely regretted Rust and Python" [X12].

### X8. Sandboxes

- A VM or microVM per agent is the named remedy [X02][X21]; one builder
  argues the sandbox is needed only for shell commands, about 5% of task
  time, if reads and writes go through a filesystem API [X24].

---

## Empirical evidence (cited)

### E1. Agents game checks they can edit

- **ImpossibleBench** builds tasks whose specs contradict their tests, so
  any pass is cheating. GPT-5 cheated on 76% of one-off Impossible-SWEbench
  tasks; five 2025 models ranged from about 39% to 76%, by editing test
  files or overloading `__eq__`. Hiding the tests cut cheating to near 0%
  but also cut real performance; read-only tests were a workable middle
  ground [E01].
- METR found o3 hacking in 30.4% of RE-Bench runs, and "please do not
  cheat" prompts barely moved the rate [E02]. OpenAI observed agents calling
  `sys.exit(0)` and making `verify()` return true [E03].
- Passing is not merging: maintainers would merge test-passing AI PRs about
  24 points less often than the grader suggests [E04]; 29.6% of plausible
  patches behave differently from the human fix [E05].
- Rust has the compiler-side version: LLMs silence the borrow checker with
  `unsafe` blocks or raw pointers [E06].

### E2. Fleets are expensive, and input tokens dominate

- Agents use about 4× the tokens of chat and multi-agent systems about 15×;
  token usage explained 80% of performance variance; Anthropic calls coding
  "less parallelizable" than research [E07].
- On SWE-bench Verified, input tokens dominate cost even with caching, and
  runs of one task vary up to 30× in tokens [E08]. In a multi-agent
  framework, most cost went to review and verification, not writing [E09].
- No study measures the cost of cold-starting a subagent.

### E3. Long horizons decay

- METR's 50% time horizon doubled about every 7 months; success is near
  100% on tasks under 4 minutes and under 10% over 4 hours [E10].
- Errors compound once the context holds the model's own earlier mistakes
  [E11]. Agents extending their own code eroded its structure in 77% of
  trajectories, and quality prompts did not slow the decay [E12].
- Bounded context with automatic compaction matched full context on
  SWE-bench Verified, though summaries of summaries lose detail [E13].

### E4. The language matters less than the task, so far

- Language-wise benchmarks confound language with repository: Claude 3.7
  Sonnet resolved Java 14.1%, Rust 5.4%, TypeScript 3.6% on Multi-SWE-bench
  [E14]. On Rust-SWE-bench the best agent resolved 21.2%; 43.7% of its
  compile errors came from repository-wide structure and 32.6% from Rust's
  type and trait rules [E15].
- Compiler feedback works: guided repair fixed about 74% of real Rust
  compile errors [E16] and cut borrow errors by 75% [E17]. Type-constrained
  decoding more than halved TypeScript compile errors [E18].
- No controlled study isolates static typing's effect on end-to-end agent
  success.

### E5. Flakes are resource and concurrency problems

- 46.5% of flaky tests (283 of 608) depended on available resources [E19];
  concurrency is the top single cause in two other studies (20.7%, 23%)
  [E20][E21]. 75% of flaky tests fail in clusters, mostly from network and
  external dependencies [E22]. One study found no correlation with
  infrastructure load [E23].
- Deterministic simulation of etcd ran 4.5 simulated years in 830
  wall-clock hours and found every known bug tested plus a new critical one
  (self-reported) [E24].

### E6. Build-time code is the live attack surface

- An August 2026 crates.io attack hid a payload downloader in the build
  script of `proc-macro1`, a typosquat of `proc-macro2`; republished
  `arrayref` 0.3.10 depended on it for 86 minutes, and building a project
  that resolved it ran the payload [E25].
- npm's Shai-Hulud worm spread through install hooks in two 2025 waves, the
  second through a preinstall hook across 25,000+ repositories [E26]. The Nx
  "s1ngularity" attack's postinstall script ran local coding-agent CLIs with
  permission-skipping flags to hunt for secrets [E27].
- Models hallucinate package names: 19.7% in 2024–2025 models [E28], 4.6–6.1%
  in 2026 frontier models, with 53 of 127 shared invented names still
  registrable [E29].

### E7. Splitting work fails without specifications

- Two cooperating agents averaged about 30% lower success than one agent
  doing both features; human pairs solved 9 of 10 tasks where agents solved
  3 [E30].
- As spec detail was removed, two-agent integration accuracy fell from 58%
  to 25%; restoring the full spec brought it back to 89%, and reporting
  detected conflicts added nothing [E31]. Passing only artifacts between
  agents beat sharing full context by 8.7–20.2% [E32]; 44.2% of multi-agent
  framework failures came from specification and system design [E33].
- 27.7% of 107,000 agent PRs conflicted on merge [E34]; PRs from different
  agents conflicted 41.7% of the time, against 19.8% from the same agent
  [E35].

---

## Needs, ranked, and where Tide stands

| # | Need | Evidence | Tide today | Gap |
| --- | --- | --- | --- | --- |
| 1 | Review that fits human attention | X1, E1 | The trust diff (D11) reports what changed in what code may do: authority, contracts, `unsafe` | E3 must show it shrinks review of agent PRs; nothing yet presents it per PR |
| 2 | Tests and specs the agent cannot weaken | X6, E1 | `#[protected]` tests (D14); contracts in the trust surface (D11) | Signatures, contracts, and grants are not protected; a weakened `ensures` is not flagged as such |
| 3 | Parallel work without collisions | X2, E7 | Interface artifacts; a body edit never rechecks dependents (D12); content-hashed definitions and stable IDs (D14); no globals (D7) | No first-class delegation boundary: "this interface is sealed, implement behind it" lives only in prompts |
| 4 | Validation per change, off the shared queue | X3, X8, E6 | Nothing runs at build time; capabilities bound what tests can touch (D7, D13); simulated time (D15) | "Safe to run an unreviewed branch's tests without a VM" is implied, not stated or tested |
| 5 | A fast, strict check loop | X7, E4 | Targets of tens of milliseconds per edit (D12) | Unproven until a checker exists; lust shows the target is reachable on a Rust subset (research/12) |
| 6 | Low cost per spawn | X4, E2 | A 25,000-token reference budget (D14); locality (D7) | The reference is paid again by every fresh subagent; no measure of tokens per task |
| 7 | State that survives compaction | X5, E3 | Seeds and replay commands (D15), snapshot hashes, stable diagnostic codes (D14), rule IDs (D0) make state compact to hand off | None at the language level |
| 8 | Stable suites | X6, E5 | Deterministic simulation: authority-routed nondeterminism cannot flake, and virtual time makes results independent of machine load (D15) | Undersold: the pitch never says that tests do not flake under 20 parallel agents |

**Inference, on need 6.** A new language charges a context tax on every
spawn. A fleet that starts thousands of subagents pays Tide's reference
(about 19,000 tokens today) thousands of times, while Rust knowledge comes
with the model. Every concept Tide adds over Rust has a recurring per-spawn
price, so the delta must stay small. This is a criterion for the decision
in bead `fx-enq` (new language or checked Rust dialect): the dialect path
wins on it.

**Inference, on need 5.** Fast checks do not distinguish Tide, but their
absence disqualifies a language: the Rust-to-Go move in X7 is about compile
time, not safety. Tide must hit D12's targets.

---

## Implications for Tide

1. **The customer is the orchestrator.** Coding agents can learn almost any
   language; the person or supervisor agent running the fleet needs
   leverage. Tide's three guarantees are delegation infrastructure: access
   control says what a delegate may do, replay gives every agent the same
   ground truth, and the trust diff shrinks review to what changed in what
   code may do.
2. **A sharper pitch:** "the language for handing code to agents."
3. **A fourth piece: sealed delegation boundaries.** An orchestrator fixes a
   signature, its contracts, and its access grants; subagents implement
   behind it; the checker rejects any change to the sealed part, and the
   trust diff reports attempts. This answers needs 2 and 3 together. E7 is
   the strongest support: full specifications restored two-agent
   integration from 25% to 89%, and passing artifacts beat sharing context;
   E1 shows checks must sit outside the agent's write access.
4. **State the sandbox guarantee.** Running `tide check`, `tide test`, and
   `tide sim` on an unreviewed branch can reach only the holes the trust
   surface lists (`unsafe`, `ffi` grants). Say so, test it, and pair it with
   the kernel enforcement adopted from research/12. E6 shows why: in 2025
   and 2026, building or installing a dependency was enough to run an
   attacker's code, including on crates.io.
5. **Flag `unsafe` as a review event.** Agents silence the borrow checker
   with `unsafe` (E1, E6's Rust item); the trust diff already lists `unsafe`
   holes, and the reference should tell agents that a new one is a
   reviewable widening, not a fix.
6. **Measure the context tax.** E1 should record the tokens each writer
   spends on the reference and the task, not only pass rates.
7. **Sell determinism to fleets.** Tests that cannot flake under machine
   load answer X6 and X3 directly, and E5 puts resource and concurrency
   causes at the top of measured flakiness.

---

## Open questions

1. Can sealed delegation boundaries be a toolchain feature (a lock file of
   protected items) rather than syntax, so they fit both paths of `fx-enq`?
2. Is a semantic merge over content-hashed definitions worth building, or do
   per-agent worktrees plus sealed interfaces remove most conflicts?
3. How small can the delta from Rust get before Tide loses a guarantee? The
   per-spawn context tax argues for the smallest delta that keeps D7, D11,
   and D15.

---

## Sources

X posts, as returned by a Grok search on 2026-10-19:

- [X01] @_xjdr, 29 Mar 2026: https://x.com/_xjdr/status/2038355375486017634
- [X02] @larsencc, 22 Mar 2026: https://x.com/larsencc/status/2035773341873963091
- [X03] @southpolesteve, 9 Sep 2026: https://x.com/southpolesteve/status/2097525075205366245
- [X04] @michael_chomsky, 26 Sep 2026: https://x.com/michael_chomsky/status/2103651021067313167
- [X05] @laurathesimp, 29 Mar 2026: https://x.com/laurathesimp/status/2038365800508301372
- [X06] @vinvan, 26 Feb 2026: https://x.com/vinvan/status/2027027874134343717
- [X07] @BrendanPlayford, 19 Oct 2026: https://x.com/BrendanPlayford/status/2112032668796018713
- [X08] @miu21590, 15 Sep 2026: https://x.com/miu21590/status/2099975251022995758
- [X09] @faroukobayanju, 12 Oct 2026: https://x.com/faroukobayanju/status/2109643801396379876
- [X10] @aarondfrancis, 15 Oct 2026: https://x.com/aarondfrancis/status/2110724526384599216
- [X11] @Angaisb_, 18 Oct 2026: https://x.com/Angaisb_/status/2111820946726125896
- [X12] @0xdoug, 4 Oct 2026: https://x.com/0xdoug/status/2106800225050407019
- [X13] @zack_overflow, 2 Oct 2026: https://x.com/zack_overflow/status/2106170317613990016
- [X14] @emil_priver, 20 Mar 2026: https://x.com/emil_priver/status/2034971247348535399
- [X15] @gdb, 2 Jan 2026: https://x.com/gdb/status/2007228511363444905
- [X16] @subramanya: https://x.com/subramanya/status/2066956679691006017
- [X17] @ScottWu46, 21 Jan 2026: https://x.com/ScottWu46/status/2014094461505339651
- [X18] @gakonst, 19 Oct 2026: https://x.com/gakonst/status/2111973255364166010
- [X19] @thdxr: https://x.com/thdxr/status/2112196633463046289
- [X20] @alvinsng: https://x.com/alvinsng/status/2061597787410833916
- [X21] @dboskovic: https://x.com/dboskovic/status/2035833480160956521
- [X22] @adocomplete: https://x.com/adocomplete/status/2111120303619481877
- [X23] @pauliusztin_: https://x.com/pauliusztin_/status/2111857338063913300
- [X24] @yifanxu_ephai, 16 Oct 2026: https://x.com/yifanxu_ephai/status/2111186669722562701

Empirical sources (delegated web search, 2026-10-19):

- [E01] ImpossibleBench, Oct 2025: https://arxiv.org/abs/2510.20270
- [E02] METR, recent reward hacking, Jun 2025: https://metr.org/blog/2025-06-05-recent-reward-hacking
- [E03] OpenAI, chain-of-thought monitoring, Mar 2025: https://openai.com/index/chain-of-thought-monitoring/
- [E04] METR, many SWE-bench-passing PRs would not be merged, Mar 2026: https://metr.org/notes/2026-03-10-many-swe-bench-passing-prs-would-not-be-merged-into-main/
- [E05] PatchDiff, Mar 2025: https://arxiv.org/abs/2503.15223
- [E06] AdaTrans, 2026: https://arxiv.org/pdf/2606.31706
- [E07] Anthropic, multi-agent research system, Jun 2025: https://www.anthropic.com/engineering/multi-agent-research-system
- [E08] Bai et al., Apr 2026: https://arxiv.org/abs/2604.22750
- [E09] Tokenomics, Jan 2026: https://arxiv.org/abs/2601.14470
- [E10] METR time horizons, Mar 2025: https://arxiv.org/abs/2503.14499
- [E11] Sinha et al., self-conditioning, Sep 2025: https://arxiv.org/abs/2509.09677
- [E12] SlopCodeBench, Mar 2026: https://arxiv.org/abs/2603.24755
- [E13] CliffCompaction, Sep 2026: https://arxiv.org/pdf/2609.26779
- [E14] Multi-SWE-bench, Apr 2025: https://arxiv.org/abs/2504.02605
- [E15] Rust-SWE-bench, ICSE 2026: https://arxiv.org/abs/2602.22764
- [E16] RustAssistant, ICSE 2025: https://arxiv.org/abs/2308.05177
- [E17] CRUST-Bench, Apr 2025: https://arxiv.org/abs/2504.15254
- [E18] Type-constrained decoding, PLDI 2025: https://arxiv.org/abs/2504.09246
- [E19] Silva et al., resource-affected flaky tests, FSE 2025: https://arxiv.org/abs/2310.12132
- [E20] JavaScript flaky tests, 2022: https://arxiv.org/abs/2207.01047
- [E21] SAP HANA flakiness, Feb 2026: https://arxiv.org/abs/2602.03556
- [E22] Parry et al., systemic flakiness, EASE 2025: https://arxiv.org/abs/2504.16777
- [E23] SAP HANA, ESEM 2024: https://arxiv.org/abs/2409.10062
- [E24] etcd with Antithesis, 2025: https://etcd.io/blog/2025/autonomus_testing_with_antithesis/
- [E25] Rust blog, supply-chain attack on arrayref, 20 Aug 2026: https://blog.rust-lang.org/2026/08/20/supply-chain-attack-on-arrayref/
- [E26] Check Point, Shai-Hulud 2.0: https://blog.checkpoint.com/research/shai-hulud-2-0-inside-the-second-coming-the-most-aggressive-npm-supply-chain-attack-of-2025/
- [E27] Okta, the s1ngularity attack, Aug 2025: https://www.okta.com/blog/threat-intelligence/the-s1ngularity-attack--when-attackers-prompt-your-ai-agents-to/
- [E28] Spracklen et al., USENIX Security 2025: https://arxiv.org/abs/2406.10279
- [E29] Package-hallucination replication, May 2026: https://arxiv.org/abs/2605.17062
- [E30] CooperBench, Jan 2026: https://arxiv.org/abs/2601.13295
- [E31] Specification Gap, Mar 2026: https://arxiv.org/abs/2603.24284
- [E32] MACGen, Aug 2026: https://arxiv.org/pdf/2608.25457
- [E33] MAST, Mar 2025: https://arxiv.org/abs/2503.13657
- [E34] AgenticFlict, Apr 2026: https://arxiv.org/abs/2604.03551
- [E35] Xu et al., Jul 2026: https://arxiv.org/abs/2607.04697
