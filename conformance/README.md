# Tide conformance corpus

D0's conformance oracle (`design/01-design.md`): every v0-core normative
rule that the checker or rustc decides carries an ID, and each ID is traced to
cases with expected verdicts that any implementation must reproduce. Because
every Tide program is a Rust 2024 crate (D18), a case also records rustc's
verdict, so the corpus compares the two checkers (`design/02-memory.md` §11).
[`rules.md`](rules.md) lists the rules and their IDs.

## Running

```
conformance/run.sh [case ...]   # check cases (default: all of cases/)
conformance/run.sh --trace      # each rule in rules.md and the cases that name it
```

A case is named by its path under `cases/` (`authority/lock_accept`) or by an
absolute path. The runner type- and borrow-checks each case with stable rustc
twice, as in the E1 trial: as a 2024-edition library (a binary when a line
starts with `fn main`, so entry-point errors such as E0580 appear) and as a
test crate (`--test`, so test code is checked too). Both builds use the stub
Tide std in `rustc/`, injected as `std`, and pass `-Zallow-features=`, as the
admission driver does (ADMIT-2); the case's rustc errors are the union of the
two. It then checks the case the same two ways with the admission driver,
`crates/tide-admit`, against the stub std built by the pinned nightly, passing
`--tide-grant=unsafe` for `//@ grants: unsafe`. The driver's `FX` codes are
the ones `tide-admit --tide-print-codes` lists, such as `FX0520`; its errors in
those codes are the union of the two runs. It needs an x86_64 host
(`authority/admit_hardware` names `core::arch::x86_64`), stable rustc and cargo
1.96, the nightly that the root `rust-toolchain.toml` pins with `rustc-dev`,
and the crates in `rustc/std/Cargo.lock`, offline. It prints one line per
case:

- `PASS`: rustc's errors match the case's rustc marks, the driver's errors
  match its marks in the driver's codes, and the case has no other `FX`
  marks.
- `AWAITS CHECKER`: both match, and `FX` marks in other codes remain, which
  only later slices of the checker produce; the line lists those codes.
- `FAIL`: rustc's errors differ from the rustc marks (rustc's output
  follows, indented); the driver's errors in its codes differ from those
  marks, or the driver accepts a case that rustc or those marks reject, or
  rejects one that neither does, or it crashes (exits with a status other
  than 0 or 1, or panics; the driver's output follows); or the case
  is malformed (no rule line, an ID not in `rules.md`, a malformed mark, an
  unknown `//@` line).

It exits 0 when no case fails, 1 when one does, and 2 on a setup error (the
stub std or the driver does not build, or `rules.md` lists no IDs).

The runner is the trusted operator: it supplies the nightly stub with
`--tide-std` and its artifact directories with `--tide-dep-dir`. The driver
rejects raw `--extern`, `-L`, `--cfg`, `--sysroot`, `-C`, `-Z`, and response
files, constructs its own metadata invocation, and runs the compiler with an
environment holding only `RUSTC_ICE=0`, so a crash writes no report file.
Candidate Cargo and toolchain files do not configure it.
Its source snapshot is the case's source directory; output and successful
content-addressed cache entries live outside it, under `target/conformance`.
This remains a compile-time stub oracle, without an OS build/test sandbox:
the stub never runs. Live outside-file and environment canaries belong to
the driver's integration tests, since the stable oracle would read them.

Only the driver's codes are enforced: in any other code, a wrong `FX` mark
still awaits the checker, and a missing one can let a case pass. The `FX`
marks are the design's expected verdicts, which each slice of the checker is
run against.

## Case format

One `.rs` file per case, under `cases/<area>/`: `memory` (02), `authority`
(03 §0–§5 and §6.5: capabilities, admission, `Pure`, `Shared::with`),
`tasks` (the rest of 03 §6–§8: tasks, channels, parallelism), and
`contracts` (04).

```rust
//@ rules: LOCK-1
// A `Shared` handle as a payload is `Send` but not `Pure`: only the checker rejects it.
use std::sync::Shared;
use std::task::Tasks;

pub fn nest(tasks: &Tasks) {
    let inner = Shared::new(0u64, tasks);
    let _outer = Shared::new(inner, tasks); //~ FX0606
}
```

- The first line, `//@ rules: ID, ID`, names the rules the case exercises.
  Every ID must be in `rules.md`.
- Optional lines after it set what the manifest and build would:
  `//@ grants: unsafe` gives the case's package the `unsafe` grant (03
  RUN-4), which it otherwise lacks, and `//@ profile: release` checks it in
  the release profile instead of debug. Only the checker acts on them, and
  the runner rejects any other `//@` line.
- A comment says what the case shows.
- Each line that must get an error ends with `//~` and its codes: `E####`
  for a rustc error, `error` for a rustc error without a code (such as
  "lifetime may not live long enough"), and `FX####` for the checker; a line
  both report on carries both. A `Send`, `Sync`, or `Fn` failure carries an
  `FX` code only where the design gives that code for it (PAR-5 to PAR-7
  do; `rules.md` writes "E0277 for `Send`" where a rule's code covers only
  its `Pure` part), and otherwise the rustc mark alone. Where the driver
  reports a code in place of rustc's (an ambient call or a path that is not
  admitted), the rustc mark records what plain rustc says. A case with no
  marks must be accepted.
- rustc's errors must equal the rustc marks as a set of (line, code) pairs.
  An error on an unmarked line, an uncoded error where no `error` mark is,
  or an error in another file fails the case. A case whose only marks are
  `FX` must pass rustc. Warnings are ignored.

The shape follows rustc's own UI tests (`//@` directives, `//~` marks) and
the D18 spike's per-line `FX` marks (`spikes/d18/admit/tests/`), rather than
one verdict in a header: a per-line mark fails a case rejected on the wrong
line or for an incidental reason, which a single verdict would let pass.

The stub is a compile-time fixture of Tide's std (`crates/tide-std`), which
owns runtime behavior. It includes Tide's std's crate-root items
(`crates/tide-std/src/root.rs`: `Root`, `Pure`, `tide::main`, the sinks and
`panic!`, the runtime entry, and Rust's re-exports) and declares the
capability types with that file's `capability!`, so every item Tide's std
exposes has one definition, at the same path, in both; the admission driver's
rules (`policy.rs`, `pure.rs`) key them alike. A case that uses only those
items is checked against Tide's std's own definitions. Every other item is the
stub's own, a signature for an item Tide's std does not provide yet. `Pure` is
a blanket impl, so rustc accepts every `Pure` bound and every `Pure` verdict
is the checker's (an `FX` mark). The stub's own bodies are `unimplemented!()`,
so the corpus decides compile-time verdicts only; rules decided at run time,
in the simulator, or in the sandbox are outside it (`rules.md` lists them).
`rustc/std` is copied from `trials/e1-dialect/rustc/std`, whose copy stays
frozen as the trial's record, and uses the trial's `macros` crate as is, and
Tide's std's macros (`crates/tide-macros`); `rustc/macros` adds `#[pure]` and
`#[trusted_pure]`, which leave their item unchanged. The trial's contract
macros are the D18 spike's (`spikes/d18/macros`), so a contract case's rustc
marks record that lowering's errors, and the case's comment says where they
differ from the design's (04 §1.8, CLAUSE-7, lists the spike's known gaps).
Add an item to the stub when a case needs it and Tide's std does not provide
it, with the signature the design gives, and mark `GUESS` on a signature no
design document fixes. When Tide's std comes to provide one of the stub's
items, the stub includes Tide's std's definition, as it includes `root.rs`,
and drops its own. If the stub cannot express a verdict a rule needs, record
the gap on the case's bead.

## Rule IDs

A rule ID is `PREFIX-N`: an uppercase prefix naming a group of rules and a
number counted from 1 within the prefix, as 03 already does (`ROOT-1`,
`LOCK-4`). Each prefix belongs to one chapter, which defines each of its rules
with a bold `**PREFIX-N.**` lead, optionally followed by a title, at the start
of the rule's bullet, list item, or sentence. An ID is never renumbered or
reused: a removed rule's ID stays retired, noted where the rule stood, and
a new rule takes the next number. A rule another chapter restates keeps its
owner's ID, and the restatement cites it. In `rules.md`'s first column a
chapter's own ID goes bare (`MOVE-1`), and `= ID` marks a row that restates
another chapter's rule.

Some rules in 02, 03, and 04 have no ID yet (`none yet` in `rules.md`). The
bead that writes their cases assigns them from these prefixes, defining each
ID in its section:

| Prefix | Section | Rules |
| --- | --- | --- |
| `MOVE` | 02 §1.2 | moves and copies |
| `LOAN` | 02 §1.4 | loans and conflicting accesses |
| `ORIGIN` | 02 §2.1, §2.2, §2.7, §2.8 | origin flow, well-formedness, bodies |
| `LIFE` | 02 §2.3, §2.5 | lifetime syntax and elision |
| `BORROW` | 02 §3 | borrowing types |
| `VAR` | 02 §4 | variance and subtyping |
| `DYN` | 02 §5.1 | trait objects |
| `OPAQUE` | 02 §5.2 | `impl Trait` |
| `CLOSURE` | 02 §5.3, §5.4 | closures and higher-ranked origins |
| `DROP` | 02 §6 | drop check |
| `SEND` | 02 §8.1 | `Send`, `Sync`, task isolation |
| `SCOPE` | 02 §8.2 | scopes |
| `SHARED` | 02 §8.4 | `Shared::with` payload references |
| `PURE` | 02 §9 | `Pure` (03 §3.1 cites it) |
| `CAP` | 03 §0–§1 | capability methods and handle constructors |
| `UNSAFE` | 03 §2.6 | what the `unsafe` grant gates (RUN-4's hole 1) |
| `AFREE` | 03 §3.2, §3.5 | `#[pure]`, `const fn`, memoization |
| `PAR` | 03 §8.1 | `join` and parallel closures (`PAR-5` on) |
| `FORM` | 04 §1.2 | clause grammar |

`PURE` goes to 02 §9 because 02 is the definition the checker implements
(`README.md`); 03 §3.1 then cites it rather than restating it. 02 §8.5
restates 03 §8.1's `join` and parallel-closure rules, so they take `PAR`
IDs and 02 cites them.
