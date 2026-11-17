Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 343df36

# fx-nkv.8 Opus r2 re-check

I re-checked only the fixes for B1 and N1–N6, in `git diff abc4b31 343df36`. I confirmed that the full diff 5029846...343df36 has SHA-256 3fae99e42c2b59a6ae0d2f1f2c29c3bfe9132b45d81051a3b16eb95d7c54c262. This was read-only, and I ran nothing.

## Verdict

approve. Every finding is fixed. The fixes add no new blocking problem. One new nit is under N3.

Proof note for the coordinator: `demo/run.sh` changed, so the earlier exit-0 runs (demo-c2, run-final-r1) do not cover 343df36. A fresh `demo/run.sh` run on 343df36 is needed before landing (AP-PROOF-001).

## B1: fixed

**Step 1, every failing seed must have the lost-hits cause (`demo/run.sh:89-101`).**
- How the check works:
  - The new awk script starts a block at each `simulation failure seed=N test=every_view_is_counted` line.
  - It takes the first `  cause: ` line after that line as the seed's cause.
  - It counts the seed as a lost hit only when that cause contains `failed: lost hits`.
  - Both counts must equal `failed` from the summary line.
- The order of lines matches the runner. `runner.rs` prints the failure line, then the trace, input and shrink lines, then `  cause:` (`:443-470`).
- Trace lines are NDJSON, so none of them starts with `  cause: `. A multi-line `assert_eq!` payload puts `lost hits` on the cause line itself.
- Each kind of wrong failure now fails the demo with exit 1:
  - A self-check failure has the cause `determinism self-check failed`, which does not match.
  - An `unwrap` panic on a `TaskError` or `LockError` does not match.
  - So does a non-string panic.
  - A failure block with no cause line before the next failure line is never counted as lost, so `lost < failed`.
- The counts are checked too. `reported == failed` catches a summary that disagrees with the failure lines. An empty `failed` from a parse failure also fails the check.
- If awk prints nothing, `read` fails under `set -e`. That still exits 1, just without the `fail` message. This fails closed and is acceptable.

**Step 2, the recorded cause (`:112-114`).** The first `  cause:` line of `record.log` must end in `failed: lost hits`. If no cause is reported, the check fails.

**Step 2, the recorded output (`:136-139`).**
- The views and hits come from the recording's sink output, which is what the test printed, through the regex `views served: N, hits recorded: M`. The check then requires `hits < views` as an arithmetic comparison. So it really compares hits with views.
- A malformed or missing line fails closed:
  - No match: `read` fails, the `|| true` absorbs that, `hits` stays empty, and the `-n` test fails.
  - Empty digits: `read` drops leading spaces, so `hits` ends up empty and the test fails.
- `set -u` is safe because the check uses `${hits:-}`.

## N1: fixed

`demo/agent-card.md:39` now reads "simulated network I/O beyond TCP loopback", which matches the guide and `net.rs`.

## N2: fixed

`demo/README.md:112-114` now says `&Tasks` is `serve`'s only capability, and that it can still write to the ambient `out`/`err` sinks. That is accurate (`crates/tide-std/src/lib.rs:15`).

## N3: fixed, with a new nit

The timing now says "about 20 seconds", which matches the builder's measurement of about 18 s. The guide now says the trust diff prints to the terminal.

New nit: "The logs of every build and test run are in `tide-demo/logs/`" is still slightly broad. The two `cargo build` steps, for the driver and std, are not logged there. Their output goes to the terminal or is captured in a variable. "every driver build and test run" would be exact. This is text-only and does not block.

## N4: fixed

The agent card now says `D=$CARGO_TARGET_DIR/debug/tide-admit`, default `target/`.

## N5: fixed

`git status --porcelain` counts untracked files as uncommitted changes. `target/` is git-ignored, so the default work directory does not mark the tree dirty. Outside a repository, the label falls back to `unknown` with no suffix, as before.

## N6: fixed

The cargo build's status is now checked on its own and reports "cannot build Tide's std". A JSON stream with no `filenames` now gives "cargo reported no library artifacts for Tide's std". Both still exit 2.

## Docs touched by the fixes

The new guide and card text matches the script:
- the sample output now has "(each cause: lost hits)";
- each replay must exit 101 and print `replay reproduced`;
- a replay of a failing recording exits 101.

One cosmetic point: `demo/README.md:164` has an unwrapped long line.

DONE
