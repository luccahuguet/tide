# 10. Concurrency and Parallelism for a Safe, Fast, Agent-Friendly Systems Language

Research date: 2026-10-19. Scope: concurrency (IO-bound tasks, servers, CLIs), parallelism (CPU/GPU throughput on pure code), and deterministic simulation testing (DST). Claims marked **Inference** are my reasoning, not something a source states. Everything else has a URL.

**Repository correction — 2026-11-12.** The HVM repository linked in F7
and Sources now lives at [HigherOrderCO/HVM1](https://github.com/HigherOrderCO/HVM1).
The old `HigherOrderCO/HVM` links below are retained as part of this
2026-10-19 evidence snapshot.

---

## Question

What is the simplest concurrency model for a new systems language that:

1. is data-race-free by construction,
2. is efficient for servers (very many connections) and for CLIs (no runtime tax),
3. avoids function coloring where possible,
4. is easy for Opus-class coding agents to write correctly,
5. gives pure/value code a deterministic parallelism story in the spirit of Bend, without HVM, and
6. makes deterministic simulation testing a language-level feature rather than a heroic library effort?

---

## Findings (cited)

### F1. Rust async: what it costs, and what 2024–2026 did and did not fix

- **Rust's own roadmap describes the problem.** The 2026 "Just add async" roadmap says "patterns which work in sync Rust don't transfer to async" and "async Rust is widely seen as qualitatively harder than sync Rust." It lists these gaps: no `&dyn Trait` with async methods, recursion needing "arcane signatures with explicit lifetimes, `Box`, `dyn Future`, and `Pin`", and no async cleanup in `Drop` ([roadmap](https://goals.rust-lang.org/2026/roadmap-just-add-async.html)).
- **Progress since 2023:** `async fn` in traits shipped in Rust 1.75 (Dec 2023) ([Rust blog](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits/)). Async closures (`async || {}`, plus the `AsyncFn*` traits) shipped in 1.85 in February 2025 ([Rust 1.85](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)).
- **Still open in late 2026:**
  - Return-type notation, the fix for the "Send bound problem", is implemented on nightly only. "Full stabilization is blocked on the next-gen trait solver work." An earlier stabilization PR was closed ([RTN goal](https://goals.rust-lang.org/2026/rtn.html)).
  - Native dyn dispatch for async trait methods is scheduled for 2026–2027 ([roadmap](https://goals.rust-lang.org/2026/roadmap-just-add-async.html)).
  - The "Continue Experimentation with Pin Ergonomics" goal was **discontinued** in mid-2026 ([program management, Jul–Aug 2026](https://blog.rust-lang.org/inside-rust/2026/08/31/program-management-2026-jul-aug/)).
  - The roadmap includes a `Move` trait (immovability as a property of the type) and "guaranteed destructors" to enable "safe scoped spawning for async" ([roadmap](https://goals.rust-lang.org/2026/roadmap-just-add-async.html)).
- **Pin was a shipping compromise.** without.boats, one of async Rust's designers, says Pin was chosen because it was "targeted, enforceable, and also shippable on time." He calls the need to pin a future trait object to await it "an unforced error." He also concludes that a language with stackful coroutines could be the better choice in other settings ([Why async Rust](https://without.boats/blog/why-async-rust/)).
- **Coloring is real.** Nystrom's five rules ("You can only call a red function from within another red function", among others) describe the split. Languages with multiple independent call stacks (Go, Lua, Ruby fibers) avoid it ([What color is your function](https://journal.stuffwithstuff.com/2015/02/01/what-color-is-your-function/)).
- **Executor fragmentation:**
  - async-std was discontinued in 2025 in favor of smol ([async-std PR #1099](https://github.com/async-rs/async-std/pull/1099); [Fedora](https://www.fedoraproject.org/wiki/Changes/Deprecate_async-std)).
  - Corrode: "libraries still need to be written against individual runtimes," mostly Tokio, and "the default mode for writing Rust should be _synchronous_" ([corrode.dev](https://corrode.dev/blog/async/)).
- **`Send + 'static` friction comes from multi-threaded defaults.** "The Original Sin of Rust async programming is making it multi-threaded by default… it curses all your code with the unholy `Send + 'static`" ([maciej.codes](https://maciej.codes/2022-06-09-local-async.html)).
- **The scoped-task trilemma.** A sound Rust API gives at most two of three properties: concurrency (the parent keeps running past the spawn), parallelizability, and borrowing. `thread::scope` and `rayon::join` choose parallelism plus borrowing ([without.boats](https://without.boats/blog/the-scoped-task-trilemma/)).
- **Cancellation by drop is a correctness hazard.**
  - Dropping a future cancels it at its current await point. `tokio::select!` drops every losing branch. Tokio's `Sender::send` is not cancel-safe: "If this future is dropped, the message is lost forever."
  - Mutexes held across awaits can leave invariants broken when the future is cancelled.
  - The talk notes there is no general solution in safe Rust today ([sunshowers, "Cancelling async Rust"](https://sunshowers.io/posts/cancelling-async-rust/)).
- **Where stackless async works well.** Embedded and no-heap targets. Embassy "requires no dynamic memory allocation, and runs on a single stack." Tasks compile into state machines that "obsolete the need for a traditional RTOS" ([embassy.dev](https://embassy.dev/)).

### F2. Green and virtual threads: successes and counter-evidence

- **Java Loom works for thread-per-request servers.**
  - Virtual threads (JEP 444, JDK 21) aim to "enable server applications written in the simple thread-per-request style to scale." The JEP says they "are not faster threads." It warns against pooling them and says to use thread-locals "only after careful consideration" ([JEP 444](https://openjdk.org/jeps/444)).
  - Pinning inside `synchronized` was a real scalability bug. JEP 491 (JDK 24) removed nearly all of it; native frames still pin ([JEP 491](https://openjdk.org/jeps/491)).
  - Structured concurrency is still in preview in JDK 26 (sixth preview) ([JEP 525](https://openjdk.org/jeps/525); [InfoQ](https://www.infoq.com/news/2026/01/timeout-joiner-refinements)).
- **Go needed preemption.** Go 1.14 made goroutines "asynchronously preemptible" so that "loops without function calls no longer potentially deadlock the scheduler" ([Go 1.14](https://go.dev/doc/go1.14)).
- **Go is uncolored but not race-free.** Uber's ThreadSanitizer-based detector found more than 2,000 races in 46 MLoC over six months; Go idioms make programs "highly susceptible to data races" ([Chabbi & Ramanathan, PLDI 2022](https://arxiv.org/abs/2204.00764)).
- **BEAM shows that isolation scales.** A new Erlang process uses 327 words, each process has its own heap, and "all data in messages sent between Erlang processes is copied," except refc binaries and literals ([Erlang efficiency guide](https://www.erlang.org/doc/system/eff_guide_processes.html)).
- **Counter-evidence 1: .NET tried green threads and stopped.**
  - The prototype was viable but slower: 162k vs 178k req/s on Plaintext.
  - "Interop with native code in green threads model is complex and comparatively slow": 100M P/Invokes went from about 300 ms to about 1,800 ms.
  - Thread-local statics caused surprises.
  - The team put the experiment on hold "primarily due to concerns about introducing a new programming model" and focused on runtime-async instead ([dotnet/runtimelab #2398](https://github.com/dotnet/runtimelab/issues/2398)).
- **Counter-evidence 2: Rust removed green threads before 1.0.** Reasons: FFI stack-switch cost, no GC to support stack copying, segmented-stack "hot split" costs, and embedded targets ([Why async Rust](https://without.boats/blog/why-async-rust/)).

### F3. Effects, direct style, and capability-passed IO

- **OCaml 5 effect handlers express async/await, generators, and lightweight threads.** They cost a mean 1% on macro-benchmarks that never use handlers ([Sivaramakrishnan et al., PLDI 2021](https://arxiv.org/abs/2104.00250)). Eio builds direct-style IO on top, with structured concurrency and resource-safe cancellation ([Eio 1.0, OCaml 2023](https://icfp23.sigplan.org/details/ocaml-2023-papers/5/Eio-1-0-Effects-based-IO-for-OCaml-5)).
- **OCaml 5 separates concurrency from parallelism.** Domains are "heavy-weight entities," map 1:1 to OS threads, and should not outnumber cores ([OCaml manual](https://ocaml.org/manual/5.3/parallelism.html)).
- **OCaml's memory model gives local DRF.** Race-free parts of a program keep sequential semantics even when other parts race, at "no overhead on x86, ~0.6% on ARM" ([Dolan, Sivaramakrishnan, Madhavapeddy, PLDI 2018](https://kcsrk.info/papers/pldi18-memory.pdf)).
- **Hiding suspension has a cost: the Ocsigen case.** Ocsigen migrated its whole stack to Eio in 2025. In July 2026 it decided to stay on Lwt, because under Eio "a call that turns out to be an RPC is syntactically indistinguishable from a local computation." Unhandled effects in browser callbacks also crash with `Effect.Unhandled` ([Ocsigen blog](https://ocsigen.org/blog/posts/why-ocsigen-stays-on-lwt-for-now.html)).
- **Zig passes IO as a parameter.**
  - "The `Io` interface is now expected to be provided by the caller, just like we already do with `Allocator`." The blocking implementation is "designed to generate machine code equivalent to using C." Thread-pool, green-thread (io_uring), and stackless (WASM) implementations are planned.
  - It separates `async` (order doesn't matter) from `concurrent` (must actually overlap): "Asynchrony is not concurrency."
  - Cancelled operations "return `error.Canceled`" ([Loris Cro](https://kristoff.it/blog/zig-new-async-io/)).
  - LWN notes that `Io.Evented` is still experimental. Commenters argue that taking an `Io` parameter is itself a color. Defenders reply that IO should be visible in APIs anyway for testability and security ([LWN, Dec 2025](https://lwn.net/Articles/1046084/)).
- **Theory for capability-passing effects.** "Effects as capabilities" (OOPSLA 2020) supplies it ([DOI](https://doi.org/10.1145/3428194)). Effekt's "contextual effect polymorphism" needs no annotations: "Just use what is in scope" ([effekt-lang.org](https://effekt-lang.org/)).

### F4. Data-race freedom mechanisms

- **Rust `Send`/`Sync`.** These are unsafe, auto-derived marker traits. Send means "safe to send to another thread," and `T: Sync` iff `&T: Send` ([Nomicon](https://doc.rust-lang.org/nomicon/send-and-sync.html)).
- **Data-race freedom does not prevent deadlocks.** In a study of real Rust code, all 59 blocking bugs were in safe code. Of 38 lock-acquisition hangs, 30 came from **double locking** and 7 from conflicting lock order. Many came from misunderstanding when lifetime-based implicit unlock happens. Insight 6 reads: "Lacking good understanding in Rust's lifetime rules is a common cause for many blocking bugs" ([Qin et al., PLDI 2020](https://cseweb.ucsd.edu/~yiying/RustStudy-PLDI20.pdf)).
- **Pony.** Six reference capabilities (`iso`, `trn`, `ref`, `val`, `box`, `tag`). "By sharing only immutable data and exchanging only isolated data we can have safe concurrent programs without locks" ([Pony tutorial](https://tutorial.ponylang.io/reference-capabilities/reference-capabilities.html)). Pony is still pre-1.0, with "regular… breaking changes" and a "very small" library pool ([Why not Pony](https://www.ponylang.io/discover/why-not-pony/)).
- **Swift 6.**
  - *Region-based isolation* (SE-0414) lets non-`Sendable` values cross isolation boundaries when they sit in a "disconnected" region. "An isolation region is a set of values that can only ever be referenced through other values within that set." It is based on PLDI 2022 work ([SE-0414](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0414-region-based-isolation.md); [Milano, Turcotti, Myers](https://www.cs.cornell.edu/andru/papers/gallifrey-types/)).
  - Swift's own vision document admits: "Writing a single-threaded program is surprisingly difficult under the Swift 6 language mode." It says "many programmers have resorted to reflexively writing `@MainActor` everywhere" ([approachable-concurrency vision](https://github.com/swiftlang/swift-evolution/blob/main/visions/approachable-concurrency.md)).
  - Swift 6.2 made nonisolated async functions run on the caller's actor by default, with `@concurrent` to opt out ([SE-0461](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0461-async-function-isolation.md)). It also added a module-level default of MainActor isolation, which practitioners criticize as a confusing mode split ([SE-0466 review](https://forums.swift.org/t/se-0466-control-default-actor-isolation-inference/78321); [Massicotte](https://www.massicotte.org/default-isolation-swift-6_2)).
  - Swift actors are reentrant by design to avoid deadlocks, so "actor-isolated state can change across an `await`" ([SE-0306](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0306-actors.md)).
- **Verona behaviour-oriented concurrency (BoC).**
  - BoC targets the actor model's weak spot: "updating multiple actors as a single atomic operation." A behaviour gets exclusive access to a set of "cowns" (concurrent owners) and runs only when all of them are available. In the Savina benchmarks it gave more compact code and comparable or better performance ([Cheeseman et al., OOPSLA 2023](https://www.microsoft.com/en-us/research/publication/when-concurrency-matters-behaviour-oriented-concurrency/)).
  - Cheeseman's thesis argues BoC is deadlock-free and race-free, provided the host language enforces isolation ([thesis record](https://spiral.imperial.ac.uk/entities/publication/a1259c4b-6ffe-45d2-ac8b-3e1162d4c107)).
  - Verona itself "is not ready to be used outside of research" ([microsoft/verona](https://github.com/microsoft/verona)).

### F5. Structured concurrency, cancellation, and timeouts

- **Trio nurseries.** "go statements are a form of goto statement." A nursery "doesn't exit until all the tasks inside it have exited." This restores black-box functions, reliable cleanup, and error propagation ([Smith](https://vorpus.org/blog/notes-on-structured-concurrency-or-go-statement-considered-harmful/)).
- **Trio cancel scopes.**
  - They are **level-triggered**: once cancelled, every later checkpoint fails.
  - They nest, and they apply implicitly to everything inside them, including nursery children.
  - They support shielding for cleanup.
  - They fix the composition failure of per-call timeouts, which make it "not possible to guarantee that requests.get will return in any finite time." Explicit tokens (Go `context`, C# tokens) are the same idea but rely on boilerplate that "humans suck at" ([Smith](https://vorpus.org/blog/timeouts-and-cancellation-for-humans/)).
- **Kotlin, Swift, and Java share the model.** Kotlin coroutines form "a tree hierarchy of parent and child tasks with linked lifecycles" ([Kotlin docs](https://kotlinlang.org/docs/coroutines-basics.html)). In Swift, a child task "does not persist beyond the scope in which it was created," and cancellation is "fully cooperative" ([SE-0304](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0304-structured-concurrency.md)). Java's StructuredTaskScope follows the same shape ([JEP 525](https://openjdk.org/jeps/525)).

### F6. IO in the io_uring era

- **io_uring breaks drop-based cancellation of borrowed buffers.** The kernel may write into a buffer after its future is dropped, so "the kernel must own the buffer." Ownership-passing APIs are the sound design ([without.boats, io-uring](https://without.boats/blog/io-uring/)).
- **io_uring is often unavailable.**
  - "60% of the submissions exploited the io_uring component" in Google's kCTF VRP. It is "disabled on production Google servers," disabled on ChromeOS, and unreachable to Android apps ([Google Security Blog](https://security.googleblog.com/2023/06/learnings-from-kctf-vrps-42-linux.html)).
  - Docker's default seccomp profile blocks `io_uring_setup/enter/register` ([Docker docs](https://docs.docker.com/engine/security/seccomp/)).
  - LWN reported in January 2026 that seccomp cannot see io_uring operations, and that the maintainer is developing a restriction mechanism ([LWN](https://lwn.net/Articles/1054225/)).

### F7. Deterministic parallelism

- **Determinism by default, nondeterminism opt-in.** Bocchino et al. argued for this in "Parallel Programming Must Be Deterministic by Default" ([HotPar 2009](https://static.usenix.org/event/hotpar09/tech/full_papers/bocchino/bocchino_html/)).
- **Rayon.** Its APIs "guarantee data-race freedom," and parallel iterators "are guaranteed to produce the same results as their sequential counterparts," except for side-effect order ([rayon](https://github.com/rayon-rs/rayon)).
- **Cilk reducers.** They give correct final results for associative operations, with a total order on views matching the serial program ([OpenCilk](https://www.opencilk.org/doc/reference/reducers/)).
- **MPL (Parallel ML).**
  - Its GC is built on hierarchical heaps and *disentanglement*: concurrent tasks don't hold pointers to each other's fresh allocations.
  - It "scales to hundreds of cores" and handles heaps around 1 TB ([MPL](https://github.com/MPLLang/mpl)).
  - The PLDI 2023 paper extends this to programs that do entangle, at small overhead. The authors report MPL is "competitive with languages such as C++, Go, Java, OCaml" ([abstract](https://cs.nyu.edu/~shw8119/23/pldi23-abstract.txt)).
  - Its POPL 2024 "automatic parallelism management" removes manual grain sizes (`parform` vs `parfor`) ([MPL](https://github.com/MPLLang/mpl)).
- **Heartbeat scheduling.** It provides "provably efficient results for arbitrary nested-parallel programs, without the tuning needed for controlling parallelism overheads." It creates threads only at a beat and is competitive with hand-tuned Cilk Plus ([Acar et al., PLDI 2018](https://www.chargueraud.org/research/2018/heartbeat/heartbeat.pdf)).
- **Futhark.** A "data-parallel, and purely functional array language" for "relatively small but compute-intensive parts of an application" ([futhark-lang.org](https://futhark-lang.org/)). In July 2026 it can compile "any Futhark program to parallel GPU code" via full flattening (not yet merged). Good performance on arbitrary irregular nesting "is a major research problem in itself," and some benchmarks regressed until a `#[sequential]` attribute was added. The same post calls Data Parallel Haskell abandoned ([Futhark blog](https://futhark-lang.org/blog/2026-07-31-full-flattening.html); [DPH wiki](https://wiki.haskell.org/GHC/Data_Parallel_Haskell)).
- **Halide.** Separates "the algorithm – no storage or order" from "the schedule – defines order, locality" ([halide-lang.org](https://halide-lang.org/)).
- **Bend 1 / HVM.**
  - LWN (March 2025): "Programs written in Bend do not yet have competitive performance compared to existing functional programming languages." The docs called the codegen "embarrassingly bad."
  - Inherently serial algorithms gain nothing, and graph reduction suits CPUs poorly because of pointer chasing ([LWN](https://lwn.net/Articles/1011803/)).
  - HVM1 was "from 50% faster to 3x slower" than GHC single-threaded ([HVM README](https://github.com/HigherOrderCO/HVM)).
- **Bend 2 (17 Sep 2026)** is a from-scratch rewrite:
  - It "compiles natively to a single C file" (clang for CPU, CUDA/Metal for GPU). Interaction nets "don't exist at runtime."
  - It has an affine dependent type checker that doubles as a proof checker, and a `LAWS.bend`/`PROOF.bend` workflow aimed at AI agents.
  - A reviewer porting Rust code found "no number here shows Bend faster than Rust on the same work" ([akitaonrails review](https://akitaonrails.com/en/2026/09/19/new-ai-language-just-released-bend-2/); [bend-lang.com](https://bend-lang.com/)).
  - A pre-launch digest says Taelin dropped interaction nets because he could not make them as fast as lower-order approaches on everyday hardware ([digest; secondary, unverified](https://digg.com/ai/xr75igqo)).
  - The site's description of parallelism: "Split the work in two, and Bend spreads the calls over every core it can find, then joins them back… No threads, no locks, no kernels to write" ([bend-lang.com](https://bend-lang.com/)).

### F8. GPU programming models

- **Mojo.** Kernels are ordinary Mojo functions on a CUDA-like grid/block model, launched through `DeviceContext`, which supports CUDA, HIP, and Metal ([Modular docs](https://max.modular.com/gpu/fundamentals)).
- **Triton.** It inverts CUDA's model: CUDA is "Scalar Program, Blocked Threads," Triton is "Blocked Program, Scalar Threads." The compiler automates coalescing, shared memory, and tensor-core selection ([Triton](https://triton-lang.org/main/programming-guide/chapter-1/introduction.html)).
- **Rust on GPUs is still unfinished.**
  - The Rust CUDA project was rebooted in 2025, with a long-term aim of an in-tree backend unified with rust-gpu ([announcement](https://rust-gpu.github.io/blog/2025/01/27/rust-cuda-reboot/)).
  - `std::offload` could move data but not yet launch kernels. Its goal was marked "Will not complete" in April 2026 ([goal](https://goals.rust-lang.org/2025h2/finishing-gpu-offload.html); [April 2026 update](https://rust-lang.github.io/goals/blog-post-2026-04.html)).

### F9. Deterministic simulation testing

- **FoundationDB.** Simulation runs "an entire FoundationDB cluster within a single-threaded process" on the Flow runtime, with simulated time, network, disks, machines, and datacenters. The team runs "tens of thousands of simulations every night" and estimates "roughly one trillion CPU-hours" in total ([FDB testing](https://apple.github.io/foundationdb/testing.html)).
- **TigerBeetle.** In the VOPR, "all non-deterministic parts of the system are stubbed out" (clock, network, disk). "The seed and Git commit hash can be used to replay back the exact simulation and bug." The team says "one minute of VOPR time is equivalent to days of real-world testing" ([VOPR](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/internals/vopr.md)). TigerStyle adds static allocation, bounded loops and queues, and always-on assertions as "a force multiplier for discovering bugs by fuzzing" ([TigerStyle](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md)).
- **Antithesis.** A deterministic bhyve-based hypervisor: one physical core per instance, virtual time from retired instructions, mediated IO ([Antithesis](https://antithesis.com/blog/deterministic_hypervisor/)).
- **Go `testing/synctest`** (experimental in 1.24, GA in 1.25):
  - Goroutines run in a "bubble" with a fake clock. "Time advances in the bubble when all goroutines are blocked."
  - If nothing can advance, the bubble is deadlocked and `Run` panics. Mutexes and external IO don't count as "durably blocked" ([Go blog](https://go.dev/blog/synctest); [Go 1.25](https://go.dev/doc/go1.25)).
  - Go 1.26 adds a reachability-based goroutine leak profile ([Go 1.26](https://tip.golang.org/doc/go1.26)).
- **Library-level DST leaks.**
  - S2 runs single-threaded Tokio with turmoil plus libc overrides, and found that any dependency "might use threads, RNGs, system time, or I/O." Its CI meta-test reruns a seed and diffs TRACE logs. DST found 17 notable bugs ([S2](https://s2.dev/blog/dst)).
  - Phil Eaton: without a whole deterministic machine "you can't test your whole program." He also writes, "determinism, even among DST practitioners, remains a spectrum" ([Eaton](https://notes.eatonphil.com/2024-08-20-deterministic-simulation-testing.html)).
  - Loom's custom scheduler enabled Java DST at about 40k Raft rounds/s on one core, but the pluggable scheduler API was removed from the preview ([jbaker](https://jbaker.io/2022/05/09/project-loom-for-distributed-systems/)).
  - madsim requires swapping in simulator crates ([madsim](https://github.com/madsim-rs/madsim)). Shuttle offers randomized PCT scheduling that is "not sound" but scales past Loom ([shuttle](https://github.com/awslabs/shuttle)).

### F10. LLM evidence on concurrency

- **LLMs reason poorly about relaxed memory.** GPT-4-class models detect races and deadlocks well under sequential consistency, but "all selected LLMs face significant challenges verifying program correctness under relaxed memory models" ([Jain & Purandare, 2025](https://arxiv.org/abs/2501.14326)).
- **Race prediction is weak.** On DRPBench (1,003 SV-COMP programs), the best model reached F1 74.89% and most scored below 60%. The failure modes were "shared-variable distraction" and "synchronization-logic myopia" ([DRPBench, ICML 2026](https://proceedings.mlr.press/v306/guo26z.html)).
- **Concurrent code generation is a known weak spot** ([CONCUR, ISSTA 2026](https://arxiv.org/abs/2603.03683)).
- I found no peer-reviewed study of LLM-written async Rust specifically. That gap is itself a finding.

---

## What works / what failed and why

| Approach | Worked | Failed or costly | Root cause |
|---|---|---|---|
| Rust stackless async | Zero-cost state machines; embedded (Embassy) | Pin, Send bounds, dyn async, cancel-by-drop, runtime lock-in; RTN still unstable 3 years after AFIT | Futures are movable values that can be dropped at any await, and the language has no standard executor interface |
| Go goroutines | No coloring; simple servers | Data races at scale (2,000 at Uber); leaks need runtime profiling | Shared memory with no ownership; closures capture by reference |
| Java virtual threads | Thread-per-request at scale | Pinning (fixed in JDK 24); structured concurrency still preview after 6 rounds | Retrofit onto existing monitors, thread-locals, and native frames |
| .NET green threads | Viable prototype | Abandoned: slow FFI, thread-local surprises, two programming models | Retrofit next to an existing async/await |
| BEAM processes | Isolation, per-process GC, fault tolerance | Copying messages; not a systems-level performance target | Isolation via copying |
| OCaml 5 effects/Eio | Uncolored direct style at ~1% cost | Ocsigen reverted: suspension points became invisible | Effects untracked in types |
| Zig `Io` parameter | One codebase, blocking or evented; C-equivalent blocking path | Evented backend still experimental; `async` vs `concurrent` misuse not compiler-checked | Too new; the parameter is a visible "color" by design |
| Swift 6 isolation | Compile-time data-race safety; regions reduce `Sendable` noise | Approachability crisis, mode flags, reentrancy hazards | Concurrency assumed by default; actors interleave at `await` |
| Pony refcaps | Lock-free race-freedom proofs | Six capabilities; tiny ecosystem | High annotation burden |
| Verona BoC | Atomic multi-resource behaviours; deadlock- and race-free | Research only | Young |
| Rayon/Cilk/MPL fork-join | Deterministic results, near-linear scaling | Grain-size tuning (heartbeat fixes it) | n/a |
| DPH / NESL vectorisation | Theory | Abandoned | Flattening cost and compiler complexity |
| HVM/Bend 1 | Automatic parallelism demos | Poor single-core speed; Bend 2 dropped runtime interaction nets | Graph reduction is pointer-chasing |
| DST (FDB, TigerBeetle) | Best-in-class reliability | Requires owning every nondeterminism source; libraries leak | Ambient authority in the language and std |

---

## Recommended design

The design is one mechanism used three ways. **Every source of nondeterminism and suspension is a capability value, `Io`, that functions receive explicitly. Code without `Io` is pure and deterministic.** That single line in the type system decides:

1. where a task can suspend or be cancelled (concurrency),
2. what may be auto-parallelized (parallelism), and
3. what a simulator must intercept (DST).

### Deliverable 1: Concurrency through structured tasks over an `Io` capability

**Surface model (what agents write):**

```
fn fetch_all(io: Io, urls: &[Url]) -> Result<Vec<Page>, Error> {
    io.scope(|s| {                                 // nursery: returns only after all children end
        let hs: Vec<_> = urls.iter().map(|u| s.spawn(|io| http::get(io, u))).collect();
        hs.into_iter().map(|h| h.join()).collect() // first error cancels siblings, then propagates
    })
}

fn transfer(io: Io, a: &Cown<Account>, b: &Cown<Account>, amt: u64) -> Result<(), Cancelled> {
    io.with_deadline(5.s, |io| {
        lock (a, b) |x, y| { x.balance -= amt; y.balance += amt; }   // no `io` in scope inside
        audit::log(io, amt)
    })
}
```

**Rules:**

1. **No `async`/`await` keywords and no futures in user code.** A function suspends only by calling an operation on `Io` (or on a value derived from it, such as a socket). The parameter is visible, greppable, and generic code accepts it like any other value. That answers Ocsigen's objection (F3).
   - Unlike Zig, `Io` is tracked by the type checker as a capability: a closure capturing `io` is effectful. Higher-order functions are capability-polymorphic in Effekt's contextual style (F3), so one `map` serves pure and effectful closures.
   - Plumbing cost can later be reduced with implicit/contextual parameters (open question).
2. **Structured only.** Tasks are spawned into a scope (nursery) and cannot outlive it.
   - Children may **borrow** from the enclosing frames, which is the parallelism+borrowing corner of the trilemma (F1).
   - Background work goes into a long-lived scope passed down as a capability, Trio style.
   - **Inference:** with stackful tasks, a scope is a stack frame that cannot be `mem::forget`-ed, so scoped borrowing is sound without Rust's "guaranteed destructors" work.
3. **Cancellation:** level-triggered cancel scopes with deadlines (F5).
   - Every `Io` operation is a checkpoint and returns `Err(Cancelled)` once its scope is cancelled.
   - **A suspended computation is never dropped.** It observes cancellation as an error and unwinds normally, running destructors.
   - `race`/`select` cancel the losers and *join* them.
   - Operations that consume ownership (`send(msg)`) return the value inside the error, so cancellation cannot silently lose data (the Tokio `send` hazard in F1). Shielded scopes are allowed for cleanup.
4. **Shared mutable state** has two forms:
   - message passing (channels of `Send` values), and
   - `lock (a, b, …) |x, y| { … }` over `Cown<T>` cells, which acquires all named cells atomically in a global order (BoC-inspired, F4).
   - The body is checked to have **no `Io` in scope and no nested `lock`**.
   - **Inference:** this makes deadlock impossible for this primitive: no nesting, all-at-once acquisition. It also rules out double-locking (30 of 38 lock hangs in Rust, F4), holding a lock across a suspension point (the Tokio mutex hazard), and reentrancy interleavings (Swift actors, F4).
   - Raw `Mutex`/atomics remain in a lower-level module for library authors.
5. **Data-race freedom:**
   - Ownership and borrowing (exclusive XOR shared), plus two auto-derived marker traits: `Send` (transferable between tasks) and `Share` (shareable by reference). LLMs already know these semantics from Rust.
   - **Inference, important:** make the **task** the isolation domain, not the OS thread. There are no ambient thread-locals, only task-locals carried by `Io`. Values that never leave a task (an `Rc` created and dropped inside it) then need no `Send`, even if the scheduler migrates the task between cores. This mirrors Swift's "task-isolated region" (SE-0414) and removes Rust's whole "future is not Send" error class.
   - Thread-bound resources (GUI handles, some FFI objects) carry a `ThreadBound` marker that pins the task while held, like Java's pinning but explicit.
6. **Execution strategy is an implementation of `Io`, chosen by the binary, never by libraries** (Zig's insight, F3). The standard library defines the `Io` interface, so there is one ecosystem, not Tokio-versus-the-rest (F1).
   - `Io::blocking()`: direct syscalls, no runtime, no threads. This is the default for CLIs and should cost what C costs.
   - `Io::threads(n)`: OS threads, simple and robust.
   - `Io::evented()`: **stackful** lightweight tasks on per-core schedulers with work stealing. Backends are io_uring when available, else epoll, kqueue, or IOCP. The runtime falls back automatically, because io_uring is blocked in many deployments (F6).
   - `Io::sim(seed)`: the simulator (Deliverable 3).
7. **Why stackful, and how.** Stackful tasks avoid Pin entirely: frames never move, so borrows across suspension points are ordinary borrows. They also keep normal backtraces and debuggers and need no state-machine transform, which is good for compile speed.
   - **Inference:** to avoid the GC-dependent stack copying that Rust rejected (F2), use non-moving stacks: a fixed virtual reservation per task (for example 256 KiB to 1 MiB, guard page, lazily committed). RSS is then the pages actually touched. Real data points for comparison: 327 words per BEAM process (F2). Measure before committing.
   - Where the compiler can bound the stack (no recursion or `dyn` in the task's call graph, as TigerStyle already demands, F9), allocate exactly that.
   - For no-MMU and WASM targets, compile `Io`-taking functions to state machines instead. The `Io` capability marks exactly which functions need it; Zig plans the same split (F3).
8. **FFI:** a foreign call runs on the task's stack only if declared `nonblocking` with a stack budget. Otherwise it hops to a carrier or blocking thread. The .NET experiment shows naive stack switching on every native call is the main green-thread tax (F2), so it must stay optional and cheap.
9. **Preemption:** tasks are cooperative at `Io` calls. CPU-heavy pure work belongs in `par` (Deliverable 2), not in IO tasks. A runtime watchdog flags long non-yielding stretches. This is the Go 1.14 lesson (F2), handled with diagnostics before signals. Open question.

### Deliverable 2: Deterministic parallelism for pure and value code (Bend's spirit, no HVM)

1. **Semantics stay sequential.**
   - `par { a(), b() }`, `join`, and parallel iterators (`par_map`, `par_reduce`, `par_scan`, `par_filter`) express **potential** parallelism.
   - They are allowed only on closures with **no `Io` capability and no reachable interior mutability through shared references** (a `Freeze`-like auto-trait). Exclusive `&mut` splits (`par_chunks_mut`) are allowed.
   - Under these rules the result equals sequential execution (Rayon's guarantee, F7), so proofs about sequential code carry over to parallel runs. **Inference, high value for the optional-proofs goal.**
2. **Granularity is the runtime's job, not the programmer's.** Use heartbeat-style scheduling (F7): a `par` costs about a branch until a heartbeat promotes it to a real task. Then `par` can be sprinkled liberally, by agents too, without performance cliffs. This is MPL's `parform` lesson.
3. **Bend's spirit:** an optional compiler pass marks independent pure calls (e.g. the two recursive calls of divide-and-conquer, independent `let`s, and `map` over arrays) as implicit `par`. That gives "write sequential code, get parallelism," as Bend 2 does, on ordinary native code.
   - Evidence that HVM isn't needed: Bend 2 itself abandoned runtime interaction nets and compiles to C and CUDA (F7).
   - Ship explicit `par` first, and the automatic pass behind a flag until benchmarks justify making it the default.
4. **Bitwise reproducibility.** Reductions use a fixed tree shape that depends on input size, not thread count or steal order. Float sums are then identical on 1 or 128 cores, which DST, caching, and proofs all need. Nondeterministic variants (`par_unordered`, racy accumulators) exist only behind explicit names (Bocchino, F7).
5. **Allocation.**
   - **Inference:** pure parallel tasks are disentangled by construction, because ownership forbids publishing fresh allocations to siblings. Per-task bump arenas can therefore be merged into the parent at join, as in MPL's hierarchical heaps (F7), without a GC.
6. **GPU, later.** Keep a small pure-array IR of second-order array combinators (map, reduce, scan, scatter, histogram) that a Futhark-like flattener can target. Expose `#[gpu]` only on functions in that subset.
   - Futhark needed until 2026 to compile irregular nesting at all, and performance is still research. Rust's `std::offload` stalled. Mojo and Triton show the explicit-kernel and tile routes (F7, F8).
   - GPU is a phase-3 feature: design for it, don't ship it in v1.

### Deliverable 3: Deterministic simulation testing as a language feature

Library-level DST fails because the rest of the world (dependencies, std, libc) has ambient access to time, randomness, threads, and IO (F9). The language fix is to **remove ambient authority**.

1. **Every nondeterminism source comes from `Io`:** clock, sleep, RNG, network, filesystem, environment and args, process spawn, task spawn, and foreign calls declared effectful. Std has no global `now()`, RNG, or `thread::spawn`.
   - Hash-map seeds come from `Io` or are fixed. Pointer addresses are unobservable in safe code, so they cannot leak into ordering.
   - Therefore, **any function without `Io` is deterministic by the type system, and any program is simulable by passing `Io::sim`.** Dependencies cannot leak, because they had to take `Io` too.
2. **`Io::sim(seed)` semantics:**
   - All tasks of all simulated nodes run in one process under a seeded scheduler that picks among runnable tasks (random or PCT-style, as in Shuttle).
   - Virtual time advances only when every task is durably blocked (synctest's rule, F9).
   - If none can ever advance, report a deadlock with the full blocked-task graph. Leaked tasks are impossible (structured scopes), and orphaned waits are reported.
   - Fault injection on every `Io` operation: drop, delay, reorder, partition, disk corruption, torn writes, crash and restart of a node scope.
   - Declarative fault-injection points in code are compiled out unless `cfg(sim)`.
3. **Pure parallelism stays parallel under simulation.** `par` regions are result-deterministic, so the simulator can run them on real cores without breaking reproducibility. **Inference:** this lifts the single-core limit that FDB and Antithesis instances live under (F9).
4. **Built-in harness:**
   - `test sim "name" (sim) { … }` runs N seeds in CI.
   - A failure prints `seed + commit` and replays bit-for-bit, as TigerBeetle does.
   - A built-in determinism meta-test runs a seed twice and diffs the event trace, as S2 does.
   - The trace is a structured artifact an agent can read: task tree, `Io` ops, faults, and assertion failures.
5. **Coverage check:** a `sim` profile compile check lists any reachable foreign effect without a simulator model, so the hole is explicit rather than silent. Antithesis-style hypervisor testing remains the complement for real binaries and C dependencies.
6. **Style support:** optional `bounded` lints for no recursion, bounded queues, and static capacity, plus always-on assertions (TigerStyle). These also enable exact stack sizing (Deliverable 1, rule 7).

**Why this suits LLM agents (Inference, grounded in F10).** Models reason acceptably under sequential consistency and badly about relaxed memory and custom synchronization. This design removes relaxed-memory reasoning and custom synchronization from application code, along with invisible suspension, Pin and `Send` puzzles, and executor choice. It also gives agents a cheap, reproducible oracle: run 10,000 seeds and get a replayable trace on failure.

---

## Design implications (ranked)

1. **Make `Io` an explicit, type-tracked capability. Use no async/await keywords and expose no ambient time, randomness, IO, or threads.** It unifies uncolored-in-practice concurrency, the pure/parallel boundary, testability, and DST. *Confidence: high on direction (Zig, Effekt, FDB/TigerBeetle, Ocsigen evidence); medium on ergonomics of plumbing.*
2. **Allow only structured concurrency: scoped nurseries whose children may borrow, plus Trio-style level-triggered cancel scopes delivered as `Err(Cancelled)` at `Io` calls. Never cancel by dropping a suspended computation.** This eliminates Rust's cancel-safety class and leaked tasks. *Confidence: high.*
3. **Let the binary pick the execution strategy through the `Io` implementation (blocking / threads / evented / sim), never the library.** CLIs get a zero-runtime blocking path and there is one ecosystem. *Confidence: high.*
4. **Implement evented `Io` with stackful, non-moving, lazily committed task stacks; use compiler stack bounds where provable and a stackless transform only for no-MMU/WASM targets.** This removes Pin and keeps backtraces. *Confidence: medium; needs memory and FFI benchmarks.*
5. **Make the task the isolation domain: ownership plus `Send`/`Share` checked only at task boundaries, task-locals instead of thread-locals, and an explicit `ThreadBound` that pins.** This removes "future is not Send" friction. *Confidence: medium-high; requires a soundness argument along the lines of SE-0414.*
6. **Provide shared state as `lock (a, b) {}` over `Cown`s: atomic multi-acquire, no nesting, no `Io` inside.** It is deadlock-free and double-lock-free by construction. *Confidence: medium-high.*
7. **Give parallelism sequential semantics: `par` and parallel iterators restricted to `Io`-free, interior-mutability-free closures; heartbeat granularity control; fixed reduction trees for bitwise reproducibility; nondeterminism opt-in by name.** *Confidence: high (Rayon, Cilk, MPL, heartbeat evidence).*
8. **Pursue Bend's automatic parallelism as a compiler pass over native code, not as a runtime model (HVM).** Bend 2 itself dropped runtime interaction nets. *Confidence: medium.*
9. **Ship DST in the box: `Io::sim(seed)` with virtual time, a seeded scheduler, fault injection, deadlock reports, trace diffing, and a compile-time list of unmodeled foreign effects.** *Confidence: high on value, medium on achieving full coverage.*
10. **Treat io_uring as opportunistic with automatic fallback, and make IO APIs ownership-passing (the kernel owns buffers).** *Confidence: high.*
11. **Design a pure-array IR now for later GPU lowering, but keep the GPU out of v1.** *Confidence: medium.*

---

## Open questions

1. **Plumbing ergonomics.** Is an explicit `io` parameter tolerable at scale, or does the language need implicit/contextual capability parameters (Effekt-style)? How do agents fare with each? This needs an experiment with real agent sessions.
2. **Stack memory and FFI cost.** What do fixed-reservation stacks really cost per task (RSS, page faults, TLB) at 1M connections against Go and Loom? What does a `nonblocking` FFI call cost against a stack hop?
3. **Soundness of task-as-isolation-domain under work stealing.** Which std types must be `ThreadBound`? How do task-locals interact with FFI libraries that use OS thread-locals?
4. **Preemption.** Is cooperative scheduling plus a watchdog enough for servers, or is signal-based preemption needed, as in Go 1.14? How does preemption interact with FFI?
5. **`lock` expressiveness.** Can fine-grained, data-dependent locking (e.g. locking a hash-map bucket chosen at runtime) be expressed with atomic multi-acquire, or is a lower-level escape hatch unavoidable?
6. **Automatic parallelism heuristics.** When does auto-marking independent pure calls help, and when does it hurt cache locality? Futhark's `#[sequential]` regressions are a warning.
7. **Distributed simulation scope.** Should `Io::sim` model multiple processes with separate address spaces, or only nodes as scopes within one process? How should crash consistency (torn writes) be modeled per storage API?
8. **Float determinism across backends.** Is it achievable between CPU and GPU, or only within one backend?
9. **Evidence gap.** No study measures LLM success rates on async Rust versus green-thread or capability-passing code. The project could run that comparison itself.

---

## Sources

**Rust async**
- https://goals.rust-lang.org/2026/roadmap-just-add-async.html
- https://goals.rust-lang.org/2026/rtn.html
- https://blog.rust-lang.org/inside-rust/2026/08/31/program-management-2026-jul-aug/
- https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits/
- https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/
- https://without.boats/blog/why-async-rust/
- https://without.boats/blog/the-scoped-task-trilemma/
- https://without.boats/blog/io-uring/
- https://sunshowers.io/posts/cancelling-async-rust/
- https://journal.stuffwithstuff.com/2015/02/01/what-color-is-your-function/
- https://github.com/async-rs/async-std/pull/1099
- https://www.fedoraproject.org/wiki/Changes/Deprecate_async-std
- https://corrode.dev/blog/async/
- https://maciej.codes/2022-06-09-local-async.html
- https://embassy.dev/
- https://doc.rust-lang.org/nomicon/send-and-sync.html
- https://cseweb.ucsd.edu/~yiying/RustStudy-PLDI20.pdf

**Green and virtual threads**
- https://openjdk.org/jeps/444
- https://openjdk.org/jeps/491
- https://openjdk.org/jeps/525
- https://www.infoq.com/news/2026/01/timeout-joiner-refinements
- https://github.com/dotnet/runtimelab/issues/2398
- https://go.dev/doc/go1.14
- https://arxiv.org/abs/2204.00764
- https://www.erlang.org/doc/system/eff_guide_processes.html

**Effects and capability-passed IO**
- https://arxiv.org/abs/2104.00250
- https://icfp23.sigplan.org/details/ocaml-2023-papers/5/Eio-1-0-Effects-based-IO-for-OCaml-5
- https://ocaml.org/manual/5.3/parallelism.html
- https://kcsrk.info/papers/pldi18-memory.pdf
- https://ocsigen.org/blog/posts/why-ocsigen-stays-on-lwt-for-now.html
- https://kristoff.it/blog/zig-new-async-io/
- https://lwn.net/Articles/1046084/
- https://doi.org/10.1145/3428194
- https://effekt-lang.org/

**Data-race freedom: Swift, Pony, Verona**
- https://github.com/swiftlang/swift-evolution/blob/main/visions/approachable-concurrency.md
- https://github.com/swiftlang/swift-evolution/blob/main/proposals/0414-region-based-isolation.md
- https://github.com/swiftlang/swift-evolution/blob/main/proposals/0461-async-function-isolation.md
- https://github.com/swiftlang/swift-evolution/blob/main/proposals/0306-actors.md
- https://github.com/swiftlang/swift-evolution/blob/main/proposals/0304-structured-concurrency.md
- https://forums.swift.org/t/se-0466-control-default-actor-isolation-inference/78321
- https://www.massicotte.org/default-isolation-swift-6_2
- https://www.cs.cornell.edu/andru/papers/gallifrey-types/
- https://tutorial.ponylang.io/reference-capabilities/reference-capabilities.html
- https://www.ponylang.io/discover/why-not-pony/
- https://www.microsoft.com/en-us/research/publication/when-concurrency-matters-behaviour-oriented-concurrency/
- https://spiral.imperial.ac.uk/entities/publication/a1259c4b-6ffe-45d2-ac8b-3e1162d4c107
- https://github.com/microsoft/verona

**Structured concurrency and cancellation**
- https://vorpus.org/blog/notes-on-structured-concurrency-or-go-statement-considered-harmful/
- https://vorpus.org/blog/timeouts-and-cancellation-for-humans/
- https://kotlinlang.org/docs/coroutines-basics.html

**IO in the io_uring era**
- https://security.googleblog.com/2023/06/learnings-from-kctf-vrps-42-linux.html
- https://docs.docker.com/engine/security/seccomp/
- https://lwn.net/Articles/1054225/

**Deterministic parallelism**
- https://static.usenix.org/event/hotpar09/tech/full_papers/bocchino/bocchino_html/
- https://github.com/rayon-rs/rayon
- https://www.opencilk.org/doc/reference/reducers/
- https://github.com/MPLLang/mpl
- https://cs.nyu.edu/~shw8119/23/pldi23-abstract.txt
- https://www.chargueraud.org/research/2018/heartbeat/heartbeat.pdf
- https://futhark-lang.org/
- https://futhark-lang.org/blog/2026-07-31-full-flattening.html
- https://wiki.haskell.org/GHC/Data_Parallel_Haskell
- https://halide-lang.org/

**Bend and HVM**
- https://lwn.net/Articles/1011803/
- https://github.com/HigherOrderCO/HVM
- https://akitaonrails.com/en/2026/09/19/new-ai-language-just-released-bend-2/
- https://bend-lang.com/
- https://digg.com/ai/xr75igqo

**GPU**
- https://max.modular.com/gpu/fundamentals
- https://triton-lang.org/main/programming-guide/chapter-1/introduction.html
- https://rust-gpu.github.io/blog/2025/01/27/rust-cuda-reboot/
- https://goals.rust-lang.org/2025h2/finishing-gpu-offload.html
- https://rust-lang.github.io/goals/blog-post-2026-04.html

**Deterministic simulation testing**
- https://apple.github.io/foundationdb/testing.html
- https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/internals/vopr.md
- https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md
- https://antithesis.com/blog/deterministic_hypervisor/
- https://go.dev/blog/synctest
- https://go.dev/doc/go1.25
- https://tip.golang.org/doc/go1.26
- https://s2.dev/blog/dst
- https://notes.eatonphil.com/2024-08-20-deterministic-simulation-testing.html
- https://jbaker.io/2022/05/09/project-loom-for-distributed-systems/
- https://github.com/madsim-rs/madsim
- https://github.com/awslabs/shuttle

**LLMs and concurrency**
- https://arxiv.org/abs/2501.14326
- https://proceedings.mlr.press/v306/guo26z.html
- https://arxiv.org/abs/2603.03683

Search note: the web-search budget ran out midway, so later sources were fetched from known primary URLs. Three claims rest on secondary summaries: the Bend 2 interaction-net rationale, the BoC thesis statement, and the MPL PLDI 2023 abstract.
