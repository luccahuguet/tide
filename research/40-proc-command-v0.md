# Proc::command in v0: the bounded process profile (fx-ayh2.1)

Status: design admission for bead fx-ayh2.1. It moves one bounded process
profile, real and simulated, into v0. Stage 2 implemented it (`tide-std`'s
`process.rs` with `process/real.rs` and `process/sim.rs`; §12's witnesses in
`process/tests.rs`), apart from §6's node crash, which waits for simulated
nodes; `Proc::exit` stays planned. Labels follow research/30: **[repo
evidence]** for what the repository says, **[source]** for cited outside
evidence, **[inference]** for this note's reasoning.

## 1. Need

**[repo evidence]** Owner decision A1, 2026-11-18, relayed by tide-desk and
quoted in fx-ayh2.1: "Bring Proc::command into the alpha, plus a network
capability if the design is sound. Then Tide v0 can express the honest
version of real code." tide-desk authorized fx-ayh2.1 as covered by those
words, and only fx-ayh2.1 blocks the alpha release candidate (fx-qlek.7).
fx-zu4f.8, the pre-registered re-screen of the 87 RustSec malicious-crate
advisories, depends on it: a large share of those crates spawn a program
(a downloader, a shell), and Tide v0 can only express the honest,
capability-passing version of such code if `Proc::command` exists.

**[repo evidence]** What exists today. 03 §1.1 already specifies `Proc`,
`Command`, `Child`, the pipe ends, `Stdio` and their receivers and
cancellation as v0 core, and `reference.md` lists `Proc` as planned. Admission
already classifies `Command`, `Child`, `ChildStdin`, `ChildStdout`,
`ChildStderr`, `Stdio` and `Signals` as carrying kind `proc`
(`crates/tide-admit/src/pure.rs`, `NOT_PURE`), and its ambient table rejects
`std::process::Command::new`, `exit` and `abort` (`policy.rs`, `TD0501`).
Tide's std has only the identity type `Proc`, "no operations yet"
(`crates/tide-std/src/lib.rs`). The simulator has no process model: 03 §9.8
lists "real child processes, which need a `sim.world.proc` model" as not
simulated, and 03 §9.9 (fx-f6xp, research/30) designs that model in Later
tier. So what A1 moves is not the real API, which v0 already promised, but
(a) the commitment to build it now, (b) the attenuation's exact matching
rule, which no document fixes, and (c) a lifecycle subset of §9.9's
simulated model, without which v0's rule that every capability has a
deterministic simulated version (reference 6.8) would have a hole exactly
where the alpha's demonstration needs one.

## 2. The profile (smallest surface)

**[inference]** In v0, real and simulated:

```rust
impl Proc {
    pub fn command(&self, program: impl AsRef<OsStr>) -> Command;
    pub fn only(&self, programs: impl IntoIterator<Item: AsRef<OsStr>>) -> Proc;
}
impl Command {   // Rust's builder methods and receivers (03 §1.1)
    pub fn arg(&mut self, a: impl AsRef<OsStr>) -> &mut Command;
    pub fn args(&mut self, a: impl IntoIterator<Item: AsRef<OsStr>>) -> &mut Command;
    pub fn env(&mut self, k: impl AsRef<OsStr>, v: impl AsRef<OsStr>) -> &mut Command;
    pub fn envs(&mut self, vars: impl IntoIterator<Item = (impl AsRef<OsStr>, impl AsRef<OsStr>)>) -> &mut Command;
    pub fn env_remove(&mut self, k: impl AsRef<OsStr>) -> &mut Command;
    pub fn env_clear(&mut self) -> &mut Command;
    pub fn current_dir(&mut self, dir: impl AsRef<Path>) -> &mut Command;
    pub fn stdin(&mut self, s: impl Into<Stdio>) -> &mut Command;   // and stdout, stderr
    pub fn output(&mut self) -> io::Result<Output>;
    pub fn status(&mut self) -> io::Result<ExitStatus>;
    pub fn spawn(&mut self) -> io::Result<Child>;
}
pub struct Child {
    pub stdin: Option<ChildStdin>,
    pub stdout: Option<ChildStdout>,
    pub stderr: Option<ChildStderr>,
    /* private */
}
impl Child {
    pub fn id(&self) -> u32;
    pub fn wait(&mut self) -> io::Result<ExitStatus>;
    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>>;
    pub fn wait_with_output(&mut self) -> io::Result<Output>;   // &mut self, 03 §1.1
    pub fn kill(&mut self) -> io::Result<()>;
}
// ChildStdin: io::Write; ChildStdout, ChildStderr: io::Read (also for &T, as TcpStream).
// Stdio: piped(), inherit(), null(), From<File>, From<ChildStdin|ChildStdout|ChildStderr>,
//        From<io::Stdout>, From<io::Stderr> (03 §1.1).
// ExitStatus: code() -> Option<i32>, success() -> bool, Display. Output: Rust's fields.
```

These are Rust's `std::process` names, signatures and receivers, with 03
§1.1's two changes: no ambient `Command::new`, and `wait_with_output(&mut
self)`. `Proc::exit` keeps its existing v0 specification and is not part of
this profile (it stays planned). Later, on both backends, and outside this
profile: `Proc::signals`, `Signals`, `Child::signal`, `ExitStatus::signal`,
`spawn_pty`/`Pty`, `run_until_exit`, parent-only process death, and §9.9's
fault families. `Proc::signals` and `Signals` were listed as v0 surfaces
(03's Proc row, companion 20); they move to Later with the rest of §9.9's
signal rules, since a v0 signal API without its simulated model would be
the one v0 capability operation with no deterministic version, and fx-ayh2.1
keeps signals out of the alpha.

**[inference] What it adds.** To the language: nothing; no syntax, no
checker rule, no new authority kind (`proc` exists). To std: the methods
above on existing types, one attenuation rule (§3), a real backend over
`std::process`, and a simulated lifecycle model (§6). To the checker: no
new classification, since admission already treats the handles as `proc`
carriers; but admission admits every public item of Tide's std unless a
rule says otherwise (`walk.rs`, `Role::TideStd`), so CAP-1 for processes
rests on `tide_std::process` exposing no other public path to a process,
which stage 2 must guard (§12). To readers: 24 tokens in
`reference.md`, measured (24,961 to 24,985; §9 below), the rest in the
companion and 03.

## 3. `only([...])`: the matching rule

**[source]** How others match program allowlists:

- Deno's `--allow-run=git` matched the name a script passed, which a
  changed `PATH` defeated: a write-up showed `--allow-run=uname` running a
  fake `uname` placed first on `PATH`, and `LD_PRELOAD` passed through the
  spawn's environment escaping the allowlist outright
  ([tty5.dev](https://tty5.dev/posts/deno-security/)). Deno then resolved
  allowlist entries to absolute paths at startup and compared resolved
  paths, refused spawning with `LD_PRELOAD` without unrestricted run
  permission (denoland/deno#25221), widened that to every `LD_` and `DYLD_`
  variable (#25271), and documents that a subprocess is not sandboxed and
  that granting a shell or `deno` itself is full escalation
  ([Deno security](https://docs.deno.com/runtime/fundamentals/security)).
- WASI has no process creation at all: the `wasi:cli` world offers
  environment, exit, standard streams and terminal interfaces, and no
  spawn ([WebAssembly/wasi-cli](https://github.com/WebAssembly/wasi-cli)).
- Capsicum's capability mode removes path-based exec; a sandboxed process
  runs a program through `fexecve` on a descriptor that carries
  `CAP_FEXECVE`, so authority is a held file, not a name
  ([cap_enter(2)](https://man.freebsd.org/cap_enter/2); FreeBSD
  `tests/sys/capsicum/fexecve.cc`).
- Fuchsia's `fuchsia.process.Launcher` takes the executable as a VMO
  handle and starts the process with only the handles its creator passes;
  there is no ambient path lookup
  ([fuchsia.process](https://fuchsia.dev/reference/fidl/fuchsia.process)).

**[inference] Options.**

1. *Spelling match, Rust's lookup.* Compare `program` with the entries as
   strings, then let `std::process` search the `PATH` the command's
   environment names. A `cmd.env("PATH", dir)` then picks any file named
   `git`. Rejected: Deno's original hole.
2. *Resolve entries to canonical files at `only`* (Deno today). Binds the
   grant to a file, but needs host file access inside an attenuation that
   cannot fail (`only` returns `Proc`), and has no meaning under
   simulation.
3. *Descriptor-based exec* (Capsicum, Fuchsia). The strongest, but it
   needs an `Fs`-derived executable handle and a new API; not Rust's shape.
4. *Spelling match, Tide-owned lookup* (chosen). Entries and programs match
   by exact `OsStr` bytes; a bare name is then resolved by Tide's std, not
   by the command's environment, against the `PATH` the process started
   with.

**[inference] Decision (option 4).** PROC-ONLY:

- An entry is a bare name (no `/`) or an absolute path. An empty entry, one
  containing NUL, or a relative path with a `/` (`./tool`, `bin/tool`)
  grants nothing, as `Net::only`'s invalid entries do. `only` on an
  attenuated `Proc` intersects: the result allows a program only if both
  lists do. Matching is by exact bytes: no case folding, no normalization,
  no canonicalization, no symlink resolution. `only(["git"])` allows
  `command("git")`, not `command("/usr/bin/git")`, and the converse.
- The check runs in `spawn`, `output` and `status`, before any resolution,
  host access or model lookup (03 §9.9 already orders `only` before model
  lookup). `output` and `status` are checkpoints and observe cancellation
  first; `spawn` is not a checkpoint (03 §9.9) and never observes
  cancellation, so a permitted `spawn` in a cancelled scope starts its
  child, and a refused one returns `PermissionDenied`. A refused program
  starts nothing. `command` itself cannot fail.
- **Lookup.** For every `Proc`, attenuated or not, a bare name is resolved
  by Tide's std against the `PATH` the process started with (the value
  `env.var("PATH")` returns on an unattenuated `Env`), using only its
  absolute elements. Empty elements, which POSIX reads as the current
  directory, and relative elements such as `tools`, which name a directory
  relative to the current directory ([environ(7)](https://man7.org/linux/man-pages/man7/environ.7.html)),
  are skipped, so lookup never depends on the working directory. The
  first element holding an executable regular file with that name wins.
  If the process started with no `PATH`, or one with no absolute
  element, a bare name finds nothing and returns `NotFound`: there is no
  default search path (glibc's `execvp` falls back to one), so no path
  is searched that the start-time `PATH` does not name.
- **Canonical exec.** Std resolves that file, or an absolute program, to
  its canonical path in the parent, before `current_dir` applies, and the
  child executes the canonical path, with argv[0] the program as spelled
  in `command` on both backends (models match the arguments after it).
  The kernel would otherwise resolve the path a second time in the child,
  after `chdir`, so an absolute `only` entry or `PATH` element through a
  symlink or a per-process link such as `/proc/self/cwd` (reached
  directly, as `//proc/..`, or through a symlink like `~/bin ->
  /proc/self/cwd/bin`) would let `current_dir` choose the program;
  resolved in the parent, `/proc/self/cwd` names the parent's own
  directory. This replaces the earlier lexical skip of `PATH` elements
  under `/proc/`, which missed those spellings and is not kept: one rule
  that covers every route is simpler than a partial one beside it.
  *Cost:* one more path walk per spawn, and a script reached through a
  symlink gets the canonical path as its script name, so its interpreter's
  `$0` or `__file__` names the target, not the link, so a script that
  dispatches on its own name through symlinks runs as its target; a program that reads
  `/proc/self/exe` already saw the target. Stated limits: whatever is at
  the canonical path when the child execs runs (a replaced file), and a
  script whose `#!` interpreter path is relative resolves it after
  `current_dir`. If canonical resolution fails, `spawn`, `output`, and
  `status` return its error and start nothing: the file lookup found can
  vanish first, resolution can hit `ELOOP`, and a deleted file or a memfd
  reached through `/proc/self/fd/N` has no resolvable path, so a program
  Rust's exec would run there fails under Tide.
- **Command `PATH`.** A command's own `env("PATH", ..)` sets the child's
  environment only; it never changes which program starts. Under
  simulation nothing is resolved: models are registered under exact
  spellings (§6). This differs from Rust in two ways, both also for an
  unattenuated `Proc`, so there is one lookup rule: Rust's Unix `Command`
  searches a `PATH` set on the command
  ([Command](https://doc.rust-lang.org/std/process/struct.Command.html)),
  which is option 1's hole; and Rust follows inherited empty and relative
  elements, so with a start-time `PATH=tools:/usr/bin` and no command
  environment, Rust runs `./tools/tool` while Tide runs `/usr/bin/tool` or
  returns `NotFound`.
- **Loader variables.** Under an attenuated `Proc`, a command whose final
  explicitly set environment (through `env` or `envs`) has a variable whose
  name starts with `LD_` or `DYLD_`, or that is any name in glibc 2.39's
  `UNSECURE_ENVVARS` (`sysdeps/generic/unsecvars.h`, 34 names: the `LD_` ones, `GCONV_PATH`, `GETCONF_DIR`, `GLIBC_TUNABLES`, `HOSTALIASES`, `LOCALDOMAIN`, `LOCPATH`, the `MALLOC_*` tunable aliases and `MALLOC_TRACE`, `NIS_PATH`, `NLSPATH`, `RESOLV_HOST_CONF`, `RES_OPTIONS`, `TMPDIR`, `TZDIR`), is
  refused with `PermissionDenied`, as is any name holding `=` (stage 2
  review s2-r1: Rust passes such a key through, and the child's C library
  reads the bytes before the `=` as the name). It is glibc's list at that version:
  another C library's variables (musl's `MUSL_LOCPATH`) and names later
  glibc adds are not refused. This is the set
  glibc itself strips for setuid programs because it makes the C library
  load code or data (`GCONV_PATH` was CVE-2021-4034's vector), so the
  prefix rule alone, Deno's, would leave escapes of the same class open.
  Variables the process itself inherited pass through as the operator
  started it. Unattenuated `Proc` sets them freely.
- **Order, on both backends.** After `output`'s and `status`'s
  cancellation check, the calls check the program against `only`, then
  the refused variables, then NUL bytes in the program, arguments,
  environment and directory (`InvalidInput`, as Rust's `spawn` returns),
  and only then run lookup or model lookup. A simulated model can
  therefore never accept a command that these checks refuse on the real
  backend; host-state failures (a missing directory, `EACCES`, `ETXTBSY`)
  are what models declare.
- **Relative program paths** run only under an unattenuated `Proc`, where
  they keep Rust's behavior.

## 4. Authority semantics, plainly

**[inference]** A child process is not capability-checked. Whatever
`Proc` starts runs with at least the operating-system authority of the Tide
process (more for a setuid program): every file, socket, environment
variable and program the user can reach. Process authority has no end: a
child, and anything it starts, keeps running after the call that started
it returns and after the Tide process exits, so lending `&Proc` or a
`Command` for one call is not a temporary grant.
`Proc` is therefore ambient authority for the child, the widest grant in
the set (03 §1.1), and `only` bounds which program image starts, not what
it does. That is the contract, and the docs say it in these words.

- **Interpreters and shells.** An allowlist naming `sh`, `bash`, `python`,
  `python3`, `node`, `perl`, `ruby`, `deno`, `env`, `xargs`, `busybox`,
  `sudo`, `ssh` or any program that runs a program named in its arguments
  or input is full authority: `only(["sh"])` equals `proc`.
- **Arguments, variables, files and scripts.** Many allowed programs run
  other programs on request: through arguments and program-specific
  variables (`git -c core.sshCommand=..`, `GIT_SSH_COMMAND`, `PAGER`,
  `EDITOR`, `find -exec`, an archiver's command option); through the files
  they read, chosen by `current_dir` or by `HOME` and similar variables
  (git's `.git/config` with `core.fsmonitor`, hooks, a Makefile,
  `build.rs`, `package.json` scripts); and as scripts, which run their
  `#!` interpreter, or `/bin/sh` for a file with no `#!` line (glibc's
  `execvp` on `ENOEXEC`), with that interpreter's code-bearing variables
  (`BASH_ENV`, `NODE_OPTIONS`, `PERL5OPT`, `PYTHONPATH`, `RUBYOPT`). `only`
  inspects none of these, so such an allowlist is full authority in
  practice. Stated limit.
- **The return path.** An allowed program that reads and prints (`cat`,
  `git show`, `printenv`) hands the calling code whatever it reads,
  whatever that code's `Fs`, `Env` and `Stdin` allow; with stdin
  inherited, a child can read the user's terminal for code that holds no
  `Stdin`. Stated limit.
- **Why `only` still exists.** It makes the intended programs reviewable,
  stops accidental and unlisted spawns (a dependency fetching a downloader
  by absolute path, a typo), and closes the program-independent escapes
  it knows (`PATH`, and the loader and C-library variables above). It is not a sandbox; kernel enforcement for
  deployed binaries is D7's v1 work, and a child stays outside it there too
  unless the sandbox is inherited.

## 5. Environment, working directory and descriptors

**[inference]** Defaults and limits:

| Topic | Default (both backends where it applies) | Residual limit |
| --- | --- | --- |
| Environment | Inherited whole, as in Rust; `env_clear`, `env_remove`, `env`, `envs` change it. An attenuated `Env` does not filter it: the child is ambient. | A child sees variables the Tide code's `Env` could not read. |
| `PATH` | Lookup uses the start-time `PATH` (§3); the command's `PATH` reaches only the child and its own lookups. | A writable directory on the start-time `PATH` lets whoever writes it choose the program. |
| Loader variables | Refused when set on a command under `only` (§3). | Inherited ones pass; other injection variables are program-specific (§4). |
| `current_dir` | As in Rust; a relative directory resolves against the parent's current directory. It needs no `Fs`: the child is ambient. Lookup and canonical resolution happen in the parent before it applies, so it cannot change which program starts, also through symlinks or `/proc/self/cwd` (§3). | A relative `#!` interpreter resolves after it (§3); it chooses the files and configuration the program reads (§4). |
| Descriptors | The child gets exactly its three standard descriptors as `Stdio` configures them. Tide's std opens every descriptor close-on-exec, and the real backend also marks every other descriptor close-on-exec in the child before exec (Linux `close_range(3, ~0, CLOSE_RANGE_CLOEXEC)`, after the standard descriptors are set up), so descriptors the Tide process inherited or a `ffi` call opened do not reach it. The sweep fails closed: if it fails (`ENOSYS` before Linux 5.9, `EINVAL` for `CLOSE_RANGE_CLOEXEC` on 5.9–5.10), the child exits before exec and `spawn` returns the error. | Descriptors passed deliberately through `Stdio::from(file)` or a pipe end are the child's. |
| Symlinks | Resolved by std in the parent to a canonical path, which the child execs (§3); entries name paths, not file contents. | Whoever can replace the file or a link on the path chooses the program. |

## 6. Simulated lifecycle subset (sim.world.proc in v0)

**[repo evidence]** research/30's accepted design and 03 §9.9 own the
mechanics; this profile takes a subset and redesigns nothing.

**[inference]** In v0:

- **Setup.** `sim.world.proc(program, ProcessModel)` registers an owned,
  declarative model under an exact program spelling, with no `PATH` search,
  shell interpretation or host execution; duplicates and malformed models
  panic during setup. Setup takes no scheduling point, draw or time and is
  recorded inline. `ProcessModel::missing()` declares a program that does
  not exist: spawning it returns `NotFound` and creates no instance, so a
  missing program is a tested outcome, not a coverage gap. The model
  builder's spelling is provisional, as §9.9's API spellings are.
- **Models.** As §9.9: accepted arguments, environment, working directory
  and stdio; ordered transitions consuming spawn, input bytes and input
  closure, and timers, emitting output, endpoint closure and exit with a
  code; finite alternatives and bounded delays, no host callbacks. A
  command shape or input the model does not accept is a coverage failure.
  Signal handling, PTY and signal-death transitions are Later, apart
  from the two terminations with no code below.
- **Lifecycle.** §9.9's `Running -> Zombie(status) -> Reaped(status)`: spawn
  failure creates no child; creation is atomic and the child may exit
  before the caller resumes; exit stores one immutable status and closes the
  child's endpoints; each `Child` owns its own wait; `try_wait` returns
  `None` while running and otherwise reaps once and caches; `wait` blocks
  for the same transition and closes the retained stdin; repeated calls
  return the cached status; dropping a `Child` neither kills nor reaps, and
  dropping its pipe fields closes them. EOF is not an exit status.
- **Kill.** `kill` on a running child submits SIGKILL to that child, as
  Rust's Unix `kill` does, and returns without waiting; termination is a
  ready model action on the existing scheduler, as research/30's
  submission, delivery and reap stages are. So `child.kill()?;
  child.try_wait()?` may return `None` before termination runs, and a
  later `wait` returns the status. If the model's normal exit runs first,
  its status stands; if termination does, the status has no code
  (`code() == None`, `success() == false`). On an exited child `kill`
  returns `Ok`, as Rust does. Cancelled `output` and `status` follow 03
  §1.1.
- **Piped I/O.** Each direction keeps byte order, with a finite buffer
  (recorded in the manifest), backpressure, and short reads and writes
  drawn from the process sub-stream. A write to a pipe whose reader closed
  returns `BrokenPipe`. The model's own write to a closed parent end
  either ends it with no code, the default, as SIGPIPE's default action
  does, or, if its model says so, fails with `BrokenPipe` and the model
  continues, as a program that ignores SIGPIPE (a Rust program) does. A v0
  model can declare nothing else for that write.
- **Statuses with no code.** Under simulation in v0 they come only from a
  `kill` whose termination runs before the model's exit, and from that
  default broken-pipe death; on the real backend any signal death gives
  one. `code()` is `None`, `success()` is `false`, and `Display` prints
  what Rust's `ExitStatus` prints on Linux for that status (`signal: 9
  (SIGKILL)`, `signal: 13 (SIGPIPE)`), on both backends. No v0 method
  returns the signal number (`ExitStatus::signal` is Later), and a v0
  model cannot declare any other death by signal. *Reason:* signal
  handling is Later, so v0 admits only the two terminations it cannot
  leave out: `kill`, which cancelled `output` and `status` need (03 §1.1),
  and a write to a closed pipe, which any piped model can reach. Matching
  `Display` keeps one tested contract on both backends (PROC-1), and the
  simulated status records which of the two ended it, which the replay
  trace needs anyway.
- **Stdio modes.** `piped` and `null` on all three descriptors; `inherit`,
  `io::stdout()` and `io::stderr()` on stdout and stderr, whose bytes enter
  the simulated process streams after earlier records (SINK-4, SINK-6).
  Inherited stdin, `From<File>`, and pipe-end chaining between children are
  unsupported modes in the v0 simulator: a coverage failure. Since Rust's
  `status` and `spawn` inherit stdin by default (`output` defaults it to
  null), code tested under simulation sets `.stdin(Stdio::null())` or
  `.stdin(Stdio::piped())`, which means the same on both backends.
- **Crash.** A whole-node crash (§9.5) ends that node's modeled children
  with it; restart starts none of them again.
- **Determinism.** §9.9's rules: the existing scheduler alone interleaves
  tasks and ready model actions; a separate process sub-stream chooses
  model alternatives, delays and short I/O sizes; a run that spawns nothing
  makes exactly today's draws; instance identities are node incarnation and
  spawn ordinal; `Child::id` returns simulated data, never a host PID. No
  scheduler, time, cancellation or deadlock rule changes; model timers count
  for §9.3's stall guard.
- **Coverage.** An unregistered program, an unsupported mode or an
  unmatched model input stops the seed with a source-located unmodelled-
  effect report naming the operation, instance and missing model, also
  under `--allow-unmodeled`; never `NotFound`, success, or a host process.
  Since tests receive only simulated capabilities, RUN-1 and RUN-3 are
  unchanged: a test never starts a host process.

## 7. Replay identities

**[inference]** Under fx-nkv.7's manifest (REPLAY-1, REPLAY-2) a run that
registers a model records inline: the process profile identity
(`proc-lifecycle-v0`) and model schema version, every registered program
spelling with its model data, the pipe buffer capacity, and the seed from
which the process sub-stream is derived. Replay refuses a manifest whose
profile, schema or model data differ from the run's (REPLAY-2), as it
refuses a changed build identity. A seed that uses `Proc` replays byte for
byte.

## 8. Diagnostics and trust diff

**[inference]**

- `TD0501` stays for `std::process::Command::new`, `std::process::exit` and
  `abort`, with the fix `proc.command(..)` or a returned `ExitCode`. Since
  Tide's `Command` has no `new`, stage 2 must keep that call reporting
  `TD0501`, not bare `E0599`.
- Declared authority: every public item whose signature reaches `Proc`,
  `Command`, `Child`, a pipe end or a `Stdio` declares kind `proc`
  (admission's existing classification), and `tide review` lists new paths
  to `proc` first (03 §4.3). Its line says that the child is not
  capability-checked and names the reaching `only` list as programs, not a
  sandbox (03 §4.3's example). Exercised-kind reporting is v1 (§4.1), unchanged.
- `TD0514` lists build-time-known unmodelled process effects, as for other
  foreign effects; the runtime coverage report of §6 covers the rest.

## 9. Cost

**[inference]** Readers and agents: one rewritten `Proc` row in
`reference.md`, 24 tokens, held inside D14's 25,000 (24,985), plus
companion 20 and 03 §1.1/§9.9 text served on demand. Implementation: a real
backend that is mostly `std::process` plus lookup and the descriptor
sweep; a model interpreter for lifecycle and pipes on the existing
scheduler; manifest fields. Platform: the sweep needs Linux 5.11
(`CLOSE_RANGE_CLOEXEC`) and fails closed before it (§5), so on an older
kernel, such as RHEL 8's 4.18 or Ubuntu 20.04's 5.4, the real backend
starts no process: `spawn`, `output` and `status` return the error.
Entropy: no second way to start a process, and
Rust's API shape is kept, so ports change only `Command::new` and the
`wait_with_output` binding. The divergence agents can trip on is the
lookup rule (§3): which binary runs, or whether one is found, differs from
Rust when a command sets its own `PATH`, and when the start-time `PATH`
has empty or relative elements, whose programs Rust finds through the
working directory and Tide never does. A second is the simulator's
default-stdin limit (§6), which `.stdin(Stdio::null())` resolves.

## 10. Admission filter

**[inference]** GOVERNANCE.md's questions:

- *Safer change?* Yes: process use becomes a declared, reviewable
  authority with a deterministic, replayable test model, where today the
  honest code cannot be written at all.
- *Overlapping ways?* None added: one constructor, Rust's builder, one
  attenuation.
- *Local reasoning?* Preserved: a function without `Proc` or a handle
  cannot spawn; the cost is stated plainly that holding `Proc` means
  holding the child's ambient authority.
- *Fast feedback?* Simulated tests run without host processes.
- *One obvious shape?* Rust's.
- *Library instead of core?* Yes: it is std and runtime only.

Verdict: admit the bounded profile, with the limits of §4 and §5 stated
in the docs.

## 11. Alternatives rejected

- **Keep Proc out of v0** (what owner decision A1 reversed on 2026-11-18): Tide
  v0 then cannot express the honest version of code that spawns.
- **Real backend only, no simulated model**: breaks reference 6.8's rule
  for the capability the alpha demonstrates, and tests could not use it at
  all, since they hold only simulated capabilities.
- **All of §9.9 in v0**: signals, PTYs, parent-only death and faults are
  fx-ayh2's post-alpha work; they double the model and the red-team
  surface.
- **Argument or interpreter filtering in `only`**: cannot be complete, and
  a partial filter would read as a sandbox.
- **A distinct `Exec` capability per program** (Capsicum or Fuchsia
  style): stronger, but not Rust's shape and not needed while D7's kernel
  enforcement is v1.

## 12. Stage 2 and red team

**[inference]** The red team (fx-ayh2.1 Scope 2, review of `fc015c3`)
attacked §3 to §5; its findings are fixed above or stated as limits.
Stage 2 requirements it added:

- the order of §3 holds on both backends: a simulated model never accepts
  a command whose program, refused variable or NUL byte these checks
  refuse on the real backend (PROC-2 exercises the variable and NUL
  refusals under simulation);
- canonical exec: with an absolute `only` entry or start-time `PATH`
  element through a symlink to `/proc/self/cwd/bin`, and a `current_dir`
  holding a different program of the same name, the child runs the file
  the parent resolved, with argv[0] as spelled (PROC-2);
- the descriptor sweep fails closed, after the standard descriptors are
  set up (PROC-1 checks that a child sees exactly its three descriptors
  while the parent holds an extra inherited one);
- admission's default for Tide's std items is guarded for
  `tide_std::process` by a test that lists its public items, `pub use`
  re-exports included (admission admits a re-exported item before any
  rule, `walk.rs` `admit_def`, so a re-exported `std::process::exit`
  would bypass both a deny glob and `TD0501`), and fails on additions. A
  `deny("tide_std::process::*")` glob in `policy.rs` may sit beside that
  test, never instead of it. Backend entry points stay `pub(crate)` or
  under `__rt`, and any new type that can hold a handle goes into
  `NOT_PURE`.

**Seed witnesses and controls.** The bead's acceptance asks for a seed
witness of every permitted finite outcome in the lifecycle subset and a
deliberately broken control. The set is research/30 §8's rows that fall
in v0, cut to their v0 part, plus the outcomes §6 and 03 §1.1's
cancellation rules permit. This table owns the set; PROC-1 refers to it.
Each outcome below has a seed witness, and the control after it must
fail:

| Outcome (seed witness) | Broken control that must fail |
| --- | --- |
| research/30 §8: output closes while the child keeps running, then it exits; it exits with buffered bytes unread. | Turn EOF into a reap, or drop buffered output on exit. |
| research/30 §8: a scope deadline and a process timer due at the same instant fire deadline first, before ready actions run. | Let the process action bypass deadline cancellation, or run a task between due timers. |
| research/30 §8: a `try_wait` spin with a future model-exit timer hits §9.3's stall guard. | Leave model timers out of the guard and wait for the wall-clock timeout. |
| research/30 §8: a failed spawn creates no instance; `kill` ends the child; dropping a `Child` neither kills nor reaps. | Create an instance on a failed spawn, ignore `kill`, or kill or reap on drop. |
| research/30 §8: a missing model or unsupported mode is a coverage failure, and no host process starts. | Treat a missing model as `NotFound` or success, or call the real backend under simulation. |
| research/30 §8: the same identity and seed give a byte-identical trace; replay refuses changed model data. | Use the host clock or a PID, or leave model data out of the manifest. |
| §6: a repeated `wait` or `try_wait` returns the cached status. | Reap again on the second call. |
| §6: the child exits before the caller resumes from `spawn`. | Keep a new child running until the caller's next scheduling point. |
| §6: after `kill`, either the model's normal exit or termination runs first; `try_wait` may return `None` in between. | End the child inside `kill`, or let termination always override a normal exit. |
| §6: short reads and writes on the pipes, sizes drawn from the process sub-stream. | Always transfer the full buffer. |
| §6: a model's write to a closed parent end ends it with no code, or, if declared, gets `BrokenPipe` and continues. | Give the default death a code, or let a model declare another signal death. |
| §6: a parent write after the model closed its input returns `BrokenPipe`. | Let the write succeed silently. |
| §6: a parent write into a full pipe buffer blocks until the model reads (backpressure). | Use an unbounded buffer. |
| §6: `wait` closes the retained stdin, so the model sees input closure; `try_wait` does not. | Close stdin in `try_wait`, or leave it open in `wait`. |
| 03 §1.1: `output` or `status` cancelled before the spawn starts nothing and returns `Cancelled`. | Start the child anyway. |
| 03 §1.1: `output` or `status` cancelled after the spawn kills the child, closes its pipe ends, waits shielded for it to exit, and returns `Cancelled`. | Return `Cancelled` without waiting, or leave the child running. |
| 03 §1.1, TASK-3: a wait inside `output` or `status` that completes before the cancellation takes effect returns its own result. | Return `Cancelled` after the wait completed. |
| 03 §1.1: a cancelled `wait` or `wait_with_output` returns `Cancelled` and leaves the `Child` with the caller, which can still `kill` and `wait` it. | Consume or drop the `Child` on cancellation. |

`ProcessModel::missing()` witnesses `NotFound`. research/30 §8's other
rows (hints, subscriptions, PTYs, signal death, parent-only death,
faults, `run_until_exit`) are Later with their surfaces.

**What stage 2 changes when the code lands.** Stage 2 fills the cases of
the conformance rows stage 1 added (`PROC-1`, `PROC-2`, `PROC-3` in
`conformance/rules.md`) and sets their status, implements §6 and §7, and
records the `ProcessModel` builder spelling it picks in 03 §9.9 and adds
one short simulated-test example to companion 20. Then it flips each
label that describes the profile as not built:

- `reference.md`'s `Proc` row: move `planned` from the row onto `exit`,
  as the `Net` row marks its planned methods (`... -> Command (...);
  planned: exit(code: i32) -> !`), unless stage 2 also builds
  `Proc::exit`; then drop it.
- `reference.md` 6.8: "(partial: not yet `Rand` or `Proc`)" loses
  `Proc`.
- companion §20's heading loses "(planned)".
- 03 §9.9's "None of it is implemented yet; §9.8's exclusions remain
  until it ships" says the lifecycle subset is implemented.
- README.md: the not-built list at line 230 ("`Proc` operations"; `Proc`
  then is more than an identity), the simulation table at line 247 ("real
  child processes" among what is not simulated), and the not-built
  summary at line 431 ("`Proc` operations").
- this note's Status line ("Nothing here is implemented"), and the `Proc`
  doc comment in `crates/tide-std/src/lib.rs` ("It has no operations
  yet").

Stage 2 recounts `reference.md` after that edit (D14).
