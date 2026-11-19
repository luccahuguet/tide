Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidates Tide e80f788 + site b0f58ba (code, site stage)

## Verdict

**Approve: no blocking findings.** Tide e80f788 changes punctuation only, and its two copies of item 9 are identical. Every site page I read is true against Tide at e80f788, and the pages agree with each other and with the owner's wording rules. The generated data matches its generator at the pin. Nothing private ships in `dist/`. Keeping beat (a) is the right call. Five non-blocking findings follow. Two are freshness gaps that a future pin could hit.

## Blocking findings

None.

## Non-blocking findings

1. **The media and hand-written copy are not tied to the pin by any check.** Site `src/pages/index.astro:68-69`.
   - The clip's caption renders "built at Tide revision `{tideRevision.slice(0, 7)}`" from `site.ts`. The paper and the clip are generated, but nothing records or checks the revision they were built at.
   - The data checks do catch a stale pin. `status-data.ts --check` fails, and `tests/site-smoke.spec.ts:221-222` asserts that the revisions in `status.json` and `charts.json` equal `tideRevision`.
   - Failure scenario: a later pin bump that regenerates the data but not the clip or the PDF passes every check, and the caption names a revision the clip was not recorded at.
   - This predates the change and is true at b0f58ba, since the clip and paper were rebuilt at e80f788 (clip.log, paper.log, matching hashes).
   - Minimum fix, later: have `record.sh` and `build-paper.sh` write a revision sidecar, and assert in e2e that it equals `tideRevision`. Or list "rebuild media" as a pin step in the site README.

2. **The questions page would omit conformance failures.** Site `src/pages/questions.astro:130-133`.
   - The page renders "{total} cases … {pass} pass and {awaits} await rules not built yet". `status.json` also carries `fail`, which is 0 today.
   - Failure scenario: a future run record with FAIL lines still generates, and the page then silently understates failures.
   - Minimum fix: render the `fail` count when it is nonzero, or have the page or generator assert `fail === 0`.

3. **The fx-zu4f.13 note hard-codes "four".** Site `scripts/status-data.ts` (the `fx-zu4f.13` round, `note:`).
   - The note says "Its pool of four earlier rows". That number is not taken from `r51Pool`, and the phrase guard checks only "chosen after they were seen".
   - Failure scenario: if research/51's pool changes, the screened cell updates and the note goes stale.
   - Minimum fix: interpolate `r51Pool`, or drop the number.
   - The fx-zu4f.15 note's other fixed phrases ("close to predictable", "No port was executed") are in research/49 today, and the four guarded phrases cover its key qualifiers.

4. **"Bugs" overstates fx-zu4f.10's unportable count slightly.** `exclusions.unportable` for fx-zu4f.10 is research/44's arm-B E4 row count, 10. research/44:137 says one of those E4 rows duplicates an earlier bug site, while the status tile reads "Bugs that failed first on portability". This is cosmetic. Either label it "rows" or use the distinct count.

5. **Copy nits, non-blocking:**
   - `questions.astro:28-29` repeats itself: "A capability-safe, replayable subset of Rust, capability-safe up to rustc's soundness bugs". Suggested: "A replayable subset of Rust that is capability-safe up to rustc's soundness bugs (see the third question): …".
   - `how-it-works.astro:46` reads "`Net` for network connections, one HTTP GET and name lookup," as if those were separate capabilities. Suggested: "`Net` for TCP connections, one HTTP GET and name lookup".
   - `how-it-works.astro:222` says the simulator does not model "other hosts", while the home page says it models HTTP peers. Tide's README fidelity table has the same split, so it is not false, but "other TCP hosts" would read better.
   - `install.astro:97-98`: "installs Tide … in a pinned Ubuntu 24.04 image. It makes two images." The try image is `FROM scratch`.
   - `dist/index.html` has two elements with `id="try"`: `index.astro:51` and `TryIt.astro:14`. This predates the change.
   - The home catch section leaves out research/49's negative "Tide found nothing unaided" (the port was written from a public quote of known-malicious code). The paper says it at §262. The owner's rule does not require it, but it would harden the text against the cold reader's attack 1.
   - `tests/site-smoke.spec.ts:262-268` checks note strings that the generator itself hard-codes, which is close to implementation trivia. The record-phrase guards in the generator are the stronger check.

## Reasoning

**Tide e80f788.** The diff is four `;` → `.` substitutions in each of `docs/at-a-glance.md` and the README's verbatim block, with the following word capitalised. Meaning is unchanged. The site generator's `--check` confirms that the README copy equals the glance.

**Pages against Tide e80f788.**
- *Home catch section:* RUSTSEC-2026-0039, TD0602 ×2 and TD0501 ×3 with their call sites, and the trust artifact declaring bg, clock and proc all match research/49 (lines 66-71 and 433). The qualified order sentence matches research/49 §3 ("consistent with … not established"). So do "not a first-time catch, not a rate", "close to predictable", "admission and type-checking only, no port executed" and "admission does not judge intent". The claim "one row of one round stays unresolved" matches research/44 (B23.128). Research/46 and research/52 report 0 unresolved.
- *Rest of home:* the costs heading now says "case-study bugs", which removes the contradiction with the catch. The capability list (Fs, Clock, Env, Net, Proc, Rand) matches tide-std. The replay list matches the README fidelity table.
- *Questions page:* the #25860 and #57893 history, the two routes left admitted on purpose (`Termination::report` and the inert `cdylib`), the reversed `From<Cancelled>` removal, HTTP GET over TLS, lookup, processes and random draws, and "Not yet: UDP, `Rand::rng()`" all match `docs/owner-guide/answers.md` and the README.
- *Install page:* the `--target dev` and `--target try` commands match `docs/docker.md` and the Dockerfile.
- *Try-it block:* the order and commands match the README "Try it" section and the owner's order: Codespaces, rustup with `rustup toolchain install`, nix run, local Docker.
- *Status data:* `status.json` has rounds .10, .12, .13 and .15. The .13 catch is labelled as the .15 catch, "not a second one", per research/51:87-89. Rand is shipped and seeded. The token count is 24,932, matching reference-companion:17. Conformance shows 393 cases, 390 pass, 3 await and 0 fail.
- *Owner rules:* "Unforgeable" appears nowhere in `src/` or `dist/`. The authority claim is scoped (safe code, printing aside, outside the grant) on the home, how-it-works and questions pages and in llms.txt. The .12 round shows counts only. llms.txt keeps "complements Rust … not a fork".

**Generators and pins.** I ran these myself:
- `bun test scripts`: 20 pass.
- `bun scripts/status-data.ts <tide> --check`: exit 0, "match e80f788…". So neither generated file is hand-edited.

The new readers fail closed:
- `tableIn` requires exactly one table with the header in the section, and it is tested for both 0 and 2 tables.
- The research/44 counts are sum-checked.
- The research/52 rows require exactly one match.
- research/51's prose count is sum-checked.
- The research/49 and research/51 qualifier phrases are guarded.
- `checkProtocols` forces every registered protocol onto the page.
- The `advisoryCounts` refactor keeps research/47's Sum = pool check.

A stale data pin is caught. Finding 1 covers the remaining media and copy gap.

**The clip.** I read `frames/sheet.png` myself. Beat (a) shows `cat` of the public conformance case, rustc with `echo $?` 0, TD0206 (CLOSURE-3), then `echo $?` 1. Beat (b) shows 8 of 16 seeds failing, including 0, and two "replay reproduced … seed=0" lines. The only path visible is `/home/tide`. The 59 s caption and description agree. `dist/media/clip.mp4` and `dist/paper/tide-v0.pdf` are byte-identical to `public/`.

Keeping beat (a) is right, for three reasons:
- research/49 §8 records rustc errors E0433 and E0599 for the chrono_anchor port, so it cannot satisfy the "rustc accepts, admission rejects" beat.
- Filming it would put archive-derived port bytes on camera, which the fx-zu4f.8 desk condition forbids.
- A leg-1-only clip would misrepresent a two-leg catch.

Presenting the catch in text, in research/49's wording, is the honest choice.

**Code quality.**
- The changed code is small and follows existing patterns. `advisoryCounts` removes a duplicate parser rather than adding one, and the `—` for empty stages is rendered and tested on both sides.
- No dead code is left: `dockerImage`, `registry.invalid` and the TODO are gone from source, tests and dist.
- The Codex B1 and B2 fixes are present in dist (rg counts: index 2, install 2, llms.txt 2, questions 1), and the e2e check asserts the scoped llms.txt wording.

**Privacy.** I searched `dist/` for `/home/lucca`, `/mnt/build`, `gmail`, `eon-agents`, `tide-lead`, `tide-desk`, `eon-desk`, `luccas-projects`, `registry.invalid`, TODO, FIXME, `.cache/` and "unforgeable". None appear. The Vercel scope string appears only in the site README, which does not ship. The `owner.invalid` placeholder is the documented owner step before publication.

**Not run:** Rust, Nix, Docker, the clip, the paper, `astro build` and Playwright. For those I relied on `checks-r1.log` and the builder's logs.

DONE
