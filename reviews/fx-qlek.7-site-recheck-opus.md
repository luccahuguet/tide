Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidates Tide e80f788 + site 0f586a2 (site re-check)

Scope: I re-checked only NB1-NB5 from target/work/site-opus.md, reading `git diff b0f58ba 0f586a2` (9 files, +27/−21) and "Site r2" in report.md. I ran `bun test scripts` (20 pass) and `bun scripts/status-data.ts <tide> --check` (exit 0, "match e80f788…"). I also read the built dist (11:03, after the amend) and the checks-r2.log tail (0 fail, 30 passed).

## Items

- **NB1, done.** Site README:102-106 now says a pin is complete only when the data generators, `record.sh` and `build-paper.sh` have all been rerun at the new `tideRevision`. It also says why: no check compares the media's build revision with the pin. This is true and does not overclaim. It is the documentation fix, not the sidecar check. That is acceptable for a non-blocking item, and the gap is now stated where the next maintainer reads.
- **NB2, done.** questions.astro:135 appends `, and N fail` only when `conformance.fail > 0`. With `fail` at 0, dist renders "3 await rules not built yet. There is no CI yet". There is no stray `false` or `0` text, because a boolean `false` renders nothing in Astro.
- **NB3, done.** The note now interpolates `r51Pool`, which comes from research/51's sum-checked prose count. Dist renders "Its pool of 4 earlier rows". The numeral is fine.
- **NB4, done.**
  - fx-zu4f.10's entry carries `unit: 'rows'`, and dist renders "fx-zu4f.10 10 rows". That is research/44's E4 row count (research/44:129), so the label is honest.
  - The tile's lead-in changed from "Bugs" to "Candidates that failed first on portability", which is true for every round's unit.
  - The `status.json` diff touches only those two fields, and `--check` matches, so nothing is hand-edited.
  - The cast at status-data.ts:491 is untidy but harmless, and `astro check` is clean per the log.
- **NB5, done.**
  - questions:28: "A replayable subset of Rust that is capability-safe up to rustc's soundness bugs". The rustc question is still the third.
  - how-it-works:46 reads "`Net` for TCP connections, one HTTP GET and name lookup", and :222 reads "other TCP hosts". Both agree with the README fidelity table, which models loopback TCP and `world.http` peers.
  - install:97-99: "builds Tide … in a pinned Ubuntu 24.04 build stage and makes two images from it. The try image … holds only the demo's closure." This matches the Dockerfile (`try-closure` FROM `nix`, `try` FROM `scratch`) and docs/docker.md:10.
  - dist/index.html now has exactly one `id="try"`, on the TryIt div, so `#try` links still resolve. No CSS or test targets `section#try`.
  - **New catch sentence.** "Tide found nothing unaided. The code was already known to be malicious, and the port was written from Socket's quote of it." This is word for word research/49:718-719 (§9 Negatives). It sits after "its authority is declared" and before "No other search round…", and it neither contradicts nor weakens the surrounding qualifiers.
- **Tests.** The note assertions that repeated the generator's hard-coded strings are replaced by a loop. It checks that every round with a note in `status.json` renders that exact text in its row, which is the page's observable contract. The wording itself stays guarded by the generator's record-phrase checks, which still run in `--check`. No check was weakened that guarded a source of truth.

## Non-blocking (new, nit)

- index.astro:160: "Socket" appears nowhere else on the site, so a launch reader cannot tell it is a security firm. research/49's own wording assumes context from its sources table. A minimal gloss would help, for example "Socket's (a software supply-chain security firm) published quote". Text only, so it needs no re-check.

## Verdict

**Approve.** NB1-NB5 are done as described, and they introduce no defect, false text or broken check. The rendered output matches the record, and the generated data is fresh at e80f788.

DONE
