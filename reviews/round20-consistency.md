# Tide consistency review, round 20: confirmation of the round-19 answers

## Verdict

**approve with notes**

Reviewed commit `51240dddc6afd7ef21daf472b13f31d19b5ca54c` (`51240dd`, "Add
round 19 and answer it: restore the sink rule, companion citations"), dated
2026-10-19 20:06 -0300. It was diffed against `6d5285c`. Scope, as asked:

- whether each round-19 item (B1, B2, N1–N5, and the budget) is closed as
  its Resolution claims, the declined rename included;
- whether the changed lines contradict 01, 02, 03, 04, the README, or the
  rest of the reference pair, and whether any citation into either
  reference file from `design/0*.md` or `README.md` is stale;
- whether the count method, now in the companion's header, can be repeated.

Rule preservation beyond the changed lines was not re-audited; round 19
did that.

## Round-19 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 | **Resolved** | `reference.md:449–450` now reads "Format arguments are always evaluated and the record formatted, whatever level is enabled, so sink configuration never changes results or panics." This matches 01 D7 (`01-design.md:443–445`) and 03 SINK-3 (`03:287–292`), and adds nothing either lacks. |
| B2 | **Resolved** | `03:19`, `03:1053`, and `03:1136` cite `reference-companion.md`, and Appendix A names `pct` and `--minimize` (`companion:459–463`). `03:29` reads "in `reference.md` or its companion". |
| N1 | **Resolved**, with a wording point (N3 below) | `reference.md:32–33`. |
| N2 | **Resolved** | `reference.md:105–109` restores the old list and the derive-helper wording; only "They are" became a colon. |
| N3 | **Resolved** | `reference.md:528` carries the setup marker, and `companion:397` keeps only `SimNode` and `NodeTask`. Companion 17's rows are faults and nodes, not setup, so nothing there lost its marker. |
| N4 | **Resolved**, one leftover in N4 below | `reference.md:336` gives `fault_point(name) -> bool` and "`false` outside simulation, companion 17". Both agree with 03 §1.1 (`03:105–107`), and companion 17 has the `sim.faults.fault_point(name, p)` row. |
| N5 | **Resolved**; the rename was **declined, reason holds** | `reference.md:846–850` is rewrapped to 80 columns or fewer. `Id<T>` reads "and also `Ord` and `Debug` (provisional)" (`:235–236`). `Stdin` points to the `&self` rule (`:334`). `impl Trait` captures "every generic parameter in scope, lifetimes included" (`:201–202`), which is Rust 2024's rule. |
| Budget | **Resolved**, measured below; two notes (N1, N2) | The core is under 25,000 tokens, and `reference.md:11` points to the companion's count. |

**The declined rename.** The reason is sound: 02 §5.1, §8.1, and the
`Pure` section use "companion trait" (`02-memory.md:462`, `:644`, `:821`),
and 01 does as well (`01-design.md:956–957`). Renaming the term in the core
alone would split one concept between two names. The cited locations are
slightly off (N4).

**Measured count.** `count.sh design/reference.md` returned **24,950**
tokens. That equals the companion's recorded figure (`companion:12`) and
leaves 50 tokens of headroom.

**Citations.** In `design/0*.md` and `README.md`, every citation of
`reference.md` or `reference-companion.md` resolves:
- `01:175` cites §12.1–§12.2, which are at `reference.md:973` and `:983`;
- `03:582` cites §6.10, at `reference.md:568`;
- `03:978` cites the companion for `par_reduce`;
- the B2 lines above;
- `README.md:57–60` lists both files.

02 and 04 do not cite either file. None of the changed lines contradicts
01, 02, 03, 04, or the README.

## New blocking problems

None.

## Non-blocking notes

### N1. Moving the count off the core's header departs from a recorded owner decision

fx-wsd records the owner's decision of 2026-10-19: "keep D14's 25,000;
record count and method in the reference header". fx-fbq's Evidence still
says "Method (recorded in the reference header)". This commit moved both
the count and the method to the companion's header (`companion:12–15`), and
the core now says only "the companion records the count" (`reference.md:11`).

The companion describes itself as "the on-demand part of
`design/reference.md`" (`companion:3`), so the pair's header arguably still
holds the count. But the owner made the decision when the reference was one
file, and neither bead records that the owner accepted the move. Moving the
count back would cost about 100 tokens, and the core has 50 to spare.

**Fix.**
- Before fx-fbq closes, record on it the owner's acceptance of the
  companion's header as the place for the count and method.
- Update the bead's Evidence line to say the method is in the companion's
  header.
- If the owner declines, the core needs about 50 tokens of other cuts
  before the method can return to it.

### N2. The recorded method leaves out two things the measurement depends on

`companion:12–15` says to take the "input tokens that `claude -p ...
--output-format json` reports". The script that produced 24,950 differs from
that text in two ways:

- **Which fields.** The script adds `usage.input_tokens`,
  `cache_creation_input_tokens`, and `cache_read_input_tokens`. In the
  Anthropic API's usage report, `input_tokens` leaves out tokens written to
  or read from the prompt cache. A reader who takes only `input_tokens`
  can therefore get a different number when caching applies. That
  `input_tokens` alone diverges here is an inference from the API's field
  definitions and from the script's choice; this review did not measure it.
- **Stdin.** The script runs `claude -p` with stdin closed. The method does
  not say so. In an unattended shell, an open stdin can stall the run or
  add to the prompt.

The count is also not tied to a revision of the core. The old text said
"Measured on this text". Now that the count is in another file, a later
edit to the core can leave `companion:12` stale without any sign of it.
With 50 tokens of headroom, the next edit to the core probably will.

**Fix.** These edits cost no core tokens.
- In `companion:13–15`, say "the sum of `usage.input_tokens`,
  `cache_creation_input_tokens`, and `cache_read_input_tokens` that
  `claude -p ...` reports, with stdin closed".
- Record the core revision for each count on fx-fbq, or name it in the
  header ("the core as of `<commit>`").

### N3. "Std's traits" can be read as all of Rust's std traits

`reference.md:32–33` says "Std's traits, every `Iterator` adapter included,
are admitted (6.9)". 6.9 (`:559`) uses "std inherent items" for Rust's std,
and ADMIT-1 (`03:333–338`) also means Rust's std by "std's types". Right
after a pointer to 6.9, then, "Std's traits" reads as every trait in Rust's
std. Two passages narrow that:

- 7 (`reference.md:583`) says there is no `Future` or `Pin`;
- 6.9 and GLOBAL-4 (`03:210–213`) exclude the `Hash` and `Debug` impls on
  raw pointers.

Specific-over-general settles both, and round 19 accepted this wording, so
nothing has to change. If the core gains headroom, the old positive list
(`Iterator`/`IntoIterator`, the conversion, formatting, operator, and `Fn`
traits, `Default`, `Deref`) would remove the ambiguity.

### N4. Nits

- **The Tasks row lost its 7.1 pointer.** The old row ended "`fault_point`
  (7.1)". That citation also served `scope`, `spawn_isolated`, `shield`, and
  `checkpoint`, which 7.1 declares (`reference.md:588–597`). The N4 edit
  replaced it. Adding "(7.1)" after `yield_now` at `:336` restores it for
  about 3 tokens.
- **Line length.** `03:20` became 85 characters in the B2 reflow.
- **Resolution citations.**
  - For N4, the Resolution cites 03 §1.2. `fault_point` is defined in
    03 §1.1 (`03:105`); §1.2 covers synchronization handles.
  - For the declined rename, it cites D7 and 03 §3.1. In 01 the term is in
    D18's list of changes to D7 (`01:956–957`), not in D7 itself. 03 §3.1
    says "companion supertrait" (`03:379`, `:400`). The term itself is in
    02 (above).
- **Headroom.** The core has 50 tokens left. Any later fix to it, whether
  from a review or from E1's held-out round, will probably have to trade
  text for text.

## Ready to implement v0

**Yes.** Round 19's blocking items are closed. The core states the sink
rule as 01 and 03 do, and 03 now credits the companion with the exclusions
it holds. The core measures 24,950 tokens, under D14's budget, so fx-jv9 can
load it. Before fx-fbq closes, record N1's owner acceptance on the bead.
N2's method fix can land with it, since it costs no core tokens.

Validation: this was a read-only review of `51240dd` (diffed against
`6d5285c`) on 2026-10-19.

- **Read:** the full diff, the changed regions of the core and the
  companion in context, Appendix A, companion 17, 03 §1.1, §2.5, SINK-3, and
  §3.1, 01 D7 and D14, and the README's design list. fx-fbq and fx-wsd were
  read with `br show` and `br comments`.
- **Citations:** grepped for `reference.md` and `reference-companion.md` in
  `design/0*.md` and `README.md`, and checked each hit against its target.
- **Count:** `count.sh design/reference.md` was run once, with stdin closed
  and a 300 s timeout, on the working tree at `51240dd` (clean). It uses
  `claude -p --model claude-opus-5-5` and differences against the prompt
  `.`. It printed `24950`.

No Tide checker exists, so every Tide verdict above is an inference from the
text. Only this file was written; nothing was staged or committed.

## Resolution

Answered in the commit that adds this review.

- **N1:** the owner accepted the count's new home in the companion header
  (2026-10-19, recorded on fx-fbq), superseding fx-wsd's decision to keep
  it in the reference header.
- **N2:** the owner chose to commit the counting script as
  `scripts/count-tokens.sh`, and the companion's method line now names it.
  The converge review of the script found that the MCP servers'
  instructions made the baseline vary by up to 98 tokens between runs.
  `--strict-mcp-config` removes the variation. The script also passes `--`
  before the prompt and reports API errors. On the final text it reads
  24,954 tokens on every run. Counts this file and earlier ones took with
  the scratch `count.sh` may be off by about 100 tokens.
- **N3:** no change, as the note allows.
- **N4:**
  - the `Tasks` row's `(7.1)` pointer is back;
  - `03:20` is rewrapped;
  - corrections to round 19's Resolution: `fault_point` is defined in 03
    §1.1, and "companion trait" is the term used in 01 D18's list of
    changes to D7 and in 02. 03 §3.1 says "companion supertrait".
