# Tide consistency review, round 32: resource handles in the `Pure` table (fx-zm4)

## Verdict

**request changes**

Reviewed the uncommitted working tree on 2026-10-20 against HEAD `d8bf49a`
("Decide Fs::remove_dir and remove_dir_all and their crash rules (fx-ngm)"):
`git diff -- design/`, which changes `02-memory.md` (§9's not-`Pure` list)
and `03-authority-concurrency.md` (§0's definition of a capability). It
implements bead fx-zm4. Scope, as asked:

- whether the bead's acceptance is met, that is, whether 02's list and 03
  §0/§1.1 name the same resource handles;
- whether any type that 03 §1.1, §5, or §11, or the reference, derives
  from a capability is still missing from either list, and whether any
  listed type carries no authority, `Command` in particular;
- whether anything contradicts 01 D7's definition of `Pure`, 03 §3.1,
  reference 6.6, or the companion's rows on `Pure`.

Rule preservation outside these passages was not re-audited.

## Acceptance status

| Check | Status | Evidence |
| --- | --- | --- |
| 02's list matches 03 §0 | **Met** | `02:816–818` and `03:33–35` both name `File`, `ReadDir`, `DirEntry`, `TcpStream`, `TcpListener`, `UdpSocket`, `Command`, `Child`, and `Signals`. |
| 03 §0 matches §1.1 | **Met** | §1.1 derives `File` (`03:70`), `ReadDir` and `DirEntry` (`03:72`, `03:112–118`), the three sockets (`03:71`, `03:73`), and `Command`, `Child`, and `Signals` (`03:78`). No other type in §1.1 is returned by a capability and acts on the world: `Instant`, `SystemTime`, `Pcg64` (Pure by `03:142–144`), `Metadata`, `OsString`, and `Vec<u8>` are values. |
| Every listed type carries authority | **Met** | Each has a method that acts on the world and takes no further capability: `Command`'s `output`, `status`, and `spawn`, `Child::wait`, `Signals::recv` (`03:78`), and iteration, `metadata`, and `file_type` on a `ReadDir` or `DirEntry` (`03:116–118`). |
| No contradiction among 02, 03, and the reference pair | **Not met** | 03 RUST-2 declares a class of resource handles that 02's list and 03 §0 leave out (B1). |

**`Command` versus `OpenOptions`.** Treating `Command` as a resource
handle is right, and it does not conflict with `OpenOptions` being a plain
builder. `OpenOptions` loses its ambient `open` (`03:90–91`,
`reference.md:402–403`), so authority enters at `fs.open_with(p, &opts)`.
The builder is inert data, and `Pure`. `Command` keeps Rust's terminal
methods, so `cmd.output()` runs a process with no capability argument. The
authority must travel in the value, as it does for `File`, and
`Proc::only([..])` is checked when `proc.command(prog)` builds it
(`03:78`). Making `Command` `Pure` would need a `proc.run(&cmd)` shape,
which would change which types are `Pure` and is outside the bead. The
only cost is wording in the reference (N2).

**§5 adds nothing to the list.** FFI-2 wrappers take a capability or are
`#[trusted_pure]` (`03:639–643`). Foreign handles are already a bullet of
their own (`02:822–823`), and `file.as_raw_fd()` (`03:659–660`) yields a
`RawFd`, an integer that can act only through `ffi` code.

**D7, §3.1, 6.6, and the companion.**

- **01 D7.** D7 calls a type `Pure` when no capability, synchronization
  handle, or foreign handle is reachable from it (`01:455–457`). 03 §0
  counts resource handles as capabilities, so 02's new bullet spells out
  what D7 already implies.
- **03 §3.1 and reference 6.6.** Both use the general phrases "the
  resource handles derived from them" (`03:427–428`) and "`File`,
  `TcpStream`, and other handles derived from them" (`reference.md:470–471`).
  02's enumeration is a subset of each and contradicts neither, apart from
  B1.
- **Companion.** Its rows agree: capturing a capability in `join` or
  `par_map` (`companion:53`), `par_for` over files (`companion:54`), and
  node-entry captures (`companion:435–439`).

## New blocking problems

### B1. 02 and 03 §0 omit the resource handles that RUST-2 declares

03 RUST-2 (`03:1343–1348`) says that when a Rust crate is imported with
declared authority, "the opaque types they return are resource handles: not
`Pure`, carrying that authority, so their methods need no further
parameters". `rusqlite::Connection` under the manifest at `03:1335` is an
example. 03 §3.1's general phrase covers these types (`03:427–428`).
02's new bullet does not. It names nine std types and points to 03 §0
(`02:816–818`), whose list is also closed to std types (`03:33–35`).

02 owns the full table (`03:24–25`), so these types fall under its "Everything
else is `Pure` exactly when its components are" (`02:829`). A Rust type
that reaches the world through a Rust `std::fs::File`, a file descriptor,
or a `PathBuf` it opens later has components the structural rule finds
`Pure`. 02 therefore admits the type to `join`, `Shared`, and contract
clauses, and 03 RUST-2 forbids the same uses. This contradiction predates
the change, since 02 named no resource handle before. But fx-zm4 exists to
make 02 record the handles 03 declares, and its second acceptance bullet
asks for no contradiction among 02, 03, and the reference pair on `Pure`.

**Fix** (02 and 03 only, no core tokens; it changes no type's status):

- in 02's bullet, after `Signals`, add "and, from v1, the opaque types
  that an imported Rust crate with declared authority returns (03
  RUST-2)";
- in 03 §0, after `Signals`, add "or RUST-2's opaque types".

## Non-blocking notes

### N1. `Child`'s pipe ends and the `lines()` iterator are unnamed handles

- **Pipes.** The reference calls `Command` "Rust's builder"
  (`reference.md:335`). That builder has `stdout(Stdio::piped())`, and
  Rust's `Child` then holds public fields `stdin: Option<ChildStdin>`,
  `stdout`, and `stderr`. If Tide keeps them, the pipe ends are resource
  handles. They read and write a process the `Proc` started, and their
  reads suspend. They appear in neither list, nor anywhere in `design/`.
  - **ROOT-4.** It says capability and handle types "have no public
    fields" (`03:230–231`). Rust's `Child` has them, and 03 §0 counts
    `Child` as a capability. This predates the change.
  - **`Stdio`.** If Tide keeps Rust's `From<File>` and `From<ChildStdout>`
    for `Stdio`, it can hold a handle too.
- **`lines()`.** `Stdin::lines()` takes `&self` and suspends (`03:122–130`),
  but its return type is never named. If the iterator borrows the `Stdin`,
  the structural rule already makes it not `Pure`. If it is owned, like
  Rust's `Lines<StdinLock<'static>>`, it is an unlisted handle.

**Fix** (03 §1.1, then 02 if needed): either state that Tide's `Child`
has no piped stdio in v0, or add `ChildStdin`, `ChildStdout`, and
`ChildStderr` to both lists and exempt them in ROOT-4. Also state that
`lines()` borrows its `Stdin`.

### N2. The reference pair does not say that a `Command` carries authority

`reference.md:335` describes `command(prog) -> Command` only as "Rust's
builder". 6.3 uses the same words for `OpenOptions`, which is `Pure`
(`reference.md:402`), but says outright that a `File` and a `DirEntry`
carry the `Fs`'s authority (`reference.md:384–385`, `:405–406`). An agent
that reads 6.1 by analogy with 6.3 will expect `fn configure(cmd: &mut
Command)` to be authority-free. 6.6's "other handles derived from them"
covers the case only if the reader already knows that `Command` is a
handle. Related gaps:

- the reference pair never names `Child`, `Signals`, `ReadDir`, or
  `proc.signals`;
- 12.3's paths table lists only `Proc`, `Command`, and `ExitCode` for
  `std::process` (`reference.md:1004`), and no `ReadDir` or `DirEntry`
  under `std::fs` (`:998`);
- 6.1's "Every capability method takes `&self`" (`reference.md:338`) is
  true of the table's capability types. It is not true of the handles
  that 03 §0 now also calls capabilities: Rust's `Command::arg`,
  `output`, and `spawn` and `Child::wait` take `&mut self`, and
  `File::close` takes `self` (`reference.md:398`).

**Fix** (companion, no core tokens): one line saying that a `Command`,
like a `File`, carries its capability's authority and is not `Pure`,
unlike `OpenOptions`, and that `&self` receivers are a property of
capability types, not of handles.

### N3. Reference 6.6 names synchronization handles that 02 does not

- **`OnceLock`.** Reference 6.6 lists it as a synchronization handle
  (`reference.md:472`). 02 does not mention it, and 03 mentions it only as
  a banned static initializer (`03:275`, `companion:35`). No text says
  whether a local `OnceLock` is admitted at all, or whether creating one
  takes `&Tasks`. If it is admitted and `Pure`, a `par_map` closure that
  captures `&OnceLock<T>` and calls `get_or_init` returns whichever
  branch's value wins the race. That breaks 02's claim that nothing
  reachable from a `Pure + Sync` value changes during a read
  (`02:855–857`). The structural rule may still catch it through
  `OnceLock`'s private atomic, but whether std's internal atomics count as
  "every atomic type" is not stated. That part is inference.
- **The general clause.** "any `Sync` type whose `&self` API can observe
  or cause mutation" (`reference.md:472–473`) is a rule that 02, the
  owner, does not state.
- **`std::sync::raw`.** 03 §3.1 lists "everything in `std::sync::raw`"
  (`03:430`), meaning `Mutex`, `RwLock`, and weaker atomic orderings
  (`03:167`). 02's list names atomics but not the locks.

**Fix:** have the owner decide `OnceLock` (banned outright, or a
synchronization handle made from `&Tasks`). Then state 6.6's general
clause and `std::sync::raw` in 02's synchronization-handle bullet.

### N4. Test-only and configuration handles have no stated status

`SimRoot` and `NodeRoot` hold capability fields, so the structural rule
makes them not `Pure`. The `Pure` status of these is unstated:

- `sim.world` and `sim.faults` (`03:1102–1106`);
- a `Node`, and the handle that `node.start` returns, which
  `sim.run_until` waits on (`03:1165–1166`, `03:1200–1201`);
- `Root`'s `log: LogConfig` (`03:174`), which can hold a `File` after
  `to_file` (`03:325–326`).

Code that receives `&Faults` can crash or partition nodes. ROOT-3
confines these types to test code (`03:224–229`), so the risk is small.

**Fix**, optional: one clause in 02 or 03 §9.2 saying that `std::sim`
handles and `LogConfig` are not `Pure`.

## Ready to merge

**No.** B1 leaves the contradiction the bead's second acceptance bullet
forbids: 03 RUST-2 says a class of resource handles is not `Pure`, and the
owner's table, now an explicit enumeration, leaves it under "everything
else". The fix is one clause in 02 and one in 03 §0, with no core tokens.
The first acceptance bullet is met: 02, 03 §0, and §1.1 name the same nine
std handles, each of which carries authority. `Command` belongs on the
list, and nothing in the change contradicts 01 D7, 03 §3.1, reference 6.6,
or the companion. N1–N4 can wait for later beads, and N3 needs an owner
decision on `OnceLock`.

Validation: this was a read-only review of the working tree at `d8bf49a`,
with `.beads/issues.jsonl`, `design/02-memory.md`, and
`design/03-authority-concurrency.md` modified, on 2026-10-20.

- **Read:**
  - the full `design/` diff and fx-zm4 (`br show`; it has no comments);
  - 02 §9;
  - in 03: §0, §1.1–§1.3, §2.1, §2.2, §2.4, §3.1–§3.5, §4.1, §5, §9.2,
    §9.5, and §11;
  - 01 D7;
  - reference 6.1–6.9 and 12.3;
  - companion 14.1 and 17;
  - `reviews/round25-consistency.md` N7.
- **Greps** over `design/` for every handle name, "resource handle",
  `OnceLock`, `sync::raw`, `ChildStd`, `Stdio`, `LogConfig`, and `RUST-`.
- **Line lengths:** the added lines are at most 74 columns. `02:820`
  (86 columns) predates the change.

No Tide checker exists, so every verdict on Tide behavior is an inference
from the text. Only this file was written; nothing was staged or committed.

## Resolution

Answered in the working tree before round 33.

- **B1:** 02's bullet and 03 §0 now include the opaque types that an
  imported Rust crate with declared authority returns (RUST-2).
- **N1, the unnamed handles:** both lists add a general clause: any value
  a handle's methods return that keeps its authority, such as a `Child`'s
  pipes or the iterator of `Stdin::lines()`. Not answered here: ROOT-4's
  "no public fields" against Rust's `Child`.
- **N2–N4:** out of this bead's scope and reported to the owner. N3
  (`OnceLock`, `Sync` types that can observe mutation, the
  `std::sync::raw` locks) needs an owner decision.
