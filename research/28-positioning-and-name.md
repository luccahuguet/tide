# Tide's positioning against Rust, lust and Blaze, and its name (fx-k778)

Research note, 2026-11-13, drafted by the Tide coordinator (Claude Opus 5.5) from questions the owner asked that night. Read-only on code: no build ran, and nothing under `trials/e1-checked/` was opened.

Evidence labels:
- **[repo]**: read in this checkout;
- **[web]**: cited URL, read 2026-11-13 unless dated;
- **[report]**: the 2026-11-12 Blaze reports named in §5;
- **[inference]**: a conclusion no measurement has tested.

## 1. What Tide is for, if it doesn't compile Rust faster

The owner asked: lust and Blaze make Rust compile faster, so what is Tide's point?

- Tide's v0 checker is rustc plus the admission driver (D18) [repo: README Status]. Its build cost relative to plain rustc has not been measured (fx-ayxr). The Tide-in-Eon ladder gates adoption on "no measurable regression" (fx-ho6o).
- Tide's thesis is not about build speed. It rests on three guarantees layered on Rust's own: no ambient authority, whole-program deterministic replay, and a reviewable trust surface [repo: `design/01-design.md` Thesis].
- For Eon, the payoff is in its recurring defect class, lifecycle races (eon-awmq.6, eon-l4e7s and the `*-flake-*` beads). Seeded replay could make those reproducible failing tests once Tide has the process and Unix-socket models they reach (fx-ayh2, fx-ssek). Replay holds only where every reached foreign effect has a simulator model [repo: `design/01-design.md` Thesis item 2] [report: eon-desk `tide-in-eon-report.md` §1–3]. Gate: fx-dett.
- A fast edit loop is a Tide design target (D12, "tens of milliseconds"), not a measurement [repo: `design/01-design.md` D12]. §2 explains why it looks reachable.

## 2. Rust, lust, Blaze and Tide

| | Rust (rustc + cargo) | lust | Blaze | Tide |
|---|---|---|---|---|
| Bet | general systems language | how fast check/build gets on a Rust subset | second Rust compiler, warm daemon, agent loops | a Rust subset in which code can't overreach and programs replay bit for bit |
| Compiler | rustc / LLVM | own front end, trait solver, borrowck; dev builds emit C for tcc | own front end (rust-analyzer lineage), Cranelift/LLVM, own linker; never runs rustc | rustc + admission driver |
| Language | all of Rust | subset of valid Rust 1.99; must match rustc or reject; no proc macros, async, unsafe | all of Rust intended, plus opt-in extensions | checked subset of valid Rust 2024; no user proc macros, async or build scripts; `unsafe` only with a grant |
| Fidelity to rustc | is rustc | "0 false accepts" in its runs | Polonius default; known missed borrow errors; its docs disagree (99.8% vs about two thirds) | pinned rustc oracle, NLL |
| Toolchain pin | the project's | rustc 1.99 semantics | ignores the pin; own nightly, bumped daily | pinned nightly |
| Warm edit check | 1.4–10 s (`cargo check`, large crate) | 19–25 ms body edits (60–250x); signature edits 4–10 s at p95 | ~20 ms is a v1 figure; v2 unmeasured | D12 target, not built |
| Cold check | 1x | ~2.5x faster | n/a | not measured; rustc plus an admission pass |
| Dev-binary speed | 1x | 3.8–4.9x slower | backend-dependent | not measured; rustc/LLVM code generation (D18) |
| Memory | baseline | daemon: 5.7 GB peak over 300 requests (older 523k-line tree); 6.6 GB retained after the first check on the larger v0.7.0 tree, peak not measured | "a few GB for a small project" | not measured |
| Authority bounds | no | no | `bstd`, partial | yes by design (D18); the v0 checker enforces the ambient-operation table |
| Deterministic replay | no | no | simulation plus crash/torn-write store model | whole program, by seed, where every reached foreign effect has a simulator model; process and socket models are still missing |
| Contracts, purity | no | no | effect sets, limited Lean contracts | executable contracts, `Pure` |
| Maturity | stable | one real project (ts-rust); last push 2026-10-22 | 6,234 reachable commits; 29 contributor identities, overwhelmingly one author; v2 unmeasured | design phase, v0 checker |

Sources:
- lust: [web: github.com/t3dotgg/lust-lang, README on branch `next`; repository created 2026-10-15, last push 2026-10-22, release v0.2.0 2026-10-17, 108 stars]. Its figures are self-reported and were not rerun.
- Blaze: [report: §5], read-only at Blaze `7260ad3`.
- Tide: [repo]. No Tide build time, run speed or memory has been measured; fx-ayxr and fx-4cdq will measure them.

Conclusions:

1. lust and Blaze compete with rustc on edit-loop speed. lust has no authority bounds or replay. Blaze has partial, overlapping implementations: attenuable `bstd` capabilities, with documented confinement and provider gaps, and a simulator. It does not admit a whole program against a capability std the way Tide's D18 does [report]. Only Blaze therefore overlaps Tide's thesis [inference].
2. lust's README gives four reasons for its speed, without ranking them [web: lust README "Why it is fast"]:
   - no LLVM in dev builds;
   - less language to check: no proc macros, async or unsafe, and trait selection without search;
   - parallel per-function work;
   - a daemon that records each body's reads and rechecks only bodies whose reads changed, with rustc kept as the oracle.

   Tide shares part of the second: no user proc macros, no async, and a bounded trait system. It still admits granted `unsafe` [repo: README, `design/reference.md`]. That supports D12's target on Tide's subset without leaving valid Rust [inference]. fx-4cdq is a post-E1 spike that tests this. If it fails, the README's "Fast feedback" claim goes back to the owner.
3. Blaze overlaps Tide's pitch most (capability std, simulation, effects) and drifts from rustc by design. Eon ruled it ideas-only (eon-vydf9) [report]. fx-jrvd records its overlap.
4. Neither lust, Blaze nor Tide changes Eon's build today [report; inference for lust, which supports no async or unsafe].

## 3. A few-word definition

The owner wanted Tide definable in a few words.

- **Adopted** (fx-vp4u, refining fx-tcde): headline "A capability-safe, replayable subset of Rust"; plain form "a checked subset of valid Rust 2024 in which code can't overreach and programs replay bit for bit", with replay conditioned on simulator models for every reached foreign effect; audience form "for code agents write".
- **Rejected:**
  - "safer Rust": vague, since Rust is already memory-safe and Tide's added safety is authority safety;
  - "pure Rust": means "no C dependencies" in the Rust community;
  - "formal Rust": proofs are later tiers, and Verus, Prusti and Creusot own that space;
  - "Bend Rust": Bend is an interop partner (D16), not Tide's identity;
  - "Capability-safe, replayable Rust" (fx-tcde's first headline): it calls Tide "Rust" rather than stating Tide's relation to Rust (Tide admits only part of Rust, compiled against its own std), and that is the use the trademark policy restricts. The owner allowed a better description (2026-11-13: "update it only if you find a better description").
- The Rust trademark policy allows stating accurately that software is compatible with the Rust programming language [web: §4]. It also says the word marks "may not be used: To refer to any other programming language; To refer to a modified version of the Rust programming language, except as permitted below for minor changes", and no use may appear "official, affiliated, or endorsed" to a casual observer [web: "The Basics: Referring to Rust/Cargo"]. Compatibility statements such as "a checked subset of valid Rust 2024" or "every Tide program is a Rust crate built by rustc" are the allowed kind. fx-tcde's first headline, "Capability-safe, replayable Rust", and its plain form "Rust that can't overreach" used "Rust" as Tide's own descriptor, which is the case the restriction addresses [inference]. The adopted "subset of Rust" form states Tide's relation to Rust instead (D18). The trademark check before any public use (fx-gbq4) still covers the tagline as well as the name. Making "Rust" part of the product's own name falls under the policy's permission rule (§4). None of this is legal advice.

## 4. The name

**The collision** [web: crates.io API and GitHub API, 2026-11-13]:

- `tide` on crates.io is http-rs/tide, an async web framework: 3,716,565 downloads, 231,934 recent, last release 0.17.0-beta.1 in 2021-12, about 5.1k stars, last push 2024-01-05, not archived.
- The `tide-` prefix is crowded with that framework's plugins. 29 of the first 100 crates.io matches for "tide", sorted by downloads (`/api/v1/crates?q=tide&per_page=100&sort=downloads`), start with `tide`, for example `tide-websockets` (55k recent downloads), `tide-disco` (updated 2026-07), `tide-rustls` and `tide-sqlx`. Tide's crates, `tide-std`, `tide-admit` and `tide-macros`, would read as more plugins.
- Other projects named Tide: Stanislas7/tide (an experimental language in C), and two near-collisions named "Tidal": Tidal-Lang (a language built in Rust) and TidalCycles (a live-coding DSL).
- Free on crates.io: `tidelang`, `tide-lang`, `tidelang-std`.
- Live clashes outside Rust (naming brainstorm, 2026-11-12; evidence in `/home/lucca/.cache/eon-agents/tide-naming/notes.md`):
  - IlanCosman/tide, a fish-shell prompt (about 4.3k stars, active), defines a `tide` command that would shadow a `tide` CLI in fish;
  - ananthakumaran/tide, an Emacs TypeScript IDE (about 1.5k stars), owns `tide-mode` and the Emacs `tide-` prefix.
- eon-overseer: `tidelang-*` is fine for Eon, and nothing in Eon uses a `tide` command or the `tide-` prefix.
- Runners-up recorded with the decision: Neap, Menir, Shore (`tide-naming/decision.md`); ambit, writ, lockstep and others were rejected (`tide-naming/notes.md`).

**The Rust trademark** [web: rustfoundation.org/policy/rust-trademark-policy]:

- "Using the Rust trademarks within another trademark requires written permission from the Rust Foundation and may include additional conditions of use and a fee for use."
- Every use must not appear "official, affiliated, or endorsed" to a casual observer.
- The policy also allows some descriptive uses without approval: accurate compatibility statements, and crate or repository names referring to use with Rust. A language whose own name contains "Rust", such as "rerust", is a trademark within the meaning of the permission rule. Using it would need the Foundation's written permission and possibly a fee, and risk a forced rename [inference from the quoted rule].

**"rerust"** (owner's suggestion): rejected.
- the trademark rule above;
- the `rerust` crate is taken (an abandoned 2021 reactive-programming crate, 1,629 downloads);
- in corrosion terms, "re-rust" means rust returning after cleaning;
- it reads as "Rust again", a fork or redo, and drops the capability half of the pitch.

**Recommendation, now adopted: keep "Tide"** [inference]:
1. Rename the crates to `tidelang-*` before anything is published.
2. Say "the Tide language" in prose and on any site.
3. Keep `#[tide::main]` and the `tide::` paths. They live inside Tide's own std, and the framework never used `tide::main`. tide-admit would map `tide::` to the published `tidelang-*` crate (fx-gbq4).
4. Ship the command-line tool as `tidelang`, not `tide`, so it doesn't collide with the fish prompt's `tide` command.
5. Never put "Rust" in the name.

The in-repo rename surface is about 95 lines using `tide::main` in `design/` and `crates/`, about 150 with `conformance/` (2026-11-13), plus the `tide-*` crate and identifier names (`tide-std`, `tide-admit`, `tide-macros`, `tide-test`, `tide-profile`, `tide-grant` and others).

**Decision (owner, 2026-11-12, relayed from the naming brainstorm): keep "Tide".** Lucca compared Menir, Shore and Tide side by side and chose "tide it is!". The record is `/home/lucca/.cache/eon-agents/tide-naming/decision.md`.

The coordinator decided the disambiguation steps above:
- `tidelang-*` crates;
- the `tidelang` CLI;
- `tidelang-mode` for any Emacs mode;
- "the Tide language" in prose;
- a trademark check that covers the tagline as well as the name, before any public use.

They land after E1 runs, because E1's materials teach the current crate spellings, and before any publish or announcement (fx-gbq4).

## 5. Announcement timing

Recommendation [inference]:

- **Not now.** Syntax is provisional until E1, and the D0 gates may still adjust or stop the design. Tide builds only on Linux x86_64, from source, with no packaged install, and the speed figures are targets.
- **Not tied to Eon adoption.** Stage 1 of the Tide-in-Eon ladder (fx-ho6o; Eon epic eon-tide-adoption-7u87k) comes after Eon's v0.2.0 cut and about seven Tide beads, and Eon has priority.
- **The order:**
  1. A preview after E1 passes and the first demo (fx-nkv) runs: a concurrency bug replayed from its seed, an authority diff in review, and E1's learnability numbers. Before it: the tagline lands and passes that trademark check, the name is settled, the speed claim is measured or clearly labelled, and there is an install path (fx-1n5n).
  2. An Eon case study when fx-dett reproduces Eon's real supervisor races by seed.
  3. Adoption news only if stage 1 holds.
- Public announcements are the owner's; no agent posts.

## Sources

- lust: https://github.com/t3dotgg/lust-lang (README, branch `next`; releases page).
- Blaze, Tide's report: `/home/lucca/.cache/eon-agents/tide-research-blaze/report.md` (Blaze `7260ad3`, 2026-11-12).
- Blaze, Eon's evaluation (eon-vydf9): `/home/lucca/.cache/eon-agents/eon-blaze-eval-work/report.md`.
- Eon fit: `/home/lucca/.cache/eon-agents/eon-desk/tide-in-eon-report.md`.
- Rust Foundation trademark policy: https://rustfoundation.org/policy/rust-trademark-policy
- crates.io API: https://crates.io/api/v1/crates/tide and the `?q=tide` search.
- http-rs/tide: https://github.com/http-rs/tide
- Other Tides: https://github.com/Stanislas7/tide, https://www.github.com/Tidal-Lang, https://en.wikipedia.org/wiki/TidalCycles
- Tide: `README.md`, `design/01-design.md` (Thesis, D0, D12, D16, D18), and beads fx-ho6o, fx-4cdq, fx-jrvd, fx-tcde and fx-dett.
