# Tide adversarial/consistency review, round 66: the resource-limit refusal (fx-arx)

## Verdict

**approve** (no blocking findings; two nits, neither of which blocks)

This is the narrow Opus confirmation round after round 65. That round found
B1: the child could lower its own soft core-size limit to zero and defeat
the one-byte sentinel that stops piped core dumps. It also raised N1, an
unwrapped line, and N2, hosts whose hard core limit is zero. The edit makes
seccomp refuse resource-limit changes, adds the self-lowering case to the
RUN-3 row, rewraps the clause, and has the harness say why when it cannot
set the limits. B1 and N2 are closed. N1 is closed for the 123-column line,
but the rewrap left a line of 83 columns (N1 below).

## What was reviewed

The uncommitted working tree on 2026-10-20 against HEAD `87f9cd3`:
`git diff -- design/ README.md`, SHA-256
`1717f00ffb241cc41a00067042566e66fcff03d96a586febb4f54edede69811b`.
`git diff --check` passes.

Scope: only the round 65 fixes. I read:

- RUN-3's start-up clause (`03:488-500`), including the core-size limits
  (`03:491-495`);
- the seccomp bullet (`03:507-522`), whose refusals are at `03:516-522`;
- the level and fail-closed text (`03:524-535`);
- the RUN-3 row (`03:650`);
- round 65's B1, N1, and N2 (`reviews/round65-consistency.md`).

I checked it against:

- the Linux syscall tables. x86-64 has `setrlimit` (160) and `prlimit64`
  (302). arm64 defines `__ARCH_WANT_SET_GET_RLIMIT`, so the generic table
  gives it `setrlimit` (164) as well as `prlimit64` (261);
- `prlimit64(pid, resource, new_limit, old_limit)`. Its new-limit pointer
  is the third argument, which a seccomp filter can compare with zero, and
  a null-pointer test has no time-of-check race;
- glibc's `sysdeps/unix/sysv/linux/setrlimit64.c` and `getrlimit64.c`.
  `setrlimit` is `prlimit64(0, resource, new, NULL)` and `getrlimit` is
  `prlimit64(0, resource, NULL, old)`. A test that calls libc's `setrlimit`
  therefore makes the same `prlimit64` call with a new limit that the rule
  refuses.

A grep of `design/` and `README.md` for `seccomp`, `core-size`,
`core dump`, `rlimit`, and `resource-limit` finds no other copy of the
seccomp list that needs the same change. `01:499` and `01:839` name only
"Landlock and seccomp", and `03:632` names them as fleet confinement.
`04-contracts.md`'s `rlimit` is the proof-obligation budget, which is
unrelated.

## Round 65 closure

- **B1 (child lowers its own soft limit).** Closed. The refused calls now
  include "resource-limit changes (`prlimit64` passes only when it sets no
  new limit)" (`03:520-521`).
  - On both v0 architectures, `setrlimit` is a resource-limit change and
    is not among the admitted families (`03:507-511`). It therefore fails
    both as a named refusal and under the allowlist default ("Every other
    call fails", `03:516`).
  - `prlimit64` with a new limit fails. With a null new limit, it is the
    read that glibc's `getrlimit` and Rust's `pthread_getattr_np` path need.
    This is exactly the case that round 65 said an implementer might admit
    whole.
  - The only reading of "sets no new limit" that seccomp can enforce is a
    null new-limit pointer, because a filter cannot dereference the
    argument to compare values. An implementer cannot admit "new equals
    old" by mistake.
- **Row.** "and again after the test tries to lower its own soft limit to
  zero" sits under "each fails or writes nothing outside the run"
  (`03:650`). The kernel lets an unprivileged process lower its soft limit
  from 1 to 0, so only the seccomp rule makes the attempt fail. The case
  therefore falsifies B1's fix directly. Via libc it exercises
  `prlimit64` with a new limit, which is the form the rule names.
- **N1 (unwrapped line).** The 123-column line is gone. One line is still
  83 columns (N1 below).
- **N2 (hard limit 0).** Closed. "or it fails to start, saying why (as on
  a host whose hard limit is below one byte, which the harness cannot
  raise)" (`03:493-495`) names the cause, as round 65's first optional
  improvement suggested. It also states the failure as a requirement, so
  an implementer has no room to fall back to a soft limit of 0.

## Consistency with the surrounding RUN-3 text

- The refusal sits with the other refused families (`03:516-522`), in the
  same form as "extended-attribute and link reads": a named family, with
  one parenthetical that says what passes.
- The admitted list (`03:507-511`) is introduced by "among them" and does
  not name limit reads. The parenthetical implies that a `prlimit64` read
  passes, and that reading does not conflict with any admitted or refused
  item.
- The rule also stops the child from changing the harness's limits through
  `prlimit64` with another process's ID. That would be possible without
  the rule, because the two run under the same credentials. A read of
  another process's limits still passes. It reveals only numbers that the
  toolchain sets, so it does not matter.
- Fail-closed: "or it fails to start, saying why" (`03:493`) agrees with
  "If the harness cannot install the level it reports for this host, it
  fails the run" (`03:531-532`). The new parenthetical is an example of
  that rule, not a second policy.

## Blocking findings

None.

## Non-blocking notes

### N1. `03:496` is 83 columns

"input, pipes for standard output and error, the result channel (RUN-6),
the run's" runs past the paragraph's wrap of about 76 columns. Rewrap
`03:494-500`.

### N2. "which the harness cannot raise" holds only without privilege

With `CAP_SYS_RESOURCE`, the harness could raise a hard limit of 0 to 1 and
would start the child. The fail-closed rule is right either way, and the
clause gives an example, not a condition. "which an unprivileged harness
cannot raise" would be exact. This is optional.

## Escape attempts that failed

These are limited to the round 65 mechanism.

- **Child calls libc `setrlimit(RLIMIT_CORE, {0, 1})`.** glibc issues
  `prlimit64` with a new limit, which is refused (`03:520-521`). The row
  tests this case (`03:650`).
- **Child issues the raw `setrlimit` syscall (x86-64 160, arm64 164).**
  It is a resource-limit change, and it is not admitted (`03:516-521`).
- **Child calls `prlimit64(harness_pid, RLIMIT_CORE, …)` to change the
  harness.** The call sets a new limit, so it is refused.
- **Child raises its soft limit.** The hard limit is 1, so the kernel
  returns `EPERM` regardless of seccomp.
- **Harness inherits a hard limit of 0.** The harness fails to start the
  child and says why (`03:493-495`).
