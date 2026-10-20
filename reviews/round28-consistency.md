# Tide consistency review, round 28: confirmation of the round-27 answers

## Verdict

**approve with notes**

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `417fbd5`
("Decide v0 contract details from the E1 dialect round (fx-eky)"):
`git diff -- design/`, which changes `01-design.md` (D15),
`03-authority-concurrency.md`, `reference.md`, and
`reference-companion.md`. It implements bead fx-i5p, whose owner comment of
2026-10-20 records the decisions on G3, G4, G5, G6, and G9. Scope, as asked:

- whether round 27's items (B1, N1–N3) are closed as its Resolution claims;
- whether those last edits contradict 01, 03, or the reference pair;
- a spot check that rounds 25 and 26's Resolutions still hold.

The rest of the change was not re-audited; rounds 25–27 did that. The token
count was not rerun, as instructed; the companion's figure is taken as the
authoring session's measurement.

## Round-27 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 | **Resolved** | TASK-2 now reads "A channel disconnect alone sets no bit, so an `Err` returned after one by a child that has observed no such cancellation is an independent failure (§6.4)" (`03:764–766`). It sits directly after the unchanged bit rule ("set when a checkpoint … returns a cancellation error that the body's `Err` or a sibling's failure caused; it never inspects `E`", `03:758–762`). So round 27's trace (child 0 sees the sibling-caused cancellation, then `try_recv` returns `Disconnected`, then child 0 returns `Err`) now has one outcome: the bit is set, child 0 is an echo, and rule 2 reports child 1. §6.4 says the same (`03:850–854`). 01 D8 (`01:534–545`) and reference 7.1 (`reference.md:608–616`) are unchanged and agree. Reference 7.3 (`reference.md:696`) and companion 14.1 (`companion:50`) say "a disconnect alone is not an echo". This keeps the owner's G3 decision, "a disconnect is never an echo" (fx-i5p comment): the disconnect itself never creates an echo, and only an earlier observed cancellation does. See N1 for a wording difference between TASK-2 and §6.4. |
| N1 | **Resolved** | `companion:50`: "the `Err` could outrank the failure that dropped the peer". This agrees with rule 2's spawn-order choice (`03:751–754`). |
| N2 | **Resolved** | `03:108`: "As Rust's does on Linux, a `ReadDir` ends after yielding its first error." Tide's own first-error rule is unchanged. |
| N3 | **Resolved as recorded**; not re-measured here | `companion:12–14` records 24,995 tokens on 2026-10-20, 5 under D14's 25,000 (`01:826–830`). The last edits add only "alone" to reference 7.3 (`reference.md:696`); the other B1 and N1 edits are in 03 and the companion, outside the budget. |

**Contradiction check of the last edits.** The edited sentences agree with
each other and with D8:

- 03 TASK-2 and §6.4 both make the disconnect a non-event for the bit.
- Reference 7.3 points to 7.1, which states the echo rule in D8's terms.
- `grep -n disconnect` over the four changed files finds no other
  disconnect-classification claim.
- 01 does not mention disconnects, and needs no change: its echo rule
  already depends only on the observed cancellation.

`git diff --check -- design/` passes.

**Rounds 25 and 26, spot-checked in the final text.**

- **Round 25:**
  - B1: `companion:47–48` and `03:845–848`.
  - B2: `OsStr` and `OsString` are listed at `reference.md:40`.
    `reference.md:383–385` says a `DirEntry` carries the `Fs`'s authority,
    and `03:72` has the `ReadDir`, `DirEntry` row.
  - N1: "(provisional)" is on the 1 ms figure at `reference.md:535`.
  - N2: `03:127` says `Denied`, `Dns`, and `Io` are not transient.
    `reqwest` appears nowhere.
  - N4: `03:108`, and `lines()` at `03:118–120`.
  - N5: `recv_deadline` is a checkpoint at `03:146` and
    `reference.md:692`. Replay points to companion 17 at
    `reference.md:542` and `:1062`.
  - N6: `03:836–840`.
- **Round 26:**
  - B1: `03:115–118`, with its companion row at `companion:49`.
  - N1: `companion:49`.
  - N2: "re-exported std types" at `reference.md:557`, as in
    `01:427`.
  - N3: the dependency rule at `03:1172–1174` and `companion:447–448`.
  - N4: `03:836–838`.

All still hold. Round 26's N6, the `remove_dir_all` crash behavior, is still
the pre-existing omission it reported.

## New blocking problems

None.

## Non-blocking notes

### N1. §6.4's condition is broader than TASK-2's

TASK-2 says a disconnect's `Err` is independent for a child "that has
observed no such cancellation" (`03:765`), meaning one that the body's
`Err` or a sibling's failure caused. §6.4 says "and has not already observed
a cancellation" (`03:851–852`), with no qualifier. §6.4 states only a
sufficient condition, and that condition implies TASK-2's, so the two do not
contradict. A reader might still take the contrapositive: that a child that
has observed some other cancellation, such as the scope's own deadline, is
not independent. TASK-2's rule 2 handles that child by its deadline
exception (`03:752–754`), not as an echo. To match TASK-2, change it to
"has not already observed such a cancellation (TASK-2)". This costs no core
tokens.

### N2. Wrapping nits in 03

- `03:1175` is 85 columns ("… This is POSIX's rename atomicity carried").
  The next line, `03:1177`, is short. It is the only changed prose line
  over 80 columns.
- `03:842` ends after 46 columns ("with `Disconnected`. `try_send` and
  `try_recv`"), a leftover from an earlier rewrap.

Reflow both paragraphs. Neither changes meaning.

## Ready to merge

**Yes.** Round 27's B1 is closed. D8's echo rule, TASK-2's bit algorithm,
reference 7.1 and 7.3, and companion 14.1 now give one outcome for a
disconnect after an observed cancellation. N1 and N2 of round 27 are
closed, N3 is recorded, and the round 25 and 26 Resolutions still hold. The
two notes above are wording points that cost no core tokens. They can land
before merge without another round.

Validation: this was a read-only review of the working tree at `417fbd5`,
with four `design/` files modified, on 2026-10-20.

- **Read:**
  - round 27 in full, including its Resolution;
  - the Resolutions of rounds 25 and 26;
  - 01 D8 and D15's diff;
  - in 03: TASK-2, §1.1's directory and stdin bullets, §6.4, and §9.5's
    crash paragraph;
  - reference 6.3's `Fs` block, 7.1, and 7.3;
  - in the companion: the count line, 14.1's new rows, and 17's crash
    bullet;
  - `br show fx-i5p` and `br comments fx-i5p`.
- **Searched:** grepped the four files for "disconnect", and for the
  round 25 and 26 phrases cited above.
- **Line lengths:** checked every changed prose line, excluding table rows
  and code, for more than 80 columns.
- **Diff check:** ran `git diff --check -- design/`.

The token count was not rerun, by instruction. No Tide checker exists, so
every verdict above is an inference from the text. Only this file was
written; nothing was staged or committed.

## Resolution

Both notes applied before commit: section 6.4 reads "has not already
observed such a cancellation (TASK-2)", and the two 03 paragraphs are
rewrapped. The verdict is approve with notes, so no further round is run.
