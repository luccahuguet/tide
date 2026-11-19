Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Re-check 9ad42b9

## Verdict

**Approve.** B1–B6 are resolved and N7 passes the requested 24,990-token cap. No new blocking defect was found in the fixes or the rebase. This is the scoped supervisor re-check of s1-r1, not a fresh full review, red-team verdict, Opus verdict, or implementation approval.

Candidate: `9ad42b9c559c8313cffa0ac9192e065a83ebebf7`, whose parent is rebased main `d58f3e425914d717360e04d3d114a926aa7ed4f8`. SHA-256 of `git diff d58f3e4..HEAD`:

`48e386714d056c32d840ff282fa92f9ca079c8a5896da91535538f52c0b099fa`

Compared `git range-diff 1a5e5fd..dd1de77 d58f3e4..9ad42b9`, the changed research text, the four affected documents, the new reference proposal, and the builder's claims under “Round s1-r1 fixes.” No tracked files, commits, or Beads state changed during this re-check.

## Finding status

### B1 — Fixed

`research/42-net-get-v0.md:89–114` and `:288–312` now define the host as written and reject `%` before percent-decoding/IDNA, as proposed. Empty labels are explicitly a Tide restriction, with one trailing dot allowed. The encoded Unicode, hidden Punycode prefix, fullwidth-character and empty-label negative cases reach all three relevant forms: URL, only entry and connect address. Percent encoding in paths/queries stays legal.

The rule is carried into `design/03-authority-concurrency.md:224–241`, companion 21, NET-HTTP-1 and stage-2 acceptance. The raw-host span follows the [WHATWG authority/host states](https://url.spec.whatwg.org/#special-authority-ignore-slashes-state); tying the span to url's parsing remains an explicit stage-2 check. Other schemes are expressly outside the admitted GET route and are rejected by get before network activity. That stated restriction is acceptable; this approval does not assert that generic Url parsing never invokes IDNA for an unsupported scheme.

### B2 — Fixed

`research/42-net-get-v0.md:391–410` now distinguishes TCP contact and ClientHello traffic from authenticated HTTP data, under HTTPS too, and explicitly retains raw connect on the same Net. It removes the misleading HTTPS mitigation advice. The corresponding NET-HOST teaching (`design/03-authority-concurrency.md:246–254`), companion 21 and soundness reasoning agree. DNS rebinding and the absence of a private-address filter are stated limits rather than a claim that certificate verification prevents contact.

### B3 — Fixed

`research/42-net-get-v0.md:171–205` now puts the bodyless-status rules ahead of generic framing: reject an unsolicited 101, skip other interim heads, and complete 204/304 at the header terminator. A lone interim head followed by close is Reset. The header-size bound also covers interim heads.

`research/42-net-get-v0.md:618–621`, companion 17 and NET-HTTP-3 restrict Reply::status to 200–599. The requested real-backend 204, 304, 100/200, lone-100 and 101 cases, plus invalid simulated statuses, are required in stage 2. Chapter 03 and companion 21 agree. Rechecked the new citations against [RFC 9110 §7.8](https://www.rfc-editor.org/rfc/rfc9110#section-7.8) and [§8.6](https://www.rfc-editor.org/rfc/rfc9110#section-8.6); they support the upgrade and 304 Content-Length explanations. No new status/framing contradiction was found in this fix.

### B4 — Fixed acceptably without changing dependencies

`research/42-net-get-v0.md:513–536` replaces the impossible-to-interpose assertion with an actual comparison of httparse plus owned framing, ureq-proto, and ureq's configurable Transport/Resolver boundary. It acknowledges the code that ureq-proto would remove, then chooses the smaller graph, stable API and direct ownership of Tide's stricter response policy. Replacement cost remains explicit.

Checked the published ureq-proto 0.7.1 source: response_body_allowed excludes 1xx/204/304, and chunked framing takes precedence over a valid Content-Length. Checked ureq 3.5.2 source and metadata for the custom transport/resolver boundary, unversioned API, optional cookie support and proxy configuration. Release dates and the cited 0.x evolution are supported by published packages; comparing release sources also shows public-interface/compatibility changes. The premise of the rejection is now materially accurate. This accepts the engineering tradeoff; it does not prove the estimated size or correctness of the future Tide framing implementation. Sources: [ureq-proto](https://crates.io/crates/ureq-proto/0.7.1), [ureq](https://docs.rs/ureq/3.5.2/ureq/).

### B5 — Fixed

`research/42-net-get-v0.md:550–599` identifies exact proposed releases, features, licenses and direct/provider-selection roles, separates the transitive inventory, and labels the lockfile, builds and adapter equivalence test as stage-2 proof. The conditional adapter alternatives are exact 1.0.0/1.1.0 releases rather than floating version families.

Independently checked crates.io's version metadata for all six direct selections (including both adapter alternatives): the releases, publication dates, license expressions and named features agree. Checked the listed transitive releases and the selected rustls/ring/idna dependency declarations. The record now supplies reproducible proposed identities without claiming that this docs-only candidate contains a built or locked graph. Sources include [rustls 0.23.46 metadata](https://crates.io/api/v1/crates/rustls/0.23.46), [ring 0.17.14 metadata](https://crates.io/api/v1/crates/ring/0.17.14), and the same version endpoint for the other named selections.

### B6 — Fixed

`design/03-authority-concurrency.md:212–225` again describes the current literal TCP key as current, labels adoption of NET-HOST by connect as planned for fx-ff13 stage 2, and ties retirement of inline, non-abandonable resolution to that stage's NET-RESOLVE work. The research's stage-2 description agrees. The prior availability contradiction is removed.

### N7 — Fixed; measured 24,986 tokens

The new `target/work/reference-proposal.diff` has SHA-256:

`071cb4b2c6e7ade038f5a02ef145a5547eb910097b508d37f5da1d601dffe42e`

Verified that the tracked reference equals main d58f3e4's version, which already includes the landed Proc design. Copied it to `target/work/s1-r1-recheck-reference/design/reference.md`, checked and applied the proposal only there, then ran:

`scripts/count-tokens.sh target/work/s1-r1-recheck-reference/design/reference.md </dev/null`

Result: **24,986**, passing **24,990** with four tokens of headroom and D14's 25,000 with fourteen. This is a count of the actual combined Proc-plus-Net reference, not the earlier arithmetic estimate. Moving get's signature into 6.7 and pointing the timing bullet to companion 17 preserves the signature and timing contract. The tracked reference remains untouched.

## Rebase and fix-local checks

The rebased candidate changes only the original five files. Against main d58f3e4, the conformance conflict resolution adds NET-HTTP-1 through NET-HTTP-4 once each; PROC-1 through PROC-3 are byte-identical to main and occur once each. The diffs in 01, 03 and the companion preserve the landed Proc text and contain the intended Net changes. Research/40 and all other files outside the five-file diff are unchanged from rebased main. The range-diff shows the original Net decision retained with the scoped corrections, not lost during the rebase.

`git diff --check d58f3e4..9ad42b9` passes. HEAD and the candidate diff hash remained unchanged through review. No further blocking finding arises from the touched fixes.

Not checked: no implementation builds, runtime tests, red-team pass or Opus review. Stage-2 behavior, including the host-span and IDNA equivalence checks, remains unproved.
DONE
