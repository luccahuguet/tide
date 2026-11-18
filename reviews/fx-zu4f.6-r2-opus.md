Model: Claude Opus 5.5 · Date: 2026-11-18 · Candidate aff5218 (docs)

## Verdict

**Changes required.** One blocking finding, and its fix is text only. The documented negative stands: I found no candidate wrongly excluded, and the counts are right (E1 10, E2 1, E3 6, E4 15; 0 eligible).

## Scope

- HEAD `aff52182cbf82c645688f8db3c4608e8e98cc6d5`, base `7863e8f`. SHA-256 of `git diff 7863e8f..HEAD`: `ef7ef47e9675c4d7b370a0d3a6a4bb35650d98cf9f34b3c351c31d1a5dca435f`. `git diff --check` is clean.
- The change is one file, `research/38-external-concurrency-search.md`. This is a docs-only review round under GOVERNANCE.md's Merge Rule, run on the tree Codex reported clean (r1, then the r1 re-check; B1 and B2 were fixed in aff5218).
- I read AGENTS.md, the Merge Rule, the bead body and its comments, the registered protocol (`sha256sum` gives `0a02f5be…6308`, as recorded), both Codex reports, and the relevant parts of `design/reference.md` and `design/03-authority-concurrency.md`.
- The embedded protocol matches `target/work/protocol.md` apart from its closing fence.
- Spot checks, from the saved `search/qN.json`: the 158 unique issues, the 119 repositories, both duplicates and their positions (Q1.2/Q8.1, Q1.7/Q8.12), and the claim that all 20 Q7 results are tokio-rs.
- Ten assessments sampled against the saved threads, fix PRs and source: A7, A12, A13, A17, A18, A24 and A30 (tokio-rs and openvmm E4s), both nearest misses (A32 with its `readjob.rs`, `diskwatch.rs` and #1963 body, and A11 through Codex's byte check), and A15 and A19 through the re-check evidence.
- Registration order checked: `protocol.md` was written at 13:10:23Z, the bead comment recording its digest was created at 13:10:48Z, and the first search started at 13:11:15Z.
- I sent no new GitHub requests and reran nothing. The only file I wrote is this one.

## Blocking findings

1. **Twelve E4 exclusions do not meet protocol §3's recording rule, and the record does not say it departs from the protocol.** `research/38-external-concurrency-search.md:88-91` says upstream source was read only for A1, A11 and A32. The protocol requires that the builder "identifies the affected code path at the buggy revision" and records each E4 exclusion "with the check and the source lines" (protocol §3, embedded around record line 640). The other twelve E4 entries (A6, A7, A8, A12-A15, A17-A19, A24, A30) give no source lines. Meanwhile lines 28-29 say the search "follows … the operational protocol".
   - I don't think any of those outcomes is wrong. In each one, the issue or the fix itself puts an operation that fails E4 on the path: signals and child processes, io_uring, IOCP, Unix sockets, real TCP RST handling, the pyo3 GIL, a Python subprocess, a weak-memory fix.
   - But a skeptical reader of a pre-registered search will ask first whether exclusions were made without reading the code. Right now the record answers that only by implication.
   - **Minimum fix (text only):** add a sentence or two in Method saying the following. For these twelve, the E4 failure was decided from the issue and fix text, without reading source at the buggy revision. That departs from protocol §3's "with the check and the source lines". It cannot change these outcomes, because the operation that fails E4 is named in the issue or the fix, not inferred from code. Where the issue already links source lines (A7 links `local_node.rs` and `unix_node.rs` at `e6b458cc`), cite them. Also qualify "follows … the operational protocol" at line 29 to point to that disclosure.

## Non-blocking findings

1. **A7's E3 reason names only a single-activity deadlock** (line 137: "a re-entrant lock acquisition deadlocks"). The deadlock that was observed is one thread re-entering the trace port lock: "re-enters the same path on the same thread, and deadlocks on the port lock" (#5103 body). The record's own E3 standard fails a cause with no second concurrent activity (A3, A26).
   - E3 does pass on a different cause that the issue names: "A similar re-entry through `RemoteNode.state` can block behind a waiting writer". That is a reader and a writer interleaving. #5103 also fixes a lock-order inversion.
   - The outcome and counts are unchanged. Suggested reason: "the issue also names a nested read of `RemoteNode.state` blocking behind a concurrently waiting writer; #5103 also fixes an opposite-order acquisition of two port locks."
2. **The summary leaves out A30's actual ground.** Lines 19-20 list the E4 grounds but not "a weak-memory reordering", which is the stated ground for A30 (line 230). Add it to that list.
3. **"Both shapes fit Tide's simulator well"** (line 23) is the record's one claim about the simulator, and nothing in the record tests it. Line 256 rightly says "This record says nothing about whether it would". Suggest: "Both are the kind of interleaving Tide's simulator explores: a thread racing a file change, and two writers losing an update."
4. **Say plainly that Q7 dominates the screened set.** All 20 Q7 results are tokio-rs repositories with at least 1,000 stars. So Q7 supplies 20 of the 32 screened issues and 9 of the 15 E4s.
   - tokio is an async runtime, and its internals use `unsafe` and OS interfaces. The frozen E4 ("no async", no `unsafe`) excludes most of that code by construction.
   - Saying so heads off both public objections: that the search was built to fail, and that the 15 E4s mean something about Rust in general. A sentence under Counts or in the first bullet at line 241 would do.
5. **"What a catch would need" doesn't say what Tide would have to add, and the intro promises that it does** (line 25: "says what would change that"). The section says where the bugs are and what replacements the near misses would need. It does not map each E4 ground to Tide's own status, which is what a reader needs. Suggested mapping, checked against the design docs:
   - `async`/`.await` (A1, A13, A15, A18): Tide rejects them by design (`TD0601`, reference §6.10; "the evented runtime" is listed under "Not in v0", reference §13). This is not a planned addition, so async Rust can only be ported by the restructuring the faithful rule forbids.
   - External crates (A1, A8, A11, A32): "imported Rust crates" are not in v0 (reference §13). Even with imports, a crate that does its own I/O, such as rusqlite's SQLite or reqwest, would bypass the simulated capabilities.
   - Processes and signals (A6, A12, A19): `Proc` is planned (reference §6 capability table).
   - Unix sockets (A15, A19): `Net` has TCP, implemented in simulation as loopback (03-authority-concurrency, around line 2035). UDP is planned. There are no Unix sockets.
   - Fs permissions (A1, A19): planned ("Times and permissions remain planned", 03-authority-concurrency line 188).
   - A `Mutex` held across effects or around another acquisition (A7, A8): `Mutex` and `RwLock` are planned in `std::sync::raw` behind the `unsafe` grant (reference §7.4), and `Shared::with` takes a `Pure` closure that cannot nest.
   - Weak memory (A30): Tide's atomics are sequentially consistent only, by design.
   - `unsafe` and kernel I/O (A13, A14, A17): outside what the simulator models.

   A short list like this would make the negative useful as a roadmap signal. It would also show that some grounds, async and weak memory, are permanent design choices, not gaps.
6. **Give the registration times in the record, and correct the bead.** Lines 28-32 say the protocol was recorded "before any candidate was examined" but give no times.
   - The bead comment that records the digest says "Protocol registered (tide-lead, 2026-11-18T13:12Z)". That would put registration after the first search at 13:11:15Z. In fact the comment was created at 13:10:48Z, so the text's "13:12Z" is a typo. A public reader who compares the two will see an apparent order violation.
   - Suggested fix: state the three times in Method (protocol written 13:10:23Z, digest recorded 13:10:48Z, first search 13:11:15Z). tide-lead should also add a bead comment correcting "13:12Z". That is a ledger write, outside this candidate.
7. Style: line 25 calls the section "What a catch would need", but its heading (line 239) is "What a catch would need, and what this search found". Use one name.

## Reasoning

**Honesty and framing.** The record claims only what its sample shows. It says the counts "are not a rate for any population" (lines 62-63), that the simulator "was … never asked to find any of these bugs" (lines 255-256), and that the Eon studies "remain the only demonstrations of a seeded catch and a replay, and both are on the owner's own project" (lines 258-260). All of that is plain and correct.

The narrowings are tied to stated v0 limits: `TD0601`, `Shared::with` being `Pure`, sequentially consistent atomics, and "Not in v0: … imported Rust crates". They were registered before any search, as the file and bead timestamps show. The one weak point for a skeptic is the twelve E4 exclusions recorded without source lines (blocking 1). Non-blocking 4 and 6 would further strengthen the record against the "designed to fail" objection.

**Eligibility.** Every exclusion I sampled holds under the frozen rules.
- A12, A13, A14, A17, A18, A24 and A30 each put signals, task internals with `unsafe`, IOCP, io_uring, time-driver atomics under a multi-thread runtime, kernel TCP RST, or a weak-memory fix on the path. The saved issue bodies and fix titles say so directly.
- A32: #1963 changes only test code, makes the test wait for the first read, and does not skip or retry the test, so E1 passes. The test's trigger goes through `app.run(..., json!(..))` into `readjob::start(..., p: Value)`, which has `use egui::Ui; use serde_json::{Value, json};`, and the test scans with `egui::Context`. So external crates are on the path, not only in setup, and E4 fails correctly.
- A11's rusqlite transaction and A1's async reqwest/fs4 path stand, as Codex verified byte for byte.
- A22 failing E1 is strict but follows the registered direct-link rule. It would almost certainly fail E4 too (wasmtime-wasi, wasm32, TCP).
- The one stated reason I found imprecise (A7's E3) doesn't change any count.

**Usefulness.** The section is accurate but stops short of the reader's real question: what Tide would need. The design docs back every claim it does make: `Net` is TCP only, `Fs` has no permissions call, there is no `Mutex` in v0, and `Shared::with` cannot nest or suspend. Non-blocking 5 gives the missing mapping.

**Prose.** The prose is plain, precise and free of filler. Assessments keep a consistent E1-E4 format with quotes. The style notes above are minor.

**Proof.** I read saved evidence and ran local Python and `git diff --check` on HEAD aff5218 in this Linux worktree. I built nothing and sent no GitHub requests, because the change is a Markdown record with no model.

DONE
