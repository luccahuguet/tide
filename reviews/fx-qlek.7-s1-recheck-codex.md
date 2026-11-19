Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate f897455 (re-check)

## Scope

Finder re-check of B1–B5 and non-blocking items 1 and 2 in `target/work/s1-codex.md`, against `git diff 2b0bdb9 f897455`. I read the original review first, then the fix brief, the builder's “Round S1 fixes” account, the S1 plan statuses, and the cited sources. This is not a new full review or approval of the deferred release work.

## Items

### B1 — Resolved

The paper abstract (`paper/tide-v0.typ:124`), answer 7 (`docs/owner-guide/answers.md:80–88`), the suggested spoken account (`docs/owner-guide/09-experiments.md:224–230`), and at-a-glance item 9 (`README.md:66–75`; `docs/at-a-glance.md:61–70`) now carry both clauses: every check is consistent with registration anchoring before the fix landed, and independent proof of that order is not established. Each also explicitly limits the re-test to admission and type-checking, with no port executed. The gap → screen → fix → new registration account remains intact.

These match the binding wording in `target/work/brief-stage1.md` and `brief-s1-fixes.md`, research/49 §3 (`research/49-fx-zu4f.15-chrono-anchor-retest.md:238–248`), and its execution limit at :61–63. The revised wording distinguishes anchoring from registration and landing from implementation; it does not assert independent proof of the order. S1 row 1.1's missing short-account qualifications are supplied.

### B2 — Resolved

`README.md:158–164` and `docs/owner-guide/09-experiments.md:205–210` now disclose the deliberate fx-nkv.3 removal, the E0283 inference problem for unannotated `tasks.scope(..)?`, the owner's 2026-11-19 reversal, and the resulting 17 scope sites naming their error type. Answer 7 (:86–88) points to chapter 9 and research/49 for the history.

The account matches research/49 §1 (:43–49) and §4 (:296–300). Both originally cited reviews support the reason for removal: `reviews/fx-nkv.3-r1-codex.md:53–73` and `reviews/fx-nkv.3-r1-opus.md:41–65`. The paper's existing compact account at :214 is preserved. S1 row 1.1's reversal requirement is met.

### B3 — Resolved

`docs/owner-guide/01-capabilities.md:56–59` replaces unrestricted `Vec` with `Vec<u8>`, explicitly says the example values hold no capabilities, and distinguishes cloning a `Vec<Fs>` as cloning capability handles. It no longer promises independent, authority-free contents for arbitrary vector elements.

This agrees with structural purity in `design/reference.md:469–480`, capability `Clone` with the same authority at :350–352, and research/45 §1's retained distinction (`research/45-bend-hvm-ideas-for-tide.md:66–75,98–105`). S1 row 1.4 accurately records the narrowed example.

### B4 — Resolved

`docs/owner-guide/01-capabilities.md:72–75` now says a signature shows the reachable authority kind, while actual bounds depend on the capability value passed. The same `&Fs` signature can receive filesystem-wide authority or one read-only directory.

This matches attenuation returning the same type (`design/reference.md:353–354`), declared authority computed from reachable kinds and access paths (`design/03-authority-concurrency.md:1301–1311`), and the trust diff's inability to show less attenuation (`docs/owner-guide/07-trust-diff.md:155–163`). The fix removes the static narrowing overclaim; S1 row 1.4 reflects it.

### B5 — Resolved

`README.md:180–183` retains the zero-catch summary, explicitly excepts fx-zu4f.10's unresolved B23.128 row from the negative classification, and links research/44. That preserves the distinction in `research/44-fx-zu4f.10-concurrency-round.md:61–62,77–85,530–537`: no catch was recorded, but unavailable required evidence does not become a completed negative. S1 row 1.15 matches the correction.

### Non-blocking 1 — Resolved for S1

“Port” is defined at its first occurrence in both `README.md:66–67` and `docs/at-a-glance.md:61–62` as a rewrite in Tide; the full README account repeats the explanation at :144–145. `README.md:143–144` explains tracker IDs, and :158–161 labels the std fix, re-test and earlier review before their issue references. Chapter 9 opens with ordinary descriptions and “tracker issue” (:184–194), replacing bare “beads”; its fix and re-test are labelled at :212–213. The paper labels those two actions at :210, after its existing tracker-ID definition at :198. These changes address the original cold-reader finding without changing the underlying research account. The paper abstract's broader terms pass remains S2 row 2.4, as the original finding allowed.

### Non-blocking 2 — Resolved

`README.md:76–77` and `docs/at-a-glance.md:71–72` explicitly name the self-attested claim: methods were fixed before results were examined. The sentence no longer uses “that order” to point back ambiguously to anchoring before the std fix landed.

That distinction matches the original source qualifications: research/37 :22–32, research/38 :84–94, research/39 :55–63, research/41 :25–36, and the pilot's own run record (`trials/e3-pilot/report.md:482–493`). The change clarifies the antecedent without strengthening the evidence. S1 row 1.16 matches the result.

## Verdict

**Approve.** All seven requested items are resolved. Confidence: high. I found no new source mismatch, wording-rule break or overclaim introduced by these fixes. Non-blocking item 3 and the S2–S6 release checks remain outside this re-check; this verdict does not complete them.

## Verification and limits

Reviewed HEAD `f8974558e7abd872fa05dac73632173aa44ce078` in the supplied Linux worktree. SHA-256 of `git diff --binary 2b0bdb9 f897455`: `6bc54a591ad0c50373c445ed5789a1886868dc454f881a95025591df61f6c4ca`. The tracked tree was clean. `git diff --check 2b0bdb9 f897455` passed. A read-only Python comparison confirmed the complete README at-a-glance block matches `docs/at-a-glance.md` byte for byte after trimming boundary whitespace; whitespace-folded inspection confirmed the explicit no-execution scope in all five requested files.

Only this report was written. No build, payload execution, PDF rendering, commit, Beads write, network verification or contact occurred. This establishes textual consistency with the cited records, not new runtime or rendered-PDF proof.

DONE
