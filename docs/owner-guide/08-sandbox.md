# 8. The RUN-3 sandbox

## The idea

Chapters 1–3 are about what the **type system** lets code do. Chapter 3
showed it has holes: rustc's soundness bugs. So Tide adds a second, separate
fence that doesn't depend on rustc at all: the **operating system kernel**.

A **sandbox** is a set of limits a program runs under, enforced by the
kernel, the core of the operating system that every file read and network
connection must go through. Linux offers two tools for it:

- **Landlock** restricts which files and directories a process may read,
  write, or execute;
- **seccomp** restricts which **system calls** (the requests a program makes
  to the kernel, such as "open a socket" or "start a process") it may make at
  all. A forbidden one fails with "operation not permitted" (`EPERM`); a
  file Landlock forbids fails with "permission denied" (`EACCES`).

Neither can be lifted by the process once installed, and neither cares what
language the program was written in or what tricks it played on its
compiler.

**RUN-3** is the design rule for test and simulation runs (`design/03` §2.6).
When the driver runs a test (`--tide-run-test`, `--replay`), it starts the
test program as a **child process** and, just before the child starts,
installs:

- **Landlock:** the child may read and execute only its own binary copy and
  the system libraries it loads, read the run's context file the driver
  hands it, and write only inside its run's private `sim/` directory. Reading
  or writing any other file's contents, or running any other program, is
  denied. Landlock does not hide the file system itself: the child can still
  ask whether a path exists and read its metadata (size, dates), subject to
  ordinary permissions, as the Linux documentation states.
- **seccomp:** no sockets (so no network at all), no starting other programs,
  no `io_uring`, no tracing other processes, no changing file links or
  permissions.
- **Isolation:** a cleared environment, no inherited open files except
  empty stdin and pipes for its output, its own session, no core dumps, and it
  dies if the driver dies.

```text
  tide-admit (driver, outside)
     │  fork
     ├──> child: install Landlock + seccomp, then exec the test binary
     │        test code: simulated Root only (chapter 5)
     │        even a forged real capability → kernel says EACCES / EPERM
     │
     └── reads the child's output pipes, copies its manifest to target/sim/
```

Before starting, the driver prints the **sandbox level**:

- `sandbox: full (Linux …, Landlock ABI …)`: both tools present;
- `sandbox: partial, not enforced: …`: the kernel lacks something, named in
  the message. The run continues under what remains, and the guarantee then
  rests on the checker and compiler alone.

**Why tests first:** agent-written code mostly runs as tests, on developers'
machines, before anyone reviews it. That is where a forged capability would do
damage soonest. RUN-3 landed on 2026-11-17 (bead fx-ip4x), moved into v0 by
the decision recorded after fx-6xi2 (chapter 3).

## Run it

See the level every demo test run got:

```sh
bash -c 'grep -h "^sandbox:" target/tide-demo/logs/*.log | sort | uniq -c'
```

Output:

```text
      6 sandbox: full (Linux 7.2.6-76070206-generic, Landlock ABI 10)
```

Run the tests that prove it. Each builds a program that admission would
reject, slips it into the driver's build cache in place of an admitted
test, runs it through the driver, and checks that the **kernel** refuses
each attempt. The first program reads files and opens a socket directly;
the second, added by the red team (fx-qlek.5), tries to start programs,
fork, use `io_uring`, trace other processes, and run code before `main`:

```sh
cargo test -p tide-admit --test build run3_kernel_denies -- --test-threads=4
```

Output, after the compile lines:

```text
running 2 tests
test run3_kernel_denies_exec_fork_io_uring_ptrace_and_constructors ... ok
test run3_kernel_denies_an_unadmitted_test_child_paths_and_sockets ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 104 filtered out; finished in 2.37s
```

(The first run compiles the driver's test suite, which takes several minutes.)

## What to look at

- `sandbox: full` with your kernel version and Landlock ABI. On this
  machine every demo run got the full level.
- Both tests pass. They show the kernel, not admission, stops the program:
  the programs they run were never admitted. The first tries to read a
  canary file, write outside its directory, follow a planted link to another
  run's files, and open a TCP connection; the second tries the process,
  `io_uring`, tracing and before-`main` routes. Every attempt fails.

## Read the code

**`crates/tide-admit/src/invocation/sandbox.rs:84-101` and `:549-578`** (8
minutes). How the level is decided, and seccomp's list of allowed system
calls.

```sh
bash -c 'sed -n "84,101p;549,578p" crates/tide-admit/src/invocation/sandbox.rs'
```

- Lines 84–101, `fn level(kernel, abi, seccomp) -> String`: builds the
  `sandbox: ...` line. It starts an empty list called `missing`, and adds
  "Landlock" if the kernel has none, "Landlock scopes for signals and abstract
  Unix sockets" if Landlock is older than ABI 6 (too old to keep signals and
  local sockets inside the sandbox), and "seccomp" if that's absent. An empty list means `full (Linux …, Landlock ABI …)`; otherwise
  `partial, not enforced:` followed by what's missing. So the level you see is
  computed from what the kernel actually offers, not assumed.
- Line 549, `ALLOW`, and line 550, `DENY`: the two answers the filter can
  give a system call. `DENY` returns the error `EPERM`, "operation not
  permitted", so a forbidden call fails politely instead of killing the test.
- Lines 552–556: the comment saying what the list admits: memory, threads,
  clocks, randomness, reads and writes on files already open, the path
  operations Landlock confines, and exit.
- Lines 557–578, `const ALLOWED`: the list itself, one `libc::SYS_<name>` per
  system call. Read it as a whitelist: `read`, `write`, `openat` (open a file,
  which Landlock then confines), `mmap` (memory), `futex` (threads waiting on
  each other), `clock_gettime`, `getrandom`, `exit`. **What isn't there is
  the point:** no `socket` or `connect` (no network), no `fork` or `clone`
  for new processes, no `ptrace`, no `io_uring`, no `link` or `chmod`.
  These calls are allowed whatever their arguments. `execve` (run a
  program) is on the list, but Landlock lets the child execute only its own
  binary, as the comment above the list says.

The rest of the filter (`fn filter`, from line 588) adds a few calls allowed
only with particular arguments: `clone` only for new threads (so the test can
run tasks, but not start processes), and narrow uses of `prlimit64`, `ioctl`,
`fcntl` and `prctl`. `clone3` gets "not supported", so the C library falls
back to `clone`. Everything else gets `DENY`. The rest of the file sets up
the Landlock rules for the run's directories. You can stop here.

## The hardest question

> "If you need a kernel sandbox anyway, why bother with capabilities? And
> what protects a deployed program?"

**Two fences do different jobs.** Capabilities work at design time: a
signature tells a reviewer what code may do, and the checker catches
mistakes as you type. The sandbox works at run time and doesn't explain
anything; it just says no. Capabilities make authority readable, and the checker
rejects ambient calls at compile time. The sandbox bounds what a test run can
reach if the type system is fooled. Short version: **the type system is the
explanation; the kernel is the backstop**, for test runs today.

**What it covers today, and what it doesn't:**

- **Covered:** test and simulation runs started through the driver's run
  modes (`--tide-run-test`, `--replay`), on Linux.
- **Not covered:** a test binary run directly gets no sandbox. **Deployed
  programs** get kernel enforcement in v1: `main`'s attenuation becomes the
  matching Landlock and seccomp policy. The **build** itself (rustc, the
  linker) has no sandbox yet (RUN-2, bead fx-wktt). The child's output isn't
  yet escaped before it reaches your terminal (RUN-6, fx-e0ty). The planned
  `tide` commands don't exist to report the level yet (fx-0ioj).
- **Never covered:** the sandbox bounds what a run can *reach*, not what it
  *consumes*. A test can still spin the CPU or fill memory until a limit
  stops it, and timing side channels are out of scope.
- **Linux only.** Off x86_64 Linux the driver builds with the sandbox
  compiled out, and its run modes are unavailable there.


## Going deeper

- `design/01-design.md` D7, the "Kernel enforcement" bullet.
- `design/03-authority-concurrency.md` §2.6 (RUN-1 the guarantee, RUN-2 the
  build, RUN-3 test processes, RUN-4 the remaining holes).
- `docs/driver.md`, "Sandbox (RUN-3, in part)"; the repository `README.md`, "Sandbox" and "Known holes".
- `crates/tide-admit/src/invocation/sandbox.rs` (the code) and
  `crates/tide-admit/tests/build.rs` (the tests named `run3_*`).
