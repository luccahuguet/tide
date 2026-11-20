Model: gpt-6.2-sol (xhigh) · Date: 2026-11-19 · Candidate site 1eb9b5a (catch section)

## Verdict

`request_changes` (high confidence). The reordered section preserves the research claims and caveats. The new caption misidentifies what the code shows.

## Blocking findings

1. `src/pages/index.astro:114` — “The crate's entry point” presents the excerpt as the original crate's entry point. Research/37:348–354 identifies `run_optional_syncs` as a function in `guard.rs` that calls two helpers. Research/49:450–473 supplies excerpts from the two Tide ports. The caption also attributes every code line to the diff, although the two closing braces were added for display. Replace the caption with “`run_optional_syncs` in the faithful and capability-passing Tide ports. The signatures and calls come from research/49’s published diff. We added the closing braces and comments.” Keep the research/49 link.

## Non-blocking

- `src/pages/index.astro:104` — “It was not a first try” repeats the opener and the mandated qualifier. Starting the sentence with “Our original pre-registered screen” would read more directly. The new prose contains no “X, not Y” contrast, and the mandated qualifier remains verbatim.

## Claims map

Line numbers refer to `src/pages/index.astro` in live `ac91d06` and candidate `1eb9b5a`.

| Live | Candidate | Verified claim |
| --- | --- | --- |
| 133–135 | 93–95 | Same section name and heading. |
| 137–139 | 105–107 | The pre-registered screen covered all 87 malicious-crate advisories in RustSec, Rust's public security-advisory database, and caught 0 of 87. The research/47 link survives. |
| 139–140 | 107–108 | Tide's std had a gap, the screen exposed it, Tide fixed it and re-tested under a new registration. The research/49 link survives. |
| 141–143 | 97–100 | The re-test meets the unchanged caught rule for RUSTSEC-2026-0039. The advisory concerns `chrono_anchor` attempting to send a project's `.env` files to a server impersonating `timeapi.io`. |
| 143–146 | 100–103 | The faithful Tide port was rejected in both runs. Two `thread::spawn` calls drew TD0602, and two ambient `Command::new` calls and one ambient `thread::sleep` drew TD0501. |
| 146–148 | 103–104 | The authority-passing port is admitted on re-test. Its trust artifact declares task, clock and process authority. |
| 150–154 | 121–125 | Verbatim. The checks support an OpenTimestamps anchor in a Bitcoin block before the fix landed, with independent proof of that order still unestablished. |
| 155–163 | 126–134 | Verbatim. Keeps the first-time and rate qualifier, one-row selection after seeing the screen's rows, near-predictability, original 0 of 87, admission and type-checking only, no execution, no intent judgment, same processes and arguments, declared authority, no unaided discovery, already-known malware and Socket's quote, no other catch, the unresolved row, and the status link with every round's counts and qualifier. |

## Verification

- **Excerpt and release condition.** Independently compared the rendered code's source lines with research/49:457–483. Site lines 28–30 match the minus lines at 468–470, and site lines 35–37 match the plus lines at 471–473, with only the diff prefixes removed. The two bare closing braces at site lines 31 and 38 are absent from the hunks. They are editorial punctuation expressly allowed by `brief.md:38`, along with the labels. The excerpt adds no other port line, payload, program name or string. Research/49:61–76 and :433 confirm the failed and successful legs, and :450–453 confirms that the two helpers contain the counted operations.
- **Anchors and document order.** Checked both revisions with `git grep` and parsed the candidate's built HTML. The unique `#catch` is immediately after `#clip` and before `#replay`, inside `main`. Its `aria-labelledby` points to its unique `h2`, in sequence with the sibling section headings under the home `h1`. The primary navigation and skip target remain valid. There are no internal links targeting `#catch` in the home page, status page, llms.txt or other site pages. Existing external fragments still have the same target, and the section's `/status/#evidence` link resolves.
- **Rendering and accessibility.** Inspected all six candidate screenshots at 320, 390 and 1280 px in both themes. The full excerpt is legible, with code wrapping on phones. `shots/results.json` shows no page overflow, matching code scroll and client widths of 282, 352 and 1222 px respectively, and zero axe violations in every run. The markup uses `figure`, `pre`, `code` and `figcaption`.
- **Checks and scope.** All nine commands in `site-checks.log` exit 0 for the candidate at Tide pin `18e58d8`. Playwright reports 40 passed and 2 skipped. `git diff --check ac91d06 1eb9b5a` passed. A source comparison and rerun of the read-only `inv-catch.py` against the saved builds confirm that all changes belong to `#catch`, including the added research/49 caption link. Browser and build evidence came from the supplied runs.

DONE
