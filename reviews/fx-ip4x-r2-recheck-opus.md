Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 43284e8

# fx-ip4x — Opus finder re-check of B1, N2, N3, N4

I reviewed `git diff 898c8ab 43284e8`. I checked the full diff's SHA-256 against the expected value, ddb52ed278e9f3052b04a5b2e426936a36bf89b0c7ff0d4614d83f932e6429c3, and HEAD is 43284e8. This was read-only: I ran no cargo commands and started no subagents.

## Verdict

approve. All four fixes hold. The fixes introduce no new blocking problem. One non-blocking note follows under B1.

## B1 — the child outlives the driver: fixed

The fix is at `sandbox.rs` `install`:
- It calls `prctl(PR_SET_PDEATHSIG, SIGKILL)` right after `setsid` and before the core limit, `close_range`, Landlock and seccomp. `prctl` therefore runs before the filter that would restrict it.
- It then checks `getppid() != parent` and calls `_exit(1)` if the parent changed. This closes the race where the parent died before the `prctl`.
- `parent` is `std::process::id()`, captured in `apply`. That is the driver process that forks, and `getppid` returns its tgid even when a non-main thread forks.
- `prctl`, `getppid` and `_exit` are all async-signal-safe, and nothing allocates.
- The death signal survives `execve`, because the test copy is not set-user-ID and has no file capabilities.
- The signal fires when the forking thread exits. That thread is the one that waits on the child in `supervise`, so it lives as long as the child.
- Landlock's signal scope does not block it. The scope limits only signals sent from inside the domain, and here the kernel delivers the signal on behalf of the driver, which is outside the domain.

**Test `run3_child_dies_with_the_driver`.**
- It swaps in an unadmitted program that prints its pid and spins.
- It starts the driver with `process_group(0)` and kills that whole group with SIGKILL, the same way `timeout` or a terminal interrupt would. The group includes the outer driver and the inner driver it re-executes. The inner driver is the sandboxed child's parent.
- It then waits up to 10 s for `/proc/<pid>` to disappear or show a zombie.
- **Without the fix it fails:** the child sits in its own session, so the group kill misses it. It keeps spinning, `gone()` stays false, and after 10 s the test panics with "outlived its driver".
- **The hang guard kills the child:** on that failure path the test calls `kill(child, SIGKILL)` before it panics. On the other path, where no pid arrives within 120 s, it kills the driver's group. There the child has either not started yet, or it has the death signal, since the pid line is printed only after `exec`.
- The pid parsing and the zombie check are correct. I see no flakiness risk beyond the usual pid reuse within 10 s, which is negligible.

**Non-blocking note:** the README now says the child "dies with the driver". That holds for the process that spawns it, and for any signal sent to the process group. It does not hold for a SIGKILL sent only to the outer pid, because `tide-admit` always re-executes itself (`invocation.rs` `ICE_VAR`).
- Examples: the `bounded()` hang guard, and the inline guard in `run3_stalled_stderr_does_not_hold_the_limit`, both call `child.kill()` on the outer process only.
- In that case the inner driver survives, and so does its child. The inner driver still enforces the simulation limit.
- This is how the code behaved before this change, and the fix did not introduce it.
- A follow-up could set `PR_SET_PDEATHSIG` on the inner re-exec as well, or reword the README. It does not block.

## N2 — publish deleted operator files at `OUT/target` and `OUT/target/sim`: fixed

`subdirectory` no longer takes a `replace` flag. If the path is a symlink (ELOOP) or a regular file (ENOTDIR), it now returns an error that names the entry and says to move it away. Nothing is unlinked anywhere.

All three callers were updated: `.tide-cache`, `sim`, and `target`/`target/sim`. The build cache already rejects a symlinked `.tide-cache` earlier, so that path behaves the same.

The README, the doc comment, and the denial test now agree:
- the test asserts that the planted link is still a symlink;
- it asserts that stderr carries the new message;
- the elsewhere directory is left untouched;
- the third run removes the link and now asserts that the run succeeded. That also covers my earlier N7 point that this run's status went unchecked.

No new problem. When publication fails, the error replaces the run's result and the run directory is still removed, so that run's manifests are lost. The README documents this, and it happens only when the operator has placed a link or file there.

## N3 — stale tide-std doc: fixed

`crates/tide-std/src/lib.rs` now says:
- run directly, only `#[sim_test]` isolates its run in a child process, without an OS sandbox;
- under the driver's run modes, the test runs in RUN-3's sandboxed child, with `#[sim_test]` in process;
- neither mode gives RUN-5's guarantee.

That is accurate. The sentence at `conformance/README.md:464` is unchanged and still true for the conformance harness. This is a text-only fix and needs no further re-check.

## N4 — tide-admit no longer built off x86_64 Linux: fixed

`mod sandbox` is now compiled only under `cfg(all(target_os = "linux", target_arch = "x86_64"))`. Other targets get an inline stub:
- `RunDir::create` returns an error saying the runs need an x86_64 Linux host (`sandbox: none`);
- the other items are no-ops.

The stub matches every item `replay.rs` uses: `RunDir.path`, `create`, `token`, `publish`, `remove`, and `Sandbox::prepare`, `level` and `apply`. The rest of `replay.rs` uses only portable `libc::write` and `std::os::unix` APIs.

The README says the run modes are unavailable off x86_64 Linux, as linking already is. That is honest, and it matches the existing `--emit=link` gate.

I could not compile it for those targets, since I had no cargo. This is a reading-based check.

DONE
