Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidates Tide 7eb5585, site 555cc9c (re-check 2)

Finder re-check of **only B2, B4 and B6**, against the “Minimum remaining fix” lines in `target/work/r1-recheck-codex.md`. I read that report and `target/work/r1-codex.md` first, then inspected site `git diff f85a495 555cc9c`, the affected callers and tests, `report.md`'s “Round r1 fixes 2”, `r1-checks3.log` and the updated `terms-pass.md`. Tide is unchanged. No unrelated findings are added.

Both HEADs match the named candidates and both tracked worktrees were clean. The inspected site source files match committed 555cc9c bytes. The site fix diff passes `git diff --check`; its SHA-256 is `e23b819d4e4d9c50dd837a89edc7f1c11465e8b8d74e860c2f8600abeac0ef3a`.

## B2 — Resolved

Evidence: site `scripts/status-data.ts:191`, `src/data/status.json`'s E4 boundary row, `src/pages/status.astro:51–61,89–90,231–232`, `src/pages/index.astro:31–36`, and Tide `target/work/terms-pass.md`, H entry 5, S entries 25/35 and the closing tile-prose inventory.

- **E4:** the gloss now explains that the experiment's frozen rules were set in bead E4 and barred outside libraries, the network and other programs. The renderer places this gloss before the quoted boundary name. “Bead” is already defined in the preceding glossary. The explanation agrees with `research/41-qin-dataset-search.md:146–151` and requires no source-wording change.
- **Protocol:** entry 25 defines a network protocol as rules for how programs talk over a network, before entries 27 and 28 use that meaning. Entry 35 explicitly distinguishes a search's written method. The reading-order dependency identified in the first re-check is repaired.
- **Alpha:** the first install-action group now has an immediately adjacent explanation: “The alpha is Tide's first public test release. It runs on Linux on 64-bit x86 processors (x86_64).” I count this explanation directly under the actions as a definition at the first action's context. Readers no longer have to reach glance item 8 to learn either its meaning or its platform scope.

The terms inventory now records these actual locations. Independent inspection of the existing built HTML confirms the protocol ordering, the E4 gloss before its quote, and the home alpha explanation. This resolves the three remaining gaps rather than merely declaring them absent.

## B4 — Resolved

Evidence: site `scripts/status-parse.ts:118–130,157–162,185–191` and `scripts/status-parse.test.ts:65–72`.

The Bitcoin payload is now passed to `blockHeight()` **before** its attestation tag is recorded. That helper requires a nonempty, complete unsigned variable-length integer, limits it to five bytes and rejects bytes following its terminating byte. Errors propagate through `parseOts()` and `anchored()`.

I traced the same witnesses through the production code. Each uses the valid proof header, version, SHA-256 operation and matching digest, followed by the Bitcoin tag and the indicated length-delimited payload:

| Payload | Static trace and isolated Python reproduction |
| --- | --- |
| Empty | Rejects at the empty-height check, before recording the tag. |
| `81` | Its continuation bit is set with no following byte. Rejects as truncated. |
| `81 01 00` | The integer terminates at byte 2 but the payload has byte 3. Rejects as bytes after the height. |
| `81 01` | Accepts the complete height 129, as a positive control. |

The new committed negative test constructs all three malformed proofs and calls `anchored()`, beside the existing pending-only negative. It checks the corresponding failure messages.

For independent read-only corroboration, I ran `python3 -I` with an inline reproduction of the builder probe's proof-tree traversal plus the updated TypeScript height checks. This was Python evidence, **not execution of TypeScript**. It rejected all three malformed witnesses. It also parsed the four real proofs at Tide 7eb5585, checked their file digests, and found valid Bitcoin heights: fx-zu4f.8 protocol **976452**, fx-zu4f.11 protocol **976487**, fx-zu4f.8 input manifest **976462**, and fx-qlek.17 protocol **976490**. The db130d0 receipt still has four pending/other attestations and no Bitcoin attestation.

This satisfies the remaining small-parser fix. Cryptographic commitment and blockchain verification remain outside this re-check's stated contract.

## B6 — Resolved

Evidence: site `src/pages/status.astro:60` and `tests/site-smoke.spec.ts:253–261`.

The interval definition now uses “a stated confidence” with no numerical example. It introduces neither a literal percentage nor a new quantitative claim. The linked, generated confidence in the evidence tile remains unchanged.

The targeted browser check now removes links and code from status `main` and rejects any remaining `%`. It separately requires the “Gate, interval” definition to contain no digit. These assertions cover the exact glossary omission that survived the first re-check.

Independent inspection with Python's HTML parser confirms that the existing built interval definition has no digit, status main has no percentage outside links/code, and the displayed **95%** confidence is an anchor to its owning record at the full 7eb5585 pin.

## Verdict

**Approve.** B2, B4 and B6 meet their minimum remaining fixes. No problem introduced by this fix requires another change. This verdict is limited to those items and their fix diff; it does not repeat or widen the earlier full review.

Builder-attributed runtime evidence: `target/work/r1-checks3.log` records Astro check with zero errors/warnings, **10 parser tests passing**, successful build and falsifier, generated-file equality at 7eb5585, rejection of the historical pending receipt, acceptance of the four complete proofs, **30 browser tests passing**, and exit 0. I inspected the log and relevant tests without rerunning them. Existing built HTML is corroboration, not an independently rebuilt artifact.

Skipped: all builds, Cargo, Nix, Bun and browser-suite execution. Risk/limit: TypeScript runtime results are builder-attributed, and timestamp cryptographic validity was not independently verified. Only this report was written; no commit or implementation change was made.

DONE
