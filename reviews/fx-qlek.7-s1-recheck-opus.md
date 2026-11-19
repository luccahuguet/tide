Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidate 45eee75 (re-check)

Scope: a finder re-check of the items in target/work/s1-opus-review.md that tide-lead's brief (target/work/brief-s1-fixes2.md) took: B1–B3 and non-blocking 1–4, 6, 7 and 9, plus the placement of 5 and 8. I read `git diff f897455 45eee75` and checked it against the cited sources. Read-only. This is not a new full review.

## Items

**B1: resolved.** docs/owner-guide/09-experiments.md:187–190 now reads "for later rounds, an OpenTimestamps proof shows each method file existed by a given time, but when the round looked still rests on our records". This matches the 2026-11-18 19:48 qualifier and at-a-glance item 9's last sentences. The overstatement is gone.

**B2: resolved.** At-a-glance item 9 (README.md:66–86, docs/at-a-glance.md:61–81):
- It defines "registration" at first use: "a method recorded in advance, before the re-test ran".
- It introduces "anchored" inside its definition ("an OpenTimestamps proof, a timestamp anchored in a public blockchain, which shows that a file existed by a given time") before the qualified-order sentence uses it.
- The binding sentences are intact: the gap, screen, fix and new-registration sentence; "Every check is consistent with the registration being anchored before the fix landed, and independent proof of that order is not established"; and "admission and type-checking only; no port was executed".
- The README block and docs/at-a-glance.md are byte-identical (`diff`).

**B3: resolved.** docs/owner-guide/02-admission.md:161–162 says "392 such cases on `main` today, 389 passing and 3 awaiting rules not built yet". That matches conformance/run-record.txt ("392 cases: 389 PASS, 3 AWAITS CHECKER, 0 FAIL") and ch. 10 and answer 9. The dated note in docs/owner-guide/README.md now names the conformance count and ch. 10's re-run outputs.

**Non-blocking 1: resolved.** The first Evidence paragraph (README.md:143–152) now reads "a catch on a separate re-test, not in the screen it re-tests". It states the screen's 0 of 87 first, then says "On the later re-test, ... meets the screen's unchanged caught rule". The paper abstract (tide-v0.typ:124) likewise gives 0 of 87 first and places the catch "on that separate re-test".

**Non-blocking 2: resolved.** At-a-glance now separates the screen ("caught none of them") from "That separate re-test, chosen after the screen had seen its one row, found a catch". This agrees with research/49 §1: a new test, with the pool chosen after the rows were seen.

**Non-blocking 3: resolved.** Every account now says the faithful port was already rejected in the screen and that leg 2 is what changed:
- README legs (:154–162);
- at-a-glance;
- ch. 9's catch bullet and its call sentence (:218–230);
- answers.md 7 (:82–83);
- paper §4.2 (:210).

This matches research/47's row (leg 1 passes; leg 2 fails on E0277) and research/49 §2. No "port is now rejected" wording remains (grep over README, docs and paper).

**Non-blocking 4: resolved.** Ch. 9:301 now says "it gives no evidence of one", which matches report.md ("gives no evidence that the trust diff helps").

**Non-blocking 6: resolved.** The README's replay bullet (:101–104) now lists declared child processes (`sim.world.proc`), consistent with README.md's fidelity table and guide ch. 5.

**Non-blocking 7: resolved.** README.md:191–193 now explains B23.128 as "one unresolved row whose required evidence, an upstream pull-request thread, was unavailable". This matches research/44:528–537 ("The required PR thread is unavailable ... does not become a completed negative").

**Non-blocking 9: resolved.** README, ch. 9 and paper §4.2 now say "task authority (`bg`, the task scope it is handed)". This matches research/49 §2 and §8 (`bg: Scope<'s, '_, TaskError>`, declared authority `bg`, `clock`, `proc`).

**Non-blocking 5 and 8: placed as briefed.**
- plan.md row 1.19 records docs/driver.md :230 and :364 as todo, after fx-qlek.19 lands and before S6.
- The status-page item stays in plan.md's site list as a must-land before release.

**Plan statuses:** rows 1.5 and 1.16 are now done and match the tree. The new rows 1.17 and 1.18 (done) and 1.19 (todo) are accurate.

**New problems introduced by the fixes:** none blocking. `git diff --check 6287b13 45eee75` passes. One formatting nit: README.md's Leg 1 bullet now has one line over 80 columns (:154–155). It doesn't affect rendering.

## Verdict

**Approve.** All three blocking findings and the seven non-blocking items taken are resolved, accurately to their sources. The binding wording is intact in every account. The fixes introduced no new source mismatch, overclaim or wording-rule break. This approves stage 1's texts only; S2–S6, the site status page and docs/driver.md remain tracked in plan.md.

DONE
