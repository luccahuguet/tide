Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate 76c7ac4

# fx-zu4f.3 r2: Opus converge round (reports, fixes nothing)

## Verdict

request_changes. Confidence: high on the facts, medium on how much finding 1 weighs. Its fix is a few sentences of README text. No code or declaration needs to change.

Scope: I reviewed `git diff main...76c7ac4`, which adds five files under `examples/case-studies/hyper-ambient-runtime/`. I also read the bead, the epic, AGENTS.md, GOVERNANCE.md, research/33 §4–5, the examples runner, the policy table and rules.md rows for TD0501, TD0601 and TD0602, `cap_ambient.rs`, `tide-std`'s `Clock`, the builder's report, Codex r1 and its re-check, and the builder's repro logs.

I checked these upstream sources against the claims:
- hyper v0.14.32's `src/client/pool.rs`, `client.rs` and `common/exec.rs`, from GitHub and from the local cargo registry;
- hyper-util `1e50e91`'s `src/client/legacy/pool.rs`;
- hyper #2808 and #2595 with their comments, #2857 and PR #2975, through the public GitHub API, read-only;
- crates.io's hyper version list.

I ran no builds or tests, started no subagents, did not open trials/e1-checked/, and contacted no one.

## Blocking findings

### 1. The opening credits admission's Tide codes with catching a cause they do not cover

Where: `README.md:3-9`, and the `buggy.rs` row of the table at `README.md:93`.

**The claim.** The lead paragraph says "Tide's admission rejects this bug's form", then gives the evidence: "Admission rejects the spawn (`TD0602`) and the `async` block it runs (`TD0601`)."

**The sources say otherwise.**
- **The panic report (#2595).** Its backtrace runs through `tokio::time::interval` ← `PoolInner::spawn_idle_interval` ← `PoolInner::put` ← `Pooled::drop`, under the reporter's own `minimal_executor` (frames 15–19 and 33–43).
  - The ambient reach that panicked is tokio's timer.
  - hyper's `tokio::spawn` default was not involved, since the reporter supplied an executor.
  - In `buggy.rs` that timer line gets only rustc's `E0433` (`buggy.rs:53`). Any compiler without the tokio crate would report the same error.
- **The open report (#2808).** No spawn and no timer exist at all; the missing reaper is the bug. `no_clock.rs` shows that Tide admits a pool with no reaper.
- **`TD0601`.** It fires because Tide has no async (REF-0601). It has nothing to do with ambient reach.

So none of the Tide-specific codes this case study shows is the mechanism of either reported failure:
- `TD0602` covers hyper's default spawner, which neither failure ran through;
- `TD0601` covers async syntax.

**What `buggy.rs` does not show.** The Tide rule that matches the panic's cause is an ambient clock or sleep, `TD0501`. The README claims it at :137–145, but only by pointing to `conformance/cases/authority/cap_ambient.rs`.

**Why it matters.** research/33 §5's brief said "the case study shows each as it occurs", and A1 forbids overclaiming. This is launch evidence, and the outreach rule's one-line Tide sentence would cite "no ambient timer or detached spawn". As written, a reader takes TD0602 and TD0601 as the catch. The top of the README does say "tokio's timer has no Tide code", but it never connects that limit to the panic the report describes.

**Minimum fix (text only).** Rewrite the lead and the table row so that each code is tied to what it covers:
- `TD0602` rejects the pool's default spawner;
- the timer that panicked in the runtime-feature report gets only `E0433`, since tokio is not a crate Tide admits;
- the Tide rule for that reach is `TD0501` on a std clock or sleep, shown in `cap_ambient.rs`, not in this case study;
- `TD0601` comes from the async spelling.

For example, replace :5–6 with: "Admission rejects the pool's default spawner, `tokio::spawn` (`TD0602`), and its `async` block (`TD0601`, since Tide has no async). The timer that panicked in the runtime-feature report gets only rustc's `E0433`, because Tide admits no external crate; Tide's own rule for that reach is `TD0501`, on Rust's ambient clock and sleep."

Another fix would also work: add a std-spelled line to `buggy.rs`, such as `std::thread::sleep` or `std::time::Instant::now()` in the interval, and add `TD0501` to its `//@ expect:`. That would show the clock rule in this case study itself. It is a bigger change, though, and the text fix is enough.

## Non-blocking findings

2. **"On every seed" claims more than the test ran** (`README.md:9`). The test runs 20 seeds. The timing argument (strict `> timeout` expiry, a tick per timeout, check-in no earlier than the ticker's start, FIN within 10 ms) does hold for every seed. Still, say "on each of its 20 seeds" or cite that argument. The landed sibling uses the same wording, so this is not blocking.

3. **"Make the test cheap and exact" is listed as a Tide gain** (`README.md:150-152`). `rust-repro.md` gets the same virtual-time determinism from tokio's paused clock. That works only because hyper 0.14 with `runtime` imports `tokio::time::Instant` (pool.rs:15–16).
   - The real difference: hyper 0.14 without `runtime` stamps with `std::time::Instant`, and so does hyper-util without a timer. tokio's paused clock does not control that clock.
   - In Tide no ambient clock can escape the simulator.
   - Say that, and drop the implication that Rust needs a hand-written fake timer.

4. **tokio is not pinned in the repro** (`rust-repro.md:36`: `tokio = { version = "1", ... }`). README:179 records tokio 1.54.1, and the builder's `Cargo.lock` confirms 1.54.1, but a later reader will resolve whatever 1.x is current. Use `=1.54.1`, or say the recorded run resolved 1.54.1.
   - The hyper pin is correct: `=0.14.32` is the latest 0.14 on crates.io.

5. **The repro's run record.**
   - What I read: the builder's `off.log` (fails at lib.rs:107, "the idle connection was still open 181s after its request") and `on.log` (ok). Both sit outside the repo at `/mnt/build/tide-agents/fx-zu4f.3-target/repro/`. The extracted `src/lib.rs` is identical to the Markdown block.
   - What is unverified: "each build was run three times" (README:179–180) rests only on the builder's report, since the logs hold one run each.
   - What to do: record the run on the bead when it lands, because that path is temporary. Neither log records rustc 1.96.0.
   - The repro is faithful:
     - with `default-features = false` and no `tcp`, an executor is required (exec.rs);
     - hyper 0.14.32's server sets no `h1_header_read_timeout` by default (server/conn.rs:263), so with `runtime` on only the pool's interval can close the connection;
     - `pool_idle_timeout` defaults to 90 s (client.rs:943);
     - the interval is `tokio::time::interval(dur)` with `dur` equal to the timeout (pool.rs:408–429);
     - `clear_expired` evicts only when idle for longer than `dur` (pool.rs:467).

     So 181 s is the correct bound for 0.14.32.

6. **"So 1.x reaps only on request"** (`README.md:75`) can be read as "per HTTP request". Say "only when the caller sets `pool_timer`; without it, expired connections are dropped only at checkout".

7. **"Runs unchanged under the threads runtime"** (`README.md:148-149`) is not run anywhere; it follows from reference §7.5. Say "is written to run unchanged", or cite §7.5 there.

8. **PR #2975 is named by number** (README:69). It is a merged PR, not an issue. The file text creates no GitHub backlink, and the commit message names no hyper number. Codex and the builder both kept it. I agree it is outside the "fixed issues" rule, and note it only so tide-lead decides it explicitly.

9. **Simplicity: concrete deletions.**
   - `README.md:167-175` restates `rust-repro.md:14-23`, the test design and the two outcomes. Replace it with one sentence pointing to rust-repro.md, and keep only the run record (:177–183).
   - `README.md:98-100` repeats the three-change list that `fixed.rs:7-9` already gives. Keep one copy.
   - "What the Rust version would need" (:185–193) partly overlaps "Would not: choose the design" (:162–163). They could merge into one paragraph.
   - `no_clock.rs` duplicating `fixed.rs` is necessary, because each variant compiles alone.

10. **Out of scope, reported only:** research/33's wording reads stronger than this case study's result.
   - §4.3's notes ("A Tide pool must take a `&Clock`, and a required parameter cannot be left out") and §5's "The Tide contrast" suggest that Tide forces reaping.
   - `no_clock.rs` shows it does not.
   - research/33 also still uses the old `FX05xx/FX06xx` codes.
   - Worth a note for whoever next edits research/33, or for fx-qlek.7's claims audit.

## Reasoning

**Claims, checked against sources.**
- **hyper 0.14.32.**
  - The `cfg(feature = "runtime")` interval is at pool.rs:407–429, and the quoted lines 423–429 match exactly.
  - The 90 s default is at client.rs:943.
  - `Exec::Default` runs `tokio::task::spawn` under `tcp`, which `runtime` turns on.
  - Without `runtime`, `std::time::Instant` is used (pool.rs:11–12), so README:142–143 is right.
  - With it, `tokio::time::Instant` is used, so `buggy.rs`'s stamp is faithful.
- **hyper-util `1e50e91`.**
  - `spawn_idle_interval` returns early without a timer (L425–441).
  - The quoted `now()` matches lines 342–346, and `std::time::Instant` is imported.
- **#2808.** It is open, filed 2022-04-07 against 0.13.10. The Fuchsia executor and `async-executor` details are in it. The maintainer's reply that day is quoted exactly.
- **#2595.** It is closed `not_planned` 2022-06-15, with "Tracking this in #2857". The quoted panic message and the `interval` → `spawn_idle_interval` backtrace match, and reqwest is named.
- **The fix.** #2857 was closed by PR #2975, merged 2022-09-15 as `23a56005`.
- **Licenses.** Both upstream licenses are MIT, © Sean McArthur. The two quotes, 7 and 5 lines, are attributed.
- **Issue numbers.** #2892 appears nowhere. No closed or fixed hyper issue is named by number; #2595 and #2857 appear only inside URLs.
- **Admission.**
  - The policy table denies `tokio::spawn` as TD0602 (policy.rs:188).
  - TD0601 is REF-0601.
  - TD0501 is CAP-1, and `cap_ambient.rs:8` covers `std::time::Instant::now()`.
  - reference §13 lists "imported Rust crates" as not in v0, so the E0433 limit is stated correctly.
  - Finding 1 concerns emphasis and attribution, not a false code.

**Test validity.**
- `buggy.rs`.
  - Its declaration `rejected TD0601 TD0602 E0433` is strict: every error must carry a declared code, and every code must appear (examples.rs:183–216).
  - The builder's driver output gives TD0602 at 52:5, TD0601 at 52:18 and 55:29, and E0433 on the five tokio paths.
  - TD0602 does come from the modelled spawn. TD0601 and E0433 come from the spelling and from the missing crate, which is finding 1.
- `fixed.rs` cannot pass vacuously.
  - "Tests pass" requires at least one `ok` and no failure (examples.rs:332–336).
  - The test requires two EOF reports between 90 s and 181 s after check-in, and the original `closed` sender stays alive, so the channel cannot disconnect.
  - The upper bound holds by the timing argument: ticks fall every 90 s from `start()`, check-in comes no earlier than that, eviction needs idle time over 90 s, and the simulated `Clock::sleep` is exact (time.rs:53).
  - The lower bound holds because no tick can evict before stamp + 90 s.
- `no_clock.rs` differs from `fixed.rs` only in the optional clock (I diffed them).
  - It spawns no ticker and keeps every connection, so `recv_deadline` times out on every seed.
  - The two `reports` lines pin both the reason ("…181s after its check-in: Timeout") and "0 passed; 20 failed (20 seeds)".

**rust-repro.md.** It is faithful and its outcome is backed by the logs, as finding 5 details. The only gaps are the unpinned tokio version and the three-run claim.

DONE
