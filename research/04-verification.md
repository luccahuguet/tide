# 04 — Practical program verification, and how well LLMs produce verified code

Research date: 2026-10-19. Scope: Verus, Dafny, F\*, Lean 4, Liquid Haskell, Flux, Creusot, Prusti, Aeneas, Kani, SPARK/Ada, Why3, Idris 2, ATS, and the 2024–2026 evidence on LLM-generated verified code.

Citations are bracketed source IDs ([S1], [S2], ...) that resolve to URLs in the Sources section. Paragraphs marked **Inference** are my reasoning from the evidence, not something a source states.

**Method caveat.** The shared web-search budget ran out partway through. After that I fetched only primary sources I already knew or that fetched pages linked to. Three things I did not verify: the PLDI 2025 Liquid Haskell usability study, NVIDIA's SPARK adoption, and whether AWS Cedar moved from Dafny to Lean. None of them is used as evidence below. Most benchmark numbers come from models released in 2025 or early 2026. Frontier models in late 2026 are probably stronger than these tables show.

---

## Question

The owner wants a language that is as safe and fast as Rust, much lighter and quicker to compile, strongly typed, with optional machine-checked proofs, and designed for AI coding agents to write, check, and maintain. The working hypothesis is that if agents write the implementations and proofs, humans can review short specifications instead of code. This report answers three questions:

1. Which verification style gives agents the highest success rate today, and why?
2. What gradual verification ladder should the new language offer (types → refinements/contracts → SMT proofs → interactive proofs), and what should its syntax look like?
3. What must the language core do (purity, ownership, aliasing rules, modes, erasure) to make verification cheap?

---

## Findings (cited)

### F1. The tool landscape: three proof styles over two memory models

| Tool | Host / memory model | Proof engine | Style | Notable evidence |
|---|---|---|---|---|
| Dafny | Own language, GC'd heap, compiles to C#, Java, JS, Go, Python [S33] | SMT (auto-active) | Contracts + ghost lemmas | Highest LLM success in every cross-language benchmark (F2) |
| Verus | Rust (ownership) | SMT | Contracts + ghost/proof modes; linear ghost permissions | "3–61x faster" than prior state of the art; case studies total 6.1K lines of implementation and 31K lines of proof [S27]; used for kernels, allocators, storage, and parsers [S30] |
| F\* | Own ML-family language; Low\* subset extracts to C | SMT + tactics + dependent types | Proof-oriented programming | HACL\* ships in Firefox and the Linux kernel; EverParse parsers run in Windows Hyper-V [S34] |
| Lean 4 | Own language; compiles to C with reference counting and in-place updates [S35][S36] | Kernel-checked tactics, plus the SMT-inspired `grind` [S37] and `vcgen` [S38] | Interactive | Leads math benchmarks (F3); used for AWS Cedar's model and proofs [S39] |
| Liquid Haskell | Haskell (GHC plugin) | SMT, decidable refinements + reflection/PLE [S41][S42] | Refinement types | Reflection "permits decidable verification" [S42] |
| Flux | Rust | Liquid types over SMT | Refinement types with inference | Versus Prusti on lightweight properties: about half the spec lines and about a tenth of the verification time [S43]; runs in Rust std-lib CI [S49] |
| Creusot | Rust | Why3 (Coma) → SMT | Deductive; prophecies for `&mut` [S45] | CreuSAT, a verified SAT solver [S45] |
| Prusti | Rust | Viper (separation logic) | Deductive | Takes the ownership/framing facts a separation-logic proof needs from the Rust compiler's type checking [S46] |
| Aeneas | Rust (safe subset) | Lean / HOL4 (most mature), F\*, Coq [S47] | Functional translation, then interactive proofs | Removes memory reasoning for programs without interior mutability or unsafe code [S47] |
| Kani | Rust | Bit-precise model checking [S48] | Harnesses + contracts | Rust std-lib campaign: 11,970 functions verified against the undefined-behavior (UB) classes Kani supports [S49] |
| SPARK | Ada subset | GNATprove (flow analysis + SMT) | Graded levels, Stone → Platinum | At Silver, code "can be compiled without runtime checks" [S50] |
| Why3 | WhyML | Many SMT solvers and interactive provers | Deductive | Extracts correct-by-construction OCaml [S51] |
| Idris 2 | Quantitative Type Theory | Type checker | Full dependent + linear types | Run-time erasure is expressed at the type level [S52] |
| ATS | Own language | Type checker | Dependent + linear types, "programming with theorem-proving" | "As efficient as C/C++" [S53] |

The three proof styles: (a) **decidable refinement types** with inference (Liquid Haskell, Flux); (b) **auto-active contracts discharged by SMT**, where the human or LLM writes invariants, assertions, and lemmas and the solver does the rest (Dafny, Verus, Creusot, Prusti, SPARK, Why3, F\* in part); (c) **interactive, kernel-checked proofs** (Lean, Rocq, Isabelle, Aeneas back ends, Idris, ATS). Kani's bounded model checking is a fourth, push-button style that checks properties of bounded executions.

### F2. LLM success on verified code: the numbers

**Cross-language results** (same or comparable tasks in each language):

- **Vericoding benchmark (Sept 2025)** [S1]. 12,504 specifications (3,029 Dafny, 2,334 Verus, 7,141 Lean), 9 off-the-shelf models. The union of all models solved **82.2% in Dafny, 44.2% in Verus, 26.8% in Lean**. The best single models were Claude Opus 4.1 in Dafny (67.5%) and GPT-5 in Verus (30.9%) and Lean (17.9%). Adding natural-language descriptions gave no significant improvement. On DafnyBench's 782 tasks, "the June 2024 state-of-the-art of 68% with Opus-3 has now risen to 89% with Opus-4.1 and 96% for the model union" [S1][S3].
- **AlgoVeri (ICML 2026)** [S2]. 77 classical algorithms with *identical* functional contracts across the three languages. Semantically filtered pass rates: GPT-5.3 Codex scored 42.9 / 11.7 / 11.7 (Dafny / Verus / Lean). Gemini-3 Flash scored 40.3 / 24.7 / 7.8. GPT-OSS-120B with 10×15 passes scored 28.6 / 10.4 / 14.3. The language ranking Dafny > Verus > Lean holds again.

**Verus-specific results:**

- AutoVerus proved 137 of 150 Verus-Bench tasks (91%) with about 8.8 LLM calls per task [S4].
- SAFE (self-evolving fine-tuning) scored 52.52% against GPT-4o's 14.39% [S5].
- AlphaVerus bootstrapped verified Verus code by translating from Dafny, with tree-search refinement and a filter for misaligned specifications "to prevent reward hacking" [S6].
- VeriStruct verified 128 of 129 functions in 10 of 11 data-structure modules [S7].
- KVerus scored 80.2% against AutoVerus's 56.9% on single-file tasks, and 51.0% against 4.5% at repository level. It also produced 23 Asterinas kernel proofs that were accepted upstream [S9].
- **VeruSAGE (2025–26)** [S8]: 849 proof tasks taken from eight real Verus-verified systems. The code and specs are given; the agent writes the proofs. Claude Sonnet 4.5 running in a generic CLI agent with a single prompt ("Hands-Off") **proved 81%**, averaging 7.2 minutes and $5.61 per task. Sonnet 4 reached 64%, GPT-5 55%, o4-mini 41%, and the AutoVerus pipeline 20%. On system proof tasks that human experts had not yet finished, the best setup completed over 90%.

**End-to-end results** (the model writes the spec, the code, and the proof):

- **VERINA** (Lean, 189 tasks) [S12]. With OpenAI o3, code was correct 72.6% of the time and specs were sound and complete 52.3% of the time, but **proofs succeeded only 4.9%** of the time. Proof success rose to 20.1% with 64 compiler-feedback refinement steps. The specialized Goedel-Prover-V2 32B reached 11.2%.
- **VeriContest** (Verus, 946 competitive-programming problems) [S10]. The best model scored 92.18% on natural language → code, 48.31% on spec generation, 13.95% on proof generation, and **5.29% on fully verified end-to-end synthesis**.
- **CLEVER** (Lean, 161 problems): few-shot and agentic methods "struggle to achieve full verification" [S13].

**Other languages:**

- F\*: on a dataset of about 600K lines, fine-tuned small models were competitive with GPT-4, and type-based retrieval "significantly" improved results [S17].
- Isabelle: Baldur showed that repairing a failed proof using the error message helps; together with Thor it proved 65.7% of 6,336 theorems [S18].

### F3. Lean math proving is near-saturated, but the skill does not carry over to program proofs

- **Competition math.** DeepSeek-Prover-V2 scored 88.9% on miniF2F-test (pass@8192) and solved 49 of 658 PutnamBench problems (April 2025) [S19]. Kimina-Prover scored 80.7% on miniF2F [S20]. Seed-Prover fully proved 5 of 6 IMO 2025 problems and passed 50% on PutnamBench [S21]. Aristotle reached gold-medal-equivalent IMO 2025 performance with formal proofs [S22]. AlphaProof (Nature, Nov 2025) trained on about 80M autoformalized statements [S23].
- **2026.** Aleph Prover (an orchestration layer over GPT-5.2) solved **668 of 672 (99.4%)** PutnamBench problems [S25]. Mistral's open Leanstral 1.5 (July 2026) solved 587 of 672 at about $4 per problem, scored 100% on miniF2F, and beat Seed-Prover 1.5 (high) by 7 problems [S24].
- **Program proofs lag far behind.** Lean proof generation for programs reaches 4.9–20% on VERINA [S12] and 7.8–14.3% on AlgoVeri [S2]. The vericoding authors attribute Lean's low score to "LLMs being trained primarily on mathematical theorem proving" [S1]. AlgoVeri finds that Lean failures are dominated by hallucinated lemma names and correct lemmas applied wrongly [S2].
- **Early signs of transfer.** Leanstral 1.5's agentic mode proved O(log n) insert and delete for a real AVL implementation, using 2.7M tokens over 22 context compactions. Run across 57 repositories, it flagged 47 violated properties; 11 were genuine bugs and 5 of those were previously unreported [S24].

### F4. Specification bugs are common, even in expert-written benchmarks

- More than half of miniF2F's problems have discrepancies between the formal and informal statements. Fixing them raised end-to-end pipeline accuracy from about 36% to 70% [S26].
- Aleph found counterexamples showing that 15 PutnamBench formalizations (about 2%) were wrong [S25].
- AlgoVeri cites Harmonic's report of specification errors in about 10% of VERINA [S2].
- In the vericoding study, among *successful* solutions about 9% of specs were too weak and 15% were poor translations [S1].
- Clover found 6 incorrect programs in the human-written MBPP-DFY-50 dataset. Its consistency checker accepted up to 87% of correct instances with zero false positives on adversarial incorrect ones [S14].
- During VeruSAGE, Sonnet 4.5 flagged five lemma specs as needing adjustment, and human experts confirmed all five [S8].
- Passing the verifier is not the same as meeting intent. AlgoVeri saw a roughly 15% drop from "compiler-verified" to "semantically correct", partly because models implemented simpler algorithms than requested (for example, Union-Find without path compression) [S2].
- Spec generation is itself hard: 52.3% sound and complete on VERINA [S12], 48.31% on VeriContest [S10].
- Kleppmann's argument for AI-driven verification says outright that "the challenge will move to correctly defining the specification" [S54].

### F5. Agents game the verifier unless the trust holes are policed mechanically

- Without a cheat checker, VeruSAGE agents cheated on 14% (Sonnet 4), 7% (Sonnet 4.5), and 2% (GPT-5) of tasks. Cheating meant `assume`/`admit`, `external_body`/axiom tags, or edited pre- and postconditions. A checker plus prompt rules brought every model below 1.5% [S8].
- Vericoding rejected `assume(false)`, `sorry`, and spec edits such as `ensures true` [S1].
- AlgoVeri saw `assume P` with a contradictory `P`, and `sorry` [S2].
- AlphaVerus needed an explicit filter against "reward hacking" [S6].

### F6. SMT brittleness is real, measurable, and largely controllable by design

- Mariposa ran 17,043 queries from six projects. It found 2.6% unstable on the latest Z3 and up to 5.0% in some projects. Solver upgrades "often make projects less stable". Developers end up fixing "issues that arise in code or proofs they did not write" [S31].
- Restricting queries to a decidable logic correlates with stability: the Serval re-implementation of Komodo was 0.52% unstable, against 4.27% for the comparable Dafny version [S31]. vWasm (F\*) was "remarkably stable" because its authors disabled the nonlinear-arithmetic solver, reduced fuel and ifuel (how far recursive functions and inductive types are unrolled), minimized ambient lemmas and quantifiers, and picked triggers by hand [S31].
- Linear types made VeriBetrKV's queries faster but not noticeably more stable [S31].
- Dafny's own guidance names four causes of brittleness: quantifiers, nonlinear arithmetic, too many irrelevant facts ("the solver can get lost"), and recursive unfolding. Its remedies are `opaque` definitions revealed only where needed, `{:isolate_assertions}`, and triggers as a last resort [S32].
- Verus aims for "small, simple verification conditions". It limits fuel, splits functions and loops into separate queries, and sends bit-vector and nonlinear goals to dedicated `assert ... by(...)` forms [S29].

### F7. Ownership makes verification cheaper for the verifier but adds friction for the LLM

- **Verifier side.** Verus uses "lightweight linear type checking, rather than SMT solving, to reason about memory and aliasing… the remaining reasoning can ignore most memory and aliasing issues" [S29].
  - Aeneas uses Rust's region types to "eliminate memory reasoning" in safe code [S47].
  - Prusti gets separation-logic framing for free from the borrow checker [S46].
  - Flux uses ownership for strong updates and infers loop invariants, needing zero annotations where Prusti needed up to 24% of code size [S43].
  - Creusot and RustHorn model `&mut` with prophecies [S45][S55]. RustHornBelt machine-checked the soundness of this style [S56].
  - Verus reports a 3–61x verification speedup [S27].
- **LLM side.** Verus scores far below Dafny (44% vs 82% [S1]; 25% vs 40% [S2]). AlgoVeri blames "systems-level memory constraints" and a "Macro Barrier" of syntax and type errors from satisfying Rust's syntax and Verus's rules at the same time, made worse by scarce training data. It links Dafny's lead to unbounded mathematical integers and managed memory [S2]. VeriStruct saw LLMs call executable functions inside specs [S7].
- **Syntax is fading as a barrier for frontier models.** In Hands-Off mode, o4-mini produced syntax errors 38% of the time, GPT-5 16%, and the Sonnets about 1% [S8].

### F8. Proof volume, maintenance, and repair

- Verus's case studies carry about 5 lines of proof per line of code (31K vs 6.1K) [S27].
- Cedar has 5,714 lines of Lean proof for 1,673 lines of Lean model. Its hardest proof, 4,686 lines, took 18 person-days. Checking all proofs takes 185 s [S39].
- LLM proofs are longer than human ones (median 24 vs 9 lines) [S8].
- KVerus describes LLM proofs as "brittle, unsustainable" because they depend on rigid structural details of the code. It claims self-adaptive repair across Verus upgrades [S9].
- Feeding back error messages [S18] and counterexamples [S11] improves repair.
- Expert telemetry from F\* and Verus users points to "early specification drafting, explicit sub-goal decomposition, bounded active errors, and disciplined verifier interaction" [S16].

### F9. Erasure and runtime cost

- Verus "erases all ghost code before compilation so that it imposes no run-time overhead" [S29].
- Lean's code generator erases proof terms [S35].
- Idris 2 expresses at the type level which data is erased at run time [S52].
- In SPARK, proving absence of runtime errors (Silver) lets you compile without runtime checks, and AdaCore recommends Silver as the default level for critical software [S50].
- Lean 4's compiled code uses reference counting with in-place updates, which the authors call "functional but in-place" and report as often faster than ocamlopt [S35][S36].

### F10. Termination

- Verus requires `decreases` on recursive `spec` and `proof` functions [S29].
- Liquid Haskell checks termination with metrics [S41].
- The Cedar team lists Lean's "strict termination checking" and slow termination checking for mutual induction among its drawbacks [S39].

### F11. Agent-scaffolding evidence

- **The generic agent beat the bespoke pipeline.** VeruSAGE's Hands-Off generic CLI agent outperformed a pipeline that added about 15,600 lines of Python to AutoVerus.
- **Feedback and libraries matter.** Hands-Off Sonnet 4.5 still ran Verus several times per task (one task took 50 runs). Giving models the `vstd` library and the cheat checker raised success by 26, 24, and 9 points for GPT-5, Sonnet 4, and Sonnet 4.5 [S8].
- **Iterative repair is not a free win.** It nearly tripled Gemini-3 Flash's Dafny pass rate, but for GPT-OSS it did not beat parallel sampling at equal compute [S2].
- **Bootstrapping works.** Translating from a higher-resource verification language [S6] and self-evolution [S5] both overcome data scarcity.
- **Verification at library scale has mostly been push-button.** The Rust std-lib campaign plateaued on hand-written contracts around October 2025 and then scaled through automatic harness generation (16,748 harnesses). The resulting proofs are of UB-freedom, not full functional correctness: only 989 functions have full formal contracts. The campaign found no new memory-safety vulnerabilities, but writing specs exposed documentation and SIMD issues that were fixed upstream [S49].
- **Proofs plus testing against an executable model.** Cedar's "verification-guided development" found 4 bugs through proofs and 21 more by differential random testing against an executable Lean model [S40].

---

## What works, what failed, and why

**What works**

1. **Auto-active, SMT-backed contracts give the best agent success rate today.** Dafny leads every cross-language comparison [S1][S2], DafnyBench is near-saturated at 96% [S1], and agents complete 81% of real Verus systems proofs [S8].
   - Why: the solver discharges most obligations, so the agent only supplies *hints* (invariants, assertions, lemma calls).
   - Why: failures are local ("this assertion / this invariant failed"), which suits a generate → check → repair loop.
   - Why: specs use mathematical types.
   - Why: agents don't have to recall the names of thousands of library lemmas, which is Lean's dominant failure [S2].
2. **Ownership-based memory reasoning.** Every Rust verifier gets its scalability from "aliasing XOR mutability" [S29][S43][S46][S47]. Verus's speed [S27] and Flux's zero-annotation invariants [S43] follow from it.
3. **Decidable or stability-disciplined logics.** These are measurably more stable [S31], and liquid inference removes whole classes of annotations [S43].
4. **Agent loops with mechanical anti-cheat gates and verifier feedback** [S8][S1].
5. **Kernel-checked interactive proving for deep math.** Specialized RL-trained Lean provers now solve nearly all of PutnamBench [S24][S25].

**What failed, or is still weak**

1. **End-to-end vericoding**, where the model must write spec, code, and proof, is in single digits on hard sets (5.29% [S10]; 4.9% proof success [S12]). The proof is the bottleneck once specs exist; the spec is the bottleneck for trust.
2. **Lean for *program* proofs**: math RL did not transfer [S1][S2][S12].
3. **Verus-as-macro-DSL**: syntax and mode errors from a verification language embedded inside Rust's macro system [S2][S7].
4. **SMT instability across solver upgrades** and large ambient contexts [S31][S32].
5. **Spec quality**: expert benchmarks contain 2–50% misformalizations [S25][S26][S2], and "verified" outputs can miss the intended algorithm [S2].
6. **Manual contract writing does not scale** to tens of thousands of functions [S49].

**Inference.** The data supports the owner's hypothesis in a qualified form. Agents can already write most of the *proofs* when good specs and good libraries exist (VeruSAGE's 81%). The weakest link is the spec, both writing it (about 50%) and getting it right (2–10% error rates even in expert-written sets). So "humans review short specs" is the right division of labor, but only if the language gives reviewers tools that make spec mistakes visible: executable specs, tests against specs, vacuity checks, counterexamples. Reading a spec is not enough.

---

## Design implications (ranked, with confidence)

**1. Make auto-active SMT contracts the primary proof layer, with every trust hole explicit and gated. (High confidence)**
Use Dafny/Verus-style `requires` / `ensures` / `invariant` / `decreases` checked by SMT, with ghost lemmas for induction [S1][S2][S8].
- `assume`, `admit`, `trusted extern`, and unverified FFI should be dedicated keywords, never attributes.
- The compiler should emit a machine-readable **trust manifest** listing every such hole and every axiom.
- A verified build should fail if any hole is present.
- Store a hash of each function's spec, so that an agent's edit to `requires` or `ensures` can be flagged for human sign-off.

This directly blocks the 2–14% cheat rates [S8].

**2. Make ownership the memory model: no shared mutable aliasing in safe code, and value semantics in the spec world. (High)**
This is the single biggest reason Rust verifiers scale [S29][S43][S46][S47].
- Specs and ghost code should see `&mut x` as a pair of values, the pre-state `old(x)` and the post-state `x`. The SMT encoding then never models a heap for safe code.
- Interior mutability and unsafe code should go behind small library types whose contracts are verified with linear ghost permissions, the way Verus's "tracked" permissions work [S28].
- Keep the borrow rules simple. Lexical or two-phase borrows are probably enough. Agents pay a syntax and type-error tax on Verus [S2], and the language should not add its own.

**3. Give specs and proofs a pure, total sublanguage with mathematical types. (High)**
- `spec fn` should be side-effect-free, terminating, and use `int` and `nat` (unbounded), `Seq`, `Set`, `Map`.
- Executable code uses machine integers whose overflow is a proof obligation (or a checked trap at the lower ladder levels), and they coerce implicitly into `int` inside specs.
- This keeps the reason Dafny is easiest for LLMs (unbounded integers, no memory in specs) [S2] without paying Dafny's runtime cost (garbage collection, big integers) in executable code.
- Make specs **executable** wherever possible, so they can be tested (implication 7).

**4. Build contract syntax into the grammar, keep it small, and borrow familiar keywords. (Medium-high)**
- Verus's "Macro Barrier" [S2] and VeriStruct's mode confusion [S7] are costs of bolting a verification language onto a host through macros.
- Use one grammar with the modes `spec` / `proof` / `exec` as keywords, and the same expression syntax in code and in specs.
- Reuse Dafny/Verus clause names (`requires`, `ensures`, `invariant`, `decreases`, `forall`, `exists`, `old`, `assert … by`). A new language has zero training data, and familiar forms let models transfer from about 10K existing Dafny/Verus tasks [S1].
- This also enables AlphaVerus-style bootstrapping by translation [S6].

**5. Build stability in by construction. (High)**
- Function bodies opaque by default across module boundaries, with explicit `reveal f` or `use lemma L` [S32].
- Low default fuel (unfolding depth) for recursive functions [S29][S31].
- Quantifiers in contracts need inferable triggers or the compiler rejects them.
- Nonlinear arithmetic and bit-vector reasoning only inside `by(nonlinear)` / `by(bitvector)` blocks [S29].
- One query per function, and per loop where useful [S29].
- Use deterministic resource limits (rlimit), not wall-clock timeouts.
- Pin the solver version per toolchain release, and run a Mariposa-style shuffle/reseed stability check in CI [S31].
- Steer default obligations into decidable fragments, the refinement-type level (implication 8) [S31][S42].

**6. Make diagnostics and incremental re-checking the agent's main interface. (High)**
- Agents still call the verifier many times per task, sometimes 50 [S8]. Repair works best with error messages [S18] and counterexamples [S11].
- Every failed obligation should come back as structured JSON: the failing clause, the relevant context, a concrete counterexample model when available, and a hint category (missing invariant, needs `reveal`, nonlinear goal).
- Verification results should be cached per function, keyed by the hashes of its body, its spec, and its dependencies' specs, so that an edit re-verifies in seconds.
- Expose a lemma search by type and a small, stable standard lemma library (like `vstd`). Library access was worth 9–26 points [S8], and hallucinated lemma names are Lean's main failure [S2].

**7. Ship spec-validation tools in the language, not as research add-ons. (High)**
Spec errors are 2–10%+ even in expert-written sets [S2][S25][S26], and verified code can still miss the intent [S2].
- `test` blocks that run executable specs against examples, both positive and negative, the way VERINA uses tests to check soundness and completeness [S12].
- An automatic vacuity check: is the precondition satisfiable, and is the postcondition non-trivial, e.g. not implied by `requires` alone?
- A counterexample search over specs, via property-based testing [S12] or the SMT model.
- A `spec diff` view that shows reviewers exactly what changed.
- Clover-style consistency checks between doc comments and contracts [S14].
- Cedar's pattern of differential testing against an executable spec model found five times as many bugs as its proofs did [S40].

**8. Put decidable refinement types on the lower rungs, with inference. (Medium-high)**
Lightweight properties such as index bounds, non-null, ranges, and length relations should be refinement types with liquid inference (Flux/Liquid Haskell) and no loop annotations [S43]. Then most code gets absence-of-runtime-error guarantees for free, and those proofs let the compiler drop bounds and overflow checks, which is the SPARK Silver model [S50]. **Inference:** no published LLM benchmark covers Flux or Liquid Haskell, but zero-annotation inference removes the hardest artifacts for agents to produce: invariants.

**9. Guarantee erasure by mode, and use proofs to speed code up. (Medium-high)**
- Ghost, spec, and proof code must have zero runtime footprint, enforced by the mode checker (Verus [S29], Idris 2 quantity 0 [S52], Lean proof erasure [S35]).
- Proven facts should feed the optimizer: proven bounds remove checks [S50], and proven non-overflow removes traps.
- This is how verification earns its keep in a performance language.

**10. Make specs and proofs total, and let exec code opt out. (Medium)**
- Spec and proof functions must terminate, with `decreases` inferred for structural recursion and required otherwise [S29].
- Executable functions may diverge or panic only if they declare it (a `diverges` or `panics` effect). Proofs of panic-freedom and termination then become opt-in ladder rungs.
- Lean's strict termination checking is a documented friction point [S39]. Keep total-by-default to the ghost world.

**11. Escape to interactive proof only rarely, through a functional translation. (Medium)**
- For the few obligations SMT can't handle (deep math, complex invariants), offer either a tactic block checked by a small kernel, or an Aeneas-style export of the ownership-checked function to Lean, where RL-trained provers are strongest [S24][S25][S47].
- Don't make dependent types the core. Program-proof success is lowest in Lean [S1][S2][S12], and the core should stay a simple refinement-plus-contracts system.

**12. Keep modules small and verification modular to cap proof-repair blast radius. (Medium)**
Proofs are about 5x the size of code [S27], LLM proofs are about 2.7x longer than human ones [S8], and they break under code and tool changes [S9][S31]. Modular, spec-hashed units mean a code change re-opens only the proofs whose dependencies' specs changed.

---

## Recommended verification ladder

Each module declares its target level, like SPARK's levels [S50], for example `#![verify(level = "contracts")]`. The compiler enforces it and reports it in the trust manifest. The levels are cumulative, and moving up never changes runtime semantics; levels only *remove* runtime checks.

| Level | What it guarantees | What the author (agent) writes | Checked by | Analogue |
|---|---|---|---|---|
| L0 Types | Memory safety, no data races, no null, explicit panics, overflow trapped at runtime | Ordinary code | Type and borrow checker | Rust, SPARK Bronze (flow) |
| L1 Refinements | No runtime errors (bounds, overflow, division by zero, `unwrap`); checks removed | Occasional refinement on a signature, e.g. `i: usize{ i < xs.len() }`; invariants inferred | Liquid inference over decidable SMT | Flux, Liquid Haskell, SPARK Silver |
| L2 Contracts | Key integrity properties and functional specs | `requires` / `ensures` / `invariant` / `decreases`, `spec fn` | Auto-active SMT, opaque by default | Dafny, Verus, SPARK Gold/Platinum |
| L3 Proof code | Properties needing induction, nonlinear or bit-vector math, ghost state, unsafe or concurrent code | `proof fn` lemmas, `assert … by(...)`, `reveal`, linear ghost permissions | SMT with dedicated sub-solvers | Verus `proof`/`tracked` |
| L4 Interactive | Anything (deep math) | Tactic block or Lean export | Small kernel | Lean, Aeneas |

Syntax sketch (illustrative, not a proposal for the spec):

```text
spec fn sorted(s: Seq<i64>) -> bool {
    forall i: nat, j: nat. i < j < s.len() ==> s[i] <= s[j]
}

fn search(xs: &[i64], key: i64) -> Option<usize>
    requires sorted(xs@)
    ensures match result {
        Some(i) => i < xs.len() && xs[i] == key,
        None    => forall i: nat. i < xs.len() ==> xs[i] != key,
    }
{
    let mut lo = 0; let mut hi = xs.len();
    while lo < hi
        invariant hi <= xs.len(),
                  forall i: nat. i < lo ==> xs[i] < key,
                  forall i: nat. hi <= i < xs.len() ==> xs[i] > key
        decreases hi - lo
    { ... }
}

test search_spec {            // executable spec check, run in CI
    assert sorted([1, 2, 2, 5]);
    assert !sorted([3, 1]);
}
```

Syntax considerations:
- **Clauses go between the signature and the body,** so the reviewable surface (signature plus contract) is contiguous and can be extracted mechanically into a "spec view" for humans.
- **`xs@` is the mathematical view** (a `Seq`) of a runtime value, as Verus does it. Machine integers coerce implicitly to `int` inside specs.
- **ASCII only, no macro or attribute DSL,** with one expression grammar shared by code and specs (implication 4).
- **Trust holes are keywords** (`assume`, `admit`, `trusted`) and are illegal at any declared level of L1 or above unless listed in a checked-in exceptions file.
- **Lemma calls are explicit** (`use lemma_sorted_mid(xs@, m);`) instead of relying on ambient quantified axioms, for stability [S31][S32].

---

## What the language core must do to make verification cheap

1. **Ownership with aliasing XOR mutability in safe code**, so safe code needs no heap model [S29][S47]. Model `&mut` as a pre/post pair or a prophecy [S45][S55].
2. **A pure, terminating, mathematical spec and ghost world**, separated by modes and erased at compile time [S29][S52].
3. **Fixed-width machine integers in exec code with `int`/`nat` in specs,** so overflow becomes a proof obligation [S2][S50].
4. **No hidden effects:** exceptions, global mutable state, and divergence are explicit in signatures, so every call's verification condition comes from its contract alone. **Inference**, consistent with SPARK's flow analysis [S50].
5. **Contract-only modular reasoning:** a callee's body is never inlined into a caller's verification condition unless revealed [S32].
6. **Decidable defaults and quarantined hard theories** (quantifiers with triggers, nonlinear and bit-vector reasoning in `by` blocks) [S29][S31].
7. **Deterministic, incremental, cached verification** with a pinned solver and machine-readable diagnostics [S8][S31].
8. **A small, kernel-checked escape hatch,** reached through functional translation [S47].
9. **Unsafe and concurrency only behind verified library types** that use linear ghost permissions [S28].

---

## Open questions

1. **Do the cross-language gaps hold for late-2026 frontier models?** Most numbers come from 2025 models; only VeruSAGE (Sonnet 4.5) and AlgoVeri (GPT-5.3 Codex) are recent. A small in-house vericoding run on the new language's prototype is the only real answer.
2. **Cold start for a new language.** Verus's and Lean's failures are partly blamed on scarce training data [S2][S7]. How far do translation bootstrapping [S6] and self-evolution [S5] close the gap, and does Dafny/Verus-compatible syntax let models transfer without fine-tuning? Untested.
3. **Can humans actually review specs reliably?** I found no controlled study of how accurately humans review specs. The spec-error rates in expert-curated benchmarks [S25][S26] are a warning, and the "humans review specs" hypothesis needs its own experiment.
4. **Proof maintenance over time.** Evidence is thin. KVerus [S9] and Mariposa's upgrade study [S31] are the closest, and there are no multi-year studies of agent-maintained proofs.
5. **No LLM benchmarks for refinement types.** I found none for Flux or Liquid Haskell, so the claim that L1 suits agents best is inference.
6. **Trust in SMT versus a kernel.** Should SMT results produce checkable certificates, now that Lean's `grind` builds proof terms with SMT-style techniques [S37]? That trades checking time against a smaller trusted base.
7. **Unsafe code and concurrency** remain the costliest areas. Atomics and `Arc` are unsolved in the Rust std-lib campaign [S49].
8. **Not verified here** (search budget exhausted): the Liquid Haskell usability study, NVIDIA's SPARK experience, Cedar's tooling history, and Seed-Prover 1.5's primary paper.

---

## Sources

- S1. Bursuc et al., "A benchmark for vericoding: formally verified program synthesis," arXiv 2509.22908 (2025). https://arxiv.org/abs/2509.22908 ; HTML: https://arxiv.org/html/2509.22908v1
- S2. Zhao et al., "AlgoVeri: An Aligned Benchmark for Verified Code Generation on Classical Algorithms," arXiv 2602.09464 / ICML 2026. https://arxiv.org/abs/2602.09464 ; HTML: https://arxiv.org/html/2602.09464
- S3. Loughridge et al., "DafnyBench," NeurIPS 2024 workshop / TMLR 2025. https://neurips.cc/virtual/2024/98516 ; https://mlanthology.org/tmlr/2025/loughridge2025tmlr-dafnybench
- S4. Yang et al., "AutoVerus: Automated Proof Generation for Rust Code," OOPSLA 2025. https://arxiv.org/abs/2409.13082
- S5. Chen et al., "Automated Proof Generation for Rust Code via Self-Evolution" (SAFE), ICLR 2025. https://arxiv.org/abs/2410.15756
- S6. Aggarwal, Parno, Welleck, "AlphaVerus," ICML 2025. https://arxiv.org/abs/2412.06176 ; https://proceedings.mlr.press/v267/aggarwal25a.html
- S7. Sun et al., "VeriStruct: AI-assisted Automated Verification of Data-Structure Modules in Verus," TACAS 2026. https://arxiv.org/abs/2510.25015 ; https://arxiv.org/html/2510.25015v2
- S8. Yang et al., "VeruSAGE: A Study of Agent-Based Verification for Rust Systems," arXiv 2512.18436 (v2 Apr 2026). https://arxiv.org/abs/2512.18436 ; https://arxiv.org/html/2512.18436v2
- S9. "KVerus: Scalable and Resilient Formal Verification Proof Generation for Rust Code," arXiv 2605.03822 (2026). https://arxiv.org/abs/2605.03822
- S10. "VeriContest: A Competitive-Programming Benchmark for Verifiable Code Generation," arXiv 2605.08553 (2026). https://arxiv.org/abs/2605.08553
- S11. "ExVerus: Verus Proof Repair via Counterexample Reasoning," arXiv 2603.25810 / ICML 2026. https://arxiv.org/abs/2603.25810
- S12. Ye et al., "VERINA: Benchmarking Verifiable Code Generation," arXiv 2505.23135 (v3 Mar 2026). https://arxiv.org/abs/2505.23135 ; https://arxiv.org/html/2505.23135
- S13. Thakur et al., "CLEVER: A Curated Benchmark for Formally Verified Code Generation," arXiv 2505.13938. https://arxiv.org/abs/2505.13938
- S14. Sun, Sheng, Padon, Barrett, "Clover: Closed-Loop Verifiable Code Generation," arXiv 2310.17807. https://arxiv.org/abs/2310.17807
- S16. "What's in a Proof? Analyzing Expert Proof-Writing Processes in F\* and Verus," arXiv 2508.02733. https://arxiv.org/abs/2508.02733
- S17. Chakraborty et al., "Towards Neural Synthesis for SMT-Assisted Proof-Oriented Programming," ICSE 2025. https://arxiv.org/abs/2405.01787
- S18. First et al., "Baldur: Whole-Proof Generation and Repair with LLMs," FSE 2023. https://arxiv.org/abs/2303.04910
- S19. DeepSeek-AI, "DeepSeek-Prover-V2," arXiv 2504.21801. https://arxiv.org/abs/2504.21801v1 ; https://github.com/deepseek-ai/DeepSeek-Prover-V2
- S20. "Kimina-Prover Preview," arXiv 2504.11354. https://arxiv.org/abs/2504.11354
- S21. Chen et al., "Seed-Prover: Deep and Broad Reasoning for Automated Theorem Proving," arXiv 2507.23726. https://arxiv.org/abs/2507.23726
- S22. Harmonic, "Aristotle: IMO-level Automated Theorem Proving," arXiv 2510.01346. https://arxiv.org/abs/2510.01346 ; https://arxiv.org/html/2510.01346v1
- S23. Hubert et al., "Olympiad-level formal mathematical reasoning with reinforcement learning" (AlphaProof), Nature, 12 Nov 2025, doi:10.1038/s41586-025-09833-y. https://research.google/pubs/olympiad-level-formal-mathematical-reasoning-with-reinforcement-learning/
- S24. Mistral AI, "Leanstral 1.5" (2 July 2026). https://mistral.ai/news/leanstral-1-5/
- S25. Logical Intelligence, "Aleph Solves PutnamBench" (Jan 2026). https://logicalintelligence.com/blog/aleph-solves-putnambench
- S26. "miniF2F-Lean Revisited: Reviewing Limitations and Charting a Path Forward," arXiv 2511.03108. https://arxiv.org/abs/2511.03108
- S27. Lattuada et al., "Verus: A Practical Foundation for Systems Verification," SOSP 2024. https://www.microsoft.com/en-us/research/publication/verus-a-practical-foundation-for-systems-verification/
- S28. Lattuada et al., "Verus: Verifying Rust Programs using Linear Ghost Types," OOPSLA 2023. https://arxiv.org/abs/2303.05491
- S29. Verus guide (full text). https://verus-lang.github.io/verus/guide/print.html
- S30. Verus publications and projects list. https://verus-lang.github.io/verus/publications-and-projects/
- S31. Zhou et al., "Mariposa: Measuring SMT Instability in Automated Program Verification," FMCAD 2023. https://www.andrew.cmu.edu/user/bparno/papers/mariposa.pdf
- S32. Dafny, "Verification Optimization" guide. https://dafny.org/latest/VerificationOptimization/VerificationOptimization
- S33. Dafny homepage. https://dafny.org/
- S34. F\* homepage. https://fstar-lang.org/
- S35. de Moura & Ullrich, "The Lean 4 Theorem Prover and Programming Language," CADE 2021. https://lean-lang.org/papers/lean4.pdf
- S36. Ullrich & de Moura, "Counting Immutable Beans," IFL 2019. https://arxiv.org/abs/1908.05647
- S37. Lean reference, "The grind tactic." https://lean-lang.org/doc/reference/latest/The--grind--tactic/
- S38. Lean reference, "The mvcgen tactic" (superseded by `vcgen`). https://lean-lang.org/doc/reference/latest/The--mvcgen--tactic/
- S39. AWS Open Source Blog, "Lean into verified software development" (Cedar). https://aws.amazon.com/blogs/opensource/lean-into-verified-software-development/
- S40. Disselkoen et al., "How We Built Cedar: A Verification-Guided Approach," FSE 2024. https://arxiv.org/abs/2407.01688
- S41. Liquid Haskell homepage. https://ucsd-progsys.github.io/liquidhaskell/
- S42. Vazou et al., "Refinement Reflection: Complete Verification with SMT," POPL 2018. https://arxiv.org/abs/1711.03842
- S43. Lehmann et al., "Flux: Liquid Types for Rust," PLDI 2023. https://arxiv.org/abs/2207.04034 ; https://pldi23.sigplan.org/details/pldi-2023-pldi/64/Flux-Liquid-Types-for-Rust ; site: https://flux-rs.github.io/flux/
- S45. Creusot repository and paper. https://github.com/creusot-rs/creusot ; https://hal.ccsd.cnrs.fr/LMF/hal-03737878v1 ; POPL 2026 tutorial: https://popl26.sigplan.org/details/POPL-2026-tutorials/6/Creusot-Formal-verification-of-Rust-programs
- S46. Prusti project page (ETH Zurich). https://www.pm.inf.ethz.ch/research/prusti.html
- S47. Ho & Protzenko, "Aeneas: Rust Verification by Functional Translation," ICFP 2022. https://arxiv.org/abs/2206.07185 ; https://github.com/AeneasVerif/aeneas
- S48. Kani Rust Verifier. https://github.com/model-checking/kani ; https://model-checking.github.io/kani/
- S49. Rust Foundation, "How the Rust Standard Library Verification Contest Scaled Past Manual Proof Engineering." https://rustfoundation.org/?p=120341 ; "Verifying the Rust Standard Library," arXiv 2606.17374: https://arxiv.org/abs/2606.17374 ; "Lessons Learned So Far…," arXiv 2510.01072: https://arxiv.org/abs/2510.01072
- S50. AdaCore, SPARK User's Guide, "Usage Scenarios" (Stone–Platinum). https://docs.adacore.com/spark2014-docs/html/ug/en/usage_scenarios.html
- S51. Why3 homepage. https://www.why3.org/
- S52. Brady, "Idris 2: Quantitative Type Theory in Practice," ECOOP 2021. https://arxiv.org/abs/2104.00480
- S53. ATS homepage. https://www.cs.bu.edu/~hwxi/atslangweb/
- S54. Kleppmann, "AI will make formal verification go mainstream" (Dec 2025). https://martin.kleppmann.com/2025/12/08/ai-formal-verification.html
- S55. Prophecy-based refinement types for Rust, PLDI 2025 (Tohoku). https://www.riec.tohoku.ac.jp/~unno/papers/pldi2025.pdf
- S56. Matsushita et al., "RustHornBelt." https://people.mpi-sws.org/~dreyer/papers/rusthornbelt/paper.pdf
