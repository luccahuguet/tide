# 07: Effects, capabilities, and purity tracking

Research note, 2026-10-19. Each factual claim links its source. **[inference]** marks my own synthesis. **[background]** marks prior knowledge I could not re-verify after the web-search budget ran out partway through. Treat those two kinds of statement as weaker evidence.

## Question

What is the lightest effect and capability design for a Rust-class systems language (no GC, ownership, monomorphized generics) that delivers all of the following?

1. **Reviewable authority.** A reviewer can see from signatures alone which code may touch the filesystem, network, clock, randomness, processes, environment, FFI, or `unsafe`.
2. **Sandboxed dependencies.** Third-party and agent-written code holds only the authority it is explicitly given.
3. **Deterministic testing and replay.** Every source of nondeterminism can be substituted in tests.
4. **Zero-cost static dispatch** in the common case.

A follow-up question is how such a design interacts with ownership, generics, async, compile speed, and with Opus-class LLMs as the main authors.

## Findings (cited)

### F1. "Effects" covers two separate problems

**[inference]** The literature puts two problems under one name:

- **Control effects** (exceptions, generators, async suspension, backtracking) need some form of captured continuation.
- **Authority effects** (I/O, clock, randomness, network) only need to answer "who may call what".

Several sources show that authority effects do not need continuations:

- Koka's evidence-passing translation runs tail-resumptive operations "in place (without needing to capture the evaluation context)". It also replaces the runtime search for a handler with "indexing with a constant offset" ([Xie & Leijen, ICFP 2020](https://www.microsoft.com/en-us/research/publication/effect-handlers-evidently/)).
- Effekt's designers concluded that "there are no builtin effects, only captures". They removed their `Console` effect in favor of a builtin `io` capability ([Effekt history](https://effekt-lang.org/evolution)).
- In Effekt, effect types express "which capabilities a computation requires from its context" ([Brachthäuser et al., OOPSLA 2020](https://ps.informatik.uni-tuebingen.de/publications/brachthaeuser20effects/)).

**[inference]** A handler that always resumes exactly once, in tail position, is semantically the same as a trait implementation passed as an argument. That covers nearly every I/O, clock, logging, and randomness handler used for mocking.

### F2. Handler-based languages

**Koka**
- Row-typed effects; the docs show inferred rows such as `<console,yield<int>>`.
- Compiles to C with Perceus reference counting "without needing a garbage collector or runtime system" ([Koka docs](https://koka-lang.github.io/koka/doc/index.html)).
- Generalized evidence passing compiles handlers to C with competitive performance against multicore OCaml, libhandler, and Haskell libraries ([Xie & Leijen, ICFP 2021](https://www.microsoft.com/en-us/research/uploads/prod/2021/08/genev-icfp21.pdf)).
- Cost of row polymorphism: Koka's inference automatically inserts `open` coercions that adjust evidence vectors at run time, and it "tends to generate many redundant open calls" ([Furudono et al., TFP 2022](https://prg.is.titech.ac.jp/papers/pdf/tfp2022adjustment.pdf)).
- Koka v3 calls itself a research language "not ready for production use" ([Koka book](https://koka-lang.github.io/koka/doc/book.html)).

**Effekt**
- Lexical handlers and capability passing. The 2020 design made all functions second-class to get lightweight effect polymorphism. The authors concede this "impedes reasoning about purity" ([OOPSLA 2020](https://ps.informatik.uni-tuebingen.de/publications/brachthaeuser20effects/)).
- System C (2022) restores first-class functions through "boxes". The type system then has to carry how impure a boxed value is ([OOPSLA 2022](https://pl.cs.uni-tuebingen.de/publications/brachthaeuser22effects/)).
- Capability-passing compilation can "guarantee full elimination of the effect handler abstraction" for a restricted subset of programs, using staging ([Schuster et al., ICFP 2020](https://pl.cs.uni-tuebingen.de/publications/schuster20capability/)).
- The optimizing pipeline was later abandoned:
  - "the pipeline was quite involved (we published three papers about it)";
  - MLton lacked the higher-rank polymorphism they needed;
  - the replacement LLVM backend is "still significantly slower" ([Effekt history](https://effekt-lang.org/evolution)).
- The 2025 LLVM runtime gives one-shot continuations constant-time capture and resumption, and copies stacks only when a continuation is resumed more than once ([Muhcu et al., ICFP 2025](https://se.cs.uni-tuebingen.de/publications/muhcu2025multiple/)).

**OCaml 5**
- Untyped effects on one-shot fibers.
- Mean overhead is about 1% on a macro-benchmark suite that does not use handlers.
- Microbenchmark: creating and switching to a fiber takes about 23 ns, perform/handle about 5 ns, resume about 11 ns, and return/free about 7 ns.
- Resuming a continuation twice raises a run-time exception, because OCaml has no linear types to rule it out statically ([Sivaramakrishnan et al., PLDI 2021](https://arxiv.org/pdf/2104.00250)).
- **[background]** Unhandled effects are also run-time errors, because the type system does not track effects.

**Unison and Frank**
- Unison's "abilities" are based on Frank. They are written in braces between the arrow and the result type. Calls are rejected with an "ability check failure" unless the ability comes from a handler or from the signature ([Unison reference](https://www.unison-lang.org/docs/language-reference/abilities-and-ability-handlers)).
- Frank avoids writing effect variables by "propagating an ambient ability inwards, rather than accumulating unions of potential effects outwards" ([Lindley, McBride, McLaughlin, POPL 2017](https://arxiv.org/abs/1611.09259)).

**Flix**
- Has the richest production effect algebra ([Madsen, LambdaDays 2025 slides](https://flix.dev/talks/lambdadays2025.pdf)):
  - primitive effects `Env, Exec, FsRead, FsWrite, Net, NonDet, Sys, IO`;
  - effect variables (`f: a -> b \ ef`), unions (`ef1 + ef2`), and exclusion (`ef - {Throw}`);
  - region heap effects (`Heap[r]`);
  - associated effects on traits;
  - user handlers advertised as "one for production, one for testing".
- Inference uses Boolean unification. It is NP-hard, yet practical: a 1.4x slowdown in type inference and 1.1x end to end ([Madsen & van de Pol, OOPSLA 2020](https://users-cs.au.dk/magnusm/papers/oopsla20/paper_b.pdf)).
- The 2025 formalization adds subeffecting that makes "all effect upcasts in the Flix standard library" unnecessary ([Lee et al., OOPSLA 2025](https://plg.uwaterloo.ca/~olhotak/pubs/oopsla25.pdf)). **[inference]** So users and library authors had been writing manual effect upcasts until then.

**Scala 3 capture checking ("Caprese")**
- Syntax: `A^{c}` is a value that may use capability `c`; `A -> B` is a pure function; `A => B` is shorthand for `A ->{any} B`. Capabilities can be passed implicitly with `using`.
- Status: still behind `language.experimental.captureChecking`. The docs call it "quite mature" with stabilization expected "soon", while separation checking is "still a bit more fluid" ([Scala docs](https://scala-lang.org/api/3.x/docs/experimental/capture-checking/overview.html)).
- Tracking capabilities inside generic data structures needed a new calculus with "reach capabilities" ([Xu et al., OOPSLA 2025](https://www.arxiv.org/abs/2509.07609)).
- 2026 work adds separation and freshness (System Capybara, [arXiv 2607.09383](https://arxiv.org/abs/2607.09383)) and capability classifiers ([arXiv 2607.24504](https://arxiv.org/abs/2607.24504)).
- In September 2025, Odersky described the separation checker as still "far from finished" ([scala3 PR 24112](https://github.com/scala/scala3/pull/24112)).
- **[inference]** Precise per-value capture sets have taken EPFL several years of research and are still not stable.

**Lean 4**
- Keeps monads and extends `do` notation with local mutation, early return, and loops, all compiled back to pure code ([Ullrich & de Moura, ICFP 2022](https://doi.org/10.1145/3547640)).
- **[inference]** This is the "monads plus good syntax" end of the design space. It works for a proof assistant, but monad transformer stacks are a poor fit for a systems language that targets zero-cost code and LLM authors.

### F3. Lightweight industrial points in the design space

- **Roc (purity inference).**
  - `->` marks pure functions and `=>` effectful ones; effectful function names end in `!`.
  - Pure functions may only call pure functions, and purity is inferred ([Roc langref](https://roc-lang.org/docs/main/langref/functions/); [Feldman gist](https://gist.github.com/rtfeldman/f46bcbfe5132d62c4095dfa687bb9aa4)).
  - "Pure" still allows crashing, hanging, and `dbg`. Purity lets every top-level value be evaluated at compile time ([Zelenya on Roc](https://dev.to/zelenya/why-im-excited-about-roc-26j1)).
  - Roc replaced an earlier Task-based design with direct calls.
- **Verse.**
  - A fixed lattice of effect specifiers: `<computes>`, `<reads>`, `<writes>`, `<allocates>`, `<transacts>`, `<decides>`, with `<no_rollback>` as the default ([Epic docs](https://dev.epicgames.com/documentation/en-us/uefn/specifiers-and-attributes-in-verse)).
  - **[inference]** This is a closed vocabulary with no user effect variables, and it ships to a large non-expert audience.
- **Nim.**
  - `raises: []` is checked and is part of the proc type.
  - `tags` are user-defined effects ([Nim manual](https://nim-docs.readthedocs.io/en/latest/manual/effect_system/)).
- **Swift typed throws.**
  - Generalizes `rethrows` with an error-type generic: a non-throwing closure infers `E = Never`, so no separate rule is needed.
  - The proposal itself says "untyped `throws` is better for most scenarios" ([SE-0413](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0413-typed-throws.md)).
  - **[background]** This echoes the Java checked-exceptions experience.
- **Zig 0.16 `std.Io`.**
  - I/O is "provided by the caller, just like we already do with `Allocator`". It is a non-generic vtable interface.
  - Devirtualization is "not guaranteed", except when only one `Io` implementation is in use ([Cro 2025](https://kristoff.it/blog/zig-new-async-io/)).
  - Shipped as the headline of 0.16.0 ([release notes](https://ziglang.org/download/0.16.0/release-notes.html)).
  - Critics note that the colors become "uses IO" versus "doesn't use IO". They add that this may be defensible, because I/O "should be part of your API" ([LWN](https://lwn.net/Articles/1046084)).
- **Kotlin context parameters.**
  - Implicit capability-style parameters: preview in 2.2 (June 2025), stable in 2.4 (July 2026). They must be named.
  - Overload ambiguity introduced in 2.3.20 required explicit context arguments ([Kotlin 2.4](https://kotlinlang.org/docs/whatsnew24.html); [JetBrains](https://blog.jetbrains.com/kotlin/2025/04/update-on-context-parameters/)).
- **Rust.**
  - The keyword-generics/effects initiative began in 2022 ([announcement](https://blog.rust-lang.org/inside-rust/2022/07/27/keyword-generics/)).
  - By 2026, const traits, the most concrete effect, are still experimental: "there has never been an accepted RFC describing its syntax" ([2026 goal](https://goals.rust-lang.org/2026/const-traits.html)).
  - **[inference]** Retrofitting effect polymorphism onto an existing language is very hard; it should be designed in from the start.

### F4. Implementation cost

- **Dynamic versus lexical handlers.**
  - Lexa compiles lexical handlers to stack switching and reduces some programs from quadratic to linear time ([Ma et al., OOPSLA 2024](https://cs.uwaterloo.ca/~yizhou/papers/lexa-oopsla2024.pdf)).
  - Its follow-up observes that existing lexical implementations "require programs to pay a cost just for having handlers in the lexical context". It restores zero overhead for exception-like effects with type-directed metadata and stack walking, and tunes the compilation strategy per effect ([Ma et al., OOPSLA 2025](https://2025.splashcon.org/details/OOPSLA/203/Zero-Overhead-Lexical-Effect-Handlers)).
- **Accidental handling.** With dynamic scoping, a higher-order function can intercept effects raised by its argument. Tunneling semantics fixes this, so that effect-polymorphic code cannot "accidentally handle effects raised by functions passed in" ([Zhang & Myers, POPL 2019](https://www.cs.cornell.edu/andru/papers/tunnel-eff)). **[inference]** Passing capabilities as lexical values gives the same guarantee.
- **Compile time.** Boolean effect unification costs about 1.4x in inference (F2). Koka's row adjustments insert redundant coercions (F2). Effekt's whole-program optimizing pipeline was dropped for complexity (F2). **[inference]** Checking a declared capability set against a callee's requirement is a subset test on small sets, which costs nothing measurable.

### F5. Ownership, linearity, and no GC

- **Multi-shot continuations break linear types.** Handlers can discard or duplicate continuations. That "conflicts with the one-use assumption" of linear types and caused real soundness bugs in Links. The fix requires tracking control-flow linearity ([Tang, Hillerström, Lindley, Morris, POPL 2024](https://arxiv.org/pdf/2307.09383)).
- **Effects break region stack discipline.** Non-local control breaks the call/return discipline that regions depend on, and multi-shot resumption breaks "a region can be exited at most once". Reconciling them currently takes an Iris program logic ([Yarrow, arXiv 2607.15876](https://arxiv.org/abs/2607.15876)).
- **Modes.** Jane Street's OCaml modes (local, portable, contended) give data-race freedom ([POPL 2025](https://popl25.sigplan.org/details/POPL-2025-popl-research-papers/23/Data-Race-Freedom-la-Mode)). Modal tracking can still reject obviously safe programs, which motivated the "mode crossing" work ([ICFP 2026](https://icfp26.sigplan.org/details/icfp-2026-icfp-papers/9/Mode-Crossing)).
- **[inference]** Effekt's second-class capabilities, OCaml's `local` mode, and Rust's borrowed references bounded by a lifetime are the same idea: a value that cannot escape its scope. A Rust-like language already has the machinery for second-class capabilities.

### F6. Capability security in languages and runtimes

- **Austral.**
  - `RootCapability` values "cannot be created in userspace"; they arrive only as the first argument of `main`.
  - Narrower capabilities are derived by proving possession of a broader one. Empty-record capabilities are "pure type-level permission slips".
  - Stated limits: no revocation, no global uniqueness, and unaudited unsafe FFI. Verdict: "The model is good—but not good enough" ([Borretti](https://borretti.me/article/how-capabilities-work-austral)).
- **Pony.**
  - `Env.root` holds `AmbientAuth`, which only `builtin` can create.
  - `NetAuth` can only be built from `AmbientAuth` ([tutorial](https://tutorial.ponylang.io/object-capabilities/derived-authority)).
  - Passing `AmbientAuth` everywhere is "far too permissive" ([patterns](https://patterns.ponylang.io/object-capabilities/authority-hierarchy)).
- **Wyvern.** A module's authority is determined by inspecting the capabilities passed to it at creation, so architects can review authority "using only its interface" ([Melicher et al., ECOOP 2017](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ECOOP.2017.20)).
- **Joe-E.**
  - A capability-secure Java subset with a verifier ([spec](https://www2.eecs.berkeley.edu/Pubs/TechRpts/2008/EECS-2008-91.html)).
  - A method whose argument types are all immutable is guaranteed pure ([Finifter et al., CCS 2008](https://people.eecs.berkeley.edu/%7Edaw/papers/pure-ccs08.pdf)).
  - Goal: "reviewably secure" systems ([Mettler 2012](https://www2.eecs.berkeley.edu/Pubs/TechRpts/2012/EECS-2012-244.html)).
  - It banned mutable statics ([Wikipedia](https://en.wikipedia.org/wiki/Joe-E), secondary source).
- **WASI and the component model.**
  - A component "starts with no ambient authority and can only do what the host explicitly grants" ([wasi.dev](https://wasi.dev/)).
  - WASI 0.3 was ratified on 2026-06-11 and moves async into the component model's canonical ABI, with `stream<T>` and `future<T>` ([WASI 0.3](https://wasi.dev/releases/wasi-p3); [Bytecode Alliance](https://bytecodealliance.org/articles/WASI-0.3)).
  - WASI-Virt composes a component that denies or virtualizes env, filesystem, stdio, and sockets, deny-all by default ([WASI-Virt](https://github.com/bytecodealliance/WASI-Virt)).
- **Deno.**
  - Deny by default, with scoped `--allow-*` flags ([permissions](https://docs.deno.com/runtime/reference/permissions.md)).
  - But "all code executing on the same thread shares the same privilege level", so dependencies get no separate tier.
  - `--allow-ffi` and `--allow-run` amount to `--allow-all` ([security](https://docs.deno.com/runtime/fundamentals/security/)).
- **cap-std.**
  - Capability-oriented Rust I/O (`Dir`, `Pool`), with `ambient_authority()` as an explicit marker.
  - It "is not a sandbox for untrusted Rust code", because `std::fs` and `unsafe` remain available ([docs](https://docs.rs/cap-std/2.0.1/cap_std)).
- **Java SecurityManager (a failure).**
  - Fine-grained permissions were brittle ("no way to have partial security"), and there were no negative permissions.
  - Stack inspection meant libraries forced extra grants on applications, so developers granted `AllPermission`.
  - It imposed "an unacceptable performance penalty" and saw little use ([JEP 411](https://openjdk.org/jeps/411)).
  - It was deprecated in JDK 17 and permanently disabled in JDK 24 ([JEP 486](https://openjdk.org/jeps/486)).

### F7. Supply-chain tooling (analysis after the fact)

- **Capslock (Go).**
  - Follows transitive calls to privileged standard-library operations ([deps.dev](https://blog.deps.dev/capslock/)).
  - Fewer than 2% of version updates add a capability, which makes "new capability" a high-signal alert.
  - It over-approximates: every branch counts as taken and every interface implementation as possible. Reflection, `unsafe`, cgo, and `linkname` are reported as opaque capabilities ([caveats](https://github.com/google/capslock/blob/main/docs/caveats.md)).
- **cargo-scan.** Automatically clears about 3,500 of the top 10,000 crates; risky operations concentrate in about 3% of crates ([Zoghbi et al.](https://arxiv.org/abs/2602.06466v1)).
- **cackle (cargo-acl).** Grants API and `unsafe` permissions per package ([config](https://github.com/cackle-rs/cackle/blob/main/CONFIG.md)).
- **Rust internals discussion (September 2025).** On adding crate permissions to Rust:
  - Rust "is not a security boundary";
  - build scripts are unsandboxed;
  - compiler soundness bugs can defeat `forbid(unsafe)`;
  - partial checks still make malware "more obviously strange" ([thread](https://internals.rust-lang.org/t/effect-system-crate-permissions/23529)).
- **A concrete attack.** In 2025 two typosquatted *logging* crates scanned files for private keys and exfiltrated them over the network, at run time rather than build time. They had 7,181 and 1,243 downloads ([Rust blog](https://blog.rust-lang.org/2025/09/24/crates.io-malicious-crates-fasterlog-and-asyncprintln/)). **[inference]** Under a no-ambient-authority design, a logger needing `Net` would be visible in its constructor's signature, and could not run without the root wiring it in.

### F8. Determinism and replay

- **TigerBeetle VOPR.** Clock, network, and disk are stubbed out. A seed plus a commit hash replays any failure ([docs](https://docs.tigerbeetle.com/about/vopr)).
- **S2 (async Rust).** Turmoil plus a paused Tokio clock were "not comprehensive enough to address what arbitrary dependencies may be doing". Nondeterminism leaked through dependencies via `clock_gettime`, `getrandom`, and randomized `HashMap` seeds. Fixing it required overriding libc symbols ([S2](https://s2.dev/blog/dst)).
- **Go `testing/synctest`.** Stable in Go 1.25. It runs goroutines in a "bubble" with a fake clock, but real network and disk I/O fall outside the bubble ([pkg docs](https://pkg.go.dev/testing/synctest@master)).
- **[inference]** Simulation and replay are only reliable when the language makes every nondeterminism source an explicit, substitutable input, including hash seeds and thread spawning.

### F9. Evidence relevant to LLM authors

- Code LLMs "struggle with low-resource languages" such as OCaml and Racket ([Cassano et al.](https://arxiv.org/abs/2308.09895)). Novel notation carries a real cost for a new language.
- Type-constrained decoding cuts compilation errors "by more than half" ([Mündler et al., PLDI 2025](https://arxiv.org/abs/2504.09246)). Local, declarative typing rules are easier to enforce during decoding than global inference.
- CaMeL tags values with capabilities in an interpreter to stop prompt injection. It solves 77% of AgentDojo tasks with provable security, against 84% undefended ([Debenedetti et al.](https://arxiv.org/abs/2503.18813)). It enforces at run time, not statically.
- **Gap.** I found no empirical study of LLMs writing effect-typed code (rows, effect variables, handlers). Every claim below about LLM ergonomics is **[inference]**.

## What works, what failed, and why

| Approach | Worked | Failed or costly | Why |
|---|---|---|---|
| No ambient authority; root capability from `main` (Austral, Pony, WASI) | Authority is traceable, and sandboxing comes by construction | Revocation, FFI, global uniqueness (Austral) | Escape hatches are the remaining holes |
| Capability *libraries* on top of an ambient std (cap-std) | Defends against path traversal | "Not a sandbox" | The ambient path still exists |
| Process-wide permissions (Deno) | Simple, deny by default | No per-dependency tier; FFI/run equal allow-all | Granularity is the process, not the module |
| Stack inspection with fine-grained ACLs (Java SM) | Nothing durable | Brittle, slow, `AllPermission` everywhere | Policy lives apart from code; authority is checked rather than passed |
| Post-hoc analysis (Capslock, cargo-scan, cackle) | Diffs carry high signal (<2% of updates add a capability) | Over-approximation; `unsafe`/reflection opaque | Analysis has to recover what the types did not state |
| Row-polymorphic effects (Koka) | Expressive, fast C | Research status; redundant `open` coercions | Polymorphism is needed everywhere, and it is visible |
| Boolean effect algebra (Flix) | Full inference; exclusion and associated effects | NP-hard inference; manual upcasts until 2025 | Expressiveness drags in a complex algebra |
| Capture types (Scala) | Precise; closures and generics covered | Years of work, still experimental in 2026; reach capabilities needed | Per-value capture sets meet generic containers |
| Second-class capabilities (Effekt) | Effect polymorphism without variables | First-class functions need boxes; the fast pipeline was dropped | Escaping values need annotation |
| Untyped one-shot effects (OCaml 5) | About 1% overhead; fast fibers | Unhandled effects and double resumption caught only at run time | No static tracking |
| Purity inference with a fixed vocabulary (Roc, Verse) | Tiny surface; compile-time evaluation | Coarse (Roc has a single effectful color) | It deliberately trades precision for simplicity |
| Caller-provided I/O interface (Zig `Io`) | Sync and async from one code path | Virtual calls; "colors" renamed | Zig has no static checking that `Io` is the only path |

## Recommended design

### Core model ("capabilities as borrowed values, effects as a closed list")

**[inference throughout, built on the evidence above]**

1. **No ambient authority.**
   - Every operation that observes or changes the outside world is a method on a capability value: filesystem, network, clock, randomness, environment, process spawn, thread/task spawn, stdio, FFI.
   - std exposes no free functions that do I/O.
   - `main` receives a `Root`; nothing else can create one, as in Austral and Pony.
   - Safe code has no mutable globals and no thread-locals, as in Joe-E.
2. **Capabilities are ordinary values of `cap` traits**, passed by borrow.
   - Attenuation is a method that returns a narrower capability, as in cap-std's `Dir` and Pony's `NetAuth`. Examples: `fs.dir("logs", Append)`, `net.only(hosts)`.
   - The *kind* of authority (Fs, Net, Clock…) is static.
   - The *scope* (which directory, which host) is a dynamic property of the value.
3. **Capability requirements are part of signatures.**
   - A function lists the capabilities it needs in a `with` clause.
   - Callers satisfy the clause implicitly from capabilities in scope, matched by type.
   - A clause with two candidates of the same type is an error that asks for an explicit argument. This is Kotlin's lesson.
   - Public items must spell out their `with` clause. Private items and closures may have it inferred, and tooling prints the inferred set.
4. **Authority carried inside values is declared on the type.**
   - A struct that stores a capability declares it: `struct HttpClient with Net { … }`.
   - The authority set of a type is computed structurally, the way Rust's auto traits are.
   - A `Pure` auto-trait marks types that contain no capability. Generic code that must be pure says `T: Pure` or `F: Fn(..) + Pure`.
   - This replaces effect variables. A borrowed closure argument needs no annotation, because it can only use authority that the caller's own signature already declared. This is Effekt and Frank's contextual polymorphism, obtained from lifetimes.
5. **Control effects are a closed, built-in list.**
   - Errors use `Result` and `?`.
   - Panics and divergence are untracked; "pure" allows panic, divergence, allocation, and `dbg`, as in Roc.
   - Suspension is an operation on the `Io` capability.
   - There are no user-defined resumable handlers and no first-class continuations in v1. Multi-shot is never allowed (F5).
6. **Purity is a fact read from the signature.** A function is pure if it has no `with` clause and none of its parameters or captures is authority-bearing (the types are `Pure`). Pure functions are eligible for compile-time evaluation, memoization, parallel maps, and the optional proof mode.
7. **Packages are gated in the manifest only for escape hatches.**
   - Type-level capabilities need no manifest: a dependency can only use what is passed to it.
   - The manifest grants `unsafe`, `ffi`, and build-time I/O per package, as cackle does.
   - Compile-time code (macros, `comptime`) runs with no capabilities.
   - The compiler emits a per-package authority summary into the lockfile. An upgrade that adds a capability kind fails CI until approved; Capslock's data says that happens in under 2% of updates.
8. **Deterministic simulation lives in std.**
   - `Sim::seeded(n)` yields a `Root` whose clock, network, filesystem, random source, executor, and spawn are deterministic.
   - Default `HashMap` hashing must not draw hidden entropy. Seeded hashing takes `Rand` or uses a fixed per-build key, which directly answers S2's `HashMap` leak.
   - A `Record` wrapper logs every capability response so a run can be replayed.

### Syntax sketch (illustrative, not a spec)

```ferox
// std: capability traits. No free I/O functions exist.
cap trait Clock { fn now(&self) -> Instant; }
cap trait Net   { fn connect(&self, a: &Addr) -> Result<Conn, NetError>; }
cap trait Fs    { fn open(&self, p: &Path, m: Mode) -> Result<File, FsError>;
                  fn dir(&self, p: &Path, m: Mode) -> Result<Dir, FsError>; } // attenuation

// Authority is reviewable from the signature: Net and Clock, nothing else.
pub fn fetch_feed(url: &Url) -> Result<Feed, FetchError> with Net, Clock {
    let deadline = Clock.now() + 5.s;        // the capability is named by its trait
    let conn = Net.connect(&url.addr())?;
    http::get(conn, url, deadline)           // the callee's `with Net, Clock` is satisfied implicitly
}

// Two capabilities of the same trait must be named, and callers pass them explicitly.
pub fn rotate(name: &str) -> Result<(), FsError> with logs: Dir, archive: Dir { ... }
// call site: rotate("app") with logs = log_dir, archive = cold_dir

// Pure: no `with`, and every parameter type is Pure. It may panic, allocate, or dbg.
pub fn parse_feed(bytes: &[u8]) -> Result<Feed, ParseError> { ... }

// No effect variables. `f` is borrowed (second-class), so it can only use
// authority that the caller's own signature already declares.
pub fn map<T, U>(xs: &[T], f: &fn(&T) -> U) -> Vec<U> { ... }

// When purity is required, say so with an auto-trait bound.
pub fn par_map<T: Send, U: Send>(xs: &[T], f: &(fn(&T) -> U + Pure + Sync)) -> Vec<U> with Spawn { ... }

// Types that store authority declare it; the checker verifies the declaration.
pub struct HttpClient with Net { net: Net, pool: Pool }

// Named bundles keep signatures short for app-level code.
cap App = Net + Clock + Log;
pub fn run() -> Result<(), AppError> with App { ... }

// The only place authority is created and attenuated.
fn main(root: Root) -> ExitCode {
    let logs = root.fs.dir("/var/log/app", Mode::Append)?;    // Fs, scoped to one directory
    let net  = root.net.only(["api.example.com:443"]);         // Net, scoped to one host
    app::run() with Fs = logs, Net = net, Clock = root.clock, Log = root.log
}

// Deterministic test: same code path, simulated root.
test fn feed_times_out() {
    let sim = Sim::seeded(42);
    let r = fetch_feed(&url) with Net = sim.net.drop_all(), Clock = sim.clock;
    assert r == Err(FetchError::Timeout);
}
```

Manifest and lockfile, also a sketch:

```toml
[dependencies]
fast-log = "1.4"                         # no grants: it cannot use unsafe, FFI, or build-time I/O
zstd     = { version = "0.13", grant = ["unsafe", "ffi"] }

# ferox.lock (generated): authority summary per package, diffed on upgrade
# fast-log 1.4.2  pub-api-caps = [Fs]      # 1.4.3 adds Net -> CI fails until approved
```

### Interaction with ownership, generics, async, and compile speed

**Ownership.**
- Capabilities are non-`Copy` values, passed implicitly as `&Cap`. Lifetimes keep a borrowed capability from escaping, which is Effekt's second-class discipline using Rust's existing checker.
- Storing a capability requires owning it, through `.share()` or a clone of an `Arc`-like handle. That makes the struct authority-bearing, and the `with` declaration on the type is checked.
- Linear (Austral-style) capabilities are not the default. Their extra power (single-use tokens) can be a library pattern on top of affine types.
- With no multi-shot continuations, the unsoundness described by Tang et al. cannot arise, and regions and arenas keep their stack discipline (F5).

**Generics and dispatch.**
- Capability parameters compile like `dyn` (one vtable pointer, or a fat context pointer when several are bundled), so they never multiply monomorphized instances.
- Production programs usually have exactly one implementation per capability trait. The compiler should guarantee devirtualization in that case, as Zig does for a single `Io`, so dispatch becomes static and a zero-sized token inlines away. Austral's empty-record capabilities show the token can cost nothing.
- Test builds hold a second implementation and pay one indirect call.
- Trait methods need no effect parameters. An iterator that reads a file holds `Fs` in its struct (object-capability style), so `Iterator::next` stays unannotated. This avoids Flix-style associated effects and Rust-style effect generics in most code.

**Async.**
- Suspension is an `Io` capability operation. There is no `async` keyword: the "color" is the `with Io` in the signature, which reviewers want to see anyway (LWN's point).
- Because suspension appears in signatures, the compiler knows statically which functions may suspend. That leaves room to compile them as stackless state machines later.
- v1 can use stackful fibers. OCaml 5 shows the speed is viable (tens of nanoseconds), but per-task stack memory costs more than Rust's state machines.
- WASI 0.3's native async in the canonical ABI is a natural lowering target for Wasm.

**Compile speed.**
- Every check is local: set inclusion for `with` clauses, plus a structural auto-trait for `Pure` and type authority.
- There are no effect rows, no Boolean unification, and no global inference across `pub` boundaries.
- Expected cost is noise next to borrow checking. This is unmeasured.

### What to avoid

- **Effect variables, rows, and Boolean effect algebra in user-facing types** (`<exn|e>`, `\ ef - {Throw}`). They are the main source of signature noise and inference cost (F2, F4). Contextual polymorphism through borrowed closures plus `Pure` bounds covers the common cases.
- **Per-value capture sets** (`T^{c}`) as the main mechanism. Scala needed reach capabilities for generic containers and is still experimental after years. Kind-level authority sets on types are coarser but cheap to check and read.
- **General resumable handlers and multi-shot continuations.** They conflict with linear and affine types and with regions (F5), and they make control flow hard to review. If generators or one-shot handlers are wanted later, add them as *lexical, one-shot, non-escaping* constructs (Lexa, Effekt LLVM) that hold no capability and run zero-overhead when unused.
- **Stack inspection or ACL policy files** (Java SM). Policy separated from code goes stale, and failures push people to grant everything.
- **A capability library on top of ambient std** (cap-std). The ambient path has to be absent, not merely discouraged.
- **Arbitrary build scripts and procedural macros with I/O.** They bypass every run-time capability, as the internals thread and Deno's FFI/run caveat both show.

## Design implications (ranked)

1. **Remove ambient authority completely** (no I/O free functions, no mutable globals in safe code, `Root` only in `main`). Without this, signatures, sandboxing, and determinism are only advisory. *Confidence: high.*
2. **Use capability values plus `with` clauses, not an effect type system.** Explicit on `pub`, inferred privately, implicit forwarding by type, and an error on ambiguity. Tail-resumptive "handlers" are just trait impls, so dispatch is static after devirtualization. *Confidence: high on the principle, medium on exact syntax.*
3. **No effect variables. Get polymorphism from borrowed closures and a `Pure` auto-trait.** Effect-typed traits are replaced by authority stored in values. This is the main reduction in burden for humans and LLMs. *Confidence: medium-high*, because it needs a prototype over a real standard library to confirm it covers iterators, callbacks, and executors.
4. **The control-effect list is closed:** `Result`, panic, and `Io` suspension; no user continuations in v1; never multi-shot. *Confidence: high on multi-shot, medium on deferring one-shot handlers.*
5. **Ship deterministic simulation in std** (`Sim::seeded`, record/replay, deterministic hashing and executor). This turns implication 1 into a testing feature that people will actually want. *Confidence: high.*
6. **Manifest grants only escape hatches** (`unsafe`, `ffi`, build-time I/O), and compile-time code gets no capabilities. The lockfile carries an authority summary per package, and upgrades are diffed. *Confidence: high.*
7. **Async is the `Io` capability, with no keyword;** the compilation strategy is chosen per target. *Confidence: medium.* Stackful-first costs memory; whether stackless is feasible through `dyn` capability calls is still open.
8. **The language is defense in depth, not a security boundary for hostile code.** Mirror capability traits onto WIT interfaces so a distrusted package can be compiled into a separate Wasm component (WASI-Virt style) with no code changes. *Confidence: medium-low on the 1:1 mapping.*
9. **Diagnostics for agents.** Report "uses `Net` but does not declare it", "declares `Fs` but never uses it" (least authority, like an unused parameter), and "`ferox caps diff`" in review. *Confidence: medium.* This is unvalidated with LLMs.

## Open questions

1. **Granularity of capability kinds.** Coarse (`Io`) or Flix-like (`FsRead`/`FsWrite`/`Net`/`Exec`/`Env`/`Sys`/`NonDet`)? Finer kinds are more reviewable but make signatures longer. Bundles may settle it.
2. **Is allocation authority?** Zig passes `Allocator` explicitly; Verse has `<allocates>`. My lean is an ambient global allocator, with an opt-in `no_alloc` mode for embedded targets. This is unresolved.
3. **Logging and tracing ergonomics.** Should a write-only trace sink be ambient, like Roc's `dbg`, or a `Log` capability? Ambient logging was the attack vector in the 2025 crates.io incident.
4. **Address-dependent nondeterminism.** Pointer-to-integer casts and address-keyed maps break replay. Ban them in safe code, or make them a capability?
5. **Revocation and single-use capabilities.** Austral could not solve these in the type system. A caretaker or forwarder library pattern may be enough.
6. **Stackless versus stackful suspension through `dyn` capability calls**, and the maturity of Wasm stack switching. I could not verify the current phase of that proposal.
7. **Will agents over-request capabilities**, for example by asking for `Root`? Lints and review tooling must make least authority the easy path. No empirical data exists.
8. **Resource exhaustion** (CPU, memory, DoS) lies outside capability discipline and needs run-time limits.

## Sources

- Koka: [docs](https://koka-lang.github.io/koka/doc/index.html), [book](https://koka-lang.github.io/koka/doc/book.html), [Effect handlers, evidently](https://www.microsoft.com/en-us/research/publication/effect-handlers-evidently/), [Generalized evidence passing](https://www.microsoft.com/en-us/research/uploads/prod/2021/08/genev-icfp21.pdf), [TFP 2022 open-adjustment](https://prg.is.titech.ac.jp/papers/pdf/tfp2022adjustment.pdf)
- Effekt: [Effects as capabilities](https://ps.informatik.uni-tuebingen.de/publications/brachthaeuser20effects/), [Effects, capabilities, and boxes](https://pl.cs.uni-tuebingen.de/publications/brachthaeuser22effects/), [Capability-passing compilation](https://pl.cs.uni-tuebingen.de/publications/schuster20capability/), [history](https://effekt-lang.org/evolution), [Multiple resumptions, ICFP 2025](https://se.cs.uni-tuebingen.de/publications/muhcu2025multiple/)
- OCaml: [Retrofitting effect handlers](https://arxiv.org/pdf/2104.00250); [Data race freedom à la mode](https://popl25.sigplan.org/details/POPL-2025-popl-research-papers/23/Data-Race-Freedom-la-Mode); [Mode crossing](https://icfp26.sigplan.org/details/icfp-2026-icfp-papers/9/Mode-Crossing)
- [Unison abilities](https://www.unison-lang.org/docs/language-reference/abilities-and-ability-handlers); [Frank](https://arxiv.org/abs/1611.09259); [Lean do unchained](https://doi.org/10.1145/3547640)
- Flix: [LambdaDays 2025 slides](https://flix.dev/talks/lambdadays2025.pdf), [effect-oriented programming](https://doc.flix.dev/effect-oriented-programming.html), [Boolean unification](https://users-cs.au.dk/magnusm/papers/oopsla20/paper_b.pdf), [Qualified types with Boolean algebras](https://plg.uwaterloo.ca/~olhotak/pubs/oopsla25.pdf)
- Scala: [capture checking overview](https://scala-lang.org/api/3.x/docs/experimental/capture-checking/overview.html), [What's in the box](https://www.arxiv.org/abs/2509.07609), [System Capybara](https://arxiv.org/abs/2607.09383), [Classifying capabilities](https://arxiv.org/abs/2607.24504), [PR 24112](https://github.com/scala/scala3/pull/24112)
- Handler implementation: [Lexa](https://cs.uwaterloo.ca/~yizhou/papers/lexa-oopsla2024.pdf), [Zero-overhead lexical handlers](https://2025.splashcon.org/details/OOPSLA/203/Zero-Overhead-Lexical-Effect-Handlers), [Tunneling](https://www.cs.cornell.edu/andru/papers/tunnel-eff), [Soundly handling linearity](https://arxiv.org/pdf/2307.09383), [Yarrow](https://arxiv.org/abs/2607.15876)
- Industrial: [Roc functions](https://roc-lang.org/docs/main/langref/functions/), [Feldman gist](https://gist.github.com/rtfeldman/f46bcbfe5132d62c4095dfa687bb9aa4), [Roc blog](https://dev.to/zelenya/why-im-excited-about-roc-26j1), [Verse specifiers](https://dev.epicgames.com/documentation/en-us/uefn/specifiers-and-attributes-in-verse), [Nim effects](https://nim-docs.readthedocs.io/en/latest/manual/effect_system/), [Swift SE-0413](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0413-typed-throws.md), [Zig new async I/O](https://kristoff.it/blog/zig-new-async-io/), [Zig 0.16 notes](https://ziglang.org/download/0.16.0/release-notes.html), [LWN on Zig I/O](https://lwn.net/Articles/1046084), [Kotlin 2.4](https://kotlinlang.org/docs/whatsnew24.html), [Kotlin context parameters update](https://blog.jetbrains.com/kotlin/2025/04/update-on-context-parameters/), [Rust keyword generics](https://blog.rust-lang.org/inside-rust/2022/07/27/keyword-generics/), [Rust const traits 2026](https://goals.rust-lang.org/2026/const-traits.html)
- Capabilities: [Austral](https://borretti.me/article/how-capabilities-work-austral), [Pony derived authority](https://tutorial.ponylang.io/object-capabilities/derived-authority), [Pony authority hierarchy](https://patterns.ponylang.io/object-capabilities/authority-hierarchy), [Wyvern](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ECOOP.2017.20), [Joe-E spec](https://www2.eecs.berkeley.edu/Pubs/TechRpts/2008/EECS-2008-91.html), [Joe-E purity](https://people.eecs.berkeley.edu/%7Edaw/papers/pure-ccs08.pdf), [Mettler thesis](https://www2.eecs.berkeley.edu/Pubs/TechRpts/2012/EECS-2012-244.html), [Joe-E Wikipedia](https://en.wikipedia.org/wiki/Joe-E), [wasi.dev](https://wasi.dev/), [WASI 0.3](https://wasi.dev/releases/wasi-p3), [WASI 0.3 article](https://bytecodealliance.org/articles/WASI-0.3), [WASI-Virt](https://github.com/bytecodealliance/WASI-Virt), [Deno security](https://docs.deno.com/runtime/fundamentals/security/), [Deno permissions](https://docs.deno.com/runtime/reference/permissions.md), [cap-std](https://docs.rs/cap-std/2.0.1/cap_std), [JEP 411](https://openjdk.org/jeps/411), [JEP 486](https://openjdk.org/jeps/486)
- Supply chain: [Capslock on deps.dev](https://blog.deps.dev/capslock/), [Capslock caveats](https://github.com/google/capslock/blob/main/docs/caveats.md), [cargo-scan](https://arxiv.org/abs/2602.06466v1), [cackle](https://github.com/cackle-rs/cackle/blob/main/CONFIG.md), [Rust internals thread](https://internals.rust-lang.org/t/effect-system-crate-permissions/23529), [crates.io malicious crates](https://blog.rust-lang.org/2025/09/24/crates.io-malicious-crates-fasterlog-and-asyncprintln/)
- Determinism: [TigerBeetle VOPR](https://docs.tigerbeetle.com/about/vopr), [S2 DST](https://s2.dev/blog/dst), [Go synctest](https://pkg.go.dev/testing/synctest@master)
- LLMs: [Cassano et al.](https://arxiv.org/abs/2308.09895), [Mündler et al.](https://arxiv.org/abs/2504.09246), [CaMeL](https://arxiv.org/abs/2503.18813)
