# Tide and debuggers

Tide does not replace a debugger. A Tide program is Rust, compiled by the
pinned rustc against Tide's std (`design/01-design.md` D18), so gdb, lldb, and
the IDEs built on them see an ordinary Rust executable, built today without
debug information. This page covers what
Tide adds to debugging, and what it doesn't. It then shows how the two compose
on the demo's failing seed and how that compares with rr. It ends with a short
answer to "why not just learn debugging?".

## What each one gives you

- **A debugger** stops a run and shows its state: breakpoints, backtraces,
  variables, stepping. It answers "what is this run doing?", for a run you
  already have.
- **Tide's simulation** (`#[sim_test]`, D15) runs tasks under a scheduler
  that picks the next task at each scheduling point from a seed. A failure
  that depends on the schedule becomes a seed that fails every time, and the
  driver's `--replay` checks that a rerun matches the recording, trace and
  outcome.
- **Tide's trust diff** (`--tide-diff`, D11) summarizes what a change may do:
  the authority each function's signature declares, and its `unsafe` holes,
  before and after. A
  debugger shows what one run did. They answer different questions. We make
  no claim that the trust diff helps reviewers catch authority changes; the
  E3 pilot, with model reviewers on small changes, gave no evidence of a
  catch benefit ([`trials/e3-pilot/report.md`](../trials/e3-pilot/report.md)).

### When Tide adds little: the Graham-scan comparator

JetBrains' "Debugging Rust in 2026" [1] walks through a Graham scan whose
comparator sorts points of equal polar angle farthest-first (`b.cmp(a)`).
The failing unit test fails on every run, with no threads and no schedule
involved. Tide adds little here. There is no seed to find, and none of Tide's
checks flags a reversed comparator. Prints, `rust-gdb`, or RustRover's
debugger, as the article shows, are the right tools, in Tide as in Rust.

## Composing them: seed, replay, debugger

Tide's part is to supply a run worth debugging. The debugger's part is to
explain it. The demo (`cargo run -p tide-demo`, [`demo/README.md`](../demo/README.md))
does all three steps on a lost update in a small concurrent service:

1. **Find the seed.** Step 1 runs the buggy suite over seeds 0 to 15. Half of
   them lose a hit, and seed 0 is the first. Each failing seed prints a
   paste-ready `replay:` command in `logs/search.log` and `logs/record.log`.
2. **Replay it.** Step 2 records seed 0 and replays the recording twice, with
   byte-identical traces.
3. **Debug the replay.** Step 5 runs that `replay:` command under `rust-gdb`
   in batch mode, with breakpoints in `record_hit` and in its two
   `Shared::with` closures, the read and the write.

Step 5 needs a gdb built with Python. It uses `$RUST_GDB` when it is set,
else `gdb` on `PATH`. It is skipped, with a line saying so, when there is
neither, or when the gdb on `PATH` cannot run `python print(1)`. A
`RUST_GDB` that is set but does not work fails the step. The
whole debugger run, the driver it detaches included, must finish within 120
seconds; otherwise the step kills every process still in gdb's process group
and fails:

```sh
RUST_GDB=/path/to/gdb cargo run -p tide-demo
```

Its output on x86_64 Linux 7.2.6 with GNU gdb 17.2:

```text
== 5. Debug the replay of seed 0 under rust-gdb, with breakpoints in record_hit
sandbox: full (Linux 7.2.6-76070206-generic, Landlock ABI 10)
gdb: thread 4 enters record_hit
gdb: thread 3 enters record_hit
gdb: thread 4 reads hits = 0
gdb: thread 3 reads hits = 0
gdb: thread 4 writes hits = 0 + 1 over hits = 0
gdb: thread 3 writes hits = 0 + 1 over hits = 1
thread 3 wrote 0 + 1 over hits = 1: it read the count before another task wrote it, so that task's hit is lost
the replay under gdb reproduced the recording: the same schedule and trace
```

Both tasks read the count before either writes it back. That is the lost
update, seen in the running code rather than inferred from the trace. The
breakpoints do not change the run. The replay under gdb still passes the
driver's comparison with the recording, because the schedule comes from the
seed, not from timing.

Every demo run writes the gdb script, `GDB_SCRIPT` in
`crates/tide-demo/src/lib.rs`, to `$CARGO_TARGET_DIR/tide-demo/debug.gdb`
(default `target/tide-demo/`), even when step 5 skips. Its comment there says
which driver functions it follows and which registers it reads, which is what
you change to adapt it to other code. gdb's full output goes to
`logs/debug.log`. To run the same thing by hand after a demo run, from the
repository, with `gdb` on `PATH` or `RUST_GDB` set:

```sh
D=${CARGO_TARGET_DIR:-target}/tide-demo
eval "set -- $(sed -n 's/^  replay: //p' "$D/logs/record.log" | head -n 1)"
rust-gdb -nx -batch -x "$D/debug.gdb" --args "$@"
```

`rust-gdb` here is rustup's proxy, so `rust-toolchain.toml` selects the
pinned toolchain's script and its pretty-printers. Drop `-batch` to keep the
session open; we have run only batch mode.

### What works today, and the gaps

- **The sandbox does not block the debugger.** The driver runs the test
  binary in RUN-3's sandboxed child, in its own session, with a seccomp
  filter that denies `ptrace` to the child itself. gdb traces the child from
  outside, as its ancestor, and that works under `sandbox: full`. We have not
  tried attaching to an already running child by PID (`gdb -p`). With
  Yama's ptrace scope 1, this host's setting, only an ancestor may trace a
  process, and the child lives for milliseconds.
- **gdb has to follow the right process.** The driver runs itself again with
  a clean environment, and it then starts the test binary in a forked child.
  In between it may start the linker. The script follows only those two
  spawns. It recognizes them by two of the driver's function names,
  `supervise` and `Command::status`, which are internal and could change.
- **No debug information for Tide code.** The driver compiles Tide code
  without `-g`. Breakpoints work by symbol name (`tide_candidate::record_hit`,
  `tide_candidate::record_hit::{closure#1}`), and backtraces name functions,
  but gdb has no source lines and no named locals. The script reads the
  closures' arguments from their registers at entry, which holds for x86_64
  at opt-level 0 with the pinned compiler, and for nothing wider.
- **Threads are not tasks.** gdb numbers threads. The trace,
  `trace.ndjson` beside the manifest, names Tide's task ids.
- **Tested with rust-gdb only.** rust-lldb and RustRover have not been tried
  on a Tide replay. Either one would have to follow the driver into its
  child the same way.

### By hand or through an agent

The batch command above is what an agent runs: one command, plain-text
output, and a replay that ends the same way every time. Since RustRover
2026.3, the IDE's MCP server exposes its debugger to agents such as Codex and
Claude Code. The agent can start a session, set breakpoints, step, and
evaluate expressions [1]. We have not tried it on a Tide replay.

## rr compared

rr is the closest tool to Tide's replay. It records a run of an ordinary
Linux program and replays it deterministically under gdb, reverse execution
included [2]. Its trade-offs differ from Tide's:

| | rr | Tide's simulation |
| --- | --- | --- |
| What it applies to | unmodified Linux programs, whatever language they were written in [3] | Tide tasks under `#[sim_test]`; code outside the simulator is not covered |
| What it needs | the CPU's hardware performance counters [3] | the pinned toolchain; no special hardware |
| How a failing run is found | record runs until one fails; rr runs one thread at a time [3], and chaos mode randomizes that scheduling to make rare failures show up [4] | run a range of seeds; each seed fixes the schedule |
| What you keep | a trace directory; `rr pack` makes it movable, if the other machine's CPU supports what the program used [5] | a seed (a number) and, per evaluation, a `tide-replay/2` manifest; the demo's is 13 KB, including its 7 KB trace |
| Where it replays | the recording machine, or one whose CPU qualifies [5] | `--replay`: the same machine with the same pinned artifacts, for now (repository `README.md`); `--seed N` reruns the schedule on current source |

rr needs no rewrite, and it records real OS threads, system calls, and
signals [3], which Tide's simulator models only in part. A Tide seed is a
number, cheap to keep and share, and a fresh `--seed` run on the same source
and toolchain fails again, as the demo's step 2 shows. The two compose. A Tide replay is a
deterministic Linux process, so rr could record it, but we have not tried
that.

## "Why not just learn debugging?"

Learn it. Tide programs are Rust, so Rust's debuggers run on them. Today the
driver builds Tide code without debug information, so a debugger sees
function names and backtraces but no source lines or locals (see the gaps
above). In the JetBrains survey, 53.2% of Rust users use no debugger at all,
and stepping breaks with async code (28.1%) and macros (23.0%) [1]. Tide
changes none of that. What Tide changes is the input. A debugger needs a run
that fails, and a bug that depends on the order tasks run in may not fail on
the run you have the debugger on, since a debugger changes timing. In Tide's
simulation the schedule comes from the seed, so the failing run happens again
under the debugger, as step 5 shows. For a bug that fails on every run, like
the article's comparator, Tide adds little to what a debugger already gives
you.

## Sources

1. Orhun Parmaksiz, "Debugging Rust in 2026", JetBrains Rust blog,
   2026-11-10: <https://blog.jetbrains.com/rust/2026/11/10/debugging-rust/>.
   The survey figures, the Graham-scan example, and RustRover's MCP server.
2. rr project: <https://rr-project.org/>.
3. Robert O'Callahan et al., "Engineering Record And Replay For
   Deployability", 2017: <https://arxiv.org/abs/1705.05937>.
4. Robert O'Callahan, "Introducing rr Chaos Mode", 2016-02:
   <https://robert.ocallahan.org/2016/02/introducing-rr-chaos-mode.html>.
5. rr wiki, "Trace Portability":
   <https://github.com/mozilla/rr/wiki/Trace-Portability>.
