Model: Claude Opus 5.5 · Date: 2026-11-19 · Bundle: Tide 801da4ea1e5edeb6c4a1b2ceab956638f1770aeb, site cac0ca3a16961f120a9e35cf7fa2755ba1a9db56 (Opus review)

## Verdict

Clean: no blocking findings. The bundle can land and be redeployed. Five non-blocking notes follow; none makes the site or README false, misleading or broken.

What I checked, independently of the Sol 6.2 files (read only after my own pass):

- **(a) Claims map.** I rebuilt it from `git show 65df11d:docs/at-a-glance.md` (872 words) against `docs/at-a-glance.md` at 801da4e. Every required qualifier is either in the new text or one link away:
  - catch qualifier: item 9 says "not first-time and not a rate" and "registration before the fix not independently proven". Its link goes to README "A malicious crate…", which says the pool was chosen afterwards, the run was admission and type-checking only, and there is no judgment of intent.
  - 0/87: item 9 says "caught 0 of 87".
  - "up to rustc's soundness bugs" and the forge-routes limit: items 3 and 8 link to `README.md#forge-routes`.
  - x86_64 Linux only: item 8.
  - replay reproduces the simulator's model: item 5, which links to Simulator fidelity.
  - E3 found no evidence that the trust diff helps: item 6, with the E3 link. The text says "summarizes", never "catches".
  - Dropped detail is all one click away: weaker orderings behind the `unsafe` grant (tide-or-rust "What it costs"); an allowed program having its own full authority (tide-or-rust "What Tide gives you"); OpenTimestamps-anchored rounds showing only file existence (status page legend, `status.astro:71`).
  - The chrono_anchor narrative keeps the standing wording at the README link. Item 3 uses "no accidental ambient authority…", and nothing says "unforgeable".
  - Every anchor exists at 801da4e. The README block is byte-identical to the source. The site copy differs only in link URLs pinned to 801da4e.
- **(b) FAQ facts and tone.**
  - The WASI entry matches `design/01-design.md:614` ("A WASI target maps capabilities onto component imports"; there is no WASM target in v0).
  - The async entry matches D8 and tide-or-rust "No async".
  - "Who should use Tide today" agrees with tide-or-rust's plain-Rust list. madsim and turmoil appear in `docs/owner-guide/answers.md:43`.
  - The tone complements Rust. The heading count ("thirteen") matches the 13 `<h3>`s, and the meta description lists the new entries.
- **(c) Nav.**
  - Before and after shots at 1280 and 390, in dark and light, including hover and focus states. In dark, the chips now read clearly as buttons: a 1px muted inset edge on the `--page-top` fill, and the moon glow no longer bleeds through Status and Paper.
  - In light, the edge is subtle and consistent. Hover shows an accent edge and underline. The focus ring is visible.
  - At 390 the chips wrap to two rows without overflow.
  - The change is CSS-only and nav-only: `text-live.txt` and `text-after.txt` are byte-identical. The site has no manual theme toggle, so `prefers-color-scheme` alone is correct.
- **(d) Re-pin.**
  - `site.ts`, `status.json` and `charts.json` all carry 801da4e. The built `dist/` has 179 blob links, all at 801da4e.
  - `public/paper/tide-v0.pdf` is unchanged in the branch: sha256 `7f3eea6c…` in paper-r2.log, and its links stay on v0-alpha.1 = e5627ef.
  - `recordings/` and the clip media are unchanged. `git diff --stat e5627ef 801da4e` touches only README, docs/at-a-glance.md, review files and `.beads`, so the clip's inputs are unchanged.
  - Tide main is 65df11d, the branch base, so a fast-forward keeps 801da4e and the pin stays valid.
- **(e) Logs.**
  - `site-checks-r2.log`: every step exits 0, including status-data `--check` matching 801da4e, typos, astro check, build, falsifier, and 40 e2e passes. `dist/` was built at 18:22:30, after cac0ca3 was committed at 18:22:07, and contains the final async text. The site worktree is clean.
  - `code-prefix-r2.log`: 2 passed, exit 0.

## Blocking findings

None.

## Non-blocking

1. **`docs/at-a-glance.md:44`–`45` (and the README copy): implicature.** "For the pilot and four of the searches, the order of method and results rests on our own records" can be read to mean the other rounds' order is independently proven. The status page, one click away, says the anchored rounds prove only that the method file existed. Optional fix: add "; the others' timestamps show only when their method files existed".
2. **`docs/at-a-glance.md:39`–`43`: juxtaposition.** "One external catch … The malware screen caught 0 of 87" leaves it to the link to explain that the catch came from a later re-test of a crate the screen had seen. Optional fix: "One catch, on a later re-test of one screened crate, not first-time and not a rate, …".
3. **Length.** The new text is about 370–380 words against the owner's ~250–300 target. It is still 57% shorter than before, and keeping every qualifier justifies the excess. Lines 13, 30, 37 and 40 exceed 80 columns. Line 37 runs 121 characters because a link was not reflowed.
4. **`site/src/pages/index.astro:68`–`69`: clip caption.** The caption says the clip was "built at Tide revision 801da4e", but the recording ran at e5627ef. The cac0ca3 commit message shows that every build input is unchanged, so the binaries are equivalent and the caption is not misleading. A stricter wording would be "recorded at e5627ef, whose build inputs 801da4e keeps". Optional.
5. **`docs/tide-or-rust.md:34`: circular citation.** It cites "at a glance, item 4" for "a program a process capability allows runs with its own full authority". The new item 4 only says "An allowed shell allows anything" and links back to tide-or-rust. The sentence itself states the fact, so nothing is lost. Also, `code-prefix-r2.log` was written 3 s before the 801da4e commit, so it ran on the uncommitted tree with the same content. That is acceptable, but a log taken after the commit would be cleaner.

DONE
