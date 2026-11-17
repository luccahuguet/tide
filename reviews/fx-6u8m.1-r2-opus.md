Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate 3afa291

# fx-6u8m.1: Opus converge round (report only)

## Verdict

**request_changes.** There are two blocking findings. B1 needs a small code change to the `tasks` pair. B2 is a text-only fix to `benchmarks/README.md` and needs no re-check (AP-PROOF-001). The equivalence check itself is sound: I found no case where the arms differ and still pass, and no case where a good pair fails.

What I read: AGENTS.md; GOVERNANCE.md's rules as AGENTS.md renders them; beads fx-6u8m.1 (with its comments), fx-6u8m, fx-6u8m.2 and fx-6u8m.6; `git diff main...3afa291`; examples.rs and build.rs's `unadmitted`; the tide-std code the pairs call (`fs.rs`, `fs/real.rs`, `net.rs`, `net/real.rs`, `task/net.rs`, `io/shared.rs`'s `BufReader`, task spawning); the contract macro's erasure; the builder's report; and Codex's r1 pass and re-check. I ran no builds or tests and started no subagents. I did not open `trials/e1-checked/`, so I stay eligible to review fx-6u8m.2.

## Blocking findings

### B1: `tasks` does the same synchronization work at every size, so it cannot show the cost of `Shared`, channels or the simulator

`benchmarks/tasks/tide.rs:73-95`, `benchmarks/tasks/rust.rs:202-225`, `benchmarks/README.md:16`

Each worker takes the lock once and sends once, so a run makes 4 spawns, 4 `Shared::with`/`Mutex::lock` calls, 4 sends and 5 receives, whatever `count` is. Raising the size only adds Collatz arithmetic, which never reaches a scheduling point.

Concrete case: fx-6u8m.2 has to report "simulator speed: the real runtime against the simulated one for workload 4", including the OS-thread baton cost and the trace-formatting cost found in fx-ic2.12. Run `tasks 10000000` both ways. The simulated run passes the baton about a dozen times and otherwise runs native arithmetic, so the simulated/real ratio comes out near 1×. The table would say the simulator is nearly free, which flatters Tide. The same applies to the `Shared`/channel against `Mutex`/`mpsc` comparison the bead names for this pair: at any size it is 4 operations against 4.

Minimum fix: make the synchronization work grow with `count`. Make the same change in both arms:
- each worker sends one message per number, or per fixed block (say 16 numbers);
- it adds its partial totals to the `Shared`/`Mutex` cell at the same cadence;
- the receiver folds the messages into per-worker summaries, so the output stays deterministic.

Keep the default size small enough that the 8-seed simulated test stays well under the 120 s timeout, and time it once. If the coordinator would rather keep this a compute-bound pair, the minimum fallback is for the README to say plainly that the pair has a fixed 4 locks and 4 sends, so it measures thread spawning and checked arithmetic, and that a simulator row taken from it measures almost no scheduling. fx-6u8m.2 would then need another workload for that row.

### B2: `benchmarks/README.md` does not name the main inherent Tide costs in `files` and `tcp`, or the forced profile flags

`benchmarks/README.md:27-43`

The README says "what differs is the cost of Tide's implementation". That is true, but the costs it lists are not the ones that will dominate the timings fx-6u8m.2 publishes. From the code:

- **`files`**: the README says "its authority checks and its path handling". The real difference is extra system calls (`crates/tide-std/src/fs/real.rs:135-158`):
  - `rename`: Tide opens both parent directories with `O_PATH`, calls `renameat`, then closes both. That is 5 syscalls; std's `rename` is 1.
  - `remove_file`: `openat` the parent, `unlinkat`, `close`: 3 against `unlink`'s 1.
  - `exists`: `openat(O_PATH)`, `fstat`, `close`: 3 against `statx`'s 1. This happens once per run.
  - `read_dir`: Tide reads the whole directory into a snapshot and sorts it (`names`, `real.rs:341-366`); std reads lazily. The program then sorts again in both arms, so Tide sorts twice.
  - In total that is about 18 syscalls per file against 12. The authority checks themselves (a mode compare, a NUL-byte scan, a cancellation checkpoint) cost little next to that.
- **`tcp`**: the README says only that operations "are also cancellation checkpoints". In fact Tide's real sockets are nonblocking (`net/real.rs:4-9`, `net.rs:258`). A read that would block costs `read`→EAGAIN, then `poll` on the socket plus the thread's eventfd, then an eventfd `read`, then `read` again (`net/real.rs:retry`, `task/net.rs:23-41, 119-143`). Each check also takes the runtime's mutex, and each wait registers a waiter. One request/reply round trip comes to roughly ten syscalls against four for std's blocking sockets. The pair measures exactly this cost.
- **Profile flags, every pair**: the driver forces `-Coverflow-checks=on`, `-Zinline-mir=no` and `-Zcross-crate-inline-threshold=never` in every profile (`crates/tide-admit/src/invocation.rs:358-367`; README build modes). The README mentions overflow traps only under `contracts`. They also apply to the arithmetic in `tasks` (`3 * n + 1`, `steps += 1`) and to the counters in `files` and `tcp`. fx-6u8m.6's flags-only Rust configuration is what separates this cost out.

Minimum fix: one sentence per item above in the README's per-pair list, and change the `contracts` bullet so it no longer implies the overflow checks are specific to that pair. Text only.

## Non-blocking findings

1. **The two harness helpers duplicate existing code.**
   - `rust_arm` (`crates/tide-admit/tests/benchmarks.rs:38-48`) is build.rs's `unadmitted` (`build.rs:4850-4857`) almost word for word.
   - `run`'s `timeout -k 10 …` command (`benchmarks.rs:51-62`) repeats examples.rs's `timed` (`examples.rs:148-159`), except that `timed` also removes `RUST_TEST_NOCAPTURE`. The simulated leg would want that too, to avoid depending on the environment.
   - Each now has a second caller, so `timed` and a plain-rustc build belong in `support/admitted.rs` (AP-MINIMAL-001). It is cheap to do now.
2. **Dead check.** `benchmarks.rs:167-173`: `pairs.len() < EXPECTED.len()` can never be true once the loop just above has required every name in `EXPECTED`. Delete it. The report keeps it as "the literal check the brief asked for", but it adds no coverage.
3. **Weak diagnostics when a run leaves files behind.** `run` (`benchmarks.rs:63-68`) checks that the directory is empty before the caller looks at the exit status. A failing `files` arm (an error in the middle of the run leaves `files-workload/`) therefore panics with only "left files behind", without the program's stderr or exit status. Include `shown(&out)` in that message, or check the directory after the status assertion.
4. **The builder's report is wrong about the explicit size.** Decision 5 says that with size `7`, "in `tasks`, one worker gets no numbers". With 7 numbers and 4 workers, every worker gets at least one (worker 3 gets {4}). A size of 3 would test the empty-worker boundary (`longest (0, 0)`). This is in the report only, not in the repository.
5. **The `contracts` overhead is concentrated in one place.** `insert_sorted`'s `requires(xs.is_sorted())` and `ensures(xs.is_sorted())` are each O(n), on an insert that is itself O(n). They roughly triple that function's cost, while the `sieve` and `isqrt` clauses are O(1). This is a fair test of executable contracts. It is worth one README clause, so fx-6u8m.2 does not present the debug-build ratio as a typical contract cost.
6. **The hand-build instructions cover only the Rust arm.** README lines 66-73 show how to build the Rust arm. For the Tide arm they point to `admitted.rs`, and that path also needs the std built first. This is acceptable until fx-6u8m.6's script exists. The README could say that the script will be the supported route.
7. **The toolchain claim has an exception.** README line 47 says the Rust arm uses "the toolchain `rust-toolchain.toml` pins". The harness uses `$RUSTC` when it is set (`benchmarks.rs:41`), as build.rs does. Add "unless `RUSTC` is set".
8. **Comment over the line width.** `support/admitted.rs:21` is a doc-comment line of about 125 columns. Rewrap it.
9. **E1 held-out provenance.** fx-6u8m.2's constraint says the programs it times count as examples under E1 protocol §2, and that their authors and reviewers must never have opened `trials/e1-checked/`. These are fx-6u8m.1's programs. Record on fx-6u8m.1 whether the builder and the Codex reviewer had opened those paths, so fx-6u8m.2 can rely on the record. This reviewer has not.

## Asymmetries left after B1 and B2, all inherent and acceptable once named

- `contracts`: the contract clauses (debug and test builds) and the forced profile flags. The algorithms, data structures and output are otherwise identical, line for line.
- `files`: the extra syscalls and the snapshot sort listed in B2. There is also a cancellation checkpoint on every `File` read and write, and two on each `write_all` (one in `write_all`, one in each `write` inside it). The calls, paths, order, `read_all` loop, 8 KiB stack buffer and allocation pattern are otherwise identical.
- `tcp`: the nonblocking socket and wait cost in B2. Tide's `BufReader` zero-fills its 8 KiB buffer, where std leaves it uninitialized: two allocations per run, which is negligible. The server is a scoped task borrowing the listener in Tide, and a detached `thread::spawn` that owns the listener in Rust (the r1 fix). On a successful run both are one OS thread, spawned and joined once, so this is negligible. Framing, one `write_all` per message, and the reply format are identical.
- `tasks`: spawns go through `Tasks::scope` and `spawn_scoped` OS threads (`task.rs:1204`), and `Shared` and the channel are runtime-mediated. After B1 these become measurable, which is the point. The Collatz loop is identical apart from overflow checks.

## Reasoning

**Lens 1: correctness of the equivalence check.**
- **Exit status.** A pair passes only if the Rust arm exits 0, `t.status.code() == r.status.code()`, and the stdout bytes are equal. A signal or a timeout gives `None` or 124, so it cannot match a Rust `Some(0)`. Panics give 101.
- **Empty output.** Empty Rust output is rejected. In the simulated leg, an empty or missing `tests::simulated` output (renamed test, `--exact` matching nothing, a wrong records filter) gives empty stderr, which cannot equal the non-empty Rust default. The eight-seed equality assertion is an extra check, not the only oracle.
- **Nondeterminism.** None reaches the output: listings are sorted, summaries are sorted, the TCP port is never printed, totals do not depend on order, and `TIDE_TEST_SEED` and `TIDE_SIM_SEED` are removed from the environment.
- **Discovery.** It is strict. Missing arms or pairs, stray files, nested directories and dotfiles all fail, and the regression test covers each case.
- **Parallel runs.** Each pair gets its own package directory, binary directory and working directory. The discovery test writes under `benchmarks/layout/`, so it does not collide with them. `std_dir`'s `OnceLock` serializes the shared std build across the pair threads. Concurrency is at most 4 rustc processes, within the heavy-run agreement.
- **Timeouts.** Every program run is bounded at 120 s plus a 10 s kill grace. Builds use the existing unbounded `checked`, as examples.rs does.
- **Arguments.** Explicit sizes go through the same parsing in both arms. Tide's `env.args()` includes argv[0], since `limit: 7` matches.
- **Contract build mode.** The equivalence run builds the Tide arm in debug, so the contracts are checked during the check. Release erasure is `cfg!(debug_assertions)` (`tide-macros/src/contract.rs:79-90`).

I found no way for the arms to differ and still pass, other than the intended one: both arms changing their output the same way.

**Lens 2: fairness.** After r1, the algorithms, data structures, I/O call sequences, buffers and sizes match line for line in `contracts`, `files` and `tcp`. The Tide arm is not handicapped anywhere: no extra attenuation, no extra work in user code. It is flattered in one place (B1), and its inherent costs are under-described in two pairs and one cross-cutting setting (B2).

**Lens 3: simplicity.** The harness is about 230 lines and reuses `admitted.rs`. A separate file rather than an extension to examples.rs is justified, since the examples grammar describes single-file outcomes. What remains is duplication of two helpers and one dead check (non-blocking 1 and 2). The pairs have no dead code. The `RunError` variants are all used, apart from `Size` and `Io` being shown only through `Debug`, which is fine.

**Lens 4: claims.** I checked each README statement against the code. Sizes, arguments, `files-workload` handling, port handling, the absence of async and unsafe, "nothing beyond std" (`fs::exists` is stable), release erasure, OS-thread tasks, discovery strictness and the simulated leg all hold. The claims that fall short are the cost descriptions (B2) and the toolchain exception (non-blocking 7). The top-level README paragraph is accurate, and "no measurements are published yet" is correct.

DONE
