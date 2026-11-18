Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate d89c8b8 (docs review round)

## Verdict

**approve.** No blocking finding. Every speed, reliability and safety claim in README.md cites a
landed report and matches it, or is labelled as a target. The A1 framing is honoured, every Scope
item is present, the quickstart is statically identical to the commands the clean check runs, and
the driver.md move is faithful with correct recipe fixes. Six non-blocking findings follow; N1 is
the one I would fix before landing, because it is a speed number a reader can misread.

## Scope

- Candidate: worktree `/mnt/build/eon-agents/worktrees/tide-fx-qlek.3`, HEAD
  `d89c8b859afcf19dee1cbac9e05f52bba6a4163a` on `fx-qlek.3`, base `d4a91a2`. Tree clean at entry
  and before this write.
- SHA-256 of `git diff d4a91a2..d89c8b8`: `82885254590d1b9d1bae18434b2fa9aa66d5d7c93af4c060203658a89cbb581e`.
- Read first: AGENTS.md, GOVERNANCE.md (Merge Rule), `br --no-db show fx-qlek.3` with all
  comments (scope-cut note, recipe note 04:28, replay-rebuild note 07:34, owner decision A1/B relayed
  12:37 with the date correction, start note); fx-qlek (Decisions, Scope, Acceptance, comments),
  fx-qlek.5 (Acceptance, comments, close reason), fx-gbq4 (tagline comment), fx-0514, fx-djwo,
  fx-x2oo, fx-a9oh, fx-wktt, fx-xw5h headers.
- Sources checked against the README directly: `trials/e3-pilot/report.md` and `protocol.md` §12;
  `trials/e1-checked/gate-report.md`; `research/36-benchmarks.md` (§0, §1, §1.1, §1.6, rows and
  tables); `reviews/2026-11-17-red-team-opus.md` (all routes); `demo/README.md` output;
  `examples/case-studies/*/README.md`; `examples/capabilities/`; `docs/install.md`;
  `docs/docker.md`; `docs/owner-guide/07-trust-diff.md`; `design/01-design.md` (D12 targets, tier
  table); `design/03-authority-concurrency.md` RUN-4 holes 5 and 6; `conformance/rules.md`
  (CLOSURE-3, REF-0402, CAP-1, REF-0912); `docs/driver.md` in full against
  `git show d4a91a2:README.md` lines 87-642 (word diff); `scripts/clean-install-check.sh`;
  `Cargo.lock` and the crates' manifests (for the `--offline` change); every relative link and
  anchor in the eleven changed files; every README citation in `design/`, `docs/`, `research/`,
  `conformance/`.
- Read only after my own pass: `target/work/report.md`, `r1-codex.md`, `r1-recheck-codex.md`.
- Not run: the Nix clean check, the cargo recipe, any build (static review, as briefed).

## Claim table

Each speed, reliability or safety claim in README.md at d89c8b8.

| README line | Claim | Source | Result |
| --- | --- | --- | --- |
| 3 | Tagline "A capability-safe, replayable subset of Rust." | fx-gbq4 comment 2026-11-17 18:01 (research/35): low risk, no rewording | confirmed |
| 5, 183 | Tide v0 alpha, x86_64 Linux only | fx-qlek Decisions (Version, Platform); install.md:3; docker.md:3 | confirmed |
| 6-9 | Checked subset of Rust 2024; pinned unmodified nightly rustc against Tide's std after `tide-admit` | design/01 D18; research/35 §1 | confirmed |
| 16-22 | Sim tests under a seeded simulator of tasks, channels, `Shared`, files, TCP loopback, clock, env, stdin; `--tide-run-test` records, `--replay` reruns on the same machine and checks identical event and output traces | driver.md:337-347 (simulator), replay paragraphs | confirmed |
| 22-24 | Demo: lost update on 8 of 16 seeds; first failing seed replayed twice, identical traces | demo/README.md:64-76 | confirmed |
| 24-26, 235-238 | Replay reproduces the model, not the program; same machine, identical pinned artifacts | driver.md replay paragraphs; Achilles W7; debugging.md:170 | confirmed |
| 27-31 | Capabilities that admitted safe code cannot forge "by any route tried so far"; std has no free function for files, clock, env, network; reach only through handed capability values, printing aside | red-team Result; conformance CAP-1 (TD0501); A1 "unforgeable capabilities", hedged | confirmed (hedge is correct; unhedged would contradict RUN-4 hole 5) |
| 31-38 | Driver closes three forge routes through two open rustc issues: #25860 `fn`-pointer (`TD0206`), #57893 blanket and projection forms (`TD0402`) | rules.md CLOSURE-3 (TD0206), REF-0402 (TD0402); 03 RUN-4 hole 5; red-team routes 6, 6b and the corpus-covered fx-6xi2 route; A1 "three rustc soundness holes closed" | confirmed |
| 38-44 | Red team tried 45 numbered routes; two #57893 forms admitted, rejected after its fixes; every other route rejected or denied except two admitted on purpose; other rustc soundness bugs stay open | red-team routes 1-45 (+6a, 6b), Result; fx-qlek.5 close reason "45 escape routes recorded"; RUN-4 hole 5 | confirmed (see N5) |
| 46-50 | `--tide-diff` summarizes what a change may do: declared authority widened/narrowed, `unsafe` sites; declared only; E3 gives no evidence it helps | driver.md:56-69; owner-guide 07; E3 report "gives no evidence"; A1 | confirmed |
| 54-71 | E3 pre-registered, negative; first two paragraphs verbatim | e3 report.md:3, 5-10 (wrap-only differences); protocol §12 sentence filled in, not reworded | confirmed |
| 73-75 | Reviewers models only; small single-file changes; large changes and humans not tested | e3 report Limits, "What the data shows" | confirmed |
| 79-83 | E1: one writer, 30 tasks twice per language, 60 trials each; parity not benefit (quote) | gate-report.md:8-15, 20-25 | confirmed |
| 85-86 | First check 56/60 vs 59/60, ratio 0.949153, floor 0.8 | gate-report.md:30, 37 | confirmed |
| 87 | Final pass 58/60 each, ratio 1 | gate-report.md:31, 38 | confirmed |
| 88-90 | Iterations 86 vs 59 runs over 58 passing trials, ratio 1.457627, limit 1.5; interval 1.285714-1.637931 crosses 1.5 | gate-report.md:32, 39, 66 | confirmed, unrounded |
| 94-96 | Four programs, Tide and plain Rust, one machine; not D0's E4/E5 gate | research/36 "What this is not" | confirmed |
| 98-103 | Stability caveat (quoted) | research/36:47-52, verbatim | confirmed |
| 105 | "Every comparison above is a loss for Tide or a tie. None is a win." | research/36:79 | confirmed |
| 108-110 | Cold debug build 3.8-4.6x; one-line-edit rebuild 5.2-6.3x incremental Rust; no incremental build | research/36:65, 67, 111 | confirmed |
| 111-112 | Release, contracts erased: 0.901 s vs 0.906 s, tie, slower in 11 of 30 | research/36:70 | confirmed |
| 113-115 | Debug `contracts` 1.05x at 1,200,000 (tie); 1.19x at 200,000 outside a quiet window, slower in all 30 (loss) | research/36:71, 209-210 | numbers confirmed; wording ambiguous (N1) |
| 116-118 | `files` small loss, 26 of 30, paired median 1.07x; `tcp` 1.18x, all 30 | research/36:73-74, 213-215 | confirmed; comparison arm unnamed (N1) |
| 119-121 | `tasks` 2.29x `rust-release`: 1.74x flags, then 1.31x | research/36:75-76, 230 | confirmed |
| 123-125 | Edit check in tens of milliseconds | design/01:869-870 (D12: ~30 ms typical, <100 ms p95) | labelled **target**, "not built" |
| 129-141 | Admission catches neither case-study bug; Eon sim test fails on buggy, passes on fix; Rust can write it, Tide adds virtual time and determinism with no fake clock; hyper fix's shape, 20 seeds | eon-slow-close/README.md:3-9; hyper-ambient-runtime/README.md:3-28, 103-105 | confirmed, negatives first |
| 143-147 | Examples declare outcomes, checked by `cargo test --workspace`; `ambient_read.rs` rejected with TD0501 | examples/capabilities/README.md:14; ambient_read.rs:1 | confirmed |
| 151-154 | Nix route with no rustup and no system C compiler; these are the commands the clean check runs | install.md; clean-install-check.sh:108-136, 146-167; static extraction below | confirmed |
| 172-173 | Prints `[3, 7, 9, 14, 20]`; generated test passes in the kernel sandbox | sorted_insert.rs:2; install.md:82-92; check script :182-184 | confirmed |
| 174 | Docker runs the same route on macOS or Windows | docker.md:3-6 | confirmed |
| 185-202 | Alpha includes the driver's checks, std surfaces, runnable evidence, conformance corpus | driver.md; conformance/README.md | confirmed |
| 209-219 | Not in this alpha: packages (fx-x2oo), `tide` (fx-a9oh), sim faults/crashes (fx-djwo), HTTP/UDP/Rand draws/Proc ops, D12 front end, contracts in the diff, sandbox level in `tide check`/`review`; one crate, `extern crate` rejected | bead headers (all DEFERRED); REF-0912 TD0912; driver.md:230, 498 | confirmed |
| 221-223 | v1 planned items | design/01 tier table line 84 | confirmed, labelled planned |
| 227-233 | Simulator fidelity table | driver.md:287-291 (par in order), 337-347 (virtual time, inputs), 496-499 (net), 513-518 (fs latency, no links/crashes/faults/partial effects) | confirmed (short-writes error fixed in r1) |
| 242-245 | Run modes under Landlock and seccomp; prints level; partial continues | driver.md:433-467 | confirmed |
| 247-252 | `full` observed on host (Linux 7.2.6, ABI 10), clean container (fx-qlek.1), Docker default seccomp (fx-qlek.2); no macOS/Windows run | install.md:86; fx-qlek.1 record; docker.md:45-64 | confirmed |
| 253-256 | Builds unsandboxed (RUN-2, fx-wktt deferred); direct runs unsandboxed | driver.md:480-482; fx-wktt DEFERRED | confirmed |
| 260-263 | Admission rests on rustc; other soundness bugs open | 03 RUN-4 hole 5 | confirmed |
| 264-267 | Routes 22 and 24 admitted on purpose | red-team table rows 22, 24 | confirmed |
| 268-270 | `core::*` globs admit later rustc additions until the pin-bump audit | red-team "Left for others" | confirmed |
| 271-275 | Metadata reads, loader on granted library, RUN-6 output escaping, RUN-2 | red-team "Reading the sandbox"; driver.md:480-482 | confirmed |
| 276-278 | `unsafe` grants are trust holes; artifact lists every site | RUN-4 hole 1; driver.md:59-60 | confirmed |
| 279-281 | Availability and side channels | RUN-4 hole 6 | confirmed |
| 282-285 | Trust diff cannot show M4-M6 | e3 report "What was tested" | confirmed |
| 289-291 | fx-0514 unspecified | fx-0514 (DEFERRED) | confirmed, one line |
| 292-293 | TD0402 fails closed (`impl<T: ?Sized + Hash> Key for T`) | red-team "Left for others" | confirmed |
| 294-298 | Whole-workspace build after the demo changes std's bytes; replay refuses with `dependency artifact mismatch` | bead comment 2026-11-17 07:34; owner-guide 06 | confirmed (placement, N2) |
| 302-307 | Production code agent-written; Codex then Opus on a frozen candidate; GOVERNANCE states exceptions | GOVERNANCE.md Merge Rule (Opus stand-in, planning-only, Starcompass exceptions) | confirmed |

Quickstart, statically: the README's one `sh` block (156-168) equals `docs/install.md` blocks 1-5
concatenated, in order; only block 6 (`nix develop`) is omitted. `clean-install-check.sh`
extracts all six install.md blocks from HEAD, substitutes only the clone URL, runs them as
`ubuntu`, and checks the output line, a `sandbox:` line and `test result: ok. 1 passed`. So the
check exercises exactly the README's commands, plus `nix develop`.

driver.md move: a word diff of old README lines 87-642 against driver.md lines 9-end shows only
(a) "has started, and its first piece" removed, (b) the install/docker pointer sentences and
the conformance/spikes paragraph removed (both restated in the new README), (c) relative links
re-rooted for `docs/`, and (d) the recipe changes. The recipe changes are correct: it builds
`tide-admit` before using `target/debug/tide-admit`; dropping `--offline` is required because
`tide_std` and `tide-macros` pull registry crates (`libm`, `rustix`, `syn`, ...) that a fresh
`CARGO_HOME` lacks; the example path exists and is a `#[tide::main]` bin with a generated
contract test; the JSON route handles Cargo's hashed `build/<pkg>/<hash>/out/` layout (bead
comment 04:28); the new paragraph carries the 07:34 replay-rebuild caveat. All relative links
and fragments in the eleven changed files resolve.

## Blocking findings

None.

## Non-blocking findings

**N1. One benchmark bullet reads as a build time; two bullets do not name their comparison arm.**
`README.md:113-115`: "Tide's plain debug build of `contracts` takes 1.05x plain Rust's debug
build at 1,200,000". The 1.05x and 1.19x are **run times** of the debug binaries
(`research/36-benchmarks.md:71`, run table :155: `tide-debug` 7.896 s vs `rust-debug` 7.506 s).
The bullet directly above (108) uses the same phrasing, "a cold debug build takes 3.8-4.6x", for
**compile** time, so a reader can take 1.05x as a compile-time tie, which would contradict 108.
Separately, `README.md:116-118`: `files`' 1.07x paired median and `tcp`'s 1.18x are against Rust
built with Tide's profile flags (`rust-release-tide-flags`, research/36:73-74), not plain
`rust-release` as the section title implies (vs `rust-release`, `tcp` is 1.19x). The numbers are
right; the reading is not pinned. Minimum fix (text-only): "in debug, Tide's `contracts` binary
runs in 1.05x the time of plain Rust's debug binary at 1,200,000 ..."; and "against plain Rust
built with Tide's profile flags, capability file I/O ... ; capability TCP ...".

**N2. The replay-rebuild caveat is not where the README tells users to build.**
`README.md:310-314` lists `cargo test --workspace` and `cargo run -p tide-demo`; the caveat sits
only in Limitations (294-298). The bead comment of 2026-11-17 07:34 asks the rewrite to "say so
wherever it tells users to build". driver.md does so at its recipe. Minimum fix: one clause after
the block, e.g. "A whole-workspace `cargo build` after the demo invalidates its recordings
([Limitations](#limitations))."

**N3. The quickstart omits its prerequisites.**
`README.md:151-168`: the block starts with `curl ... | sh` and `git clone`, but the README does
not say the machine needs `curl`, `xz` and `git`, or that the Nix installer asks for `sudo` once to
create `/nix` (install.md:8-9, 24). The clean check installs these as root before the guide
(clean-install-check.sh:161-163). A first-time visitor copying the block onto a minimal machine can
fail at line 1. Also `sh <(...)` is bash process substitution. Minimum fix: one sentence before
the block: "It needs `curl`, `xz` and `git`, and a bash-compatible shell; the Nix installer asks
for `sudo` once to create `/nix`."

**N4. `git clone <repository-url> tide` is not runnable as published.**
`README.md:159` (with the HTML comment at 170), as in install.md. This is a known owner action at
release (report D3) and the check substitutes it, so it does not block this round. It must not
reach the tag unfilled: recommend tide-lead record it on fx-qlek.7, whose Acceptance already
checks links "point at the tag".

**N5. The red-team count differs from the record's own count.**
`README.md:41` says "45 numbered escape routes"; the record's Result says "48 routes tried,
counting the accepted controls for route 6", and its table has rows 1-45 plus 6a and 6b. 45
matches fx-qlek.5's close reason, but 6b, one of the two forms the same sentence names, is not
among "45 numbered". Minimum fix: "tried routes numbered 1 to 45, plus the projection form 6b",
or cite the record's 48.

**N6. Two design citations now resolve only through a pointer.**
`design/reference.md:15` ("**(partial)** ... (README's Status)") and
`design/reference-companion.md:433` ("the replay manifest (README Status)") relied on the old
Status's detailed text, which is now in docs/driver.md; README Status (204-205) points there, so
the citations still lead a reader to the facts in one hop. `design/03-authority-concurrency.md:2011`
still resolves directly: README Status names RUN-6, RUN-2 and the `tide check`/`review` level.
Both files are single-holder, so this bead cannot edit them. Minimum fix: tide-lead records "cite
docs/driver.md" on fx-djp or for the reference's next holder, as the author's report proposes.

Nits (no action required): `README.md:218-219` lists the "sandbox level in `tide check` and
`tide review`" separately, though line 212 already says the `tide` command is absent.
"Not in this alpha" omits sealed interfaces and RUN-5's unreviewed-branch guarantee, which the
tier section (412-416) does list as not built. The Project Process bullets on `autorun.sh` and
`handoff-guard.sh` are internal tooling detail for a public visitor.

## Reasoning

- **Owner decisions (checked against the bead comments).** A1: the alpha leads with replay
  (16-26) and admission (27-44), naming unforgeable capabilities, hedged to the routes tried, and
  three closed forge routes. E3 is published as a pre-registered negative pilot. Its first
  paragraph, the protocol §12 sentence, is quoted unchanged, and the ceiling explanation follows
  it verbatim. The trust diff appears only as "summarizes what a change may do" (47), with "gives
  no evidence that it helps" (50) and no claim that it improves catching. A2 (trust diff in the
  headline) is not used. B is a filing decision with no README effect. The 04:28 recipe note is
  handled in driver.md (JSON route, hashed dirs, checked on a fresh clone per the report). The
  07:34 note is present but placed only in Limitations (N2). The fx-gbq4 tagline conditions hold:
  "Tide's std" is never called Rust's std, and nothing pairs "Rust" with official, original or
  only. tidelang.com as public home matches the 2026-11-15 owner note on fx-qlek.4 and fx-zu4f.
- **Scope items.** Status line and what works (183-205); headline order per A1; evidence links
  (demo guide, examples, both case studies, benchmark note); speed claims cited or labelled
  (benchmarks section and the D12 target); install (install.md, docker.md); names and tagline;
  "Not in this alpha" with packages, CLI, sim faults, HTTP, Proc and single crate/no
  dependencies; fidelity table plus "replay reproduces the model, not the program"; sandbox
  host and Docker levels and RUN-2; known holes (fx-qlek.5 left no admitted route open, and the
  README lists the two intended admissions and the residual holes); the fx-0514 line. Planned work
  is labelled planned or deferred. The prose is present tense; `rg 'design phase'` finds nothing,
  and there is no "now", "new" or "we added".
- **Speed claims.** Every number matches research/36, and the README carries the note's stability
  caveat and its "none is a win" verdict, so nothing reads as a speed win. The removed "as safe and
  fast as Rust" is the right call against these numbers. The owner owns positioning, so the
  report's relay list should reach tide-desk.
- **First-time visitor.** The opening paragraph says what Tide is. The bullets say what works,
  each followed by its limit. "Try it" is a copyable route with the expected output. Status, "Not
  in this alpha", fidelity, sandbox and known holes say what it does not do. The tone is
  evidence-first: every result that could flatter Tide (E3, E1, benchmarks, case studies) states
  its negative or tie first. Only "AI-welcome because it is checkable" is positioning, and it is
  the owner's (brief, beads), followed at once by the checkable process. N3 is the one place a
  newcomer is likely to stumble.
- **Agreement with Codex r1.** I independently reached its conclusions on A1, E1, the benchmark
  numbers, the case studies, quickstart equivalence and the move. Its B1 (short writes) and B2
  (`--offline`) are fixed at d89c8b8, and I confirmed both against the sources. N1, N2, N3, N5 and
  N6's per-citation analysis are not in its review.

DONE
