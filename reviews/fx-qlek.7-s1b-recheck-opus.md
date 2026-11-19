Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 0a15a24 (re-check)

## Scope

This is the finder's re-check of B1, B2 and non-blocking items 1 to 9 from `target/work/s1b-opus-review.md`, against the fix diff `git diff 5246ee1 0a15a24`. That diff touches `docs/tide-or-rust.md` only. I also read the builder's "Round S1b fixes 2" account. Item 10 went to S6 and is not checked here. `git diff --check 5246ee1 0a15a24` is clean. Everything was read-only, with no builds.

## Blocking items

**B1, the selection disclosure: resolved.** The catch bullet now says: "That separate re-test was chosen after the screen had seen its one row, and its outcome was close to predictable." This matches research/49 §1 and §10 (protocol §1) and the README's wording at lines 71-72 and 182-183. The bullet keeps:

- the binding sentence about the std gap, now as "the screen exposed it"
- the qualified order
- "admission and type-checking only"
- "not a first-time catch or a rate"

**B2, macOS and Windows: resolved.** The page now says: "On macOS or Windows, docs/docker.md runs it in a Linux container, a route checked only on a Linux host so far (under emulation on Apple silicon)." This matches `docs/docker.md` lines 41-42 and 58-64. The parenthesis would read more clearly as "(and under emulation on Apple silicon)", but that is style only.

## Non-blocking items

1. **What the reader gains, and intent: resolved.** The page says the admitted version has "its task, clock and process authority now declared in its parameters and its trust artifact. Admission does not judge intent: the admitted version still starts the same processes." This matches research/49 §2 (`bg`, `clock`, `proc` in the trust artifact) and §9 ("still runs the same processes with the same arguments"). The new clause "it had already rejected this port" is correct: research/49 §2 says the leg-1 codes are the ones fx-zu4f.8 recorded.
2. **Tracker ID: resolved.** The page now says "the screen exposed it", as at a glance does.
3. **Order of the result: resolved.** The bullet now leads with what the re-test shows, and the 0-of-87 screen follows as "the re-test's scope". No disclosure was lost in the move.
4. **E1 interval: resolved.** The page adds "its information-only interval, 1.29 to 1.64, crosses 1.5". The README gives 1.285714 to 1.637931.
5. **E3 ceiling: resolved.** The page says "It could not have shown a benefit: plain review of its small changes already caught almost every planted one". This matches chapter 9 (lines 306-309: "left no room to show a benefit") and the README's 67-of-72 plain arm. There is no "catch" claim for the trust diff and no rate.
6. **Replay evidence: the case study is resolved, but the new sentence about searches is not accurate.** The eon-discovery-race sentence ("13 of 100 seeds and each failing seed reruns with an identical trace") matches README.md lines 276-283. The added sentence is the problem. See R1 below.
7. **Terms: resolved for the items named.**
   - `tide-admit` replaces "the driver" in the replay bullet.
   - "profile flags" is glossed.
   - The `contracts` program is explained.
   - "paired" is explained as "slower in 26 of 30 paired rounds, median per-round ratio 1.07x" (research/36 §1.1).
   - The 2.29x is now given as a product, "1.74x … times 1.31x", which matches research/36.

   One leftover, which the fix did not introduce: "driver" now first appears at line 100 ("The driver admits one crate") and is never tied to `tide-admit`. Writing "`tide-admit` admits one crate" would fix it. This is non-blocking.
8. **Leaving Tide: resolved.** The page now names the toolchain attributes (`#[tide::main]`, `#[sim_test]`, `#[requires]`, `#[ensures]`, `#[error]`), as in reference §2.3.
9. **The "use Rust" list: resolved.** It now says "a second crate, from crates.io or your own workspace" (README.md:365-367) and adds a bullet for anything under Missing pieces.

## Finding the fixes introduced

**R1 (blocking): the new sentence on external searches implies more than the record shows.**

- **Where:** `docs/tide-or-rust.md:45-46`: "No external search round has yet found a **new** bug this way."
- **Evidence:** README.md's Evidence section says "The other external search rounds (issues fx-zu4f.5 to .7 and .9 to .11) caught nothing", with one unresolved row in fx-zu4f.10 (research/44). Several of those rounds looked for known historical bugs:
  - fx-zu4f.10 arm B judged historical fixes.
  - fx-zu4f.9 drew from Qin et al.'s dataset of known bugs.
- **Why it blocks:** "No new bug" is literally true, but it tells the reader that the rounds only failed to find *new* bugs. That suggests known external bugs were found or replayed. None were. This is the one place the page states what the searches found, and it overstates the result in Tide's favour.
- **Minimum fix:** change "found a new bug this way" to "caught an external bug this way". Optionally add "(one row is unresolved)".

## Verdict

Changes required. B1, B2 and items 1 to 9 are resolved (item 6 only in its case-study part). R1 needs the one-word change "new" → "an external" (or "caught an external bug") at line 46. Once R1 is fixed, I approve. Re-checking R1 needs only that line.

DONE
