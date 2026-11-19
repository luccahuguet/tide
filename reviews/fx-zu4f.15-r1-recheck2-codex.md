Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate beae640 (re-check 2)

## B1 — Resolved

The amendment makes the minimum correction requested in `target/work/r1-recheck-codex.md`. I read that report first, then the complete `git diff c4fb315 beae640`, RUN's “Round r1 fixes 2” account, tide-lead's brief, and the affected claims in context. This re-check covers only B1 and defects introduced by its fix.

Evidence in `research/NN-fx-zu4f.15-chrono-anchor-retest.md`:

- **§3, lines 76–81 and 93–100:** the registration chronology is attributed to Git history and Tide's bead comment. The ordering bullet identifies the protocol requirement, labels Git times self-reported, and immediately says that the checks are consistent with the chronology but do not independently establish anchoring before landing. The heading is thus qualified where readers first meet it.
- **Lines 101–133:** the impossible-before-mining premise is removed and explicitly corrected: an inclusion path can be computed from a block template before mining. The Git link is limited to the order of the two commits, conditional on the abbreviation not being forged; it says nothing about when block 976536 was mined. The 7-hex-digit/28-bit forgery limitation, lack of ancestry, and absence of absolute dating are retained. No mining-order conclusion remains in this argument.
- **Lines 134–170 and 225–230:** the header, digest and inclusion checks remain, with explorer trust and no full-node consultation disclosed. The header time remains `2026-11-19T05:34:04Z`, compared with the self-reported landing at `06:35:00Z`. Their difference is 3,656 seconds (60 minutes 56 seconds), correctly rounded to 61 minutes. The text states that this is inside the disclosed roughly two-hour tolerance and cannot on its own order anchoring before landing. The positive conclusion is inclusion and file existence by the attestation time.
- **Lines 231–241:** the chronology is expressly a reported chronology consistent with the checks. The summary plainly states that independent proof of anchoring before landing is **not established**, and identifies the reliance on the timestamp within its tolerance and Tide's own records. “No check found the order reversed” reports the absence of a contrary finding; it is not used as proof of the proposed order.
- **§9, lines 670–678:** the former categorical anchoring-before-landing assertion is replaced with the numerical comparison, its tolerance, the reliance on Tide's own records, and the explicit statement that the order is not independently established. Readers encounter the limitation in §9 itself, before the cross-reference to §3.

The ordering claims are now no stronger than the evidence described. The independent mining-before-landing gap is disclosed rather than claimed closed. I found no new defect introduced by this correction.

Verification: `git diff --check c4fb315 beae640` exited 0. The fix diff changes only the record. A plain `python3 -I` comparison against both Git blobs confirmed that the two shell/Python blocks in §3, the header evidence and check list, and §10's protocol reproduction are byte-identical across the amendment. The working record equals the `beae640` blob; its SHA-256 is `3ee4393d86d2bd819350a865d3531396f41aff6b819e7953d1d0b47e82f6a8fe`.

## Verdict

**approve** — B1 is resolved. Confidence is high. No override was applied. This verdict is confined to the requested B1 re-check.

No build, admission, network request, contact, commit or bead write was performed. Only this report was written. The unchanged inclusion verifiers and admission result were not rerun or re-reviewed; independent mining-before-landing ordering remains unproved, as the amended record now discloses.

DONE
