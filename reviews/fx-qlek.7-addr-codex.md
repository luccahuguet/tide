Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidates Tide b25fbc0 + site 78357bd (address fill)

## Verdict

request_changes — high confidence. The address fill, paper rebuild, generated data and re-pin are sound. One newly introduced availability statement conflicts with the brief's explicit pre-publication constraint. Approval needs only the wording fix below. No override.

Reviewed read-only against the relayed 2026-11-19 decision, Lucca: "Yeah", authorizing luccahuguet/tide and v0-alpha while publication and tag creation remain pending. Neither checkout was changed. Only this report and the explicitly authorized independent PDF output were written.

## Blocking findings

1. **P2 — The site describes the repository as already public.** Site:README.md:30 says "is the public Tide repository" and Site:src/site.ts:2 calls it "The public Tide repository on GitHub". These are newly introduced present-tense availability descriptions, although the repository remains private. README.md:33 correctly says its links resolve once the repository is public, leaving the paragraph internally inconsistent. This violates the brief's express wording constraint.

   Minimum fix: make README.md:30 say that githubRepository sets the Tide repository address, luccahuguet/tide, and make src/site.ts:2 describe "The Tide repository address on GitHub, as owner/name". Keep the existing truthful conditions about publication and tag creation. No code, pin or PDF change is needed for this fix.

## Non-blocking findings

1. **Inherited asset provenance mismatch.** Site:NOTICE:16 names Tide d835458 as the assets' revision. Site:NOTICE:17 and Site:NOTICE:20 give obsolete PDF and clip sizes/hashes, copied verbatim into dist/licenses.txt:18. The candidate PDF is 472349 bytes, SHA-256 d3d489a260d6dbb1a30f9b9ebfe2c2d96d040a658b3d126be281869582f74a26. The retained clip is 1351777 bytes, SHA-256 a1f899c02ce271cc91e223ba2776f4e806ee78c0b4de88d5981ce307912d8b3b. Both mismatches already exist in site parent d622b19, whose PDF is 406613 bytes and whose clip is identical to this candidate. Follow-up minimum: distinguish the assets' provenance and refresh the current metadata, then regenerate licenses.txt through the site build. This inherited discrepancy does not block the address fill.

## Evidence and boundaries

- Frozen identities: Tide b25fbc0d7195e800154a3293bd3df4bf1c94c852 on 582abd5, site 78357bdc61565712bef2bbc10e1db361eae14efe on d622b19. Both tracked working trees stayed clean. Tide review diff SHA-256 e6f64a275ac28d18cfa622327f73ec45f85e0e27f9fe7f17864611effadb2aa7. Site binary diff SHA-256 51e0cbd3ed06548511c68a234508b160ef1f7921e7cd5270ec69cc3b2f9e9d2e.
- In this Linux worktree, `typst compile --root . paper/tide-v0.typ /mnt/build/tide-agents/fx-qlek.7-addr/codex/tide-v0.pdf` exited 0 with no warnings or other output. `node paper/figures/build.mjs --check` exited 0: "figures match their records (22 files)". Tide's scoped `git diff --check` passed.
- The paper defaults to the chosen URL and v0-alpha. Removing links-pending leaves no dependent conditional text. Status now states where the hyperlinks point, without saying they resolve. The unchanged "once the repository is public" registration commitment remains true. No false public-availability claim was found in the paper.
- `pdftotext` output is byte-identical for the independent compile, site public PDF and dist PDF. Extracted-text SHA-256: f5cac6010badb7b1970e6184d8e45e64cd68565bd2fbea19a49248c43decc2fb. The two site PDF copies are byte-identical and match the builder's recorded PDF hash. Their external URI multisets also match the independent compile: 133 repository URI actions cover 56 distinct paths, all present at b25fbc0 and linked under blob/tree/v0-alpha. The byte-level PDF difference from the independent compile does not affect extracted text or URI targets.
- githubRepository, repositoryUrl, Codespaces, clone, nix run and repoFile have the intended ownership and values. Existing dist home/install/llms outputs contain the chosen commands. Generated status/charts pin the full b25fbc0 SHA. All 31 status inputs and 3 chart inputs independently hash to the committed records at that revision and are unchanged from the old pin. Their JSON payloads differ from the site parent only in tideRevision. The at-a-glance output exactly matches the pinned source with the expected URL substitutions. Generator/parser code is unchanged.
- The builder's checks.log records status-data.ts --check passing at b25fbc0, alongside the site checks and 30 passing browser tests. I did not execute Bun or rebuild the site under this review's command allowlist. chart-data.ts has no --check mode, so no such result is claimed. Its output consistency was checked through the unchanged-input and payload comparison above.
- Clip retention follows the README rule. Git shows no change to Cargo/flake inputs, driver/std, admission case, demo, patch or recording scripts. flake.nix:61 restricts Cargo source inputs to Cargo.toml, Cargo.lock and crates. The builder's clip-inputs.log records identical driver/std store paths at the old and new pins, also recorded in site commit 78357bd. Clip and still bytes are unchanged from the site parent and agree with dist. No clip or Nix build was run.
- All files in site src, dist and public were scanned for owner.invalid, OWNER, RC-TAG and host/private-path markers. PDF text, literal objects and inflated streams were also checked. No matches were found. No new private or host-specific shipped content was found. The retained clip was not replayed or visually inspected.
- No live-link verification was attempted: the repository is private and the tag does not exist. A landing that changes the pinned Tide identity requires the corresponding site re-pin and artifact refresh.

DONE
