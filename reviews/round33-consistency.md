# Tide consistency review, round 33: confirmation of the round-32 answers

## Verdict

**approve with notes**

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `d8bf49a`
("Decide Fs::remove_dir and remove_dir_all and their crash rules (fx-ngm)"):
`git diff -- design/`, which changes `02-memory.md` (§9's not-`Pure` list)
and `03-authority-concurrency.md` (§0's definition of a capability). It
implements bead fx-zm4. This is a narrow confirmation round. Scope, as
asked:

- whether round 32's B1 and the answered part of N1 are closed as its
  Resolution claims;
- whether the new general clause, "any value their methods return that
  keeps their authority", is well defined and consistent with 01 D7, 03
  §3.1, 03 RUST-2, and reference 6.6, and in particular whether it could
  make an ordinary returned value, such as a `Vec<u8>` or a `String`, not
  `Pure`;
- whether deferring N2–N4 suits the bead's stated scope.

Nothing else was re-audited.

## Round-32 item status

| Item | Status | Evidence |
| --- | --- | --- |
| B1 | **Resolved**, with a scope point (N2) | 02's bullet ends "and the opaque types an imported Rust crate with declared authority returns (03 §0, RUST-2)" (`02:820–821`). 03 §0 ends "and an imported crate's opaque types under RUST-2" (`03:37`). RUST-2 (`03:1345–1350`) and 03 §3.1's general phrase (`03:430`) now agree with both lists. No type's status changes. |
| N1, pipes and `lines()` | **Resolved** by the general clause, with wording points (N1, N3) | `02:818–820` adds "any value their methods return that keeps that authority (a `Child`'s pipes, the iterator of `Stdin::lines()`)", and `03:36–37` adds the same clause with the pipes as its example. An owned `lines()` iterator or a pipe end is now not `Pure` whether or not the structural rule reaches a capability through its fields. |
| N1, ROOT-4 and Rust's public `Child` fields | **Declined**, as the Resolution states | `03:232–233` is unchanged. The new example now depends on how this point is settled (N3). |
| N1, `Stdio` | **Not mentioned** by the Resolution | See N3. |
| N2 | **Deferred**, reasonable | The bead asks only that reference 6.6 agree (Scope). 6.6's "`File`, `TcpStream`, and other handles derived from them" (`reference.md:470–471`) agrees with both new lists. N2 concerns 6.1, 6.3, and 12.3. |
| N3 | **Deferred**, reasonable; needs a durable home | Settling `OnceLock` may change which types are `Pure`, which the bead puts out of scope. See N4. |
| N4 | **Deferred**, reasonable | The `std::sim` handles and `LogConfig` are confined to test code (`03:226–231`), and none of them is a resource handle in 03 §1.1. |

**The general clause and ordinary values.** On a careful reading, the
clause cannot make a `Vec<u8>` or a `String` non-`Pure`:

- **01 D7.** A type is `Pure` if no capability, synchronization handle, or
  foreign handle is reachable from it. Classification is structural over
  every reachable type (`01:455–457`, `01:464–465`).
- **03 §0.** The changed definition makes an authority-keeping returned
  value a capability (`03:33–37`). D7 therefore already covers it, and the
  02 bullet adds no new criterion.
- **No authority in plain data.** A `Vec<u8>` or a `String` carries no
  authority: no method of either reads or changes the world.
  `02:832–835` ("Everything else is `Pure` exactly when its components
  are", std collections included) settles both types.
- **Values returned by handles.** 03 already names such values as plain
  data: `DirEntry::file_name() -> OsString` (`03:116`), and "`OsStr` and
  `OsString` keep Rust's API, which does no I/O" (`03:122–123`). The
  buffer from `File::read_owned` (`03:72`) is the same case.
- **RUST-2's test.** RUST-2 gives the operational meaning of carrying
  authority: the methods "need no further parameters" to exercise it
  (`03:1348–1349`).
- **3.1 and 6.6.** 03 §3.1's "resource handles derived from them"
  (`03:430`) and 6.6's "other handles derived from them"
  (`reference.md:470–471`) are general phrases. The new clause is a
  subset of each.

The remaining risk is in the wording. The clause is phrased over values,
and "keeps that authority" is not defined where it is used (N1).

## New blocking problems

None.

## Non-blocking notes

### N1. The general clause speaks of values in a table of types

- **Values and types.** `Pure` is a property of a type (`01:455`,
  `02:832`), but the clause classifies "any value their methods return
  that keeps that authority" (`02:818–819`, `03:36`).
  - A reader who takes "keeps" as provenance could treat bytes read
    through a `File` as keeping the `Fs`'s authority. Nothing in `design/`
    defines a taint notion of that kind; a grep for "taint" and
    "provenance" finds nothing. D7's "structural over every reachable type"
    (`01:464–465`) rules that reading out. The clause itself does not.
  - The value phrasing also leaves open what happens to a type whose
    values carry authority only sometimes. The type must then be not
    `Pure` as a whole. `Stdio` is that case if it can hold a `File` (N3).
- **`Stdin::lines()` lies outside the clause's own antecedent.** In 02,
  "their methods" refers to the listed resource handles. `Stdin` is a
  capability type, listed in the previous bullet (`02:814`). The bullet's
  lead, "the resource handles derived from a capability" (`02:816`), still
  covers the iterator, so the example is misplaced rather than wrong. 03
  §0 avoids the problem by giving only the pipes as its example (`03:36–37`).

**Fix** (02 and 03 §0, no core tokens). Phrase the clause over types and
borrow RUST-2's test. For example: "any type returned by a capability's or
a handle's method whose values can act on the world with no capability
argument (a `Child`'s pipes, the iterator of `Stdin::lines()`)". The
second half of that wording also covers the antecedent.

### N2. 02 states RUST-2's handles more broadly than RUST-2 does

- **Which functions.** RUST-2 makes resource handles of "the opaque types
  they return", where "they" means free-function and constructor wrappers
  (`03:1346–1349`). 02 says "the opaque types an imported Rust crate with
  declared authority returns" (`02:820–821`), which includes method
  returns. Take a `rusqlite` statement or row type that a `Connection`
  method returns:
  - 02 makes it not `Pure`;
  - RUST-2 says nothing about it;
  - 03 §0's general clause does not reach it, because its "their methods"
    refers to the std handles (`03:34–37`).
- **Undefined "opaque".** RUST-2 does not define "opaque", and 02 uses the
  word for `impl Trait` and `dyn` types (`02:799`). If every imported type
  counts as opaque, 02's literal reading makes a crate's plain data or
  error type that a method returns not `Pure`. That would be an ordinary
  returned value made non-`Pure`. RUST-2 already has this ambiguity for
  constructor returns, such as the error type of `Connection::open`. 02
  widens it to every function. This is an inference from the wording; no
  text decides it.
- **Version label.** Round 32's suggested fix said "from v1". The applied
  text omits it. Rust crate imports are v1 (`03:1332`, `01:83`), and the
  citation of RUST-2 implies the version without stating it.

**Fix** (no core tokens). Put the general clause after the RUST-2 item in
both lists, so it covers the methods of both kinds of handle. In 02, change
the RUST-2 item to "from v1, the opaque types that RUST-2 makes resource
handles". Defining "opaque" belongs in RUST-2, not in this bead.

### N3. The pipe example presumes an API that 03 does not state

- **Pipes as method returns.** Rust's `Child` exposes its pipe ends as
  public fields (`child.stdout`), not as values that its methods return.
  The example "a `Child`'s pipes" (`02:819`, `03:36–37`) fits the clause
  only if Tide's `Child` returns them from methods. ROOT-4's "no public
  fields" (`03:232–233`) requires the same, but 03 §1.1 lists no such
  method (`03:80`). The example is now the only text in `design/` saying
  that a `Child` has pipes at all. That is consistent with ROOT-4, but it
  implies an API decision the Resolution says it has not made.
- **`Stdio`.** Round 32's N1 also raised `Stdio`, which can hold a `File`
  or a pipe end if Tide keeps Rust's `From` impls. The Resolution neither
  answers nor declines it. A `Stdio` built by `Stdio::from(file)` comes
  from `Stdio`'s constructor, not from a handle's method, so the clause's
  literal wording misses it. Only the bullet's lead (`02:816`) covers it.

**Fix**: none in this bead. List ROOT-4, the pipe accessors, and `Stdio`
with N2–N4 in whatever the owner gets, so that settling ROOT-4 also fixes
the example.

### N4. N3's deferral needs a durable home

Deferring round 32's N3 is right. The bead excludes "changing which types
are `Pure`", and `OnceLock` needs an owner decision. But N3 is a
disagreement on `Pure` between 02, the owner, and reference 6.6:

- 6.6 lists `OnceLock` and "any `Sync` type whose `&self` API can observe
  or cause mutation" (`reference.md:472–473`);
- 02 names neither (`02:822–824`).

That is the class of contradiction named by the bead's second acceptance
bullet. The `.beads/issues.jsonl` diff changes only fx-zm4's own status,
so "reported to the owner" currently exists only in round 32's Resolution
(`round32:229–231`). AP-PLAN-001 allows that. If no bead records N3, the
next review of `Pure` will find it again.

## Ready to merge

**Yes.** B1 is closed: 02 and 03 §0 both include RUST-2's handles, and no
type's status changes. The answered part of N1 is closed by a general
clause that, read with 01 D7 and 02's structural rule, leaves `Vec<u8>`,
`String`, `OsString`, and other plain returned data `Pure`. The clause is
consistent with 03 §3.1, RUST-2, and reference 6.6. Deferring N2–N4 suits
the bead's scope. N1 and N2 are wording fixes in 02 and 03 that cost no
core tokens and can land before merge. N3 waits on ROOT-4. N4 asks the
owner whether to file a bead for `OnceLock`.

Validation: this was a read-only review of the working tree at `d8bf49a`,
with `.beads/issues.jsonl`, `design/02-memory.md`, and
`design/03-authority-concurrency.md` modified and
`reviews/round32-consistency.md` untracked, on 2026-10-20.

- **Read:**
  - the full `design/` diff and the `.beads/issues.jsonl` diff;
  - fx-zm4 (`br show`);
  - `reviews/round32-consistency.md`, including its Resolution;
  - 02 §9's `Pure` sections;
  - in 03: §0, §1.1 with its notes, ROOT-2 to ROOT-4, §3.1, and §11
    RUST-1 to RUST-5;
  - 01 D7's authority-free bullets;
  - reference 6.6.
- **Greps** over `design/` for "opaque", "taint", "provenance", `Stdio`,
  `ChildStd`, "pipe", `lines()`, and phrases about carrying or keeping
  authority.
- **Line lengths:** the added lines are at most 73 columns.

No Tide checker exists, so every verdict on Tide behavior is an inference
from the text. Only this file was written; nothing was staged or committed.

## Resolution

Applied before commit.

- **N1:** both lists now define a resource handle as a type, by RUST-2's
  test: a type whose values act on the world with no capability argument
  because they carry the authority of the capability that made them. The
  general clause covers "any other such type a capability or handle method
  returns", so `Stdin::lines()`'s iterator falls inside it.
- **N2:** both lists put the RUST-2 item last, scoped to "the opaque types
  that RUST-2 makes resource handles" and labeled "from v1".
- **N3:** the `Child`'s-pipes example is dropped. It waits on ROOT-4's
  public-field question, as does `Stdio`.
- **N4:** reported to the owner along with round 32's N2–N4.

The verdict is approve with notes, so no further round is run.
