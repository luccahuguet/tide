Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate fc0fa0c

# fx-6u8m.1: finder re-check of r2-opus.md

## Verdict

**approve**, on one condition: a one-line correction to `benchmarks/README.md` (see B2). That correction is text only, so under AP-PROOF-001 it needs no re-check. B1 is fixed. Every non-blocking item from r2 was either taken or answered with a reason. The amendment introduces no new code defect.

Scope of this pass: `git diff 3afa291 fc0fa0c`, the updated `report.md`, `workspace-r3.log`, and the tide-std sources that the new README text describes (`sync/mpsc.rs`, `sync.rs`). The worktree is clean at fc0fa0c, and `git diff --check` passes. I ran no builds or tests and started no subagents.

## B1: `tasks` synchronization did not grow with size — fixed

- **Both arms now work in blocks.** `tasks/tide.rs:93-115` and `tasks/rust.rs:78-101` share the same `BLOCK = 16` loop. After each block, a worker updates the shared cell once (`Shared::with` in Tide, `Mutex::lock` in Rust) and then sends once. The order of operations, the `Results` type, the fold, and the output format are identical in the two arms. The only extra line is Tide's `let numbers = block.numbers;`, which copies a value out before the `Pure` closure.
- **The amount of synchronization now scales with `count`.** With 4 workers and the default 2,000 numbers, each worker gets 500 numbers, which is ⌈500/16⌉ = 32 blocks. That makes 128 cell updates, 128 sends and 128 receives per run, growing linearly with `count`.
- **The channel semantics match.** Tide's `mpsc::channel(tasks)` is unbounded (`sync/mpsc.rs:12-17`), like std's `mpsc::channel`, so neither arm applies back-pressure.
- **The output is deterministic.** The program folds blocks into a table indexed by worker. `Results::longest` keeps the larger step count and breaks ties toward the smaller number, so the result does not depend on the order in which blocks arrive. The block counts depend only on `count`.
- **The empty-worker case is covered.** With `EXPLICIT = "3"`, worker 3 sends no blocks and prints `longest 0 (0 steps)` in both arms.
- **The default size stays small.** The default is still 2,000. The report gives 0.055 s for the 8-seed simulated test, and 2.72 s for the whole benchmarks binary in `workspace-r3.log`. The simulated leg's comparison code is unchanged.

## B2: inherent costs under-described — fixed, with one statement still wrong

The new text describes four kinds of inherent cost. I checked each against the code:

- **Flags in every pair.** `invocation.rs:356-361` confirms the four flags.
- **`files`.** The syscall counts (5 against 1, 3 against 1, 3 against 1, about 18 against 12), the double sort and the per-call checkpoints all match `fs/real.rs` and `fs.rs`.
- **`tcp`.** The EAGAIN → poll → eventfd read → re-read sequence matches `net/real.rs` and `task/net.rs`.
- **`contracts`.** The concentration of cost in `insert_sorted` is stated accurately.

**One statement is still wrong** (`benchmarks/README.md`, `tasks` bullet): "with sends and receives also cancellation checkpoints". In the Tide std, sends on the unbounded channel are not cancellation checkpoints:
- the doc comment on `channel` says "Sending does not check cancellation or block" (`sync/mpsc.rs:12`);
- `send` checks `cancelled()` only when `self.bound.is_some()` (`sync/mpsc.rs:113`).

**Minimum fix:** change it to "receives are also cancellation checkpoints; sends on this unbounded channel are not". This is text only, so no re-check is needed.

**Optional improvement, not blocking:** say that `Shared::with` acquires the lock in FIFO order (`sync.rs:352`), while std's `Mutex` is not fair. Under contention this is a real behavioural difference, but with 4 workers and one update per 16 numbers it should be small.

## Non-blocking items from r2

1. **Duplicated helpers: taken.** `timed` and `plain_rustc` now live in `support/admitted.rs`, and `examples.rs`, `build.rs` and `benchmarks.rs` use them.
   - Each helper carries `#[allow(dead_code)]` for the one test binary that does not call it. That is needed because `admitted.rs` is included as a module in each binary.
   - `examples.rs` still uses `Stdio` (in its rejected-variant branch), so keeping that import is correct.
   - `benchmarks.rs` now also clears `RUST_TEST_NOCAPTURE`.
   - The behaviour of `examples.rs` and `build.rs` is unchanged: same arguments, same environment, same working directory.
2. **Dead `pairs.len()` check: taken.** It is deleted, and the doc comment now states why the check was implied.
3. **Diagnostics when files are left behind: taken.** The panic message now includes `shown(&out)`.
4. **Explicit size: taken.** It is now 3, and report decision 5 is corrected.
5. **README note on contract cost: taken.**
6. **Hand-building a Tide arm: taken.** The README now points to `admitted.rs` and fx-6u8m.6.
7. **The `RUSTC` exception: taken,** in the README, in the `benchmarks.rs` doc comment, and in the `plain_rustc` doc comment.
8. **Over-wide line in `admitted.rs`: taken.** The line is rewrapped.
9. **E1 held-out provenance: answered with a reason.** The builder states it never opened `trials/e1-checked/`, and it leaves the Codex reviewer's provenance for tide-lead to record on the bead. That is reasonable. tide-lead should record it before fx-6u8m.2 relies on these programs. This reviewer never opened those paths.

## New findings

No code defects. Two text nits; neither blocks:

- **Line wrapping.** The README line "`rust-toolchain.toml` pins, unless `RUSTC` is set), runs both in an empty directory, once without an" runs past the paragraph's wrap width. The `benchmarks.rs` doc comment (lines 3-5) now has a short line, "toolchain), runs both, each in its own", in the middle of the paragraph. Rewrap both.
- **Flag description.** The every-pair bullet says "std's generic code is neither MIR-inlined nor shared across crates". In fact `-Zinline-mir=no` turns off MIR inlining for all code, not only std's generics. Suggested wording: "no MIR inlining at all, no inferred cross-crate inlining, and generics instantiated per crate".

Process note, not a finding on the candidate: the report says that one focused run started at a 1-minute load of 16.8, above the limit of 12 in the heavy-run rule. The builder says so plainly, and the final proof ran under that limit.

DONE
