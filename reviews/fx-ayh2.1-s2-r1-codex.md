Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate 412fc87 (code)

## Verdict

**Changes required.** The bounded profile and its implementation direction are sound, but the loader-variable gate is bypassable, real error paths lose child ownership, and some real/simulated lifecycle behavior violates the contract. Confidence: high in the source findings; the new failure scenarios below were not executed because this review was restricted to inspection and non-heavy checks.

## Scope

Reviewed `8b7e8f7..412fc87e6585459384e109437de6f3328589c9f6`, including all 25 changed files, their relevant callers, admission/export policy, handle classification, scheduler adapters, manifest recording/comparison, and the process tests. Stage 1's owning contracts are research/40, design/03 §1.1 and §9.9, companion 20, and PROC-1–3. Net/fx-ff13 is excluded.

Observed HEAD: `412fc87e6585459384e109437de6f3328589c9f6`. Observed SHA-256 of `git diff 8b7e8f7..412fc87`: `093d17e08423cd1229a3d133bf97b4458379a09148708eda1fff97c77180535d`, matching the brief. The tracked working tree was clean on inspection. The only review write is this report; no source, ledger, or commit was changed.

## Blocking findings

### B1 — An equals sign in an environment key bypasses glibc-variable refusal

**Location:** `crates/tide-std/src/process.rs:85–87,220`; real forwarding at `crates/tide-std/src/process/real.rs:81–85`.

`refused_variable` compares the entire supplied key against the glibc names. The pinned Rust backend does not reject `=` in a `Command::env` key: `CommandEnv::set` stores it, and Unix `construct_envp` concatenates `key + "=" + value`. The child's C environment interprets the bytes before the first `=` as the variable name.

Consequently, `proc.only(["git"]).command("git").env_clear().env("GCONV_PATH=/tmp/injected", "modules")` passes admission but sets the child's **GCONV_PATH** to `/tmp/injected=modules`. A directory with that legal name can supply injected conversion modules to a program using glibc's conversion machinery. Likewise, `.env("GLIBC_TUNABLES=glibc.malloc.check", "3")` sets the supposedly refused GLIBC_TUNABLES to `glibc.malloc.check=3`. This defeats the program-independent injection protection that stage 1 explicitly requires; it is separate from the documented program-specific environment limits.

Evidence: the pinned `nightly-2026-08-09` Rust sources, `library/std/src/process.rs` (`env`), `sys/process/env.rs` (`set`), and `sys/process/unix/common.rs` (`construct_envp`). A plain Python check of the candidate predicate and that serialization confirms both examples are accepted keys that serialize to refused variable names; no child program was launched.

**Suggested fix:** check the effective environment name—the bytes before the first `=`—against the refused set under `only`, before NUL validation or either backend lookup. Keep the refusal shared by both backends. Add a regression for an equals-bearing glibc key, including refusal before model lookup and before a later NUL error.

### B2 — Errors after a successful spawn can abandon a running child or zombie

**Location:** `crates/tide-std/src/process/real.rs:102–112,202–206`; `crates/tide-std/src/process.rs:398–430`.

After `std_cmd.spawn()` succeeds, a failed nonblocking ioctl returns through `?`, dropping Rust's `Child` without killing or reaping it. The same ownership loss occurs in `output`/`status`: `finish` cleans up only `Cancelled`, so another error from pipe draining, epoll creation/registration, pidfd creation, or socket-wait setup returns through `other => other`. Their local `Child` then drops, and the caller never receives a handle.

Concrete case: let `epoll_create1` fail with `EPERM` under an inherited syscall policy, or with descriptor exhaustion after concurrent allocations, while an `output()` child is running. The call returns an error and leaves the child running; after exit it can remain a zombie. A denied/failing FIONBIO ioctl reaches the equivalent problem in `spawn` itself. This differs from an explicit caller dropping a successfully returned `Child`, whose non-reaping behavior is intentional.

**Suggested fix:** retain cleanup ownership across all fallible work after OS spawn. On failure before returning a `Child`, and on any error from a convenience call that hides its child, close the pipe ends, attempt termination, and reap shielded while preserving/reporting the original error appropriately. Do not change ordinary `Child` drop semantics. Add an isolated injected-failure test that verifies the child is not left running or waitable after the error.

### B3 — Real waits skip level-triggered cancellation when status is already available

**Location:** `crates/tide-std/src/process/real.rs:139–154,201–212`; public entry points at `crates/tide-std/src/process.rs:352–384`.

The real `wait` calls `try_wait` and returns an available or cached status before checking cancellation. Its loop also allocates a pidfd before the first cancellation check. The simulated wait instead calls `begin` before checking status, as TASK-3 requires.

Concrete case: start and wait for a child, cancel a scope, then call that child's `wait()` inside it. Real execution returns the cached `Ok(status)`; simulation returns `Cancelled`. A child that exited before its first wait has the same problem. `wait_with_output()` also skips its checkpoint when there are no retained output pipes: `drain`'s loop never runs, then the cached wait succeeds. This is cancellation already present at call entry, not a wait that completed before a later cancellation.

**Suggested fix:** check cancellation at entry to every cancellable real wait/drain, including cached-status and empty-pipe paths, before allocating wait resources or reaping. Keep shielded cleanup exempt and preserve the existing rule that a completed in-flight operation retains its result. Test an already-exited/cached child under an already-cancelled scope on both backends.

### B4 — Closing simulated stdout/stderr does not close an inherited or null endpoint

**Location:** `crates/tide-std/src/process/sim.rs:265–275,676–687,780–785`.

Only piped streams initialize the open-end flags. `Close` changes `writer`, but `Write` handles `sink` and `null` before inspecting that flag. Thus a model such as `ProcessModel::new().close_stdout().write("after close\n").exit(0)`, run with inherited stdout, still emits `after close` and succeeds. With piped stdout the same transition correctly fails coverage as a write after closure; with null stdout it is silently discarded instead.

This makes endpoint closure depend on routing and lets the simulator report impossible post-close output. Ordered endpoint closure is part of the shipped lifecycle subset, not a Later feature.

**Suggested fix:** represent the child's open/closed endpoint state for every supported route and enforce it before sink/null routing, or reject the invalid transition during model validation. Add a close-then-write check for inherited and null output as well as a pipe.

### B5 — A model panic is discarded and its task is reported successful

**Location:** `crates/tide-std/src/task/proc.rs:99–106`; reachable arithmetic at `crates/tide-std/src/process/sim.rs:656–659`.

`spawn_model` ignores the result of `catch_unwind` and always finishes the scheduler task with `Outcome::Ok`. An accepted model can reach this: advance virtual time by one nanosecond, then spawn a model with `.delay(Duration::MAX, Duration::MAX)`. Validation accepts those ordered finite bounds, but `mc.now() + d` overflows. The panic is swallowed; no process failure is stored, the instance remains running, and a test that drops the child can finish successfully. A caller waiting for that instance instead gets a misleading deadlock/stall after its model task has disappeared.

**Suggested fix:** propagate unexpected model panics into the simulation's failure state, with the model/instance location, and wake affected waiters; never finish such a task as successful. Use checked time arithmetic with an explicit supported outcome for overflow. Add the accepted-model overflow regression so a caught internal panic cannot silently pass a seed.

### B6 — Explicit acceptance claims still lack checks that would catch their failure

**Locations:** `crates/tide-admit/tests/build.rs:3815–3826`; `crates/tide-std/src/process/tests.rs:189–191,347–358,483–487,507–516`; `crates/tide-std/src/process.rs:410–425`.

The new tests cover much of the witness table, but four required boundaries remain unproved:

- **Proc manifest replay and identity refusal.** Both new replay tests compare `sim.trace()` from ordinary executions. Neither records/replays a Proc run through the manifest harness nor changes its inline model/profile/schema/capacity data. Removing process registration from `replay_input` would leave these tests green. The existing generic manifest tests exercise other inputs, not this new registration. Research/40 §7 and §12 explicitly require the changed-model refusal witness and a failing omission control.
- **A failed descriptor sweep.** The descriptor test exercises a successful `close_range` only. A regression that ignores a failed sweep still passes on the current kernel. PROC-1 expressly requires spawn failure when the sweep fails. Use an isolated process that denies that syscall, and verify the error and that the target program never executes.
- **Canonical exec through a PATH element.** The test really spawns the absolute allowlist case, but its PATH case calls `lookup` and `canonicalize` directly. Removing canonicalization only from the bare-name spawn path would still pass. Start the isolated test process with the symlinked PATH, then actually spawn the bare name with the alternate `current_dir` and check the parent's target ran.
- **Shielded reap before cancellation returns.** `a_cancelled_call_after_spawn_kills_and_waits` checks only that no delayed output appears after it advances time another 20 ms. Killing the child and omitting the final wait/reap still satisfies that check. `NoCleanupWait` currently skips the entire cleanup, so it detects missing kill as well and cannot isolate missing wait. Make the control omit only the final wait, and assert termination/reap has completed before the call returns. Include cancelled `wait_with_output` retaining the caller's child, which the present `wait`-only witness does not exercise.

**Suggested fix:** extend the existing focused tests at those boundaries; no broad test framework is needed. These are concrete stage 1 acceptance requirements and trust/lifecycle regressions, rather than requests for general coverage.

### B7 — The companion still records the old reference count

**Location:** `design/reference-companion.md:17–19`.

The candidate changes the core reference but leaves its companion's count at **24,961**, dated 2026-11-17, despite the immediately following instruction to update it after every core change. On this frozen candidate, `scripts/count-tokens.sh design/reference.md` exits 0 and reports **24,979**. The D14 budget passes, but its durable count is stale.

**Suggested fix:** record 24,979 and the recount date on the companion line, updating it again if the core changes in the fixes. This is the requested reference/companion synchronization leftover; it needs no implementation change.

## Non-blocking findings

None beyond the blocking correctness, proof, and count issues above. No alternative API, dependency, or expansion into signals, PTYs, nodes, or Net is needed.

## Reasoning

The shared admission gate otherwise implements the chosen order: program allowlist, explicitly set refused variables, then NUL, before real lookup or model lookup; `output` and `status` first check cancellation, while `spawn` does not. Invalid allowlist entries grant nothing and repeated `only` intersects exact OS-string spellings. Bare-name resolution uses the parent's PATH, skips empty/relative elements, has no default PATH, and ignores the command's own PATH. Canonicalization precedes child `current_dir`, and `arg0` retains the requested spelling. The relative-program exception is limited to unrestricted Proc as designed. The env-key bypass in B1 is the trust-boundary exception to these conclusions.

The new public resource handles are all present in `NOT_PURE`. I found no new public backend constructor or unguarded `__rt` route to spawn without Proc/a derived command. PROC-3's current guard includes public-use lines and separately fixes the literal four-name Rust process re-export; the present exports expose no ambient constructor. The ambient Command::new conformance expectation remains TD0501 alongside E0599. The trust diff prioritizes items gaining proc authority and labels the child ambient, without claiming exercised authority.

For successful real spawns, the pre-exec hook is a raw close_range syscall plus errno capture, with no allocation or fcntl fallback. The pinned Rust implementation declines posix_spawn when pre-exec hooks exist, sets up stdio before the hook, and reports/reaps pre-exec failure through its own error channel. Existing Rust wait/status caching protects repeated successful waits and kill after reap. Owned descriptors close on ordinary unwinding; the important missing resource ownership is the child itself on the error paths in B2. Pipe reads/writes retry EINTR, and shielded std wait retries interrupted waitpid. Concurrent stdout/stderr draining avoids the ordinary full-pipe deadlock.

The simulator uses the existing scheduler and separate process RNG stream, with the declared 65,536-byte capacity, `proc-lifecycle-v0 schema=1`, exact registered spellings, and root seed. Registration journals model data inline. Its ordinary missing-program, early exit, EOF/buffer retention, kill/normal-exit race, short transfers, backpressure, and broken-pipe cases have focused witnesses and controls. The added scheduler state does not itself consume draws when no child is spawned. B4/B5 and the missing B6 checks prevent treating the lifecycle/replay contract as fully proved.

Verification observed, without starting a build or running compiled Tide code:

- Candidate identity and diff SHA-256 match the brief; `git diff --check 8b7e8f7..412fc87` passes.
- The token-count command passes: 24,979 / 25,000.
- Read `target/work/suites/candidate.txt` and the completed candidate workspace log/exit file: candidate 412fc87; exit 0; 417 passed, 0 failed across its result lines.
- The candidate conformance log was still growing and had no exit/end record at the last inspection. Its completion is not claimed here. The pre-rebase f2f908b logs show exit 0 and 390 cases, 387 PASS, 3 AWAITS CHECKER, 0 FAIL; those are prior-revision evidence only.

No heavy checks, failure-injection runs, or new executable probes were performed. The suite successes do not exercise the failure scenarios identified above and do not override them. After the fixes, re-check these findings under the Merge Rule; because B1 changes a trust boundary, apply its fresh-review requirement before the later Opus round.

DONE
