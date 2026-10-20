# Tide consistency review, round 30: confirmation of the round-29 answers

## Verdict

**approve with notes**

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `f74dace`
("Decide std and simulator details from the E1 dialect round (fx-i5p)"):
`git diff -- design/`, which changes `03-authority-concurrency.md` (the
§1.1 `Fs` row, the §1.1 append and durability note, §9.5), `reference.md`
(6.3's `Fs` comment and Durability bullet), and `reference-companion.md`
(17 Crashes, the recorded count). It implements bead fx-ngm. Scope, as
asked:

- whether each round-29 item (B1, N1–N5) is closed as its Resolution
  claims;
- whether the final crash rules of 03 §9.5 are coherent as a whole: every
  allowed state reachable by a real file system, every synced change
  surviving under all the dependency clauses (creation of entries and
  ancestors, name reuse, directory removal with removals and renames out
  of it, the sync closure), no clause circular or self-contradictory,
  companion 17 a faithful summary, and 01 D15 still accurate;
- whether the new N1 text matches Rust 1.96 on Linux, checked with a probe;
- the core's size, from the companion's recorded figure (not re-measured).

## Round-29 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 | **Resolved** | `03:1184–1187`: "a directory's removal survives only if every change that emptied it does, each removal in it and each `rename` out of it. A `sync_dir` that makes a directory's removal durable makes those changes durable too." Round 29's example (`rename("d/x", "e/x")`, `remove_dir("d")`, `sync_dir(".")`) now forces the `rename` to survive, so `x` is at `e/x`. Companion 17 says the same (`companion:451–453`). See N1 for the one directory removal the wording leaves implicit. |
| N1 | **Resolved**, with gaps in N3 | `03:100–102`: "An `Fs` never removes its own base: as in Rust, `remove_dir_all(".")` empties it, then fails with `InvalidInput`, and `remove_dir(".")` fails at once." The probe below confirms both halves. Other spellings of the base and the durability of the emptied contents are not stated (N3). |
| N2 | **Resolved** | `03:1183–1184`: "a change that takes a name survives only if the change that freed the name does". In the clear-and-rebuild pattern, the durable creation of the new `out` now forces the old `out`'s removal, and through B1's clause the removals inside it, so no old file can reappear. Companion: `companion:451–452`. |
| N3 | **Resolved** | `03:97–98` cites §9.5 and gives the reason: "(§9.5): nothing below a removed directory is reachable". |
| N4 | **Resolved**, citation nit in N4 | `03:98–100`: "A cancelled or failed `remove_dir_all` can leave part of the tree, as a cancelled write can leave part of its data (TASK-3)." |
| N5 | **Resolved** | `reference.md:384` spells out `remove_dir, remove_dir_all`, so a grep for `remove_dir_all` finds the core again. The companion records 24,997 tokens (`companion:12`), 3 under D14's 25,000. The core's Durability bullet says "removals" (`reference.md:417`), which covers both methods. |

**Rust facts checked.** A probe compiled with `rustc 1.96.0 --edition
2024` and run on Linux in a scratch directory outside the repository set up
`base/{f, sub/deep/g}`, made `base` the working directory, and called each
method:

| Call | Result | `base` afterwards |
| --- | --- | --- |
| `remove_dir(".")` | `InvalidInput` (`EINVAL`, 22) | intact |
| `remove_dir_all(".")` | `InvalidInput` (22) | exists, empty |
| `remove_dir_all("./")` | `InvalidInput` (22) | exists, empty |
| `remove_dir("sub/..")` | `DirectoryNotEmpty` (`ENOTEMPTY`, 39) | intact |
| `remove_dir_all("sub/..")` | **`Ok(())`** | exists, empty |
| `remove_dir("")`, `remove_dir_all("")` | `NotFound` (2) | intact |

The rust-src of 1.96.0 explains the `Ok`: `remove_dir_all_recursive` wraps
its final `unlinkat(.., AT_REMOVEDIR)` in `ignore_notfound`
(`sys/fs/unix.rs:2673–2675`), and `sub/..` no longer resolves once `sub` is
gone. So the N1 text is accurate for `"."`, and "an `Fs` never removes its
own base" holds for every spelling probed.

**Coherence of §9.5.** The surviving set is a set of whole changes closed
under four rules (`03:1181–1187`):

- **No cycle.** Each rule points from a change to earlier ones: the
  creation of its entry and ancestors, the change that freed its name, and
  the changes that emptied a removed directory. The dependency relation is
  part of program order, so it is acyclic, and no clause refers back to
  itself.
- **Synced changes survive.** The set of all changes made before the crash
  satisfies every rule, so a set containing the synced changes always
  exists. Any such set must contain their dependencies, which the "only if"
  form forces. Nothing makes a synced change unsatisfiable. N2 notes that
  only the removal rule states this closure.
- **Each surviving set can be applied.** Replayed in order, each surviving
  change finds its precondition met. A `create` or `rename` finds its
  parent, by the ancestor rule, and a free name, by the name rule. A
  removal finds its entry, by the creation rule. A `remove_dir` finds its
  directory empty, by the emptying rule. One case relies on reading
  "a directory's removal" broadly (N1).
- **Reachability.** Prefixes of program order are what a journaling file
  system leaves. Non-prefix sets, such as one subtree of a `remove_dir_all`
  removed and a sibling kept, arise on a file system that writes directory
  blocks back independently. Every rule forbids a state that no file system
  reaches with atomic renames. These are the round-29 B1 state, an old file
  inside a re-created directory, and a creation inside a missing
  directory. Round 29's argument still holds.
- **D15** (`01:845–848`) says each unsynced directory-entry change survives
  whole or not at all, and that atomic replace works as on a journaling
  file system. Treating `remove_dir_all` as one change per path keeps every
  change whole, and the new rules only remove states. D15 is still
  accurate and needs no edit.
- **Companion 17** (`companion:440–455`) carries every rule: the list of
  changes, rename atomicity, `remove_dir_all` as one change per path, the
  three dependency rules, and the durability of `sync_dir` of the parent.
  It omits the general closure sentence, but gives its consequence for
  `remove_dir_all`. It omits the base rule, which belongs to 03 §1.1. It is
  a faithful summary.

**Leftovers.** `grep` over `design/`, `README.md`, and `GOVERNANCE.md` for
`remove_`, "entry change", "directory-entry", "new subdirectory", "never
neither", and "half done" finds no list the diff missed.
`git diff --check -- design/` passes. The only added line over 80 columns
is the §1.1 table row (`03:69`), a table convention.

## New blocking problems

None.

## Non-blocking notes

### N1. A `rename` over an empty directory is a directory removal, but the emptying rule names only `remove_dir`

The `rename` is "as std::fs" (`reference.md:384`). On Unix, Rust's `rename`
replaces an empty directory when the source is a directory (`fs.rs:2789`).
That replacement removes the target directory, so round 29's B1 recurs if
the emptying rule is read as covering only `remove_dir`:

```rust
fs.rename("d/x", "e/x")?;   // unsynced; d is now empty
fs.rename("c", "d")?;       // c is a directory; replaces the empty d
fs.sync_dir(".")?;          // the replacement is durable
```

If the first `rename` is dropped, `x` is "as it was" at `d/x`
(`03:1178–1180`). But `d` now names `c`'s directory, and the old `d` is
gone. So `x` is either nowhere or inside a directory it never entered, and
no file system reaches either state. The wording "a directory's removal"
(`03:1184`, `companion:452`) covers this case in plain English. But the
list of changes beside it (`03:1175`) names `remove_dir` as the
directory-removing change, so a simulator author may map the rule to
`remove_dir` alone. The case predates fx-ngm, since `rename` has always
been "as std::fs". The emptying rule is new, though, and is the place to
close it.

**Fix** (03 §9.5 and companion 17, no core tokens): "a directory's removal,
by `remove_dir` or by a `rename` over it, survives only if …".

### N2. The sync closure is stated only for the removal rule

`03:1186–1187` says a `sync_dir` that makes a directory's removal durable
makes the changes that emptied it durable too. The same consequence holds
for the other two rules, through their "only if" form. One example is
`create_dir_all("a/b")?; fs.write_synced("a/b/f", ..)?`: `write_synced`
syncs `f`'s entry, so the creations of `a` and `a/b` must survive although
nothing synced their parents. Another is a durable `create` of a reused
name, which carries the change that freed it. Stating the closure for only
one rule invites the reading that the other two do not carry their
dependencies. Under that reading, a synced change whose dependency is lost
can neither survive nor be lost. A general sentence would replace the
special one: "A change made durable makes durable every change it depends
on."

A related wording point predates this change. The creation rule speaks of
"its entry and every directory above it" (`03:1182–1183`), and a `rename`
has two paths. The dependency on the source side follows from the rule
applied to the source entry's own creation. It does not follow when that
entry is old and only a directory above it was renamed, unsynced, after
the entry was created. In an inode model the resulting state is reachable
and harmless. In a path-keyed simulator, it asks for a `rename` from a path
that does not exist. Either reading is defensible, and a phrase such as
"(for a `rename`, both its paths)" would choose one.

### N3. Other spellings of the base, `root.fs`, and the durability of an emptied base

- **`sub/..`.** `03:100–102` gives the outcome for `"."` only. The probe
  shows that Rust's `remove_dir_all("sub/..")` empties the base, then
  returns `Ok(())`, and that `remove_dir("sub/..")` returns
  `DirectoryNotEmpty`. `RESOLVE_BENEATH` allows `sub/..`, since it stays
  beneath the base (`03:84–85`). An agent that checks the `Ok` would
  believe the base is gone. "Never removes its own base" still holds, so
  this is a gap in the stated error, not a contradiction. Either "a path
  that resolves to the base fails with `InvalidInput`" or "as Rust does for
  each spelling" would settle it, and the simulator has to pick one.
- **`root.fs`.** On `root.fs` a relative path resolves against the working
  directory (`reference.md:407–408`), so `"."` is not the base there. The
  sentence is exact for an `Fs` from `dir()`, which round 29 asked about.
  Nothing removes `/`, so the rule holds.
- **Durability.** Round 29's N1 fix also proposed saying that
  `sync_dir(".")` makes the emptied contents durable. 03 does not say so.
  The answer follows from the rules: each removed entry is an entry of
  `"."`, and the closure carries everything below. An agent can derive it,
  so leaving it out is acceptable.

### N4. Nits

- **The TASK-3 citation.** `03:99–100` says "as a cancelled write can leave
  part of its data (TASK-3)". TASK-3 says only that "A cancelled write may
  already have reached the peer" (`03:784`), which means it may have taken
  effect, not that part of it did. Partial data on cancellation is true, as
  Rust's `write_all`, but TASK-3 does not state it. "(TASK-3)" could move
  to after "cancelled", or the clause could read "as a cancelled write may
  already have taken effect".
- **"Only … survives."** `03:1172–1174` and `companion:442` begin "Only
  data made durable … survives" (or "Only synced data survives"). Read
  literally, that forbids every unsynced change from surviving, which the
  next sentences allow. The intent, that synced data survives and other
  writes may be lost, is clear from context. The wording predates this
  change.

## Ready to merge

**Yes.** Every round-29 item is closed as its Resolution claims. The §9.5
rules are acyclic, every synced change can survive, and each state they
allow can be applied and is one a real file system reaches. The one
exception is a `rename` over an empty directory, and only under a narrow
reading of "a directory's removal" (N1). 01 D15 is still accurate, and
companion 17 summarizes 03 faithfully. The N1 text matches Rust 1.96 for
`"."`. The core is recorded at 24,997 tokens. N1 costs no core tokens and
is worth landing before merge. N2–N4 are optional.

Validation: this was a read-only review of the working tree at `f74dace`,
with `.beads/issues.jsonl` and three `design/` files modified and
`reviews/round29-consistency.md` untracked, on 2026-10-20.

- **Read:**
  - the full `design/` diff;
  - fx-ngm via `br show`;
  - round 29 with its Resolution;
  - in 03: §1.1 (table, Paths, Append and durability), TASK-3, and §9.5;
  - 01 D15;
  - reference 6.3;
  - companion 17 Crashes and the recorded count.
- **Rust:** read rust-src 1.96.0 `sys/fs/unix.rs:2614–2694`
  (`remove_dir_all_recursive`, `remove_dir_all_modern`) and the Unix
  directory rule in `fs.rs`'s `rename` doc. Compiled and ran one probe
  (`rustc --edition 2024`, Linux) in the session scratchpad, outside the
  repository. Its outputs are tabulated above.
- **Checks:** `git diff --check -- design/` passed. Added lines were checked
  for width. The token counter was not run, as instructed.

No Tide checker or simulator exists, so every verdict on Tide behavior is an
inference from the text. Only this file was written; nothing was staged or
committed.

## Resolution

Answered in the working tree before round 31.

- **N1:** 03 §9.5 and companion 17: "a directory's removal, by
  `remove_dir` or by a `rename` over it".
- **N2:** the creation rule now reads "its entries and every directory
  above its paths", which covers both paths of a `rename`. The general
  sync closure is declined: durability flows from a synced removal to the
  changes that emptied the directory, because a file system cannot remove
  a non-empty directory. It does not flow from a synced entry to the
  unsynced creation of a directory above it, which POSIX does not
  guarantee and which is why reference 6.3's acknowledged write also syncs
  the parent. 03 §9.5 now says so, and its opening reads "Data made
  durable ... survives, provided the entries of the directories above it
  are durable too". The companion matches.
- **N3:** 03 §1.1: `remove_dir_all` on any path that names the base
  empties it and leaves it in place. Rust's `InvalidInput` is noted for
  `"."` only, and `sync_dir(".")` makes the emptying durable.
- **N4:** the TASK-3 citation now says cancellation can stop the removal
  between paths. The "Only data made durable" wording is replaced as in N2.
