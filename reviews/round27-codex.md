# Tide consistency review, round 27 (Codex): fx-i5p confirmation

## Verdict

**request changes** — B1 needs a small qualification to preserve D8's
cancellation-echo rule. Round 26's B1 and N1–N5 are answered as its
Resolution claims; N6 remains an acknowledged pre-existing omission.
The other findings below are non-blocking. Confidence is high on the
textual checks and compiled Rust checks, with the platform and token-count
limits stated below.

Scope: reviewed the working-tree `git diff -- design/` against HEAD
`417fbd5ee98a72a0926f3d3003b5d3cf6a4e1021` on 2026-10-20. The changed
files are `design/01-design.md`, `design/03-authority-concurrency.md`,
`design/reference.md`, and `design/reference-companion.md`. The reviewed
diff's SHA-256 is
`80078aa14a56a63616c1a251981cd138eab4f28023efe259cf512d12d9e67ee7`.
Read `AGENTS.md`, `GOVERNANCE.md`, `BR_WORKFLOW.md`, the README, both
untracked reviews (`reviews/round25-consistency.md` and
`reviews/round26-consistency.md`, including their Resolutions), relevant
parts of chapters 01–04 and the reference pair, and
`trials/e1-dialect/grading.md` §5. Ran `br show fx-i5p` and
`br comments fx-i5p`; the owner's 2026-10-20 comment is the decision source
for G3, G4, G5, G6, and G9. No implementation or planning changes were
authorized or made.

## Round-26 confirmation

These labels refer to round 26, not this review's findings.

| Item | Result | Current evidence |
| --- | --- | --- |
| B1 | Closed | `design/03-authority-concurrency.md:116–118` now distinguishes unambiguous inherent calls from an unused `Read` import; `design/reference-companion.md:49` gives `FX0913` and the safe fix. This agrees with `design/reference.md:984`. The inherent-method probe below confirms the warning. |
| N1 | Closed | `design/reference-companion.md:49` explicitly retains `Read` for both `Stdin` and `&Stdin`, including generic-reader use. |
| N2 | Closed | `design/reference.md:557–559` now limits the I/O-method ban to **re-exported** std types, matching D7 (`design/01-design.md:419–430`) and the Tide-owned directory handles (`design/03-authority-concurrency.md:102–111`). |
| N3 | Closed for the cases reported | `design/03-authority-concurrency.md:1170–1172` and `design/reference-companion.md:447–448` now require survival of the entry's creation and every ancestor directory's creation. This excludes both a surviving rename without its source creation and a surviving entry under a lost new directory. |
| N4 | Closed | `design/03-authority-concurrency.md:836–838` conditions only the last four traits (`Clone`, `Copy`, `PartialEq`, `Eq`) on `T`. `Debug`, `Display`, and `Error` are unconditional. The Rust trait probe confirms this distinction. |
| N5 | Closed for the identified prose lines | `design/reference.md:535–537` replaces the reported 93-column line; the channel paragraph is rewrapped at `design/03-authority-concurrency.md:838–842`. No semantic issue follows from its line breaks. |
| N6 | Deferred, not fixed | `design/reference.md:384` still lists `remove_dir_all`, while `design/03-authority-concurrency.md:69` omits it and `:1164–1165` does not specify its multi-entry crash behavior. Both the reference listing and table omission exist at HEAD, so the Resolution correctly calls this pre-existing and out of scope. |

## Owner decisions and cross-document checks

| Gap | Confirmation |
| --- | --- |
| G3 | The three error enums, payload recovery, `is_cancelled`, unchanged `try_*` errors, and iteration ending at the first error are stated in `design/03-authority-concurrency.md:818–847`; the core summarizes them at `design/reference.md:688–699`. `recv_deadline` is a checkpoint in both the table (`design/03-authority-concurrency.md:146`) and the prose. Disconnect itself sets no echo bit, as the owner chose; the added explanation needs B1's qualification. |
| G4 | D15's entry-atomicity clause (`design/01-design.md:845–848`), the normative crash rules (`design/03-authority-concurrency.md:1160–1176`), the companion (`design/reference-companion.md:440–450`), and atomic replace (`design/reference.md:415–423`) agree. File contents may tear; directory-entry changes are whole. The dependency restriction answers round 26's reported namespace-coherence cases. |
| G5 | `design/03-authority-concurrency.md:121–127` and `design/reference.md:501–506` give exactly the owner's transient set: `Timeout`, `Reset`, `Refused`, and status 408/429/5xx. The round trip, reply delay, no-peer refusal and unknown-host DNS result agree between `design/03-authority-concurrency.md:1112–1117` and `design/reference-companion.md:404–408`; the core gives the successful-GET timing at `design/reference.md:535–536`. |
| G6 | `design/03-authority-concurrency.md:102–111` gives `io::Result<DirEntry>` items, the four portable methods, inherited `Fs` authority, suspension, and Rust's `OsStr`/`OsString` API. `design/reference.md:40` admits the string types, and `:383–385` retains std-like directory listing with the authority qualification. The repaired re-export wording at `:557` removes the earlier admission conflict. N2 below concerns only the Unix attribution. |
| G9 | Inherent `&self` stdin methods and both `Read` impls are explicit in `design/03-authority-concurrency.md:112–118`, `design/reference.md:334`, and `design/reference-companion.md:49`. The owner's setup bounds and `Send + 'static` peer appear in `design/03-authority-concurrency.md:1095–1103`, `design/reference-companion.md:409–415`, and the core HTTP row (`design/reference.md:526`). |

No additional contradiction was found in the changed lines against 02,
04, or the README. Directory resources follow 03's existing non-`Pure`
resource-handle rule (`design/03-authority-concurrency.md:415–428`) and
the core's rule (`design/reference.md:470–477`). The HTTP peer's `Send`
bound does not establish a missing `Sync` requirement: v0 simulation runs
one task at a time on baton-passed threads
(`design/03-authority-concurrency.md:1079–1086`). The existing omission of
resource handles from 02's explicit non-`Pure` list
(`design/02-memory.md:812–829`) was already reported in round 25 N7;
this change does not introduce it. The README correctly identifies the
decision owner, normative chapters, and reference split (`README.md:55–74`).

**Provisional markers:** no fx-i5p decision remains marked provisional in
the core. The marker at `design/reference.md:338` qualifies the general
rule about *every* capability method, not the separately decided stdin
row at `:334`. The markers at `:530` and `:535` apply to `Request`/`Record`
and file-operation latency, respectively; the owner did not decide those
details in fx-i5p. Network classification, GET cost, setup bounds and peer
`Send` no longer carry the removed markers.

## New blocking problems

### B1. A disconnect must not override an already observed cancellation

**Evidence.** New text at `design/03-authority-concurrency.md:764–765`
says an `Err` returned after a channel disconnect is an independent
failure. The new explanation at `:849–851` repeats that without a
qualification. D8 instead excludes every child error returned after
observing cancellation caused by the body or a sibling
(`design/01-design.md:534–545`). TASK-2 implements that policy with a bit
set on observation; it never inspects the eventual error value
(`design/03-authority-concurrency.md:757–763`). The core preserves that
rule (`design/reference.md:614–617`). Non-checkpoint `try_recv` keeps
Rust's errors (`design/03-authority-concurrency.md:841–842`).

**Concrete conflicting trace (inference from those rules).** Child 0
observes cancellation after child 1 fails and drops its sender. Child 0
then calls `try_recv`, gets `Disconnected`, and returns an `Err` for it.
TASK-2's bit is already set, so D8 reports child 1's error. The new
unqualified sentence instead calls child 0 independent; because child 0
was spawned earlier, that reading reports child 0's disconnect error.
Level-triggering does not prevent this trace: `try_recv` is not a
checkpoint. A runtime implementer gets two different specified outcomes.

**Smallest fix.** In both new explanations, say that a disconnect **alone**
sets no echo bit and that its `Err` is independent **if the child has not
already observed the relevant cancellation**. Keep D8 and the existing
bit algorithm. This preserves the owner's decision that disconnect is
not itself cancellation; it needs no new error-inspection policy.

## Non-blocking notes

### N1. A disconnect error only *may* outrank its peer's failure

`design/reference-companion.md:50` says such an error "would outrank"
the failure that dropped the peer. D8 and TASK-2 select the
earliest-spawned independent failing child (`design/01-design.md:540–541`;
`design/03-authority-concurrency.md:751–754,769`). If the failed peer was
spawned first, its error still wins. This is an explanatory overstatement;
the ordering algorithm is explicit. Change **would** to **could**. The
advice to return `Ok` for an expected pipeline disconnect remains sound.

### N2. Rust's first-error `ReadDir` behavior is not uniformly established for Unix

**Evidence.** `design/03-authority-concurrency.md:108–109` attributes
first-error termination to Rust "on Unix". Rust 1.96's Linux branch sets
`end_of_stream` unconditionally when the directory read fails
([Rust source, lines 865–887](https://github.com/rust-lang/rust/blob/1.96.0/library/std/src/sys/fs/unix.rs#L865-L887)).
The macOS-selected branch instead sets it only if `entry_ptr` is null;
that pointer persists across skipped dot entries within the call
([Rust source, lines 946–985](https://github.com/rust-lang/rust/blob/1.96.0/library/std/src/sys/fs/unix.rs#L946-L985)).
Apple's `readdir_r` error path returns before assigning its result pointer
([Apple source, lines 149–175](https://github.com/apple-oss-distributions/Libc/blob/main/gen/FreeBSD/readdir.c#L149-L175),
read on 2026-10-20).

**Inference and probe limit.** An error after a skipped `.` can therefore
leave the iterator able to continue. A compiled model of that branch,
with a stub that returns dot, then EIO without overwriting the pointer,
then a file, asserted `Err(EIO)`, `Ok("later-file")`, then `None` and
exited 0. This proves the control-flow counterexample, not execution on
macOS. Narrow the attribution to **Linux**, or remove the attribution.
Tide's explicit first-error policy is consistent and can remain unchanged.

### N3. The recorded token count could not be independently reproduced

`design/reference-companion.md:12–14` records **24,993** tokens. D14's
budget is **25,000** (`design/01-design.md:826–830`), leaving seven tokens
if that count is accurate. The required script calls the Claude API
(`scripts/count-tokens.sh:12–25`). This review ran:

```sh
timeout 400 scripts/count-tokens.sh design/reference.md </dev/null
```

It exited **1** with `API Error: Unable to connect to API (ESERVFAIL)`
and produced no count. There is no evidence here that the reference
exceeds its budget, but neither the recorded number nor its headroom is
independently certified. Rerun the same command when the API is reachable;
no design edit is proposed on the basis of this infrastructure failure.

## Rust verification

Environment: `rustc 1.96.0 (ac68faa20 2026-05-25)`, edition 2024,
`x86_64-unknown-linux-gnu`, LLVM 22.1.2; installed rust-src from the same
toolchain. Probes were isolated under `/tmp/tide-round27-rustfacts/`.
Each used `rustc --edition 2024 <name>.rs -o <name>`; successful probes
were executed with stdin closed. The summary records the checked behavior
so the temporary binaries are not the proof record.

| Probe | Observation and result |
| --- | --- |
| `facts_ok` | Verified Rust's unit `RecvError`, tuple `SendError`, timeout enum, three stated `From` conversions, all seven traits for the non-generic errors and `SendError<u8>`, and unconditional `Debug + Display + Error` for `SendError<Opaque>` where `Opaque` implements none of them. Also type-checked `Read` for `Stdin`/`&Stdin`, `IsTerminal` for `Stdin`, all four `DirEntry` method results, `Command::args` with `OsStr`/`String` items, independent `env` key/value types, and both text and bytes as `AsRef<[u8]>`. Compile/run exit 0. |
| `inherent_ok` | A stand-in stdin type has an inherent `read_to_string(&self, ..)` and `Read` impls for the value and its reference whose `read` panics. Value and reference calls use the inherent method and produce the asserted text. rustc warns `unused import: std::io::Read`. Compile/run exit 0. |
| `recv_binding_ok` | Against a local enum with `Disconnected` and `Cancelled`, bare `Err(RecvError)` compiles as a binding, catches `Cancelled`, and warns `non_snake_case`. Compile/run exit 0. |
| `send_shape_fail` | Against a local `SendError<T>` enum, the old `Err(SendError(v))` pattern fails with **E0532**, exit 1 as expected. |
| `peer_send_fail` | A moved `Rc<Cell<_>>` captured by `Fn(&()) + Send + 'static` fails with **E0277**, exit 1 as expected. |
| `read_dir_state_ok` | The platform-branch model described in N2 asserts continuation after EIO. Compile/run exit 0; no macOS runtime claim. |

Source inspection also confirmed the exact bounds and impl ownership:
installed `std/src/sync/mpsc.rs:406–454,1143–1157,1185–1258`,
`io/stdio.rs:411–498,1247–1265`, `fs.rs:2539–2641`, and
`process.rs:774–777,810–813`. Official references:
[SendError](https://doc.rust-lang.org/std/sync/mpsc/struct.SendError.html),
[Stdin](https://doc.rust-lang.org/std/io/struct.Stdin.html),
[DirEntry](https://doc.rust-lang.org/std/fs/struct.DirEntry.html),
[Command](https://doc.rust-lang.org/std/process/struct.Command.html#method.args),
[method lookup](https://doc.rust-lang.org/reference/expressions/method-call-expr.html),
[E0532](https://doc.rust-lang.org/error_codes/E0532.html), and
[E0277](https://doc.rust-lang.org/error_codes/E0277.html).
The companion's diagnostic rows (`design/reference-companion.md:47–49,51`)
are correct. The only Rust attribution qualification found is N2.

## Ready to merge

**No: fix B1's unqualified disconnect classification.** The smallest
correction changes two explanatory clauses in 03 and costs no core
reference tokens. The earlier round's requested edits are present and the
owner's decisions are carried through. N1 and N2 need only wording
clarifications; N3 is an explicit verification limit.

`git diff --check -- design/` passed. This review does not prove Tide
runtime behavior: no checker/runtime exists for this design
(`README.md:36–44`). macOS execution and the exact D14 token count were
not verified. Only `reviews/round27-codex.md` was written in the repository;
no design files were edited and nothing was staged or committed.

## Resolution

Answered in the working tree before round 28.

- **B1:** Codex's smallest fix, applied. TASK-2 now says a disconnect
  alone sets no bit, so an `Err` after one is independent only for a child
  that has observed no such cancellation. §6.4 says the same. Reference
  7.3 and companion 14.1 now read "a disconnect alone is not an echo". D8
  and the bit algorithm are unchanged.
- **N1:** companion 14.1 says "could outrank".
- **N2:** 03 §1.1 attributes first-error termination to Rust on Linux.
- **N3:** `scripts/count-tokens.sh design/reference.md` reran in the
  authoring session after these edits and printed 24,995. The companion
  records that figure.
