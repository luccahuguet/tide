Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate bb62682 (re-check of 95ab967 findings)

# Verdict: approve

Both blocking findings are resolved. Every new number checks out against the records. The edits break nothing and overclaim nothing. One wording nit remains (B2 below), which does not block.

Scope: `git diff 95ab967 bb62682`. It changes only `research/36-benchmarks.md` (71 lines added, 33 removed). The worktree HEAD is bb62682.

## B1. Ties that were losses: resolved

**The new rule.** The stability note now reads ties and losses from paired rounds: "slower" means slower in at least 20 of 30 rounds.
- The stated p = 0.049 is correct: the sum of C(30,k) for k = 20..30, divided by 2^30, is 0.04937.
- The rule is symmetric in effect. No comparison has Tide faster in 20 or more rounds; the nearest is `contracts` release, faster in 19 of 30. So line 71's "none is a win" still holds.
- §1.6 now points to this rule and says the CV-based reading would have hidden the `files` loss.

**Recomputed from the measurement records:**

| Note's claim | Recomputed |
| --- | --- |
| `files` slower in 26/30, paired median 1.066, IQR 1.026-1.091 | 26/30; median 1.0657; quartiles 1.0257/1.0912 (Python's default exclusive method) |
| `tcp` slower in all 30 | 30/30 |
| `contracts` release: 11/30, a tie | 11/30 |
| Profile flags slower in 15, 18 and 16 of 30 (`contracts`, `files`, `tcp`); `tasks` in all 30 | 15, 18, 16; 30 |
| Debug contracts at 200,000: 1.19x, slower in all 30 | 30/30 (summary ratio 1.190) |
| Test build against plain debug: slower in all 30 | 30/30 |

**Placement.** The summary rows that README or paper authors would quote now state both losses plainly:
- `files`: "a small loss, of a few percent";
- debug contracts: "At 200,000 … 1.19x, slower in all 30 rounds, a loss", next to the 1.2M tie.

§1.4 matches both rows. §2.6 now says the per-round counts and paired ratios come from the measurement records.

## B2. Measured revisions: resolved

- Line 10-11 now says the header revisions are earlier, amended states that are not in the repository's history.
- §2.6 drops the false sentence ("adds the note and both record files to `c04dc44`").

I verified each new claim:

| Claim | Check |
| --- | --- |
| Tree ids `6e7f8814…` (bf6b958) and `6a882d0b…` (c04dc44) | match `git rev-parse <rev>^{tree}` |
| Both share the candidate's parent | all three have parent 11a0bbb |
| `pairs.rs` SHA-256 `89a2e8b1…` and `9a8da93f…` | match `git show <rev>:…/pairs.rs \| sha256sum` |
| They differ from the candidate only in the listed files | `git diff --stat bf6b958 bb62682` and `c04dc44 bb62682` touch only `benchmarks/README.md`, `pairs.rs`, `tests/benchmarks.rs`, the note and its records |
| Programs, toolchain pin, driver, std and support files are byte-identical | follows from the above |
| The changes made after each run | match my earlier diffs: the test phase and `--phase` after bf6b958; `keep` in the run phase and the `start`/`open` split after c04dc44 |
| "The landed `pairs.rs` measures as the two measured versions did" | consistent with those diffs for a full run |

§2.6 is now accurate. The coordinator chose not to add tags. That is the option-2 fix I offered: the revisions themselves are lost after landing, but the hashes let anyone verify a recovered copy, and the note says the revisions are unreachable.

Nit, non-blocking: "they differ from it only in … and this note's directory". The note file `research/36-benchmarks.md` also differs, and it sits beside that directory, not in it. "This note and its records" would be exact.

## Non-blocking items

| Item | Status | Check |
| --- | --- | --- |
| N1, memory range | resolved | now "+17% to +39%" |
| N2, size units | resolved | "18.5-20.9 MiB against 4.5-5.3 MiB" matches 18,992-21,374 KiB and 4,588-5,400 KiB divided by 1024 |
| N3, rebuild peaks | resolved | the rebuild memory sentence now separates `tide-debug` from incremental `rust-debug` (117-122 against cold 117-152), which matches the records |
| N4, simulator arms | resolved | the summary and §1.5 now say "under `tide-release` / `tide-debug`", that is, the whole configuration |
| N8, test-build attribution | resolved | the vector scan is labelled [repo evidence]; unrepresentativeness is labelled [inference]; the libtest harness and `cfg(test)` differences are named as unseparated |
| N10, provenance of ratios | resolved | §2.6 now says "a quotient of two summary medians" |
| N5, `--phase` handling; N6, `build_harness` | not taken | code changes, still non-blocking. Leaving them is acceptable, since they would cost a code re-review |
| N7, test coverage | not taken | acceptable |
| N9, `--quick` evidence on d8d9425 | not taken | the argument from my review still holds, and the candidate's `pairs.rs` is unchanged since 95ab967. The coordinator can record it on the bead |

No new overclaims. The new evidence labels in §1.4 are applied correctly.

DONE
