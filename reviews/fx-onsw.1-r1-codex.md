Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate c291b19 (research note)

## Verdict

**approve**. No blocking finding. Confidence: high for this research-only candidate's factual alignment and scope; this pass supplies no implementation, performance, or soundness proof. The subsequent Opus review remains required.

## Scope

Read all 400 lines of `research/45-bend-hvm-ideas-for-tide.md`, the candidate diff, AGENTS.md, GOVERNANCE.md, the bead and parent, the bead's comments, partner notes and research report. Compared the affected Tide claims with D2/D7/D8/D9/D10/D15/D16/D17/D18, the core reference, authority/concurrency chapter, admission owners, capability declarations, parallel runtime and cited conformance fixtures. Consulted prior research/01, /11 and /31 for context.

Candidate: `c291b19fabd47f63085e3cf67129eaaca9c93ab2`, branch `fx-onsw.1`; note SHA-256 `0f15284f075e2b1cb83fb24dcbd4144c42719925155395231257b9ba0c593e6d`. Main at inspection: `c19b1a10f98060cc515be13010820d7803929d6e`. The design directory and inspected implementation/conformance surfaces have no diff against that main. The commit adds only research/45. Read fx-onsw.2-.7 from main's committed ledger with git because this worktree's older ledger predates their filing; made no ledger writes.

Inspected saved Lafont, Watrous, Mazza and DPOR text, Silq HTML, and the saved Kahn page images (the Kahn .txt contains no usable text). H1/H2/H3/B1/B2/K2/Q1 were not saved in target/work; filled those gaps by reading the note's pinned public primary sources. No builds, tests, installs, commits, agent launches or messages to people. Only this report was written.

## Blocking findings

None. No minimal corrective patch is required for this pass.

## Non-blocking findings

1. **Expand abbreviations and introduce terms earlier.** `research/45-bend-hvm-ideas-for-tide.md:6`, `:51`, `:231`, `:277`. SUP appears before its explanation in §2, IC is first used without an expansion, and DPOR never expands to “dynamic partial-order reduction.” HVM likewise stays unexpanded. SPSC is spelled out at its substantive introduction, which is good. Introduce SUP as a superposition node on first use, expand IC and DPOR there, and give HVM's name once. These are readability fixes; none changes the described semantics.

## Reasoning

### Citation spot-checks

The following are individual claim/citation checks, including more than ten against the saved artifacts. “Matches” means the cited passage supports the claim at the stated level, including its qualifications.

| Note location / citation | Source passage inspected | Result |
| --- | --- | --- |
| 29–31, L1 | Saved Lafont §1.1: ports, wiring and one principal port per cell | Matches. Distinct active pairs cannot share a cell. |
| 31–33, L1 | Saved Lafont §2.1: constructor, duplicator, eraser | Matches; copying and disposal are graph operations, not forbidden operations. |
| 149–150, L1 | Saved Lafont §1.2: rule uniqueness and preserved free-port interface | Matches. |
| 151–153, L1 | Saved Lafont Proposition 1 and its disjoint-pair proof | Matches the one-step diamond for distinct reducts. |
| 154–156, L1 | Saved Lafont consequence immediately after Proposition 1; §2.1's nontermination example | Matches: equal reduction length is conditional on reaching an irreducible result; no universal termination claim. |
| 84–85, S1 | Saved Silq HTML, Annotations / Classical types | Matches: classical values admit duplication and dropping. The saved HTML hash equals the note's recorded hash. |
| 85–89, S1 | Saved Silq HTML, Other Functions / dup | Matches the basis-value map and distinction from cloning an unknown state. The superposed-input equation is correctly labeled inference. |
| 122–123 and 348–352, Q2 | Saved Watrous §2.1.2, equations (2.12)–(2.13) | Matches complex unit-vector representation of pure states, modulo global phase. |
| 122–130, Q2 | Saved Watrous §2.3.1, Definition 2.34 and measurement-probability formula | Matches. Relative-phase dependence follows from these formulas; the note identifies that derivation as inference. |
| 221–224, K1 | Saved Kahn scan page 1, §1 | Matches FIFO communication, blocking waits and nonblocking sends. The language offers no readiness test. |
| 249–257, K1 | Saved Kahn scan pages 1 and 3, §§1/3 | Matches the unbounded-send model and fixed-point account; bounded-write backpressure is a limitation of the proposed Tide transfer, not a claim about Kahn's model. |
| 269–274 and 365–369, A1 | Saved Mazza thesis introduction's chapter-4 synopsis, printed p.13 | Matches the two-principal-port amb construction and its attribution to Khalil. The Tide mapping is explicitly an analogy. |
| 276–278, P1 | Saved DPOR abstract/introduction | Matches dynamic exploration of representative interleavings. |
| 280–284, P1 | Saved DPOR §3 Definition 1 | Matches both enabledness preservation and commutation; distinct handle identity alone is correctly rejected as sufficient evidence. |

Additional direct checks filled the saved-source gaps:

| Note location / citation | Primary source inspected | Result |
| --- | --- | --- |
| 35–40, H1 | [Pinned HVM2 paper](https://raw.githubusercontent.com/HigherOrderCO/HVM2/654276018084b8f44a22b562dd68ab18583bfb5b/paper/HVM2.typst), Syntax / Interpretation / Call / Commute | Matches paired wire variables, incremental copying, and static-definition expansion. |
| 46–55 and 158–163, H1 | Same paper, Only one Duplicator Node / Ultra-Eager Evaluation Only / Massively Parallel Evaluation | Matches the translation restriction, eager evaluator and claimed confluence/atomic-linking mechanism. The note does not promote those claims into a proof of arbitrary source translations. |
| 42–49 and 110–115, H2 | [Pinned HVM3 README](https://github.com/HigherOrderCO/HVM3/blob/fba2e9c82faf6e2f019c9ecea94c32f19a8b7820/README.md) | Matches affine binders and first-class duplication/superposition; historical compiler-target intentions stay qualified. |
| 51–55, 110–120 and 302, H3 | [Pinned IC specification](https://github.com/VictorTaelin/Interaction-Calculus/blob/e9f0185682ba8b41205691efd4f58491dc405a53/README.md), Specification / Collapsing | Matches weak-head evaluation, APP-SUP distribution, same-label projection, different-label commutation, erasure and global substitution. The rules provide no amplitudes or Born measurement. |
| 57–61 and 299/303, B1 | [Pinned BendRT paper](https://raw.githubusercontent.com/bendlang/bend/f0cf0a7c573145501981c44d9c33e326185f008e/bend2/docs/BendRT/main.typ), Introduction / Program and Its Contract / Compilation | Matches affine live values, Data-only reusable binders, source-copy restriction, compiler sharing/borrowing, explicit forks, equal-work expectation and absence of interaction nets. |
| 57–59 and 300, B2 | [Pinned Bend guide](https://github.com/bendlang/bend/blob/f0cf0a7c573145501981c44d9c33e326185f008e/guide/GUIDE.md), Quantities / Laws and Proofs | Matches + binders and the unproved translation into the checking kernel. |
| 74–82, Q1 | [Wootters–Zurek scan](https://painterlab.caltech.edu/wp-content/uploads/2019/06/qe_wotters_no_cloning.pdf), pp.802–803, superposed-input argument | Matches the universal-cloning prohibition; the note does not conflate use-count linearity with linear quantum evolution. |
| 224–227 and 247, K2 | [TR702 October 2012 revision](https://scholarworks.iu.edu/dspace/bitstreams/51e17b47-c0e4-4ad5-848c-aef0b72bfb67/download), abstract/introduction and §2.1 | Matches monotone updates plus restricted threshold reads. The note identifies the particular model rather than asserting a guarantee for every later LVars extension. |

### Tide facts, recommendations and disagreement

The capability correction at lines 63–72 agrees with reference §6.1 and root.rs's Clone derive. D7's structural Pure rule excludes authority and synchronization handles; Pure alone does not imply Sync. D9 supplies the successful-result/sequential-equivalence contract, disjoint mutable captures and ordered sinks. The note's partial-runtime qualification agrees with par.rs and chapter 03 §8.4. Simulation/replay are described as contracts with partial implementation, not completed universal coverage. The v1 label for par_reduce is correct.

D16 remains Later with its existing backend boundary. The recommendations retain present moves, Clone, admission and fork ownership; they reject additions without a demonstrated Tide invariant. They authorize no v0 language, reference, std or simulator implementation change. Later options have fit/cost/alpha ratings, caller or evidence gates and concrete falsifiers. No benchmark or quantum advantage is asserted, and the tone contains no judgment of the other projects' authors.

“D3-b” here is the partner's decision-table identifier, not a Tide D3 decision. The partner recommends later deterministic task-scope research and claims that it could remove seed exploration and schedule recording, while acknowledging schedule-dependent error identity. Lines 229–265 retain the recommendation and explain why the researcher disputes that stronger payoff: bounded sends, cancellation, faults and progress remain observable. Both sides' substantive positions survive. The qualification is justified; agreement on successful output does not prove schedule independence for failures or liveness. Fixed-order fan-in over separate SPSC channels is also correctly distinguished from arrival-order merging.

### Follow-ups and the next round

The note agrees with the filed children: .2 restricted pipeline research; .3 a separate LVars alternative after .2; .4 simulator reduction with an independence argument and exhaustive tiny-model comparison; .5 observe existing closure failures before a lint; .6 a concrete caller before a consumable-resource design; .7 a public workload and ordinary enumerator baseline before shared-alternative search. The parent receives quantum API questions. All remain post-alpha/deferred and add no RC work.

The bead's later owner direction assigns the SUP usefulness reframe and evaluation of shared-prefix seed exploration to the upcoming Opus round. Those are pending work for that round, not conclusions already present in c291b19. In particular, this approval does not approve a shared-prefix simulator design or claim that it subsumes DPOR. Any changed conclusion must be reconciled with the affected filed beads as that direction requires.

Limits: no runtime behavior, benchmark or formal theorem was rerun or proved. The next Opus round still decides the recommendations and handles the recorded owner direction.

DONE
