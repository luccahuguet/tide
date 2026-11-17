Model: gpt-6.1-sol (high) · Date: 2026-11-17 · Candidate 7632f00 (re-check of d1353ee findings)

Verdict: approve

1. **Resolved — direct-example discovery compatibility.** `crates/tide-admit/tests/examples.rs:416`, `:419`, `:439`, `:499`. Only case-study members receive `member = true`; direct examples retain main's direct-entry `.rs` selection without the added directory rejection. The updated regression asserts that both an empty `plain/assets/` and a populated `plain/more/b.rs` leave the direct example's discovered variants unchanged. Group recognition remains limited to the direct `examples/case-studies/` directory. Its empty-group, stray-file, missing-README, missing-variant, and empty/populated deeper-directory rejection cases remain covered. The adjusted documentation matches this distinction. No breakage identified in the fix.

2. **Resolved — stop-cause wording.** `examples/case-studies/eon-slow-close/README.md:86`. The bullet now ends at “OS scheduling under load.” It no longer attributes the slow stops to OS scheduling. Lines 23–26 retain the explanation supported by record 01: slow-to-exit programs incur Orbit's 500 ms grace, accumulated by serial stopping. No claim or model behavior was otherwise changed.

Scope: inspected only `git diff d1353ee 7632f00`, the affected discovery context, main's previous discovery implementation, and the author's response map. The existing `target/work/workspace-r1.log` reports the discovery regression and examples outcome checks passing; this is author evidence, not a reviewer rerun. No additional findings introduced by these fixes were identified.

Not checked: no builds or tests run; no new full review, subagents, or access to `trials/e1-checked/`. Approval is limited to resolution of the two original findings and their fixes' immediate effects.

DONE
