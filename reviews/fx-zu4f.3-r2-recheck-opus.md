Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate 29bd248 (re-check of 76c7ac4 findings)

# fx-zu4f.3 r2 re-check (Opus): only my r2 findings

## Verdict

approve. Confidence: high.

Scope:
- `git diff 76c7ac4 29bd248` changes only `examples/case-studies/hyper-ambient-runtime/README.md`, with 60 lines added and 55 removed.
- `buggy.rs`, `fixed.rs`, `no_clock.rs`, `rust-repro.md` and every `//@` declaration are byte-identical to 76c7ac4, so the examples run recorded for 76c7ac4 still holds.
- `git diff --check` is clean.
- I checked each new sentence against the upstream sources I read in r2.
- No new full review, no builds, no file changed except this report.

## Findings

1. **Blocking: the lead credited TD0602 and TD0601 with the catch. Resolved.**
   - The lead (:3–28) now opens: "No Tide admission code in this case study catches either reported failure."
   - The runtime-feature report is tied to tokio's timer. That timer gets only `E0433`, with the note that any compiler without tokio reports the same. `TD0501` is named as Tide's own rule for that reach, shown in `cap_ambient.rs`, "not in this case study".
   - #2808 is "a reaper that never starts", and the lead says Tide does not prevent it (`no_clock.rs`).
   - `TD0602` is tied to the default spawner, "which neither reported failure ran through". That is correct: #2808 has no spawn, and #2595 ran under the reporter's own `minimal_executor`.
   - `TD0601` is tied to Tide having no async.
   - The table row (:103) says the same.
   - The "Would" bullet (:146–147) now says it states the rule, "not this case study's catch".
   - No reader is left thinking TD0602 or TD0601 catches either report.
   - Each new factual sentence matches the sources: the panic's frames, the executor, `E0433` from reference §13, and `cap_ambient.rs:8`.

2. **"On every seed": resolved.** The lead now says "on each of its 20 seeds".

3. **"Cheap and exact" as a Tide gain: resolved.** The bullet is now "Keep every clock inside the simulator". It credits tokio's paused clock for `rust-repro.md` and states why that works only with `runtime`: hyper 0.14's `cfg` import of `tokio::time::Instant` (pool.rs:11–16). It also says the paused clock does not control the `std::time::Instant` stamps used without `runtime` and by hyper-util without a timer. That matches hyper-util's imports, and I found no new overclaim in it.

4. **tokio unpinned: resolved.** I offered two fixes, and this is one of them. The README now says the `Cargo.toml` asks for tokio 1 and the recorded run resolved 1.54.1.

5. **Repro run record: resolved as text; one item for tide-lead.**
   - The README condenses the run record and keeps "each build was run three times".
   - The builder's logs hold one run per build, and neither records rustc 1.96.0. That claim still rests on the builder's report.
   - This was never a text defect. tide-lead should record the run on the bead at landing, since `/mnt/build/tide-agents/fx-zu4f.3-target/repro/` is a temporary path. Not blocking.

6. **"Reaps only on request": resolved.** It now reads: "runs the idle interval only when the caller sets `pool_timer`; without it, expired connections are dropped only at checkout". That is accurate per hyper-util `1e50e91` (L425–441, and `expires` at checkout).

7. **"Runs unchanged under the threads runtime": resolved.** It now reads "is written to run unchanged …; only the simulator runs it here."

8. **PR #2975 named by number: unchanged.** I raised this only so tide-lead decides it, and it stays outside the rule against numbering fixed issues. #2892 is still absent, and no closed or fixed issue is named by number.

9. **Simplicity: resolved.**
   - The repro section is cut to a pointer and the run record.
   - The three-change list now points to `fixed.rs`'s header (:7–9), which does list all three.
   - "Choose the design" is merged into "What the Rust version would need" with its meaning kept.
   - Nit, not blocking: the reflow leaves a short line, "test, shared with" (:110). It renders the same in Markdown.

10. **research/33 wording: out of scope, unchanged as expected.** It remains a note for whoever next edits research/33, or for fx-qlek.7.

## Regressions and new overclaims

None found. The new lead claims less than before, and each new sentence is sourced.

DONE
