# 36 — Benchmarks: Tide against plain Rust (fx-6u8m.2)

Research for fx-6u8m.2, 2026-11-17 to 18, by a Claude Opus 5.5 builder. It reports
what Tide's additions cost against plain Rust on the same four programs, as
measured on one machine. It decides nothing and optimizes nothing: losses are
reported here, not fixed.

Evidence labels apply to each following paragraph, list or table:

- **[measured]**: from the raw records this note names (§2.6), by the rerun
  tool as it stood at the revision its header records. Those revisions are
  earlier, amended states of this note's commit and are not in the
  repository's history; §2.6 gives their identities and what differs.
- **[repo evidence]**: Tide's tree at branch `fx-6u8m.2`.
- **[inference]**: this note's reasoning from the evidence.

## What this is not

- **Not D0's E4/E5 gate run.** E4 (compile speed) and E5 (runtime) are
  "benchmark suites against the D12 targets and against Rust"
  (`design/01-design.md`, D0). This note is neither: four small programs, one
  machine, no D12 target checked. Nothing here passes or fails a gate.
- **Not lust or Blaze.** Those reruns are fx-6u8m.3-.5, after the alpha.
- **Not fx-ayxr.** fx-ayxr owns Tide's build time and compiler memory against
  plain cargo on the Eon fixture. This note reuses its method (interleaved
  rounds, medians with CV, the `wait4` peak-RSS wrapper), on these pairs.
- **Not fx-4cdq.** Warm edit-to-diagnostic latency for a future fast front
  end is fx-4cdq's. This note measures today's driver, which checks at rustc
  speed plus admission.
- **E1 held-out rule.** The bead's E1 held-out constraint bound only until
  fx-ic2.11 published the gate report (a029f11). fx-ic2.11 is closed, so the
  rule no longer binds (fx-6u8m.2's 2026-11-17 comment). Even so, the author
  did not open `trials/e1-checked/`; the programs measured are fx-6u8m.1's
  `benchmarks/` pairs.

## 1. Results

All numbers and ratios in this section are [measured], from two runs (§2.5):
the main run, 2026-11-18 03:00-03:29Z, revision `bf6b958`, in a booked quiet
window; and a run of the test-build phase alone (`contracts` as a test
build, §1.4), 04:25-04:30Z, revision `c04dc44`, **outside a quiet window**.
Both had clean trees and 30 rounds per series (§2.4). A cell gives the median, then the CV and N in parentheses, then the
ratio to the column the table names. Times are seconds of wall time. Why a
number is what it is was not measured: each explanation is marked
[repo evidence] when it rests on the code, or [inference].

> **Stability: almost no series met the tool's criterion.** All 60 series
> of the main run ran to the 30-round cap with a CV above 1% (from 1.04% to
> 46.7%), so the run marks every one unstable (§1.6). In the test-build run,
> one of four series met it (`tide-debug-test`, CV 0.81%); the other three
> did not (3.2% to 5.9%). The medians and ratios below are descriptive
> observations from these unstable runs, not settled measurements.
>
> **Ties and losses are read by paired rounds.** Each round measures every
> arm back to back, so each round gives one ratio between two arms. An arm
> is called slower when it is slower in at least 20 of the 30 rounds (a
> one-sided sign test at about 5%: p = 0.049); otherwise the comparison is
> a tie. "No measurable cost" and "tie" below mean exactly that. The counts
> are recomputed from the records' `run` and `test` measurements (§2.6).

### 1.1 Summary

| Question | Answer on these four programs |
| --- | --- |
| Cold build, debug | `tide-debug` takes **3.8-4.6x** `rust-debug`'s time (0.68-0.84 s against 0.15-0.22 s). |
| Cold build, release | `tide-release` takes **1.7-2.1x** `rust-release-tide-flags`'s time and 1.8-2.2x `rust-release`'s. |
| Rebuild after a one-line edit | `tide-debug` takes **5.2-6.3x** incremental `rust-debug`'s time (0.67-0.85 s against 0.13-0.14 s): the driver rebuilds from scratch. |
| Compiler peak memory | Tide builds peak at 143-193 MiB against 117-152 MiB for rustc in the same profile: **+17% to +39%**. |
| Binary size | `tide-release` loads **1.4-2.0x** the bytes of `rust-release-tide-flags`; `tide-debug` files are about 4x `rust-debug`'s (18.5-20.9 MiB against 4.5-5.3 MiB). |
| Contracts in release | **No measurable cost in this run**: a tie (Tide's arm slower in 11 of 30 rounds). `tide-release` 0.901 s against 0.906 s for `rust-release-tide-flags` (0.99x, CVs 2.5% and 1.2%). |
| Contracts in debug | At 1,200,000: `tide-debug` 7.90 s against `rust-debug` 7.51 s, **1.05x**, a tie (slower in 16 of 30 rounds). At 200,000, in the separate test-build run: **1.19x**, slower in **all 30** rounds, a loss. This plain debug build checks only the cheap clauses; the two sortedness clauses run only in test builds. |
| Contracts in a test build | At 200,000 (a separate run, outside a quiet window): the debug test build takes **7.7x** the plain debug build, **9.2x** `rust-debug` and **71x** the erased `tide-release` (2.80 s against 0.36, 0.30 and 0.039 s). |
| Capability file I/O | `files`, `tide-release` against `rust-release-tide-flags`: **1.04x** by medians; Tide's arm slower in **26 of 30** rounds, paired median ratio 1.07x: **a small loss**, of a few percent. |
| Capability TCP | `tcp`, same comparison: Tide's arm takes **1.18x** as long, slower in all 30 rounds. |
| Profile flags alone | `rust-release-tide-flags` against `rust-release`: **1.74x** on `tasks`, 1.00-1.01x on the other three. |
| Tasks runtime | `tide-release` 2.29x `rust-release` on `tasks`: 1.74x from the flags, then 1.31x between Tide's arm and its twin with the same flags. |
| Simulator | One seed of `tasks` at 1,000,000 numbers: **12.8x** the real run under `tide-release`, **16.1x** under `tide-debug`; about 1.8 and 3.8 µs per traced event. |

Every comparison above is a loss for Tide or a tie. None is a win. The
largest costs are the edit loop (the driver has no incremental build) and,
at run time, Tide's profile flags on the arithmetic-heavy `tasks` (this run
does not separate which flag costs what).

### 1.2 Compile time and compiler memory

**Cold build**: median s (CV, N); ratio to `rust-debug`

| Pair | `tide-debug` | `tide-release` | `rust-debug` | `rust-release` | `rust-release-tide-flags` | `stable-release` |
| --- | --- | --- | --- | --- | --- | --- |
| contracts | 0.681 (6.2%, 30); 4.60x | 0.462 (2.6%, 30); 3.12x | 0.148 (18.2%, 30) | 0.206 (6.6%, 30); 1.39x | 0.220 (5.9%, 30); 1.49x | 0.214 (46.7%, 30); 1.45x |
| files | 0.700 (1.8%, 30); 3.93x | 0.529 (1.4%, 30); 2.97x | 0.178 (4.8%, 30) | 0.278 (2.7%, 30); 1.56x | 0.281 (2.8%, 30); 1.58x | 0.257 (4.1%, 30); 1.44x |
| tasks | 0.839 (2.6%, 30); 3.82x | 0.646 (4.5%, 30); 2.94x | 0.219 (7.8%, 30) | 0.359 (4.2%, 30); 1.64x | 0.381 (4.4%, 30); 1.74x | 0.311 (5.3%, 30); 1.42x |
| tcp | 0.795 (1.0%, 30); 4.43x | 0.590 (2.0%, 30); 3.29x | 0.179 (2.8%, 30) | 0.263 (2.2%, 30); 1.47x | 0.299 (1.8%, 30); 1.67x | 0.237 (8.8%, 30); 1.32x |

`tide-release` against `rust-release-tide-flags`, the same flags and
optimization: 2.10x (`contracts`), 1.88x (`files`), 1.69x (`tasks`), 1.98x
(`tcp`). [repo evidence] A Tide build's time includes admission, the driver's re-executed
compile child, its cache writes, Tide's std's metadata, and a GNU ld link
where rustc uses `rust-lld` (§3); no configuration separates these.

**Rebuild after a one-line edit**: median s (CV, N); ratio to `rust-debug`

| Pair | `tide-debug` | `rust-debug` (incremental) |
| --- | --- | --- |
| contracts | 0.672 (2.7%, 30); 5.18x | 0.130 (15.3%, 30) |
| files | 0.699 (1.5%, 30); 5.18x | 0.135 (4.2%, 30) |
| tasks | 0.855 (2.1%, 30); 6.26x | 0.136 (19.0%, 30) |
| tcp | 0.799 (1.1%, 30); 5.91x | 0.135 (3.1%, 30) |

The driver's rebuild costs what its cold build costs (0.67-0.85 s in both
tables); [repo evidence] it has no incremental mode. rustc's incremental rebuild takes
0.62-0.88x its own cold build. These programs are a few hundred lines, so
most of rustc's time is fixed (linking, loading std); [inference] on a large
crate the gap in the edit loop would widen, because the driver's rebuild
grows with the whole crate and rustc's incremental one with the edit.

**Compiler peak RSS**: median MiB, the largest process in the compiler's tree

| Pair | `tide-debug` | `tide-release` | `rust-debug` | `rust-release` | `rust-release-tide-flags` | `stable-release` |
| --- | --- | --- | --- | --- | --- | --- |
| contracts | 163 | 143 | 117 | 117 | 117 | 115 |
| files | 173 | 154 | 140 | 132 | 131 | 130 |
| tasks | 189 | 178 | 152 | 146 | 143 | 146 |
| tcp | 193 | 170 | 139 | 130 | 131 | 133 |

`tide-debug` rebuilds peak the same as its cold builds (163-193 MiB).
Incremental `rust-debug` rebuilds peak at 117-122 MiB, at or below its cold
builds' 117-152 MiB.

### 1.3 Binary size

**Binary size**: file KiB / loaded KiB (the `PT_LOAD` segments); every build
of a series wrote the same size

| Pair | `tide-debug` | `tide-release` | `rust-debug` | `rust-release` | `rust-release-tide-flags` | `stable-release` |
| --- | --- | --- | --- | --- | --- | --- |
| contracts | 18992 / 906 | 4584 / 465 | 4588 / 362 | 4442 / 341 | 4443 / 341 | 4246 / 341 |
| files | 19714 / 1315 | 4796 / 636 | 4850 / 424 | 4476 / 367 | 4476 / 367 | 4281 / 368 |
| tasks | 21374 / 1395 | 4864 / 659 | 5400 / 487 | 4528 / 400 | 4532 / 405 | 4342 / 410 |
| tcp | 20776 / 1601 | 4954 / 743 | 4940 / 439 | 4498 / 379 | 4499 / 381 | 4306 / 383 |

Loaded bytes, `tide-release` against `rust-release-tide-flags`: 1.36x, 1.73x,
1.63x, 1.95x. No binary is stripped. [inference] The `tide-debug` files are
about 4x `rust-debug`'s because Tide's std, built in the dev profile, links its debug
information in; Rust's std brings less. Release file sizes differ by 3-10%:
most of every release file is std's symbols and debug information, which no
segment loads.

### 1.4 Runtime

**Run**: median s (CV, N) at the timed size; ratio to `rust-release`

| Pair | `tide-debug` | `tide-release` | `rust-debug` | `rust-release` | `rust-release-tide-flags` | `stable-release` |
| --- | --- | --- | --- | --- | --- | --- |
| contracts | 7.896 (5.5%, 30); 8.71x | 0.901 (2.5%, 30); 0.99x | 7.506 (3.7%, 30); 8.28x | 0.906 (1.1%, 30) | 0.906 (1.2%, 30); 1.00x | 0.907 (1.1%, 30); 1.00x |
| files | 1.379 (3.3%, 30); 1.29x | 1.124 (4.0%, 30); 1.05x | 1.143 (6.1%, 30); 1.07x | 1.069 (4.1%, 30) | 1.081 (4.6%, 30); 1.01x | 1.047 (4.2%, 30); 0.98x |
| tasks | 2.252 (6.7%, 30); 4.05x | 1.271 (4.3%, 30); 2.29x | 1.924 (7.1%, 30); 3.46x | 0.556 (5.7%, 30) | 0.967 (4.3%, 30); 1.74x | 0.546 (2.3%, 30); 0.98x |
| tcp | 1.699 (3.1%, 30); 1.73x | 1.161 (3.2%, 30); 1.19x | 1.070 (2.7%, 30); 1.09x | 0.980 (3.2%, 30) | 0.986 (2.9%, 30); 1.01x | 0.979 (3.6%, 30); 1.00x |

Read by the cost each row isolates:

- **Contracts.**
  - Release: `tide-release` runs `contracts` in 0.901 s against 0.906 s for
    `rust-release-tide-flags`, a difference inside both series' spread.
    [repo evidence] That the clauses are erased, not just cheap, is checked
    by `crates/tide-admit/tests/build.rs`
    (`executable_contracts_run_in_debug_test_and_erase_in_release`); this
    benchmark shows only that the erased release build cost nothing
    measurable in this run.
  - Debug: `tide-debug` is 1.05x `rust-debug`, also inside the spread (CVs
    5.5% and 3.7%).
  - [repo evidence] What that debug arm checks. A plain debug build checks
    the clauses on `isqrt` and `sieve`, the `invariant!`, and
    `insert_sorted`'s length `ensures`. `insert_sorted`'s two
    `xs.is_sorted()` clauses are sampled clauses: a method call outside a
    short list of cheap ones (`len`, `get`, `contains` and the like) makes a
    clause sampled (`crates/tide-macros/src/contract.rs`), and a sampled
    clause runs only in a debug test build. In a plain debug binary, each
    call instead reports the clause as unchecked
    (`std::__contract::unchecked`, which records it once) and never
    evaluates it. The tool builds `contracts` as a plain binary, so neither
    sortedness scan ran in this run. The 1.05x covers the cheap clauses,
    that bookkeeping, and the other differences of §3, Tide's unoptimized
    std among them.
  - Test build: the only build in which the sortedness scans run, measured
    in its own run (the test-build phase, §2.3), at 200,000 rather than the
    timed 1,200,000, outside a quiet window (§2.5). Before its rounds, a
    check showed that the sampled clauses run in that build: an unmeasured
    test called `insert_sorted` on an unsorted vector and panicked on
    `xs.is_sorted()`. Every round's test-build output contained the plain
    `tide-debug` program's output.

    **Test-build phase**: `contracts` at 200,000; median s (CV, N); ratio of
    `tide-debug-test` to each arm

    | Arm | Build | Median | CV | N | `tide-debug-test` is |
    | --- | --- | --- | --- | --- | --- |
    | `tide-debug-test` | debug test build: every clause checked | 2.802 | 0.81% | 30 | |
    | `tide-debug` | plain debug: sampled clauses unchecked | 0.362 | 4.2% | 30 | 7.73x |
    | `tide-release` | release: every clause erased | 0.039 | 5.9% | 30 | 71.0x |
    | `rust-debug` | Rust twin, debug | 0.305 | 3.2% | 30 | 9.20x |

    At 200,000, the debug test build took 7.7x as long as the plain debug
    build, slower in all 30 rounds. [repo evidence] Its sampled clauses
    scan a vector on every insert into it. [inference] That makes this
    workload unrepresentative of typical contract cost; the test build also
    differs from the plain one by the libtest harness and its `cfg(test)`
    code, and this run does not separate those. In this run `tide-debug` is
    1.19x `rust-debug`, slower in all 30 rounds: a loss. At the main run's
    1,200,000 it was 1.05x, slower in 16 of 30 rounds: a tie.
- **Capability-mediated I/O against direct std I/O.** `files`: 1.04x by
  medians (`tide-release` against `rust-release-tide-flags`), but Tide's arm
  was slower in 26 of 30 rounds, with a paired median ratio of 1.066
  (interquartile range 1.026-1.091): a small loss, of a few percent.
  `tcp`: 1.18x, slower in all 30 rounds: Tide's arm took about 18% longer
  on this loopback exchange. [repo evidence] `benchmarks/README.md`
  counts about ten system calls per exchange against four (nonblocking
  sockets, `poll`, an eventfd); [inference] that is the likely source, but
  this run does not time those calls separately, and the comparison keeps
  §3's std and linker differences. In debug, `files` is 1.21x and `tcp`
  1.59x `rust-debug`; those ratios include Tide's unoptimized std.
- **Profile flags against plain release.** `rust-release-tide-flags` costs
  nothing measurable on `contracts`, `files` and `tcp` (1.00-1.01x; slower
  in 15, 18 and 16 of 30 rounds: ties) and 1.74x on `tasks` (all 30
  rounds). [inference] The Collatz loop in `tasks` is arithmetic
  that the flags make overflow-checked, but the flags are overflow checks
  plus three inlining and generics limits, and this run does not separate
  them.
- **The rest of Tide's cost on `tasks`.** With the flags matched,
  `tide-release` is 1.31x `rust-release-tide-flags`: the aggregate
  remaining difference between the two programs. [inference] The Tide arm's
  runtime operations (`Tasks::scope`, `Shared::with` and the runtime's
  channel, once per block of 16 numbers) are the likely source; they are not
  timed separately, and §3's std and linker differences remain.
- **`stable-release`** (informational) runs within 2% of `rust-release` on
  every pair.

Run peak RSS is 3.5-8.4 MiB everywhere and mostly `timeout`'s (§3); the
records keep it, and this note draws nothing from it.

### 1.5 Simulator against the real runtime (`tasks`)

**Sim**: `tasks` at 1,000,000 numbers, one seed; median s (CV, N)

| Arm | Median | CV | N | Trace events | Peak RSS (MiB) |
| --- | --- | --- | --- | --- | --- |
| `tide-debug` (real) | 0.184 | 5.4% | 30 | | 3.5 |
| `tide-debug-sim` | 2.962 | 1.7% | 30 | 734,697 | 305 |
| `tide-release` (real) | 0.110 | 5.7% | 30 | | 3.5 |
| `tide-release-sim` | 1.413 | 1.7% | 30 | 734,697 | 304 |

- The simulated run takes **12.8x** the real one under `tide-release` and
  **16.1x** under `tide-debug`: each configuration's whole build (its
  optimization, flags and std), not the std alone.
- Divided over the 734,697 traced events, the extra time is **1.77 µs per
  event** (release) and **3.78 µs** (dev). This is an upper bound on each
  event's cost (§3), not a measured baton hand-off.
- v0's simulator runs each task on its own OS thread and passes a baton so
  that one runs at a time (`design/03-authority-concurrency.md` §9.1, D18).
  fx-ic2.12 measured a standalone optimized baton probe at 2.7 µs per pass
  with 9 threads. Here the whole simulator costs 1.77 µs per traced event
  with 5 tasks, so [inference] not every traced event is a hand-off between
  threads, and this run does not measure the baton's cost on its own.
- The simulated run peaks at 305 MiB against 3.5 MiB for the real run,
  about 430 bytes per traced event. [repo evidence] The scheduler keeps
  every event in memory as data, and renders the events as text when the
  trace is read.
- **fx-ic2.12's finding**: on 2026-11-14, about 95% of a simulation-heavy
  test's CPU went to formatting the scheduler's trace as text while it ran.
  fx-ic2.12 (6e2bbd3) made the scheduler record events as data and format
  them only when the trace is read; that test's workload dropped from 89 s
  to 44.9 s with a dev std. This run measures the simulator after that fix:
  the timed test never formats the trace (§2.3). The finding is therefore
  history, not part of these numbers.

### 1.6 Stability

In the main run, no series met the 1% CV rule: all 60 ran to the 30-round
cap and are marked unstable, as the records and the tool's closing line
say. (In the separate test-build run, `tide-debug-test` met it, at 0.81%,
and its three other series did not.) CVs range from
1.04% (`tcp` build, `tide-debug`) to 46.7% (`contracts` build,
`stable-release`). The largest come from single outlier rounds, mostly
round 0, the first measurement after the std builds (Opus N9): that
`stable-release` build took 0.819 s in round 0 against a median of 0.215 s.
Medians over 30 rounds absorb such rounds; CVs do not. [inference] A 1% rule
is tighter than a shared 16-thread laptop at `nice 19` delivers. Ties and
losses are therefore read by paired rounds (the rule in §1's stability
note), not by CVs: a CV-based reading would have called the `files` I/O
cost (slower in 26 of 30 rounds) a tie.

## 2. Method

### 2.1 Programs

[repo evidence] Four pairs, each a Tide program (`benchmarks/<pair>/tide.rs`)
and its plain-Rust twin (`rust.rs`) with byte-identical output, checked by
`crates/tide-admit/tests/benchmarks.rs` in the workspace suite.
`benchmarks/README.md` says, per pair, what Tide's implementation does
differently; the summary:

| Pair | Work | Timed size | What the Tide arm adds |
| --- | --- | --- | --- |
| `contracts` | prime sieve, integer square root, sorted insert | 1,200,000 | `#[requires]`, `#[ensures]`, `invariant!`: checked in debug, except the two sampled `is_sorted` clauses, which run only in test builds; erased in release |
| `files` | create, append, list, read, rename, remove files | 30,000 files | every call through the `Fs` capability: about 18 system calls per file against std's 12 |
| `tasks` | four workers, a shared cell and a channel | 10,000,000 numbers | `Tasks::scope`, `Shared::with` (FIFO), runtime channels |
| `tcp` | a server task and a client exchange lines on loopback | 100,000 messages | nonblocking sockets with `poll` and an eventfd: about ten system calls per exchange against four |

Every Tide program also gets the driver's fixed flags in every profile
(`crates/tide-admit/src/invocation.rs`): `-Coverflow-checks=on`,
`-Zinline-mir=no`, `-Zcross-crate-inline-threshold=never`,
`-Zshare-generics=no`.

### 2.2 Configurations

[repo evidence] The rerun tool, `crates/tide-admit/benches/pairs.rs`, builds
each pair under six configurations (`benchmarks/README.md` gives each one's
flags):

| Configuration | Compiler | Profile |
| --- | --- | --- |
| `tide-debug` | the admission driver; Tide's std built in Cargo's dev profile | `--tide-profile=debug` |
| `tide-release` | the admission driver; Tide's std built in Cargo's release profile | `--tide-profile=release` |
| `rust-debug` | pinned nightly rustc | Cargo dev's code generation |
| `rust-release` | pinned nightly rustc | Cargo release's code generation |
| `rust-release-tide-flags` | pinned nightly rustc | `rust-release` plus the driver's four fixed flags |
| `stable-release` | stable rustc | as `rust-release`; for information only |

The comparisons that isolate one cost:

- the profile flags: `rust-release-tide-flags` against `rust-release`;
- Tide's std, capabilities, runtime and contracts (and the linker, §3):
  `tide-release` against `rust-release-tide-flags`, which share the flags;
- the debug edit loop: `tide-debug` against `rust-debug`.

### 2.3 Phases

[repo evidence] For each pair, the tool measures, in order:

1. **build**: a cold build in a new directory (the driver's cache never
   hits), with the compiler process's wall time, its peak RSS (the largest
   process in its tree, from `wait4`), and the binary's file size and loaded
   size (the bytes of its `PT_LOAD` segments, which leave out debug
   information and symbol tables).
2. **rebuild**, `tide-debug` and `rust-debug` only: after one unmeasured warm
   build in a kept directory, each round adds `let _edit: u64 = <round>;` as
   the first line of `main` and rebuilds. `rust-debug` passes
   `-Cincremental` with its cache in that directory, as Cargo's dev profile
   does. The driver has no incremental mode; its cache is keyed by the whole
   source, so it misses after any edit and compiles from scratch.
3. **run**: the built program at its timed size, in a new empty directory,
   under `timeout`. Every run's output must equal `tide-debug`'s.
4. **sim**, `tasks` only, at 1,000,000 numbers: under `tide-debug` and
   `tide-release`, the real program against the same program run under the
   simulator, on one seed, as a test the tool appends (`bench_sim::timed`).
   The simulated run's output must equal the real one's. An unmeasured run
   counts the events the scheduler traced.
5. **test**, `contracts` only, at 200,000: the program as a debug test build
   (`tide-debug-test`, a test the tool appends that calls `run`), in which
   the sampled `is_sorted` clauses run, against the plain `tide-debug`,
   `tide-release` and `rust-debug` programs at the same size. Its output
   must contain `tide-debug`'s output. An unmeasured test first checks that
   a sampled clause runs. It was added after the main run and measured in a
   run of its own (`--phase test`).

### 2.4 Rounds and statistics

[repo evidence] fx-ayxr's method: each round measures every configuration of
a phase once, in table order, so arms interleave A B A B. A full run starts
with 5 rounds and adds 5 while any series in the phase has a coefficient of
variation (sample standard deviation over mean) above 1%, up to 30. Each
series reports N, its median, and its CV; a series still above 1% at 30
rounds is marked unstable. Medians are the reported value. CV is the spread.

### 2.5 Machine and toolchains

[measured, from the records' header]

- Machine: 13th Gen Intel Core i5-13450HX, 16 hardware threads, 31 GiB of
  memory, kernel 7.2.6-76070206-generic; a laptop shared with other agent
  sessions.
- Pinned toolchain (`rust-toolchain.toml`): rustc 1.99.0-nightly 771916f90
  (2026-08-08), LLVM 23.1.0; it runs the driver and builds every Tide arm
  and every `rust-*` arm.
- Stable toolchain: rustc 1.96.0 ac68faa20 (2026-05-25), LLVM 22.1.2, for
  `stable-release` only.
- Linkers: both rustc toolchains default to `gnu-lld-cc` with the bundled
  `rust-lld`; the driver links with `cc` 13.3.0 (Ubuntu 13.3.0-6ubuntu2~24.04.1)
  and GNU ld 2.42.
- Conditions: run through `heavy.sh` in a booked 45-minute quiet window
  (2026-11-18 03:00-03:45Z), at `nice 19`, `CARGO_BUILD_JOBS=4`, with the
  fleet memory cap raised to 8 GiB for the job (`HEAVY_MEM=8G`; the largest
  measured process peaked at 305 MiB). One-minute load 3.15 at the start and
  5.20 at the end. The run directory was on the build disk
  (`/mnt/build`), an NVMe drive.
- N: 30 rounds for every series (§1.6). Timed sizes as in §2.1; the
  simulator phase at 1,000,000 numbers, seed 0.
- The test-build run (`--pair contracts --phase test`, revision `c04dc44`):
  2026-11-18 04:25:35-04:29:54Z, **outside any quiet window**, so other
  sessions' work could run beside it. Through `heavy.sh` at `nice 19`,
  `CARGO_BUILD_JOBS=4`, `HEAVY_MEM=8G`. The one-minute load was 3.74 just
  before it started; the records' header, written after the tool's own
  compile and std builds, shows 15.37 (5-minute 8.89), and the end record
  4.64. Its measurements took 108 s. The largest process peaked at 3.7 MiB.
  Its CVs (0.81% to 5.9%) are of the same order as the main run's.

### 2.6 Raw records and rerun

- Raw records: `research/36-benchmarks/2026-11-18-bf6b958/records.jsonl`,
  284,127 bytes, SHA-256
  `6c7d946267e2d7110e48de0a6fc7b12969937534ce7adb8aa4e7066e5ed96c39`. It is
  the run's complete dataset: header, 1,800 measurement records, 60
  summaries, and the `end` record. Every number in §1 is a summary record's
  value or a quotient of two summary medians, except the per-round counts
  and paired ratios, which come from the measurement records; the built
  binaries are not kept.
- Test-build records:
  `research/36-benchmarks/2026-11-18-c04dc44-test/records.jsonl`, 20,224
  bytes, SHA-256
  `bf3c7456bd4abf1c580af48aa09215a25dabc3437f107dcb5324be42170c49bb`:
  header, 120 measurement records, 4 summaries (the test-build arm's with
  `sampled_clause_ran`), and the `end` record.
- **The measured revisions are not in the repository's history.** The main
  run measured revision `bf6b958` (tree `6e7f88141f48adf2d096ea6417150435a950b153`),
  the test-build run `c04dc44` (tree `6a882d0becd0564b3c8cd83b472a805e7d5c8ca3`).
  Both were earlier states of this note's single commit, later amended
  with review fixes, so neither commit is reachable once the commit lands.
  Both have the same parent as this commit, and they differ from it only in
  `crates/tide-admit/benches/pairs.rs`, `crates/tide-admit/tests/benchmarks.rs`,
  `benchmarks/README.md` and this note's directory. The benchmark programs,
  `rust-toolchain.toml`, the driver, Tide's std and macros, and the tool's
  support files (`benches/support/`, `tests/support/admitted.rs`) are
  byte-identical in all three.
- `pairs.rs` as measured: SHA-256
  `89a2e8b1cf3dd9c1b8ea17459e1f230122df23880c9c739783d988265cf39066` (main
  run) and `9a8da93f031ecfb952430b4bdeb19433b756a706dc0d1f00eaacbb7a8a7a2b5e`
  (test-build run). What changed in it after each run:
  - after the main run: the test-build phase, `--phase`, and the header's
    test-phase settings were added;
  - after the test-build run: the run phase builds missing programs (a
    no-op after a build phase, so a full run is unchanged), and its start
    was split into `start` and `open` for a regression test.

  The build, rebuild, run and simulator measurements are unchanged
  throughout, and the test-build measurement since it was added: the landed
  `pairs.rs` measures as the two measured versions did.
- Rerun, from the repository root (`benchmarks/README.md` gives the
  heavy.sh form this repository's agents use):

  ```sh
  cargo bench -p tide-admit --bench pairs -- --out /absolute/results/dir
  ```

  A full run took 29 minutes here (1,753 s) before the test-build phase
  existed; that phase adds about 2 minutes at 30 rounds. `--pair contracts
  --phase test` reruns it alone. `--quick` measures one round of every
  pair, phase and configuration at the programs' small default sizes:
  the same rows and columns as §1's tables, with N = 1 and no CV, for
  checking the tool, not for numbers.

## 3. Biases and limits

- **Linkers differ.** [repo evidence] The driver links with `/usr/bin/cc` and
  GNU ld (rustc's bundled linker off); the rustc arms link through `cc` with
  the bundled `rust-lld`. Every Tide build time includes GNU ld's link, and
  no configuration nets it out (a seventh, `rust-release-tide-link`, would;
  fx-6u8m.6's note). Run times are unaffected except through layout.
- **Std builds differ.** [repo evidence] Tide's std is built by Cargo in the
  arm's profile: in `tide-debug` its non-generic code is unoptimized and
  carries debug information into the link. Rust's std is always the
  toolchain's optimized build. So `tide-debug` against `rust-debug` also
  compares an unoptimized std with an optimized one.
- **Fixed order.** `tide-debug` runs first in every round, and round 0 of
  each phase follows the std builds or the previous phase. Medians absorb a
  first-round outlier; CV does not, so it can only add rounds.
- **Shared machine.** Runs go through `heavy.sh` at `nice 19` in a booked
  quiet slot, on a machine other sessions share; the header records the load
  at start and end.
- **Process start is in every time.** Run times include `timeout`'s start and
  the program's, about a millisecond; a simulated run also includes the test
  harness's start.
- **Run peak RSS is mostly `timeout`'s** (about 2.4 MiB) for programs that
  use less.
- **The rebuild edit is the lightest one.** One new statement in `main` lets
  rustc's incremental cache reuse almost everything; an edit to a widely used
  function would cost `rust-debug` more. It costs the driver the same, since
  any edit is a full build.
- **The simulator's cost is per traced event, not per baton pass.** The
  per-event figure (§1.5) divides the whole difference between simulated and
  real runs by the events traced, so it charges every event with the setup,
  the event store and the scheduler's work as well; not every event is a
  hand-off between threads. It is an upper bound on what each event costs,
  not a measured baton cost. Both arms run one seed, as one process each.
- **Runs at different sizes.** The simulator phase runs `tasks` at
  1,000,000 numbers, the run phase at 10,000,000, so the two phases' real
  runs do not compare with each other directly.
- **Four small programs.** They are written to isolate one cost each, not to
  represent applications.
