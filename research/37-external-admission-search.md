# External bug search for a Tide admission catch (fx-zu4f.5)

**Result: a documented negative.** The registered search is complete, and no
examined candidate is caught, so there is no case study.

- P2 has no candidate. Every one of its 60 results was open at search time,
  so P2 as registered tests nothing about admission.
- P1 has 87 qualifying advisories, and none is caught.
- The nearest is `chrono_anchor` (RUSTSEC-2026-0039). Its std-only code is
  rejected with `TD0602` and `TD0501`. But its capability-passing version is
  not admitted in Tide v0: `Proc` has no `command` yet, and the check stops
  there. Whether anything else in that version would be rejected is untested.

The search follows the frozen criteria on bead fx-zu4f.5 and the operational
protocol tide-lead registered before any candidate was examined:
`target/work/protocol.md`, SHA-256
`0e5cebada4f0c379f72480a10331a55a18fdff37377659510f9cea0b6b424fb0`, recorded
on the bead at 2026-11-18T05:49Z and checked again before this work began. The
protocol is reproduced verbatim at the end of this record.

Who did what:

- A Codex builder ran the snapshot and the six searches. OpenAI's content
  filter then stopped that builder.
- An Opus builder did the first screening, the assessment and the ports
  (bead comment, 2026-11-18T05:55Z).
- An Opus fixer, in review round r1, redid the screening as a full prose
  pass over all 1,507 advisories (see "Screening").
- Four linked write-ups refuse automated reads. They were read from
  web.archive.org copies, under the owner's approval for exactly those four
  URLs (relayed by tide-desk and tide-lead, 2026-11-18; see "Archived
  write-ups"). The relay added tide-desk's conditions. The archive-derived
  ports broke one of them, because they ran outside `target/work`; see
  "Deviation from the archive condition".

## Counts

| | P1 (RustSec malicious) | P2 (GitHub issues/PRs) |
| --- | ---: | ---: |
| Pool | 1,507 in-date advisories screened; 87 qualify | 60 raw results, 60 unique URLs, 0 duplicates |
| Excluded before assessment | — | 60 not candidates: 40 fail the stars or language check, 20 are open |
| Examined candidates | 87 | 0 |
| Caught | 0 | 0 |
| Not caught: quoted code ported or assessed | 12 | — |
| Not caught: no portable source | 75 | — |
| Out of class | 0 | — |
| Unresolved | 0 | — |

The two not-caught labels are disjoint. "Quoted code ported or assessed"
means a source quotes the crate's affected code, and that code was ported or
assessed. "No portable source" means no available source quotes it; such an
entry is assessed from prose only. The 12 are:

- the seven amaperf crates, which share one port (`postgresderive` is counted under no portable source);
- `evm-units` and `sha-rust`, assessed from their quoted code;
- `uniswap-utils`, `finch-rust` and `chrono_anchor`, each ported and run.

`rustdecimal` is among the 75: its process-capability assessment rests on
prose, because no source quotes its code.

No candidate appears in both pools. These counts describe this registered
sample only. They are not a rate for any population.

## P1: RustSec malicious-crate advisories

**Snapshot.** `https://github.com/rustsec/advisory-db`, cloned with
`--no-checkout` at 2026-11-18T05:50:47Z (git 2.43.0). HEAD and search snapshot
are `233155b4025b854c68a4b199c0e32d7fd53291a2` (committed
2026-11-17T11:14:35+01:00). Files were read with `git ls-tree` and `git show`
at that commit; nothing was checked out or pulled.

**Inventory.** There are 1,507 Markdown advisories under `crates/`; the
repository's `rust/` directory covers toolchain advisories, not crates.io
crates. Every advisory has a parseable `date`, from 2016-05-09 to 2026-11-16,
so all 1,507 fall on or before the 2026-11-18 cutoff. None was unresolved for
its date. The Opus builder re-derived this inventory from the pinned commit,
and its list of paths matched the Codex builder's `advisory-inventory.json`
exactly.

**Screening.** Every one of the 1,507 in-date advisories was read and
judged on its own metadata and prose. No keyword, title or category stood in
for that judgment. This full pass was done in round r1 of the review
(2026-11-18), after the Codex pass found that the first screen had read some
records by title only. The first screen had three passes: the 85 records
categorized `malicious`, read in full; 173 records matched by a broad pattern,
read by title and then in full where the title left doubt; and the other
1,249, read by title, with 280 of their bodies read. That screen found the
same 87.

- **Text read.** A script printed each record from the pinned commit: its ID,
  crate, date, `categories`, `keywords`, `informational`, `withdrawn`, the host
  of its `url`, its title, and all of its prose. Fenced code blocks were shown
  only as a line count, and links kept their text and host. No code block in
  the 1,507 records contains "malic", "malware", "backdoor", "exfiltrat",
  "typosquat", "steal" or "trojan". The inventory text matches the pinned
  commit for all 1,507 records (`git show`, byte for byte).
- **Readers.** Five batches of about 252 records each were read by fresh Opus
  subagents of the r1 fixer, one at a time, under one written brief with the
  protocol's inclusion rule. The fixer read the sixth batch (247 records)
  directly. The fixer also read a seeded random sample of 36 records across
  all six batches (seed 20261118): all 36 agree with the batch readers.
- **Log.** `target/work/screen-full.tsv` has one row per advisory: its pinned
  path, ID, date, qualifies (`y`/`n`) and a one-line reason. The batch texts,
  the brief, the per-batch logs and the fixer's sample are in
  `target/work/screen/`.
- **Result.** 87 qualify, 1,420 do not, and none is ambiguous. The 87 are
  exactly the 87 the first screen found, so no new advisory needed
  assessment and every count below is unchanged by the full pass.

Inclusion follows the protocol: a record qualifies when it reports
intentional malicious behaviour in a crates.io crate. 87 qualify: 84
categorized `malicious`, plus `logprinter` (RUSTSEC-2026-0084), `arone`
(RUSTSEC-2026-0259) and `aronenao` (RUSTSEC-2026-0261), whose titles say
"removed from crates.io for/due to malicious code" but which carry no category.

Screening notes on ambiguous or excluded records:

- `safe-agent-rs` (RUSTSEC-2026-0101, categorized `malicious`) and `tinymember`
  (RUSTSEC-2026-0263): removed "for being affiliated with" or "due to
  affiliation with" malicious code. Both advisories say the crate "did not
  directly contain malicious code". Excluded: neither reports the crate itself
  as malicious.
- `better-macro` (RUSTSEC-2021-0077): deliberate code execution "to prove a
  point", and the advisory says the page it opens "doesn't appear to have any
  malicious content". Excluded: no malicious designation.
- `plutonium` (RUSTSEC-2020-0011) and `inconceivable` (RUSTSEC-2022-0058):
  deliberate unsafety by design, with no malicious designation. Excluded.
- `rustpython-vm` (RUSTSEC-2026-0394): a build script that leaked the build
  environment into the binary by accident. Excluded: no intent.
- Crates whose own advisory designates them malicious because they depend on a
  malicious crate (`uniswap-utils`, `finch-rust`, `tracing_checks`,
  `arrayref`, `internment`, `append-only-vec`) are included. In each case the
  advisory reports a confirmed malicious dependency, which is more than
  speculation that a dependency was compromised.
- The bead named three crates to check. `rustdecimal` is RUSTSEC-2022-0042
  and is included. `faster_log` and `async_println` have no advisory at the
  pinned commit: no path or advisory text mentions them. The frozen pool is
  RustSec advisories, so they are outside it.

**Evidence rule.** Code came only from the advisory text and from excerpts
quoted in the write-ups the advisory links. No crate source, archived crate
source or payload was fetched, built or run. None of the 87 advisories quotes
code. Three linked write-ups quote crate code:

- Veracode (formerly Phylum), on amaperf;
- Socket, on evm-units and uniswap-utils;
- Socket, on finch-rust and sha-rust, and on chrono_anchor.

The Rust blog posts, the rust-decimal issue thread, the Rust security
announcement, the OSSF record and Socket's Contagious Interview campaign page
describe behaviour in prose, or not at all.

**Outcomes.** "Not caught (no portable source)" means no available source
quotes the affected function. Writing a port without the code would invent
it, which goes beyond the faithful rule. The protocol says that missing
evidence of this kind rules out a caught claim. Such entries were assessed as
far as their prose allows, and no port was run for them.

**Ports.** The original runs (2026-11-18, 06:04Z to 12:53Z) went through the
fx-rpsb.1 examples runner, as temporary, uncommitted
`examples/case-studies/zz-probe-*` directories in the worktree, removed
afterwards. Their exact command lines, including each `timeout` value, were
not retained: the saved logs (`amaperf/run1.log`, `amaperf/run2.log`,
`run3.log` to `run6.log` in `target/work/ports/`) begin at heavy.sh's own
output. So the record's proof is the rerun below of the same port files at
the stated revision, with every command recorded, not those original runs.
Each port file was last modified before its original run (`amaperf` at
06:03:16Z, the others at 12:37:47Z), except `chrono_anchor/buggy.rs`, last
modified at 12:38:25Z, between `run3.log` and `run4.log`: its declaration
line gained `E0433`, as the table of original runs shows. `//@` lines are
comments to the driver.

**Rerun (round r1b).** `target/work/ports/rerun.sh` (SHA-256
`1abd8e5f80b5f7e69e94adc548e8e90cbeef147f6046fd378b9fd9e4ff9e41cc`) ran on
2026-11-18 from 13:27:36Z to 13:37:24Z. Its log is
`target/work/ports/rerun.log` (SHA-256
`f20f563ebdc0f9ac97ed5a66e37af826e5bff3f66d96755d3749599ed91ff32b`), which
holds each command line, start and end times, exit status and the full
diagnostics. Under tide-desk's condition (below), it invokes the driver
directly on the port files in `target/work/ports/`, not through the examples
runner, and writes the driver's output under
`/mnt/build/tide-agents/fx-zu4f.5-target/ports-out/20261118T132735Z/`.

- Revision: worktree HEAD `5dc6b40e0254d5dd83ab83b0f2d5a1dd2c2ec6dc`, tracked
  tree clean. Its `crates/`, `Cargo.toml`, `Cargo.lock` and
  `rust-toolchain.toml` equal the original runs' base `33085fd`.
- Toolchain: `nightly-2026-08-09`, `rustc 1.99.0-nightly (771916f90
  2026-08-08)`, `cargo 1.99.0-nightly (b07e5a086 2026-08-07)`, on
  x86_64-unknown-linux-gnu.
- Environment: every command runs as `bash
  ~/.cache/eon-agents/eon-overseer/heavy.sh env CARGO_BUILD_JOBS=4
  CARGO_TARGET_DIR=/mnt/build/tide-agents/fx-zu4f.5-target
  TMPDIR=/mnt/build/tide-agents/fx-zu4f.5-target/tmp timeout <secs> <command>
  </dev/null`. The 1-minute load was between 2.15 and 3.52 at each start.
- Builds: `cargo build --locked -p tide-admit` (timeout 1200; cargo found it
  fresh), then Tide's std by `docs/driver.md`'s operator recipe, `cargo build
  --locked -p tide_std --message-format=json` (timeout 1200), whose library
  directories become the `--tide-dep-dir` arguments.
- Each admission run (timeout 600) is

  ```sh
  $TARGET/debug/tide-admit --crate-type=lib --emit=metadata \
    --tide-std $TARGET/debug/libtide_std.rlib --tide-dep-dir <dir>... \
    --out-dir $TARGET/ports-out/20261118T132735Z/<port> \
    target/work/ports/<port>.rs
  ```

  with `TARGET=/mnt/build/tide-agents/fx-zu4f.5-target`. The log gives each
  line in full, with all ten `--tide-dep-dir` directories and absolute paths.
  It is the library, metadata-only mode that the runner's rejected-variant
  check selects with `--crate-type=lib --emit=metadata`
  (`crates/tide-admit/tests/examples.rs`, `check`), so nothing is linked or
  run. The two capability-passing
  ports were originally declared `tests pass`, which the runner checks with
  a test build; the rerun checks them in metadata mode only. In both, they
  fail with the errors below before any code generation.

| Port file | SHA-256 (before and after) | Exit | Errors, by code | Original log |
| --- | --- | --- | --- | --- |
| `amaperf/buggy.rs` | `fb0cbeb6…01ef44` | 1 | `TD0501` ×3, `TD0103` ×1, `E0433` ×5, `E0425` ×2, `E0599` ×2 | `amaperf/run1.log` |
| `amaperf/capability.rs` | `da119b29…37bb80` | 1 | `TD0103` ×1, `E0433` ×5, `E0425` ×1 | `amaperf/run2.log` |
| `chrono_anchor/buggy.rs` | `b1ab4d8d…ec529c` | 1 | `TD0602` ×2, `TD0501` ×3, `E0433` ×5 | `run3.log`, `run4.log` |
| `chrono_anchor/capability.rs` | `a6093630…804090` | 1 | `E0599` ×2 | `run4.log` |
| `finch_rust/buggy.rs` | `a61934db…cdaac7` | 1 | `E0433` ×1 | `run5.log` |
| `uniswap_utils/buggy.rs` | `7f3aa417…d9ec` | 1 | `E0433` ×2 | `run6.log` |

Full SHA-256 values are in the log's header. Every rerun reports the same
error codes, with the same counts, as the original run of that port, so no
result in this record changes. The uniswap-utils run took 5 min 8 s of wall
time, against 2 to 3 s for the others. heavy.sh granted its slot at the
start, and it exited 1 with the same two errors; the log does not show where
the time went.

After the rerun, in review round r2, the header comment of
`chrono_anchor/buggy.rs` was corrected from "owner condition" to
"tide-desk's condition". Nothing else in the file changed, and it was not
rerun. Its SHA-256 is now `393b22cb…2b0c1a`.

What the original runs' logs retain (times are the logs' modification times,
2026-11-18):

| Log | Time (UTC) | heavy.sh slot | Probe, declared outcome | Result |
| --- | --- | --- | --- | --- |
| `amaperf/run1.log` | 06:04:09 | `build.lock` | `zz-probe-amaperf/buggy`, `rejected TD0501` | test FAILED: `TD0501` ×3 plus undeclared `TD0103`, `E0433`, `E0425`, `E0599` |
| `amaperf/run2.log` | 06:04:39 | `build2.lock` | `zz-probe-amaperf/capability`, tests pass | test FAILED: `E0433` (`reqwest`, `serde_json`, `cfg_if`), `TD0103`, `E0425` |
| `run3.log` | 12:38:17 | `build2.lock` | `zz-probe-a/chrono_buggy`, `rejected TD0602 TD0501` | test FAILED only on the undeclared `E0433`; `TD0602` ×2 and `TD0501` ×3 fire |
| `run4.log` | 12:38:35 | `build2.lock` | `chrono_buggy`, `rejected TD0602 TD0501 E0433`; `zz-probe-b/chrono_capability`, tests pass | `chrono_buggy` meets its declaration; test FAILED on `chrono_capability`: `E0599` ×2 (`command` on `&Proc`) |
| `run5.log` | 12:44:24 | `build2.lock` | `zz-probe-c/finch_buggy`, `rejected TD0501` | test FAILED: only `E0433` (`sha_rust`), no Tide code |
| `run6.log` | 12:53:43 | `build2.lock` | `zz-probe-d/uniswap_buggy`, `rejected TD0501` | test FAILED: only `E0433` (`ctor`, `evm_units`), no Tide code |

Every original run's summary line is `test result: FAILED. 7 passed; 1
failed`: the runner's seven self-tests pass, and
`examples_meet_their_declared_outcomes` fails on the probe. A failed
declaration is the expected way the runner reports a probe's actual codes.
The landed example variants the runner reached before the failing probe met
their declarations; the runner stops at the first mismatch, so later variants
were not checked by these runs.

The port files, `rerun.sh` and the run logs stay in the builder's WORK
directory (`target/work/ports/`), not in the repository. Under tide-desk's
condition for the archived write-ups, the ports made after that approval
contain no network-fetch code: hosts, paths, program names and literal
arguments are inert placeholders. Ports whose effect is a network fetch
through a third-party client were assessed from the quoted code, not run. No
payload was executed: the ports contain no test functions, every buggy probe
was rejected, and both capability probes failed to compile, so no probe code
ran. The rerun only checks metadata, so it links and runs nothing.

**Deviation from the archive condition.** Lucca's approval was "1.2", option
2: read the archived copies. tide-desk added conditions in its relay; one was
"Everything stays inside the worktree's target/work." The four
archive-derived ports (runs 3 to 6 above) were written to temporary
`examples/case-studies/zz-probe-*` directories, which are outside
`target/work`. They were removed after the runs, and nothing from them was
committed. tide-lead reported the deviation to tide-desk on 2026-11-18.
tide-desk's disposition: acknowledged; disclose it here; any further
archive-derived work stays in `target/work`. The round r1b rerun followed
that disposition: the driver read the port files in `target/work/ports/`,
nothing was written under `examples/`, and, as tide-lead's brief directed,
the output (the driver's `.tide-cache` state, the std build messages and a
copy of each run's diagnostics) went under the build target's
`ports-out/20261118T132735Z/`, outside the repository tree.

### The amaperf port (incident A)

Source: Veracode Threat Research (formerly Phylum), ["Rust malware staged on
crates.io"](https://www.veracode.com/blog/rust-malware-staged-on-crates-io/)
(August 2023, read 2026-11-18). The post quotes the `build.rs` that `xrvrv`,
`postgress`, `if-cfg`, `serd`, `oncecell`, `lazystatic` and `envlogger`
shipped on 2023-08-16. It quotes four excerpts, in order:

1. a Telegram token and channel id;
2. a mutex file under `env::var("OUT_DIR")`, opened with `File::open` and
   created with `File::create`;
3. `reqwest::blocking::get("http://ip-api.com/json")`, a `serde_json` parse
   and a `cfg_if::cfg_if!` platform switch;
4. a `format!`ed Telegram `sendMessage` URL, fetched with
   `reqwest::blocking::get`, then `Ok(())`.

Classes: ambient environment, ambient filesystem, ambient network.

Port: the four excerpts joined in the post's order, as one function. The name
and signature (`pub fn callback() -> Result<(), Box<dyn std::error::Error>>`)
are not quoted, so the port adds the smallest signature that fits the quoted
`?` and `Ok(())`. Two other adaptations: the token is redacted, and the
markdown angle brackets around the two URLs are dropped. This port predates
tide-desk's no-network-code condition. It was admission-checked only, never
run. The capability-passing counterpart takes `env: &Env, fs: &Fs` and calls
`env.var`, `fs.open` and `fs.create`; nothing else changes.

- Buggy port (`rerun.log`; originally `amaperf/run1.log`): the driver exits
  with status 1. It reports `TD0501` for `std::env::var`,
  `std::fs::File::open` and `std::fs::File::create`. It also reports `TD0103`
  for `cfg_if!`, `E0433` for `reqwest`, `serde_json` and `cfg_if`, `E0425` and
  `E0599` for the ambient `env::var` and `File` items, and `E0425` for
  `platform`, which the unexpanded `cfg_if!` never binds.
- Capability-passing port (`rerun.log`; originally `amaperf/run2.log`): no
  `TD0501`: the port names no ambient env or fs item. It is not admitted:
  `E0433` for `reqwest`, `serde_json` and `cfg_if`, `TD0103` for `cfg_if!`,
  and `E0425` for `platform`. With these errors, the run does not show
  whether the `Env` and `Fs` reads would be admitted.

Result: **not caught.** `TD0501` fires on the ambient environment and file
reads, which is the class the bead targets. But the caught rule also needs
the capability-passing version admitted, and it is not. Tide admits no
external crate. Its std has no HTTP client (`Net::get` is planned, reference
§6.1), and replacing `reqwest`'s exfiltration with hand-written HTTP over
`Net::connect` would change the code beyond what Tide's syntax and std
require.

### The chrono_anchor port (incident L)

Source: Socket Threat Research, "5 Malicious Rust Crates Posed as Time
Utilities to Exfiltrate .env Files" (2026-03-10), archived copy (see below).
It quotes three functions from `chrono_anchor`'s `guard.rs`:

- `fetch_ref_background` detaches a `std::thread::spawn` that runs `curl` via
  `std::process::Command` as cover traffic;
- `run_optional_syncs` calls the other two;
- `submit_snapshot` detaches a thread that sleeps with `std::thread::sleep`,
  then uploads the `.env` file with `curl -F`.

Everything is std; there is no third-party crate.

Classes: detached spawn, ambient process, ambient clock (and the network
through the process).

Port: the three functions with their structure kept. Hosts, paths, the
program name and the literal arguments are inert placeholders (`"<program>"`,
`"<arg>"`), per tide-desk's condition. `crate::paths::ENV_FILE_PATH` is not
quoted, so a one-constant `mod paths` (`".env"`, the file the write-up names)
is the smallest scaffolding.

- Buggy port (`rerun.log`; originally `run3.log`, `run4.log`): rejected. It
  gets `TD0602` twice, for `std::thread::spawn`, and `TD0501` three times, for
  `Command::new` (×2) and `std::thread::sleep`. rustc adds `E0433` on the same
  paths, which Tide's std does not export. The runner check `expect: rejected
  TD0602 TD0501 E0433` passed in the original runs (`run4.log` to `run6.log`).
- Capability-passing port: the threads become `bg.spawn` tasks in a `Scope<'s,
  '_, TaskError>` the caller hands in, the sleep becomes `clock.sleep`, and
  the command becomes `proc.command(..)` on a handed-in `&Proc`. The run
  reports only `E0599` (×2; `rerun.log`, originally `run4.log`) for the
  missing `command`: "no method named `command` found for reference `&'s
  tide_std::process::Proc`". Both errors sit inside the `bg.spawn` closures
  of `fetch_ref_background` and `submit_snapshot`. These type errors stop
  rustc's later analysis of the two functions, and Tide's checks that need
  their types, so the run does not show whether the rest of the port, the
  scope tasks and the clock included, would be admitted.

Result: **not caught.** Two named TD codes fire on the faithful buggy port,
and both are the classes the bead targets. But Tide v0's `Proc` "has no
operations yet" (`crates/tide-std/src/lib.rs`; reference §6.1 lists
`command` as planned). So no capability-passing version can be admitted
without dropping the process call, and dropping it would change the effect.
It would need to be re-checked once `Proc::command` exists; this run does
not show whether that alone would be enough for admission.

### The other quoted-code assessments

- **evm-units** (RUSTSEC-2025-0147; Socket, "Malicious Rust Crate evm-units
  Serves Cross-Platform Payloads for Silent Execution", archived):
  - `get_evm_version` decodes a URL and calls `Runtime::new().block_on(check(..))`.
  - Each `check` is an `async fn`. It downloads with `reqwest`'s `Client`,
    writes under `temp_dir()` with `File::create`, and starts the payload with
    `Command::new("nohup")` inside an `unsafe` block, or with
    `Command::new("powershell.exe")` on Windows.
  - Classes: ambient env, fs, network and process, and `unsafe` without a
    grant.
  - Not caught. No capability-passing version can be admitted: `reqwest` and
    tokio are external crates, Tide has no async, and `Proc` has no operations.
  - Not run: its effect is a network fetch through a third-party client.
- **uniswap-utils** (RUSTSEC-2025-0145; same write-up): the quoted
  `#[ctor::ctor] fn test_version() { evm_units::get_evm_version(); }`, ported
  verbatim. The port gets only `E0433` (`ctor`, `evm_units`) and no Tide code
  (`rerun.log`; originally `run6.log`), so it is not caught.
- **sha-rust** (RUSTSEC-2025-0146; Socket, "Malicious Crate Mimicking 'Finch'
  Exfiltrates Credentials via a Hidden Dependency", archived):
  - `_q8w3r` reads a file with `fs::read` and `USER` with `env::var`.
  - It binds a `UdpSocket` to learn the local address.
  - It POSTs with `reqwest::blocking::Client`.
  - Not caught. No capability-passing version can be admitted: `reqwest` is
    external, and `Net` has no UDP (`bind_udp` is planned, reference §6.1).
  - Not run: its effect is a network fetch through a third-party client.
- **finch-rust** (RUSTSEC-2025-0148; same write-up): the quoted
  `to_serialized` from `src/filtering.rs`, ported verbatim, without Socket's
  inline comments. Its only effect is a call to `sha_rust::from_str()`. The
  port gets only `E0433` (`sha_rust`) and no Tide code (`rerun.log`;
  originally `run5.log`), so it is not caught.

### Archived write-ups

These Socket pages, linked from qualifying advisories, return HTTP 403 to
automated reads:

- `https://socket.dev/blog/malicious-rust-crate-evm-units-serves-cross-platform-payloads`
  (RUSTSEC-2025-0145, -0147);
- `https://socket.dev/blog/malicious-crate-mimicking-finch-exfiltrates-credentials`
  (RUSTSEC-2025-0146, -0148);
- `https://socket.dev/blog/5-malicious-rust-crates-posed-as-time-utilities-to-exfiltrate-env-files`
  (RUSTSEC-2026-0039; it covers -0030, -0031, -0032 and -0036 as well);
- `https://socket.dev/supply-chain-attacks/north-korea-s-contagious-interview-campaign`
  (RUSTSEC-2026-0084).

The owner approved reading web.archive.org copies of exactly these four URLs
(relayed by tide-desk and tide-lead, 2026-11-18). tide-desk's relay added the
conditions: port only std-only logic, never execute payloads, keep
network-fetch code out, and keep everything inside `target/work`. Before that, an attempt to
read archived or mirrored copies had been refused in the builder's
environment. The copies read, all fetched 2026-11-18 between 12:34:52Z and
12:35:44Z, are:

- `https://web.archive.org/web/20261001045823/https://socket.dev/blog/malicious-rust-crate-evm-units-serves-cross-platform-payloads`
- `https://web.archive.org/web/20260228044406/https://socket.dev/blog/malicious-crate-mimicking-finch-exfiltrates-credentials`
- `https://web.archive.org/web/20260826055615/https://socket.dev/blog/5-malicious-rust-crates-posed-as-time-utilities-to-exfiltrate-env-files`
- `https://web.archive.org/web/20260823214832/https://socket.dev/supply-chain-attacks/north-korea-s-contagious-interview-campaign`

The campaign page lists cargo among its ecosystems, but names no Rust crate
and quotes no code. Its package table did not render in the archived copy.

Other independent write-ups were not needed for any outcome and were not
read: The Hacker News (403), SentinelLabs' CrateDepression post (429), and the
mirror and archive routes for any other page.

### P1 assessments

Incident key:

| Key | Incident |
| --- | --- |
| R | `rustdecimal` |
| A | `amaperf` typosquats |
| B | `http-tiny` |
| C | the Windows-payload clusters of 2023-11/12 (`Kraded`, `TerryDavisSoldier`, `gabielle55131`, `alexrichton`) |
| D | the environment-exfiltration crates of 2025 |
| E | `rands` |
| F | `replit_ruspty` |
| H | `evm-units` |
| I | `sha-rust`/`finch-rust` |
| J | `sha-rst`/`finch-rst` |
| K | the Polymarket typosquats |
| L | the time-utility `.env` crates |
| M | `tracing-ethers` |
| N | Contagious Interview loggers |
| O | `microsoftsystem64`'s owner |
| P | the Sui build scripts |
| Q | the payload loaders of 2026-06 |
| S | `onering` |
| T | the `arrayref` attack |
| U | PolinRider |
| V | the Bitbison batch of 2026-11 |

Class entries marked "per incident", "per write-up", "per announcement" or
"per OSSF" come from prose, not from code. "Per incident" means the class the
incident's sources report taken together: the advisories and the write-ups
they link. It may come from a sibling advisory or a write-up rather than
from that row's own advisory.

| Advisory | Crate | Date | Incident | Class (evidence) | Outcome | Reason |
| --- | --- | --- | --- | --- | --- | --- |
| [RUSTSEC-2022-0042](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/rustdecimal/RUSTSEC-2022-0042.md) | `rustdecimal` | 2022-05-10 | R | env, fs, net, process (per announcement) | not caught (no portable source) | No code is quoted in the advisory or in the linked issue comment and announcement. The announcement says `Decimal::new` checked `GITLAB_CI`, downloaded a payload to `/tmp/git-updater.bin` and executed it. Running a binary needs a process operation, and Tide's `Proc` has none yet, so no capability-passing port could be admitted either. |
| [RUSTSEC-2023-0097](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/lazystatic/RUSTSEC-2023-0097.md) | `lazystatic` | 2023-08-16 | A | env, fs, net | not caught | Ported from the Veracode excerpts (see "The amaperf port"). The buggy port gets `TD0501` three times. The capability-passing port is not admitted: `reqwest`, `serde_json` and `cfg_if!` (`E0433`, `TD0103`) have no Tide counterpart, and replacing the HTTP client goes beyond the faithful rule. |
| [RUSTSEC-2023-0098](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/if-cfg/RUSTSEC-2023-0098.md) | `if-cfg` | 2023-08-16 | A | env, fs, net | not caught | Ported from the Veracode excerpts (see "The amaperf port"). The buggy port gets `TD0501` three times. The capability-passing port is not admitted: `reqwest`, `serde_json` and `cfg_if!` (`E0433`, `TD0103`) have no Tide counterpart, and replacing the HTTP client goes beyond the faithful rule. |
| [RUSTSEC-2023-0099](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/envlogger/RUSTSEC-2023-0099.md) | `envlogger` | 2023-08-16 | A | env, fs, net | not caught | Ported from the Veracode excerpts (see "The amaperf port"). The buggy port gets `TD0501` three times. The capability-passing port is not admitted: `reqwest`, `serde_json` and `cfg_if!` (`E0433`, `TD0103`) have no Tide counterpart, and replacing the HTTP client goes beyond the faithful rule. |
| [RUSTSEC-2023-0100](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/xrvrv/RUSTSEC-2023-0100.md) | `xrvrv` | 2023-08-16 | A | env, fs, net | not caught | Ported from the Veracode excerpts (see "The amaperf port"). The buggy port gets `TD0501` three times. The capability-passing port is not admitted: `reqwest`, `serde_json` and `cfg_if!` (`E0433`, `TD0103`) have no Tide counterpart, and replacing the HTTP client goes beyond the faithful rule. |
| [RUSTSEC-2023-0101](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/oncecell/RUSTSEC-2023-0101.md) | `oncecell` | 2023-08-16 | A | env, fs, net | not caught | Ported from the Veracode excerpts (see "The amaperf port"). The buggy port gets `TD0501` three times. The capability-passing port is not admitted: `reqwest`, `serde_json` and `cfg_if!` (`E0433`, `TD0103`) have no Tide counterpart, and replacing the HTTP client goes beyond the faithful rule. |
| [RUSTSEC-2023-0102](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/serd/RUSTSEC-2023-0102.md) | `serd` | 2023-08-16 | A | env, fs, net | not caught | Ported from the Veracode excerpts (see "The amaperf port"). The buggy port gets `TD0501` three times. The capability-passing port is not admitted: `reqwest`, `serde_json` and `cfg_if!` (`E0433`, `TD0103`) have no Tide counterpart, and replacing the HTTP client goes beyond the faithful rule. |
| [RUSTSEC-2023-0103](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/postgress/RUSTSEC-2023-0103.md) | `postgress` | 2023-08-16 | A | env, fs, net | not caught | Ported from the Veracode excerpts (see "The amaperf port"). The buggy port gets `TD0501` three times. The capability-passing port is not admitted: `reqwest`, `serde_json` and `cfg_if!` (`E0433`, `TD0103`) have no Tide counterpart, and replacing the HTTP client goes beyond the faithful rule. |
| [RUSTSEC-2023-0110](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/postgresderive/RUSTSEC-2023-0110.md) | `postgresderive` | 2023-08-18 | A | env, fs, net (per incident) | not caught (no portable source) | Same cluster as A, but the excerpted post predates this crate and does not list it; the advisory links only the vendors' home pages. If its build.rs is the same, the A result applies. |
| [RUSTSEC-2023-0104](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/littest/RUSTSEC-2023-0104.md) | `littest` | 2023-11-06 | B | env, net (per incident) | not caught (no portable source) | Advisory prose only ("payload in build.rs to exfiltrate host information"); it links only the vendors' home pages. |
| [RUSTSEC-2023-0114](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/tiny-server/RUSTSEC-2023-0114.md) | `tiny-server` | 2023-11-06 | B | env, net (per incident) | not caught (no portable source) | Advisory prose only ("payload in build.rs to exfiltrate host information"); it links only the vendors' home pages. |
| [RUSTSEC-2023-0106](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/lfest-main/RUSTSEC-2023-0106.md) | `lfest-main` | 2023-11-15 | C | process (payload run, per incident) | not caught (no portable source) | Advisory prose only ("run an arbitrary malware payload on Windows hosts"); no linked write-up. |
| [RUSTSEC-2023-0107](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/lasso-rs/RUSTSEC-2023-0107.md) | `lasso-rs` | 2023-11-15 | C | process (payload run, per incident) | not caught (no portable source) | Advisory prose only ("run an arbitrary malware payload on Windows hosts"); no linked write-up. |
| [RUSTSEC-2023-0109](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/win-crypto/RUSTSEC-2023-0109.md) | `win-crypto` | 2023-11-15 | C | process (payload run, per incident) | not caught (no portable source) | Advisory prose only ("run an arbitrary malware payload on Windows hosts"); no linked write-up. |
| [RUSTSEC-2023-0111](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/monero-api/RUSTSEC-2023-0111.md) | `monero-api` | 2023-11-15 | C | process (payload run, per incident) | not caught (no portable source) | Advisory prose only ("run an arbitrary malware payload on Windows hosts"); no linked write-up. |
| [RUSTSEC-2023-0112](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/monero-rpc-rs/RUSTSEC-2023-0112.md) | `monero-rpc-rs` | 2023-11-15 | C | process (payload run, per incident) | not caught (no portable source) | Advisory prose only ("run an arbitrary malware payload on Windows hosts"); no linked write-up. |
| [RUSTSEC-2023-0113](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/win-base64-rs/RUSTSEC-2023-0113.md) | `win-base64-rs` | 2023-11-15 | C | process (payload run, per incident) | not caught (no portable source) | Advisory prose only ("run an arbitrary malware payload on Windows hosts"); no linked write-up. |
| [RUSTSEC-2023-0115](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/acceptxmr-rs/RUSTSEC-2023-0115.md) | `acceptxmr-rs` | 2023-11-15 | C | process (payload run, per incident) | not caught (no portable source) | Advisory prose only ("run an arbitrary malware payload on Windows hosts"); no linked write-up. |
| [RUSTSEC-2023-0116](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/registry-win/RUSTSEC-2023-0116.md) | `registry-win` | 2023-11-15 | C | process (payload run, per incident) | not caught (no portable source) | Advisory prose only ("run an arbitrary malware payload on Windows hosts"); no linked write-up. |
| [RUSTSEC-2023-0118](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/win_run_rs/RUSTSEC-2023-0118.md) | `win_run_rs` | 2023-11-15 | C | process (payload run, per incident) | not caught (no portable source) | Advisory prose only ("run an arbitrary malware payload on Windows hosts"); no linked write-up. |
| [RUSTSEC-2023-0120](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/windows-service-rs/RUSTSEC-2023-0120.md) | `windows-service-rs` | 2023-11-15 | C | process (payload run, per incident) | not caught (no portable source) | Advisory prose only ("run an arbitrary malware payload on Windows hosts"); no linked write-up. |
| [RUSTSEC-2023-0121](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/libusb1-main/RUSTSEC-2023-0121.md) | `libusb1-main` | 2023-11-15 | C | process (payload run, per incident) | not caught (no portable source) | Advisory prose only ("run an arbitrary malware payload on Windows hosts"); no linked write-up. |
| [RUSTSEC-2023-0122](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/winx-rs/RUSTSEC-2023-0122.md) | `winx-rs` | 2023-11-15 | C | process (payload run, per incident) | not caught (no portable source) | Advisory prose only ("run an arbitrary malware payload on Windows hosts"); no linked write-up. |
| [RUSTSEC-2023-0105](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/windowsservice/RUSTSEC-2023-0105.md) | `windowsservice` | 2023-11-18 | C | process (payload run, per incident) | not caught (no portable source) | Advisory prose only ("run an arbitrary malware payload on Windows hosts"); no linked write-up. |
| [RUSTSEC-2023-0119](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/openvpn-plugin-rs/RUSTSEC-2023-0119.md) | `openvpn-plugin-rs` | 2023-11-18 | C | process (payload run, per incident) | not caught (no portable source) | Advisory prose only ("run an arbitrary malware payload on Windows hosts"); no linked write-up. |
| [RUSTSEC-2023-0123](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/hann-rs-service/RUSTSEC-2023-0123.md) | `hann-rs-service` | 2023-11-18 | C | process (payload run, per incident) | not caught (no portable source) | Advisory prose only ("run an arbitrary malware payload on Windows hosts"); no linked write-up. |
| [RUSTSEC-2023-0108](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/tauri-win-rt-notification/RUSTSEC-2023-0108.md) | `tauri-win-rt-notification` | 2023-11-20 | C | process (payload run, per incident) | not caught (no portable source) | Advisory prose only ("run an arbitrary malware payload on Windows hosts"); no linked write-up. |
| [RUSTSEC-2023-0117](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/tauri-winrt-notifications/RUSTSEC-2023-0117.md) | `tauri-winrt-notifications` | 2023-11-20 | C | process (payload run, per incident) | not caught (no portable source) | Advisory prose only ("run an arbitrary malware payload on Windows hosts"); no linked write-up. |
| [RUSTSEC-2023-0124](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/bit-flags/RUSTSEC-2023-0124.md) | `bit-flags` | 2023-12-22 | C | process (payload run, per incident) | not caught (no portable source) | Advisory prose only ("run an arbitrary malware payload on Windows hosts"); no linked write-up. |
| [RUSTSEC-2025-0158](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/jfrog_quotes/RUSTSEC-2025-0158.md) | `jfrog_quotes` | 2025-01-30 | D | env, net (per incident) | not caught (no portable source) | Advisory prose only ("exfiltrate environmental data"); no linked write-up. |
| [RUSTSEC-2025-0160](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/custom-req-on-workers/RUSTSEC-2025-0160.md) | `custom-req-on-workers` | 2025-01-30 | D | env, net (per incident) | not caught (no portable source) | Advisory prose only ("exfiltrate environmental data"); no linked write-up. |
| [RUSTSEC-2025-0155](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/rands/RUSTSEC-2025-0155.md) | `rands` | 2025-02-10 | E | unknown (payload linked at build) | not caught (no portable source) | Advisory prose only; no linked write-up. |
| [RUSTSEC-2025-0159](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/sophosfirewall-python/RUSTSEC-2025-0159.md) | `sophosfirewall-python` | 2025-02-15 | D | env, net (per incident) | not caught (no portable source) | Advisory prose only ("exfiltrate environmental data"); no linked write-up. |
| [RUSTSEC-2025-0156](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/tree-sitter-pkl/RUSTSEC-2025-0156.md) | `tree-sitter-pkl` | 2025-03-10 | D | env, net (per incident) | not caught (no portable source) | Advisory prose only ("exfiltrate environmental data"); no linked write-up. |
| [RUSTSEC-2025-0157](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/statsrelay-protobuf/RUSTSEC-2025-0157.md) | `statsrelay-protobuf` | 2025-08-26 | D | env, net (per incident) | not caught (no portable source) | Advisory prose only ("exfiltrate environmental data"); no linked write-up. |
| [RUSTSEC-2025-0154](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/replit_ruspty/RUSTSEC-2025-0154.md) | `replit_ruspty` | 2025-11-04 | F | net, process (per OSSF) | not caught (no portable source) | The linked OSSF record says only that it contacts a malicious domain and runs malicious commands; it quotes no code. |
| [RUSTSEC-2025-0145](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/uniswap-utils/RUSTSEC-2025-0145.md) | `uniswap-utils` | 2025-12-03 | H | load-time call into a malicious dependency (quoted code) | not caught | Quoted `#[ctor::ctor] fn test_version() { evm_units::get_evm_version(); }`. The port gets only `E0433` (`ctor`, `evm_units`) and no Tide code, so no named TD code fires. |
| [RUSTSEC-2025-0147](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/evm-units/RUSTSEC-2025-0147.md) | `evm-units` | 2025-12-03 | H | env, fs, net, process, `unsafe` (quoted code) | not caught | Quoted `get_evm_version` and `check` (Socket). The download goes through `reqwest::Client`, inside a tokio `Runtime` and `async fn`; the payload runs through `Command::spawn`, inside an `unsafe` block. A capability-passing version cannot be admitted: Tide admits no external crate and has no async, and `Proc` has no operations. Port not run: tide-desk's condition keeps network-fetch code out of ports, and the counterpart cannot exist. |
| [RUSTSEC-2025-0146](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/sha-rust/RUSTSEC-2025-0146.md) | `sha-rust` | 2025-12-05 | I | env, fs, net (quoted code) | not caught | Quoted `_q8w3r` (Socket) reads a file with `fs::read` and `USER` with `env::var`, binds a `UdpSocket`, and POSTs with `reqwest::blocking::Client`. A capability-passing version cannot be admitted: `reqwest` is external and `Net` has no UDP (`bind_udp` is planned). Port not run, for tide-desk's condition on network-fetch code. |
| [RUSTSEC-2025-0148](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/finch-rust/RUSTSEC-2025-0148.md) | `finch-rust` | 2025-12-05 | I | call into a malicious dependency (quoted code) | not caught | Quoted `to_serialized` (src/filtering.rs) calls `sha_rust::from_str()`. The port gets only `E0433` (`sha_rust`) and no Tide code. |
| [RUSTSEC-2025-0150](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/finch-rst/RUSTSEC-2025-0150.md) | `finch-rst` | 2025-12-09 | J | fs, net (per incident) | not caught (no portable source) | Advisory prose only; separate incident (2025-12-08) from I, with no linked write-up. |
| [RUSTSEC-2025-0151](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/sha-rst/RUSTSEC-2025-0151.md) | `sha-rst` | 2025-12-09 | J | fs, net (per incident) | not caught (no portable source) | Advisory prose only; separate incident (2025-12-08) from I, with no linked write-up. |
| [RUSTSEC-2025-0152](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/finch_cli_rust/RUSTSEC-2025-0152.md) | `finch_cli_rust` | 2025-12-09 | J | fs, net (per incident) | not caught (no portable source) | Advisory prose only; separate incident (2025-12-08) from I, with no linked write-up. |
| [RUSTSEC-2026-0010](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/polymarket-clients-sdk/RUSTSEC-2026-0010.md) | `polymarket-clients-sdk` | 2026-02-06 | K | env, fs, net (per incident) | not caught (no portable source) | Advisory prose only; links go to crates.io or vendor home pages. |
| [RUSTSEC-2026-0011](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/polymarket-client-sdks/RUSTSEC-2026-0011.md) | `polymarket-client-sdks` | 2026-02-13 | K | env, fs, net (per incident) | not caught (no portable source) | Advisory prose only; links go to crates.io or vendor home pages. |
| [RUSTSEC-2026-0014](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/rpc-check/RUSTSEC-2026-0014.md) | `rpc-check` | 2026-02-19 | K | env, fs, net (per incident) | not caught (no portable source) | Advisory prose only; links go to crates.io or vendor home pages. |
| [RUSTSEC-2026-0015](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/polymarkets-client-sdk/RUSTSEC-2026-0015.md) | `polymarkets-client-sdk` | 2026-02-19 | K | env, fs, net (per incident) | not caught (no portable source) | Advisory prose only; links go to crates.io or vendor home pages. |
| [RUSTSEC-2026-0016](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/polymarkets-rs-clob-client/RUSTSEC-2026-0016.md) | `polymarkets-rs-clob-client` | 2026-02-20 | K | env, fs, net (per incident) | not caught (no portable source) | Advisory prose only; links go to crates.io or vendor home pages. |
| [RUSTSEC-2026-0017](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/clob-sdk/RUSTSEC-2026-0017.md) | `clob-sdk` | 2026-02-20 | K | env, fs, net (per incident) | not caught (no portable source) | Advisory prose only; links go to crates.io or vendor home pages. |
| [RUSTSEC-2026-0018](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/rpc-check/RUSTSEC-2026-0018.md) | `rpc-check` | 2026-02-24 | K | env, fs, net (per incident) | not caught (no portable source) | Advisory prose only; links go to crates.io or vendor home pages. |
| [RUSTSEC-2026-0019](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/tracing-check/RUSTSEC-2026-0019.md) | `tracing-check` | 2026-02-24 | K | env, fs, net (per incident) | not caught (no portable source) | Advisory prose only; links go to crates.io or vendor home pages. |
| [RUSTSEC-2026-0027](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/tracings/RUSTSEC-2026-0027.md) | `tracings` | 2026-02-26 | K | env, fs, net (per incident) | not caught (no portable source) | Advisory prose only; links go to crates.io or vendor home pages. |
| [RUSTSEC-2026-0028](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/tracing_checks/RUSTSEC-2026-0028.md) | `tracing_checks` | 2026-02-26 | K | env, fs, net (per incident) | not caught (no portable source) | Advisory prose only; links go to crates.io or vendor home pages. |
| [RUSTSEC-2026-0030](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/time_calibrator/RUSTSEC-2026-0030.md) | `time_calibrator` | 2026-03-03 | L | process, net (per write-up) | not caught (no portable source) | The write-up says it uses "the same exfiltration workflow" but quotes only chrono_anchor's code. If its code is the same, the chrono_anchor result applies. |
| [RUSTSEC-2026-0031](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/time_calibrators/RUSTSEC-2026-0031.md) | `time_calibrators` | 2026-03-03 | L | process, net (per write-up) | not caught (no portable source) | As RUSTSEC-2026-0030. |
| [RUSTSEC-2026-0032](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/dnp3times/RUSTSEC-2026-0032.md) | `dnp3times` | 2026-03-04 | L | process, net (per write-up) | not caught (no portable source) | As RUSTSEC-2026-0030. |
| [RUSTSEC-2026-0036](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/time-sync/RUSTSEC-2026-0036.md) | `time-sync` | 2026-03-04 | L | process, net (per write-up) | not caught (no portable source) | As RUSTSEC-2026-0030. |
| [RUSTSEC-2026-0039](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/chrono_anchor/RUSTSEC-2026-0039.md) | `chrono_anchor` | 2026-03-10 | L | detached spawn, process, clock (quoted code) | not caught | Quoted guard.rs (Socket), std only. The buggy port gets `TD0602` (×2, `std::thread::spawn`) and `TD0501` (×3, `Command::new`, `thread::sleep`). The capability-passing port (scope tasks, `Clock`, `Proc`) fails on `E0599`: `Proc` has no `command` in v0 (planned, reference §6.1). See "The chrono_anchor port". |
| [RUSTSEC-2026-0040](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/tracing-ethers/RUSTSEC-2026-0040.md) | `tracing-ethers` | 2026-03-14 | M | fs, net (per incident) | not caught (no portable source) | Advisory prose only. |
| [RUSTSEC-2026-0081](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/logtrace/RUSTSEC-2026-0081.md) | `logtrace` | 2026-04-05 | N | net, process (per incident) | not caught (no portable source) | Advisory prose only; links only a vendor home page. |
| [RUSTSEC-2026-0084](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/logprinter/RUSTSEC-2026-0084.md) | `logprinter` | 2026-04-09 | N | net, process (per incident) | not caught (no portable source) | The linked Socket campaign page (archived copy) names no Rust crate and quotes no code. The advisory says `trace()` downloaded code over HTTP and executed it. |
| [RUSTSEC-2026-0100](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/pretty-changelog-logger/RUSTSEC-2026-0100.md) | `pretty-changelog-logger` | 2026-04-13 | O | fs, net, process (per incident) | not caught (no portable source) | Advisory prose only; links only vendor home pages. |
| [RUSTSEC-2026-0102](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/microsoftsystem64/RUSTSEC-2026-0102.md) | `microsoftsystem64` | 2026-04-13 | O | fs, net, process (per incident) | not caught (no portable source) | Advisory prose only; links only vendor home pages. |
| [RUSTSEC-2026-0107](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/mysten-metrics/RUSTSEC-2026-0107.md) | `mysten-metrics` | 2026-04-22 | P | net (per incident) | not caught (no portable source) | Advisory prose only. |
| [RUSTSEC-2026-0108](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/sui-execution-cut/RUSTSEC-2026-0108.md) | `sui-execution-cut` | 2026-04-23 | P | net (per incident) | not caught (no portable source) | Advisory prose only. |
| [RUSTSEC-2026-0155](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/exploration/RUSTSEC-2026-0155.md) | `exploration` | 2026-06-02 | Q | net, process (per incident) | not caught (no portable source) | Advisory prose only. |
| [RUSTSEC-2026-0171](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/logflux/RUSTSEC-2026-0171.md) | `logflux` | 2026-06-03 | Q | net, process (per incident) | not caught (no portable source) | Advisory prose only. |
| [RUSTSEC-2026-0175](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/onering/RUSTSEC-2026-0175.md) | `onering` | 2026-06-10 | S | fs, net (per incident) | not caught (no portable source) | Advisory prose only. |
| [RUSTSEC-2026-0259](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/arone/RUSTSEC-2026-0259.md) | `arone` | 2026-08-20 | T | net, process (per incident) | not caught (no portable source) | The linked Rust blog post quotes no crate code (only a `find` command for checking caches). |
| [RUSTSEC-2026-0260](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/arrayref/RUSTSEC-2026-0260.md) | `arrayref` | 2026-08-20 | T | net, process (per incident) | not caught (no portable source) | The linked Rust blog post quotes no crate code (only a `find` command for checking caches). |
| [RUSTSEC-2026-0261](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/aronenao/RUSTSEC-2026-0261.md) | `aronenao` | 2026-08-20 | T | net, process (per incident) | not caught (no portable source) | The linked Rust blog post quotes no crate code (only a `find` command for checking caches). |
| [RUSTSEC-2026-0262](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/append-only-vec/RUSTSEC-2026-0262.md) | `append-only-vec` | 2026-08-20 | T | net, process (per incident) | not caught (no portable source) | The linked Rust blog post quotes no crate code (only a `find` command for checking caches). |
| [RUSTSEC-2026-0264](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/proc-macro-en/RUSTSEC-2026-0264.md) | `proc-macro-en` | 2026-08-20 | T | net, process (per incident) | not caught (no portable source) | The linked Rust blog post quotes no crate code (only a `find` command for checking caches). |
| [RUSTSEC-2026-0265](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/proc-macro1/RUSTSEC-2026-0265.md) | `proc-macro1` | 2026-08-20 | T | net, process (per incident) | not caught (no portable source) | The linked Rust blog post quotes no crate code (only a `find` command for checking caches). |
| [RUSTSEC-2026-0266](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/internment/RUSTSEC-2026-0266.md) | `internment` | 2026-08-20 | T | net, process (per incident) | not caught (no portable source) | The linked Rust blog post quotes no crate code (only a `find` command for checking caches). |
| [RUSTSEC-2026-0280](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/greentic-setup-dev/RUSTSEC-2026-0280.md) | `greentic-setup-dev` | 2026-09-07 | U | unknown (editor-triggered) | not caught (no portable source) | Advisory prose only. |
| [RUSTSEC-2026-0281](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/greentic-setup/RUSTSEC-2026-0281.md) | `greentic-setup` | 2026-09-07 | U | unknown (editor-triggered) | not caught (no portable source) | Advisory prose only. |
| [RUSTSEC-2026-0466](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/build_dep/RUSTSEC-2026-0466.md) | `build_dep` | 2026-11-06 | V | env, net (per incident) | not caught (no portable source) | Advisory prose only; links only a vendor home page. |
| [RUSTSEC-2026-0467](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/top_level_crate/RUSTSEC-2026-0467.md) | `top_level_crate` | 2026-11-06 | V | env, net (per incident) | not caught (no portable source) | Advisory prose only; links only a vendor home page. |
| [RUSTSEC-2026-0468](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/shinchina/RUSTSEC-2026-0468.md) | `shinchina` | 2026-11-06 | V | env, net (per incident) | not caught (no portable source) | Advisory prose only; links only a vendor home page. |
| [RUSTSEC-2026-0471](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/sec-gemini/RUSTSEC-2026-0471.md) | `sec-gemini` | 2026-11-06 | V | env, net (per incident) | not caught (no portable source) | Advisory prose only; links only a vendor home page. |
| [RUSTSEC-2026-0472](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/shinchina-test/RUSTSEC-2026-0472.md) | `shinchina-test` | 2026-11-06 | V | env, net (per incident) | not caught (no portable source) | Advisory prose only; links only a vendor home page. |
| [RUSTSEC-2026-0464](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/webhook_commands/RUSTSEC-2026-0464.md) | `webhook_commands` | 2026-11-09 | V | env, net (per incident) | not caught (no portable source) | Advisory prose only; links only a vendor home page. |
| [RUSTSEC-2026-0465](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/electrumd/RUSTSEC-2026-0465.md) | `electrumd` | 2026-11-09 | V | env, net (per incident) | not caught (no portable source) | Advisory prose only; links only a vendor home page. |
| [RUSTSEC-2026-0469](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/env-test-guard/RUSTSEC-2026-0469.md) | `env-test-guard` | 2026-11-09 | V | env, net (per incident) | not caught (no portable source) | Advisory prose only; links only a vendor home page. |
| [RUSTSEC-2026-0470](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/gabby-types-sec/RUSTSEC-2026-0470.md) | `gabby-types-sec` | 2026-11-09 | V | env, net (per incident) | not caught (no portable source) | Advisory prose only; links only a vendor home page. |
| [RUSTSEC-2026-0483](https://github.com/rustsec/advisory-db/blob/233155b4025b854c68a4b199c0e32d7fd53291a2/crates/lwd-uri-util/RUSTSEC-2026-0483.md) | `lwd-uri-util` | 2026-11-10 | V | env, net (per incident) | not caught (no portable source) | Advisory prose only; links only a vendor home page. |


## P2: GitHub issues and pull requests

**Method.** The Codex builder sent each of the six frozen queries once, on
2026-11-18 between 05:50:10Z and 05:50:15Z. Each request went through the
read-only `gh api --method GET search/issues` with `language:Rust`,
`per_page=10` and `page=1`, and with no `sort` or `order` parameter: GitHub's
default best-match sort. Each response is complete (`incomplete_results`
false) and has 10 items. The requests and responses are saved in WORK
(`search/q1..q6-request.json`, `q1..q6.json`); they were not rerun.

Next, each result's repository metadata was looked up (05:50:43Z to
05:50:52Z). A repository must have at least 1,000 stars and Rust as its
primary language. The lookup for `conda/rattler` failed with HTTP 502, and one
retry by the Opus builder at 2026-11-18T05:58:29Z returned 466 stars and
primary language Rust (`search/repos/conda--rattler-retry.json`). Results that fail this check are
not replaced. For the 20 results that pass, the issue or PR was fetched by GET
(05:58Z). Every one is still open, so none is a closed issue or a merged PR,
and none is a candidate. No duplicates occur within or across the six lists.
Every one of the 60 results was open at search time, so P2 as registered
contributed no candidate and tests nothing about admission.

Every search result is listed below. "State at search" is the search
response's `state` field.

#### Q1: `"env::var" bug language:Rust`

Request started 2026-11-18T05:50:10Z; `total_count` 72960; `incomplete_results` false; 10 items.

| # | Result | Kind, state at search | Repository stars, language | Assessment |
| ---: | --- | --- | --- | --- |
| 1.1 | [parth-patel010/StorByte#7](https://github.com/parth-patel010/StorByte/issues/7): Bug: OAuth redirect env var name mismatch (.env.example vs Rust) | issue, open | 0, Rust | not a candidate: 0 stars (<1,000) |
| 1.2 | [conda/rattler#3181](https://github.com/conda/rattler/issues/3181): bug(shell): invalid syntax in `restore_env_var` for Xonsh and NuShell breaks deactivation | issue, open | 466, Rust | not a candidate: 466 stars (<1,000) |
| 1.3 | [open-feature/rust-sdk-contrib#194](https://github.com/open-feature/rust-sdk-contrib/pull/194): chore(main): release open-feature-env-var 0.2.2 | PR, open | 15, Rust | not a candidate: 15 stars (<1,000) |
| 1.4 | [chroma-core/chroma#7816](https://github.com/chroma-core/chroma/pull/7816): [BUG](ef): Respect explicit api_key_env_var | PR, open | 29709, Rust | not a candidate: PR open, not merged (detail fetched 2026-11-18T05:58Z) |
| 1.5 | [PrimeIntellect-ai/prime-agent#1811](https://github.com/PrimeIntellect-ai/prime-agent/issues/1811): fix: handle empty env vars in config (prevent relative path bug) | issue, open | 22809, Rust | not a candidate: issue open (detail fetched 2026-11-18T05:58Z) |
| 1.6 | [Growth-Circle/cadis#6418](https://github.com/Growth-Circle/cadis/issues/6418): bug(config): CADIS_MODEL_PROVIDER env var is set directly without validation or normalization | issue, open | 59, Rust | not a candidate: 59 stars (<1,000) |
| 1.7 | [Growth-Circle/cadis#6409](https://github.com/Growth-Circle/cadis/issues/6409): bug(config): CADIS_LOG_LEVEL env var accepted without validation against known log levels | issue, open | 59, Rust | not a candidate: 59 stars (<1,000) |
| 1.8 | [dbt-labs/dbt#17121](https://github.com/dbt-labs/dbt/issues/17121): [v2 Bug] state:modified selects sources whose env_var database renders differently when schema is not set | issue, open | 14189, Rust | not a candidate: issue open (detail fetched 2026-11-18T05:58Z) |
| 1.9 | [wakamex/delta#1](https://github.com/wakamex/delta/issues/1): 🐛 Delta broken when PAGER env var has arguments | issue, open | 0, — | not a candidate: 0 stars (<1,000); primary language None |
| 1.10 | [Growth-Circle/cadis#6419](https://github.com/Growth-Circle/cadis/issues/6419): bug(config): CADIS_TCP_AUTH_TOKEN env var accepted with no minimum length or entropy check | issue, open | 59, Rust | not a candidate: 59 stars (<1,000) |

#### Q2: `"SystemTime::now" flaky language:Rust`

Request started 2026-11-18T05:50:11Z; `total_count` 1301; `incomplete_results` false; 10 items.

| # | Result | Kind, state at search | Repository stars, language | Assessment |
| ---: | --- | --- | --- | --- |
| 2.1 | [Calhooon/bsv-wallet-cli#469](https://github.com/Calhooon/bsv-wallet-cli/issues/469): The spend gate's clock-dependent tests are flaky and can make a mutant look caught: every floored file's tests take a fixed clock | issue, open | 13, Rust | not a candidate: 13 stars (<1,000) |
| 2.2 | [spnngl/speculaire#91](https://github.com/spnngl/speculaire/issues/91): Polls miss a registry config change stamped by the coarse clock (flaky agent test) | issue, open | 0, Rust | not a candidate: 0 stars (<1,000) |
| 2.3 | [jatmn/jModGuard#666](https://github.com/jatmn/jModGuard/issues/666): test: audit CI history for flaky tests and fix them | issue, open | 1, Rust | not a candidate: 1 stars (<1,000) |
| 2.4 | [gattaca-com/helix#521](https://github.com/gattaca-com/helix/issues/521): Flaky/slow test_duration_into_slot in helix-types::clock | issue, open | 141, Rust | not a candidate: 141 stars (<1,000) |
| 2.5 | [JordanTheJet/zeroclaw#44](https://github.com/JordanTheJet/zeroclaw/issues/44): Flaky test: workspace_probe_path_is_hidden_and_unique (clock-tick uniqueness) | issue, open | 1, Rust | not a candidate: 1 stars (<1,000) |
| 2.6 | [open-telemetry/opentelemetry-rust#3862](https://github.com/open-telemetry/opentelemetry-rust/pull/3862): fix: read the browser clock on wasm64 | PR, open | 2774, Rust | not a candidate: PR open, not merged (detail fetched 2026-11-18T05:58Z) |
| 2.7 | [microsoft/oxidizer#864](https://github.com/microsoft/oxidizer/pull/864): perf(observed): benchmark filtering and sampler overhead, compare filtering with tracing | PR, open | 193, Rust | not a candidate: 193 stars (<1,000) |
| 2.8 | [AlphaB135/BitQuan#146](https://github.com/AlphaB135/BitQuan/issues/146): refactor: Abstract SystemTime via TimeSource trait for testability | issue, open | 9, Rust | not a candidate: 9 stars (<1,000) |
| 2.9 | [cocoindex-io/cocoindex#2712](https://github.com/cocoindex-io/cocoindex/pull/2712): fix(files): don't trust an equal mtime that was fresh when read | PR, open | 11908, Rust | not a candidate: PR open, not merged (detail fetched 2026-11-18T05:58Z) |
| 2.10 | [erans/pgsqlite#92](https://github.com/erans/pgsqlite/issues/92): test_batch_delete_edge_cases is flaky under full-suite parallel load | issue, open | 227, Rust | not a candidate: 227 stars (<1,000) |

#### Q3: `"hardcoded path" language:Rust`

Request started 2026-11-18T05:50:12Z; `total_count` 26859; `incomplete_results` false; 10 items.

| # | Result | Kind, state at search | Repository stars, language | Assessment |
| ---: | --- | --- | --- | --- |
| 3.1 | [vicrodh/chilpa#39](https://github.com/vicrodh/chilpa/pull/39): feat: resolve hardcoded path from env | PR, open | 104, Rust | not a candidate: 104 stars (<1,000) |
| 3.2 | [Snehal1112/rocket-factory#82](https://github.com/Snehal1112/rocket-factory/issues/82): Audit: docs drift and hardcoded paths (group B) | issue, open | 0, Rust | not a candidate: 0 stars (<1,000) |
| 3.3 | [getfloresta/Floresta#1460](https://github.com/getfloresta/Floresta/issues/1460): [Bug] `floresta-rpc` tests locate `florestad` through a hardcoded path | issue, open | 453, Rust | not a candidate: 453 stars (<1,000) |
| 3.4 | [66HEX/frame#257](https://github.com/66HEX/frame/pull/257): fix(bundle-windows): resolve Visual Studio and Inno Setup instead of hardcoded paths | PR, open | 2306, Rust | not a candidate: PR open, not merged (detail fetched 2026-11-18T05:58Z) |
| 3.5 | [Snehal1112/rocket-factory#95](https://github.com/Snehal1112/rocket-factory/issues/95): Hardcoded paths outside Layout, and the test that should catch them does not | issue, open | 0, Rust | not a candidate: 0 stars (<1,000) |
| 3.6 | [CharleGutierrez/hagibis#1](https://github.com/CharleGutierrez/hagibis/pull/1): Add ARCHITECTURE.md and fix hardcoded path in tests | PR, open | 0, Rust | not a candidate: 0 stars (<1,000) |
| 3.7 | [nosduco/nforwardauth#58](https://github.com/nosduco/nforwardauth/pull/58): Tweak hardcoded paths | PR, open | 161, Rust | not a candidate: 161 stars (<1,000) |
| 3.8 | [avariiiiiiiiii/UniDTK#164](https://github.com/avariiiiiiiiii/UniDTK/issues/164): Explorer: Hardcoded C: paths for Recycle Bin and home fallback | issue, open | 1, Rust | not a candidate: 1 stars (<1,000) |
| 3.9 | [Perxenic-Acid/baba_is_rust#1](https://github.com/Perxenic-Acid/baba_is_rust/pull/1): security: fix path traversal, pin dependencies, remove hardcoded paths | PR, open | 0, Rust | not a candidate: 0 stars (<1,000) |
| 3.10 | [Growth-Circle/cadis#5373](https://github.com/Growth-Circle/cadis/issues/5373): pr-autoreview.yml hardcoded private paths in regex | issue, open | 59, Rust | not a candidate: 59 stars (<1,000) |

#### Q4: `"thread" "outlives" language:Rust`

Request started 2026-11-18T05:50:13Z; `total_count` 15104; `incomplete_results` false; 10 items.

| # | Result | Kind, state at search | Repository stars, language | Assessment |
| ---: | --- | --- | --- | --- |
| 4.1 | [avariiiiiiiiii/UniDTK#308](https://github.com/avariiiiiiiiii/UniDTK/issues/308): Micrographics: each uncached DNS lookup starts a thread that outlives its timeout | issue, open | 1, Rust | not a candidate: 1 stars (<1,000) |
| 4.2 | [Tripstack-Corp/spyc#1139](https://github.com/Tripstack-Corp/spyc/issues/1139): jar S4: spyc outlives its terminal | issue, open | 19, Rust | not a candidate: 19 stars (<1,000) |
| 4.3 | [cerulion-inc/cerulion#676](https://github.com/cerulion-inc/cerulion/issues/676): gateway: an egress lane's sender thread outlives its topic's demand, so a standing netd accumulates threads | issue, open | 120, Rust | not a candidate: 120 stars (<1,000) |
| 4.4 | [NVIDIA/cuda-rust#1768](https://github.com/NVIDIA/cuda-rust/issues/1768): [BUG]: shared_allocation pending_copy test fails under parallel test threads | issue, open | 4066, Rust | not a candidate: issue open (detail fetched 2026-11-18T05:58Z) |
| 4.5 | [ericvh/purgatorio#256](https://github.com/ericvh/purgatorio/issues/256): Kernel: an export worker outlives net.wasm's export test (TCG, one CPU, intermittent) | issue, open | 2, Rust | not a candidate: 2 stars (<1,000) |
| 4.6 | [astral-experiments/ohm-rust#226](https://github.com/astral-experiments/ohm-rust/pull/226): Reuse the outlives environment for opaque-item lifetime pairs | PR, open | 0, Rust | not a candidate: 0 stars (<1,000) |
| 4.7 | [daniel-freiermuth/flourish#176](https://github.com/daniel-freiermuth/flourish/pull/176): feat(daemon): no process the daemon started outlives its stop | PR, open | 2, Rust | not a candidate: 2 stars (<1,000) |
| 4.8 | [RustPython/RustPython#9925](https://github.com/RustPython/RustPython/pull/9925): Stop importing `threading` at interpreter shutdown | PR, open | 22490, Rust | not a candidate: PR open, not merged (detail fetched 2026-11-18T05:58Z) |
| 4.9 | [theSherwood/temen#2881](https://github.com/theSherwood/temen/issues/2881): JIT: a `thread.spawn` after `join_all` drained the joins is never joined and outlives its Domain | issue, open | 1, Rust | not a candidate: 1 stars (<1,000) |
| 4.10 | [jarviisha/onehand#273](https://github.com/jarviisha/onehand/issues/273): An agent's process group outlives onehand when the app exits | issue, open | 1, Rust | not a candidate: 1 stars (<1,000) |

#### Q5: `"global state" bug language:Rust`

Request started 2026-11-18T05:50:14Z; `total_count` 18462; `incomplete_results` false; 10 items.

| # | Result | Kind, state at search | Repository stars, language | Assessment |
| ---: | --- | --- | --- | --- |
| 5.1 | [sugiura-hiromiti/poison_girl#371](https://github.com/sugiura-hiromiti/poison_girl/issues/371): Avoid process-global state mutation in parallel font and dev-fs tests | issue, open | 2, Rust | not a candidate: 2 stars (<1,000) |
| 5.2 | [openai/codex#60672](https://github.com/openai/codex/issues/60672): Windows: local and SSH projects disappear from the sidebar after repeated global-state parse failures | issue, open | 135697, Rust | not a candidate: issue open (detail fetched 2026-11-18T05:58Z) |
| 5.3 | [openai/codex#55565](https://github.com/openai/codex/issues/55565): [Linux/Wayland] Disabled avatar bounds oscillate, triggering ~10 full global-state saves/sec | issue, open | 135697, Rust | not a candidate: issue open (detail fetched 2026-11-18T05:58Z) |
| 5.4 | [Pometry/Raphtory#2648](https://github.com/Pometry/Raphtory/issues/2648): `ShuffleComputeState::copy_over_next_ss` does not copy over global state | issue, open | 654, Rust | not a candidate: 654 stars (<1,000) |
| 5.5 | [quickwit-oss/quickwit#6840](https://github.com/quickwit-oss/quickwit/issues/6840): [bug] File-backed metastore holds the global state lock across a lazy index load, stalling unrelated metadata operations | issue, open | 11803, Rust | not a candidate: issue open (detail fetched 2026-11-18T05:58Z) |
| 5.6 | [brig-sh/hvi-vmm#123](https://github.com/brig-sh/hvi-vmm/issues/123): boot takes process-global state from its embedder and never returns it | issue, open | 10, Rust | not a candidate: 10 stars (<1,000) |
| 5.7 | [getwyrd/wyrd#387](https://github.com/getwyrd/wyrd/issues/387): Accept ADR-0035 — No DST-reachable shared mutable global state (seed-determinism discipline) | issue, open | 0, Rust | not a candidate: 0 stars (<1,000) |
| 5.8 | [265866/agent-browser#1608](https://github.com/265866/agent-browser/pull/1608): fix(wait): stop the global --state flag from swallowing `wait <sel> --state hidden` | PR, open | 0, Rust | not a candidate: 0 stars (<1,000) |
| 5.9 | [model-checking/kani#5150](https://github.com/model-checking/kani/issues/5150): Models for process-global OS state used by `std::sys` (environment, `execve`, signal alt stacks) | issue, open | 3703, Rust | not a candidate: issue open (detail fetched 2026-11-18T05:58Z) |
| 5.10 | [openai/codex#61549](https://github.com/openai/codex/issues/61549): [Windows desktop] Mouse Back after Upgrade causes persistent Loading screen across app and PC restarts | issue, open | 135697, Rust | not a candidate: issue open (detail fetched 2026-11-18T05:58Z) |

#### Q6: `"std::process::Command" unexpected language:Rust`

Request started 2026-11-18T05:50:15Z; `total_count` 2007; `incomplete_results` false; 10 items.

| # | Result | Kind, state at search | Repository stars, language | Assessment |
| ---: | --- | --- | --- | --- |
| 6.1 | [rust-lang/rust#122660](https://github.com/rust-lang/rust/issues/122660): std::process::Command sometimes ignores PATH env variable on Windows | issue, open | 122301, Rust | not a candidate: issue open (detail fetched 2026-11-18T05:58Z) |
| 6.2 | [EffortlessMetrics/tokmd#2946](https://github.com/EffortlessMetrics/tokmd/pull/2946): sentinel: harden git subprocess boundaries with tokmd_git 🛡️ | PR, open | 5, Rust | not a candidate: 5 stars (<1,000) |
| 6.3 | [ocx-sh/ocx#581](https://github.com/ocx-sh/ocx/issues/581): ci: run clippy on the Windows leg of verify-deep | issue, open | 20, Rust | not a candidate: 20 stars (<1,000) |
| 6.4 | [rust-lang/rust#148278](https://github.com/rust-lang/rust/issues/148278): Command::spawn() does not clear stdout's buffer on child process | issue, open | 122301, Rust | not a candidate: issue open (detail fetched 2026-11-18T05:58Z) |
| 6.5 | [anefzaoui/misterdownloadmanager#1333](https://github.com/anefzaoui/misterdownloadmanager/pull/1333): fix(handoff): browser downloads open only dialog without raising or launching main window | PR, open | 12314, Rust | not a candidate: PR open, not merged (detail fetched 2026-11-18T05:58Z) |
| 6.6 | [EffortlessMetrics/perl-lsp-swarm#14150](https://github.com/EffortlessMetrics/perl-lsp-swarm/issues/14150): fix(ci-hygiene): make relative PATH discovery coherent with child cwd | issue, open | 0, Rust | not a candidate: 0 stars (<1,000) |
| 6.7 | [surrealdb/surrealdb#5502](https://github.com/surrealdb/surrealdb/issues/5502): Expose a validate SurrealQL function | issue, open | 33316, Rust | not a candidate: issue open (detail fetched 2026-11-18T05:58Z) |
| 6.8 | [mattsse/chromiumoxide#71](https://github.com/mattsse/chromiumoxide/issues/71): Drop stderr/stdout reader may cause some unexpected behavior | issue, open | 1437, Rust | not a candidate: issue open (detail fetched 2026-11-18T05:58Z) |
| 6.9 | [reubeno/brush#1252](https://github.com/reubeno/brush/pull/1252): fix(commands): resolve relative external command paths against shell cwd | PR, open | 2314, Rust | not a candidate: PR open, not merged (detail fetched 2026-11-18T05:58Z) |
| 6.10 | [crossterm-rs/crossterm#692](https://github.com/crossterm-rs/crossterm/issues/692): `libc::read` syscall can block reading `stdin`  | issue, open | 4267, Rust | not a candidate: issue open (detail fetched 2026-11-18T05:58Z) |


## What a catch would need, and what this search found

The caught rule has two legs: a named TD code on the faithful buggy port,
and admission of the fixed or capability-passing version. Of the cases whose
code was quoted, amaperf and chrono_anchor were run against both legs, and
finch-rust and uniswap-utils against the first; evm-units and sha-rust were
assessed from their code.

- **First leg.** `TD0501` and `TD0602` fire on every ported ambient
  environment, filesystem, process, clock or spawn call in its std spelling:
  `TD0501` ×3 in amaperf, for `std::env::var`, `File::open` and
  `File::create`; `TD0602` ×2 and `TD0501` ×3 in chrono_anchor, for
  `std::thread::spawn`, `Command::new` and `std::thread::sleep`.
  finch-rust and uniswap-utils fail this leg: no TD code fires on their
  buggy ports, because their only effect is a call into the malicious
  dependency, which rustc rejects with `E0433`.
- **Second leg.** amaperf, chrono_anchor, evm-units and sha-rust fail it,
  each on at least one of: an external HTTP client, with no `Net::get` yet
  (amaperf, evm-units, sha-rust); a process run, with no `Proc` operations
  yet (chrono_anchor, evm-units); async or tokio (evm-units); or UDP, with
  no `bind_udp` yet (sha-rust). The amaperf and chrono_anchor runs end in
  those errors, so they do not show whether anything else in either
  capability-passing port would be admitted. chrono_anchor fails first on
  the missing process run; whether it fails on anything else is untested.

The other 75 entries have no portable source, so this search tested nothing
about them.

None of this is a malware-proofing claim. The rejections above are of
ambient std calls in quoted code, and malicious code can still try rustc
soundness holes (`reviews/2026-11-17-red-team-opus.md`).

## Registered protocol (verbatim)

The file below is `target/work/protocol.md` in the fx-zu4f.5 builder's WORK
directory, SHA-256
`0e5cebada4f0c379f72480a10331a55a18fdff37377659510f9cea0b6b424fb0`.

````markdown
# fx-zu4f.5 operational search protocol

Prepared before examining candidates, on 2026-11-18, against worktree base
`33085fd851224ec835230c3f7f068ee9cd5e40d2`. The frozen criteria in the bead
remain binding. This file supplies methods and the explicit conservative P2
narrowing below; it does not change the bug classes or caught rule.

## Registration and stop

The builder writes this file and `PROTOCOL-READY`, then stops. No advisory
fetch, repository discovery, candidate lookup, or GitHub search precedes
tide-lead recording this file's SHA-256 on fx-zu4f.5 and sending `go`.
The builder does not write the bead ledger. The final landing record will
include this protocol verbatim and its registered digest so the ignored WORK
artifact is not the only durable record. Any missing registration or access
failure is reported rather than silently changing the search.

## P1: RustSec malicious-crate advisories

After `go`, fetch only the advisory database:

```sh
git clone --no-checkout https://github.com/rustsec/advisory-db.git target/work/advisory-db
git -C target/work/advisory-db rev-parse HEAD
```

Record the full returned commit, fetch UTC timestamp, remote URL and Git
version. Treat that first HEAD as the immutable search snapshot; do not pull
or update it. Read database files with `git ls-tree` and `git show` at that
commit, without checking out or executing repository contents. The resulting
advisory corpus is the Markdown advisory records under `crates/`, including
withdrawn records. Record the inventory count and advisory publication-date
cutoff, inclusive through 2026-11-18; use each record's advisory `date`, not
its latest edit date. Missing or malformed dates are reported as unresolved,
not silently dropped. Link records using their commit-pinned GitHub URLs and
their RustSec IDs.

Identify malicious advisories by reading the metadata and prose of every
in-date advisory, not by selecting only a few familiar crate names or only
one metadata spelling. A record qualifies if it reports intentional malicious
behavior in a crates.io crate: for example a malicious-package designation,
intentional credential theft, backdoor, or malicious typosquat. Mere
vulnerability, unsoundness, unmaintained status, dependency compromise
speculation, or incidental occurrence of the word "malicious" does not
qualify. Record the explicit statement supporting inclusion. Ambiguous
records get an explanation in the screening notes; do not infer intent from
an ordinary bug. Check the bead-named rustdecimal, faster_log, and
async_println entries against this same pinned inventory without giving them
preferential inclusion. Multiple qualifying advisories remain separate
records, with shared incidents cross-linked.

For every qualifying advisory, read its text and relevant linked primary
incident reports or reputable write-ups. Do not download, install, build, run,
or fetch the source of a malicious crate, including archived crate source.
Use code only where quoted in the advisory or reputable write-ups, and cite
the exact excerpt source. A linked archive may establish historical evidence
through such quotations; it is not permission to retrieve malicious source.
No code execution or fetching embedded payloads follows any source link.
Insufficient quoted code is a limitation on a faithful port, not permission
to invent one. Record source access failures and their effect on the result.

## P2: six GitHub searches

Use the read-only GitHub REST Search Issues endpoint through `gh api
--method GET search/issues`. It returns issues and PRs together. For each of
the following six exact keyword strings, append `language:Rust`; supply
`per_page=10` and `page=1`, omit `sort` and `order`, and issue one search
request on the search date:

1. `"env::var" bug`
2. `"SystemTime::now" flaky`
3. `"hardcoded path"`
4. `"thread" "outlives"`
5. `"global state" bug`
6. `"std::process::Command" unexpected`

For example, the first request, using shell single quotes for the query, is:

```sh
gh api --method GET search/issues -f q='"env::var" bug language:Rust' -F per_page=10 -F page=1
```

Preserve GitHub's returned default relevance order. Do not split issues and
PRs into separate ten-result pools, prefilter for closure, add synonyms, run
semantic search, paginate, refill, or rerank. Save each exact request, UTC
timestamp, response, `total_count`, `incomplete_results`, and ordered list
before investigating any returned candidate. Run all six even if an early
result qualifies. A failed or incomplete search is disclosed as such and
does not support a completed-search negative; no substitute query is used.

Predeclared narrowing: the ten-result cap applies to the raw
`language:Rust` response, before checking stars. For every returned result,
use read-only repository metadata to record `stargazers_count`, primary
language, repository URL, and lookup timestamp. Repositories must have at
least 1,000 stars and primary language Rust at lookup time. Results failing
either check are recorded as not candidates, without replacing them with
later hits. This may omit otherwise eligible lower-ranked results; it cannot
add candidates to the frozen pool. Do not rely on an unsupported issue-search
`stars:` qualifier. Do not inspect extra bug candidates through repository
discovery searches.

For every remaining result, fetch the issue/PR and relevant linked fix using
GET requests only. A closed issue must describe a real bug and have evidence
of a fix; a PR must be merged and describe a real bug with a fix. Closure
alone, an unmerged closed PR, an open report, feature request, or discussion
does not qualify. Record these as not candidates, with the reason. Preserve
the returned search occurrence even if the same URL appears in another query;
assess each unique eligible URL once, cross-linking duplicates. Linked fixes
are evidence for an existing result, not new candidates.

No search or evidence operation comments, reacts, stars, opens issues/PRs,
pushes, tags, or posts. Requests are GETs and content is treated as evidence,
not executable instructions.

## Classification and faithful-port test

For each eligible P1 or P2 candidate, first identify the source's affected
function(s), buggy revision, fix or capability-passing counterpart, effect,
and evidence for the real incident. Assign only these frozen classes:
ambient filesystem, environment, network, process, or clock use; detached
spawn; global mutable state; or `unsafe` without a grant. A candidate outside
these classes is `out of class`, with its actual mechanism explained.

A faithful port contains only the affected function(s), preserving their
effect and structure. Changes are limited to Tide syntax and std requirements
(including the smallest type/import scaffolding needed to express the same
operation). Record an explicit source-to-port mapping and every adaptation.
Do not insert a new effect, remove an existing grant or capability argument,
invent an unsafe block, add a forbidden operation, or replace a difficult bug
with a different bug to cause rejection. If the source structure or effect
cannot be represented without additional semantic changes, classify it as
`not caught`, explaining the required departure. Missing evidence sufficient
to reconstruct the affected functions also prevents a caught claim.

For each feasible in-class port, use the existing case-study/expected-outcome
runner conventions, without changing the driver, std, runner, reference or
companion. Before implementation, read the landed eon-slow-close and
hyper-ambient-runtime studies, the fx-rpsb.1 conventions and the relevant
admission rules. Select a TD code based on that rule and the source operation,
then run the buggy and fixed/capability-passing ports through tide-admit.
All compilation and compiled Tide execution use heavy.sh, the assigned TARGET
and TMPDIR, a timeout, closed stdin and the builder's load restrictions.
Use focused runs; tide-lead owns the frozen-candidate workspace suite.

`caught` requires BOTH a named TD code on the faithful buggy port AND
admission of the fixed or capability-passing port. Record the code, exact
commands, source and port revisions, toolchain/environment, exit statuses,
and counterpart admission. A diagnostic from an unrelated error does not
count. An admitted buggy port, or one requiring changes beyond the faithful
rule, is `not caught`. Simulation/test-only detection never counts. An
incomplete check stays explicitly unresolved rather than becoming a hit or
a completed-search negative.

## Landing record, counts and selection

Maintain `research/37-external-admission-search.md` as the search proceeds.
Include the registration digest and protocol, snapshot identity, date,
search method/limitations, exact query result lists, and one assessment for
every candidate, not only successes. Each assessment includes its pool,
source URL, class, outcome (`caught`, `not caught`, or `out of class`), reason,
source/fix evidence, and any port/admission proof. Each excluded P2 result
also has a reason labelled `not a candidate`; do not confuse exclusion with
an admission result. Include explicit notes for ambiguous P1 screening and
unresolved source/access failures.

Report raw P2 occurrences, unique URLs, duplicates, exclusions and eligible
candidates separately. The examined-candidate count counts each eligible
candidate once (each advisory ID for P1; each unique issue/PR URL for P2),
with caught/not-caught/out-of-class subtotals and unresolved entries disclosed
separately. Cross-pool overlap is labelled. Reconcile all lists and counts;
do not present the sampled counts as an effectiveness or population rate.

Examine the entire registered pool even after a hit. If several candidates
are caught, choose the one with the shortest faithful affected-function port;
break ties by clearest primary buggy/fixed evidence, then stable source URL
order. List other catches in the search record. The selected case study
cites primary incident and fix sources, license evidence, model boundaries,
and expected buggy rejection/fixed admission. Present it as a demonstration
of the class and "no accidental ambient authority". For malicious crates,
state that this is no malware-proofing claim: malicious code can still try
rustc soundness holes, as the cited red-team review explains.

If the completed search has no catches, write a documented negative and no
case study, with the coordinator notified through WORK/report.md for
fx-qlek.7. Any incomplete pool is disclosed and is not described as an
exhaustive negative. No widening, replacement searches, or stretched ports
are permitted to avoid a negative result.
````
