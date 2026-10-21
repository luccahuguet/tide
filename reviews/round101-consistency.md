# Tide adversarial/consistency review, round 101: review-loop confirmation (fx-djp)

## Verdict

**approve**: no blocking findings, two non-blocking notes.

The revision closes all six blocking findings from rounds 99 and 100 and
handles round 99's notes N1 to N5. GOVERNANCE.md, the overlay, README, and
fx-djp state the same order and the same tier test. One wording overlap
remains, for a change that edits v1 text (N1 below). The block test reads
first and decides the case, so the overlap does not block.

## What was reviewed

The uncommitted working tree against HEAD `bcf8866`: `git diff -- .
':!.beads'`, SHA-256
`13d20cad3cb792d0b5c5f0dd99cb396472283f568d17bb8b655c96d52c8f4e67`
(recomputed, matches). `git diff --check` passes.

- `sha256sum .agent-protocols.local.md` gives `5192cfac...2e75`, which
  matches the manifest's new `overlay.sha256`.
- The repository-local section of AGENTS.md (from `### Tide` on) is
  byte-identical to the overlay.
- I read GOVERNANCE.md's Merge Rule (lines 36-76), the overlay's
  Governance bullets, README's governance bullet, `br show fx-djp`, the
  tier table (`design/01-design.md`, Tiers), and rounds 99 and 100.

## Closure of rounds 99–100

| Finding | Status | Where |
| --- | --- | --- |
| 99 B1, earlier-tier hole | Closed | "a tier the change edits or an earlier one" in GOVERNANCE:48, the overlay, and fx-djp |
| 99 B2, missing test and dropped "only" | Closed | The test is now "its fix would change ...", stated in all three. The overlay says "only later-tier text the change does not edit". fx-djp now gives round96 B1 as a v0 defect that *blocks*, not as a case to defer |
| 99 B3, converge's own Codex step | Closed | "The supervisor pass and its re-check replace converge's own Codex round" (GOVERNANCE:47). The overlay says "These Codex passes replace converge's own Codex round" |
| 99 B4, parallel review vs converge edits | Closed | Both first-round reviews are read-only and run on "one frozen candidate, the working tree at its recorded diff SHA-256". The converge round "reports instead of fixing", and "Nothing edits the tree until both reports are in" (GOVERNANCE:46). The overlay says the same |
| 100 B1, editing barrier | Closed | Same text as 99 B4. Later converge rounds "fix as usual" only after both reports are in |
| 100 B2, own-tier v1 conflict | Closed | An unsound rule in v1 text that the change edits fails the deferral test ("text that the change does not edit") and meets the block test ("a tier the change edits"), so it blocks. See N1 for unedited v1 text |

Round 99 notes:

- **N1** (re-check when nothing changed): handled. The re-check covers
  "the changes made since the frozen candidate, unless there are none".
- **N2** (round numbering): handled. "The two reviews take consecutive
  round numbers, Opus first". This agrees with line 74's
  `round<N>-codex.md` and with how rounds 99 and 100 were numbered.
- **N3** (README omits the tier rule and the planning-only exemption):
  handled. README now says "except a planning-only one" and "only findings
  in the tiers a change edits, or earlier ones, block it".
- **N4** ("block" vs "block or restart"): handled. Both texts say "blocks
  or restarts a round".
- **N5** (who decides the change's tier): handled. The tier is the one
  "the change edits", and the text the diff edits fixes it.

## Blocking findings

None.

## Non-blocking notes

### N1. "Later-tier" is relative in the block test but absolute in GOVERNANCE's parenthetical

GOVERNANCE:48 defers a finding "whose fix would change only later-tier
text that the change does not edit (v1 or Later in `design/01-design.md`'s
tier table)". Take a change that edits v1 text, where a finding's fix
would change other v1 text the change does not touch. The parenthetical
makes that text "later-tier", so the deferral sentence applies. The
preceding sentence also applies, because v1 is "a tier the change edits",
so the finding blocks. The block test comes first and is the stated rule.
fx-djp also says a finding goes to the bead "only when" the block test
fails. A careful reader therefore blocks, so this is not blocking. The
overlay and the bead do not have the parenthetical and read "later-tier"
relative to the change.

Optional fix: change the parenthetical to "a tier later than every tier
the change edits; for a v0 change, v1 or Later". Or drop it. "That the
change does not edit" can stay, since it covers a change that edits text
in more than one tier.

### N2. README promises a re-check that GOVERNANCE can skip

README says every non-planning change gets "a narrow Codex re-check once
the Opus reviews come clean". GOVERNANCE skips the re-check when nothing
changed after the frozen candidate. Nit. "a narrow Codex re-check of any
fixes" would match.

## Out of scope

- The converge skill (`~/.claude/skills/converge`) still describes its own
  Codex round and fixing in the first round. The repository texts now
  override it, as round 99 noted. The skill itself is outside the
  repository.
- GOVERNANCE:42 says "supervisor pass" and the overlay says "verdict".
  Both wordings predate this change.
