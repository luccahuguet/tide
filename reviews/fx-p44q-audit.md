# fx-p44q audit: taught surfaces against what v0 delivers

Bead `fx-p44q`. Tree: `main` at `502650c` (fx-iwjq included). Audited:
`README.md`, `design/reference.md` (core, sections 0–13), and
`design/reference-companion.md` (C14–C20). Each surface is checked against
`crates/tide-admit`, `crates/tide-std`, `crates/tide-macros`, `conformance/`
and `scripts/`. Nothing was built: every row is checked by reading source.
Evidence is `file:line` in the tree, or "absent" when a search of `crates/`
for the item's name found nothing.

Every surface below is v0 tier (`design/01-design.md` tier table) unless the
Tier column says otherwise.

Status:

- **implemented**: built as taught; provisional details can differ.
- **partial**: built in part. The docs now say which part, with the
  `(partial)` label or a reworded sentence.
- **planned**: not built. The docs now carry `(planned)`, or a sentence
  that says so.
- **reworded**: a README claim restated in a graded form that holds for v0.

Abbreviations: `A` = `crates/tide-admit/src`, `S` = `crates/tide-std/src`,
`M` = `crates/tide-macros/src`. Every `FX` code the core reference teaches,
except `FX0516`, `FX0920` and `FX0921`, is emitted by the driver: each has a
`Code::FxNNNN` use outside `A/diag.rs` (one row per group below).

## Toolchain and driver

| # | Surface | Where taught | Status | Evidence |
| --- | --- | --- | --- | --- |
| 1 | Admission driver: one `.rs` crate root, rustc built against Tide's std | ref 0; README Status | implemented | `A/main.rs:56`, `A/invocation.rs:164-335` |
| 2 | Stable FX codes for Rust habits | ref 0 | implemented | `A/diag.rs:180-252`; uses in `A/walk/*.rs` |
| 3 | Machine-applicable fixes, NDJSON diagnostics with `safe`/`suggest` | ref 0, 13 | planned | absent: no `ndjson`, `Applicability` or fix plan in `A`; only `--error-format short` (`A/invocation.rs:208`) |
| 4 | `tide check`, `tide fix`, `tide test`, `tide sim`, `tide eval`, `tide explain`, `tide review`, `tide fmt` | ref 6.10, 7.4, 9, 10, 11, 12.1, 13; C15.4, C17, C18, C19 | planned | absent: no `tide` binary; `tide-admit` is the only CLI (`A/main.rs:103`) |
| 5 | `tide test` precursor: `tide-admit --test` builds a libtest binary with property, sim and contract tests | ref 10.2; README Status | partial | `A/invocation.rs:164`, `A/synthesis/contracts.rs`; no `tide test` frontend, filter or `--trace` |
| 6 | `tide sim --replay` precursor: `--tide-run-test NAME [--seed N]`, `--replay MANIFEST` | ref 6.8, 13; C17 | partial | `A/invocation.rs:188-275,1057-1058`; `S/replay.rs:6`; tests only, same machine; no `tide sim <bin>` |
| 7 | `tide review` precursor: `<crate>.interface.json` and `tide-admit --tide-diff` (declared authority, holes) | ref 11; README bullets | partial | `A/interface.rs:68`, `A/interface/diff.rs:28`; no contracts, grants, approvals, exercised authority |
| 8 | Typed holes (`todo!()`, `hole!(name)` reporting expected type and bindings) | ref 2.1, 13 | planned | absent: no `hole` macro in `M` or `S`; `todo!` is Rust's (`S/root.rs:22`) |
| 9 | `--tide-grant=unsafe` operator grant | ref 11, 12.4 | implemented (as an operator flag) | `A/invocation.rs:170` |
| 10 | `-Zallow-features=`, `-Zpolonius=no` (ADMIT-2) | README Status | implemented | `A/invocation.rs:291-292` |
| 11 | Overflow traps in every profile | ref 4.1; README | implemented | `A/invocation.rs:302` (`-Coverflow-checks=on`) |
| 12 | No `build.rs`, install hooks or build-time package code (`FX0915`); driver runs rustc, not Cargo | ref 12.4; README | implemented | `A/invocation.rs:598-603` |
| 13 | Edits check in tens of milliseconds | README bullets | reworded | D12 target; no incremental front end exists, each check is a rustc run (`A/invocation.rs`) |
| 14 | Kernel sandbox for test and sim runs (RUN-3), unreviewed-branch guarantee (RUN-5) | ref 11; C19; README | planned | absent: no Landlock or seccomp in `crates/`; bead `fx-ip4x`; `S/lib.rs:247-249` says so |
| 15 | Conformance corpus through stable rustc and the driver | README Status | implemented | `conformance/run.sh`, `conformance/rules.md` |
| 16 | Token count of the core | C header | implemented | `scripts/count-tokens.sh` |

## Surface, lifetimes, numbers, traits (ref 1–5)

| # | Surface | Where taught | Status | Evidence |
| --- | --- | --- | --- | --- |
| 17 | Rust's std types and modules re-exported | ref 1 | partial | `S/root.rs:18-30`; `Arc`/`Weak` lack parts of Rust's API (row 50) |
| 18 | Built-in `!` forms; other forms `FX0103`; `macro_rules!`/proc macros `FX0101` | ref 2.1 | implemented | `A/walk/surface.rs:100`, `A/invocation.rs:981`, `S/root.rs:22-34` |
| 19 | `dbg!` an error in release (`FX0507`); library `println!` (`FX0508`) | ref 2.1, 6.5 | implemented | `A/walk/sinks.rs:18-25` |
| 20 | `log::info!`, `tracing::info!` prefixed forms | ref 2.1 | planned | absent; `S/lib.rs:242` lists them as not provided |
| 21 | `invariant!` | ref 2.1, 10.1 | implemented | `M/lib.rs:274` |
| 22 | Closed derive set; `Serialize`/`thiserror` `FX0102` | ref 2.2 | implemented | `A/walk/surface.rs:64-69` |
| 23 | `Error` derive with thiserror semantics | ref 2.2 | implemented | `M/lib.rs:38`; limits at `S/lib.rs:121-123` |
| 24 | `Encode`/`Decode`, `std::json` | ref 2.2 | implemented | `M/lib.rs:63-72`, `S/json.rs:96-103` |
| 25 | `NetError`, `UrlError` as std error types | ref 2.2, 6.7 | planned | absent (covered by the 6.7 label) |
| 26 | `const fn` rules (`FX0518`) | ref 2.3 | implemented | `A/walk.rs:1297` |
| 27 | `#[tide::main]`, `#[pure]`, `#[sim_test]`, `#[requires]`, `#[ensures]`, `#[invariants]` | ref 2.3 | implemented | `M/lib.rs:23,55,84,253,260,267` |
| 28 | `#[protected]`, `#[trusted_pure]` | ref 2.3, 11 | planned | no attribute in `M` or `S`, so neither is usable; the driver already records trusted types and holes for `#[trusted_pure]` (`A/walk.rs:103-105,260-273`, `A/pure.rs:441-443`); `#[protected]` absent |
| 29 | `#[path]` (`FX0911`), `#[tokio::main]` etc. rejected | ref 2.3, 12.1 | implemented | `A/invocation.rs:927`; `A/walk/habits.rs:19` |
| 30 | Lifetimes as Rust 2024; `FX0201`, `FX0202`, `FX0203`, `FX0210` | ref 3 | partial | `A/walk/surface.rs:131,155`, `A/diag/labels.rs:129,136`; wrong-input elision keeps rustc's error with no FX code (`crates/tide-admit/tests/invocation.rs:50-59`) |
| 31 | `Arena`, `Id` | ref 3.6 | implemented | `S/arena.rs:35-154` |
| 32 | Lossless `as` (`FX0301`); `truncate`, `round_from`, `try_from_float` | ref 4.2 | implemented | `A/walk/surface.rs:175`, `S/convert.rs:6-10` |
| 33 | `std::float`, `std::math`; float observers and math methods `FX0520` | ref 4.3 | implemented | `S/float.rs:30-35`, `S/math.rs` |
| 34 | Power ceiling `FX0401`–`FX0408` | ref 5 | implemented | `A/walk/traits.rs:39-274` |

## Authority (ref 6)

| # | Surface | Where taught | Status | Evidence |
| --- | --- | --- | --- | --- |
| 35 | No ambient std I/O (`FX0501`); admission allowlist (`FX0520`) | ref 6.1, 6.9 | implemented | `A/policy.rs:66`, `A/walk.rs:330` |
| 36 | `Fs` and `File` methods, `OpenOptions`, `Mode`, `dir`, `read_only`, durability calls | ref 6.1, 6.3 | implemented | `S/fs.rs:54-486` |
| 37 | `Net::connect`, `bind`, `only`, `no_bind`; `TcpStream`, `TcpListener` | ref 6.1, 6.7 | implemented | `S/net.rs:137-322` |
| 38 | `Net::get`, `NetError`, `Url`, `UrlError` (HTTP) | ref 6.1, 6.7, 7.1; C15.2, C21 | planned | absent; `S/lib.rs:240`, `S/lib.rs:277` |
| 39 | `Net::bind_udp`, `UdpSocket`, `Net::resolve` | ref 6.1, 12.3 | planned | absent |
| 40 | `Clock` | ref 6.1 | implemented | `S/time.rs:28-69` |
| 41 | `Rand` draws (`next_u64`, `gen_range`, ...), `rng()`, `Pcg64` | ref 6.1, 12.3; C14.2 | planned | `S/lib.rs:283-289`: an identity with no methods |
| 42 | `RandomState::from_rand`, fixed-key `HashMap` | ref 6.4, 6.8 | partial | `S/collections.rs:23-40`: real `Rand` only; a simulated `Rand` panics through `unmodeled("Rand")` |
| 43 | `Env` | ref 6.1 | implemented | `S/env.rs:55-148` |
| 44 | `Stdin` | ref 6.1 | implemented | `S/io/stdio.rs:49-124` |
| 45 | `Proc`: `command`, `exit`, `only`, `Command`, `signals` | ref 6.1, 12.3; C14.2, C20 | planned | `S/lib.rs:301-308`: an identity with no operations |
| 46 | `Tasks` methods (scope, `spawn_isolated`, `shield`, `checkpoint`, `yield_now`, `limit`) | ref 6.1, 7.1 | implemented | `S/task.rs:1094-1307` |
| 47 | `Tasks::fault_point` | ref 6.1; C17 | planned | absent |
| 48 | `#[tide::main] fn main(root: Root)`; `FX0512`, `FX0503`, `FX0504`, `FX0506` | ref 6.2 | implemented | `M/lib.rs:84`, `A/invocation.rs:1017`, `A/walk/authority.rs:93,160`, `A/walk.rs:1535` |
| 49 | No globals (`FX0502`) | ref 6.4 | implemented | `A/walk.rs:1173` |
| 50 | `Arc`, `Weak` with Rust's API minus count observers | ref 1, 6.4 | partial | `S/sync.rs:18-137`: no `Arc::ptr_eq`; unsized payloads not provided (`S/lib.rs:243`) |
| 51 | Sinks, `io::stdout()`/`stderr()`, `root.log` | ref 6.5 | implemented | `S/io/stdio.rs:169-172`, `S/log.rs:52-59` |
| 52 | `std::trace::span` | ref 6.5, 12.3 | planned | absent; `S/lib.rs:241` |
| 53 | `Secret<T>` | ref 6.5, 12.3 | planned | absent; `S/lib.rs:242` |
| 54 | `Pure` decided structurally (`FX0517`), `#[pure]` (`FX0511`) | ref 6.6 | implemented | `A/pure.rs:210`, `A/walk.rs:1247` |
| 55 | `Root::sim`, `SimRoot`, `sim.world.file/args/env_var/stdin` | ref 6.8 | implemented | `S/root.rs:289`, `S/sim.rs:19-121` |
| 56 | `sim.elapsed`, `records`, `trace`; `Record`, `Sink` | ref 6.8; C17 | implemented | `S/sim.rs:66-144` |
| 57 | `sim.world.http`, `Request`, `Reply` | ref 6.8, 12.3; C15.2, C17 | planned | absent |
| 58 | `sim.net.drop_all` | ref 6.8; C17 | planned | absent |
| 59 | `sim.faults` operations, `Fault`, `sim.node`, `NodeRoot`, `SimNode`, `NodeTask`, `run_for`, `run_until`, crashes and restarts | ref 6.3, 6.8, 12.3; C15.4, C17 | planned | `S/sim.rs:124-127`: `Faults` is an identity with no operations; the rest absent |
| 60 | Each simulated run writes a replay manifest | ref 6.8; C17 | partial | only driver `--tide-run-test` runs record (`S/replay.rs:6`, `A/invocation.rs:188`); direct binary runs do not |
| 61 | Virtual time; wall time from 2000-01-01; deadlock report | ref 6.8 | implemented | `S/time.rs:34`, `S/lib.rs:90-94` |
| 62 | Ambient std calls `FX0501` with fixes | ref 6.9 | partial | code: `A/policy.rs:66`; fixes absent (row 3) |

## Concurrency, parallelism, errors (ref 7–9)

| # | Surface | Where taught | Status | Evidence |
| --- | --- | --- | --- | --- |
| 63 | Scopes, tasks, failure precedence, deadlines, cancellation | ref 7.1, 7.2 | implemented | `S/task.rs:1112-1748` |
| 64 | `FX0601`, `FX0602`, `FX0605` | ref 7 | implemented | `A/walk/habits.rs:19,103`, `A/policy.rs:188` |
| 65 | Channels | ref 7.3 | implemented | `S/sync/mpsc.rs:14-286` |
| 66 | `Shared<T>`, `LockError`; `FX0603`, `FX0604`, `FX0606`, `FX0607`, `FX0608` | ref 7.3, 7.4 | implemented | `S/sync.rs:145-425`, `A/pure.rs:211-212`, `A/policy.rs:206`, `A/walk/sinks.rs:65,125` |
| 67 | Handle creation takes `&Tasks` (`FX0505`) | ref 7.4 | implemented | `A/walk/authority.rs:264`, `S/sync.rs:23-30` |
| 68 | Atomics through `&Tasks` (`AtomicU64::new(0, tasks)`), `std::sync::atomic` | ref 7.4, 12.3 | planned | absent from `S/sync.rs` (Rust's atomics are internal only, `S/sync.rs:12`); `core::sync::atomic` is `FX0520` |
| 69 | `std::sync::raw` | ref 6.6, 7.4, 11 | planned | absent |
| 70 | Runtime chosen by `[bin] runtime`; `"blocking"` | ref 7.5, 12.4 | planned | absent: no manifest reader; the threads runtime is the only one (`S/task.rs`) |
| 71 | Threads runtime, one OS thread per task | ref 7.5 | implemented | `S/lib.rs:56-67` |
| 72 | `join`, `par_map`, `par_for`, `par_sort`; `FX0610`–`FX0612` | ref 8 | implemented (pool partial, outside the reference's text) | `S/par.rs:51-110`, `A/pure.rs:213-214`, `A/walk.rs:968` |
| 73 | `FX0702`, `FX0701` | ref 9 | implemented | `A/walk/habits.rs:124`, `A/policy.rs:216` |
| 74 | Rerun of aborted tests and seeds by `tide test`/`tide sim` | ref 9 | planned | absent (row 4) |

## Contracts (ref 10)

| # | Surface | Where taught | Status | Evidence |
| --- | --- | --- | --- | --- |
| 75 | `#[requires]`, `#[ensures]`, `invariant!`, `old`, `result`, quantifiers; `FX0802`, `FX0803` | ref 10.1 | implemented | `M/lib.rs:253-280`, `A/walk/contracts.rs:64,232` |
| 76 | Checked and sampled clauses, step budget; over-budget clauses reported as unchecked | ref 10.1 | partial | clauses run via driver test builds (`S/lib.rs:7-12`); over-budget evaluation is recorded (`S/contract.rs:163-175`) but no runner prints it (`S/lib.rs:190-192`) |
| 77 | Property tests, guidance, vacuity floor, `#[sim_test]` parameters | ref 10.2 | implemented | `S/lib.rs:124-140`, `M/lib.rs:47` |
| 78 | Generated contract tests, `#[allow(vacuous)]` | ref 10.2 | implemented | `S/lib.rs:141-150`, `A/synthesis/contracts.rs` |
| 79 | Contract hash; human approval for contract changes | ref 10.3 | planned | absent: the trust artifact has no contracts (`A/interface.rs:5`) |

## Trust holes, review, packages (ref 11–12)

| # | Surface | Where taught | Status | Evidence |
| --- | --- | --- | --- | --- |
| 80 | `unsafe` grant (`FX0901`), `// SAFETY:` (`FX0902`), addresses `FX0520` | ref 11 | implemented | `A/walk.rs:549`, `A/walk/modules.rs:159-182` |
| 81 | Trust diff with contracts and grants; approvals recorded outside the agent's write scope and verified by CI | ref 11 | planned | absent (row 7 has the partial precursor) |
| 82 | `tide.seal`, `FX0920`, `FX0921` | ref 11; C18 | planned | absent: no reader of `tide.seal`, no code |
| 83 | `run: covered` testing of unreviewed branches | ref 11; C19 | planned | absent (row 14) |
| 84 | Files are modules; `FX0910`, `FX0912`, `FX0913`, `FX0914` | ref 12.1, 12.2 | implemented | `A/diag/labels.rs:122`, `A/walk/modules.rs:25,94,126` |
| 85 | One-file crate root | ref 12.1 | implemented (through `tide-admit`) | `A/main.rs:56` |
| 86 | Prelude and std paths table | ref 12.3 | partial | rows above; missing items carry `(planned)` where taught |
| 87 | `tide.toml` manifest, `[dependencies]`, per-dependency grants | ref 12.4 | planned | absent: nothing in `crates/` or `scripts/` reads `tide.toml` |
| 88 | Authority policies `[[policy]]`, `FX0516` | ref 12.4 | planned | absent: no code |
| 89 | `tide.lock` with hashes, authority and holes | ref 12.4 | planned | absent: nothing reads or writes `tide.lock` |

## Companion (C14–C20)

| # | Surface | Where taught | Status | Evidence |
| --- | --- | --- | --- | --- |
| 90 | Habit table codes | C14.1 | partial | codes as rows 18–80; the `AtomicU64` and `world.http` fixes are planned (labelled) |
| 91 | Ambient-call table: `rand.gen_range`, `proc.command`, `proc.exit` | C14.2 | planned | rows 41, 45 |
| 92 | Example 15.1 (CLI with files) | C15.1 | implemented | every item it uses is in rows 36, 43, 55 |
| 93 | Example 15.2 (fetcher) | C15.2 | planned | needs rows 38, 57 |
| 94 | Example 15.3 (contracts) | C15.3 | implemented | rows 75–78 |
| 95 | Example 15.4 (crash-safe log) | C15.4 | planned | needs row 59 |
| 96 | File backend behaviour | C16 | implemented | `S/fs.rs`, `S/lib.rs:161-169` |
| 97 | Faults, nodes, crashes, network time, packet loss | C17 | partial | per-packet TCP latency is built (`S/task/net.rs:97-98`, `S/net/sim.rs:181-184`); faults, nodes, crashes, loss and `Net::get` timing are planned (row 59) |
| 98 | Replay manifest contents | C17 | partial | row 60 |
| 99 | Sealed interfaces | C18 | planned | row 82 |
| 100 | Unreviewed-branch testing | C19 | planned | rows 14, 83 |
| 101 | Process handles | C20 | planned | row 45 |

## README claims

| # | Claim | Status | Evidence |
| --- | --- | --- | --- |
| 102 | Lead: "code can't overreach and programs replay bit for bit" | reworded | rows 7, 14, 60; W1 |
| 103 | "No ambient authority ... No code, in any dependency" | reworded | the driver admits one crate (`A/invocation.rs:320-335`), so "any dependency" held only because v0 has none; owner comment 2026-11-15 |
| 104 | "Every program replays ... dependencies included ... the build lists any that do not" | reworded | tests replay through the driver (row 6); no `tide sim <bin>`; no unmodeled-effect listing exists (v1 imports) |
| 105 | "The compiler diffs authority, contracts, and `unsafe` holes" | reworded | row 7: declared authority and holes only |
| 106 | "As safe and fast as Rust", overflow, no build-time package code | implemented | rows 11, 12 |
| 107 | "Edits check in tens of milliseconds" | reworded | row 13 |
| 108 | "Contracts run as checks and tests today" | implemented | rows 75–78 |
| 109 | v0 core tier list | reworded | the list is the design tier; the README now says which parts are not built |
| 110 | Status: "`dbg!` ... remain unimplemented" | reworded | `dbg!` is built (`M/lib.rs:145-182`, `S/root.rs:34`); removed from that list |

## Budget move

To keep the core within D14's 25,000 tokens after labelling, the planned
HTTP detail moved to the companion unchanged: 6.7's `NetError` and `Url`
paragraph is companion 21, and 6.8's `sim.world.http` and
`sim.net.drop_all` rows open companion 17's table. The core keeps
`Net::get` with a `(planned)` label and pointers to both.

## Counts

Rows 1–101 (reference, companion and README Status surfaces):
implemented 52, partial 14, planned 34, reworded 1 (row 13, the README's
speed claim). Rows 102–110 (README claims): reworded 7, implemented 2.

Totals over all 110 rows: implemented 54, partial 14, planned 34,
reworded 8.

Round 1 (Codex, `r1-codex.md`) moved rows 17, 30, 42, 50 and 76 from
implemented to partial and row 97 from planned to partial, corrected row
28's evidence, and added row 110.
