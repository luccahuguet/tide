Model: claude-opus-5-5 · Date: 2026-11-15

# Tide's Achilles heel: adversarial review

Read-only review of `/home/lucca/pjs/lab/tide` at `116ecdc`. I read the design record, the
reference, the README, the E1 gate report, research/28, the driver's invocation and allowlist code,
the open beads, plus a web check of the related tools. Nothing was built or run. **[E]** marks
evidence (file:line, bead, URL). **[I]** marks my inference.

## The Achilles heel

**Tide's guarantees hold only because Tide owns a very small world, and real programs live outside
it.** "No ambient authority in any dependency" and "every program replays bit for bit" are true
because every effect must go through Tide's own std, and each effect needs a real backend, a
simulator model, allowlist entries and conformance cases. Today that world is a single crate with no
dependencies. It has no TLS, no HTTP, no processes, no Unix sockets, no crates.io and no async
ecosystem. Even the owner's own motivating project, Eon, has its signature bug class outside
v0's models. There are only two ways to enlarge the world:

- import Rust crates as trust holes, with libc interposition and `--allow-unmodeled`;
- grow the std by hand, effect by effect, two implementations at a time, on one owner's review
  budget.

The first weakens the two headline guarantees into "except for the parts that do I/O". The second
is a treadmill that never reaches what infrastructure code needs. So Tide loses either way: it is
safe and replayable while it cannot run what users need, or it can run it and the guarantees become
the same caveats as Rust plus a sandbox plus Antithesis. This beats the other weaknesses because it
is their root cause:
- the value thesis can't be measured on real programs (W2);
- an OS sandbox ends up as the real enforcement anyway (W3);
- the "just use Rust + tools" alternative wins on reach (W4);
- governance cost grows with every surface the std adds (W6).

## Ranked weaknesses

### W1. Closed world: the guarantees grow weaker as the reachable world grows (FATAL unless designed around)
- **Claim.** v0 can't express most of its own stated target, "agents, proxies, storage engines,
  schedulers". The way out that the design plans, importing Rust crates in v1, keeps the
  guarantees only on paper.
- **Evidence.**
  - [E] The driver admits one `.rs` entry. Its only `--extern`s are Tide's std and the paired test
    library (`crates/tide-admit/src/invocation.rs:320-335`), and `extern crate` is `FX0912`
    (`design/reference.md` §12.2). Nothing under `crates/` reads `tide.toml`, `tide.lock` or
    `tide.seal` (grep: 0 files), yet `design/reference.md:1017-1048` teaches `[dependencies]`,
    grants and lockfile authority as v0.
  - [E] There is no TLS or HTTPS anywhere in `crates/tide-std/src` (grep: 0). `Net::get` is in the
    reference (`design/reference.md:500`) but not in tide-std.
  - [E] Imported Rust crates, FFI and the evented runtime are v1 (`design/reference.md:1071`; tier
    table at `design/01-design.md:72-95`). Async Rust (tokio, axum) and serde's `'de` are out of
    interop scope for good (`design/01-design.md:62-65`).
  - [E] Proc and PTY models are in the Later tier (`design/03-authority-concurrency.md:2156-2170`).
    Unix sockets are P3 (fx-ssek), and so is `sim.world.proc` (fx-ayh2). research/28 §1 says
    Eon's recurring defect class (lifecycle races) needs exactly those. The case-study bead
    fx-zu4f.1 had to exclude Eon's supervisor races "because v0 cannot express them".
  - [E] In v1, an unmodeled crate's file or socket calls fail with `EPERM`, or the run is marked
    non-replayable (`design/03-authority-concurrency.md:2150-2154`). Imported crates and their
    build scripts are listed as trust holes (03 RUN-4 item 3).
- **[I]** "In any dependency" holds today only because nothing can be a dependency. When imports
  arrive, the guarantee becomes "in Tide-written dependencies". For a CLI or infrastructure
  program that is mostly not the code that does I/O (hyper, rustls, rusqlite, tokio-based
  clients).
- **Cheapest test.** Take three real v0-target programs: Eon's supervisor, a small HTTPS-calling
  CLI, and a TCP proxy. List every effect and crate each one needs, then mark which ones v0 or v1
  Tide covers (1) with a simulator model and (2) without a trust hole. If under ~70% is covered
  with no hole, the headline guarantees don't apply to the programs the pitch names.
- **Strongest mitigation.** Make the trade-off explicit and graded instead of absolute:
  - per-item guarantee levels in the trust surface: *confined and modeled*, *confined and
    unmodeled*, *trusted hole*;
  - a headline that claims what holds: "authority and replay you can read for the Tide-written
    part, with every hole listed";
  - priority on the effect surface infrastructure code needs (TLS, process, Unix sockets) ahead of
    later-tier language features, or an embedding such as WASI components, where the host
    supplies capabilities and Tide doesn't have to own the world.

### W2. The value side of the thesis is unmeasured; the headline evidence measures cost (SERIOUS)
- **Claim.** E1 shows that writing Tide costs a model little, on tiny tasks. Nothing yet shows
  that Tide's guarantees help anyone. The public-alpha headline is "models write Tide as reliably
  as Rust" (fx-qlek). That is a parity claim, and parity is not a reason to switch.
- **Evidence.**
  - [E] Tide was behind Rust on every E1 measure: F 56 vs 59, P tied at 58, iterations 1.46x
    against the 1.5x limit. The I-ratio CI of 1.29–1.64 crosses 1.5. Sessions had a median of
    40 s, and first snapshots were 2–6 KB (`trials/e1-checked/gate-report.md:22-27,64-72,346-350`).
  - [E] Tide used 9.0x Rust's cache-read tokens (5.16M vs 0.57M), 1.9x the turns and 1.32x the
    wall time (gate-report table "Informational observation"). That is the per-session cost of
    loading a 25k-token reference.
  - [E] All four final failures were a literal `sync_channel(` substring check (gate-report:362-366).
    So P measures spelling at ceiling, not behaviour.
  - [E] E1 had one writer model and the owner declined a second ("nah, lets focus on making
    progress"). The grading chain was all Claude (gate-report:351-360). Coverage leaves out Rand,
    HTTP, Proc, distributed faults, multi-file and granted packages, and `tide review/fix`
    (gate-report:409-414).
  - [E] E3 (does the trust diff help reviewers catch planted widening?) and E6 (do agents pass
    `Root` everywhere?) are scheduled for "month 9" (`design/01-design.md:137-160`). No open bead
    runs either of them (`br --no-db list`: none).
  - [E] The case studies reproduce "the bug's class in a model of the affected part". fx-zu4f.1
    concedes "a hand-written model reproducing a race only proves the model".
- **[I]** A skeptic would say Tide has shown it is cheap to adopt and has not shown it is worth
  adopting. The E1 margin is thin enough that a harder task set or a different model could fail
  threshold 3.
- **Cheapest test.** Run an E3 pilot before the alpha announcement:
  - 12 planted authority-widening or exfiltration diffs in a Tide codebase of about 2k lines;
  - fresh Opus and Sol reviewers, half with `tide-admit --tide-diff` output and half with only the
    code diff;
  - a few hours of agent time, measuring the catch rate against D0's 1.5x threshold.
- **Strongest mitigation.** Lead the alpha with a benefit measurement (E3 pilot plus one replayed
  real race) and make E1 the supporting "and it costs you little" line. Add a second writer model
  before quoting E1 externally.

### W3. Capability safety sits on an unsound type system; v0 has no kernel backstop (SERIOUS for the security pitch)
- **Claim.** Against a *malicious* dependency, which is the threat the README names, safe code can
  forge a capability through a known rustc soundness bug. The fix the design specifies is not
  implemented, and kernel enforcement is in v1.
- **Evidence.**
  - [E] The design itself says that, through rust#25860 and cve-rs, "even safe code can forge a
    capability" (`design/03-authority-concurrency.md:879-881`).
  - [E] CLOSURE-3 / `FX0206` re-checks implied bounds at fn-pointer coercion
    (`conformance/rules.md:58`; case `conformance/cases/memory/closure_fnptr_reject.rs`). But
    `FX0206` appears nowhere in `crates/`, and the conformance README says codes the driver
    doesn't enforce "still await the checker" (`conformance/README.md:64-67`).
  - [E] Landlock/seccomp for deployed binaries is v1 (D7, `design/01-design.md` "Kernel
    enforcement"). The test sandbox (RUN-3) is also unbuilt; no `tide test` CLI exists.
  - [E] Web: #25860 is still open, and a 2025 fix attempt (PR #147543) did not close it
    (https://github.com/rust-lang/rust/issues/25860, https://github.com/rust-lang/rust/pull/147543,
    https://rustsec.org/advisories/RUSTSEC-2025-0028.html).
  - [I] FX0206 blocks only one route. rustc has many open I-unsound issues, and the
    #25860/#84591/#100051 family needs type-system implications to be fixed in general.
- **Cheapest test.** Port cve-rs's `transmute` (about 40 lines of safe code) into a candidate
  crate without an `unsafe` grant. Use it to turn bytes into an `Fs`, or into anything holding a
  directory fd, and read `/etc/hostname`. Run it through tide-admit on a throwaway worktree. I
  expect it to be admitted.
- **Strongest mitigation.**
  - Short term: implement FX0206 and change the README to "no *accidental* ambient authority".
  - Real fix: move kernel enforcement (Landlock + seccomp derived from `main`'s attenuation)
    into v0 for binaries. That makes the type system the explanation and the kernel the guarantee.

### W4. The premise is fragile: Rust plus off-the-shelf tools gets most of the way, and model gains shrink the rest (SERIOUS)
- **Claim.** Each pillar has an existing alternative that works on all of Rust or on any binary,
  and model improvements weaken the "agents need a smaller language" argument.
- **Evidence.**
  - [E] Authority per dependency: cackle (`cargo-acl`) declares fs/net/process API classes per
    crate and sandboxes build scripts and tests, though its author says it is "not a security
    boundary" (https://github.com/cackle-rs/cackle). Clippy `disallowed_methods` and cap-std
    cover the lint-plus-API half.
  - [E] Whole-program DST: Antithesis runs "any containerized x86 software" deterministically, with
    no language change (https://antithesis.com/product/how_does_antithesis_work,
    https://antithesis.com/blog/deterministic_hypervisor/). madsim, turmoil, shuttle and loom
    cover it within Rust.
  - [E] Models: Rust already passed first check at 98.3% on E1's tasks (gate-report:22). There is
    almost no "Rust is too hard for agents" error left for a dialect to remove, and Tide's
    reference adds a 9x token cost per session (W2).
- **[I]** Tide's distinct value is the *combination*: a compile-time signature bound, plus replay
  that covers the whole program, plus a trust diff in one artifact. That is a real wedge, but it
  needs W2's measurement to sell, and W1 limits it. "Low-entropy for models" ages badly; "readable
  authority for humans" ages well. The pitch already leans toward the second (README); the alpha
  headline (fx-qlek) leans toward the first.
- **Cheapest test.** Take a 300-line program and plant authority widening in it twice: once as
  Rust with cackle, `disallowed_methods` and turmoil, once as Tide. Compare what each catches and
  replays.
- **Strongest mitigation.** Position against that stack explicitly. The claim to make: Tide's
  check is semantic and type-directed and needs no list of APIs to deny, and the bound shows up in
  the signature reviewers read. Prove it with the E3 pilot.

### W5. Toolchain coupling: a pinned nightly, 17 rustc_private crates, a pin that has never moved (SERIOUS, slow-burning)
- **Evidence.**
  - [E] The pin is `nightly-2026-08-09` with `rustc-dev`. Its comment says the allowlist "and the
    label layer's message and span matching" are reviewed when it moves (`rust-toolchain.toml`).
  - [E] git log: the pin's file has two commits (2026-10-21 and 2026-11-11), and the channel has
    not changed since it was set.
  - [E] tide-admit links rustc_{abi, ast, codegen_ssa, data_structures, driver, errors, hir,
    hir_pretty, hir_typeck, infer, interface, lint, middle, parse, session, span,
    trait_selection}, with about 170 `rustc_` references.
  - [E] `diag/labels.rs` matches rendered diagnostic text such as `"borrowed here"` (lines 17-40).
  - [E] The allowlist is default-deny for core/alloc, but `allow("std::path::*")` with ten
    hand-listed I/O denials is a denylist inside an allowed glob
    (`crates/tide-admit/src/policy.rs:337-348`). A Path I/O method added to Rust later is admitted
    silently unless someone notices it at review.
  - [E] `-Aunstable-name-collisions` carries a pin-bump TODO (`invocation.rs:321-324`).
- **[I]** Each bump means a rustc_private port, plus an allowlist audit of every new
  core/alloc/std item, plus label regressions, all on one owner with agents. Security fixes in std
  and LLVM arrive only through bumps. The longer the pin waits, the bigger the eventual jump.
- **Cheapest test.** Bump to the current nightly in a worktree. Count driver compile errors,
  conformance verdict changes, and the new `core`/`alloc`/`std::path` items that no rule matches.
  That is one heavy run, booked through the coordinator.
- **Mitigation.** Do a scheduled monthly bump with an automated "new unlisted items" diff
  (rustdoc JSON of std at both pins). Turn `std::path::*` into explicit allows. Key labels on
  error codes and spans, never on message text.

### W6. Project throughput: one owner, process-heavy governance, design far ahead of code (SERIOUS)
- **Evidence.**
  - [E] 588 of 589 commits are by one identity.
  - [E] Since 2026-11-01, 184 of 408 commit subjects are records, reviews, rounds or ledgers. Doc,
    research and review insertions (about 52.9k lines) exceed crate insertions (about 38.9k).
  - [E] `reviews/` holds 544 files.
  - [E] Every code change needs a Codex pass, then an Opus converge round, plus re-checks
    (AGENTS.md Governance). The Codex budget caps it at one agent until about 11-17 (memory note).
  - [E] The design record is 9.3k lines across `design/`, for a checker that has no CLI yet: no
    `tide check/test/sim/review/fix`, only `tide-admit` and `--tide-diff`.
- **[I]** W1's treadmill (each new effect needs two backends, allowlist entries, conformance cases
  and two review passes) runs into this cost directly. Bus factor is 1, and the explanation of the
  design lives in prose that only agents read at this volume.
- **Cheapest test.** Measure median calendar days per landed production bead, from claim to
  landing, over the last 30. Project that over the std surface W1's test says is missing.
- **Mitigation.**
  - Freeze design text for v0, so new prose only records decisions.
  - Batch the effect-surface beads.
  - Use one review pass for low-risk std additions with a strong conformance case. That is the
    owner's governance call.

### W7. Simulator fidelity: "replays exactly" means the model replays exactly (MANAGEABLE, but overclaimed)
- **Evidence.**
  - [E] Not simulated: CPU time, memory exhaustion, stack depth, parallel speed-up, hardware faults
    beyond disk errors, kernel behaviour beyond the models, signals, and real child processes.
    Multi-call file operations apply whole, so a cancellation never shows partial effects
    (`design/03-authority-concurrency.md:2156-2163`).
  - [E] Virtual time advances only when every task is blocked (D15), so a slow computation
    never fires a deadline.
  - [E] Real and sim backends are separate implementations (`crates/tide-std/src/{fs,net}/{real,sim}.rs`).
  - [E] Production record and replay is v1 (D17), and the replay manifest is still in progress
    (fx-nkv.7).
- **[I]** Bugs that live in the real backend, in kernel semantics (fsync error behaviour, short
  writes, TCP RST and half-close timing), or in CPU-time races are invisible to the simulator.
  "Concurrency bugs become ordinary failing tests" holds for scheduling bugs in Tide-written logic,
  which is real but narrower than the README suggests.
- **Cheapest test.** Run a backend-differential suite: the same programs and seeds on real and sim
  backends, comparing observable outcomes on short writes, EIO and connection resets. Plant one
  bug that only CPU time triggers and confirm the simulator misses it.
- **Mitigation.** Publish a fidelity table next to the replay claim. Add a CPU-cost model (charge
  virtual time per basic block or per step under the simulator), and FoundationDB-style "buggify"
  points in the real backends.

### W8. Name and discoverability (MANAGEABLE)
- [E] `tide` on crates.io is http-rs/tide, with about 3.7M downloads, and the `tide-` prefix is
  crowded. A fish prompt and an Emacs mode also claim `tide` (research/28 §4).
- [E] The planned mitigations are `tidelang-*` crates and a `tidelang` CLI (fx-gbq4). Searches for
  "Rust tide" will keep returning the web framework.
- **Test.** Search "tide rust language" in an incognito session after launch.
- **Mitigation.** Stay consistent with `tidelang` everywhere.

## Checked and not real

- **Weak memory under the simulator.** Atomics are SC-only (`design/reference.md:742`; 03 table
  row at :366), so a baton-passing simulator loses nothing.
- **Build-time code execution.** No Cargo runs. The driver builds every rustc invocation, rejects
  raw flags and response files, clears the environment, and rejects user macros, `#[path]`,
  `include_str!`/`env!` and `build.rs` (README Status; FX0101/0911/0915). fx-kw8.5's narrow
  `build.rs` detection is cosmetic, because nothing ever runs build.rs.
- **Unstable features as an escape.** `-Zallow-features=` is empty (ADMIT-2).
- **Reaching Rust std through core/alloc.** Admission is default-deny on crate identity for
  core/alloc ADTs and impls (`walk.rs:317-347`). The exception is the `std::path::*` glob (W5).
- **E1 gaming or leakage.** Audits found 0 confinement breaches and 0 forbidden tools. The three
  Rust integrity holds were reviewed, and the anomaly in trials 1–3 was investigated. The report
  states its own limits honestly. The weakness is in what E1 measures, not in how it was run.
- **Trademark exposure in the tagline.** "subset of Rust" is the compatibility-statement form the
  policy allows (research/28 §3).
- **Async coloring.** Excluding it is a deliberate scope choice with a reversal condition (D18),
  not a hidden defect. Its cost is counted under W1.

## Suggested next steps (no beads; the owner decides)

1. Before the alpha announcement, run the **E3 pilot** (W2) and the **cve-rs admission probe**
   (W3). Both are cheap, and either one could change the headline.
2. Run the **coverage audit** of three real v0-target programs, Eon's supervisor included (W1).
   Let it decide whether the next quarter goes to the effect surface (TLS, Proc, Unix sockets) or
   to the language.
3. Reword the README's two absolute claims ("No code, in any dependency"; "Every program
   replays") into graded forms that match v0, and say which commands exist today.
4. Decide whether kernel enforcement moves into v0 for binaries (W3); it hedges W1 as well.
5. Do one **pin-bump rehearsal** now, while the driver is small (W5).
6. Add a second writer model to E1 before external quotation (W2).

DONE
