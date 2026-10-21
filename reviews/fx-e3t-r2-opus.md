# Tide review, fx-e3t round 2 (Opus): cancellation gaps, confirmation

Reviewer: Opus, round 2 (confirmation), read-only, candidate `git diff`
SHA-256 `f8bdc00cf79742f9787b712de46327905e509a8a4aea64fc51ae9b8fa85736b0`
(HEAD `76dfe06`, branch `fx-e3t`), 2026-10-21. The hash matched before and
after the review. `git diff --check` passes. `scripts/count-tokens.sh
design/reference.md` gives 24,996, which matches the companion's count line
and is under D14's 25,000.

## Verdict

**changes needed.** Round 1's Codex B1 and B2 are resolved, and the fixes
introduce no contradiction elsewhere in `design/`. One new gap in the B1 fix
blocks: the failed-kill path can hang because of the call's own pipes.

## Blocking findings

- **B1. A cancelled `output()` whose kill failed can deadlock on its own
  pipes** (`03` §1.1 "Cancelled `output` and `status`"; TASK-5;
  companion §20 Cancellation). The new text says the call "waits for the
  child to exit by itself", and TASK-5 says such a child "holds it until it
  exits". `output()` captures stdout and stderr through pipes, and the text
  says only that "output read so far is lost". It does not say that the
  call stops holding those pipes. Suppose an implementation stops reading
  but keeps the read ends open while it waits. A child that survived the
  kill then blocks writing to a full pipe (64 KiB on Linux), never exits,
  and the scope hangs forever. That contradicts "until it exits", because
  the call itself prevents the exit. The case is not exotic. `sudo` sets
  its real and saved user IDs to root, so `proc.command("sudo")..output()`
  is exactly the `EPERM` case the text names. `status()` is unaffected,
  since it inherits stdio by default.
  Smallest fix, in 03 only: after the kill attempt, the call "closes its
  ends of the child's pipes, then waits". A child that is still writing
  then gets `EPIPE`, or dies of `SIGPIPE`, which Rust's `Command` resets to
  the default in the child. A clause in the companion bullet is optional.
  The fix changes the v0 tier this change edits, so it blocks.

## Non-blocking findings

- **N1. Core 2.2 and 7.2 still omit `io::BufError` and the
  `output`/`status` wait.** Round 1 N7 deferred these for budget, and the
  core has 4 tokens of headroom. Neither omission contradicts anything: core
  7.2's "Every capability operation that may suspend is a checkpoint" still
  holds, since `output` stays a checkpoint and returns `Cancelled` after its
  wait.
- **N2. Core 7.1's "scope exit and `join` wait, never cancelled (7.2)"
  reads awkwardly** (nit). 7.2 and D8 ("Joining a scope cannot be
  cancelled") carry the meaning.
- **N3. `trials/e1-dialect/programs/backup.rs` uses `fs.exists(p)` as a
  `bool`.** It is a record of a past trial and lies outside `design/`, so it
  needs no edit. Future trial graders should expect `exists(p)?`.
- **N4. Effect attribution of std-emitted records (for fx-djp).** SINK-9
  (v1) makes sink use an exercised effect. The text does not say whether
  `output`/`status` contribute `log` through the new `warn!`. The same
  question already applies to the unflushed-`BufWriter` warning and the
  evented runtime's 100 ms warning. Only v1 text would change.

## Reasoning

- **Codex B1 (fallible kill) is resolved.** 03 now says "tries to kill",
  waits whether or not the kill succeeded, logs the kill's error in a
  `warn!` record, and still returns `Cancelled`. It also explains why
  returning the kill's error at once would orphan the child. TASK-5 and
  companion §20 add the wait, which can last until the child exits. The
  `EPERM` condition matches `kill(2)`: the sender's real or effective UID
  must equal the target's real or saved UID, so the call fails only once
  the setuid child has changed both. B1 above is a gap inside this fix,
  not a reopening of it.
- **Codex B2 is resolved.** Core 7.2 now reads "like an uncancellable
  system call, are never interrupted, so deadlines are soft". That matches
  TASK-3's split between calls the kernel can cancel and calls it cannot,
  and FIFOs no longer fall on the wrong side.
- **Opus round-1 fixes.**
  - N1: TASK-3 now leads with the single-`write` contract and gives the
    readiness loop as "for example", with retries.
  - N2: the simulator sentence now covers "every rule of TASK-3". Simulated
    short writes were skipped as out of scope, which is acceptable.
  - N3: the "finish or frame" sentence now names the writes the
    cancellation may cut short.
  - N4: the `exists` gloss now says `NotFound`, and "the other main `Fs`
    operations" leaves out `dir` and `read_only`.
  - N5: the `Vec::with_capacity(n)` guidance is in 03.
- **No new contradictions.** I grepped `design/` for `exists`, `output`,
  `status`, `kill`, `warn!`, `read_owned`, `write_owned`, `BufError`,
  `written`, `write_all`, "uncancellable", "never interrupted", "cannot be
  interrupted", deadline, and soft.
  - The only `exists` callers in `design/` are the 03 §9 test (`.unwrap()`)
    and the FX0520 row (`?`), and core 6.3 lists `exists` under "as
    std::fs".
  - The `Receivers` note, the `output`/`status` note, and TASK-3/5/6 agree.
  - D8 is satisfied: a cancelled operation still returns `Cancelled`, and
    joins stay uncancellable.
  - §9.8 says real child processes are not simulated, so the kill path has
    no simulator obligation.
- **`warn!` is consistent and the right minimal choice.** SINK-1 and core
  6.5 make a record write-only, so it neither suspends nor makes a
  checkpoint. It has two std precedents: the unflushed-writer warning in
  TASK-6 and the 100 ms warning in TASK-5. It is emitted before the
  possibly unbounded wait, so it explains a hang while the hang is
  happening. Putting the kill error inside the returned error would explain
  the hang only after the child exits. Changing the error kind would break
  `is_cancelled()` and level-triggered handling.
- **Soundness against Rust and tokio.**
  - `BufError` copies `IntoInnerError`'s methods and its `From` impl, and
    its `Send + Sync + Pure` claim follows from core 2.2's `io::Error`.
  - `exists` is `std::fs::exists`.
  - Killing a child that the caller never sees matches tokio's
    `kill_on_drop` choice. A caller that wants the child to outlive the
    scope keeps it with `spawn`.
  - The single-`write` guarantee can be met by readiness backends and by
    io_uring with cancel (a completion that carries n wins).
- **Tier rule.** B1's fix touches 03 §1.1, which is v0 text this change
  edits, so B1 blocks. N1–N3 are nits or out of scope. N4 is v1-only and
  goes to fx-djp.
