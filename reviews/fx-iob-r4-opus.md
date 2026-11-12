Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: approve

Scope: re-check of my round-3 findings only, on `git diff a810843 214235a` (HEAD
`214235a56f291be6a3bed991b25c3e136c268144`, one commit on `dde6c57`, worktree clean) and the
report's 'Fix round 2' section. I applied the coordinator's decisions as binding: label findings 1
and 2, add the cutoff for finding 3, add the assertion for nit 4, skip nit 5. Read-only;
`trials/e1-checked/` was not opened.

## Finding 1 (blocking in r3): resolved

The label appears in all five places: `par.rs:7-11`, `lib.rs:31-35`, `README.md:283-286`,
`conformance/README.md:222-225`, and `design/03-authority-concurrency.md:1832-1836` ("this part of
PAR-2 remains partial"). It is accurate. A right branch that has already spawned runs to completion
after a left panic. When no token was free, `fork` runs `(a(), b())` inline, so `b` never starts.
That is why the outcome "can depend on machine load", and the qualifier "already spawned" is
correct. The README and the crate overview now limit lowest-index propagation to "terminating"
branches, which matches what the probe showed. The normative PAR-2 paragraph is unchanged, as it
should be. One wording point, not a defect: "Stop flags are per call" reads as if `join` had a flag,
but it has none. The next sentence covers `join` explicitly, so I'm not asking for a change.

## Finding 2 (non-blocking in r3): labelled, accurate; one completeness nit

The label is present in all five places (`par.rs:11-13`, `lib.rs:35-37`, `README.md:289-291`,
`conformance/README.md:228-230`, `03:1843-1845`). The design and conformance notes correctly add
that simulation is unaffected, since regions run inline there. Nit, non-blocking (for fx-djp with
the transport mechanism): the same missing location and violation also feed `same_kind`
(`runner.rs:138-152`), so on the thread runtime shrinking can also depend on scheduling, not only
the discard verdict. Adding "and can change which shrunk case is reported" would make the label
complete.

## Finding 3 (non-blocking in r3): fix correct, unobservable apart from speed, no new defect

- **Same tree.** The midpoint split, the singleton leaves, the stop checks (`par.rs:235`, `:250`) and
  the merge are untouched. Only the choice between `fork` and inline calls changed (`par.rs:237-246`).
- **Same comparison and sink order.** The inline branch makes exactly the calls `fork` already made
  when it had no token: left, then right, then the merge on the caller. Above the cutoff, `fork` still
  appends left's records, then right's, before the caller's merge. So the comparator's sink trace is
  post-order either way. The new test (`par/tests.rs:336-387`) proves it: lengths 4095, 4096, 4097
  and 8192, with 1 and 4 workers, give equal ordered traces and equal sorted results. It also checks
  that no helper compares below the cutoff and that one does at or above it. With one worker, the
  old and new code make the same calls, so the trace also equals round 3's. Decision 4 holds.
- **Panics.** Below the cutoff, a merge panic propagates directly, not through a capture. That is the
  same behavior as the old no-token path. The left panic still prevents the right sibling from
  running, and `stop.fetch_min` is unchanged. There is no new `unsafe`, no new shared state and no
  change in drops.
- **Probe R4-1**, same harness as round 3, built from `git show 214235a:crates/tide-std/src/par.rs`
  with the spawn counter, `rustc +stable -O`. It ran at 11:44Z, after quiet-until (11:22Z) had
  expired, with load 2.3, 19 GB available, `nice -n 19`, `timeout`, `ulimit -c 0` and closed stdin.
  It was not a cargo build.

  | n | std `sort` | `par_sort` | Helper spawns | Round 3 `par_sort` |
  | ---: | ---: | ---: | ---: | --- |
  | 1M | 25.6 ms | 114 ms | 106 | 304 ms, 10,503 spawns |
  | 10k | 0.6 ms | 2.5 ms | 3 | 251 spawns |
  | 4095 | — | — | 0 | — |

  Every result equals std `sort`. The remaining 4.5x gap to std `sort` comes from the per-merge index
  vectors and the sequential root merge. The report records it as future work, and the docs make no
  speed claim. It is not a defect of this fix.

## Nit 4: resolved

`par/tests.rs:389-432` reads `sim.records()` after the caught winner. It asserts `["left\n"]` for
`join` and `["0\n","1\n"]` for `map`/`for`, so the simulated SINK-4 prefix is now actually tested.
The author reports 9/9 `par::tests` passing in debug and release, and 3/3 for `tide-admit`
`parallel_`. I did not rerun them.

## Nits 5 and 6

Nit 5 was skipped by coordinator decision. Nit 6 (the bead wording for decision 4) stays with the
coordinator.

DONE
