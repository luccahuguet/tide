# 6. Replay: what it reproduces, and what it doesn't

## The idea

Chapter 5 found a failing seed. **Replay** turns that failure into something
you can hand to anyone, or any agent, and get the exact same run back.

When the driver runs a test with `--tide-run-test NAME --seed N`, it records
two files:

- a **manifest** (`run.manifest.json`, format `tide-replay/2`): what's needed
  to rerun it, namely the seed, hashes of the source code, the compiler, the
  std and every library the build loaded, the build options, the test's name,
  and every input the run consumed (arguments, environment, stdin, files the
  test created);
- a **trace** (`trace.ndjson`): every event of the run, one per line. Every
  scheduling decision, channel message, and line of output, each with its
  source location.

`--replay MANIFEST` then does two things:

1. **The identity check.** Before running anything, it checks that the
   current source, compiler, std, loaded libraries, and build options are
   exactly the ones the manifest recorded. If anything differs, it refuses. A replay
   against changed code would be a different run pretending to be the same
   one.
2. **The comparison.** It reruns the test, feeding back the recorded choices,
   and compares the new trace and outcome with the recording, byte for byte.
   A match prints `replay reproduced`. A mismatch fails.

```text
  --seed 0 ──> run ──> run.manifest.json + trace.ndjson
                              │
  --replay manifest ──> identity check ──(differs)──> refuse, run nothing
                              │ (same)
                              v
                        rerun ──> compare traces ──> "replay reproduced" or fail
```

Two different things are easy to confuse: `--seed N` is a **fresh run** of
seed N on the current code. Use it after you edit, to check a fix.
`--replay` is the **identity-checked rerun** of one recorded run, and refuses
edited code.

## Run it

The demo's step 2 records seed 0 and replays it twice:

```text
== 2. Record seed 0, then replay the recording twice
cause: assertion `left == right` failed: lost hits
                 events  event trace       output trace
recording            40  9d66e1613fc69090  db1996770f19511f
replay-1             40  9d66e1613fc69090  db1996770f19511f
replay-2             40  9d66e1613fc69090  db1996770f19511f
output: views served: 2, hits recorded: 1
both replays reproduce the recording: identical event and output traces
recording: tide-demo/out/record/target/sim/run-…/run.manifest.json
```

Now replay it yourself, and then watch the identity check refuse an edited
source. Run `demo/run.sh` first. The manifest records only the libraries the
build loaded, so a later build that leaves them as they were, such as
`demo/run.sh`'s own `cargo build` commands, doesn't stop the replay. A build
that changes one of them does. A plain `cargo build` of the whole workspace
is one: it rebuilds Tide's std into different bytes, and replay refuses with
`dependency artifact mismatch` and the line `std: recorded …, current …`.
Running `demo/run.sh` again rebuilds the demo's std and records the seed
again.

The log prints each failing seed's replay command on a line starting
`  replay:`. Save it to a file, look at it, and run it:

```sh
bash -c 'grep -m1 "^  replay: " target/tide-demo/logs/record.log | sed "s/^  replay: //" > target/guide/replay.sh'
cat target/guide/replay.sh
bash target/guide/replay.sh
```

Near the top it prints
`replay reproduced: test=every_view_is_counted seed=0 outcome=panic: …`,
then the run's trace and the test's usual failure report. It exits 101, which means "the recorded
failing test failed again, the same way". (nushell may add a line saying the
command failed; that's expected.)

The demo's buggy revision is a generated copy in
`target/tide-demo/revisions/buggy/`, so editing it touches nothing in the
repository. Add a comment to it, and run the same replay again:

```sh
bash -c 'echo "// an edit" >> target/tide-demo/revisions/buggy/src/hits.rs'
bash target/guide/replay.sh
```

Output:

```text
tide-admit: replay paired-library source mismatch; use fresh --seed for changed inputs
  file:hits.rs: recorded hits.rs:0544f7bd5e14, current hits.rs:560b5107dea2
```

It exits 2 and runs nothing. A one-line comment was enough. Each changed
file gets one line, with the first 12 digits of its old and new hash.

Running `demo/run.sh` again regenerates the copy.

## What to look at

- In step 2, the three rows (recording, replay-1, replay-2) have the same
  event count and the same two hashes: the event trace and the output trace
  are byte-identical.
- The refusal names what changed (the source) and runs nothing. A replay is
  either the same run or no run.
- Open a trace and read a few lines:
  `bash -c 'head -5 $(find target/tide-demo/out/record -name trace.ndjson | head -1)'`.
  Each line is one event, with its kind (`sched`, `cell`, `sink`, ...) and
  source location.

## Read the code

**`demo/run.sh:107-141`** (8 minutes). The demo's step 2, in Bash: how it
records a seed, replays it twice, and proves the three runs match. You
don't need Bash to follow it with this walkthrough.

```sh
bash -c 'sed -n "107,141p" demo/run.sh'
```

- Line 107, `step "2. ..."`: print the step's heading.
- Lines 108–109: run the buggy suite with `--tide-run-test` and
  `--seed "$SEED"`, a fresh run of the failing seed, which records a manifest.
  `suite` is a helper defined earlier that calls `tide-admit`.
- Lines 110–111: collect the manifests written, and insist on exactly one,
  and on exit code 101 (a failing test).
- Lines 112–114: insist that the failure's cause is `lost hits`, not some
  other crash.
- Line 115: remember where the recording's trace is.
- Lines 116–126, `for n in 1 2`: twice, run `--replay` on that manifest. Each
  must exit 101 and print `replay reproduced`, and each must write one new
  manifest and trace of its own (lines 123–125).
- Lines 127–132: print the table: event count and two hashes (a hash is a
  short fingerprint of a file: same file, same hash) per run.
- Lines 133–135, `cmp -s`: compare each replay's trace with the recording's,
  byte for byte. Any difference fails the demo.
- Lines 136–139: pull the printed output line out of the trace and check it
  really shows fewer hits than views. Lines 140–141 print the summary.

What to take away: the script trusts nothing. It checks the exit codes, the
cause, the driver's own "reproduced" verdict, and the bytes of the traces,
separately.

## The hardest question

> "rr already records and replays any Linux program. And what does 'bit for
> bit' really cover?"

**rr vs Tide.** rr records one real execution at the system-call level and
plays it back for debugging. It's excellent, and it needs the run to happen
first. Tide's replay works on simulated runs: you **search** many seeds for a
failure, then replay the one that failed from a manifest of a few kilobytes,
with an identity check that refuses mismatched code. Search, then replay, is
the point.

**What it reproduces:** everything the simulator models (chapter 5). The
schedule, the simulated clock, files, local TCP, inputs, and the output, for
one test, on the same machine with the same pinned toolchain. The demo
shows it: identical traces three times over.

**What it doesn't, and you should say so:**

- **Same machine only.** Today's promise is the same machine with identical
  pinned compiler, std, and system C library. Replay across machines isn't
  promised yet.
- **Only what's modeled.** CPU time, real kernel behavior, and unmodeled
  effects are outside the trace, so a bug that lives there doesn't replay
  because the simulator never saw it.
- **Tests only.** Replay covers test runs started through the driver.
  Replaying a whole program (`tide sim --replay`) is planned. **Production
  record and replay** (record a real deployed run, replay it under the
  simulator) is v1.
- **Through the driver only.** Running a test binary directly records no
  manifest and gets no identity check.

## Going deeper

- `design/01-design.md` D15 (every simulated run writes a manifest), D17
  (production record and replay, v1).
- `design/03-authority-concurrency.md` §9.6 (REPLAY-1, the manifest; REPLAY-2,
  the identity check).
- The repository `README.md`, Status: the replay paragraphs list exactly what
  is checked.
- `demo/README.md`, "What the script does", step 2.
