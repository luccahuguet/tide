Model: claude-opus-5-5 · Date: 2026-11-12

Verdict: request_changes

Candidate: `001cbda` on base `5db3d53`. The SHA-256 of `git diff --binary 5db3d53 001cbda` is `f40798722fb8f524a87593e59c3dbd7de2cb1898abe37717ad255e6f541e18f9`, which matches the brief, and the worktree is clean. I read the whole diff, the touched callers, reference §10.1–10.2, design/04 §1.8 and §2.4, and the binding comments on fx-m72.2, fx-mhd and fx-m72.3. I did not open `trials/e1-checked/`. I made no writes outside my scratchpad and this file.

What holds, judged on its own terms:

- **Stopping rule.** The counting loop matches decision 2 exactly (`runner.rs:263-293, 383-403`):
  - it stops at N accepted or 100*N generated;
  - shrink evaluations are not counted;
  - it counts first rejections, because each conjunct is its own short-circuiting helper;
  - ties go by source order, via the sort followed by a strict `>` fold.
- **Callee rule.** It matches decision 3 (`runner.rs:209-222`). The location and payload cross-check keeps a caught earlier violation from turning an unrelated panic into a discard.
- **Single owners.** `run_config` is the one runner, and `sim::__run` keeps only the child transport. Clause parsing reuses `contract::lower` and `requires_conjuncts`. The stub includes `test/mod.rs` (and so `guided.rs`) by `#[path]`, and includes `sim.rs` and `sim/runner.rs`, so fx-nkv equivalence holds structurally.
- **Docs.** The README, the tide-std crate docs, design/03 and design/04 §2.4 item 5 state the count, the ratio and the message truthfully.
- **Planner.** The guidance planner never breaks compilation for valid user clauses, as far as I can find. Each operand is either translated or dropped to filter-only, and every candidate is re-checked by the full requires, release included.
- **fx-m72.3.** It gets what it needs. `Config`, `Source` and `run_config` are a reusable runner. Copying a tested function's requires onto a synthesized `#[test]` gets guidance for free. `allow(vacuous)` needs only one more `Config` field.

## Findings

1. **Blocking: guided sequences and dependent guided bounds don't shrink. The flagship `i < xs.len()` shape reports an essentially unshrunk input.**

   Evidence:
   - `crates/tide-std/src/test/guided.rs:72` tags each guided scalar offset with `Kind::Integer(width)`, and `width` depends on the other parameter's value.
   - `guided.rs:123-124` draws a guided sequence's length as one `g.index(hi - lo + 1)` choice, followed by bare element choices.
   - `Gen::draw` (`test/mod.rs:117-121`) supplies 0 and does not pop when the kinds differ.

   So shrinking the length source changes the index's kind. The index replays as 0, which passes. Deleting element choices doesn't shorten the vector, because there are no per-element `Kind::More` flags like `collection` has (`mod.rs:153`). Every span deletion fails, and the 128-evaluation stall budget runs out inside `delete_spans` before `minimize` reaches the index.

   This is a regression from filter-only generation, where ordinary `Vec` choices delete cleanly. The new code causes it.

   Probe: frozen driver and std artifacts, under nice 19 and JOBS=4, with load 8 and 15 GB free, outside any quiet window. Sources are in my scratchpad under `shrink/`.

   ```rust
   #[test] #[requires(i < xs.len())]
   fn index(i: usize, xs: Vec<u16>) { assert!(i < 3); }
   ```

   Output: `input: xs = [0 × 31], i = 30` after `shrink evaluations: 132`, meaning it stalled. The minimum is `xs = [0, 0, 0, 0], i = 3`.

   A `lo <= hi` pair with `assert!(hi - lo < 5)` shrank to `lo = 1, hi = 6`, close to but not at `0, 5`. The fixture's `shrink_filtered` and `simulated` cases pass only because no draw follows the guided sequence.

   Fix:
   - Record guided offsets under a kind that doesn't depend on width, such as `Kind::Integer(T::MAX_KEY - T::MIN_KEY)`. Generate with the real width and keep the existing `offset.min(width)` clamp, so replay stays aligned when a dependency moves.
   - Encode a guided sequence as `lo` unconditional elements followed by up to `hi - lo` `Kind::More`-flagged elements, as `collection` does. Span deletion then shortens it.
   - Add the probe above as a driver regression expecting `input: xs = [0, 0, 0, 0], i = 3`.

2. **Non-blocking: shrinking a failing `#[sim_test]` now runs inside that seed's 300 s backstop.**

   Evidence: `sim/runner.rs:85-115` resets the timer only on `TIDE_SIM_SEED_START` lines. `runner.rs:290` emits one per generated seed, but `runner.rs:340-348` then runs up to 1,000 shrink evaluations with no further progress line. Each evaluation builds a fresh simulator, and the failing seed's two self-check runs add to the time.

   Before this change, a seed cost at most two evaluations. Now a failing simulation that takes more than about 0.3 s per run is killed as `per-seed wall-clock timeout`. The seed and trace survive in the echoed stderr, but the shrunk input and cause are lost. The documented "300 s per seed" no longer describes what the user gets.

   Fix: emit a progress line per shrink evaluation, or reset the backstop per evaluation, or document that a seed's shrink shares its window.

3. **Non-blocking: the tested function's requires only filters; it does not guide.** The docs should say so.

   Evidence: reference §10.2 says "`#[requires]`, on a test or a tested function, filters and guides generation". `Guidance::new` (`test_guidance.rs:157-171`) reads only the test's own attributes, and callee requires act only through the decision-3 discard. That is a defensible reading: guidance from a callee is plausibly fx-m72.3's generated-test case.

   The README's "Its own `#[requires]` guides…" implies the limit, but the std crate docs' "Property implementation limits" (`tide-std/src/lib.rs:107-126`) never state it.

   Fix: add one sentence there saying a called function's requires discards cases but does not guide. Record the reading on fx-m72.2 or fx-djp next to the `distinct` note.

4. **Nit: the planner rejects the same operand types twice.** The checks at `test_guidance.rs:251-253` and `:254-260` overlap; the second already covers the first's non-`equal` scalar case. Merge them into one condition: `len ? sequence(ty) : scalar(ty) || (equal && char/bool)`.

5. **Nit: `std::test::run` is now orphaned.** The macro calls only `run_config` (`test.rs:186-190`). The hidden `run` (`runner.rs:242-246`) and its re-export (`test/mod.rs:16`) remain only for `build.rs:1343`. Point that test at `run_config(.., Config::default(), ..)` and delete `run` (AP-MINIMAL-001 leftovers).

6. **Nit: the CLAUSE-7 note in `conformance/rules.md:215-217` is in the wrong order.** It now begins with "`contracts/guided_properties.rs` also checks…" ahead of the item's own label, "Runtime violation outcome;…". Put the new sentence after the existing list so the "also" has a referent.

7. **Nit: a plain property's parameter stream isn't derived from the seed.** With a `seed: u64` parameter, the generator's stream is seeded with the same value that the body passes to `Root::sim(seed)` (`runner.rs:286` with `Gen::new`). `#[sim_test]` instead derives the stream with `mix(seed ^ RUN_SEED)`, which is decision 4's "stream derived from it". Nothing correlates today, because simulated randomness is planned. Deriving the stream the same way whenever a fixed `seed` parameter is present would match decision 4 for both forms.

Probe command (scratchpad):

```sh
nice -n 19 env CARGO_BUILD_JOBS=4 timeout 120 target/debug/tide-admit --crate-type=lib --test --emit=link --tide-std target/tmp/build/std/artifacts/libtide_std.rlib --tide-dep-dir target/tmp/build/std/artifacts --out-dir <scratch>/shrink/out <scratch>/shrink/package/src/demo.rs </dev/null
```

Then `env -u TIDE_TEST_SEED <scratch>/shrink/out/demo-test --exact index --nocapture </dev/null` (and the same with `--exact span`).

Finding 1 must be fixed, and I re-check only that fix. DONE
