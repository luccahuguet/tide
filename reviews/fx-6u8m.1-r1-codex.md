Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 37ee129

Verdict: request_changes

What this change does: Adds four Tide/plain-Rust workload pairs and a workspace integration test comparing their successful output at default and explicit sizes. The tasks workload also runs under eight simulator seeds and compares its simulated output with the Rust arm.

Reviewed the frozen candidate 37ee12930dbde21cd05d733be36a915802693b8e against main, AGENTS.md, GOVERNANCE.md, both beads and their comments, the builder report, the shared helpers and their consumers, and the relevant production std implementations. Diff SHA-256: 525d9f0ca0e619a93fb3669ae162e12b895718daa87964d8ed155af50afbb2ee. Working tree was clean. Static review only; no builds, tests, or subagents. Confidence: high in the code paths described below; performance effects were not measured.

1. **Blocking — The file pair does not hold its read strategy and confinement setup constant.** `benchmarks/files/rust.rs:66`, `benchmarks/files/tide.rs:63`, and `benchmarks/files/tide.rs:48`.

   The Rust arm's `fs::read` opens each file, obtains its metadata, allocates a size-based buffer, and reads into that buffer (the pinned Rust source, `library/std/src/fs.rs:341`). Tide's `Fs::read` instead starts with an empty Vec and reads through an initialized 8 KiB stack buffer, copying each chunk into the Vec (`crates/tide-std/src/fs.rs:254`, `crates/tide-std/src/fs/real.rs`'s `Reader`, and `crates/tide-std/src/io/shared.rs:366`). This happens for every file even at the default size: equal bytes and hashes do not make the buffering, allocation, copying, or metadata syscall work equivalent. Tide also opens and attenuates a directory capability, using confined `openat2` resolution thereafter; the Rust twin only constructs a `Path` (`benchmarks/files/rust.rs:49`). That is an additional selected operation and confinement policy in just one workload.

   For the requested controlled timed comparison, these differences mix different workload recipes into the result. Minimum fix: read files with the same explicit fixed-size `Read::read`/Vec-extension loop in both arms, including the same Interrupted handling; use the root `Fs` with joined paths in the Tide arm to match the Rust arm's unconfined path recipe. Keep the capability backend's own checks and syscall overhead: those are the implementation costs being compared. No production std change is needed. If a confined-versus-unconfined high-level API comparison is intended instead, that is a different benchmark contract and must be stated explicitly rather than presented as matching buffering and I/O work.

2. **Blocking — Discovery can silently lose coverage while the equivalence test stays green.** `crates/tide-admit/tests/benchmarks.rs:125`–`141`.

   Only directories survive the root filter, only `tide.rs` and `rust.rs` are inspected within them, and only `tasks` is required to remain present. Removing the entire `contracts`, `files`, or `tcp` directory therefore removes its check without a failure. An added `benchmarks/extra.rs` or `benchmarks/files/other.rs` is also silently ignored. A malformed or relocated workload can escape the checker despite the requirement to keep all four pairs covered and fail on skipped sources.

   Minimum fix: require the four chosen pair names to be present while retaining discovery for additional pairs, reject unexpected root source files (allow the README), and reject extra `.rs` files or nested workload directories inside a pair instead of ignoring them. This can stay a small directory-shape check; no new framework or declaration language is needed.

3. **Non-blocking — A direct Rust TCP run can hang on its connection failure path.** `benchmarks/tcp/rust.rs:53` and `benchmarks/tcp/rust.rs:75`; `benchmarks/README.md:50`.

   If `connect` fails before establishing a connection, the client returns an error but the main thread joins the server first. The server is still blocked in `accept`, so the error is never returned. The integration check is protected by its external timeout; the README's direct invocation is not. This is also acknowledged in the builder report.

   Minimum fix for this workload-only bead: show the direct invocation under an external timeout and state that the timing runner must bound both arms. This preserves the successful-path workload and avoids adding asymmetric retry or polling work.

Other review results:

- Empty stdout is rejected. Rust must succeed, and Tide must have the same exit code; equal panics, errors, signals, or timeouts cannot pass merely by matching output.
- The simulated leg has its own success-and-stderr comparison against nonempty Rust default stdout. A missing or ignored `tests::simulated` produces no matching stderr and cannot silently pass. The eight-seed equality assertion alone is not the sole oracle.
- Current outputs contain no ports, timing, paths, or thread arrival order. File names and task summaries are sorted. The contracts algorithms match apart from their intentional executable contracts; task partitioning, Collatz computation, totals, and summaries match; TCP framing and application-level request/reply sizes match.
- The Rust arms contain no async or unsafe. All four parse the first size argument and have small defaults. Zero-sized workloads are handled by the inspected loops; numeric exhaustion and unrealistic sizes are not bounded by these programs.
- Current pair threads use separate package, executable, and working-directory paths. Loopback binding uses port 0 on the listener retained for accept, avoiding a reserve/drop/rebind race. Successful file runs remove their workload directory. A failed file operation may leave its private scratch contents for diagnosis and prevents a clean rerun in that same directory.
- Executable runs, including the simulator test, have a 120-second external timeout with a 10-second kill grace and closed stdin. Compile steps use the existing unbounded `checked` helper; the builder's recorded workspace command supplied an overall timeout. Concurrent independent test invocations sharing one target directory remain unsupported by the existing shared-std helper.
- `tests/support/admitted.rs` changes comments only. No helper signature or behavior changed for `examples.rs` or `build.rs`; the new package names are under a separate `benchmarks/` namespace.

Minimum patch for approval: resolve findings 1 and 2, then have this reviewer re-check those fixes. Finding 3 does not block. The builder's reported successful suite and mutation checks were read as prior evidence, not independently rerun.

Not checked: builds, tests, timings, or runtime fault injection, as instructed. The file comparison and discovery holes above remain unresolved in this candidate.

DONE
