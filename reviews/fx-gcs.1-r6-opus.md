# Tide review, fx-gcs.1 round 6 (Opus): converge round

Fresh pass after round 5. Clean: no edits. Diff hash (`git diff HEAD -- . ':!.beads' ':!reviews' | sha256sum`): `0f4488c00183ff06ab28ebde6ead718118bfdf4b4d34b5c03d83ec37af493ef6` at start and end.

## Checked and found sound

- `conformance/run.sh`: IDs come only from rules.md's first table column (`= ID` rows and the excluded list do not count); empty and malformed marks fail; the `: error[E####]:` offsets, uncoded `error`, other-file path prefixes, unlocated `?:error`, and skipped `aborting due to` lines parse correctly; `log+="$(...)"$'\n' || code=1` picks up rustc's exit status; `grep -qw '^fn main'` does not match `main_helper`; exit codes and `--trace` behave as the README says.
- `conformance/rules.md`: about 50 line references in 02, 03, and 04 point at the cited rule; `git diff 2d82c40 HEAD -- design/` is empty.
- `conformance/README.md`: the section references (02 §11; 03 §2.6, §3.1, §3.2, §3.5, §6.4, §6.5, §8.1) exist and match; format, verdicts, and exit codes match run.sh; the D0 pointer and the top-level README line are accurate.
- Seeds: each comment matches its verdict, and together they cover accept, rustc reject, and AWAITS CHECKER.

## Left alone

- `tide_macros` and `oracle_macros`, the stub's `pub` items, `pub fn` in cases, the README example: as earlier rounds found.
- A removed FX mark still passes: the README's stated limit until the checker exists.
- Out of scope: AGENTS.md's Verification section does not mention `conformance/run.sh`; AGENTS.md is generated from `.agent-protocols.local.md`.

## Verification (rustc 1.96.0, offline)

- `conformance/run.sh` exits 0 (3 PASS, 2 AWAITS CHECKER: FX0520, FX0606); `--trace` exits 0.
- Mutations: E0277 flipped to E0308 FAILs; a removed rustc mark FAILs; a removed FX0606 mark PASSes (the documented limit).
