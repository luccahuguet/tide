Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate f897455 (docs, pre-RC stage 1)

## Verdict

**Changes required.** Three blocking findings, each a small text fix. The direction is right. The catch is accurate to research/49 everywhere it appears, and the binding wording is present in every account. What blocks: one new overstatement of the later search rounds' evidence (guide ch. 9), two terms used before they are defined in the README's first screen (the owner's terms rule, which also covers at-a-glance), and one stale conformance count that now contradicts the counts this stage updated. Confidence: high on B1 and B3; medium-high on B2, which turns on applying the 2026-11-18 19:51 terms rule to at-a-glance, as that comment does.

## Scope

- Read AGENTS.md, GOVERNANCE.md (Merge Rule), `br --no-db show fx-qlek.7` and all its comments. The binding comments were 2026-11-19 05:14, 07:12 and 07:22, plus the 2026-11-18 rules: 14:06 and 19:48 (pre-registration and OTS qualifiers), 19:51 (terms), 20:02 (tone), 12:40 (#57893), 13:53, 22:48 and 2026-11-19 00:12 (research/45).
- Read target/work/s1-codex.md, s1-recheck-codex.md, report.md, and the S1 rows of plan.md.
- Read the full diff `6287b13..f897455` (11 files).
- Checked claims against research/49 §§1–4 and §9, research/47's chrono_anchor rows, research/45 §1, design/reference.md §6.1 and the Net/sim lines, and trials/e3-pilot/report.md (first paragraph, ceiling, observations, limits).
- Read-only checks: the at-a-glance copy matches docs/at-a-glance.md (diff, identical); `research/49` hits README, the paper, at-a-glance, ch. 9 and answers.md; no "no data" phrase remains under docs/; conformance/run-record.txt reports 392 cases (389 PASS, 3 AWAITS CHECKER).
- Not done: no build, no PDF render, no network, no repeat of the Codex pass.

## Blocking findings

### B1. Guide ch. 9 implies the later search rounds' order is independently established

**Location:** docs/owner-guide/09-experiments.md:186–188.

**Evidence:** "Each fixed its rules before it looked (for the earliest rounds, that order rests on our own records only)". By contrast, this tells the reader that for the later rounds the order rests on more than Tide's records. It doesn't. From fx-zu4f.8 on, an OpenTimestamps proof shows only that each method file existed by its attestation time (bead comment 2026-11-18 19:48: "OTS-anchored from .8 on, meaning only that each file existed by its attestation time"). When each round looked at its candidates is still self-attested. At-a-glance item 9, edited in this same stage, says exactly that: "When those searches examined their candidates still rests on our records" (README.md:79–80). The new sentence overstates the later rounds' evidence and contradicts the README.

**Minimum fix:** Complete the parenthesis. For example: "(for the earliest rounds, that order rests on our own records only; for later rounds, an OpenTimestamps proof shows each method file existed by a given time, but when the round looked still rests on our records)". Or point to at-a-glance item 9's wording.

### B2. At-a-glance item 9 uses "registration" and "anchored" before defining them

**Location:** README.md:70–75; docs/at-a-glance.md:65–70 (the same block).

**Evidence:** The new sentences say "re-tested under a new registration" and "Every check is consistent with that registration being anchored before the fix landed". This is the README's first screen. "Registration" is never defined there; the reader has to infer it from "fixed their methods before looking at results". "Anchored" is explained only three sentences later, in the existing sentence "an OpenTimestamps proof, a timestamp anchored in a public blockchain". The owner's rule (comment 2026-11-18 19:51) is that every term is defined in plain words at its first appearance in reading order, and that the same rule and check apply to "Tide at a glance". Plan row 1.16 marks this stage's cold-reader terms done. For a skeptical cold reader, the qualified-order sentence is the least clear sentence in the catch, and it is the one that most needs to be clear.

**Minimum fix:** Define both terms at first use and keep the binding wording. For example: "...and re-tested under a new registration (a method recorded in advance, before the re-test ran). That registration carries an OpenTimestamps proof, a timestamp anchored in a public blockchain; every check is consistent with it being anchored before the fix landed, and independent proof of that order is not established." The later OTS sentence can then refer back to the definition. Keep README and docs/at-a-glance.md byte-identical.

### B3. Guide ch. 2 still says 389 conformance cases "on main today", contradicting ch. 10 and answer 9

**Location:** docs/owner-guide/02-admission.md:161–162. It contradicts docs/owner-guide/10-v0-and-planned.md:31 ("392 conformance cases") and answers.md:107–109 ("392 conformance cases ... 389 pass and 3 await").

**Evidence:** conformance/run-record.txt:400 reads "392 cases: 389 PASS, 3 AWAITS CHECKER, 0 FAIL". This stage corrected the same stale count in answers.md item 9 and ch. 10, and plan row 1.5 ("Stale claims about ... counts") is marked done. Ch. 2's "389 such cases (on `main` today)" is the sibling leftover, so the guide now gives two different current totals. (Ch. 2:193, "389 in all" after the red team, is history and is correct as written.)

**Minimum fix:** Change ch. 2:161 to "392 such cases" (optionally "389 pass and 3 await rules not built yet", matching answer 9). Add the conformance count to the dated-update note in docs/owner-guide/README.md:9–11, which today lists only E3, Proc, Net, the simulator models and the re-test, although ch. 10's re-run outputs and counts changed too.

## Non-blocking findings

1. **The first Evidence sentence can read as a catch in the original screen.** README.md:139–143: "RUSTSEC-2026-0039 ... meets the caught rule of our pre-registered screen of all 87 malicious-crate advisories". A skimmer can read this as "the 87-advisory screen caught it". The preceding "records a catch on a re-test" and the later "0 caught of 87" correct it, but the sentence itself should carry the timing: "meets, at the re-test, the caught rule of ...". The paper abstract (tide-v0.typ:124) does the same in two sentences ("meets a pre-registered screen's caught rule ... That came on a re-test"). Merging them, "on a re-test meets ...", is safer.
2. **"One search found a catch on a re-test"** (README.md:66; at-a-glance:61) blends fx-zu4f.8 with the separate re-test, which research/49 insists "does not revise, rerun or replace fx-zu4f.8". Item 9 also opens with "fixed their methods before looking at results", next to a catch whose one-row pool was chosen after the screen had seen it. The gap sentence that follows conveys the sequence honestly, so this is not a binding-rule break. "A separate re-test, chosen after our screen had seen the row, found one catch" would remove the blend.
3. **"The port is now rejected"** (ch. 9:225–226; answers.md:83–84). The faithful port was already rejected in fx-zu4f.8 (leg 1 passed there, research/47 row). What changed is leg 2. Prefer "the faithful port is rejected, and the version that declares its authority is now admitted."
4. **"It found none"** (ch. 9:298). The report says the pilot "gives no evidence that the trust diff helps", and that "at this ceiling a benefit and its absence give nearly the same counts". "Found no evidence of one" matches the source; "found none" reads as evidence of absence. The error runs against Tide, not in its favor, so it doesn't block.
5. **docs/driver.md is stale and has no plan row.** docs/driver.md:230 says "`Rand` and `Proc` remain identities" and :364 says "Simulated HTTP/randomness ... remain planned". The file is held by fx-qlek.19, and report.md names it as an open item, but plan.md, the durable checklist, has no row for it. Add one (after fx-qlek.19, before S6).
6. **README.md:99**, the Evidence lead's list of simulator models, omits declared child processes, while README.md:381 and guide ch. 5 include them. This is pre-existing, but it is the line a reader sees first.
7. **"B23.128"** (README.md:182) is an unexplained row label in the README. "one row whose required evidence was unavailable (B23.128)" says what it means.
8. **Status page.** "The status page lists every round" (README.md:80–82 and :183–184) is not yet true for fx-zu4f.15. plan.md tracks this as a must-land site item (Codex non-blocking 3). It must land before the README goes public.
9. **"Task authority"** (README.md:153, paper §4.2, ch. 9). research/49 records the declared authority as `bg` (sync), `clock` and `proc`, where `bg` is a task scope. "Task" is a fair gloss, but the S6 claims audit should know the artifact's word is `sync`.

## S1 plan statuses against the tree

- 1.1 done: confirmed. Every account carries both chronology clauses, admission-and-type-checking-only and the gap sentence. The full accounts carry the fx-nkv.3 reversal. There is no affirmative first-time or rate claim.
- 1.2, 1.3, 1.4, 1.6, 1.14, 1.15 done: confirmed.
- **1.5 done: not complete** (B3).
- **1.16 done: not complete** (B2).
- 1.7 to 1.13 are honestly marked partial, todo or n/a.

## Reasoning

The core facts hold against research/49:
- TD0602 ×2 and TD0501 ×3 at the cited calls;
- clean admission of the capability-passing port, with port files identical byte for byte to fx-zu4f.8's;
- the advisory text (`.env`, `timeapi.io`);
- "Tide found nothing unaided" and "the same processes with the same arguments";
- the 61-minute gap inside the roughly two-hour tolerance;
- fx-zu4f.8 at 0 caught of 87;
- the fx-nkv.3 removal for E0283, the 2026-11-19 reversal and the 17 annotated scope sites.

The qualified order appears in every account in the approved form: consistent with anchoring before the fix landed, independent proof not established. "First time" appears only in negations: "Not a first-time catch" and "Never say Tide caught it the first time". No rate is claimed.

The E3 quote matches report.md's first paragraph verbatim, and the ceiling arithmetic (1.5 × 67 > 72) and the 11-of-12 ties match the report. The research/45 fold-in matches reference §6.1 (not Copy; Clone with the same authority, provisional; attenuation returns the same type) and research/45 §1 ("who may duplicate"). The Codex B3 and B4 fixes are correct. The Proc, Net and simulator updates match reference §6.1 and the sim lines (Net `get`/`resolve`; Proc `command` with `only`, `exit` planned; `world.http` and declared process models).

The three blockers share a cause. Edits made in one place left the same claim different in another. B1 is a new parenthesis whose contrast implies more than at-a-glance says. B2 is new sentences inserted ahead of an existing definition. B3 updated one count but not its sibling. None changes the direction of the change, and each fix is a sentence or a number.

DONE
