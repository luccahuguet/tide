Model: Claude Opus 5.5 · Date: 2026-11-18 · Re-check 2 8882063

# fx-ayh2.1 stage 1, re-check 2 of s1-r2 (R1 only)

- **Candidate:** `888206374b5beb2aeee25d4d9c174502fd96dab0`, which amends `7eac384`.
- **Round diff:** `git diff 7eac384..8882063`, SHA-256 `3e15477942c2c64a41b22c94dcc8d7024ab8b52b4af97fc88c214c90e640b885`. It touches `conformance/rules.md` and `research/40-proc-command-v0.md`.
- **Full diff:** `git diff 91a2f36..8882063`, SHA-256 `718503f70aea1867f3c64af9621c3330e535a1b656c9acd2a6acbfa124abf20c`.

## Verdict

**Approve.** R1 is fixed.

## R1 status

**Every outcome I named now has a row, with a control that fails for that outcome:**

| Outcome named in R1 | Row in research/40 §12 | Control, and why it fails |
| --- | --- | --- |
| `output`/`status` cancelled before the spawn | 03 §1.1, before the spawn | "Start the child anyway": the witness requires that nothing starts. |
| `output`/`status` cancelled after the spawn: kill, close the ends, shielded wait, `Cancelled` | 03 §1.1, after the spawn | "Return `Cancelled` without waiting, or leave the child running": the witness requires the shielded wait and the kill. |
| A wait that completes before the cancel takes effect returns its own result (TASK-3) | 03 §1.1, TASK-3 | "Return `Cancelled` after the wait completed": the witness requires the call's own result. |
| Cancelled `wait`/`wait_with_output` leaves the `Child` with the caller | 03 §1.1, cancelled waits | "Consume or drop the `Child`": the witness requires the caller to still `kill` and `wait` it. |
| Parent write after the model closed its input returns `BrokenPipe` | §6, parent `BrokenPipe` | "Let the write succeed silently". |
| A full buffer blocks the writer (backpressure) | §6, backpressure | "Use an unbounded buffer": with no limit, no write ever blocks. |
| `wait` closes the retained stdin; `try_wait` doesn't | §6, stdin on wait | "Close stdin in `try_wait`, or leave it open in `wait`". |

**Ownership.** The table's intro now says it owns the set and that PROC-1 refers to it. PROC-1 replaced its restated list with "every outcome in research/40 §12's seed-witness table has a seed witness and its broken control fails". One owner holds the set, and the rest of the row is unchanged.

DONE
