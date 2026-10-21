# Tide adversarial/consistency review, round 99: parallel Codex pass and tier-scoped findings

## Verdict

**revise**: four blocking findings, five non-blocking notes.

The three governing texts state the new order the same way. The overlay
and AGENTS.md match byte for byte, the manifest's overlay hash matches the
overlay, and no text outside `reviews/` still describes the old order.
Two problems remain. The tier rule has two holes: it says nothing about
findings in an earlier tier, and it gives no test for "concerns only a
later tier", which the overlay also drops. The parallel first round is
also unclear for converge on code, because converge has its own Codex
step and its rounds edit the tree that Codex is reading.

## What was reviewed

The uncommitted working tree against HEAD `bcf8866`: `git diff -- .
':!.beads'`, SHA-256
`6d9f505a8a2597b354208b1d4bb1aa131176fec1111e5e076b1337a5d2c61be9`
(`.agent-protocols.json`, `.agent-protocols.local.md`, `AGENTS.md`,
`GOVERNANCE.md`, `README.md`). `git diff --check` passes.

- `sha256sum .agent-protocols.local.md` gives `e12d142c...8d3a`, which
  matches the manifest's new `overlay.sha256`.
- The repository-local section of AGENTS.md (from `### Tide` on) is
  identical to `.agent-protocols.local.md`.
- I read all of `GOVERNANCE.md`, the overlay, README's governance bullet
  (`README.md:165-169`), bead `fx-djp` (`br --no-db show fx-djp`), the
  tier table (`design/01-design.md:71-84`), the converge skill
  (`~/.claude/skills/converge/SKILL.md`), and the Codex invocation memory.
- I grepped every non-`reviews/`, non-`archive/` file for "codex",
  "supervisor", "converge", "come clean"/"comes clean", "converged diff",
  "confirmation round", and "re-check". The only hits outside the changed
  files are history or unrelated: `spikes/d18/README.md:8` ("has not been
  through converge or the supervisor"), `research/00-old-design-salvage.md`
  and `research/14-autoresearch.md:84` (the supervisor rule in general),
  and model names in research. `BR_WORKFLOW.md`, `design/`, and `scripts/`
  (only `count-tokens.sh`) say nothing about the review order.

## Blocking findings

### B1. A change in a later tier can land a defect in an earlier tier

`GOVERNANCE.md:48`: "only findings in the tier the change works on block
or restart a round". The overlay says the same (`.agent-protocols.local.md:25-27`,
`AGENTS.md:239-241`).

Read literally, a change that works on v1 or Later text (a Bend bead,
for example, as fx-ms8 partly was) cannot be blocked by a finding that it
broke the v0 core. That finding is not in its tier, and it is not a
later-tier finding, so the rule does not send it to `fx-djp` either. It
is neither fixed nor recorded. The owner's decision was that *later-tier*
findings stop blocking, not earlier-tier ones.

Smallest fix: in both texts, "findings in the tier the change works on,
or an earlier tier, block ...".

### B2. The test for "a later-tier finding" is missing, and the overlay drops "only"

GOVERNANCE defers "a finding that concerns only a later tier"
(`GOVERNANCE.md:48`). The overlay, which agents actually load, defers "a
later-tier finding" (`.agent-protocols.local.md:26`, `AGENTS.md:240`). A
finding that shows a v0 rule is wrong when it meets a v1 or Later
feature is "a later-tier finding" in the loose sense, but it does not
concern *only* a later tier.

`fx-djp` takes the loose reading. Its evidence names
`reviews/round96-consistency.md` B1 as a later-tier finding. But that
finding was a defect in the change's own v0 text: TASK-3's abandonment
rule overreached and caught an in-process Bend kernel, and the fix went
into TASK-3. `round58` B1 (a descriptor missing from RUN-3's v0 list) is
the same kind of finding. An author who follows the bead's examples will
defer defects in the v0 text the change wrote. That is the exact way the
tier rule could let a change skip a defect in its own tier.

Smallest fix: give one test in GOVERNANCE and the overlay. For example:
"A finding blocks when its fix would change text in the change's tier or
an earlier one; it goes to `fx-djp` only when the fix would change
later-tier text alone." Use "concerns only" in the overlay too. Then
reword the bead's evidence line so it does not present round96 B1 as a
deferrable case. Under this test that finding blocks, which is right,
because deferring it would have left the v0 rule wrong.

### B3. Converge's own Codex step conflicts with the new order

For code, the first Opus pass is "a converge round"
(`.agent-protocols.local.md:18`, `GOVERNANCE.md:46`). The converge skill
has its own Codex step. When a diff is over about 200 lines or carries
`deep-review`, it runs "Claude rounds until one is clean, then a Codex
round" with `-s workspace-write`, and that round fixes things (skill,
Setup step 6 and "Convergence"). That is the old order. An agent running
converge on a large code change will either run Codex three times (the
supervisor pass, converge's Codex round, and the re-check) or have to
guess which order wins. Neither text says.

Smallest fix: one clause in GOVERNANCE and the overlay: "the supervisor
pass and its re-check stand in for converge's own Codex round".

### B4. "Parallel on the same diff" cannot hold while a converge round edits the tree

Opus and Codex review "in parallel on the same diff"
(`.agent-protocols.local.md:17-18`, `GOVERNANCE.md:46`). A docs review
round is read-only, so this holds for design work. A converge round is a
fix pass that edits the working tree in place. The Codex pass runs from
the same checkout (`codex exec -C <repo>`) for up to 30 minutes, so it
can read a mix of files from before and after the converge round's
edits. Its verdict is then about no single candidate. The re-check
reviews "the fixes since its pass", but which fixes came after its pass
is undefined if the pass read some of them.

Smallest fix: for code, have Codex review a frozen copy of the
fingerprinted candidate, such as a detached worktree at a `git stash
create` commit or an exported patch plus the touched files. Or run the
converge round in its own worktree. Say which in GOVERNANCE, in one
sentence.

## Non-blocking notes

- **N1.** When the Codex pass approves and the first Opus round is clean,
  there are no fixes since its pass. No text says the re-check is then
  skipped, though "of the fixes since its pass" implies it. Consider
  adding "(none needed when nothing changed)".
- **N2.** The parallel first round is two reviews, but `GOVERNANCE.md:74`
  still names one file per pass as `round<N>-codex.md`, and past rounds
  numbered the Opus and Codex reviews separately. Say whether the pair
  shares N or takes consecutive numbers.
- **N3.** `README.md:165-169` summarizes the order but leaves out the
  tier rule. It also says "every change to `main` gets a Codex supervisor
  pass", while planning-only changes skip it. The second gap was already
  there before this change. A short "(planning-only changes skip it;
  later-tier findings go to `fx-djp`)" would close both.
- **N4.** GOVERNANCE says the rule blocks "or restart[s]" a round, while
  the overlay says only "block a review round". Use the same words in
  both.
- **N5.** Neither text says who decides which tier a change works on. The
  bead's tier, or the tier of the text the diff edits, would do. One
  phrase would settle it.

## Out of scope

- `GOVERNANCE.md:42` says "without an AI supervisor pass", while the
  overlay says "verdict". Both wordings were there before this change.
- The converge skill lives outside the repository
  (`~/.claude/skills/converge`). B3's fix belongs in the repository texts,
  but the skill's Codex step may also deserve its own update.
- `.agent-protocols.local.md:39` is over 80 columns. That was true before
  this change.
