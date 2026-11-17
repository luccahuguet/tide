Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate f02bad9

# fx-qlek.12 r2 re-check: Opus (fixes to non-blocking items 1-4 only)

## Scope and identities

This re-check covers only the fixes for items 1-4 of r2-opus.md, read from
`git diff 9c0649a 7459ff2`. It checks these identities:

- f02bad9 = f02bad9a244ffd2ad17abc233236836568f2615d. Its parent is main 318ec10, and the worktree HEAD is f02bad9 with a clean tree.
- `git diff main...f02bad9` hashes to 2a751fa662ea025450454b9a9f6878bde445c88dc1c97ad62b58ac579d3b62e5, which matches the brief.
- `git diff 9c0649a 7459ff2` touches six paths: README.md, walk/reify.rs, tests/build.rs, tests/code_prefix.rs, tests/driver.rs, and `.beads/issues.jsonl` (main's ledger, from the earlier rebase).
- Between 7459ff2 and f02bad9, among this bead's paths only `.beads/issues.jsonl` changes.

What main added since 3345f5c:

- `trials/e3-pilot/`, `reviews/fx-1vwf.1-*` and `.beads`, as tide-lead said.
- Commit 318ec10, "fx-xw5h: add 'How Tide works' …", changes only `.beads/issues.jsonl`.
- `git grep 'FX[0-9]{4}'` on f02bad9, outside .beads, archive, research, reviews, spikes and trials/e1*, finds nothing.

I ran no builds and no tests.

## Verdict

approve

## Item 1: lowercase `fx` names

**Fixed.**

- `walk/reify.rs`: `fn fx0206` and its two call sites are now `td0206`.
- `tests/build.rs:3006`: the test is now `float_banned_methods_still_report_td0520_with_production_std`.
- `tests/driver.rs`: the `fx_lines` helper and all of its call sites are now `td_lines`, and the loop variable `fx` at :1973 and its use at :1984 are now `td`.
- No other lines change.

A search of f02bad9 for `\bfx_…`, `…_fx`, `fx` used as a word, `fx####` and `Fx####` in crates, conformance, demo, docs, scripts, README.md and design finds only the guard's own negative sample (`Fx0206` in code_prefix.rs:65). That sample is intended. rustc's `rustc_data_structures::fx` imports aren't our names.

## Item 2: the guard skips `trials/e1`, not all of `trials/`

**Fixed.**

- `KEPT` is now `["archive/", "research/", "reviews/", "spikes/", "trials/e1"]`. The prefix covers e1, e1-checked, e1-dialect, e1-origins and e1-round2.
- `trials/` itself (rel `trials`) doesn't match, so the walk descends into it, and `trials/e3-pilot` and later trials are guarded.
- `.beads/` left the list, because the rule that skips dot-directories covers it.

The builder's probe shows both sides:

- `trials/e9-probe/p.md` was flagged.
- `trials/e1/probe-kept.md` was skipped.

Before this round, the rebase brought in the newly guarded files, so the guard's inputs changed. `guard-g1.log` shows the focused guard test running at 03:10:26, 22 seconds after the rebase finished at 03:10:04 on f02bad9. `guard.sh` refuses to run unless HEAD equals the SHA it is given and the tree is clean. Both tests passed and the exit code was 0. The e3-pilot files hold no FX codes, so this agrees with the static grep.

## Item 3: the guard walks the filesystem without git

**Fixed.**

- `scan()` walks from the canonicalized repository root with `std::fs::read_dir`. It doesn't call git, so a source archive without `.git` runs it the same way.
- It skips three things:
  - directories named `target` at any depth, which covers the workspace target and the E3 harness's;
  - dot-directories, which covers `.git`, `.beads` and `.worktrees`, so the main checkout's nested worktrees aren't read twice;
  - the kept prefixes.
- It doesn't follow symbolic links.
- It reads every regular file, dot-files included, and it panics with the path on any I/O error, so a problem fails loudly and can't pass by accident.
- The `checked > 100` floor still stops an empty walk from passing.
- No tracked file sits under a `target/` directory or a dot-directory other than `.beads/`, so the skips that go by name hide nothing live.

The builder ran the compiled guard under `env -i PATH=/nonexistent`. Git couldn't run there, and it found exactly the two probes it should have.

One residual nit, which doesn't need a change: the walk now also reads git-ignored files outside `target/` and dot-directories, such as `debug/`, `*.rs.bk` or `mutants.out*/`. A stale file there that quoted an old code would fail the suite. None exists, cargo-mutants output would copy today's TD source, and a loud false failure is the right way for this to go wrong.

## Item 4: the README clause

**Fixed.** README.md:756-760 now says that the documents in `research/`, `reviews/`, `trials/`, `tasks/` and `archive/` "keep that name as written, and older records spell diagnostic codes `FX####`, with the same digits as today's `TD####`."

- It sits in the History bullet on naming, where a reader who follows the README's links into the records will look.
- It agrees with the note in conformance/rules.md.
- It uses the literal `FX####`, which the guard allows.

## Suites

- On 7459ff2, the r2 logs show the guard passing in the workspace run. The builder reports 346/0 for the workspace and 386 PASS for conformance.
- f02bad9 differs from 7459ff2 only in files main brought in, and among those the guard is the only check that reads them. It passed on f02bad9 (`guard-g1`).
- So no other check needs a rerun for this landing.

DONE
