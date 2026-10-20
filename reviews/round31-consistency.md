# Tide consistency review, round 31: confirmation of the round-30 answers

## Verdict

**approve with notes**

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `f74dace`
("Decide std and simulator details from the E1 dialect round (fx-i5p)"):
`git diff -- design/`, which changes `03-authority-concurrency.md` (the
§1.1 `Fs` row, the §1.1 append and durability note, §9.5), `reference.md`
(6.3's `Fs` comment and Durability bullet), and `reference-companion.md`
(17 Crashes, the recorded count). It implements bead fx-ngm, with round
30's Resolution applied. Scope, as asked:

- whether each round-30 note (N1–N4) is closed or declined as its
  Resolution claims, and whether the reasoning for declining part of N2
  holds;
- whether the reworded opening of 03 §9.5 ("Data made durable ...
  survives, provided the entries of the directories above it are durable
  too") is consistent with 01 D15, reference 6.3's Durability bullet,
  companion 17, and companion 15.4's crash-safe log;
- whether the crash-rule clauses of 03 §9.5 still form a coherent,
  non-circular set in which a synced change survives whenever the rules
  say it does.

Nothing beyond that was re-audited. The token counter was not run, since
the core did not change in this pass.

## Round-30 item status

| Item | Status | Evidence |
| --- | --- | --- |
| N1 | **Resolved** | `03:1188–1190`: "a directory's removal, by `remove_dir` or by a `rename` over it, survives only if every change that emptied it does". Companion: `companion:452–454`, "(by `remove_dir` or a `rename` over it)". Round 30's example (`rename("d/x", "e/x")`, `rename("c", "d")`, `sync_dir(".")`) now forces the first `rename` to survive. |
| N2, creation rule | **Resolved** | `03:1186–1187`: "the changes that created its entries and every directory above its paths". Companion: `companion:450–451`. A path-keyed simulator never replays a `rename` from a missing source. |
| N2, general closure | **Declined. The reasoning holds for the case it names, but the wording leaves two gaps** (N1, N2 below) | `03:1190–1194` keeps the closure for removals and says "durability does not flow the other way, from a synced entry to the unsynced creation of a directory above it". That is right for the directories above the synced entry. POSIX does not guarantee them, and reference 6.3's acknowledged write syncs the parent for that reason (`reference.md:419–421`). But the reworded opening leaves out the file's own entry (N1). The other dependencies (the name rule, and the source side of a `rename`) still carry durability without saying so (N2). |
| N3 | **Resolved** | `03:101–104`: "`remove_dir_all` on a path that names the base empties it and leaves it in place (Rust then reports `InvalidInput` for `"."`); `sync_dir(".")` makes the emptying durable." The on-disk outcome is now stated for every spelling, `sub/..` included, and Rust's return value is left to Rust. The durability claim follows from the removal closure (`03:1190–1192`). |
| N4 | **Resolved** | TASK-3: `03:99–100`, "since cancellation (TASK-3) or an error can stop it between paths", which TASK-3's level-triggered checkpoints (`03:782–784`) support. "Only data made durable" is gone from `03:1174` and `companion:442`. |

**Consistency of the reworded opening.**

- **01 D15** (`01:845–848`) says unsynced writes may be lost or torn,
  that each unsynced entry change survives whole or not at all, and that
  atomic replace works as on a journaling file system. D15 makes no
  promise about synced data in a directory whose own entry is unsynced, so
  the new proviso narrows nothing D15 states. Atomic replace still yields
  the old file or the new one. If the parent's creation is lost, the
  target is absent, which was its state before the replace. D15 is still
  accurate.
- **Reference 6.3** (`reference.md:415–423`). The proviso matches the
  acknowledged write: "the first time a file or directory is created,
  `sync_dir` of its parent". Applied to each directory created on the way
  down, that rule makes every entry above the data durable. The atomic
  replace line syncs the entries of `"."`. The core says nothing about
  ancestors, so the opening adds detail without contradicting it. The
  opening's omission of the file's own entry is N1.
- **Companion 17** (`companion:442–443`): "Synced data survives (6.3) if
  the directories above it are synced too". Syncing every directory above
  the data covers the file's own entry, held by its parent, and each
  ancestor's entry. The companion's condition is therefore sufficient, and
  slightly differently worded from 03's. The rule list (`companion:449–454`)
  matches `03:1185–1190`. The companion does not state the removal closure
  in general, but it gives the `remove_dir_all` consequence
  (`companion:447–449`).
- **Companion 15.4** (`companion:300–368`). The test asserts that every
  acknowledged line survives (`companion:359`). Before any line is
  acknowledged, the protocol has made durable:
  - each line's bytes, by `self.file.sync()` (`:326`);
  - the `events` entry, by `dir.sync_dir(".")` (`:313`);
  - the `/data` entry, by `r.fs.sync_dir("/")` (`:343`). `/` is the
    simulated root, which has no entry.

  So the proviso holds under 03's wording and the companion's. No "only if"
  rule can take a line away, since the protocol reuses no name and removes
  nothing. A crash between `open_with` and `sync_dir` comes before any
  acknowledgement, and the torn-tail cut drops only unacknowledged bytes.
  The protocol still guarantees what the test asserts.

**Coherence of §9.5.**

- **No cycle.** Each rule still points from a change to earlier ones:
  - the creations of its entries and of the directories above both
    paths;
  - the change that freed its name;
  - the changes that emptied a removed directory.

  Dependency follows program order, so the relation is acyclic.
- **Synced changes can always survive.** Each rule is an "only if", so
  the set of all changes satisfies every rule. The synced changes, closed
  under their dependencies, form a smaller set that also satisfies every
  rule. The only exemption, the denial at `03:1192–1194`, is matched by
  the opening's proviso: a synced change whose directories above are not
  durable is not promised to survive. Read literally, the set is coherent.
  It relies on two readings the text does not state, which are N1 and N2.
- **Replay.** With the creation rule over both paths, each surviving
  change finds its precondition met. A `rename` finds its source, and a
  `remove_dir` finds its directory empty. Round 30's argument for
  reachability is unchanged.

**Leftovers.** `git diff --check -- design/` passes. The only added line
over 80 columns is the §1.1 table row (`03:69`), a table convention.

## New blocking problems

None.

## Non-blocking notes

### N1. The opening's proviso omits the file's own entry

`03:1174–1177`: "Data made durable by `sync` or `sync_dir` (§1.1)
survives, provided the entries of the directories above it are durable
too". `file.sync()` makes a file's contents durable (`03:92–93`) but not
its entry. Take this program:

```rust
let mut f = fs.create("f")?;   // entry unsynced
f.write_all(b"x")?;
f.sync()?;                     // contents durable; no sync_dir
```

No directory above `f` lacks a durable entry, so the opening promises that
the data survives. The `create` is an unsynced entry change that "survives
whole or is lost" (`03:1178–1179`). Read one way, the two sentences
contradict each other. Read the other way, `sync()` carries the `create`
with it, because the denial at `03:1192–1194` exempts only directories
above. Then the simulator never loses a file whose first-time `sync_dir`
is missing. That is the bug reference 6.3's acknowledged write
(`reference.md:419–421`) and companion 14.1 (`companion:71`, "`file.sync()`,
plus `fs.sync_dir(parent)`") tell the programmer to prevent, and the
simulator would stop catching it. Companion 17's wording, "if the
directories above it are synced too" (`companion:442–443`), avoids the gap,
because syncing the parent covers the file's entry. 03 is the owner and
should say it too.

**Fix** (03 §9.5 only, no core tokens): "survives, provided its entry and
the entries of the directories above it are durable too".

### N2. Durability still flows through the name rule and a `rename`'s source side, unstated

The denial at `03:1192–1194` blocks durability from flowing to "the
unsynced creation of a directory above" a synced entry. The text states
that durability flows in one case only: a synced removal carries the
changes that emptied the directory. Two other dependencies must also
carry durability, or the opening's promise fails. The text says neither.

- **A `rename`'s source side.** In this program, `d` is created and never
  synced in `"."`, and `e` is durable:

  ```rust
  fs.create_dir("d")?;            // d's entry in "." unsynced
  fs.write_synced("d/x", data)?;  // syncs d's entries, not d's own
  fs.rename("d/x", "e/x")?;
  fs.sync_dir("e")?;
  ```

  The data now lives at `e/x`, and the directories above it are durable,
  so the opening promises it survives. The creation rule, over "its
  paths", lets the `rename` survive only if `d`'s creation does. `d` is a
  directory above the `rename`'s source path, so if the denial covers it,
  `x` can end up nowhere. No file system reaches that state after
  `fsync(e)` has returned: an ext2-style file system keeps `e/x`, and a
  journaling one keeps both. The rules stay coherent only if `sync_dir("e")`
  carries `d`'s creation, and nothing says so.
- **The name rule.** `create_dir("e")` (unsynced), then `rename("d/a",
  "e/a")`, then `create("d/a")`, then `sync_dir("d")`, with `d` durable.
  The new `d/a` is synced, and its directories above are durable. It
  survives only if the `rename` that freed its name does, and the `rename`
  survives only if `e`'s creation does. `e` is not above `d/a`, so the
  denial does not reach it, but no sentence says the flow happens.

Round 30's N2 warned about exactly this: stating the closure for one rule
invites the reading that the other rules carry nothing. The new denial
makes that reading more likely. The reason for declining holds: the
synced entry's own ancestors are what POSIX leaves unsynced. Only the
scope of the denial is unclear.

**Fix** (03 §9.5, one clause, no core tokens): after the denial, "Every
other change a durable change depends on is made durable with it", and
make the denial's "a directory above it" read "a directory above the
entry it synced". Companion 17 can keep its shorter summary.

## Ready to merge

**Yes.** Every round-30 note is closed, and the partial decline of N2
rests on sound reasoning. 01 D15 and reference 6.3 still agree with 03.
Companion 17 summarizes it faithfully, and companion 15.4's log still
guarantees what its test asserts. Read literally, the §9.5 rules are
acyclic and coherent. N1 is a one-phrase fix in 03 and is worth landing
before merge, because one reading of it lets the simulator hide a missing
first-time `sync_dir`. N2 is optional, but cheap.

Validation: this was a read-only review of the working tree at `f74dace`,
with `.beads/issues.jsonl` and three `design/` files modified and
`reviews/round29-consistency.md` and `reviews/round30-consistency.md`
untracked, on 2026-10-20.

- **Read:**
  - the full `design/` diff;
  - fx-ngm via `br show`;
  - round 30 with its Resolution;
  - in 03: §1.1 (table, Paths, Append and durability), TASK-3, and §9.5;
  - 01 D15;
  - reference 6.3 (Paths, Durability, Torn tails);
  - companion 14.1's `sync_all` row, 15.4, and 17 Crashes.
- **Checks:** `git diff --check -- design/` passed, and added lines were
  checked for width. The token counter was not run, as instructed.

No Tide checker or simulator exists, so every verdict on Tide behavior is
an inference from the text. Only this file was written; nothing was staged
or committed.

## Resolution

Applied before commit.

- **N1:** 03 §9.5 reads "provided its entry and the entries of the
  directories above it are durable too"; companion 17 matches.
- **N2:** 03 §9.5 adds "every other change a durable change depends on is
  made durable with it". The exception is widened past the suggested
  wording so that it also covers the entry's own creation: "durability
  never flows from synced data or a synced entry to the unsynced creation
  of its own entry or of a directory above it". Without that,
  `file.sync()` would make an unsynced `create` durable and undo N1.

The verdict is approve with notes, and this slice has had its two review
restarts (rounds 30 and 31), so no further round is run.
