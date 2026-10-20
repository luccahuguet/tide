Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-20.

## Verdict

**approve — high confidence (0.96).**

B2 is closed, including the child’s ability to lower its soft limit. The fixes introduce no blocking contradiction, overclaim, or runtime startup requirement within this re-check’s scope.

## Scope

- HEAD: `87f9cd37f824b03d6eefac66e84850d02db9666f`
- Diff: `git diff -- design/ README.md`
- Diff SHA-256: `dd19e147cf3c22cf8896c6028fc40a0a0ddbe64ae3d7c1d37fbf40235815036b`

Read AGENTS.md, GOVERNANCE.md’s supervisor brief, `br --no-db show fx-arx`, rounds 64–66, the diff, and relevant surrounding design. Reviewed only B2’s closure and consequences of these fixes. Settled findings were not reopened.

## Item status

- **Round 64 B2 — closed.** The [startup clause](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:491) requires both limits at exactly one byte before exec, with an explained startup failure when that cannot be established.
- **Round 65 B1 — closed.** The [seccomp clause](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:517) refuses resource-limit changes, including setting `prlimit64` calls and raw `setrlimit`.
- **Conformance coverage — sufficient for this design change.** The [RUN-3 row](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:651) covers an inherited zero soft limit and a child attempting to lower its own limit.
- **Compatibility — no blocker found.** Necessary limit reads remain available; the harness can establish limits before installing seccomp.

## Blocking findings

None.

## Non-blocking notes

None within scope. No patch or alternative design is required.

## Reasoning

Linux initializes the core-dump limit from the soft `RLIMIT_CORE` value. Its pipe path rejects exactly one byte before starting the collector; its file path rejects a limit below the executable format’s minimum core size. Requiring both values at one therefore establishes the intended invariant. [Linux core-dump implementation](https://github.com/torvalds/linux/blob/v6.17/fs/coredump.c#L828-L942).

Resource limits survive exec and are shared by a process’s threads. An unprivileged harness cannot raise an inherited hard limit of zero, so refusing startup is consistent and deliberate. [Resource-limit semantics](https://man7.org/linux/man-pages/man2/getrlimit.2.html).

The seccomp exception is implementable as `new_limit == NULL`, inspecting the third argument without dereferencing memory. All non-null setting requests fail, preventing the child from replacing the one-byte sentinel with zero. Seccomp filters persist across exec and permitted thread creation. [Kernel seccomp documentation](https://docs.kernel.org/userspace-api/seccomp_filter.html).

The relevant runtime paths need reads. Glibc’s `getrlimit` uses `prlimit64` with a null new-limit pointer; its early pthread initialization reads `RLIMIT_STACK` and adjusts local thread attributes. Rust’s main-stack setup calls `pthread_getattr_np`, whose glibc implementation also reads that limit. These paths require no limit-setting exception. [Glibc limit reads](https://github.com/bminor/glibc/blob/master/sysdeps/unix/sysv/linux/getrlimit64.c), [pthread initialization](https://github.com/bminor/glibc/blob/master/sysdeps/nptl/pthread_early_init.h), [Rust stack setup](https://github.com/rust-lang/rust/blob/1.96.0/library/std/src/sys/pal/unix/stack_overflow.rs), [glibc stack query](https://github.com/bminor/glibc/blob/glibc-2.42/nptl/pthread_getattr_np.c).

On Linux 7.1.5 x86-64, bounded in-memory Python/ctypes probes confirmed that inherited soft zero can become `(1,1)`, queries pass, and raw `setrlimit`, setting `prlimit64`, and libc `setrlimit` return `EPERM` under the diagnostic filter. Ripgrep 15.2.0 started successfully after exec with that filter inherited. A separate child confirmed that hard zero prevents establishing `(1,1)`.

`git diff --check -- design/ README.md` passed. HEAD and diff identity remained unchanged; no files were modified.

Not checked: live core collection, other collector modes, or the future Tide runtime; `strace` was blocked by the environment.  
Residual risk: the diagnostic probes establish mechanism feasibility; implemented sandbox enforcement still requires conformance testing.