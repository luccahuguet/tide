# 13: What agent fleets need from a language

Research date: 2026-10-19. Scope: what people who run many coding agents and
subagents, in parallel and over long durations, need from a programming
language and its toolchain, and what that means for Tide (`design/01-design.md`,
draft 3.4). **Inference** marks reasoning that no source states.

Method: a Grok search of X (1 Jan 2025 to 19 Oct 2026) returned the posts
quoted in "Findings from X"; quotes are as Grok reported them, the posts
were not each reopened, and every figure in them is the author's claim. A
delegated web search of practitioner write-ups supplied "Practitioner
reports" (three primary sources returned 403 and are quoted through the
secondary source named); a second search of papers, benchmarks, and incident
reports supplied "Empirical evidence"; most 2026 items there are arXiv preprints, and figures
marked *secondary* were seen only in write-ups about a study. "Tooling
prior art" was added on 2026-11-09 (bead `fx-zmr`); its sources were fetched
directly that day, and a figure a project measured on itself is marked
self-reported.

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

## Practitioner reports (cited)

### P1. Fast mechanical feedback the agent cannot forge

- "give the agent fast, high quality tools to automatically tell it when it
  is wrong" [P01]. "Specific programming languages have inbuilt back
  pressure through their type system" [P02].
- Stripe's pre-push lints take "less than five seconds", and it runs "at
  most two rounds of CI" because CI costs "tokens, compute, and time" [P03].
  "Advice does not cause builds to fail, allowing drift to accumulate
  quietly" [P04].

### P2. Verification the agent cannot game

- Anthropic's long-running harness tells the agent "It is unacceptable to
  remove or edit tests" and keeps the feature list in JSON because the model
  "is less likely to inappropriately change or overwrite JSON files compared
  to Markdown" [P05].
- StrongDM keeps its scenarios "outside the codebase (similar to a 'holdout'
  set)" [P06]. "36% of commits made by coding agents add mocks to tests,
  compared with 26% by non-agents" [P07].

### P3. Review is the bottleneck

- "the natural bottleneck on all of this is how fast I can review the
  results" [P08]; "the pull request review clearly turns into the
  bottleneck", and "if input grows faster than throughput, you have an
  accumulating failure" [P09]; beyond 3–4 parallel sessions, "I'm the
  bottleneck" [P10].

### P4. State lives in files, not context

- "each new session begins with no memory of what came before";
  "compaction isn't sufficient" [P05]. Yegge treats compaction as "a failure
  mode" and keeps work in Beads, "a Git ledger of all of your work" [P11].

### P5. Isolation between parallel agents

- Building a C compiler with agents: "Merge conflicts are frequent", and on
  the Linux kernel "Every agent would hit the same bug, fix that bug, and
  then overwrite each other's changes" [P12]. With locking, twenty agents
  produced the throughput of two or three [P13].
- Hundreds of agents compiling at once produce "many GB/s reads and writes
  of build artifacts" [P14], and Cargo's shared `target/` lock blocks agents
  in separate worktrees [P15].

### P6. Sandboxing and supply chain

- Sandboxing "reduces permission prompts by 84%" and needs "both filesystem
  and network isolation" [P16]. "The only solution that's credible is to run
  coding agents in a sandbox" [P17].
- Agent platforms should ship with install scripts disabled; agents "should
  never regenerate a lockfile unless explicitly asked to"; new package
  versions should wait 24–72 hours before agents can install them [P18].

### P7. Tool output that is cheap and bounded

- "The test harness should not print thousands of useless bytes", and
  "Claude can't tell time and, left alone, will happily spend hours running
  tests" [P12]. "Tools need to be fast"; "Crashes are tolerable; hangs are
  problematic" [P19].

### P8. Failures must be attributable

- Agents "disavow work. They say, 'Someone completely unrelated to me broke
  the build'" [P11]. Cursor accepted a small constant error rate rather than
  a permanently green trunk, since "Even a single small error ... would
  cause the whole system to grind to a halt" [P14].

### P9. What practitioners say about languages

- Ronacher recommends Go: its context "explicitly flows through the code
  execution path"; "In Go, tests run straightforwardly and incrementally",
  while agents misread how to invoke `cargo test`; low churn "greatly
  reduces the likelihood of AI generating outdated code" [P19].
- His wish list for a language for agents [P20]: typed results, since
  "Agents struggle with exceptions"; greppable names, since "agents really
  like local reasoning"; no macros ("Agents often struggle with macros"); no
  import cycles; tests that are hard to make flaky; code that "either runs
  or doesn't"; no significant whitespace; and declared effects so tests can
  "precisely mock out these side effects" (a proposal, not an observation).
- "Fast compile-test cycles matter more than code simplicity and
  readability"; agents compile and test "one-to-two orders of magnitude more
  often" than humans [P21]. Rust gives "extreme correctness" but "the
  compilation speed is slow" [P02].
- Type-checker feedback raised success from 79.6% to 83.9% with "21% fewer
  steps" on well-typed code, and gave "no meaningful improvement" on lightly
  typed code (secondary) [P22]. "AI's ability to write code in a language is
  proportional to how much of that language it's seen" [P23]; Willison
  doubts that still holds [P24].

**Gaps.** No controlled fleet-scale language comparison; no practitioner
report of language-level effects or capabilities (every sandbox found works
at the OS level); no data that language-level determinism reduces flaky
tests.

**Inference.** Ronacher's list reads like Tide's design: `Result` and `?`
(D5), no user macros (D6), no module cycles (`FX0914`), braces (D1),
capabilities as declared effects that the simulator substitutes (D7, D15),
and `tide check` before anything runs. The gap that no one reports
language-level capabilities is both Tide's opening and its risk: the idea is
unproven in practice.

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

## Tooling prior art (cited, added 2026-11-09)

### T1. One compiler budget for the whole machine

- mbx is a build cache for Cargo projects with a machine-wide scheduler.
  Every compiler it starts takes a permit from one pool shared by all
  builds that use the same cache directory; CPU and predicted memory are
  budgeted separately; a compilation identical to one already running waits
  for it and restores its result; a dead process's permits return to the
  pool [T02]. Its README advises giving each build its own target
  directory, to avoid Cargo's directory lock (P5) [T01].
- Six overlapping check, Clippy, and test-compile jobs on one CI runner,
  from an empty store (self-reported; three trials, medians; a 15-crate
  workspace): 2m12s in turn; 1m7s in parallel with no scheduler, peaking at
  172 compilers; 54.7 s with the scheduler, peaking at 31 of 32 permits.
  Cache hits over the batch rose from 23 to 3,550, because the scheduler
  holds identical compilations until the first finishes. A second project
  gave 50.7 s, 48.2 s with 162 compilers, and 39.2 s with 32 [T03].
- kache, which mbx credits as its design's inspiration [T01], has the same
  two parts: a miss "joins a machine-wide flight, takes a memory-weighted
  permit and a per-key lock" [T04]. Its vendor names "fleets of agents that
  each want their own checkout" as a use it builds for, and reports that a
  second Firefox worktree on a reflink filesystem added about 3 GB of new
  data, against 16.6 GB for its build directory on paper (self-reported, one
  run) [T05]. In v2.0.0, `kache test-runner` takes test slots from the
  compiler scheduler's pool and sets `RUST_TEST_THREADS` to the slots it
  holds when no thread count was supplied. On Unix, `kache init` enables
  this unless passed `--no-shell`; the runner can time out and run without
  slots [T14].
- `research/03-compile-speed.md` §11 has both tools' cache design, their
  other numbers, and their limits.

**Inference.** Need 10 combines concurrent cache reuse with one machine-wide
compiler budget. A cache that concurrent agents can share removes repeated
work. It does not stop N agents from each starting a full set of compilers: without
a scheduler, six jobs ran 162 to 172 compilers on a machine with 32
permits. A fleet needs one budget for the machine, held by whatever starts
the compilers. Under D18 that is Tide's driver, which owns every rustc
invocation. Kache's test runner is prior art for extending that budget to
tests [T14]. How Tide's tests and simulations draw on it remains open.

### T2. Copy-on-write checkouts, and an overlay over a live tree

- ovnfs gives each agent an "isolated, copy-on-write" view of a folder
  without copying it. One daemon serves every checkout as a folder of a
  single mount: FUSE on Linux, NFSv3 on macOS, SMB or WinFsp on Windows,
  with no root. Writes go to the checkout's own layer; `diff` shows them and
  a three-way `apply` merges them back. Build outputs and ignored files show
  the live folder until the checkout writes them [T06]. The repository was
  four days old when fetched (created 2026-11-05, v0.3.0 on 2026-11-08).
- Its stated limits: it "isolates files, not processes: it is not a
  sandbox" [T06], and "the view is best effort, not a frozen snapshot: a
  source file the user rewrites while the agent is reading it can show
  either version" [T07].
- Its figures are self-reported, with no hardware named: opening a checkout
  of a 49,000-file repository takes "about a second" and `git status` inside
  it "under three" [T08]. The repository has a script that compares Cargo
  build times in a git worktree, an ovnfs checkout, and a copy-on-write
  clone, but publishes no results [T09], so what a FUSE mount costs a build
  is not known.
- The failure that motivates snapshots is documented in oh-my-pi issue
  #4627 (5 July 2026). That agent's OverlayFS backend used the live,
  writable repository as the overlay's lower layer. By the reporter's
  analysis, copy-up reads the lower file in chunks without a lock, so a
  parent's concurrent write left the child a file that was part old and
  part new, with no error; the reporter reproduced it in 500 of 500 runs on
  fuse-overlayfs (the reporter's figure). Git's `index.lock` did not exclude
  across layers either, so parent and child could both hold it [T10]. The
  kernel documentation says that if the
  underlying filesystem changes while it is part of a mounted overlay, "the
  behavior of the overlay is undefined" [T11].
- ovnfs did not replace that backend. The maintainer closed the issue the
  same day ("I don't really see this as an issue"), and the pull request
  removing OverlayFS was closed unmerged [T10]; the backend is still listed
  in oh-my-pi's README [T13]. Since version 18.17.0
  (2026-11-06) oh-my-pi gives isolated subagents ovnfs checkouts when it
  finds the ovnfs binary and the backend setting is `auto` [T12].

**Inference.** For need 3 a per-agent checkout needs a point-in-time source
base. ovnfs's default live-tree view lacks a frozen-snapshot guarantee
[T07], while `ovnfs new --rev REV` starts from a revision's tracked files
[T08]. The same live-tree hazard reaches a build cache under a fleet. A
parent agent may edit a file while a child's build reads it, so a content-addressed
cache stores a wrong artifact under a right-looking key unless the key is
computed from the bytes the compiler read. D18's driver snapshots the
manifest with the sources, and [RUN-2](../design/03-authority-concurrency.md)
serves rustc's source reads through the driver's file loader, where the
driver can bind the key to the bytes it serves. A cache outside the tree
also makes the checkout mechanism irrelevant to build
reuse, provided keys do not depend on the checkout's path (research/03
§11). Restoring by reflink or hard link needs the store and the target on
one filesystem, and a FUSE checkout is a different one, so outputs restored
into such a checkout would be copies; this was not tested.

---

## Needs, ranked, and where Tide stands

| # | Need | Evidence | Tide today | Gap |
| --- | --- | --- | --- | --- |
| 1 | Review that fits human attention | X1, P3, E1 | The trust diff (D11) reports what changed in what code may do: authority, contracts, `unsafe` | E3 must show it shrinks review of agent PRs; nothing yet presents it per PR |
| 2 | Tests and specs the agent cannot weaken | X6, P2, E1 | `#[protected]` tests (D14); contracts in the trust surface (D11) | Signatures, contracts, and grants are not protected; a weakened `ensures` is not flagged as such |
| 3 | Parallel work without collisions | X2, P5, E7, T2 | Interface artifacts; a body edit never rechecks dependents (D12); content-hashed definitions and stable IDs (D14); no globals (D7) | No first-class delegation boundary: "this interface is sealed, implement behind it" lives only in prompts |
| 4 | Validation per change, off the shared queue | X3, X8, P6, E6 | Nothing runs at build time; capabilities bound what tests can touch (D7, D13); simulated time (D15) | "Safe to run an unreviewed branch's tests without a VM" is implied, not stated or tested |
| 5 | A fast, strict check loop | X7, P1, P9, E4 | Targets of tens of milliseconds per edit (D12) | Unproven until a checker exists; lust shows the target is reachable on a Rust subset (research/12) |
| 6 | Low cost per spawn | X4, E2 | A 25,000-token reference budget (D14); locality (D7) | The reference is paid again by every fresh subagent; no measure of tokens per task |
| 7 | State that survives compaction | X5, P4, E3 | Seeds and replay commands (D15), snapshot hashes, stable diagnostic codes (D14), rule IDs (D0) make state compact to hand off | None at the language level |
| 8 | Stable suites | X6, P9, E5 | Deterministic simulation: authority-routed nondeterminism cannot flake, and virtual time makes results independent of machine load (D15) | Undersold: the pitch never says that tests do not flake under 20 parallel agents |
| 9 | Bounded runs and attributable failures | P7, P8 | The simulator counts steps (D15); every failure carries a seed and a replay command (D14) | Promise that every `tide` command is bounded in time and output, and that a failure names the change and seed that caused it |
| 10 | Builds that many agents can share | P5, T1 | Content-hashed definitions and cached interface artifacts (D12, D14) | State that the build cache is safe for concurrent agents across worktrees, unlike Cargo's `target/` lock, and give the driver one compiler budget for the machine (T1) |

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
   **Inference.** This complements microVMs rather than replacing them. An
   agent runs arbitrary shell commands, so its session still needs a
   microVM or OS confinement. A microVM, though, does not protect the
   credentials inside it (E6's s1ngularity attack read local secrets),
   gives the reviewer no evidence, and is gone in production. Tide moves
   validation out of the VM: tests and seeded simulations run in-process
   with only granted capabilities and virtual time, so a fleet needs one VM
   per agent session, not one per test run, and kernel enforcement gives
   the deployed process its own OS sandbox.
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

Tooling sources (fetched directly, 2026-11-09):

- [T01] mbx (Mr. Boxington) README: https://github.com/jdx/mr-boxington
- [T02] mbx, how it works (machine-wide scheduling): https://mr-boxington.jdx.dev/how-it-works
- [T03] mbx benchmarks (self-reported): https://mr-boxington.jdx.dev/benchmarks
- [T04] kache architecture: https://kunobi.ninja/docs/kache/how-it-works/architecture
- [T05] kache storage report (self-reported): https://kunobi.ninja/blog/kache-storage-worktrees
- [T06] ovnfs README: https://github.com/stencil-hq/ovnfs
- [T07] ovnfs limitations: https://github.com/stencil-hq/ovnfs/blob/main/docs/limitations.md
- [T08] ovnfs checkouts: https://github.com/stencil-hq/ovnfs/blob/main/docs/checkouts.md
- [T09] ovnfs development (benchmark script): https://github.com/stencil-hq/ovnfs/blob/main/docs/development.md
- [T10] oh-my-pi issue #4627, 5 Jul 2026: https://github.com/can1357/oh-my-pi/issues/4627
- [T11] Linux kernel, overlay filesystem: https://docs.kernel.org/filesystems/overlayfs.html
- [T12] oh-my-pi coding-agent changelog, 18.17.0: https://github.com/can1357/oh-my-pi/blob/main/packages/coding-agent/CHANGELOG.md
- [T13] oh-my-pi README (pi-iso backends): https://github.com/can1357/oh-my-pi
- [T14] kache v2.0.0 configuration, test runner: https://github.com/kunobi-ninja/kache/blob/v2.0.0/docs/getting-started/configuration.mdx#test-runner

Practitioner sources (delegated web search, 2026-10-19):

- [P01] Mitchell Hashimoto, 5 Feb 2026: https://mitchellh.com/writing/my-ai-adoption-journey
- [P02] Geoffrey Huntley, Ralph, 14 Jul 2025: https://ghuntley.com/ralph/
- [P03] Stripe, Minions, 9 Feb 2026: https://stripe.dev/blog/minions-stripes-one-shot-end-to-end-coding-agents
- [P04] Factory, linters to direct agents, 5 Sep 2025: https://factory.com/using-linters-to-direct-agents
- [P05] Anthropic, effective harnesses for long-running agents, 26 Nov 2025: https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents
- [P06] Simon Willison on StrongDM's software factory, 7 Feb 2026: https://simonwillison.net/2026/Feb/7/software-factory/
- [P07] Hora and Robbes, Jan 2026: https://arxiv.org/abs/2602.00409
- [P08] Simon Willison, parallel coding agents, 5 Oct 2025: https://simonwillison.net/2025/Oct/5/parallel-coding-agents/
- [P09] Armin Ronacher, the final bottleneck, 13 Feb 2026: https://lucumr.pocoo.org/2026/2/13/the-final-bottleneck/
- [P10] Charlie Guo, harness engineering, 22 Feb 2026 (also quotes OpenAI's "Harness engineering", which returned 403): https://www.ignorance.ai/p/the-emerging-harness-engineering
- [P11] Steve Yegge on Software Engineering Daily, 12 Feb 2026: https://softwareengineeringdaily.com/wp-content/uploads/2026/02/SED1908-Steve-Yegge.txt
- [P12] Nicholas Carlini, building a C compiler, 5 Feb 2026: https://www.anthropic.com/engineering/building-c-compiler
- [P13] Cursor, scaling agents, 14 Jan 2026: https://cursor.com/blog/scaling-agents
- [P14] Cursor, self-driving codebases, 5 Feb 2026: https://cursor.com/blog/self-driving-codebases
- [P15] Rust forum, cargo-worktree, 26 Mar 2026: https://users.rust-lang.org/t/announcements-cargo-worktree-fix-build-isolation-in-git-worktrees/139192
- [P16] Anthropic, Claude Code sandboxing, 20 Oct 2025: https://www.anthropic.com/engineering/claude-code-sandboxing
- [P17] Simon Willison, living dangerously, 22 Oct 2025: https://simonwillison.net/2025/Oct/22/living-dangerously-with-claude/
- [P18] Andrew Nesbitt, package security for AI agents, 9 Apr 2026: https://nesbitt.io/2026/04/09/package-security-defenses-for-ai-agents
- [P19] Armin Ronacher, agentic coding, 12 Jun 2025: https://lucumr.pocoo.org/2025/6/12/agentic-coding/
- [P20] Armin Ronacher, a language for agents, 9 Feb 2026: https://lucumr.pocoo.org/2026/2/9/a-language-for-agents/
- [P21] Wes McKinney, agent ergonomics, 20 Jan 2026: https://wesmckinney.com/blog/agent-ergonomics/
- [P22] PyCon US 2026 typing summit recap, 14 May 2026: https://bernat.tech/posts/pycon-us-2026-typing-summit-recap/
- [P23] GitHub, Anders Hejlsberg interview, 6 Nov 2025: https://github.blog/developer-skills/programming-languages-and-frameworks/typescripts-rise-in-the-ai-era-insights-from-lead-architect-anders-hejlsberg
- [P24] Simon Willison, not so boring, 9 Mar 2026: https://simonwillison.net/2026/Mar/9/not-so-boring/
