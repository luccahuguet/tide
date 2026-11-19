Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate 8d7181c (S5 re-check)

B1 — RESOLVED.
`docs/at-a-glance.md:63` and `README.md:69` now say “held privately and appears publicly only as a count”. This describes custody, not completed notification, and agrees with `research/52-fx-zu4f.12-crypto-crates-round.md:18-27` (nothing sent or posted) and `research/protocols/fx-zu4f.11-protocol.md:328-330` (no reporting claim before a report actually happens). A Python assertion confirmed the README embeds the canonical at-a-glance text verbatim.

B2 — RESOLVED.
`README.md:163-167` now limits the property to admitted safe code, explicitly excludes the `unsafe` grant, and retains “no accidental ambient authority”, printing and rustc's soundness caveat. `paper/tide-v0.typ:213` excludes declared trust holes, explicitly including `--tide-grant=unsafe`, while retaining its compiler-bug definition of accidental. Its printing/sink exception remains at :135; :215 refers back to the scoped claim and still disclaims unforgeability.

The same safe-code/outside-unsafe-grant scope appears in `docs/owner-guide/01-capabilities.md:210-216`, `docs/owner-guide/03-soundness-bugs.md:176-183`, `docs/owner-guide/answers.md:19-20`, and `docs/tide-or-rust.md:16-25`. The first two retain explicit printing and compiler-soundness qualifications; answers item 3 supplies the compiler caveat, and tide-or-rust retains printing and links the precise claim to answers. These agree with README's existing trust-hole warning (:532-534), RUN-4/UNSAFE-1 (`design/03-authority-concurrency.md:1028-1048`), and the granted assembly/foreign-code case (`conformance/cases/authority/unsafe_granted.rs`). The previously identified granted-code counterexample is now expressly outside the claim. No conflicting claim introduced by these edits was found.

New blocking findings: none.

Verdict: approve. B1 and B2 are resolved; confidence high within this bounded finder re-check. This is not the subsequent Opus verdict.

Proof and limits: inspected the seven-file `git diff 93d33f5 8d7181c`, surrounding texts and cited evidence, and the S5 r1 account in `target/work/report.md`. HEAD was `8d7181c8cba04b436346dccc74a7bfb37edda892`; tracked worktree clean. `git diff --check 93d33f5 8d7181c` exited 0. Commands used only git, rg and python3 -I in the supplied worktree. No builds, rendering, runtime checks or full-review repeat; no implementation or ledger changes, commits or contacts. Only this report was written.

DONE
