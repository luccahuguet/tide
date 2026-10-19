# Round 1 Review: Performance and Compile Speed

Reviewer persona: systems performance engineer (shipped Rust, C++, and Go
services, databases, and compilers).

Scope: `design/01-design.md` draft 1, read against
`design/00-first-principles.md`, `research/03`, `research/09`, `research/10`,
and the parts of `research/02`, `07`, and `08` that the design cites.

Question: can Ferox be as fast as Rust at run time and much faster to compile?

## Verdict

**request_changes**

The front end can be much faster than rustc. The backend and runtime choices
in D8 and D12 give up both promises.

The front-end decisions are right: compiled interfaces, required signatures,
body-local inference, no macros, and fixed impl placement. They make the 30 ms
edit-to-diagnostics target credible. The plan breaks after checking:

- D12 has no v1 backend that can produce a debug build anywhere near the D11
  target. Rust emission measured 53 s for a clean build and 17 s for a
  one-function edit on 100k lines.
- D11's witness-table generics ran 6–24x slower than monomorphized code even
  when optimized, and the plan's release half cannot run under rustc.
- D8's stackful tasks have no sound lowering through rustc and LLVM once a
  task with Rust frames on its stack migrates threads. They also cost a page
  of RSS and two kernel VMAs per task.
- D3's always-on overflow traps cost 3–28x in the integer kernels that "data
  systems" run.
- D6's reflection-based serialization is several times slower than serde in
  the closest existing implementation.

None of these needs a new thesis. Each has a small fix, listed below.

| Question | Answer |
| --- | --- |
| Release runtime within 10–20% of Rust | Plausible for typical service code once B5, B6, and N1 are fixed. Not as specified for integer kernels, serialization-heavy services, or servers with 100k+ connections. |
| 30 ms edit to diagnostics | Credible for type, borrow, and authority checks if the checker is a query engine from the first commit (N15). Not with proofs in the loop. |
| Clean check under 2 s | Credible: tsgo checks 104k lines in 0.7 s (research/03). rustc took 13 s on my 100k-line sample. |
| Clean debug build around 1 s | Not credible in v1. No backend in D12 can produce it (B1). |
| Release build time | Unstated. At least rustc's time on the emitted code: 137 s for my 100k-line sample. |
| crates.io interop for serde, tokio, axum | Not as claimed (B7). |
| Embedded, WASM, kernels | Undecided in the design. Recommendation in N12. |

## Evidence Base

Local measurements were taken on an Intel i5-13450HX (16 threads) with 31 GB
RAM, Linux 7.1, and rustc 1.96.0. The release profile used `opt-level=3` and
`codegen-units=1` unless stated. The machine carried other load, so
microbenchmark ratios varied between runs; I give the range. These are
microbenchmarks and a synthetic crate. They do not replace the benchmark suite
the design needs (N14), but they are enough to test specific claims.

**M1. Rust emission compile times.** I generated a 100,457-line crate (91,340
non-blank lines) shaped like one emitted Ferox package. It has 172 modules of
structs, enums, `match`, `?`, iterator chains with closures, `format!`,
`HashMap`, and a few generic instantiations, with no dependencies and with
`overflow-checks = true` to model D3.

| Operation | Time |
| --- | --- |
| `cargo check`, clean | 13.1 s |
| Debug build, clean | 53.2 s wall, 58.7 s CPU, 3.8 GB peak RSS |
| Debug rebuild after editing one function body | 17.2 s |
| Release build, clean | 136.9 s wall, 340.6 s CPU |

**M2. Overflow traps (D3).** Trapping arithmetic compared with wrapping
arithmetic, release build:

| Kernel | 16K elements (cache-resident) | 1M elements |
| --- | --- | --- |
| `i32` sum | 5.3x | 3.3x |
| `i32` dot product | 2.9x | 1.6x |
| `i32` `3*a[i] + b[i]` | 3.0–4.5x | 1.6x |
| `u8` `a[i] + k` | 10.8–27.9x | 6.9x |

- The `i32` sum used an explicit `checked_add`. rustc's own
  `-C overflow-checks=on` on a plain `+` loop gave the same 5.3x.
- With a deferred trap (overflow flags OR-ed together, one trap at loop exit),
  the `u8` kernel ran at 0.8–1.4x of wrapping. The `i32` multiply-add did not
  recover (3.6–4.9x): LLVM did not vectorize overflow-checked `i32` multiply
  on AVX2.
- Code size: on a 17.5k-line slice of M1's crate, `.text` grew from 1,312,364
  to 1,554,338 bytes (+18.4%) with overflow checks on.
- Whole-program costs are small. Dan Luu measured about 1% for trap-only
  checking on bzip2 ([Luu](https://danluu.com/integer-overflow/)). The cost
  sits in loops that would otherwise vectorize.

**M3. Witness tables compared with monomorphization (D11).** `u64`
elements:

| Kernel | Slowdown |
| --- | --- |
| Release: simulated witness-table sum (runtime size, indirect copy and add) | 23.9x at 16K elements, 5.6x at 1M |
| Release: iterator pipeline with dynamically dispatched `next` and closures | 8.0x at 16K, 3.3x at 1M |
| Debug (`opt-level=0`) against release: monomorphized sum | 106x |
| Debug against release: monomorphized iterator pipeline | 20x |
| Debug: witness tables on top of debug | another 1.6–2.1x |

**M4. Stackful task stacks (D8).** I mapped each stack with `mmap` (256 KiB
or 1 MiB, `MAP_NORESERVE`), added an `mprotect` guard page, and touched one
page, for 10k and 100k tasks.

- Each task used exactly 2.00 VMAs and 4.0 KiB of RSS.
- Without pooling, creating a stack took 3.7–4.1 µs and destroying one took
  2.6–3.1 µs.
- The kernel's default `vm.max_map_count` is 65530
  ([kernel docs](https://docs.kernel.org/admin-guide/sysctl/vm.html)), so a
  default host stops near 32k concurrent tasks per process. This machine's
  distribution raises the limit.

**M5. Lowering D2 to Rust.** Stable rustc 1.96 rejects three patterns that D2
accepts or implies:

| Pattern | rustc error |
| --- | --- |
| Polonius problem case #3: return a `get_mut` borrow conditionally, otherwise insert and look up again | E0499 |
| A method borrowing `self.buf` mutably while another borrows `self.stats` (D2's declared field borrows) | E0502 |
| The literal one-region view lowering, `struct Lexer<'a> { src: &'a str, out: &'a mut Vec<&'a str> }`, then reading `out` after the lexer is dropped | E0502 |

## Blocking Objections

### B1. D12 (with D11): no v1 backend meets the build targets, and dev code runs 20–100x slower than release

**Problem.** D11 promises a clean debug build of about 1 s plus link for 100k
lines. D12 gives v1 two backends: a reference interpreter and Rust emission.
The native development backend comes "later, once the interpreter stops being
fast enough."

- The interpreter is not a build. It removes compile time by making every
  test run slower than unoptimized native code: unit tests, property tests
  (D10's `test` mode), fuzzing, and simulation seeds (D8; research/10 promises
  "run 10,000 seeds").
- Rust emission is a build, but it runs the compiler whose speed Ferox exists
  to beat.

**Evidence.**

- M1: 53 s for a clean debug build, 17 s after a one-function edit, and 13 s
  just to type-check the emitted code. That is 50x over D11's 1 s build and
  170x over research/03's 100 ms edit-to-runnable target, before Ferox's own
  front end runs.
- The debug build used 58.7 CPU-seconds in 53 wall-seconds. One Ferox package
  becomes one Rust crate and goes through rustc's mostly serial front end, so
  more cores do not help.
- M3: even native unoptimized code is 20–106x slower than release on iterator
  and numeric code.

research/03 sets a dev-runtime target of "within ~1.5–3x of release" and
misreads its own evidence:

- TPDE is within ±9% of LLVM `-O0`, not of `-O2`.
- The Liftoff and Cranelift figures come from Wasm, which LLVM has already
  optimized and inlined before the Wasm compiler sees it.

Iterators, closures, `Option`, and `?` cost nothing only after inlining.

**Scenario.** An agent fixing a bug in a 100k-line storage engine edits one
function, then runs 200 property tests and 1,000 simulation seeds that take
4 s in a release build.

- Interpreter path: the build is instant, and the test phase takes minutes.
- Rust path: 17 s to rebuild, then tests that run 20x slower than release.

The 30 ms check does not help this loop.

**Smallest fix.**

- Move the native dev backend into v1 and hold it to the D11 targets: Ferox
  mid-IR to Cranelift (a Rust library that embeds in the compiler), a
  per-function object cache, and function-level relinking.
- Inline small std generics and closures in the mid-IR before dev codegen, so
  iterator code is not 20x slower.
- Keep the interpreter as the semantic oracle, not as the dev runtime.
- Add three targets to D11:
  - edit to first test result;
  - dev runtime within 3–5x of release on the benchmark suite;
  - release build within 1.2x of rustc's time on the emitted code, measured.
- Drop "Go's build speed" from the pitch as it applies to release builds.

### B2. D11 (with D4 and D12): the generics plan is slow in dev and cannot be implemented in release

**Problem in dev.** Witness tables pass size, alignment, copy, move, and drop
at run time. Each generic `Vec<T>` index multiplies by a runtime size, and
each local of type `T` needs a dynamically sized slot. Each trait call and
closure call is indirect and blocks inlining. This is Swift's unspecialized
path, which research/03 and research/08 describe as a performance cliff.

**Problem in release.** D11 says release builds "monomorphize through a
content-addressed instantiation cache across the whole package graph." D12
says release codegen is rustc and "every Ferox package is a Rust crate." The
two cannot both hold.

- If Ferox emits generic Rust, rustc instantiates generics per crate. At
  opt-level 2 and 3 it does not reuse upstream instances: `share_generics()`
  returns `false` for `OptLevel::More | OptLevel::Aggressive` unless an
  unstable flag overrides it
  ([rustc_session](https://doc.rust-lang.org/nightly/nightly-rustc/src/rustc_session/config.rs.html)).
  The cross-package cache has nothing to do.
- If Ferox emits pre-monomorphized Rust, the output is not the readable Rust
  that the "eject" promise needs, and exported crates cannot offer generic
  APIs to Rust callers.

**Evidence.**

- M3: witness tables ran 5.6–23.9x slower than monomorphized code in
  optimized builds, and 8.0x slower for an iterator pipeline.
- Zig rebuilds 464k lines in 35 ms with full monomorphization (research/03,
  section 1). Function-level incrementality buys that latency; witness tables
  do not.
- research/08 rates the cost of a second codegen path as "High."

**Smallest fix.**

- Monomorphize in every profile, through the instantiation cache D11 already
  describes, keyed by definition hash and type arguments.
- Forbid polymorphic recursion, a local and decidable check, so instantiation
  sets stay finite.
- Keep the IR able to fall back to dictionary passing when one definition
  exceeds an instantiation budget. Keep D4's parametricity rules, which make
  that fallback safe.
- State the release split: Ferox monomorphizes the internal program and emits
  monomorphic Rust for it. Only `pub` generic items at a crate-export
  boundary are emitted as generic Rust.

### B3. D8 (with D12): stackful tasks have no sound lowering through rustc and LLVM, and they miss the server memory targets

The cost of a context switch is not the issue: a stackful switch saves a few
registers in tens of nanoseconds. Soundness, memory, and stack sizing are.

**Problem: soundness.** D8 puts stackful tasks on work-stealing schedulers, so
a suspended task can resume on another OS thread. Under Rust emission that
stack holds rustc-compiled frames: Ferox code, the Ferox runtime crate, Rust
std, and any crates.io code that called back into Ferox.

- LLVM assumes that "the thread-id cannot change within a function with the
  exception of suspend points of pre-split coroutines"
  ([LLVM PR #88418](https://github.com/llvm/llvm-project/pull/88418)). Pre-split
  coroutines are stackless, C++-style coroutines. LLVM may therefore keep a
  thread-local address or a `pthread_self` result in a register across the
  call that switches stacks.
- Rust's `Send` rules exist because some values must stay on one thread: on
  some platforms a `MutexGuard` must be unlocked by the thread that locked it.
  Migrating a stack moves every value on it, beneath the type system.
- Concrete case: a crates.io cache calls a user closure while holding its
  internal lock. The Ferox closure does I/O and suspends. The task resumes on
  another thread, still holding the lock. This reintroduces the "lock held
  across a suspension" hazard that D8's `lock` design removes.
- The `may` crate's README warns that "access TLS in coroutine might trigger
  undefined behavior" ([may](https://github.com/Xudong-Huang/may)).
- tokio finds its runtime through a thread-local context, so tokio types fail
  with "no reactor running" on a Ferox worker thread that has not entered a
  tokio runtime.

D7's "no thread-locals in safe code" covers Ferox source only. It does not
cover std, the allocator, or crates.io code on the same stack.

**Problem: memory.**

- M4: at least one page of RSS and two VMAs per task, and 3.7–4.1 µs to
  create an unpooled stack.
- tokio tasks need "a single allocation and 64 bytes of memory" plus the
  future ([tokio tutorial](https://tokio.rs/tokio/tutorial/spawning)). Go
  starts goroutines at 2 KiB (`stackMin = 2048` in
  [runtime/stack.go](https://github.com/golang/go/blob/master/src/runtime/stack.go)).
- On a default kernel (`vm.max_map_count` = 65530), a process stops near 32k
  concurrent tasks. The sysctl applies to the whole host and cannot be raised
  from inside a container. Elasticsearch documents a minimum of 1,048,576 as a
  deployment prerequisite
  ([Elastic](https://www.elastic.co/guide/en/elasticsearch/reference/current/vm-max-map-count.html)).
- On arm64 with 16 KiB pages (Apple Silicon), the RSS floor is 16 KiB per
  task, or 16 GiB of stack pages for 1M connections.

**Problem: stack size.** research/10 proposes exact stack sizes for tasks
whose call graph has no recursion and no `dyn`.

- Under Rust emission, rustc and LLVM decide frame sizes after inlining.
- Every I/O call goes through a capability (D7).

Ferox therefore cannot compute the bound, and every task must reserve for the
worst case. Every call into C or crates.io must then either declare a stack
budget or switch stacks:

- No agent can know the stack budget of, say, `regex::Regex::new`.
- Switching stacks is expensive. cgo pays about 40–50 ns per call
  ([Go issue 42469](https://go.dev/issue/42469)), against about 2 ns for a
  plain call
  ([Cockroach Labs](https://www.cockroachlabs.com/blog/the-cost-and-complexity-of-cgo/)).
- .NET's green-thread prototype took 100M P/Invokes from about 300 ms to about
  1,800 ms (research/10).

**Smallest fix.** In the Rust backend (D12), lower suspending functions to
stackless state machines: emitted `async fn` with compiler-inserted `.await`
at capability calls.

D8's rule that only capability operations suspend already tells the compiler
where every suspension point is. That is the information Rust's `async`
keyword exists to give rustc. A higher-order function gets a suspending
instantiation when its closure argument captures a capability, which
monomorphization (B2) provides. The result is:

- tasks measured in tens of bytes;
- no thread-local hazard;
- tokio compatibility (B7);
- WASM support (N12).

Accept and document the costs:

- boxed futures for suspending calls through `dyn` (or devirtualize, N1);
- boxed recursion of suspending functions;
- `unsafe` in emitted code for scoped child borrows. This is sound because
  Ferox, unlike Rust, never forgets a scope.

Keep stackful execution for the interpreter and the simulator.

If the project keeps stackful release tasks instead, the minimum is:

- no migration while any non-Ferox frame is live;
- a published per-task memory and spawn benchmark at 100k and 1M tasks on a
  default kernel, before D8 is frozen;
- the VMA requirement in the deployment documentation.

### B4. D2 (with D12): the borrow model does not lower to Rust that rustc accepts

**Problem.** D12 says rustc is "a second check, never the definition" and that
the emitted Rust is readable. D2 specifies:

- a checker at "the precision of Polonius";
- methods that declare the fields they borrow;
- views with "one implicit region per type."

Stable rustc uses NLL and has no view types.

**Evidence.** M5: rustc rejects all three patterns. Each one forces the
emitter into one of three moves:

1. Restructure the code: turn field-borrowing methods into associated
   functions that take one reference per field.
2. Invent lifetimes: one per reference field.
3. Emit raw pointers inside `unsafe`. This removes rustc's second check from
   exactly the functions where a second check is worth having.

The one-region rule also affects Ferox itself. A view holding a `&mut
Vec<&str>` together with the `&str` it fills from is Rust's `&'a mut T<'a>`
trap. Either Ferox rejects a common pattern with an error about a region the
user cannot see, or Ferox's checker does not use the rule D2 states.

**Smallest fix.**

- For v1, set the checker's precision to NLL plus declared field borrows, not
  Polonius, so everything Ferox accepts has a safe Rust lowering.
- Specify the lowering in D12:
  - field-borrowing methods become associated functions with split borrows;
  - each reference field of a view gets its own inferred lifetime parameter,
    invisible in Ferox source.
- Track the share of emitted lines inside `unsafe` in CI, with research/09's
  10% kill criterion.
- Revisit Polonius when rustc ships it.

### B5. D3 (with D10): overflow traps in every profile conflict with "release builds compete with Rust and C"

**Problem.**

- Always-on traps cost little in typical code and a lot in integer loops that
  would otherwise vectorize. Codecs, compressors, hash functions, checksums,
  columnar scans, and image processing spend their time in such loops.
- The escape hatch, `.wrapping_add()`, changes the meaning of the code. A
  performance engineer who wants "this cannot overflow, and it is fast" gets
  "silently wrong if it does overflow."
- D10 then forbids the one tool that could remove checks soundly: "Specs never
  affect typing or code generation."

**Evidence.**

- M2: 3–28x slower on cache-resident kernels and 1.6–6.9x when memory-bound.
  rustc's own `-C overflow-checks=on` reproduces the 5.3x on a plain `i32`
  sum. `.text` grows 18% on business-logic code.
- Dan Luu measured the same effect: about 6x on a summing loop, mostly from
  lost vectorization.

"Within 10–20% of Rust" can hold on a service-wide average while an LZ4 or
CRC kernel runs 5x slower. Bounds checks match Rust's, and the rule below
removes them too.

**Smallest fix: two rules.**

1. **Imprecise traps.** Inside one loop or one pure call, an overflow may be
   reported at any point before the next capability operation or the region's
   exit. Memory written through `&mut` by the abandoned task is unspecified but
   valid. No I/O can happen after an overflow that should have trapped. In my
   runs this brought the `u8` add kernel to 0.8–1.4x of wrapping. It did not
   recover `i32` multiply-add.
2. **Proven facts may delete checks.** When a package is in `prove` mode and
   the solver proves that an arithmetic or index operation stays in range,
   codegen omits the check. Deleting a check that provably never fires cannot
   be observed, so dev and release behave the same. This makes D10 a
   performance feature that Rust lacks.

Multiply-heavy kernels still need explicit `wrapping_*` or widening. The
reference should say so.

### B6. D6: `Shape`-driven serialization is several times slower than serde, and constructing values through `Shape` is a soundness hole

**Problem: speed.** D6 makes serialization "ordinary generic library code"
over `Shape`. JSON and protobuf encoding and decoding are often among a
service's largest CPU consumers.

- serde is fast because derive emits monomorphic code for each type, which
  LLVM inlines end to end.
- Generic code that walks `T::SHAPE.fields` needs to dispatch on each field's
  type, through function pointers in the field table.
- LLVM devirtualizes those calls only if it fully unrolls the loop over a
  constant table. Ferox cannot ask for that: it has no `inline for`, and
  `const fn` "computes values, never declarations or types."

**Evidence.**

- research/08 cites facet as precedent. facet-json was initially "between
  five to seven times slower than serde_json"
  ([fasterthanli.me 2025 recap](https://fasterthanli.me/articles/2025-recap)).
- The maintainers' benchmark runs show deserialization 1.2–10x slower
  ([CodSpeed](https://codspeed.io/facet-rs/facet/branches/perf-tests)).
- facet reached parity only by adding a JIT ("you can now beat serde while
  staying in the facet ecosystem, at least for JSON"). A Ferox release binary
  has no JIT.
- Deser, which uses dynamic dispatch, is about 10% slower on JSON reads, with
  binary size "quite a bit worse" (research/08).

**Problem: soundness.** `Shape` is on by default and "opt-out for types with
invariants," and a reflective deserializer can build any type field by field.
One forgotten opt-out is enough: if `unsafe` code relies on an invariant of
that type (a buffer with `len <= cap`, a sorted index, validated UTF-8), safe
code can now break memory safety.

**Smallest fix.**

- Add `Encode` and `Decode` to D6's closed list of compiler-implemented
  derives. They generate monomorphic code against a serde-like data-model
  trait that format libraries implement. Roc treats encoding and decoding the
  same way, as built-in abilities derived for records and tags
  ([Roc](https://roc-lang.org/examples/EncodeDecode/README.html)).
- Keep `Shape` for printing, diffing, and tooling. Dev builds may use the
  `Shape`-interpreted path to save compile time.
- Make `Shape` read-only by default. Constructing a value through `Shape`
  should require every field to be visible to the caller, or an explicit
  opt-in.

### B7. D12 and Positioning: crates.io interop does not reach the APIs the target market uses

**Problem.** Positioning says Ferox packages "call Rust crates" and that teams
adopt Ferox one module at a time. The target market is services and data
systems. Their crates.io dependencies are:

- async: tokio, hyper, axum, tonic, reqwest, sqlx;
- lifetime-generic: serde's `Deserialize<'de>` and `Visitor<'de>`,
  `Cow<'a, str>`, borrowed iterators;
- macro-driven: `#[derive(Serialize)]`, `#[tokio::main]`, axum extractors.

research/09 defers "async, macros/derive, and lifetime-heavy APIs" to
hand-written shims. D12 leaves that limitation out.

**Scenarios.**

- An axum handler is an `async fn` whose future must be `Send`, and a stackful
  Ferox task is not a future. Bridging the two needs a second scheduler, a
  second I/O reactor (tokio's epoll beside Ferox's io_uring), and a wake
  across schedulers on every request.
- Ferox has no lifetime parameters, so it cannot name `Deserialize<'de>`,
  implement `Visitor<'de>`, or bound a generic by either. A Ferox type
  exported to Rust has no serde impls, so Rust code in the same workspace
  cannot serialize it.
- tokio types fail on Ferox worker threads (B3).

**Smallest fix.**

- Narrow the claim in Positioning and D12 to what explicit-instantiation glue
  supports: synchronous APIs whose types are owned or carry a single
  lifetime.
- Define the import mapping:
  - a Rust type with exactly one lifetime parameter imports as a Ferox view;
  - a `for<'de> Deserialize<'de>` bound imports as owned-only deserialization
    (`DeserializeOwned`);
  - anything else needs a shim.
- Have the Rust emitter generate serde impls from `Encode` and `Decode` (B6)
  for exported types.
- Make interop with the tokio ecosystem depend on the stackless lowering in
  B3, under which Ferox's evented runtime can be a tokio runtime.

## Non-Blocking Issues

**N1. D7: "guaranteed devirtualization" cannot hold as designed.** research/07
promises static dispatch when only one implementation exists. That condition
fails in practice:

- attenuation (`root.net.only(hosts)`, `fs.dir(...)`) adds implementations to
  every production binary;
- separate compilation (D11) hides the set of implementations;
- rustc does not devirtualize trait objects without whole-program
  information.

For I/O this does not matter: an indirect call is small beside a syscall of
100 ns or more. It does matter for:

- `Rand`: a generator step of about 1 ns becomes a call that cannot be inlined
  and blocks vectorization of shuffles and Monte Carlo loops;
- per-byte buffered reads.

Fix:

- the binary picks each capability's implementation at build time, as Rust
  does with `#[global_allocator]`, so capability types are concrete;
- attenuation is a policy field checked at the syscall boundary, not a new
  type;
- `Rand` hands out a seeded value-type generator, so randomness authority is
  used at seeding and generation is pure.

**N2. D7 and D8: `Pure` admits shared mutable cells.** A function that takes
`&Cown<T>` has no capability in its signature, so D7 calls it pure,
deterministic, and memoizable. Yet it can `lock` and observe interleavings,
and a contended acquisition suspends, which contradicts D8's "a pure function
never suspends." Fix: `Pure` means no capability and no reachable shared
interior mutability. This is the `Freeze`-like condition D9 already uses for
`par`.

**N3. D8: `lock` has only an exclusive mode, and safe code has no atomics or
per-thread state.** The consequences:

- read-mostly state (routing tables, config, caches) serializes every reader;
- metrics counters contend on one cache line;
- per-core sharding, the standard scaling technique for counters, pools, and
  allocators, is impossible without thread-locals.

Fix:

- add a shared mode, `lock (read a)`;
- add safe atomic counters with a small fixed set of memory orderings;
- add a std `Sharded<T: Merge>` per-core value;
- add a snapshot cell (in the style of arc-swap or RCU) for read-mostly data.

Keep the rules against nesting and suspension inside `lock`. They are good.

**N4. D9: heartbeat scheduling is not available on the D12 backend.** The TPAL
paper ([PLDI 2021](https://www.andrew.cmu.edu/user/mrainey/tpal/tpal.html))
describes heartbeat scheduling as lacking a practical implementation. Its own
implementation needed compiler transformations, a specialized runtime,
interrupt delivery, and "promotion-ready program points." With rustc and LLVM
as the release backend, Ferox cannot place those points in optimized loops,
and polling a flag on every iteration blocks vectorization. Fix: specify
Rayon-style adaptive splitting for v1, and drop "cheap enough to use freely"
until a benchmark supports it.

**N5. D9: fixed-tree reductions need two clarifications.**

- A fixed tree is not the sequential left fold. "Results equal sequential
  execution" is true only if `par_reduce` is defined as the tree in every
  profile, the interpreter included.
- The automatic parallelization pass must never rewrite a sequential float
  fold or a trapping integer fold, because changing the association changes
  whether a trap happens.

For SIMD leaves, bitwise determinism across targets requires a fixed lane
count (for example, 8 accumulators) on AVX-512, AVX2, and NEON alike. That is
acceptable, but it should be documented as the cost.

**N6. D2: eliding `Rc` counts can move drop timing.** If removing a clone/drop
pair changes when the last reference dies, a destructor that closes a file or
flushes a buffer runs at a different time. research/00 (K25) forbids that.
Restrict elision to pairs where the last drop provably does not move. The gain
is small anyway, because clones in Ferox are explicit.

**N7. D2: generational arenas match Rust's slotmap idiom, not `&'arena T`.**
Vale measured 10.84% for generation checks on its terrain benchmark
(research/02). Keep handles at 8 bytes (a `u32` index and a `u32` generation).
Document D2's arena-scoped references as the fast path for traversal-heavy
code such as compilers and graph engines.

**N8. D4: without specialization, Ferox's own std cannot express Rust std's
internal fast paths.** Examples: `TrustedLen` extend, `Copy` clone as
`memcpy`, `zip` over `TrustedRandomAccess`, in-place `collect`, and `io::copy`
to `copy_file_range` or `sendfile`.

- Release builds that lower to Rust's std inherit these.
- The interpreter and the Cranelift backend in B1 do not.

Fix: allow sealed, always-applicable specialization in std only, or const
layout queries (`needs_drop`, `is_bitwise_copy`) carried in the instantiation
data. Either way, apply the differential-testing obligation that D4 already
implies.

**N9. D7 and D8: `HashMap` seeding forces a choice between determinism and
security.**

- Rust's `RandomState` reads OS randomness ambiently. D7 forbids that, and the
  random iteration order breaks "pure functions are deterministic."
- A fixed seed lets an attacker who knows it precompute collisions, which
  exposes every Ferox service to HashDoS.

Fix: deterministic iteration order (insertion order, or hash order under a
fixed seed) for pure code, plus a keyed map whose key comes from `Rand` for
maps fed by untrusted input.

**N10. D8: io_uring needs owned buffers, and D8 does not say so.** research/10
recommends I/O that passes ownership. D8 only says that operations which
consume a value return it inside their error. If `read(&mut buf)` lends a
stack buffer to the kernel, cancellation must wait for the kernel's completion
before unwinding past that frame. Fix: the evented runtime's read and write
APIs take owned buffers.

**N11. D12: debug info, profiling, and binary size go through emitted Rust.**

- Rust has no `#line`. Pre-RFCs from 2017 to 2024 did not land
  ([2017](https://internals.rust-lang.org/t/pre-rfc-supporting-line-directives-either-directly-or-with-an-extension/4765),
  [2024](https://internals.rust-lang.org/t/pre-rfc-line-attribute/21069)).
  Backtraces, `perf`, and debuggers will show generated `.rs` paths and the
  Rust-mangled names of generated items.
- Fix for debugging: emit one Rust function per Ferox function with stable
  names, write a span sidecar file, ship `ferox symbolize`, and budget for an
  upstream path-remapping or `#[line]` contribution.
- Budget LTO for release. Without it, the Ferox runtime crate's small
  functions do not inline across the crate boundary.
- Binary size: overflow traps add about 18% `.text` (M2) plus a panic-location
  static at each trap site. Report binary size beside runtime in N14.

**N12. Embedded, WASM, and kernels: decide now.** The design is silent, and as
written it excludes them:

- D7 needs a `Root` from an operating system;
- D8 needs a runtime with MMU guard pages;
- D5 recovers from bugs by unwinding at task boundaries;
- allocation is ambient.

Recommendation:

- v1 targets hosted 64-bit Linux, macOS, and Windows.
- Add `wasm32-wasip2` if B3's stackless lowering is adopted. Rust emission
  provides that target for free; stackful tasks do not, because WASM stack
  switching is not yet a portable baseline across engines, to my knowledge.
- Declare no_std, embedded, and kernels out of v1, but keep three doors open:
  - fallible allocation APIs;
  - a core library with no capabilities;
  - an abort-on-trap profile.

**N13. The documents disagree on targets.** 00-first-principles says "a few
seconds" for a 100k-line debug build; D11 says "around 1 s plus link." Pick
one. Then add release-build time, edit-to-test latency, and the dev-runtime
ratio (B1).

**N14. No benchmark set backs the performance claim.** "Within 10–20% of
Rust" needs a named benchmark set before D3, D6, and D8 are frozen. It should
include:

- a compression or codec kernel;
- a JSON service;
- a KV store with 100k connections;
- a parser;
- the Benchmarks Game.

Report Ferox against Rust on release runtime, build time, RSS, and binary
size, and apply research/09's kill criteria in CI.

**N15. Build the checker as the incremental query engine from the first
commit.** D12 makes "checker plus reference interpreter" the first
deliverable. A batch checker built first would later need the query
architecture retrofitted, and research/03 documents how that went for rustc.
The 30 ms target depends on this.

## Strengths Worth Keeping

- **The front-end architecture.** Compiled interfaces that only direct
  importers read, required item signatures, body-local inference, no
  overloading, no macros, impls only in the type's or trait's module, and
  acyclic imports. research/03 identifies these decisions as setting the
  ceiling on compile speed. rustc took 13 s to check my 100k-line sample.
  Nothing in Ferox's front-end rules stops its checker from being 10–50x
  faster, as tsgo and ty are.
- **Parametric generics (D4).** No specialization in user code, no type
  inspection, and no overlapping impls. These let the compiler choose
  monomorphization or dictionary passing per instantiation without changing
  behavior, including the budgeted fallback in B2.
- **Capabilities as values (D7).** They make purity checkable and allow
  simulation without library cooperation. They also tell the compiler where
  every suspension point is, which makes B3's stackless lowering possible
  without an `async` keyword.
- **Structured tasks with level-triggered cancellation and no cancellation by
  drop (D8).** This removes tokio's cancel-safety class of bugs. `lock` with
  no suspension and no nesting inside it is the right latency discipline.
- **Explicit conversions (D3).** No `as`; `widen`, `narrow`, `truncate`, and
  explicit wrapping and saturating methods instead.
- **Visible costs.** `dyn` is always spelled out, and `Option` uses niche
  layout.
- **The interpreter as a semantic oracle,** with differential testing against
  every backend.
- **Rust emission as the release and export backend.** It is the right
  adoption bet once it is scoped honestly (B3, B4, B7) and paired with a
  native dev backend (B1).
