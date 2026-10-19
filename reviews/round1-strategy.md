# Round 1 Review: Strategy and Adoption

Reviewer: adversarial adoption strategist.
Date: 2026-10-19.
Read in full: `design/00-first-principles.md`, `design/01-design.md`,
`research/00`, `research/01`, `research/06`, `research/09`. Consulted by
search: `research/04`, `research/07`, `research/10`. The 2026 competitor
state was checked on the web; sources are listed at the end. Where a fact
comes from secondary coverage, the text says so.

## Verdict

**request_changes**

The engineering direction is sound and backed by evidence. The strategy is
not. As written, the design competes with Bend on its headline claim, which
the owner ruled out. It also breaks its own central promise at the Rust
boundary, and it leads with the features Rust is already absorbing. All
three problems can be fixed without restarting the design.

Short answers to the brief:

- **Will the thesis make people switch, or will Rust absorb it first?** It
  won't make people switch as written. The one-line pitch bundles four
  features ("Rust's guarantees, Go's build speed, Dafny's proofs, and
  authority you can read"). Two of them already reach Rust users without a
  language switch:
  - **Proofs:** Verus, Kani, Dafny→Rust, and Rust's own contracts work.
  - **Lighter borrowing:** the Rust 2026 goals include Polonius alpha, a
    view-types experiment, and ergonomic ref-counting.

  A third, build speed, holds for checking but not for release builds,
  which go through rustc. What Rust cannot absorb are the *subtractive*
  guarantees:
  - no ambient authority anywhere in the dependency graph;
  - no code that runs at build time;
  - deterministic execution of the whole program;
  - a language small enough to load into an agent's context.

  Together these make the trust surface *sound* rather than approximate.
  The pitch should be built on them (B3).
- **Honest wedge.** The wedge is teams that run fleets of coding agents and
  need to merge agent-written CLIs, tools, and infrastructure components
  without reading every body. Deterministic simulation serves the stateful
  ones. The wedge is *not* verified parsers shipped as crates:
  - Dafny→Rust (AWS) and Verus already serve that niche, with more training
    data.
  - People who use a verified crate never learn its source language.
    HACL\* in CPython did not make F\* popular.
- **Rust emission plus Cargo: strength or trap?** It is a strength as a
  distribution channel and an exit door. It is a trap as an identity and as
  a semantic boundary. Three precedents frame the choice:
  - Kotlin worked because it shared a runtime with Java and fixed pain
    (nulls, verbosity) that Java fixed slowly.
  - CoffeeScript died when ES6 absorbed its sugar.
  - Scala paid for a semantic mismatch with its host, slow compiles, and
    binary breaks.

  "No lifetime annotations" is CoffeeScript-shaped value: Rust is absorbing
  it. Authority and determinism are TypeScript-shaped value: checks the host
  will never add. Emission stays a strength under three conditions:
  1. Ferox's borrow rules never accept more than stable rustc accepts, so
     emitted code stays safe Rust, rustc stays a real second checker, and
     "eject" stays real.
  2. Rust interop covers synchronous leaf crates with declared authority
     (B2, B4).
  3. Ferox is never marketed as "Rust without lifetimes".
- **Name.** There is no serious conflict for a language name. There is
  practical friction:
  - The `ferox` crate on crates.io is taken (a ray tracer, about 8.9K
    downloads).
  - `feroxbuster` (about 146K downloads) dominates searches for "ferox rust".
  - A tiny "Ferox web framework" exists (`ferox_macros`).

  Every Ferox package becomes a crate, so reserve a crate prefix now. Also
  finish the legal clearance that the March 2026 trademark review called
  for. Not blocking.

## Blocking objections

### B1. The design competes with Bend on its headline, and the Bend tandem cannot be built today

**Problem.** The owner's constraint is "do not compete with Bend."
`design/01` says Bend 2 "owns proved, massively parallel pure computation"
and that Ferox calls it for kernels. That misreads how Bend 2 positions
itself:

- **Bend's own claim.** Its site's tagline is "a fast language that blocks
  AI mistakes via proof", and the site says it "works best on the back-end."
- **Ferox's claim.** Ferox's pitch is "software that agents write and humans
  trust" with "Dafny's proofs". That is the same claim.
- **The research already said so.** `research/01` concluded that Bend 2 "is
  the closest peer and competitor to this project, not a backend for it."

D14 also cannot be built:

- Bend's C output "is one program with a `main`". Exporting definitions is
  "planned, not scheduled."
- "The proven core cannot be linked into a Rust application today."
- Bend has no U64 or F64 types.

**Evidence.** bend-lang.com tagline; Bend `WONTFIX.txt` (library target,
U64/F64 under "SOON"); `research/01` bottom line and F4.

**Smallest fix.**
1. Rewrite Positioning. Ferox is the effectful, authority-checked shell, and
   Bend is the pure, proved core.
   - Lead Ferox's pitch with authority, determinism, and Rust interop.
   - Describe Ferox contracts as SMT-backed checks on *effectful* systems
     code. Bend has no authority model, its IO is a single-threaded event
     loop, and its Base library has no TLS, HTTP, or DNS.
   - Explicitly leave proof-gated pure and parallel kernels, and GPU work,
     to Bend.
2. Replace D14 with a staged plan:
   - (a) Now: Bend programs run as out-of-process kernels behind the `Proc`
     capability, with a typed wire format.
   - (b) Offer to co-build Bend's native library target: a C header plus a
     lifecycle. Both projects need that in-process ABI.
   - (c) Later: map Ferox `requires` and `ensures` on pure functions to Bend
     laws, so a kernel can be specified in Ferox and proved in Bend.
3. Remove "Dafny's proofs" from the one-line pitch.

### B2. The trust surface lies at the Rust boundary

**Problem.** D7 promises three things:
- a signature shows everything a function can do;
- there are "no build scripts and no install hooks";
- the lockfile records each package's authority.

Positioning and D12 say Ferox calls Rust crates and lives inside Cargo.
These conflict:
- Rust crates reach ambient `std::fs`, `std::net`, `std::time`, and thread
  spawning.
- Many ship `build.rs` scripts and procedural macros, and Cargo runs them
  at build time.
- A real Ferox program that depends on `reqwest`, `rustls`, or `zstd-sys`
  therefore runs unchecked build code and has unchecked authority.
- Nothing in the design says how a call into Rust appears in the trust
  surface.

This is Kotlin's platform-type problem, applied to the whole thesis rather
than to nullability.

**Evidence.**
- Implication 6 of `research/09` said Rust dependencies' build scripts must
  "run inside a sandbox and are flagged as a capability". D7 dropped this.
- The crates.io malware `faster_log` and `async_println` stole keys at run
  time through ambient I/O (`research/09` F6).
- The Evidence section of `design/01` says library-level simulation fails
  "because dependencies reach ambient time, randomness, and threads." S2 had
  to override libc symbols to stop it (`research/07`).

**Smallest fix.** Treat every Rust crate as a trust hole whose authority is
declared, enforced on the Ferox side, and measured:
- **Declare.** The manifest names each crate's authority:
  `regex = { version = "1", authority = [] }`,
  `reqwest = { authority = ["net"], build = "sandboxed" }`.
- **Enforce.** Generated shims require the declared capability as a
  parameter (`&Net`), so Ferox callers pay for the authority at the call
  site.
- **Measure.** After the build, verify each claim with symbol-level analysis,
  as cargo-acl and cargo-capslock do. A mismatch such as
  `declared: net / observed: net, fs` is an error.
- **Sandbox the build.** Run `build.rs` and proc macros of Rust dependencies
  in a sandbox (cargo-acl does this with bubblewrap) and list them in the
  trust surface.
- **Report separately.** `ferox review` shows Rust dependencies in their own
  band: "authority claimed; verified by symbol scan, not by types."
- **Gate simulation.** The `sim` profile refuses to build if any reachable
  Rust crate has authority the simulator does not model. Deliverable 3,
  item 5 of `research/10` already proposes this coverage check.

### B3. The pitch leads with what Rust is absorbing instead of what Rust cannot have

**Problem.** The thesis is authority plus proved contracts plus a trust
surface, on a lighter Rust. Here is what reaches Rust users in 2026 without
a language switch:

| Ferox claim | Delivered on or inside Rust |
| --- | --- |
| No lifetime annotations, precise borrows | Rust 2026 goals include "Stabilize and model Polonius Alpha" and "View types experiment" (D2's methods that declare which fields they borrow). They also include "Ergonomic ref-counting". |
| Contracts the toolchain proves | Rust's experimental contract attributes. Kani verifies 16,000+ harnesses per change to Rust's std. Verus is in industrial use at Microsoft and Amazon, and agents finish 81% of real Verus systems proofs (VeruSAGE). Dafny→Rust (AWS) ran at most 22% slower than hand-written Rust on 9 of 10 benchmarks. |
| Go-like build speed | Rust goals "Promoting Parallel Front End", "Improve `rustc_codegen_cranelift` performance", and "Incremental Systems Rethought". They won't give 100x, but they blunt the claim, and Ferox release builds run through rustc anyway. |
| Authority | cargo-capslock (Rust Foundation, per binary, experimental), cargo-acl (per-crate API allow-lists, sandboxed build scripts), Cargo Scan, capsec. Zig 0.16 made `Io` a parameter of every I/O function. |
| Trust diff | Can be assembled from cargo-semver-checks, a capslock-style scan, unsafe counts, and Verus spec diffs. That is a three-month tool. |

A Rust shop's rational move is "Rust plus agents plus Verus plus cargo-acl
plus a trust-diff script." Meanwhile `design/01`'s own Evidence section says
two things:
- frontier agents already do best in Rust;
- most LLM errors in Rust are type mismatches and trait bounds, not
  lifetimes.

So "lighter Rust" mostly helps human writers, and the thesis says humans
are no longer the main writers.

Rust cannot remove ambient authority from `std`, macros and `build.rs` from
its ecosystem, or async coloring. It cannot run a whole program, including
its dependencies, deterministically. It cannot fit a 25K-token reference.
Only a new language can make these subtractive guarantees, and together
they make the trust surface sound.

**Evidence.** Rust 2026 goals list; Kani paper (2026); Verus FMCAD 2026
tutorial abstract; Dafny→Rust paper (POPL 2025 workshop); Alpha-Omega's
cargo-capslock update; `design/01` Evidence bullets 3–4.

**Smallest fix.**
- **Lead with one claim.** Rewrite Thesis and the pitch around a single
  quantified, falsifiable claim, the way Android's "1000x fewer
  memory-safety bugs" sold Rust. For example: "In Ferox, a signature is a
  sound upper bound on what the code and all its dependencies can do, and
  every program replays deterministically from a seed."
- **Demote proofs.** Make proofs the upper rungs of the trust ladder, not
  the lead.
- **Drop unmeasured comparisons.** Remove "Go's build speed" and "Dafny's
  proofs" from the one-liner until they are measured.
- **Make simulation a decision.** Promote research/07's high-confidence
  "deterministic simulation lives in std" to a numbered decision. Today it
  appears only in the example test (`Root::sim(42)`).

### B4. Services are where Cargo interop works worst, and the evented runtime is the costliest item in the design

**Problem.** Positioning lists services first. Rust's service ecosystem is
async: tokio, hyper, axum, tonic, sqlx, reqwest. D8 has no async and uses
stackful tasks. That causes two failures:
- **Calling in.** Ferox cannot call async Rust except by bridging to a
  separate executor.
- **Calling out.** To tokio, Ferox libraries exported to Rust are blocking
  calls.

Emission is also costly here. Child tasks borrow from their parent's frames
and unwind when cancelled. Expressing that in Rust needs unsafe
stack-switching code, in the style of corosensei or `may`. The unsafe share
of emitted code is exactly what the kill criterion in `research/09` watches
(more than 10% unsafe).

Meanwhile, an evented stackful runtime over io_uring, epoll, kqueue, and
IOCP is a multi-year job. Zig 0.16, with 244 contributors, shipped `std.Io`
with only the threaded backend production-ready. Reviews report that the
evented backend had io_uring and lacked kqueue.

**Evidence.** `research/09` §2 (async is deferred in glue mode); Zig 0.16
release coverage; `design/01` Open Question 4.

**Smallest fix.**
- **v0 target.** CLIs, batch and data tools, libraries, and
  simulation-tested infrastructure components, on the blocking and threaded
  runtimes. Services stay the v1 goal.
- **Interop scope.** Limit Rust interop to synchronous leaf crates: regex,
  serde_json values, compression, image, crypto primitives.
- **Native I/O.** The I/O stack is Ferox-native. Authority and simulation
  require that anyway, so state it as a feature.
- **Defer the evented runtime.** Make it a v1 decision with its own kill
  criterion: measure per-task memory and the FFI cost of stack switching
  before committing (`research/10` Open Questions).

### B5. The authority signal and the specs will decay into noise or false assurance

**Problem.** The trust surface stops meaning anything in practice in two
ways.

(a) **Capability bundling.** With explicit capability parameters, adding one
log line to a deep function changes every signature on the call path. Teams
respond the way Go (`context.Context`) and Zig 0.16 did. In Zig, `main`
receives one injected parameter that carries the allocator and the I/O.
Ferox teams will likewise pass one `ctx: &AppCtx` everywhere. Then every
signature says "can do anything", the diff says nothing, and reviewers
rubber-stamp. Logging is still undecided (`research/07` Open Question 3, and
`faster_log` attacked through a logging crate), and `design/01` mentions
only `dbg`.

(b) **Spec quality.** In `research/04`, the best model wrote correct specs
for only 48.31% of VeriContest problems, and even expert-written spec sets
have 2–10% error rates. Its conclusion: "Reading a spec is not enough." The
design asks humans to review contracts in the trust diff but gives them no
tool to tell a vacuous or wrong spec from a good one.

**Evidence.** `research/04` Inference paragraph and §2; `research/10` Open
Question 1 (capability plumbing "needs an experiment with real agent
sessions"); Zig 0.16 coverage.

**Smallest fix.**
- **Show what code uses.** Compute *exercised* authority (transitive and
  derived from bodies) and show it next to *declared* authority.
- **Lint over-grants.** Add a lint for over-granted capabilities whose
  machine fix narrows the parameter type, for example `&AppCtx` to `&Net`.
- **Decide logging now.** Either a structured, write-only `Log` sink routed
  by the binary that cannot reach the network and is tracked in the trust
  surface, or a capability. Justify the choice against `faster_log`.
- **Add spec tooling.** Extend D10 and D13 with:
  - vacuity and satisfiability checks;
  - generated concrete inputs that satisfy and violate each `ensures`;
  - a mutation score: how many mutants of the body the spec catches.

  Show the results next to each changed contract in the trust diff.
- **Run the experiment first.** Do the plumbing experiment `research/10`
  asks for before freezing D7.

### B6. No plan to falsify the thesis, and no v0 scope

**Problem.** `research/09` says the thesis is falsifiable: "if Ferox doesn't
beat Rust on agent cost for equivalent safety, the pitch fails." It asks for
a benchmark in Phase 1. `design/01` has targets for compile speed and none
for the thesis itself: no agent pass rate against Rust, no review-time
claim, no escaped-defect claim. It has no phasing either, so D1–D14 read as
a 1.0 specification.

The development loop with Rust dependencies is also undefined.
- `research/09` recommends Cranelift with C-ABI glue for dev builds.
- D12 chooses interpreter-first and defers the native backend, but never
  says how the interpreter calls a Rust crate.
- If the answer is "emit Rust and run rustc", the agent loop runs at rustc
  speed, against the latency principle in `design/00`.

**Evidence.** `research/09` recommended path §1 item 3 and Open Question 4;
the collapse of V's credibility (`research/09` F1); the Agent Loop section
of `design/00`.

**Smallest fix.**
- **Add D0: FeroxBench plus a review study, with go/no-go gates.**
  - *Month 3.* An agent given only the agent card reaches pass@1 within an
    agreed margin of Rust on a fixed task set.
  - *Month 6.* Tokens and wall-clock time per solved task are at or below
    Rust's.
  - *Month 9.* On seeded overreaching or malicious PRs, `ferox review`
    catches more in fewer reviewer minutes than Rust with cargo-acl and
    cargo-semver-checks.

  Publish negative results.
- **Add a v0 scope section** (see Cuts).
- **Keep rustc out of the dev loop.** The interpreter calls Rust through a
  prebuilt, cached C-ABI boundary library (glue mode from `research/09`), so
  editing Ferox code never runs rustc.

## Bold ideas worth adding

Ranked by leverage.

1. **Deterministic simulation as the headline feature.** `ferox sim` runs
   any `main(root: Root)` program under simulation:
   - several nodes, with simulated network faults, disk, clock, and
     scheduler;
   - a failure prints `seed + commit`, and that pair replays bit for bit.

   Why Ferox can promise this:
   - Rust can't, because its dependencies leak ambient nondeterminism.
   - Zig 0.16's `Io` gets closer, but Zig has no memory safety and doesn't
     stop direct syscalls.

   For agents the loop closes: the simulator finds a failure, the seed
   reproduces it, the agent fixes it, and the simulator confirms the fix.
   The killer demo is an agent-written replicated key-value store or job
   queue where simulation finds seeded bugs that tests miss. FoundationDB,
   TigerBeetle, and Antithesis each built a following on this technique.
   Ferox can make it a property of every program.
2. **Review by trust diff as a GitHub check from day one.** `ferox review`
   posts the trust diff on every PR:
   - signatures;
   - declared versus exercised authority;
   - contracts, with vacuity and mutation scores;
   - trust holes;
   - dependency authority.

   A change to the trust surface needs human approval. Under project policy,
   body-only changes with green checks can merge automatically. This is the
   product an agent-heavy team will pay for. Measure reviewer minutes.
3. **Run the review tool on Rust workspaces too (the `checkJs` on-ramp).**
   TypeScript won partly because its checker improved plain JavaScript in
   VS Code before anyone converted a file. On a Rust workspace,
   `ferox review` would give approximate results, labelled as such:
   semver-checks, capslock-style authority from symbols, and unsafe counts.
   On Ferox modules the results are labelled exact. Teams adopt the workflow
   first, then convert modules to turn "approximate" into "exact".
4. **Grow the ecosystem by agent porting, not only by FFI.** Bun ported 535K
   lines of Zig to Rust in 11 days for about $165K (`research/09` F2).
   - Use agents to port the most-used synchronous leaf crates to Ferox.
   - Use each original crate's test suite as the oracle, plus differential
     fuzzing.
   - Publish each port with its computed authority, for example "serde_json
     port: Pure, 0 trust holes".

   This turns the ecosystem gap into evidence for the authority story and
   closes B2's leak for the most-used crates.
5. **Prove by emitting Verus.** Ferox's `requires`, `ensures`, `invariant`,
   and `decreases` already use Verus's names, and Ferox already emits Rust.
   Emitting Rust with Verus annotations would give a working `prove` mode
   without building verification-condition generation, trigger handling, or
   solver-stability CI. It also inherits Verus's record with agents
   (VeruSAGE: 81%), while Ferox's surface removes the "macro barrier" that
   `research/04` blames for Verus's lower LLM scores. Build a native prover
   later only if Verus's supported subset or its version pinning becomes the
   bottleneck.
6. **Authority-aware registry and resolver.**
   - **Published facts.** Every published version carries computed
     authority, trust-hole counts, proof status, and provenance.
   - **Resolver.** With minimal version selection, the resolver refuses an
     upgrade that widens authority beyond the manifest's grant.
   - **`ferox add`.** It refuses packages that don't exist and shows
     authority before editing the manifest. This defends against
     slopsquatting: 19.7% of model-recommended packages did not exist
     (`research/09` F3).
   - **Queries.** Agents can search by property, for example "JSON parser,
     Pure, proved panic-free".
7. **Interface files owned by humans.** Optional `foo.fxi` files hold
   signatures, authority, and contracts; humans own them and the toolchain
   protects them. Agents write the `foo.fx` bodies. This is Ada's spec/body
   split, and Bend's LAWS/PROOF workflow applied to effectful code: in
   `research/01` F5, the proof gate caught an off-by-one that tests missed.
   It makes "humans review interfaces" literal, and it gives the oracle
   protection that ImpossibleBench calls for a file boundary.
8. **Agent tools that are safe to run, via WASM components.** Rust emission
   gives `wasm32-wasip2` almost for free.
   - Map Ferox capabilities one to one onto WASI resources.
   - An agent-written tool, plugin, or MCP server then carries its authority
     in its types and is also sandboxed at run time.

   That is defense in depth for a 2026 market with a likely champion: agent
   platforms. `research/09` says Ferox needs a champion; the design names
   none.
9. **Replay known attacks as documentation.** For `faster_log`,
   `async_println`, Shai-Hulud, and xz, show the Ferox rule that blocks each:
   no ambient network in a logging crate, no install hooks, no build
   scripts. That is concrete, checkable, and better marketing than a feature
   list.
10. **Bend as a partner.** Co-design Bend's library ABI and offer Ferox as
    the effect host for Bend kernels (the B1 fix). Making the closest
    competitor the reference partner satisfies the owner's constraint better
    than leaving the overlap unaddressed.

## Cuts for v0

Target: a credible v0 within 12 months, built by one to three humans with
fleets of agents.

Keep:
- parser and formatter;
- the checker: types, ownership with views and `from`, and authority;
- the reference interpreter;
- NDJSON diagnostics with machine-applicable fixes;
- typed holes;
- a test runner with `Root::sim`;
- `ferox review`;
- Rust emission for release builds and crate export;
- the agent card (under 8 KB) and the full reference (under 25K tokens);
- FeroxBench.

| Cut or defer | v0 replacement | Why |
| --- | --- | --- |
| `prove` mode with Ferox's own SMT pipeline, ghost code, lemmas, and solver-stability CI | Spec syntax with `assert` and `test` modes; a spike on emitting Verus (Bold 5) | Verus took years at Microsoft Research. Runtime checks plus property tests deliver most of the agent benefit, and the claim stays honest. |
| Evented stackful runtime (io_uring, epoll, kqueue, IOCP, work stealing) | Blocking, threaded, and simulated runtimes | Zig 0.16 shipped only its threaded backend as production-ready; see B4. |
| Heartbeat scheduling, deterministic float-reduction trees, automatic parallelization | `par` and `join` lowered to plain fork-join: rayon-style in emitted Rust, sequential in the interpreter | Performance work with no adoption value at v0. |
| The `lock (a, b)` atomic multi-cell primitive | Channels plus one mutex-like cell type | A new concurrency primitive needs its own soundness work. |
| Borrow checking at Polonius precision, and methods that declare their borrowed fields | NLL precision, which stable rustc accepts | Emitted Rust stays safe and the eject promise stays real. Revisit when Rust's view types land. |
| Automatic import of Rust crates from rustdoc JSON (nightly-only) | Hand-written `extern rust` declarations with authority grants, plus generated C-ABI shims | rustdoc JSON is unstable (`research/09` F4). |
| A cross-package, content-addressed instantiation cache; a native dev backend | The interpreter, using dictionary passing, for dev; rustc monomorphizes in release | Partly deferred already; keep it deferred. |
| Package registry | Git dependencies plus a lockfile with hashes and authority | A registry is a service to operate. Build it once packages exist. |
| A semantic write API (`ferox patch`, writes against a base hash) | Text edits, with a snapshot hash in check output only | `research/06`: strong models edit text reliably, and structured edits add 1–5 points. |
| `Arena` with arena-scoped references; elided reference-count traffic | Generational handles; plain `Rc` and `Arc` | Soundness surface outweighs v0 value. |
| In-process Bend tandem (D14) | Out-of-process calls through `Proc` | Bend has no library target. |
| Windows | Linux and macOS | Bend, cargo-acl, and Zig's evented backend all started there. |
| Latency targets for 100K-line projects | Targets for 20K lines | Measure what v0 projects actually look like. |
| Editions and `ferox migrate` | A written stability promise, plus fix metadata on deprecations | Migration machinery can wait until there is code to migrate. |

Keep `Shape`, but minimal (fields, variants, names). Without macros, JSON
and `Debug` need it.

## Competitor table (October 2026)

| Project | State, Oct 2026 | Overlap with Ferox | Threat | Lesson |
| --- | --- | --- | --- | --- |
| Rust | The default for new systems code at large companies. The 2026 goals include Polonius alpha, a view-types experiment, ergonomic RC, parallel front end, Cranelift performance, contracts, and crates.io hardening. | Everything except the subtractive guarantees | High: it absorbs the ergonomics and contracts claims | Compete on what Rust cannot remove. |
| Verus | Industrial use at Microsoft and Amazon (FMCAD 2026 tutorial; secondary report of AWS Nitro). Active agent research: VeruSAGE (81%), KVerus, StarVerus. | Proved contracts on Rust | High for the proof claim | Reuse it (Bold 5). |
| Kani plus verify-rust-std | 16,000+ harnesses verified per change to std; function and loop contracts | Panic-freedom and UB proofs on Rust | Medium | Push-button checks get adopted. |
| Dafny→Rust | AWS. Readable Rust, at most 22% slower than hand-written on 9 of 10 benchmarks. Dafny leads agent vericoding (82%). | The "verified libraries shipped as crates" wedge | High for that wedge | Don't pick that wedge. |
| cargo-capslock, cargo-acl, Cargo Scan, capsec | Experimental to usable; per binary or per crate | Authority analysis for Rust | Medium | Coarse but no switching cost. Ferox must be sound and per signature. |
| Bend 2 | Launched 17 Sep 2026; v2.0.37. Pitch: "blocks AI mistakes via proof". No library target; numbers are U32/F32 only. | Agents, proofs, fast back-end code | High overlap on the pitch, which the owner rules out | Partner with it and leave pure kernels to it (B1). |
| Zero (Vercel Labs) | v0.3.x with a LABS-EXPERIMENT badge, about 5.4k stars, activity down since June 2026 | Agent-first systems language with explicit effects | Low | Its graph-first source churned three times; text-first wins. |
| MoonBit | 0.8.0 in Feb 2026; 1.0 still unannounced. AI-native toolchain, Wasm target. | Agent-friendly typed language with an integrated toolchain | Low to medium; different domain | Integrated toolchain and MCP from day one. |
| Mojo | 1.0 on Aug 11, 2026. Compiler open-sourced Aug 18, after Qualcomm's ~$3.9B purchase of Modular. | Systems performance with Python interop | Low; AI and GPU niche | A platform owner matters more than design. |
| Swift 6.3 | Shipped, with an Android SDK and embedded improvements. `~Escapable` lifetime dependencies are still experimental. Servers via NIO. | "Safe without lifetimes" | Low to medium | Borrows without lifetime annotations are hard even for Apple. |
| Hylo | Research project; no 2026 release found | Mutable value semantics, second-class references | Low | An unfamiliar model adopts slowly. |
| Rue (Klabnik) | v0.1 research project written largely by Claude; about 1.1k stars in early 2026 | "Between Rust and Go", inout parameters, built by AI | Low now, but shows that "lighter Rust" is a crowded lane | A lighter Rust alone is not a moat. |
| Zig 0.16 | Released Apr 2026. `Io` is passed as a parameter; evented backend incomplete. | Capability-like I/O that suits simulation | Medium on the simulation wedge (TigerBeetle uses Zig) | Users accept an explicit I/O parameter, but Zig's is coarse. |
| Carbon | 0.1 no sooner than end of 2026 | C++ successor | None | Interop with a complex host is a sink. |
| The agent-language wave (Vera, Thermite, Intent, SEMAPRAX, AILANG, Hale, and others; 42 in the catalog) | Mostly hobby compilers with little outcome data. Thermite already emits Rust with req/ens/fx checked by Verus or Lean. | Contracts, effects, Rust emission | Low one by one; high noise | Ferox will be filed as entry #43 unless it publishes evidence. |
| Gleam | v1 in 2024; admired, still small | A small, consistent language | None | Ship the toolchain together and pledge stability. |
| Austral, Roc | Austral passes a root capability to `main` and has little adoption. Roc has platforms and is mid-rewrite to Zig. | Capabilities | None | Capabilities alone don't attract users; pair them with a pain users feel. |

## Strengths worth keeping

- **The problem statement in `design/00`.** Writing code is cheap; trust and
  change are the cost; human attention should move to interfaces. This is
  the right framing, and no competitor states it as clearly.
- **The trust ladder.** Levels 0–5 work as product vocabulary and give
  reviewers and agents a shared scale.
- **D1.** Rust syntax where the meaning matches, new syntax where it
  differs, and fixes for Rust habits. The evidence supports it
  (`research/05`, `research/09`).
- **D6 and D7 together.** No macros, no build scripts, closed derives,
  capabilities as values, and purity as a fact about types. These are the
  subtractive guarantees the strategy should rest on.
- **D5.** `?` converts errors only through `from` variants that the
  destination type declares.
- **D10's gradient of one spec with three uses.** The `assert` and `test`
  modes are useful before any prover exists.
- **D11.** Interface artifacts and per-function caching. The compile-speed
  thinking is at the level of language design, not backends.
- **D12.** Interpreter first as the oracle, no self-hosting, and Rust
  emission as the exit door.
- **D13's agent contract**, especially protected specs and tests and
  `ferox review`.
- **Explicit rejections.** Every decision lists what it rejects. Keep that
  discipline.
- **Honest research.** The research is candid about its gaps and contains
  kill criteria. Move those criteria into the design.

## Sources

In-repo: `design/00-first-principles.md`, `design/01-design.md`,
`research/00`, `01`, `04`, `06`, `07`, `09`, `10`.

Web, checked 2026-10-19:

- Bend 2 site: https://bend-lang.com
- Bend WONTFIX (library target, U64/F64): https://github.com/bendlang/bend/blob/main/WONTFIX.txt
- Rust 2026 project goals: https://goals.rust-lang.org/2026/goals.html
- Rust std contracts goal (2025h1): https://goals.rust-lang.org/2025h1/std-contracts.html
- Kani paper (2026): https://www.emergentmind.com/papers/2607.01504
- Verus FMCAD 2026 tutorial / industrial use: https://repositum.tuwien.at/handle/20.500.12708/230444 ; AWS Nitro report (secondary): https://zeli.app/story/49700153
- Dafny→Rust: https://arxiv.org/abs/2501.05111 ; https://popl25.sigplan.org/details/dafny-2025-papers/14/Performant-Readable-and-Interoperable-Rust-from-Dafny
- cargo-capslock (Rust Foundation, Alpha-Omega): https://alpha-omega.dev/?p=1575
- cargo-acl (formerly cackle): https://github.com/cackle-rs/cackle/releases ; https://docs.rs/cargo-acl
- Cargo Scan: https://www.alphaxiv.org/abs/2602.06466
- capsec: https://docs.rs/crate/capsec/0.1.3
- Zig 0.16 release notes: https://ziglang.org/download/0.16.0/release-notes.html ; review: https://www.youngju.dev/blog/2026-07-16-zig-016-io-interface.en
- Zero: https://github.com/vercel-labs/zerolang
- Agent-language catalog: https://agentlanguages.dev/
- MoonBit blog: https://www.moonbitlang.com/blog/
- Mojo 1.0 and open source: https://www.phoronix.com/news/Modular-Mojo-Open-Source ; https://www.techzine.eu/news/devops/143570/mojo-programming-language-reaches-version-1-0/
- Swift 6.3: https://swift.org/blog/swift-6.3-released/ ; lifetime dependencies pitch: https://forums.swift.org/t/pitch-3-compile-time-lifetime-dependency-annotations/84968
- Hylo: https://docs.hylo-lang.org
- Rue: https://infoq.com/news/2026/01/steve-klabnik-rue-language-ai/ ; https://urandom.io/blog/2026-02-23-steve-klabnik-rue-language-rust-veteran-ai-compiler
- crates.io name search: https://crates.io/search?q=ferox
