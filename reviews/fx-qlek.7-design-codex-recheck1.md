Model: gpt-6.2-sol (xhigh) · Date: 2026-11-19 · Re-check 7298037 -> 83946bf (design)

Verdict: changes required

## Original blocking findings

1. **Paper link over the moon and stars — resolved.** The shared nav puts opaque sky-colored backgrounds behind the labels and draws an inset focus ring on the nav links (`src/styles/site.css:170–177`). In Chromium 151, I checked Paper at 390, 620, 640, 641, 656 and 1024 px, with pointer hover and keyboard Tab focus in both themes. Contrast on the chip was 13.03:1 normal, 6.31:1 hover and 5.77:1 for the focus ring in light, and 12.82:1, 7.91:1 and 8.65:1 in dark. The refreshed 620 px dark and 640 px dark focus screenshots show the moon behind the chip. The new opacity check (`tests/site-smoke.spec.ts:133–146`) fails on 7298037 because the computed background is transparent. The check covers opacity, and the browser probe covers hover and focus.

2. **Pressed footer link contrast — resolved.** Pressing Repository in Chromium changed its underline from 2 px to 3 px and kept opacity at 1. The pressed foreground and footer background give 5.45:1 in light and 7.06:1 in dark (`src/styles/site.css:84–87,353–357`). The new check (`tests/site-smoke.spec.ts:148–164`) fails on 7298037 in both projects. It receives opacity 0.8 and a 2 px underline. See the pressed-feedback regression below for button-styled links.

3. **Walkthrough overflow with disclosures open — resolved.** The long admission source link wraps under the new rule (`src/pages/how-it-works.astro:119–126`, `src/styles/site.css:291–292`). I opened every disclosure in mobile Chromium on the five routes at 320, 360 and 390 px in both themes. Every document matched its viewport width on 83946bf. The walkthrough measured 408 px at all six width and theme combinations on 7298037. The new check (`tests/site-smoke.spec.ts:117–131`) fails on 7298037 at its first affected case, 408 px against a 321 px limit.

4. **Autoplay after enabling reduced motion — resolved.** An automatically started clip paused when reduction was enabled and stayed paused after I scrolled away and back (`src/components/Video.astro:43–83`). Space then started it and advanced the playback clock by 0.50 s in a 0.50 s sample. A manually started clip kept playing when I enabled reduction while it was in view. Disabling reduction with a paused, eligible clip in view resumed autoplay, and a subsequent user pause survived preference changes and scrolling. The new mid-visit check (`tests/site-smoke.spec.ts:204–219`) fails on 7298037 in both projects because the clip remains playing after the preference change. See the preference-release regression below.

`falsify-tests.log` records six expected failures on 7298037 and two project-scoped skips. `checks-r2.log` records all six corresponding instances passing on 83946bf as part of 36 passes and two skips.

## Regressions introduced by the fixes

- **Button-styled links lose pressed feedback under reduced motion.** The new active rule excludes `.button`, and the reduced-motion hover rule already sets the same transform as the button's active rule (`src/styles/site.css:86,246–247,382–385`). With reduction enabled, I held the pointer down on “Install the alpha” (`src/pages/index.astro:31–33`). On 83946bf the underline, border, shadow, transform and opacity all matched the hover state in both themes. On 7298037 the press changed opacity from 1 to 0.8. Give these links a pressed border or underline that remains visible with reduced motion while keeping full opacity.

- **Releasing reduced motion can pause a manually started clip.** Load with reduction enabled, scroll to the clip, press Space, scroll to the top and turn reduction off. The clip was still playing offscreen at 0.31 s on 83946bf. The preference change re-observed the clip, and the initial offscreen entry paused it at 0.32 s (`src/components/Video.astro:61–78`). The same clip remained playing on 7298037, advancing from 0.32 s to 0.78 s over the preference change. Refreshing the observer for paused clips when reduction is released would preserve this manual playback.

## Text invariance and proof scope

I reran `python3 -I text-invariance.py` on the existing production, 7298037 and 83946bf builds. All three outputs matched their saved dumps and shared SHA-256 `33ffd421dff9118735e6b1d625b27c023c64b9b9656754ad4780e8ec85993a54`. All five HTML files also matched byte for byte after normalizing the stylesheet hash, removing exactly the two approved decoration blocks per candidate page and isolating the changed inline Video script on the home page. The script changes playback state. Every generated CSS `content` value in the candidate is empty.

The worktree was clean at 83946bf and `git diff --check 7298037 83946bf` passed. Browser probes loaded the existing builds offline in Chromium 151. The complete e2e suite results come from the saved logs.

DONE
