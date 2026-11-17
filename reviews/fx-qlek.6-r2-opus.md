Model: claude-opus-5-5 · Date: 2026-11-18 · Candidate 4d54489

Reviewed `git diff main...4d54489` (merge base `199e6f5`, HEAD `4d544890a9f8f02f9813f4ebf75b9742246a2b06`, clean worktree; diff SHA-256 `257448e2297317b407cf2c30cec6f14833ecd5935c90944c922531f276e08c18`). I read AGENTS.md, GOVERNANCE.md, the bead and all its comments, report.md, r1-codex.md and r1-recheck-codex.md. Read-only: I ran no builds and started no subagents.

## Verdict

**request_changes**, with high confidence. One blocking finding, and its fix is two sentences of CONTRIBUTING.md. Everything else in the change is accurate and meets the bead. The non-blocking items can land as they are or be folded into the same fix.

## Blocking findings

### B1. CONTRIBUTING hides the Development Rule from human contributors

`CONTRIBUTING.md:9-14`

**Problem.** GOVERNANCE.md's Development Rule says production code is agent-produced, and that "human-written production code is **forbidden by default** and must be explicitly marked if it occurs as an exception."

CONTRIBUTING presents it the other way round:
- "an AI-written change meets the same bar as any other" reads as if hand-written code is the norm and AI the newcomer;
- "Most of Tide's own code is agent-written" understates the rule, which covers all production code;
- "how a change says whether its code was written by hand" makes hand-writing sound like a disclosure choice.

A stranger who writes Rust by hand and opens a PR learns only at review time that their contribution is the forbidden-by-default case. That is the most consequential rule for an outside human, and the page leaves it out. The brief requires CONTRIBUTING to describe GOVERNANCE.md truthfully. This text is not false, but it misleads by omission. The README Project Process line ("production code is agent-produced") is the only place a newcomer would find the rule.

**Minimum fix.** Replace lines 10-14 with a plain summary that links the rule without restating its details. For example:

> Tide's production code is agent-produced: humans set direction, write specifications and review, and agents write the code. Hand-written production code is an exception that a change marks as one ([GOVERNANCE.md](GOVERNANCE.md), Development Rule). An AI-written change meets the same bar as every change: cross-model review rounds, and runnable proof scoped to what it touches. GOVERNANCE.md binds the review, and [AGENTS.md](AGENTS.md) binds the proof.

This also resolves N1. The fix changes what the document says a rule is, so under GOVERNANCE.md it is not text-only. I re-check it.

## Non-blocking findings

- **N1. `CONTRIBUTING.md:13`: GOVERNANCE.md is named as binding for both review and proof, but the proof rules live in AGENTS.md.** "both" covers review rounds and runnable proof. GOVERNANCE.md holds the review rules but says nothing on proof scope beyond "tests too weak for the risk". The proof rules are AP-PROOF-001 and AP-TEST-001 in AGENTS.md. Fix: point proof at AGENTS.md, as in B1's text. This is a cross-reference fix.

- **N2. `NOTICE:5` claims "Each adapted file also says so at the site", which is not true of every file listed.**
  - The API-mirroring entry (`NOTICE:19-21`) names `sync::mpsc`, but `crates/tide-std/src/sync/mpsc.rs` has no such note. Its first line is a `use`.
  - The four red-team cases name their rust-lang issue, which is adequate, but do not say that they follow it.

  Mirroring an API's names and signatures needs no notice. Fix: narrow the sentence to "Each file adapted from upstream code says so at the site", or drop it.

- **N3. No one is named as the owner.**
  - README "License and Name" says the name "belong[s] to the owner", and CONTRIBUTING Roles say "the owner", but neither says who that is.
  - NOTICE has no `Tide / Copyright <year> <owner>` header, which the Apache NOTICE convention puts first (builder follow-up B).

  Neither is a legal defect. But a naming policy that reserves a name to an unnamed party is weak for a stranger to act on. Recommendation for the owner, before fx-qlek.7: name the owner, by legal name or GitHub handle, in README and NOTICE. This needs the owner's decision, because it is personal information. Agents should not insert it.

- **N4. Some crates outside the workspace have no Apache-2.0 declaration.** The acceptance item, as the verification note narrows it to workspace packages, is met: all five inherit `Apache-2.0` from `[workspace.package]`, which matches LICENSE. The remaining crates:
  - `archive/v0-design/scaffold/Cargo.toml:16` still declares `license = "MIT"`, and its seven crates inherit it. A public reader sees two licenses.
  - Nine other manifests outside the workspace declare no license: `conformance/rustc/{macros,std}`, `spikes/d18/*` (5) and `trials/*` (3). The root LICENSE covers them.

  Recommendation: put the scaffold's MIT line to the owner before fx-qlek.7, as the builder already proposed (follow-up A). It changes a historical record's license statement, so agents should not change it on their own.

- **N5. `CONTRIBUTING.md:19-21` states another project's policy from secondary coverage.**
  - fx-qlek.13's step 1, reading the official policy text, has not run yet.
  - "limits" is soft enough to be safe, and the tone is neutral, not a rebuke.

  Fix: once fx-qlek.13 reads the official text, link it here.

- **N6. `README.md:786`: "A fork or modified version uses another name" can be read as covering private, unpublished modifications.** Fix: "A fork or modified version that is distributed uses another name". This is a nit.

- **N7. TASKS.md stays at the root, while README lists it under History.** GOVERNANCE.md names it the canonical completed-task archive, so keeping it at the root is defensible. No action is needed.

### Checked and correct

- **Acceptance:**
  - The history scan is recorded clean: gitleaks 8.30.1 over all refs and archives, plus the stash merge. Its tool, command and revision are recorded, and tide-lead's exact-value scan covers known credentials.
  - The root holds no FEROX_*, ORBIS_* or PUBLIC_NAME_REVIEW.md. The moves are R100 into `archive/v0-design/`.
  - No live Markdown link targets an old root path. The archived spec's links to these files now resolve.
  - All workspace crates declare Apache-2.0.
  - Making the repository public stays the owner's action.
- **NOTICE and Apache-2.0:**
  - Every listed upstream is MIT OR Apache-2.0, and the Apache option is stated. MIT's terms therefore impose no separate notice obligation.
  - rust-lang/rust, serde and thiserror ship no NOTICE file, so §4(d) carries nothing over.
  - §4(b) change notices are present where files were modified: `io/shared.rs` ("adapted, with changes"), the rustc case ("Changed by Tide … made `bad` public, dropped `main`…"), and `codec.rs:15-19`.
  - `tide-std/src/error.rs` and `tide-macros/src/error.rs` credit thiserror. The Rust source edits are comment-only.
  - The rustc case keeps a four-line header, so line 11 and the quoted diagnostic position are unchanged.
- **Naming policy:**
  - It explicitly disclaims a registered trademark ("not a registered trademark claim") and reads as the owner's policy.
  - Its §6 reference is accurate: the license grants no trademark rights beyond describing a work's origin. "may say it derives from Tide" matches §6's origin carve-out.
- **Positioning:**
  - The page says "complements Rust and is not a fork".
  - AI-written contributions are welcome because changes are checked, under the same bar.
  - rustc soundness findings go upstream through humans, framed neutrally with no rebuke.
  - This matches the owner's 04:37 comment.
- **Roles:** they match the owner's 03:52 comment.
  - The owner sets direction and scope.
  - Maintainers and co-leads own technical areas.
  - Maintainers join by the owner's invitation.

  Dropping "final merge authority" avoids contradicting GOVERNANCE.md's "human maintainer".
- **Codex's four r1 findings:** they are resolved as its re-check says. I found nothing that contradicts that re-check.

## Fresh-contributor notes

The reader starts cold at the root, README and CONTRIBUTING.

- **What Tide is: clear.** README's tagline and first paragraph, and CONTRIBUTING:3-5, say it well.

- **How to build and test: present, but hard to find.**
  - The commands and prerequisites are at `README.md:135-146`: the pinned nightly with `rustc-dev`, the `stable` toolchain, and `python3` with `libmpfr6`.
  - They sit about 80 lines into a dense paragraph on the admission driver.
  - CONTRIBUTING:32 points to "README.md, Status" with no anchor.

  Suggest a short "Build and test" heading in README that CONTRIBUTING links to, or copying the two commands and three prerequisites into CONTRIBUTING. Also warn that the first `cargo test --workspace` builds a rustc driver, which is slow and needs disk space.

- **How to propose a change: the weakest part.**
  - CONTRIBUTING never says "open a pull request".
  - Step 1 names two trackers: GitHub issues ("open an issue") and beads with `br`. It leaves unexplained:
    - what a bead is;
    - that `br` is beads_rust, or where to get it;
    - that outsiders cannot write `.beads/`, since only the coordinator writes it.
  - BR_WORKFLOW.md does not explain `br` either, and it still lists an "initial seeded queue" of T072-T077, which is stale for a newcomer.

  Suggest one sentence: "Open a GitHub issue or pull request; maintainers track accepted work as beads in `.beads/issues.jsonl` (readable as JSON lines, or with `br` from beads_rust)."

- **What happens to an AI-written PR: roughly clear.** It is welcome, it meets the same bar, and a maintainer runs cross-model review. It does not say that the contributor is expected to fix or rebut findings, or that review records may be committed. One sentence would cover it.

- **Who decides: partly answered.** The owner decides direction and scope. CONTRIBUTING does not say who decides whether a PR lands. GOVERNANCE.md says the human maintainer is final merge authority. Suggest step 4 read: "A maintainer runs the review the Merge Rule requires and decides whether the change lands."

- **Internal jargon:**
  - CONTRIBUTING itself is light on jargon. Its unexplained terms are "beads", "`br`", "Merge Rule", "cross-model" and "red-team".
  - The root is heavier, outside this diff:
    - AGENTS.md's Parallel work section names machine-local paths (`~/.cache/eon-agents/eon-overseer/heavy.sh`), Eon panes and a coordinator;
    - README's Project Process bullets on autorun, handoff-guard and Starcompass assume insider context;
    - "tide-lead" and "pane" do not appear in README, CONTRIBUTING, GOVERNANCE or BR_WORKFLOW.

  For fx-qlek.7: add one README line saying that AGENTS.md is the maintainers' agent operating rules and that contributors do not need it. Also decide whether the machine-local paths should be public. AGENTS.md is rendered from single-holder files, so this needs its own bead.

- **Off-putting: only B1.** A human outsider who writes code by hand is the one reader who would feel misled. Everything else reads as welcoming.

## Reasoning

The legal and metadata work is careful and correct:
- The attributions are backed by a recorded clone search.
- The change notices satisfy §4(b).
- The license metadata uses one owner, `[workspace.package]`.
- The moves are pure renames that also repair the archived spec's links.
- The naming policy is worded with the right restraint.

The remaining defect is in what CONTRIBUTING leaves out. After round 1, the text was trimmed so that it would not restate GOVERNANCE.md. That trim also removed the one rule an outside human most needs to know before contributing. Under GOVERNANCE.md it is not a nit: the page misdescribes the governing rule to the exact reader it is written for. The fix is a plain one-sentence summary with a link, which keeps the "defer to GOVERNANCE.md" decision intact.

The non-blocking items are owner-facing decisions (N3, N4) or small accuracy edits (N1, N2, N5, N6). The fresh-contributor notes are improvements that fx-qlek.7's launch-claims audit can take or leave.

DONE
