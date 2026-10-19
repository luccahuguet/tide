# 13: What agent fleets need from a language

Research date: 2026-10-19. Scope: what people who run many coding agents and
subagents, in parallel and over long durations, need from a programming
language and its toolchain, and what that means for Tide (`design/01-design.md`,
draft 3.4). **Inference** marks reasoning that no source states.

Method: a Grok search of X (1 Jan 2025 to 19 Oct 2026) returned the posts
quoted in the findings below. Quotes are as Grok reported them; the posts
were not each reopened, and every figure in them is the author's claim. Web
practitioner reports and empirical studies are added in the sections marked
as such when the delegated searches return.

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

## Needs, ranked, and where Tide stands

| # | Need | Evidence | Tide today | Gap |
| --- | --- | --- | --- | --- |
| 1 | Review that fits human attention | X1 | The trust diff (D11) reports what changed in what code may do: authority, contracts, `unsafe` | E3 must show it shrinks review of agent PRs; nothing yet presents it per PR |
| 2 | Tests and specs the agent cannot weaken | X6 | `#[protected]` tests (D14); contracts in the trust surface (D11) | Signatures, contracts, and grants are not protected; a weakened `ensures` is not flagged as such |
| 3 | Parallel work without collisions | X2 | Interface artifacts; a body edit never rechecks dependents (D12); content-hashed definitions and stable IDs (D14); no globals (D7) | No first-class delegation boundary: "this interface is sealed, implement behind it" lives only in prompts |
| 4 | Validation per change, off the shared queue | X3, X8 | Nothing runs at build time; capabilities bound what tests can touch (D7, D13); simulated time (D15) | "Safe to run an unreviewed branch's tests without a VM" is implied, not stated or tested |
| 5 | A fast, strict check loop | X7 | Targets of tens of milliseconds per edit (D12) | Unproven until a checker exists; lust shows the target is reachable on a Rust subset (research/12) |
| 6 | Low cost per spawn | X4 | A 25,000-token reference budget (D14); locality (D7) | The reference is paid again by every fresh subagent; no measure of tokens per task |
| 7 | State that survives compaction | X5 | Seeds and replay commands (D15), snapshot hashes, stable diagnostic codes (D14), rule IDs (D0) make state compact to hand off | None at the language level |
| 8 | Stable suites | X6 | Deterministic simulation: authority-routed nondeterminism cannot flake, and virtual time makes results independent of machine load (D15) | Undersold: the pitch never says that tests do not flake under 20 parallel agents |

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
   trust diff reports attempts. This answers needs 2 and 3 together.
4. **State the sandbox guarantee.** Running `tide check`, `tide test`, and
   `tide sim` on an unreviewed branch can reach only the holes the trust
   surface lists (`unsafe`, `ffi` grants). Say so, test it, and pair it with
   the kernel enforcement adopted from research/12.
5. **Measure the context tax.** E1 should record the tokens each writer
   spends on the reference and the task, not only pass rates.
6. **Sell determinism to fleets.** Tests that cannot flake under machine
   load answer X6 and X3 directly.

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
