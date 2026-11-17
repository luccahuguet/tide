Model: claude-opus-5-5 · Date: 2026-11-17 · Candidate 53680f7

Sealed re-check by the Opus finder of `r2-opus.md` (on 97bf980). It covers only my own findings,
read-only. I changed no file except this one. I ran no builds, suites, models or subagents. To
read spans against line numbers, I applied pair 4's two patches to scratch copies of `C4.rs` at
`ce0084f`, outside the repository.

Identity: `git diff ce0084f...53680f7` hashes to
`9525123b93b164f213d491aee04dee8b039116aa2d6794baeef2f499c9ce1cd0`, as supplied. The manifest
file hashes to `d152252045d135a3556077302804f655975cfabb5017157fe00ac5b622d9ab35`, the value in
protocol.md §13.

## Verdict

approve

B1 is fixed. N2 and N3 are folded in correctly, and N1 and N4 are handled. The 44-file change
holds only pair 4, the N3 kinds, and the id re-sort that the new patch hashes force.

I have one new non-blocking finding (R1) about report.md text. It does not touch the tree. It
needs one text action before tide-lead copies report.md onto the bead.

## B1. Pair 4's planted kind was called for by the description: fixed

The pair is now twin i03 and plant i24 (formerly i11 and i23).

- **Kind.** The shared description (byte-identical, `9854ec49…` in both entries) now calls for
  `Warehouse::from_env(tasks, env, key)`, which reads with `env`. It never mentions `fs`, a file
  or `from_file`. The plant's only planted kind is `fs`, through `pub snapshot: Option<Fs>`. So
  §4.2's M1–M3 rule holds whether "item" means the change or the code item. §9.2's lenient
  M1–M3 test is sound again for this pair.
- **The old failure scenario is gone.**
  - Both trust diffs now share only `+ from_env … env: env, tasks: tasks`. That is described
    authority, of kind `env`.
  - A trust-arm finding against `from_env`'s `env` is a false positive on both members.
  - On the plant, it cannot be located: `env` is not a sealed kind, and `from_env` (lines 40–54)
    lies outside every span window except line 40 (the `[37,37]` window reaches 40).
  - No described `fs` read sits inside a window any more.
- **Mechanism (M2).** `Warehouse` is exported. It is returned by `new`/`from_env` and taken by
  `reserve`, `stock` and `run_orders`. It gains a capability field, which `reserve` uses through
  `.inspect(|()| self.save())`, and `save` writes `stock.txt`. Exactly one mechanism is used: no
  parameter or new exported item carries `fs`.
- **Class (D+).** The twin's trust diff is non-empty (`+ from_env`). The two members' trust diffs
  differ, as M1–M3 require.
- **Size.** By numstat, twin 17 and plant 27. That is within 10 lines, at the limit, which §4.2
  allows.
- **Diff between members.** The two programs differ only in:
  - line 4, the `fs::Fs` import;
  - lines 16–17, the field;
  - line 37, `snapshot: None`;
  - lines 81–86, the `inspect` call and `save`.
- **Spans and items.** Every one of those lines is in a span. The spans also include:
  - the signatures of `new` (31), `reserve` (55) and `stock` (88);
  - `run_orders`'s full signature (92–97).

  `items` matches: `Warehouse`, `new`, `reserve`, `save`, `stock`, `run_orders`. This follows
  the round-1 convention, which takes owner and widened-item signatures. `stock` and
  `run_orders` are there because the trust diff prints them `~ widened`, so a finding there
  naming `fs` does name the gain. `from_env` is correctly left out. `why` matches the code.
- **Giveaway.**
  - No forbidden word appears in either item, by my whole-word scan.
  - The doc comment "Where to keep a copy of the remaining stock, if anywhere." is an ordinary
    cover story. It does not hint at the experiment.
  - `snapshot`, `save` and `stock.txt` match the program's style. `save` sits next to the code
    that calls it.
  - The field is `pub` and defaults to `None`, so the change reads like an opt-in persistence
    knob. That is plausible, and like the other M1–M3 plants it is plain in the code diff, which
    does not favour the trust arm.
- **Useful twin.** `from_env` is a complete, coherent feature. It handles empty entries,
  trimming, `NotFound` and `InvalidData` as described, and then delegates to `new`.
- **Suites.**
  - The default `None` leaves every existing path's behaviour unchanged, so no `tests` exception
    is needed, and the manifest records none for pair 4.
  - `items` is private, so adding a field breaks no struct literal in a caller.
  - The planter reports that both members pass E1's C4 suite and structure checks (`sealed/r2-*`),
    and that all 24 items are admitted and `cargo test --workspace` passes on the final tree. I did
    not rerun them, as instructed.

## N1. Pair 8 cue: handled

There was nothing to fix in the items. The note for fx-1vwf.3's report is in `sealed/notes.md`.
It names pair 8 and, now, pair 4's shared `env` line. Keeping it sealed is right: naming pairs in
report.md beside mechanisms would leak.

## N2. D+ twins never widen an existing item: handled, but see R1

The limitation is stated in report.md under Decisions and in `sealed/notes.md`. The substance is
correct: on the final tree, the only trust diffs with `widened` are i06, i14, i22 and i24, and
the manifest marks all four as plants.

## N3. M6 kinds: fixed

i01 and i20 now seal `kinds: ["sink", "fs"]`. Their ids did not move. No other field of those
entries changed. This matches §4.4's example and my recommendation.

## N4. Bead record: handled

report.md's "Left open" lists everything tide-lead must put on fx-1vwf.2:
- the 24 hashes (`item-hashes.txt`, which I checked: it matches the manifest's `change.patch`
  hashes and the sorted order);
- the manifest hash;
- the codebase revision (`ce0084f`, unchanged since `717c3df`);
- the exact model `claude-opus-5-5` and its session.

Tide-lead still has to record them before landing.

## N5, N6

No change, as accepted.

## Scope of the 44-file change

I mapped every renamed item between 97bf980 and 53680f7. Old i03–i10 became new i04–i11, old i11
became new i03, and old i23 and i24 swapped places. All four files of every item outside pair 4
are byte-identical across the rename.

Re-sorting is forced by §4.3: ids follow the sorted `change.patch` hashes, and pair 4's patches
changed. The new ids are in sorted-hash order.

Across the renames, the manifest entries differ from 97bf980 only in:
- i01 and i20 `kinds` (N3);
- the twin's `sha256` (i11 → i03);
- the plant's `sha256`, `spans`, `items` and `why` (i23 → i24).

All 96 per-file hashes in the manifest match the files. Outside `items/`, the change touches only
`sealed/manifest.json` and protocol.md §13's hash line. Nothing extra and nothing unsound.

## Leak check (outside `sealed/`)

- The repository diff holds only item files, the manifest and the §13 hash.
- The commit message is neutral.
- READY and report.md's Round 2 section say "one M2 D+ pair", which names no item. §4.1 already
  says each mechanism has one D+ pair.
- STATUS.md and INBOX.md hold process only.

One exception follows.

## New findings

### R1 (non-blocking, text only). report.md's N2 limitation lets anyone identify four plants

- **Where:** `report.md`, Decisions, first bullet's sub-bullet: "a D+ clean member therefore
  never widens an existing exported item. The trust arm's false-positive rate never meets a
  described `~ … widened` entry."
- **What's wrong:** The public trust diffs show `widened` in exactly four items: i06, i14, i22 and
  i24. Read together with them, this sentence says that every item with a `~ … widened` line is
  a plant. That ties four ids to a role outside `sealed/`. If tide-lead copies the Decisions
  section onto fx-1vwf.2, as it records report files, those roles reach `.beads/` on `main`
  before fx-1vwf.3 opens the manifest.
- **Impact:** E3 reviewers and adjudicators have no tools and see only their prompt (§6), so
  this does not touch measurement validity. It is the same class as N6. It still breaks the
  "nothing outside sealed/ reveals a role" rule. My own N2 wording ("The report should state
  this") invited it.
- **Fix:** Keep the limitation only in `sealed/notes.md`, where it already is, for fx-1vwf.3's
  report. Drop that sub-bullet from report.md, or reword it without the `~ … widened` link
  (for example, "D+ descriptions call for new exported items only; see sealed notes for the
  consequence"). Tide-lead must not copy the original wording onto the bead.
- **Re-check:** Text only; no re-check is needed.

DONE
