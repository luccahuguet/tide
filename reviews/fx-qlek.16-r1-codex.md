Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidates Tide d84d22a, site d7ced61 (code)

## Verdict

Changes required. The implementation direction is sound. The remaining blockers concern evidence labels, source parsing, and explicit editorial acceptance requirements. Confidence: high on the current-source findings, moderate on runtime behavior because this pass ran no build.

## Scope

Read-only review of Tide `04dde61..d84d22a` and site `9aefc6e..d7ced61`, including their instructions, GOVERNANCE.md, fx-qlek.16's scope, acceptance and comments, fx-qlek.7's qualifier and owner rules, the builder report and terms pass, source records, generator, renderer, falsifier and tests. No implementation, ledger or history changes. Only this report was written.

Frozen diff SHA-256s, from `git diff BASE..HEAD`:

- Tide: `e363e149e21aa47b1ceb60962950190ded71a1f0c71c9beada90b094073368ec`.
- Site: `e73cf50e0d6af77add530e4bbb0f1e290aea312665118482b2d20924f3de2553`.

## Blocking findings

### B1. The first search's denominator is relabeled and its second pool disappears

**Locations:** site:scripts/status-data.ts:255, site:src/pages/status.astro:217. Source: Tide:research/37-external-admission-search.md:53.

The row titled “RustSec malicious-crate advisories, and GitHub issues” displays **87 screened / 12 eligible / 0 caught**. The source distinguishes **1,507 advisories screened**, **87 qualifying/ examined candidates**, and **12 with quoted code ported or assessed**. Its GitHub pool separately has **60 unique results**, all excluded before assessment, and **0 examined candidates**. The generator takes only the first numeric column of “Examined candidates” and calls it `screened`. Readers therefore lose the actual screening denominator and every count for the GitHub half of the titled search. The shared “Eligible” heading also hides that 12 is a source-availability/assessment category rather than the source's qualifying-candidate count.

**Minimum fix:** preserve the stages and units for this round. Show the advisory screen, qualified/assessed stages and GitHub pool separately, with labels matching the record. Separate rows or a compact breakdown in this row are sufficient. Extract those values from the source and add one check against its distinct stage counts, rather than checking only the renderer against the already-mislabeled JSON.

### B2. The first-use terms check does not satisfy the owner rule

**Locations:** site:src/pages/status.astro:104, site:src/pages/status.astro:126, site:src/pages/status.astro:136, site:src/pages/status.astro:234; Tide:docs/at-a-glance.md:3. Review artifact: Tide:target/work/terms-pass.md.

The status page introduces the “core reference” in its capability section but defines it only in the final tile. Its capability cells introduce “ambient authority” and “companion 20” before the companion explanation. “Seeded simulator” never explains what a seed is on this surface. The tables introduce TCP, DNS, UDP, TLS, HTTP, RST, CPU and PTYs without plain explanations. “Self-attested” and “OpenTimestamps-anchored” are first used in the search table and defined below it. The at-a-glance introduction uses “AI” without expanding it. These are explicit first-use requirements, not suggestions about technical writing. The terms artifact's note 5 acknowledges definitions after first use while declaring no missing terms.

**Minimum fix:** put the necessary explanations before or at each first occurrence, including the table vocabulary, or remove vocabulary the page does not need. Move the existing reference, companion and registration definitions earlier. Expand AI at first use in the shared glance source. Update the terms artifact with actual reading-order locations and remaining gaps. Quoted cells and headings still count under the owner's rule.

### B3. Source format changes can silently move planned behavior into Shipped

**Locations:** site:scripts/status-data.ts:55, site:scripts/status-data.ts:178.

The table reader discards its header and separator without validating either, and the capability reader assigns columns by position. Swapping “Main methods” and “Attenuation” in the source table silently publishes attenuation text as shipped methods. Changing `planned: ` to another understandable marker, such as `planned ` or the reference's previous `(planned; ...)` form, leaves the planned method in `shipped` and silently defaults `planned` to an empty string. Missing trailing cells are also accepted through TypeScript non-null assertions, which perform no runtime validation. Thus a format change can produce a successful export containing a false shipped claim, contrary to the requested fail-closed behavior.

**Minimum fix:** validate the expected headers, separator and cell counts for the consumed tables. Reject unrecognized planned markers or ambiguous splits instead of treating them as shipped text. Add small negative checks for a reordered capability table and an unrecognized planned marker. The browser tests currently compare the page with the same generated JSON and cannot detect either failure.

### B4. Registration provenance is inferred from a filename, and its inputs are not hashed

**Locations:** site:scripts/status-data.ts:39, site:scripts/status-data.ts:248.

`qualifier()` calls a round anchored whenever `git cat-file -e` finds its `.ots` path. It reads neither the protocol nor the proof. A pending calendar receipt, a proof for an earlier protocol, or even an empty file receives the same “timestamp anchored in a public blockchain” claim. This distinction exists in this repository's history: research/46 records pending proofs at `db130d0` and completion only at `584b291`. File existence is therefore not evidence of completion. Neither protocol nor proof appears among the export's 15 input SHA-256s. Git failures in the existence probe also collapse into the ordinary self-attested classification.

The labels at this candidate match the published records, but the new generator does not establish the condition its label asserts or record all inputs to that decision.

**Minimum fix:** derive the classification from explicit registration/completion evidence in the pinned record, with the named protocol and proof checked for the identities relied upon, or use an existing proof verifier. Read and hash those input bytes. Fail on missing, pending, mismatched or unreadable evidence rather than asserting an anchor from a path. A pending-proof negative check is the cheapest useful regression test. This does not require introducing a new timestamp service or writing a general cryptographic verifier.

### B5. The new status tables render semicolons in prose

**Locations:** site:src/components/Inline.astro:7, site:src/pages/status.astro:117, site:src/pages/status.astro:147, site:src/pages/status.astro:167. Rule: site:AGENTS.md:7.

The renderer preserves prose semicolons from the records. Current examples include Proc's “a handle; the child has ambient authority”, its narrowing sentence, the Tasks shipped cell, two simulator cells, and the atomics sentence. These are outside code spans. The site rule exempts code syntax, not quoted prose. Checking only the glance Markdown at generator line 97 misses all six affected generated strings.

**Minimum fix:** adapt punctuation in displayed prose while keeping the source facts and links intact. Preserve actual code syntax. Add one rendered-prose assertion covering the new status page, excluding code elements, so copied records cannot reintroduce this violation. The builder report's proposed quotation exemption is not in the site instructions.

### B6. Some quantitative claims bypass generation or their required source links

**Locations:** site:src/pages/status.astro:205, site:src/pages/status.astro:61, site:tests/site-smoke.spec.ts:213; Tide:docs/at-a-glance.md:40, Tide:docs/at-a-glance.md:46.

The confidence level `95%` is typed directly into the status template rather than exported, and is not linked. The conformance exit status is generated but printed outside its source link. In the shared glance text, the 25,000-token budget and three closed soundness routes have no links to their evidence. A general status-page link at the end does not identify the source for either claim. The test checks only elements voluntarily tagged `data-number`, so all of these omissions pass it.

**Minimum fix:** extract the confidence level from the published E3 result sentence or its owning record and render it with a source link. Link the exit status and the glance's quantitative claims directly to their owning records at the intended revision. Extend the check to cover these specific omissions rather than asserting that every already-tagged link is a link.

## Non-blocking findings

1. **Fs's section link goes to the capability table instead of the named section.** site:src/pages/status.astro:117 renders “reference §6.3” but uses `capabilities.anchor`, which is `61-the-rule-and-the-capability-set`. Point the numeric section-reference case to its actual heading. This does not change the shipped/planned classification.
2. **The pin identity has no assertion at the rendering/test boundary.** site:src/pages/status.astro:5 and site:tests/site-smoke.spec.ts:210 take facts from `status.json` but construct links from `site.ts`'s revision. Both identities currently match. A later pin-only edit can render old facts under new links while these tests still pass. Add the inexpensive equality assertion between `status.tideRevision` and `tideRevision` to the normal verification path. The existing manual `--check` can detect this when invoked.

## Reasoning

The contract is a static status page whose facts come from committed records at one Tide revision, with faithful labels, source links and a check that rejects drift. The implementation follows the existing site architecture without dependencies or an unnecessary runtime layer. The source-to-export-to-render flow is easy to follow.

Independent read-only observations on the named candidates, using git and isolated Python:

- Both HEADs matched the requested candidates and both tracked worktrees were clean before this report. Both candidate diffs passed `git diff --check`.
- All 15 recorded input SHA-256s match `git show d84d22a:PATH` bytes.
- The conformance record's body equals the saved `target/work/conformance-a4ab41c.stdout` verbatim. That stdout's SHA-256 is `29f9e80c4814dd100de69a98f4f56bafe39b6d24fdd9605c582a628c50c3138d`. The saved metadata names `a4ab41c975442fe360363677d038aca71d9d4c10`, a clean unchanged tree and exit 0. The record contains exactly the pinned corpus's 392 case identities: 389 PASS, 3 AWAITS CHECKER, 0 FAIL. The declared run-input diff contains only the added run record. The reader's current corpus, tally, exit and stale-input checks agree with those observations.
- The three at-a-glance copies match verbatim. The README marker comparison is a real byte-for-byte string comparison. The generator's `--check` also compares complete output strings, writes neither output, and exits nonzero on a difference.
- The remaining search count extractions match the named rows, including the special repository denominator for fx-zu4f.11. The pilot's 66/72, 67/72, ratio 0.99, interval [0.95, 1.00] and “not met” match result.json. B1 concerns lost/mislabeled stages, not fabricated numeric values.
- Proc::command and Net::get are present at the pin. Net::resolve is in the shipped text, bind_udp is planned, and signals/PTYs are Later. The Tasks edit retains its methods, synchronization handles, fault_point return type, false-outside-simulation note and companion reference while correctly separating the planned item.
- The reference budget edit matches D14's 25,000-token cap. The companion and export agree on 24,999, which is within that cap. The builder records the recount at this candidate. I did not rerun the Claude-based counter, so the token total is source-reported rather than independently recounted in this pass.
- Items 3 and 4 retain the required Net/Proc conditions, printing exception, outgoing-peer versus listening distinction, no_bind() and allowed-program authority limit. Item 8 keeps replay's model/machine distinction and the open rustc soundness-hole limit. Item 9 distinguishes self-attestation from existence-by-attestation-time. The new surfaces introduce no “unforgeable”, “race-free by construction” or “Tide caught” claim. The hero uses the requested slogan followed immediately by the handed-&Fs mechanism and a source-derived example.

The falsifier edits the aggregate PASS/FAIL verdict counts in a scratch export, rebuilds the scratch site, requires both the conformance section and affected area row to change, and cleans up in finally. Under its documented fresh-baseline-build precondition, this is a meaningful check of export-to-tile dependence. It does not prove source-to-export parsing or numeric-source links. The page uses scoped table headers, captions, named keyboard-focusable overflow regions and a labeled reference-size graphic. Static inspection found no additional accessibility blocker.

No build, Bun, Cargo, Nix, heavy run, browser suite or falsifier was executed in this review. The builder's successful runtime checks remain attributed evidence. Approval requires B1–B6 to be addressed, with targeted parser/provenance checks for the behavior changes and an honest first-use terms pass.

DONE
