Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 8d7181c (docs, S5)

## Verdict

request_changes. There are two blocking findings. Both are leftovers in texts this change edits, and each needs a one- or two-line fix. The rest of the candidate holds. The fixes make the texts more honest without weakening any true claim. The rebuttals in report.md are supported. All checks I ran pass.

## Blocking findings

### B1: the README's Status list leaves processes and randomness out of the alpha's std

- **Location:** README.md:447-448 ("Tide's std: capabilities (`Fs`, `Net` for TCP, one HTTP/1.1 `get` and `resolve`, `Clock`, environment, stdin, output and log sinks)").
- **The contradiction:** "The alpha includes" lists every capability except `Proc` and `Rand`. Other texts list them as shipped:
  - at-a-glance item 3 lists `&Proc`;
  - "Not in this alpha" (README.md:469-472) says `Rand` "has its five draws" and `Proc` "starts processes (`command`, `only`)";
  - owner-guide/answers.md:119-120 (item 10) lists "starting processes, clock, random draws";
  - the paper's §2.2 and its abstract list both.
- **Failure scenario:** a launch reader checks "what's in v0" at the README's Status, the list a reader treats as the inventory. They conclude that processes and randomness are not in the alpha. Twenty lines later they find "Not in this alpha" describing both as present. That is the inconsistent v0 surface the brief asks about.
- **Minimum fix:** add both to the parenthetical, e.g. "…, `Clock`, `Proc` for declared child processes (`command`, `only`), `Rand` draws, environment, …". Optionally also list randomness in at-a-glance item 3 ("… the network, other programs and randomness … `&Net`, `&Proc` and `&Rand`"), as the paper's abstract does. That item is copied verbatim, so edit docs/at-a-glance.md and README together.

### B2: owner's guide chapter 3 still calls the rust#57893 forms a find, with the old route count

- **Location:** docs/owner-guide/03-soundness-bugs.md:50-53 ("an agent tried over 40 escape routes against admission and the sandbox. It found two forms of one more admitted route, rust#57893.").
- **The leftover:** this is the wording the cold reader flagged (Q2, Q5, A4). The change fixed it in owner-guide/answers.md:24-32, which cites this chapter as its source [3], and in the README and the paper. It also edited this same file at :177. But this paragraph still:
  - reads as a discovery of rust#57893;
  - says "over 40" where every other text now says 48 (controls included);
  - never says the issue has been open upstream since 2019 with both forms on record.

  The same chapter does say this for #25860 (:21, "open since 2015").
- **Failure scenario:** the repository goes public with this chapter. A Rust reader, or a funder the owner sends to the guide, reads "It found … rust#57893" as a Tide discovery. That is the reading Q2 and fx-qlek.13's resolution (KNOWN upstream) exist to prevent, and the positioning toward Rust makes it costly. The README now says "neither is a Tide discovery", so the chapter contradicts it.
- **Minimum fix:** "An AI red team (Claude Opus) tried 48 routes, controls included, against admission and the sandbox, and found that admission let through two forms of one more known route, rust#57893 (open upstream since 2019, both forms already on record there)." This matches answers.md:24-27.

## Non-blocking findings

1. **The new Evidence lead is hard to parse.** README.md:163-167 nests two appositives ("a property, no accidental … `--tide-grant=unsafe`), and a mechanism, seeded replay; …"). A cold reader loses where "a property" ends. Suggest a colon and two short sentences: "What v0 offers is a property and a mechanism. The property: no accidental ambient authority in admitted safe code, apart from printing, outside the `unsafe` grant and up to rustc's soundness bugs. The mechanism: seeded replay."
2. **The paper's §4.2 definition sits in the wrong place.** In paper/tide-v0.typ:213, "where accidental means code that does not exploit a compiler soundness bug" follows the `unsafe`-grant example, so it reads as defining the grant. Suggest moving it next to "_accidental_": "no _accidental_ ambient authority, where accidental means code that does not exploit a compiler soundness bug, in admitted safe code outside declared trust holes such as …".
3. **The tide-or-rust copy keeps the garble that item 9 lost.** docs/tide-or-rust.md:84 still says "chosen after the screen had seen its one row". The cold reader flagged this phrase as garbled, since the screen had 87 rows. It was fixed in at-a-glance item 9 but not here. Use item 9's wording: "That re-test's pool, one crate the screen had already seen, was chosen afterwards, …".
4. **Figure 14's new caption implies a complete list.** paper/tide-v0.typ:414 says "the v0 core, with the items v0 has not built yet marked planned". The column omits v0-core items that README.md:676-681 lists as not built (packages and per-dependency grants, sealed interfaces, the `tide` commands). It also shows "The test sandbox on Linux" unmarked, although README.md:680-681 says the rest of RUN-3/RUN-5 is unbuilt. Suggest "selected v0-core items, those not built marked planned", or add a "Packages and per-dependency grants (planned)" bullet to the figure record. This is not blocking because the figure makes no claim to be complete.
5. **GOVERNANCE.md still contradicts the corrected texts (list only; single-holder file).** GOVERNANCE.md:13-14 lists "defining architecture" and "writing specifications" as human responsibilities. CONTRIBUTING.md:9-11 (which cites that section), README.md:558-559 and :697-699, and the paper's disclosure now say AI agents wrote the design text. The change moved the contradiction rather than created it, and report.md records it as left open. It needs a scheduled single-holder edit before the RC. GOVERNANCE binds every change, so rewording its human responsibilities may be the owner's call rather than a technical one.
6. **The owner's guide chapter 9 lags the new catch wording.** docs/owner-guide/09-experiments.md:192 still says "One re-test found a catch". Item 9 dropped "found a catch" for "counts one catch". At :210-212 the chapter gives the reversal's cost but not its reason ("so that the documented idiom compiles in a task body", now in the README and research/49). The surrounding text is qualified, so this is a wording-alignment nit.
7. **The paper's closing sentence is unqualified.** paper/tide-v0.typ:431, "Both trials ran under protocols frozen before the first session", has no qualifier, while §4.5 now adds "self-attested" for E3. One clause would make it agree: "…, an order our own records show".
8. **The code is not on the first screen.** The Figure 1 code block (README.md:93-107) comes after the 90-line at-a-glance, so it is not literally on the first screen. The rebuttal holds, since the owner placed at-a-glance under the tagline. The block is still a clear improvement.
9. **One long line.** README.md:560 is a single unwrapped line in a hard-wrapped file. Cosmetic.

## Reasoning

### 1. First screens

- **README:**
  - The tagline's caveat (README.md:3-4) ties "capability-safe" to Rust's "memory-safe", as the owner's "keep the caveat tied to it" requires.
  - Item 3's "up to rustc's soundness bugs (item 8)" closes the cold reader's overclaim 4.
  - Item 9 is no shorter, but it now opens accurately. "Recorded their methods in advance and published every count" is supported: research/37:15-30 records the protocol hash on the bead before the search, self-attested.
  - "Held privately and appears publicly only as a count" matches the counts-only decision and research/52:18-27.
  - Item 9 now carries the full research/49 §1 negatives and the qualified order.
- **tide-or-rust:** its first screen is unchanged apart from the scoped claim. The new cap-std, cackle and turmoil paragraph (:101-106) and the pinned-nightly bullet (:145-148) answer attacks 4 and 5 briefly, and each cites the answers item it rests on (I checked items 2, 4 and 9).
- **Paper:**
  - The PDF compiles without warnings, and pdftotext shows no U+2032 or U+2018.
  - The abstract is split into three paragraphs without cutting anything.
  - The upstream credit in §4.2 is supported by the fx-qlek.13 tracker comment: Niko Matsakis on #57893 (2019-09-06), and rust#152783 by theemathas (2026-02-18), closed as a duplicate.
  - "Neither form is a Tide discovery" holds.
- **Overclaims and underclaims:** I found no new overclaim. The texts now consistently scope the property to admitted safe code outside the `unsafe` grant, apart from printing, up to rustc's soundness bugs:
  - README.md:163-167;
  - paper:213;
  - tide-or-rust:23-24;
  - answers item 2;
  - owner-guide chapters 1 and 3.

  The paper's "accidental" now means not exploiting a compiler soundness bug, which is narrower and more honest than the cold reader's "deliberately". "No experiment yet shows that Tide makes reviewers or models better" is accurate given E3 and E1, and is not an underclaim, because the property and replay are stated next to it.

### 2. Rebuttals

The rebuttals hold:

- the first-use glosses (owner rule, fx-qlek.16 and fx-qlek.7 acceptance);
- tracker IDs (defined at first use; the repository goes public with `.beads/`);
- the public catch as the lead (owner decision PUBLIC CATCH, 2026-11-19);
- the repeated qualified order (binds "everywhere");
- "move faster": the owner's own wording, "advance rust faster", sits in a sentence that also says "complements Rust and is not a fork", so it is not "faster than Rust";
- the 25,000 budget (D14 plus the owner's decision against a 30k cap);
- the paper's digressions (owner-directed research).

No fix weakens a true claim. The figure 12 cell went from "yes, by design" to "planned". That is a correction: per-dependency grants are planned (fx-x2oo), and the second line still says "by design".

### 3. Agreement across texts

These agree across the texts:

- **Red team:** 48 routes. README: 45 + 6b + 2 controls. Paper Figure 8: 42 + 2 + 2 + 2. The record's header says "48 routes tried, counting the accepted controls". Answers item 3 says 48. Chapter 3 says "over 40" (B2).
- **Rounds:**
  - README.md:224 lists .5-.7 and .9-.12 as caught nothing, and .13 counting the same catch once.
  - This matches search.json and the paper's Table 2.
  - research/51 §2 confirms four rows and "counted once".
  - research/52 says "No catch".
- **Conformance:** 393/390/3 in 02-admission, answers and 10-v0. It matches run-record.txt, which Codex verified byte for byte against S4's log.
- **E1:** the rounding (0.95, 1.46, 1.29-1.64) is right, and the cached tokens, 5,156,350 against 569,596, match gate-report.md:115. tide-or-rust matches.
- **Trust holes:** consistent everywhere. The exception is the v0 surface: Proc and Rand are missing from the README Status list (B1).
- **Planned items:** consistent, apart from figure 14's selection (N4).

### 4. Leftovers

I searched outside research/, reviews/, trials/ and archive/ for the old phrasings:

- "honest limits", "fixed their methods", "described honestly", "yes, by design", "what v0 delivers", "therefore exact";
- `<repository-url>`;
- 392 and 389;
- "found a catch", "over 40 routes", "every other escape route", "published whatever";
- "write(ing) specifications".

What remains:

- B2, in chapter 3;
- N6, in chapter 9;
- N5, in GOVERNANCE;
- 02-admission.md:193, "389 in all" after fx-qlek.5, which is history and correctly kept;
- 05-seeds-and-simulator.md:139, "The honest limits:", an unrelated chapter label (optional);
- 09-experiments.md:146, "published whatever it is", which states the E3 protocol's rule and is correct.

The rest:

- **design/reference.md and design/reference-companion.md:** neither quotes any changed passage. No hits for "accidental", "over 40", "57893", "capability-safe" or "specifications". List only; nothing is needed.
- **Guard tests:** no test in crates/ reads README.md, at-a-glance or tide-or-rust beyond code_prefix. code_prefix passed on 8d7181c (report.md). The new README `rust` block matches paper/figures/excerpts/code-rust.rs, code-tide.rs and diagnostic.txt.
- **README and at-a-glance:** the README's at-a-glance block is verbatim docs/at-a-glance.md (checked with python3 -I).
- **Install:** the README and docs/install.md clone lines match. Codex verified the clean-install script's substitution.

### Checks run on 8d7181c

- `typst compile --root . paper/tide-v0.typ /mnt/build/tide-agents/fx-qlek.7-target/s5o/tide-v0.pdf`: exit 0, no warnings.
- `node paper/figures/build.mjs --check`: "figures match their records (22 files)", exit 0.
- pdftotext of that PDF: no U+2032 and no U+2018.
- The README's at-a-glance block equals docs/at-a-glance.md.

I made no edits, commits or `.beads/` writes, started no subagents and used no network. The only file I wrote is this report.

DONE
