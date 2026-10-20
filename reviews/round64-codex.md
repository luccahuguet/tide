Model: `gpt-6.1-sol`. Effort: `xhigh`. Date: 2026-10-20.

## Verdict

**request_changes — high confidence (0.93).**

B1 is closed. The core-dump fix needs one precise correction: Linux’s piped-core suppression tests the **soft** limit, whereas RUN-3 specifies only the hard limit.

## Scope

- HEAD: `87f9cd37f824b03d6eefac66e84850d02db9666f`
- Diff: `git diff -- design/ README.md`
- Diff SHA-256: `b57ad1c31cb1354dc12d30a317e52d85c56d1372df0e77c766afdb279eb5d02d`

Read AGENTS.md, GOVERNANCE.md’s supervisor brief, `br --no-db show fx-arx`, rounds 62–63, the diff, and relevant surrounding design. This re-check covers only the requested fixes and their consequences. HEAD and diff identity remained unchanged.

## Item status

- **Round 62 B1 — closed.** RUN-2 now confines rustc, LLVM, and the linker at the OS file-open boundary. Granted assembly, including directives constructed by assembler macros, meets that boundary. The conformance row checks both access and leaked canary bytes.
- **Piped-core note — partially closed.** The collector-host conformance case is present, and resource limits survive exec. The specified hard limit alone does not establish the claimed suppression; B2 below remains.
- **Consistency and feasibility.** The host-library allowance, cache-local temporary files, linker exception, shared reporting level, and build failure rule resolve round 63’s corresponding concerns. No other blocking contradiction was found within scope.

## Blocking findings

### B2. A hard core limit of one byte does not necessarily suppress piped dumps

**Where:** [design/03-authority-concurrency.md:491](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:491).

Linux initializes the dump limit from `rlimit(RLIMIT_CORE)`, which reads `rlim_cur`, the soft limit. Its pipe path suppresses collection only when that value equals **exactly 1**; otherwise it replaces the limit with infinity. The hard limit merely bounds the soft limit. [Linux limit accessor](https://github.com/torvalds/linux/blob/v6.17/include/linux/sched/signal.h#L703-L716), [Linux core-dump implementation](https://github.com/torvalds/linux/blob/v6.17/fs/coredump.c#L917-L942).

**Concrete risk:** An inherited soft limit of zero can remain zero when the harness sets the hard limit to one. That legal `(soft=0, hard=1)` configuration satisfies the stated hard-limit requirement but permits a crash to invoke a piped collector and transmit the core outside the run. The review environment itself starts with `(soft=0, hard=unlimited)`. [Resource-limit semantics](https://man7.org/linux/man-pages/man2/getrlimit.2.html).

**Minimal patch:** Require **both soft and hard `RLIMIT_CORE` limits to equal one byte before exec**, failing startup if that cannot be established. Retain the existing denial of calls that could change them. In the RUN-3 conformance row at [line 646](/home/lucca/pjs/lab/tide/design/03-authority-concurrency.md:646), exercise the collector case with the harness initially inheriting a zero soft limit.

## Non-blocking notes

None within this re-check’s scope.

## Reasoning

The build sandbox addresses B1’s shared cause: backend reads no longer depend on passing through rustc’s source loader. Landlock checks file-opening permissions, so `.include` and `.incbin` cannot bypass the allowed read set by changing directive syntax. Cache reads are explicitly admitted, and weaker hosts explicitly lack this protection. [Landlock filesystem rules](https://docs.kernel.org/userspace-api/landlock.html#filesystem-flags).

The revised build policy is implementable in principle and agrees with the operating guidance and D13. It confines compiler access while preserving the stated possibility of malicious emitted code.

The core-limit approach is also implementable. Unlike dumpability, resource limits survive exec; setting both values to one supplies the pipe sentinel and prevents ordinary ELF core files. B2 corrects the mechanism without changing the chosen design. [Resource-limit inheritance](https://man7.org/linux/man-pages/man2/getrlimit.2.html), [Linux core-file check](https://github.com/torvalds/linux/blob/v6.17/fs/coredump.c#L828-L837).

`git diff --check -- design/ README.md` passed.

Not checked: live sandbox or collector execution, other collector modes, or token counts; no files were modified.  
Residual risk: enforcement remains unproven until the implemented toolchain passes conformance.