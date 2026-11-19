Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidates Tide e80f788 + site 72a2696 (code, site stage)

## Verdict

request_changes. Two public claims need small wording fixes. The Tide punctuation change is correct, the A1 refresh is present, and the data checks pass. Confidence is high in the findings. This is a review of the frozen site-stage candidate, not publication approval.

## Blocking findings

1. **The Docker note incorrectly promises no registry pull.** `site:src/site.ts:38`.
   - Failure scenario: a reader follows the Docker path on a clean machine expecting the stated “Nothing is pulled from a registry.” Tide's pinned `Dockerfile:14` starts its build from `ubuntu:24.04@sha256:…`, which Docker must fetch when absent. This is different from not publishing a prebuilt Tide image. The shared note appears on the home page, install page and in `llms.txt`.
   - Minimum fix: replace the final sentence with “The Tide image is built locally rather than pulled from a registry.” Do not promise that the build has no registry dependency. The commands and order are already correct.

2. **The machine-readable summary drops the authority guarantee's trust boundary.** `site:src/pages/llms.txt.ts:15`.
   - Failure scenario: a consumer reads “no accidental ambient authority for admitted code” as the stated v0 guarantee. Admission also accepts code under `--tide-grant=unsafe`, which can forge capabilities without a rustc soundness bug. The following soundness-bug caveat does not cover that deliberate trust hole, and this summary never mentions the grant or the printing exception. Tide `README.md:165` limits the property to admitted safe code, apart from printing and outside the grant. `README.md:534` explicitly says the grant can forge capabilities.
   - Minimum fix: say “no accidental ambient authority in admitted safe code, apart from printing and outside the unsafe grant (`--tide-grant=unsafe`)”. Retain the existing rustc-soundness caveat. This omission predates the candidate but remains in a touched public surface within the requested trust-hole/leftovers review.

## Non-blocking findings

- **The documented publication gate remains.** `site:src/site.ts:8` still names `owner.invalid/tide`, so repository links and repository-based try-it paths deliberately cannot work publicly yet. `site:README.md:108` also documents the PDF's relative repository links until that placeholder is replaced and the paper rebuilt. These are explicitly deferred owner steps in the account and site README, not an undisclosed regression in this site stage. Do not publish these candidate assets unchanged.

## Evidence and coverage

Frozen identities, both initially and finally checked clean:

- Tide HEAD: `e80f788cfd34d46bab552320a9bdf35681865e99`, base `6d94c19`.
- Site HEAD: `72a2696d6f47389f963c21ef8940ff0cab848f0c`, base `6005174`.
- SHA-256 of `git diff --binary BASE CANDIDATE`: Tide `2e284757f528c497ceda222bbaa67ff53cd2b66502d7f20e18ad513cf9c0b664`; site `f17f5d2bc6b714a383f5730f1ab9aa10dade041b0f3b643f9c175de5918e3ee3`.

Tide: an isolated Python comparison proves that exactly the four semicolon/full-stop and following-capital substitutions produce each new file from its base. The README's marked block equals `docs/at-a-glance.md` byte for byte. Meaning is unchanged. The site generator checks its transformed copy against that exact source. No additional live verbatim copy requiring the punctuation change was found.

A1: the built `/status/` exists and is linked from navigation and the home page. The questions page now includes HTTP GET over TLS, lookup, processes and random draws, with UDP and `Rand::rng()` still planned. Status data includes seeded Rand, rounds .10/.12/.13/.15, and the 24,932-token reference count. The glance includes the qualified catch. The PDF has the new catch section and its bytes match the recorded rebuild hash. The try-it paths follow Codespaces, rustup, nix run, local Docker, with `rustup toolchain install` and both Docker build/run commands. Finding 1 concerns the accompanying Docker claim.

The catch text preserves the std gap, the screen exposing it, the fix and fresh registration, 0/87 for the screen, the post-hoc one-row pool, no rate or first-time catch, qualified chronology, and admission/type-checking only. The .13 row explicitly identifies the same catch rather than a second one. The .12 row exposes aggregate counts without identifying its private candidate. The other pages retain the soundness and replay limits, negative E3 evidence, and upstream-known #57893 wording. Finding 2 concerns the separate machine-readable authority summary.

Read-only checks performed here, with no network, installs or builds:

- `bun scripts/status-data.ts /mnt/build/eon-agents/worktrees/tide-fx-qlek.7-site --check`: exit 0, both committed outputs match e80f788. This also exercises conformance-record freshness and protocol/proof guards.
- `bun test scripts`: 20 pass, 0 fail, 35 assertions, Bun 1.4.4.
- Both data files name the full e80f788 pin, and every recorded input SHA-256 matches `git show` at that pin. Charts were independently rederived in memory from their pinned inputs and match the complete committed JSON. No hand-edited generated-data discrepancy was found.
- `node paper/figures/build.mjs --check`: the bare invocation cannot resolve `d3-scale` in this worktree. An import-only in-memory resolver using the site's installed d3-scale 4.0.2 reruns the unchanged checker successfully: “figures match their records (22 files)”. Its transitive dependency versions differ from the paper lockfile, so this is not a reproduction in the paper's exact dependency environment.
- Text diffs pass `git diff --check` (the binary PDF excluded). Built local links and fragments resolve. All 59 distinct linked Tide paths exist at e80f788. External endpoints were not contacted.
- Built media match `public/` byte for byte. PDF, clip and still SHA-256s match the builder's generation logs. The checked source/public text, built HTML and decoded PDF streams showed no host-path, private-draft or common secret-marker leak. No concrete dead-code regression was found.

Clip: inspected `recordings/record.sh`, `record-clip.rs`, `recorder.nix`, the recorded success/failure logs, the supplied four-second frame sheet, and the full-size published still. The wrapper installs assets only after recorder success. The recorder retains per-command deadlines, filmed pipeline-status checks, expected admission diagnostics and replay-output checks. The fail-closed log records invalid arguments and a filmed timeout leaving the published assets unchanged. The frames show the public #25860 conformance case, plain rustc exit 0, TD0206 and admission exit 1, eight failing seeds including 0, and two seed-0 reproductions. The 59-second caption/description agrees. Only the synthetic `/home/tide` path is visible. No archive-derived port or private text is visible in those samples.

Keeping beat (a) is justified: research/49's faithful malware port also has rustc errors against Tide's std, so it cannot meet the current rustc-accepts/admission-rejects beat. Its archive-derived body also cannot be filmed under the stated release condition. A rejection-only clip would not demonstrate the two-leg catch. The chosen public conformance case meets the intended beat without those problems.

The account's `checks.sh` and log record successful typo/Astro checks, status tests and freshness, build, status falsifier, and 30 browser tests. The separate media logs record generation and fail-closed checks. These are reviewed prior-run evidence, not reruns by this pass.

Not rerun: Rust, Nix, Docker, paper/clip builds, or checks that write generated/browser artifacts. Media provenance relies on the matching logged hashes and supplied sampled frames, not a fresh recording or examination of every video frame. No publication or external availability check was performed.

DONE
