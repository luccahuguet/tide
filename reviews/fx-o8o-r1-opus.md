# Tide review, fx-o8o round 1 (Opus): converge review round

Candidate: ff2b251 on branch `fx-o8o` (base 3fdc020), `git diff 3fdc020 ff2b251`, 99 files under `trials/e1-checked/`. The worktree's tracked tree was not modified; nothing heavy was run.

Model: claude-opus-5-5 · Date: 2026-11-07

## Verdict

request_changes. The design is sound, and I accept the six Opus decisions as implemented:
- Bubblewrap jail;
- driver-only Tide with no fallback;
- fx-dsa boundary marker;
- per-case doctest adapter;
- no writer-directory wrappers;
- setup-time vendoring.

The Acceptance is met as written, with one exception: B1 is a suite that fails correct Rust programs. Five findings block. Four are not text-only (B1–B4) and one is text-only (B5).

**Acceptance, row by row.**
- **Rust arm.** `target/e1-checked/rust-results.json` in the worktree records `sandbox: true` and 30/30 passed, which matches the host report.
- **Tide arm.** I ran `br show` on every owner in the map:
  - fx-yg5, fx-08m, fx-5vj, fx-byk, fx-zc1, fx-o8j, fx-iob, fx-o4h, fx-nkv.2/.3/.4/.6, fx-mhd, fx-m72, fx-hxw, fx-ecb and fx-dsa are all open, and each one blocks fx-ic2 directly.
  - Scope fit:
    - `std::float` and `HashMap` belong to fx-yg5, which names them, and its Acceptance covers every reference-admitted item tasks.md names, so the rest of `std::collections` falls to it too.
    - `io::Result` and `io::Write` (B3, B4) go to fx-byk. That is an inference, but a sound one: every `Fs`/`File` signature fx-byk must deliver returns `io::Result`.
    - TCP `Write` belongs to fx-hxw, which names it. BufRead/BufReader belong to fx-ecb. Contracts belong to fx-mhd, channels at `std::sync::mpsc` to fx-o4h (reference §7.3 puts them there), and `Shared` to fx-nkv.4.
  - The map gives first errors only. That fits the Acceptance's "its failure", and readiness.md says later diagnostics may surface more (see N7).
- **Trial validity.**
  - The writer directory holds only the prompt, the reference copies (Tide), the package files and the source.
  - Snapshots, session state, suites and references stay outside it. A suite enters a jail only at grading time, after the session.
  - `candidate()` refuses a symlink that points outside the writer directory.
  - Writer code runs only inside bwrap, with:
    - `--unshare-all`, so the only network is loopback;
    - one writable bind;
    - a clean environment;
    - no `/home`, `/etc`, repository or host `/tmp`.
  - No executable is put in the writer directory, so the dispatcher cannot be edited or replaced from there.
  - The read-only `/usr`, `/etc/alternatives` and `/nix/store` set is acceptable. It goes beyond "the system libraries they load" in §10.3, but it meets that section's stated purpose (no repository file, trial, suite or reference is readable). N8 asks for this reading to be recorded on fx-ic2.

## Findings

**B1 (not text-only, blocks). B10's Rust suite is stricter than the task, and the asymmetry favours Tide.** At `suites/rust/B10.rs:24-34`, case 3 asserts `Report { calls: 1, last_ok: None, elapsed: Duration::ZERO }`.
- The task gives the Rust arm slack for every case: "Rust: `elapsed` at least the stated time and less than it plus 150 ms". For case 3 ("`elapsed` zero"), that means anything in [0, 150 ms).
- Failure scenario: a correct Rust writer records `start.elapsed()` at each call. Its first call reports a few hundred nanoseconds, so it fails final pass. The Tide arm's virtual time is exactly zero, so Tide is unaffected.
- The reference passes only because it hard-codes zero for call 0 (`references/rust/B10.rs:23-28`, "The immediate first call is the start…"). That special case is evidence the suite encodes the reference's choice, not the task.
- This breaks §6 ("exactly what the task's checked cases say") and adds an asymmetry §11 does not allow.
- Fix: assert `(r.calls, r.last_ok) == (1, None)` and `r.elapsed < Duration::from_millis(150)`, as cases 1 and 2 do. Optionally drop the reference's special case, so the reference proves the slack.

**B2 (not text-only, blocks). A4 case 3 does not test what the checked case states.** At `suites/common/A4.rs:13-17`, the test multiplies two literals: `Poly::new(vec![1, 1]) * Poly::new(vec![-1, 1])`.
- Case 3 reads: "`(x + 1) * (x - 1)` built from owned values". The suite never calls owned `+`, and it calls owned `-` only in case 4.
- Failure scenario: a program without `impl Add for Poly` (owned) passes the whole suite in both arms. That inflates P for both arms, and the frozen suite cannot be tightened later (§6: "No test is added or tightened").
- Fix: build the product from owned operations, for example `let x = Poly::new(vec![0, 1]); let one = Poly::new(vec![1]); ((x.clone() + one.clone()) * (x - one)).coeffs() == [-1, 0, 1]`.

**B3 (not text-only, blocks). A writer action can brick its session for good.** In `harness.py:357-362`, `command()` increments `runs` and creates `snapshot-NN` before `candidate()` reads the source. It persists `session.json` only after the read.
- `candidate()` raises in four cases:
  - the writer deleted or renamed `src/lib.rs` (`FileNotFoundError`);
  - it made the path a directory;
  - it made the path a symlink out of the directory (`ValueError` from `relative_to`);
  - it wrote non-UTF-8 bytes (`UnicodeDecodeError`).
- In each case the exception escapes to `main` (exit 2, "E1 setup error"), and `runs` is never saved.
- On the next `./check`, the same `snapshot-NN` already exists, so `snapshot.mkdir()` raises `FileExistsError` on this and every later call. The writer can never run again, and those calls are not counted.
- If this happens on the first run, `snapshot-01/source.rs` never exists, and `grade --first` crashes instead of recording a first-check failure.
- Fix: make an unreadable candidate an ordinary counted failure:
  - catch the read error;
  - write `source.rs` as the raw bytes, or an empty file;
  - write a `results.json` record with code 1 and the reason;
  - persist `runs` before returning 1.
  - At minimum, persist `runs` before anything that can raise.
- Add one test: delete `src/lib.rs`, run twice, and expect exit 1 both times with `runs` equal to 2.

**B4 (not text-only, blocks). Run trees are never cleaned, so disk use grows without bound.**
- Every dispatched command leaves `snapshot-NN/run/`, including a full Cargo target dir under the jail (`harness.py:364-366`, `build_dir`).
- `references` also creates its sessions under `STATE/sessions/`, outside its `TemporaryDirectory` (`harness.py:335`, `:455`), so every readiness run leaks 30 sessions.
- Measured: the worktree's `target/e1-checked/sessions` already holds 724 sessions and 7.9 GB, at up to 66 MB per `./check` (A9, which builds serde).
- A Rust `./test` also does a debug build plus doctests. At 60 Rust sessions × up to 20 runs, the gate needs on the order of 100 GB.
- Failure scenario: the disk fills mid-gate, sessions end through infrastructure faults, and after a second fault §3 counts the trial as failed. Rust's larger builds make this fall unevenly on the arms.
- Fix: after each command, delete `run/` except what grading or the record needs (`results.json` and `source.rs` are already outside it). Have `references` put its sessions under its temporary root, or delete them.
- `test_native_wrappers…` reads `snapshot-02/run/writer-ran`. Assert the marker through the test's results or stdout instead.

**B5 (text-only, blocks). The docs contain stale and session-narrative statements.**
- `readiness.md:3`: "Candidate: committed draft 2e158b0 on main 3fdc020 plus current follow-up edits". The candidate is ff2b251.
- `readiness.md:18-21`: proof links into `/home/lucca/pjs/lab/tide/.worktrees/fx-o8o/target/...`. AP-GIT-001 removes the worktree after landing, so the links will dangle, and AP-PROOF-001 says a temporary path is not proof. Keep the hashes and point to the bead comment.
- `readiness.md:25-26`: "Preserve base_config's LD_LIBRARY_PATH … the orchestrator supplied that fix". This is an instruction to agents, and harness.py's comment already covers it.
- `readiness.md:84`, `:97`, `:135-139`:
  - "supplied bead inventory" and "No bead is created by this session" are false, because fx-dsa and fx-ecb exist;
  - "copy this map onto fx-o8o" is already done;
  - "This documentation update … makes no commit" is false.
  Replace them with current state and remaining work only.
- `README.md:51-52`: "this worktree's target, never the main checkout's target". After landing, `STATE` is the target of whichever checkout runs the harness.
- `README.md:34`, `:43-45`, `:131-135`: the absolute `/home/lucca/.cache/.../heavy.sh` and "Run only one heavy command at a time in this session" are operator-session policy, not harness usage. Show the plain commands and give one line of advice.
- `README.md:115-127`: "the orchestrating agent verified…", "Opus's revised decision … replaces dependency discovery", "`readelf`/`ldd` are no longer setup requirements" are change history. State the current mount policy and its reason in present tense.

**N1 (nit).** The Tide sibling-module probe skews more than visibility (`harness.py:199-204`, `grade` at `:383-384`):
- writer tests and acceptance tests share one crate, so each stage runs the other's tests;
- a writer's `#[ignore]` test fails the acceptance stage, which allows no ignored tests;
- writer code that uses `crate::Item` paths breaks once it is moved into `mod solution`.

All of this is masked by the fx-dsa marker today. Add a comment on fx-dsa saying the probe must be replaced, not extended.

**N2 (nit).** `structure()` adds a Tide-only `"#[tide::main]"` search for B1/B2/B5/B8 (`harness.py:59-60`). The tasks say "The binary has `#[tide::main]`" but, unlike every other structure requirement, do not mark it "(structure: …)". §6 checks only where a task "states a structure requirement". Drop the check, since admission already requires the entry point. Otherwise record the reading on fx-ic2.

**N3 (nit).** `run()` buffers unbounded stdout and stderr, and the jail sets no memory limit. A binary task stuck in a print loop can push gigabytes into the harness's memory within its 10 s. Cap the bytes captured and kill past the cap, and consider an `RLIMIT_AS` for jailed children.

**N4 (nit).** At `harness.py:249-255`, a Rust acceptance build whose JSON yields no test executable passes without running anything. Fail when `acceptance` is set and `binaries` is empty.

**N5 (nit).** Suite strength, all symmetric and none failing a correct program:
- B3 case 4 checks `NotFound` only through `summarize`; also check `top_at(fs, "missing", 1)`.
- A6 case 5's `get(&i)` always hits the most recent key, so an O(n) recency list with the newest entry at the front passes. Get an older live key, e.g. `i.saturating_sub(50_000)`.
- A7 case 3's absent word `"absent"` shares no prefix. Also assert `!t.remove("ca")`.
- A9 indexes `items[0]`/`items[1]` where looking up by sku states only what the case says.

**N6 (nit).** `suites/rust/fs.rs:9-10` reuses `e1-checked-<pid>`, and PIDs repeat across fresh PID namespaces. `create_dir_all` keeps leftovers from a killed test. This cannot change a verdict today, because the earlier test already failed. Run `remove_dir_all` first, so §6's "fresh temporary directory" holds by construction.

**N7 (nit, planning).**
- fx-dsa lacks the `e1-prereq` label that §10.1 asks of E1-filed beads.
- `std::task::Cancelled` is mapped to fx-zc1 alone, although `Clock::sleep` (fx-o8j) and cancellation state (fx-nkv.3) return it too. Name one owner on those beads.
- The first-error map hides known later owners: fx-ugm for C4's `with_all`, fx-nkv.5 for every simulated suite. Listing them would save fx-ic2 a rerun cycle.

**N8 (nit, planning).** §10.3 is frozen, and the harness reads "the system libraries they load" as whole read-only `/usr`, `/etc/alternatives` and `/nix/store`. The Opus mount decisions are on fx-o8o only. Per the bead's Scope ("a problem found in the frozen protocol goes on fx-ic2"), add a one-line comment on fx-ic2 so the gate report states the actual boundary.

**N9 (nit).** Results record no per-command wall time. The decision note asked reviewers to check headroom in `rust-results.json`, and that data is not there. From cargo's own output, A9's dependency rebuilds take about 3.7 s for check and 1.8 s for test on this host, so headroom is fine. Record `elapsed` in `run()` anyway, since §7.4 wants wall time per run.

**N10 (nit).**
- `harness.py:354` tests `config["started"] is not None`, but `prepare` always sets it, so the branch is dead.
- The 40-minute clock starts at `prepare`. That is documented, but fx-ic2 must then prepare each session just before it starts.
- `base_config`'s 120 s timeout on the tide-admit build turns a cold build into a setup error.

## Checked and acceptable

- **Doctest adapter.** It selects merged cases through `RUSTDOC_DOCTEST_BIN_PATH` → script → `timeout 10s`, and the marker file turns a killed `should_panic` into a failure. The fixture now uses `standalone_crate`, and the self-test sums 4 passed. `doctest-runner.sh` is mode 100755.
- **Timeouts.**
  - A per-test limit of 10 s, under a whole-command deadline of 120 s that is also bounded by the session.
  - `killpg` on bwrap, with `--die-with-parent` and the PID namespace reaping the jailed tree.
  - A completed-result check that catches a silent `exit(0)`.
- **Vendoring.** Content-addressed by the lockfile hash, published by atomic rename, safe under concurrent setup, and checked for an unchanged lockfile. Cargo verifies checksums on `--locked` builds.
- **Jailed Cargo.** It sees no candidate config: only `src/<entry>` is copied, the operator writes `.cargo/config.toml`, and no ancestor directories are mounted.
- **Binary cases.** Each case gets exactly its env, stdin and files. `binaries.json` fits both arms: Tide's `#[tide::main]` uses std `Termination` (`crates/tide-std/src/root.rs:153`), so exit codes and the `Error: ` prefix match.
- **Tide execution.** It goes only through the driver with the production std, uses one out-dir per mode, and its two markers stay in place whenever a plain harness passes, as the rewritten self-test checks.
- **The other suites.** Band C suites and the remaining Band A and Band B suites match their checked cases. C5 and C10 use the stated slack and concurrency witnesses.
