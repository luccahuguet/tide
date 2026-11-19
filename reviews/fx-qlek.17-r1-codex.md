Model: gpt-6.1-sol (high) · Date: 2026-11-19 UTC · Candidate 3ed545a (research record)

## Verdict

approve

No blocking findings. Confidence: high in protocol fidelity and the bounded paper comparisons; this is not independent proof of deployed behavior or implemented enforcement. Opus review and the coordinator's landing decision remain pending.

## Scope

Read-only round 1 for fx-qlek.17. Reviewed the candidate diff and research/43-solana-account-capabilities.md, the assessment manifest, the frozen protocol and case/selection ledgers at 384d775, GOVERNANCE.md, bead history, target/work/report.md, and saved source material. Checked relevant current driver documentation and interfaces for the claim that this is not a Tide v0 capability.

HEAD is 3ed545a8c096bc7d33094193460f6efac2949592. Its diff from e245199 has SHA-256 92b6c9b262f101c468ea5e2b3d7a265d04c677ed371f2dd083d9b4acceaa4cb6, matching the builder report. That diff changes only the research article and separate assessment manifest. No implementation or language decision is included.

Used local saved evidence, Git and plain Python inspection; fetched nothing, contacted nobody, executed no upstream code and ran no build. The only review write is this report. Existing untracked .ots.bak files were left untouched.

## Blocking findings

None.

## Non-blocking findings

1. research/43-solana-account-capabilities.md:8–11, 247: expand PDA (program-derived address), SBF (Solana's bytecode format/VM target) and CPI (cross-program invocation) at their first uses. The capability predicates are explained, but these acronyms are assumed. This is a readability nit; it does not change a rule, source map or verdict.

## Reasoning

### Protocol and denominators

The protocol, cases.tsv, entries.tsv, selection.tsv and registered sources.json remain byte-identical to 384d775. Independently checked all six registered subject hashes against Git, all six current proof hashes, and their embedded subject digests. This establishes local identity, not full OpenTimestamps/Bitcoin verification. The article correctly attributes block anchoring to tide-lead and distinguishes the stamped draft from the final article (lines 498–533). Pre-go exposure remains explicitly self-attested and non-blind.

The five cases appear in registered order: Solend, Wormhole, Cashio, Crema, SolSea. The manifest's ten start/result events agree with that order and occur after the recorded go. No case replacement or post-go R1/selection change is evident. All side verdicts use §5 vocabulary. The table and manifest agree on frozen/examined 5, pair-assessable 3, incomplete 2, paired support 3 and assessable negatives 0; each side has three assessed verdicts and two unknowns. The enabled/disabled split is visible in both the opening and aggregate result. Unknowns are not counted as successes or negatives.

### Source checks and the three conditional pairs

Spot-checked the following identities, URLs, revisions, quotations and predicates against saved source bodies/envelopes, rather than relying on the builder's summaries:

- **C01 Solend** (article lines 574–625): Rooter/Nope's report supports the quoted liquidation symptom, attacker-created market, thwarted theft and refunded liquidations. Saved commit 132d74cf171dac66f896c4009ad6836f8c0b3799 has first parent 871935cafc590bf1985bebfbc7e6399b153a5b40 and only the four-line reserve.lending_market comparison. Read both processor handlers and the relevant state helpers. The original already checks native owners, market authority/signature and stored-bump PDA. R1.5's missing reserve-to-market relationship is the credited predicate; the translation does not insert it into the buggy validator. The intended reserve-specific authorization policy is stated as trusted input, and enforcement is expressly conditional.
- **C02 Wormhole** (lines 636–685): the Halborn narrative supports the quoted signature-bypass symptom and forged Instructions account. Commit 7edbbd3677ee6ca681be8722a607bc576a3912c8 has first parent 79ab522f802ccc5ba34278d3c648fa62e06f4f1c. Read both verify_signature.rs versions, SDK v1.7.0/v1.9.4 instruction loaders and Solitaire account/peel/persistence helpers. The checked SDK functions actually test the exact sysvar key and return an error on mismatch; the old byte APIs cannot authenticate it. The map credits R1.4, preserves the remaining signature checks and account-creation branch, and acknowledges unmodeled parsing/cryptography/global lifecycle. Unrelated commit hunks are disclosed. It does not turn a constructor defect inside the trusted boundary into an admission catch.
- **C03 Cashio** (lines 698–773): read the corrected Sec3 analysis, both validators and entrypoint files, print_cash, bankman creation code, pinned Anchor wrappers, addresses/Saber helpers, lockfile and Vipers 2.0.1 macro. Commit 7df658184c2610139fa2c0058363c66b28add4c4 has first parent a51c3c59d544a5763b64abb4a8d82c49b0abd6d0. Its actual relation is bank.crate_mint == crate_mint; the old validator already checks collateral.mint == arrow.mint. The article preserves this disagreement with Halborn and Sec3's published correction. Both unconditional entrypoint failures are real error returns, not debug assertions. Counting the exact disabled patch as an admitted fixed shape is consistent with §5's shape criterion: that criterion requires no enabled-service outcome. The record repeatedly disclaims a working repaired mint and separates this pair from the two enabled comparisons. It adds no extra bank/crate comparison or hypothetical re-enablement. The buggy rejection is conditional on the declared R1.5 operation policy, not automatic recovery of that policy.

Also independently checked hashes/sizes for 66 manifest artifact records, including all 46 additional request artifacts and the associated checked metadata/inherited bodies/derived images. Saved request-envelope URLs agree with the manifest. Decoded contents for the eight main buggy/fixed source files agree with their readable local copies. The failed Wormhole lookup and obsolete Cashio URL remain failures, not positive evidence. The short case quotations checked above agree with the saved narratives; the Crema and SolSea quotations agree with their respective saved narrative/extraction, subject to the latter's explicit transport limitation.

### The two not-assessable cases

**C04 Crema** (lines 792–822): Halborn supplies a tick-spoofing narrative, CertiK explicitly reports private source and uncertain function attribution, and Rekt distinguishes the prior swap repair from this Claim incident. Read those narratives and inspected the saved same-incident search and six-repository result. None supplies the matched historical Claim code pair. §4 requires more than a cause label; both unknown verdicts are justified. The record does not assert that its bounded search proves universal unavailability or that ownership alone would settle the case.

**C05 SolSea** (lines 830–860): the saved registered disclosure extraction states both source unavailability and the distinction between fee-payer and listing-owner signatures. It also reports that the issues were fixed. The saved follow-up search supplies no SolSea Rust handler/fix. Public SPL Token code is a different program and cannot complete this pair. Retaining both unknowns follows §4; inventing a signer comparison would violate it. The extraction is labeled as such under A1, not represented as original HTML or independently verified source code.

### Claims and disclosure

The conditional comparisons assume correct trusted operations and declared policy; they do not demonstrate how to choose that policy, confine every alias, implement lifecycle invalidation or compile for SBF. Those limits are explicit at lines 537–542, in each case and at 866–890. Existing Anchor/SDK checks receive credit. No prevented-loss, discovery, catch-rate, superiority or Tide v0 enforcement claim is made, and no judgment of project authors is introduced. The disposition remains research and a recommendation, not adoption or authorization for implementation.

The committed analysis concerns the registered public historical incidents/disclosure. I found no new claim identifying an undisclosed or presently unfixed live-program weakness. Solend's report records deployment of its fix; the Wormhole and Cashio comparisons use public historical repairs/mitigation; the SolSea disclosure explicitly reports remediation. Crema's exact historical repair remains unverified and is not modeled as established. This review did not independently verify present deployments or remediation status; no such verification is implied by approval.

No alternative design or patch is required for approval. Preserve the source snapshots as the record requires: hashes alone cannot recover them. Full Bitcoin anchoring verification, original Medium HTML, deployed-byte identity, runtime enforcement and empirical benefit were not checked or established here.

DONE
