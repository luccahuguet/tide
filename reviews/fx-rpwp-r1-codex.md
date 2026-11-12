Model: gpt-6.1-sol · Effort: high · Date: 2026-11-12

Verdict: request_changes

Reviewed frozen candidate `7070c0237bdf27069a7cac93667648b3f32bde7a` against `dde6c57aff0f037e29bd20c6bb600e308bad19c2`. SHA-256 of `git diff dde6c57..7070c02`: `4a8ff0f9972631aff77bf884ab31e90e1e7f80035e1d7ae53a6944665d4635e6`. Confidence: high in the findings below; live isolation and runtime Acceptance remain unproven. This tree is not Codex-clean for the subsequent Opus round.

The change adds the external writer launcher, argument-free MCP broker, persistent coordinator, identities, canaries, store scan and scripted dry writer. The division of responsibilities is generally sound, but the following checks and lifecycle transitions need correction.

1. **Blocking — The tool-surface probe accepts ordinary tool errors as proof of confinement.**

   Evidence: `trials/e1-checked/coordinator.py:94–96`, `:393–400`.

   A forbidden Read is counted as denied whenever its tool result has `is_error: true`. A forbidden Write receives the same treatment. Neither the probe nor its audit exception verifies that the request was valid or that the error was an access denial. For example, outside Reads with an invalid offset and an outside Write missing `content` satisfy these predicates even when no filesystem restriction was exercised. Successful inside controls and the exact init tool list do not prove that the same file tools reject valid outside requests.

   A Python-only control supplied these validation errors and successful inside controls to `probe()`; it wrote `probe.json` with `passed: true`. This makes the required pre-gate confinement proof pass open.

   Fix: require complete, schema-valid forbidden requests and verify the pinned CLI's actual permission-denial result. Reject missing, malformed and unrelated failures. Add a regression where invalid-argument errors cannot satisfy the probe. Keep the live valid-request probe mandatory.

2. **Blocking — Exhausting 20 runs does not end the writer session.**

   Evidence: `trials/e1-checked/coordinator.py:169–177`, `:209–215`; `trials/e1-checked/harness.py:445–447`; contract: `trials/e1-checked/protocol.md:103–108`.

   The monitor ends a writer at the time limit or CLI completion. It never observes exhaustion of the run budget. The harness only refuses subsequent commands. A writer can therefore make its twentieth call, continue editing through Read/Write/Edit, and submit a different final program later. That later program is frozen and graded. The protocol says the session ends when its budget runs out, and defines the final snapshot at that end.

   Fix: have the harness expose authoritative run-budget exhaustion and have the coordinator terminate the writer when the last permitted command finishes, then capture the final snapshot. Keep the numerical budget policy in the harness. Test that edits attempted after exhaustion cannot enter the final snapshot, while the twentieth command's results are retained.

3. **Blocking — Grading infrastructure failures are counted as solution failures without the required retry.**

   Evidence: `trials/e1-checked/coordinator.py:254–262`, `:274–277`, `:347–351`; `trials/e1-checked/harness.py:239–240`, `:598–599`.

   `freeze_and_grade()` records any grader exit, including setup exit 2 and its own 600-second timeout mapped to 2, without setting a fault. `run_slot()` then marks the trial complete with no rerun and failed measures. Separately, a Bubblewrap setup failure represented by `infrastructure: true` inside a grade becomes ordinary grade exit 1. Thus a temporary jail/setup failure after an otherwise normal writer can lower an arm's measured pass rate as though the submitted program were wrong.

   A Python-only control returning setup exit 2 from both grading processes produced `complete: true`, `reruns: 0`, `failed_infrastructure: false`. The existing retry tests mock grading and do not cover this transition.

   Fix: propagate grading setup/guard failures distinctly, including infrastructure flags in grade records. Do not finalize a mechanical solution failure from them. Route infrastructure faults through the recorded single-fresh-attempt policy, withholding the faulted attempt's grades from scoring; a second infrastructure fault fails the trial. Add coverage for setup failure inside a grade and grader exit/timeout failure.

4. **Blocking — Interruption recovery can start a fresh attempt while the old writer is still alive.**

   Evidence: `trials/e1-checked/coordinator.py:159–160`, `:192–198`, `:308–317`, `:369–370`.

   Writers start in separate sessions, but their process identities are not persisted and recovery never checks or terminates them. If the coordinator process dies on the same boot, its writer can survive; resuming consumes the saved interruption marker and launches attempt 2 without reclaiming attempt 1. This can leave an unmonitored writer and broker running past the coordinator's deadline and outside its recorded jobs limit. Ordinary Ctrl-C also reaches the main thread waiting on the executor, not `run_writer()` in its worker thread, so its local interruption handler does not cancel active writers promptly.

   Fix: make coordinator shutdown cancel and reap active writer groups, and make same-boot recovery prove the previous attempt is stopped before rerunning it. A parent-death guard or persisted, safely verified process identity can supply that guarantee. Add a process-lifetime regression for coordinator interruption and resume. This finding is established by code inspection; the synthetic process-lifetime control was blocked by the command guard and was not executed.

5. **Blocking — The tmpfs mount hides runtime executables for an accepted run directory under `/tmp`.**

   Evidence: `trials/e1-checked/harness.py:207–210`, `:337–339`, `:354–356`; accepted directory boundary: `trials/e1-checked/coordinator.py:431–432`.

   Read-only executable binds are added before `--tmpfs /tmp`. For a valid external run such as `/tmp/e1-run`, the executable lives below `/tmp` and that later mount covers its bind. The subsequent writable bind restores only the execution scratch directory, which is a sibling of the executable's output directory. The test executable is therefore inaccessible. Compilation can work while runtime setup fails; canaries under the checkout's STATE path do not exercise this layout.

   A Python-only command-construction control with an existing temporary executable confirmed the ordering: executable bind, replacement of `/tmp`, execution scratch bind.

   Fix: mount tmpfs `/tmp` before adding read-only inputs beneath it. Add a regression covering an external session under `/tmp`, including the separate executable and runtime scratch paths, and verify execution when machine scheduling permits.

The report's technical choices beyond decisions 1–9 were reviewed. Generic dry controls respect the held-out constraint and can produce deliberately failing grades; monotonic timing, honoring the operator's Cargo job count, one effective diagnostic cap, retained store mount pending its experiment, and fixed initialization options are reasonable. Uniform tmpfs, the separate Bubblewrap status channel, and threaded recovery need the corrections above. No additional abstraction or dependency is needed to address these findings.

Verification: the same 31 non-compiling tests passed independently (Python 3.14, `PYTHONDONTWRITEBYTECODE=1`, `CARGO_BUILD_JOBS=4`, `timeout 60s`, `nice -n 19`, closed stdin). Additional Python-only controls confirmed findings 1, 3 and 5. The research body is byte-identical to `/home/lucca/.cache/eon-agents/tide-research-gate/report.md` after the added header. The protocol diff changes only the authorized Owner remap, §10.2(1), and §10.2(3). `readiness.md` is unchanged. The fx-m72.3 writer runner and exact-segment acceptance filter remain intact. The tracked working tree stayed clean; no code, ledger, staging or commits were changed.

Not checked: builds, Bubblewrap execution, actual canaries, the no-store experiment, dry runs or a live Claude probe; these remain machine-gated. The command guard also rejected the synthetic process-lifetime control. Isolation and runtime Acceptance must not be inferred from the passing Python tests.
DONE
