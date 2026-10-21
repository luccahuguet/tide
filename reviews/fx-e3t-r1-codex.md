Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-21.

## Verdict

**request_changes** — high confidence (90%). The direction is sound; two v0 specification gaps need small corrections.

## Scope

- HEAD: `76dfe0634a41c314112760e2e44cfef0893a35af`, branch `fx-e3t`.
- Recomputed `git diff` SHA-256: `396e2f8b592a66cf2af406b9020b3de796156e3f7c77f9c0c75115611660b7cb` — **matches**. It remained unchanged at the final check.
- Read `GOVERNANCE.md` first, then `AGENTS.md`, `README.md`, `fx-e3t` through `br --no-db show`, `design/01-design.md`, all changed sections, the requested core and companion sections, and consistency rounds 95–97.
- Searched all of `design/` for the affected APIs and cancellation rules; inspected the relevant error-purity and simulation rules.
- Checked primary Rust, Tokio, and Linux documentation. `git diff --check` passes.
- Changed no files.

## Blocking

1. **Child termination failure has no defined cleanup outcome.**

   **Location:** [03 §1.1, “Cancelled `output` and `status`”](/home/lucca/pjs/lab/tide-fx-e3t/design/03-authority-concurrency.md:222), TASK-5, and [companion §20](/home/lucca/pjs/lab/tide-fx-e3t/design/reference-companion.md:556).

   The rule promises to kill the child, reap it, and return `Cancelled`, but `Child::kill` is fallible. For example, an unchecked setuid helper can change its real and saved user IDs so its unprivileged parent cannot signal it. Rust exposes a fallible kill operation; Linux can refuse it with `EPERM`. [Rust `Child::kill`](https://doc.rust-lang.org/std/process/struct.Child.html#method.kill), [Linux `kill(2)`](https://man7.org/linux/man-pages/man2/kill.2.html).

   Propagating that failure immediately drops the only child handle and recreates the orphaning gap. Continuing to wait preserves ownership, but may wait indefinitely for a child that was never killed—another exception the present explanation omits.

   **Smallest fix:** specify an *attempt* to kill, require the shielded wait even when termination fails, and retain `Cancelled` as the result after reaping. State how the termination failure is reported and that denied termination can require waiting for natural exit. Align TASK-5 and companion §20.

2. **The core makes the uncancellable class broader than TASK-3.**

   **Location:** [reference §7.2, Level-triggered](/home/lucca/pjs/lab/tide-fx-e3t/design/reference.md:665), against [03 TASK-3](/home/lucca/pjs/lab/tide-fx-e3t/design/03-authority-concurrency.md:1228).

   The added phrase says a “file-system call already running” is never interrupted. TASK-3 instead classifies calls by cancellability and explicitly includes pipe waits among cancellable operations.

   A filesystem path can name a FIFO. A `File` reading that FIFO can be waiting for pipe data when cancellation arrives; TASK-3 permits cancellation, while the core’s categorical wording says it cannot interrupt the call. FIFOs are filesystem entries with pipe behavior. [Linux `fifo(7)`](https://man7.org/linux/man-pages/man7/fifo.7.html).

   **Smallest fix:** replace “like a file-system call already running” with “like an uncancellable system call.” This matches TASK-3 and shortens the core.

Both fixes affect the edited v0 tier, so both block under the merge rule.

## Non-blocking

No additional findings. No later-tier-only finding to route **for fx-djp**.

## Reasoning

All five items have explicit decisions and reasoning in 03.

- **`Fs::exists`:** `io::Result<bool>` preserves missing-file, permission, confinement, and cancellation outcomes. The core, habit table, and replicated-node example agree. This follows Rust’s fallible filesystem API. [Rust `fs::exists`](https://doc.rust-lang.org/std/fs/fn.exists.html).
- **Owned-buffer success semantics:** one read appending into spare capacity gives a measurable length change without another return field. EOF and zero spare capacity are covered. Returning the unchanged input on failure is compatible with a single read’s error contract. [Rust `Read::read`](https://doc.rust-lang.org/std/io/trait.Read.html#method.read).
- **`BufError`:** the concrete buffer type, recovery methods, and explicit buffer-dropping conversion are justified and small. The method set follows `IntoInnerError`; `kind`, `is_cancelled`, and `written` serve Tide’s cancellation and recovery contracts. [Rust `IntoInnerError`](https://doc.rust-lang.org/std/io/struct.IntoInnerError.html).
- **Write counts:** distinguishing a single `write` from repeated writes is correct. A successful partial write reports `Ok(n)`; `write_owned` retains cumulative progress, while `write_all` keeps its existing result type and documents the lost count. Tokio makes the same cancellation distinction. [Rust `Write`](https://doc.rust-lang.org/std/io/trait.Write.html), [Tokio write cancellation](https://docs.rs/tokio/latest/tokio/io/trait.AsyncWriteExt.html#method.write_all).
- **Cleanup deadlines:** making them explicitly soft fits Tide’s checkpoint model and compulsory joins. Killing and reaping the hidden direct child is a reasonable library policy under D18; it does not require new Rust syntax or process-tree supervision.

The core edits are balanced: the explicit `exists` signature and duplicated join wording pay for the deadline clarification. Net change is only **19 bytes and two whitespace-delimited words**. That is consistent with the author’s reported **24,995 tokens**, but is not an independent token measurement.

Not checked: the exact token recount or runtime behavior; this was a read-only design review. The reported count leaves five tokens of headroom.