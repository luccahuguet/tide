Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidates Tide 7eb5585, site f85a495 (re-check)

Finder re-check of B1–B6 and N1–N2 from `target/work/r1-codex.md`, read first. I inspected only their fixes and side changes in `git diff d84d22a 7eb5585` and `git diff d7ced61 f85a495`, the affected source records, the amended export and renderer, the new checks, `report.md`'s “Round r1 fixes” and `terms-pass.md`. No implementation, ledger or history changes. Only this report was written. No build, Cargo, Nix or Bun command was run.

Both HEADs match the requested candidates and both tracked worktrees were clean. Fix-diff SHA-256s:

- Tide: `62550640270658ef9ac6c151814ca8d58202d65cd88102cda1f1efedbac455d7`.
- Site: `7f71e695f47fcd21cf25cd2aae0dac4bfdae7ab85345abe1d96697aaee324d3a`.

## B1 — Resolved

Evidence: site `scripts/status-data.ts:247–320,338–388`, `src/pages/status.astro:257–279`, `src/data/status.json`, and the count sections of research/37, 38, 39, 41, 46 and 47 at Tide 7eb5585.

The first round now has separate RustSec and GitHub rows. RustSec shows **1,507 in-date advisories** screened, **87 qualify, 12 with quoted code ported or assessed**, and **0** caught. GitHub shows **60 unique results, 60 excluded before assessment**, **0 examined candidates**, and **0** caught. These retain the source's distinct stages instead of calling 87 screened or 12 eligible without qualification. The generator checks qualifying against examined candidates, the examined outcome sum, and the GitHub raw/unique/excluded/examined relationships.

I checked every other round's stage labels against its record:

- fx-zu4f.6: **32 unique issues past the filters / 0 eligible / 0 caught**.
- fx-zu4f.7: **119 unique issues past the scope filter / 0 / 0**.
- fx-zu4f.9: **100 rows of the dataset / 0 / 0**.
- fx-zu4f.11: **7 repositories, 42 candidate rows / 0 candidate units / 0 candidate units**. The units remain distinct.
- fx-zu4f.8: **87 advisories / 16 with a portable source / 0**. The 16 is 87 less the record's 71 without portable source, explicitly labeled as source availability.

The amended generator checks these source-stage relationships before export. I also read the existing built HTML with isolated Python: its seven rendered rows contain those labels and counts. This meets the requested source check as well as the renderer check.

## B2 — Not resolved

Evidence: site `src/pages/status.astro:26–68,86–93,228–232`, `src/pages/index.astro:25–47`, Tide `docs/at-a-glance.md:3–4,47–48`, and `target/work/terms-pass.md:69–129`.

Most named gaps are fixed. The visible glossary precedes the tiles and explains core reference, companion, ambient authority, seed, the network acronyms, CPU, PTY and the registration qualifiers. The shared glance expands AI. Research/41's quoted names now have adjacent plain glosses.

The first-use requirement still fails:

1. **E4 remains unexplained.** The terms pass correctly declares it MISSING, and the rendered boundary name still says “Frozen experiment operation bans (bead E4)”. Declaring the gap fixes the artifact's honesty, not the page's acceptance condition. Keeping the quote does not prevent adding a definition before it or in its preceding gloss.
2. **The glossary itself uses “protocol” before defining it.** Entry 27 says “HTTP is the protocol of the web”, and entry 28 says wire parsing reads “a protocol's bytes”. Entry 35 supplies the first definition, and defines only a search's written method, not the network meaning used earlier. The artifact's claim that every entry is explained by plain words or earlier entries is therefore false. Its reading-order pass misses this dependency.
3. **The home terms pass omits the first “alpha”.** The home first uses it in “Install the alpha” (`index.astro:32`), before the glance's item 8 defines it. The H inventory lists eyebrow, lede and caption terms but skips this intervening action. This is within the home material changed by the B2 fix.

Minimum remaining fix: explain E4 beside or before its quote, define the network meaning of protocol at its first glossary occurrence (or remove that dependency), and define alpha at its first home occurrence. Correct the terms inventory to follow those actual locations. No change to research/41's source wording is needed.

Side-change check: the shorter lede retains the handed-`&Fs` mechanism immediately after the required slogan. The x86_64 Linux alpha scope is **not deleted from the home**: glance item 8 still says “an alpha (an early test version), runs on Linux on 64-bit x86 processors (x86_64)”, within the same hero section. The install page also retains its x86_64 Linux eyebrow and prerequisite. However, the first home alpha action now precedes both its definition and the platform scope. A brief explanation at that action can preserve the scope where readers first meet the alpha as well as repair its first-use gap.

## B3 — Resolved

Evidence: site `scripts/status-parse.ts:16–87`, its callers in `scripts/status-data.ts`, and `scripts/status-parse.test.ts:7–44`.

The table parser now validates the exact expected header, separator and every consumed row's cell count. Reordered capability columns throw rather than reinterpreting their values. Missing cells and malformed separators throw. The planned-method splitter accepts the documented forms and throws on other mentions of “planned”, including the former parenthesized form and repeated or ambiguously placed markers.

The committed negatives cover the requested reordered table and unknown planned marker, plus missing cells and the separator. The new normal `test:status` command is in package.json and the README verification sequence. The builder log records 9 tests passing. I inspected those tests and their production callers without rerunning them.

## B4 — Not resolved

Evidence: site `scripts/status-data.ts:27–41,322–335`, `scripts/status-parse.ts:118–174`, `scripts/status-parse.test.ts:48–76`, and `target/work/r1-checks.log:88–96`.

The main improvements are present: Git failures throw, the protocol and proof bytes are read and hashed, the proof's SHA-256 digest must equal the protocol's digest, and a pending-only tree fails. All **19** exported input hashes independently match `git show 7eb5585:PATH` bytes, including both protocols and both proofs. The actual proofs for fx-zu4f.8 and .11 have matching protocol digests and Bitcoin tags. The builder's recorded real-pending test rejects the db130d0 receipt with “no Bitcoin attestation (4 pending or other)”.

But `parseOts()` does not parse the Bitcoin attestation itself. At lines 143–145 it records any eight-byte attestation tag and skips its length-delimited payload. `anchored()` then treats presence of the Bitcoin tag as completion. A **zero-length Bitcoin payload** is accepted, although it cannot even contain a Bitcoin block height.

Concrete static witness, using the parser's own format:

`magic || 01 || 08 || SHA256(protocol) || 00 || 0588960d73d71901 || 00`

This 75-byte object has the right file digest, a Bitcoin tag and an empty attestation payload. The builder's existing Python probe (`target/work/ots_probe.py`, whose relevant algorithm matches the TypeScript) accepts it. I constructed it in memory with `python3 -I` and observed that acceptance. In the TypeScript, the same path pushes the tag, consumes zero payload bytes, reaches end-of-file and returns true from `anchored()`. This is a static trace of the TypeScript, not a claimed Bun execution.

The original minimum required unreadable evidence to fail and completion to be established, rather than inferred from a name. Replacing the filename check with a tag check that accepts a missing attestation payload leaves that failure open.

Minimum remaining fix: parse the Bitcoin attestation's block-height payload and require its complete, valid encoding, rejecting empty, truncated and extra payload bytes. Add a malformed-Bitcoin-payload negative alongside the pending negative. This requires neither blockchain access nor cryptographic commitment verification. The latter remains outside the requested small-parser check.

## B5 — Resolved

Evidence: site `src/components/Inline.astro:8–14`, the Inline callers on the status page, and `tests/site-smoke.spec.ts:259–269`.

The renderer changes semicolons only in prose segments outside backticks, to periods with following capitalization. I inspected all six affected exported strings: the two Proc cells, Tasks, the two simulator cells and the atomics sentence. Their statements, qualifiers and API identifiers remain intact. Code segments take the unchanged branch, so actual code punctuation is preserved. The source links enclosing or accompanying these cells remain in place.

The added browser assertion removes code/pre/script/style elements before checking status main prose, and also covers the home hero. The successful builder run includes this assertion on both viewports. Independently reading the existing built status HTML with Python found no semicolons in main prose. No fact or code change was introduced by the punctuation rewrite.

## B6 — Not resolved

Evidence: site `scripts/status-data.ts:233–238,390–399`, `src/pages/status.astro:60,116,254`, `tests/site-smoke.spec.ts:207–209,213–216,250–254`, and Tide `docs/at-a-glance.md:43–56`.

All four original locations are repaired. The evidence tile's confidence is extracted from result.json's published sentence, with the sentence's interval checked against `interval.text`, and rendered as a source-linked number. Exit status links to the run record. The glance's budget links to D14 and its three routes link to the README's owning bullet. Those file/anchor pairs exist at 7eb5585, and the site copy rewrites them to that pinned revision. I confirmed the requested hrefs in the existing built HTML. The browser checks cover the confidence, exit and both glance links.

However, the fix introduces another literal, unlinked **95%** at `status.astro:60`: “A 95% interval is the range a statistical method gives for the true value at that confidence.” It is a second occurrence on the same page, in the new glossary, outside the generated confidence field and the source-link checks. The existing built HTML confirms it is ordinary glossary text. Thus the exact generation/link omission survives at a new location, while the lede still promises that every number links to its source.

Minimum remaining fix: define “interval” without a numerical example, or use the generated confidence with its source link in that definition. Extend the targeted check to cover this occurrence too. The evidence tile's extraction itself is correct.

## N1 — Resolved

Evidence: site `scripts/status-data.ts:150–155`, `src/pages/status.astro:170`, and the built Fs link.

Numeric section references now resolve the named heading. Fs's “reference §6.3” points to `design/reference.md#63-files-paths-append-durability` at 7eb5585, rather than the capability table. A missing heading fails generation.

## N2 — Resolved

Evidence: site `tests/site-smoke.spec.ts:213–216`, package.json and the README verification path.

The normal browser test explicitly asserts `status.tideRevision === tideRevision` and also checks the chart pin. A stale export under new source links now fails that path. Both committed exports currently match site.ts's 7eb5585 pin.

## Verdict

**Changes required.** B1, B3, B5, N1 and N2 meet their minimum fixes. B2, B4 and B6 remain open for the specific reasons above. No unrelated review findings are added.

Verification evidence: both fix diffs pass `git diff --check`; isolated Python verified all 19 input identities, the README/source copy and the site's pinned-link rewrite, inspected the existing built rows and links, and demonstrated the malformed-attestation issue with the builder's Python probe. `target/work/r1-checks2.log` records Astro check with zero errors/warnings, 9 parser tests passing, build, falsifier, generated-file equality at 7eb5585 and 30 browser tests passing, exit 0. These runtime results are builder-attributed evidence, not reruns by this reviewer; they do not cover the remaining gaps.

Skipped: builds, runtime suites and cryptographic timestamp verification, as requested. Risk: first-use acceptance and confidence-link coverage remain incomplete, and malformed Bitcoin attestations can still receive an anchored label.

DONE
