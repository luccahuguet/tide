Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate site 6cf8217 (design pass, Opus review)

## Verdict

APPROVE. No blocking findings. Everything below is non-blocking.

## What I checked (independently)

- **Text invariance.** I ran WORK/text-invariance.py on the worktree's `dist/` myself, and its output is byte-identical to `inv-before.txt` (baseline 62373cb). The dist matches the candidate: it has the r3 CSS (`forced-colors`, `outline-offset:-3px`, the 999px radius) and the r3 Video script. The extraction is complete for what this diff could change:
  - every CSS `content:` in the candidate is `''` (site.css:182, 202, 260, 343);
  - there is no `text-transform`;
  - Sea.astro has no SVG `<text>`, `<title>` or `<desc>`;
  - aria-hidden text is reported separately and is empty on all 5 pages;
  - attributes include `aria-label`, `title`, `alt`, `href`, `src`, `poster` and meta `content`.
- **Contrast.** I recomputed the pairs that contrast.md leaves out:
  - the hero h1 gradient end (`--accent-deep`) is 6.66–9.02 in light and 8.92–12.11 in dark against the sky, page-top and wave layers;
  - the dark focus ring on the sky is 8.65.
  
  `--tie` is under 4.5 on the light waves (4.04), but it is never used as a text colour: it appears only in a border, bar fills, axis lines and the ceiling stroke, all of which need 3:1 and get it. Links have no `:visited` override, so visited, hover and active links keep the accent colour. The footer links sit on `--wave-near` at 5.45 (light) and 7.06 (dark).
- **The moon.** Dark text on the dark moon disc would be 1.01:1, so I traced the moon's position at 320, 641, 656, 760, 900, 1000 and 1440 px. No body text reaches the disc. Only the nav, at about 640–700 px, comes near it, and the nav sits on opaque chips; the new e2e sweep checks those chips at every 20 px in both themes.
- **Reduced motion and forced colours.** The global `animation/transition: none !important` covers the waves, the glints and the button transition. A media query applies live, so turning reduced motion on mid-visit stops the decoration too. I traced the Video.astro state machine (`automatic` / `autoplayed` / `userPaused`) through these cases, and it is correct in each:
  - autoplay, then leaving the screen;
  - a visitor's pause, then scrolling back;
  - reduced motion turned on mid-play, then off;
  - a visitor's play under reduced motion, then scrolling away.
  
  Under `forced-colors` the sea is hidden and the h1 gradient is removed.
- **Phone, bytes and layout shift.** WORK axe.json covers 30 runs (5 pages × 3 viewports × 2 themes): 0 violations and no overflow. The checks-r3 e2e run has 40 passed and 2 skipped. The CSS grows from 5,745 to 12,673 bytes (4.1 KB gzip). There are no fonts, no images and no new JS bundles; the JS chunks are byte-identical. The sea, the seabed and the separators are absolutely positioned or fixed-height, so they cause no layout shift. No custom property is unused.
- **Charts.** BenchChart and E3Chart are legible in both themes: the on-bar labels use `--on-accent`, and loss and tie keep their meaning. The paper figures exist only inside the PDF, which this diff does not touch.

## Blocking findings

None.

## Non-blocking

1. **The committed axe test runs only in the light theme** (tests/site-smoke.spec.ts:430). Dark is a whole new colour system, but only the throwaway WORK/shots run proves it, so a later token edit could break dark-theme contrast without failing `test:e2e`. Minimum fix: wrap the loop in `for (const colorScheme of ['light','dark'])` with `page.emulateMedia({ colorScheme })`.
2. **No e2e check that reduced motion stops the decoration.** The brief's hard constraint ("prefers-reduced-motion stops all motion") is tested only for the clip. Cheap guard: under `reducedMotion: 'reduce'`, assert `document.getAnimations().length === 0` on `/`.
3. **The nav chips show as boxes** (site.css:172, 175). Each chip is painted in flat `--sky`, but the sky behind it is a gradient with stars, so every label sits in a faintly visible rectangle (see top/home-phone-dark.png and the "Status" chip in top/home-desktop-dark.png). At about 640–700 px the second-row "Paper" chip also clips the moon's edge (nav/nav-656-dark-focus.png). Either make the chips read as deliberate pills (a hairline border or a rounder radius), or move the moon down or left at 641–720 px.
4. **The hero h1 gradient tints whichever words fall at the right of the box** (site.css:213): "an" on the home page, "alpha." on /install. Because it depends on the viewport, it looks like emphasis on arbitrary words. Also, with `-webkit-text-fill-color: transparent`, selected h1 text loses its `::selection` colour. Consider a vertical or much subtler gradient, plus `.hero h1::selection { -webkit-text-fill-color: var(--on-accent); }`.
5. **Stars can land inside heading glyph areas** (Sea.astro; for example the dot beside "an" in top/home-desktop-dark.png). This is cosmetic; keep the star field out of the left 60% of the band, or drop the stars at y > 160.
6. **Dead or duplicated rules.** site.css:319 sets `.chart .axis line { stroke: var(--line) }`, and line 320 immediately overrides it. contrast.md lists a `code-muted #9fb3c8` token that site.css does not define. That is WORK-only, but the table should match the tokens that ship.
7. **Infinite wave animations keep running while the band is scrolled off screen** (site.css:158). They are composited and cheap; pausing them off screen is optional.
8. **Old-browser fallback** (Video.astro:78). `MediaQueryList.addEventListener` throws on Safari before 14. The throw happens after the observer is set up, so autoplay still works and only the mid-visit change handling is lost. Acceptable.

## Design quality (launch first impression)

The "night tide" concept delivers the wonder the owner asked for. The dark theme (moon, stars, layered swells, glints) is striking and still calm. The light "morning tide" is no longer plain white: it reads as pale sea and sky, though it is the quieter of the two.

The decoration stays in the top band, in the section ripples and in the seabed above the footer. The claims, the `&Fs` code block, the clip and the charts sit on even backgrounds and remain the hero. The book-serif headings give the slogan a voice without a font download.

Items 3 and 4 are the only polish I would do before launch. Neither blocks.

DONE
