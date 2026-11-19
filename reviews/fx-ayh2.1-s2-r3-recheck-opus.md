Model: Claude Opus 5.5 · Date: 2026-11-18 · Candidate 08a80d9 (re-check)

# fx-ayh2.1 stage 2, s2-r3 finder re-check (Opus)

## Verdict

**Approve.**
- B2 is resolved.
- The B2, N1 and N2 code changes introduce no defect within their own changes.
- B1 and N3 are confirmed.
- N4 and N5 are applied as decided.
- One wording nit is noted under N1; it does not block.

## Scope

- HEAD `08a80d9b2cceec506d8b2a5401d6376dbcfcf0c1`; tree clean.
- Round diff `git diff 4c88de9 08a80d9`: 8 files, +76/−18, SHA-256
  `0f2cb970dd83051f756280222e4e8cf3d8361c519deb31004afe46e46ad936b6`.
- I read the whole round diff, the current `sim.rs` `try_wait`/`kill`, the drain paths in
  `real.rs`, and report.md "Fixes s2-r3".
- I re-checked only my own findings. I ran nothing.
- Suites on this candidate:
  - When I looked, `target/work/suites/` held `candidate.txt` (08a80d9), `test.start` and
    `test.log`, with no exit files yet.
  - So I have not observed the new tests passing. This approval is by inspection and depends
    on those suites exiting 0.

## B1: companion §14.2 "(planned)"

**Confirmed.**
- `design/reference-companion.md:86` now reads `` `proc.command("git")` `` with no label.
- `proc.exit(1) (planned)` on line 88 stays, which is correct.
- The change is text only.

## B2: simulation-identity check in sim `try_wait` and `kill`

**Resolved.**
- `sim.rs:398` (`try_wait`) and `sim.rs:451` (`kill`) now call `self.ctx()` first, before the
  instance is locked, read or changed.
- `self.ctx()` is `Tasks::proc_context`, which asserts the caller's live simulation, the same
  check `Pipe::read`, `wait` and `drain` use.
- Both cases from my report are closed:
  - a cached status is no longer returned in another simulation or outside any;
  - `kill` can no longer set `st.kill` before panicking.

**Nothing new inside the change.**
- Within its own simulation, `ctx()` is a check with no scheduling point, draw or trace record,
  so traces and seeds are unchanged.
- `finish`'s cleanup `kill` always runs in the caller's live simulation, so it still passes.
- `try_wait` calls `ctx()` again later; that repeat is harmless.

**Regression test.** `a_pipe_from_another_simulation_is_refused` adds a child from S1, waited so
its status (code 3) is cached in the `Child`.
- In S2, `try_wait` and `kill` must each panic with "simulation handle used outside its
  simulation".
- Without the fix, `try_wait` returns `Ok(Some(..))` and `kill` returns `Ok`, so the test fails.
  It is a real regression test.

## N1: reachable `unreachable!` arms

**Applied; no defect.** All three arms now panic with the existing identity text plus the
backend that did not match.
- **`real.rs:80`, simulated `Stdio` on a real `Command`.**
  - `real::spawn` calls `outside_simulation()` first, so inside a live simulation the "real
    capability used inside a simulation" panic still wins.
  - Outside one, the handle really is outside its simulation, so the message is accurate.
  - No child has started yet. A slot-0 `Stdio` already built is dropped on unwind, so no
    descriptor leaks.
- **`real.rs:231`, simulated end in a real drain.** It panics after the ends are taken from the
  caller's `Child`. That `Child` stays with the caller. The `output()` and `status()` paths
  cannot hold such an end, so `finish` never meets this panic.
- **`sim.rs:496`, real end in a simulated drain.** This arm fires only inside the live
  simulation, because `ctx().begin` runs first.
  - **Nit (non-blocking, text only):** strictly this is a real handle inside a simulation. The
    house message for that is "real capability used inside a simulation", not "simulation handle
    used outside its simulation". It changes no behavior.
- **`mixed_process_backends_are_refused`.** It covers all three arms.
  - Both real children are waited before their ends move, so the shared test process is left
    with no zombie.
  - The `true` it starts needs only `PATH`.

## N2: `cancellable` parameter on `drain`

**Applied; no defect.**
- Both call sites passed `true`.
- The simulated drain now always runs `self.ctx().begin(..)` and waits with `cancellable =
  true`, which is exactly the old behavior on every path.

## N3: kernel and syscall wording

**Confirmed.**
- research/40 §5 now says `ENOSYS` before 5.9 and `EINVAL` for `CLOSE_RANGE_CLOEXEC` on 5.9 and
  5.10. That is accurate: `close_range` arrived in 5.9 and the flag in 5.11.
- The comment at `real.rs:18` now names x86_64 and aarch64.

## N4 and N5 (outside the re-check request; noted)

- **N4.** `authority/proc.rs` adds `only(["/bin/true"])` refusing `/usr/bin/true`.
  - The refusal comes before lookup and before any model lookup, so it needs no model or file on
    either backend.
  - The result goes into the shared OBS line, so the build test compares it across the two
    backends.
  - tide-lead's acceptance decision on the symlink case is recorded on the bead.
- **N5.** `("tide_std::process::Signals", PROC)` is removed from `NOT_PURE`. No other reference
  to it remains in `crates/` or `conformance/`.

DONE
