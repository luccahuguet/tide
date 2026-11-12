Source: `/home/lucca/.cache/eon-agents/tide-research-gate/report.md`, 2026-11-10; copied 2026-11-12.

# Research: E1 gate machinery (fx-rpwp)

Date 2026-11-10. Research only: no repository or `br` write, no builds. Under `trials/e1-checked/` I opened only `protocol.md`, `README.md`, `readiness.md`, `harness.py`, `harness_test.py`.

The brief says "each open design choice fx-08m faces"; I read that as fx-rpwp (fx-08m is the JSON codec).

Evidence labels: **[local]** = read or run on this machine (Claude Code 2.1.321, codex-cli 0.167.1, bubblewrap 0.12.0, Python 3.14.8); **[doc]** = fetched page, URL given; **[memory]** = my recollection, not fetched this session, verify before relying on it.

## 0. What the existing harness already fixes

- `command()` (harness.py:428-466) holds an exclusive `flock` on `session.json`, refuses when `runs >= 20` or `now - started >= 2400` (exit 2, "E1 budget exhausted"), stores the source before checking it, and bounds each command by `min(120, time left)`.
- `started` is set by `prepare()` (harness.py:412), so the 40 minutes run from preparation, not from writer launch (README says so too).
- Only one source file is read from the writer directory (`candidate()`), and each command runs in a fresh `snapshot-NN/run` tree, the only writable bind in the jail.
- `sandbox_command()` uses `bwrap --unshare-all --die-with-parent --new-session --cap-drop ALL`, a clean explicit environment, and read-only binds of the toolchain, `/usr`, `/etc/alternatives`, **all of `/nix/store`**, and for Tide the driver, std and `target/.../deps` directories.
- `grade` reads the *live* writer directory for the final snapshot; only run snapshots are stored.
- Exit 2 means both "budget exhausted" and "setup error" (README). The broker has to tell these apart.
- Nothing constrains the writer agent itself. That is fx-rpwp's main job.

## 1. Design choices

### 1.1 Writer tools: exactly file read/write in its directory plus logical `./check` and `./test`

**Claude Code** (the protocol's default writer, `claude-opus-5-5`).

Evidence:
- `--tools` selects the built-in tool set; `""` disables all [local `claude --help`]. It does not affect MCP tools [doc, summary: https://code.claude.com/docs/en/cli-reference].
- `--restricted` removes the command-running tools and WebFetch unless `--tools` names them, ignores user, project and local settings, confines the file tools to the working directories, and refuses `bypassPermissions` [local help; doc: https://code.claude.com/docs/en/cli-reference].
- `--strict-mcp-config` uses only the servers in `--mcp-config` [local help; same doc].
- `--permission-mode dontAsk` "Auto-denies every call that would otherwise prompt"; rules evaluate deny, ask, allow; MCP tools are allowed by `mcp__<server>__<tool>` names [doc: https://code.claude.com/docs/en/permissions].
- `--disallowedTools "mcp__*"`-style bare names remove tools from the model's context [doc: cli-reference].
- The Bash sandbox "covers shell commands only. Claude's file tools, MCP servers, and hooks run outside it" [doc: https://code.claude.com/docs/en/sandboxing]. So the built-in sandbox does not confine Read/Write, and an MCP broker runs unsandboxed, which is what a trusted broker needs.
- `--bare` skips hooks, skills, plugins, MCP discovery, auto-memory and CLAUDE.md, but never reads OAuth or keychain credentials, so it needs `ANTHROPIC_API_KEY` [doc: https://code.claude.com/docs/en/headless]. `--safe-mode` disables the same customizations with normal auth [local help].
- stream-json's `system/init` message "reports session metadata including the model, tools, MCP servers, and loaded plugins" [doc: headless].

Options:
- A. Claude Code CLI, built-in tools cut to `Read,Write,Edit`, plus one stdio MCP server exposing `check` and `test`.
- B. A, with the whole CLI process also inside an outer bwrap that hides the repository, reaching the broker through a Unix socket.
- C. A custom agent loop on the Messages API with three tools.

Recommendation: **A**, with three fail-closed checks, and B held in reserve.

```
cd <writer dir outside the repository tree>
env -i HOME=<fresh per-session config home> PATH=... <auth> DISABLE_AUTOUPDATER=1 \
claude -p --model claude-opus-5-5 --restricted --tools "Read,Write,Edit" \
  --strict-mcp-config --mcp-config <session>/mcp.json \
  --allowedTools "Read" "Write" "Edit" "mcp__e1__check" "mcp__e1__test" \
  --permission-mode dontAsk --permission-prompts none --disable-slash-commands \
  --safe-mode (or --bare with an API key) --no-session-persistence \
  --output-format stream-json --verbose  < prompt.md  > <session>/writer.jsonl
```

No `--effort` flag: the protocol says "the harness's default effort", and `--restricted` already ignores a user setting that would change it.

The three checks:
1. **Init assertion.** The coordinator reads the first `system/init` line and kills the session unless the tool list is exactly `{Read, Write, Edit, mcp__e1__check, mcp__e1__test}`, the MCP server list is exactly `e1` (connected), there are no plugins, and the model is the recorded one. A mismatch before any model turn is an infrastructure fault, not a trial.
2. **Log audit.** After the session, every `tool_use` in `writer.jsonl` must name one of the five tools, and every file path must resolve inside the writer directory. A violation flags the trial for fx-ic2 (the protocol has no "void" rule; see §4).
3. **Tool-surface probe.** Before the gate, one session with a non-task prompt asks the agent to read a repository file, read `/etc/hostname`, write outside its directory, fetch a URL, and run a shell command, and the log must show each refused or impossible. Flags such as `--restricted` are version-dependent, so this is the only proof the pinned CLI version behaves as documented.

Why A: it is the least code and uses documented switches. Why not B now: it needs a socket relay and a mount policy for the CLI's own install and credentials; build it only if check 3 fails. Why not C: the protocol records "one agent harness ... at the harness's default effort", which reads as an existing harness, and C makes our own loop the thing under test.

Two details that matter:
- **Writer directory outside the repository.** `prepare` takes any directory; today's examples put it under `target/`. Claude Code discovers CLAUDE.md and project settings from the working directory upward [memory], and auto-memory is keyed by project directory [local: this session's memory path]. A directory under the checkout risks loading Tide's own instructions. Use e.g. `~/.cache/tide-e1/<run>/w/<slot>-<attempt>` with opaque slot names.
- **Naming.** MCP tool names cannot be `./check`. Name them `check` and `test` and let the tool description say "Runs ./check as your instructions describe". The prompt stays the preface plus the task, unchanged (§4.2).

**Codex CLI** (only if fx-ic2 records another writer).
- Codex reads files through its shell; there is no separate read tool. `features.shell_tool` is "Enable command execution tools (stable; on by default)", and the docs do not say what `false` leaves [doc: https://learn.chatgpt.com/docs/config-file/config-reference]. `read-only` mode still "can read files and run commands within the read-only sandbox" [doc: https://learn.chatgpt.com/docs/agent-approvals-security].
- `web_search = "disabled"` turns search off; MCP servers take `enabled_tools`, `disabled_tools`, `tool_timeout_sec` (default 60) [doc: https://learn.chatgpt.com/docs/extend/mcp?surface=cli]. `--ignore-user-config`, `--ephemeral`, `--skip-git-repo-check`, `--json` exist [local `codex exec --help`]; `turn.completed` carries token usage [doc: https://learn.chatgpt.com/docs/non-interactive-mode].
- So a protocol-conforming Codex writer needs `features.shell_tool=false`, search disabled, and a broker that also serves `read_file`, `write_file` and `list_dir` confined to the directory. Note `tool_timeout_sec` must exceed 120 s.

Recommendation: **do not build the Codex path** in fx-rpwp. The protocol names one writer; keep the broker's tool table small enough that file tools could be added later.

### 1.2 Broker

Options: (a) stdio MCP server, Python standard library; (b) HTTP MCP server run by the coordinator; (c) executable `./check` files in the writer directory (README forbids running writer-editable files outside isolation).

Recommendation: **(a)**. About 80 lines: `initialize`, `tools/list`, `tools/call` over newline-delimited JSON-RPC, no dependency.

- The coordinator writes `mcp.json` with the session path in the broker's argv. The tools take **no arguments**, so the writer cannot name another session or mode.
- A call runs `python3 harness.py command SESSION check|test` with stdin closed, `start_new_session=True`, and a 150 s guard timeout (the harness already bounds itself to 120 s).
- Output is stdout then stderr, verbatim, plus one `exit: N` line. §4.1 says diagnostics print "as they are", so add nothing else.
- **21st run and 40 minutes**: the harness is the single owner of both rules (AP-OWNERSHIP-001); the broker only relays. Give the budget refusal its own exit code (say 3) so the broker and coordinator never confuse it with a setup error at exit 2, and return it as an MCP error result with the harness's message.
- The broker appends `{time, mode, exit, seconds}` to `<session>/broker.jsonl`.
- **Output size.** Claude Code caps MCP output with `MAX_MCP_OUTPUT_TOKENS`, default 25,000 tokens [doc, summary: https://code.claude.com/docs/en/mcp]. A long rustc error list could hit it. Set the variable high (e.g. 200,000) and have the broker cap at a fixed byte size with a marker line, the same in both arms, logging each truncation.
- **Session clock.** Have the coordinator set `started` under the flock at the moment it spawns the writer, and prepare immediately before. Otherwise the Tide arm's driver build in `base_config` and CLI start-up are charged unevenly.
- **End at 40 minutes.** The coordinator waits `started + 2400 - now`, then kills the CLI's process group. Because the harness command runs in its own session and is already limited to the time left, it finishes and writes `results.json`; the coordinator takes the flock once before grading to be sure.
- **"Writer says it is done"** is `claude -p` exiting 0 with a `result` message. No `--max-turns`.

### 1.3 Coordinator

**Order.** §3: the triples "sorted, shuffled by `random.Random(20261031).shuffle`". The text does not say how task names sort. Python's default sort is lexicographic (`A1, A10, A2`), while `harness.TASKS` is natural order. The two give different permutations [local]:

| Reading | First four slots | SHA-256 of `json.dumps(order)`, first 16 hex |
| --- | --- | --- |
| `sorted()` of `(task, arm, replicate)` tuples, arms `rust`/`tide`, replicates 1/2 | C1 rust 2, B6 rust 1, A9 tide 1, A10 tide 1 | `9d0951c1c2301efa` |
| natural task order | C1 rust 2, B7 rust 1, A10 tide 1, A2 tide 1 | `cc8946a62f32faa0` |

Recommendation: the literal reading, plain `sorted()` on tuples. "Sorted" in a sentence that names Python's shuffle means Python's sort. Arm case and replicate numbering do not change the permutation. Pin the first slots and the hash in `harness_test.py`, and record the full order on fx-ic2 before the first session.

**Batching and concurrency.** The protocol asks for batches that bound the coordinator's context; it does not mention parallel sessions. Parallel sessions put load on the 10 s test limit, which fx-ic2's operations comments already flag. Inspect separates wall time from "working time" for the same reason [doc: https://inspect.aisi.org.uk/setting-limits.html]. Recommendation: the coordinator is a plain Python script, not an agent; `run --batch N` executes the next N unfinished slots from a state file and prints one line per slot (slot, end reason, runs, grades). A `--jobs` value, default 1, is fixed for the whole run and recorded. Slots start in order; the shuffle spreads load across arms.

**Infrastructure faults.** Prior art: SWE-bench reports "instances with errors" (no result could be produced) apart from "unresolved" [doc: https://github.com/SWE-bench/SWE-bench/blob/main/docs/guides/evaluation.md]; Inspect's `retry_on_error` retries errored samples and keeps the original errors in `error_retries` [doc: https://inspect.aisi.org.uk/handling-errors.html].

Recommendation, a closed list:
- Fault: init assertion failure; CLI exit non-zero or an error `result` (API error, crash); broker crash; harness exit 2 setup error or bwrap failure; coordinator or host interruption.
- Not a fault: budget refusal, the 40-minute kill, the writer stopping early, wrong or empty output.
- The rerun happens immediately in the same slot with a fresh directory and session; the faulted attempt is kept on disk, marked, and never graded. A second fault records the trial as failing first check and final. The run summary counts reruns.
- Each fault record states the trigger and the evidence line, so a grader can dispute the classification.

**Grading.** After each session: copy the final source to `<session>/final/source.rs` with its SHA-256, make the writer directory read-only, then run `grade SESSION --first --compile-only` and `grade SESSION`. Storing the final source is what lets §8's "commands that reproduce its grading from the stored snapshots" work; today the final grade depends on a live directory. First-check pass is the exit code of the `--first --compile-only` grade, which re-checks the first source whatever mode the first run used; a session with no run fails through the existing missing-snapshot path.

**Per-trial record** (`trial.json`): slot, attempt, end reason, run count, wall time, turns, token sums by kind, peak context (largest per-message input + cache read + cache creation), Tide reference sections read (from `Read` offsets), both grades, integrity flags, audit result.

### 1.4 Isolation canaries

One command, `harness.py canary ARM`, that runs a probe through the same `run()` and `sandbox_command()` path writer code takes, once per jail shape: compile jail, test-execution jail (`read_only=[binary]`), and binary-case jail with a case environment.

The canary plants random tokens first: a file in a fake second session directory, an environment variable in the coordinator's own environment, and TCP listeners on the host's loopback and on its LAN address. The probe then tries:

| Forbidden access | Probe | Must see |
| --- | --- | --- |
| Repository file | open `ROOT/AGENTS.md`, `ROOT/design/reference.md` by absolute path | ENOENT |
| Suite, reference, task file | open paths under `HERE/suites`, `HERE/references`, `HERE/tasks.md` (by path only; the canary never prints content) | ENOENT |
| Other session | open the planted file; list `STATE/sessions` | ENOENT / empty |
| Network | connect to both host listeners; resolve a name | ENETUNREACH / refused; no token received |
| Inherited environment | print the environment | planted variable absent |

Fail closed means four things:
1. **Positive controls inside the jail**: the probe reads a file in its own work tree, and binds and connects on its own loopback (C10 needs this). A probe that did not run, or a jail too tight to work, fails.
2. **Negative control outside the jail**: the same probe with `sandbox: False` must reach every token. This proves the probe can detect a leak.
3. A missing sentinel line, an unexpected errno, or any token in the output is a failure. No "skipped".
4. The coordinator refuses to start a non-dry run unless a canary record exists for the current harness revision, bwrap version and boot ID, in both arms.

A shell or Python probe from `/usr` is enough: the canary tests the jail, and the two arms differ only in mounts. Tide code could not perform these accesses without grants anyway.

### 1.5 The Nix store holds no repository copy

The whole store is readable in every jail. Local facts: the repository has no `flake.nix`, `shell.nix`, `default.nix` or `.envrc`; the store has 58,256 entries, 703 named `*-source`; none has `trials/e1-checked`, `design/01-design.md` or `design/reference.md` at depth 1 [local]. That is a first look, not proof.

Options: (a) scan the store by content; (b) stop binding the store, or bind only the closure the toolchain needs.

Recommendation: **(a) now**. One pass over the store listing file sizes; for files whose size equals a tracked file under `trials/e1-checked/` or `design/`, compare SHA-256. Any match fails the canary. Run it before the first session and after the last, since the store can gain paths mid-run, and record both results. `design/reference.md` is included because the Rust arm must not see it. Option (b) is the smaller surface, and worth one experiment: if `test_jailed_toolchain_executes_and_links` passes without the bind on the gate machine (rustup toolchains, `/usr/bin/cc`), dropping it removes the question. It changes a mount Opus approved in fx-o8o, so record the decision on the bead.

### 1.6 Rust arm: B9 fixtures on tmpfs

`environment()` sets `TMPDIR` to the work directory, and `test_command()` uses a temporary directory inside it. Sessions live under `target/e1-checked/sessions/`, which is ext4 on the NVMe disk here [local `df -T`]; `/dev/shm` is a 16 GB tmpfs. So today fixtures hit the disk, as the bead comment suspects.

Options:
- A. In test-execution jails add `--size <bytes> --tmpfs /tmp` and set `TMPDIR=/tmp` [bwrap options: https://github.com/containers/bubblewrap/blob/main/bwrap.xml].
- B. Move all session state to `/dev/shm`.
- C. Prove the fsync exposure gone on the gate machine.

Recommendation: **A, in both arms' runtime jails** (writer tests, doctests, acceptance tests). It keeps one jail shape, costs nothing for Tide (its B9 uses the simulated file system), and also covers a Rust writer's own fsyncing tests. Keep the libtest logfile where it is, in the host-visible execution directory. B puts per-run build trees in RAM with several sessions at once. I could not check that the fixture helper uses the process temporary directory, because the suites are held out; the canary settles it: inside the Rust test jail, report the filesystem type of `$TMPDIR` and require `tmpfs`, and add a harness test that a B9-shaped run passes `TMPDIR=/tmp`.

### 1.7 Identity record

One JSON file written by `harness.py identity`, shown again at every session start:

- Tide: `git rev-parse HEAD`, the tree hash of `crates/`, `rust-toolchain.toml` channel, `rustc -Vv` of the nightly, SHA-256 of root `Cargo.lock`, of the built driver and of the `tide_std` rlib.
- Rust: `rustc -Vv` and `cargo -V` for 1.96.0, SHA-256 of `rust/Cargo.lock` (README names this as the dependency identity).
- Documents: SHA-256 of `design/reference.md` and the companion (§3: byte-identical to the recorded revision).
- Harness: blob hashes of `harness.py`, `doctest-runner.sh`, broker and coordinator; the suites' tree hash once fx-ic2 freezes them.
- Writer: `claude --version`, the full argv, the model string the API returns in `system/init` and in each assistant message, effort "default (flag not passed)".
- Host: kernel, bwrap version, Python version, CPU count, `--jobs`, seed, order hash, canary and store-scan results.

Fail closed: `identity` refuses a dirty tree for `crates/`, `trials/e1-checked/`, `rust-toolchain.toml`, `Cargo.lock` and the two reference documents. Before each session the coordinator recomputes the cheap fields (HEAD, driver hash, CLI version, model) and stops the run on a change, since §5 says a mid-run toolchain change voids it. Pin the CLI and turn its auto-update off for the run (`DISABLE_AUTOUPDATER=1` [memory]).

### 1.8 Dry run with a scripted stand-in writer

The stand-in replaces the model only. It is a small MCP client that spawns the broker from the same `mcp.json` the CLI would get, writes the source file into the writer directory, and calls `check` and `test`.

- Tasks: two per arm from those whose references already run furthest in both arms; readiness.md lists A2, A3, A4 and A10 as passing Tide source checks. The script takes sources through `harness.source()` and never prints them.
- Script: write a deliberately broken source, call `check` (first snapshot fails first check); write the reference, call `test`; then call `check` until the 21st call and assert the refusal.
- 40 minutes: a `--dry-run` flag is the only way to shorten the session length (e.g. to 20 s); the stand-in sleeps past it and asserts both that the next call is refused and that the coordinator ended the session.
- Dry-run sessions go to a separate state directory, carry `"dry_run": true` in every record, and the gate summary refuses to count them.
- Tide final grades will fail until fx-m72.3 replaces the R placeholder; the acceptance asks only that both grades are produced.

## 2. Pitfalls

**Tool leakage through the agent CLI**
- MCP servers from user or plugin configuration, skills, hooks and subagents load unless excluded; this very research session received an unrequested MCP server mid-run. `--strict-mcp-config`, `--safe-mode`/`--bare` and the init assertion cover it.
- The Read tool is not sandboxed [doc: sandboxing page]; confinement rests on `--restricted` and `dontAsk`. Hence the probe and the log audit.
- Session transcripts and auto-memory persist under the user's home and could carry one writer's program to another session or to a Tide implementer. Use a fresh config home per session and `--no-session-persistence`.

**Network**
- The writer process needs the model API, so it is not network-isolated; only its tools are. WebFetch and WebSearch must be absent from the init tool list.
- Abstract Unix sockets and host loopback are cut by the network namespace; the canary's host-loopback listener checks it.
- Inspect's generated compose file defaults to `network_mode: none`, and a custom file silently replaces it [doc: https://inspect.aisi.org.uk/sandboxing.html]. The same failure exists here: any new `bwrap` call that omits `--unshare-all`. Keep one function that builds jail arguments.

**Shared caches**
- Builds are fresh per run, so nothing leaks between sessions. The cost is that the Rust arm compiles its three crates in every run inside the 120 s limit while Tide links a prebuilt std. That is the frozen design, but the dry run should record Rust's fixed compile time, and fx-ic2 should know it.
- `STATE/rust-build` is shared in `--local` mode only; the coordinator must never pass `--local`.
- Session names come from two path components; duplicate slot names would collide. Use `<slot>-<attempt>`.

**Clock and budget**
- `time.time()` jumps with clock steps; record a monotonic reading beside it and flag a session where they disagree by more than a second.
- API retries and rate-limit waits consume the writer's 40 minutes. Record them; whether heavy throttling is a fault is §4.
- The run counter increments before the command runs, so a harness crash consumes a run. That session is a fault anyway.

**Contamination through logs**
- The coordinating agent must see only the one-line summaries. Raw `writer.jsonl`, snapshots and diagnostics hold writer programs and task text; §2 forbids them reaching the reference, companion or diagnostics. Keep them outside the checkout and out of every agent brief.
- ImpossibleBench and METR report agents editing tests or exiting early to pass [research/13, E1: https://arxiv.org/abs/2510.20270 [memory for the URL]]. The harness already keeps suites out of reach and flags forging constructs; Harbor likewise uploads tests only "after the agent runs" [doc: https://docs.harborframework.com/tasks/verifier], and METR's standard gives the agent only the instructions [doc: https://github.com/METR/task-standard/blob/main/STANDARD.md].

## 3. Cheapest falsifying tests

| Recommendation | Test |
| --- | --- |
| Tool set (1.1) | Unit: init-assertion function rejects a list with one extra tool or server. Live: the non-task probe session; any successful forbidden call falsifies. |
| Broker (1.2) | Stand-in: tools accept no arguments; 20 calls pass through, the 21st returns the distinct refusal; a 1 MB diagnostic is capped with the marker. |
| Clock (1.2) | With a 20 s dry-run limit, the coordinator kills a sleeping stand-in within a second of the limit and a command in flight still writes `results.json`. |
| Order (1.3) | `harness_test.py`: order hash `9d0951c1c2301efa…`, 120 unique triples, 60 per arm. |
| Rerun (1.3) | Fake runner: fault then success gives one counted rerun and a graded second attempt; two faults give a failed trial; a budget refusal gives no rerun. |
| Grading (1.3) | Mutating the writer directory after session end does not change the final grade; a no-run session fails first check. |
| Canaries (1.4) | The unsandboxed control reaches every token; removing `--unshare-all` or adding a `--ro-bind` of the repository in a test double makes the canary fail. |
| Store (1.5) | Plant a copy of one tracked file in a temporary directory scanned as a fake store: the scan reports it. |
| tmpfs (1.6) | Inside the Rust test jail, `$TMPDIR` reports `tmpfs`; `sandbox_command` for runtime jails contains `--tmpfs /tmp`. |
| Identity (1.7) | Touching a file under `crates/` makes `identity` refuse; changing the recorded CLI version stops the coordinator before the next session. |
| Dry run (1.8) | Dry-run records are rejected by the gate summary. |

## 4. Where the text is silent, and what I recommend

1. **Sort order of the triples** (§3). Use Python's default tuple sort; record order and hash on fx-ic2. This is the one ambiguity that changes the experiment.
2. **When the 40 minutes start** (§4.2 says "wall time" per session; README says from preparation). Start at writer launch.
3. **Parallel sessions.** Not mentioned. Default to one at a time; any other value is fixed for the run and recorded.
4. **What counts as "no act of the writer"** (§3). Use the closed list in 1.3. Open edge: a session throttled by the API for a large share of its 40 minutes but not failed. I recommend counting it as a fault only if the CLI ends with an error, and recording retry time per trial so the report can show it.
5. **A trial whose log audit shows a tool outside the allowed set.** The protocol has no rule. Recommend treating it as an infrastructure fault (the harness failed to isolate), with the usual single rerun, and noting it on fx-ic2.
6. **How the writer is told about `check`/`test`.** The prompt is fixed; the tool description is harness text. Keep it to one neutral sentence, identical in both arms.
7. **Output truncation** (§4.1 "as they are"). Any cap is a deviation; use a high one, identical in both arms, and count its uses.
8. **Rust-arm tmpfs.** fx-ic2's comment says it "stays within the protocol"; applying it to both arms keeps the jails symmetric.
9. **The tool-surface probe uses a real model session.** Acceptance says "No real gate writer runs". A probe with a non-task prompt is not a gate writer, but it spends model credit; the coordinator should confirm that reading with the owner's standing rules before running it.
10. **Problems in the frozen protocol** (items 1, 2 and 4 above) go on fx-ic2 as comments, per fx-rpwp's Scope; none needs a protocol text change beyond the two already decided.

Not verified this session: Harbor's and Terminal-Bench's retry rules, tau-bench's seed and trial flags (its README no longer documents them: https://github.com/sierra-research/tau-bench), Inspect's limit pages beyond search summaries, the JSON `result` field names for turns in Claude Code, and whether `features.shell_tool=false` leaves Codex able to edit files.

DONE
