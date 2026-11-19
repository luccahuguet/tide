# fx-zu4f.15 protocol: chrono_anchor and same-class rows, re-tested after the std conversion fix

Written on 2026-11-19 by the fx-zu4f.15 protocol builder (Claude Opus), in
worktree branch `fx-zu4f.15` from main `04dde61`, before fx-qlek.18 (the std
conversion fix) landed on main and before any port of this test was copied
or run. The "Frozen criteria" on bead fx-zu4f.15 stay binding. This file adds
operational detail and narrows them where it says so. It never widens them,
and it leaves fx-zu4f.8's caught rule unchanged.

This is a new pre-registered test. It does not revise, rerun or replace
fx-zu4f.8 (`research/47-malware-rescreen.md`), whose documented negative
stands as published, or fx-zu4f.5 (`research/37`). The record cites both as
prior work and edits neither.

## 1. Disclosure: why this pool, and when it was chosen

This text, or text with the same meaning, opens the record and any write-up.

- **The pool was chosen after fx-zu4f.8 saw the rows.** fx-zu4f.8 screened 87
  malicious-crate advisories and caught none. Its nearest miss was
  chrono_anchor (RUSTSEC-2026-0039): the buggy port drew counted TD codes, and
  the capability-passing version failed only on rustc `E0277` at the table
  form `clock.sleep(d)?`, because Tide's std at that revision had no
  conversion from `Cancelled` into `TaskError`.
- **Why it is re-tested.** Tide's std had a gap: the documented idiom
  `clock.sleep(d)?` did not compile inside a task body. The screen exposed
  it. The owner chose to fix the std (fx-qlek.18, an ergonomics fix on its own
  merits) and to re-test under a new registration (relayed owner decision,
  Lucca via tide-desk, 2026-11-19: "1").
- **What the result can and cannot show.** The outcome at R' is close to
  predictable: fx-qlek.18 adds a case proving the idiom compiles in a task
  body, and fx-zu4f.8 disclosed that two non-registered variants of this
  very port were admitted. A catch here shows that the re-tested row meets
  the caught rule at R'. It never shows that Tide caught the row the first
  time, and no text may say or imply that. fx-zu4f.8's result stays "0
  caught of 87".
- Whether a catch here counts as the RC's public catch is decided by
  fx-qlek.7's rule and its cold-reader pass, not by this test.

## 2. Incorporated by reference, unchanged

Each item below binds this test exactly as it bound fx-zu4f.8. RUN checks
every SHA-256 before first use and stops if one differs.

| Item | Where | SHA-256 |
| --- | --- | --- |
| fx-zu4f.8's registered protocol (sections 0 to 9: names, registration, R, sources, order of determination, the faithful port and its mapping table, the capability-passing version with §6's closed list of allowed diff lines and allowed fixes, commands, safety terms, the record) | `research/protocols/fx-zu4f.8-protocol.md`, committed `f04beb8`, OpenTimestamps proof `research/protocols/fx-zu4f.8-protocol.md.ots` (anchored in Bitcoin block 976452) | `4e2016dff7838f9bdcccf4b2cedbabf7051ab998e2a19af77f6e7347cfdc7e26`; proof file `ed68e64aed7841562106f75ec1c3fd1b0ce456597570e1af055f7441be4c6146` |
| fx-zu4f.8's frozen criteria, including the caught rule (fx-zu4f.5's, unchanged) | bead fx-zu4f.8, `description` field (closed 2026-11-19T04:50:38Z) | whole field `f20b56674098188f148c9270893ac11b3136dc72cfaf48e53c0ea7f12cb8a603`; the "**Caught rule**" block alone `7cb2479cf213cea88c8727a96d73cb99ef8173ac30453ec312ef43b8b3b89d57` |
| RULING-0039 (tide-lead, 2026-11-19 03:54Z): the table form of `thread::sleep(d)` is `clock.sleep(d)?`; `let _ =` is not an allowed diff line; `.map_err(..)` only for a `?` the source already had | `~/.cache/eon-agents/tide-lead/fx-zu4f.8-work/RULING-0039.md`; stated in `research/47` §2 | `794f6885e978cec817e02eef50fc5dc374c73c6ca9b1931c21ca43837b2fd8d2` |
| RULING-0175 with its correction of 2026-11-19 (the definition of "affected function"; item 3 corrected) | `~/.cache/eon-agents/tide-lead/fx-zu4f.8-work/RULING-0175.md`; stated in `research/47` §2 and deviation D2 | `ff5992e1effd5fbfc4c38734d1ade55b95491477f829564abc1d74985b5882c0` |
| tide-desk's archive release condition, amended, 2026-11-18 14:04 (its four limits) | bead fx-zu4f.8, comment 1106, 2026-11-18T14:04:39Z | `7ad118994870800cec644936f2bd77cfafc4f229e19ad98d3460ec8648e5d836` |
| tide-desk's confirmation of how that condition applies (it binds over port fidelity; a row that cannot meet it without changing its effect-carrying calls is "unresolved (archive condition)") | bead fx-zu4f.8, comment 1312, 2026-11-19T04:20:42Z | `cd8d3ba0c32d326434ba485122e510f95efa9c3a8a8d05ccffe40ad66d9871d0` |
| fx-zu4f.8's committed record, source of the pool and of the carried-over determinations (section 5) | `research/47-malware-rescreen.md` at `fc43b37a8852092eecf9fe832e5e06d1a26d10e9` | `6c6111b7789d0fedd34a4439c403c103b2f8a57442360afe9083660ffc3b896e` |

How the bead-field digests are taken: the SHA-256 of the field's UTF-8 text
exactly as `br --no-db show fx-zu4f.8 --json` (`description`) or
`br --no-db comments fx-zu4f.8 --json` (`text` of the comment with that
`id`) returns it, with no trailing newline added. The caught-rule block is
the description's text from `**Caught rule**` up to, not including,
`**Safety terms**`, with trailing newlines removed.

The caught rule, quoted from that block for the reader (the block above
binds): a candidate is caught when a named TD code fires on a faithful Tide
port of the malicious code, and the capability-passing version, which
receives the authority it uses explicitly, is admitted. fx-zu4f.8's protocol
§4 makes it operational: leg 1, a counted TD code on an in-class operation of
an affected function of the buggy port; leg 2, a clean admission (driver exit
0, no error diagnostic) of the capability-passing version.

Not carried over: fx-zu4f.8's post-registration UDP and resolve reporting
addition (bead fx-zu4f.8, 2026-11-19T00:14:56Z). It added outputs to that
round only.

## 3. The pool and the selection rule

**Rule.** The pool is chrono_anchor (RUSTSEC-2026-0039), which the bead
names, plus every row of fx-zu4f.8's committed record whose leg-2 failure
was the same conversion class. A row of `research/47` §7 (at `fc43b37`) is
in that class when all of these hold:

- C1. Its `capability_run` cell records a run of the capability-passing
  version that exited non-zero (`exit N; ...`, N not 0). Leg 1 passed, and
  leg 2 failed on a run.
- C2. Every diagnostic that cell lists is rustc `E0277`. No other rustc
  error and no TD code remains.
- C3. Each `E0277` is annotated as a `?` that cannot convert a type X into a
  type Y (the enclosing error type).
- C4. Each such X is a type that Tide's std defines at fx-zu4f.8's run
  revision R = `972aba0ee5917a0a70e59f6b2e7429e6e2378e74` (`pub struct X` or
  `pub enum X` under `crates/tide-std/src`): a std error type.
- C5. Record §2's RULING-0039 list of rows "where a table form brings in a
  `?` the source lacked" names the row's crate, or the first word of its
  incident. So the failing `?` is a table form's, not one the source had
  (for which §6 would have allowed a `.map_err`).

**Script.** `research/protocols/fx-zu4f.15-pool/select.py` applies the rule
mechanically. It runs from the repository root as
`python3 -I research/protocols/fx-zu4f.15-pool/select.py`, reads the record
at commit `fc43b37a8852092eecf9fe832e5e06d1a26d10e9` with `git show`, checks
its SHA-256, looks up C4's types with `git grep` at R, and prints every row that recorded
a capability-passing run, with each criterion's verdict and the row's
evidence lines in the record, then the pool. It compiles, runs and fetches
nothing else, and prints no time, so its output is byte-stable. Its output
is committed beside it as `select-output.txt`.

**Result at registration.**

- 8 of the 87 rows recorded a capability-passing run (C1). The other 79 did
  not: 76 never reached a capability-passing run (no port, or stopped on
  paper), 2 failed leg 1 (uniswap-utils,
  finch-rust), 1 stopped at capability absent on paper (evm-units).
- amaperf, checked by the rule and not by hand: its seven rows
  (RUSTSEC-2023-0097 to -0103, one shared port) pass C1 and C5 (amaperf is
  in RULING-0039's list for `Url::parse(..)?`, which converts without
  error), and fail C2: their only errors are `E0433` ×2 on `serde_json`, a
  third-party non-authority crate. Not selected.
- chrono_anchor passes C1 to C5. Its cell reads "exit 1; E0277 x1 (34:44:
  `?` cannot convert Cancelled to TaskError)"; `Cancelled` is defined at R in
  `crates/tide-std/src/task.rs:1871`, named in RULING-0039's list. Record
  lines 450 (§7 row) and 618 (§8 final pass).
- **POOL = { RUSTSEC-2026-0039 }** (1 row). The bead names it, and the rule
  selects it on its own.

**Cross-check against fx-zu4f.8's work evidence** (private, not committed;
done by the protocol builder before registration). No disagreement:

- `fx-zu4f.8-work/screen.tsv` (SHA-256
  `ed96cd00da3cfc88c140eb4cef90638c64ebeb486b7fe486724ae3e40c7f28aa`): 87 rows; the
  `capability_run`, `buggy_codes` and `outcome` cells equal the committed
  table's for all 87 (the record's outcome cells add the access-failure
  note). The same 8 rows have a run; the same values.
- `fx-zu4f.8-work/ports/final.log` (SHA-256
  `f23eb58e817685abf12e7c00214944f72564748dde5e742d91e06ccfda1298f6`, as
  research/47 §8 records): the only capability-passing runs are RUSTSEC-2023-0097
  (`exit 1; E0433 x2`, `serde_json` at 29:15 and 29:35) and
  RUSTSEC-2026-0039 (`exit 1; E0277 x1`, at 34:44: "`?` couldn't convert
  the error to `tide_std::task::TaskError`: the trait
  `tide_std::convert::From<tide_std::task::Cancelled>` is not implemented
  for `tide_std::task::TaskError`"). Line 34 of the
  chrono_anchor capability port is the table form `clock.sleep(..)?`.
- `fx-zu4f.8-work/ports/r1.log` (SHA-256
  `58bc8c019ffb844dbcdbf343f3b36d32ccdaee7ca5f8f9499caa02aef680afe2`): no
  capability-passing run. `ports/dev.log`'s one `E0277` is chrono_anchor's
  development run of the same port, not an outcome.
- The port files' SHA-256s in fx-zu4f.8's work copy equal research/47 §8's
  table (section 5 below).

## 4. The run revision R'

- **Choice.** R' is main's HEAD at the moment tide-lead sends `go`.
  tide-lead sends `go` only after fx-qlek.18 is closed and its landing
  commit is an ancestor of that HEAD, and only after the registration steps
  of section 6. R' is that landing commit when main has not moved since;
  otherwise the later HEAD, and the record says which. The `go` message gives
  R''s full SHA, and RUN copies it into the record. tide-lead chooses R'
  without any result of this test.
- If fx-qlek.18 is closed without landing a conversion that makes the
  documented idiom compile in a task body, this test is not run, and the
  bead records why.
- Everything fx-zu4f.8's §2 says about R applies to R': frozen once the
  first port file is copied; the capability set at R' (present when
  `design/reference.md` at R' lists the operation without "planned" or
  "partial", and its owning bead's landing commit is an ancestor of R');
  tree identity of `crates`, `Cargo.toml`, `Cargo.lock` and
  `rust-toolchain.toml` with R', and a clean tracked tree. The record names
  fx-qlek.18's landing commit and quotes the std conversion it added.
- **Table forms at R'.** The `thread::sleep(d)` row's form is the one
  companion §14.2 gives at R' (RULING-0039 item 1, read at R'). At
  registration it is `clock.sleep(d)?`. If R' gives a different form, the
  record says so, and that form is the table form.

## 5. How fx-zu4f.8's protocol is read for this test

Only names, the pool, the revision and the places listed here differ. Each
item substitutes a name or narrows; none widens.

1. **Names (§0).** RUN is this test's run builder. WT is its worktree;
   WORK = `WT/target/work`; TARGET = `/mnt/build/tide-agents/fx-zu4f.15-target`.
   PRIOR = `/home/lucca/.cache/eon-agents/tide-lead/fx-zu4f.8-work/`, read
   only; nothing in it is executed. POOL is section 3's pool. RUN checks POOL
   at start by rerunning `select.py` and comparing its output byte for byte
   with the committed `select-output.txt`, and stops if they differ.
2. **Registration (§1).** tide-lead records this file's SHA-256 (and the
   pool files') on fx-zu4f.15. RUN starts only after section 6's steps and
   `go`.
3. **Sources (§3), narrowed.** No new search and no new fetch. Each pool
   row's sources are those fx-zu4f.8 collected and recorded for it
   (research/47 §7: chrono_anchor reads S020, S469, S470, S471, S473, S476 and
   the archive copy socket-3; code quoted by socket-3). Nothing is fetched
   again, so no evidence can enter after the result is known.
4. **Order of determination (§4).** Steps 1 to 5 (advisory, sources, class,
   faithful port on paper, safety gate) carry over from fx-zu4f.8's record
   for each pool row, unchanged. Steps 6 to 8 run again at R': the buggy
   port's run (leg 1), the mapping on paper at R', and the
   capability-passing version's run (leg 2). The outcome labels and the
   counted TD codes are §4's, read at R' (including a code R''s
   `conformance/rules.md` may give ambient `Command::new`).
5. **The ports (§5, §6), narrowed.** RUN uses fx-zu4f.8's final port files
   byte for byte, copied from PRIOR into `WORK/ports/<ID>/` and checked
   against research/47 §8:
   - `RUSTSEC-2026-0039/buggy.rs` `3c296fb448ab2742e979d8cdc5373171b7a9b4a37e88da2ce3d933fcd2e4a110`;
   - `RUSTSEC-2026-0039/capability.rs` `51f348513fc80a727a35ce416fe304a0e4bf6ccfa8c5067d7207887c69d179a4`;
   - `RUSTSEC-2026-0039/capability.diff` `c8a002fb329e6695b179556943bbf85cef4dec2bfbbc8b6c3be40429d46dbdf7`.

   No port is rewritten. §6's allowed fixes stay available, unchanged, for
   a rustc error that R' itself causes; each one is recorded as a numbered
   diff against these hashes with the run that prompted it. RUN rechecks
   the capability diff line by line against §6's closed list before the
   final pass, as §6 requires.
6. **Commands (§7).** The harness is fx-zu4f.8's `PRIOR/ports/screen.sh`
   (SHA-256
   `0b45dcbc5a4231059ae794b669ece349a0a3ed7cb2f723586e9cee446ffca043`,
   matching research/47 §8's `0b45dcbc…a043`), copied to
   WORK with only its log path, output root, run list and labels changed, as
   fx-zu4f.8's `screen-r1.sh` was. Every other §7 term applies: heavy.sh,
   the load check, timeouts, the admission command and its forbidden flags,
   the log's contents, development runs in `WORK/ports/dev.log`, and one
   final scripted pass whose log is never overwritten.
7. **Safety (§8) and the archive condition.** chrono_anchor's ports derive
   from archive copy socket-3. The condition of section 2 binds them: hard-
   coded RFC 5737 or 6761 endpoints; nothing archive-derived is executed
   (admission runs only, `--emit=metadata`); authority shape only; work in
   `target/work`. Any build artifact an admitted run leaves (`.rmeta` and
   its cache copies) is listed with its SHA-256 in
   `WORK/ports/undeleted-artifacts.sha256` and deleted unused, by tide-lead
   if RUN's tools refuse. For an admitted version, the record keeps the
   `interface.json`'s SHA-256 and declared authority (§6's trust artifact),
   read before deletion.
8. **The record (§9).** A new file in `research/`, numbered when it lands,
   reported as a new pre-registered test. It contains, in order: section 1's
   disclosure; the result; the registration digests and OpenTimestamps
   anchors; R' and the capability set at R', with fx-qlek.18's landing
   commit; the method, with section 5's narrowings; the counts over this
   pool (never a rate); the pool's rows in §9's row format; the port proof
   (both runs' command lines, environment, toolchain, file hashes, exit
   statuses and diagnostics, from the final pass); the catch or the
   documented negative; and this protocol verbatim. fx-zu4f.8's protocol is
   cited by path and digest, not reproduced again. A catch's write-up, if
   any, follows §9's rules and negatives and repeats section 1's
   disclosure. No port body, payload or archive-derived code is committed;
   §9's `diff -U0` hunks (changed lines only) may be shown.

## 6. Registration and stop

1. The protocol builder commits this file and
   `research/protocols/fx-zu4f.15-pool/` (`select.py`, `select-output.txt`)
   on branch fx-zu4f.15, byte-identical to WORK, writes
   `WORK/PROTOCOL-READY` with every SHA-256, and stops.
2. tide-lead stamps the files with OpenTimestamps, records their SHA-256s on
   fx-zu4f.15, and upgrades the proofs once anchored, all before fx-qlek.18
   lands, so the order can be checked: the anchoring block's time precedes
   fx-qlek.18's landing commit.
3. tide-lead sends `go`, naming R', only after fx-qlek.18 lands. RUN checks
   every digest in sections 2 and 5 and this file's registered digest
   before first use, and stops on any mismatch.
4. The branch lands by `git merge --no-ff` after GOVERNANCE.md's Merge Rule
   for docs (Codex pass, then an Opus review round). Nothing archive-derived
   or private is committed.
