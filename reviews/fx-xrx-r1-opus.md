# Tide review, fx-xrx round 1 (Opus): docs/design review round

Candidate: commit a9af3ac on branch `fx-xrx` (base d12d7e0). `git diff d12d7e0 a9af3ac` has SHA-256
`e44b5857131bf36493fa705df3ef9053cc3fecadc665838c9eba6307b4e76f20`, byte-identical to Codex's frozen
`.codex-fx-xrx/implementation-candidate.patch`.

Model: claude-opus-5-5 · Date: 2026-11-07

## Verdict

Clean: no blockers. Three nits and one out-of-scope observation, none blocking.

- **Faithful to the decision.** Every instruction in the Opus decision (bead comment, `DECISION.md`) is applied. The edits are 01 D7, 03 §2.5 / UNSAFE-1 / RUN-2 / RUN-4, `rules.md` UNSAFE-1 (both cells), reference 2.1's three edits and the §11 row trim, the companion count, and the two extended cases. The cases match the decision's text verbatim. `crates/` is untouched. The diff touches exactly the seven prescribed files.
- **Acceptance met.** The design text and `policy.rs:88` (`granted("core::arch::naked_asm")`, ahead of the `core::arch::*` deny) agree. Each side has a case: `unsafe_global_asm.rs` covers the ungranted side and `unsafe_granted.rs` the granted side. Codex's proof ran on this exact diff: `cargo test --workspace` exit 0 with 38 passed, and `conformance/run.sh` exit 0 with 218 cases (194 PASS, 24 AWAITS CHECKER, 0 FAIL), both assembly cases PASS. A PASS means the driver emits exactly the marked FX codes: `FX0901` on `#[unsafe(naked)]` and `FX0520` on `naked_asm!` ungranted, nothing granted. Per AP-PROOF-001 I did not rerun the suites on the unchanged candidate. The shared build lock was also queued behind 45-minute jobs.
- **Driver behaviour matches the text.** `walk.rs:1120-1131` routes `AttributeKind::Naked` through `granted("an unsafe attribute")`. Codex's `trust-artifact.json` records both holes on the item ("an unsafe attribute", "uses `core::arch::naked_asm`"). That matches 03 UNSAFE-1's "all are trust holes" and RUN-4's "each appears in the trust diff as a hole".
- **Budget.** I ran `scripts/count-tokens.sh design/reference.md` in the worktree: **24,994** (exit 0). This is under 25,000 and equals `reference-companion.md:14`.
- **The decision is sound.** Admission adds no authority a granted package lacks. `global_asm!` plus an `unsafe extern` declaration already places the same code, and a safe naked fn is no different from any safe wrapper over granted `unsafe`, which RUN-5 already gates. Denial would leave `#[unsafe(naked)]` admitted but unusable, since rustc requires a `naked_asm!` body. It would also need a special case to reconcile the `core::arch::*` deny's `FX0520` with 2.1's `FX0103` catch-all. Naming `#[unsafe(naked)]` completes UNSAFE-1's list of Rust 2024 unsafe attributes. The text promises nothing beyond admission and hole reporting, and leaves ABI soundness to the author. Assembler file reads remain under RUN-2's sandbox row, which now names `naked_asm!`.
- **Sweep.** No remaining text lists the asm macros without `naked_asm!`. I grepped `design/`, `conformance/`, `crates/`, `README.md`, and `trials/`. `01:843-844` ("unsafe grants (unsafe attributes and `global_asm!` included)") stays correct: it is non-exclusive, and `naked_asm!` cannot appear without the `#[unsafe(naked)]` unsafe attribute. Reference 6.9's "hardware intrinsics (`core::arch`)" is pre-existing, and 2.1, being more specific, overrides it, as fx-k0z r2 already ruled.

## Findings

No blockers.

N1 (text-only, nit). `design/03-authority-concurrency.md:712-713`: the UNSAFE-1 attribute list is not rewrapped. Line 713 is just `` `#[unsafe(naked)]`), and `` (29 columns) inside a paragraph that runs at about 68 to 76 columns. Fix: reflow lines 711-715 to the paragraph width. Line 715's 126-column overflow is pre-existing and could be reflowed in the same pass.

N2 (text-only, nit). `design/01-design.md:430`: the line is now 81 columns, while D7's bullet runs at 66 to 77. Fix: move `` `global_asm!`, `` to line 431 and rewrap. The decision asked for "rewrap at each file's existing width".

N3 (text-only, nit). `design/reference.md:66-67`: the trimmed remedy drops "embedded data". The old text pointed a reader of the same paragraph's `include_str!` `FX0520` to "run a generator program and commit its `.rs` output" for generated code *or embedded data*. The new text, "commit a generator's `.rs` output", follows the `FX0101` colon and so reads as a remedy for macros only. The guidance survives in companion 14 ("`include_str!` ... → a generated `const`") and through the reference's `:1051` pointer "generators: 2.1", so nothing is lost outright. Optional fix, only if it costs no tokens: "(test tables), or commit a generator's `.rs` output (generated code, data)". The fix must be recounted under 25,000; at 6 tokens of headroom, leaving it is reasonable.

Out of scope (pre-existing, report only). `conformance/rules.md`'s `03:NNN` line citations are already stale at base: UNSAFE-1's `03:606` points into the RUN-1 paragraph, and the definition is at about 03:705. Several later citations (`03:727`, `03:735`, `03:762`) also point to unrelated lines. The one line this change inserts at 03:713 shifts every later citation by one more. This change did not cause the drift. It is a candidate follow-up bead (refresh or drop line numbers in the Section column), not a fix for this bead.

fx-djp comments: none.
