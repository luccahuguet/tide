# Reference readiness — fx-o8o

Verified revision: 31a9800 on main 3fdc020. The orchestrator ran the round-3
corrections outside Codex's sandbox through heavy.sh. The protocol/tasks/
thresholds are unchanged and suites are not frozen. This is not an E1 gate run.
31a9800 was amended into the next commit with only this file changed, so its
proof covers that commit's code.

## Host proof and identities

Host proof on 31a9800 (2026-11-08), run by the orchestrator outside Codex's
sandbox through heavy.sh:

- harness_test.py: 18/18 OK (exit 0).
- references rust, sandboxed: 29/30 PASS in the full run. B9 acceptance
  case1 hit the protocol's 10-second per-test limit once, while the disk
  was under heavy shared I/O; its reference fsyncs the store file and its
  directory. Run alone after that, B9 passed 3 of 3 sandboxed runs at load
  10-11. A fourth run failed only because the owner's weekly cargo-sweep
  timer (Sun 03:00) deleted build fingerprints mid-build. Then B9 passed 5
  of 5 runs once the sweep had ended.
- references tide, sandboxed and --local: 0/30 PASS in each (exit 1).
- No reference grade raised an integrity flag.
- Both Tide files: 26 source-check failures; A2/A3/A4/A10 pass source checks.
  All 30 structure checks pass. Those four tasks execute 7/5/5/4 plain
  acceptance tests successfully, graded from libtest logfiles: 21
  acceptance cases per run. Their only failing records are the
  fuller-runner and separate-library-boundary markers.

The failing-stage profile matches the a1f0b5c and ff2b251 runs. The full
first-failure map is on fx-o8o, in the comments dated 2026-11-08 02:44 and
02:45 UTC (`br --no-db show fx-o8o`). These counts are observations of
31a9800 and do not extend to later revisions.

Rust is pinned to 1.96.0. Both Tide files identify nightly-2026-08-09,
rustc 1.99.0-nightly 771916f9028e7fe56d2685f2c4f698de5d7d6a45,
x86_64-unknown-linux-gnu, LLVM 23.1.0, using production std and the driver.
Dependency identity (SHA-256 of rust/Cargo.lock):
d786dc06f8ba1e6c9b616638b9e90c213ec2a4b7b563767da43878ae8ee1ec1a.
All 13 crates are vendored offline at setup into the checkout's ignored
target/e1-checked/rust-vendor-d786dc06f8ba1e6c, then mounted read-only.

## Complete Tide first-failure map

Every failing stage is covered below. Compiler diagnostics are abridged to
their code and missing item; raw text and command identity remain in the JSON.
The map assigns first errors, not every subsequent compiler diagnostic.

Stage keys: S = source ./check; ML/MT = grade check-lib/check-test metadata;
W = linked writer-harness build; A = linked acceptance-harness build;
B = linked binary build; R = fuller Tide-runner check; L = library-boundary check.
For a row listing several compilation stages the first error is the same in
each, except the explicitly split B3 and C8 cells.

Additional independently failing stages apply to the table's source-failure rows:

| Tasks | Stage / first error | Owner |
| --- | --- | --- |
| All 30 | R, code 2: tide test runner unavailable; property, simulation, generated contracts and vacuity need the fuller runner | fx-nkv.6, fx-m72 |
| All 26 libraries (all except B1/B2/B5/B8) | L, code 2: Tide library boundary unproven; acceptance is a sibling-module probe | fx-dsa |

Thus R applies to every row below, and L to every library row, even when
compilation already failed. A2/A3/A4/A10 have only R/L failures. No failed
test listing or test execution adds another stage. The four binary builds
fail before process cases execute.

| Task | Failing stages | First error (abridged) | Owner(s) |
| --- | --- | --- | --- |
| A1 | S, ML, MT, W, A | E0432: unresolved std::collections | fx-yg5 |
| A2 | R, L | Runner unavailable; library boundary unproven | fx-nkv.6, fx-m72; fx-dsa |
| A3 | R, L | Runner unavailable; library boundary unproven | fx-nkv.6, fx-m72; fx-dsa |
| A4 | R, L | Runner unavailable; library boundary unproven | fx-nkv.6, fx-m72; fx-dsa |
| A5 | S, ML, MT, W, A | E0433: std::float missing | fx-yg5 |
| A6 | S, ML, MT, W, A | E0432: unresolved std::collections | fx-yg5 |
| A7 | S, ML, MT, W, A | E0432: unresolved std::collections | fx-yg5 |
| A8 | S, ML, MT, W, A | E0432: unresolved std::collections | fx-yg5 |
| A9 | S, ML, MT, W, A | E0432: unresolved std::json | fx-08m |
| A10 | R, L | Runner unavailable; library boundary unproven | fx-nkv.6, fx-m72; fx-dsa |
| B1 | S, ML, MT, W, B | E0599: Env::args missing | fx-5vj |
| B2 | S, ML, MT, W, B | E0599: Env::args missing | fx-5vj |
| B3 | S, ML, MT, W, A | S/ML: E0425 io::Result; MT/W/A: E0433 fs::Mode missing | fx-byk |
| B4 | S, ML, MT, W, A | E0432: fs::OpenOptions, io::Write missing | fx-byk |
| B5 | S, ML, MT, W, B | E0432: unresolved std::collections | fx-yg5 |
| B6 | S, ML, MT, W, A | E0432: collections, fs::Mode missing | fx-yg5, fx-byk |
| B7 | S, ML, MT, W, A | E0432: unresolved std::collections | fx-yg5 |
| B8 | S, ML, MT, W, B | E0432: unresolved std::json | fx-08m |
| B9 | S, ML, MT, W, A | E0432: unresolved std::collections | fx-yg5 |
| B10 | S, ML, MT, W, A | E0432: task::Cancelled, time::Duration missing | fx-zc1, fx-o8j |
| C1 | S, ML, MT, W, A | E0599: Vec::par_map missing | fx-iob |
| C2 | S, ML, MT, W, A | E0432: unresolved std::par | fx-iob |
| C3 | S, ML, MT, W, A | E0433: std::sync missing (mpsc::sync_channel import) | fx-o4h |
| C4 | S, ML, MT, W, A | E0432: collections, sync::Shared, task::TaskError missing | fx-yg5, fx-nkv.4, fx-nkv.2, fx-nkv.3 |
| C5 | S, ML, MT, W, A | E0432: task::TaskError, time::Duration missing | fx-nkv.2, fx-nkv.3, fx-o8j |
| C6 | S, ML, MT, W, A | E0432: task::TaskError missing | fx-nkv.2, fx-nkv.3 |
| C7 | S, ML, MT, W, A | Missing requires attribute | fx-mhd |
| C8 | S, ML, MT, W, A | S/ML: requires missing; MT/W/A: ensures missing | fx-mhd |
| C9 | S, ML, MT, W, A | E0432: task::TaskError missing | fx-nkv.2, fx-nkv.3 |
| C10 | S, ML, MT, W, A | E0432: io::{BufRead,BufReader,Write}, net::{TcpListener,TcpStream}, task::TaskError missing | fx-hxw (TCP/Write), fx-nkv.2/fx-nkv.3 (TaskError); fx-ecb (BufRead/BufReader) |

### Ownership evidence

The named owners are prerequisites of fx-ic2. §10.1 assigns data
surface to fx-yg5, codec to fx-08m, process input to fx-5vj, files/io results
and OpenOptions/Mode to fx-byk, cancellation to fx-zc1, time to fx-o8j,
parallelism to fx-iob, channels to fx-o4h, Shared to fx-nkv.4, scoped lifecycle/
failure results to fx-nkv.2/fx-nkv.3, and contract attributes to fx-mhd.
fx-dsa owns separate admitted-library import and blocks fx-ic2.
fx-kw8.9 is closed and is not a remaining failure owner.

C10's TcpListener/TcpStream and stream Read/Write belong to fx-hxw (its
Scope explicitly names them). The owner-approved fx-ecb now owns the generic
std::io::BufRead/BufReader surface retained by design/reference.md §1, over
both TCP and file readers, and blocks fx-ic2. Every first-error component in
this map has an existing prerequisite owner.

Known later prerequisites include fx-ugm for C4's with_all and fx-nkv.5/fx-nkv.6
for seeded simulation: file fixtures B3/B4/B6/B7/B9 and B10/C3–C6/C9/C10 suites.

Later diagnostics may expose further gaps after these first failures clear.
fx-u65 owns the separate negative FX gate; positive references cannot prove it.
fx-3iu is still a protocol prerequisite but manual Display implementations
avoid the optional Error derive in these references.

## Execution, setup and next steps

Driver-only builds use separate output directories for metadata, writer tests,
acceptance tests and binaries. Per-test limits and named libtest log checks
remain active. The first-check library measure also explicitly retains fx-dsa.
Its sibling-module probe also changes crate-path resolution and test selection;
fx-dsa owns replacement with a separate admitted crate.
prepare writes no check/test scripts; the broker dispatches logical commands
directly to trusted harness.py command SESSION MODE. fx-ic2 owns that broker.

Bubblewrap keeps --unshare-all, one writable work tree, and the approved
read-only /usr, /etc/alternatives and optional /nix/store mounts.
Before gate writers, fx-ic2 owes isolation canaries, toolchain/model identities
and confirmation the Nix store contains no repository copy, plus suite freeze.

Remaining work is resolution of the mapped prerequisites and governance
review. fx-ic2 owns gate setup,
isolation canaries, broker enforcement and the pre-session freeze.
