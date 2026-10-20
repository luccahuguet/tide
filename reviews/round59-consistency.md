# Tide adversarial/consistency review, round 59: the unreviewed-branch guarantee (fx-arx)

## Verdict

**request_changes**

Round 58's three blocking findings are closed. So are N1, N2, N4, N5, N6,
and N7. The answers on N3 (moot now), N8 (no v0 route), and N9 (budget) are
reasonable. No fix from rounds 54 to 57 was dropped. Refusing process
creation fits everything that runs in a v0 test or simulation binary. The
exception is the later-tier Bend supervisor (N5).

This round has three blocking findings, and each needs one clause:

- B1: the agent text says the guarantee covers "the toolchain commands"
  and names only `tide run` and built binaries as exceptions. `tide eval`
  builds and runs the branch's code with an authority no document
  specifies, and RUN-1 does not cover it.
- B2: RUN-5 says `tide review` "starts no process from the candidate's
  build". Four lines later, the same rule has v1's behavior diff run the
  candidate's seeds as RUN-3 processes.
- B3: the agent text tells agents to run `tide review <base>` but never
  says that `<base>` must be a reviewed revision. If an agent uses the
  branch's own parent commit, `run: covered` can come back for a branch
  whose earlier commits add an `unsafe` grant.

## What was reviewed

The uncommitted working tree on 2026-10-20 against HEAD `87f9cd3`:
`git diff -- design/ README.md`, SHA-256
`4c44a3653f06206475d382b9e81d6498343bca2f80209c3234583d31225fb4a6`.

The changed text:

- 03 §2.6 at `03:423-618`, the ADMIT-1 entry at `03:410-412`, and REPLAY-1
  at `03:1464-1472`;
- D7 at `01:420-431`, D11's behavior-diff bullet at `01:719-723`, the D13
  bullet at `01:830-860`, and the tier row at `01:82`;
- companion §19 at `companion:495-519` and its header at `companion:3-16`;
- `reference.md:5-9` and `:949-967`;
- `README.md:148-152`.

I checked it against:

- 01 D5 (`01:362-372`), D14 (`01:863-879`), D16 (`01:925-934`), and open
  question 5 (`01:1127-1128`);
- 03 §1.1 to §1.3 (`03:149-213`), PANIC-1 (`03:293-308`), TASK-5
  (`03:1030-1035`), §4.3 and §4.4 (`03:812-848`), §5.2 (`03:896-904`), §9
  (`03:1321-1505`), §10 (`03:1507-1560`), and §11 (`03:1562-1604`);
- 04 §2.1 (`04:511-520`);
- `reference.md` §11 to §13 (`:949-1070`);
- rounds 54 to 58.

I did not count tokens.

## Round 58 closure

- **B1 (Bend descriptor).** Closed. RUN-3's list now ends with "and, in the
  later tier, a connection to each Bend kernel server it needs (RUN-4,
  hole 4)" (`03:479-481`), which agrees with hole 4 (`03:533-536`).
- **B2 (metadata reads).** Closed. seccomp admits "`stat`-family and
  `access` calls" (`03:492-493`) and refuses "extended-attribute and link
  reads" (`03:500-501`). The RUN-3 row tests both (`03:615`). The stated
  residual is still "whether it exists, its size and times"
  (`03:504-505`), which `stat` exceeds a little (N2).
- **B3 (printed commands).** Closed. The harness builds every printed
  command "only from values it checks: a test name the build declared, an
  integer seed, and a path it chose" (`03:570-573`), and the RUN-6 row tests
  it (`03:618`). Test names are Rust paths, and Rust identifiers are
  XID-only, so a declared name holds no shell metacharacter. This follows
  from the Rust Reference's identifier grammar, which is not quoted here.
- **N1 (System V IPC, namespaces).** Closed. "System V IPC (shared memory,
  semaphores, and message queues)" (`03:499-500`), "namespaces"
  (`03:498`), and the row's "System V shared memory" (`03:615`).
- **N2 (implementability).** Closed. The list says "among them" and adds
  randomness, polling, signal handling within the process, and closes
  (`03:489-492`). `clone3` returns `ENOSYS`, so the runtime falls back to an
  inspectable `clone` (`03:493-495`). See N3 for the `execve` sentence.
- **N3 (descendants).** Moot, as answered: process creation is refused
  (`03:497`), and the row forks (`03:615`).
- **N4 (escaping).** Closed: "control characters (C0 and C1) and
  bidirectional overrides in everything it prints, to a terminal, pipe, or
  log ... file names included" (`03:573-577`). The row covers a pty and a
  log (`03:618`).
- **N5 (VM for kernel bugs).** Closed: "which confines a kernel bug only if
  it is a VM" (`03:589`).
- **N6 (summaries).** Closed in D13 (`01:850-851`) and companion §19
  (`companion:514-515`).
- **N7 (Landlock-less kernel).** Closed. The row now expects `partial`,
  naming Landlock (`03:615`), which agrees with RUN-3's definitions
  (`03:509-510`).
- **N8 (features).** Accepted. No v0 build sets a feature cfg.
- **N9 (core states only `run:`).** Accepted for budget. Note that B3 also
  touches the core line (`reference.md:965-967`).

## Earlier fixes retained (rounds 54 to 57)

- **R54.** The package-wide clause (`03:559-563`) and the descendant row
  (`03:617`). ADMIT-1 cited in RUN-1 (`03:436`). The build in the session
  sandbox (`03:589-591`).
- **R55.** Symbolic links (`03:458-463`, row `03:614`). Lockfile identity
  (`03:560`). Operator registries (`03:446`, `03:471`). One code for
  `global_asm!` (`03:410-412`, `03:518`). Unshared artifacts
  (`03:590-591`). The harness and derives in hole 5 (`03:537-543`). Side
  channels (`03:550`). Inline manifests (`03:1470-1472`).
- **R56.** The transitive clause (`03:561-563`), the Rust crate and Bend
  clause (`03:557-558`), the pipes and new session (`03:476-479`), and
  compile-time-only review (`03:551-553`; see B2).
- **R57.** Allowlists with "Every other call fails" (`03:484-502`). The
  pre-run sandbox report (`03:507-508`). The RUN-5 covered row (`03:617`).
  The behavior-diff condition (`03:563-566`, `01:721-723`). The exposure
  sorting (`03:581-591`). Core dumps (`03:477`). Tests never started
  (`03:570`). The kernel in hole 5 (`03:543-544`). Regular files and
  directories, no links (`03:486-487`). "Refused (§9.8)" (`03:506-507`).

## Blocking findings

### B1. `tide eval` runs the branch's code, but the agent text implies the guarantee covers it

**Where.** Companion §19: "This covers the toolchain commands only: `tide
run`, a built binary, and anything else the branch asks you to execute still
need your session's sandbox" (`companion:517-519`). 03 names the same two
exceptions (`03:595-597`). RUN-1 covers `tide check`, `tide test`,
`tide sim`, and `tide sim --replay` (`03:430-431`).

**Problem.** `tide eval` "runs an expression against the current package,
through a scratch crate built by rustc" (`01:872-873`). The core lists it
among the toolchain commands an agent uses (`reference.md:1063`). No
document says what `Root` the scratch crate gets, whether it runs under
RUN-3, or whether it applies RUN-2's environment. Its design is open
(`01:1127-1128`). An agent reading §19 counts `tide eval` as one of "the
toolchain commands" and runs it outside any VM. Suppose the scratch crate's
entry builds a real `Root`, as `#[tide::main]` does (`03:191-193`). Then
`tide eval 'pkg::run(root)'` runs the branch's ungranted code with host
authority, even on a covered branch. `tide verify` (v1) runs a solver and
spec evaluation over the branch's code, and it is not covered either.

**Evidence vs. inference.** Evidence: the quoted lines and the open
question. Inference: that an implementation gives eval a real root. The
text neither requires nor forbids it, and that silence is the overclaim.

**Smallest fix.** In companion §19, write "This covers `tide check`,
`tide test`, `tide sim`, and `tide review` only: `tide run`, `tide eval`, a
built binary, ...". In 03's "What RUN-1 does not cover" (`03:595-597`),
add "`tide eval`" next to `tide run`. Alternatively, put `tide eval` in
RUN-1 under RUN-3 with a simulated root, but that decides an open question.

### B2. RUN-5 says `tide review` starts no process from the candidate's build, then has the behavior diff start one

**Where.** "`tide review <base>` runs the candidate's code only at compile
time (RUN-1) and starts no process from the candidate's build"
(`03:551-553`). In the same rule: "v1's behavior diff (D11), which runs the
candidate's seeds under RUN-3, runs only for a covered candidate on a host
with `sandbox: full`, unless the operator asks for it" (`03:563-566`). The
row repeats the tension: "An access trace of `tide review` shows no process
started from the candidate's build, and no behavior-diff run for a
candidate not covered" (`03:617`). D11 has `tide review` rerun "the same
simulation seeds on base and head" (`01:719-723`).

**Problem.** From v1 on, the first sentence is false for every covered
candidate on a `full` host, and for any candidate when the operator asks.
The first sentence is the one an operator relies on to run `tide review` on
any branch. Round 58 closed round 57's B4 without catching this. The
behavior is safe, since it runs only where testing is already allowed. The
statement contradicts it.

**Smallest fix.** Write "... runs the candidate's code only at compile time
(RUN-1), and starts no process from the candidate's build except the
behavior diff's RUN-3 children below". In the row, write "shows no process
started from the candidate's build for a candidate not covered".

### B3. The agent text never says that `<base>` must be reviewed

**Where.** Companion §19: "Before testing another agent's branch without a
VM of its own, run `tide review <base>`" (`companion:502-503`). Then: "only
when it prints `run: covered` and `sandbox: full`" (`companion:511-512`).
The core says: "only if `tide review` prints `run: covered`"
(`reference.md:965-967`). Only 03 defines the base, as "a reviewed
revision, such as the protected branch it will merge into" (`03:427-428`),
and D13 says "relative to a reviewed base" (`01:846`).

**Problem.** RUN-5 compares the candidate against `<base>` (`03:556-563`),
so coverage is only as good as the base. Suppose a branch's first commit
grants `unsafe` to a new in-tree package and adds an `unsafe` block, and
its second commit edits only ungranted code. `tide review HEAD~1` prints
`run: covered`, because nothing changed against that base. Agents commonly
diff against the parent commit or the delegate's starting point. Nothing in
the text an agent loads tells them not to. The result is `run: covered` on
a branch that reaches an unreviewed hole, through the procedure the
reference teaches.

**Smallest fix.** In companion §19, write "run `tide review <base>`, where
`<base>` is a reviewed revision, such as the branch it will merge into, and
never a commit of the branch itself". In the core line, write "prints `run:
covered` against a reviewed base". Optionally, RUN-5 could print the base
revision it compared against, so a log shows it.

## Non-blocking notes

### N1. Nothing binds the review's verdict to the tree that `tide test` builds

The guidance says to run tests "for a covered candidate" (`03:581-583`). No
rule makes `tide test` check that the tree it builds is the one
`tide review` judged. In a fleet, the orchestrator often tests in the
delegate's worktree, and the delegate may keep committing between the two
commands. D14 already gives every response a snapshot hash (`01:877`). Have
the guidance and §19 say to compare the two hashes, or let
`tide test --covered <base>` refuse a tree that is not covered.

### N2. `stat` returns more than the stated residual

The residual reads "whether it exists, its size and times" (`03:504-505`).
`stat` also returns the file type, mode, owner, inode, and link count.
Landlock does not restrict `O_PATH` opens, as I infer from its `f_mode`
check (`O_PATH` sets neither read nor write), so an `O_PATH` descriptor
yields the same data through `fstat`, and no more. Write "its type, size,
owner, mode, and times".

### N3. "`execve` succeeds only for ... its own binary" contradicts the Landlock clause

Landlock lets the child read and execute "the binary and the libraries it
loads" (`03:485-486`). The kernel needs the execute right on the ELF
interpreter, and that is a library. So `execve` also succeeds on `ld.so`
and on libraries, and `ld.so` as a program can map an ELF from the output
directory. It gives no escape: the sandbox persists across exec, and
calling `execve` already needs hole 1 or 5, which gives native code (round
58). The sentence is still inaccurate. Write "succeeds only for files
Landlock lets the child execute (its binary and its loader and libraries),
so the child starts no other process".

### N4. The sandbox does not fail closed

`tide review` reports the level "before anything runs", and every run
result repeats it (`03:507-510`). If the harness later installs less than
it reported, for example because a ruleset call fails, the child still
runs, and the result says so only afterwards. That is round 57's B2 at
install time. Add: "If the harness cannot install the level it reports, it
does not start the child, unless the operator asks."

### N5. The Bend supervisor starts and restarts servers, but the child may not

D16 runs kernels "as a supervised child process" (`01:931`). §10 has "the
supervisor" run up to `budget` servers, and a cancel "restarts the server"
(`03:1526-1530`). Under RUN-3, the child starts no process (`03:496-497`),
and hole 4 has the harness start the server (`03:533-536`). That is
consistent only if the harness also restarts servers for the child, and
supplies the `stderr_tail` of `BendFault::Crashed` (`03:1542`). The rule
is later-tier. Say in hole 4 that the harness owns the servers' life cycle
under RUN-3.

### N6. RUN-3's channels leave no path for `--replay` input, and the output directory path collides with RUN-2

- `--replay` reads a manifest (`03:441-442`), and `--faults <plan.toml>`
  reads a plan (`03:1442`). The child can open only its binary, its
  libraries, and this run's output directory (`03:485-487`). It inherits
  only the listed descriptors, with an empty standard input (`03:477-481`).
  Say that the harness reads these inputs and places them in the run's
  output directory, or passes them through a listed channel.
- §9.6's replay path is stable: `target/sim/<test>-<seed>.manifest.json`
  (`03:1456`). RUN-2 fails a run "if anything already holds" the output
  directory's path (`03:461-463`). Under both rules, a second run fails.
  Say whether each run uses its own directory, or whether the harness
  replaces one it created itself.

### N7. Plain-text spoofing of the verdict

RUN-6 escapes control characters (`03:573-577`). Branch text still reaches
`tide review`'s lines as plain text, for example a Bend kernel file named
`run: covered    sandbox: full.bend` in a reason line, or a string literal
in a `reaches:` line (`03:818-822`). A consumer that greps the output for
`run: covered` is fooled, though the first line still says `not covered`.
Say in RUN-5 and §19 that the verdict is the first line or the `--json`
field, and that later lines quote the branch.

### N8. Forged traces mislead v1's behavior diff, not only results

RUN-6 limits "a pass" to "evidence only up to those holes" (`03:577-579`).
The behavior diff compares traces the child writes (`01:719-723`,
`03:1460-1462`), so hole-1 or hole-5 code can make the head's trace match
the base's. Extend the caveat: "a pass, a trace, or a behavior diff".

### N9. Small wording

- The phrase "tests and seeded simulations run in-process" (`03:599`) sits
  beside RUN-3's "as a child process" (`03:475`). Write "in one sandboxed
  test process".
- D13's "refuse ... IPC" (`01:840`) is broader than RUN-3. The child does
  use pipes. Write "System V and POSIX message-queue IPC" or drop it.
- Hole 5 names "the compiler" (`03:540`), but the system linker also reads
  the branch's objects at build time. Name it, or say "the compiler and
  linker".
- The core's hole table (`reference.md:951`) lists `#[unsafe(..)]` but not
  `global_asm!`. `FX0520` reports it anyway, so this is a budget call.

## Escape attempts that failed

- **Signals to the harness through `F_SETOWN`/`SIGIO` on an inherited
  pipe.** This needs hole 1 or 5 to reach `fcntl`. Landlock's signal scope
  (`03:487-488`) also covers SIGIO delivery, by inference from its
  file-owner hooks in ABI 6, and an older kernel reports `partial`.
- **`O_PATH` opens of host paths.** These yield only metadata (N2). Opens
  relative to the descriptor are checked by Landlock again. `readlinkat`,
  `fgetxattr`, `linkat`, and `execveat` fail under seccomp (`03:492-502`).
- **Planting dotfiles in the output directory** (`.cargo/config.toml`,
  `.envrc`, a `.git` gitfile). This needs hole 1 or 5, and it affects only a
  tool later started with its working directory inside `target/sim`. Links
  and FIFOs are refused (`03:486-487`).
- **A replay manifest committed in the branch.** REPLAY-2 compares the
  current build identity with the manifest's and does not adopt it
  (`03:1473-1477`). The replay runs under RUN-3 (`03:430-431`).
- **Shell metacharacters in test names.** Declared names are Rust paths
  (B3 closure), and child-written names never reach a printed command
  (`03:570-573`).
- **The real capability implementation linked into test builds**
  (`03:211-213`). Safe code cannot reach the selector. ADMIT-1 bars the
  minting entry (`03:413`). A bug here is hole 5.
- **libtest's subprocess mode.** libtest forks per test only under
  `panic=abort`. Tide panics unwind (`01:362-372`), so tests run on threads
  inside the child. This is inference from libtest's behavior.
- **Manifest edits to a granted package.** Dependency changes hit the
  transitive lockfile clause (`03:560-563`). Lowering contract use needs
  approval, and `assert` is the floor (`04:511-520`). Choosing a granted
  package as allocator, or changing the runtime, uses a reviewed
  abstraction (hole 1, `03:521-523`).
- **Generic trait impls fed to base `unsafe` code.** If the base relies on
  them, it is an unsound reviewed abstraction (hole 1).

## Process creation versus what runs inside the child

- The simulator's baton-passed OS threads (`03:1326`) use `clone` with
  thread flags (`03:493-495`).
- PANIC-1's reruns and fresh children are started by the harness, not the
  child (`03:300-305`).
- `sim.world.proc` holds models. "Real child processes" are not simulated
  (`03:1338`, `03:1504-1505`).
- TASK-5's 100 ms watchdog belongs to the evented runtime (v1). The
  simulator counts steps instead (`03:1033-1035`). Neither starts a
  process.
- The `blocking` and `threads` runtimes make OS threads only (`03:198-200`).
- An imported Rust crate's `fork`/`execve` (RUST-3, `03:1586`) is refused
  by seccomp in the child. That agrees with hole 2's "unless the sandbox
  refuses it" (`03:525-526`) and with §9.8's `EPERM` for unmodeled calls
  (`03:1497-1498`).
- The exceptions are `tide eval` (B1) and the Bend supervisor (N5).

## Consistency checks that passed

- `global_asm!` reads the same in ADMIT-1 (`03:410-412`), D7
  (`01:430`), hole 1 (`03:518`), the RUN-4 row (`03:616`), and D13
  (`01:842`).
- The tier row (`01:82`), D13's "(v0)" (`01:830`), and the README
  (`README.md:150-151`) agree. Holes 2 to 4 and the behavior diff carry the
  same tiers as `01:83-84`.
- RUN-1's sandbox clauses match D13 (`01:836-840`) and companion §19
  (`companion:500-502`).
- §10's loopback-TCP server (`03:1531`) is compatible with hole 4 if the
  harness makes the connection.
- `reference.md:958-959` cites 12.4 for declared authority, and 12.4
  defines it (`reference.md:1042-1043`).
- The companion header's "17–19" (`companion:10`) and the core pointer
  (`reference.md:8-9`) agree.
