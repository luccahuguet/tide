# Tide review, fx-gcs.1 round 1 (Opus): conformance corpus format, runner, and rule inventory

## Verdict

request_changes. The format, runner, and inventory are mostly sound, and the seeds behave as the bead requires. I found one demonstrated gap in the runner, and the ID scheme misses a prefix for one section.

## What was reviewed

- Diff SHA, before: `git diff HEAD -- . ':!.beads' | sha256sum` printed `f0354b1a49ba26ef145c143101855d3e0c579645f8ce61e156cc67a914350d3e`. After: the same value. `git status` showed the same 15 staged paths both times and nothing untracked. I edited nothing in the repository.
- Read: AGENTS.md, GOVERNANCE.md, `br show fx-gcs.1`, `br show fx-gcs`, the whole diff, and `trials/e1-dialect/rustc/run.sh`. I diffed `conformance/rustc/{std,macros}` against `trials/e1-dialect/rustc/`. They differ only in the description and doc strings and in the relative `tide_macros` path, and `Cargo.lock` is byte-identical. I checked about 45 of rules.md's file:line references against the design text and all of them hold. I also checked the design chapters' definitions for `ROOT-1`, `LOCK-*`, `PAR-4`, and 02 §8.4–§8.5, and the `reference.md` lines that findings 1–3 cite. Every `**PREFIX-N` defined in 02, 03, and 04 (82 IDs) is in rules.md, either as a row or inside the excluded FFI-1..7 and RUST-1..6 ranges.
- `conformance/run.sh` (rustc 1.96.0): exit 0. Output: `AWAITS CHECKER authority/admit_atomic.rs: FX0520`, `PASS lock_accept`, `PASS lock_payload_not_send`, `AWAITS CHECKER lock_payload_shared: FX0606`, `PASS par_map_cell`.
- `conformance/run.sh --trace`: exit 0, 25 IDs. `ADMIT-1`, `LOCK-1/4/5`, and `PAR-4` list their cases and the others show `-`.
- Mutations, run on scratch copies with absolute paths:
  - Each of these gave FAIL and exit 1: mark removed, wrong code, extra mark on an accepted line, mark on the wrong line, an E mark added next to FX, no rules line, unknown ID, a prose comment containing `//~`.
  - Two passed that the docs say should not, or that the docs do not cover: a `//@ rules:` line that is not the first line (and a second rules line), and an unused import (warnings are not denied).
  - An empty `//~` counts as no mark, so it only fails when rustc errors on that line.
  - ROOT-1 shows the blocking gap below.

## Blocking

1. **`conformance/run.sh:58-60`: the runner checks only `--test`, so it cannot produce ROOT-1's design-cited rustc verdict.** 03:268-270 says "rustc rejects parameters on a plain `main` (E0580)", and `rules.md:82` records it. Under `--test`, rustc does not treat a case's `main` as the entry point:
   - A case with `fn main(root: Root) { .. } //~ E0580` FAILs with "rustc errors [], marked [3:E0580], exit 0".
   - The same case without the mark PASSes.
   - Compiling the same file with `--crate-type bin` gives `error[E0580]`.

   The trial mechanism the bead says to reuse compiled each program twice, once as lib/bin and once as `--test`. This runner dropped the first build. fx-gcs.3 would then have to change the format the bead was meant to fix once.

   **Fix:** check each case twice, as a library (a binary when it defines `fn main`) and as a test crate, and compare the union of rustc's errors with the marks; update `conformance/README.md` ("Running") and the comment at `run.sh:2-3` to match.

2. **`conformance/README.md`, Rule IDs table: 02 §8.4 has no prefix.** Row `rules.md:61` (02:743, the higher-ranked `with`/`with2` payload references) is `none yet` and is 02's own rule: 03 §6.5 only gives the signature. No prefix in the table covers 02 §8.4, yet the README tells the case beads to assign IDs "from these prefixes". Row `rules.md:63` (02:764, §8.5 closure bounds) also has no prefix. It restates 03 §8.1 (`PAR`) without being marked as a restatement.

   **Fix:** a text-only change, two parts:
   - Add a row such as ``| `SHARED` | 02 §8.4 | `Shared::with` payload references |``. Alternatively, state that 02:743 takes the next `LOCK` number in 03 §6.5.
   - Note that 02 §8.5 restates 03 §8.1, so its closure rules take `PAR` IDs.

## Non-blocking

1. **`run.sh:22` against README "The first line, `//@ rules:`":** the runner reads a rules line anywhere and merges several of them. Fix: `sed -n '1s#^//@ rules: *##p'`. The other option is to reword the README to "a `//@ rules:` line".
2. **`run.sh:53,55-57` parse the marks three times, with two different locators.** sed's greedy `.*//~` takes the last `//~` on a line and awk's `index` takes the first. An empty `//~` is silently treated as no mark. Optional fix: build one `line:code` list with a single awk, print `NR ":"` for an empty mark, then derive `bad`/`want`/`fx` from it with grep. Guard the empty-list case, because `<<<""` yields one empty line, which would trip the `bad` check.
3. **`conformance/README.md`, "It exits ... 2 when the stub std does not build":** `run.sh:20` also exits 2 when rules.md has no IDs. Fix: "2 on a setup error (the stub std does not build, or `rules.md` lists no IDs)".
4. **`conformance/README.md`, Rule IDs paragraph:** it says each prefix's rules are defined as `- **PREFIX-N. Title.**` bullets "as 03 already does". In fact `ROOT-1` has no title (03:268) and 04 defines `CLAUSE-N` as numbered list items (04:385). Fix: "a bold `**PREFIX-N.**` lead (optionally with a title) at the start of the rule's bullet or list item".
5. **`conformance/cases/authority/lock_accept.rs:8-12`:** the `let n: Result<u64, LockError> = ...; n` binding repeats what the fn's return type already asserts for LOCK-5. It also shadows the closure parameter `n`. Fix: make the body `hits.with(|n| { *n += 1; *n })` and update line 2's comment if needed.
6. **`conformance/rules.md:184-210`, "Findings for the case beads":** this puts open defects in a third home. The parent epic's acceptance says to fix them in `design/` or file them, and AP-PLAN-001 says issues hold material defects. After this round, put each finding as a comment on the sibling bead that owns it (findings 2 and 6 on fx-gcs.2; 1 and 5 on fx-gcs.3; 3 on fx-gcs.5; 4 split by area), then delete the section. "Tier unclear" can stay, since it explains the inventory's own boundary.
7. **`.agent-protocols.local.md:43` (rendered into AGENTS.md):** "Verification" still says design changes are checked only by review rounds and trials. Add a sentence: "`conformance/run.sh` checks the conformance corpus; run it after changing a case, the stub std, or a rule a case names." Then re-render. The main checkout has uncommitted edits to both files, so coordinate with that work.

None of these is v1 or Later-tier design text, so nothing goes to fx-djp.

## Left alone

- **New dependencies `proc-macro2`, `quote`, `syn`** (`conformance/rustc/macros/Cargo.toml`). They stay: the trial's pinned exact versions with an identical `Cargo.lock`; `oracle_macros` needs them for the `#[test]`/`#[sim_test]` shims and `derive(Error)`.
- **New dependencies `oracle_macros` and `tide_macros`** (`conformance/rustc/std/Cargo.toml`). They stay: the trial's std wiring (`#[tide::main]` and contract lowering from spikes/d18, plus the shims).
- **The stub std's whole public API and the macros crate's pub items.** They stay: a verbatim copy of the trial's stub apart from doc lines; trimming would only produce a fork.
- **Copying the stub rather than pointing at `trials/e1-dialect/rustc/`.** It stays: the trial stays frozen as a record, and one copy here stops four sibling beads from each copying it.
- **`pub fn` in the cases and in the README example.** They stay: without `pub`, every accepted case produces a `function is never used` warning.
- **The README example duplicates `lock_payload_shared.rs`.** It stays: a short format illustration.
- **The defensive `( -n "$want" && $code == 0 )` check, filenames with spaces or colons, and crate names starting with a digit.** Left alone: each fails safe as FAIL.
- **`rules.md:6`, "taken at `2d82c40`":** still accurate; `git diff 2d82c40 HEAD -- design/` is empty.
- **The `--trace` loop is O(IDs × files):** fine at this corpus size.
