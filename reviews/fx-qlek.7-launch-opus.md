Model: Claude Opus 5.5 · Date: 2026-11-19 · Candidates d751d4a + e5627ef (launch commits, Opus review)

Verdict: approve. Tagging e5627ef as v0-alpha.1 is consistent with every current text in the tree.

Frozen candidate: e5627ef631430abf6e92e9743dd8f08158371cad on base 24f0951, worktree clean. Reviewed independently. I read the Codex report only after taking my own measurements, and checked each of its claims against them.

## Blocking findings

None.

## Non-blocking

1. The run record's `inputs:` line includes `conformance/`, and e5627ef changes `conformance/run-record.txt`, which sits inside that directory. A record can never name its own commit, so this is unavoidable. No tool reads the record: `rg` over crates/, scripts/ and conformance/*.md finds no consumer. Outside the record itself, `git diff d751d4a e5627ef` touches no suite input, so the recorded results hold at e5627ef. No change needed.
2. The paper does not cite a Zenodo DOI. Fx-qlek.7's notes make the DOI Lucca's step at the tag ("the paper then cites the DOI"), so it can only come after the tag. A paper that adds the DOI will need its own commit after v0-alpha.1. This is a reminder for after the tag, not a defect in the candidate.
3. The only remaining spellings of `v0-alpha` (without `.1`) are in the historical review records `reviews/fx-qlek.7-addr-{codex,opus}.md`. They record the earlier decision and stay as history. Nothing else that is current needs the new tag: there is no CHANGELOG or release-notes file, every crate and the flake are at `0.0.0`, and the README and docs say "v0 alpha" with no tag name.

## Evidence

- Identity by patch-id (`git show <c> --pretty=format: --binary | git patch-id --stable`): c671731 = df118d2 = 2521e90f…; 1130752 = f7c2a9c = ad80e5a3…
- d751d4a changes one line, `paper/tide-v0.typ:31`, the `tide-tag` default `v0-alpha` → `v0-alpha.1`. Every repository link goes through `#repo` (lines 34–38), and the Status line (93) prints the tag with `#raw(tide-tag)`, so the tag is spelled in only one place. A `git grep v0-alpha` outside reviews/, research/, trials/, archive/ and .beads/ finds only line 31.
- Build: `typst compile --root . paper/tide-v0.typ /mnt/build/tide-agents/tide-launch-work/opus/tide-v0.pdf` printed nothing and exited 0. This used typst 0.15.1 here; the paper names 0.15.0 from the flake pin. The PDF has 68 URIs, 56 of them repository links. Every repository link matches `https://github.com/luccahuguet/tide/(blob|tree)/v0-alpha.1/`. For each one, `git cat-file -t HEAD:<path>` gives the object kind its blob/tree segment needs, so all 56 resolve in the candidate.
- e5627ef changes only lines 2–3 of `conformance/run-record.txt`. `commit: d751d4abf3b2…` equals `suites/candidate.txt`, and `date: 2026-11-19T19:56:30Z` equals `suites/conformance.end`. The `exit: 0` line was already 0 and equals `suites/conformance.exit`. The command and inputs lines are unchanged.
- Record body: a `python3 -I` script collected the result lines and summary from `suites/conformance.log`, dropping three lines (the heavy.sh slot line and two cargo lines). The record's body after its header equals them byte for byte, with a trailing newline. It has 390 PASS and 3 AWAITS CHECKER (TD0809, TD0808, TD0205), no FAIL, and ends `393 cases: 390 PASS, 3 AWAITS CHECKER, 0 FAIL`. The docs that quote counts (owner-guide 02, 10, answers.md) give 393/390/3 and still agree.
- Full `cargo test`: `suites/test.exit` is 0. All 16 `test result:` lines say `ok`, and the log has no FAILED or panicked lines. The run went from 19:33:24Z to 19:39:04Z, in the tide-launch worktree's target dir. `candidate.txt` names d751d4a, and `head-check` says `HEAD unchanged`. The conformance run reused that build (`Finished … in 0.01s`).
- Nothing private or host-specific ships. The record has no host paths; the `/mnt/build/...` paths appear only in the suite logs, which are not committed. The paper diff touches only the tag. The `Claude-Session:` trailer on e5627ef follows existing practice (91 commits on main carry one). `git diff --check 24f0951 e5627ef` passes.
- Remaining risk: links resolve only after Lucca creates v0-alpha.1 at the landed tip. If the rebase changes the commit IDs, the record's `commit:` must still name the tested tree. Today it names d751d4a, which is what was tested.

Skipped: cargo and nix runs (out of scope); visual layout of the PDF; live HTTP checks.

DONE
