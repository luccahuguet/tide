# Tide adversarial/consistency review, round 63: the unreviewed-branch guarantee (fx-arx)

## Verdict

**approve**

This is the narrow Opus confirmation round after Codex round 62, which
found B1 (assembler `.include` and `.incbin` read host files outside the
file loader) and made a non-blocking note on piped core dumps. The edits
close B1. The new build sandbox bounds every read the backend makes,
however the directive was built. It agrees with RUN-1, RUN-4 to RUN-6, the
guidance, the complement, D12, D13, D18 and ADMIT-2, and companion §19.
Nothing elsewhere still says that build-time compiler bugs meet only the
session's sandbox.

The core-dump note is closed in outcome, but the mechanism named is one
that `execve` undoes if the harness applies it before exec (N1). Three
smaller notes follow, and none blocks. They cover the build sandbox's
stated read and write set, its level and fail-closed rule, and RUST-5's
proc macros.

## What was reviewed

The uncommitted working tree on 2026-10-20 against HEAD `87f9cd3`:
`git diff -- design/ README.md`, SHA-256
`11db1bec358fa5e796550b900195c1b4c5340bdacd6403f001bc86dc5191cf5b`.
`git diff --check` passes.

I read the changed text:

- the new RUN-2 bullet (`03:460-467`);
- RUN-3's non-dumpable clause (`03:487-489`);
- the operating guidance (`03:605-618`);
- the RUN-2 and RUN-3 rows (`03:640`, `03:641`);
- D13's sentence (`01:834-837`).

I checked it against:

- RUN-1 (`03:431-449`), RUN-2's other bullets (`03:450-484`), RUN-3's
  level clause (`03:520-525`), RUN-4 holes 3 to 5 (`03:542-563`), RUN-5's
  first line (`03:568-578`), and the complement (`03:620-631`);
- ADMIT-2 (`03:417-421`), D12's backend and cache (`01:779-782`), and
  D18's driver bullets (`01:985-997`);
- RUST-5 (`03:1619-1623`) and companion §19 (`companion:496-523`);
- the core's §11 pointer (`reference.md:965-967`) and README (`README.md:150-151`);
- the execve(2) man page (dumpability across exec) and the kernel's
  `fs/coredump.c` (`coredump_pipe`'s `limit == 1` check and
  `coredump_file`'s `min_coredump` check).

I did not count tokens.

## Round 62 closure

- **B1 (assembler file reads).** Closed. "rustc, with its LLVM backend, and
  the linker run in a build sandbox, Landlock and seccomp on Linux as in
  RUN-3: they may read only the package's source directories, the
  toolchain, and the artifact cache, write only the cache, and create no
  socket or other process. So a read the file loader never sees, such as an
  assembler `.include` or `.incbin` in granted `unsafe` assembly, cannot
  reach outside them" (`03:460-465`).
  - Landlock denies at `open`, so the check happens before the backend
    reads any byte. Codex required that, and a substring filter could not
    give it.
  - A directive built by an assembler macro meets the same check. The row
    now covers that case: "written directly and built by an assembler
    macro: the build opens nothing outside its allowed set, and the
    canary's bytes appear in no diagnostic, artifact, or cache entry"
    (`03:640`).
  - The claim does not overreach on a weaker host: "without `full` the
    build's reads rest on the loader alone" (`03:466-467`). Codex's
    rejected alternative was printing only `not covered`. The text instead
    reports the real level.
- **Piped core dumps (note).** Closed in outcome, though the mechanism is
  the wrong one (N1). The child is "non-dumpable, so no core dump reaches a
  file or a piped collector" (`03:488-489`). The row crashes the child "on
  a host that pipes core dumps to a collector" (`03:641`).

## Consistency with the rest of §2.6 and the decision record

- **Implementability with a `rustc_private` driver.** rustc runs inside
  the driver's process (D18, `01:985-987`; ADMIT-2, `03:417`), so the
  build sandbox necessarily confines the driver too. That is consistent:
  - the parsers and fetcher run before the build. The guidance keeps their
    bugs in the session's sandbox (`03:613-615`), and RUN-1 has the
    toolchain fetch, never the build (`03:447-449`);
  - the sysroot (Tide's std, `--extern std=`, `01:974-976`) lies under
    "the toolchain";
  - D12's content-addressed cache (`01:781-782`) is the one writable
    place, which matches the D18 bullet "keeps its own content-addressed
    cache" (`01:992-997`);
  - the snapshot hash (`03:573-578`) is computed over bytes the loader
    serves, inside the sandbox.

  Installing the sandbox between fork and exec of the driver, as RUN-3
  does for test children, covers rustc's threads from the start. The set
  needs naming in two places (N2).
- **"Its level is reported with RUN-3's" and RUN-5.** These do not
  conflict. Both sandboxes use the same mechanisms on the same kernel, so
  a single level stands for both. RUN-5's first line, "the sandbox level
  this host's runs get (RUN-3)" (`03:572`), can carry it. RUN-3's
  fail-closed sentence (`03:523-524`) speaks only for the harness (N3).
- **No surviving "only the session's sandbox" for build-time compiler
  bugs.** The guidance now says "compiler bugs exploited at build time
  meet the build sandbox (RUN-2), which bounds their file and network
  access but not the code they emit" (`03:611-613`). What meets only the
  session's sandbox is now just "bugs in the harness, the parsers, the
  fetcher, or the kernel, and resource use" (`03:613-615`). Hole 5
  (`03:552-563`) is unchanged, which is correct, because the build sandbox
  bounds where a compiler bug reaches and not whether RUN-1 holds.
  - Other text that mentions a sandbox was also checked: D13
    ("inside which the build itself runs", `01:853-854`); companion §19's
    "your session's sandbox" (`companion:523`), which concerns `tide run`,
    `tide eval`, and built binaries; and README (`README.md:150-151`).
    None of them assigns build-time compiler bugs.
- **The guidance's "So".** "So the build, `tide check` and `tide review`
  included, also runs inside the session's sandbox" (`03:615-616`) still
  follows, because the parsers and fetcher run as part of those commands.
- **D13** (`01:834-837`): "in a build sandbox that also bounds assembler
  file reads". This matches RUN-2.
- **RUN-1's toolchain writes** (`03:445-447`). The build writes only the
  cache. The harness, outside the build, writes the output directory and
  the `target/sim/` copies. These agree.

## Earlier fixes retained (rounds 54 to 61)

Spot-checked against round 61's list. Everything is still present:

- symbolic links resolve beneath the root (`03:468-471`);
- the output directory and copies (`03:471-474`, row `03:640`);
- `global_asm!` gets one code (`03:410-412`, `03:533-534`);
- `execve` and `clone3` (`03:506-510`);
- the pre-run level (`03:520-522`);
- the reviewed base and the snapshot gate (`03:574-578`, row `03:643`);
- the behavior-diff condition (`03:587-589`);
- printed commands (`03:593-596`);
- hole 5's punctuation rewrite (`03:555-560`).

## Blocking findings

None.

## Non-blocking notes

### N1. `execve` resets dumpability; name a mechanism that survives it

RUN-3 lists "non-dumpable" among the properties the harness starts the
child with (`03:487-489`), next to the environment, session, and
descriptors, all of which survive exec. Dumpability does not survive. The
execve(2) man page says the process's "dumpable" attribute "is set to the
value 1, unless a set-user-ID program, a set-group-ID program, or a program
with capabilities is being executed". `PR_SET_DUMPABLE` set between fork and
exec is therefore lost before the static constructors that the next
sentence covers (`03:494-496`).

The row (`03:641`) tests the outcome and would catch this, so the outcome
stands. The wording invites the broken implementation, though. A
mechanism that holds from the first instruction is a hard `RLIMIT_CORE`
of 1, set before exec. The kernel's `coredump_pipe` aborts a piped dump
when the limit is 1 ("RLIMIT_CORE is set to 1, aborting core"), and
`coredump_file` writes nothing below the format's `min_coredump`. seccomp
already refuses `prctl` and `prlimit64` changes, because they are not on
its list (`03:502-506`).

Suggested text: "non-dumpable from its first instruction (on Linux, a hard
core limit of 1 set before exec, which the kernel honors for files and
piped collectors), so no core dump ...". If the pinned kernel's
core-socket mode (`@` in `core_pattern`) ignores the limit, the row's
collector host should include that mode too.

### N2. Name the build sandbox's read and write set exactly

The set "the package's source directories, the toolchain, and the
artifact cache; write only the cache" (`03:462-463`) needs three readings
that the text leaves implicit:

- The linker links against the host's C library and start files. RUN-3's
  child loads host libraries (`03:498-499`), so these are host paths
  outside the toolchain unless "the toolchain" includes the system linker
  and libraries it uses. Say so.
- rustc creates temporary files and directories under `TMPDIR`, and so do
  the linker driver and its archive handling. "Write only the cache"
  requires the toolchain-set environment (`03:475-477`) to point `TMPDIR`
  into the cache. One clause would fix this.
- "Create no socket or other process" (`03:463`) must allow rustc to
  start the toolchain's linker (and `cc` its `collect2` and `ld`). Read
  literally, it forbids that. "Start no program but the toolchain's linker"
  says what is meant, and mirrors RUN-3's `execve` rule (`03:508-510`).

Granted assembly can also `.incbin` any cache entry, including artifacts of
other projects built in the same session. That stays within the session
and inside the build sandbox. Limiting reads to the cache entries this
build's lockfile names would close it, but the change is optional.

### N3. One level for build and runs, and fail closed for both

"Its level is reported with RUN-3's" (`03:466`) is fine if it means one
level. Three small edits would make that explicit:

- RUN-5's "the sandbox level this host's runs get (RUN-3)" (`03:572`)
  could read "this host's builds and runs get (RUN-2, RUN-3)".
- RUN-3's "If the harness cannot install the level it reports for this
  host, it fails the run" (`03:523-524`) has no build counterpart. Add
  "and the driver fails the build" to RUN-2's bullet.
- `tide check` builds but produces no run result. RUN-3 reports levels
  only in `tide review` and run results (`03:520-522`), so `tide check`
  reports no build level. Codegen-free checking never invokes the
  assembler, so only the loader bullet matters there. A clause would
  still make the reporting complete.

The RUN-2 row's assembly case (`03:640`) has no host qualifier. Compare
the RUN-3 row's "the result says `sandbox: full`" (`03:641`). Adding "on a
host with `sandbox: full`" would keep the row from failing on a
Landlock-less kernel, where RUN-2 promises only the loader.

### N4. RUST-5's proc macros (v1) run inside rustc, so in the build sandbox

RUST-5 says "the crate's proc macros run in the same sandbox" as build
scripts (`03:1623`), and hole 3 repeats it (`03:542-544`). Proc macros are
loaded into rustc's process, so they now run in RUN-2's build sandbox,
which can write the cache. RUST-5's sandbox has "a writable output
directory only" (`03:1620-1621`). This is v1 and predates the diff. The
new bullet makes the mismatch visible, though: either proc macros run out
of process, or RUST-5 says they meet the build sandbox. Worth a line when
v1 is drafted. Nothing is needed now.

### N5. Wording

- `03:616` is still unwrapped (96 columns). Round 61's N5 flagged it.
- `03:489` ends short ("piped collector; and with no"). Rewrap the
  paragraph.

## Escape attempts that failed

These are limited to what the edits could introduce.

- **A `.incbin` path built by an assembler macro, or reached through a
  committed symbolic link in a source directory.** Landlock checks the
  resolved object at `open`, so a link to `/etc` reads nothing
  (`03:460-465`, row `03:640`).
- **Granted assembly that `.incbin`s a session credential under the
  home directory.** The path is outside the read set and is denied.
- **A compiler bug at build time that opens a socket or execs a shell.**
  The build may create no socket and no process beyond the linker
  (`03:463`, N2). The guidance states the remaining reach: the code it
  emits, which RUN-3 then confines (`03:611-613`).
- **A crash that sends candidate-controlled memory to a piped collector.**
  The outcome is required and tested (`03:488-489`, `03:641`). N1 gives
  the mechanism that achieves it.
