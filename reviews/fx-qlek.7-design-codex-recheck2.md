Model: gpt-6.2-sol (xhigh) · Date: 2026-11-19 · Re-check 83946bf -> 6cf8217 (design regressions)

Verdict: approve

## Regressions from the r2 fixes

1. **Pressed feedback on button-styled links under reduced motion — resolved.** The active rule adds an underline and a 2 px inset ring (`src/styles/site.css:246–247`), and the reduced-motion rule disables transitions and the hover transform (`src/styles/site.css:382–385`). In offline Chromium 157, I held down each of the three home-page button-styled links at 320 and 1440 px. In both themes, pressing underlined the label and drew the 2 px ring with opacity 1 and no transform. In forced colors, the underline still appeared. On 83946bf, the pressed and hovered computed styles matched in every case. The new e2e check fails on 83946bf in both projects at `tests/site-smoke.spec.ts:184` because it receives `none` for the underline. Both instances passed on 6cf8217 (`falsify-r3.log`, `checks-r3.log`).

2. **Releasing reduced motion pauses a manually started clip — resolved.** The observer now checks whether playback was automatic before pausing an offscreen clip (`src/components/Video.astro:48–57,61–68`). At both widths in offline Chromium 157, I loaded with reduced motion, pressed Space on the clip, scrolled it fully offscreen, and turned reduction off. On 6cf8217 the clip stayed playing and its clock advanced about 0.66 s during the 650 ms sample. On 83946bf it paused after advancing at most 0.02 s. On 6cf8217 an automatically started clip still paused offscreen, resumed in view, and paused on enabling reduction. The new e2e check fails on 83946bf in both projects at `tests/site-smoke.spec.ts:256` because `paused` is `true`. Both instances passed on 6cf8217 (`falsify-r3.log`, `checks-r3.log`).

## Text invariance and proof scope

I reran `python3 -I text-invariance.py` on the existing builds for both revisions. The outputs matched their saved dumps and the production baseline `inv-before.txt` byte for byte (121,392 bytes, SHA-256 `33ffd421dff9118735e6b1d625b27c023c64b9b9656754ad4780e8ec85993a54`). The five HTML pages also matched byte for byte after normalizing the stylesheet hash and the inline video script. The four CSS `content` declarations were empty in both builds.

I found no regression introduced by these two fixes. The site worktree was clean at 6cf8217 and `git diff --check 83946bf 6cf8217` passed. The supplied `shots/after` images predate 6cf8217 and show the home-page buttons at rest. The interaction evidence comes from the offline browser probes and the saved e2e logs (40 passed, two project-specific skips on 6cf8217).

DONE
