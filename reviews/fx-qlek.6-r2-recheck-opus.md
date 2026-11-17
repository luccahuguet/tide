Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate 08f2d38

I re-checked only my own findings from r2-opus.md. The diff checked is `git diff 4d54489 08f2d38`, at HEAD `08f2d388f7e2bdb266fdf24d203e30b7277e9bad` with a clean worktree. It touches only CONTRIBUTING.md, NOTICE and README.md. I also read report.md's "Round 2" section and rust-toolchain.toml. Read-only: I ran no builds and started no subagents.

## Verdict

**approve**, with high confidence. B1 is fixed. Every non-blocking item and contributor note was either taken correctly or declined with a sound reason. The fix introduces no blocking problem. Two new nits follow, and neither blocks.

## B1: CONTRIBUTING hid the Development Rule (fixed)

`CONTRIBUTING.md:7-18`, "Who writes the code", now opens with the rule:

- production code is agent-produced;
- humans set direction, write specifications and review;
- agents write the code;
- hand-written production code is an exception that a change marks as one, with a link to GOVERNANCE.md, Development Rule.

This matches GOVERNANCE.md: forbidden by default, and marked when it occurs as an exception. "Exception" plus the stated norm conveys "forbidden by default" without restating the rule's text. GOVERNANCE.md stays the binding reference.

The owner's positioning is intact:
- AI-written contributions are welcome because every change is checked, under the same bar;
- lines 3-5, "complements Rust and is not a fork", are unchanged;
- the upstream-soundness section is neutral, with no rebuke.

The roles section is unchanged.

## N1: proof was bound to the wrong file (fixed)

The review is now bound to GOVERNANCE.md (Merge Rule) and the proof to AGENTS.md (AP-PROOF-001, AP-TEST-001). Both are correct owners.

## N2: NOTICE's at-the-site claim (fixed)

NOTICE:5 now reads "Each file adapted from upstream code says so at the site." That holds for the files it covers: io/shared.rs, error.rs, convert.rs, codec.rs, tide-macros error.rs and the rustc case. API mirroring and the cases based on issue reproductions fall outside it.

## N3: owner not named (declined, correctly)

The builder left this to the owner, because it is personal information. That is the right handling. It stays open for fx-qlek.7.

## N4: archived scaffold declares MIT, out-of-workspace crates declare none (declined, correctly)

The builder left this to the owner as follow-up A, since it changes a historical record's license statement. That is the right handling.

## N5: Rust policy cited from secondary coverage (declined, acceptable)

"limits" stays, and the link waits for fx-qlek.13. That is acceptable: the wording is soft and neutral.

## N6: fork naming could cover private changes (fixed)

README now says "A fork or modified version that is distributed uses another name".

## N7: TASKS.md at the root (no action, as recommended)

No change was needed, and none was made.

## Fresh-contributor notes (taken correctly)

- **Build and test.** CONTRIBUTING:28-44 gives the two commands and their prerequisites, which I checked:
  - rust-toolchain.toml pins `nightly-2026-08-09` with `rustc-dev` and `rust-src`;
  - the `stable` toolchain;
  - python3 with libmpfr6;
  - a first-run warning.

  The builder says the 1.4 GB figure was measured with `du`, which is plausible for that nightly with those components. "rustup installs it on first use" holds for current rustup, where auto-install is on by default.
- **Proposing a change.** Step 1 now says to open a GitHub issue or pull request. It glosses beads as the issues in `.beads/issues.jsonl` and `br` as beads_rust, with a link checked against the installed br binary. It says contributors do not edit `.beads/`. The two-tracker confusion is gone.
- **Review and who decides.** Step 4 says a maintainer runs the Merge Rule review and decides whether the change lands. It says contributors fix or rebut each finding, and that review records "may" be committed under `reviews/`. This matches GOVERNANCE.md: the author fixes or rebuts, a human maintainer is final merge authority, and records go in the PR discussion or in reviews/.
- **Jargon.** "red-team" became "adversarial testing of the checker".
- **Notes outside the diff** were deferred to fx-qlek.7 and the owner: BR_WORKFLOW's stale queue, a README line on AGENTS.md, and AGENTS.md's machine-local paths. That is correct, since AGENTS.md is single-holder.

## New findings (non-blocking)

- **R1, `CONTRIBUTING.md:15-16`: the review summary is slightly stronger than GOVERNANCE.md.** "they meet the same bar as every change: review rounds by models from different providers" says more than the rule does:
  - an Opus stand-in can replace a refused Codex pass, so both reviewers may come from one provider;
  - generated Starcompass import updates skip review entirely.

  GOVERNANCE.md is linked on the next line as the binding text, so this is a summary nuance, not a contract a contributor relies on. Optional fix: "review rounds, normally by models from different providers". This is a nit.
- **R2, `CONTRIBUTING.md:28-44`: the build and test instructions now exist in two places.** CONTRIBUTING repeats the commands and prerequisites from `README.md:135-146`, so a later change to the toolchain or prerequisites must update both. I offered this option in r2-opus, and the duplicated text is short. Optional fix: when either copy next changes, keep the copy in one place and link to it from the other. This is a nit.

DONE
