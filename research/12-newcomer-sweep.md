# 12: Newcomer sweep, 2025–2026

Research date: 2026-10-19. Scope: languages and language projects that
appeared or changed significantly in 2025–2026, checked for ideas the Tide
design (`design/01-design.md`, draft 3.1) is missing. Every factual claim has a
URL. Lines marked **Inference** are my reasoning, not something a source
states. Sources here use the name Tide; older research files say Ferox.

Already covered elsewhere, so not repeated here: Zero, Vera, MoonBit, Unison,
Darklang, and the size of the agentlanguages.dev catalog (`research/06`);
Mojo 1.0, Carbon's schedule, MoonBit's beta, and Rue's bootstrap story
(`research/09`); the competitor table in `reviews/round1-strategy.md`
(Rust goals, Verus, Kani, Dafny→Rust, Bend 2, Zig 0.16, Swift 6.3, Hylo).
This sweep goes one level deeper. It asks what each newcomer's strongest idea
is and whether Tide has it.

---

## Question

Which 2025–2026 languages and tools carry a strong idea that Tide lacks, in
five areas?

1. typed embeddable languages (Lust and its peers);
2. agent-first and LLM-first languages beyond Zero;
3. lighter-Rust and systems languages, and Rust's own 2026 direction;
4. verification-oriented and capability-oriented languages and tools;
5. deterministic simulation, replay, and review tooling.

For each idea the question is: does Tide have it, and should Tide adopt, adapt,
or ignore it?

---

## Projects reviewed

Verdict confidence: H high, M medium, L low.

| Project | What and state | Strongest idea | In Tide? | Verdict |
| --- | --- | --- | --- | --- |
| Lust (lust-lang.dev) | Typed Lua-like embeddable language in Rust with a dynasm trace JIT; 9 stars, "heavily WIP" [L1][L2] | Gas budget on compiled loops; typed extern stubs | Fuel for `const fn` only (D6) | Ignore (H) |
| Roto | NLnet Labs' compiled, typed scripting language for Rust hosts; Cranelift JIT; hot reload [R1][R2] | Host registers the only types and functions a script can use | No embedding story | Open question |
| Mog | Typed embedded language for LLM-written plugins; ~3,200-token spec; AOT native; Show HN, Mar 2026 [M1] | Agent-written code inherits the host's permissions | Capabilities, but no plugin host | Open question |
| Hale | Concurrent systems language for model authors; v0.22 (30 Sep 2026), v0.25.1 (18 Oct 2026); 41 stars [H1][H2][H3] | Transitive effect certificates with budgets; architecture claims with countermodels; union-only policy sets | Partly (D7 `Pure`, D17 policies) | Adapt (M) |
| Thermite | Contract-first language that emits Rust; `req`/`ens`/`fx`; Verus, bounded checking, Lean [T1] | Per-clause assurance ladder; `fx` compiled to a seccomp allowlist | Ladder yes (D11); sandbox no | Adapt the sandbox (M) |
| Boruna | Deterministic, capability-safe workflow VM; v0.2.0 [B1] | Hash-chained evidence of effects, model responses, and approvals | Replay (D17); no chain | Ignore for now (M) |
| Plasm | Agent language over typed API graphs; "plan before live HTTP" [P1] | Dry-run plan reviewed before live effects | No | Adapt (L–M) |
| reqlan | Named requirements linked to symbols and tests; dangling link is an error [Q1] | Requirement traceability checked by the compiler | No | Adapt as process (M) |
| Modula-9 | Wirth-lineage language emitting C; self-hosted v0.4.1 [N1] | Differential execution against the code it replaces; goldens the gate never regenerates | Origin cross-check only (D12) | Adapt as process (M) |
| NanoLang | Jordan Hubbard's LLM-target language; mandatory `shadow` tests; v3.3.7 [K1][K2] | Coq-proved core: progress, preservation, determinism | No mechanized core | Adapt (M) |
| AILANG | Capability-effect language "written autonomously by AI agents" [A1] | Multi-model benchmark on every release | E1 at gates only (D0) | Adapt (M) |
| CodeSpeak | Andrey Breslav's spec-as-source system; LLM compiles Markdown specs; alpha [C1][C2] | Specs versioned, code regenerated | Humans own specs (GOVERNANCE) | Ignore (H) |
| Rue | Klabnik's affine-ownership language, "Implemented primarily by Claude"; 4,870 commits [U1][U2] | Spec rules traced to tests (989 of 997) | Partial rule IDs, no gate | Adopt (H) |
| lust (Theo Browne) | Checker and compiler for a subset of valid Rust with an incremental daemon; dev builds emit C for `tcc`; 2,736 commits on `next` (Oct 2026) [LT1][LT2][LT3] | Warm edit checks 60–250x faster than `cargo check`, by rechecking only the edited body | Same cuts and edit-loop target (D12); Tide is not valid Rust | Confirms the speed target; speed is not a moat (H) |
| Valen | Vale's successor; group borrowing; runs rustc as a library (Sep–Oct 2026) [V1][V2] | Direct calls into generic Rust with borrow checking across the boundary | C-ABI shims (D13) | Spike in v1 (L) |
| Hylo, Carbon | Hylo: no 2026 release [Y1]; Carbon: safety strategy rewritten (p5914), 0.1 late 2026 [Y2] | Value semantics; fail-stop safety modes | Origins chosen instead (D2) | Ignore (H) |
| Swift 6.2 | `-strict-memory-safety`, `unsafe` expressions (SE-0458) [S1]; lifetime dependencies still experimental [S2] | Per-expression audit marker | Trust holes (D11) | Ignore (M) |
| Jai, Odin, C3, Hare, Inko | Jai closed beta [J1]; Odin implicit `context` [O1]; C3 0.8 with `@require`/`@ensure` [C3][C4]; Hare 0.26, freeze at 1.0 [E1][E2]; Inko 0.18 hybrid ownership [I1][I2] | Compile-time execution; implicit allocator context; doc-comment contracts | Rejected (D6, D7) or present (D10) | Ignore (H) |
| Rust 2026 goals | 80 goals; contracts, view types, reflection and comptime; none on determinism or capabilities [RG1][RG2][RG3] | View-type syntax `Foo.{ bar }` | Tide's `&mut self.{pos}` (D2) | Track for D1 (M) |
| Move borrow checker in Lean | Ilya Sergey; soundness proved with Claude Code; OOPSLA'26 [W1] | AI-assisted mechanized soundness in under a month | No | Adopt (M–H) |
| capsec, capcheck, cargo-capslock | Rust capability crate (Mar 2026) [X1]; Go capability lockfile (Aug 2026) [X2]; Capslock → seccomp (FOSDEM 2026) [X3] | Analysis output compiled into a kernel sandbox | Lockfile yes (D13); sandbox no | Adopt (M–H) |
| WASI 0.3 | Ratified 11 Jun 2026; completion-based async in the Component Model [Z1] | Runtime-enforced capability imports | Not a target | Adapt (M) |
| TigerBeetle protocol-aware DST | Simulator checks per-replica safety and liveness (Aug 2026) [D1] | Inside-out invariants across nodes | Node inspection only (D15) | Adapt (M) |
| vibesys `vs-sim`, eta, celld | DST by default with timing-dependent tests rejected [D2]; Erlang DST [D3]; Workers runtime DST [D4] | Make nondeterministic tests impossible to write | Yes, by construction (D7, D15) | Confirms Tide |
| Antithesis Hegel and skills | Hypothesis-style PBT for Rust and more [D5]; agent skills [D6] | `sometimes` assertions [D7] | Shrinking yes; reachability goals no | Adapt (L–M) |
| Undo MCP, rr-mcp | Time-travel debugging for agents [D8][D9] | Backward-only navigation; "where did this value come from" | Queryable traces (D17), no query design | Adapt (M) |
| Golem, Obelisk, Continuum | Durable execution by replaying an effect log [D10][D11][D12] | Resume after crash from the effect journal | Recording (D17) | Open question |
| RealDiff | Runtime behavior diff for PRs (Aug 2026) [D13] | First changed behavior per call tree | Static exercised authority (v1) | Adapt (M) |

---

## Findings (cited)

### F1. Typed embeddable languages: a crowded niche Tide is not in

- **Lust** pairs "Lua's ergonomics" with nominal static types, `Option`, and
  `Result`. It has a trace JIT on dynasm-rs and an embedding crate. The
  repository has 9 stars and calls itself "heavily WIP" [L1][L2]. The only
  resource-control feature is `VM::set_gas_budget` [L2].
- **Roto** compiles typed scripts through Cranelift for Rust hosts. Registered
  Rust types must be `Clone` or `Copy`, and only concrete types can be
  registered [R1][R2].
- **Mog** targets agents that extend themselves. The guest has "no direct
  syscalls, libc, or memory access"; the host supplies every callable function.
  The author's framing: "the less expressivity you trust them with, the
  better." He concedes Mog "will likely not yet beat existing systems at
  basically anything" [M1].
- **Inference:** all three treat "the script can only call what the host
  registered" as the security model. Tide already has that model in its type
  system (D7) and an interpreter (D12), so Tide could serve as a plugin
  language. That is a product question, not a missing mechanism.

### F2. Agent-first languages beyond Zero

`research/06` counted the catalog. The entries below have ideas that research
did not examine.

- **Hale.** Functions carry `@effects(...)` and `@budget(alloc_per_call = 0)`.
  Its ten effect classes include alloc, recursion, block, and spawn. The
  compiler "proves the declaration across everything the function can reach. A
  violation names the path" [H2]. Architecture claims such as
  `forbid reaches(planners, audit) via { calls }` print "a minimal
  countermodel in your own names." Shared claim sets ("constitutions") compose
  "by union only, so an environment can add a rule and cannot weaken one" [H2].
  A call through a function value is "unknown", and "a hard contract fails
  closed on it" [H2]. Hale has no whole-program simulator. Its README says "no
  garbage collector and no borrow checker" and ships `hale mcp` [H3].
- **Thermite.** It records the engine used for each clause and places it on a
  ladder from L0 (`#[slag]` escape) to L4 (Lean-reconstructed). The project
  result is "the minimum across the functions in scope." Forge "derives a
  seccomp allowlist for hosted executables" from `fx`, per effect verb, not
  per resource [T1].
- **Boruna** chains step inputs, outputs, model responses, and approvals into a
  Merkle structure and claims bit-identical re-runs from the bundle. It has one
  author and 34 commits [B1].
- **Plasm** lowers programs to an execution DAG reviewed as a dry run: "Plan
  before live HTTP" [P1].
- **reqlan:** "A dangling edge is a compile error. A rewritten paragraph is
  not." It cannot check that a linked test still matches intent [Q1].
- **Modula-9** validates ports against the code they replace. "Goldens recorded
  from an oracle run" are checked in, and "the gate that reads them never
  regenerates them, so it cannot pass by construction" [N1].
- **NanoLang** requires a `shadow` test block on every function. Its core has
  6,170 lines of Coq with 193 theorems, including determinism, and "zero
  axioms" [K2]. On HN its author called it "a total thought experiment" [K1].
- **AILANG** runs 33 tasks across 8 frontier models on every release [A1].
- **CodeSpeak** treats Markdown specs as source and generated code as an
  artifact. It is an alpha [C1][C2].
- **Evidence on language choice.** Discussing Dan Luu's 2026 eval, a MirrorCode
  author reported "little sign of inter-language differences in solve rates"
  across Python, C, Rust, Go, OCaml, and Ada, with Ada using about 25% more
  tokens [F1][F2]. **Inference:** this weakens the case that Rust spelling (D1)
  buys success on long tasks. It does not weaken the case that Rust spelling
  lowers the cost of learning a new language, which is what E1 measures.

### F3. Lighter Rust and systems languages

- **Rue** is "Designed by Steve Klabnik" and "Implemented primarily by Claude";
  Codex also contributes. It emits x86-64 and ARM64 directly and reports "Spec
  rules traced: 989 of 997" with 4,177 spec tests [U1]. The README says "The
  test suite tracks normative specification coverage and known implementation
  gaps explicitly" and lists differential-oracle, fuzz, and sanitizer suites
  [U2].
- **Valen** (Vale's successor) runs rustc through `rustc_driver` so it can
  query Rust types and generics directly; the two compilers coordinate
  monomorphization. This needed about 100 lines of rustc patches, "definitely
  not upstreamable" [V1]. The second post: "Valen has group borrowing (except
  for closures), and it borrow checks across the boundary." Rust signatures do
  not say where returned references point, so Valen adds "wildcard descendant
  paths" [V2].
- **lust**, by Theo Browne (no relation to the Lua-like Lust in F1), is "an
  experiment" that compiles "a subset of Rust": "Every program lust accepts
  must be valid Rust (rustc 1.99, edition 2024)" [LT1]. Theo's post calls it
  "a subset of Rust that compiles 100x faster" [LT3]. The README's claim is
  narrower: "a warm lust daemon checks a one-line edit inside a function
  body about 60x to 250x faster than cargo check at the median", which it
  calls "the best case"; signature edits take "4 to 10 s at p95", and "A
  cold check is about 2.5x faster than cargo check with the same features"
  [LT1]. It rejects proc macros, derives other than the built-in ones, and
  generic associated types on purpose; `async` and `unsafe` are "Not
  supported yet". "Most crates from crates.io will not compile", "It is not
  a general Rust replacement", and lust-built binaries run 3.8x to 4.9x
  slower than Cargo debug builds [LT1]. These figures are self-reported; this
  sweep did not rerun them.
  **Inference:** Tide's edit-loop target (D12) is reachable on a checked
  subset of Rust without a new language. Fast builds support Tide's thesis
  but do not distinguish it; the guarantees of D7, D15, and D11 do. A
  checked subset with a capability std might also carry much of D7 and D15
  while staying valid Rust (open question 7).
- **Hylo** has no 2026 release; its 2025 ECOOP talk calls the design "a
  coherent system" [Y1]. **Carbon**'s p5914 requires temporal, spatial, and
  type errors to be rejected at compile time or caught with fail-stop
  behavior [Y2].
- **Swift 6.2** shipped opt-in strict memory safety: an `unsafe` expression
  marker and `@unsafe`/`@safe` attributes that add diagnostics without
  changing semantics [S1]. Lifetime dependencies are still a pitch [S2].
- **Jai** is still in closed beta [J1]. **Odin** passes "a pointer to the
  current context as an implicit additional argument," and allocation uses
  "the current context's allocator" [O1]. That is the ambient channel D7
  rejects. **C3**, whose core language is settling in 0.8 [C3], has had
  `@require`/`@ensure` contracts since 0.6 [C4]. **Hare** (0.26, Feb 2026)
  plans to freeze at 1.0 [E1][E2]. **Inko** pairs single ownership with
  runtime reference counting [I1][I2].

### F4. Rust's 2026 direction

- The 2026 goals list has 80 entries, including "Contracts: primitive
  ownership assertions", "View types experiment", "reflection and comptime",
  "Const Traits", "crates.io hardening against supply chain attacks", and
  "Stabilize and model Polonius Alpha". None concerns determinism or
  capabilities [RG1].
- `core::contracts` is still a nightly-only experimental API with
  `requires`/`ensures` attributes [RG2].
- Nightly has an experimental `view_type` macro written `Foo.{ bar }`. In lang
  team notes Niko Matsakis says the form is `self.{messages}` but "not final"
  [RG3][RG4].
- **Inference:** Tide's `fn advance(&mut self.{pos})` (D2) already follows
  Rust's direction. If Rust settles a different spelling, D1 says Tide adopts
  it. Rust is absorbing contracts and compile-time reflection, as the strategy
  review predicted. Ambient authority and simulation remain outside its plans.

### F5. Verification and capability tooling

- **Mechanized soundness has become cheap.** Sergey proved type soundness for
  Move's new borrow checker in Lean, with "zero axioms or `sorry`
  declarations." The work took about 27 working days and produced about 39,000
  lines. Claude Code wrote most of the proofs, and the author estimates 5–6
  months without AI. Caveats: "Novel proof strategies remain beyond AI's reach
  (for now)," and the call rule alone took nearly two days of near-continuous
  runs. The encoding was validated with conformance tests drawn from the
  production compiler's suite [W1].
- **Specs are the bottleneck.** Lahiri: "there is no oracle for specification
  correctness other than the user." He proposes lightweight tests that
  "disambiguate likely misinterpretations" [W2]. Kleppmann predicts AI will
  bring verification into the mainstream and that "the challenge will move to
  correctly defining the specification" [W3].
- **Capability tooling is spreading to existing languages.**
  - capsec provides `CapRoot`, `#[capsec::requires]`, and wrapped `std` modules
    for Rust [X1].
  - capcheck commits a Capslock-based capability lockfile for Go and fails CI
    when "a dependency (or your own code) gains the ability to do something it
    couldn't before" [X2]. deps.dev found in 2024 that "less than 2% of version
    updates" add a capability [X4].
  - The Rust Foundation's cargo-capslock feeds seccomp profiles "so that
    updates cannot quietly add new attack surface" [X3].
- **Kernel enforcement.** Landlock lets an unprivileged process restrict file
  hierarchies and, from ABI v4, TCP bind and connect to allowed ports [X5].
  seccomp cannot see io_uring operations (`research/10`, F6).
- **WASI 0.3.0** was ratified on 11 June 2026, moving async into the Component
  Model's canonical ABI [Z1].

### F6. Simulation, replay, and review tooling

- **Protocol-aware DST.** TigerBeetle's VOPR asserts "on every commit" that
  committed requests match across replicas. It checksums storage across
  replicas and checks two liveness properties: local recovery without
  coordination when the WAL is intact, and global repair of missing blocks
  [D1].
- **DST by default.** A UW vibesys roadmap issue (10 Oct 2026) would make a
  timing-dependent test "very hard, ideally impossible, to write" [D2]. eta
  brings DST to Erlang [D3], and celld's simulator found unknown bugs in a
  Workers runtime [D4]. **Inference:** Tide gets this by construction; these
  projects build it per codebase.
- **Antithesis** released Hegel, Hypothesis-style property testing starting
  with Rust [D5], and agent skills that turn "the property catalog into SDK
  assertions" [D6]. Its `sometimes` assertions check that a condition holds at
  least once, which shows whether tests reach meaningful scenarios [D7].
- **Time travel for agents.** Undo's MCP server "does not expose forward
  stepping, `continue` or similar forward-only operations," because "an LLM
  offered both forward and reverse navigation tends to overwhelmingly prefer
  forward navigation, which leads to worse analyses." It also drops
  breakpoints. Value tracing answers "where did this value come from?" No
  measurements are published [D8]. rr-mcp exposes reverse execution to agents
  [D9].
- **Durable execution.** Golem's oplog is "an append-only record of inputs,
  messages, effects, and decisions, powering observability and replay" [D10].
  Obelisk forbids workflows from reading ambient host state so that replay
  holds [D11]. Continuum scans Elixir workflow code at compile time for clock,
  randomness, and I/O [D12].
- **Behavior diffs.** RealDiff builds both revisions, runs the tests, and
  "reports the first changed behavior in each call tree," including files the
  PR never touched. HN readers could not tell exactly how it works [D13].

---

## Ideas Tide missed (ranked)

**1. Enforce declared authority at the kernel boundary too (D7, D13, D11;
v0 on Linux).**
Today Tide's guarantee is language-level only, so `unsafe`, `ffi`,
`#[trusted_pure]`, imported Rust crates, and compiler bugs can bypass it.
Thermite [T1] and cargo-capslock [X3] already compile capability analyses into
seccomp. Tide knows more than either: `main` attenuates `Root` explicitly
(`root.fs.dir(...)`, `root.net.only([...])`).
- Proposal: consuming `Root` applies a matching Landlock domain (paths and
  TCP ports [X5]) plus a seccomp filter for syscall classes no capability
  uses. A WASI 0.3 component target maps capabilities onto imports [Z1]. The
  round-1 strategy review proposed the WASI half; D12 never adopted it.
- The trust surface marks each grant as language-enforced or also
  kernel-enforced.
- Limits: the kernel bounds the whole process, not each package; io_uring
  escapes seccomp (`research/10`); no macOS equivalent in v0.
- Confidence: medium-high on value, since E3 and the security pitch get
  defense in depth for free; medium on cost.

**2. Mechanize the core calculus with agent help (D2, D7 `Pure`, D15; a D0
gate).**
The replay promise rests on "an authority-free function cannot observe
nondeterminism." The memory model rests on the origin calculus, which round 1
found unsound in draft 1. The only checks today are the rustc cross-check and
reviews.
- Sergey's month-long Lean proof [W1] and NanoLang's determinism theorem [K2]
  show the cost has collapsed.
- Proposal: build a Lean model of v0 origins, drop check, and the `Pure`
  noninterference theorem, plus an executable checker tested against Tide's
  corpus.
- Confidence: medium-high. Risk: noninterference is less standard than
  progress and preservation, which is where Sergey says AI stalls.

**3. Behavior diff from deterministic traces (D11, D15).**
Exercised authority is static and arrives in v1. RealDiff shows demand for
runtime behavior diffs [D13], and Tide can make them noise-free.
- Proposal: `tide review <base>` replays protected tests and a fixed seed set
  on base and head, diffs the capability event traces, and reports the first
  divergence per test. Examples: a new host contacted, a new path written, an
  extra `sync`.
- Same seed and same toolchain means every divergence comes from the change,
  except unmodeled foreign effects, which the build already lists.
- Confidence: medium. E3 should measure whether it beats the static diff.

**4. Normative rule IDs and conformance traceability as the compiler's oracle
(D0, D14, GOVERNANCE; new D18).**
GOVERNANCE has agents write the code and humans write the specs; Rue shows
what keeps that honest. It reports 989 of 997 spec rules traced, plus
differential-oracle suites [U1][U2]. Modula-9 adds goldens that no gate
regenerates [N1]; reqlan makes dangling links errors [Q1].
- Tide's chapters label only some rules (`SINK-`, `RUST-`, `PAR-`).
- Proposal: every normative rule gets an ID, and each ID needs passing and
  failing tests. CI publishes the traced count, and golden outputs are
  `#[protected]`.
- Once the interpreter exists, E1 runs on every release with at least two
  models, as AILANG does [A1].
- Confidence: high. Cost: low.

**5. A plan root: dry-run a program against the real world (D15, D17;
new).**
Plasm reviews "plan before live HTTP" [P1]; Boruna gates effects on
approval [B1]. Every Tide effect is a capability call, and the simulator
already models filesystems.
- Proposal: `tide run --plan` gives `main` a root that reads the real world
  and captures writes, sends, and spawns into an overlay, then prints them as
  a plan. `--apply` reruns and aborts if the effect sequence diverges.
- This fits v0's targets (CLI tools, developer tooling) and agent-run tools.
- Limit: planning stops at the first write whose response matters, such as a
  network request.
- Confidence: medium-low.

**6. Backward-first replay queries for agents (D14, D17).**
D17 says traces are "queryable structured data" but defines no queries.
Undo's design keeps agents off forward stepping and breakpoints and adds
value provenance [D8].
- Proposal: `tide replay <manifest> --why <expr> --at <event>` returns the
  last write and the capability responses a value depends on. The interpreter
  computes it, and the agent interface offers backward queries only.
- Confidence: medium. The evidence is a vendor claim without data.

**7. Witness paths, reachability claims, and union-only policy sets (D17,
D11, D7).**
Hale prints countermodels, forbids reachability between components, and
composes policy sets that can only grow [H2].
- Proposal:
  - every authority or policy diagnostic carries a witness call path;
  - add `forbid reaches(a, b)` over the call graph next to D17's module-path
    rules;
  - let a workspace or organization policy file bind dependents, which may
    add rules but never weaken them.
- Confidence: medium.

**8. Contract review by disambiguating examples (D10, D11).**
D10 checks contracts for vacuity and runs mutation tests. Neither shows a
human what a changed contract now allows. Lahiri argues the user is the only
oracle [W2].
- Proposal: when a contract changes, `tide review` shows solver- or
  generator-produced inputs on which the old and new clauses disagree. The
  approval record covers those examples, not just the formula.
- Confidence: medium. Cost: low, given the property-test engine.

**9. Resource budgets as reported effects (D7, D11; `research/07` open
question 2).**
Hale certifies allocation, recursion, and blocking budgets transitively [H2].
`Pure` says nothing about allocation, yet storage engines and proxies are
v0 targets.
- Proposal: v1's exercised summaries also report allocates, recurses, and
  blocks for each public item. An opt-in `#[no_alloc]` check comes later.
  Allocation does not become a capability.
- Confidence: low-medium.

**10. Rust import through rustc as a library (D13, v1 spike).**
Valen calls generic Rust APIs and borrow-checks across the boundary [V1][V2].
D13's C-ABI shims reach only monomorphic, synchronous leaf APIs. Tide is
written in Rust, and its origins already map onto lifetimes (D12's
cross-check).
- Proposal: run a v1 spike that measures how much more of crates.io becomes
  reachable.
- Risk: Valen patches rustc internals; tracking nightly is a standing cost.
- Confidence: low.

---

## Direct competitors

No project found combines Tide's four pillars: Rust spelling, no ambient
authority, whole-program deterministic simulation, and a compiler-computed
trust diff.

- **Rust plus bolt-on tools** remains the strongest substitute.
  - Rust is absorbing contracts, view types, and compile-time reflection
    [RG1][RG2][RG3].
  - Capability lockfiles and kernel profiles now exist for Go and Rust
    [X2][X3].
  - Microsoft's research push to rewrite C and C++ with AI targets Rust
    [G1].
  - Rust's 2026 goals contain nothing on determinism or ambient authority
    [RG1]. Tide's moat holds, but the "trust diff" pillar is the one an
    existing ecosystem can approximate first.
- **Hale** is the closest single language: agent-targeted, systems-level,
  with transitive `@deterministic`/`@no_syscall` certificates, architecture
  claims, and an MCP server [H2][H3]. It is small (41 stars), has no Rust
  surface and no simulator, and changes surface quickly. Watch it. It is not
  yet a threat.
- **Thermite** overlaps D10 and D11: contracts, effects, Rust emission, and
  per-obligation assurance [T1].
- **Rue and Valen** occupy the lighter-Rust lane; Valen's Rust interop would
  undercut D13 if it matures [U1][V2].
- **lust** shows the lighter-Rust lane can be a checked subset of valid Rust
  at Tide's edit-loop speed [LT1]. It has no capability or determinism
  story. It is the strongest evidence that Tide's speed pillar is not a
  moat.
- **Boruna, Plasm, Golem, and Obelisk** sell replay and evidence for agent
  workflows, not systems code [B1][P1][D10][D11].

---

## Open questions

1. **Durable runtime.** Should a later-tier runtime journal capability
   responses and resume after a crash by replay, as Golem and Obelisk do
   [D10][D11]? Tide enforces the determinism they need by construction. Code
   upgrades during a journal are the hard part.
2. **Plugin host.** Should the interpreter be embeddable as a capability-safe
   plugin host for Rust programs (Mog, Roto, Lust; F1)?
3. **Per-package enforcement.** The kernel sandbox bounds a process. Is
   per-package enforcement worth process separation for `ffi` grants?
4. **Trace-diff noise.** Will legitimate changes drown the behavior diff? E3
   should test it.
5. **Liveness and reachability.** Should `tide sim` add a liveness phase
   (faults stop, then named progress must resume within a bound), as
   TigerBeetle checks per replica [D1], and `sometimes`-style goals [D7]
   that report rare states never reached?
6. **E1 thresholds.** If long-horizon solve rates barely differ by language
   [F1], should D0's E1 and E2 thresholds weight token cost and iteration
   count more than pass rate?
7. **New language or checked Rust subset.** lust accepts only valid Rust
   [LT1]. Could Tide's guarantees ship as a checked subset of Rust with a
   capability std, keeping rustc release builds and much of the crates
   ecosystem? What does not fit today: `requires`/`ensures` syntax (it would
   become attributes), `Pure`, `main(root: Root)`, overflow traps in release
   (a build setting), and v1's field-borrow declarations and evented
   runtime. 01's thesis says tools layered on Rust cannot give the
   guarantees; that holds for Rust as a whole, not yet shown for a checked
   subset.

---

## Sources

- [L1] https://lust-lang.dev/
- [L2] https://github.com/ArchangelJTW/lust-lang
- [LT1] https://github.com/t3dotgg/lust-lang (README, branch `next`, read 2026-10-19)
- [LT2] https://x.com/aigeeknews/status/2112186488318509385
- [LT3] https://x.com/theo/status/2110713238593908868
- [R1] https://blog.nlnetlabs.nl/introducing-roto-a-compiled-scripting-language-for-rust/
- [R2] https://roto.docs.nlnetlabs.nl/en/latest/overview/why_roto.html
- [M1] https://news.ycombinator.com/item?id=47312728 (read via https://hn.svelte.dev/item/47312728)
- [H1] https://agentlanguages.dev/languages/hale/
- [H2] https://hale-lang.org
- [H3] https://github.com/hale-lang/hale
- [T1] https://agentlanguages.dev/languages/thermite/
- [B1] https://agentlanguages.dev/languages/boruna/
- [P1] https://agentlanguages.dev/languages/plasm/
- [Q1] https://agentlanguages.dev/languages/reqlan/
- [N1] https://agentlanguages.dev/languages/m9/
- [K1] https://news.ycombinator.com/item?id=46684958
- [K2] https://agentlanguages.dev/languages/nanolang/
- [A1] https://agentlanguages.dev/languages/ailang/
- [C1] https://codespeak.dev/docs/programming-system
- [C2] https://www.i-programmer.info/news/98-languages/18737-from-kotlin-to-codespeak.html
- [F1] https://news.ycombinator.com/item?id=49245936
- [F2] https://arxiv.org/abs/2606.30182
- [U1] https://rue-lang.dev/
- [U2] https://github.com/rue-language/rue
- [V1] https://verdagon.dev/blog/golden-spike-reviving-vale-valen
- [V2] https://verdagon.dev/blog/boundary-memory-safety
- [Y1] https://2025.ecoop.org/details/plss-2025-papers/12/Designing-Hylo-a-programming-language-for-safe-systems-programming
- [Y2] https://docs.carbon-lang.dev/proposals/p5914.html
- [S1] https://github.com/swiftlang/swift-evolution/blob/main/proposals/0458-strict-memory-safety.md
- [S2] https://forums.swift.org/t/pitch-3-compile-time-lifetime-dependency-annotations/84968
- [J1] https://en.wikipedia.org/wiki/Jai_(programming_language)
- [O1] https://odin-lang.org/docs/overview/
- [C3] https://c3-lang.org/blog/0_8_0_the_core_language_is_settling/
- [C4] https://c3-lang.org/blog/new-contract-syntax-with-064/
- [E1] https://harelang.org/documentation/process/roadmap.html
- [E2] https://harelang.org/blog
- [I1] https://inko-lang.org/news/inko-0-18-1-is-released
- [I2] https://nlnet.nl/project/Inko/
- [RG1] https://goals.rust-lang.org/2026/goals.html
- [RG2] https://doc.rust-lang.org/nightly/core/contracts/index.html
- [RG3] https://doc.rust-lang.org/beta/core/macro.view_type.html
- [RG4] https://hackmd.io/pIbUgMQuQcGaibJcinOcEw
- [W1] https://proofsandintuitions.net/2026/03/18/move-borrow-checker-lean/
- [W2] https://arxiv.org/abs/2603.17150
- [W3] https://martin.kleppmann.com/2025/12/08/ai-formal-verification.html
- [X1] https://docs.rs/crate/capsec/latest
- [X2] https://pkg.go.dev/github.com/git-pkgs/capcheck
- [X3] https://fosdem.org/2026/schedule/event/QGCFDA-using_capslock_analysis_to_develop_seccomp_filters_for_rust_and_other_services/
- [X4] https://blog.deps.dev/capslock/
- [X5] https://docs.kernel.org/userspace-api/landlock.html
- [Z1] https://bytecodealliance.org/articles/WASI-0.3
- [D1] https://tigerbeetle.com/blog/2026-08-20-protocol-aware-dst/
- [D2] https://github.com/uw-syfi/vibesys/issues/1636
- [D3] https://erlangforums.com/t/eta-deterministic-simulation-testing/5893
- [D4] https://celld.dev/docs/engineering/deterministic-simulation-testing/
- [D5] https://antithesis.com/blog/2026/hegel/
- [D6] https://github.com/antithesishq/antithesis-skills
- [D7] https://antithesis.com/docs/properties_assertions/assertions/
- [D8] https://docs.undo.io/HowUndoAIWorks.html
- [D9] https://pypi.org/project/rr-mcp/
- [D10] https://learn.golem.cloud/concepts
- [D11] https://obeli.sk/docs/latest/concepts/workflows/
- [D12] https://elixirforum.com/t/continuum-a-library-for-durable-and-crash-resistant-workflows/75574
- [D13] https://github.com/issacnitin/RealDiff (HN: https://news.ycombinator.com/item?id=49464459)
- [G1] https://www.theregister.com/2025/12/24/microsoft_rust_codebase_migration/
