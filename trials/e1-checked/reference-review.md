# Reference concurrency inspection (draft, 2026-11-07)

Author's inspection of the reference sources, not an independent governance
verdict. This covers only these reference solutions. Protocol §6 still needs
an individual, line-cited review of every writer's final C1–C6/C9 snapshot.
All reference structure searches match actual code, not comment mentions.

| Task | Rust evidence | Tide evidence |
| --- | --- | --- |
| C1 | `references/rust/C1.rs:4`: one scoped child per `chunks(4096)`; all handles collected before joining and folding buckets. | `references/tide/C1.rs:3`: chunks of at most 4096, then `par_map` and a sequential fold, per reference §8. |
| C2 | `references/rust/C2.rs:28`: base case at 2048; otherwise one half spawns while the calling thread sorts the other; join precedes the in-place stable merge. | `references/tide/C2.rs:28`: the same recursion with `std::par::join`; `T: Send + Pure`, shared key closure `Sync + Pure`. No `par_sort`. |
| C3 | `references/rust/C3.rs:8`: a bounded channel and scoped producer for every input; handles are started before the caller reads and merges heads. Error messages are data, all streams drained, smallest input error chosen. | `references/tide/C3.rs:16`: the same producer/merge arrangement in `Tasks::scope`, with `sync_channel(capacity, tasks)`; producer errors are channel data rather than scope failures. |
| C4 | `references/rust/C4.rs:56`: batches of at most `workers` scoped children; each batch spawns before joining in order. Per-item mutexes are acquired in map order; checks precede every stock write. | `references/tide/C4.rs:43`: `with_all` acquires only the order's distinct cells; its closure captures pure names/quantities, not handles. `references/tide/C4.rs:65` spawns orders under `tasks.limit(workers)`, collecting results as data in input order. |
| C5 | `references/rust/C5.rs:10`: all job children spawn before any join, sharing one start instant and sleeping to their finish or batch deadline. No detached sleeping threads remain after return. | `references/tide/C5.rs:10`: all jobs spawn into an unbounded scope with one deadline; cancelled sleeps become `TimedOut` values, so the scope returns partial results rather than cancelling away the summary (03 TASK-4). |
| C6 | `references/rust/C6.rs:8`: one scoped child per haystack. Each checks the shared atomic flag every 4096 candidate byte positions; a winner sets it before returning its first local match. | `references/tide/C6.rs:3`: one child per haystack, `checkpoint()` every 4096 positions, `s.cancel()` on a match. Other children turn cancellation into `Ok(None)` so the successful result survives. No additional atomic API prerequisite. |
| C9 | `references/rust/C9.rs:23`: at most `workers` scoped jobs at once; every handle is explicitly joined and each panic counted without scope propagation. | `references/tide/C9.rs:23`: limited children call `spawn_isolated`, whose panic result is returned as data. 03 §6.1 says isolated calls take no extra limit slot while the caller waits. |
| C10 | `references/rust/C10.rs:27`: accept loop spawns a child before accepting the next connection. `suites/rust/C10.rs:23` keeps the first connection open while the second completes. | `references/tide/C10.rs:27` uses the same accept/spawn order; `suites/tide/C10.rs` has the paired 20-seed witness. |

Rust C1–C6/C9's checked outcomes execute successfully. The orchestrating agent's run outside
Codex's sandbox also passed both C10 cases (all 30 Rust references passed).
Codex's sandbox itself refuses TCP bind. Tide's concurrency behavior is
unproven until the production runtime and executable test adapter exist.
