Model: Claude Opus 5.5 · Date: 2026-11-18 · Re-check 7eac384

# fx-ayh2.1 stage 1, re-check of s1-r2 (B1, N4, N5)

- **Candidate:** `7eac384e44456a2ac61daf011c23535e493b6f23`, which amends `107688e`.
- **Round diff:** `git diff 107688e..7eac384`, SHA-256 `07b9c51822855d3fdbd7e742a55526f0466fb242ae3ba2ad09ed5fcec26127aa`. Full diff `git diff 91a2f36..7eac384`, SHA-256 `cabf7175c0db37475a9cc02f484dc460d493c01718e237ab68f9b444f902984f`.
- **reference.md:** unchanged in this round, still 24,985 tokens.
- **Scope:** B1, N4 and N5 in 03, research/40, PROC-1 and the companion, plus a read of the text-only fixes. Nothing was compiled.

## Verdict

**Changes required.** One item. B1 and N4 are fixed and consistent everywhere. N5's new table leaves out several outcomes of the subset, so it doesn't yet meet the bead's "every permitted finite outcome" bar (R1 below).

## Item status

| Item | Status | Notes |
| --- | --- | --- |
| **B1** | **Fixed** | research/40 §12 now moves `planned` onto `exit`, using the `Net` row's pattern, unless stage 2 builds `exit`. It also lists every label that describes the profile as not built: reference 6.8, the companion §20 heading, 03 §9.9's "None of it is implemented yet", README.md:230, :247 and :431, research/40's Status line, and the `Proc` doc comment in tide-std. It ends with the recount. I checked that each cited README line exists with that text. |
| **N4** | **Fixed** | The rule is the same in 03 §9.9's v0 header, research/40 §6, PROC-1 and companion 20. In v0, a simulated status with no code comes only from `kill` (when termination runs first) or the default broken-pipe death. A model can also declare `BrokenPipe` and continue, and can declare no other signal death. `code()` is `None` and `success()` is `false`. `Display` is Rust's Linux text on both backends, and no v0 method returns the signal number. This is correct: Rust's Unix `ExitStatus` prints `signal: N (SIGNAME)`; Rust's `Command` resets SIGPIPE to its default in the child, so real children die by SIGPIPE by default; and Rust programs ignore SIGPIPE. It doesn't conflict with the Later body of §9.9 ("Models may declare death by any signal number"), because the v0 header scopes it. |
| **N5** | **Changes required (R1)** | The table and PROC-1 match row for row, and each control fails for its own outcome. But the set is incomplete. |
| B2, B3, N1-N3, N6-N9 (text-only) | Applied; nothing plainly wrong | `Signals` (Later) appears in 03 §0, 03 §1.1 and 02-memory. A1 is attributed to the owner. The kernel floor is now in 03, the companion and §9 Cost. With no start-time `PATH`, a bare name returns `NotFound` (03, companion, research/40). The writable-`PATH` limit is in 03 and the companion. Option 2's third reason is removed. |

## R1 (blocking): the seed-witness set leaves out outcomes the v0 subset permits

research/40:487-509 calls the table "the set" and says each outcome in it has a witness. PROC-1 repeats it. But research/40 §6 keeps several of the subset's outcomes in v0 on both backends, and neither the table nor PROC-1 names them:

- **Cancelled `output` and `status`** (§6: "follow 03 §1.1"; 03:398-421). A cancel before the spawn starts nothing. A cancel after it kills the child, closes the pipe ends, waits shielded and returns `Cancelled`. A wait that completes before the cancel takes effect returns its own result (TASK-3). These are the outcomes that interact with the kill-versus-exit race, and that race is why `kill` is in v0. Broken controls: return `Cancelled` without waiting; leave the child running; or return `Cancelled` after the wait completed.
- **Cancelled `wait` and `wait_with_output`** leave the `Child` with the caller (03:380-388). Control: consume or drop the `Child`.
- **Parent-side pipe outcomes** (research/40:331-334). A parent write after the model closed its input returns `BrokenPipe`. A full buffer blocks the writer (backpressure). Controls: succeed silently, or use an unbounded buffer.
- **`wait` closes the retained stdin, `try_wait` doesn't** (research/40 §6, Lifecycle). The model sees input closure only after `wait`. Control: close stdin in `try_wait`, or leave it open in `wait`.

**Minimum fix:** add these rows to research/40 §12's table, each with its control. In PROC-1, replace the restated list with a reference to that table ("every outcome in research/40 §12's table has a seed witness and its broken control fails"). Then one owner holds the set (AP-OWNERSHIP-001), and the two lists can't drift apart.

DONE
