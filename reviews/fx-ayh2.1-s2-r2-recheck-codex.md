Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Candidate a353639 (re-check)

## Verdict

**Changes required.** B1 and B3 are resolved by inspection with appropriate regression tests. B2 fixes the reported same-simulation field-transfer case, but its new endpoint-reading path bypasses the endpoint's simulation identity. Resolve that defect within B2 before approval.

This is a finder re-check only of `6308c95..a353639` and the directly relevant code/tests, not another full review. Observed HEAD: `a3536393519d3804f5164c45d490c5f00a7e123b`; tracked tree clean. SHA-256 of `git diff 6308c95 a353639`: `f18dbf64bd8bb66a568283029996716675b93db0e341a3b5eaafe41748e33d61`. `git diff --check 6308c95 a353639` passes.

The suite directory identifies this exact candidate and a start at 2026-11-18 18:12:20Z, but `test.log` was empty and there were no completion/exit files at inspection. Thus completed candidate suites and the builder's claimed focused/mutant results are not independently observed here. No heavy runs or executable reproductions were started. This report is the only review write; no implementation, ledger or commit changed.

## B1 — Failed pipe setup retains unprocessed endpoints

**Resolved.** `crates/tide-std/src/process/real.rs:133–139` now takes and drops all remaining Rust child endpoints before termination and the shielded wait. Already-extracted endpoints unwind out of the setup closure, so the cleanup path no longer retains the unprocessed stderr reader that caused the reported deadlock. It preserves the original setup error and existing reap behavior.

`real_a_failed_setup_closes_every_pipe_before_reaping` (`process/tests.rs:685–709`) targets the failure: setup is injected to fail at stdout, both kill syscall routes are denied, the child writes enough stderr to fill a retained pipe, completion is bounded at 20 seconds, and ECHILD verifies no waitable child remains. The extracted seccomp helper also preserves the existing sweep test's denial behavior. I found no new defect in this fix or its test by inspection; execution remains unverified in this pass.

## B2 — Simulated drain ignores the moved pipe's owner

**Partially resolved; one blocking defect inside the fix.** The drain now selects each supplied endpoint's instance and slot and registers a shared readiness signal across the actual open sources. It continues servicing both outputs together. The conformance addition (`conformance/cases/authority/proc.rs:21–28`) waits for both children, moves B's stdout onto A, and compares the resulting bytes/status through the existing real/simulated harness. This correctly targets the original deterministic mismatch without introducing the earlier SIGPIPE race.

However, **the new direct endpoint access omits its simulation-identity check** at `crates/tide-std/src/process/sim.rs:490–494,505–507,529–532`. Only the receiving child's `self.ctx()` is validated. The supplied pipe's `tasks` is never consulted before its buffers are read or changed.

Concrete failure: in simulation S1, spawn a model with piped stdout, wait for it to exit with buffered bytes, and retain `child.stdout.take().unwrap()` outside S1's scope. Then start S2, spawn/wait a child there, assign that old pipe to its public stdout field, and call `wait_with_output`. The new drain consumes S1's buffered bytes and returns them successfully inside S2. Reading the exact same pipe through its ordinary `Read` implementation instead rejects it as `simulation handle used outside its simulation` (`process/sim.rs:562–575`, `task/proc.rs:17–18`, `task.rs:521–524`). The existing `simulated_handles_require_their_own_live_simulation` test records this runtime invariant. The fix has added a path that uses an ended simulation's resource under another simulation's scheduler and random stream.

**Smallest suggested fix:** validate every extracted simulated endpoint against the current live simulation before inspecting/registering/mutating its state, using the existing `p.tasks.check_simulation()` or equivalent context validation. This requires no extra scheduling point or abstraction. Retain the current per-endpoint instance/slot draining. Add a regression retaining a buffered pipe from S1 and attaching it to a child in S2; both direct reading and `wait_with_output` must reject that pipe. The new same-simulation conformance scenario cannot detect this missing guard.

This finding is confined to the new B2 drain implementation; no unrelated surface was reopened.

## B3 — Killed model records a second normal exit

**Resolved.** `crates/tide-std/src/process/sim.rs:669–678` now checks the existing status under the same instance lock used for the transition and returns before recording a normal exit if termination already won. The status and terminal event therefore agree. Recording after releasing the resource lock introduces no scheduling handoff in this code path.

`a_killed_instance_has_one_terminal_event` (`process/tests.rs:715–733`) covers the implicit-default-exit race over seeds 0–63, including the source-derived seed 17 from the finding. It requires exactly one terminal event matching the collected status and witnesses both normal exit and SIGKILL. I found no new defect in this fix or its test by inspection; execution remains unverified in this pass.

DONE
