# Tide review, fx-gcs.1 round 4 (Opus): converge round

Fresh pass after round 3. One comment-only fix, staged, not committed. Diff hash (`git diff HEAD -- . ':!.beads' ':!reviews' | sha256sum`): start `7ae22b21d6d14eac474fc5a52a763ffa9364536e0a5244a49dd4751d75d2c3bd`, end `77b8084a9523b67c5b79d927140756e531f868334c61042d2947979309c58d3b`.

## Fixed

- `conformance/run.sh:2-9` became lines 2-5. The header restated the mechanism that `conformance/README.md` owns ("Running" and "Case format"), and the two had begun to drift (the header said "when it defines `fn main`", the README "when a line starts with `fn main`"). The header now names the runner and points to README.md; the Usage and Exit lines stay.

## Left alone

- New dependencies, the stub's public API, `pub fn` in cases, and the README example: as earlier rounds found (verbatim trial stub; `tide_macros` supplies the contract attributes and `#[tide::main]`; `pub` keeps dead-code warnings out of FAIL logs).
- `AGENTS.md`'s "Verification" does not mention `conformance/run.sh`. Nothing there is false, and AGENTS.md is generated from `.agent-protocols.local.md`, so this is reported, not changed.
- FAIL messages for bad IDs and marks end with a trailing space: cosmetic.
- Inventory: every `**ID.**` defined in 02, 03, and 04 appears in `rules.md` (FFI-2..6 and RUST-2..5 through the v1 range line); the off-looking line references (PAR-3, CLAUSE-6, CLAUSE-4's "also 04:189") point at the part the row covers. The design files are unchanged since 2d82c40.
- Seeds match the design text: 03:455 for atomics, LOCK-1/4/5 at 03:1307-1323, PAR-4 with 03:780-782 (`Cell` is not `Sync`).

## Verification

- `bash -n conformance/run.sh` passed; `conformance/run.sh` exits 0 (3 PASS, 2 AWAITS CHECKER: FX0520, FX0606); `--trace` exits 0.
- Mutations (run with the pre-edit script): a removed `E0277`, an FX mark changed to `E0277` on a rustc-accepted case, an unknown ID, an empty `//~`, an FX-only mark where rustc rejects, no rules line, and a mark on the wrong line each FAIL; an uncoded `//~ error` lifetime case and `fn main(x: u32)` with `//~ E0580` each PASS.
