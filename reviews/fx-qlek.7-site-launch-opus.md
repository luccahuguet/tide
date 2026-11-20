Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate site 494f87c0d3446f9a2f64c75fd60c02b16e56da53 (launch integration, Opus review)

## Verdict

**Approve.** No blocking findings. The candidate is ready to land and deploy once v0-alpha.1 is public, which it already is: `git ls-remote origin refs/tags/v0-alpha.1` gives e5627ef.

## Blocking findings

None.

## Non-blocking

1. `README.md:33-34`: "These links resolve once the repository is public" and "so those links resolve once the tag exists" are still true, but both conditions now hold, so the future tense is stale. The README is not in dist. If you want, change both to present tense on a later pin.
2. `report.md` §2/§5 counts each PDF link twice (`/URI` plus the visible text): 260 `blob` and 6 `tree`. My extraction found 56 distinct Tide targets, all at `v0-alpha.1`, which matches Codex's 133 link actions. This affects only the report, not the site.
3. The checks log (`site-checks-2.log`, last written 17:08:17) predates the commit time of 494f87c (17:08:30), so the checks ran on the uncommitted amended tree. I confirmed that this tree equals the head. The worktree is clean, no source file is newer than the log, and dist's clip.mp4, clip.png and tide-v0.pdf match `git show 494f87c:public/...` byte for byte. The proof stands.

## What I checked

1. **Merges.** f0dbc8f and dbfa2d5 each have an empty `git show --cc`. `git merge-tree --write-tree` of each pair of parents reproduces the committed tree exactly (67c4c8b, d899c6f). `git diff --stat dbfa2d5 494f87c` touches only seven files: README.md (one tag line), src/site.ts (the note and the pin), status.json, charts.json, at-a-glance.md, tide-v0.pdf and clip.mp4. Nothing else changed.
2. **Pin.** `tideRevision` is e5627ef in site.ts, both JSON files and all three at-a-glance links. I recomputed all 31 status input sha256s and all 3 chart input sha256s from `git show e5627ef:<path>`, and every one matches. The conformance tile's `ranOn` d751d4a is e5627ef's parent. Between the two commits, the only change in the record's inputs (conformance, crates, Cargo.*, rust-toolchain.toml) is run-record.txt itself, so "ran the whole corpus at d751d4a" is true of the pin. The record shows exit 0 and 393 cases. Outside the PDF, the only site files that name a tag are README.md:34 (`v0-alpha.1`). The local and origin tag both resolve to e5627ef.
3. **PDF.** From b25fbc0 to e5627ef, paper/ changes only the `tide-tag` default (v0-alpha → v0-alpha.1). The PDF's sha256 is 7f3eea6c…5efb, matching paper.log. It has 68 distinct URIs: 56 point at `luccahuguet/tide/{blob,tree}/v0-alpha.1/`, and each path exists at e5627ef with the right kind (blob or tree, checked with `git cat-file -t`). The other 12 are external references. No URI uses another tag.
4. **Clip.** The clip is h264, 1280×720 and 57.6 s, with sha256 675a2268…cf74 matching record.log. record.sh builds the recorder against `git+file://<tide>?rev=$tide_revision`, and the log shows tidelang-std and tidelang-admit built fresh. I grabbed my own frames (WORK/review-frames-opus/) and checked them against the old clip:
   - Beat (a), at 21 s: rustc exits 0, then tide-admit prints `TD0206 … (CLOSURE-3)` and exits 1. Both `error:` labels are bold salmon. Read line by line against old-20.png, the commands, source view and output text are identical.
   - Beat (b), at 44 s and at the end: the search shows 8 failures over 16 seeds, and both replays print `replay reproduced … seed=0`. The text is identical to old-45.png and to the still.
   - clip.png (8317a0ed…) is unchanged from 0298ff9.
   - NOTICE:16-21 still describes the media correctly: recorded by record.sh with Kinestra d1589705…, using the flake's driver and std, in DejaVu Sans Mono. recorder.nix uses `pkgs.dejavu_fonts`. NOTICE does not change in this range.
5. **Text invariance.** I ran text-invariance.py independently on both dists. After normalising the pins (176 full occurrences and 2 short ones in each dist, all replaced one for one), only these differences remain:
   - the gdb note on the home page, the install page and in llms.txt (4438 → 4606 bytes, with no semicolon);
   - /status/'s run commit and date (9ee96fa 12:37:45Z → d751d4a 19:56:30Z).

   Nothing else differs. The note reads well and is true at e5627ef:
   - The Dockerfile's `dev` stage, which .devcontainer builds, installs only ca-certificates, curl, xz-utils and git, plus Nix. It has no gdb.
   - flake.nix:180-195 replaces PATH for `nix run .#tide-demo` anyway.
   - demo/README.md:44-47 says step 5 prints `skipped: …` without a gdb built with Python, and step 5 is "Debug the replay … under rust-gdb".
   - The rustup path, where `cargo run -p tide-demo` uses the caller's PATH, is the one that runs it. This agrees with Tide README:360-366.
6. **Checks.** Every step in site-checks-2.log prints `=== exit 0`: install, status-data --check, bun test (20/0), typos, astro check (0/0/0), test:status (20/0), build, status-falsifier, and e2e (40 passed, 2 skipped). The skips are tests 12 [desktop] at site-smoke.spec.ts:117 (`phone widths only`) and 14 [mobile] at :133 (`one sweep of widths is enough`). Both are the expected `test.skip` viewport gates, and each test runs on the other project.

DONE
