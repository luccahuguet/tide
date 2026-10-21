# Tide review, fx-e3t round 1 (Opus): cancellation gaps

Reviewer: Opus, round 1, read-only, candidate `git diff` SHA-256
`396e2f8b592a66cf2af406b9020b3de796156e3f7c77f9c0c75115611660b7cb`
(HEAD `76dfe06`, branch `fx-e3t`), 2026-10-21. The hash matched before and
after the review. `git diff --check` passes.

## Verdict

**clean.** No blocking findings. Seven non-blocking notes, all nits or
clarity improvements; none contradicts another text in `design/`.

## Blocking findings

None.

## Non-blocking findings

- **N1. TASK-3 states a mechanism where it needs a contract**
  (`03` TASK-3, the new single-`write` paragraph). "It waits for the
  descriptor to accept data and then makes one non-blocking write" says how
  the runtime does it, not what the caller can rely on. The contract is:
  a socket or pipe `write` returns a cancellation error only if the kernel
  accepted no byte, and any accepted bytes return `Ok(n)`. The `evented`
  row (`03` §1 runtime table) lists io_uring and IOCP first, so a readiness
  description reads as a backend constraint. The literal text is also off
  by one case: when another task holding `&TcpStream` fills the send
  buffer between readiness and the write, the write gets `EAGAIN` and
  waits again, so it is not "one" write. The guarantee still holds in that
  case. Smallest fix: lead with the contract and give the mechanism as
  "for example", or write "a non-blocking write, waiting again if it would
  block".
- **N2. Where "The simulator follows the same rules (§9.3)" now sits**
  (`03` TASK-3). The sentence used to close the several-system-calls
  paragraph. It now ends the writes paragraph, so it can be read as
  covering only writes. §9.3 itself says cancellation cuts latencies short
  "only where TASK-3 lets it", so this is not a contradiction. Related: no
  text says simulated socket writes can be short. If they never are, the
  partial-`write_all` hazard this change documents can never happen under
  the simulator. A clause in §9.3 or §9.4 saying a simulated write may
  accept a seed-chosen prefix would make that hazard reachable (optional).
- **N3. Wording of the "finish or frame" sentence** (`03` TASK-3). "Code
  that must finish or frame a stream after a cancellation, under `shield`,
  uses `write` or `write_owned`". What matters is that the write that may be
  cancelled is a `write` or `write_owned`, so the shielded cleanup knows
  where the stream stands. Inside `shield`, nothing is cancelled anyway. The
  next clause, about a failed `write_all`, recovers the meaning, but "the
  write that may be cancelled uses ..." would be clearer.
- **N4. The `exists` gloss overstates Rust slightly** (`03` §1.1 `exists`).
  "`Ok(false)` when nothing is at `p`" covers a path like `file.txt/x`, but
  Rust's `fs::exists` maps only `NotFound` to `Ok(false)` and returns `Err`
  for `NotADirectory`. "Is Rust's `std::fs::exists`" governs, so there is no
  contradiction. Also, "like every `Fs` method" includes `read_only()`, which
  is not a checkpoint; "like every `Fs` operation in the table" would be
  exact.
- **N5. `read_owned(Vec::new())` returns `Ok(empty)`, which looks like end
  of input** (`03` §1.1 Owned buffers). The no-spare-capacity rule matches
  Rust's `read` into an empty slice and tokio-uring's `IoBufMut`, and the
  text states it. Still, `Vec::new()` is the most natural argument, and a
  loop that treats an empty append as EOF stops at once. A lint or a
  companion example with `Vec::with_capacity(n)` would cover it. This is
  not a soundness hole, and the rule needs no change.
- **N6. The `output`/`status` kill path does not say what happens if
  `kill` itself fails** (`03` §1.1 Cancelled `output` and `status`). This
  is rare: `SIGKILL` to an unreaped child of this process seldom fails. The
  likely intent is "waits anyway". Optional.
- **N7. Core coverage of the new rules** (`reference.md` 2.2, 7.1, 7.2;
  companion §20). These do not block, since the core does not name owned
  buffers at all.
  - Core 2.2's list of std error types that are `Error + Send + Sync +
    Pure` does not name `io::BufError`.
  - Core 7.1's "scope exit and `join` wait, never cancelled (7.2)" is
    slightly awkward. It loses no meaning: 7.2 still says joins and scope
    exit "are not" checkpoints and are never interrupted.
  - Core 7.2's Cleanup bullet still says "bounded cleanup" without
    qualification. The new "so deadlines are soft" two bullets earlier
    closes item 5.
  - The companion's Cancellation bullet in §20 omits the TASK-5 caveat that
    the post-kill wait cannot be cut short. One clause would carry it
    outside the budget.

## Reasoning

- **All five items are decided in 03 with reasons.**
  1. `output`/`status` kill the child and wait for it under `shield`. The
     note rejects two alternatives: leaving the child running (the tokio
     default without `kill_on_drop`) and waiting for the child to finish
     on its own. It also points to `spawn` for a child that should outlive
     a cancellation, and adds the uninterruptible wait to TASK-5.
  2. `exists` becomes `io::Result<bool>`, Rust's `std::fs::exists`
     (stable since 1.81, with broken symlinks giving `Ok(false)`, as the
     note says). It gives the reason that a `false` returned on
     cancellation would report a file that is there as missing.
  3. The error type is `io::BufError`, modeled on `io::IntoInnerError`.
     Rust's method names and its `From<IntoInnerError<W>> for io::Error`
     are cited correctly.
  4. A single `write` returns a cancellation error only when it sent
     nothing. `write_owned` reports a count through `written()`.
     `write_all` keeps Rust's signature and, as in Rust, loses the count.
  5. TASK-5 and TASK-6 now say that cleanup deadlines are soft, and core
     7.2 says "so deadlines are soft".
- **No contradictions.** I grepped every touched API across `design/`
  (`exists`, `output`, `status`, `read_owned`, `write_owned`, `BufError`,
  `written`, `write_all`, `kill`, "never interrupted", deadline, shield).
  - The only `exists` caller (the 03 §9 sim test) is updated to `.unwrap()`.
  - The companion's FX0520 row gives `fs.exists(path)?`.
  - Core 6.3 lists `exists` under "as std::fs".
  - Nothing in `design/` still says these calls return `io::Result` or a
    bare `bool`.
  - The Receivers note on `wait_with_output` still agrees with the new
    `output`/`status` note, which explains why the two cases differ.
  - D8's wording, "operations that hand a value over return it inside the
    error", covers `BufError`.
- **The decisions are sound.**
  - The new single-write guarantee strengthens TASK-3's "completes before
    the cancel takes effect returns its own result". It also keeps
    `BufWriter`-style callers exact: their bookkeeping relies on a failed
    `write` having written nothing.
  - Tide's `ErrorKind::Cancelled` is distinct from `Interrupted`, so
    `write_all`'s retry loop does not spin on it.
  - `SIGKILL` cannot be ignored, and the remaining case, a child stuck in an
    uninterruptible system call, is stated.
  - Grandchildren holding the child's pipes cannot stall a cancelled
    `output`, because it waits for the child's exit, not for EOF on its
    pipes.
  - `BufError` holds an `io::Error` (core 2.2: `Send + Sync + Pure`) and a
    `Vec<u8>`, so its `Send + Sync + Pure` claim holds.
- **The additions are justified.**
  - Before this change, the return shape for `read_owned` and `write_owned`
    was the buffer on success with no count. Under that shape,
    write-all semantics for `write_owned` and "one read, appended"
    semantics for `read_owned` are the only choices that need no new
    field.
  - Of `BufError`'s methods, `is_cancelled()` is required by §1.1's intro
    and `written()` is item 4. The other four copy `IntoInnerError`'s
    names, which keeps the set predictable. Only `kind()` is pure
    convenience, and it costs little.
- **Budget.** I did not rerun the counter. The core edits drop the `exists`
  signature line, which "as std::fs" now covers, and trim 7.1 without
  losing meaning (see N7). The companion's count line is updated.
- **Tier rule.** Every note concerns v0 text and is a nit or a clarity
  improvement, so none blocks, and none goes to fx-djp.
