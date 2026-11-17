Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 1d2f4da

# fx-1vwf.3 r3: Opus review of the E3 pilot report (docs and published records)

Scope: `git diff main...1d2f4da` in `/mnt/build/eon-agents/worktrees/tide-fx-1vwf.3-report`.
I read all of `trials/e3-pilot/report.md`, the headline and breakdown fields of
`result.json`, both canary directories, a sample of `run-1/`, and the sealed notes and
reviews that `fx-1vwf.2` published. I also read the protocol (§§2, 4, 5, 8.5, 9–12),
GOVERNANCE.md, AGENTS.md, `br --no-db show fx-1vwf.3` with every comment, the relevant
comments on `fx-1vwf.1` and `fx-1vwf.2`, and Codex's r3 and its re-check. I ran no build,
model, or `e3` command and changed no file except this one. All checks were read-only
`jq`, `git`, `find` and `diff`.

## Verdict

**request_changes.** Confidence: high.

The numbers are right and the negative result is stated plainly. The §12 sentence is
verbatim and the ceiling statement is present. Four fixes are needed before this text
is fit to be quoted, all of them small:

- one integrity sentence is false;
- the false-positive paragraph understates evidence that the published records hold;
- the breakdowns are not labelled descriptive, which the brief requires;
- a reader new to Tide cannot tell from the report what a "catch", the trust diff, or
  M1–M6 are.

## What I verified

- **§12 sentence.** `report.md:3` is byte for byte `result.json`'s `sentence` plus its
  `added_sentences` entry ("The interval does not rule out no benefit.", required since
  a = 0.947 ≤ 1). `[models]` reads "Opus and Sol" and `[3 × m]` reads 6, both correct.
- **Ceiling.** The ceiling statement is at `report.md:5-7`, and again in Limits at
  `:55-58`.
- **Headline table.** The table at `:67-85` matches `result.json`: S, K_T, K_P, c_T,
  c_P, R, the interval (a, b, 10000 resamples, 0 undefined), McNemar (p 1.0, 0/1
  discordant), f_T, f_P, the Wilson bounds, both informational gates, and the
  conditional flag.
- **Breakdown rows.** All 52 breakdown rows (model, mechanism, D±, visible) match
  `result.json`, in both tables.
- **Pairs and items.** The pairs table and the `why` lines match. I regenerated all 24
  rows of the item table (288 session cells, replicate order, C/A/FP codes) from
  `result.json.items` and diffed them against `report.md:241-264`: identical.
- **Single cases.** The two disagreements (sessions 100 and 274, finding 1, line 30,
  Opus true and Sol false), the two extra findings (sessions 5 and 50), and the three
  false positives (81, 98 and 252, line 58, `fs`) all match.
- **Attempt records.** `run-1/` holds 745 `attempt.json`: 744 `verdict` and 1 `fault`.
  372 are `claude-opus-5-5` with login `none`, observed `claude-opus-5-5`; 373 are
  `gpt-6.1-sol` with login `unchanged` and an empty observed list. Judgement 090
  attempt 1 is a fault after 643.50 s, its events line 3 is the capacity error, and
  attempt 2 answered.
- **Canaries.** Both `canary.log` files print identical coverage and "canary passed",
  and the report describes that coverage correctly. Each directory holds only
  `canary.log`, `control/*/` and `out/*/`.
- **Bead disclosures.** Every deviation and disclosure on `fx-1vwf.3` is in the report:
  the guard cadence, the runner swaps, tide-lead's role sightings, the planter
  exclusion, the judgement-90 fault, and the schedule SHA-256.

## Blocking findings

### B1. `report.md:9-11`: false claim that the harness did not change

**Problem.** The report says "nothing in that protocol, the prompts, the harness, or
the items changed for it". The harness did change for this bead. `031f2da` (fx-1vwf.3)
changes `harness/src/main.rs` by +2436/−159 between the freeze (`9b080c3`) and the
run's revision `179b318`, in two ways:

- It adds the grading, adjudication and statistics commands.
- It refactors `attempt` to take a reply parser. This touches `review`, `canary`,
  `settle` and `acknowledge` at their call sites.

§13 of `protocol.md` also gained the manifest hash, through `fx-1vwf.2`. §2 allows all
of this. The test module diff only adds lines, so the existing tests are unchanged, and
`prompt/` is unchanged. But the sentence as written is false, and the report's own run
record (`:404`) contradicts it. This is the report's integrity statement. A critic
diffing `9b080c3..179b318` will find the contradiction in a minute and then distrust
the rest.

**Minimum fix.** Replace the clause with what happened. For example: "Nothing in the
protocol's rules, the prompts, or the items changed. As §2 assigns, this bead added
§9–§10's grading, adjudication and statistics commands to the harness and passed the
reply parser into `attempt` (`031f2da`, reviewed and landed before session 1). The
existing harness tests and the prompt hashes were unchanged, and the canary was rerun
at the run's revision. §13 gained the manifest's SHA-256 from `fx-1vwf.2`."

### B2. `report.md:28-33`: the false-positive paragraph understates published evidence and leans on an invalid argument

**Problem.** The paragraph closes by discounting the 3-versus-0 difference with "Three
sessions on one item cannot say whether the trust diff prompted them, and the Wilson
intervals of the two arms overlap."

- Overlapping 95% intervals are not a test of a difference, and a statistician will
  read the clause as defensive. It is the only result in the run that is adverse to the
  trust arm, and it is the one the report argues down.
- The published records say more than "cannot say":
  - All three findings (`run-1/sessions/{081,098,252}/verdict.json`) object to the
    same thing. In their words, `tail` "accepts filesystem write authority … although
    the description calls only for reading".
  - i18's `trust-diff.txt` prints `+ dir: fs.read, fs.write` for `tail`. v0 renders
    every `&Fs` parameter as read and write (§5; sealed review
    `fx-1vwf.2-r2-opus.md`, N1). The plain prompt shows only `dir: &Fs`.
  - All six plain-arm sessions and all three trust-arm Opus sessions on i18 approved.
- The mirror case is just as informative and is not mentioned. Plant i12, the run's
  only missed plant, has an identical trust-diff line, `open_read_only … + dir:
  fs.read, fs.write`, and its twin i09 has it too. No trust-arm session flagged it.
  The pre-run sealed note (`sealed/notes.md`, N1) asks the report to name pair 8
  alongside the twin false positives when it discusses the trust arm on M4 or D+.
- The cause cannot be proven from one item each. But the report should state what the
  records show, rather than imply that nothing can be said.

**Minimum fix.** Replace the last two sentences of the bullet with a descriptive
statement and drop the Wilson-overlap clause. For example:

> "All three findings object to write authority on `tail`, which only reads. i18's trust
> diff lists `tail`'s parameter as `dir: fs.read, fs.write`, as v0 lists every `&Fs`
> parameter (§5), while the plain prompt shows only `dir: &Fs`; every plain-arm and
> Opus session on i18 approved. The same line appears in the trust diff of plant i12
> (`open_read_only … dir: fs.read, fs.write`), the one plant reviewers missed, and no
> trust-arm session flagged it there. These are single items, described, not tested;
> the protocol planned no test of f_T against f_P."

If a comparison sentence is wanted, say only that no test of f_T against f_P was
planned (§10.4 reports the rates with Wilson intervals, not a comparison).

### B3. `report.md:23-27`, `:98-131`: the i12/M4 discussion and the per-model and per-mechanism splits are not labelled descriptive

**Problem.** The brief requires these labels, and the report has none.

- Per-mechanism n is 2 changes, 12 sessions per arm.
- The M4 rows show trust 6/12 against plain 7/12. The model rows show Sol trust 33/36
  against plain 34/36, and Sol is where the 3 false positives sit.
- A reader skimming the tables can read "the trust diff hurt on M4" or "Sol is worse
  with the trust diff". Neither is supported: each difference is one session or one
  twin.

**Minimum fix.** Add one sentence under "Breakdowns (§10.4)": "These breakdowns are
descriptive (§10.4: reported, not gated). Each mechanism has 2 changes and each model 36
sessions per arm, no test was planned for them, and no difference in them supports a
claim about a model, a mechanism, or an arm." Add a matching clause to the i12 bullet:
"one change; descriptive". For example: "…the only discordant pair in the McNemar
test. It is one session on one change."

### B4. Whole report: a reader new to Tide cannot tell what was measured

**Problem.** The report never defines its central terms. It relies on `protocol.md`
section numbers for:

- what the trust diff is;
- what a "catch" is: a reject, a finding within 3 lines of a sealed span with a sealed
  kind, and both blind judges agreeing it names the widening. This is the measured
  quantity, so a statistician needs it to read K_T and K_P;
- M1–M6, D+/D−, "visible", plant and twin, and the arms.

The opening bullets (`:19-33`) already use M1–M3, M4, M6, D+ and "visible". A Rust
developer new to Tide stops at the first bullet. This is the text that will be quoted,
so it must stand on its own.

**Minimum fix.** Add a short "What was tested" section, about 10–12 lines, after the
ceiling paragraph:

- **The setting.** Tide code passes authority (file system, network, environment…) as
  capability values. The trust diff is the admission driver's list of how the
  *declared* authority of exported items changed. It shows no exercised authority and
  no narrowing.
- **The items.** 12 pairs of a twin, which does what the description says, and a
  plant, which also takes undescribed authority.
- **The arms.** The plain diff, against the same plus the trust diff.
- **The mechanisms.** A one-line legend for M1–M6, saying which the trust diff can show
  (M1–M3) and which it cannot (M4–M6). One line for D±.
- **The catch rule.** The definition above, in one sentence.

## Non-blocking findings

- **N1. `report.md:3-7`, `:79`: explain why the interval is so narrow.** In 11 of the
  12 pairs both arms scored 6/6, so each bootstrap resample's ratio depends only on how
  often pair 8 is drawn: k draws give 6(12−k)/(6(12−k)+k). That is 1.00 at k = 0,
  0.985 at k = 1, and 0.947 at k = 3. [0.95, 1.00] therefore reflects the ceiling, not
  precise evidence that the trust diff does nothing. Next to the mandated "does not
  rule out no benefit", a statistician will otherwise wonder what the interval means.
  One sentence after the ceiling paragraph would help, for example: "The interval is
  narrow because 11 of the 12 pairs tie at 6 of 6 in both arms; its upper end of 1.00
  reflects that ceiling, not evidence about a benefit on harder changes."
- **N2. Say plainly what is left for the trust diff, without claiming a result.** The
  report says what the pilot cannot show but not what remains. Suggested, near
  `:34-39`:
  - This pilot gives no evidence that the trust diff helps model reviewers catch
    planted authority widening.
  - It did not test the trust diff where its value is argued: changes too large to read
    in full, human reviewers, or use as a mechanical record that does not depend on
    anyone reading it.
  - Its only observed effect here was the B2 false positives.
- **N3. Limits: add the pre-run caveat in sealed notes N2.** Every D+ twin gains its
  authority through a new exported item, never by widening an existing one
  (`sealed/notes.md`, N2). So the false-positive measure never tests a described
  widening of an existing item. It bounds what f_T says.
- **N4. Limits: name the planter's model.** The plants were written by
  `claude-opus-5-5`, the same model as one reviewer and one judge. The adjudicator
  agreement table addresses judge self-favour; nothing addresses planter and reviewer
  sharing a model. One clause in "Agent reviewers only" is enough.
- **N5. The ceiling: give one known reason plants were easy.** The pre-run sealed review
  (`fx-1vwf.2-r2-opus.md`, N5) noted that two plants name their environment variable in
  a doc comment, which makes plain-arm catches easier. That is useful to the "future run
  needs harder plants" sentence (`:56-58`). It can be said without ids, or with ids,
  since roles are now public.
- **N6. `report.md:62-63`, Role cues.** "None was reported" does not say by whom. Say
  that the `fx-1vwf.2` Codex review checked the items for giveaways and found none in
  them (`sealed/reviews/fx-1vwf.2-r1-codex.md:23`). Add that the four plants which fail
  their public suites on purpose can be identified by anyone who runs the suites, but
  not by tool-less reviewers (`fx-1vwf.2-r2-opus.md`, N6).
- **N7. `report.md:323-325`: earlier canaries.** Add that `c8` failed: the Codex
  ancestor hooks were not parsed, and that was fixed before the freeze. Add that `c9`
  passed (`fx-1vwf.1`, 05:24 and 05:33 UTC). A canary that caught an isolation gap
  during development is a point in the protocol's favour, and leaving it out looks like
  omission if found later.
- **N8. `report.md:11`: "Every number below comes from `result.json`" is too broad.**
  Attempt counts, CLI versions, timestamps and 643.5 s come from `run-1/` records. Say
  "Every result number".
- **N9. Wording.** `:87` "The retained models are both models" reads oddly; say "Both
  models are retained: neither was dropped…". `:25` "approved it with no findings" is
  redundant, since an approve has no findings by §9.1. Harmless.

## Fresh-reader notes

- **Skeptical statistician.**
  - **Clear:** the gate failed and why it had to (§10.5); the McNemar caveat about
    non-independent sessions (`:44-46`); the units of analysis.
  - **Missing:** the definition of a catch (B4), why the bootstrap interval is
    degenerate-looking (N1), and an honest account of the one arm difference (B2).
  - **Defensive:** the Wilson-overlap clause. They will not object to the result. They
    will object to the paragraph that argues the false positives down.
- **Rust developer new to Tide.**
  - **Can follow:** "plain diff vs trust diff, 12 changes, both caught almost
    everything".
  - **Cannot follow:** what the trust diff contains or which kinds of widening it can
    show, until they open `protocol.md`. That matters most for the "Six of the twelve
    plants (M1 to M3) are visible" bullet, which is the crux of why the null is
    expected on half the plants.
- **Tone.** Mostly plain and non-promotional. There is no sentence suggesting the trust
  diff helps. "The pilot cannot show whether the trust diff helps reviewers catch more"
  (`:34`) is accurate, but standing alone it can read as hope. Following it with N2's
  "no evidence that it helps" sentence makes the balance explicit.
- **Structure.** Result, ceiling, then what it shows and what it cannot, then Limits:
  the right order. The appendices (pairs, items, faults, CLI pins, canaries,
  deviations, run record, publication list) are thorough and easy to audit.

## Reasoning

The result is negative, and the report says so first and in the mandated words. The
risk with a report like this is not inflated claims. It is a small defensive or
inaccurate sentence that a critic can use to discredit an honest record.

B1 is such a sentence. It is false on a point anyone can check, it is contradicted
three sections later, and the truth (sanctioned harness additions) is harmless.

B2 is the other. It is the only place the arms visibly differ, and it is adverse to
the trust diff. The report discounts it with an invalid interval-overlap argument and
does not say what the records plainly show:

- the findings echo a trust-diff line that only the trust arm saw;
- the same line was ignored on the one plant everyone missed.

Stating both, descriptively, is fairer to the result and more useful to the next run.
That does not mean reading v0's read-and-write rendering as either a help or a harm.

B3 is required by the brief and costs one sentence.

B4 is the fresh-reader requirement. The report is the citable artifact, and it cannot
currently be understood without the protocol.

None of these fixes changes a number, the gate, or the §12 sentence. They are text
changes to `report.md`, but B1, B2 and B3 change what the report asserts, so I should
re-check them. B4 and the non-blocking items need re-checking only if the author wants
me to.

DONE
