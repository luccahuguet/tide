Model: gpt-6.1-sol (high) · Date: 2026-11-16

Verdict: approve

## Scope and identity

Read-only finder re-check for fx-p44q under AP-PROOF-001 and GOVERNANCE.md. Read the original report, the amended author report, `br --no-db show fx-p44q`, and the complete `git diff ecea71b 1f6ef58`. Re-checked only the five blocking findings, the two non-blocking findings the author folded in, and the changes addressing them. This is not a repeat of the full review.

Candidate: `1f6ef588d4d2e249e6bf9774571fd534106b7b4e`, branch `fx-p44q`, base/main `502650c96b74e6454df083b6da80702ba64d7f49`. `git diff main...HEAD | sha256sum` returned `ea4991de25ad90bc4f85123a55a8ead1504aa1c8d1bac9ab8812a73cb3513066`, matching the brief. The working tree was clean on entry. The observable contract remains accurate distinction between delivered and planned/partial v0 behavior, without changing language decisions.

## Original blocking findings

1. **Fixed — sampled-clause unchecked reporting.** `design/reference.md:883-884` now distinguishes counting an exhausted clause as unchecked from the planned reporting of it. `reviews/fx-p44q-audit.md:127` marks row 76 partial and explains the missing runner output. This matches the exhaustion record and early return at `crates/tide-std/src/contract.rs:163-175` and the explicit planned-printing limit at `crates/tide-std/src/lib.rs:190-192`. The intended reporting behavior is retained without claiming it is delivered.

2. **Fixed — partial Arc/Weak support.** `design/reference.md:38-41` explicitly qualifies both types as partial and names missing `ptr_eq` and unsized payload support. `reviews/fx-p44q-audit.md:53,91` changes rows 17 and 50 to partial and supplies the implementation boundary. The qualification matches the current wrapper at `crates/tide-std/src/sync.rs:18-137` and the unsized-payload exclusion at `crates/tide-std/src/lib.rs:243`. The intended Rust-shaped API and the existing section 6.4 differences remain taught.

3. **Fixed — randomized hashing under simulation.** `design/reference.md:446-447` now warns that `RandomState::from_rand` panics with a simulated Rand. `design/reference.md:511-512` qualifies the blanket simulation promise for Rand and Proc. Audit row 42 is partial at `reviews/fx-p44q-audit.md:83`. This accurately reflects `crates/tide-std/src/collections.rs:23-40`, including the simulated-Rand rejection through `unmodeled("Rand")`; the real-backend recommendation remains intact.

4. **Fixed — wrong-input elision diagnostic coverage.** `design/reference.md:157-160` labels the wrong-input case partial and identifies rustc's error as today's behavior, while retaining the intended FX0202 teaching. Audit row 30 at `reviews/fx-p44q-audit.md:66` explicitly says this case has no FX code. That matches the source regression at `crates/tide-admit/tests/invocation.rs:50-59`, which expects the lifetime error and asserts absence of an FX diagnostic. Rust lifetime semantics are unchanged.

5. **Fixed — stale dbg! Status claim.** `README.md:356-357` removes `dbg!` from the unimplemented list while retaining the remaining limits. New audit row 110 at `reviews/fx-p44q-audit.md:176` records the correction. This matches the macro at `crates/tide-macros/src/lib.rs:145-182`, its export at `crates/tide-std/src/root.rs:34`, and its sink at `crates/tide-std/src/root.rs:137-143`.

## Folded-in non-blocking findings

6. **Fixed — implemented TCP latency understated.** `design/reference-companion.md:429-433` now identifies per-packet TCP latency as built and keeps HTTP timing and packet faults planned. Audit row 97 is partial with corresponding evidence at `reviews/fx-p44q-audit.md:158`. The seeded 1–10 ms draw exists at `crates/tide-std/src/task/net.rs:97-98`, and connect applies two draws at `crates/tide-std/src/net/sim.rs:181-184`; the amended split is accurate.

7. **Fixed — trusted_pure driver support omitted from evidence.** `reviews/fx-p44q-audit.md:64` now distinguishes the unavailable public attributes from the existing trusted-type/hole machinery. The cited support exists at `crates/tide-admit/src/walk.rs:103-105,261-273` and `crates/tide-admit/src/pure.rs:441-443`. Keeping the usable surface planned is appropriate.

## New blocking findings

None. No new inaccuracy was found in the fixes or their changed audit classifications/counts.

## Reasoning

All seven findings are addressed by documentation qualifications or evidence corrections within their original scope. The incremental diff changes only README, the reference, its companion, and the audit. It introduces no implementation or language-decision change requiring a renewed full review.

On the unchanged candidate, in this worktree with the installed Claude CLI counter, `timeout 650 scripts/count-tokens.sh design/reference.md </dev/null` exited 0 and returned **24,931**. This is **69 tokens below 25,000** and exactly matches `design/reference-companion.md:16-17`. The script was inspected: it invokes Claude and Python, compiling nothing.

`git diff --check ecea71b 1f6ef58` exited 0. A direct Python count of the audit table confirmed 110 rows: 55 implemented, 13 partial, 34 planned, and 8 reworded, matching `reviews/fx-p44q-audit.md:188-197`.

Confidence: high for this narrow documentation re-check. No override. No repository or bead changes; only this requested external report was written. The required subsequent Opus review remains outside this Codex re-check.

Skipped as requested: Cargo, conformance, compiler probes, and a renewed full audit. Runtime behavior was inspected in source, not executed.

DONE
