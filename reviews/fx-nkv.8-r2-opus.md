Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate abc4b31

# fx-nkv.8 Opus review r2

## Verdict

request_changes. There is one blocking finding: steps 1–2 of `demo/run.sh` never check that the failure they report is the lost update. Everything else holds up. The finding has a small, local fix in a file this change adds.

## Scope

- `git diff 5029846...abc4b31`. I confirmed its SHA-256 is ea75423a0abee2d7206f72b554f78217fe41f83d28b7dcd6f67ba9ba7fc3b77a. It adds `demo/run.sh`, `demo/service/src/hits.rs`, `demo/tests/src/hits_sim.rs`, both patches, `demo/README.md` and `demo/agent-card.md`, and one paragraph in `README.md`.
- The contract is the Acceptance of fx-nkv.8 and of fx-nkv, read with `br --no-db show`. I applied AGENTS.md and the GOVERNANCE.md Merge Rule tier rule (line 48).
- Run evidence: `report.md` and `demo-c2.log` (fresh user, `CARGO_TARGET_DIR` unset, exit 0 on abc4b31).
- Production code I read to check claims:
  - `crates/tide-std/src/test/runner.rs`: the seed loop, the determinism self-check, and the failure and summary lines.
  - `crates/tide-std/src/sim/manifest.rs`: how replay renders and compares the trace and when it prints `replay reproduced`.
  - `crates/tide-admit/src/invocation/replay.rs`: the build identity and the replay exit statuses.
  - `crates/tide-std/src/sync.rs`: `Shared::with`, `Pure`, and `acquire_set`.
  - `crates/tide-std/src/net.rs`, `crates/tide-std/src/task/net.rs` and `crates/tide-std/src/lib.rs`: which sinks are ambient.
  - `crates/tide-macros/src/sim_test.rs`.
  - `conformance/rustc/std/src/lib.rs`: the stub's includes of production source.
  - `rust-toolchain.toml` and `.gitignore`.
- I ran nothing. This review was read-only.

## Blocking findings

### B1. Steps 1–2 accept any schedule-dependent test failure as "the bug"

Location: `demo/run.sh:84`, `:101`, `:123`.

What the script checks:
- **Step 1** checks only that the suite exits 101, that at least one seed passed, and that some `simulation failure seed=N` lines exist.
- **Step 2** checks only that the `--seed` record run exits 101 and wrote one manifest.
- **The cause line** (`:101`) is printed under `|| true`, so it is optional.
- **The output line** (`:123`) is printed and never compared.

So no line of the script establishes that the failing seed lost a hit. The guide promises it does (`demo/README.md:20`: "exits 0 when every step shows what it should"; step 1 is titled "Find a seed that exposes the bug"). fx-nkv's Acceptance asks for "a seed that exposes a logical concurrency bug".

Failure scenarios:
1. **Wrong panic.** Under some schedules, the buggy revision fails for a reason other than a lost update. For example, `serve(...).unwrap()` panics on a `TaskError` or `LockError::Poisoned`, or another assertion fires. A later edit to `lost-update.patch` or the suite could cause this. So could the planted library changes the report expects from E3 (report decision 1). The demo still exits 0, prints "first failing seed: N", and replays that panic identically. The only clue is unchecked text.
2. **Self-check failure.** `runner.rs:396-418` turns a seed whose trace differs between two evaluations into a failure with cause `determinism self-check failed`. Step 1 counts such seeds in `failing seeds:` and in the `8 failed` summary, exactly like lost updates. The guide then says these seeds "interleave the two read-write pairs and lose a hit" (`demo/README.md:148`). Step 2 inspects only the first failing seed, so the rest of the printed set is never checked.

Fix: check what step 2 already prints. All three checks read existing log or trace lines.
- `record.log` has `cause: assertion `left == right` failed: lost hits`.
- The recorded output from `output_of` shows fewer hits than views.
- Step 1: the number of `  cause: ... lost hits` lines in `search.log` equals the `failed` count, so no failing seed failed for another reason.

Why it blocks: the fix changes `demo/run.sh`, which this change adds. That is the change's own tier (v0 demo), so under GOVERNANCE.md's Merge Rule it blocks rather than going to fx-djp. It is not a nit. This script is the bead's done condition, and fx-qlek.9 will port it into a workspace test. The check needs to prove the claim it prints, not only that something failed.

## Non-blocking findings

- **N1. `demo/agent-card.md:39` is false as written.** It lists "network I/O beyond TCP loopback" as planned. Production `Net` already does real TCP to real hosts (`crates/tide-std/src/net.rs:1-5`, `:185`). Only *simulated* networking is loopback-only. The guide has it right at `demo/README.md:214` ("simulated network I/O beyond TCP loopback"). The card should say "simulated" too. Text-only.
- **N2. `demo/README.md:111` overstates.** It says "`serve` can touch nothing but its tasks". Tide's `out`/`err` sinks (`println!`, `eprintln!`) are ambient (`crates/tide-std/src/lib.rs:15`), and the suite itself uses `println!`. The rest of the sentence is accurate: it holds no capability but `&Tasks`, so it can neither read files nor open sockets. Suggested wording: "`&Tasks` is its only capability". Text-only.
- **N3. `demo/README.md:32-33` has two small inaccuracies.**
  - "Logs of every driver run are in `tide-demo/logs/`": the `--tide-diff` run goes to stdout, not a log.
  - "After the first build it takes a few seconds": the builder measured about 18 s.
  Text-only.
- **N4. `demo/agent-card.md:15` hard-codes `D=target/debug/tide-admit`.** Line 5 of the card says the script honors `CARGO_TARGET_DIR`. Suggest `$CARGO_TARGET_DIR/debug/tide-admit (default target/)`. Text-only.
- **N5. The revision label misses untracked files.** `demo/run.sh:33` uses `git diff --quiet HEAD`, which ignores untracked files, so an added but uncommitted demo file still prints a clean revision. fx-nkv asks for the exact revision setup. `git status --porcelain` would be stricter. Minor.
- **N6. Wrong error message.** In `demo/run.sh:39-41`, if `grep` finds no `filenames` the error says "cannot build Tide's std" even though cargo succeeded. It still fails closed with exit 2. Cosmetic.

## Reasoning

**What the script does prove, with no shortcut:**
- **Schedule dependence.** Step 1 requires `passed > 0` and at least one failing seed under one fixed input. `(( passed > 0 ))` fails closed on an empty parse. `#[sim_test(seeds = N)]` makes simulation seeds the literal indices 0..N (`runner.rs:362`: `if config.simulation { index }`). So "seeds 0 to 15" in step 3 is an explicit set, and it is the same set step 1 searched.
- **Replay is real, not a copy.** In replay, `manifest::finish` renders the trace from that run's own events. It asserts outcome, roots and trace equality against the manifest, writes a fresh `trace.ndjson`, and only then prints `replay reproduced:` and writes the completion file (`manifest.rs:196-252`).
  - The driver returns 101 without a reproduction check on a timeout or signal (`replay.rs:263-271`). The script closes that hole by requiring both exit 101 and the `replay reproduced: test=… seed=…` line (`run.sh:108-109`). That was the r1 fix, and it is correct.
  - The new-manifest check (`comm -13`) and the byte `cmp` against the recording are independent of the driver's own check. Identical bytes across three runs also show the trace carries no run-specific paths.
  - The output trace is the `sink` subset of the same file, so it is covered by the same `cmp`. That matches the builder's account that simulated `println!` goes to sink records.
- **Replay identity.** The driver hashes the source, the paired library, the std rlib, every dependency directory, `/usr/bin/cc` and `/usr/bin/ld`, the compiler ID, the semantic options, the driver binary and the wire versions (`replay.rs:11-56`). The guide's sentence "sources, build options, compiler, std, and test configuration match" is accurate.
- **Fixed revision.** `" 0 failed ("` on exit 0, plus seed `$SEED` on its own with exit 0. The fix is correct: one `with` does the read-modify-write while holding the cell, and `acquire_set` serializes it. The bug is real and logical: two separate `with` calls, each a simulation scheduling point through `SimCell::begin_set` and `record_set`, with `Pure` closures and no data race. The 8/16 split shows the dependence on the schedule.
- **Authority diff.** Both sides are admitted from the same service file. The fixed file is the base, and the head is that file with `metrics-report.patch` applied. A patch that stops applying is a setup error. The script requires the exact lines `~ hits::serve (function) widened` and `+ metrics: net`. The widened revision is never run, which fx-nkv explicitly allows.
- **Statuses.** Every driver status is captured and compared. `revision()` runs in a command substitution, and its exit 2 propagates through `set -e`. The remaining unchecked pipelines (`hash_of`, `grep -c`, the cause line) are display-only. B1 covers the cause line.

**fx-nkv's std rule.** It is met by construction. For every item the demo exposes, the stub's modules `include!` or `#[path]` production's own source: `task.rs`, `sync.rs`, `sim.rs`, `test/mod.rs`, `net.rs` and `io/mod.rs` (`conformance/rustc/std/src/lib.rs:46-137`).
- The stub adds `Net::get`, `drop_all`, `NetError`, `Url`, and `SimRoot::node`/`run_for`. The demo uses none of them.
- The builder's one-off check found the same verdicts and artifacts under both surfaces, differing only in the std crate name.
- The guide's table and its sentence on the stub additions are accurate.

**Docs.** The guide's runs-today and planned lists match the driver and std as they are. The `tide test … --seed N` line is correctly labelled as the planned frontend. The sample output matches `demo-c2.log`. The README paragraph is accurate. N1 is the only false sentence I found. It understates rather than overstates support, so it is non-blocking but should be fixed with B1.

**Leftovers.** No crate, conformance, design, governance or `.beads/` file changed. `target/` is git-ignored, so the default work directory leaves a clean checkout clean. Nothing is orphaned.

DONE
