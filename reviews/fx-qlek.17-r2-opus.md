Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 3ed545a (research record)

# fx-qlek.17 Opus review round (docs/research), on the Codex-clean tree

## Verdict

**Changes required.** One blocking finding: the Cashio (C03) fixed-side verdict does not follow R1 as registered. Every other verdict, the denominators, the disclosure and the claim discipline hold up against the primary sources. The fix is small and changes counts only: C03's pair becomes incomplete, which leaves two paired supports, not three.

Confidence: high on B1's mechanism, which I checked line by line against the saved patch and action source. Medium on which repair the author picks (reclassify, or do the R1.6 analysis).

## Scope

- Read: AGENTS.md, GOVERNANCE.md, design/01-design.md (D7, D18 headings, tier context), design/reference.md §6.1, docs/driver.md, `br --no-db show/comments fx-qlek.17`, the registered protocol (§1–§6, A1), cases.tsv, all of research/43 (lines 1–998), and research/protocols/fx-qlek.17-assessment-sources.json (structure, results, revision_pairs, gap note).
- Primary sources checked in target/work/sources/:
  - C01: Solend fix JSON plus the fixed handler.
  - C02: the Wormhole buggy/fixed `verify_signature.rs` diff, the SDK v1.7.0 and v1.9.4 loaders, and the commit file list.
  - C03: the Cashio fix JSON (all hunks), the fixed `lib.rs`, `actions/mod.rs`, `actions/print_cash.rs`, and Sec3's corrected article.
  - C04: the Halborn and CertiK texts.
  - C05: the disclosure quote.
  - C01: the report text (the theft-attempt and refund wording).
- Not repeated: Codex r1's fidelity checks, and its PDA/SBF/CPI first-use note.
- Not done: no builds, no network, no edits other than this file.

## Launch sentence (item 3)

The record supports one honest sentence. This is the form that holds after B1 is fixed:

> In a small pre-registered paper exercise (not a Tide feature; nothing was compiled or run), a proposed Solana account-capability rule, given as declared policy the account check that each later public fix added, rejected the vulnerable code and admitted the fixed code for two of five public, already-fixed Rust exploits (Solend 2021, Wormhole 2022); Cashio's fix could be judged on its vulnerable side only, two cases had no public code, and Anchor and the Solana SDK can already express the same checks.

Shorter form for the launch post:

> We also ran a small pre-registered paper study of a proposed account-capability rule for Solana programs. It is research toward a direction, not a Tide feature, and its main lesson is that signer, owner and PDA checks are not enough without checked account relationships.

If tide-lead rebuts B1 and keeps C03 as support, replace "two of five" with "three of five, one of them a patch that disables its entry points".

Launch drafts must not say, or imply, any of these:
- that Tide "catches", "would have caught" or "would have prevented" these exploits;
- a catch rate, or "N of 5";
- any amount of money saved;
- that the rule is part of Tide v0, of D7's capability set, or of the alpha.

## Blocking findings

### B1. C03 Cashio: "admits fixed shape" rests on runtime reachability, not on admission of the fixed code

**Where:**
- `research/43-solana-account-capabilities.md:548`, `:553-558`, `:726`, `:737-742`, `:761-765`
- headline `:5-7`
- `research/protocols/fx-qlek.17-assessment-sources.json:289-294` (`results`)

**Evidence:**
- The fixed patch `7df6581` puts `vipers::invariant!(false, "temporarily disabled");` in front of `actions::print_cash::print_cash(ctx, deposit_amount)` and `actions::burn_cash::burn_cash(...)` (fixed `programs/brrr/src/lib.rs` 40–41, 51–52). That guard returns an error at runtime. The action bodies after it are still compiled, and they are still application code that admission checks.
- R1 is a static admission rule.
  - Protocol §3: "Admission forbids bypassing those constructors and operations in application code".
  - The trigger is a handler "trying to … invoke another program".
  - The record's own §2.2 (`:267-270`) says static admission checks that the program "cannot skip those constructors on a protected operation".
- The body of the fixed `print_cash` action (`actions/print_cash.rs`, the same in both revisions) does three things:
  1. a Token `transfer` CPI that writes `crate_collateral_tokens` (lines 29–39);
  2. then the `crate_token::cpi::issue` CPI (42–60);
  3. then a read of `self.common.crate_collateral_tokens.mint` from the copy that Anchor deserialized before both CPIs (line 64).
- R1.6 says a CPI that may change checked data invalidates the proofs that depend on it, which must be reloaded before reuse. It also says: "Any fixed shape needing an unsupported lifecycle remains unassessable." The record states that no such model was built (`:765`, "No claim of a working re-enabled CPI/lifecycle model is made"). It sets R1.6 aside on the ground that "the exact fixed entrypoint aborts before both CPIs" (`:762-764`, `:726`). That is a runtime-path argument. It is not admission of the fixed code.
- The runtime-path reading also makes the fixed verdict vacuous. On that path the code runs `validate()` and then aborts, so no capability is ever needed. A patch that only added the disable, without the `bank.crate_mint == crate_mint` check, would get the same "admits fixed shape". The fixed verdict therefore does not depend on the fix it is supposed to credit, and §5's "the source's actual fixed checks supply every capability needed" is met by default, not by those checks.

**Consequence:**
- The aggregate "paired support 3" and the headline "three conditional paper pairs support the proposed rule" count a pair whose fixed side was not tested under R1.
- This is a verdict that breaks the registered rule.
- The buggy-side rejection is unaffected: the missing relationship still blocks the issue permit.

**Minimum fix.** Either option is acceptable.

- **(a) Preferred, the smallest change.**
  - Reclassify C03 as Buggy `rejects buggy shape` / Fixed `not assessable`. Reason: under R1.6, the admitted action bodies need a CPI/reload lifecycle that this pass did not model, and a runtime disable cannot stand in for admission of the compiled code.
  - Keep the buggy rejection as a supported one-sided observation.
  - Recount everywhere: frozen 5, examined 5, pair-assessable 2, incomplete 3, paired support 2, assessable negatives 0. Buggy: reject 3 / admit 0 / NA 2. Fixed: admit 2 / reject 0 / NA 3.
  - Update the headline (`:5-7`), the table row (`:548`), the counts paragraph (`:553-558`), the C03 verdict paragraph (`:737-742`) and lines `:761-765`, plus `results` in the assessment JSON.
  - Record the review-driven correction in §4.5. It is a correction applying R1, not a rule change; no other conclusion changes.
- **(b)** Do the static R1.6 analysis of the fixed `print_cash` and `burn_cash` bodies, and report whatever it yields. The analysis must cover:
  - which proofs the transfer CPI invalidates;
  - whether the issue permit's facts survive it;
  - the read at line 64 after the CPIs.

  If that read needs a reload the source does not perform, the honest outcome is `rejects fixed shape` (an assessable negative) or `not assessable`, not admission.

## Non-blocking findings

1. **The record never states its relation to Tide's capability design.**
   - What the record says is correct: v0 implements none of this (`:10-11`); policy.rs names Tide's capability types and has no signer, owner or PDA operations (`:410-412`).
   - What is missing: it never says how R1's `Signer`/`Owned<T>`/`Pda<S>` relate to D7 or reference §6.1. Skeptical readers will assume these are an extension of D7.
   - Suggested sentence for §2: "R1's values are hypothetical and are not D7 capabilities: D7 (reference §6.1) governs ambient host authority (`Fs`, `Net`, `Clock`, …) handed to functions, while R1's values would be runtime-validated, invocation-bound facts about caller-supplied accounts; they share only the discipline that authority arrives as parameters."
   - Nothing in the record implies a v0 decision. §2.4 and §4.4 are explicit about that.
2. **Say plainly that the support is circular.**
   - In C01, and in C03 if it stays, the relationship the "declared policy" requires is taken from the later fix. The rejection means: had the author declared the check they omitted, admission would have forced it at the operation.
   - The record says this in pieces (`:9`, `:540`, `:870-874`). The headline would be stronger with one clause such as "given, as declared policy, the relationship each fix added".
   - C02 is different, and the record could say so. The Instructions-sysvar address requirement belongs to the trusted operation library, not to an application recipe. That makes Wormhole the case least dependent on hindsight policy. It is also already covered by the SDK's `*_checked` loaders.
3. **One known gap is missing from the record.** The manifest's `known_discovery_log_gap` (the C01 post-go search output was not saved; JSON `:1320`) is absent from §4.5's list of access limits. Add one clause.
4. **Attribution.** The record header says "Author: Codex", and the bead says the Codex builder wrote it. Commit 3ed545a's trailer instead credits `Claude Opus 5.5` as co-author. Correct it on the amend that fixes B1.
5. **§2.4 "Choices for Opus" lands undecided.** AGENTS.md says Opus decides these choices. Given recommendation 9 (no implementation), deferring is reasonable. At landing, tide-lead should record on the bead which disposition was taken, or state that choices 1–8 are deferred and not adopted, so the published record does not read as awaiting a decision forever.
6. **Wording precision (`:403`).** "rejects raw rustc flags and user proc macros": docs/driver.md says it rejects user macro definitions (`TD0101`) and derive or bang forms outside the closed sets (`TD0102`/`TD0103`). Anchor's `#[program]` and `#[derive(Accounts)]` fall under the closed-set rules. Use the driver's own wording.

## Reasoning

- **C01 Solend.**
  - The fix `132d74c` (parent `871935c`) inserts exactly four lines comparing `reserve.lending_market` with the supplied market key. The owner checks, the market-owner match, the signer check and the PDA derivation are already present in both revisions.
  - The report confirms the attacker used "the newly created Lending Market to subvert an auth check", and that "no funds were stolen", with five users refunded.
  - The R1.5 derivation is faithful. It adds no repair, and there is no CPI, so R1.6 is not engaged. The verdicts follow R1 and §5.
- **C02 Wormhole.**
  - The buggy `load_current_index`/`load_instruction_at` take raw bytes taken from `try_borrow_mut_data()` of an unchecked `Info`. v1.9.4's `*_checked` variants call `check_id` and return `UnsupportedSysvar`.
  - The fix touches only those two call sites in this file. The other hunks (Cargo, lock files, Docker files) are disclosed as outside the modeled path.
  - The account-creation CPI and the re-check at persistence are handled under R1.6. The verdicts follow R1.
- **C03 Cashio.**
  - The root-cause account matches Sec3's corrected analysis. The fake bank is the root of trust; the patch's `bank.crate_mint == crate_mint` is sufficient through bankman's PDA invariant; the bank curator remains trusted.
  - The record correctly declines Halborn's simpler arrow-mint story (`collateral.mint == arrow.mint` is already checked).
  - The buggy rejection is sound. The fixed admission is not (B1).
  - The record is honest that the patch is disabled. Being honest about that does not make the verdict meet R1.
- **C04 Crema and C05 SolSea.** Both "not assessable" verdicts follow from §4 ("no public code … is not assessable"), on primary statements:
  - CertiK: "the exploited project's source code is private".
  - SolShield: "program source code was not available to us".

  Neither is used to dodge a hard case. The record also declines to substitute an illustrative `Owned<Tick>` model or the SPL Token repair, as §4 requires.
- **Claims and tone.**
  - The record says, more than once, that no result is a Tide v0 catch. It names the prior art fairly: Neodyme's checked-type pattern, Anchor's constraints, Move, VRust, Ackee and Sec3. It credits the existing SDK and Anchor with the decisive checks, and denies catch rates, prevalence, prevented loss and superiority.
  - All five incidents are public and fixed or disclosed. Nothing describes a live, undisclosed weakness. Projects and authors are not judged.
  - Terms: Codex r1 already covers PDA, SBF and CPI.
- **Consistency.** The protocol, the cases.tsv row order, the assessment JSON's `results`/`revision_pairs` (including C03 `fixed_path_enabled: false`), the bead comments and the commit message agree with each other. They will need the B1 recount together.

DONE
