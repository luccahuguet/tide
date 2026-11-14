# 31 — Bend update and D16 re-check

Research date: **2026-11-14** (fx-5i0o). Builds on [report 01](01-bend-hvm.md),
[report 11](11-bend-interop.md), and [report 28](28-positioning-and-name.md).
**Inference** marks conclusions and proposed checks, not measurements.
Repository statements below describe the inspected 2026-11-14 snapshot unless
another date is given. No Bend installation, compilation, proof check or benchmark
ran. No held-out trial material was opened.

Evidence boundary: Bend `main` at
[`f0cf0a7c573145501981c44d9c33e326185f008e`][2] (2026-11-14 19:03Z), compared
with the correction's `f311527776559ce40095108ad6a7514a6c34aaa1` (2026-11-12
09:02Z), and published release 2.0.40. These are different candidates. An
unreleased fix does not establish behavior of the installed release.

## Question

What changed after the November 12 correction; which D16 claims survive; what
independent parallel-performance evidence exists against Rust or C; what should
Tide borrow or reject; and is a small Stage 0 experiment worth doing before the
alpha examples?

## Bottom line

**Keep D16 in Later, with Stage 0 as an untested integration proposal and Stage 1
conditional.** On 2026-11-14, 2.0.40 is still the latest published release; the
native library target remains unscheduled, and the scalar/array limits behind
D16's type map remain documented. [Releases][1], [WONTFIX][3].

The evidence has improved in two directions. Third-party CPU/OpenMP and CUDA
measurements exist, though their principal results use older Bend versions.
Taelin also describes a self-hosted, certified Bend 3 compiler, but the inspected
public tree still contains the TypeScript compiler and its audit warning. Treat
those announcements as author claims, not a shipped replacement or independent
proof of native-code correctness. [Performance][22], [Bend 3 post, November 4][13],
[README][5].

**Inference, 2026-11-14:** a bounded scalar process experiment could test the
missing boundary. It should follow a usable Tide process interface, outside
fx-rpsb: today's `Proc` has no operations. It would establish neither a GPU
advantage nor Tide's full `Accel` contract.

## Findings (cited)

### F1. Actual changes since the correction

The [GitHub publication record][1], inspected 2026-11-14, still ends at
**2.0.40, published 2026-11-06 02:06Z**, tag commit `624dd9f4607c`. The changelog
labels it November 5. No later release appeared in that record. The inspected
tree is 38 commits ahead of that tag, **five** ahead of the correction's commit.
The [comparison][2] contains these five changes:

| Commit date (UTC) | Change recorded by upstream | Boundary relevance |
|---|---|---|
| 2026-11-12 | `9162bfe`: remember failed typed-let parsing attempts (#2133) | Removes repeated parsing of nested annotations; no new wire type |
| 2026-11-13 | `011b54d`: preserve quantity literals after `return` in formatting (#2136) | Agent edits must preserve meaning |
| 2026-11-13 | `4e9c229`: avoid rebuilding strings in short `Map.get` steps (#2145) | Library optimization |
| 2026-11-13 | `38103b3`: separate colliding datatype traversal keys (#2143) | Fixes omission of a reachable closure from GPU code |
| 2026-11-14 | `f0cf0a7`: flush stdout before native file/process jobs (#2155) | Fixes output ordering around helper-thread effects |

These changes do not alter WONTFIX, the documented number set or the native
library promise. In particular, [#813][4] still shows `closed/not_planned`, with
the maintainer's **2026-09-19** explanation that the proposal moved to SOON;
[WONTFIX][3] still calls selected exports, a header and lifecycle planned without
a schedule. Closing the issue does not mean that feature shipped.

### F2. Direction, people and funding: announcements versus available artifacts

These posts predate the correction except for the November 12 clarification;
they are **new evidence for this report**, not changes that all occurred after
November 12. X blocked direct retrieval; the author's post text and UTC dates
were read through FXTwitter's JSON mirror, with original post URLs below.

| Date (UTC) | Author's statement | What the evidence establishes |
|---|---|---|
| 2026-11-04 | Taelin says Opus 5.5 rewrote Bend in Bend, including a certified compiler, models of Bend/C/CUDA, and a claimed 2× improvement. [Post][13] | A development announcement. No released compiler or checked certificate for that replacement was located in the inspected public release/main surfaces. “Zero bugs” is his claim. |
| 2026-11-07 | Taelin announces an up-to-$8m round at a $120m post-money cap, a $1m personal investment, and concentration on symbolic AI research. He says Bend support continues, with fewer tokens than he would like. [Post][14] | Fundraising intent and resource priorities, not confirmation that $8m closed or a runway figure. |
| 2026-11-11 | Taelin thanks investors, reports Studio Vibi's merger with HOC and user-facing Bend progress; he says HVM5 remains private. [Post][16] | A reported merger and continued development. No staffing allocation, merger terms or total funds raised are given there. |
| 2026-11-12 | Taelin says Bend 3 keeps the language and changes compiler speed. [Post][17] | Intended source continuity; it supplies no compatibility test or ABI commitment. |

The [company's deck][15], inspected 2026-11-14, corroborates the smaller round
and symbolic-AI focus. It names Victor Taelin, Vitor Chiarelli Neves, Nicolas
Abril, Lorenzo Battistela and Paulo Cavalcanti, and says Bend will likely stay
open source and free. It does not establish a completed raise. The current
[README credits][5] name additional contributors; credits and a pitch-deck team
list are not a headcount or support contract.

**Inference, 2026-11-14:** Bend's agent-first positioning remains relevant to
Tide, but HOC's commercial priority is symbolic AI, with Bend an enabling tool.
Do not schedule Tide around Bend 3, SupGen or #813. Report 28's capability and
replay positioning remains useful without a claim of superiority over Bend's
proofs or compiler speed.

### F3. Process transport and proof boundaries need narrower wording

**Observed 2026-11-14:** Base still offers `IO.args`, file effects and output,
without a dedicated stdin effect. `Process.run(..., input, ...)` writes to a
*child's* stdin; it is not an input reader for the current Bend process.
[Base][7], [guide][8].

However, “no stdin at all” is too strong. Native [`File.open`][10] opens a path
using POSIX `open`; [`File.read`][11] uses `read`, without requiring a seek.
**Inference, untested, 2026-11-14:** on a suitable Unix host,
`File.open("/dev/stdin", "r")` followed by a read may consume inherited stdin.
That depends on the host's descriptor filesystem, permissions and execution
lane. It is not a documented portable stdin API. An implementation should test
pipe input, EOF and bounds before choosing it; Stage 0 can use arguments and
files without relying on it. Do not change the companion's loopback-server
transport merely on this inference.

**Already changed before the correction:** since **2.0.38, published
2026-10-20**, native runtime options use `+RTS ... -RTS`. For example:
`./kernel +RTS --threads 4 --gpu off -RTS 12`. Plain `--threads 4` is an
application argument. The current native parser also reads `BENDRTS`; a
supervisor must control that environment when enforcing a budget. This corrects
report 11's abbreviated flag guidance. [2.0.38 release][28], [runtime parser][6].

**Observed 2026-11-14:** a typed Bend wrapper can call a kernel at a declared
signature, and the checker can reject a type mismatch. That does not verify
text decoding, schema identity, limits, foreign IO or generated machine code.
The [guide][8] distinguishes ordinary checking from `--verdict`, and warns that
the source-to-kernel translation lacks a proof. **Inference:** check a pure
kernel/law artifact separately from the effectful protocol wrapper; require
the verdict to cover the kernel and fail closed on excluded dependencies.

**Correction to report 11's proof-fragment restriction, observed 2026-11-14:**
U32 arithmetic is not uniformly opaque to proofs. Base defines U32 through
`Word(32n)`, implements `U32.add` in Bend, and supplies a proof of
`U32.add_comm`. F32 operations remain axiomatic. This does not supply a Tide
contract translator: its semantics must distinguish wrapping U32 operations
from Tide's checked arithmetic. The proof support is thin: Base has no other
U32 arithmetic law (no sub, mul, div or mod lemmas), and the definitions were
already proof-visible at report 11's snapshot, so this corrects report 11
rather than reporting an upstream change. [Base definitions][7], [WONTFIX][3].

### F4. Independent performance evidence exists, with version and workload limits

Here “third-party measured” means the benchmark author reports measurements
outside Bend's own benchmark repository. Tide did not reproduce them or audit
the investigators' affiliations. It does not mean multiple independent
replications of each result.

**Bend's own figures.** The [M4 Max pin][27], inspected 2026-11-14, still records
**2026-09-17, `d0db7b3e`**: Game of Life takes C 6.776 s, Bend sequential
7.803 s, 16-core 0.647 s and Metal 0.063 s. Those columns compare a sequential
C twin with Bend's parallel implementations; they contain no parallel Rust or
CUDA baseline. They are historical, self-reported measurements, not timings of
2.0.40. Report 01's paper figures describe a different snapshot.

**Third-party CPU versus C/OpenMP.** Wakamex's [preserved performance report][22]
dates its run **2026-09-17**: Bend **2.0.3**, commit `b9d1352c9f45`, Ryzen 9
3950X, RTX 3090. The author reports medians of ten timed executions after
correctness checks and warmup:

| Workload | Bend, 16 CPU threads | C/OpenMP, 16 threads | Reading |
|---|---:|---:|---|
| Independent 4×4 Game of Life soups | 0.742 s | 0.824 s | Bend finishes sooner |
| N-Queens | 0.834 s | 0.343 s | Bend takes 2.43× as long |
| Tree bitonic / GNU parallel sort | 1.689 s | 0.087 s | Different algorithms; not scheduler overhead alone |
| UTS, 30,399,117 nodes | 7.797 s | 2.263 s | This natural Bend port gains no speedup over its 7.368 s one-thread run |

The author's median Bend/OpenMP ratio over the 16 vendor workloads is 1.32;
Bend's median speedup over itself is 13.58 at 16 threads. Conventional
implementations include algorithm and representation changes. The report
preserves configurations and output contracts, but says its full original raw
sample directory is excluded from Git. Close differences need confirmation.

**Third-party GPU versus CUDA.** The same investigator's [startup-scaling
study][23], inspected at **2026-11-14 revision `40c8effdf7a2`**, uses Bend
2.0.3 and doubling workloads, five alternating rounds per size, checked outputs,
and fitted fixed-cost/work terms. Its reported median ratio for added work is
**5.7× Bend/conventional GPU time**, with Game of Life near 1.3× and HotSpot
near 300×. These are fitted work ratios, not directly timed kernel-only ratios.
The study explains why short whole-process runs can favor Bend despite slower
sustained computation. Sources, algorithm choices and memory ceilings constrain
each comparison; this is not a universal CUDA penalty.

The [paired version comparison][24] records runs on **2026-09-23** comparing
2.0.3 with **2.0.26**: geometric-mean time ratios near 1.03 on CPU and 0.97 on
GPU, with substantial per-workload variation and shared-host interference.
Those data cannot be extrapolated to 2.0.40 or the announced Bend 3 compiler.

**Third-party Rust comparison, weaker evidence.** Koru's [board][25] reports
**2026-10-25**, Bend **2.0.5**, Apple M2 Pro under load, against Rust and C.
For `coins`, it lists Bend-par 33.2 ms, Rust 91.9 ms and clang C 79.1 ms; for
`powmod`, 816.5 ms, 28.6 ms and 28.8 ms. Its [harness and ports][26] check an
output oracle, but the Rust/C columns are sequential, the harness does not pin
the Bend worker count, and some Bend ports use `@unsafe`. The repository's
`results/latest.json` is dated October 23, so it is not raw confirmation of the
October 25 board. Treat the board as exploratory evidence, not a parallel
Rust/Rayon comparison or evidence for proved kernels.

**Search conclusion, 2026-11-14:** independent parallel C/OpenMP and CUDA
evidence is available; a controlled current-version Bend-versus-parallel-Rust
comparison was not located. Akita's CLI ports and Astropulse's image port remain
the limited application evidence described in report 01; they do not fill that
gap. **Inference:** D16 has a plausible throughput use case, but Tide should
claim a gain only for a measured kernel, including process/serialization cost.

### F5. What Tide should adopt or reject from the process

**Observed 2026-11-14:** the [Bend contributor map][18] distinguishes the
human-written parser/theory/checker from compiler/runtime code, and identifies
the Lean kernel behind `--verdict`. The public [README][5] still calls the
compiler 99% AI-written and not fully audited. Repository-size caps and
[cross-checking gates][20] constrain work; their existence does not prove that
all gates passed at this snapshot. The [PR workflow][19] obtains gate code from
`main` and also runs on merge groups, so a proposed change cannot approve itself
by editing that gate.

**New evidence beyond report 01:** Bend's [checker eval][21], published in a
repository commit on **2026-10-22**, labels its results **2026-10-10**. It reports
Opus 5.5 solving 20/20 tasks with checker access for $18.24, versus 0/20 blind
for $12.92. The runner preserves task text, forbids redefinition/unsafe escapes,
and grades with `--verdict`. This is a vendor experiment on 20 supplied tasks,
not an independent language comparison. The blind arm cannot use tools and
must answer once; the checker arm iterates. **Inference:** it supports prompt
feedback and protected specifications, without isolating language superiority
or predicting Tide's learnability results.

Recommendations **[inference, 2026-11-14]**, building on reports 01 and 28:

- **Adopt the separation of claims, implementations and evidence.** Keep the
  specification outside the editing agent's discretion; review its adequacy,
  use negative mutations, and preserve an independent checker/oracle. Tide
  already has the rustc oracle; this evidence does not justify replacing it.
- **Adopt protected gates where useful.** Evaluate a patch with a trusted gate
  identity and check the merged candidate. A token cap can bound review cost,
  but cannot substitute for architectural review or semantic tests.
- **Reject “proved source means a bug-free executable.”** Require evidence
  covering parsing, translation, code generation, runtime and the boundary
  before widening a proof claim. The Bend 3 announcement does not provide that
  evidence for the public Bend 2 toolchain.
- **Reject a parallel-language redesign for Tide.** Balanced kernels fit an
  optional boundary; mandatory explicit types, affine closures and equal-work
  forks need not become Tide restrictions. Keep report 28's capability/replay
  purpose and measure agent outcomes instead of copying the marketing.

## D16 claim-by-claim table

Verdicts are **as of 2026-11-14**. **Holds** means the inspected Bend surface
supports the proposal or its stated restriction, not that Tide implemented it.
**Unknown** means a missing check or artifact prevents the stronger conclusion.
No new upstream change requires reversing D16; the qualifications below matter
before implementation. D16 owns the decision; its
[companion §10](../design/03-authority-concurrency.md#10-the-bend-boundary-later-tier-d16)
elaborates the boundary.

| D16 claim | Verdict | Dated evidence and limit |
|---|---|---|
| Pure, proved, massively parallel Bend kernels | **Holds** | 2026-11-14: source laws/kernel checking and balanced parallel execution exist. A proof covers source semantics; speed and native correctness need their own evidence. [Kernel][29], [CPU study][22]. |
| Bend calls Tide libraries once Tide exports C ABIs | **Holds** | 2026-11-14: foreign C effects remain available; their ABI tracks the compiler. This conditional claim does not establish Tide's future export implementation. [Effects guide][9]. |
| Stage 0 feasible through supervised processes and text | **Holds** | 2026-11-14: native programs, arguments, file input and stdout support the mechanism, and `TCP.listen`/`accept` remain in Base for the companion's loopback transport (inference: untested). No Tide/Bend end-to-end run was made. [Arguments][12], [file effects][10], [runtime][6]. |
| Exact dependency pin and cached builds | **Unknown** | 2026-11-14: Tide specifies these; no Bend dependency integration was demonstrated. Patch releases can break invocation, as 2.0.38 did. [Release][28]; local companion §10. |
| Generated shim lets Bend verify the signature | **Holds** | 2026-11-14: the mechanism is an annotated call. A deliberately mismatched generated shim must be rejected before this counts as integration proof; the generator is untested. Wire encoding remains separate. [Guide][8]. |
| Bend crash becomes Tide `Err` | **Unknown** | 2026-11-14: runtime failures can terminate the child, supporting process isolation. Tide must observe nonzero exit, signals, timeout and bad output; a parent adapter has not been tested. [Runtime][6]. |
| Stage 1 only if a native library target ships | **Holds** | 2026-09-19 promise, rechecked 2026-11-14: #813 closed, header/lifecycle/exports still unscheduled. No native library target is established. [Issue][4], [WONTFIX][3]. |
| No native 64-bit Bend numeric types in the map | **Holds** | 2026-11-14: U64/F64 remain SOON; Nat is not a substitute for all u64 values. [WONTFIX][3], [Base][7]. |
| Bounded naturals | **Holds** | 2026-11-14: theory is unbounded, runtime fail-stops above 2^48−1. Validate both directions; total source code can still exhaust runtime resources. [WONTFIX][3]. |
| Power-of-two arrays | **Holds** | 2026-11-14: balanced tree shape is a runtime requirement; unbalanced public constructors can type-check then trap. Padding needs a checked logical length. [WONTFIX][3], [Base][7]. |
| Strings copied; values move/copy, never share; no closures/handles | **Holds** | 2026-11-14: this is Tide boundary policy. Strings are character lists and arrays affine. A process encoding can copy values without exposing heaps; this does not follow from a common ABI. [Base][7]; companion §10. |
| `Accel`, one runtime per process, thread budget | **Holds** | 2026-11-14: conservative policy fits global runtime state. Budgeting must control runtime arguments/environment and child count. No `Accel` enforcement was demonstrated. [Issue][4], [parser][6]. |
| Kernels pure and total | **Holds** | 2026-11-14: source guarantee requires a fully covered verdict with no unsafe/foreign dependency for the kernel. The IO wrapper, allocation failure and finite machine bounds remain outside it. [Guide][8], [kernel][29]. |
| Export a Tide contract as a Bend law; import at lower trust | **Unknown** | 2026-11-14: no translator was demonstrated. U32 has proof-visible definitions; preserving wrap/overflow semantics is still required. The lower-trust qualification remains appropriate: translation/backends are unproved. [Base][7], [README][5]. |
| Deterministic `par` supports differential kernel tests | **Holds** | 2026-11-14: a test strategy, not universal cross-lane equality. Integer outputs can be compared; F32 lane differences are documented. Purity alone does not establish simulation coverage. [WONTFIX][3]; companion §10. |

The changed assertions are in the **research evidence**, rather than a newly
shipped D16 capability: F3 qualifies absolute stdin absence and corrects U32
proof opacity and runtime-flag guidance; F4 supplies third-party parallel C/CUDA
evidence; F2 updates the announced compiler/company direction. No 64-bit or
library-target claim changes to “available.”

## Recommendations

### R1. D16 disposition: keep its architecture, qualify its evidence

Options **[inference, 2026-11-14]**:

| Option | Consequence | Recommendation |
|---|---|---|
| Keep D16 and cite this report; interpret feasibility as source-supported, untested | Preserves Later scope and both stages | **Adopt.** The upstream constraints still fit; no integration measurement justifies promotion. |
| Qualify D16's Stage 0 sentence as “upstream mechanisms available; Tide integration untested” | Changes evidence wording without changing the chosen behavior | **Recommend at the next authorized design edit.** Retain the typed-call check, and state that wire correctness needs separate validation. |
| Promote tandem into v0/alpha examples or depend on Bend 3/#813 | Adds process, build, proof and toolchain obligations before a measured payoff | Reject for this alpha. |

No design file changes in this bead. If the owner chooses a stdin transport
change later, options are to keep the specified loopback server, test a
Unix-only descriptor-path adapter, or wait for a documented stdin API.
**Recommendation:** retain the specified transport pending a check; “may work
through `/dev/stdin`” is insufficient reason to replace it. Likewise, U32 proof
support justifies investigating a future translator, not silently expanding its
accepted fragment.

### R2. The smallest worthwhile Stage 0 experiment

**Recommendation [inference, 2026-11-14]: yes as a later, bounded experiment;
no as an fx-rpsb alpha example.** At this checkout,
[`tide-std`'s `Proc`](../crates/tide-std/src/lib.rs) is an authority identity
with no operations. D16's dependency builder, generated bindings and `Accel`
also remain planned. The examples epic **fx-rpsb** requires admitted, runnable
workspace examples showing Tide's distinctive features (bead read 2026-11-14).

Proposed contract: **a Tide caller sends one bounded scalar request to a pinned
Bend child, receives the exact integer result, and retains control on child or
protocol failure.** Choose the README's balanced `pow2` shape, restrict the
requested depth to 0..20, and compare with Tide's integer reference. Pass depth
as an argument, return one versioned text line on stdout. This avoids an stdin
assumption. It is a one-shot precursor, not an implementation of the companion's
persistent TCP server.

The proposed check would:

1. Pin **2.0.40 and its artifact identity**; build once. Keep a pure kernel/law
   file for `--verdict` and a separate IO entry wrapper. Do not claim the latter
   is proved.
2. Generate one typed call; deliberately change its parameter/result type and
   require checker rejection. Check an unproved law and an unsafe dependency
   fail the proof admission path too.
3. Invoke from an admitted Tide program through an explicit, reviewed process
   boundary. Until such a route exists, report that prerequisite rather than
   substituting an ordinary Rust host and calling it Tide integration.
4. Compare depths 0, 1 and 20 and repeated calls; reject depth 21, malformed
   replies and a schema/version mismatch. Force a nonzero exit, timeout and
   truncated output; the caller must return an error and reap the child.
5. Use one and four CPU workers, GPU disabled, with controlled `BENDRTS`.
   Report build time separately from spawn/encode/run/decode time. Equal
   outputs establish these cases, not a performance win.

**Cost estimate [inference]:** allow half a day for the Bend scalar wrapper,
proof/checker probes and protocol, then up to a day for a Tide caller and error
checks **once a usable process interface exists**. A granted unsafe adapter
could bypass that prerequisite but would add a trust hole and would not validate
the planned `Accel` boundary; do not build it just to make this demo appear ready.
Do not spend this experiment implementing general dependency syntax, a complete
type map, a server pool, GPU support or simulator models.

Success would justify confidence in one scalar path, source-signature rejection
and child-failure isolation. It would not establish array/string costs, a
cross-language proof translator, replay semantics, performance amortization,
native-code certification or Stage 1. A later workload with actual application
value must measure those costs before tandem moves into the product examples.

## Sources

All sources accessed **2026-11-14**. Repository permalinks identify inspected
artifacts; a date on a measurement or post is its own date, not access time.
Author-post mirrors supplied text only, not independent corroboration.

1. Bend [release publications][1], especially [v2.0.40](https://github.com/bendlang/bend/releases/tag/v2.0.40), 2026-11-06; GitHub API publication timestamps also inspected.
2. Bend [correction-to-current comparison][2], 2026-11-12–14; [release-to-current comparison](https://github.com/bendlang/bend/compare/v2.0.40...f0cf0a7c573145501981c44d9c33e326185f008e).
3. Bend [WONTFIX.txt][3], inspected at `f0cf0a7`, including SOON and runtime limits.
4. Bend [#813 and maintainer replies][4], 2026-09-18–19, state rechecked 2026-11-14.
5. Bend [README][5], inspected at `f0cf0a7`: positioning, limitations and credits.
6. Bend [compiler/runtime source][6], inspected at `f0cf0a7`: runtime state, failures and option parser.
7. Bend [Base][7], inspected at `f0cf0a7`: types, IO, U32 definitions and proofs.
8. Bend [language guide][8], inspected at `f0cf0a7`: checking, laws and IO.
9. Bend [foreign-effects guide][9], inspected at `f0cf0a7`: native integration and ABI scope.
10. Bend [native file-open effect][10], inspected at `f0cf0a7`.
11. Bend [native file-read effect][11], inspected at `f0cf0a7`.
12. Bend [native argument effect][12], inspected at `f0cf0a7`.
13. Victor Taelin, [Bend 3 compiler announcement][13], **2026-11-04 14:23Z**; [JSON mirror](https://api.fxtwitter.com/status/2117986155186487779).
14. Victor Taelin, [fundraising and research direction][14], **2026-11-07 16:21Z**; [JSON mirror](https://api.fxtwitter.com/status/2119102876521824572).
15. HOC, [SupGen fundraising deck][15], undated, accessed 2026-11-14, pp. 3–6.
16. Victor Taelin, [investors, Studio Vibi and HVM5][16], **2026-11-11 17:07Z**; [JSON mirror](https://api.fxtwitter.com/status/2120564095615156509).
17. Victor Taelin, [Bend 3 language-continuity statement][17], **2026-11-12 17:31Z**; [JSON mirror](https://api.fxtwitter.com/status/2120932449979163022).
18. Bend [AGENTS.md][18], inspected at `f0cf0a7`, ownership and gate map.
19. Bend [repository-gate workflow][19], inspected at `f0cf0a7`.
20. Bend [verdict cross-check gate][20], inspected at `f0cf0a7`.
21. Bend [checker eval][21], commit `854bdd852670a79f0fe40d665b0ca35e892125cc`, **2026-10-22**; results labelled 2026-10-10; [runner](https://github.com/bendlang/checker-eval/blob/854bdd852670a79f0fe40d665b0ca35e892125cc/run.py).
22. Wakamex, [CPU/OpenMP performance report][22], **2026-09-17** measurements, inspected at `40c8effdf7a282df2fd28ebb2cf35a60de7cea52` (2026-11-14).
23. Wakamex, [GPU startup/work scaling study][23], September–October 2026 runs, inspected at `40c8eff`; [measurement policy](https://github.com/wakamex/bend-bench/blob/40c8effdf7a282df2fd28ebb2cf35a60de7cea52/MEASUREMENT.md).
24. Wakamex, [Bend 2.0.26 paired comparison][24], **2026-09-23** run identifiers, inspected at `40c8eff`.
25. Koru, [compute benchmark board][25], **2026-10-25** measurement label, accessed 2026-11-14.
26. Koru, [benchmark harness][26], inspected at `f84914374d06192624b8ea3c8ce52553e14effe2` (2026-10-25); [parallel Bend ports](https://github.com/korulang/koru-benchmarks/tree/f84914374d06192624b8ea3c8ce52553e14effe2/suites/osprey-compute-kernels/bend-par); [October 23 JSON](https://github.com/korulang/koru-benchmarks/blob/f84914374d06192624b8ea3c8ce52553e14effe2/results/latest.json).
27. Bend [M4 Max runtime pin][27], **2026-09-17**, `d0db7b3e`, preserved at inspected `f0cf0a7`.
28. Bend [2.0.38 release][28], **2026-10-20**, runtime-argument change.
29. Bend [Lean kernel and claims][29], inspected at `f0cf0a7`.

[1]: https://github.com/bendlang/bend/releases
[2]: https://github.com/bendlang/bend/compare/f311527776559ce40095108ad6a7514a6c34aaa1...f0cf0a7c573145501981c44d9c33e326185f008e
[3]: https://github.com/bendlang/bend/blob/f0cf0a7c573145501981c44d9c33e326185f008e/WONTFIX.txt
[4]: https://github.com/bendlang/bend/issues/813
[5]: https://github.com/bendlang/bend/blob/f0cf0a7c573145501981c44d9c33e326185f008e/README.md
[6]: https://github.com/bendlang/bend/blob/f0cf0a7c573145501981c44d9c33e326185f008e/bend2/comp.ts
[7]: https://github.com/bendlang/bend/blob/f0cf0a7c573145501981c44d9c33e326185f008e/bend2/base.bend
[8]: https://github.com/bendlang/bend/blob/f0cf0a7c573145501981c44d9c33e326185f008e/guide/GUIDE.md
[9]: https://github.com/bendlang/bend/blob/f0cf0a7c573145501981c44d9c33e326185f008e/guide/EFFECTS.md
[10]: https://github.com/bendlang/bend/blob/f0cf0a7c573145501981c44d9c33e326185f008e/bend2/effs/file_open.c
[11]: https://github.com/bendlang/bend/blob/f0cf0a7c573145501981c44d9c33e326185f008e/bend2/effs/file_read.c
[12]: https://github.com/bendlang/bend/blob/f0cf0a7c573145501981c44d9c33e326185f008e/bend2/effs/args.c
[13]: https://x.com/VictorTaelin/status/2117986155186487779
[14]: https://x.com/VictorTaelin/status/2119102876521824572
[15]: https://bend-lang.com/deck.pdf
[16]: https://x.com/VictorTaelin/status/2120564095615156509
[17]: https://x.com/VictorTaelin/status/2120932449979163022
[18]: https://github.com/bendlang/bend/blob/f0cf0a7c573145501981c44d9c33e326185f008e/AGENTS.md
[19]: https://github.com/bendlang/bend/blob/f0cf0a7c573145501981c44d9c33e326185f008e/.github/workflows/repo-gate.yml
[20]: https://github.com/bendlang/bend/blob/f0cf0a7c573145501981c44d9c33e326185f008e/gates/safe.ts
[21]: https://github.com/bendlang/checker-eval/blob/854bdd852670a79f0fe40d665b0ca35e892125cc/README.md
[22]: https://github.com/wakamex/bend-bench/blob/40c8effdf7a282df2fd28ebb2cf35a60de7cea52/PERFORMANCE.md
[23]: https://github.com/wakamex/bend-bench/blob/40c8effdf7a282df2fd28ebb2cf35a60de7cea52/STARTUP_SCALING.md
[24]: https://github.com/wakamex/bend-bench/blob/40c8effdf7a282df2fd28ebb2cf35a60de7cea52/BEND_2_0_26.md
[25]: https://www.korulang.org/benchmarks
[26]: https://github.com/korulang/koru-benchmarks/blob/f84914374d06192624b8ea3c8ce52553e14effe2/bench.sh
[27]: https://github.com/bendlang/bend/blob/f0cf0a7c573145501981c44d9c33e326185f008e/bench/runtime/_pin_/apple_m4_max.txt
[28]: https://github.com/bendlang/bend/releases/tag/v2.0.38
[29]: https://github.com/bendlang/bend/blob/f0cf0a7c573145501981c44d9c33e326185f008e/bend2/bendtt.lean
