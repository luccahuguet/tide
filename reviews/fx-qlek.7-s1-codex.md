Model: gpt-6.1-sol (high) · Date: 2026-11-19 · Candidate 2b0bdb9 (docs, pre-RC stage 1)

## Verdict

**Changes required.** Five blocking findings. Confidence: high on the source mismatches; the chronology finding applies the brief's explicit requirement to use both parts of the qualified order wherever the catch appears. The patch needs wording corrections, not a change of direction.

## Blocking findings

### B1. Short catch accounts omit part of the binding qualification

**Locations:** paper/tide-v0.typ:124; docs/owner-guide/answers.md:80; docs/owner-guide/09-experiments.md:217.

**Evidence:** The abstract and answer 7 say only that independent proof that registration preceded the fix is not established. They omit the required positive qualification: every check is consistent with the registration's anchoring preceding the fix's landing. The guide's suggested sentence to say on a call omits both chronology qualifications, despite presenting itself as reusable launch wording. The complete accounts at README.md:155, paper/tide-v0.typ:212 and guide chapter 9:212 contain both parts. Research/49 §3 distinguishes anchoring from registration and implementation from landing; the 2026-11-19 07:12/07:22 bead comments and this brief bind the launch wording to that distinction. A qualification elsewhere in the chapter does not accompany the sentence the guide tells its reader to quote.

**Minimum fix:** Include both clauses in the abstract, answer 7 and the suggested spoken sentence: every check is consistent with anchoring before the fix landed; independent proof of that order is not established. Keep the gap/screen/fix/new-registration wording. Also make the short summaries' scope explicit: admission and type-checking only, with no port executed. That scope currently appears in the full accounts but not these summaries or at-a-glance item 9. S1 row 1.1 is not complete as marked.

### B2. The README and owner guide omit the intentional reversal

**Locations:** README.md:151; docs/owner-guide/09-experiments.md:204 (also the summary at answers.md:80).

**Evidence:** These passages describe adding a missing conversion, without saying that an earlier review deliberately removed the same conversion. Research/49:43–49 names fx-nkv.3's removal, the E0283 inference regression that motivated it, the owner's reversal, and the cost of naming the error type at 17 scope sites. Both earlier reviews support that history: reviews/fx-nkv.3-r1-codex.md B1 and reviews/fx-nkv.3-r1-opus.md B1. The brief expressly requires the reversal as research/49 §1 states it. The paper includes it at :214; the other full public explanations do not.

**Minimum fix:** Add the intentional removal/reversal and its inference cost to the README's full account and guide chapter 9. Let the short answer point to that explanation. The paper's existing sentence supplies a sufficient compact pattern. This is another missing part of S1 row 1.1.

### B3. The copying explanation treats every Vec as authority-free data

**Location:** docs/owner-guide/01-capabilities.md:56–57.

**Evidence:** “clone() on a String or a Vec makes an independent value. No authority moves: data carries none” does not constrain the vector's elements. A Vec<Fs> contains authority, and cloning it clones its capability handles; a Vec<&Fs> copies references to the same authority. Reference §6.6 (design/reference.md:469–480) makes collection purity depend on its parts, and §6.1 (:350–352) permits capability Clone with the same authority. Research/45 §1 preserves those rules rather than giving collections an authority-free exemption. The unqualified example undermines the distinction this fold-in is meant to teach.

**Minimum fix:** Use Vec<u8> instead of an unrestricted Vec and say that these example values contain no capabilities. Do not promise independent contents for arbitrary element types. S1 row 1.4 needs this correction before it meets its consistency check.

### B4. A signature does not show how far a capability was narrowed

**Location:** docs/owner-guide/01-capabilities.md:70–71.

**Evidence:** The added sentence says a signature tells the reader “which kind of authority a function can reach, and how narrow it is.” Reference §6.1 explicitly says attenuation returns the same type (design/reference.md:353–354). A function taking &Fs has the same signature whether its caller passes a filesystem-wide handle or a read-only directory handle. Design/03 §4.1 computes declared authority from reachable kinds and access paths, not runtime directory/host restrictions. Guide chapter 7:161–163 also says today's diff cannot show a wider directory caused by less attenuation. The new sentence contradicts both sources.

**Minimum fix:** End the sentence after the reachable authority kind, then say that the actual bounds depend on the capability value the caller passes. This also prevents S1 row 1.4 from being considered complete yet.

### B5. The search-round summary drops fx-zu4f.10's unresolved outcome

**Location:** README.md:171–174.

**Evidence:** The sentence groups fx-zu4f.5–.7 and .9–.11 as “documented negatives,” without an exception. Research/44-fx-zu4f.10-concurrency-round.md:61–62 and :77–85 retains one unresolved arm-B row, B23.128. Its :530–537 explains the missing required evidence and explicitly says it “does not become a completed negative.” Zero catches is supported; turning that whole round into an unqualified negative loses the source's distinction.

**Minimum fix:** Keep the zero-catch summary but state that fx-zu4f.10 retains one unresolved row, and link research/44 for that qualification. The status-page link alone does not state it in this account.

## Non-blocking findings

1. **Cold-reader terms and tracker IDs:** README.md:66 introduces “port” without explaining that it means code adapted to Tide; docs/at-a-glance.md:61 repeats it. The new README evidence section uses fx-zu4f.8, fx-qlek.18 and fx-zu4f.15 without identifying them as tracker IDs. The paper already explains that ID format at :198. Guide chapter 9:184 also opens with “beads.” Prefer ordinary descriptions first, with IDs as labelled references. Paper-abstract first-use definitions remain part of S2 row 2.4, not a completed S1 deliverable.
2. **Ambiguous antecedent:** README.md:74 and docs/at-a-glance.md:69 say “For the pilot and four of the searches, that order rests on our own records” immediately after discussing anchoring before a std fix. Those older experiments' qualification concerns methods before examination, not that fix. Repeat the intended order in plain words.
3. **Later-stage claims remain pending:** The builder explicitly defers the paper's existing tone problems and research/45's corrected D7/D9 parallel-region fold-in to S2 (plan rows 2.5 and 2.9). Those rows are honestly marked todo. The README heading at :105, the existing conditional “unforgeable” discussion in the paper, the site/status synchronization, and final links still require their recorded later-stage checks. This verdict is not approval of those outstanding release claims. In particular, plan.md's site inventory says its status page still lacks research/49, while the texts say it lists every round; resolve that synchronization before release.

## Reasoning

I read AGENTS.md, GOVERNANCE.md, the bead and its comments first, then the complete 11-file diff, the touched passages with context, target/work/report.md and target/work/plan.md. I compared the catch with research/49 §§1–4 and research/47's result, registration and capability records; the E3 edits with trials/e3-pilot/report.md; and the borrowing fold-in with research/45 §1, its Opus review's fold-in decisions, and reference §§6.1/6.6. I checked the other-round summary against research/37, /38, /39, /41, /44 and /46. The Net, Proc and simulator updates agree with the reference, research/40, /42 and /48 and their implementation surfaces. The upstream-known projection wording follows the bead's resolved 2026-11-18 12:40 record; I did not repeat that upstream investigation.

The main catch facts hold: TD0602 twice and TD0501 three times, clean admission of the capability-passing version, byte-identical reused ports, task/clock/process authority, the original 0 caught of 87, no execution and no intent-detection claim. The detailed chronology preserves the 61-minute gap and roughly two-hour timestamp tolerance. I found no new affirmative first-time-detection, population-rate, unforgeability or “Tide prevented” claim. Negative reminders about forbidden claims do not assert those claims.

Read-only checks confirmed that the at-a-glance copy is byte-identical; answer 7 and chapter 9 quote E3's opening verbatim after whitespace folding; docs contain no remaining “no data” phrase; the conformance record contains 389 PASS and 3 AWAITS CHECKER entries; the reference has 10 literal “(planned)” markers; and br --no-db list --label v0-gap matches the printed 11-item list. The driver lists 17 rustc internal crates. The research/49 edit changes only the title number. git diff --check passed.

S1 rows 1.2, 1.3, 1.5, 1.6 and 1.14 have the claimed textual results. Rows 1.1 and 1.4 are marked done but fail the requirements identified above. The research/45 paper fold-in remains explicitly assigned to S2, so its absence does not falsely complete an S1 row.

Review identity: HEAD 2b0bdb99084a0eaa61650ee502d2391bfe1e720d, parent 6287b133536eb04e567288025a0a48cfca1aad59; SHA-256 of `git diff --binary 6287b13 2b0bdb9`: `509711f759c278901f855801dcea575f98e7ecb9c3f3bf622c11bad2bd29109a`. The tracked tree was clean. Commands ran in the supplied Linux worktree. Only this report was written; no build, payload execution, PDF rendering, commit, Beads write, network check or contact occurred. The checks establish source consistency and text structure, not a fresh runtime or rendered-PDF result.

DONE
