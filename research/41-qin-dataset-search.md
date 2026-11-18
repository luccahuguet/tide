# Search record: the Qin et al. PLDI 2020 concurrency bugs (fx-zu4f.9)

Bead fx-zu4f.9, the final pre-registered round of fx-zu4f's search for an
external concurrency bug that a seeded Tide `sim_test` catches and replays,
following [research/38](38-external-concurrency-search.md) and
[research/39](39-std-concurrency-search.md).
Initial builder: Claude Opus 5.5; r1 corrections: the initial builder, then
three successive Codex (gpt-6.1-sol) authors, 2026-11-18.
This record is a sample description.
Its counts describe these 100 rows only and are not a catch rate.

## Source and who chose the bugs

The pool is the concurrency-bug dataset of Boqin Qin, Yilun Chen, Zeming Yu,
Linhai Song and Yiying Zhang, "Understanding Memory and Thread Safety
Practices and Issues in Real-World Rust Programs", PLDI 2020. The artifact is
[system-pclub/rust-study](https://github.com/system-pclub/rust-study) at
`403b06f30809d10d2bb066d2fbcd33caf3446d17` (README v1.1, 2020-04-17,
Apache-2.0 and MIT). Its README says the commit numbers are in column B of
tabs `section-6.1-blocking` and `section-6.2-non-blocking` of the Google
Sheet `1GWbiPSBCIbqH2g3Vzc-o2XdxOskXyDhkJjNUjFi1pzs`. **The dataset's authors
chose these bugs, not Tide.** Tide's simulator was not used to find any of
them. Each candidate's property would come from the bug's own report.

## Pre-registration

"Pre-registered" means three things. First, the search criteria were
committed in the body of bead fx-zu4f.9 (ledger `1e673c5`, 14:13:02Z).
Second, the operational protocol and the pool export were committed on
branch fx-zu4f.9 as commit `891bdacc3d46a908ee05be1fc996cb28988e9008`
(14:19:58Z). Third, tide-lead recorded the three SHA-256s on the bead before
any bug was examined. The first per-bug request (`fetch-log.txt`) was at
14:21:39Z, after tide-lead's bead comment at 14:20Z.
Every timestamp here, git or br, is self-attested.
Publishing the repository will show only that these commits existed by the
publication date, not the order before it.

| File | SHA-256 |
| --- | --- |
| [`research/protocols/fx-zu4f.9-protocol.md`](protocols/fx-zu4f.9-protocol.md) | `56e9f6e14989d66346c229def74b70dbd943697553ac1be0edf337757f737d57` |
| [`research/protocols/fx-zu4f.9-pool/section-6.1-blocking.csv`](protocols/fx-zu4f.9-pool/section-6.1-blocking.csv) | `f7946a0d61f2478f3cf7a13ba008062c5106b45e0e7273dc15070a082ac8dac8` |
| [`research/protocols/fx-zu4f.9-pool/section-6.2-non-blocking.csv`](protocols/fx-zu4f.9-pool/section-6.2-non-blocking.csv) | `4f5ce8902b98686804d315e44ec3fe683b1ff8844c99c4001b64506cffe9c1a0` |

The protocol lists what was read before it was written: the sheet's page,
to find the tab gids; the two exports; the README; the names in the
reproductions directory; and the metadata of 21 repositories. **Disclosed
deviation:** after the protocol's bytes and hash were fixed, and before the
commit, `git diff --cached --check` printed every data row and most CSV text
(166 flagged lines; embedded-newline header text and some summary lines were
not printed). From then on the builder had seen every data row of the two
tabs: commits, bug IDs, dates and the authors' classification marks. No commit, pull request, issue or
reproduction had been read. The protocol was not changed afterwards.

## Export

Each tab was exported once, by GET of
`https://docs.google.com/spreadsheets/d/1GWbiPSBCIbqH2g3Vzc-o2XdxOskXyDhkJjNUjFi1pzs/export?format=csv&gid=<gid>`,
which answered with a 307 redirect and then HTTP 200, `text/csv`.

| Tab | gid | Exported (UTC) | CSV records | Data rows |
| --- | --- | --- | ---: | ---: |
| `section-6.1-blocking` | `1219539227` | 2026-11-18 14:14:14.474Z to 14:14:16.539Z | 106 | 59 |
| `section-6.2-non-blocking` | `1166836418` | 2026-11-18 14:14:19.543Z to 14:14:20.511Z | 62 | 41 |

There are 100 rows, the same number as the paper's 100 concurrency bugs.
Column B names 99 distinct commits. `5f1ce7f281b4…` appears twice (rows B48
and B59, as `-0` and `-1`), and it is assessed once (protocol §1).

## Method

The [protocol](protocols/fx-zu4f.9-protocol.md) is followed as registered:
§2 maps column B to a repository and its fix and buggy revisions, and gives
the evidence read for every row; §3 covers E1-E3, §4 the version-gap rule
and §5 E4-E5. All upstream access was by read-only GET
(`gh api --method GET`, raw-source GETs, the GitLab API for Redox,
and the linked Mozilla issue API). Nothing upstream was
built, run, installed or posted to. The checks run in order, and each row
records its first failure. A reason that would also fail a later check is
marked "additional" and is not counted.

## Result: a documented negative

**No row of the 100 is eligible, so no model was built or run and no
case study is written.** This was the last round of fx-zu4f's search. The
bead says that after a negative there are no more search rounds unless
Lucca asks. fx-qlek.7 launches with the negatives.

| Count | Value |
| --- | ---: |
| Rows (59 blocking, 41 non-blocking) | 100 |
| Unique fix commits | 99 |
| Duplicates (one commit in two rows, B48 and B59) | 1 |
| Screened | 100 |
| Not eligible, E1 (no public fix) | 2 |
| Not eligible, E2 (no stated symptom) | 28 |
| Not eligible, E3 (not concurrency-shaped) | 2 |
| Not eligible, E4 (not portable to Tide v0) | 68 |
| Not eligible, E5 (outside the simulator) | 0 |
| Eligible | 0 |
| Caught / not caught / unportable | 0 / 0 / 0 |
| Unresolved | 0 |

The screen is complete: every row's fix lookup reached a decision. Ninety-nine
column-B commits resolved by GET in the project's repository (or the
repository it was renamed to). Row N34's `redox#303` names an issue whose
accessible saved evidence names no fix (current notes access is limited, below),
and there is no merge request 303 on the project's
GitLab and no pull request 303 on its GitHub mirror.

By tab and project, counting rows by first failure:

| Tab | Project | Rows | E1 | E2 | E3 | E4 | E5 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| blocking | servo | 12 | 0 | 1 | 0 | 11 | 0 |
| blocking | servo-webrender | 1 | 0 | 0 | 0 | 1 | 0 |
| blocking | tikv | 4 | 0 | 1 | 0 | 3 | 0 |
| blocking | redox | 2 | 0 | 0 | 0 | 2 | 0 |
| blocking | crossbeam | 2 | 0 | 0 | 1 | 1 | 0 |
| blocking | rust-threadpool | 1 | 0 | 0 | 0 | 1 | 0 |
| blocking | rayon | 2 | 0 | 1 | 0 | 1 | 0 |
| blocking | lazy-static.rs | 1 | 1 | 0 | 0 | 0 | 0 |
| blocking | parity-ethereum | 34 | 0 | 3 | 0 | 31 | 0 |
| non-blocking | servo | 18 | 0 | 7 | 1 | 10 | 0 |
| non-blocking | tikv | 3 | 0 | 2 | 0 | 1 | 0 |
| non-blocking | crossbeam | 8 | 0 | 5 | 0 | 3 | 0 |
| non-blocking | rand | 2 | 0 | 2 | 0 | 0 | 0 |
| non-blocking | redox | 3 | 1 | 2 | 0 | 0 | 0 |
| non-blocking | tock | 2 | 0 | 1 | 0 | 1 | 0 |
| non-blocking | parity-ethereum | 4 | 0 | 2 | 0 | 2 | 0 |
| non-blocking | renderdoc-rs | 1 | 0 | 1 | 0 | 0 | 0 |

## Why the rows fail

These describe this sample only.

- **E4 (68 rows) is where most rows end.** No row reached E5. The E4 failures fall
  into a few groups, often several at once:
  - **Locks held across other locks or effects.** The commonest blocking
    bug is a lock held while the same lock, another lock, a channel
    operation or I/O is reached. This is the paper's "double lock" and
    "conflicting order" pattern. In Tide v0, `Shared::with` takes a `Pure`
    closure that can neither nest nor suspend, so the source's structure
    cannot be written with safe `Shared`. That interface rules this shape
    out by construction,
    but that is not a seeded catch, and the round does not count it as one.
  - **External crates on the path.** parity-ethereum's locks are
    `parking_lot` (through `util`'s re-export, checked at each revision)
    and its I/O service runs on `mio`. Servo's messages go over `ipc-channel`. TiKV,
    rayon and others use further crates. The bans on external crates,
    network and subprocesses come from frozen bead E4; protocol §5 bars
    similar Tide substitutes, following fx-zu4f.6.
  - **Omitted by design, plus a reconstruction ban.** Condition waits use
    channels; waiting on several sources uses one enum channel. Sync once-cells
    and lazy statics give way to compile-time values or initialization in
    `main`; Arc counts and try-unwrapping are omitted because they observe
    other tasks. The design citations and protocol §5's reconstruction ban
    are distinguished below.
  - **Platform mechanisms.** The Redox and Tock kernel paths depend on
    scheduling, hardware or interrupt mechanisms outside the registered map.
  - **Unsafe sharing and memory unsafety.** Many non-blocking rows share
    memory through `unsafe`, raw pointers or `static mut`, or report memory
    unsafety. That excludes them by §5's narrowings.
  - **Old runtime internals.** Two 2013-2014 servo rows (B01, B05) are
    channel-only deadlocks between the compositor and the constellation:
    the shape Tide's channels and simulator handle. Both depend on what the
    old `std::comm` runtime did to messages queued on a dropped `Port`, and
    their triggers run through windowing and graphics crates (§4). B05's PR
    also says its fix leaves the window open.
  - **Network.** The nearest non-blocking row, N18 (a servo HTTP-cache
    assertion), needs two network fetches.
- **E2 (28 rows).** These name only a code change, a failure only as possible,
  or a bare cause label without an enumerated symptom, or quote failures
  that the evidence does not link to the row's bug. N05, N13-N14, N19,
  N25, N28, N30, N36 and N40 retain those quotes as disclosed context.
  Each E2 failure records the §2 evidence read.
- **E1 (2 rows).** B25's commit only documents a deadlock, and N34's
  accessible saved evidence names no fix.
- **E3 (2 rows).** B21's hang is a single-threaded compare-and-swap loop
  that never ends because padding bytes differ. N06 drops logs after lock
  poisoning regardless of relative thread timing.

## Which limits decided the screen

The language sources are [reference §7.4 and §11](../design/reference.md)
and the frozen [protocol §4-5](protocols/fx-zu4f.9-protocol.md). These
boundaries have different meanings:

| Boundary | Meaning and affected rows (overlapping grounds) |
| --- | --- |
| Safe-design restriction | `Shared::with` takes a `Pure` closure: it cannot nest acquisition, suspend or perform effects. `with2` acquires its set atomically and cannot preserve a source lock-order cycle. This excludes B02-B04, B06-B07, B12, B14, B16, B18, B27 and many parity lock paths. |
| Planned, unshipped interfaces | Raw `Mutex`/`RwLock` and `std::sync::atomic` are planned (§7.4); raw locks and weaker orderings need the `unsafe` grant (§11). Raw locks provide no current fallback for those guarded paths. Atomic operations may already map one-for-one to SC `Shared` steps (§5); atomic API absence alone decides no first failure. |
| Frozen experiment operation bans (bead E4) | The bead bans external-crate operations, network and subprocesses; protocol §5 additionally bars replacing an external operation with a similar Tide operation. Examples: B07-B09, B13-B14, B16, B18-B20, B23, B27, N13/N18's HTTP paths and N40's parking_lot session map. |
| Omitted by design, plus a reconstruction ban | `Condvar` is omitted: condition waits use channels ([design/03 §6.5](../design/03-authority-concurrency.md#65-sharedwith)); waiting on several sources uses one enum channel, with no `select` ([design/03, line 377](../design/03-authority-concurrency.md#12-synchronization-handles), [reference §7.3, TD0605](../design/reference.md#73-channels)). `OnceLock`/`LazyLock` (`TD0520`) and `lazy_static!` (`TD0502`) are not admitted; use a `const`/`static` or build the value in `main` ([design/03 GLOBAL-2](../design/03-authority-concurrency.md#22-no-mutable-globals-no-thread-locals), [reference §6.4](../design/reference.md#64-no-globals-deterministic-hashing) and [§6.9](../design/reference.md#69-ambient-rust-calls-and-the-admission-allowlist)). Arc counts, `get_mut` and `try_unwrap` are omitted because they observe other tasks ([reference, lines 447-449](../design/reference.md#64-no-globals-deterministic-hashing)). Protocol §5 refuses to reconstruct these operations from channels or other model code. Condvar excludes B15, B22-B23, B27, B31-B32; select B20; Arc operations B15 and N38. Once/lazy initialization is an additional ground on B25 and N30, not a separate first failure. |
| Protocol-only unsafe and symptom exclusions | Tide has an `unsafe` grant (§11), but these models do not take it. Only two explicit slot/pointer mappings are allowed. The protocol also excludes memory-unsafety symptoms even for tiny operations. Examples: N02-N03, N05, N11-N12, N14, N23, N25-N27. These are experiment restrictions, not E2 symptom rules or a promise that Tide cannot express unsafe code. |
| Protocol-only historical fidelity | Old runtime implementation details cannot be rebuilt or replaced with modern behavior (§4): B01 and B05 depend on 2013-2014 queued-message teardown. |
| Platform mechanisms outside the model | The Redox and Tock kernel paths, kernel scheduling, hardware/interrupt behavior, FFI, signals and process-exit handling have no registered mapping: B10, B13, B18-B19, B32, N21, N30, N32-N36. These are E4 grounds, including additional grounds for rows that fail earlier. |
| Protocol-only size budget | The 400-line non-test source cap decides no first failure; B08 already fails on external IPC without a path-size estimate. |

No row reaches E5. E5(a) weak memory, (b) switches inside a model step,
(c) OS/hardware exploration gaps and (d) unreportable spins therefore
decide **no first failure**. Later-criterion concerns in individual blocks
are not simulator experiments. Safe `Shared`'s prevention claim does not
extend to planned raw interfaces or unsafe-granted code.

## What this round does not show

- Tide found nothing unaided. No property was written and no seed was run,
  because no candidate reached the model stage.
- The negative says nothing about how often a seeded simulation catches real
  Rust concurrency bugs. The exclusions combine safe-design restrictions,
  deliberate omissions, planned unshipped interfaces and frozen experiment
  rules (distinguished above), rather than a model that ran and missed.
- The initial screen of B07-N41 was drafted by a screening subagent (Claude
  Opus); the initial builder screened B01-B06 and spot-checked that draft.
  Codex r1 found multiple wrong calls and a skipped required thread. The
  successive Codex authors then re-audited all 94 delegated rows themselves,
  reading every saved thread in full, fetching missing linked evidence by
  read-only GET and checking the affected first-parent paths. No subagent
  performed that re-audit. Each row's completed audit was logged separately;
  the corrected assessments and counts below replace the initial decisions.
- A failure passes E2 when the evidence ties it to the row's bug, even
  tentatively (B32, N04, N37); a failure the evidence attributes to another
  defect, a PR-only revision or new code, or that nothing in the evidence
  links to the bug, does not.
- N34 uses saved public GitLab evidence: its notes response is empty despite
  a metadata count of 18, and re-audit GETs of the numeric-project notes,
  encoded-project notes and discussions endpoints return HTTP 401. No fix
  is named in the accessible evidence; unavailable notes are not claimed
  absent, and the entire current thread was not independently reproduced.

## Authors' reproductions

The pinned artifact's `reproduced_concurrency_bugs/` directory-name
inventory maps to rows as follows. Mappings are inferred from project and
per-project numbers in column A, not verified from reproduction contents.
No reproduction was used as a fix, symptom report or property.

| Row | Directory (under `blocking-bugs/` unless stated) | Existence / read status |
| --- | --- | --- |
| B28 | `ethereum-blocking-3` | exists, not opened; inferred mapping |
| B29 | `ethereum-blocking-4` | exists, not opened; inferred mapping |
| B25 | `lazy-static-blocking-1` | exists, not opened; inferred mapping |
| B23 | `rayon-blocking-1` | exists, not opened; inferred mapping |
| B24 | `rayon-blocking-2` | exists, not opened; inferred mapping |
| B18 | `redox-blocking-1` | exists, not opened; inferred mapping |
| B16 | `tikv-blocking-3` | exists, not opened; inferred mapping |
| N06 | `non-blocking-bugs/servo-nonblocking-6` | exists, not opened; inferred mapping |
| N16 | `non-blocking-bugs/servo-nonblocking-16` | exists, not opened; inferred mapping |
| Unmapped | `deadlock-test` | exists, not opened; row mapping unresolved |

For each of the other 91 rows, the inventory names no row-matched entry;
none was opened. The unassigned `deadlock-test` is an inventory-mapping
uncertainty, not an unresolved fix lookup or an additional pool row. §6
allows reproduction contents only as trigger evidence after the row's
upstream evidence; no row reached the model stage.

## Rows

Each row's outcome is its first failing criterion. The assessments below
give the evidence, the quotes and the citations.

| Row | Column B | Column C | Project | First failure | Reason |
| --- | --- | --- | --- | --- | --- |
| B01 | `6d3429ad0372` | servo#1500 | servo | E4 | deadlock outcome depends on 2014 std::comm teardown internals (§4); windowed path via external crates |
| B02 | `ccfd977076ca` | servo#13388 | servo | E4 | same-Mutex reacquisition through an `if let` scrutinee guard (rustc 1.11 temporaries) |
| B03 | `6fbd2aa5b762` | - | servo | E4 | same-Mutex reacquisition through an `if let` scrutinee guard |
| B04 | `2952ccfae2e4` | - | servo | E4 | shared write lock held while `CSSRule::new_specific` re-acquires it |
| B05 | `183c387d8b5c` | servo#1415 | servo | E4 | trigger via windowing/gl/png external crates; deadlock depends on 2013 runtime teardown (§4); fix incomplete per PR |
| B06 | `1e380137831e` | - | servo | E4 | shared lock read taken while the parser holds it for writing |
| B07 | `702445a3ec4d` | - | servo | E4 | stderr/stdout guard held across ipc-channel send/recv (window.rs:454-464); ipc-channel external |
| B08 | `2416072dc2b2` | servo#11585 | servo | E4 | constellation blocks in Pipeline::exit on ipc-channel recv (pipeline.rs:312-327); ipc-channel external; synchronous pipeline shutdown |
| B09 | `7a72981d87c5` | servo#11129 | servo | E4 | hyper HTTP read in a spawned thread (network, external crate); the stated double lock is in lines commented out at the buggy revision |
| B10 | `2accabd554a4` | - | servo | E4 | blocks in glutin wait_events (ports/glutin/window.rs:524): external crate, OS window events |
| B11 | `13fe502cd63d` | - | servo | E2 | no PR/issue; commit message names only the code change (try_recv -> recv_opt) |
| B12 | `fdb1e511bde3` | - | servo | E4 | reentrant std Mutex in logger held across ipc-channel send (constellation.rs:345-346) |
| B13 | `8bed7713dbd1` | servo#2478 | servo-webrender | E4 | ipc-channel blocking send when socket buffer full (render_backend.rs:512-518, channel_ipc.rs:5); external, OS buffer |
| B14 | `bcb7745e56f3` | tikv#1768 | tikv | E4 | read guard in match scrutinee held across wl() of same std RwLock (pd/async/client.rs:205,207); gRPC on path |
| B15 | `6c487ed44adb` | tikv#539 | tikv | E4 | Condvar wait/notify_all (util/event.rs:74,81,109) and Arc::strong_count liveness test (l.70) |
| B16 | `237c23017d43` | tikv#2493 | tikv | E4 | match-scrutinee read guard held into reconnect() wl() of same std RwLock (pd/util.rs:288,294,167); grpcio |
| B17 | `cd330aa9c1ce` | tikv#1016 | tikv | E2 | issue/PR ask only for acquire/release symmetry; no failure stated |
| B18 | `c7eccb3a1349` | - | redox | E4 | same spin::Mutex relocked in DiskScheme::dup (drivers/ahcid/src/scheme.rs:47,51); spin external |
| B19 | `d329f7c7d254` | - | redox | E4 | kernel: files lock held across scheme dup + context::switch (process.rs:178-186, user.rs:48-55); spin external |
| B20 | `5987357a740a` | - | crossbeam | E4 | test-only fix; select! over zero-cap channels (no v0 select), channel internals use parking_lot (zero.rs:10) |
| B21 | `cfbbb6687792` | crossbeam#338 | crossbeam | E3 | single-threaded infinite CAS loop from padding-byte comparison (#388 comment), not interleaving |
| B22 | `2694e24bebed` | - | rust-threadpool | E4 | Condvar wait/notify_all in join (lib.rs:353, 150-155) |
| B23 | `36577d60a1b5` | - | rayon | E4 | LockLatch Condvar wait in in_worker_cold (registry.rs:355, latch.rs:85-88); coco::deque external |
| B24 | `f17d745e1251` | - | rayon | E2 | lost wakeup named only as possible ("could"); issue #9 reports performance only |
| B25 | `eae87a9f9736` | - | lazy-static.rs | E1 | documentation-only commit (adds a deadlock note to docs) |
| B26 | `8ed632eb961d` | parity-ethereum#592 | parity-ethereum | E2 | potential deadlock; review comment conditional ("If there is another attempt ... would be") |
| B27 | `c1aed4af45c5` | parity-ethereum#9385 | parity-ethereum | E4 | state lock held across VerificationQueue::flush Condvar wait (light_sync/mod.rs:437, queue/mod.rs:451); parking_lot |
| B28 | `f20f4c74d2ab` | parity-ethereum#9952 | parity-ethereum | E4 | lock-order inversion of two parking_lot RwLocks (blockchain.rs:1190-1191 vs 1533-1534) |
| B29 | `19a672543040` | parity-ethereum#8977 | parity-ethereum | E4 | recursive read of parking_lot RwLock best_block (blockchain.rs:1454 -> l.91/136); writer-preference dependent |
| B30 | `8b692e6d9db1` | parity-ethereum#484 | parity-ethereum | E2 | Fix potential deadlock; PR has no other text |
| B31 | `d95e9710306e` | - | parity-ethereum | E4 | Condvar wait/notify_all in IO worker shutdown (io/worker.rs:83,118); crossbeam chase_lev external |
| B32 | `a1a002f4da37` | parity-ethereum#8889 | parity-ethereum | E4 | observed panics pass E2; first-parent shutdown needs Condvar, parking_lot and process-exit behavior |
| B33 | `44e1e74bfad2` | parity-ethereum#10294 | parity-ethereum | E4 | same parking_lot Mutex relocked inside match on its guard (miner.rs:869,876) |
| B34 | `0d593199d035` | parity-ethereum#9932 | parity-ethereum | E4 | try_lock on parking_lot Mutex (light_sync/mod.rs:627); no v0 try-acquire; holder blocks on Condvar |
| B35 | `1f39a1bd768a` | parity-ethereum#8808 | parity-ethereum | E4 | last-Arc drop on IO worker joins itself (authority_round/mod.rs:701-717, service_mio.rs:463-492); OS EDEADLK, mio |
| B36 | `5bdd9a1a0e06` | parity-ethereum#8530 | parity-ethereum | E4 | read guard alive in else-arm write() of same parking_lot RwLock (ancient_import.rs:58,64) |
| B37 | `114e93ab308f` | parity-ethereum#8078 | parity-ethereum | E4 | state parking_lot Mutex held across recursive updater_step relock (updater.rs:338,347) |
| B38 | `31e64c0a5f64` | parity-ethereum#6403 | parity-ethereum | E4 | usb parking_lot Mutex relocked in read_device_info (hw/src/ledger.rs:105,137); hidapi FFI |
| B39 | `a7505be627fd` | parity-ethereum#3892 | parity-ethereum | E4 | best_seen guard held into maintain_sync relock (light_sync/mod.rs:196,202,323); util = parking_lot |
| B40 | `a33e2f2e0d26` | parity-ethereum#3426 | parity-ethereum | E4 | timers read guard alive during write() of same parking_lot RwLock (io/service.rs:252,254); mio |
| B41 | `89011dcc34e5` | parity-ethereum#3759 | parity-ethereum | E2 | fix locking patterns names only the change; Tendermint PR has no symptom |
| B42 | `73c19fd4b517` | parity-ethereum#1885 | parity-ethereum | E4 | lock-order inversion sealing_work/transaction_queue parking_lot Mutexes (miner.rs:739,743 vs 386,394) |
| B43 | `ab079fd7bf7d` | parity-ethereum#1883 | parity-ethereum | E4 | lock-order inversion blooms/cache_manager parking_lot RwLocks (trace/db.rs:297,171 vs 177,180) |
| B44 | `7c015f023fa8` | parity-ethereum#1780 | parity-ethereum | E4 | lock-order inversion traces/cache_manager parking_lot RwLocks (trace/db.rs:267,171 vs 177,179) |
| B45 | `8a8cfb133fbc` | parity-ethereum#1672 | parity-ethereum | E4 | recursive read of parking_lot RwLock info (network/host.rs:754, ~773); mio network |
| B46 | `eef9586c5769` | parity-ethereum#1569 | parity-ethereum | E4 | std Mutex guard held across update_sealing relock (miner.rs:496,502,615) |
| B47 | `5fc944ae1d2b` | parity-ethereum#985 | parity-ethereum | E4 | std Mutex guard held while trace! calls self.status() relock (miner.rs:244,250,169) |
| B48 | `5f1ce7f281b4`-0 | parity-ethereum#1360 | parity-ethereum | E4 | site -0: std RwLock info write guard held across sessions/handlers locks and callbacks (network/host.rs:441-463); mio |
| B49 | `79d996300bdf` | parity-ethereum#3702 | parity-ethereum | E4 | std Condvar + thread::park lost wakeup in verifier drop (verification/queue/mod.rs:292,309,674) |
| B50 | `16ddb8843513` | - | parity-ethereum | E4 | parking_lot sealing_work held across engine.set_signer callback that relocks it (miner.rs:770,773) |
| B51 | `18965be0473d` | parity-ethereum#3903 | parity-ethereum | E4 | Condvar lost wakeup in verification queue drop (queue/mod.rs:141, ~301, ~690) |
| B52 | `8fe73c9dd4bc` | parity-ethereum#3505 | parity-ethereum | E4 | parking_lot state guard held across fetch whose callback relocks it (updater.rs:268,278,209) |
| B53 | `63c959c4ee9a` | parity-ethereum#1584 | parity-ethereum | E4 | std RwLock sessions write guard held across handler callback that reads it (network/host.rs:892,895,217) |
| B54 | `43cc4ea13dec` | parity-ethereum#4354 | parity-ethereum | E4 | recursive read of parking_lot RwLock info in external_url (network/host.rs:44,520) |
| B55 | `f057b88f998b` | parity-ethereum#1005 | parity-ethereum | E4 | std Mutex guard held across enable_and_prepare_sealing relock (miner.rs:260,275,76) |
| B56 | `12fd8563a9d6` | parity-ethereum#10796 | parity-ethereum | E4 | parking_lot restoration relocked by abort_restore (snapshot/service.rs:719,726,856) |
| B57 | `05f9606bf2fc` | parity-ethereum#11013 | parity-ethereum | E4 | parking_lot reserved_nodes/session lock-order inversion closed by writer preference (devp2p/host.rs:576,725,730,983) |
| B58 | `b246806571d4` | parity-ethereum#8804 | parity-ethereum | E4 | engine removal; last-Arc drop joins own IoService worker, EDEADLK, mio (tendermint/mod.rs:77; transition.rs:69-71) |
| B59 | `5f1ce7f281b4`-1 | parity-ethereum#1360 | parity-ethereum | E4 | duplicate of B48 (assessed once); site -1: std RwLock reserved_nodes read held while kill_connection rereads it (network/host.rs:447,862) |
| N01 | `362c2718f714` | - | servo | E2 | race label and reproduction steps only; no enumerated wrong behavior stated |
| N02 | `e67ea42c3fa0` | servo#14485 | servo | E4 | double insertion into raw AtomicPtr free list, double free (rule_tree/mod.rs:155-160,481,525-569) |
| N03 | `d465abdb1c21` | servo#947 | servo | E4 | stale reflow-complete message; DOM then raced through *mut Node + transmute (layout_task.rs:363; node.rs:52) |
| N04 | `4f866625ccd8` | servo#8257 | servo | E4 | ipc-channel queries in constellation (constellation.rs:22,1216-1219) |
| N05 | `a13b2d447bdf` | servo#15471 | servo | E2 | PR crash belongs to a transmute experiment; no observed symptom tied to the first-parent race |
| N06 | `89e129bf9f0d` | servo#12639 | servo | E3 | stated missing logs pass E2; poison-error handling has no interleaving cause or blocking cycle |
| N07 | `b29d0c6c318a` | servo#17360 | servo | E4 | process stdout/stderr interleaving; reentrant stderr lock (console.rs:34,35,42) |
| N08 | `2a7f262b7df8` | servo#6917 | servo | E4 | process stdout interleaving read by external harness (window.rs:345) |
| N09 | `c98d35ea6b70` | servo#7027 | servo | E4 | process stderr/stdout interleaving read by external harness (window.rs:362-366) |
| N10 | `1997d034faef` | servo#10589 | servo | E2 | states the change's purpose only (prevent interleaving); no observed failure |
| N11 | `432580bd043c` | servo#13472 | servo | E4 | reentrant HandOverHandMutex with transmuted 'static guard in UnsafeCell (remutex.rs:78,95,111-114) |
| N12 | `d11716e27b53` | servo#2369 | servo | E4 | parallel layout via UnsafeLayoutNode raw pointers; borrow held across push (parallel.rs:295,305,366) |
| N13 | `8a4a5c0cb515` | - | servo | E2 | PR CI failures not linked to the body race; commit names only the code change |
| N14 | `89a29a7f12c3` | - | servo | E2 | PR regressions and memory-count defects are separate from the first-parent sharing bug |
| N15 | `d303f50784af` | - | servo | E2 | bare race message only; general source warning is not an allowed observed symptom report |
| N16 | `3c288a5b80ff` | - | servo | E2 | commit and full PR name a race only; source warning supplies no reported occurrence |
| N17 | `a31535a46e41` | servo#1097 | servo | E4 | shutdown/process-exit ordering with glfw FFI teardown (servo.rc:21,163-169; pipeline.rs:192-194) |
| N18 | `96c124230b36` | servo#21079 | servo | E4 | network fetches on the path; guards across channel sends and nested locks (http_loader.rs:1105-1148,952) |
| N19 | `a28e3606e8be` | TiKV#2279 | tikv | E2 | CI failure on the fix PR not linked to the race; bare race title only |
| N20 | `12c36194d325` | TiKV#516 | tikv | E2 | message names only the change (atomic commit); no observed failure |
| N21 | `f3a1bedd3b72` | TiKV#2848 | tikv | E4 | libc getmntent static buffer via FFI (config.rs:764-771) |
| N22 | `9d423946835b` | - | crossbeam | E2 | bare race label only; no enumerated wrong behavior stated and no linked thread |
| N23 | `a30957726d14` | crossbeam#36 | crossbeam | E4 | double free/segfault in epoch participant reclamation (participants.rs:96-110) |
| N24 | `268c02814a65` | crossbeam#29 | crossbeam | E2 | race stated only as "it's possible"; no observed failure |
| N25 | `525c22fe0975` | - | crossbeam | E2 | executed bar output belongs to PR-only Default; no observed symptom of the first-parent Rc defect |
| N26 | `eab00e91147c` | crossbeam#50 | crossbeam | E4 | epoch GC frees in-use nodes; reclamation logic, reused memory (participant.rs:102,104) |
| N27 | `6e333047cebf` | crossbeam#146 | crossbeam | E4 | use-after-free of stack Signal; park/unpark via raw pointer (ms_queue.rs:177-178,298-299) |
| N28 | `9178ce15a59f` | crossbeam#382 | crossbeam | E2 | failed/timed-out CI checks not linked to the counter race; bare race title only |
| N29 | `f8bee404e7aa` | crossbeam#345 | crossbeam | E2 | seqlock stamp wraparound only hypothetical ("would take a few lifetimes") |
| N30 | `e0e8263c25dc` | rand#48 | rand | E2 | linked std PR CI failure not tied to the row's race; wrong-value symptom conditional only |
| N31 | `bccefbc27f6d` | rand#466 | rand | E2 | message names only the change; race acknowledged, never observed |
| N32 | `2a66a84a5073` | - | redox | E2 | message names only the change; no linked evidence |
| N33 | `979d80a8c71e` | - | redox | E2 | message names only the change; no linked evidence |
| N34 | redox#303 | redox#303 | redox | E1 | issue 303 names no fix commit or merged request; MR 303 and GitHub PR 303 do not exist |
| N35 | `bf8e0e2e0ecf` | - | tock | E4 | RF233 radio hardware overwrites frame during interrupt-driven SPI read (rf233.rs) |
| N36 | `36ac574543e1` | tock#10 | tock | E2 | wrong loopback bytes belong to newly wired DMA code and remain unfixed; no repaired first-parent symptom |
| N37 | `796482c12917` | parity-ethereum#5910 | parity-ethereum | E4 | sign() between check and store takes parking_lot locks, guard across locks (signer.rs:50-51) |
| N38 | `7503d6695a8e` | parity-ethereum#440 | parity-ethereum | E4 | Arc::try_unwrap panics on live clone; Arc counts unmappable; mio (network/host.rs:638-639,661) |
| N39 | `145e766db2de` | parity-ethereum#3191 | parity-ethereum | E2 | race occurrence in tests stated, but no enumerated symptom; no PR comments or linked issue |
| N40 | `6999957869a1` | parity-ethereum#5764 | parity-ethereum | E2 | wider-PR signing failures concern other defects; duplicate-session commit says only "fixed possible race" |
| N41 | `22351fe532cf` | RUSTSEC-2019-0018 | renderdoc-rs | E2 | advisory and issues: "could lead to unexpected and unpredictable behavior"; FFI soundness |

## Assessments

One block per unique commit, in row order. Line numbers refer to the files at
the buggy revision (the fix's first parent), read by GET. Each E2 pass quotes
its source. Each E2 failure states that every item in §2 was read.

### B01 — servo 6d3429ad0372 (servo#1500)
- Repo: servo/servo; fix 6d3429ad03728028fa4059dd192456de292f939d; buggy 342844ed7b5684d67e16ca74ada6dcdb7bcf92fc; PRs: #1500 merged 2014-01-16; threads read: servo/servo#1500 (body "r? @metajack", 0 comments, 0 review comments, 0 reviews, timeline)
- E1: ✓ commit resolves; it moves the constellation's shutdown into the compositor's message loop (`ExitMsg` then a `ShutdownComplete` reply) instead of a blocking exit handshake after the loop.
- E2: ✓ "Fix crashes and deadlocks when running the content tests" (PR title, https://github.com/servo/servo/pull/1500); the commit message describes the cycle ("resulting in deadlock").
- E3: ✓ blocking cycle: the compositor has left its loop and waits for the constellation's exit reply, while the constellation, handling a pending `RendererReadyMsg`, sends `SetIds` to the compositor and waits for its reply (commit message).
- E4: ✗ the reported outcome depends on the January-2014 `std::comm` runtime's teardown internals (protocol §4 narrowing). The compositor's `Port` belongs to `NullCompositor` (src/components/main/compositing/headless.rs:12-29) or `IOCompositor`, both dropped when their `create` returns, before `CompositorTask::create` sends `ExitMsg(exit_chan)` and blocks on `exit_response_from_constellation.recv()` (src/components/main/compositing/compositor_task.rs:185-212, the send and wait at l.211-212, lines the fix removes). What then happens to a `SetIds` queued on, or sent to, a dropped `Port` decides between a deadlock and a crash: whether the queued message, and the `Chan` inside it, is dropped (so that `set_ids`'s `port.recv_opt()`, src/components/main/constellation.rs:805-815, returns `None`), or whether `send` fails the task. The PR title reports both outcomes. A model would have to choose one teardown behavior for Tide's channels, so it could not keep the source's behavior. The windowed `IOCompositor` path also runs through `azure`, `layers`, `opengles` and `png` (src/components/main/compositing/compositor.rs:19-33), which are external crates, and it already drains its port once after the loop (compositor.rs:194-196).
- E5: not assessed
- First failure: E4

### B02 — servo ccfd977076ca (servo#13388)
- Repo: servo/servo; fix ccfd977076ca09aa89de236a55071938a562176b; buggy fb52bb7c8d70f0d7b85bb35d89779d83a25f2bc2; PRs: #13388 merged 2016-09-24; threads read: servo/servo#13388 (body, 6 comments: highfive ×2, emilio r+, bors ×3; 0 review comments; 0 reviews; timeline)
- E1: ✓ commit resolves; it binds the body guard once.
- E2: ✓ (read leniently, as a reproduction of the pattern) "On playpen, similar code caused a deadlock on 1.11 stable" (commit ccfd977076ca, PR #13388 body). The same text calls the deadlock in servo itself only "possible".
- E3: ✓ a thread blocks on a `Mutex` it already holds (double lock).
- E4: ✗ same-lock reacquisition. At components/net/fetch/methods.rs:315-323, `else if let ResponseBody::Done(ref vec) = *response.body.lock().unwrap() { … } else { assert!(*response.body.lock().unwrap() == ResponseBody::Empty) }`: the scrutinee's guard lives through the `else` branch, which locks the same `Mutex` again. The deadlock also depends on rustc 1.11's temporary-scope rule (emilio, https://github.com/servo/servo/pull/13388#issuecomment-249147631: "the lifetime of the objects in if conditions was fixed not so long ago"). Additional: compiler-version semantics (§4).
- E5: not assessed
- First failure: E4

### B03 — servo 6fbd2aa5b762 (no C)
- Repo: servo/servo; fix 6fbd2aa5b7628bd47971806ddf438cd350a60bee; buggy 07c9cfecec754805cf1276995dd99a5d27cec766; PRs: #12472 merged 2016-09-22 (it carries this commit); threads read: servo/servo#12472 (body, 123 comments, review comments, reviews, timeline), #9186 (the issue #12472 fixes: body, 2 comments, timeline)
- E1: ✓ commit resolves; it binds the body guard once.
- E2: ✓ "Avoid deadlock in main_fetch" (commit 6fbd2aa5b762), not hedged.
- E3: ✓ double lock.
- E4: ✗ same-lock reacquisition. At components/net/fetch/methods.rs:364-371, `if let ResponseBody::Done(ref vec) = *response.body.lock().unwrap() { … } else { assert!(*response.body.lock().unwrap() == ResponseBody::Empty) }` relocks the `Mutex` whose guard the scrutinee still holds.
- E5: not assessed
- First failure: E4

### B04 — servo 2952ccfae2e4 (no C)
- Repo: servo/servo; fix 2952ccfae2e4883efc4886988dcf17d07d89f66c; buggy d9491187dc7509b4114bfb3d606986fac28e5ff8; PRs: #16014 merged 2017-03-19 (carries this commit); threads read: servo/servo#16014 (body, 44 comments, review comments, reviews, timeline), #15998 (linked: body, 14 comments, timeline)
- E1: ✓ commit resolves; it drops the write guard before `CSSRule::new_specific`.
- E2: ✓ "I can get Servo to hang reliably with" (SimonSapin, 2017-03-18, servo#16014 comment), and "Fix another deadlock" (commit message).
- E3: ✓ a thread blocks on a lock it holds.
- E4: ✗ same-lock reacquisition. `CSSRuleList::insert_rule` holds `parent_stylesheet.shared_lock.write()` (components/script/dom/cssrulelist.rs:93) across `insert_rule` and `CSSRule::new_specific`, which, as the fix's own comment says, "re-acquires the lock for @support and @media".
- E5: not assessed
- First failure: E4

### B05 — servo 183c387d8b5c (servo#1415)
- Repo: servo/servo; fix 183c387d8b5c9c2b19fea77517f09bf8b23f42f2; buggy c5d81f13c167ba46836b78359389e8d1a3f817d6; PRs: #1415 merged 2013-12-15; threads read: servo/servo#1415 (body, 1 comment by pcwalton, 0 review comments, 0 reviews, timeline)
- E1: ✓ commit resolves; the compositor drains its port after the loop, and `set_ids` tolerates a discarded reply.
- E2: ✓ "Drain compositor_chan on shutdown, causes deadlocks on constellation" (PR title, https://github.com/servo/servo/pull/1415); "I've only seen it with \"-o\" so far" (PR body).
- E3: ✓ blocking cycle. The master task waits on the constellation's exit reply (src/components/main/servo.rc:162-169). The constellation waits in `set_ids` for the reply to a `SetIds` sitting in the compositor's queue (constellation.rs:807-815). The compositor has left its loop (compositing/run.rs:395-424).
- E4: ✗ two reasons.
  - **The compositor's exit runs through external crates.** The loop ends (`done = true`) on window events or after writing the PNG (run.rs:321-387), through `glfw` windowing, `opengles` `gl2::read_pixels` and `png`.
  - **The deadlock depends on the December-2013 runtime's channel teardown (§4).** `compositor_task`, which owns the port, is dropped when `run` returns (servo.rc:170-180). The `set_ids` wait lasts only if the old runtime keeps the queued `SetIds` (and the `Chan` inside it) alive after that `Port` drop. Tide's channels would need a chosen behavior there.
  - Additional: the PR body says the fix is incomplete ("It is technically still possible to trigger a SetId message between when the compositor finishes running and when the constellation shuts down"). This residual race was not modeled or tested, and the record makes no prediction about a seed budget; the E4 exclusion rests on the two reasons above.
- E5: not assessed
- First failure: E4

### B06 — servo 1e380137831e (no C)
- Repo: servo/servo; fix 1e380137831eaf94a1f602c9d8dfae08f10893fa; buggy 1bacd0eb1582f609ad9a9c9a8a33a737bf4bc6f3; PRs: #16014 merged 2017-03-19; threads read: servo/servo#16014, #15998 (as B04)
- E1: ✓ commit resolves; it refactors `StylesheetLoader` so its implementations need no shared lock.
- E2: ✓ "I can get Servo to hang reliably with" (servo#16014 comment, 2017-03-18), and the commit message "This fixes a deadlock: https://github.com/servo/servo/pull/16014#issuecomment-287527067".
- E3: ✓ a thread blocks on a lock it holds.
- E4: ✗ same-lock reacquisition. `StylesheetContextSource::url` takes `document.style_shared_lock().read()` (components/script/stylesheet_loader.rs:66, in the `impl StylesheetContextSource` at l.61-76 that the fix removes). It runs from `request_stylesheet`, which the stylesheet parser calls while it holds the same shared lock for writing (components/style/stylesheets.rs, `@import` parsing).
- E5: not assessed
- First failure: E4

### B07 — servo 702445a3ec4d (no C)
- Repo: servo/servo; fix 702445a3ec4d406472a618750f5dab64c38459bb; buggy e6958d3947adedda7f3f52f26fae321f0b4d0ad4; PRs: #12563 merged 2016-07-27 (stylo restyling PR carrying this commit); threads read: servo/servo#12563 (body, 60 comments, 5 review comments, 0 reviews, timeline), servo/servo#12591 (reported in a PR comment: body, 4 comments, timeline; separate reverted panic-hook change)
- E1: ✓ commit resolves; it changes code (scopes the stdout/stderr locks in `Window::Alert`).
- E2: ✓ "Otherwise if you enable debug logging, you deadlock." (commit 702445a3ec4d406472a618750f5dab64c38459bb)
- E3: ✓ deadlock: the script thread holds the process-wide stderr lock while waiting on the constellation, which (with debug logging) blocks writing to stderr.
- E4: ✗ `Window::Alert` (components/script/dom/window.rs:449-468) holds `stderr.lock()` (l.454) and `stdout.lock()` (l.456) across `ipc::channel()` (l.461), `constellation_chan().send` (l.462) and `receiver.recv()` (l.464): a guard held across channel operations and I/O; the channel is `ipc_channel::ipc` (use at l.43), an external crate; the counterparty is std's stderr lock taken by the logger in the constellation (OS I/O).
- E5: not assessed
- First failure: E4

### B08 — servo 2416072dc2b2 (servo#11585)
- Repo: servo/servo; fix 2416072dc2b264191ab9a6515a8f9c3415849b71; buggy bdecfa13d2114281472d5df4548a8faaf8a5bd87; PRs: #11585 merged 2016-06-04; threads read: servo/servo#11585 (body, 40 comments, 0 reviews, 0 review comments, timeline), servo/servo#11546 (issue the PR fixes: body, 10 comments, timeline)
- E1: ✓ commit resolves; makes pipeline closure asynchronous (constellation.rs, pipeline.rs, script_thread.rs).
- E2: ✓ "Deadlock when removing iframe from document." (issue title, https://github.com/servo/servo/issues/11546) and "Constellation is waiting on `Pipeline::exit()`." with gdb backtraces (https://github.com/servo/servo/issues/11546#issuecomment-223075323)
- E3: ✓ blocking cycle: "in `Pipeline::exit()` the constellation is sending a `ConstellationControlMsg` to itself, then waiting to the effect of processing that message" (#issuecomment-223076707).
- E4: ✗ `Pipeline::exit` (components/constellation/pipeline.rs:306-329) blocks the constellation thread on `receiver.recv()` (l.314) of an `ipc::channel()` (l.312) and on `paint_shutdown_port.recv()` / `layout_shutdown_port.recv()` (l.326-327); the channels are `ipc_channel::ipc` (use at pipeline.rs:15, constellation.rs:24), an external crate (the issue's backtrace stops in `ipc_channel::platform::linux::recv`/`recvmsg`); the cycle runs through the constellation event loop (constellation.rs:386), the script, layout and paint threads, with external IPC operations at the cited blocking points (no line-budget exclusion is needed).
- E5: not assessed
- First failure: E4

### B09 — servo 7a72981d87c5 (servo#11129)
- Repo: servo/servo; fix 7a72981d87c54017ae2cc0bc4c0af02a7d654f1c; buggy 98863746aecea688884ca5a0e3c2f6f139f6e787; PRs: #11129 merged 2016-05-12; threads read: servo/servo#11129 (body, 8 comments, 1 review comment, 0 reviews, timeline)
- E1: ✓ commit resolves; replaces the live `read_to_end` body read with a `read_block` loop.
- E2: ✓ "I found out that it wasn't poor performance, it was actually a deadlock occuring." (https://github.com/servo/servo/pull/11129#issuecomment-218465813)
- E3: ✓ self-deadlock (double lock): `res_body` locked for the `if let` and locked again to store `Done` (PR body; review comment 62851648).
- E4: ✗ network and an external crate on the path. At the buggy revision the body is read by a spawned thread (components/net/fetch/methods.rs:850) through `res.response.read_to_end` (l.854), a `hyper` HTTP response (hyper uses at l.9-17). The double lock the PR thread describes (`let mut completed_body = res_body.lock()` with `*res_body.lock().unwrap() = …` inside, review comment 62851648) is in lines 876-879, which are commented out at the buggy revision; the live code (l.852-858) locks again only after the first guard is dropped. The stated deadlock is therefore in code the PR itself enables, and modeling it would mean modeling code the buggy revision does not compile.
- E5: not assessed
- First failure: E4

### B10 — servo 2accabd554a4 (no C)
- Repo: servo/servo; fix 2accabd554a4d34e1d5fe2cbc112c85763e7ac3e; buggy 20b961493ac73abbb5260f93f550692cb9ef8fec; PRs: none returned by commits/SHA/pulls (timeline of #3926 names PR #4461, not in the list; the commit message names #3926); threads read: servo/servo#3926 (body, 13 comments, timeline)
- E1: ✓ commit resolves; changes `Window::wait_events` to poll when writing an output file (it also re-marks a reftest flaky, but the code change is a fix).
- E2: ✓ "This stops some ref tests on linux blocking while waiting for user input events and never outputting the file." (commit 2accabd554a4d34e1d5fe2cbc112c85763e7ac3e)
- E3: ✓ (weak) a hang: the compositor's window loop blocks in a wait for windowing-system events while the output file is due; the dataset places it among blocking bugs. #3926 itself is a style-sharing-cache timing bug that the commit only marks flaky.
- E4: ✗ the blocking call is `glutin`'s `window.wait_events()` (ports/glutin/window.rs:524; `use glutin` at l.32-33), an external crate over the OS windowing system's event queue (X11); no v0 operation.
- E5: not assessed
- First failure: E4

### B11 — servo 13fe502cd63d (no C)
- Repo: servo/servo; fix 13fe502cd63d3adc19a36a34447c09fab38ae47c; buggy 1ac79c64da4b14f83fed6ca123a81417b9c0fc87; PRs: none (commits/SHA/pulls returned `[]`); threads read: none exist (the message names no issue or PR; column C empty)
- E1: ✓ commit resolves; replaces a `try_recv` busy loop with blocking `recv_opt` and stops aborting on send errors (components/net/sniffer_task.rs).
- E2: ✗ the only text is the commit message, "Don't abort the sniffer task whenever an error occurs. Also, switch to blocking recv_opt vs. non-blocking try_recv.", which names only the code change; no symptom is stated. Every listed item was read: the commit (message, files, patch), the empty PR list; there are no PRs, issues, cells or advisories naming a symptom.
- E3: not assessed
- E4: not assessed
- E5: not assessed
- First failure: E2; additional (labelled): the busy loop `loop { match self.data_receiver.try_recv() { … Err(_) => () } }` (sniffer_task.rs:35-46) would be a spin without a pending timer (E5 (d)), and the path is the 2014 `std::comm`/`proc` runtime.

### B12 — servo fdb1e511bde3 (no C; merge of #12637)
- Repo: servo/servo; fix fdb1e511bde3fa9e3b3c524a07687dc52131fa0b (merge; fix = diff against first parent); buggy (first parent) b5fe7db8b451f86777fb5c8d1f086319be8170c2; PRs: #12637 merged 2016-07-29; threads read: servo/servo#12637 (body, 16 comments, 8 review comments, 0 reviews, timeline), #12619 (issue the PR fixes: body, 4 comments, timeline), #12553 (issue the PR names: body, 5 comments, timeline)
- E1: ✓ merge resolves; replaces the logger's `Mutex` with a new same-repository `ReentrantMutex` (components/util/remutex.rs).
- E2: ✓ "ERROR:servo: called `Result::unwrap()` on an `Err` value: RecvError └ thread panicked while processing panic. aborting." (CRASH in a test run, https://github.com/servo/servo/issues/12553) and "locks aren't re-entrant, which means that if any logging generates log entries, we get a panic" (https://github.com/servo/servo/issues/12619#issuecomment-235778054). The PR links the crash to the lock only as "may be caused", so the link from the observed crash to this lock is hedged; the comment's "we get a panic" is unhedged.
- E3: ✓ (double lock) a thread re-entering the logger's lock while holding it.
- E4: ✗ reentrant locking: `FromScriptLogger::log` holds `self.constellation_chan.lock()` (components/constellation/constellation.rs:345) across `chan.send(msg)` (l.346) on an `ipc_channel::ipc::IpcSender` (use at l.24), an external crate; the re-entry is a log call made while that guard is held; std `Mutex` relocking on one thread is unspecified (deadlock or panic), an implementation behavior, not a documented one.
- E5: not assessed
- First failure: E4

### B13 — servo-webrender 8bed7713dbd1 (servo#2478)
- Repo: servo/webrender; fix 8bed7713dbd1cae1192ac10d27bcd7f787888e85; buggy d7735c7da9844775d4a3e6ec8b6cea25fb0b0486; PRs: commits/SHA/pulls returns servo/webrender#2480 (merged 2018-02-28); threads read: servo/webrender#2478 (column C issue: body, 7 comments, timeline), servo/servo#20127 (issue #2478 names: body, 19 comments, timeline), servo/webrender#2480 (fix PR)
- E1: ✓ commit resolves; buffers payloads receiver-side (render_backend.rs, channel_ipc.rs).
- E2: ✓ "Most runs deadlock, they don't panic." (https://github.com/servo/servo/issues/20127) and the gdb backtrace of "the layout thread in its stuck state" in `__libc_send` under `ipc_channel::platform::unix::OsIpcSender::send` (https://github.com/servo/servo/issues/20127#issuecomment-369027780)
- E3: ✓ blocking cycle: the transaction sender blocks sending the payload while the render backend waits on the API channel (#2478 body; #20127#issuecomment-369345332).
- E4: ✗ the blocking operation is `ipc_channel`'s byte-channel send blocking when the socket buffer is full ("IPC send is blocking if the buffer is full", #2478); the receive loop `self.payload_rx.recv_payload()` / `self.payload_tx.send_payload(data)` (webrender/src/render_backend.rs:512-518) runs over `ipc_channel::ipc::IpcBytesSender/Receiver` (webrender_api/src/channel_ipc.rs:5), an external crate whose blocking depends on the OS socketpair buffer size (also E5 (c)); the cycle spans servo's layout, font cache and webrender threads.
- E5: not assessed
- First failure: E4

### B14 — tikv bcb7745e56f3 (tikv#1768)
- Repo: tikv/tikv; fix bcb7745e56f351946902d858df593a15faa9f76c; buggy d68092a1612c4aa97b4ad1b63688d2f009dd1158; PRs: #1768 merged 2017-04-14; threads read: tikv/tikv#1768 (body, 8 comments, 2 reviews, 1 review comment, timeline)
- E1: ✓ commit resolves; binds the `try_connect_leader` result before matching (src/pd/async/client.rs), plus a test.
- E2: ✓ "Reading then writing a `RwLock` in a match block will cause a deadlock." (https://github.com/tikv/tikv/pull/1768, body, with a runnable demo); PR title "pd: fix a deadlock".
- E3: ✓ double lock (read guard alive while the same thread takes the write lock).
- E4: ✗ same-lock reacquisition: the temporary read guard `client.leader_client.inner.rl()` in the `match` scrutinee (src/pd/async/client.rs:205) lives through the arm that takes `client.leader_client.inner.wl()` (l.207) on the same `std::sync::RwLock` (src/pd/async/util.rs:15; `HandyRwLock` over std at src/util/mod.rs:22,101); the scrutinee `try_connect_leader` (client.rs:147-) connects over gRPC (`grpc`, `PDAsyncClient`, `futures`; uses at l.19-30), external crates and network.
- E5: not assessed
- First failure: E4

### B15 — tikv 6c487ed44adb (tikv#539)
- Repo: tikv/tikv; fix 6c487ed44adbf297cd054f09e2892809c8718bc6; buggy 8ec78e1898c34c5c2919f3f8acef5eaadae9b03e; PRs: #539 merged 2016-05-07; threads read: tikv/tikv#539 (body, 2 comments, 0 reviews, 0 review comments, timeline)
- E1: ✓ commit resolves; replaces the `Arc::strong_count` liveness test with an explicit `shared_cnt` (src/util/event.rs), plus a test.
- E2: ✓ "util: fix event dead lock" (PR title https://github.com/tikv/tikv/pull/539 and commit 6c487ed44adbf297cd054f09e2892809c8718bc6; unhedged, as the builder accepted "Avoid deadlock in main_fetch" for B03). The added test's comment is hedged ("This may cause dead lock if we count on arc's strong count"); a reviewer asked "why origin can cause dead lock?" with no answer. Borderline: the only unhedged text is the title.
- E3: ✓ a waiter misses its last wakeup: `Event::drop` notifies while the `Arc` is still shared by a clone, so `wait` keeps waiting (test comment in the fix).
- E4: ✗ `Condvar` wait and notify on the path: `Event::wait` blocks in `self.inner.1.wait(l)` / `wait_timeout` (src/util/event.rs:74, 81) and `Event::drop` / `set` call `notify_all` (l.109, 26); the liveness test is `Arc::strong_count(&self.inner) == 1` (l.70), which Tide's `Arc` lacks.
- E5: not assessed
- First failure: E4

### B16 — tikv 237c23017d43 (tikv#2493)
- Repo: tikv/tikv; fix 237c23017d439def50e04cdea36144e5dabb328b (cherry-pick of #2489); buggy 21e1a848789e75baf421e0636c3b92f534514294; PRs: #2493 merged 2017-11-20; threads read: tikv/tikv#2493 (body, 1 comment, 1 review, timeline), tikv/tikv#2489 (body, 4 comments, 3 reviews, 2 review comments, timeline)
- E1: ✓ commit resolves; binds the request result before `match` (src/pd/util.rs), plus tests.
- E2: ✓ "Fixed a deadlock." (https://github.com/tikv/tikv/pull/2493, body); also "Read lock won't be dropped inside of the `match` block." (https://github.com/tikv/tikv/pull/2489)
- E3: ✓ double lock: read guard held while the same thread takes the write lock.
- E4: ✗ same-lock reacquisition: in `sync_request` the scrutinee `func(&client.inner.rl().client)` (src/pd/util.rs:288) keeps the read guard alive into the `Err` arm, which calls `client.reconnect()` (l.294) taking `self.inner.wl()` (l.167) on the same `std::sync::RwLock` (l.16, 50); `func` is a gRPC call over `grpcio` (`grpc::…`, `PdClient`, l.25-30) and `reconnect` connects over the network, external crates.
- E5: not assessed
- First failure: E4

### B17 — tikv cd330aa9c1ce (tikv#1016)
- Repo: tikv/tikv; fix cd330aa9c1cea04b99d9f813e2c9e03d453ac1b8; buggy c7852e0458ec3541c776e824f95e4738bef58e48; PRs: #1016 merged 2016-09-01; threads read: tikv/tikv#1016 (empty body, 5 comments, 0 reviews, 0 review comments, timeline), tikv/tikv#1015 (issue the commit fixes: body, 1 comment, timeline)
- E1: ✓ commit resolves; adds `self.release_lock(&ctx.lock, cid)` to `Scheduler::on_read_finished`.
- E2: ✗ no wrong behavior is stated. The issue asks for symmetry: "release_lock() and acquire_lock() should always match, otherwise the code is not generic and error-prone" (#1015); in the PR a reviewer first says read commands take no latches, then "I didn't notice that we call `acquire_lock` for read only commands too. LGTM." (#1016#issuecomment-243977452). No hang, stuck command or test failure is reported. Every listed item was read: commit message and patch; PR #1016 body (empty), 5 comments, 0 reviews, 0 review comments; issue #1015 body and its 1 comment; timelines (no further links).
- E3: not assessed
- E4: not assessed
- E5: not assessed
- First failure: E2

### B18 — redox c7eccb3a1349 (no C)
- Repo: redox-os/redox (GitHub; resolved at lookup step 1's GitHub mirror, so GitLab step 2 not needed); fix c7eccb3a13493c582de8d452a1112b28224a973d; buggy b79c69cbfca48d4d1e6c3306ddcccc2122131f4d; PRs: none (pulls `[]`); threads read: none exist (message names none)
- E1: ✓ commit resolves; changes `DiskScheme::dup` to reuse the held guard.
- E2: ✓ "Fix dup deadlock, add stat" (commit c7eccb3a13493c582de8d452a1112b28224a973d; unhedged, minimal)
- E3: ✓ double lock: `dup` relocks the handle table it holds.
- E4: ✗ same-lock reacquisition on `spin::Mutex` (use at drivers/ahcid/src/scheme.rs:6), an external crate (`spin`, named in the protocol): `DiskScheme::dup` holds `self.handles.lock()` (l.47) and calls `self.handles.lock().insert(..)` (l.51); the path is the Redox AHCI driver's scheme handler (syscall interface).
- E5: not assessed
- First failure: E4

### B19 — redox d329f7c7d254 (no C)
- Repo: redox-os/redox (GitHub); fix d329f7c7d2540fb5bd73624adee6151092248dc8; buggy 791dbfa7ad80d29cec4fe22b78eae37423a05852; PRs: none (pulls `[]`); threads read: none exist
- E1: ✓ commit resolves; changes kernel `RootScheme::dup`, `UserInner::call`, and `clone`'s file duplication.
- E2: ✓ "Fix deadlock" (commit d329f7c7d2540fb5bd73624adee6151092248dc8; unhedged, minimal; the commit does not say which of its three changes)
- E3: ✓ deadlock (lock held across a blocking scheme call; dataset label supports).
- E4: ✗ kernel internals and an external lock crate: `clone` holds `context.files.lock()` (kernel/syscall/process.rs:178) across `scheme.dup(file.number)` (l.186), which for a user scheme pushes a packet and spins on `unsafe { context::switch(); }` (kernel/scheme/user.rs:48-55); the locks are `spin::RwLock`/`spin::Mutex` (kernel/scheme/root.rs:7), external; the wait is the Redox kernel scheduler (`context::switch`), an OS mechanism with no v0 operation.
- E5: not assessed
- First failure: E4

### B20 — crossbeam 5987357a740a (no C)
- Repo: crossbeam-rs/crossbeam (at this revision the tree is crossbeam-channel's, later merged into crossbeam); fix 5987357a740a5a2768c264301ba7243edb1ccb71; buggy d08c04464a50f372e2c2bf13145875359caad5bc; PRs: none (pulls `[]`); threads read: none exist
- E1: ✓ (unsure) commit resolves; it changes only test code (tests/zero.rs, tests/zero_select.rs): the per-iteration `assert!` that always fails is moved after the loop. It neither skips, deletes, retries nor only adds a test, so the narrowing's list does not exclude it, but no library code changes; the bug is in the test.
- E2: ✓ "Fix deadlocks" (commit 5987357a740a5a2768c264301ba7243edb1ccb71; unhedged, minimal)
- E3: ✓ blocking cycle: in `fairness` the main thread's assertion fails after the first `select!` (tests/zero.rs:399-405), so it panics while the spawned thread stays blocked in its `select!` receiving from `r1`/`r2` (l.389-395) on zero-capacity channels whose senders the panicking thread still owns, and `crossbeam::scope` waits for it.
- E4: ✗ the path is `select!` over `bounded(0)` channels (tests/zero.rs:384-406); Tide v0 has no `select`, so the model would have to rebuild crossbeam-channel's selection machinery (src/select.rs, src/context.rs, src/waker.rs, src/flavors/zero.rs), which uses `parking_lot::Mutex` (src/flavors/zero.rs:10; `parking_lot = "0.5"` in Cargo.toml:17), an external crate; `crossbeam::scope` is the external `crossbeam = "0.3.0"` dev-dependency at this revision (Cargo.toml:21).
- E5: not assessed
- First failure: E4

### B21 — crossbeam cfbbb6687792 (crossbeam#338; merge of #391)
- Repo: crossbeam-rs/crossbeam; fix cfbbb6687792517dd5f9c60ac98f1b13f3f49259 (merge of #391; fix = diff against first parent); buggy 32ef2fd7e2c26ee47937eea5f2697c27d3785a3b; PRs: #391 merged 2019-06-13; threads read: #391 (body, 1 comment, 0 reviews, 0 review comments, timeline), #388 (issue #391 closes: body, 3 comments, timeline), #338 (column C: an unmerged "Add a bounded SPSC queue" PR unrelated to this commit: body, 44 comments, 14 reviews, 16 review comments, timeline)
- E1: ✓ merge resolves; changes `AtomicCell::compare_exchange` to compare with `T::eq` (crossbeam-utils/src/atomic/atomic_cell.rs).
- E2: ✓ "I see no further output after `[src/main.rs:22] data_i = 150` when running on Windows." (https://github.com/crossbeam-rs/crossbeam/issues/388, body)
- E3: ✗ the stated cause is not an interleaving: "`byte_eq` includes the padding bytes in the equality, however the `ptr::read` does not copy the padding bytes. Therefore if there is junk in the padding bytes, `byte_eq` will never return true causing an infinite loop" (https://github.com/crossbeam-rs/crossbeam/issues/388#issuecomment-499308739), with a single-threaded reproduction; the hang is a retry loop that never succeeds regardless of timing (also: column C names an unrelated PR).
- E4: not assessed
- E5: not assessed
- First failure: E3; additional (labelled): E4, the wrong value comes from comparing uninitialized padding bytes through `slice::from_raw_parts` (atomic_cell.rs:640-646 `byte_eq`), memory-level behavior, and the trigger is Windows' `Instant` layout (E5 (c)).

### B22 — rust-threadpool 2694e24bebed (no C)
- Repo: rust-threadpool/rust-threadpool; fix 2694e24bebed9ee9af8d59e6c7d4d93eec1b0ff9; buggy 900bf9876f5cd8d6d2b1aab7bf7e61559b256eb9; PRs: #63 merged 2017-07-12 ("Implementation of the join operation"); threads read: rust-threadpool#63 (body, 13 comments, 5 reviews, 10 review comments, timeline), #66 (named as also fixed in #63 comment: body, 2 comments, timeline)
- E1: ✓ commit resolves; changes `ThreadPool::join`'s wait condition and `Sentinel::drop`'s notify (lib.rs).
- E2: ✓ "I found a very nasty deadlock with the `fn test_multi_join()`." (https://github.com/rust-threadpool/rust-threadpool/pull/63#issuecomment-314261569); commit "Fix nasty deadlock".
- E3: ✓ lost wakeup / blocking: joiners wait on a shared `bool` trigger that another joiner resets (`*lock = false`) and on notifications sent only when panicking (comment 314261569 names the `Condvar` and `empty_trigger` mutex).
- E4: ✗ `Condvar` wait and notify on the path: `ThreadPool::join` waits in `self.shared_data.empty_condvar.wait(lock)` (lib.rs:349-357, wait at l.353) and `no_work_notify_all` calls `self.empty_condvar.notify_all()` (l.150-155) under `empty_trigger` (l.135-136).
- E5: not assessed
- First failure: E4

### B23 — rayon 36577d60a1b5 (no C)
- Repo: rayon-rs/rayon; fix 36577d60a1b54bfa5618f1fda34a6b12a2380a10; buggy 3217551cb25c4360456bf71153a0a075b0bce70b; PRs: #449 merged 2017-10-06; threads read: rayon#449 (body, 0 comments, 1 review, 0 review comments, timeline)
- E1: ✓ commit resolves; adds `Registry::in_worker_cross` with a `TickleLatch` (rayon-core/src/registry.rs, latch.rs) and removes the doc warning.
- E2: ✓ "Solve inter-pool deadlocks" (commit/PR title, https://github.com/rayon-rs/rayon/pull/449); the removed doc at the buggy revision states the behavior unhedged: "If the other pool were to call `install()` back to the first, then they'll both be blocked." (rayon-core/src/thread_pool/mod.rs, removed lines).
- E3: ✓ blocking cycle between two pools' only worker threads.
- E4: ✗ `Registry::in_worker_cold` (rayon-core/src/registry.rs:350-357) injects the job and blocks in `job.latch.wait()` (l.355) on a `LockLatch`, whose `wait` is a `Condvar` wait (rayon-core/src/latch.rs:72, 85-88); the worker deques are `coco::deque` (registry.rs:2), an external crate; the worker loop also uses the `Sleep` module (registry.rs:10) and the thread-local `WorkerThread::current()` raw pointer.
- E5: not assessed
- First failure: E4

### B24 — rayon f17d745e1251 (no C)
- Repo: rayon-rs/rayon; fix f17d745e12511516396f6ba5027a30e88e981bfb; buggy 59f9a7111f1c6155977157c19ed0738452a2b7c7; PRs: #11 merged 2016-01-01; threads read: rayon#11 (body, 6 comments, 0 reviews, 0 review comments, timeline), rayon#9 (issue #11 fixes: body, 12 comments, timeline)
- E1: ✓ commit resolves; `LockLatch` keeps its flag under the mutex (src/latch.rs).
- E2: ✗ the race is named only as possible: "avoiding a race where the notify could come in between the probe and the call to wait" (commit message); "adjusted somewhat, primarily to address the race" (#11#issuecomment-168336687) and "The bool does need to be set under mutex, as I'll show in reply to that comment" (#11#issuecomment-168300434; the reply is not among the PR's review comments, which number 0) describe the fix, not an observed hang. Issue #9 reports only performance ("Busy yield-loops hurt performance", sched_yield time, speedup figures). Every listed item was read (commit; PR #11 body, 6 comments, 0 reviews, 0 review comments; issue #9 body and 12 comments; timelines, whose cross-reference #10 "join hangs if a worker thread panics" is a different bug); none states that the lost wakeup occurred.
- E3: not assessed
- E4: not assessed
- E5: not assessed
- First failure: E2; additional (labelled): E4, `LockLatch::wait` is a `Condvar` wait (src/latch.rs:71-75) with `set` storing an `AtomicBool` outside the mutex and calling `notify_all` (l.65-68).

### B25 — lazy-static.rs eae87a9f9736 (no C)
- Repo: rust-lang-nursery/lazy-static.rs; fix eae87a9f9736b5c35e9e03dec1a8f8c3f8d2cf36; buggy f026ac23e37fd6722b0259e8610da6c4fd275d73; PRs: none (pulls `[]`); threads read: none exist
- E1: ✗ the commit only documents the behavior: its one change adds "Note that this can lead to deadlocks if you have multiple lazy statics that depend on each other in their initialization." to the crate docs (src/lib.rs, +2 -1); no code changes (protocol §3 E1 narrowing: "only documents the behavior").
- E2: not assessed
- E3: not assessed
- E4: not assessed
- E5: not assessed
- First failure: E1; additional (labelled): E2, the text is hedged ("can lead to"); E4, the path is `lazy_static!` / `Once` initialization.

### B26 — parity-ethereum 8ed632eb961d (parity-ethereum#592; merge)
- Repo: paritytech/parity-ethereum → openethereum/parity-ethereum; fix 8ed632eb961debe5b09fe1ed38fe2ff58701e607 (merge of #592; fix = diff against first parent); buggy (first parent) 87cd8ebd261d2bafd6f46b02c947248e0ee0e9e5; PRs: #592 merged 2016-03-04; threads read: #592 (empty body, 0 comments, 0 reviews, 3 review comments, timeline)
- E1: ✓ merge resolves; binds `public_address` before the `match` (util/src/network/host.rs).
- E2: ✗ hedged only: title "Fixed potential deadlock on startup"; the review comment is conditional: "If there is another attempt to write-lock it between here and line 408 there would be a deadlock" (https://github.com/openethereum/parity-ethereum/pull/592#discussion_r55088417). Every listed item was read (merge message, PR body (empty), 0 comments, 0 reviews, 3 review comments, timeline whose only cross-reference is [#11172](https://github.com/openethereum/parity-ethereum/pull/11172), a 2019 PR by BurtonQin, the dataset's first author, that calls its own bug "similar to #592" but states no #592 symptom); none states the deadlock occurred.
- E3: not assessed
- E4: not assessed
- E5: not assessed
- First failure: E2

### B27 — parity-ethereum c1aed4af45c5 (parity-ethereum#9385)
- Repo: paritytech/parity-ethereum → openethereum/parity-ethereum; fix c1aed4af45c523dbbf84b5db8720ddad6fedcfd3; buggy c12447c7c5ae886389782929b915f83c6bda19d7; PRs: #9385 merged 2018-09-04; threads read: #9385 (body, 2 comments, 5 reviews, 5 review comments, timeline), #8918 (issue the commit names: body, 5 comments, timeline), #1 (picked up from the commit message's "#1"-style text: "Unsigned 256 arithmetic", unrelated; body, 1 comment)
- E1: ✓ commit resolves; adds a `try_lock` variant of `is_major_importing` for the informant (ethcore/sync/src/light_sync/mod.rs, parity/informant.rs).
- E2: ✓ "Sometimes, when making an RPC request, the light client doesn't answer and becomes unresponsive through the JSON-RPC interface." (https://github.com/openethereum/parity-ethereum/issues/8918) and "I could reproduce and trace. Any RPC such as getBalance don't get answered." (#8918#issuecomment-402078849)
- E3: ✓ blocking cycle across two threads (commit message: Thread1 holds `state` and waits on the queue's condvar; Thread2, the verification worker, needs `state`).
- E4: ✗ `LightSync::maintain_sync` holds `self.state.lock()` (ethcore/sync/src/light_sync/mod.rs:437) across `begin_search` → `flush_queue` (l.413-420), which blocks in `VerificationQueue::flush` on `self.empty.wait(&mut unverified)` (ethcore/src/verification/queue/mod.rs:448-451), a `Condvar` wait; locks and condvar are `parking_lot` (light_sync/mod.rs:50, queue/mod.rs:27), an external crate; Thread2's path runs through `IoChannel::send_sync` (util/io/src/service_mio.rs, mio).
- E5: not assessed
- First failure: E4

### B28 — parity-ethereum f20f4c74d2ab (parity-ethereum#9952)
- Repo: openethereum/parity-ethereum; fix f20f4c74d2aba567cc80133f222bd47c97634551; buggy 1fdfa1e6c6833ec18e171e67e5925e17a243a21b; PRs: #9952 merged 2018-11-25; threads read: #9952 (body, 0 comments, 6 reviews, 4 review comments, timeline), #8643 (PR #9952 names as the cause: body, 19 comments, 42 reviews, 71 review comments, timeline), #9385 and #9932 (PRs #9952 names: bodies, comments, reviews, review comments, timelines; #9385 is byte-identical to the evidence read for B27), #8918 (read for B27), #6350 (issue #8643 closes: body, 5 comments, timeline)
- E1: ✓ commit resolves; swaps the order of two write locks in `BlockChain::commit` (ethcore/src/blockchain/blockchain.rs) and changes the informant.
- E2: ✓ "A deadlock has been introduced in #8643:" (https://github.com/openethereum/parity-ethereum/pull/9952, body; unhedged); commit "Fix dead-lock in `blockchain.rs`". The interleaving itself is described as "it could happen that …".
- E3: ✓ lock-order inversion between `commit` and `chain_info` (PR body).
- E4: ✗ guard held across another lock's acquisition in conflicting orders: `commit` takes `self.best_ancient_block.write()` (blockchain.rs:1190) then `self.best_block.write()` (l.1191), while `chain_info` takes `self.best_block.read()` (l.1533) then `self.best_ancient_block.read()` (l.1534); the locks are `parking_lot::RwLock` (use at l.44), an external crate.
- E5: not assessed
- First failure: E4

### B29 — parity-ethereum 19a672543040 (parity-ethereum#8977)
- Repo: openethereum/parity-ethereum; fix 19a67254304075017190311ce869853ea052a166; buggy 48a54efcb23ef9c3013d8750f5fa503030514738; PRs: #8977 merged 2018-06-26; threads read: #8977 (body, 5 comments, 3 reviews, 0 review comments, timeline)
- E1: ✓ commit resolves; `chain_info` calls the helper methods before taking its read guards (ethcore/src/blockchain/blockchain.rs).
- E2: ✓ "it deadlocks randomly and only when it happens that there is a `write()` lock request in between the two reads." (https://github.com/openethereum/parity-ethereum/pull/8977#issuecomment-400358443)
- E3: ✓ recursive read lock with a queued writer in between (two threads).
- E4: ✗ reentrant locking on `parking_lot::RwLock` (use at ethcore/src/blockchain/blockchain.rs:26), an external crate: `chain_info` holds `self.best_block.read()` (l.1454) and then calls `self.first_block_number()` (l.91 → `block_number`, l.136 → `block_header_data`, l.254-265), which reads `best_block` again; the deadlock depends on parking_lot's writer-preferring fairness (the PR body cites parking_lot's RwLock docs), not std's documented behavior (also E5 (c)).
- E5: not assessed
- First failure: E4

### B30 — parity-ethereum 8b692e6d9db1 (parity-ethereum#484; merge)
- Repo: openethereum/parity-ethereum; fix 8b692e6d9db18f0f7150b8ee79b5a901039c573b (merge of #484; fix = diff against first parent); buggy (first parent) 6448d073a703466628cfeed02e65c323a00b357a; PRs: #484 merged 2016-02-21; threads read: #484 (empty body, 0 comments, 0 reviews, 0 review comments, timeline)
- E1: ✓ merge resolves; scopes the discovery lock guard before `update_nodes` (util/src/network/host.rs).
- E2: ✗ hedged only: "Fix potential deadlock on node table update" (merge message and PR title); every listed item was read (merge message and patch, PR body (empty), 0 comments, 0 reviews, 0 review comments, timeline with no other link); none states that it occurred.
- E3: not assessed
- E4: not assessed
- E5: not assessed
- First failure: E2

### B31 — parity-ethereum d95e9710306e (no C)
- Repo: openethereum/parity-ethereum; fix d95e9710306e95552a2ba3ad4df66067c77bea96; buggy 33e8d749d2da4bc56213d4348bd163bccfea3c88; PRs: #440 "UDP Discovery" merged 2016-02-19 (carries this commit); threads read: #440 (body, 6 comments, 0 reviews, 8 review comments, timeline)
- E1: ✓ commit resolves; re-orders the `deleting` check around the `Condvar` wait in the IO worker and changes orderings (util/src/io/worker.rs, ethcore/src/block_queue.rs).
- E2: ✓ (borderline) "Prevent deadlocks" (commit d95e9710306e95552a2ba3ad4df66067c77bea96; unhedged, minimal, accepted on the same footing as B03's "Avoid deadlock in main_fetch"). PR #440 says nothing about this change.
- E3: ✓ lost wakeup at shutdown: `Drop` stores `deleting` and notifies before a worker reaches its wait, so the worker waits forever and `join` blocks.
- E4: ✗ `Condvar` wait and notify on the path: `Worker::work_loop` waits in `wait.wait(lock)` (util/src/io/worker.rs:83) and `Drop` calls `self.wait.notify_all()` (l.118); the work queue is `crossbeam::sync::chase_lev` (l.21), an external crate in parity.
- E5: not assessed
- First failure: E4; additional (labelled): E5 (a) in part, the fix also changes `Relaxed` to `Acquire`/`Release` (worker.rs:80,84,117; block_queue.rs).

### B32 — parity-ethereum a1a002f4da37 (parity-ethereum#8889)
- Repo: openethereum/parity-ethereum; fix a1a002f4da3713e312f11b5aa207268af163670b (squash of #8999); buggy 5ef41ed53e159585559a87424274d9ee45f2e278; PRs: #8999 merged 2018-07-02; threads read: #8999 (body, 5 comments, 2 reviews, 2 review comments, timeline), #8889 (column C: "Fix some warns on nightly", unrelated to this commit: body, 0 comments, 2 reviews), rust-lang/rust#48919 (named by #8889: lint tracking issue, body, 22 comments)
- E1: ✓ commit resolves; replaces the aborting panic hook with a graceful-shutdown hook and an `ExitStatus` + `exiting` flag (parity/main.rs, util/panic_hook/src/lib.rs).
- E2: ✓ "Panics in our code indicates logic errors. However, it still happens." and "panic errors we occasionally see in some issue reports" (https://github.com/openethereum/parity-ethereum/pull/8999, body). "Possibly" qualifies whether aborting without Drop handlers explains those errors, not whether panics occurred. The squash's "Avoid double exit deadlock" concerns intermediate shutdown code introduced within this PR, not the first parent.
- E3: ✓ the fix addresses exit-notification ordering between callbacks and the waiting main thread. The first parent's restart callbacks update state and notify (parity/main.rs:140-142), whereas the main thread waits unconditionally (l.159-160); the fix adds a `should_exit` check before waiting. The PR explicitly describes a panic notification before the wait, but that panic callback is new; the first-parent callbacks already have the corresponding notify-before-wait shape. No first-parent double-exit deadlock or observed lost wakeup is asserted here.
- E4: ✗ the actual first-parent shutdown path waits on `exit.1`, a `Condvar`, under a `parking_lot` Mutex (parity/main.rs:135, 159-160; import l.42), invokes the `CtrlC` signal handler (l.153-156), and its panic hook aborts the process without unwinding (util/panic_hook/src/lib.rs:27-30, 69-70). Condvar, the external lock crate, signal handling and process-exit behavior are protocol §5 exclusions. The intermediate double-exit code cannot be substituted as the buggy revision.
- E5: not assessed
- First failure: E4

### B33 — parity-ethereum 44e1e74bfad2 (parity-ethereum#10294)
- Repo: openethereum/parity-ethereum; fix 44e1e74bfad2588ac1906e1171f0a3e00188f958 (a commit on the #10294 branch); buggy 8365347244f600001e4cc4f3cbc4320d59647c33 (the branch's previous commit, which introduced the code); PRs: #10294 merged 2019-02-11; threads read: #10294 (body, 3 comments, 7 reviews, 10 review comments, timeline), #9125 (feature request the PR answers: body, 10 comments, timeline)
- E1: ✓ commit resolves; binds the guarded value as `gp` instead of relocking (ethcore/src/miner/miner.rs).
- E2: ✓ "Fix deadlock in set_minimal_gas_price()" (commit 44e1e74bfad2588ac1906e1171f0a3e00188f958; unhedged, minimal)
- E3: ✓ double lock (the `match` guard is alive when the arm relocks).
- E4: ✗ same-lock reacquisition: `match *self.gas_pricer.lock()` (ethcore/src/miner/miner.rs:869) keeps the guard alive while the arm calls `self.gas_pricer.lock().recalibrate(..)` (l.876) on the same `parking_lot::Mutex` (use at l.34, field l.228), an external crate.
- E5: not assessed
- First failure: E4

### B34 — parity-ethereum 0d593199d035 (parity-ethereum#9932)
- Repo: openethereum/parity-ethereum; fix 0d593199d0352915077828abb03568a84116f289; buggy 832c4a7565cfefd75136469f8aa41a3203e6d103; PRs: #9932 merged 2018-11-26; threads read: #9932 (body, 0 comments, 6 reviews, 8 review comments, timeline), #9385 (PR #9932 names: body, 2 comments, 5 reviews, 5 review comments)
- E1: ✓ commit resolves; adds an `is_idle` flag updated with `state`, read by `is_major_importing` (ethcore/sync/src/light_sync/mod.rs).
- E2: ✓ "the default to `is_major_importing` was set to false if the lock couldn't be acquired. This lead to false logs in the informant." (https://github.com/openethereum/parity-ethereum/pull/9932, body) and "indeed it works well (at least during 4hours when the deadlock happens usually in less than one)" (review https://github.com/openethereum/parity-ethereum/pull/9932#pullrequestreview-177754967)
- E3: ✓ the informant's `try_lock` fails while the sync thread holds `state`, so the reported state depends on timing.
- E4: ✗ `is_major_importing_do_wait` uses `self.state.try_lock()` (ethcore/sync/src/light_sync/mod.rs:627) on a `parking_lot::Mutex` (use at l.50), an external crate; Tide v0's `Shared` has no try-acquire; the contending holder is `maintain_sync`'s `self.state.lock()` (l.437) held across the verification queue's `Condvar` flush (see B27).
- E5: not assessed
- First failure: E4

### B35 — parity-ethereum 1f39a1bd768a (parity-ethereum#8808)
- Repo: openethereum/parity-ethereum; fix 1f39a1bd768ae8b490068313bb41ca78bccf964b; buggy 13efb6586dde45fba7a28a95d3a15e3f3981e680; PRs: #8803 merged 2018-06-08; threads read: #8803 (body, 1 comment, 3 reviews, 2 review comments, timeline), #8088 (issue the commit closes: body, 1 comment, timeline), #8808 (column C: a "[Backports] old stable 1.9" PR, body, 1 review) and #8802 (named by #8808: body, 3 reviews)
- E1: ✓ commit resolves; `TransitionHandler` holds the step and client instead of a `Weak<AuthorityRound>` (ethcore/src/engines/authority_round/mod.rs).
- E2: ✓ "thread '<unnamed>' panicked at 'failed to join thread: Resource deadlock avoided (os error 35)'" ("When trying to stop parity instances, I often see:", https://github.com/openethereum/parity-ethereum/issues/8088)
- E3: ✓ two threads trying to join each other at shutdown (issue body: "Here are stacks of two threads that are trying to join each other").
- E4: ✗ the path is reference-count-driven destruction and OS thread joining: `TransitionHandler::timeout` upgrades `self.engine` (`Weak<AuthorityRound>`, ethcore/src/engines/authority_round/mod.rs:701, 708, 717) on an IO worker thread; when that strong reference is the last, dropping it drops `transition_service: IoService<()>` (l.387), whose shutdown joins the worker threads (util/io/src/service_mio.rs:463-465, `Drop` at l.492) from a worker itself; the stated failure is the OS `pthread_join` error EDEADLK ("os error 35"). Tide v0's `Arc` has no reference counts or last-drop destructors, and the IO service is `mio` (service_mio.rs:20-22) with `crossbeam::sync::chase_lev` (l.23), external crates.
- E5: not assessed
- First failure: E4

### B36 — parity-ethereum 5bdd9a1a0e06 (parity-ethereum#8530)
- Repo: openethereum/parity-ethereum; fix 5bdd9a1a0e06813eda559ac1fb01d7de4251faa0 (a commit of PR #8530); buggy dfac48701b7be05c0ff1c5015cd2f79b4a0b9dc4; PRs: #8530 merged 2018-05-09; threads read: #8530 (body, 0 comments, 3 reviews, 2 review comments, timeline), openethereum/parity-ethereum#8186 (the PR's "partially addresses" issue; the fetcher's GET of paritytech/parity#8186 returned 404, re-fetched by GET from openethereum/parity-ethereum: an event-loop refactoring request, body and 3 comments)
- E1: ✓ commit resolves; `AncientVerifier::verify` releases the read guard before taking the write lock; private-tx keeps a `Weak` provider (4 files).
- E2: ✓ "Get rid of deadlock and fix disconnected handler." (commit 5bdd9a1a0e06813eda559ac1fb01d7de4251faa0; unhedged); the review comment explains it: "the lock for `cur_verifier` is still held in `else` branch above, so you cannot lock it again for writing" (https://github.com/openethereum/parity-ethereum/pull/8530#discussion_r186622819).
- E3: ✓ double lock (read guard alive while the same thread write-locks).
- E4: ✗ same-lock reacquisition: `if let Some(ref cur_verifier) = *self.cur_verifier.read()` (ethcore/src/client/ancient_import.rs:58) keeps the read guard alive in the `else` arm, which calls `*self.cur_verifier.write() = ..` (l.64) and then recurses into `self.verify` (l.66); `cur_verifier` is a `parking_lot::RwLock` (use at l.27, field l.35), an external crate.
- E5: not assessed
- First failure: E4

### B37 — parity-ethereum 114e93ab308f (parity-ethereum#8078)
- Repo: openethereum/parity-ethereum; fix 114e93ab308f4b401b6e3e4a5c44920233bf9c65 (a commit of PR #8078's branch); buggy bb988246447533204d3ca337f53bfd444e1ab12b; PRs: #8078 merged 2018-04-03; threads read: #8078 (body, 9 comments, 21 reviews, 31 review comments, timeline), #7272 (issue the PR names: body, 4 comments, timeline)
- E1: ✓ commit resolves; drops the `state` guard before calling `updater_step`/`fetch` (updater/src/updater.rs).
- E2: ✓ "updater: drop state to avoid deadlocking" (commit 114e93ab308f4b401b6e3e4a5c44920233bf9c65; unhedged, minimal, as B03's "Avoid deadlock in main_fetch")
- E3: ✓ double lock: `updater_step` called while its caller holds `state`.
- E4: ✗ same-lock reacquisition: `updater_step` holds `let mut state = self.state.lock()` (updater/src/updater.rs:338) and, in the already-fetched branch, calls `self.updater_step()` recursively, which locks `self.state` again (l.338); the fetch callback holds `this.state.lock()` (l.347) and calls `this.updater_step()`; `state` is a `parking_lot::Mutex` (use at l.23, field l.145), an external crate; `fetch` downloads over the network.
- E5: not assessed
- First failure: E4

### B38 — parity-ethereum 31e64c0a5f64 (parity-ethereum#6403)
- Repo: openethereum/parity-ethereum; fix 31e64c0a5f644aa364c28db9ee3cd8ac20e430b4 (a commit of PR #6403); buggy 7e0853c211015c49938c82aa4dbc2b29a6adc41b; PRs: #6403 merged 2017-09-14 ("Trezor Support"); threads read: #6403 (body, 9 comments, 30 reviews, 71 review comments, timeline), #4500 (issue named: body, 19 comments, timeline)
- E1: ✓ commit resolves; passes the held `usb` guard into `read_device_info` instead of relocking (hw/src/ledger.rs).
- E2: ✓ "Fix deadlock for Ledger" (commit 31e64c0a5f644aa364c28db9ee3cd8ac20e430b4; unhedged, minimal)
- E3: ✓ double lock.
- E4: ✗ same-lock reacquisition: `Manager::update_devices` holds `let mut usb = self.usb.lock()` (hw/src/ledger.rs:105) and calls `self.read_device_info(&device)`, which takes `self.usb.lock()` again (l.136-137); `usb` is a `parking_lot::Mutex` (use at l.25, field l.82) over `hidapi` (l.24), external crates (USB HID through FFI).
- E5: not assessed
- First failure: E4

### B39 — parity-ethereum a7505be627fd (parity-ethereum#3892)
- Repo: openethereum/parity-ethereum; fix a7505be627fdc6ac25d32133b6e7dd2a49dd891b (a commit of PR #3892); buggy 8970946b746130aaa5d9272fafa740a93cd225d1; PRs: #3892 merged 2017-01-05 ("Naive light client synchronization"); threads read: #3892 (body, 12 comments, 2 reviews, 2 review comments, timeline)
- E1: ✓ commit resolves; scopes the `best_seen` guards (sync/src/light_sync/mod.rs) and changes a `match` in ethcore/light/src/net/mod.rs.
- E2: ✓ "fix deadlocks" (commit a7505be627fdc6ac25d32133b6e7dd2a49dd891b; unhedged, minimal)
- E3: ✓ double lock.
- E4: ✗ same-lock reacquisition: `on_connect` holds `let mut best = self.best_seen.lock()` (sync/src/light_sync/mod.rs:196) through `self.peers.write()` (l.201) and `self.maintain_sync(..)` (l.202), whose `begin_search` locks `*self.best_seen.lock()` again (l.323); `on_announcement` does the same (l.276, 278); the locks are `util::{Mutex, RwLock}` (l.38), which util re-exports from `parking_lot` (util/src/standard.rs:49 at the buggy revision), an external crate.
- E5: not assessed
- First failure: E4

### B40 — parity-ethereum a33e2f2e0d26 (parity-ethereum#3426)
- Repo: openethereum/parity-ethereum; fix a33e2f2e0d262382b6cc2bb77863c7d21cfb34b5 (a commit of PR #3426); buggy fc4664ab12e8bfcaf47450b7cd3a818361c70048; PRs: #3426 merged 2016-11-16 ("Authority round consensus engine"); threads read: #3426 (body, 5 comments, 26 reviews, 26 review comments, timeline)
- E1: ✓ commit resolves; clones the timer out of the read guard (util/io/src/service.rs).
- E2: ✓ "fix deadlock after merge" (commit a33e2f2e0d262382b6cc2bb77863c7d21cfb34b5; unhedged, minimal)
- E3: ✓ double lock (read guard alive while the same thread write-locks).
- E4: ✗ same-lock reacquisition: in `IoManager::timeout`, `if let Some(timer) = self.timers.read().get(&token.0)` (util/io/src/service.rs:252) keeps the read guard alive while `self.timers.write().remove(&token_id)` (l.254) write-locks the same `parking_lot::RwLock` (use at l.28), an external crate; the handler runs in a `mio` event loop (l.20-22).
- E5: not assessed
- First failure: E4

### B41 — parity-ethereum 89011dcc34e5 (parity-ethereum#3759)
- Repo: openethereum/parity-ethereum; fix 89011dcc34e5255efd830f808d4e6bcb5f10b3a9 (a commit of the Tendermint branch merged by #3759); buggy 2f5aeda44fb9d2b92cb41e3a668d834819366246; PRs: #3759 merged 2016-12-15 ("Tendermint Engine"); threads read: #3759 (body, 5 comments (all coveralls status), 3 reviews, 3 review comments, timeline)
- E1: ✓ commit resolves; `SignedVote::vote` takes one write guard instead of relocking `votes` (ethcore/src/engines/signed_vote.rs), plus a test.
- E2: ✗ the commit message "fix locking patterns, add simple test" names only the code change; PR #3759's body describes the Tendermint engine, its 5 comments are coveralls coverage reports, and its 3 review comments concern `message.rs`. Every listed item was read; none states a deadlock, hang or failing test.
- E3: not assessed
- E4: not assessed
- E5: not assessed
- First failure: E2; additional (labelled): E4, `if let Some(mut old) = self.votes.write().get_mut(..)` keeps the write guard alive into the `else` arm's `self.votes.write().insert(..)` (same-lock reacquisition) on util's `parking_lot::RwLock` (util/src/standard.rs:49 at the buggy revision).

### B42 — parity-ethereum 73c19fd4b517 (parity-ethereum#1885)
- Repo: openethereum/parity-ethereum; fix 73c19fd4b517dada829c31182e4fa15b872adbd0; buggy ab8b763ea5eb2f4b59b737dfb798fd101f68f10e; PRs: #1885 merged 2016-08-09; threads read: #1885 (body, 1 comment (coveralls), 0 reviews, 0 review comments, timeline), #1883 (issue it closes: body, 0 comments, timeline)
- E1: ✓ commit resolves; `update_sealing` reads the transaction queue before locking `sealing_work` (ethcore/src/miner/miner.rs).
- E2: ✓ "Deadlock in miner" (issue title, https://github.com/openethereum/parity-ethereum/issues/1883, with label F2-bug; body: "Make sure sealing_work & transaction_queue locks are acquired in correct order."). Unhedged but minimal; no observation beyond the title.
- E3: ✓ lock-order inversion between two threads.
- E4: ✗ guard held across another lock's acquisition in conflicting orders: `update_sealing` holds `self.sealing_work.lock()` (ethcore/src/miner/miner.rs:739) and takes `self.transaction_queue.lock()` (l.743), against the declared order transaction_queue → sealing_work (comment at l.168; e.g. l.386 then l.394, l.672-678); both are util's `Mutex` = `parking_lot::Mutex` (`use util::*` at l.20; util/src/standard.rs:49), an external crate.
- E5: not assessed
- First failure: E4

### B43 — parity-ethereum ab079fd7bf7d (parity-ethereum#1883; merge of #1833)
- Repo: openethereum/parity-ethereum; fix ab079fd7bf7d7f9ad04c6fe581dd680480fe5303 (merge of #1833; fix = diff against first parent); buggy (first parent) dd60146c2405fd6f786ad7d0171f70b748bf337d; PRs: #1833 merged 2016-08-04; threads read: #1833 (body "#1800", 1 comment, 0 reviews, 0 review comments, timeline), #1800 (issue the PR names: body, 8 comments with gdb backtraces, timeline), #1883 (column C: the miner deadlock issue of B42, not this commit; body, 0 comments)
- E1: ✓ merge resolves; calls `note_used` before taking `blooms.write()` (ethcore/src/trace/db.rs).
- E2: ✓ "After not getting any new blocks for a while I try to do a Ctrl + C … No luck, can't get it to exit." (https://github.com/openethereum/parity-ethereum/issues/1800) and the reporter's gdb backtrace "So, it happened again, during `parity import`" with threads in `parking_lot::rwlock::…::write` at `ethcore/src/trace/db.rs:177` and `:171` (#1800#issuecomment-237449448).
- E3: ✓ lock-order inversion between `import` and `collect_garbage`.
- E4: ✗ guard held across another lock's acquisition in conflicting orders: `TraceDB::import` holds `self.blooms.write()` (ethcore/src/trace/db.rs:297) while `note_used` takes `self.cache_manager.write()` (l.170-171); `collect_garbage` holds `self.cache_manager.write()` (l.177) and then takes `self.traces.write()` and `self.blooms.write()` (l.179-180); the locks are util's `RwLock` (l.23) = `parking_lot::RwLock` (util/src/standard.rs:49; the backtrace shows parking_lot-0.2.6), an external crate.
- E5: not assessed
- First failure: E4

### B44 — parity-ethereum 7c015f023fa8 (parity-ethereum#1780; merge)
- Repo: openethereum/parity-ethereum; fix 7c015f023fa8fa7eae9d0836595faa274b23ae5a (merge of #1780; fix = diff against first parent); buggy (first parent) dbfc5cd31c3bffeb08d65f006c24cd87b9224b5d; PRs: #1780 merged 2016-07-31; threads read: #1780 (empty body, 1 comment (coveralls), 0 reviews, 0 review comments, timeline); also, by GET, #1784 (the timeline's cross-referencing issue in which a maintainer names this commit as the fix: body and comments)
- E1: ✓ merge resolves; calls `note_used` before taking `traces.write()` (ethcore/src/trace/db.rs).
- E2: ✓ "Running the latest build, the node would eventually stop synching and just freeze up." (https://github.com/openethereum/parity-ethereum/issues/1784, reported on `v1.3.0-unstable-dbfc5cd`, the buggy revision) with the maintainer's "This should be fixed in latest master (7c015f0)" (#1784#issuecomment-236431093); title "Prevent deadlock on trace GC".
- E3: ✓ lock-order inversion between trace import and garbage collection.
- E4: ✗ guard held across another lock's acquisition in conflicting orders: `TraceDB::import` holds `self.traces.write()` (ethcore/src/trace/db.rs:267) and calls `note_used`, which takes `self.cache_manager.write()` (l.170-171); `collect_garbage` holds `self.cache_manager.write()` (l.177) and takes `self.traces.write()` (l.179); the locks are util's `RwLock` (l.23) = `parking_lot::RwLock` (util/src/standard.rs:50 at the buggy revision), an external crate.
- E5: not assessed
- First failure: E4

### B45 — parity-ethereum 8a8cfb133fbc (parity-ethereum#1672)
- Repo: openethereum/parity-ethereum; fix 8a8cfb133fbc8ed8652acd1559367c3e3cda451e; buggy 9c7395a6bef125ddd3b787e167abb131ef5ac60e; PRs: #1672 merged 2016-07-20; threads read: #1672 (empty body, 1 comment (coveralls), 0 reviews, 1 review comment, timeline)
- E1: ✓ commit resolves; binds `s.readable(..)` before matching and reads `reserved_nodes` later (util/src/network/host.rs).
- E2: ✓ "Deadlock on incoming connection (#1672)" (commit 8a8cfb133fbc8ed8652acd1559367c3e3cda451e; PR title "Fixing the deadlock on incoming connection"; unhedged); review comment "Lock was already taken at 754" (https://github.com/openethereum/parity-ethereum/pull/1672#discussion_r71478197).
- E3: ✓ recursive read lock (with a writer queued) on one thread.
- E4: ✗ reentrant locking on `parking_lot::RwLock` (use at util/src/network/host.rs:45; `info` field l.331): `match s.readable(io, &self.info.read())` (l.754) keeps the read guard alive while the arm takes `self.info.read()` again (about l.773, the review comment's line), and `self.reserved_nodes.read()` is held across it; whether it deadlocks depends on parking_lot's writer preference (writer at l.443); the session I/O is `mio` (l.28-29), network.
- E5: not assessed
- First failure: E4

### B46 — parity-ethereum eef9586c5769 (parity-ethereum#1569)
- Repo: openethereum/parity-ethereum; fix eef9586c57694142fabf79a50b953185219d42b0; buggy 57c14eedfa861cbde1926595c328c7e8916cf732; PRs: #1569 merged 2016-07-08; threads read: #1569 (empty body, 2 comments, 0 reviews, 0 review comments, timeline)
- E1: ✓ commit resolves; scopes the queue guard in `import_external_transactions` before `update_sealing` (ethcore/src/miner/miner.rs), plus comments.
- E2: ✓ "Fixing deadlock in miner (#1569)" (commit eef9586c57694142fabf79a50b953185219d42b0; unhedged); a reviewer asks for a comment "so the bug won't be (re)introduced" (#1569#issuecomment-231385229).
- E3: ✓ double lock.
- E4: ✗ same-lock reacquisition: `import_external_transactions` holds `let mut transaction_queue = self.transaction_queue.lock().unwrap()` (ethcore/src/miner/miner.rs:496) across `self.update_sealing(chain)` (l.502), which locks `self.transaction_queue.lock()` again (l.615); the lock is std's `Mutex` (util re-exports `std::sync::*`, util/src/standard.rs:39 at the buggy revision); a guard held across the same lock's acquisition cannot keep its structure in `Shared::with` (§5).
- E5: not assessed
- First failure: E4

### B47 — parity-ethereum 5fc944ae1d2b (parity-ethereum#985; merge)
- Repo: openethereum/parity-ethereum; fix 5fc944ae1d2bf24017c6976acc20a4601484280c (merge of #985; fix = diff against first parent); buggy (first parent) 5ccf653841e9600c8bb89b379d417a64a9eca32d; PRs: #985 merged 2016-04-21; threads read: #985 (empty body, 0 comments, 0 reviews, 0 review comments, timeline)
- E1: ✓ merge resolves; the trace logging reads the held guard instead of calling `self.status()` (miner/src/miner.rs).
- E2: ✓ "Fixing transaction_queue deadlock" (merge message and PR title, https://github.com/openethereum/parity-ethereum/pull/985; unhedged, minimal)
- E3: ✓ double lock.
- E4: ✗ same-lock reacquisition: `import_own_transaction` holds `self.transaction_queue.lock().unwrap()` (miner/src/miner.rs:244) and its `trace!` lines call `self.status()` (l.250, 254), which locks `self.transaction_queue` again (l.168-169), a std `Mutex` (`use util::*`, l.20; util re-exports `std::sync::*`, util/src/standard.rs:39 at the buggy revision); the trigger also needs `own_tx` trace logging enabled.
- E5: not assessed
- First failure: E4

### B48, B59 — parity-ethereum 5f1ce7f281b4 (rows 5f1ce7f…-0 and -1; parity-ethereum#1360)
- Repo: openethereum/parity-ethereum; fix 5f1ce7f281b4f4bb03995ead519d948ae64237a4 (a branch commit of #1360); buggy 495487ec071a2d62e76203645878f6a091f96f0c; PRs: #1360 merged 2016-06-21 ("Reserved peers rpc API"); threads read: #1360 (body, 4 comments, 0 reviews, 0 review comments, timeline). Rows B48 and B59 carry the same commit (suffixes -0, -1) and identical evidence directories; assessed once. The commit has two change sites in `Host::set_non_reserved_mode` (util/src/network/host.rs): site 0 adds `drop(info)`, site 1 clones `reserved_nodes` instead of holding its read guard. I take B48 (-0) as site 0 and B59 (-1) as site 1; the evidence does not say which row is which, and the outcome is the same for both.
- E1: ✓ commit resolves; it releases the `info` write guard and the `reserved_nodes` read guard before disconnecting sessions.
- E2: ✓ "fix deadlock" (commit 5f1ce7f281b4f4bb03995ead519d948ae64237a4; unhedged, minimal, the same standard as B03). #1360's body and comments describe the feature and its testing, with no symptom.
- E3: ✓ a guard held across another lock's acquisition and the same lock read twice.
- E4: ✗ for both sites. Site 0 (B48): `let mut info = self.info.write().unwrap()` (util/src/network/host.rs:441) is held across `self.sessions.write()` (l.449), each session's `e.lock()` (l.450), `s.disconnect(io, ..)` and `self.kill_connection(p, io, false)` (l.463), which takes `self.sessions.write()`, `self.handlers.read()`, `self.nodes.write()` and calls the protocol handlers' `disconnected` callback (l.836-863): guards held across other locks and across an effectful callback. Site 1 (B59): `let reserved = self.reserved_nodes.read().unwrap()` (l.447) is held while `kill_connection` takes `self.reserved_nodes.read()` again (l.862): reentrant locking on std's `RwLock` (`use std::sync::*`, l.20). The session I/O is `mio` (l.28-29).
- E5: not assessed
- First failure: E4 (both rows)

### B49 — parity-ethereum 79d996300bdf (parity-ethereum#3702; issue #3686)
- Repo: openethereum/parity-ethereum; fix 79d996300bdfd2990cb7d60c50c11667fec5fcc7 (later merged in #3702); buggy 3837114eb2002f3a7bf9abcec96f6f51c27a50c6; PRs: #3702 merged 2016-12-05 ("possible fix for queue drop deadlock"); threads read: #3686 (issue named in the message: body with gdb backtraces, 1 comment, timeline), #3702 (body, 9 comments, 1 review, 1 review comment, timeline), #1-#24 (issues the fetcher linked from `#N` text in the backtraces; all read: 2015 planning stubs, unrelated), #0 (fetch error, not an item).
- E1: ✓ commit resolves; `VerifierHandle::conclude` sets `deleting` and clears `sleep` before unparking (ethcore/src/verification/queue/mod.rs).
- E2: ✓ "Test execution halts on verification queue cleanup sometimes" (https://github.com/openethereum/parity-ethereum/issues/3686, title "Deadlock in verification queue", with a gdb backtrace). The commit message ("possible fix") hedges the fix, not the failure.
- E3: ✓ lost wakeup between `drop` and the verifier's wait.
- E4: ✗ Condvar and park: the verifier thread waits on a std `Condvar` (`use std::sync::{Condvar as SCondvar, ..}`, l.22; `more_to_verify = wait.wait(more_to_verify)`, l.309) and on `::std::thread::park()` (l.292); `drop` sets `deleting` (l.663), calls `conclude` (unpark), `notify_all`s outside the mutex (l.674) and joins the threads (l.677-679). The registered mapping permits park/unpark, but forbids this Condvar wait/notify path; no replacement waiter primitive may be reconstructed.
- E5: not assessed; additional (labelled): the fix comment "is an mfence necessary?" and #3686's comment on the atomic ordering of `Thread::unpark` would also raise E5(a).
- First failure: E4

### B50 — parity-ethereum 16ddb8843513 (parity-ethereum#3937)
- Repo: openethereum/parity-ethereum; fix 16ddb8843513c4864c3a66ba6b4e9ae0399289c6 (a branch commit of #3937); buggy 53ce07940746339a291d2b823d0bf6df0da26c10; PRs: #3937 merged 2017-01-10 ("Validator/authority contract"); threads read: #3937 (body, 10 comments (5 coveralls), 2 reviews, 2 review comments, timeline), #3932 (linked PR: body, 5 comments, timeline), ethcore/contracts#27 (linked from #3937's body; GET returned 404, no item).
- E1: ✓ commit resolves; scopes the `sealing_work` guard before `self.engine.set_signer(..)` in `set_engine_signer` (ethcore/src/miner/miner.rs).
- E2: ✓ "fix deadlock" (commit 16ddb8843513c4864c3a66ba6b4e9ae0399289c6; unhedged, minimal). The review comment at miner.rs:768 asks only for a comment on the braces.
- E3: ✓ the same thread reacquires a mutex it holds.
- E4: ✗ same-lock reacquisition through a callback: `set_engine_signer` holds `self.sealing_work.lock()` (ethcore/src/miner/miner.rs:770) across `self.engine.set_signer(address, password)` (l.773); Tendermint's `set_signer` calls `self.to_step(Step::Propose)` (ethcore/src/engines/tendermint/mod.rs:552-555), which calls `update_sealing` → the client's `update_sealing` (l.202-210, 133-135) → the miner's sealing path, which locks `sealing_work` again (e.g. miner.rs:442, 512). The mutex is util's `parking_lot::Mutex` (util/src/standard.rs:49 at the buggy revision), an external crate.
- E5: not assessed
- First failure: E4

### B51 — parity-ethereum 18965be0473d (parity-ethereum#3903; merge)
- Repo: openethereum/parity-ethereum; fix 18965be0473d19aa6f1d2a2d587deeac3337d899 (merge of #3903; fix = diff against first parent); buggy (first parent) 5ba16e48673aabbaaaee660cc3a518cdfbd58ec0; PRs: #3903 merged 2016-12-19; threads read: #3903 (body "Closes #3885", 3 comments, 3 reviews, 3 review comments, timeline), #3885 (issue it closes: empty body, 0 comments, timeline)
- E1: ✓ merge resolves; `Drop` takes `more_to_verify`'s mutex around `notify_all` (ethcore/src/verification/queue/mod.rs); the rest is whitespace.
- E2: ✓ "fix deadlock in queue drop" (merge message and PR title, https://github.com/openethereum/parity-ethereum/pull/3903; unhedged), closing issue "test_mem_limit test can deadlock" (#3885, label F4-tests).
- E3: ✓ lost wakeup: a verifier between its exit check and its Condvar wait misses `notify_all`.
- E4: ✗ Condvar: the verifier waits with `state.1.wait(&mut cur_state)` (util's parking_lot `Condvar` via `use util::*` l.23; `state: Arc<(Mutex<State>, Condvar)>` at ethcore/src/verification/queue/mod.rs:141, wait at l.301) and on the std `more_to_verify` Condvar (l.22); `Drop` notifies without holding the waiters' mutex (l.688, 691) and joins the threads (l.695). Tide v0 has no Condvar.
- E5: not assessed
- First failure: E4

### B52 — parity-ethereum 8fe73c9dd4bc (parity-ethereum#3505)
- Repo: openethereum/parity-ethereum; fix 8fe73c9dd4bc37b7577c4a8391794ec651004b4c (a branch commit of #4196); buggy 558f199dbced90c7a0c47766ce50c10fb3a879b2; PRs: #4196 merged 2017-01-17 ("Updater fixes"); threads read: #4196 (empty body, 1 comment, 1 review, 1 review comment, timeline); #3505 (column C, "Auto-updating", merged a month earlier: full body, 35 comments, 38 review comments, 38 reviews and timeline read during the r1 re-audit), #3329 (associated issue named in #3505 comment: body, no comments, timeline)
- E1: ✓ commit resolves; drops the updater state guard before starting the fetch (updater/src/updater.rs).
- E2: ✓ "Fix deadlock." (commit 8fe73c9dd4bc37b7577c4a8391794ec651004b4c; unhedged, minimal)
- E3: ✓ the fetch-completion callback relocks a held mutex.
- E4: ✗ guard held across an effectful operation and the same lock: `let mut s = self.state.lock()` (updater/src/updater.rs:268) is held across `self.weak_self.lock()` (l.276) and `self.fetcher.lock()...fetch(b, Box::new(f))` (l.278), a network download whose callback `fetch_done` locks `self.state` again (l.206-209); the mutexes are util's `parking_lot::Mutex` (l.25; util/src/standard.rs:49 at the buggy revision); the fetcher is `hash_fetch` on the `parity_reactor` event loop (l.28-31).
- E5: not assessed
- First failure: E4

### B53 — parity-ethereum 63c959c4ee9a (parity-ethereum#1584)
- Repo: openethereum/parity-ethereum; fix 63c959c4ee9aec1e8040ce3040fde9b55e388952 (a branch commit; linked PRs #1572 and #1584); buggy 03beead07269662d765aff0bb70fcd2277a664f5; PRs: #1572 merged 2016-07-11 ("Major sync <-> client interactions refactoring"), #1584 merged 2016-07-14 (column C, "Sync IPC interface"); threads read: #1572 (body, 4 comments, 14 review comments, timeline), #1584 (body, 4 comments, 2 review comments, timeline), #1586 (linked: body, 0 comments)
- E1: ✓ commit resolves; `Host::with_context` no longer holds `sessions.write()` while running the action (util/src/network/host.rs).
- E2: ✓ "fix deadlock" (commit 63c959c4ee9aec1e8040ce3040fde9b55e388952; unhedged, minimal)
- E3: ✓ the same thread reacquires an RwLock it holds for writing.
- E4: ✗ guard held across a callback that takes the same lock: `with_context` holds `let sessions = self.sessions.write().unwrap()` (util/src/network/host.rs:892) across `action(&context)` (l.895); a protocol handler's context resolves peers through `self.sessions.read()` on the same `Arc<RwLock>` (l.217), a std `RwLock` (`use std::sync::*`, l.20); the host runs on `mio` (l.28-29).
- E5: not assessed
- First failure: E4

### B54 — parity-ethereum 43cc4ea13dec (parity-ethereum#4354)
- Repo: openethereum/parity-ethereum; fix 43cc4ea13dec86cc4721f85bdbd8cc0e151bd5a2; buggy 15ffd9a09c17412a5e991823b23a9a038c6b767e; PRs: #4354 merged 2017-01-30 ("Fixed race condition deadlock on fetching enode URL"); threads read: #4354 (body "Closes #4339", 0 comments, 1 review (approval), 0 review comments, timeline), #4339 (issue it closes: body (full lldb backtrace of all threads), 1 comment, timeline)
- E1: ✓ commit resolves; `external_url` and `local_url` take `self.info.read()` once (util/network/src/host.rs).
- E2: ✓ "Deadlock when using RPCs" (issue title, https://github.com/openethereum/parity-ethereum/issues/4339, labels F0-consensus and P2-asap) with a backtrace of the hung process, threads parked in `parking_lot_core::thread_parker::…::park`.
- E3: ✓ recursive read lock with a writer queued between the two reads.
- E4: ✗ reentrant locking on `parking_lot::RwLock` (util/network/src/host.rs:44): `external_url` evaluates `self.info.read().public_endpoint.as_ref().map(|e| … self.info.read().id() …)` (l.519-520), a second read while the first read guard lives; a writer (`self.info.write()`, e.g. l.479, 585, 739) queued between them blocks the second read under parking_lot's writer preference; an external crate and a same-lock reacquisition.
- E5: not assessed
- First failure: E4

### B55 — parity-ethereum f057b88f998b (parity-ethereum#1005)
- Repo: openethereum/parity-ethereum; fix f057b88f998b968641856dca42b2c84570f0982f (a branch commit of #1005); buggy 38be2e9f40aa1ddcefb5c7b114d7e5817c4a2d67; PRs: #1005 merged 2016-04-28 ("Enable pending block when there is local transaction pending."); threads read: #1005 (body "Closes #925", 2 comments, 0 reviews, 4 review comments, timeline), #925 (issue it closes: feature request, 0 comments, timeline)
- E1: ✓ commit resolves; releases the transaction-queue guard before `enable_and_prepare_sealing` (miner/src/miner.rs).
- E2: ✓ "Fixing import transaction deadlock" (commit f057b88f998b968641856dca42b2c84570f0982f; unhedged, the standard applied to B03). The deadlock is in code the same PR introduced (the call to `enable_and_prepare_sealing`); the buggy revision, the first parent, contains it, so the protocol's buggy revision has the bug.
- E3: ✓ double lock.
- E4: ✗ same-lock reacquisition: `import_own_transaction` holds `let mut transaction_queue = self.transaction_queue.lock().unwrap()` (miner/src/miner.rs:260) across `self.enable_and_prepare_sealing(chain)` (l.275), which calls `prepare_sealing` (l.164), which locks `self.transaction_queue` again (l.76); std `Mutex` (util re-exports `std::sync::*`, util/src/standard.rs:39 at the buggy revision); a guard held across the same lock cannot keep its structure in `Shared::with` (§5).
- E5: not assessed
- First failure: E4

### B56 — parity-ethereum 12fd8563a9d6 (parity-ethereum#10796)
- Repo: openethereum/parity-ethereum; fix 12fd8563a9d6a4f8d6c8182e235e13a2ce72ee24 (a branch commit of #10796); buggy 25233d1219441b21543939fa62d00005e1c43d8d; PRs: #10796 merged 2019-07-01 ("Better logging when backfilling ancient blocks fail"); threads read: #10796 (body, 0 comments, 17 reviews, 23 review comments, timeline), #10793 (linked issue: body, 0 comments, timeline), #10792 (linked issue: logging request, 0 comments)
- E1: ✓ commit resolves; `feed_chunk` releases `restoration` before `abort_restore` and sets the status with a timed try-lock (ethcore/src/snapshot/service.rs).
- E2: ✓ "the client is stuck and Ctrl+c is not enough to shut down properly" (https://github.com/openethereum/parity-ethereum/issues/10793, label F2-bug; the title's "Probable" qualifies the diagnosis, the hang is stated as observed); review comment "parking_lot locks are not re-entrant" (#10796, service.rs:724).
- E3: ✓ the same thread reacquires a mutex it holds.
- E4: ✗ same-lock reacquisition on `parking_lot::Mutex` (ethcore/src/snapshot/service.rs:41): `feed_chunk` holds `let mut restoration = self.restoration.lock()` (l.719) and on error calls `self.abort_restore()` (l.726), which does `*self.restoration.lock() = None` (l.853-856); the fix also uses `try_lock_for`, a timed try-acquire Tide v0 lacks.
- E5: not assessed
- First failure: E4

### B57 — parity-ethereum 05f9606bf2fc (parity-ethereum#11013)
- Repo: openethereum/parity-ethereum; fix 05f9606bf2fc9fb51a7dd3033d9fc2ce55f1a2e9; buggy 00124b5a4bf8a38ad6c660b462a5e5addf13cab3; PRs: #11013 merged 2019-09-02; threads read: #11013 (body with reproduction and explanation, 2 comments, 4 reviews, 2 review comments, timeline)
- E1: ✓ commit resolves; takes `reserved_nodes.read()` before the session mutex in `session_readable` and before `nodes.write()` in `update_nodes`, and takes `nodes.write()` once in the timer (util/network-devp2p/src/host.rs).
- E2: ✓ "Deadlocks for me in less than a minute." (https://github.com/openethereum/parity-ethereum/pull/11013#issuecomment-526933476); the PR body: "deadlock will occur in 5 minutes with this docker-compose.yaml."
- E3: ✓ lock-order inversion among three threads (two orders plus a queued writer).
- E4: ✗ guards held across other locks' acquisition in conflicting orders, on `parking_lot` (`use parking_lot::{Mutex, RwLock}`, util/network-devp2p/src/host.rs:38): `connect_peers` holds `self.reserved_nodes.read()` (l.576) and takes session mutexes (l.603-604 per the PR) and `self.nodes.read()` (l.626); `session_readable` holds `session.lock()` (l.725, 744) and takes `self.reserved_nodes.read()` (l.730, 768); `update_nodes` holds `self.nodes.write()` while taking `self.reserved_nodes.read()` (l.983); the cycle closes only because a writer (`self.reserved_nodes.write()`, l.365, 407) blocks new readers, parking_lot's writer preference, an external crate's policy. Sessions run on mio.
- E5: not assessed
- First failure: E4

### B58 — parity-ethereum b246806571d4 (parity-ethereum#8804)
- Repo: openethereum/parity-ethereum; fix b246806571d4fe4378cd056858ef4b10f855e0a9; buggy 67479f1d541037752df186d20b0762e6b795a845; PRs: #9980 merged 2018-11-28 ("Remove tendermint engine support"); threads read: #9980 (body, 0 comments, 2 reviews, 0 review comments, timeline), #8804 (column C: body, 5 comments, timeline), #8088 (linked: body, 1 comment, timeline)
- E1: ✓ commit resolves; it deletes the Tendermint engine (ethcore/src/engines/tendermint/, transition.rs, vote_collector.rs, its spec and test), which removes the failing path rather than repairing it. §3's narrowing names test-only and docs-only changes, not deletion of the affected feature, so I pass E1; unsure.
- E2: ✓ "basically the same as #8088" (https://github.com/openethereum/parity-ethereum/issues/8804, "Panic (deadlock) on Parity shutdown when using Tendermint engine") with #8088's "When trying to stop parity instances, I often see: … panicked at 'failed to join thread: Resource deadlock avoided (os error 35)'" (https://github.com/openethereum/parity-ethereum/issues/8088). #8804's own text states the symptom only by reference.
- E3: ✓ which thread drops the last `Arc` depends on scheduling.
- E4: ✗ OS mechanism and drop ordering: Tendermint owns an `IoService<Step>` (ethcore/src/engines/tendermint/mod.rs:77, started at l.174) whose handler `TransitionHandler` holds a `Weak<Engine>` (ethcore/src/engines/transition.rs:36) and upgrades it on each timeout (l.69-71); when the IO worker holds the last strong reference, dropping the engine drops the `IoService`, which joins its own worker thread, and the OS returns EDEADLK ("Resource deadlock avoided"), panicking. At this first parent, `IoService::stop` sends shutdown and joins the service thread (util/io/src/service_mio.rs:457-468), and its `Drop` calls `stop` (l.491-494); mio is imported at l.20-22. The path needs an external mio operation and last-owner destructor ordering, neither of which has a registered mapping (same mechanism as B35).
- E5: not assessed
- First failure: E4

### B59 — parity-ethereum 5f1ce7f281b4 (duplicate of B48)
- Assessed once in the block "B48, B59" above (same commit 5f1ce7f281b4f4bb03995ead519d948ae64237a4, suffix -1, identical evidence). Site 1: the `reserved_nodes` read guard (util/src/network/host.rs:447) held while `kill_connection` reads the same std `RwLock` again (l.862).
- First failure: E4

### N01 — servo 362c2718f714 (no column C)
- Repo: servo/servo; fix 362c2718f7147a633eeb3b1cfd45aa13a0bf4250; buggy a76f761ed3b0f780df2b8c8aa805abb1c5247835; PRs: #1315 merged 2013-11-27; threads read: #1315 (body, 1 comment, 0 reviews, 0 review comments, timeline)
- E1: ✓ commit resolves; it disables parallel selector matching (`num_tasks = 1`, src/components/main/css/matching.rs), a workaround in production code, not a test change.
- E2: ✗ the commit and PR say only "main: Disable parallel selector matching due to races." and give steps (enable optimizations and visit the Yellow River Wikipedia page), without stating a crash, wrong output, lost data, hang or other enumerated symptom. The complete #1315 body, its one comment, reviews, review comments and timeline state no observed malfunction; a race label plus instructions is not a symptom quote (§3). The timeline cross-references later #1319 by title only, "Fix race in parallel selector matching"; that is also a cause label.
- E3-E5: not assessed. Additional E4 concern: the first-parent selector-matching tasks mutate DOM layout data through raw pointers and transmute (src/components/main/layout/util.rs:188-205), outside the two permitted unsafe mappings.
- First failure: E2

### N02 — servo e67ea42c3fa0 (servo#14485)
- Repo: servo/servo; fix e67ea42c3fa03264b8c897430f7c17a7c153e5bd (a branch commit of #14485); buggy 8412008525192455e4f3411a482c546a04bf6c72; PRs: #14485 merged 2016-12-19 ("style: Add a simple rule-tree benchmarks."); threads read: #14485 (body, 20 comments, 9 reviews, 18 review comments, timeline)
- E1: ✓ commit resolves; `StrongRuleNode::drop` and `pop_from_free_list` null the free-list pointer while modifying it (components/style/rule_tree/mod.rs); the benchmarks are additional.
- E2: ✓ "it's that we're dropping the same node twice from multiple threads … something that doesn't happen while styling but happens during these benchmarks" (https://github.com/servo/servo/pull/14485#issuecomment-267564742); the commit: "Fix rule node drop race."
- E3: ✓ check-then-act race on the free list between dropping threads.
- E4: ✗ memory-unsafety symptom and unsafe beyond the mapped forms: the same rule node is pushed onto the lock-free free list twice (a double insertion that leads to a double free via `Box::from_raw`, components/style/rule_tree/mod.rs:481, 551); the free list is raw `AtomicPtr<RuleNode>` fields (l.155-160) with `compare_and_swap` in `Drop for StrongRuleNode` (l.525-569) and `unsafe fn pop_from_free_list` (l.429-444).
- E5: not assessed
- First failure: E4

### N03 — servo d465abdb1c21 (servo#947)
- Repo: servo/servo; fix d465abdb1c2162ca9eeb72f391ea4a721332500a (a branch commit of #947); buggy 37787d55d0008879fd776b012b5d7994cff99fe9; PRs: #947 merged 2013-09-17; threads read: #947 (empty body, 1 comment "This fixes #916.", 0 reviews, 0 review comments, timeline), #916 (the issue the PR names as fixed, fetched by GET: body, 4 comments, timeline)
- E1: ✓ commit resolves; tags each reflow with an id that `handle_reflow_complete_msg` checks, and makes DOM mutation wait for layout (script_task.rs, layout_task.rs, node.rs, window.rs, document.rs).
- E2: ✓ "sometimes cause task fail … task <unnamed> failed at 'style() called on node without a style!'" (https://github.com/servo/servo/issues/916, which the PR names as fixed; the reporter attributes it to layout traversing the DOM while script appends a node); the commit: "Fix a race where a newly-initiated reflow would be considered complete when receiving the completion notice for the previous reflow."
- E3: ✓ message-ordering race between the script and layout tasks.
- E4: ✗ unsafe beyond the two mapped forms: the stale `ReflowCompleteMsg(self.id)` (src/components/main/layout/layout_task.rs:363) makes script drop `layout_join_port` (src/components/script/script_task.rs:579-583), so `join_layout` (l.219-228) skips waiting and script mutates the DOM while layout reads it; layout reaches DOM nodes as `AbstractNode { priv obj: *mut Node<View> }` (src/components/script/dom/node.rs:51-52) through `unsafe { transmute(..) }` (layout_task.rs:180-181, 323-336). The harm is unsynchronized access through raw pointers.
- E5: not assessed
- First failure: E4

### N04 — servo 4f866625ccd8 (servo#8257)
- Repo: servo/servo; fix 4f866625ccd81a682589640b360065eb43ffdf85; buggy 1dfc0481efd4687658ab34e68c883caeadd8b20f; PRs: #8257 merged 2015-10-30; threads read: #8257 (body, 23 comments, 0 reviews, 0 review comments, timeline)
- E1: ✓ commit resolves; the constellation asks layout for the webfont load state before asking script whether it is idle (components/compositing/constellation.rs).
- E2: ✓ "Attempt to fix webfont-related intermittent test failures." (commit and PR title, https://github.com/servo/servo/pull/8257): the test failures are stated as observed; the link to this race is hedged ("I'm not completely sure the race condition outlined in this patch is actually the race condition causing failures"); the author later reports "Three tries, zero font-related failures". Unsure on the link.
- E3: ✓ check order across two tasks (font loaded but not yet notified).
- E4: ✗ ipc-channel on the path: `handle_is_ready_to_save_image` queries layout over `ipc::channel()` (`use ipc_channel::ipc`, components/compositing/constellation.rs:22; l.1216-1219) and script over a std channel (l.1208-1209), each with a blocking `recv`.
- E5: not assessed
- First failure: E4

### N05 — servo a13b2d447bdf (servo#15471)
- Repo: servo/servo; fix a13b2d447bdf1e6df37dea90d6df18728f3dadf8 (a branch commit of #15458); buggy e602162a3d8af7aa8fff4fde63a1570838aab368; PRs: #15458 merged 2017-02-22 ("Update heartbeats-simple dependencies for bug fixes"); threads read: #15471 (column C and message: body, 2 comments, timeline), #15458 (body, 25 comments, 12 reviews, 14 review comments, timeline; all comments read during the r1 re-audit)
- E1: ✓ commit resolves; guards the `static mut HBS` pointer with an `AtomicBool` spinlock (components/profile/heartbeats.rs).
- E2: ✗ #15458 reports "I would end up getting an Illegal Instruction deep within the hashmap code" and supplies a crash backtrace (https://github.com/servo/servo/pull/15458#issuecomment-278367733). This is an executed transmute experiment during the PR, not an observation of the first-parent race; the author separately says "we've never had any problems that I'm aware of" with that race. The allowed thread also records a later CI failure. Nothing in the listed evidence links either failure to the row's race; every §2 item listed above was read, and none states its observed symptom.
- E3 (additional): ✓ the fix synchronizes concurrent access to the global pointer; its first-parent race is the path assessed, not the separate transmute experiment.
- E4 (additional): ✗ first-parent `static mut HBS: Option<*mut HashMap<..>>` is unsynchronized (components/profile/heartbeats.rs:17); `cleanup` takes ownership with `Box::from_raw` and frees it (l.29-39), while `maybe_heartbeat` dereferences/mutates it (l.57-66). This is excluded unsafe sharing and reclamation, with an external `heartbeats_simple` call on the path (l.7, 63). No first-parent crash or model outcome is inferred from the PR experiment.
- E5: not assessed
- First failure: E2

### N06 — servo 89e129bf9f0d (servo#12639; merge)
- Repo: servo/servo; fix 89e129bf9f0d2d388f7506854aa2eb2456cb2d4d (bors merge of #12639; fix = diff against first parent); buggy (first parent) 1f34d4f219a8d237acbf02737343ce520d73fd7a; PRs: #12639 merged 2016-07-29; threads read: #12639 (column C: body, 5 comments, 0 reviews, 0 review comments, timeline), #12638 (issue it fixes: body, 1 comment, timeline)
- E1: ✓ merge resolves; the loggers send through a poisoned `constellation_chan` lock via `into_inner` (components/constellation/constellation.rs).
- E2: ✓ "we are missing log messages if the lock is poisoned" (https://github.com/servo/servo/issues/12638). That states lost output. "this may be the source of some missing panic logs" (issuecomment-236018105) hedges the diagnosis of particular missing logs, not the stated behavior.
- E3: ✗ the report and fix identify poison-error handling, not an interleaving or blocking cycle. At the first parent, both loggers send only for `Ok` from `lock()` (components/constellation/constellation.rs:345-347, 384-386); once poisoned, each subsequent call drops the message regardless of relative thread timing. A panic followed by logging on one thread suffices. Sharing the logger does not itself establish a concurrency-shaped cause under §3.
- E4: not assessed
- E5: not assessed
- First failure: E3; additional (labelled): E4, the script logger holds a lock across an external `ipc_channel::ipc::IpcSender` send (constellation.rs:317, 345-346).

### N07 — servo b29d0c6c318a (servo#17360)
- Repo: servo/servo; fix b29d0c6c318ab3bede814a97a7dd83ea42d03c7b; buggy 66c130d55aa0d7af1104c00e93a5bf950f23a383; PRs: #17360 merged 2017-06-17; threads read: #17360 (body, 6 comments, 3 reviews, 4 review comments, timeline), #12797 (linked: body, 3 comments, timeline)
- E1: ✓ commit resolves; the Console methods hold the stderr lock while printing to stdout (components/script/dom/console.rs).
- E2: ✓ "TIMEOUT [expected OK] /html/browsers/browsing-the-web/navigating-across-documents/014.html" with the interleaved output "…parent.events is undefinedALERT: RESULT: …" (https://github.com/servo/servo/issues/12797); the PR: "This addresses a longstanding source of intermittent test failures where the stderr and stdout outputs get interleaved".
- E3: ✓ interleaving of two threads' writes to the process's output.
- E4: ✗ OS mechanism and a guard held across effects: the failure is the interleaving of the process's stdout (`println!`, components/script/dom/console.rs:34, 42) with stderr written by `env_logger` on other threads, as seen by the external test harness; the fix holds std's reentrant `stderr().lock()` across stdout writes and `send_to_devtools` (l.35), which may log to stderr again (review comment at console.rs:46: "The stderr lock is reentrant, yes."). Tide's `println!` and `stdout()` write task-owned sinks in program order and record whole records, not a process byte stream ([reference §6.5](../design/reference.md#65-write-only-sinks-are-not-authority)); the symptom is observed by an external harness parsing that stream, and protocol §5 has no sink mapping ("anything else: unportable").
- E5: not assessed
- First failure: E4

### N08 — servo 2a7f262b7df8 (servo#6917)
- Repo: servo/servo; fix 2a7f262b7df8761261a0fa618394f4e991733a5e; buggy 1809748dc12ec63e3179b66109c91983f744c235; PRs: #6917 merged 2015-08-03; threads read: #6917 (body, 7 comments, 0 reviews, 0 review comments, timeline)
- E1: ✓ commit resolves; `Window::Alert` writes through a locked, flushed stdout (components/script/dom/window.rs).
- E2: ✓ "sometimes the alert output is interleaved with other output on stdout, causing wptrunner to classify the test result as a timeout" (commit 2a7f262b7df8761261a0fa618394f4e991733a5e and PR #6917 body; the fix is hedged, "I hope", the failure is not).
- E3: ✓ interleaving of output from several threads.
- E4: ✗ OS mechanism: `println!("ALERT: {}", s)` (components/script/dom/window.rs:345) interleaves with other threads' writes to the process's stdout, buffered and flushed by std and read by an external harness; the fix takes `stdout().lock()` and flushes it. Tide's `println!` and `stdout()` write task-owned sinks in program order and record whole records, not a process byte stream ([reference §6.5](../design/reference.md#65-write-only-sinks-are-not-authority)); the symptom is observed by an external harness parsing that stream, and protocol §5 has no sink mapping ("anything else: unportable").
- E5: not assessed
- First failure: E4

### N09 — servo c98d35ea6b70 (servo#7027)
- Repo: servo/servo; fix c98d35ea6b70d341696e72f1d8aeff9dd945f423; buggy 729e003c5d954694e3fa6fd2b72702515ef94c6c; PRs: #7027 merged 2015-08-06; threads read: #7027 (body, 10 comments (bot and r+ comments included), 0 reviews, 0 review comments, timeline)
- E1: ✓ commit resolves; `Window::Alert` also holds and flushes the stderr lock (components/script/dom/window.rs).
- E2: ✓ "2a7f262b7df8761261a0fa618394f4e991733a5e was unsufficient for the case where the interleaved output was actually on stderr rather than stdout, such as output from the error macro." (commit c98d35ea6b70d341696e72f1d8aeff9dd945f423 and PR #7027 body; unhedged); "this will stabilize intermittents" (#7027 comment).
- E3: ✓ interleaving of threads' output.
- E4: ✗ OS mechanism: `Alert` locks only stdout (components/script/dom/window.rs:362-366) while other threads write to the process's stderr through the `error!` macro; the two streams interleave in the external test harness's view (the follow-up to N08). Tide's `println!` and `stdout()` write task-owned sinks in program order and record whole records, not a process byte stream ([reference §6.5](../design/reference.md#65-write-only-sinks-are-not-authority)); the symptom is observed by an external harness parsing that stream, and protocol §5 has no sink mapping ("anything else: unportable").
- E5: not assessed
- First failure: E4

### N10 — servo 1997d034faef (servo#10589)
- Repo: servo/servo; fix 1997d034faef7a09ba2d8e3c8b5cc31a1e22b1e3; buggy 4f6331953e3d6cdb40e2c96c9119a0a3298026c1; PRs: #10589 merged 2016-04-14; threads read: #10589 (body, 9 comments, 0 reviews, 0 review comments, timeline)
- E1: ✓ commit resolves; `print_buckets` writes through one stdout lock (components/profile/time.rs).
- E2: ✗ the commit and PR body state the change's purpose only: "Acquiring the stdout lock while printing the profile data prevents other messages printed to stdout from being interleaved with prints from elsewhere."; the 9 comments are review of `ignore()` versus `unwrap()` and r+; no observed failure. Every listed item was read.
- E3: not assessed
- E4: not assessed
- E5: not assessed
- First failure: E2; additional (labelled): E4, process stdout interleaving (OS mechanism), as N08.

### N11 — servo 432580bd043c (servo#13472)
- Repo: servo/servo; fix 432580bd043c402fad1a1dd428c2921b27456b05 (a branch commit of #13472); buggy 1ed3521dcfdf573dd68afe18148e785a8af2389a; PRs: #13472 merged 2016-10-04 ("Don't log in the middle of mutating a reentrant lock"); threads read: #13472 (body, 8 comments, 2 reviews, 2 review comments, timeline), #13463 (linked: body with backtrace, 4 comments, timeline)
- E1: ✓ commit resolves; `HandOverHandMutex::unlock` releases the guard before asserting ownership (components/util/remutex.rs).
- E2: ✓ "▶ CRASH [expected FAIL] /css-flexbox-1_dev/html/flexbox-basic-iframe-vert-001.htm" (https://github.com/servo/servo/issues/13463, comment by asajeffrey: "Yay, at least I managed to replicate the crash locally!"); issue body "TIMEOUT [expected FAIL]" with a backtrace; the PR states the cause unhedged: "Servo uses a reentrant lock for logging, so logging in the middle of mutating the lock is A Bad Idea." The issue's own theory was hedged ("half-baked theory").
- E3: ✓ a failing assertion logs through the reentrant logging lock while the lock's owner and guard fields are mid-update; unsure, the interleaving is with the thread's own reentry as much as with other threads.
- E4: ✗ unsafe beyond the two mapped forms and reentrant locking: `HandOverHandMutex` stores a `MutexGuard<'static, ()>` in an `UnsafeCell` via `mem::transmute(guard)` (components/util/remutex.rs:78, 95, 106) and `unlock` asserts the owner before clearing it (l.111-114); the logger it protects is reentrant; the reported symptom is a crash (`_sigtramp`).
- E5: not assessed
- First failure: E4

### N12 — servo d11716e27b53 (servo#2369)
- Repo: servo/servo; fix d11716e27b539a98bba64f25c79c06ad92a409aa; buggy 15d3257a29125103e368787f7e9790ee4314b29f; PRs: #2369 merged 2014-05-08; threads read: #2369 (body, 5 comments, 0 reviews, 0 review comments, timeline), #2308 (issue named in the message: body with reproduction, 2 comments, timeline)
- E1: ✓ commit resolves; ends the layout-data borrow before enqueueing children (src/components/main/layout/parallel.rs).
- E2: ✓ "task 'LayoutWorker' failed at 'RefCell<T> already mutably borrowed'" (https://github.com/servo/servo/issues/2308, body with reproduction steps and backtrace); a comment also reports it in the mac builder's content tests.
- E3: ✓ a child's work unit borrows the parent's layout data while the parent still holds a mutable borrow.
- E4: ✗ unsafe beyond the two mapped forms: `recalc_style_for_node` (src/components/main/layout/parallel.rs:237) holds `node.mutate_layout_data()` (l.295) while pushing child work units (l.305) onto the parallel work queue, whose units are `UnsafeLayoutNode` raw pointers transmuted to and from nodes (l.18-19, 31-33, "This code is highly unsafe", l.7); children then reach the parent's layout data through `borrow_layout_data_unchecked` (l.366).
- E5: not assessed
- First failure: E4

### N13 — servo 8a4a5c0cb515 (no column C)
- Repo: servo/servo; fix 8a4a5c0cb515242823289691a67fc553244eaa2e (a branch commit of #14465); buggy be896712ac56f69d95b31e0d5a459fbd86f71bd8; PRs: #14465 merged 2016-12-15 ("Make the fetch target non-optional."); threads read: #14465 (body (Reviewable badge only), 11 comments, 0 reviews, 0 review comments, timeline)
- E1: ✓ commit resolves; `http_network_fetch`'s worker swaps the body to `Done` under one lock instead of two (components/net/http_loader.rs).
- E2: ✗ "Test failed - linux-rel-wpt" and "1 tests failed unexpectedly" (https://github.com/servo/servo/pull/14465#issuecomment-267359671 and #issuecomment-267405245). These are observed CI failures during the wider PR, retried as known intermittents; they are not attributed to the first-parent body race. The commit's own message states only the code change. Every listed §2 item was read; none links these CI failures to the row's bug or states another observed symptom of it.
- E3 (additional): ✓ the worker empties a `ResponseBody::Receiving` vector under one lock, releases it, then marks `Done` under another; concurrent readers can see the inconsistent intermediate state (first-parent components/net/http_loader.rs:1104-1113).
- E4 (additional): ✗ the actual `http_network_fetch` worker reads a network response (`read_block(&mut res)`, l.1096), using external hyper (imports l.16-28), and sends channel notifications while holding the receiving-body guard (l.1098-1100). Network, external operations and guarded effects have no permitted mapping.
- E5: not assessed
- First failure: E2

### N14 — servo 89a29a7f12c3 (no column C)
- Repo: servo/servo; fix 89a29a7f12c3cee5af947c87602737a666919f93 (a branch commit of #13459); buggy d986fd2d2fea92c69bcf6f4f21fc0eeec7d8f4f5; PRs: #13459 merged 2016-10-04 ("Use parking_lot::RwLock for PropertyDeclarationBlock"); threads read: #13459 (body, 28 comments, 2 reviews, 2 review comments, timeline), #13176 (linked, unmerged predecessor: body, 11 comments, reviews, 4 review comments, timeline), servo/heapsize#37 (linked: body, 20 comments, timeline), #13177 (thread-safety issue named by #13176: body, 4 comments, timeline), Mozilla bug 1305141 (linked: description and all 10 comments)
- E1: ✓ commit resolves; replaces `DOMRefCell<PropertyDeclarationBlock>` with `Arc<parking_lot::RwLock<..>>` (components/script/dom/cssstyledeclaration.rs and others).
- E2: ✗ "FAIL [expected PASS] /_mozilla/css/iframe/hide_after_load.html" and `assert_equals: expected "5px" but got "20px"` (https://github.com/servo/servo/pull/13459#issuecomment-250327681). The author traces these observed PR regressions to changing copy-on-write into in-place mutation; they are not observations of the first-parent sharing defect. The heapsize discussion also states incorrect memory counts. These are separate defects, not symptoms tied to the row's sharing bug. Every listed §2 item was read; #13177 states only that `RefCell` is not `Sync`, and none states an observed symptom of this bug.
- E3 (additional): ✓ the first-parent style path shares `DOMRefCell<PropertyDeclarationBlock>` between script mutation and layout reads; #13176 describes unsynchronized animation/CSSOM access. This cause is assessed separately from the PR's cache regression.
- E4 (additional): ✗ first-parent components/style/domrefcell.rs has unsafe Send/Sync impls for a non-thread-safe RefCell (l.20-25) and `borrow_for_layout` returns `&*self.value.as_ptr()` (l.35-38), bypassing its borrow counter. This unsynchronized sharing is outside the two allowed unsafe mappings. The fix's external `parking_lot::RwLock` is an additional restriction, not a primitive present in the first parent.
- E5: not assessed
- First failure: E2

### N15 — servo d303f50784af (no column C)
- Repo: servo/servo; fix d303f50784afedb3291d41d06b0e3bf6a9be5bad; buggy 7b8a0fd2a39cfcdb52965694ae83df7731c80abf; PRs: none found (commit-pulls empty); threads read: none (no linked items)
- E1: ✓ commit resolves; layout's `match_attr` uses `get_attr_val_for_layout` and `html_element_in_html_document_for_layout` instead of borrow-flag-touching DOM methods (src/components/main/layout/wrapper.rs).
- E2: ✗ the commit message names a borrow-flag race, without an enumerated symptom. The note about spurious task failure is a general warning in source/patch text, not a statement of occurrence in a §3 evidence source. Every §2 item was read: commit, empty PR list, empty column C, no named issue.
- E3: not assessed
- E4: not assessed
- E5: not assessed
- First failure: E2; additional (labelled): the existing source uses unsafe DOM sharing beyond the two mapped forms (§5 E4); no model was attempted.

### N16 — servo 3c288a5b80ff (no column C)
- Repo: servo/servo; fix 3c288a5b80ff53061e44101a746c2ebd2a081fb0 (a branch commit of #1772); buggy 94a12469f05f8f00092a88ff1ae11fc33c086bbb; PRs: #1772 merged 2014-02-27 ("layout: Stop racing on the JSManaged borrow flags"); threads read: #1772 (body "r? @jdm", 3 comments, 0 reviews, 0 review comments, timeline)
- E1: ✓ commit resolves; layout's wrapper uses `transmute_copy` and `unsafe_get` instead of `Cast` functions and `.get()` (src/components/main/layout/wrapper.rs).
- E2: ✗ the commit and PR #1772 title name a borrow-flag race, without an enumerated symptom. The note about spurious task failure is a general warning in source/patch text, not a statement of occurrence in a §3 evidence source. Every §2 item was read: commit; PR #1772 body, all 3 comments, empty reviews/review-comments and timeline; empty column C. No permitted text states that the failure occurred.
- E3: not assessed
- E4: not assessed
- E5: not assessed
- First failure: E2; additional (labelled): the existing source uses unsafe DOM sharing beyond the two mapped forms (§5 E4); no model was attempted.

### N17 — servo a31535a46e41 (servo#1097; merge)
- Repo: servo/servo; fix a31535a46e4191544d451f53e712402802cde736 (bors merge of #1411; fix = diff against first parent); buggy (first parent) 44404766da6a551d5f24ebffa290ae9a6f5e14a5; PRs: #1411 merged 2013-12-15; threads read: #1411 (body, 8 comments, 0 reviews, 1 review comment, timeline), #1097 (column C and message: body, 9 comments, timeline)
- E1: ✓ merge resolves; `Pipeline::exit` waits until the render and layout tasks are destroyed, and the main thread waits for the constellation's exit confirmation (src/components/main/pipeline.rs, servo.rc, render_task.rs, layout_task.rs and others).
- E2: ✓ "I get the same error under Linux.  Servo will just hang after the error." (https://github.com/servo/servo/issues/1097, comment by dhedlund); the issue body: after closing the window "the following error popped up in the command line and the window stayed open".
- E3: ✓ shutdown ordering between the main thread and the render/layout tasks.
- E4: ✗ process-exit ordering and FFI: the main thread in `start` (src/components/main/servo.rc:104) sends `ExitMsg` and waits only for the constellation (l.163-169) while `Pipeline::exit` only `try_send`s exit messages (src/components/main/pipeline.rs:192-194), so the main thread can finish and call `glfw::terminate()` (glfw FFI, `extern mod glfw`, servo.rc:21) while the render task still frees GL resources; #1097's comments trace the hang to libGL during display teardown.
- E5: not assessed
- First failure: E4

### N18 — servo 96c124230b36 (servo#21079)
- Repo: servo/servo; fix 96c124230b367c0c9658e3628bcb0a33533cc96c (a branch commit of #21079); buggy 3279d1fe85d3aa8dbb99c59b4184e4bc3cc6c54b; PRs: #21079 merged 2018-06-23 ("Handle case of refreshed cache entry still receiving data"); threads read: #21079 (body, 14 comments, 1 review, 1 review comment, timeline), #20802 (linked: body with panic backtrace, 1 comment, timeline)
- E1: ✓ commit resolves; `HttpCache::refresh` keeps a done channel when the cached body is still `Receiving` (components/net/http_cache.rs).
- E2: ✓ "assertion failed: `(left == right)` left: `Receiving([...])`, right: `Empty` (thread fetch thread for https://aframe.io/a-blast/assets/models/gun.json, at components/net/fetch/methods.rs:445)" (https://github.com/servo/servo/issues/20802, title "A-blast showcase fails with an assertion").
- E3: ✓ a revalidation (304) on one fetch thread refreshes a cache entry whose body another fetch worker is still receiving.
- E4: ✗ network on the path, and guards held across channel operations and other locks: the trigger needs two HTTP fetches, one still receiving the cached body in the network worker thread (`http_network_fetch`, components/net/http_loader.rs:1032, worker spawned l.1105, reading the hyper response) and one getting a 304 from the network (l.951-953); §5 requires "no network". The worker holds `res_body.lock()` across `done_sender.send(..)` (l.1134-1136, 1140-1148), and `refresh` runs under `http_cache.write()` (l.952) while taking `cached_resource.data.metadata.headers.lock()` (http_cache.rs:642); the panic is `assert_eq!(*body, ResponseBody::Empty)` (fetch/methods.rs:445).
- E5: not assessed
- First failure: E4

### N19 — tikv a28e3606e8be (TiKV#2279)
- Repo: tikv/tikv; fix a28e3606e8beeb4f81a3f5875ece0027dc805e91; buggy 60265ecf7cdda5a5c44663da37d7afaade60c450; PRs: #2266 merged 2017-09-11 ("threadpool: fix race"); threads read: #2266 (body, 6 comments, 5 reviews, 4 review comments, timeline), #2279 (column C, the cherry-pick: body, 1 comment, 1 review, timeline)
- E1: ✓ commit resolves; moves the stop flag under the queue's mutex and notifies while holding it (src/util/threadpool.rs).
- E2: ✗ "CI failed" (https://github.com/tikv/tikv/pull/2266#issuecomment-328093077). This is the stated observed CI failure; the bare race title supplies only a cause label. The failure is on the fix PR itself, and nothing in the evidence links it to the fixed race. Every listed §2 item was read; none states an observed symptom of the row's bug.
- E3 (additional): ✓ lost wakeup between the stop flag and the Condvar wait.
- E4 (additional): ✗ Condvar: workers wait with `cvar.wait_timeout(task_queue, t)` / `cvar.wait(task_queue)` (src/util/threadpool.rs:262-270) on `Arc<(Mutex<FifoQueue<Ctx>>, Condvar)>` (l.16, 146), and `stop` sets an `AtomicBool` outside the mutex and calls `cvar.notify_all()` (l.214-216).
- E5: not assessed
- First failure: E2

### N20 — tikv 12c36194d325 (TiKV#516)
- Repo: tikv/tikv; fix 12c36194d325051c9cea13c2850bab108451f146; buggy 2c0e32645de2add50ddfffd8460b810193ebec90; PRs: #516 merged 2016-05-03; threads read: #516 (body (reviewer mentions only), 3 comments, 0 reviews, 2 review comments, timeline)
- E1: ✓ commit resolves; takes `self.storage.wl()` once, before the engine write, and updates the storage fields under it (src/raftstore/store/peer.rs).
- E2: ✗ the message names only the code change ("store: atomically committing and changing storage."); the code comment added by the fix states the intent ("Lock here to guarantee generating snapshot sees a consistent view data."); the comments are LGTMs and the review comments discuss whether the batch writes must be atomic. No observed failure. Every listed item was read.
- E3: not assessed
- E4: not assessed
- E5: not assessed
- First failure: E2; additional (labelled): E4, the path writes through rocksdb (`use rocksdb::{DB, WriteBatch, Writable}`, src/raftstore/store/peer.rs:19; `match self.engine.write(wb)`, l.630) and the fix holds the storage write guard across that write.

### N21 — tikv f3a1bedd3b72 (TiKV#2848)
- Repo: tikv/tikv; fix f3a1bedd3b726e6af42e05d4a4cc3d15b739251f; buggy 8dcc5ab06930f0c5ad8dd078a5118c03412b352d; PRs: #2848 merged 2018-03-21 ("add lock on `get_fs_info`."); threads read: #2848 (body, 0 comments, 5 reviews, 3 review comments, timeline)
- E1: ✓ commit resolves; serializes `get_fs_info` with a `lazy_static` mutex (src/util/config.rs).
- E2: ✓ "So we use a lock to protect it in order to avoid `make dev` fail." (commit and PR #2848 body; unhedged). Borderline: the failing `make dev` is implied, not described.
- E3: ✓ concurrent calls overwrite `getmntent`'s static buffer.
- E4: ✗ OS mechanism and FFI: `get_fs_info` calls `libc::setmntent` and `libc::getmntent` (src/util/config.rs:764-771), whose result "points to a static area of memory which is overwritten by subsequent calls" (`man 3 getmntent`, quoted in the fix); the fix adds a `lazy_static!` lock.
- E5: not assessed
- First failure: E4

### N22 — crossbeam 9d423946835b (no column C)
- Repo: crossbeam-rs/crossbeam; fix 9d423946835b69edb395a5654b85649161f34d3b; buggy 4ed4267ed9d2960dc582e41ca33679a78488c663; PRs: none (commit-pulls empty); threads read: none (no linked items)
- E1: ✓ commit resolves; `start_send` advances `tail.index` with `fetch_add` instead of a `store` that could overwrite the disconnect mark (crossbeam-channel/src/flavors/list.rs).
- E2: ✗ the only message is "channel: fix a race in disconnection" (commit 9d423946835b69edb395a5654b85649161f34d3b). It labels a cause without stating a hang, lost item, wrong result, panic or other enumerated symptom. Every §2 item was read: commit message, files and patch, empty commits/SHA/pulls response, empty column C; no linked issue or PR is named. Unlike a deadlock, a bare race label does not satisfy §3's symptom rule.
- E3: not assessed
- E4: not assessed
- E5: not assessed
- First failure: E2; additional (labelled): E4, block allocation and reclamation in `start_send` / `Block::destroy` (crossbeam-channel/src/flavors/list.rs:92-108, 223, 249, 387-392) would exclude the path even if a symptom were stated.

### N23 — crossbeam a30957726d14 (crossbeam#36)
- Repo: crossbeam-rs/crossbeam; fix a30957726d1437326c60e6dd7e2a303ae255b591 (a branch commit of #36); buggy 234aaa2d146513aa6738e3627bb450464c0420f4; PRs: #36 merged 2016-01-28 ("Fix bugs preventing memory reclamation"); threads read: #36 (body, 4 comments, 0 reviews, 0 review comments, timeline)
- E1: ✓ commit resolves; adds an `unlinked` flag CASed before a participant is passed to `unlinked()` (src/mem/epoch/participant.rs, participants.rs).
- E2: ✓ "this appears to be failing some CI tests with a segfault" (https://github.com/crossbeam-rs/crossbeam/pull/36, comment by aturon) and "I think I've tracked down this crash, which also happens to be in lazy Participant reclamation code" (msullivan); the commit names the bug "a double-free bug".
- E3: ✓ two threads both unlink the same participant node.
- E4: ✗ memory-unsafety symptom (double free, segfault) and reclamation logic: `Iter::next` unlinks an inactive participant with `self.next.cas_shared(Some(n), cur, Relaxed)` and hands it to `self.guard.unlinked(n)` (src/mem/epoch/participants.rs:96-110), epoch garbage collection that frees it later.
- E5: not assessed
- First failure: E4

### N24 — crossbeam 268c02814a65 (crossbeam#29)
- Repo: crossbeam-rs/crossbeam; fix 268c02814a6589577dffada83174532f6111c352 (a branch commit of #29); buggy c30f3b129e950815836725c4df148cbfd91fac83; PRs: #29 merged 2016-01-26 ("Fixing race condition in garbage collection"); threads read: #29 (body, 5 comments, 0 reviews, 0 review comments, timeline)
- E1: ✓ commit resolves; `ConcBag::collect` takes the list with `swap` instead of `load` then `store(null)` (src/mem/epoch/garbage.rs).
- E2: ✗ the race is stated only as possible: "The collect function in ConcBag is unsound - it's possible for a thread inserting into the linked list at the same time a thread enters collect to make an insertion that isn't visible." (PR #29 body); the maintainer first answers that "There are no threads racing" because of an external invariant, then agrees the migration scheme breaks it ("you appear to be right"); no comment reports an observed failure. Every listed item was read.
- E3: not assessed
- E4: not assessed
- E5: not assessed
- First failure: E2; additional (labelled): E4, epoch garbage reclamation with `Box::from_raw` on the path (src/mem/epoch/garbage.rs, `collect`); E5(a), the fix also changes `Relaxed` to `Acquire`.

### N25 — crossbeam 525c22fe0975 (no column C; message "(#42)")
- Repo: crossbeam-rs/crossbeam (history imported from crossbeam-rs/crossbeam-utils); fix 525c22fe097543c5db6051e2e2cb0338017e7b58; buggy 49561df24f7cd5d1657d4cd059aecb4bffbaedbe; PRs: crossbeam-utils#42 merged 2018-09-20 ("code simplification of thread joining + code rearrangements"); threads read: crossbeam-rs/crossbeam#42 (the item the fetcher resolved from "(#42)": a 2016 PR "Fix multiple segfault-causing bugs", body, 16 comments, 1 review comment; a different PR in the same-numbered slot of the other repository, unrelated to this commit), and, by GET, crossbeam-rs/crossbeam-utils#42 (the PR the message names: body, 11 comments, review comments)
- E1: ✓ commit resolves; rewrites `scope` and joining, and replaces the handle's `Rc<RefCell<_>>` with `Arc<Mutex<_>>` (src/thread.rs).
- E2: ✗ "This prints `"bar"` on my machine, which indicates UB." (https://github.com/crossbeam-rs/crossbeam-utils/pull/42#issuecomment-422051850); the author quotes and accepts it in issuecomment-422174035. This is an executed reproduction, not a hypothetical program. It uses the `Scope::default()` constructor introduced and removed during the PR, absent at the frozen first parent; it is not evidence that the first parent's Rc defect printed `"bar"`. The correct thread, all 11 issue comments, all review comments/reviews and timeline were read and saved. The unrelated 2016 crossbeam#42 supplies no symptom for this fix. Every listed §2 item was read; none states an observed symptom of the first-parent Rc/RefCell defect.
- E3 (additional): ✓ a separate concurrency-shaped defect exists at the first parent: `ScopedJoinHandle` is unsafely `Send` and `Sync` while containing `Rc<RefCell<Option<JoinState<T>>>>` (src/thread.rs:195-203). `spawn` clones this Rc into the scope's deferred-join list (l.347-352), while `join` takes its shared state through `borrow_mut` (l.379-381). The correct PR's review requests replacing Rc/RefCell with Arc/Mutex (discussion_r218100259, pullrequestreview-155963898); the author's reply says the non-Default issues predated the PR (issuecomment-422174035).
- E4 (additional): ✗ first-parent shared Rc reference counting and RefCell access rely on unsound `Send`/`Sync` implementations (src/thread.rs:195-203, 347-352, 379-381), unsafe beyond the two mapped forms in §5. Replacing these operations with `Shared` would add synchronization absent in the source. The executed Default-constructor use-after-lifetime defect is distinct and cannot be modeled at this revision; its memory-unsafety symptom would independently be excluded at E4. No different buggy revision is substituted.
- E5: not assessed
- First failure: E2

### N26 — crossbeam eab00e91147c (crossbeam#50; merge)
- Repo: crossbeam-rs/crossbeam; fix eab00e91147c93ca4646c1ba06f2f16054b30e77 (merge of #50; fix = diff against first parent); buggy (first parent) 6e18ad4484701514501aeb44908da0d139466512; PRs: #50 merged 2016-02-09; threads read: #50 (column C: body, 1 comment, 0 reviews, 0 review comments, timeline), #46 (issue it closes: body, 12 comments, timeline)
- E1: ✓ merge resolves; `Participant::try_collect` collects the old bag before publishing the new local epoch (src/mem/epoch/participant.rs).
- E2: ✓ "Both @msullivan and I have gotten assertion errors in the stress-msq.rs test" (https://github.com/crossbeam-rs/crossbeam/issues/46); msullivan: "I've observed a thread dequeue 82101, then 82001, 82002, …" and, with poisoning, "can get the queue to spit out poisoned val[ues]".
- E3: ✓ another thread advances the epoch and adds garbage to the bag being collected.
- E4: ✗ reclamation logic and memory-unsafety symptom: `try_collect` stores `self.epoch` (src/mem/epoch/participant.rs:102) before `global::get().garbage[new_epoch.wrapping_add(1) % 3].collect()` (l.104), epoch-based garbage collection that frees nodes still in use; #46's diagnosis is reuse of freed memory (poisoned values).
- E5: not assessed; additional (labelled): the fix also changes `Relaxed` to `Release`.
- First failure: E4

### N27 — crossbeam 6e333047cebf (crossbeam#146)
- Repo: crossbeam-rs/crossbeam; fix 6e333047cebf3bcb6d9e87794ee809b653d8bdba (a branch commit of #146); buggy 36783bded050439705738592b938d9af07194e00; PRs: #146 merged 2017-07-24; threads read: #146 (column C: body, 2 comments, 3 reviews, 2 review comments, timeline), #107 (linked: body, 9 comments, timeline)
- E1: ✓ commit resolves; clones the waiting thread's handle before setting `ready`, then unparks the clone (src/sync/ms_queue.rs).
- E2: ✓ "If I try it on my 8 core box … then I do indeed get a segfault" (https://github.com/crossbeam-rs/crossbeam/issues/107, comment by BurntSushi; title "MsQueue Segfault").
- E3: ✓ the woken thread returns and drops its stack `Signal` before the waker's `unpark` dereferences it.
- E4: ✗ memory-unsafety symptom (use after free, segfault) and park/unpark through a raw pointer to a stack `Signal` (src/sync/ms_queue.rs:38): the popper does `(*signal).ready.store(true, Release)` then `(*signal).thread.unpark()` (l.177-178) while the pusher waits in `while !signal.ready.load(Acquire) { thread::park(); }` (l.298-299).
- E5: not assessed
- First failure: E4

### N28 — crossbeam 9178ce15a59f (crossbeam#382)
- Repo: crossbeam-rs/crossbeam; fix 9178ce15a59f1ef8398323d0a5890c3db3f3f59d (a branch commit of #382); buggy 240cf90ee32dc63a8b885c47eabe7ddee337996a; PRs: #382 merged 2019-05-21; threads read: #382 (column C: empty body, 6 comments (bors), 0 reviews, 0 review comments, timeline)
- E1: ✓ commit resolves; the last sender and last receiver each disconnect, and a `destroy` flag decides which one frees the counter (crossbeam-channel/src/counter.rs).
- E2: ✗ "Build failed" and "Timed out" (https://github.com/crossbeam-rs/crossbeam/pull/382#issuecomment-494194132 and issuecomment-494208395). These report observed failed/timed-out CI runs; the bare race title alone would not pass. Nothing in the evidence links the failed checks to this race. Every listed §2 item was read; none states an observed symptom of the row's bug.
- E3 (additional): ✓ the last sender and the last receiver release concurrently.
- E4 (additional): ✗ deallocation logic on the path: `Sender::release` and `Receiver::release` decrement with `fetch_sub` and, when `disconnect` returns false, `drop(Box::from_raw(self.counter))` (crossbeam-channel/src/counter.rs:61-64, 114-117); which side frees the shared counter is the race, and §5 excludes deallocation and reclamation logic.
- E5: not assessed
- First failure: E2

### N29 — crossbeam f8bee404e7aa (crossbeam#345)
- Repo: crossbeam-rs/crossbeam; fix f8bee404e7aa9bd005b6a16c6bbcbe59d04ce0a7; buggy 86eff71c893644d5150c209fe903472472a15189; PRs: none in commit-pulls; threads read: #345 (column C: body, 6 comments, timeline), Amanieu/seqlock#3 (linked: body, 2 comments)
- E1: ✓ commit resolves; replaces `AtomicCell`'s one-word stamped lock with a wide sequence lock (crossbeam-utils/src/atomic/atomic_cell.rs, seq_lock).
- E2: ✗ the failure is hypothetical: "the `usize` in the lock can, in some extreme scenario, wrap around in the middle of a read" and "On 64-bit machines, this would take a few lifetimes to occur, on 32-bit machines it might require the reader thread to be inactive for around one second." (#345 body, quoting seqlock#3); "I'm not sure if this worth caring about"; the seqlock maintainer: "the probability of hitting this is so low that it may as well be non-existent". Every listed item was read.
- E3: not assessed
- E4: not assessed
- E5: not assessed
- First failure: E2; additional (labelled): E4, torn reads through `ptr::read` of a non-atomic value (unsafe, memory unsafety).

### N30 — rand e0e8263c25dc (rand#48)
- Repo: rust-random/rand; fix e0e8263c25dc291f818cd20c034912de5ae05189 (a branch commit of #48); buggy 8457d200cd573c40a5fec395138dc6cc95902a3e; PRs: #48 merged 2015-04-19; threads read: #48 (column C: body "Fixes an issue described in rust-lang/rust#21538.", 3 comments, 0 reviews, 0 review comments, timeline), rust-lang/rust#21538 (body, 2 comments, timeline), rust-lang/rust#24526 (linked std fix: body, 9 comments, empty reviews/review-comments, timeline)
- E1: ✓ commit resolves; `is_getrandom_available` checks once through `Once::call_once` instead of two `Relaxed` atomics (src/os.rs).
- E2: ✗ "Test failed - [auto-linux-32-opt]" (https://github.com/rust-lang/rust/pull/24526#issuecomment-94265900), the std counterpart linked in rand#48 comment 93976502. This reports an observed test failure, without attributing it to the fixed race. In contrast, rust#21538 names the getrandom wrong-value symptom only conditionally ("The failure mode would be reading the initial false value"); the bare race title is not a symptom. The CI failure is on another repository's PR, and nothing in the evidence links it to the row's bug. Every listed §2 item was read; none states an observed occurrence of its conditional wrong-value symptom.
- E3 (additional): ✓ a thread may see `GETRANDOM_CHECKED` set before `GETRANDOM_AVAILABLE`.
- E4 (additional): ✗ system call and FFI on the path: the check calls `getrandom`, which is `syscall(NR_GETRANDOM, buf.as_mut_ptr(), buf.len(), 0)` through an `extern` declaration (src/os.rs:34-49), inside `is_getrandom_available` (l.97-101); the fix uses `Once`.
- E5: not assessed; additional (labelled): E5(a), with both stores in program order the stale read needs `Relaxed` reordering (l.100-101), weak memory; rust#21538: "relies on the ordering of reads and writes that use Ordering::Relaxed" and "two atomic writes can be swapped".
- First failure: E2

### N31 — rand bccefbc27f6d (rand#466)
- Repo: rust-random/rand; fix bccefbc27f6db0f59de15b6cce0b59785d383f27 (a branch commit of #466); buggy cf84ba66d74a00ad4f241e47ebedcf074c508df4; PRs: #466 merged 2018-06-15 ("Add Unix fork protection"); threads read: #466 (body, 10 comments, 11 reviews, 20 review comments, timeline)
- E1: ✓ commit resolves; the fork counter becomes an `AtomicUsize` and registration an `AtomicBool` (src/rngs/adapter/reseeding.rs).
- E2: ✗ no observed failure: the message names only the change ("Use atomic fork counter"); in #466 the author writes "I thought it didn't matter if checking/setting the static was a bit racy, but that was wrong." after the reviewer's argument that reading a mutable static concurrently with a write is not guaranteed to give the old or new value; both the fork tests in the thread succeed. Every listed item was read. Unsure: "that was wrong" could be read as a stated defect, though not an observed one.
- E3: not assessed
- E4: not assessed
- E5: not assessed
- First failure: E2; additional (labelled): E4, `pthread_atfork` and `fork` (OS mechanism) on the path, and the old counter is a `static mut` (unsafe).

### N32 — redox 2a66a84a5073 (no column C)
- Repo: redox-os/redox; fix 2a66a84a507335c54e84bee3270b35a8a49f3b9e; buggy cc8fe85e6a5ab811c79eb0c98ec8c5e6ed972264; PRs: none (commit-pulls empty); threads read: none (index lists no threads; no GitLab item named)
- E1: ✓ commit resolves; the bump allocator moves behind a `spin::Mutex` (alloc/bump_allocator/src/lib.rs), among other kernel cleanups.
- E2: ✗ the message names only the code change ("Cleanup, use spinlock for allocator"); no other evidence exists. Every listed item (the commit) was read.
- E3: not assessed
- E4: not assessed
- E5: not assessed
- First failure: E2; additional (labelled): E4, kernel allocator (`__rust_allocate` over `static mut HEAP`, unsafe) and `spin`.

### N33 — redox 979d80a8c71e (no column C)
- Repo: redox-os/redox; fix 979d80a8c71e2605859c65e46ccef674178f1e71; buggy b756dd59ebdaeeab8c3769990b2b45845a66188e; PRs: none (commit-pulls empty); threads read: none (index lists no threads; no GitLab item named)
- E1: ✓ commit resolves; `print!` writes through a `spin::Mutex<SerialPort>` instead of a fresh unlocked `SerialConsole` (arch/x86_64/src/lib.rs, serial.rs).
- E2: ✗ the message names only the code change ("Fix locking in serial port"); no other evidence exists. Every listed item (the commit) was read.
- E3: not assessed
- E4: not assessed
- E5: not assessed
- First failure: E2; additional (labelled): E4, kernel serial port I/O (`Pio`, x86 port I/O) and `spin`.

### N34 — redox redox#303 (redox#303)
- Repo: redox-os/redox on gitlab.redox-os.org; column B is `redox#303`, looked up per §2 "Column B forms" (the bulk fetch saved nothing for this row; the GitLab and GitHub responses were read by GET separately): merge request 303 of `redox-os/redox` (`/api/v4/projects/redox-os%2Fredox/merge_requests/303`) → HTTP 404; issue 303 (`/issues/303`) → HTTP 200, state closed (created 2015-11-20; `closed_at` is null), https://gitlab.redox-os.org/redox-os/redox/-/work_items/303, "Redox lacks abstractions, causing undefined behavior and an overuse of `unsafe`" (imported from GitHub, "Created by: mahkoh"); its saved notes response (`/issues/303/notes`) is `[]`, although metadata reports `user_notes_count: 18`; an r1 re-audit GET of `/api/v4/projects/2/issues/303/notes?per_page=100&page=1` returns HTTP 401, so no additional notes could be read; GitHub `redox-os/redox` issue 303 → HTTP 410 ("Issues are disabled for this repo"), pull 303 → HTTP 404. Threads read: issue 303 (full saved description and empty saved public notes response; metadata/notes discrepancy and current access limit disclosed).
- E1: ✗ §2's narrowing: a `redox#N` row passes E1 only as a merged merge request or pull request whose commits resolve, or an issue whose thread names a resolvable commit or merged request as its fix. The accessible issue evidence names neither (the saved notes response is empty; the description proposes a rewrite of `do_sys_open` but names no commit); no merge request or pull request 303 exists. Every returned item was read; unavailable notes are not claimed absent. This E1 decision uses the saved public lookup, not an independently reproduced full current thread.
- E2: not assessed
- E3: not assessed
- E4: not assessed
- E5: not assessed
- First failure: E1; additional (labelled): E2, the saved issue description gives code review findings ("This is racy because other user threads might currently be modifying said memory"; "Naturally this is only true if Redox runs on multiple cpus (cpu cores). But even if it does not right now…"), not an observed failure; E4, kernel system-call code reading user memory, memory unsafety.

### N35 — tock bf8e0e2e0ecf (no column C; message names #617)
- Repo: tock/tock; fix bf8e0e2e0ecf11449642dab9426372b86f53227a (a branch commit of #758); buggy e23594ad215ca9e0a933d6a57694d3b7135ef7de; PRs: #758 merged 2018-03-01; threads read: #758 (body, 4 comments, 0 reviews, 0 review comments, timeline), #617 (issue named in the message: body, 7 comments, timeline), #581 (linked from #617, "6LoWPAN Fragmentation": body, 14 comments, 13 reviews, 19 review comments; it carries the test suite, not the fix)
- E1: ✓ commit resolves; the RF233 driver disables reception after a frame arrives and re-enables it after the frame is read out (capsules/src/rf233.rs).
- E2: ✓ "This bug can be reliably replicated by running the `lowpan_frag_dummy.rs` test suite … Tests number 7 and 25 fail" (https://github.com/tock/tock/issues/617, "Radio Race Condition"); later: "I just ran into this problem again in the 6lowpan test suite".
- E3: ✓ the radio receives a second frame while software reads the first over SPI.
- E4: ✗ hardware mechanism: the race is between the RF233 radio's frame buffer, which a newly arriving frame overwrites, and the driver's interrupt-driven SPI reads (capsules/src/rf233.rs: IRQ_STATUS handling, l.157-164, 313, 375, 425; the state machine whose receive states the fix extends, l.124-136); §5 excludes interrupt handlers and hardware behaviour.
- E5: not assessed
- First failure: E4

### N36 — tock 36ac574543e1 (tock#10)
- Repo: tock/tock; fix 36ac574543e13aa25fe5e9b89538ce4ff7c40739 (a branch commit of #10); buggy 8f2e6d2e258e68880737ef8c64ee670374db7d8a; PRs: #10 merged 2016-01-13 ("Spi"); threads read: #10 (body, 0 comments, 0 reviews, 18 review comments, timeline)
- E1: ✓ commit resolves; it wires DMA channels to a new SPI driver and dispatches their interrupts (src/chips/sam4l/chip.rs and others).
- E2: ✗ "Reads of loopback fill the buffer is 0x01" (commit 36ac574543e13aa25fe5e9b89538ce4ff7c40739). This is a stated wrong output in the newly wired DMA driver, although the commit says it remains unfixed. It is not evidence of that output at the first parent; no different buggy revision is substituted. Every listed §2 item was read; none states an observed symptom of a repaired first-parent bug.
- E3 (additional): ✗ neither the message, complete #10 discussion nor first-parent diff identifies the reported wrong bytes as caused by an interleaving, timing fault or blocking cycle. The message offers a memory-pointer problem or the synchronous framing byte as explanations, while callbacks and reentrance guards remain to-dos. Wiring asynchronous DMA and the dataset's concurrency classification do not establish that cause. No concurrency-shaped repaired defect at the frozen first parent is identified.
- E4-E5: not assessed. Additional E4 restriction: the actual path uses DMA, memory-mapped registers, interrupt handlers and `static mut` peripherals, all excluded hardware/unsafe mechanisms.
- First failure: E2

### N37 — parity-ethereum 796482c12917 (parity-ethereum#5910)
- Repo: openethereum/parity-ethereum; fix 796482c12917768e09376557f0c37adc057bf322; buggy b6fd18a9ca09832ad72f1cced2da22453e5f5237; PRs: #5910 merged 2017-06-22; threads read: #5910 (body "Might be related to #5907", 1 comment, 1 review, 0 review comments, timeline), #5907 (linked: body, 11 comments, timeline)
- E1: ✓ commit resolves; `AuthorityRound::generate_seal` claims the step with `compare_and_swap` on `proposed` instead of `load` then `store` (ethcore/src/engines/authority_round.rs).
- E2: ✓ "Error: Engine(DoubleVote(dd0bb0e2a1594240fed0c2f2c17c1e9ab4f87126))" (https://github.com/openethereum/parity-ethereum/issues/5907, body). It is an observed error; PR #5910 hedges its relation to this race ("Might be related"). Later comments report improvement after network-topology changes and suggest another fix. The symptom passes E2 without asserting that this commit repairs the reported network case.
- E3: ✓ check-then-act: two threads both see `proposed == false` and both seal the same step.
- E4: ✗ external crate and a guard held across other locks and an effect: between the check and the store, `generate_seal` (ethcore/src/engines/authority_round.rs:321-322) calls `self.signer.sign(header.bare_hash())`, which is `self.account_provider.lock().sign(*self.address.read(), self.password.read().clone(), hash)` (ethcore/src/engines/signer.rs:50-51): util's `Mutex`/`RwLock` = `parking_lot` (util/src/standard.rs:49 at the buggy revision), the provider's guard held while two more locks are taken and the account provider signs.
- E5: not assessed
- First failure: E4

### N38 — parity-ethereum 7503d6695a8e (parity-ethereum#440)
- Repo: openethereum/parity-ethereum; fix 7503d6695a8ef22fa0aa4530745ceed95a260f63 (a branch commit of #440); buggy 718646f943b1388fb68afa95f5e79aca13e53a93; PRs: #440 merged 2016-02-19 ("UDP Discovery"); threads read: #440 (body, 6 comments, 0 reviews, 8 review comments, timeline)
- E1: ✓ commit resolves; `start_session` waits until no other thread holds the handshake's `Arc` before `Arc::try_unwrap` (util/src/network/host.rs).
- E2: ✓ "Fixed panic on session creation" (commit 7503d6695a8ef22fa0aa4530745ceed95a260f63; unhedged, states the observed panic).
- E3: ✓ another thread holds a clone of the handshake `Arc` when the session is created.
- E4: ✗ `Arc` reference counts and mio: `start_session` does `Arc::try_unwrap(h).ok().unwrap()` (util/src/network/host.rs:638-639), which panics while another thread's clone from `handshakes.get(token).cloned()` (l.661) is alive; Tide v0's `Arc` has no `try_unwrap`/`get_mut`/counts (§5 mappings), and the fix spins on `Arc::get_mut`. The host runs on `mio` (l.24-25).
- E5: not assessed
- First failure: E4

### N39 — parity-ethereum 145e766db2de (parity-ethereum#3191)
- Repo: openethereum/parity-ethereum; fix 145e766db2de4b20b208a9bb2478479b74f855fe; buggy caab72394d8c84a4fe2c5e59de7f2523147f4401; PRs: #3191 merged 2016-11-04 ("Fixing possible race in ethcore_hashContent"); threads read: #3191 (body, 0 comments, 0 reviews, 0 review comments, timeline)
- E1: ✓ commit resolves; the `on_done` closure blocks on `rx.recv()` instead of `rx.try_recv()` (rpc/src/v1/impls/ethcore.rs).
- E2: ✗ the commit/title say "Fixing possible race condition in ethcore_hashContent"; the complete #3191 body says "Was happening especially in tests (when `TestFetch` was spawning and executing the thread before `ready` handle was transmitted." This states occurrence of a race, but no panic, wrong value, lost output, hang, failing test or other enumerated symptom. The PR has no comments, reviews or review comments and its full timeline adds no symptom. Inferring a panic from the patch's `try_recv().expect(..)` is source analysis, not an allowed quoted symptom report (§3).
- E3-E5: not assessed. Additional E4 restriction: the actual `hash_content` path holds `self.fetch.lock()` across an external HTTP request and uses JSON-RPC `Ready` (first-parent rpc/src/v1/impls/ethcore.rs:298-313); no model was attempted.
- First failure: E2

### N40 — parity-ethereum 6999957869a1 (parity-ethereum#5764)
- Repo: openethereum/parity-ethereum; fix 6999957869a14b22fe3b22d797a232ab8976caf5 (a branch commit of #5764); buggy a792b3015dd464d65b0419b32146d912fcf9bea8; PRs: #5764 merged 2017-07-06 ("SecretStore: generating signatures"); threads read: #5764 (column C: body, 0 comments, 6 reviews, 6 review comments, timeline), #5502 (linked: body, 3 comments, timeline)
- E1: ✓ commit resolves; session maps check for a duplicate id inside `insert` instead of `contains` then `insert` (secret_store/src/key_server_cluster/cluster_sessions.rs).
- E2: ✗ "signing only worked for t-of-N groups, where t was even" and "it was not compatible with some non-distributed EC-Schnorr implementation" (https://github.com/openethereum/parity-ethereum/pull/5764, body §1.1-1.2). These are stated failures in the wider PR, not observations of the duplicate-session race; that commit's own message is only "fixed possible race". They concern the PR's signature-calculation defects. Every listed §2 item was read; none ties an observed symptom to the row's duplicate-session race.
- E3 (additional): ✓ the fix makes the duplicate-session check and insertion atomic; the first parent releases the read guard after `contains` and only later acquires a write guard in `insert` (secret_store/src/key_server_cluster/cluster_sessions.rs:344-345, 352-362), allowing concurrent creators to overwrite the same session id. This is separate from the PR's signature-calculation bugs.
- E4 (additional): ✗ the actual first-parent map operations use external `parking_lot::RwLock` (same file l.22), excluded anywhere on the affected path by §5. No synthetic replacement of that primitive is allowed; no model was attempted.
- E5: not assessed
- First failure: E2

### N41 — renderdoc-rs 22351fe532cf (RUSTSEC-2019-0018)
- Repo: ebkalderon/renderdoc-rs; fix 22351fe532cf480e26a7ad8717d86a023da8f1d6 (squash of #32); buggy 6a01ca45455d3e7d22f2d5cd985ee3155e883f98; PRs: #32 merged 2019-05-18; threads read: #32 (body "Fixes #27. Fixes #28.", 0 comments, 0 reviews, 0 review comments, timeline), #27 and #28 (issues it fixes: bodies, 0 and 2 comments), #24 and #19 (named in the CHANGELOG lines of the message: doc-comment fixes), and the advisory `crates/renderdoc/RUSTSEC-2019-0018.md` in rustsec/advisory-db (GET; saved as WORK/screen/N41/RUSTSEC-2019-0018.md), whose `url` is #32
- E1: ✓ commit resolves; `trigger_multi_frame_capture` and `set_capture_file_comments` take `&mut self` (src/api.rs).
- E2: ✗ hedged as possible only: the advisory "This is technically unsound and calling these methods from multiple threads without synchronization could lead to unexpected and unpredictable behavior." (RUSTSEC-2019-0018, keywords `undefined_behavior`); #28 repeats it ("could lead to"); #27: "it is critical to fix this in order to prevent unsound behavior in future versions"; the only comment suggests filing an advisory "when fixing UB". No observed failure. Every listed item was read.
- E3: not assessed
- E4: not assessed
- E5: not assessed
- First failure: E2; additional (labelled): E4, a soundness advisory (memory-unsafety symptom) over FFI calls into the RenderDoc C API (`(self.entry_v110().TriggerMultiFrameCapture.unwrap())(num_frames)`, unsafe).
