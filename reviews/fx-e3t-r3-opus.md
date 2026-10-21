# Tide review, fx-e3t round 3 (Opus): cancellation gaps, confirmation

Reviewer: Opus, round 3 (confirmation), read-only, candidate `git diff`
SHA-256 `e235cd773182e719d66d29e26eddd129a074dc629329e8703a95dd817c95dfe0`
(HEAD `76dfe06`, branch `fx-e3t`), 2026-10-21. The hash matched before and
after the review. `git diff --check` passes. `scripts/count-tokens.sh
design/reference.md` gives 24,996, which matches the companion's count line
and is under D14's 25,000.

## Verdict

**clean.** Round 2's B1 is resolved, the pipe rule is correct, and I found
no contradiction across `design/`. No finding blocks. This is a full clean
pass.

## Blocking findings

None.

## Non-blocking findings

- **N1. "its own ends of the child's pipes" assumes the pipes exist**
  (nit; `03` §1.1 "Cancelled `output` and `status`", companion §20
  Cancellation). `status()` inherits all three streams by default, so it
  usually holds no pipe. `output()` pipes only stdout and stderr by default.
  Any piped stdin is already closed before either call waits, since Rust's
  `wait` and `wait_with_output` drop `stdin` first. The rule is vacuously
  true where there are no pipes, and it never claims they exist. "closes any
  pipes it holds to the child" would be exact. No reader is misled.
- **N2. The 03 sentence carrying the new clause is long** (nit). The kill,
  the pipe clause with its reason, the shielded wait, and the return all sit
  in one sentence. "then waits for it to exit" still binds to the child.
  Splitting off the pipe clause would read better.
- Round 2's N1–N3 still stand as they were. N4 is still for fx-djp. None is
  now wrong. The `exists(&self, p) -> bool` copies in
  `trials/e1-dialect/rustc/std/src/lib.rs` and `trials/e1-origins/*/reference.md`
  are records of past trials, as N3 said.

## Reasoning

- **r2 B1 is resolved.** After the kill attempt and before the wait, 03 now
  closes the call's ends of the child's pipes. A surviving child can then no
  longer block on a full stdout or stderr pipe that the call stopped
  draining, so the call cannot keep its own child from exiting. The
  companion bullet says the same thing, in the same order.
- **The pipe rule is correct.**
  - *Readers.* Rust creates the parent's pipe ends `O_CLOEXEC`. The child
    and its descendants hold only write ends. Once the call closes its read
    ends, no reader is left, so a write gets `EPIPE`, or `SIGPIPE` under the
    default disposition.
  - *Signal permissions.* The kernel raises `SIGPIPE` in the writer itself,
    so `kill(2)`'s permission check does not apply. The rule therefore
    works for the setuid `EPERM` child that motivated it.
  - *Disposition.* Rust's `Command` resets `SIGPIPE` to `SIG_DFL` in the
    child, both in its fork/exec path and with
    `posix_spawnattr_setsigdefault`. A Rust child then ignores it again at
    startup and sees `EPIPE`. "SIGPIPE or EPIPE" covers both cases.
  - *Stdin.* A piped stdin is already closed, so the rule changes nothing
    there and adds no new EOF behaviour.
  - *Grandchildren.* They may keep write ends open, but the call waits for
    the child's exit, not for EOF, so they cannot hold it.
  - *Children that ignore `EPIPE`.* Such a child may keep running, which is
    exactly TASK-5's "until the child exits". The call no longer causes the
    wait, so TASK-5's wording is now accurate.
- **No contradictions.** I grepped `design/` for `exists`, `output`,
  `status`, `kill`, `pipe`, `SIGPIPE`, `EPIPE`, `stdin`, `warn!`,
  `read_owned`, `write_owned`, `BufError`, `written`, `write_all`,
  "uncancellable", "never interrupted", "cannot be interrupted", deadline,
  and soft. The findings:
  - The Receivers note still describes `wait_with_output` closing `stdin`
    and taking the output pipes. That is the caller-held-`Child` case and
    agrees with the new rule.
  - The `exists` signature appears consistently in the 03 table and note,
    core 6.3's "as std::fs" list, the FX0520 row, and the §9 test.
  - TASK-3, TASK-5, TASK-6, core 7.1 and 7.2, and D8 ("Joining a scope
    cannot be cancelled") agree.
  - §9.8 leaves real child processes unsimulated, so the kill and pipe path
    places no new obligation on the simulator.
- **Soundness.**
  - Killing the child, then closing the pipes, then a shielded reap is what
    tokio's `kill_on_drop` with reaping would give, adjusted for Tide's
    structured scopes. The `spawn` escape hatch keeps Rust's
    no-kill-on-drop behaviour.
  - `pidfd_send_signal`, which Rust uses where available, applies the same
    permission check as `kill`, so the `EPERM` case holds on either path.
  - Shielding and checkpoints are unchanged: the call stays a checkpoint
    and still returns `Cancelled`, and only its internal wait is
    uninterruptible, as TASK-5 lists.
- **Tier rule.** N1 and N2 are nits in v0 text this change edits. Nits never
  block.
