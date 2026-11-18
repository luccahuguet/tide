# 45 — Bend/HVM ideas for Tide

The useful connection is control over duplication and interference. Tide already
has moves, explicit `clone`, structural authority checks and deterministic
parallel regions. HVM, the Higher-order Virtual Machine, supplies a different
explanation for those boundaries, not evidence that Tide needs graph reduction.
A superposition node (SUP) is worth something to Tide as a classical idea,
sharing work across alternatives until they differ. It applies to simulator
exploration and to search tooling, not to the language. Its quantum name is a
name-level analogy. Retain Tide's checked parallel fragment; investigate
deterministic task pipelines after the alpha. [T1], [T2], [T3], [H1], [H2],
[H3], [Q2].

Research date: **2026-11-18**. Bead: **fx-onsw.1**. Codex research, incorporating
the independent Opus partner's borrow/reject/later decisions on 2026-11-18.
Reviewed under the Merge Rule (Codex r1; Opus review round 2026-11-18, which
decided the §3 D3-b disagreement).

**Evidence boundary.** This is source research, not an implementation, benchmark,
soundness proof or quantum-domain proposal. External claims below describe the
identified artifacts, not an assertion about the latest release. Sources give
versions/commits and dates read; **Inference** labels Tide proposals and cost
judgments. Reports [01](01-bend-hvm.md), [11](11-bend-interop.md) and
[31](31-bend-update.md) supply the prior context, including report 31's corrections
to older interop claims. [D16](../design/01-design.md#d16-bend-tandem) stays
unchanged, in Later. Only this research note changes.

## 1. Linearity, explicit DUP and no-cloning

### Evidence

Lafont's interaction nets connect ports by wires and reduce active pairs at
principal ports. Each agent has one principal port, so distinct active pairs
cannot overlap in an agent. His interaction combinators include a duplicator
and an eraser. This is controlled copying and disposal within a graph formalism,
not a prohibition on copying. [L1]

HVM2's paper requires each wire variable to occur twice, denoting the two ends
of one wire; it represents duplication with a `DUP` node. Its commutation rule
propagates copying through a graph. It also copies static definitions through
`REF` expansion. Thus “nothing is copied implicitly” is accurate only after
specifying the representation and its rewrite rules. It does not establish that
a surface-language author writes a DUP for each copy. [H1] (Syntax, Call, Commute)

HVM3's README and the Interaction Calculus specification use affine binders:
a variable occurs at most once, with first-class duplication and superposition.
The explicit duplication construct makes two uses possible. Affine usage permits
discarding; an exactly-once linear resource would impose an additional obligation.
The HVM2 paper also warns that its single duplicator cannot soundly translate
arbitrary nested higher-order copying without a source restriction or extra
bookkeeping. Do not combine the different HVM generations into one guarantee.
[H1] (Only one Duplicator Node), [H2], [H3] (Specification)

The Interaction Calculus (IC) specification gives weak-head evaluation and
incremental DUP rules; those describe demand-driven expansion in that
evaluator. HVM2 instead reduces available redexes eagerly. “Lazy DUP” should
therefore name the calculus and evaluator, not imply one universal HVM strategy.
[H1] (Ultra-Eager Evaluation Only), [H3] (Specification)

Bend 2 provides a surface-level precedent: `+x` permits multiple uses when
`x` has kind `Data`, while ordinary live binders are affine. This marks
permission to copy at the binder; it does not spell each copy as a DUP. BendRT
states that the compiler may remove a source copy but may not add one. It also
states that its runtime no longer uses interaction nets. [B1], [B2]

Tide's [D2](../design/01-design.md#d2-memory-ownership-with-rusts-lifetimes-checked-as-origins)
keeps Rust ownership: a non-`Copy` value moves, and explicit `Clone` can produce
another value where its type permits it. **Correction to the brief's premise:**
capabilities in [reference §6.1](../design/reference.md#61-the-rule-and-the-capability-set)
are non-`Copy`, provisionally **`Clone` with the same authority**, and
`Send + Sync`. The [capability macro](../crates/tide-std/src/root.rs) derives
`Clone`; a [parallel rejection case](../conformance/cases/tasks/par_join_reject.rs)
even clones a clock before rejecting its use in `join`. `Root` has stricter
entry-only rules. Tide does not have a general “non-Clone capabilities” contract.
[T1], [T2], [T3]

The no-cloning theorem excludes a universal physical operation that makes two
independent copies of an arbitrary unknown quantum state. Wootters and Zurek's
proof derives the conflict from linear evolution of superposed inputs. [Q1]
**Inference:** this is a useful analogy for an opaque resource handle that the
program cannot duplicate, but a different meaning of “linear” from use-count
linearity. An HVM DUP copies classical terms; adding an explicit spelling does
not make unknown-state cloning physically possible. Refusing `Clone` on a
handle would prevent one source-level aliasing route, not validate a quantum
backend or its resource API. [H1], [H3], [Q1]

Silq makes the distinction concrete: `!τ` marks classical values that may be
duplicated and dropped. Its `dup` maps basis values `|v⟩` to `|v,v⟩` and the
documentation states that this respects no-cloning. **Inference:** extending
that map linearly gives `α|00⟩ + β|11⟩` for a superposed input, rather than
two independent copies of `α|0⟩ + β|1⟩`. Explicit duplication and physical
state cloning remain different operations. [S1], [Q1]

### Options and recommendation (inference)

| Option | Recommendation and reason | Fit | Cost | Alpha relevance |
| --- | --- | --- | --- | --- |
| Keep moves, `Copy`, `clone`, borrowing and explicit attenuation | **Borrow the explanation; retain current rules.** Distinguish copying data, duplicating authority and sharing a borrow. Existing syntax already exposes nontrivial duplication. | High | Small | No; existing contract |
| Add a DUP keyword or require explicit copies of `Copy` data | **Reject.** Adds a second copy mechanism and breaks Rust familiarity without a demonstrated invariant. | Low | Large | No |
| Make all capabilities non-`Clone` | **Reject as a blanket change.** Borrowing `&Fs` still permits repeated use and delegation; removing `Clone` would not make authority single-use. It would change a separate capability policy. | Low | Medium | No |
| Use an opaque non-`Copy`, non-`Clone` type for a genuinely consumable resource | **Later, only with a caller and resource contract.** Move-based APIs can express consumption; preventing borrowed duplication and hidden aliases belongs to the resource implementation. This adds no general language feature. | High | Medium | No |

The new lesson is to ask **who may duplicate which resource**, including through
borrows and methods. It is not a reason to make existing Tide capabilities
single-use. A future single-use resource's cheapest falsifier would try to use
it twice, clone it, recreate it from a raw identifier, and retain an alias across
its consuming operation; a negative `Clone` test alone would be insufficient.

## 2. SUP: is it worth anything for Tide?

### Evidence

In the Interaction Calculus, `&L{a,b}` is a labeled superposition node (SUP).
Application of a SUP of functions distributes application over its alternatives
while inserting a DUP for the argument. A DUP meeting a SUP with the same label
projects its branches; different labels commute and propagate structure. These
are specified classical rewrite operations. HVM3 describes first-class
superpositions as storing two terms in one location. [H2], [H3] (Specification)

A SUP is a labeled pair whose application distributes over its branches;
matching labels correlate projections, while different labels commute. The
collapse extension exposes a tree of alternatives. None of these cited rules
specifies complex amplitudes, relative phase or Born-rule measurement. [H3] (Specification, Collapsing)

### What Tide could take from it (inference)

Under the cited rules, SUP's working idea is classical: hold several
alternatives in one place, share every part they have in common, and pay
separately only where they differ. Labels record which choices are correlated.
[H2], [H3] That idea is worth something to Tide, but in tooling, not in the
language:

1. **Exploring alternative executions in `tide sim`.** The simulator could
   share one execution's prefix and branch only at its choice points. D17
   already records this direction as Later ("Snapshot and branch").
   §3 rates the concrete form, superposed seed exploration. [T1] (D17)
2. **Search over candidate values or programs.** This applies only behind a
   conventional interface and only for a real workload (fx-onsw.7).

It is not worth adding to Tide itself. First-class SUP, DUP and collapse would
give every reader a second evaluation model, and the number of alternatives a
representation holds says nothing about speed.

**Side note: the quantum analogy.** The shared name is a name-level analogy.
Quantum states use complex amplitudes and measurement probabilities, and
relative phase can change outcomes through interference. [Q2] The cited SUP
rules specify none of these. [H3] One resemblance is structural: a linear
operator distributes over a sum of states, much as application distributes
over a SUP's branches. That resemblance supplies no amplitudes and no
interference. A classical evaluator could encode quantum mathematics only
with an added state, gate and measurement model, which SUP does not provide.

### Options and recommendation (inference)

| Option | Recommendation and reason | Fit | Cost | Alpha relevance |
| --- | --- | --- | --- | --- |
| Share execution prefixes across alternative runs in `tide sim` | **Later, under D17's snapshot and branch.** The best use of the idea; rated in §3 with dynamic partial-order reduction (DPOR). No language surface. | Medium | Large | No |
| Investigate shared alternatives in an external pure search tool | **Later, conditional on a real search problem.** Compare a normal enumerator with deduplication before an HVM engine, and keep the engine outside Tide semantics. | Medium | Medium | No |
| Add first-class SUP/DUP and collapse to Tide expressions | **Reject for Tide's core.** Labels, distribution and collapse add another evaluation model for all readers. | Low | Large | No |
| Treat SUP as quantum superposition or a quantum speedup | **Reject (side note).** The cited semantics supplies neither interference nor measurement. | Low | Large | No |

A later comparison should return the same candidate set under the same budget,
including enumeration and decoding costs. Inputs with abundant common structure
and inputs with little sharing would test the proposed benefit. No speed claim
follows from the word “superposition” or from counting represented alternatives.

## 3. Confluence, race freedom and seeded replay

### Evidence: three different properties

Lafont permits at most one rule per pair of agent symbols, and a local
replacement preserves the interface. With those restrictions, distinct active
pairs do not compete for a node: reducing either leaves the other available,
and the two paths rejoin in one further reduction each. For distinct one-step
reducts `c1` and `c2` of `c`, there is a `d` with `c1 → d` and `c2 → d`.
If a net has an irreducible result, every reduction reaches it in the same
number of abstract interactions. This bounds neither peak memory nor wall-clock
balance and does not assert that an arbitrary net terminates. [L1] (§1)

The HVM2 paper claims strong confluence for its extended interaction rules and
uses atomic linking to implement concurrent rewrites. It separately documents
higher-order translation restrictions and eager evaluation of all available
redexes. This is evidence about a specified evaluator, not a proof that any
source language, effect extension or implementation inherits the result.
[H1] (Interaction Table, Linker, Limitations)

Tide separates three obligations:

| Obligation | Existing Tide owner | What remains outside the claim |
| --- | --- | --- |
| Avoid unsafe concurrent memory access | Rust ownership, borrowing, `Send` and `Sync` under D2 | Synchronized code can still have order-dependent application results. |
| Make a parallel region independent of scheduling | D7/D9: `Pure`, disjoint mutable access, ordered sinks and specified panic/grouping rules | Concurrent I/O, channel protocols, cancellation and shared-cell coordination. |
| Reproduce a particular effectful execution | D15: seeded scheduling and modeled inputs, replay identity and event trace | Proof over all seeds, or unmodeled foreign effects. |

These are Tide's stated contracts, with implementation qualifications below.
A seed selects one permitted execution of effectful concurrent code. Repeating
it reproduces that execution; sampling seeds does not prove that all executions
agree. A confluent computation instead has an order-independent answer within
its formal model. [T1] (D2, D7, D9, D15), [T2] (§§6.6, 6.8, 8)

### Tide already checks the useful fragment

D9 already specifies library `join`, `par_map` and `par_for`, with no authority
reachable through closures, shared captures or elements. `Pure + Send` permits
owned inputs; `Pure + Sync` permits shared reads; disjoint `&mut` splits permit
parallel local mutation. `Pure` alone is insufficient: `Cell` and `RefCell` are
`Pure` but lack `Sync`. Capabilities, channels, `Shared`, atomics and `Arc` are
not `Pure`. [T1] (D7/D9), [T2] (§§6.6, 8)

This has an admission owner today. The driver's [structural purity analysis](../crates/tide-admit/src/pure.rs)
and [`parallel_closure`](../crates/tide-admit/src/walk.rs) combine Tide checks with
rustc's trait and borrow checks. Rejection fixtures cover owned authority,
borrowed authority/interior mutation, overlapping mutable captures and impure
elements, including callables behind reference/box adapters. The production
[parallel-region fixture](../conformance/cases/tasks/parallel_regions.rs) covers
successful values, disjoint updates, ordered sinks and empty inputs. These are
inspected verification surfaces, not tests rerun by this research. [T3]

**Implementation qualification:** [par.rs](../crates/tide-std/src/par.rs) labels
the scheduler partial, describes OS-thread forks rather than the planned pool,
and records panic/divergence and helper-metadata limits. The reference also
labels parts of simulation/replay partial. Therefore this note establishes an
existing contract and enforcement mechanism, not a formal confluence theorem
or a claim that the runtime has completed all of D9/D15. [T2] (§6.8), [T3]

### Options and recommendation (inference)

| Option | Recommendation and reason | Fit | Cost | Alpha relevance |
| --- | --- | --- | --- | --- |
| Use the existing D7/D9 library fragment and admission checks | **Borrow the construction-first framing.** Some code can exclude scheduling interference before execution, while effectful tasks retain simulation. No extra module or marker is needed. | High | Small incremental cost | No; existing contract |
| Add a lint suggesting extraction of pure work from tasks | **Later, if diagnostics show a need.** A suggestion can improve code shape; it cannot prove confluence. A mandatory rejection rule would need the same transitive admission owner as `Pure`. | Medium | Medium | No |
| Add a wholly immutable, total sub-language | **Reject for this purpose.** D9 already allows independent computation and useful disjoint mutation. A separate language adds translation and termination obligations. | Low | Large | No |
| Replace seeded simulation with a confluence claim for all Tide tasks | **Reject.** Services must order external inputs, coordinate mutable state, cancel work and model faults. Removing those operations changes the application domain. | Low | Large | No |

D9 already owns the successful-result/sequential-equivalence contract; its
panic and trust-hole qualifications continue to apply. Reuse the existing
`tasks/par_*` rejection cases and successful parallel-region fixture as the
cheapest falsifiers before adding a check. An advisory lint cannot replace the
transitive admission checks. [T1] (D9), [T2] (§§6.6, 11), [T3]

### A new task fragment worth investigating later (inference)

D8 task code remains a separate question. Kahn process networks give a relevant
precedent: deterministic sequential processes communicate over unbounded FIFO
channels with blocking reads; they cannot branch on whether input is currently
available. LVars offers another discipline, monotone shared-state updates and
restricted reads, rather than banning shared state altogether. These are
specific communication models, not consequences of “no shared mutable state.”
[K1], [K2]

The independent Opus partner recommended **Later (v1+), research only** for a
deterministic task scope, possibly expressed through `Tasks` attenuation or a
D17 policy. Its initial candidate would use single-producer/single-consumer
channels and exclude sender cloning/multi-producer channels, `try_*`, deadlines,
clock/random/I/O capabilities, arbitrary `Shared` cells and race-to-completion
selection. Compute on owned data and join outputs in a fixed order. Keep LVars
as an alternative requiring a separate lattice API and theorem, not another
feature in the first candidate. [T1] (D7/D8/D17), [K1], [K2]

Admission would have to track channel ownership and the allowed operations
through calls and captures. Existing `Pure` excludes communication handles, so
a task-network policy cannot merely reuse that bound. Fixed-order reads from
several separate SPSC channels could still implement deterministic fan-in;
arrival-order selection is the restriction that matters.

| Option | Recommendation and reason | Fit | Cost | Alpha relevance |
| --- | --- | --- | --- | --- |
| Restricted deterministic task scope | **Later research; the best new semantic option.** Reuses structured tasks, but requires a new communication discipline and transitive admission rules. | Medium | Medium | No |
| LVars-style monotone shared state | **Later alternative.** More expressive than isolated pipelines, but needs lattice/update/read laws and a distinct API; no current caller establishes the cost is worthwhile. | Medium | Large | No |

**Decision on the partner's proposed payoff** (Opus review round, 2026-11-18):
a deterministic task scope gives a by-construction guarantee. For an admitted
pipeline, successful outputs do not depend on scheduling. It gives no cut in
simulation or replay. Replay already re-derives the schedule from the seed and
uses the trace only as a check. [T4] Under D8, the first failure cancels
siblings, so which error a scope reports can depend on timing. Across tasks,
the simulator writes sink records in scheduler order (SINK-4), so a scope must
buffer sinks per task or treat sink output as schedule-dependent. Kahn's model
assumes unbounded channels; a bounded Tide pipeline can block on writes, so
whether it completes depends on its capacities and is a liveness question.
The initial contract should therefore claim only deterministic successful
outputs under stated progress assumptions, and keep simulation, at unchanged
seed counts, for failure, cancellation, sink order and liveness. No finite seed
sample proves the general claim. [K1], [T1] (D8/D15), [T2] (§§6.8, 7), [T4]

The cheapest future falsifiers: reject two producers obtained by cloning one
sender; require equal successful outputs across seeds for an admitted pipeline;
then probe small capacities, a failed child, interleaved sink output from two
stages, cancellation and a blocked cycle.
A counterexample ends the stronger claim. Successful probes would still leave a
formal argument and an admission audit before schedule exploration could be
removed. This scope rules out ordinary multi-client servers, arrival-order
merges and timeouts; its likely users are fixed dataflow pipelines.

### Choice and independent transitions

The `amb` interaction-net extension permits nondeterministic choice using an
agent with two principal ports, so the nonoverlap argument above no longer
applies. **Inference:** Tide capabilities and synchronization handles play a
similar architectural role by locating observations of choice outside `Pure`
regions. This is an analogy, not an encoding or proof that the models coincide.
[A1], [T1] (D7)

#### Exploring the schedule space: DPOR and shared-prefix forking (inference)

The independent Opus partner recommended **Later** for dynamic partial-order
reduction (DPOR) inside `tide sim`. DPOR explores representative
interleavings using an independence relation between transitions. [P1] It
could avoid redundant exploration without adding Tide syntax (fit **high**,
cost **large**, alpha **no**). Named channels, cells and capabilities help
identify candidate dependencies. But separate handles can share a world
resource, and cancellation can affect siblings. An implementation must justify
commutation and enabledness preservation; handle inequality alone is
insufficient. DPOR's presentation also leaves out nondeterministic operations
(P1 §2.1, footnote 1), so random draws and fault choices need their own
treatment.

A second candidate borrows §2's sharing idea: **superposed seed exploration**.
Many seeds run as one shared execution that forks only at nondeterministic
choice points (a scheduler pick, a fault injection, a random draw). Everything
before a fork is shared, and every branch stays replayable. This is a strategy
for D17's Later "Snapshot and branch". Tide's simulator makes the choice points
discrete: the scheduler draws only when two or more tasks are runnable and
traces `runnable` and `chosen`, and network and process draws use their own
sub-streams. [T1] (D15, D17), [T4]

In the literal form, each branch stays a seed, and sharing is small. Each
seed drives an effectively independent stream, so two seeds agree at a point
with b alternatives with probability 1/b. Beyond the choice-free prefix, N
seeds share only about log_b N levels, and a wide random draw or a generated
test input ends sharing at once. Branching on every alternative at each choice
point shares more. But its branches are choice vectors, not seeds. Tide's
replay reruns a seed under a pinned scheduler version and compares the
recorded trace; it records no schedule to force. That form therefore needs a
replay manifest that records forced choices, which REPLAY-1/2's owner must
decide. Both forms also need a simulator that can branch: today each task runs
on its own OS thread, and POSIX `fork` copies only the calling thread. The
options are single-threaded stackful tasks plus process forking, checkpoint
and restore, or re-executing the prefix, which shares nothing. [T4], [X1]

The two ideas **complement** each other. DPOR decides which interleavings to
run; forking decides how a chosen set runs, continuing from shared states
instead of re-executing them. DPOR's own authors list forking new processes at
backtracking points, alongside re-execution and checkpointing, as ways to
implement its stateless search. [P1] (§3) Forking alone still runs every
equivalent interleaving, and DPOR alone re-executes a prefix per backtrack.

| Option | Recommendation and reason | Fit | Cost | Alpha relevance |
| --- | --- | --- | --- | --- |
| DPOR in `tide sim` | **Later.** Prunes equivalent schedules; needs a justified independence relation for Tide's actions. | High | Large | No |
| Shared-prefix forking (superposed seed exploration) | **Later, with DPOR, under D17's snapshot and branch.** Seed-named branches share little; choice-named branches need a replay format decision. Both need a forkable simulator. | Medium | Large | No |

The cheapest falsifiers need no simulator change. First, build the trie of
recorded `sched` choices from N seeds' existing traces of current sim tests,
and measure the shared fraction of steps; a small fraction ends the literal
form. Second, any later branch must produce a trace byte-identical to an
independent replay of the same seed or choice vector; the existing REPLAY-2
trace comparison is that check. Compare either technique with exhaustive
schedules of tiny modelled programs, faults included, before claiming that
bug coverage is preserved. Defer both until simulation cost or missed schedule
coverage supplies a concrete need.

Do not generalize schedule independence into arbitrary reassociation. D9 fixes
`par_reduce`'s grouping in v1; ordinary floating-point addition cannot justify
an associativity law. Confluence of a chosen evaluator would not authorize
changing its arithmetic semantics. [T1] (D9)

## 4. Other ideas to borrow or reject

The following dispositions are inferences from the cited evidence. They add no
implementation or roadmap commitment.

| Idea and evidence | Options and recommendation | Fit | Cost | Alpha relevance |
| --- | --- | --- | --- | --- |
| **Ownership at fork boundaries.** BendRT describes affine ownership, explicit forks and compiler-managed sharing/borrowing. [B1] | **Borrow the boundary discipline through existing D9.** Keep Rust closure kinds and disjoint mutation. Reject importing Bend's equal-work expectation as a Tide type rule; retain D9's runtime-owned granularity. | High | Small for existing boundary | No; existing design |
| **Small checked claims, separate from generated executables.** Bend's guide distinguishes source checking and `--verdict`; it states that translation to the kernel lacks a proof. [B2] | **Borrow the separation of specification, checker and evidence.** Tide already has an admission driver and rustc oracle. Defer a new proof pipeline; do not present a source verdict as machine-code certification. [T1] (D10/D18) | High | Small for claim discipline; large for a new pipeline | No; existing practice |
| **Graph sharing/optimal reduction as a general backend.** HVM2 documents higher-order copying restrictions, eager evaluation and sequential overhead; BendRT describes a different runtime mechanism. [H1], [B1] | **Reject an HVM target here.** Keep the current Rust backend. A future symbolic workload would need its own measured benefit, complete translation contract and resource bounds. | Low | Large | No |
| **Global binders and explicit erasure.** IC allows global variable scope and has ERA rules. [H3] | **Reject for Tide.** Keep lexical Rust scope and D2's drop behavior; do not add nonlocal substitutions or an erase spelling without a missing resource invariant. | Low | Large | No |
| **One execution model across CPU/GPU.** BendRT describes common runtime data and device execution, with scheduling/workload constraints. [B1] | **Later through the existing D16 boundary.** Reject a Tide runtime or numeric redesign to accommodate that mechanism. Measure a useful kernel at the whole boundary before making a throughput claim. [T1] (D16) | Medium | Large | No |

## Questions left open

1. Would clearer diagnostics for D9 prevent agents from moving impure state into
   parallel closures? Recommend observing existing failures before authorizing a
   lint or another annotation.
2. Can a public symbolic-search workload show a benefit from labeled shared
   alternatives after accounting for memory, conversion and validation? Recommend
   an ordinary enumerator baseline before selecting an engine.
3. Does a future consumable-resource API need exactly-once completion rather
   than affine “use at most once”? Recommend deciding with its concrete caller;
   Rust moves alone do not enforce eventual use.
4. For the deferred quantum-domain parent fx-onsw: how should distinct mutable
   gate operands, resource disposal and uncomputation fit a concrete API?
   Recommend evaluating them there; this note chooses no quantum API.

No finding here supplies a reason to reopen D16.

## Sources and identities

External artifacts below were read **2026-11-18**. Repository permalinks name
snapshots; publication year identifies papers. Local Tide sources were inspected
at baseline `19db31f6f836c150bd58d6675cb6a42449edba83` on the same date.

- **L1.** Yves Lafont, *Interaction Combinators*, Information and Computation
  137(1), 69–101 (1997), [DOI](https://doi.org/10.1006/inco.1997.2643).
  Read the [author's March 25, 1997 manuscript](https://www.i2m.univ-amu.fr/perso/yves.lafont/pub/combinators.ps),
  §§1.1–1.2 (Proposition 1 and its consequence), §2.1.
- **H1.** Victor Taelin / HigherOrderCO, *HVM2: A Parallel Evaluator for
  Interaction Combinators*, work-in-progress paper (2024),
  [`654276018084b8f44a22b562dd68ab18583bfb5b`](https://github.com/HigherOrderCO/HVM2/blob/654276018084b8f44a22b562dd68ab18583bfb5b/paper/HVM2.typst).
- **H2.** HigherOrderCO, HVM3 README,
  [`fba2e9c82faf6e2f019c9ecea94c32f19a8b7820`](https://github.com/HigherOrderCO/HVM3/blob/fba2e9c82faf6e2f019c9ecea94c32f19a8b7820/README.md).
  Its compiler-target intentions are historical claims, not Bend's current architecture.
- **H3.** Victor Taelin, *Interaction Calculus*, Specification and Collapsing,
  [`e9f0185682ba8b41205691efd4f58491dc405a53`](https://github.com/VictorTaelin/Interaction-Calculus/blob/e9f0185682ba8b41205691efd4f58491dc405a53/README.md).
- **B1.** HigherOrderCO / bendlang, *BendRT: A Parallel Runtime for CPUs and
  GPUs*, paper source (2026),
  [`f0cf0a7c573145501981c44d9c33e326185f008e`](https://github.com/bendlang/bend/blob/f0cf0a7c573145501981c44d9c33e326185f008e/bend2/docs/BendRT/main.typ).
- **B2.** Bend language guide,
  [`f0cf0a7c573145501981c44d9c33e326185f008e`](https://github.com/bendlang/bend/blob/f0cf0a7c573145501981c44d9c33e326185f008e/guide/GUIDE.md).
- **Q1.** W. K. Wootters and W. H. Zurek, *A single quantum cannot be cloned*,
  Nature 299, 802–803 (1982), [DOI](https://doi.org/10.1038/299802a0);
  read the [Caltech-hosted paper scan](https://painterlab.caltech.edu/wp-content/uploads/2019/06/qe_wotters_no_cloning.pdf).
- **Q2.** John Watrous, *The Theory of Quantum Information*, January 2018
  pre-publication manuscript, [author-hosted PDF](https://cs.uwaterloo.ca/~watrous/TQI/TQI.pdf),
  §§2.1.2 (pure states as complex unit vectors modulo global phase), 2.2 and
  2.3.1 (measurement probabilities, Definition 2.34). Relative-phase dependence
  follows from the state/measurement formulas.
- **S1.** ETH Zürich Silq [documentation](https://silq.ethz.ch/documentation),
  Annotations and Other Functions, snapshot read 2026-11-18 (unversioned page;
  retrieved HTML SHA-256 `606310e9c8b462653a471b11ca1212077320bcd79cfa87ef7f19141899e90b6c`).
- **K1.** Gilles Kahn, *The Semantics of a Simple Language for Parallel
  Programming*, Information Processing 74 (1974), pp. 471–475,
  [paper scan](https://www.cs.columbia.edu/~sedwards/papers/kahn1974semantics.pdf),
  §§1–3. Read scan pages 1 and 3, including nonblocking sends and the least-fixed-point model.
- **K2.** Lindsey Kuper and Ryan R. Newton, *A Lattice-Theoretical Approach to
  Deterministic Parallelism with Shared State*, Indiana TR702, October 2012
  revision, [paper](https://scholarworks.iu.edu/dspace/bitstreams/51e17b47-c0e4-4ad5-848c-aef0b72bfb67/download),
  introduction and §2.1. This is the earlier LVars technical report, not an
  assertion that every later LVars extension is deterministic in the same sense.
- **A1.** Damiano Mazza, *Interaction Nets: Semantics and Concurrent Extensions*,
  doctoral thesis, December 2, 2006, version 1.1,
  [author-hosted PDF](https://www.lipn.fr/~mazza/papers/Thesis-1.1.pdf),
  chapter 4 and introduction's chapter-4 synopsis. Mazza treats the multiport
  extension and credits the two-principal-port `amb` construction to Khalil.
- **P1.** Cormac Flanagan and Patrice Godefroid, *Dynamic Partial-Order Reduction
  for Model Checking Software*, POPL 2005, pp. 110–121,
  [author-hosted PDF](https://patricegodefroid.github.io/public_psfiles/popl2005.pdf),
  §2.1 and footnote 1 (deterministic processes; nondeterministic operations set
  aside); §3 Definition 1 (independence) and backtracking by re-execution,
  forking or checkpointing, [DOI](https://doi.org/10.1145/1040305.1040315).
- **T1.** Tide [decision record](../design/01-design.md), D2, D7, D9, D10,
  D8, D15, D16, D17 and D18; these own the language decisions.
- **T2.** Tide [core reference](../design/reference.md), §§6.1, 6.6, 6.8, 7, 8
  and 11; current/partial/planned labels govern availability.
- **T3.** Tide [capability declarations](../crates/tide-std/src/root.rs),
  [purity owner](../crates/tide-admit/src/pure.rs),
  [admission walk](../crates/tide-admit/src/walk.rs),
  [parallel runtime](../crates/tide-std/src/par.rs),
  [rule map](../conformance/rules.md) and the `tasks/par_*_reject.rs` and
  `tasks/parallel_regions.rs` conformance cases cited above.
- **T4.** Tide simulator and replay contracts: [chapter 03](../design/03-authority-concurrency.md)
  §9.6 (REPLAY-1/2, `--strategy`, `--faults`) and SINK-4; the
  [scheduler](../crates/tide-std/src/task/scheduler.rs) (`pick`, sub-stream draws),
  [task runtime](../crates/tide-std/src/task.rs) (one OS thread per task under a
  baton), [replay manifest](../crates/tide-std/src/sim/manifest.rs) and
  [replay wire format](../crates/tide-std/src/replay.rs).
- **X1.** The Open Group, POSIX.1-2024, [`fork()`](https://pubs.opengroup.org/onlinepubs/9799919799/functions/fork.html):
  a multithreaded process's child contains “a replica of the calling thread”
  only (DESCRIPTION and APPLICATION USAGE). Retrieved HTML SHA-256
  `ab97739ed39e8a21f7304a03d5efd039fc601d5744f5f6ac0e93d9ddd96a41c1`.

[L1]: https://www.i2m.univ-amu.fr/perso/yves.lafont/pub/combinators.ps
[H1]: https://github.com/HigherOrderCO/HVM2/blob/654276018084b8f44a22b562dd68ab18583bfb5b/paper/HVM2.typst
[H2]: https://github.com/HigherOrderCO/HVM3/blob/fba2e9c82faf6e2f019c9ecea94c32f19a8b7820/README.md
[H3]: https://github.com/VictorTaelin/Interaction-Calculus/blob/e9f0185682ba8b41205691efd4f58491dc405a53/README.md
[B1]: https://github.com/bendlang/bend/blob/f0cf0a7c573145501981c44d9c33e326185f008e/bend2/docs/BendRT/main.typ
[B2]: https://github.com/bendlang/bend/blob/f0cf0a7c573145501981c44d9c33e326185f008e/guide/GUIDE.md
[Q1]: https://painterlab.caltech.edu/wp-content/uploads/2019/06/qe_wotters_no_cloning.pdf
[T1]: ../design/01-design.md
[T2]: ../design/reference.md
[T3]: ../crates/tide-admit/src/pure.rs
[Q2]: https://cs.uwaterloo.ca/~watrous/TQI/TQI.pdf
[S1]: https://silq.ethz.ch/documentation
[K1]: https://www.cs.columbia.edu/~sedwards/papers/kahn1974semantics.pdf
[K2]: https://scholarworks.iu.edu/dspace/bitstreams/51e17b47-c0e4-4ad5-848c-aef0b72bfb67/download
[A1]: https://www.lipn.fr/~mazza/papers/Thesis-1.1.pdf
[P1]: https://patricegodefroid.github.io/public_psfiles/popl2005.pdf
[T4]: ../design/03-authority-concurrency.md
[X1]: https://pubs.opengroup.org/onlinepubs/9799919799/functions/fork.html
