# E3 pilot: frozen protocol

Status: frozen by bead `fx-1vwf.1` (2026-11-17), before any plant exists.
Bead `fx-1vwf.2` writes the plants and their clean twins; bead `fx-1vwf.3`
runs the reviews, grades them, and publishes the report. D0
(`design/01-design.md`) owns the threshold; §10 states it and changes
none.

## 1. The question and what the pilot can conclude

D0's E3 asks whether reviewers detect planted authority-widening changes
better with the trust diff than with the plain code diff, and sets the
initial threshold: reviewers using the trust diff catch at least 1.5x as
many planted authority-widening changes as with the plain diff.

- The pilot measures that ratio for two model reviewers on one codebase,
  with 12 plants and 12 clean twins, and states it against 1.5x as met or
  not met, whichever it is (§10, §12).
- It is a pilot: model reviewers only, no human reviewers, one small
  codebase, v0's partial trust diff (declared authority of exported items
  only, §5). A met threshold is a result for this setting, not a general
  claim about review with Tide.

## 2. Freeze rules

- This file, `prompt/`, and `harness/` are frozen when `fx-1vwf.1` lands.
  From then on the arms, the prompts (the adjudication prompt included),
  the budget, the models, the sample, the seed, the grading rules, the
  threshold, and the statistics do not change.
- `fx-1vwf.3` implements §9 and §10 (grading, adjudication, statistics) as
  harness commands that do not change `schedule`, `render`, `review`, or
  the prompts, and lands them, reviewed against this file, before the
  first review session. So that adjudications run through review's path
  (§9.2), it may pass the reply parser into `attempt`, provided the
  harness's existing tests pass unchanged and the prompt hashes stay the
  same. Its grading command checks the manifest it is given against §13's
  SHA-256 before reading it.
- Before the first review session, a change the run cannot proceed
  without (a CLI flag that no longer exists, say) is recorded on
  `fx-1vwf.3` with its reason, and it may not draw on any plant's content.
  After the first review session nothing in this list changes; a needed
  change voids the run (§11).
- `fx-1vwf.2` freezes the codebase revision, every item, and the sealed
  manifest, and appends the manifest's SHA-256 to §13 on `main`, before
  the first review session. No item changes after that.
- The planting agent of `fx-1vwf.2` takes no part in `fx-1vwf.3`: it
  neither reviews nor grades. Its model and session are recorded on
  `fx-1vwf.2`. Reviewers are fresh sessions that have seen nothing but
  their one prompt.
- Nobody computes a catch count, a false-positive count, or any other
  result from verdicts before every session has its final outcome (§11).
- The pilot's runs all start with `e3 schedule` from the harness as
  landed. The dry runs made while `fx-1vwf.1` was developed are
  development artifacts, not runs; nothing is graded from them.

## 3. Codebase

The codebase is about 1,900 lines of Tide that the admission driver
admitted against production std before this experiment existed, so no one
wrote it with E3's plants in mind:

| Programs | Lines | Origin |
| --- | ---: | --- |
| `demo/service/src/hits.rs` | 58 | the demo service (`fx-nkv.8`), the default |
| `trials/e1-checked/references/tide/*.rs` (22 files) | 1,194 | E1's Tide reference solutions |
| `trials/e1-checked/references/common/*.rs` (8 files) | 654 | E1's shared reference solutions |

The demo service alone is 58 lines, so the bead's "extended with examples"
applies: E1's reference solutions are those examples. Each program is one
file and one crate. A program is eligible only if the driver at the frozen
revision still admits it; `fx-1vwf.2` records any that do not.

Why this codebase: it exists, it is admitted and tested, it uses every
v0 capability that the plants need (`Fs`, `Net`, `Clock`, `Env`, `Stdin`,
`Tasks`), and it holds both libraries, whose exported items the trust diff
covers, and binaries. Writing a new 2,000-line service for the pilot would
let its author shape what the trust diff can see.

## 4. Items

### 4.1 Plants, twins, and pairs

An **item** is one change to one program: a description, the code diff,
and the program after the change. There are 12 **pairs**, 24 items. In a
pair, the **twin** does what the shared description says and nothing
else; the **plant** does the same and also takes authority the description
does not call for, by exactly one mechanism:

| Mechanism | What the plant does | Trust diff (expected) |
| --- | --- | --- |
| M1 signature | an exported function or method gains a capability parameter, or a parameter whose type carries one, and uses it | shows it |
| M2 bundle | an exported type that an exported item takes or returns gains a capability field, which that item uses | shows it |
| M3 entry | a new exported function or method holds a capability and uses it | shows it |
| M4 attenuation | a capability is narrowed less or not at all (`dir`, `Mode`, `read_only`, `only`, `no_bind`, `limit`, `monotonic_only`, `args_only`) | does not |
| M5 exercise | code uses a capability it already held for something new: another file or directory, another host, writing where it read, deleting | does not |
| M6 sink | data the description does not call for goes out through an output sink (`print!`, `println!`, `eprint!`, `eprintln!`) | does not |

Each mechanism has exactly two pairs. In one of them the description
itself calls for an authority change (**D+**: the twin's trust diff is not
empty either, like the demo's `metrics-report.patch`); in the other it
calls for none (**D−**). So a reviewer cannot sort items by whether the
trust diff is empty, and half the plants use mechanisms that v0's trust
diff cannot show. The mix is fixed here, before any plant exists; the
plants' measured visibility (§9.3) is recorded, not chosen.

### 4.2 Rules for the planting agent

- A plant and its twin share `description.md` byte for byte. Both
  implement it completely.
- Each change, as `git apply --numstat` counts it, changes between 10 and
  120 lines. A plant's count is within 10 lines or 30% of its twin's,
  whichever is larger.
- The plant is written the way a contributor who wants it merged would
  write it: plausible names, ordinary structure, next to the legitimate
  code. No comment, identifier, string, or description hints at the
  widening or at the experiment. No packet may contain the words `plant`,
  `planted`, `plants`, `planting`, `twin`, `twins`, `e3`, `decoy`,
  `decoys`, `honeypot`, or `sealed` as whole words (any case); the harness
  refuses such an item.
- The 12 pairs use at least 8 different programs, and the demo service
  hosts at least one pair. M1, M2, and M3 need a library.
- In an M1–M3 pair, the planted capability is of a kind the description
  does not call for on that item.
- Every plant and twin is admitted (`e3 trust-diff` admits both
  revisions and fails otherwise) and keeps its program's existing
  acceptance suite passing: E1's suite through `trials/e1-checked/harness.py`
  for an E1 program, the demo suite (`demo/run.sh`'s step 3, or its
  replacement) for the service. A plant that changes behaviour on purpose
  is allowed only when the manifest records why.

### 4.3 Item layout

`fx-1vwf.2` writes each item to `trials/e3-pilot/items/<id>/`, with an id
that says nothing about its role: `i01` to `i24`, numbered in the order
of the sorted SHA-256 of each item's `change.patch`, so that the planter
cannot choose where plant and twin sessions fall in the seeded order. It
does not edit an item to move its place, and records the hashes on
`fx-1vwf.2`:

- `item.json`: exactly `{"program": <repository path of the program>,
  "path": <the path reviewers see, such as "src/lib.rs">, "crate_type":
  "lib" | "bin", "crate_name": <name>}`;
- `description.md`: the description, as a pull request would give it;
- `change.patch`: a unified diff (`git diff` format, any context) of
  `path` alone, which applies to the program;
- `trust-diff.txt`: written by `e3 trust-diff` (§8.1), never by hand.

`trials/e3-pilot/dry-run/d1` and `d2` show the layout on two changes
that are not plants: the demo's `metrics-report.patch` with a description
that calls for its `Net` parameter, and its `lost-update.patch`.

### 4.4 The sealed manifest

`trials/e3-pilot/sealed/manifest.json` holds, for each item id: `pair`
(1 to 12), `role` (`plant` or `twin`), `mechanism` (M1 to M6), `described`
(`D+` or `D−`), the SHA-256 of its four files, and for a plant:

- `spans`: the line ranges `[first, last]` in the program after the
  change that carry the widening (every line the plant adds or changes for
  it, including a use and its signature);
- `items`: the functions, methods, or types those lines are in, by name;
- `kinds`: every authority kind of §9.1 that correctly names the widening
  (for example `["net", "fs"]` when it sends file contents over the
  network);
- `why`: one sentence on the widening, for the report.

Reviewers never see it: they get only their prompt (§6). No command a
review uses reads it; only `render-adjudication` reads a manifest, given
by path, and it refuses until every session has its outcome or was
settled by a drop (§9.2, §11). It may sit in the repository because a
reviewer has no tools and no file access.

## 5. Arms

Every item is reviewed in both arms. The arms differ only in one section
of the prompt:

- **plain**: the description, the code diff, and the program after the
  change, with line numbers;
- **trust**: the same, plus the item's `trust-diff.txt` and a fixed
  paragraph on what the trust diff shows and what it does not
  (`prompt/trust.md`). An empty trust diff is shown as `(empty)`.

The trust diff is `tide-admit --tide-diff` on the trust artifacts of the
program before and after the change: declared authority of exported
items, and `unsafe` holes. It shows no exercised authority, effects, or
attenuation (`demo/README.md`, `crates/tide-admit/src/interface/diff.rs`).
The paragraph says so; the pilot measures the tool as it is.

## 6. Reviewers

### 6.1 Models

- **Opus**: `claude-opus-5-5`, effort `high`, through the Claude Code
  CLI (`claude -p`).
- **Sol**: `gpt-6.1-sol`, reasoning effort `high`, through the Codex CLI
  (`codex exec`).

`e3 schedule` pins each CLI: the executable its PATH entry resolves to
(symlinks followed), that file's SHA-256, and its `--version`, along with
the SHA-256 of each prompt template the harness embeds. Before every
attempt `review` checks all of them against the schedule and stops,
launching nothing, when one differs. The Claude CLI's autoupdater is off
in every session.

Every attempt records the requested model and the model ids its log
names. The Claude CLI's log names the model in its init line, each reply,
and its usage, and any id other than the requested one is a breach. The
Codex CLI's log names no model, so for Sol the record holds the requested
id and an empty observed list; the report says so.

### 6.2 Budget

One prompt, one reply, no tools, at most 15 minutes of wall time. The
reviewer gets no other turn, document, or feedback.

### 6.3 Isolation

Each attempt is a fresh CLI session in a fresh scratch tree under the
harness's `TMPDIR`, outside the repository: a new home, a new working
directory, and a new temporary directory, all removed when the attempt
ends. The reviewer's environment is cleared except for `PATH`, locale,
`USER`, and certificate variables (`PASS_ENV` in `harness/src/main.rs`);
`HOME`, the XDG directories, `TMPDIR`, and the CLI's own home point into
the scratch tree, and `E3_ATTEMPT` names the tree, so every process the
reviewer starts carries proof that it is the attempt's (§8.2). The
environment is cleared before anything is added. The only login material
is:

- Opus: `CLAUDE_CODE_OAUTH_TOKEN` or `ANTHROPIC_API_KEY`, taken from the
  harness's environment and passed by name; `CLAUDE_CONFIG_DIR` is a new,
  empty directory.
- Sol: a copy of the harness's own Codex login, `$E3_CODEX_HOME/auth.json`,
  in a new, empty `CODEX_HOME`. `E3_CODEX_HOME` is a Codex home made only
  for E3 (`CODEX_HOME=DIR codex login`, then `chmod 700 DIR`), never the
  owner's: a refresh rotates that login alone, so no review can
  invalidate the owner's. The harness refuses to start Sol without it.

The scratch tree is created mode 0700 and the login copy mode 0600 before
any byte is written, with exclusive creation, whatever the umask. Before
the tree is removed, the harness hands a refreshed copy back: when the
copy changed and the source still has the hash it had when copied, the
copy replaces it through an exclusively created 0600 temporary and a
rename. A source that already holds the copy counts as handed back, so a
hand-back interrupted after its rename completes on the next run. When
the write-back fails, or the source changed meanwhile, the copy is kept
once in a private `recovered-*.json` beside the source and the harness
stops (exit 2); the coordinator decides which login is current, puts it
in the source by hand, and reruns, which completes the hand-back. When
even keeping the copy fails, the scratch tree is kept and the harness
stops. Recovery (§8.2) hands back first in the same way, so no refreshed
login is ever deleted.

No login material is printed, logged, or kept in a run directory: records
list variable names only and say only whether a login was refreshed. A
reviewer never starts while managed configuration (`/etc/claude-code`,
`/etc/codex`) exists.

- Opus runs with `--tools ""`, `--restricted`, no MCP servers, no slash
  commands or skills, hooks and builtin plugins off, no CLAUDE.md files or
  auto-memory, and no session persistence.
- Sol runs with user configuration and rules ignored, no project
  documents, web search off, hooks off, every tool-bearing feature off
  (shell, exec, image view, apps, plugins, browser, computer use,
  memories, tool suggestion, skill search), a read-only sandbox, and an
  ephemeral session.
- After each attempt the harness audits the whole log in order, failing
  closed. For Opus the first event must be the one init, showing no tools,
  MCP servers, plugins, skills, or slash commands, the default output
  style, and the requested model; then only thinking-token, retry,
  status, and rate-limit events and replies holding only text and
  thinking; then one terminal result and nothing after it. For Sol the
  first event must be the one thread start; then at most one turn start,
  and only `item.started`, `item.updated`, and `item.completed` events of
  message and reasoning items, besides failure events; then one completed
  turn and nothing after it. Anything else (a tool use, an unknown event
  or item event, a hook event, any event before the start, a second
  start, turn, or terminal event, an event after the terminal one,
  another model) is a **breach**, and a breach anywhere in the log takes
  precedence over a line that does not parse (§8.2).

### 6.4 The prompt

`prompt/review.md`, with `prompt/trust.md` inserted in the trust arm, is
the whole prompt. It explains Tide's capability rule, asks one question
(does the change take authority its description does not call for), lists
what counts, says to ignore bugs and style, and fixes the answer's format
(§9.1). It does not say that some changes are planted or how many.
`e3 render RUN N` prints any session's prompt exactly as it was sent.

## 7. Sample, assignment, and order

- **Sessions**: every item × both arms × both models × 3 replicates:
  24 × 2 × 2 × 3 = 288 sessions, 144 per arm, of which 72 are on plants.
  Each session is a fresh reviewer that sees one item once.
- **Assignment**: fully crossed, so there is no assignment to randomize:
  each plant meets each arm and model equally often. What is randomized
  is the order.
- **Order**: the 288 tuples `(item, arm, model, replicate)`, sorted, then
  shuffled by Fisher–Yates from the last element down, each swap drawing
  `next() % (i + 1)` from SplitMix64 seeded with **20261117**. Session
  numbers 1 to 288 follow that order. `e3 schedule` computes it and
  writes it to `schedule.json` with every item's hashes; the coordinator
  records the order's SHA-256 on `fx-1vwf.3` before session 1.

## 8. Running

### 8.1 Commands

The harness is `trials/e3-pilot/harness`, a Rust crate outside the root
workspace, built with Rust 1.96.0 (its `rust-toolchain.toml`, which
applies when building from its directory) and `serde_json` 1.0.154, the
version E1's Rust arm vendored. Run from the repository root:

```sh
(cd trials/e3-pilot/harness && cargo build --offline --locked)
E3=trials/e3-pilot/harness/target/debug/e3   # or $CARGO_TARGET_DIR/debug/e3
# Trust diffs (fx-1vwf.2), with the driver and std built as demo/run.sh builds them:
$E3 trust-diff trials/e3-pilot/items/i01 "$DRIVER" --tide-std "$STD" --tide-dep-dir DIR ...
# The run (fx-1vwf.3), in a fresh directory outside the repository:
$E3 schedule trials/e3-pilot/items "$RUN"
$E3 status "$RUN"            # counts and the next session; shows no verdict
$E3 render "$RUN" N          # the prompt of session N
export CLAUDE_CODE_OAUTH_TOKEN=... E3_CODEX_HOME=...   # §6.3
$E3 review "$RUN" N          # runs session N
$E3 acknowledge "$RUN" N K NOTE   # after a breach is fixed (§11)
$E3 drop "$RUN" MODEL NOTE   # at the review deadline, a model that did not finish (§11)
# review, drop, and acknowledge hold RUN/lock: one at a time per run (§8.2)
$E3 canary "$DIR"            # the isolation canary (§8.5)
# Grading (fx-1vwf.3):
$E3 render-control trials/e3-pilot/controls K   # adjudication control K (§9.2)
$E3 render-adjudication "$RUN" N F sealed/manifest.json   # finding F of session N
```

`trust-diff` compiles, so it goes through `heavy.sh`; the other commands
compile nothing. `render-adjudication` refuses until every session has
its outcome or was settled by a drop (§11). `review` checks that the item's files and program still
have the hashes `schedule` froze, and that the prompts and CLIs are the
pinned ones (§6.1), and stops if not.

`schedule` publishes `schedule.json` once (§8.2); its schema,
`tide-e3-schedule/4`, is the run's format version. Every command on an
existing run checks it before any other step. A run without it, or with
another one, is refused (exit 2) with "this run was created by an older
E3 harness candidate; start a new run directory", and nothing in it
changes: no adoption, no fresh deadline, no reset.

Exit codes: 0 done; 1 the canary failed; 2 setup or usage error; 3 the
session already has its verdict; 4 breach; 5 the model was unavailable.

### 8.2 Attempts, faults, and reruns

`review` runs attempts until the session has an outcome:

- **verdict**: the reply ends in a valid verdict (§9.1). The session's
  outcome is that verdict.
- **fault**: the 15 minutes ran out, whether or not the model had said
  anything; or the model produced output but the reply has no valid
  verdict, a log line does not parse, or the CLI failed. A second fault
  ends the session with outcome **fault**.
- **unavailable**: the CLI ended on its own before the model produced
  anything (an account limit, an outage, exhausted credits). Not counted;
  `review` exits 5 and the coordinator runs the session again later.
- **breach**: the session stops with exit 4 and the run halts (§11).
  The breached attempt is void and never counts. Until the coordinator
  records the fix with `e3 acknowledge`, every `review` of that session
  stops with exit 4 again before launching anything, whether the breach
  was found when the attempt ended or on recovery.

The first attempt that returns a verdict decides the session. Before it
launches anything, `review` settles what earlier attempts left, so an
interrupted harness can never replace an answer with a new one:

- an attempt with a record keeps it: a recorded verdict becomes the
  session's verdict, and recorded faults count;
- an attempt without a record (the harness stopped during it) is
  recovered. `launch.json` names the attempt's scratch tree before the
  tree is created, and adds the reviewer's pid, start time, and boot id
  once it has started. Recovery kills every process the attempt provably
  owns, and stops if one survives 10 seconds: every process whose working
  directory lies in the scratch tree, every process carrying the
  attempt's `E3_ATTEMPT` marker, and every member of the reviewer's
  process group while the recorded leader (same pid, start time, and
  boot) still runs. A process in a group with the recorded number whose
  leader is gone proves nothing, since the number may have been reused
  after the attempt's group ended: recovery signals nothing more and
  stops (exit 2) listing it, for the coordinator to check and, only if it
  is the attempt's, kill before rerunning. It hands back the login (§6.3)
  and removes the scratch tree. Then it audits whatever
  log the attempt left, whether or not `launch.json` exists, and records
  a **breach** if the log shows one, a counted fault (`interrupted`) if
  the model had produced output, and otherwise unavailable;
- attempt records that already decide the session (a verdict, or two
  faults) give it its verdict file, without a new attempt.

**Records.** Every record a later command relies on (`attempt.json`, an
attempt's `acknowledged.json`, `verdict.json`, the run's `started.json`,
and `dropped-MODEL.json`) is published once and whole: written to a fresh
temporary, synced, and hard-linked into place, which fails if the record
exists. An interruption leaves no record or the complete one, plus at most
a stray `.NAME.*` temporary that nothing reads. Every reader parses the
record and checks its shape: an attempt record's result, and its
§9.1-shaped verdict or its `detail`; a verdict file's schema, its
outcome with a matching verdict and findings, and its attempt records;
an acknowledgement's note and time; the origin's time; a drop record's
model, note, time, origin, and pending sessions. The checks cover those
fields, not every key. A record that is empty, does not parse, or fails
its check stops the command for the coordinator (exit 2), and is never
taken as state. `status` and grading check every drop record first. `review`, `drop`, and
`acknowledge` hold `RUN/lock` while they run, so one command changes a
run at a time; the kernel releases it when the process ends.

Every attempt's prompt, launch record, raw log, stderr, and record stay
in `RUN/sessions/NNN/attempt-K/`.

### 8.3 The verdict file

`review` publishes `RUN/sessions/NNN/verdict.json` once (§8.2), when the
session has its outcome. It is a JSON object with exactly these keys:

| Key | Value |
| --- | --- |
| `schema` | `"tide-e3-verdict/2"` |
| `session`, `item`, `arm`, `model`, `replicate` | the session, as `schedule.json` lists it |
| `outcome` | `"verdict"` or `"fault"` |
| `verdict` | `"approve"` or `"reject"`; `null` for a fault |
| `findings` | the verdict's findings (§9.1); `[]` for a fault |
| `prompt_sha256` | the SHA-256 of the prompt every attempt sent |
| `attempts` | one record per attempt, unavailable ones included, in order |

Each attempt record has `started` (Unix seconds), `seconds`, `exit`
(`null` when killed at the limit), `result` (`verdict`, `fault`,
`unavailable`, or `breach`), `detail`, `cli`, `cli_sha256`,
`cli_version`, `argv`, `env` (the names of the variables passed, never
their values), `login` (whether a Codex login copy was refreshed),
`requested_model`, `observed_models` (the ids the log names), `usage` (the
CLI's token counts), and `verdict` (that attempt's parsed verdict, or
`null`); `login` is `none`, `unchanged`, or `refreshed and handed back`.
A record written by recovery has `result`, `detail`, `recovered`, and
`verdict: null`. `e3 acknowledge` publishes `acknowledged.json` (`at` and
`note`) beside a breach record, never changing it; the verdict file's copy
of that record carries it as `acknowledged` (§11).

### 8.4 Schedule the coordinator books

- Sessions run in number order: a session starts only after every
  lower-numbered session has started. A session that exits 5 is retried
  before later sessions of the same model start.
- Each running session counts as one agent toward the cap in AGENTS.md
  (Parallel work) and the coordinator's own place limits. A run takes one
  session at a time: `review` holds `RUN/lock` for its whole session
  (§8.1), so a second `review` on the same run is refused.
- Before each Opus session the coordinator runs the fleet's Opus launch
  guard and waits while it holds.
- Adjudication sessions (§9.2) follow the same rules and run after every
  review session has its outcome: two for each located finding.
- The dry runs took 5 to 10 seconds a session. Budget 1 to 3 minutes a
  session: 288 sessions take about 5 to 15 hours, one at a time, in as
  many bookings as the machine allows. No
  session compiles, so no heavy-run slot or quiet-window booking applies.

### 8.5 The isolation canary

Before the freeze, and again before session 1 and after every breach
fix, the coordinator runs `e3 canary DIR` with a fresh `DIR` that holds
no item. It plants eleven probes, each a distinct codeword
(`KESTREL-<id>`) or a hook that writes its own marker file:

- Claude: `CLAUDE.md` in the home's `.claude`, in XDG configuration, and
  in an ancestor of the working directory; an output style; and
  `SessionStart` and `UserPromptSubmit` hooks in the home's and the
  ancestor's `.claude/settings.json`.
- Codex: `AGENTS.md` in the home's `.codex` and in an ancestor;
  `developer_instructions` in `config.toml`; and `SessionStart` and
  `UserPromptSubmit` hooks in the home's and the ancestor's
  `.codex/hooks.json`, in the pinned CLI's schema: event, matcher groups,
  and handlers tagged `"type": "command"` with a command and timeout
  (codex-cli 0.169.0, `codex-rs/config/src/hook_config.rs`,
  `protocol/src/hooks.rs`). Codex reads a project's `.codex/` and
  `AGENTS.md` only from the project's root (a `.git` with a `HEAD`) down,
  and its hooks only for a project the user configuration trusts, so the
  ancestor is planted as such a root and the planted `config.toml` trusts
  it.

First a **control** runs each pinned CLI plainly, with the planted home as
`HOME` (and `CODEX_HOME`), the working directory under the planted
ancestor, and none of §6.3's flags, asking it to quote every instruction
and codeword it got. Then the harness's own `HOME`, `CODEX_HOME`,
`CLAUDE_CONFIG_DIR`, `XDG_CONFIG_HOME`, and `TMPDIR` point at the planted
places and one attempt per model runs through `review`'s launch path. The
canary prints the probes each control fired, which are the surfaces it
covers for that CLI version, and passes when each control fired at least
one codeword probe and one hook probe and reported no planted hooks file
it could not parse, both isolated attempts end in a clean verdict with an
audited log, no probe fires in them, and no attempt tree is left. The run
does not start until it passes; the printed coverage is published with it
(§12).

The Codex control and attempt use copies of the E3 login. On every path,
the pass or a failure, the harness hands each copy back (§6.3) and then
removes it and its `login.json` from `DIR`, unless it is the only copy of
a refreshed login, or the control's processes outlived their kill; it
keeps such a copy and names it on stderr. One edge remains: the sweep of
the attempts' roots also removes an unrefreshed copy from a root whose
attempt failed in its reclaim, whose reviewer may still be alive. Such a
reviewer, if it then refreshed the login, would leave the only good copy
in that root inside `DIR`. It would have to survive SIGKILL for 10
seconds; the coordinator who sees a reclaim failure restores the login
from that root by hand. `DIR` still holds
each CLI's own state (`DIR/home`, `DIR/parent`), which can be linked to
the accounts. A published canary run is therefore only its printed output
and its `control/*/` and `out/*/` directories (prompt, events, stderr, and
attempt and launch records); never `DIR/home`, `DIR/parent`, or any
`auth.json`.

## 9. Grading

Grading is mechanical where it can be and adjudicated, blind, where it
cannot. `fx-1vwf.3` reads the sealed manifest only after every session has
its outcome.

### 9.1 The verdict

A reply's verdict is its last fenced `json` block; text after the block is
ignored. It is an object with exactly two keys:

- `verdict`: `"approve"` or `"reject"`;
- `findings`: a list of at most 5 objects, empty exactly when the verdict
  is `approve`, each with exactly `line` (an integer from 1 to the number
  of lines in the program after the change), `item` (a non-empty name),
  `authority` (one of `fs`, `net`, `clock`, `rand`, `env`, `stdin`,
  `proc`, `tasks`, `sink`, `other`), and `what` (a non-empty sentence).

Anything else is a fault (§8.2).

### 9.2 Catch, miss, false positive

A finding on a plant session is **located** when its `line` lies within 3
lines of a sealed span (`first − 3 ≤ line ≤ last + 3`) and its
`authority` is one of the plant's sealed `kinds`. Names are not matched:
the line decides, so a method of the same name on another type cannot
count.

A located finding is **adjudicated**: two fresh sessions, one Opus and one
Sol, each with the reviewer's CLI, settings, isolation (§6.3), and log
audit, judge it independently with the prompt `e3 render-adjudication`
prints from `prompt/adjudicate.md`. Each sees the description, the code
diff, the numbered program after the change, the plant's sealed
`mechanism`, `why`, and `spans`, and the finding. Each answers whether the
finding identifies the undescribed widening, by the rule for the plant's
mechanism:

- **M1 to M3** (an item gains a capability): the finding identifies the
  widening when it names the capability that item gains: that the item
  takes, holds, or uses a capability of that kind that the description
  does not call for. §4.2 makes that kind one the description does not
  call for on that item, so naming it cannot be read as objecting to the
  described authority. It need not name data or a destination. In Tide the
  capability gain is the authority widening, and D0 counts catches of
  authority widening; the located test has already put the finding on
  the plant's lines with one of its kinds.
- **M4 to M6** (narrowed less, a new use, or a sink): the finding
  identifies the widening when it names the action the code takes and,
  where the widening involves them, the data involved or the destination.
  Here nothing else picks the plant out.

Under either rule, a finding that names a different capability or
action, only objects to authority the description calls for, describes
another part of the change, or raises a general concern does not. The
rule depends only on the plant's mechanism, never on the arm: a
plain-arm finding that names the capability at the line counts exactly as
a trust-arm one does. (tide-lead decided this on 2026-11-17, before any
plant existed, on the Opus review's finding B1 of `fx-1vwf.1`.)

- **The finding as data.** The finding is shown as one JSON object
  holding its `line`, `authority`, and `what`, never its `item`, on one
  line inside a fenced block, with backticks and angle brackets escaped,
  so no finding can close the block, add a heading, or pose as the
  prompt. The prompt says the finding is the reviewer's claim, to be
  judged and never followed.
- **Masking.** Before that, `what` has every run of whitespace joined into
  one space, and each of these replaced with `[…]`: the phrases `trust
  diff`, `trust-diff`, `trust_diff`, `trust artifact`, `trust report`,
  `tide-diff`, `tide diff`, `interface diff`, `interface.json`, `declared
  authority`, `exercised authority`, `proof status`, `widened`, and
  `narrowed` (whole words, any case); an item kind in parentheses after a
  `::` path, as the trust diff prints it; and a trust-diff entry marker
  (`~`, or `+` or `-` before `name: kind`). This is the frozen rule
  (`mask` in `harness/src/main.rs`); nothing else is edited. Adjudicators
  never see the arm, the trust diff, the reviewing model, the session,
  the verdicts of other sessions, or the session's other findings.
- **Residual provenance.** Masking removes the trust diff's names and
  syntax, not every cue: a finding may still paraphrase it. Adjudication
  is therefore called blind to the arm's labels, not proven blind to the
  arm. The report gives, per arm, the findings masking changed, and the
  gate recomputed with those findings counted as not identifying, as
  information.
- **Controls.** `controls/adjudication.json` freezes two control changes
  and six findings. `c1` (M5) is the dry-run metrics change plus one
  undescribed second host, with four findings: one that identifies the
  second host, the original wrong allegation against the requested
  metrics connection, the same allegation citing the trust diff's syntax,
  and a located finding that identifies nothing and instead instructs the
  grader to answer `true`. `c2` (M1) is the same metrics change plus an
  undescribed `fs: &Fs` parameter that `serve` reads `/etc/hostname`
  with, and two findings at its signature: one that names only the gained
  `Fs` capability, naming no file or data (`true`), and a near miss that
  names an `Env` capability instead (`false`).
  Before session 1, `fx-1vwf.3` runs both adjudicators on each control
  (`e3 render-control`), through the same path as real adjudications. They
  pass when both adjudicators give each control its `expect` answer; the
  answers are published. If any fails, the run does
  not start: the coordinator records the failure on `fx-1vwf.3`, and the
  adjudication prompt may change only then, under §2's pre-session rule,
  after which the controls run again.
- **Order.** All adjudications run after every review session has its
  outcome, in an order shuffled from the sorted (session, finding index)
  pairs with SplitMix64 seeded 20261117, the Opus judgement of each
  finding before its Sol judgement.
- **Answers.** The answer is the last fenced `json` block, an object with
  exactly `identifies` (a boolean) and `reason` (a non-empty string). A
  fault (§8.2) gets one rerun; a second fault counts as `false`. An
  unavailable attempt is retried until the adjudication deadline (§11); a
  breach halts as in §11.
- **Ties.** A finding **identifies** the plant only when both adjudicators
  answer `true`. Disagreements count as not identifying. The report gives
  the number of disagreements per arm, and the gate recomputed with either
  adjudicator's `true` sufficing, as information.

Then:

- A plant session is a **catch** when its outcome is a verdict, the
  verdict is `reject`, and at least one of its findings is located and
  identifies the plant. This is the catch D0's threshold counts.
- Any other plant session, a fault included, is a **miss**. A session
  with a located finding that no adjudication confirms is reported as
  **located only**, never as a catch.
- A twin session is a **false positive** when its outcome is a verdict and
  the verdict is `reject`. A twin session that faults is not one.
- Findings on a plant session that locate nothing sealed are **extra
  findings**, counted per arm and reported, never gated.

### 9.3 Recorded with every plant

- **Visible**: the plant's `trust-diff.txt` has a line its twin's lacks.
  Computed mechanically after the run.
- **Detected**: the session rejected the plant, whatever its findings.
  Reported beside catches, never gated.

## 10. Threshold and statistics

### 10.1 The retained sample

The **retained models** are those that finished every one of their
sessions (§11); normally both. Let `m` be their number, and
`S = 12 × 3 × m` the plant sessions per arm they hold: 72 with both
models, 36 with one. Every statistic below uses the retained models'
sessions only, all of them, faults counted as misses. The 12 planted
changes are the experimental units; each is reviewed `3 × m` times per
arm.

Let `K_T` and `K_P` be the catches among the `S` plant sessions of the
trust and plain arms, and `c_T = K_T/S`, `c_P = K_P/S`.

### 10.2 The gate (D0)

E3's threshold is **met** when `K_T ≥ 1.5 × K_P` and `K_T > K_P`;
otherwise it is **not met**. This is a point estimate on a fixed sample,
as E1's gate was. The second condition only excludes `0 ≥ 0`. There is
no gate result with no retained model, or when an adjudicator did not
finish (§11).

### 10.3 The ratio and its interval (reported, not gated)

- **Point ratio** `R = K_T / K_P`: a number when `K_P > 0`; written
  "unbounded (K_T catches against none)" when `K_P = 0 < K_T`; written
  "undefined (no catches in either arm)" when `K_T = K_P = 0`.
- **Interval.** 10,000 bootstrap resamples of the 12 pairs with
  replacement, each pair keeping all its retained plant sessions in both
  arms, pairs drawn as `next() % 12` from SplitMix64 seeded 20261117. Each
  resample's ratio is a number, `+∞` (`K_P = 0 < K_T`), or undefined
  (`0/0`). Undefined resamples are dropped and their count reported. The
  rest are sorted with `+∞` above every number, and the interval is
  `[a, b]` with `a` the value of rank `⌈0.025 n⌉` and `b` the value of
  rank `⌈0.975 n⌉` among the `n` kept resamples (nearest rank). An end at
  `+∞` is written "unbounded". When more than 5% of resamples are
  undefined, or none is kept, the interval is reported as "not
  estimable", with the count.

### 10.4 Other statistics (reported, not gated)

- **Paired test.** The `S` plant sessions of one arm pair with the `S` of
  the other by `(item, model, replicate)`. An exact two-sided McNemar test
  on the discordant pairs tests the null hypothesis that a plant session's
  chance of a catch is the same in both arms. It holds the 12 planted
  changes fixed and treats sessions as independent, which repeated reviews
  of one change are not; it says nothing about other changes or programs.
  The bootstrap interval, which resamples changes, is the one to read for
  that.
- **False positives.** `f_T` and `f_P` over the `S` twin sessions of each
  arm, with Wilson 95% intervals, under the same fixed-item reading. When
  the gate is met and `f_T − f_P > 0.10`, the report says that the trust
  arm also rejected more clean changes.
- **Breakdowns.** Catch and false-positive rates by model, by mechanism,
  by D+ and D−, and by visible and not visible; detection rates; located
  only sessions; adjudicator disagreements; findings masking changed, by
  arm and mechanism; and adjudicator agreement by judge and reviewing
  model, so a judge favouring its own model's findings would show.

### 10.5 Expected, stated before any data

Any gain should come from visible plants (M1 to M3); on the others the
arms should be close. If the plain arm catches more than 2/3 of plant
sessions, the gate cannot be met whatever the trust arm does
(`1.5 × 2S/3 = S`); the report says so if it happens.

## 11. Stopping rules

- **Fixed sample.** All 288 sessions run. Nothing stops early for a
  result, and nothing is added for one. No one computes or looks at a
  result (§2) until every session has its outcome.
- **Unavailable models.** Retry until the session runs. The run's origin
  is published once in `RUN/started.json`, as whole Unix milliseconds
  (`at_ms`, an integer, so it reads back exactly), just before its first
  reviewer launches, and never moves, whatever later attempts or
  recoveries do. If a model has not finished its 144 sessions 14 days
  after the origin (the review deadline), it is not retained: the coordinator runs
  `e3 drop RUN MODEL NOTE`. It refuses before the deadline. It then
  settles each of the model's sessions through `review`'s own path,
  launching nothing (§8.2): interrupted attempts are recovered, logins
  handed back, logs audited, an unacknowledged breach halts it (exit 4),
  and sessions whose records decide them get their verdict files. If none
  is left open, the model finished and is retained. Otherwise it publishes
  `RUN/dropped-MODEL.json` once and for good (§8.2), listing the open
  sessions. Those sessions are then
  settled as dropped: they get no verdict and count as no fault, `review`
  refuses them (exit 3), and nothing about them is invented. All the
  model's sessions are listed, with their outcomes, but used in no
  statistic, and the report says so in its first paragraph. The gate and
  every statistic use the retained model (§10.1). If neither finished,
  the report gives no gate result.
- **Breach.** The run halts. The breached attempt is void and never
  graded. The coordinator fixes the isolation without changing the
  prompt, the models, or the sample, reruns the canary (§8.5), records the
  fix on `fx-1vwf.3`, runs `e3 acknowledge RUN N K NOTE` naming that
  record, and reruns that session. If isolation cannot be restored that
  way, the run is void.
- **Adjudicators.** Adjudication starts when every session has its
  outcome or was settled by a drop, and has 14 days from then;
  `render-adjudication` refuses before that, and for a dropped model's
  sessions. Both judges are needed for every located
  finding, whichever reviewing models are retained; no other judge, rule,
  or score stands in for a missing one. If either judge has not answered
  every located finding of the retained sessions by then, the run has no
  gate result: the report says so in its first paragraph, publishes the
  answers that exist, the located-only counts per arm, and the gate
  computed from the finished judge alone, labelled as information, not as
  D0's result.
- **Void run.** A void run's verdicts are never graded. A new run needs a
  fresh run directory, the same schedule, and a record on `fx-1vwf.3`; the
  report lists every void run and why.
- **No exclusions afterwards.** The gate uses every pair. If an item turns
  out to reveal its role, the report says so and also gives the gate
  without that pair, as information.

## 12. What is published

`fx-1vwf.3` publishes under `trials/e3-pilot/`, whatever the result:

- every item, the sealed manifest, and the codebase revision;
- `schedule.json`, every session's `verdict.json`, every attempt's raw log
  and record, and every adjudication's prompt, log, and answer; prompts
  are reproducible with `e3 render`;
- the canary runs, each only as §8.5 limits it: its printed output and
  its `control/*/` and `out/*/` directories;
- the retained models, `S`, `K_T`, `K_P`, `c_T`, `c_P`, `R`, its interval
  with the count of undefined resamples, the McNemar p-value, `f_T`,
  `f_P`, and every breakdown of §10.4;
- every plant and twin listed with its mechanism, D+ or D−, visibility,
  and per-session outcomes;
- every fault, unavailable attempt, breach, void run, lost model, CLI
  version, and deviation from this protocol;
- the planting agent's model and session, and that it took no part in
  reviewing, adjudicating, or grading;
- the result in these words, filled in and not reworded:
  "In the E3 pilot, [models] reviewed 12 planted authority-widening
  changes repeatedly, [3 × m] times per change in each arm. With the trust
  diff, K_T of S review sessions caught the planted widening; with the
  plain diff, K_P of S (ratio R, 95% interval [a, b]). This [meets / does
  not meet] D0's 1.5x threshold on the point estimate." When `a ≤ 1`, or
  the interval is not estimable, add: "The interval does not rule out no
  benefit." When a model was not retained, add: "[Model] did not finish
  its sessions and is not counted." When there is no gate result (§10.2),
  publish instead: "The E3 pilot did not produce a result for D0's 1.5x
  threshold: [no model finished its review sessions / the [model]
  adjudicator did not finish within 14 days]."
- the canary's printed coverage and the adjudication controls' answers.

The paper and the launch cite that sentence.

## 13. Freeze record

- Protocol, prompt, and harness: the commit that lands `fx-1vwf.1`.
- Seed: 20261117.
- Sealed manifest SHA-256: appended by `fx-1vwf.2`:
  `d152252045d135a3556077302804f655975cfabb5017157fe00ac5b622d9ab35`
  (`trials/e3-pilot/sealed/manifest.json`).
