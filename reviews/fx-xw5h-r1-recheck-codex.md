Model: gpt-6.1-sol (high) · Date: 2026-11-18 · Re-check tide-site eec47c8

## Verdict

Approve. B1–B4 are fixed. No new blocking defect found in the fixes or the code they touch. High confidence within this re-check's scope. This clears the Codex findings for the subsequent Opus round.

## Scope and identity

Re-checked only the four findings from r1 and their fixes, including the changed recorder, its negative checks, replacement media and caption, license endpoint/footer/tests, and Shared wording. Read the current Tide AGENTS.md and GOVERNANCE.md and the author's “Round r1 fixes” account. Treated that account as claims to verify.

- Full candidate: `eec47c846829d60c38c0abc8e437ec0cc2a94eae`.
- Round diff: `git -C /home/lucca/pjs/lab/tide-site diff 570f076..eec47c8`.
- SHA-256 of that command's stdout bytes: `3a28a72622150b02b8776f3a073b2791d66d254292aa56a2cdcf75aad87ae311`.
- Tide evidence pin remains `33085fd851224ec835230c3f7f068ee9cd5e40d2`.

The site source repository remained clean at that HEAD, checked again after verification. No source-repository edits, commits or .beads/ writes. All execution and generated-media writes took place in the existing scratch clone `/mnt/build/tide-agents/fx-xw5h-r1-target/site`, updated to the candidate. This report is the only deliverable written to the Tide worktree.

## Findings' status

### B1 — Fixed: filmed results now gate successful recording

**Evidence:** `recordings/record-clip.rs:268` installs a prompt hook that records pipeline statuses from the filmed shell. Lines 279–284 capture that terminal's output with `script --flush`. Lines 314–330 wait for each command's completion, validate its statuses, and check its own output segment before proceeding. The bounded wait at line 117 fails on timeout; the former fixed pauses now follow completion. Admission requires the driver's status 1 and a TD diagnostic, the search requires failure/working filters and its seed/summary output, and the replay segment must contain both reproduction results. The command text itself does not supply the required result strings.

`recordings/record.sh:26–33` propagates recorder failure, cleans temporary output, and checks that both assets exist before installing them. An error inside the filmed sequence therefore prevents replacement of the published files.

Independently ran `recordings/check-fail-closed.sh /home/lucca/pjs/lab/tide` on the candidate in the scratch clone. All six reported checks passed: injected crate type, space/substitution/separator filenames, a real filmed-command timeout with `CLIP_TIMEOUT_MS=1`, and unchanged published clip/still hashes. Then ran the normal `recordings/record.sh /home/lucca/pjs/lab/tide` to completion: exit 0, a fresh 47.266667-second clip, and the same final-frame still hash. This exercised the successful on-camera gates as well as the timeout path.

The candidate's committed replacement clip independently measures 47.266667 seconds, H.264, 1280×720, with no audio stream. Its SHA-256 is `adbaaa0998221d6accecbdc7552d30da981547e15cf118786694404abb9453a5`, matching NOTICE. The caption's update to 47 seconds agrees. The still remains `bf6686afc2a634b6880bbf0dffbf4776af53a996eabdd61646468ee3946cef35`.

### B2 — Fixed: caller arguments cannot become shell commands

**Evidence:** `recordings/record-clip.rs:180–187` accepts only literal `bin` or `lib` and rejects unsafe admission basenames before copying fixtures or running the driver. `safe_name` at line 42 permits only a nonempty ASCII alphanumeric/underscore/hyphen stem ending in .rs and forbids a leading hyphen. Spaces, quotes, command substitutions, separators and extra extensions cannot reach the filmed shell through that argument. This is an explicit refusal policy, documented in the recipe/README.

The rehearsal at lines 215–218 invokes the driver with separate argv elements and checks its own exact rejection status. The shell launcher at lines 273–284 quotes every command word. The negative checks independently rejected the original injected CRATE_TYPE and all three unsafe filenames with the intended validation messages. The normal supported input also completed a fresh recording successfully.

### B3 — Fixed: complete notices ship in the static build

**Evidence:** `src/pages/licenses.txt.ts:2–5` imports the site's LICENSE/NOTICE and the installed d3-selection license/version at build time. The static endpoint emits them at `/licenses.txt`, and `src/components/Page.astro:26` links it from the footer. The new assertions in `tests/site-smoke.spec.ts:35–41` exercise the published endpoint.

Independently built the candidate and compared `dist/licenses.txt` with all three source texts. It contains the complete trimmed NOTICE, Apache LICENSE and upstream d3-selection LICENSE, including Mike Bostock's copyright, permission condition and disclaimer. The focused endpoint/assets test passed at both widths. This resolves the missing notices in the browser distribution without copying license text by hand.

### B4 — Fixed: Shared's sink exception is explicit

**Evidence:** `src/pages/how-it-works.astro:171–176` now restricts capability waits and acquisition of another Shared cell, and explicitly says printing remains allowed and can block when output backs up. Compared it with `git show 33085fd:docs/owner-guide/04-tasks-and-shared.md`, lines 60–67. It preserves the source's exception and removes the unconditional no-I/O/no-wait guarantee.

## New blocking defects

None found in the reviewed fixes. No minimum fix remains outstanding for B1–B4.

## Independent checks and limits

Every build/run used the required form, from the scratch clone, with one-minute load below 12 before starting (observed 2.59 before the first run and 1.31 before the final playback run):

```sh
bash ~/.cache/eon-agents/eon-overseer/heavy.sh timeout 1800 nix develop /home/lucca/pjs/lab/tide-site -c bash -c '<cmd>' </dev/null
```

The first command body, after unsetting LD_LIBRARY_PATH and enabling shell failure propagation, ran:

```sh
recordings/check-fail-closed.sh /home/lucca/pjs/lab/tide
recordings/record.sh /home/lucca/pjs/lab/tide
bun run check
bun run test:e2e --grep "pages expose canonical|the admission clip"
```

Exit 0. Negative checks passed, fresh recording succeeded, Astro reported zero errors/warnings/hints, and all six selected desktop/mobile tests passed. These tests covered the shipped license endpoint and reduced-motion/manual and automatic muted playback of the freshly generated clip.

Afterward restored only the scratch clone's media bytes from the frozen candidate and reran the same six focused e2e tests through the wrapper. All six passed again, now against the committed candidate media. The scratch clone's tracked tree was clean afterward. The regenerated movie is not assumed byte-reproducible; the committed movie's own metadata/hash and playback were checked separately. `git diff --check 570f076..eec47c8` also passed.

Did not repeat the original full-site claims audit, unchanged chart tests, full accessibility suite, or real onboarding checks. No publication, deployment, push or post was attempted. Those limits do not reopen the four resolved findings.

DONE
