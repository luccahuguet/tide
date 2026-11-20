Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidates Tide b25fbc0 + site 62373cb (address fill, Opus review)

## Verdict

**Approve.** No blocking findings. The address fill is true before the flip and the tag. The paper, the site and the Tide texts agree. No placeholder remains. The PDF matches an independent compile. The NOTICE's new wording holds under the README's pin steps. One landing precondition follows from main having moved (N1). It concerns the landing, not the candidate's content.

Read-only. I wrote only this report and an independent PDF compile at /mnt/build/tide-agents/fx-qlek.7-addr/opus/. Both worktrees stayed clean.

## What I checked

1. **Truth.**
   - Tide diff 582abd5..b25fbc0 (excluding .beads) touches only `paper/tide-v0.typ`.
   - The Status line now says the paths are "linked at the release tag `v0-alpha`". That says where the links point, not that they resolve, so it is true now.
   - The header comment and the "one place" comment carry no placeholder claim. `links-pending` is gone, with no remaining reference: the compile is clean.
   - Unchanged conditionals stay correct:
     - paper :306, "once the repository is public, a registration means…";
     - Tide README :357–358, `docs/install.md:40` and `docs/nix.md:12`, "works once the repository is public".
   - Site:
     - README "Repository links" says `githubRepository` *sets the address*.
     - `repoFile` links "resolve once the repository is public", and the PDF links "resolve once the tag exists". Both conditions are right.
     - `site.ts:2` says "address", not "public repository". Codex's B1 fix is applied as prescribed.
     - The site commit message's conditional is right too.
     - "Home of Tide v0 alpha" no longer claims the site is public.
2. **Consistency.**
   - Paper: `luccahuguet/tide`, tag `v0-alpha`.
   - Site: `githubRepository = 'luccahuguet/tide'`, `tideRevision = b25fbc0d…`. `status.json` and `charts.json` carry the same SHA, and `at-a-glance.md` links at it.
   - The Tide README's Codespaces URL, clone and `nix run github:` commands name the same repository.
   - The site README's account of who owns the PDF links (the paper, at the tag) matches `build-paper.sh`, which now compiles the paper unchanged.
   - `site-pins.sh` still reads both values. `repository_url` is now used only by its own guard, which is harmless.
3. **Placeholders.**
   - Site: `rg 'owner\.invalid|OWNER|RC-TAG'` over src, dist, public, README.md, NOTICE and scripts finds nothing.
   - Tide (outside archive/research/reviews/trials): no `RC-TAG`, `owner.invalid` or `OWNER/`. The only `<owner>/<repo>` left is the `--input` usage example in the header comment.
   - PDF text: no OWNER, RC-TAG, owner.invalid, "pending" or "placeholder".
   - The only `.invalid` left is the e2e guard `tests/site-smoke.spec.ts:70`, which is intended.
4. **The PDF.**
   - My compile of b25fbc0 (typst 0.15.1, exit 0, no output) has `pdftotext` text identical to site `public/paper/tide-v0.pdf` (sha256 d3d489a2…, as reported). `dist/paper/tide-v0.pdf` is byte-identical to public.
   - URI annotations are the same multiset in both: 147 annotations, 68 distinct URIs, 56 of them repository links.
   - Every repository link is `github.com/luccahuguet/tide/{blob|tree}/v0-alpha/<path>`. I checked each against b25fbc0: every path exists, and `blob` and `tree` match file and directory in every case.
   - `node paper/figures/build.mjs --check` exits 0: "figures match their records (22 files)".
5. **The pin.**
   - Site pin values are consistent across site.ts, status.json, charts.json and at-a-glance.md.
   - All 59 distinct `blob|tree/b25fbc0…` paths in dist exist at b25fbc0.
   - `dist/licenses.txt` carries the new NOTICE. It has no d835458, bc603edb or 449394e1.
   - The NOTICE wording is pin-relative. The README requires the paper to be rebuilt at every pin and the clip re-recorded when its inputs change, with the evidence recorded otherwise. So "built at that revision" and "re-records them when their inputs change, otherwise its commit records…" stay true when the steps are followed.
   - Clip retention is justified: 633f905..b25fbc0 touches no clip input, and the builder's store-path evidence is in the commit message.
6. **Leftovers.**
   - Codespaces `codespaces.new/luccahuguet/tide`: 3 in dist, plus the e2e assertion.
   - `nix run github:luccahuguet/tide#tide-demo`: 3.
   - Clone `https://github.com/luccahuguet/tide`.
   - `repoFile`, the glance links and llms.txt all derive from `githubRepository`.
   - The removed rewrite wrapper and its README text are fully gone.
   - Nothing stale remains in the site's AGENTS.md.
7. **Privacy.**
   - No host paths (/home, /mnt, ~/.cache), email or private details in the changed lines, src, dist, public or the PDF text.
   - Two pre-existing items are intended: the README's `/home/tide` sandbox description, and the paper's `GCONV_PATH=/tmp/injected` example.
   - PDF metadata has the author name and the Typst version only.

## Blocking findings

None.

## Non-blocking findings

- **N1 — landing precondition: the site pin will not survive a rebase.**
  - `main` is now 53cb001, a ledger-only commit (`.beads/issues.jsonl`) on 582abd5, so b25fbc0 no longer fast-forwards.
  - Rebasing the Tide candidate gives a new SHA. Site 62373cb pins b25fbc0, which then lands nowhere. After the flip, all 178 `repoFile`/glance links (`blob/b25fbc0…`) would point at an unreachable commit.
  - Before the site lands, re-pin it at the landed Tide SHA: tideRevision, both generators and the paper rebuild. The rebuild is needed because `SOURCE_DATE_EPOCH` is the commit time, and the README requires it at every pin anyway. The paper text and its tag links will not change, since the rebase changes only `.beads/`.
  - The builder flagged this as conditional. It is now certain unless the coordinator lands without changing b25fbc0's identity.
- **N2 — the tag target.**
  - `v0-alpha` must point at a commit that contains this paper change and all 56 linked paths. All 56 exist at b25fbc0.
  - If any linked path is moved or deleted before the tag, the PDF's links break silently.
  - At tag time, repeat the path check from this review: extract the PDF's `/URI`s and `git cat-file -e <tag>:<path>` each one.
- **N3 (nit) — NOTICE lead line.**
  - "Original assets, generated from Tide at the revision that tideRevision in src/site.ts pins" covers the clip, which was recorded from an earlier pin (afc5a41) with identical driver and std store paths. The next sentence qualifies this correctly.
  - The list also includes `public/favicon.svg`, which is not generated from Tide. This structure predates the change.
  - Wording such as "Original assets. The paper is built from Tide at the revision…" would be exact.
- **N4 (nit) — duplicated Kinestra SHA.** NOTICE hard-codes the Kinestra SHA, a copy of `recordings/record.sh:22` that no check compares. A recorder bump needs a NOTICE edit by hand.
- **N5 (nit) — publish order.** The site README's Publish section doesn't say that publishing before the flip and the tag ships dead links. The "Repository links" section implies it. One clause there would help the owner's launch sequencing.
- **N6 (already noted by the builder) — stale at the flip.** The Tide README :357–358, `docs/install.md:40`, `docs/nix.md:12` and the site README's "resolve once…" sentences go stale at the flip and the tag. The flip bead should drop them.

DONE
