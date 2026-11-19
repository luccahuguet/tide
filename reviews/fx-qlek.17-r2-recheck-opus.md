Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate deaf434 (re-check)

# fx-qlek.17 Opus finder re-check of B1 and of non-blocking items 1–6

Scope: `git diff 3ed545a deaf434`, target/work/fixes.md, and a scripted recount of the record's table and of the JSON. The protocol and the cases/ files are byte-unchanged between 3ed545a and deaf434 (`git diff --quiet` passes), and `git diff --check e245199 deaf434` passes.

## B1, Cashio fixed verdict: resolved

The reclassification is exactly option (a).

- **Table:** row C03 (`research/43:566`) now reads `rejects buggy shape | not assessable`. The reason given is the R1.6 CPI/reload lifecycle that the compiled fixed action bodies would need.
- **Verdict paragraph:** `:753-756` reads "Fixed: not assessable. Pair: not assessable."
- **Supporting paragraph:** `:775-779` now says that the runtime disable leaves both action bodies compiled and subject to static admission. It names the transfer CPI and the read from the pre-CPI copy after both CPIs, and it says R1.6 requires invalidation and reload. This matches the evidence I gave.
- **Buggy side:** the rejection is kept as a one-sided result (`:579`, `:754`).
- **Recount by script.** Rows parsed from the table: buggy 3 reject / 2 not assessable; fixed 2 admit / 3 not assessable; pair-assessable 2; paired support 2.
  - The prose at `:571-576` gives the same numbers: frozen 5, examined 5, pair-assessable 2, incomplete 3, paired support 2, negatives 0.
  - So does JSON `results`, which also sets `support_disabled_fixed_entrypoints` to 0.
  - The headline (`:5-8`) and the commit message agree.
  - No remaining text claims three supports or a disabled-path support. The `§4.4` mention of Cashio is about its existing checks, which is still true.
- **§4.5 (`:923-927`):** it records the correction as applying the registered R1.6, and states that "R1 and the registered case list are unchanged". The JSON `deviations` entry says the same. This is an application of the rule, not a rule change.
- **History:** the JSON keeps the initial verdict as `review_correction` (`initial_fixed: admits fixed shape`).
- **Nit, does not block:** the C03 `paper_assessment_recorded` event keeps its original 00:03:38Z timestamp but now carries the corrected verdict fields. The initial verdict sits only in the nested `review_correction`. Optionally, put the correction date beside the fields. This is not misleading, because `review_correction` is explicit.

## Non-blocking items

1. **R1 vs D7 and reference §6.1: resolved.**
   - §2.1 (`:226-232`) separates D7's host-authority values (`Fs`, `Net`, `Clock`) from R1's *hypothetical* `Signer`/`Owned<T>`/`Pda<S>`. It describes the latter as runtime-validated facts about caller-supplied accounts for one invocation, and says the two share only the discipline of passing authority as parameters.
   - Both anchors match their headings: `#d7-authority-capabilities-as-values-no-ambient-authority` and `#61-the-rule-and-the-capability-set`.
   - Nothing implies that R1 is part of D7 or of v0.
2. **Conditional support and C02's library requirement: resolved.**
   - The headline conditions support on "declaring as policy the account relationship each later fix added".
   - It says that for C02 the trusted operation library owns the exact-address requirement, which the SDK's `*_checked` loaders already enforce.
   - The C02 verdict paragraph (`:684-691`) places the requirement in the trusted instruction-history operation. Credit to the existing SDK is kept in the headline and in §4.4.
   - Nothing is overstated.
3. **C01 discovery-log gap: resolved.** §4.5 (`:915-917`) discloses that the initial post-go C01 search output was not saved, and names the evidence that was retained.
4. **Attribution: resolved.** The deaf434 trailer credits Codex (GPT-6.1-Sol and GPT-6.2-Sol). It matches "Author: Codex".
5. **§2.4 disposition: resolved.**
   - The section is retitled "Choices and disposition", and the record states that tide-lead adopted choice 9 on 2026-11-19, alpha unchanged. Choices 1–8 are recorded as unadopted recommendations.
   - §4.4 (`:890-893`) refers back to it, and says that any SBF/adapter experiment needs separate authorization.
   - This matches tide-lead's decision as relayed.
6. **Driver wording: resolved.** §3.2 (`:418-421`) now uses the wording of docs/driver.md: user macro definitions (`TD0101`), derives and bang forms outside the closed sets (`TD0102`, `TD0103`), and `#![no_std]`.

Codex r1's item, first-use definitions: **resolved.** The first uses are PDA at `:13-14` ("program-derived address (PDA)"), SBF at `:16` ("SBF (Solana's bytecode format)") and CPI at `:259` ("cross-program invocation (CPI)"), and none of the three appears earlier.

## Launch sentence

It still matches the record. "Two of five (Solend 2021, Wormhole 2022)" and "Cashio's fix could be judged on its vulnerable side only" agree with the headline and the table. The clause "given as declared policy the account check that each later public fix added" agrees with the headline's condition. Use it unchanged; the "three of five" fallback no longer applies.

No new problem was introduced by the fixes beyond the chronology nit above.

## Verdict

**Approve.**

DONE
