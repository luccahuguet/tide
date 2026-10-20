# Tide adversarial/consistency review, round 57: the unreviewed-branch guarantee (fx-arx)

## Verdict

**request_changes**

Round 56's blocking findings are closed. N2 is closed only in hole 5, and
round 56's B4 fix opened a new v1 gap (B4 below). This round has five
blocking findings:

- B1: RUN-3's sandbox is worded as a denylist on top of Landlock, which
  leaves host writes and credential reads open to code that reaches holes 1
  or 5.
- B2: the `sandbox: full` condition can only be checked after the run, and
  D13 leaves it out.
- B3: RUN-5's positive conformance row contradicts its rule.
- B4: v1's behavior diff runs uncovered candidates inside `tide review`
  itself.
- B5: RUN-5's list of what remains exposed leaves out hole 4 and the
  toolchain parts that run outside the sandbox.

Each fix is a sentence or a clause plus a conformance row. The rest are
non-blocking.

## What was reviewed

The uncommitted working tree on 2026-10-20 against HEAD `87f9cd3`:
`git diff -- design/ README.md`, SHA-256
`503e6478591589122d9f8bc5e798c84eac3d1999a612b77788ec3de939193440`.

I read the changed text: 03 §2.6 at `03:423-590`, the ADMIT-1 entry at
`03:410-412`, and REPLAY-1 at `03:1436-1444`; D7 at `01:420-431`, the D13
bullet at `01:828-855`, and the tier row at `01:82`; companion §19 at
`companion:495-518` and its header at `companion:3-16`; and
`reference.md:5-10` and `:951-967`. I checked it against 01 D11
(`01:704-754`), D13, D15 (`01:889-918`), and D18 (`01:959-1010`); 03 §1.1
(`03:63-91`), §1.3 (`03:175-213`), ROOT-1..5 (`03:217-241`), PANIC-1
(`03:293-308`), SINK-1..10 (`03:313-371`), ADMIT-1/2 (`03:379-421`),
§9.2-§9.8 (`03:1306-1478`), §10 (`03:1479-1532`), and §11
(`03:1534-1588`); 04 §11 (`04:1187-1232`); `reference.md` §6.9
(`reference.md:545-559`); the D18 spike driver
(`spikes/d18/admit/scripts/tide-rustc:18-32`); and round 56. I ran two
probes with rustc 1.96, described where they are used, and checked one
claim about Landlock against the kernel's documentation. I did not count
tokens.

## Round 56 closure

- **B1 (transitive dependencies of a granted package).** Closed: RUN-5 says
  "or of any package it depends on, transitively" (`03:553-556`), and the
  row tests the lockfile case (`03:590`). See B3 for the row's positive
  case, which now contradicts the rule.
- **B2 (existing Rust crates and Bend dependencies).** Closed: "adds or
  changes no imported Rust crate (its declaration or lockfile entry), build
  script, or Bend dependency (its pin, kernel sources, or law files)"
  (`03:550-552`), and the row has both cases (`03:590`). See B5 for a
  covered candidate that still reaches hole 4.
- **B3 (inherited terminal).** Closed for the child: a new session with no
  controlling terminal, an empty stdin, stdout and stderr as pipes, escaped
  output (`03:477-481`), and a pseudo-terminal row (`03:588`). RUN-1 still
  cites §2.4 for "the sinks go to the harness" (`03:442-443`), but RUN-3
  now makes that true (N10). The same class remains open for the
  toolchain's own output (N2).
- **B4 (`tide review` runs no candidate code).** Closed as worded
  (`03:544-547`), but the v1 clause added by the fix opens B4 below.
- **N1 (early exit).** Closed for started tests (`03:483-485`, row
  `03:588`). Tests that never started are N4.
- **N2 (review's parsers).** Closed in hole 5 (`03:528-531`). The closing
  paragraph (`03:540-543`) and RUN-5's list of what remains exposed
  (`03:559-561`) still omit the harness and the parsers, which run outside
  the child sandbox (B5).
- **N3 (new dependency granting `unsafe` downstream).** Closed in the row
  (`03:590`).
- **N4 (inline replay manifest).** Closed: REPLAY-1 owns it
  (`03:1442-1444`), and RUN-1 cites it (`03:431-433`). D15's summary
  (`01:900-903`) still says "files" without "inline". That is acceptable,
  since REPLAY-1 owns the format.
- **N5 (broad summaries).** Closed: D13 (`01:836-837`) and companion
  (`companion:499-502`) now say binary, libraries, and output directory, and
  "the branch's tree".
- **N6 (link at the output directory's path).** Closed: the harness creates
  the directory and fails the run if anything already holds its path
  (`03:472-474`).
- **N7 (`global_asm!` absent from reference.md).** I agree it can stay.
  `reference.md:553` makes `core::arch` items `FX0520`, which is
  conservative for an agent, and the checker's diagnostic gives the
  grant-specific rule.
- **N8 (hole 6 title).** Closed (`03:535`).

## Blocking findings

### B1. RUN-3's sandbox does not confine what RUN-4 and RUN-5 say it confines

**Where.** RUN-3 lists what the child may do and what it may not: open by
path only the binary, libraries, and output directory; and "it may not
create symbolic or hard links, create sockets, set up io_uring ..., or trace
or signal other processes. ... Memory, threads, futexes, and clock reads
remain" (`03:488-497`). RUN-4's closing paragraph says holes 1, 2, 3, and 5
"meet RUN-3's sandbox" (`03:540-541`). RUN-5 says an unsound reviewed
abstraction and toolchain bugs are "both confined by the sandbox at run
time" (`03:559-561`). The RUN-3 row tests only the listed operations
(`03:588`).

**Problem.** Read plainly, the seccomp part is a denylist: it names the
syscalls that are refused, and "remain" explains why a few others stay. On
that reading, native code in the child keeps three routes that RUN-4 does
not list. Hole 5 lets safe code get native code: a covered candidate with
no grant can use `cve-rs`-style code (`03:531-534`).

1. **Metadata writes on host files.** Landlock cannot restrict
   `chmod(2)`, `chown(2)`, `setxattr(2)`, `utime(2)`, `stat(2)`, or
   `access(2)`. The kernel documentation says so: "It is currently not
   possible to restrict some file-related actions accessible through these
   syscall families: chdir(2), stat(2), flock(2), chmod(2), chown(2),
   setxattr(2), utime(2), fcntl(2), access(2)" (docs.kernel.org,
   userspace-api/landlock). So the child can make the operator's private key
   world-readable, change timestamps that make-style builds rely on, or
   probe which files exist. These are writes outside the run.
2. **The kernel keyring.** `keyctl` reads the user keyring, which every
   process of the user shares. Kerberos `KEYRING:` credential caches live
   there. These are credentials that RUN-3 is said to keep from test code
   (`03:575-576`).
3. **System V and POSIX IPC** (`shmget`/`shmat`, `msgsnd`, `mq_open`) with
   other processes of the same user. Neither Landlock nor the listed
   denials cover them.

What the child reads it can then print to its stdout pipe. That output
reaches the operator's logs.

**Evidence vs. inference.** Evidence: the quoted text, the row, and the
Landlock documentation. Inference: that an implementer builds the
denylist the text describes. The RUN-3 row would pass such an
implementation.

**Smallest fix.** In RUN-3: "seccomp admits only the system calls the
binary needs, for memory, threads, futexes, clock reads, process exit, and
I/O on descriptors it holds and on files Landlock allows, and refuses every
other call, metadata changes, keyrings, System V and POSIX IPC, and
namespaces among them". Add a `chmod` of a canary file, a read of a canary
key in the user keyring, and an attach to a canary System V segment to the
RUN-3 row.

### B2. `sandbox: full` is known only after the run, and D13 drops the condition

**Where.** RUN-5: "When a covered candidate's runs report `sandbox: full`,
an orchestrator may run its `tide test` and `tide sim` without a VM per
run" (`03:557-559`). Companion §19 says the same: "only when it prints
`run: covered` and the test results say `sandbox: full`"
(`companion:513-514`). RUN-3 puts the level only in "every run result"
(`03:498-501`). D13 says a covered branch "can be tested inside the agent's
session without a VM per run" (`01:846-847`), with no sandbox condition.

**Problem.** The level is reported only after the children have run, so an
orchestrator following §19 has already run the branch before it learns the
level is `none` (macOS in v0) or `partial`. On such a host, B1's routes and
plain file reads are open to a covered candidate that exploits hole 5. The
complement paragraph's claim that RUN-3 keeps the session's credentials
from test code (`03:575-576`) then fails without notice. D13, the decision
record, states the permission unconditionally. That contradicts RUN-5 and
overclaims on every host without Landlock.

**Smallest fix.** Make the level a property of the host that is reported
before any child starts. `tide review <base>` prints `sandbox: <level>`
beside `run:`, and `tide test`/`tide sim` determine the level before
starting the first child. RUN-5 and §19 then condition on the level
`tide review` printed. Add "on a host where the sandbox is `full` (RUN-3)"
to D13's sentence. A stronger option is for the harness to refuse to start
a child below `full` unless the caller passes an explicit flag. Add the
pre-run report to the RUN-3 or RUN-5 row.

### B3. RUN-5's positive row contradicts its rule

**Where.** The rule makes a candidate not covered if it changes "a source
file ... of a package with an `unsafe` or `ffi` grant or of any package it
depends on, transitively" (`03:552-556`). The row expects "`run: covered`
for a candidate that edits only packages without grants" (`03:590`).

**Problem.** Take an in-tree package without grants that a granted in-tree
package depends on (a workspace `util` crate under a granted `ring`). By the
rule, editing it is not covered. By the row, it is covered. An
implementation that passes the row breaks round 56's B1 fix through source
edits rather than lockfile edits. No row case covers the source-edit form
of the transitive rule.

**Smallest fix.** Change the row to "edits only packages without grants on
which no granted package depends", and add "an edit to a package without
grants that a granted package depends on: `run: not covered`".

### B4. v1's behavior diff runs uncovered candidates inside the review step

**Where.** RUN-5: "v1's behavior diff (D11) runs its seeds under RUN-3"
(`03:546-547`). D11: "`tide review` reruns the same simulation seeds on base
and head" (`01:719-721`). RUN-5's purpose: "Surfacing holes before running
... Otherwise the orchestrator reviews the listed holes first, or runs the
branch inside a VM" (`03:544`, `03:565-566`).

**Problem.** In v1, `tide review` runs the head's code for every candidate,
covered or not, before the operator has read the `run:` line. A candidate
that adds an `ffi` grant or a Bend kernel reaches hole 2, or hole 4, which
is outside the child's sandbox (`03:522-524`). It does so during the step
meant to decide whether to run it at all. RUN-3 does not confine either
hole, so "under RUN-3" does not make this safe. Round 56's B4 fix added
this clause.

**Smallest fix.** "v1's behavior diff runs the head's seeds, under RUN-3,
only when the candidate is covered or the operator asks for it (D11)".
Mirror it in D11's behavior-diff bullet.

### B5. RUN-5 understates what a covered candidate is exposed to

**Where.** "The exposure left is an unsound reviewed abstraction and
toolchain bugs, both confined by the sandbox at run time, and resource use"
(`03:559-561`). The closing paragraph says only the session sandbox
confines "hole 4, compiler bugs exploited at build time, and resource use"
(`03:541-543`). Hole 4: "A test calling a kernel without a `reference` uses
a server the harness starts outside the child's sandbox" (`03:522-524`).
Hole 5 includes bugs "in the harness, which reads the result channel and
output directory outside the sandbox; or in `tide review`'s parsers"
(`03:528-531`).

**Problem.**

1. Take a base with a process-mode Bend dependency (`03:1482`, no grant
   needed). A covered candidate can drop `reference = ..` from a `#[bend]`
   item, or add a `#[bend]` call to an existing def, in a package without
   grants. Neither change touches the pin, the kernel sources, or the law
   files, so RUN-5 prints `run: covered`. Its tests then send inputs the
   candidate chooses to the real kernel server, outside the child sandbox.
   Bend's runtime is not hardened against hostile input (inference).
2. Harness and parser bugs run outside the child sandbox by definition, so
   "confined by the sandbox at run time" is false for them.

Both are listed holes, so there is no new route. The sentence telling the
orchestrator what remains is what overclaims (later tier for item 1, v0 for
item 2).

**Smallest fix.** "The exposure left is an unsound reviewed abstraction and
toolchain soundness bugs, which meet RUN-3's sandbox; bugs in the harness
and in `tide review`'s parsers, and the base's Bend kernels (hole 4), which
meet only the session's sandbox; and resource use". Add the harness and
the parsers to the closing paragraph's list.

## Non-blocking notes

### N1. Safe code can forge results, `#[protected]` tests included

Hole 1 says unsafe code can write the result channel, "so it can forge
results too" (`03:511`). Hole 5 says only that safe code can "forge a
capability" (`03:533-534`). A covered candidate with no grant can use a
rustc soundness bug to corrupt the in-process harness. It can then report
a pass for any test in the same child, in CI too, `#[protected]` tests
included (`04:1229`). Add "and forge results" to hole 5, and note in RUN-5
that results are evidence only modulo holes 1 and 5. 04 §11's anti-gaming
argument rests on test results and does not name this. That is out of
scope here, a possible follow-up bead.

### N2. Escaping covers only the child's output

RUN-3 escapes the child's output (`03:481-482`). `tide check` diagnostics
and `tide review` lines also print text the candidate controls: file names
(`companion:510`, and git allows ESC bytes in names), package names, and
the `#[deprecated]`, `compile_error!`, and `#[diagnostic]` strings that
Tide's checker relays. A probe with rustc 1.96 showed that it replaces
control characters in such messages with control pictures (U+241B). Tide's
checker and `tide review` should promise the same. Move the escaping
sentence to one that covers every toolchain command.

### N3. Core dumps

PANIC-1's stack-overflow aborts (`03:293-308`), or native code raising
SIGSEGV, can make the kernel write a core file through `core_pattern`, to
the working directory or to systemd-coredump. Both are outside the output
directory, and the content is attacker-chosen (whether Landlock applies to
the kernel's core write is inference). Have the harness set `RLIMIT_CORE`
to 0 before exec.

### N4. Tests that never started

RUN-3 fails a started test whose child exits (`03:483-485`), and PANIC-1
reruns tests that had not run, but only after an abort (`03:303-305`).
Neither covers a normal exit before a listed test starts. Say that such
tests are rerun in a fresh child, or fail the run.

### N5. The kernel is the sandbox's trusted base

The closing paragraph and RUN-5 call holes 1, 2, 3, and 5 confined. Kernel
bugs reachable through the system calls the sandbox allows break that.
Name them in hole 5, or in the closing paragraph.

### N6. Summaries narrower than RUN-4

D13 lists the holes as "... toolchain soundness bugs, and resource use"
(`01:841-842`). RUN-4 also has input-exploitable bugs and side channels
(`03:527-531`, `03:535-538`). Companion §19 says "except through its trust
holes" (`companion:498`), but hole 5 is not the branch's trust hole. Say
"toolchain bugs, resource use, and side channels", and "except through the
holes 03 §2.6 lists".

### N7. Special files in the output directory

RUN-3 lets the child "create and write" files in the output directory
(`03:489-490`). If Landlock's `MAKE_FIFO` is granted, a FIFO named like a
result file blocks the harness, which reads that directory outside the
sandbox. Allow only regular files and directories, and have the harness
open them with `O_NONBLOCK` and a file-type check.

### N8. Error code

"A foreign file or socket call that no model intercepts fails with `EPERM`
(§9.8)" (`03:497-498`). §9.8's `EPERM` comes from interposition
(`03:1470-1472`). A path the kernel's Landlock denies fails with `EACCES`
(kernel documentation). Say "fails (`EPERM` from the interposer, §9.8, or
the sandbox's refusal)".

### N9. D7 grammar

"`option_env!`); and `global_asm!` without an `unsafe` grant" follows "...;
and compile-time reads" (`01:429-430`), so the list has two "and"s. Fold
`global_asm!` into the list before the last item.

### N10. Citations

RUN-5 cites RUN-2 for compile-time evaluation (`03:545`), which RUN-1
defines (`03:434-436`). RUN-1's "the sinks go to the harness (§2.4)"
(`03:442-443`) is true by RUN-3's pipes, not by SINK-2. Cite RUN-1 and
RUN-3.

### N11. Readability: RUN-3 and RUN-5 are overloaded

RUN-3 (`03:475-502`) now holds three concerns in one paragraph: process
setup (environment, session, descriptors, escaping), result integrity (the
result channel and early exit), and the OS sandbox with its level. Each
already has its own conformance sentences. Splitting out the result rules
as "RUN-3b. Results", or renumbering them, would make each rule
falsifiable on its own and give B2's pre-run report a home. RUN-5 mixes the
coverage definition with operating guidance (the no-VM permission, what
remains exposed, the session sandbox for builds, artifact sharing). Moving
that guidance to the "What RUN-1 does not cover" paragraph would leave
RUN-5 as the definition plus its output. Neither split is required.

## Escape attempts that failed

- **Edition downgrade to make `#[no_mangle]` safe.** In Rust 2021,
  `#[no_mangle]` compiles without `unsafe(..)`; I checked with rustc 1.96.
  But D18 fixes the edition at 2024 (`01:966`), RUN-2 lets the tree choose
  no rustc flags (`03:462-466`), and the spike driver rejects
  `--edition 2021` and adds `-F unsafe_code` (`tide-rustc:18-19`, `:32`).
  No route.
- **`#[link(name = ..)]` to pull a committed object into the test binary.**
  `#[link]` acts only on extern blocks: on a module, rustc 1.96 warns and
  links nothing. Extern blocks need `unsafe extern` in 2024 and the `ffi`
  grant (FFI-1, `03:838`). No route.
- **Proc macros of an imported Rust crate reading host files** (v1, for
  example a derive with a folder attribute). These run in RUST-5's sandbox
  (`03:1565-1569`), whose "read-only sources" does not say that nothing else
  is readable. It is hole 3 and listed, but RUST-5 should say the sandbox
  reads only the crate's sources and the candidate's tree.
- **Network beacon through registry fetches.** The candidate's lockfile
  makes the toolchain fetch named versions from the operator's registries
  (`03:445-448`). That tells a registry-side observer that the branch was
  built, but no code has run yet, so it leaks nothing secret.
- **The candidate controlling `.git/config`** (`core.fsmonitor`). This is
  outside the threat model: the candidate is "the branch's tree"
  (`03:427`), fetched into the operator's repository. If fleets run
  `tide review` in a delegate's own clone, it becomes a route; consider
  saying so.
- **Prompt injection of an agent orchestrator through test output.**
  RUN-1 bounds what code reaches, not what its output says, and the branch's
  source carries the same text statically. Companion §19 could add one line
  telling agents to treat test output as data.
- **Grants moved by renaming or shadowing a dependency.** A changed lockfile
  source or a new grant makes the candidate not covered (`03:549-556`).
- **Replay of a manifest the child wrote.** REPLAY-1's inline contents and
  REPLAY-2's identity check bound it (`03:1442-1450`).

## Consistency checks that passed

- The ADMIT-1 entry (`03:410-412`), D7 (`01:430`), hole 1 (`03:508-509`),
  and the RUN-4 row (`03:589`) agree on `global_asm!`: `FX0520` without a
  grant, a hole under one.
- `#[unsafe(..)]` in `reference.md`'s hole table (`reference.md:953`) matches
  hole 1.
- The tier row (`01:82`) puts the guarantee and the Linux sandbox in v0;
  hole tags v1 and later match `01:83-84`; the behavior diff is v1 in both
  D11 and RUN-5.
- RUN-1's use of ROOT-1, ROOT-3, and ROOT-5, and of ADMIT-1's runtime entry
  points, matches `03:217-241` and `03:413`. PANIC-1's child process and
  start records match RUN-3.
- RUST-5's sandbox for build scripts and proc macros (`03:1565-1569`)
  matches hole 3 and the closing paragraph.
- `reference.md`'s pointer (`reference.md:7-9`) still reaches every
  companion section. It cites 14.2 at `:550`, 17 at `:336` and elsewhere,
  18 at `:963`, 19 at `:966-967`, and Appendix A at `:1073`, and it names 15
  and 16.
- The README line and companion header match the chapter, apart from
  B2's missing pre-run condition.
