Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Re-check fc015c3

## Verdict

**approve** — findings 1–3 are resolved, and non-blocking item 4 is addressed. No new blocking defect found in the fixes. Confidence: high. This is the limited finder re-check required by GOVERNANCE.md, not another full review or an Opus/red-team verdict.

## Scope and identity

- Candidate: `fc015c3988a22a43651e13131390fbf17cc6ab05`, branch `fx-ayh2.1`; tracked working tree clean.
- Reviewed `git diff 998a0cb fc015c3`, the corresponding owning text and its immediate dependencies. Only 03, research/40 and companion 20 changed in this round. The builder's “Round s1-r1 fixes” account was treated as claims.
- Round diff SHA-256: `852fafaba65d0befaaca9acd98a9cf3b210e137ba95f90c4723158e33f5661d7`.
- Full candidate diff SHA-256 (`git diff 91a2f36 fc015c3`): `37ffbe01eb0c2292a8e065503076be0148c1c5b5a75427915e227b355f9259b2`.

## Finding statuses

### 1. Spawn cancellation ordering — fixed

`design/03-authority-concurrency.md:290` and `research/40-proc-command-v0.md:155` now distinguish the operations: output/status observe cancellation before the allowlist check; spawn does not observe cancellation. All three check authority before resolution, host access or model lookup. Research/40 explicitly gives the permitted/refused spawn outcomes in a cancelled scope.

This agrees with the unchanged no-new-checkpoint rule at `design/03-authority-concurrency.md:2405`. Companion 20's cancellation description (`design/reference-companion.md:612`) remains confined to output/status and does not add a spawn checkpoint. The unchanged decision summary (`design/01-design.md:1018`) and PROC-2 (`conformance/rules.md:97`) add no contrary ordering.

### 2. Immediate simulated kill — fixed

`research/40-proc-command-v0.md:256` now specifies submission without waiting, followed by termination as a ready action on the existing scheduler. It explicitly permits `kill(); try_wait()` to observe None before termination, preserves a normal exit that wins the race, and gives no-code status when termination wins. Exited-child kill still returns Ok.

The subset summary at `design/03-authority-concurrency.md:2251` agrees. It also agrees with direct-child submission (`:2275`), immutable lifecycle status (`:2312`), submission versus termination (`:2353`) and scheduler ownership (`:2385`). Companion 20 delegates these lifecycle details to that subset (`design/reference-companion.md:659`); neither 01's summary nor PROC-1 (`conformance/rules.md:96`) requires synchronous termination. No public signal/fault API or second scheduler was introduced by the fix.

### 3. PATH compatibility and POSIX explanation — fixed

`research/40-proc-command-v0.md:165` distinguishes empty PATH elements from relative directory prefixes. Its compatibility explanation and example (`:173`) now cover both command-level PATH overrides and inherited empty/relative elements. The cost discussion (`:331`) names both differences and their effect on program selection or lookup failure.

`design/03-authority-concurrency.md:295` consistently states absolute-only start-time lookup, skipped empty/relative entries, and child-only command PATH. Companion 20 (`design/reference-companion.md:638`), 01 D7 (`design/01-design.md:511`) and PROC-2 (`conformance/rules.md:97`) remain consistent summaries. The selected lookup/attenuation policy was not loosened.

### 4. Default inherited stdin — addressed

Companion 20 now includes `.stdin(Stdio::null()).status()` (`design/reference-companion.md:594`) and explains inherited stdin defaults and the simulator restriction immediately below (`:597`). Research/40 states the same requirement, additionally distinguishing output's null default (`research/40-proc-command-v0.md:271`).

The change agrees with §9.9's existing supported-mode boundary (`design/03-authority-concurrency.md:2255`) and the coverage-failure requirement in PROC-1. The pinned nightly's `library/std/src/process.rs` documentation and status implementation confirm inherited stdin for status/spawn and non-inherited stdin for output. The example configures the same supported mode for either backend; it makes no claim that a model is unnecessary.

## New blocking defects

None found within the fixes and their affected contracts. No minimum patch remains for these findings.

## Verification and limits

`git diff --check 998a0cb fc015c3` passes. `design/reference.md` is byte-identical between the reviewed revisions: both Git blob IDs are `7e949b288e8db0161c86ab3dc9ba0be6350abead`. No recount was needed; the prior measured 24,985/25,000 tokens still applies to that unchanged file.

No source edits, commits or Beads writes were made; this requested report is the sole write. No compilation or runtime tests were run for these documentation fixes. Runtime implementation and seed witnesses remain stage-2 obligations; the separate red team and Opus review remain outside this re-check.

DONE
