Claude Opus 5.5 · 2026-11-11

**Verdict: approve.** Finding A is fixed and so is the REF-0407 nit. I found no new defects.

**Status:** I checked HEAD 30eb8aa. The diff hash `1ddbeab3…c785` matches, the checkout is clean, and the driver's SHA-256 `d0d17b05…b82b` matches the frozen record. I ran every probe one at a time under `nice -n 19` with `CARGO_BUILD_JOBS=4`, stdin closed and a timeout. Load stayed between 4 and 7, about 15 GB was free, and the quiet window had ended at 01:09:13Z. I made no edits or git/`br` writes and did not open `trials/e1-checked/`. My probes are in `/tmp/claude-1000/r5u653/`.

1. **A is fixed.**
   - **1500 acyclic functions:** 0.71 s, down from 114.6 s.
   - **300 functions:** 0.50 s acyclic and 0.25–0.38 s cyclic.
   - **Cyclic diagnostics:** the cyclic case still gives exactly 299 FX0407. Their FX lines are byte-identical to my round-4 output (sha `7fd8928b…1f27`).
   - **Small probes:** all 48 driver reruns give the same FX lines and rustc error codes as the round-4 baseline. They cover `a01`–`a29`, the `i_*` cases, `r3_*` (including `reset` and `fixed_wrap`) and `p01`–`p03`.
2. **The SCC pass is correct.** The old rule reported an expanding edge `u→v` when `v` can reach `u`. Given the edge exists, that holds exactly when `u` and `v` are in the same component, so the check is unchanged.
   - Kosaraju is implemented correctly (`traits.rs:272-318`): finishing order comes from a forward pass, then components are assigned on the reverse graph in reverse finishing order.
   - New probes, all with the expected result:

     | Probe | Case | Result |
     |---|---|---|
     | `s01` | expanding self-loop | FX0407 |
     | `s02` | non-expanding self-loop | accepted |
     | `s03` | expanding and non-expanding edges in parallel, inside a cycle | one FX0407, on the expanding call (1:48) |
     | `s04` | the same parallel pair, no cycle | accepted |
     | `s05` | `f::<(T, T)>` | one FX0407 |
     | `s06` | expanding edge between two separate cyclic components | accepted |
     | `s07` | `f::<Vec<B>, B>` (no cycle at the parameter level) | accepted |
     | `s08` | expanding and fixed self-calls in parallel | FX0407 on the expanding one |

   - Removing duplicates is exact: within one argument, every parameter occurrence produces the same caller, callee, span and growth flag.
3. **It is iterative.** Both traversals use explicit heap stacks and there is no recursion.
   - A 20,000-function chain with one cycle (`d02`) gives exactly one FX0407, at 20000:38, in 2.9 s.
   - A 20,000-function acyclic chain where every edge expands (`d01`) is accepted. Its time grows linearly: 0.98 s, 2.0 s and 3.6 s at 5k, 10k and 20k functions.
   - Neither run overflowed the stack.
4. **The order is deterministic.** Component IDs only feed an equality test, and the hash map is used only for lookups, never iterated. Reports follow the order of `trait_calls`, and the existing diagnostic owner then sorts them.
   - Three cyclic outputs (round 4, this round's first run and a repeat) have identical hashes, and their lines are in ascending order.
5. **The nit is fixed.** `conformance/rules.md:160` now reads "…require selected local impls; unresolved aliases and rustc instance-depth errors alone prove no growth".
6. **No new defects.** Every node index lookup is inserted before it is read, so none can panic. The 512-graph exhaustive unit test plus the 20k-node deep test are enough to protect the helper.

**New findings:** none.
