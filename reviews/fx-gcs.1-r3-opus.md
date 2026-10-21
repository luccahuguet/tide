# Tide review, fx-gcs.1 round 3 (Opus): converge round

Fresh pass after round 2. Three fixes, staged, not committed. Diff hash (`git diff HEAD -- . ':!.beads' ':!reviews' | sha256sum`): start `b1c14b2ad1c037eeddee0052df2df89697aed434a49dcdf028c600be74956c22`, end `7ae22b21d6d14eac474fc5a52a763ffa9364536e0a5244a49dd4751d75d2c3bd`.

## Fixed

1. `conformance/cases/authority/admit_atomic.rs:3-4`: ADMIT-1 (03 §2.5) checks every path and method call against the allowlist, so the old line `a.into_inner()` (a core atomic method) would also be `FX0520`, yet only the constructor line was marked; once the checker exists, per-line marking would fail the case for an incidental reason. The case now touches an atomic on one line only: `let _a = core::sync::atomic::AtomicU64::new(0); //~ FX0520`.
2. `conformance/run.sh:26-34` (`--trace`): it ran `sed` and `grep` per rule-and-case pair; it now reads each case's rules line once and runs one `awk` per ID. Output byte-identical.
3. `conformance/README.md:30-32`: the `FAIL` bullet now says rustc's indented output follows a rustc mismatch, since the runner does not print only one line per case then.

## Left alone

- New dependencies (`proc-macro2`, `quote`, `syn`, `oracle_macros`, `tide_macros`), the stub's public API, `pub fn` in cases, and the README example: as rounds 1 and 2 found (the stub is a near-verbatim copy of `trials/e1-dialect/rustc/`; `pub` keeps dead-code warnings out of FAIL logs).
- `par_map_cell.rs`: rules.md lists PAR-4 as both `FX0612` and `E0277`; nothing settles whether the checker also reports `FX0612` on a line rustc rejects, so the single `E0277` mark stays.
- `lock_payload_not_send.rs`: `Rc<u64>` is `Pure` (02 §9 looks through `Rc`), so `E0277` alone is correct.
- rules.md line numbers: every ID row points at its bold definition, except PAR-3 (03:1465) and CLAUSE-6 (04:466), which point at the part they cover; every rule ID defined in 02, 03, and 04 appears in rules.md.

## Verification (rustc 1.96.0)

- `conformance/run.sh`: exit 0; PASS for lock_accept, lock_payload_not_send, par_map_cell; AWAITS CHECKER for admit_atomic (FX0520) and lock_payload_shared (FX0606).
- `conformance/run.sh --trace`: exit 0, byte-identical before and after the rewrite.
- Mutations on scratch copies: a removed rustc mark, a wrong code, a wrong line, `E0277` swapped for an FX code and back, a malformed `FX66`, an empty `//~`, an unknown ID, no rules line, and `error` where rustc gives a coded error each FAIL. A binary `fn main(x: u32)` marked `E0580`, a `#[cfg(test)]`-only `E0308`, and an uncoded lifetime error marked `error` each PASS. The edited admit_atomic awaits the checker, and passes with its FX mark removed (the documented limitation).
