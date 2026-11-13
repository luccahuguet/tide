# 11 — Working in tandem with Bend 2

Research date: 2026-10-19. Builds on report 01 (`research/01-bend-hvm.md`) without repeating it. Bracketed numbers refer to Sources. **Inference** marks my reasoning, not a sourced claim.

## Correction — 2026-11-12

The findings below remain a 2026-10-19 evidence snapshot. These updates
supersede its repository-home and current-cadence claims:

- **Repository homes.** Bend is now at [bendlang/bend](https://github.com/bendlang/bend);
  `HigherOrderCO/Bend` redirects there. HVM's former repository is now
  [HigherOrderCO/HVM1](https://github.com/HigherOrderCO/HVM1).
- **Release cadence.** The “37 releases in 26 days” statements describe the
  launch burst. Releases have since slowed to roughly one every one to two
  weeks: [2.0.37 on October 13](https://github.com/bendlang/bend/releases/tag/v2.0.37),
  [2.0.38 on October 20](https://github.com/bendlang/bend/releases/tag/v2.0.38),
  [2.0.39 on October 23](https://github.com/bendlang/bend/releases/tag/v2.0.39),
  and [2.0.40 on November 6](https://github.com/bendlang/bend/releases/tag/v2.0.40),
  the latest release as of this correction. These are GitHub publication dates;
  the changelog dates 2.0.40 November 5.
- **Native library target (#813).** The [issue](https://github.com/bendlang/bend/issues/813)
  is closed as `not_planned`, while [WONTFIX.txt at the checked revision](https://github.com/bendlang/bend/blob/f311527776559ce40095108ad6a7514a6c34aaa1/WONTFIX.txt#L152-L154)
  still says “planned, not scheduled”. Stage 1 has not become available.
  Stage 0 remains feasible with current Bend through a process boundary
  (standalone binaries with argument, file and output effects but no stdin,
  F5); neither stage is implemented in Tide, whose tandem remains in the
  Later tier (D16).

## Question

Ferox must not compete with Bend 2 (HigherOrderCO); it should work alongside it or be a plain Rust replacement. What is Bend 2's interop surface today, which division of labor is most valuable, what must Ferox's design include to make it smooth, where should the languages deliberately differ, and what are the risks?

**Bottom line.** Use model (a): Ferox is the application and systems shell, and Bend 2 supplies pure, total, law-checked kernels. Deliver it in two stages so Ferox never depends on Bend's roadmap: **Stage 0** (works today) is a process boundary with a generated wire format; **Stage 1** is in-process linking once Bend ships its native library target (#813, "planned, not scheduled" [4][13]). Add (f) shared agent tooling and a narrow (e): Ferox writes the law statement, Bend proves it, Ferox imports it as a labeled external claim. Option (b), Bend calling Ferox, comes almost free with C-ABI library export. Reject (c) and (d).

The main risk is that the valuable in-process boundary depends on an unscheduled feature from a very small team. That team shipped 37 releases in 26 days and states "There is no ABI promise" [3][5].

## Findings (cited)

### F1. Invocation and outputs

- **One CLI, written in TypeScript.** `bend PROOF.bend` checks a file. The `-o` suffix picks the output: a bare name gives a native binary, and `.c`, `.js`, `.cjs` or `.mjs` give C, JavaScript, CommonJS or an ES module [2][5].
- **How the native build works.** A native build runs `clang -std=c11 -O3 file -lpthread -lm` (plus X11/ALSA or CUDA/Metal flags) in a temporary directory, then deletes it. It reads only `CC`, `CUDA_HOME` and `PATH`, and accepts no `CFLAGS`, `LDFLAGS`, extra objects or libraries [8]. It needs clang 14+, or 19+ for `!` builds [1]. **Inference:** the only way to link extra native code is to emit `-o out.c` and compile it yourself.
- **Whole program only.** The maintainer reply on #813: "the C output is a whole program with a `main` and process-global runtime state" [13]. A native library target "for pure defs (a header, a lifecycle, chosen exports) is planned but not scheduled" [4][13]. A third-party porter concluded that "the proven core cannot be linked into a Rust application today" [14].
- **The only library-shaped output is JavaScript.** A loader makes a `.bend` file a Node or Bun module that exports "every filled, non-base, non-IO def", with constructors as `{$: "Name", field: value}` [6][8]. Its `Nat` encoding has already changed: `BigInt` at the commit cited in #813 [8], a plain number since 2.0.37, marked "Breaking for JS callers" [5].
- **Build cost.** There is no separate or incremental compilation: "One C file per program" [1]. One port measured 62 s for 1,000 lines, mostly "clang -O3 on one generated C file" [14]. A built binary starts in 2 ms [5].

### F2. Foreign effects: Bend's only FFI

- **Declaration.** The body of `def e(x: A) -> IO(B):` is `import "./e.c"` plus `import "./e.js"` [2]. All Base effects work this way: about 30 C/JS pairs in `bend2/effs/` [19].
- **C side** [3]:
  - Each effect is `Term name_run(Env e, Term* f, IoWork* w)`, registered from a constructor with `io_eff(CID(Name), run)`.
  - "The compiler splices your `.c` into the program's C source after the runtime, so every runtime symbol is in scope."
- **Marshalling.** A `U32` arrives as `(u32)f[0]`, and a `String` is read with `io_cstr`, a malloc'd copy the effect frees. Results are built with `(Term)n`, `io_str`, `io_node(e, CID(K), a, b)`, `io_done`/`io_fail` and `io_tup` [3][10]. The C encodings of `Nat`, `F32` and arrays are undocumented [3].
- **Threading.** Effects run only on the event loop: "proofs, termination and the GPU never touch host code" [2]. `io_work` runs blocking work on a helper thread that "may only touch `w`", and `io_wait_on` parks on a descriptor or deadline [3].
- **Resources.** Handles move or are borrowed and returned, and user-defined handle types are not supported (#825) [3][4].
- **No stability promise.** "The C side tracks the exact compiler version … There is no ABI promise. Rebuild your effects with every update" [3]. The helper API already changed in the first month (2.0.26, 2.0.28) [5].
- **Foreign dependencies are visible to tools.** "The verdict names the defs that rely on a foreign def" (2.0.18). The loaders "report unsafe and foreign dependencies" (2.0.25) [5].

### F3. Data representation

| Bend type | Representation | Notes for a boundary |
|---|---|---|
| Any term | One 64-bit word: "a tag, a 16-bit aux … and a 40-bit heap location". Small values sit inline [6] | Locations index Bend's own heap |
| `U32`, `F32` | Machine words, stored inline [6] | U64 and F64 are "SOON", and their PRs are refused (#1027, #1120) [4]. Array indexes wrap around [2] |
| `Nat` | Inline below 2^48; fail-stop above 2^48−1 (#779) [4][6] | Not a bignum in practice |
| `Array<T>` | `ALeaf \| ANode` with a single owner [7], stored flat: "packed 32-bit cells for an array of machine words" [6] | The slot count is a power of two [2]. An unbalanced tree or a block class above 31 fail-stops [4] |
| `String` | `SNil \| SCon{head: Char, tail}`, where `Char = Chr{code: U32}` [7] | "Strings are linked lists of characters, so text processing is slow" [1] |
| Bytes | `List<U32>` holding values 0..255. There is no `U8` and no bytes type [7] | Each byte is a list cell |
| ADTs | `is Data` types can be copied; `is Type` types are affine [2] | Base has `Maybe` and `Result{Fail, Done}` [7] |
| Closures | Affine: "can be called at most once" [2] | Not useful across a boundary |

All executors share "one flat array of words" as the heap [6]. **Inference:** Ferox pointers cannot enter Bend's heap. Every value that crosses must be copied into that heap. Flat `Array<U32>`/`Array<F32>` blocks are the only plausible zero-copy candidates.

### F4. Runtime ownership and failure

- **Threads.** The host pool has up to 128 threads and "opens at a program's first fork"; `--threads` and `--gpu` choose the executor at run time [2][6]. "The event loop runs on one thread" [6].
- **GPU.** A `!` build writes `file.gpu`, which must stay beside the binary [2]. `!` runs on the GPU only at the root of an IO step (#1065) [4], and "The CPU and the GPU never compute at the same time" [6].
- **Failure.** Out of memory is fatal (#792) [4], overflows are "numbered fail-stop[s]" [6], and the runtime "handles runtime errors with `_exit(1)`" [13].
- **Determinism.** No document guarantees deterministic results. Joins write disjoint slots, and every executor must print the same pinned checksum [6]. But F32 NaN bits and `F32.show` ties differ on the JS lane, and `bend file.bend` is lazy while compiled lanes are strict (#775, #797, #1079) [4].

### F5. What a standalone Bend binary can talk to

| Channel | Available | Details |
|---|---|---|
| Command-line arguments | Yes | `IO.args` returns `List<String>` [7] |
| Files | Yes | `File.read`/`File.write` use `String`; `File.read_bytes` returns `List<U32>` [7] |
| Network | Partial | TCP is IPv4-only [4]; UDP and channels also exist [2] |
| Child processes | Yes | `Process.run` "does not sandbox the child" [2] |
| Standard input | No | No stdin effect exists in `base.bend` [7], and one port reported "no stdin at all" [14] |
| Output | Yes | `IO.print`, files, and the exit code [6] |

### F6. Field evidence: the owner's own Rust boundary

The owner (GitHub user luccahuguet) opened issue #813 on 2026-09-18 [13]. It asked for a native library target whose pure defs a C or Rust host could call repeatedly, for example `step(state, event) -> decision`, with selected exports, a header, lifecycle and reentrancy rules, value ownership, recoverable errors and version-mismatch detection. A first maintainer reply, marked as AI-written, called it "out of scope for now"; Taelin then moved it to WONTFIX's SOON section and said the workaround "stays the way until then" [13].

The workaround in use is: "Bend owns the lifecycle and policy, a small C adapter handles value marshalling and event-loop parking, and a Rust static library owns the PTY, processes, sockets, terminal state, and cleanup" [13].

Akita reached matching conclusions: a syscall-heavy tool "would be a C program with a Bend `main`"; Bend suits "a small pure core with a spec that tests cover badly (allocators, schedulers, codecs, state machines, CRDT merges, game rules)"; and "Nothing about IO, JSON or rendering is proven" [14].

### F7. License, governance, cadence

- **License.** The compiler is Apache-2.0 with a patent grant, "Copyright 2026 HigherOrderCO" [12]. Hub packages are "public and permanent" and default to MIT-0 [18]. BendAI is "proprietary" and paid [17].
- **Release cadence.** 2.0.0 shipped on 17 Sep and 2.0.37 on 13 Oct: 37 releases in 26 days, with "Breaking" entries in 2.0.17, 2.0.32 and 2.0.37 [5]. The repository history was squashed and force-pushed during launch, then restored [15].
- **Team size.** Core size caps: "only Taelin changes them" [11]. "keeping the compiler at a size a small team can maintain" [13]. "We're a small team" [15].

## Division-of-labor options evaluated

| Option | Works today? | Value to Ferox users | Cost to Ferox | Competes with Bend? | Verdict |
|---|---|---|---|---|---|
| (a) Ferox shell, Bend kernels | Process boundary only; in-process blocked on #813 | **High** | Medium | No | **Adopt, in stages** |
| (b) Bend app calls Ferox for IO | Yes, through foreign effects (the owner's workaround) | Medium | Low if Ferox exports a C ABI | No | Support as a by-product |
| (c) Shared zero-copy ADT layer | No | Low | Very high | No | Reject |
| (d) Ferox emits Bend for a pure subset | Technically | Low | High | **Yes** (a second front end for Bend) | Reject for v1 |
| (e) Shared proof artifacts | Statement level only | Medium | Medium | Partly | Adopt narrowly |
| (f) Shared agent tooling | Yes | Medium–high | Low | No | Adopt |

**(a) Ferox shell, Bend kernels.** Bend does pure, total, law-checked code on CPU threads or a GPU from one source [1][6]. Ferox does everything Bend lacks or refuses: 64-bit numbers, stdin, TLS and HTTP, Windows, libraries, the Rust ecosystem [4][7][14]. Two workloads fit:

- **Proved policy cores.** Examples are the `step(state, event)` shape from #813 and Akita's list of state machines, schedulers and codecs [13][14]. Calls are fine-grained and latency-sensitive.
- **Uniform data-parallel kernels.** Bend's own benchmark set includes game of life, mandelbrot, n-body, raytracing, k-means and histograms [20]. Report 01 shows the GPU pays off only on uniform work. Calls are coarse and throughput-bound.

The two stages:

- **Stage 0 (today):** Ferox generates a Bend `main` shim, builds a kernel binary, and calls it as a subprocess (arguments or files) or as a long-running server on loopback TCP, using a generated text format, since Bend has no stdin and its bytes are `List<U32>` [7]. **Inference:** startup (2 ms [5]) and per-character list decoding are amortized over seconds of compute, so this fits coarse kernels, not tight loops. Crash isolation is a real advantage: a Bend `_exit(1)` cannot take down Ferox.
- **Stage 1 (after #813):** in-process calls through Bend's header, using the same Ferox declaration.

**Inverted embedding (the owner's workaround).** Bend owns `main` and Ferox ships as a static library behind foreign effects. This gives the (a) split in-process today, but Ferox gives up process lifetime and error handling, and the adapter targets internals that any release may rename [3]. Keep it as an expert escape hatch.

**(b) Bend calls Ferox.** This is the same mechanism without the policy split. It matters to Bend users who need IO that Base will never ship (#846 puts TLS and crypto in hub packages) [4]. Ferox already plans C-ABI library output (report 01 implication 8, report 09). A small generator that writes the `.c` adapter would make Ferox the natural IO-library language for Bend apps. **Inference:** that positions Ferox as Bend's systems partner rather than its rival, at little cost. Do not shape Ferox's core around it.

**(c) Shared ADT layer.** Rejected. Zero copy would tie Ferox's memory layout to Bend's 64-bit term word and 40-bit heap [6], which has no ABI promise [3], and Bend's strings and lists are slow linked cells [7]. One exception is worth requesting for #813: borrowed views of flat `Array<U32>`/`Array<F32>` blocks.

**(d) Ferox emits Bend.** Rejected. The target fragment is narrow (closures called once, no mutual recursion, no `if`, mandatory termination, 32-bit numbers [1][2][4]), the surface changes weekly [5], and it makes Ferox a competing front end to Bend's runtime, which breaks the owner's constraint. People who want Bend's GPU lane should write Bend.

**(e) Shared proofs.** Proof terms cannot move between the two systems. BendTT is an affine `Type : Type` theory (report 01), while Ferox's planned proof layer is auto-active SMT (report 04).

Statements can move for a first-order fragment over `Nat`, `Bool`, lists and ADTs, but not for U32 arithmetic, which is "opaque to proofs" [14]. The mechanically honest direction is Ferox → Bend: Ferox renders its `ensures` clause as a Bend `law`, an agent proves it in `PROOF.bend`, and Ferox imports the claim tagged with Bend's version and hashes. The reverse, a human asserting that a hand-written law means what a Ferox contract says, is weak.

**(f) Shared tooling.** Report 01 recommends copying Bend's gate shape: a human-owned laws file, agent-written proofs and a commit gate. If Ferox follows it, one agent workflow covers both languages: `ferox check` runs `bend PROOF.bend` and `--verdict` on Bend modules in the build and reports them as JSON beside Ferox's own diagnostics.

## Required Ferox design hooks

**H1. A `bend` dependency kind and build step.**

- `ferox.toml` pins an exact Bend version. Effects track "the exact compiler version" [3], and breaking changes ship within patch releases [5].
- The build calls `bend` and caches by content hash (sources, laws, proofs, Bend version), keeping 60-second whole-program clang builds [14] out of Ferox's edit loop.
- Dev profiles may run kernels via `bend file.bend` (JS lane, no clang) [3]; release profiles use native or `!` binaries plus `file.gpu` [2].
- Unsupported hosts (Windows, no clang 14+) get one clear diagnostic or a declared Ferox fallback implementation.

Illustrative syntax, consistent with report 07:

```ferox
#[bend(src = "kernels/life.bend", def = "Life.steps", version = "2.0.37")]
extern bend fn life_steps(board: BendArray<u32>, n: u32) -> BendArray<u32>
    with Accel
    ensures result.len() == board.len() by bend_law("kernels/LAWS.bend", "steps_keep_size");
```

The build generates a Bend shim that calls `Life.steps` at exactly the declared types. Bend's own checker then rejects any mismatch, so Ferox never has to parse Bend bodies. (**Inference**; untested.)

**H2. A closed type map, checked at build time.**

| Ferox | Bend 2 | Rule |
|---|---|---|
| `u32` | `U32` | Bend arithmetic wraps; Ferox's checked arithmetic stops at the boundary. Document it in the generated binding |
| `f32` | `F32` | NaN payloads are unspecified across the boundary [4]. Floats cannot be proved [1] |
| `u64`, `i64`, `f64`, `usize` | none | Compile error until #1027/#1120 ship [4] |
| `nat` (spec) or a checked `BendNat` | `Nat` | Range-checked below 2^48 before the call [4] |
| `bool`, `()`, `Option<T>`, `Result<T, E>` | `Bool`, `Unit`, `Maybe`, `Result` | Copied [7] |
| First-order `enum`/`struct` of the above | `type … is Data` | Copied, with the field order generated |
| `BendArray<u32 \| f32>` | `Array<U32 \| F32>` | Padded to a power of two, with the logical length passed [2] |
| `Vec<T>`, `String`, `[u8]` | `List<T>`, `String`, `List<U32>` | Copied; costs O(n) heap cells [7] |
| Closures, references, capabilities, handles | — | Never cross |

**H3. Ownership at the boundary.** Arguments are moved or explicitly copied. Bend arrays are single-owner and a read "hands the array back" [2], so a `BendArray` moved in and returned maps exactly onto Ferox move semantics. Borrowed Ferox data is always copied in; no Ferox reference ever points into Bend's heap.

**H4. Capability and effect rules.** The math is pure, but the call is not: Bend's runtime is process-global, starts up to 128 threads, may own the GPU, and can `_exit` the process [6][13]. So calls require an `Accel` capability that only `main`'s `Root` can create (report 07), with three rules:

- **One runtime per process, no reentrancy.** Calls are serialized until #813 documents otherwise.
- **A thread budget.** The capability carries a `--threads` budget, so Bend's pool and Ferox's work-stealing pool do not oversubscribe the cores.
- **Pure, total kernels only.** The build rejects kernels whose defs are IO-typed or depend on foreign or `@unsafe` defs, using Bend's own dependency report [5], unless the manifest grants an exception.

The *result* is a pure function of the inputs, so Ferox's deterministic simulation (report 07) need not record it.

**H5. Proof-claim import.** An `ensures … by bend_law(…)` clause is accepted only after three steps:

1. Ferox renders the clause as a Bend law (first-order fragment only).
2. `bend PROOF.bend` and `--verdict` pass.
3. The verdict lists no foreign or `@unsafe` dependencies [5].

The claim enters Ferox's checker as an axiom at a distinct trust level, "external: Bend", with provenance (Bend version, law and proof hashes) in the lockfile. It must never count as Ferox-proved: Bend's parser and translator are unverified, laws describe Bend's semantics while "The backends can still differ", and the compiler is "99% AI-written" [1].

**H6. Error and panic mapping.**

- Bend `Result`/`Maybe` map to Ferox `Result`/`Option`.
- In process mode, an exit code, a fail-stop number or stderr becomes `Err(BendFault { code, message })`.
- In-process, until #813 defines recoverable errors, every fail-stop kills the host [4][13]. So Ferox pre-validates known limits (Nat below 2^48, array sizes [4]) into `Err`, requires the manifest's `ffi` grant for in-process mode, and defaults to process isolation for unbounded inputs.

**H7. Deterministic parallel semantics.** Ferox's `par` should guarantee results identical to sequential evaluation for pure code, so a Ferox reference implementation and a Bend kernel are interchangeable as values. Differential tests (Ferox reference vs Bend C lane vs `!` lane) then catch backend divergence that Bend's laws cannot see [1][4].

**H8. The export side, for (b).** C-ABI static library output plus `ferox bend-effect`, which generates the `.c` adapter (`io_eff`, `io_work` and parking). Pin the adapter to the Bend version, since there is no ABI promise [3].

## Complementary positioning

| Dimension | Ferox | Bend 2 |
|---|---|---|
| Domain | Applications, services, CLIs, systems libraries, IO | Pure, total kernels: policy cores, uniform data-parallel compute [14][20] |
| Safety model | Ownership with second-class borrows, mutation allowed, capabilities (reports 02, 07) | Affine values, no shared mutation, IO only in the event loop [2][6] |
| Proofs | Optional auto-active SMT contracts on everyday imperative code (report 04) | Dependent-type laws with explicit proof terms on pure code [1] |
| Numbers | Full-width integers and floats, checked arithmetic | Nat/U32/F32 for now [1] |
| Parallelism | CPU `par` with work stealing; no GPU in v1 | Fork-join plus same-source GPU [6] |
| IO and platforms | Full standard library, Windows, Rust crates | Small Base, hub packages, no Windows [4] |
| Build | Separate, incremental compilation, fast check loop | Whole program, clang-only [1][4] |
| Audience | Rust users and agents shipping production systems | Agents writing small proved cores; GPU experimenters |

**Deliberate differences, so that the two complement rather than compete:**

- **Leave the GPU to Bend.** Ferox ships no GPU lane, and its docs send same-source GPU kernels to Bend.
- **Lead with the Rust story.** Do not headline "blocks AI mistakes via proof", Bend's slogan [16]. Ferox's pitch is Rust's guarantees, faster compiles and agent ergonomics, with proofs as an optional rung.
- **Do not build a dependent-type proof kernel.** Point deep pure proofs at Bend through H5.
- **Keep everything Bend chose not to have:** `if` and loops, mutable owned locals, `FnMut` closures, mutual recursion, local inference, Rust-like syntax. These are not competitive moves; they are what an IO shell needs.
- **Accept the unavoidable overlap.** CPU fork-join over owned data is shared by Bend's parallel let and Rust's `rayon::join` [15]; that is one of the "few similarities" allowed.

## Risks

1. **The library target may never ship.** #813 is "planned, not scheduled", and the first (AI-written) maintainer reply called it "out of scope for now" [4][13]. *Mitigation:* Stage 0 must stand on its own, and no Ferox feature may require Stage 1.
2. **Churn.** 37 releases in 26 days, several breaking, and "There is no ABI promise" [3][5]. *Mitigation:* exact version pins, a generated text wire format instead of runtime internals, and CI against each new Bend release.
3. **Bus factor.** One lead decides core policy [11], the team calls itself small [13][15], the compiler is largely AI-written and unaudited [1], and history was rewritten during launch [15]. *Mitigation:* keep Bend an optional dependency kind, removable without touching Ferox's core.
4. **Licensing.** Low for the compiler (Apache-2.0 with a patent grant [12]). The risk sits in future differentiators that are proprietary, such as BendAI and SupGen [17]. Hub defaults (MIT-0, "permanent" [18]) are permissive.
5. **Boundary performance.** Every value is copied, strings and bytes become linked cells, numbers are 32-bit, arrays are power-of-two sized, and the GPU runs only at IO-step roots [1][2][4][7]. **Inference:** the tandem pays off only when kernel compute dominates the copy, or when the proof is the point.
6. **Semantic mismatch.** Bend wraps U32 arithmetic, and silent U32 wraparound caused real bugs in one port (report 01). Lanes differ: lazy versus strict, and F32 NaN bits [4]. *Mitigation:* H2 and H7.
7. **Trust inflation.** Imported Bend claims bring in Bend's unverified parser and translator [1]. *Mitigation:* H5's separate trust label.
8. **Platform and build time.** No Windows, clang only, and 60-second release builds [4][14]. *Mitigation:* H1's cache and per-platform fallback.

## Open questions

1. If #813 ships, will it define reentrancy, more than one runtime per process, thread budgets, GPU ownership, and recoverable errors instead of `_exit`?
2. Can a user effect omit its `.js` twin for native-only builds? Bend's documentation shows both [2][3].
3. Will Bend expose machine-readable signatures and verdicts (JSON), so Ferox can generate bindings and import claims without scraping?
4. Will Bend add stdin or a file-to-`Array` read, which would make Stage 0 much cheaper?
5. Can laws about U32 ever be proved [14], and what changes when F32 becomes computed in the checker (#1017) [4]?
6. What per-event latency did the owner's adapter experiment measure? It decides whether policy cores can use Stage 0.
7. Would HOC adopt a shared, versioned boundary spec (a wire format, plus array views for #813) co-designed with Ferox?

## Sources

1. Bend 2 README — https://github.com/bendlang/bend (raw: https://raw.githubusercontent.com/bendlang/bend/main/README.md)
2. Bend 2 GUIDE — https://github.com/bendlang/bend/blob/main/guide/GUIDE.md
3. Bend EFFECTS guide — https://github.com/bendlang/bend/blob/main/guide/EFFECTS.md
4. Bend WONTFIX.txt — https://github.com/bendlang/bend/blob/main/WONTFIX.txt
5. Bend CHANGELOG (2.0.0–2.0.37) — https://github.com/bendlang/bend/blob/main/CHANGELOG.md
6. BendRT paper source — https://github.com/bendlang/bend/blob/main/bend2/docs/BendRT/main.typ (PDF: https://github.com/bendlang/bend/blob/main/paper/BendRT.pdf)
7. Bend `base.bend` — https://github.com/bendlang/bend/blob/main/bend2/base.bend
8. Bend `main.ts` (CLI, JS loader, clang invocation), commit 38ad338 — https://github.com/bendlang/bend/blob/38ad3382d8b611fa83bba769c15fa3612cce9fc7/bend2/main.ts
9. Bend `effs/tcp_recv.c` — https://github.com/bendlang/bend/blob/main/bend2/effs/tcp_recv.c
10. Bend `effs/args.c` — https://github.com/bendlang/bend/blob/main/bend2/effs/args.c
11. Bend AGENTS.md — https://github.com/bendlang/bend/blob/main/AGENTS.md
12. Bend LICENSE — https://github.com/bendlang/bend/blob/main/LICENSE
13. Issue #813, "Add a native library target for pure definitions" — https://github.com/bendlang/bend/issues/813
14. Akita, bend-tests findings — https://github.com/akitaonrails/bend-tests/blob/main/docs/findings.md
15. Hacker News, Bend 2 thread — https://news.ycombinator.com/item?id=49746163
16. Bend site — https://bend-lang.com
17. BendAI — https://bend-lang.com/bendai
18. Bend hub — https://hub.bend-lang.com
19. Bend effect sources directory — https://github.com/bendlang/bend/tree/main/bend2/effs
20. Bend runtime benchmarks directory — https://github.com/bendlang/bend/tree/main/bench/runtime
21. Internal: `research/01-bend-hvm.md`, `research/02-memory-models.md`, `research/04-verification.md`, `research/07-effects-capabilities.md`, `research/09-adoption-interop-backends.md`
