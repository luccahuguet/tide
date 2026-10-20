# Tide adversarial/consistency review, round 65: the core-size limit fix (fx-arx)

## Verdict

**request changes** (one blocking finding, text-only)

This is the narrow Opus confirmation round after Codex round 64, which
found B2: RUN-3 set only a hard `RLIMIT_CORE` of one byte, while Linux's
piped-core suppression tests the soft limit. The edit closes B2 as Codex
stated it. The harness now sets both limits to exactly one byte before
exec and fails to start otherwise, and the row starts the harness under a
soft limit of zero.

The same mechanism has one gap left, and it sits inside the child. A
process can lower its own soft limit to zero without privilege. RUN-3
never says that seccomp refuses changes to resource limits. The runtime
plausibly needs `prlimit64` to read limits, so an implementer following
"admits only the system calls the runtime needs" could admit it
unfiltered. The row does not test this case (B1).

## What was reviewed

The uncommitted working tree on 2026-10-20 against HEAD `87f9cd3`:
`git diff -- design/ README.md`, SHA-256
`22918860b57bf545925c894a2e3ec320060983176b967b905980cfc0f2c47692`.
`git diff --check` passes.

Scope: only the B2 fix. I read:

- RUN-3's core-size clause (`03:491-493`);
- RUN-3's seccomp bullet (`03:505-518`) and level and fail-closed text
  (`03:520-531`);
- the RUN-3 row (`03:646`);
- Codex round 64's B2 (`reviews/round64-codex.md:25-33`).

I checked it against:

- the kernel's `fs/coredump.c` at v6.17. `vfs_coredump` sets
  `.limit = rlimit(RLIMIT_CORE)`, which is the soft limit, and
  `coredump_pipe` aborts only when `cprm->limit == 1` ("RLIMIT_CORE is set
  to 1, aborting core");
- glibc's `sysdeps/unix/sysv/linux/getrlimit64.c`, where `getrlimit` is
  `prlimit64 (0, resource, NULL, rlimits)`;
- getrlimit(2): limits survive `execve`. An unprivileged process may lower
  its hard limit, irreversibly, and may set its soft limit anywhere from 0
  up to the hard limit.

A grep of `design/` and `README.md` for `dumpable`, `core dump`,
`core-size`, and `RLIMIT` finds only `03:492` and the row. No
"non-dumpable" wording survives anywhere.

## Round 64 closure

- **B2 (soft limit).** Closed. "with both soft and hard core-size limits at
  exactly one byte, which survive exec and stop core dumps to a file or a
  piped collector, or it fails to start" (`03:491-493`).
  - Soft = 1 meets `coredump_pipe`'s `limit == 1` sentinel.
  - Soft = 1 is below ELF's `min_coredump`, so `coredump_file` writes
    nothing.
  - Hard = 1 stops the child from raising the soft limit back past 1.
  - "Survive exec" is accurate for resource limits. The round 63 N1
    objection, which applied to dumpability, is gone.
  - "Or it fails to start" is the right fail-closed rule. The harness
    cannot raise an inherited hard limit of 0 to 1 without
    `CAP_SYS_RESOURCE`.
- **Row.** "and crash, on a host that pipes core dumps to a collector,
  with the harness started under a soft core-size limit of zero"
  (`03:646`). This is exactly the `(soft=0, hard=…)` configuration from
  Codex's concrete risk. It sits under "each fails or writes nothing
  outside the run".

## Consistency with the surrounding RUN-3 text

- The clause sits among the harness's start-up properties (environment,
  session, descriptors), all of which hold across exec. This now includes
  the limits, so the list is consistent.
- "Or it fails to start" agrees with "If the harness cannot install the
  level it reports for this host, it fails the run" (`03:527-528`).
- Limits are set by the harness before exec. Landlock and seccomp are
  installed "between fork and exec" (`03:498-499`). The order between the
  limits and seccomp is not stated, and it does not need to be: setting
  limits comes before `execve`, and the seccomp filter governs only the
  child after installation. An implementation must set the limits before
  it installs the filter, or the filter must admit the harness's own
  `prlimit64`. Either is ordinary.

## Blocking findings

### B1. The child can lower its own soft limit to zero; RUN-3 does not say seccomp refuses it

**Where.** The seccomp bullet (`03:505-518`) admits "only the system
calls the runtime needs". Its named refusals cover process creation,
namespaces, sockets, io_uring, keyrings, System V IPC, POSIX message
queues, extended-attribute and link reads, metadata changes, and tracing.
They do not include resource-limit changes. The row (`03:646`) crashes
only a child whose limits the harness set.

**Why it matters.** The mechanism holds only while the child's soft limit
stays at exactly 1:

- getrlimit(2) lets an unprivileged process lower its soft limit to 0, and
  0 is not the sentinel. A test or static constructor under an `unsafe`
  grant, which RUN-4 hole 1 says "can make raw system calls", could call
  `prlimit64(0, RLIMIT_CORE, {0, 1}, NULL)` and then crash. On a host with
  a piped collector the kernel would then pipe the full core, with
  candidate-controlled memory, outside the run. That is B2's exact risk,
  reached from inside the child.
- The allowlist refuses this only if `prlimit64` is not admitted, and the
  runtime plausibly needs it. In glibc, `getrlimit` is `prlimit64` with a
  null new limit. `pthread_getattr_np` reads `RLIMIT_STACK` for the main
  thread, and Rust's std calls it at start-up for its main-thread stack
  guard. That last point is inference from std's Linux guard code and was
  not traced on the pinned toolchain.
- An implementer who sees `prlimit64` fail at start-up and admits it whole
  satisfies every word of the bullet and reopens the hole. seccomp can
  inspect `prlimit64`'s new-limit pointer, so a precise rule costs nothing.
- `prctl` does not matter here. `PR_SET_DUMPABLE` can only suppress dumps
  further, and the limit governs the pipe path regardless.

**Minimal patch.**

1. In the seccomp bullet's list of refused calls (`03:514-518`), add
   resource-limit changes. For example, "... metadata changes (permissions,
   owners, extended attributes, times), resource-limit changes
   (`prlimit64` passes only with no new limit), and tracing."
2. In the RUN-3 row (`03:646`), extend the crash case: "and crash, ...
   with the harness started under a soft core-size limit of zero, and
   again after the test lowers its own soft core-size limit to zero".

## Non-blocking notes

### N1. `03:493` is unwrapped

`03:493` is 123 columns ("dumps to a file or a piped collector, or it
fails to start; and with no inherited descriptor but an empty standard
input,"). Rewrap the paragraph.

### N2. Hosts with a hard core limit of zero always fail to start

The CIS benchmarks recommend `* hard core 0`, and hardened hosts often
set it. Under that limit the harness cannot raise the hard limit to 1, so
every `tide test` and `tide sim` fails to start. Failing closed is correct
when `core_pattern` pipes, because a soft limit of 0 does not stop a
piped dump. When it does not pipe, a limit of 0 already suppresses file
dumps. No change is required, since failing closed is the safe reading.
Two optional improvements:

- the failure could name the cause ("hard core-size limit 0; raise it to
  1");
- the clause could say that this is deliberate, so that an implementer
  does not treat it as a bug and "fix" it by falling back to a soft limit
  of 0.

## Escape attempts that failed

These are limited to the B2 mechanism.

- **Harness inherits `(soft=0, hard=unlimited)`.** The harness sets both
  limits to 1 before exec (`03:491-493`), and the row tests this case
  (`03:646`).
- **Harness inherits `(soft=0, hard=0)`.** The harness cannot reach 1 and
  fails to start, so nothing runs (`03:493`).
- **Child raises its soft limit to unlimited.** The hard limit is 1, so
  the call fails with `EPERM` regardless of seccomp.
- **A static constructor crashes before `main`.** The limits are set
  before exec, so they hold from the first instruction.
- **Child lowers its soft limit to 0, then crashes.** This one succeeds
  unless seccomp refuses the change, and the text does not say that it
  does (B1).
