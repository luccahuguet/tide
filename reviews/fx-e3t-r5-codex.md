Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-21.

## Verdict

**approve** — high confidence (95%). B1 and B2 are resolved. The targeted Opus fixes and conformance stub change are correct; no blocking contradiction was found.

## Scope

Reviewed commit `209a7f51de6c22ab46a4b648b7e7c027adb521bb` against main `0082d96`, using `git diff main..HEAD`.

Diff SHA-256: `37592cc3eae1a986d658eb24c2d88312f7774152e0c7e61274f18d7aa688f564`.

Read `GOVERNANCE.md` first, repository instructions, README, bead `fx-e3t`, reviews r1–r4, the affected design sections and their surrounding cancellation, sink, purity, and simulator rules, plus the conformance stub, README, runner, and relevant rule inventory. Searched `design/` and `conformance/` for affected APIs and contradictory wording.

This was a narrow re-check of the post-r1 fixes and rebase interaction. Changed no files. `git diff main..HEAD --check` passes; the working tree remains clean.

## Blocking

None. No patch is required for approval.

## Non-blocking

No new findings.

The existing question about attributing std-generated warnings to v1 exercised effects remains **for fx-djp** (`design/03-authority-concurrency.md`, §4.1/SINK-9), as recorded in Opus r2. It does not block this v0 correction.

## Reasoning

- **B1: termination failure and cleanup.** `03` §1.1 now requires a kill *attempt*, preserves ownership through the shielded wait regardless of its result, and returns `Cancelled` only afterward. The failed-kill `warn!` records the error before the potentially unbounded wait. This fits SINK-1’s non-checkpoint semantics and existing std warnings; delivery remains subject to ordinary sink filtering and loss rules. The permission-denied example is valid for an unprivileged parent after the child changes its real and saved IDs. [Rust `Child::kill`](https://doc.rust-lang.org/std/process/struct.Child.html#method.kill), [Linux `kill(2)`](https://man7.org/linux/man-pages/man2/kill.2.html).

- **B1: pipes and “until it exits.”** Closing the call’s pipe ends before waiting removes the specific deadlock where it stops draining captured output but keeps the readers open. With the readers closed, writes fail through `SIGPIPE`/`EPIPE`. A child may handle that failure and continue running; TASK-5 correctly allows waiting until its eventual exit, without promising a hard deadline. The companion specifies the same sequence. Waiting for the direct child’s exit also avoids depending on pipe EOF from descendants. [Linux `pipe(7)`](https://man7.org/linux/man-pages/man7/pipe.7.html).

- **B2.** Core §7.2 now says “like an uncancellable system call.” That agrees with TASK-3’s distinction between cancellable waits and uncancellable calls. It no longer categorically places filesystem-backed FIFOs in the uncancellable class.

- **Opus N1 and N3.** TASK-3 leads with the observable single-write contract: accepted bytes produce `Ok(n)`; cancellation errors report no accepted bytes. The readiness loop is explicitly an example and retries when the descriptor would block. The cleanup guidance now correctly applies `write`/`write_owned` to the writes cancellation may interrupt, preserving progress information for subsequent shielded cleanup. This agrees with Rust’s single-write contract. [Rust `Write::write`](https://doc.rust-lang.org/std/io/trait.Write.html#tymethod.write).

- **Opus N4 and N5.** The `exists` gloss explicitly limits `Ok(false)` to `NotFound`, propagates other failures, and restricts the checkpoint comparison to the main `Fs` operations. The owned-read guidance requires spare capacity and gives `Vec::with_capacity(n)`, preventing an empty destination from being mistaken for EOF. Neither clarification changes the underlying buffer contract. [Rust `fs::exists`](https://doc.rust-lang.org/std/fs/fn.exists.html), [Rust `Read::read`](https://doc.rust-lang.org/std/io/trait.Read.html#method.read).

- **Stub and rebase.** `Fs::exists(&self, p: impl AsRef<Path>) -> io::Result<bool>` matches 03, core §6.3’s “as std::fs,” and the updated design examples. No conformance case calls it. The trial copy is explicitly frozen historical material. The rebased purity rules do not conflict with `BufError`’s pure components. TASK-3’s simulator sentence now covers every rule; the optional short-write modeling suggestion remains outside this re-check.

- **Core balance and evidence.** The core diff adds only **13 bytes and one whitespace-delimited word**, with no net line increase. Delegating `exists` to “as std::fs” and compressing duplicated join wording preserve their contracts. `design/` and `conformance/` are unchanged from the r4 candidate `ac29d69`, so its recorded **24,996-token** count and runner result remain applicable prior evidence.

Not rerun: the token counter, conformance runner, or runtime cancellation tests.  
Runtime behavior remains unproven here; the recorded core count leaves four tokens of headroom.