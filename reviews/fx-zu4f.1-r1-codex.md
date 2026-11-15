Model: gpt-6.1-sol · Effort: high · Date: 2026-11-15

## Verdict

request_changes — high confidence. The research direction is sound, but the counts, evidence descriptions and ranked hyper brief need correction.

## Scope

Read-only review of `ac7124b40d9ae7bd994a1066ae5e29823ac7a11f`, adding only `research/33-case-study-candidates.md`.

Read AGENTS.md, GOVERNANCE.md, both beads and their directions, and design/reference.md. Verified all eight hyper matching rows, the sqlx advisory, the other cited public issues and PRs, and project licenses. Checked seven Eon rows through `br --no-db show`, including six corresponding fix diffs through `git show`.

No files changed. No builds or heavy commands ran. No held-out E1 materials or excluded directories were opened.

## Blocking

1. **An ineligible candidate contributes to the matching headline.**
   `research/33-case-study-candidates.md:164`

   **Evidence:** eon-2yj90 appears among the 18 matches and contributes to class 4’s count of three, although lines 174–176 explicitly identify its loader as outside-contributor code and therefore ineligible. The coordinator’s direction also says to keep it out. Its qualified “yes” further makes the table’s v0 column inconsistent with a simple count of ten yes verdicts.

   **Fix:** Move it to the exclusions or rejected candidates, outside the eligible matching table. Recalculate the affected counts and shares throughout the note, distinguishing eligibility from expressibility.

2. **The headline overstates the reading performed.**
   `research/33-case-study-candidates.md:120`

   **Evidence:** Lines 91–94 admit that some descriptions were read only through their first 650 characters. Nevertheless, the table, share discussion and outcome repeatedly describe all 40 as “read in full.”

   **Fix:** Call these “bodies inspected and classified,” state how many were partial reads, and reserve “read in full” for records actually read completely. Alternatively, complete those reads before retaining that claim.

3. **Normal scope completion does not cancel the reaper.**
   `research/33-case-study-candidates.md:374`

   **Evidence:** The fixed hyper brief says the reaper is cancelled when its scope ends. Reference §7.1 instead requires scope exit to wait for every child. Normal completion does not initiate cancellation; §7.2 describes explicit cancellation and deadlines. An infinite sleeping reaper therefore keeps this proposed scope waiting.

   **Fix:** Specify an explicit shutdown transition, such as the caller invoking `bg.cancel()` after the workload, and show the reaper handling cancellation at `clock.sleep`. State that scope exit then joins the terminated child.

4. **Explicit runtime injection is overstated as eliminating the lifecycle bug.**
   `research/33-case-study-candidates.md:356`

   **Evidence:** The statement that hyper 1.x “fixed the class by design” covers all three clustered bugs, including #2892. However, hyper’s [`Executor` interface](https://github.com/hyperium/hyper/blob/master/src/rt/mod.rs) has no structured lifetime guarantee; its own example uses `tokio::spawn`. Supplying an executor does not ensure that clients and their dispatchers share a lifetime. [Issue #2892](https://github.com/hyperium/hyper/issues/2892) supplies a reporter’s cross-runtime diagnosis, not proof that runtime injection eliminates it.

   **Fix:** Describe timer/executor injection as removing the ambient runtime dependency. Present Tide’s scoped lifetime rule separately, and qualify #2892’s diagnosis as reported or inferred. Remove the claim that hyper 1.x eliminated the entire cluster by design.

5. **The open Eon sibling uses a superseded bound as current evidence.**
   `research/33-case-study-candidates.md:148`

   **Evidence:** `br --no-db show eon-pimct` explicitly says `TERMINAL_START_TIMEOUT` is now **15 seconds** and that the five-second observation is historical. The row says it “still” waits up to five seconds, and the ranked sibling brief repeats that bound at line 337.

   **Fix:** Separate the historical five-second observation from the current 15-second bound. Identify which revision and bound the sibling model represents.

## Non-blocking

1. **The overflow advisory is a runtime check, not an admission rejection.**
   `research/33-case-study-candidates.md:287`

   **Evidence:** Class 5 is defined as a pattern an admission rule rejects. Reference §4.1 instead makes dynamic arithmetic overflow panic at runtime; it supplies no admission diagnostic for this parser. The [advisory](https://rustsec.org/advisories/RUSTSEC-2021-0079.html) and linked fixes support the overflow mechanism.

   **Fix:** Label the benefit as runtime overflow detection and adjust the class assignment and class totals consistently.

## Reasoning

The note exceeds the minimum candidate count, selects three briefs with two from Eon, supplies licenses, and generally distinguishes models from proof about original programs. The sqlx advisory, vulnerable cast, fix PR and patched version check out. The other public citations exist and largely support their descriptions.

The historical population counts also check out: 309 Eon bug beads before the candidate, and 74 hyper issues plus four advisories. Listed category subtotals add up. The problems are which candidates contribute and how confidently the evidence and proposed shutdown behavior are described.

The smallest sufficient revision changes the research text, counts and briefs; it needs no implementation or broader survey. The Opus review remains pending under the Merge Rule.

Not checked: executable rewrites or full historical reading coverage; no builds were requested.
Risk: these remain proposed models, with no demonstrated bug-catching result yet.