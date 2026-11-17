Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 577a452

# fx-qlek.15 Opus docs review (r2): Owner's guide, the landed E3 protocol

## Verdict

request_changes

Three blocking findings. Each is a sentence that says more than the protocol does, and each takes a
one-sentence fix. Everything else in the E3 text matches `trials/e3-pilot/protocol.md`, the
harness, and the fx-1vwf.1 landing record. No sentence hints that E3 has data, and nothing planned
is described as done.

## Scope and checks

- Contract: `br --no-db show fx-qlek.15` (Scope, Acceptance).
- Diff: `git diff 08face4...577a452`. Its SHA-256 is
  24fd7c52b5649d2cbe2e621bf58c1915c81759ccdd74a057ac315986cd7ad821, which matches. It touches 4
  files, all in `docs/owner-guide/`. `git diff --check` passes.
- Main is ce0084f. `trials/` and `reviews/` are the same at the base and at main, so I checked
  against the worktree's copies.
- No `TODO(tide-lead, fx-1vwf.1)` marker is left in `docs/`.
- I read all of protocol.md. In the harness I read `probes()`, the subcommand table, and the canary
  prompt. I also read commit 9b080c3's message (the canary proof and the fired probes),
  `reviews/fx-1vwf.1-r9-opus.md` and its re-check, Codex r1, D0 in `design/01-design.md`, and
  beads fx-1vwf.2, fx-1vwf.3 and fx-qlek (the "launch anyway" decision).
- I ran the new `sed` command (ch. 9, line 170). It only reads, and it prints exactly §1 and §10.2.
- I wrote nothing but this file. I built nothing, launched no model, and started no subagent.

## Blocking findings

**B1. The freeze is stated with no exceptions, but the protocol has two.**
`docs/owner-guide/09-experiments.md:51-53`, and with it `:93-95`.

- What the guide says: "the arms, prompts, budget, models, sample, seed, grading rules, threshold
  and statistics don't change". The controls bullet then says the run "doesn't start" if a judge
  misses a control.
- What the protocol says:
  - §2, third bullet: before the first review session, a change the run can't proceed without may
    still be made. It is recorded on fx-1vwf.3 and may not draw on any plant's content.
  - §9.2 Controls: if a control fails, "the adjudication prompt may change only then", and the
    controls run again.
- Why it blocks: read together, the guide says the judges' prompt can never change. The protocol
  says it can, before session 1. A critic holding §9.2 could say the owner misstated the freeze.
- Minimum fix: add one sentence after "don't change". For example: "Before the first review
  session, the only change allowed is one the run can't proceed without, recorded on fx-1vwf.3 and
  drawing on no plant. That includes fixing the judges' prompt if they fail a control." In the
  controls bullet, change "or the run doesn't start" to "or the run doesn't start until the judges'
  prompt is fixed and both judges pass".

**B2. Chapter 7 still says the plain arm gets the "code diff alone".**
`docs/owner-guide/07-trust-diff.md:170-171`.

- What the guide says: "reviewers with the code diff alone".
- What the protocol says (§5): the plain arm also gets the description and the complete numbered
  program after the change.
- Codex r1 blocked the same understatement at 09:72, and chapter 9 was fixed. Chapter 7 now
  contradicts it.
- Why it blocks: it makes the comparison sound tilted toward Tide ("of course a bare diff misses
  things"). The bead's Acceptance asks that every E3 statement match protocol.md.
- Minimum fix: "...than reviewers given the same change without it", or "...than reviewers with
  the plain code diff" (D0's own words).

**B3. "Designed to be hard to pass" is a claim the protocol never makes.**
`docs/owner-guide/09-experiments.md:191-193`, the answer to the hardest question.

- This is context text, unchanged by the diff, but it is an E3 statement the Acceptance covers. It
  is also the paragraph the owner will quote.
- It is the same kind of overclaim as the "can't flatter the tool" clause that Codex r1 blocked.
- What the protocol actually says:
  - half the plants use mechanisms the trust diff can't show (§4.1);
  - so the protocol expects any gain to come from the other half (§10.5);
  - its gate is a 1.5× point estimate over 12 changes, with no significance requirement (§10.2).
- A hostile statistician's first reply would be "a point estimate on 12 units is not a hard bar".
  The guide gives the owner nothing to answer that with.
- Minimum fix: replace "and it is designed to be hard to pass: half its plants use mechanisms the
  v0 trust diff isn't expected to show, and the result is published either way" with "half its
  plants deliberately use mechanisms the v0 trust diff isn't expected to show, so the protocol
  expects any gain to come from the other half, and the result is published either way."

## Non-blocking findings

1. **"Passes" should be "passed".** At `09:120`, "The isolation canary passes live", and at
   `10:36`, "its isolation canary passes".
   - The pass is one run, on 7055031, with codex-cli 0.169.0 and the pinned Claude CLI (commit
     9b080c3's message).
   - §8.5 makes the canary run again before session 1 and after every breach fix. A new CLI version
     can fail it.
   - Suggested wording: "passed at landing (2026-11-17)".
2. **"Harness frozen" needs one qualifier.** At `10:35`, "Its protocol and harness are frozen".
   §2 still lets fx-1vwf.3 add the grading commands and pass the reply parser into `attempt`, and
   chapter 9 says the grading commands are still to come.
   - Suggested wording: "its protocol, prompts and review path are frozen".
3. **"1.5× more often" can be read two ways.** At `answers.md:68`, unchanged context, and in
   meaning at `09:11-13`. "1.5× more often" can be heard as 2.5×.
   - The protocol's words are "at least 1.5x as many".
   - Suggested wording: "at least 1.5 times as often".
4. **What is counted: 12 changes, or 72 sessions?**
   - At `09:97`, "catches at least 1.5 times as many plants". The gate counts plant *sessions*,
     72 per arm (§10.1–10.2), not the 12 plants. Better: "catches the plant in at least 1.5 times
     as many sessions".
   - At `answers.md:70`, "288 fresh ... review sessions" is accurate. But a statistician will call
     it pseudo-replication. §10.1 says the 12 changes are the experimental units, and the interval
     resamples those 12.
   - One clause in answer 7, or in chapter 9's bar bullet, would head that off: "the 12 changes
     are the units; the sessions are repeated reviews of them".
5. **"Can't just reject" is too strong.** At `09:68-70`: "so a reviewer can't just reject every
   non-empty trust diff".
   - A reviewer can. The D+ twins make that strategy cost false positives, and those aren't gated.
   - The strategy also doesn't yield catches by itself: a catch needs a finding on the planted
     lines, of the right kind, that both judges confirm.
   - Closer to §4.1 ("cannot sort items by whether the trust diff is empty"): "so rejecting every
     non-empty trust diff would also reject clean twins."
6. **One safeguard is missing.** §10.4: when the gate is met and the trust arm's false-positive
   rate is more than 0.10 above the plain arm's, the report must say the trust arm also rejected
   more clean changes. That is the direct answer to "false positives aren't gated, so a
   trigger-happy trust arm could pass", and the guide doesn't mention it.
7. **"Isolated review sessions" is slightly loose.** At `09:122`, "the isolated review sessions
   picked up none". They were one canary attempt per model, launched the way reviews are, not
   reviews of items. This is a nit: "the isolated attempts, launched the way reviews are, picked
   up none".
8. **For the owner's own awareness: who made the "launch anyway" call.** At `09:115-117`, "The
   owner decided on 2026-11-17". fx-qlek records this as the owner's decision by delegation: he
   said "autodecide", and tide-lead took recommendation (4). The guide's wording is accurate as
   recorded, but he should know it was delegated if a funder asks.

## Fresh-reader notes

**Could he explain E3 in his own words?** Mostly, if he reads slowly. The section is 12 dense
bullets with no short summary first. Three plain sentences before the bullets would give him
something to say on a call. For example:

> "Twelve small changes each come in two versions: an honest one, and one that sneaks in extra
> authority. Fresh AI reviewers see every version, some with the trust diff and some without. If
> reviewers with the trust diff catch the sneaky version at least 1.5 times as often, the bar is
> met."

**Terms:**

- *Twin* and *plant*: defined well (`09:59-62`).
- *Arm*: never defined as a word. "That alone is the plain arm" only implies it. Add "an arm is one
  of the two conditions reviewers are put in".
- *Adjudicator*: the guide says "judges", which is fine. But the `sed` command prints §10.2, which
  says "adjudicator". Add "(the protocol calls them adjudicators)".
- *Interval*: not explained. He can't yet say why "low end 1 or less" means "doesn't rule out no
  benefit". One sentence would do it: "a range for the true ratio, computed by resampling the 12
  changes; a ratio of 1 means no difference."
- *Paired test*: unexplained. Either drop it or say "a test that compares the two arms on the same
  change and model".
- *Authority kinds*: `09:85`, "names one of its authority kinds". This needs one line: each finding
  carries a label from a fixed list (fs, net, clock, ...), and the label must match the plant's.
- *Mechanism*, *attenuation*, *point estimate*: clear from context and earlier chapters.

**A gap in the reasoning:** the guide never says *why* invisible plants make the test harder. The
trust diff can't help on that half, so any gain from the other half is diluted in the pooled ratio
(§10.5). This is also the honest form of B3's claim.

**Would the hardest-question answer survive a hostile statistician? Partly.**

- Survives:
  - "Model reviewers only": §1's limit is quoted.
  - "Is the grading blind?": masking is mentioned, with the honest "not proven blind".
  - "What if it fails?": answered well.
- Doesn't survive yet:
  - "1.5× on 12 changes, with no significance test?" The answer to this is in the bar bullet (a
    point estimate; the report must say if the interval doesn't rule out no benefit). It is not in
    the hardest-question answer, and B3's "hard to pass" invites exactly this question.
  - "288 is pseudo-replication": note 4.
  - "Ungated false positives": note 6.
  - "AI judges grading reviewers from the same AI families". Two judges from different families
    must agree, the controls are checked first, and the report gives agreement by judge and by
    reviewing model (§10.4). The guide mentions only the first.
- One added sentence in the answer would cover the first two: "The bar is a point estimate; the
  report also gives an interval over the 12 changes and must say when it doesn't rule out no
  benefit."

## Reasoning

What I checked, statement by statement:

- **Correct as written:**
  - the numbers: 12 pairs and 24 items; six mechanisms with two pairs each; 288 sessions, 72 of
    them on plants per arm; 1,906 lines, so "about 1,900"; the models; the 15-minute, no-tool
    budget; the fixed seed; the 14-day deadlines; 5–15 hours;
  - the catch rule: within 3 lines, of the right kind, two judges agreeing, keyed to the mechanism;
  - the masking caveat and the pilot caveat;
  - the result sentence fixed in advance;
  - the canary's coverage: 11 probes; Claude's control fired C-HOME-MD, C-STYLE, C-PARENT-MD and
    c-home-hook, but neither C-XDG-MD nor c-parent-hook. Those two are `home/.config/claude/CLAUDE.md`
    and `parent/.claude/settings.json` in `probes()`, which matches the guide's description;
  - the two remaining beads' status: fx-1vwf.2 is in progress, fx-1vwf.3 is open;
  - the "What to look at" quotes, which are verbatim.
- **No sentence implies data.** The guide says "no data" in all four files, and chapter 9 says E3
  has no results to reproduce.
- **The three blocking findings** share one pattern: each sentence is stronger than the protocol
  taken as a whole.
  - B1 drops the protocol's own escape hatch.
  - B2 understates what the control arm gets.
  - B3 asserts a difficulty the protocol never claims.
- **Why they block now:** the owner will quote these lines to funders and critics, and each fix is
  one sentence. The rest is about making the section easier to explain and harder to attack. None
  of it is an accuracy defect.
- **Tier rule:** this is docs only, and every finding is in files this change edits, so nothing
  goes to fx-djp.

DONE
