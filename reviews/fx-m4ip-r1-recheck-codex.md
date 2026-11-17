Model: gpt-6.1-sol (high) · Date: 2026-11-17 · Candidate d46dbff
Verdict: request_changes

Confidence: high. Re-checked only findings 1–9 and the additional edits in `git diff d23276c d46dbff`, against the updated builder report and affected sources. HEAD is `d46dbff3703ef963c5b0302f9cbb96460104a9e9`; the worktree is clean. The full `fcd7758...d46dbff` diff SHA-256 matches `0b11fd65ce368d842116c412befd38a8cf9c5892ad02d891ca4816c51e0ffc72`. This is a source-based re-check, not a new full review.

## Finding 1 — Compiler soundness and author intent

Fixed. Chapter 3 no longer asserts that accidental encounters are impossible or that agents cannot produce the patterns. It explicitly makes the type-based claim depend on compiler soundness regardless of intent, and distinguishes closed routes from other possible bugs. This matches README's qualification and the limited scope of the probes and `FX0206`. A separate sink-exception wording issue in this amended paragraph is recorded below.

## Finding 2 — Production scheduling versus simulator scheduling

Partially fixed; one blocking overclaim remains in the fix. Chapter 4 now correctly identifies production OS threads, arbitrary instruction preemption, modeled simulator points, and exclusive access to the particular cell during `with`. Its final explanation agrees with `design/03-authority-concurrency.md` §9.3: synchronization-point interleavings cover outcomes observable within the model, with model limits.

However, `docs/owner-guide/04-tasks-and-shared.md:43–44` adds: “The operating system can still pause any thread at any moment to run another; that's ordinary time-sharing, and it changes no result.” That last clause is not true of the concurrent program. In the chapter's own `demo/patches/lost-update.patch`, preempting a worker after its first `with` and before its second lets another worker read the same count and changes the final result. The source guarantees that the simulator models these interleavings, not that OS scheduling cannot change results. Isolated authority-free computation has a narrower determinism property; the sentence currently says “any thread” without that restriction.

Minimum fix: delete “and it changes no result”, or explicitly restrict it to the return value of isolated authority-free computation. Keep the later explanation that the simulator explores the relevant interleavings. This matters because the requested guide must explain why races exist and why simulation finds them, without implying that production scheduling is result-neutral.

## Finding 3 — Cancellation and deadlines

Fixed. The revised chapter describes errors returned at checkpoints for callers to propagate or handle, names non-checkpoints, and calls deadlines soft. Computation, clock reads, unlimited spawn, `Shared::with`, joins, and already-running uncancellable syscalls are no longer promised interruption. This agrees with TASK-3 through TASK-5, `Clock::now`, `Scope::spawn`, and `Shared::with`. The earlier blanket “every capability operation” and automatic-unwinding claims are gone.

## Finding 4 — Filesystem confinement versus invisibility

Fixed. Chapter 8 adds the readable context file, describes content/operation confinement, and explicitly permits existence/metadata observation subject to ordinary permissions. `sandbox.rs:237–249` grants the binary, context, private simulation area and exact loaded libraries; its syscall list admits the metadata calls. The distinction also agrees with the [Linux Landlock filesystem documentation](https://docs.kernel.org/userspace-api/landlock.html#filesystem-flags). The kernel-backstop wording is appropriately limited to driver-started test runs today.

## Finding 5 — Antithesis versus library simulators

Fixed. Answer 4 applies simulated-API discipline to library simulators, separately identifies Antithesis's whole-system hypervisor approach, and makes Tide's case the combination of authority signatures, local modeled tests and review artifacts. This agrees with chapter 5, Achilles W4, and [Antithesis's product description](https://antithesis.com/product/), which describes whole-system deployment and instruction-level deterministic execution. “No code discipline” is understood here in its immediate context: no requirement to replace every effect with a simulation-library API.

## Finding 6 — Meaning of unsafe

Fixed. Chapters 2 and 7 now explain permission for operations whose safety conditions the author must establish, rather than switching off compiler guarantees throughout a block. Chapter 2 explicitly retains ordinary type and borrow checks. This agrees with the grant boundary and the [Rust Book's unsafe explanation](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html#unsafe-superpowers).

## Finding 7 — Interpreting E3's threshold

Fixed. Chapter 9 now distinguishes missing the preset 1.5× threshold from finding no benefit, requests actual rates, false positives and uncertainty, and recognizes an inconclusive pilot. This matches D0 and fx-1vwf. The extra edits describing half the plants by mechanism rather than asserting actual invisibility preserve the draft protocol's intended split without predicting the eventual artifacts. The draft's six mechanisms and twelve pairs support that description; its in-review status remains explicit.

## Finding 8 — Conformance counts and awaited rules

Fixed. Chapter 2 records 369 PASS, 3 AWAITS CHECKER, 0 FAIL out of 372 cases, explains awaited codes and the runner's zero-exit behavior, and ties 342 workspace tests to the recorded landing run. Chapter 10 and answer 9 carry the same distinction. Independent read-only counting finds 372 case files; the existing fx-ip4x `conformance-c5.log` contains 369 PASS and 3 AWAITS CHECKER entries. The runner's verdict logic supports the explanation. No suite was rerun or fresh execution proof claimed.

## Finding 9 — Complete seccomp filter

Fixed. Chapter 8 now distinguishes unconditional allowances from argument-checked `clone`, `prlimit64`, `ioctl`, `fcntl` and `prctl`, and describes the `clone3` unsupported/fallback exception. This matches `sandbox.rs::filter`, including thread-only clone and ENOSYS for clone3. It no longer presents `ALLOWED` alone as the entire filter.

## New finding 10 — Carry the sink exception through the amended explanations

Non-blocking accuracy issue, confined to edited wording. Chapter 1 correctly adds the printing exception, but two related amended passages still overstate the restriction:

- `docs/owner-guide/03-soundness-bugs.md:156` says code cannot call the world's operations without a capability. Printing is an admitted counterexample even with a sound compiler. Add “other than printing”, as chapter 1 now does.
- `docs/owner-guide/04-tasks-and-shared.md:60–61` says a task holding a cell never waits for a lock or I/O. LOCK-4 expressly permits sinks inside `with` (printing is a lint), while SINK-5 says v0 writes synchronously under the destination's lock and can block for backpressure. The safe claim is that the callback cannot acquire another `Shared` or perform capability-mediated waits; it can still block on permitted output. Narrow this sentence accordingly. The cell-ordering deadlock explanation itself can stay.

These are source-backed wording corrections, not requests to change sink or locking behavior.

The other extra edits introduce no identified inaccuracies or improper weakening: chapter 1's printing, Root and compiler-soundness qualifications; chapter 5's same-code/input qualification and real-authority guard; chapter 8's test-only kernel backstop; and chapter 10's explicit planned status all fit their sources. `task.rs::outside_simulation` and `fs.rs::checkpoint` support the real-file-operation guard described in chapter 5.

Minimum patch for approval: remove or precisely scope chapter 4's result-neutrality claim under finding 2. Finding 10 is a small non-blocking consistency improvement. No implementation changes or new runtime proof are needed for these wording fixes; the coordinator's fresh Opus round remains separate.

Not checked: no builds, program runs, new probes, subagents or fresh-reader trial. No repository files were changed; only this requested report was written. Remaining risk: the owner could repeat the unqualified scheduling claim as a guarantee about concurrent programs.
DONE
