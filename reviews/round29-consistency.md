# Tide consistency review, round 29: `Fs::remove_dir` and `remove_dir_all` (fx-ngm)

## Verdict

**request changes**

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `f74dace`
("Decide std and simulator details from the E1 dialect round (fx-i5p)"):
`git diff -- design/`, which changes `03-authority-concurrency.md` (the
§1.1 `Fs` row, the §1.1 append and durability note, §9.5),
`reference.md` (6.3's `Fs` comment and Durability bullet), and
`reference-companion.md` (17 Crashes, the recorded count). It implements
bead fx-ngm, which asks 03 to decide whether `Fs` has `remove_dir_all` and
`remove_dir`, and if so to give their durability and crash behavior on top
of fx-i5p's model (rounds 25–28): each unsynced directory-entry change
survives whole or is lost. Scope, as asked:

- whether the bead's acceptance is met: 03 and reference 6.3 agree on the
  methods, and 03 states their crash behavior;
- whether the crash and durability rules are coherent: every allowed state
  reachable by a real file system, every synced change surviving under the
  dependency rules, and "the whole removal is durable once `sync_dir` of
  `p`'s parent returns" justified against §9.5 and 01 D15;
- whether the change contradicts 01 D7 or D15, the rest of 03 (§1.1 Paths
  and attenuation), the reference pair, or Rust's `std::fs::remove_dir` and
  `remove_dir_all`;
- whether it loosens D15;
- the core's size, from the companion's recorded figure (not re-measured).

## Acceptance and checks

| Question | Finding |
| --- | --- |
| Methods agree | **Yes.** 03's `Fs` row lists `remove_dir` and `remove_dir_all` (`03:69`); reference 6.3 says `remove_dir(_all): as std::fs` (`reference.md:384`). See N5 on the spelling. |
| 03 states crash behavior | **Yes.** §1.1 (`03:93–97`) and §9.5 (`03:1169–1182`): one whole change per removed path, a dependency rule for directory removals, and `sync_dir` of `p`'s parent making the whole removal durable. Companion 17 repeats it (`companion:443–452`). |
| Every allowed state reachable | **No**, for a `rename` out of a directory that is then removed (B1). Partial trees are reachable: the allowed sets (per-path changes, closed under "a directory goes only after its contents") include every crash prefix a journaling file system leaves, and the non-prefix ones arise on a file system that writes directory blocks back independently, which fx-i5p's "otherwise unspecified" already admits. |
| Synced changes survive | **Yes for removals.** A synced `remove_dir` survives, and the closure sentence (`03:1179–1180`) forces the removals inside it, so the dependency rule never makes a synced removal lose. It does not cover the `rename` of B1. |
| "Whole removal durable once `sync_dir` of `p`'s parent returns" | **Justified**, but not by POSIX alone. On ext4 and XFS, `fsync` of a directory commits the journal up to that point, earlier unlinks included. Under bare POSIX it guarantees only the parent's entries, but once `p`'s entry is gone nothing below `p` is reachable: `Fs` has no hard links (`03:69`), and a crash closes every `File`. So the promise holds observably on any file system. It agrees with §9.5's "Only data made durable by `sync` or `sync_dir` … survives" (`03:1167–1168`) and D15 (`01:845–848`). Only an `Fs`'s own base is a hole: its parent is outside the `Fs` (N1). |
| Loosens D15 | **No.** D15 says each unsynced entry change "survives whole or not at all" (`01:846–848`). Treating `remove_dir_all` as one change per path keeps every change whole. It adds no torn state, and Rust's `remove_dir_all` is not atomic even without a crash (it may return `DirectoryNotEmpty` after removing part of the tree, `fs.rs` doc). The new dependency sentence only removes states, as round 25's N3 rule did. The closure sentence promises nothing observable beyond what `sync_dir` of the parent already makes durable. No owner question arises. |
| Budget | `companion:12` records 24,992 tokens, 8 under D14's 25,000. The diff removes about three core tokens ("`remove_file`" becomes "removals"). It was not re-measured, as instructed. |

**Rust facts checked** against rust-src of rustc 1.96.0
(`library/std/src/fs.rs:3130–3234`, `sys/fs/unix.rs:2614–2693`):

- `remove_dir` "removes an empty directory" (`rmdir`), and fails on a
  missing path, a non-directory, or a non-empty directory.
- `remove_dir_all`'s errors are defined as those of `remove_file` and
  `remove_dir` "on *any* constituent paths, *including* the root `path`".
  The Unix implementation is a post-order walk: `unlinkat(fd, name, 0)` for
  each non-directory, then `unlinkat(.., AT_REMOVEDIR)` for each directory.
  It never follows a symlink, and it `unlink`s a symlink given as `p`. So
  §1.1's "`remove_file` and `remove_dir` on every path below and including
  `p`" (`03:95–96`) is accurate. "As Rust specifies it" holds by the error
  contract and the implementation.
- It tolerates `NotFound` on constituents removed concurrently, and returns
  `NotFound` only if it removed nothing.
- Never following symlinks fits §1.1's `RESOLVE_BENEATH` rule
  (`03:84–85`). A symlink inside the tree that points outside the `Fs` is
  unlinked, not traversed, so the walk cannot leave the base.
- Probed in the scratchpad (`rustc --edition 2024`, Linux): with the
  working directory as the target, `remove_dir_all(".")` deleted every
  entry inside it, then returned `InvalidInput` (`EINVAL`, 22).
  `remove_dir(".")` returned `InvalidInput`, and `remove_dir_all` on a
  regular file returned `NotADirectory`.

**Leftovers.** `grep` over `design/`, `README.md`, and `GOVERNANCE.md` for
"entry change", "directory-entry", "new subdirectory", "symlink", and
`remove_` finds no list of entry changes or `Fs` methods that the diff
missed. 01 D15 (`01:847`) speaks of directory-entry changes generically and
needs no edit. `git diff --check -- design/` passes. The only added lines
over 80 columns are the §1.1 table row (`03:69`, a table convention) and
the 6.3 code comment (`reference.md:384`, the same width as before).

## New blocking problems

### B1. A `rename` out of a removed directory can be neither done nor undone

The new rule (`03:1177–1180`, `companion:450–452`) says "a directory's
removal survives only if the removals of everything in it do". A
`remove_dir` needs the directory to be empty when it runs, and a `rename`
can also empty it, by moving an entry out. The rule does not count a
`rename` as a removal:

```rust
fs.rename("d/x", "e/x")?;   // unsynced
fs.remove_dir("d")?;        // d is now empty
fs.sync_dir(".")?;          // d's removal is durable
```

After a crash, `d`'s removal must survive. Nothing forces the `rename` to
survive, so the seed may drop it. §9.5 then says the `rename` "did not"
happen, so `x` is back at `d/x` with the source "as it was"
(`03:1173–1176`). But `d` is gone, so `x` is in neither place, which is the
"never neither" state §9.5 rules out. No real file system produces this
while keeping the rename atomicity Tide promises. On a journaling file
system, the `rename` precedes the `rmdir` in the log, and the `fsync`
commits both. On one without a journal, `x` is orphaned for fsck to put in
`lost+found`. The same hole exists without any sync, if the unsynced
`remove_dir` survives and the `rename` does not. `remove_dir_all` meets it
when a task renames entries out of the tree first.

This contradicts §9.5's own rename rule and D15's "each unsynced
directory-entry change, a `rename` included, survives whole" (`01:846–848`),
so the bead's second acceptance item fails.

**Fix** (03 §9.5 and companion 17, no core tokens): "a directory's removal
survives only if every change that emptied it does, the removal of each
entry in it and each `rename` out of it; a `sync_dir` that makes a
directory's removal durable makes those changes durable too."

## Non-blocking notes

### N1. Removing an `Fs`'s own base is unspecified, and Rust's behavior there is surprising

§1.1 Paths (`03:84–85`) and reference 6.3 (`reference.md:408–412`) say how
a path that leaves the base fails. They do not say what
`fs.remove_dir_all(".")` or `fs.remove_dir(".")` does on an `Fs` from
`dir()`, or `root.fs.remove_dir_all("/")`.

- **What Rust does.** The probe above shows that Rust's `remove_dir_all(".")`
  removes every entry under the base, then fails with `InvalidInput`.
  `rmdir` of `.` is `EINVAL`, and Linux has no "remove this directory by
  descriptor" call.
- **The authority question.** Removing the base itself would change an
  entry in the base's parent, outside the subtree the `Fs` grants. So the
  "cannot leave it" rule should forbid it.
- **The durability gap.** §1.1's "durable once `sync_dir` of `p`'s parent
  returns" (`03:97`) is unreachable when `p` is the base, since its parent
  is outside the `Fs`. An agent that writes `dir.remove_dir_all(".")` to
  clear a scratch directory gets an error after the deletion, and has no
  way to follow the stated durability recipe for it.

**Fix** (03 §1.1, a clause; the core can stay as is): "A path that resolves
to the `Fs`'s base cannot be removed: `remove_dir` and `remove_dir_all`
fail with `InvalidInput`, as `rmdir(".")` does. `remove_dir_all`, like
Rust's, has first removed the base's contents, which `sync_dir(".")` makes
durable." The other choice is to fail before removing anything, which
departs from Rust and needs saying. Either way, the simulator has to know.

### N2. Changes to the same name are not ordered, so a re-created directory could show old files

A common pattern is "clear and rebuild":
`fs.remove_dir_all("out")?; fs.create_dir_all("out")?;
fs.write_synced("out/f", ..)?; fs.sync_dir(".")?`.

- **What the rules force.** The new `out`'s creation is durable. Its
  dependencies are "the changes that created its entry and every directory
  above it" (`03:1177–1178`). That does not include the removal of the old
  `out` that freed the name. So the old `out`'s removal, and the removals of
  its contents, may be dropped.
- **What a file system does.** In an inode model this is harmless: the name
  names the new directory, and the old contents are unreachable.
- **The risk.** A simulator that keys entries by path, and drops changes
  independently, would bring old `out/g` back inside the new `out`. No file
  system reaches that state.

**Fix** (03 §9.5, optional): "and a change that takes a name survives only
if the change that freed it does". This could go in the same sentence as
B1's fix.

### N3. §1.1's "so" needs §9.5's closure sentence

`03:95–97` derives "the whole removal is durable once `sync_dir` of `p`'s
parent returns" from "`remove_file` and `remove_dir` on every path". That
does not follow alone: `sync_dir` of the parent covers only `p`'s own
entry, and the rest comes from §9.5's closure sentence (`03:1179–1180`).
Adding "(§9.5)" after the sentence makes the derivation checkable. The
reason it is safe is the unreachability argument in the table above, and
§9.5 could say so in a few words: "nothing below a removed directory is
reachable".

### N4. A cancelled or failed `remove_dir_all` can leave part of the tree without any crash

`Fs`'s operations all suspend and are checkpoints (`03:69`). TASK-3 cancels
an in-flight operation before the error returns, and says "A cancelled
write may already have reached the peer" (`03:778–779`). A
`remove_dir_all` cancelled midway, or failing on one constituent (Rust's
`DirectoryNotEmpty` under concurrent writes, or `PermissionDenied`), has
removed part of the tree. 03 says this only for crashes (`03:1171–1172`).
"As std::fs" implies the failure case, but cancellation is Tide's own.
**Fix** (03 §1.1): "a cancelled or failed `remove_dir_all` may have removed
part of the tree."

### N5. The core no longer contains the string `remove_dir_all`

`reference.md:384` writes `remove_dir(_all)`. An agent that greps the core
for `remove_dir_all`, as the bead's own evidence did, now finds nothing. The
regex-like shorthand appears nowhere else in the reference. Writing
`remove_dir, remove_dir_all` costs about three tokens of the eight left.
Optionally, the core could also say that `remove_dir_all` is not atomic
across a crash. 6.3 says "a `rename` is never half done (companion 17)"
(`reference.md:419`), and an agent may assume the same of a recursive
removal. That would cost more than the remaining budget allows, so leaving
it to companion 17 is acceptable.

## Ready to merge

**No.** B1 is a contradiction between the new directory-removal rule and
§9.5's rename atomicity, and the bead's acceptance requires none. The fix is
one clause in 03 §9.5 and companion 17, with no core tokens. N1–N5 can land
with it. N1 is the one most likely to bite an agent. Otherwise:

- the methods match in 03 and reference 6.3;
- Rust's `remove_dir` and `remove_dir_all` are described accurately;
- the per-path crash model and the durability promise for `sync_dir` of the
  parent are sound;
- nothing loosens D15 or contradicts 01 D7.

Validation: this was a read-only review of the working tree at `f74dace`,
with `.beads/issues.jsonl` and three `design/` files modified, on
2026-10-20.

- **Read:**
  - the full `design/` diff;
  - fx-ngm via `br show`;
  - in 03: §0 (terms), §1.1, TASK-3, and §9.5;
  - 01 D7 and D15;
  - reference 6.3;
  - companion 17 Crashes and the recorded count;
  - `reviews/round25-consistency.md` and `round28-consistency.md`.
- **Rust:** read rust-src 1.96.0 `std/src/fs.rs` (the `remove_dir` and
  `remove_dir_all` docs) and `sys/fs/unix.rs` (`remove_dir_all_recursive`,
  `remove_dir_all_modern`). Compiled and ran one probe in the scratchpad
  with `rustc --edition 2024`: `remove_dir_all(".")`, `remove_dir(".")`,
  and `remove_dir_all` on a file.
- **Checks:** `git diff --check -- design/` passed. The token counter was
  not run.

No Tide checker or simulator exists, so every verdict on Tide behavior is an
inference from the text. Only this file was written; nothing was staged or
committed.

## Resolution

Answered in the working tree before round 30.

- **B1:** 03 §9.5 and companion 17: a directory's removal survives only
  if every change that emptied it does, each removal in it and each
  `rename` out of it. A `sync_dir` that makes the removal durable makes
  those changes durable too.
- **N1:** 03 §1.1: an `Fs` never removes its own base. As in Rust,
  `remove_dir_all(".")` empties it, then fails with `InvalidInput`, and
  `remove_dir(".")` fails at once (Linux `rmdir` rejects a final `.`
  with `EINVAL`).
- **N2:** 03 §9.5 and companion 17: a change that takes a name survives
  only if the change that freed the name does.
- **N3:** 03 §1.1 cites §9.5 and gives the reason: nothing below a removed
  directory is reachable.
- **N4:** 03 §1.1: a cancelled or failed `remove_dir_all` can leave part of
  the tree, as a cancelled write can (TASK-3).
- **N5:** the core spells out `remove_dir, remove_dir_all`. It measures
  24,997 tokens.
