Model: Claude Opus 5.5 · Date: 2026-11-18 · Red team, candidate fc015c3 (design)

# Red team: fx-ayh2.1, Proc::command in v0

- Candidate: branch `fx-ayh2.1`, HEAD `fc015c3988a22a43651e13131390fbf17cc6ab05`, base `main` `91a2f36`.
- Diff: `git diff 91a2f36..HEAD`, SHA-256 `37ffbe01eb0c2292a8e065503076be0148c1c5b5a75427915e227b355f9259b2` (the same diff Codex approved).
- Read: research/40 (whole), the diff to 01, 03, the companion, reference.md, rules.md and research/30, plus 03 §1.1, §3, §4, §9.9; `crates/tide-admit/src/{policy,walk,crates,pure}.rs`; `crates/tide-std/src/{lib,root,net}.rs`; both Codex reports, which I did not repeat.
- This was a read-only design review. Nothing was compiled or run, and every claim about stage 2 behavior is inference from the design and the pinned sources cited.

## Verdict

**Changes required.** The authority mechanics hold: matching is exact, lookup is owned by Tide, the attenuation algebra cannot widen, and the sim refuses in the same order as the real backend. I found no route that starts a process without `Proc`. Three statements about what the authority means are narrower than the truth, though, and a reviewer reading the trust diff or companion 20 would be misled by them (F1–F3). All three fixes are text plus at most one fixed refusal list. None touches `design/reference.md`, which stays at 24,985 tokens. F4–F9 are limits to state or requirements for stage 2.

## Findings

### F1. "Full authority" is tied to arguments and variables, but files, scripts and the return path give it too (misleading docs, blocking)

**Where.** 03:303–306 (PROC-ONLY: "a program that runs a program named in its arguments or variables (`git -c core.sshCommand`, `find -exec`) is full authority"). Companion 20:646–649 ("a program that runs what its arguments name"). research/40:192–210 (§4). 01:512–516. 03:284 and research/40:192 ("the child runs with the operating-system authority of the Tide process").

**Attack.** A reviewer sees `root.proc.only(["git"])` and a call `command("git").args(["status"]).env_clear()`, with fixed arguments and no variables. Every document then says it is bounded. It isn't:

1. **Files the program reads, chosen without `Fs`.** `current_dir` needs no `Fs` (03:309). `git status` in a directory whose `.git/config` sets `core.fsmonitor = <cmd>` runs `<cmd>`. So do `.git/hooks/*`, `core.hooksPath`, `make`'s Makefile, `cargo build`'s `build.rs` and `.cargo/config.toml`, and `npm install`'s `package.json` scripts. Setting `HOME`, `XDG_CONFIG_HOME` or `GIT_CONFIG_GLOBAL` on the command picks the user-level config the same way. Code that can name a directory with attacker-chosen contents (a cloned repository, a downloaded archive, any path it can write through its own `Fs`) has full authority with fixed arguments.
2. **Scripts.** An allowed program that is a `#!` script runs its interpreter. That covers `pip`, `npm`, `npx`, `meson`, `ldd` and many `/usr/bin` wrappers. The interpreter then reads its own code-bearing variables. `BASH_ENV='$(cmd)'` runs `cmd` before a bash script starts, with no file needed, because bash expands the value. `NODE_OPTIONS=--require ..`, `PERL5OPT`, `PYTHONPATH` and `RUBYOPT` do the same. Also, the real backend's `pre_exec` descriptor sweep forces Rust's fork-and-`execvp` path, and glibc's `execvp` re-runs a file without a `#!` line under `/bin/sh` on `ENOEXEC`, even for a path that contains a slash. So an allowed text file is a shell script. Readers cannot see from a name that it is an interpreter.
3. **The return path.** With stdout piped, any allowed program that reads and echoes brings what it read back to the Tide code itself, not just to the child. Examples: `cat`, `git show`, `printenv`, `tar -c`, `curl file://`. So `Proc` gives the code everything that program can read, whatever its own `Fs::dir`, `Env::only` or lack of `Stdin` allows. Stdin is inherited by default, so a child can also read the user's terminal for code that holds no `Stdin`.
4. **More than the process's own authority.** setuid, setgid and file-capability programs run with more privilege than the Tide process. A child that inherits the terminal can also use the user's cached `sudo` timestamp. "The operating-system authority of the Tide process" is a lower bound, not the extent.

**Impact.** A reviewer approves a "narrow" `only([...])` that is full authority. The bead's Scope 2 asks for exactly this statement ("git runs hooks, aliases and core.sshCommand"). The candidate says it for arguments only.

**Minimum fix (text).**

Replace the last sentence of 03's PROC-ONLY bullet (03:303–306) with:

> `only` bounds which program image starts, not what it does. The grant is full authority whenever the program can be steered: an allowed shell, interpreter, `env`, `xargs`, or `sudo`; a script, which runs its `#!` interpreter (or `/bin/sh` for a file with no `#!` line) and that interpreter's variables (`BASH_ENV`, `NODE_OPTIONS`, `PYTHONPATH`); a program that runs what its arguments or variables name (`git -c core.sshCommand`, `GIT_SSH_COMMAND`, `find -exec`); and a program that runs what the files it reads name: its configuration and hooks, chosen by `current_dir` or by `HOME` and similar variables, or a working directory's Makefile, `build.rs`, or `.git/config`. An allowed program that reads and prints (`cat`, `git show`, `printenv`) hands the calling code whatever it reads, whatever that code's `Fs`, `Env`, and `Stdin` allow. A setuid program runs with more than the process's own authority. `only(["git"])` is bounded only when the arguments, environment, working directory, and the files git reads are all fixed and trusted.

At 03:284 and research/40:192, change "the operating-system authority of the Tide process" to "at least the operating-system authority of the Tide process (more for a setuid program)". In companion 20's `only` bullet (646–649), add one sentence: "A script, a program steered by the files it reads (git's config and hooks in `current_dir`, a Makefile), or one whose output you read back gives full authority as well (03 PROC-ONLY)." In research/40 §4's second bullet, name files and scripts beside arguments and variables. reference.md needs no change: its row already says "the child has ambient authority" and "an allowed shell is full authority".

### F2. "Closes the program-independent escapes" overclaims: glibc honors more loader variables than `LD_` (misleading docs, blocking)

**Where.** research/40:208–210 ("closes the program-independent escapes (`PATH`, loader variables)"), research/40:181–185 and :222, 03:302–303, 01:514 ("loader variables refused"), and companion 20:645.

**Attack.** Under `only(["git"])`, the command sets `GCONV_PATH=/dir`, where `/dir` holds a `gconv-modules` file and a shared object that the code wrote through its own `Fs`, or that already exists. Any dynamically linked glibc program that calls `iconv_open` for a charset not built into glibc then loads that object. git does this for `i18n.commitEncoding` and the log encodings, and glib-based programs do too. That is the CVE-2021-4034 vector, and it has the same strength as `LD_PRELOAD`: both need a shared object on disk. glibc strips a fixed list for setuid programs for this reason: `UNSECURE_ENVVARS` in glibc's `sysdeps/generic/unsecvars.h` is `GCONV_PATH`, `GETCONF_DIR`, `HOSTALIASES`, `LD_*`, `LOCALDOMAIN`, `LOCPATH`, `MALLOC_TRACE`, `NIS_PATH`, `NLSPATH`, `RESOLV_HOST_CONF`, `RES_OPTIONS`, `TMPDIR` and `TZDIR`. That list is the closest thing to a canonical set of program-independent variables. The design refuses only the `LD_` and `DYLD_` prefixes, which is Deno's set and narrower than glibc's own.

**Impact.** The docs claim a closure that doesn't exist, and a loader escape of the same class as the one refused stays open.

**Minimum fix.** Recommended: under an attenuated `Proc`, refuse every variable whose name starts with `LD_` or `DYLD_`, or that is in glibc's `UNSECURE_ENVVARS` list, cited by source path and pinned glibc version. It is the same mechanism and the same `PermissionDenied`, and it keeps the claim true. Update PROC-2's text to "an `LD_`/`DYLD_` or glibc unsecure variable". The alternative is to keep the prefix rule and drop "program-independent escapes" from research/40 §4, saying instead: "`only` refuses `LD_` and `DYLD_` variables. Other variables that make the C library load code, such as `GCONV_PATH`, pass through, as program-specific ones do."

### F3. The promised trust-diff wording has no owner text (misleading docs, blocking)

**Where.** research/40:316–318 ("`tide review` lists new paths to `proc` first (03 §4.3). Its line says the kind is ambient for the child."). 03 §4.3 (03:1207–1226) owns the trust diff format, and the candidate does not change it. The only `proc` statement there is "New paths to `proc` ... are listed first" (03:1225).

**Attack.** In 03 §4.3's format, a reviewer sees `+ dep::x  public  proc  via app.proc` / `reaches: main: root.proc.only(["git"])`. That reads as a narrow grant. The bead's Scope 2 requires the ambient-for-the-child fact to be "stated plainly in the trust diff". Only research/40 promises it, and research/40 is not the owner, so stage 2 has no contract to implement.

**Minimum fix.** In 03 §4.3, after "New paths to `proc` ... are listed first", add: "A `proc` line also says that the child is not capability-checked and names the reaching `only` list as programs, not a sandbox, for example: `proc (child is ambient; only names programs, 03 PROC-ONLY)`." Then point research/40 §8 at that text.

### F4. A child outlives the call that lent the authority, and the Tide process (limit to state, non-blocking)

**Where.** 03:375 and :2318, research/40:254 and companion 20:620 describe drop semantics ("dropping a `Child` neither kills nor reaps"). None of them draws the authority consequence.

**Attack.** `fn helper(proc: &Proc)` runs `proc.only(["sh"]).command("sh").args(["-c", "sleep 600; curl ..."]).spawn()?` and drops the `Child`. Everywhere else in Tide, lending `&Fs` or `&Net` for one call bounds the callee's authority to that call: there are no globals, and tasks are structured. A spawned child keeps the process's ambient authority after `helper` returns and after `main` exits. Even kill-on-drop would not close this, since `setsid cmd &` escapes it.

**Limit text** (03 Processes note and companion 20 Authority bullet): "Process authority has no end. A child, and anything it starts, keeps running with the process's authority after the call that started it returns, and after the Tide process exits. Lending `&Proc` or a `Command` for one call is not a temporary grant."

### F5. PROC-ONLY's non-program checks aren't stated as backend-independent or ordered (limit to state, non-blocking; stage 2 acceptance)

**Where.** research/40:155–160 orders only the program check ("before any resolution, host access or model lookup"). The loader-variable refusal (research/40:181–185) says "at spawn" and gives no order or backend. PROC-2 (rules.md) doesn't say "both backends" for it. 03 §9.9 models declare "accepted environment".

**Attack.** A model that accepts any environment, combined with a command that sets `LD_PRELOAD` under `only`, gives sim success where real returns `PermissionDenied`. The same shape applies to NUL in the program, arguments, environment or `current_dir`, where Rust's real `spawn` returns `InvalidInput` and an unchecked model may accept it, and to an empty program.

**Fix text** (03 PROC-ONLY): "On both backends, `spawn`, `output`, and `status` check, in order: the program against `only`; the command's final explicitly set environment against the refused variables (F2); and NUL bytes in the program, arguments, environment, and directory (`InvalidInput`, as Rust does). Only then does lookup or model lookup run." PROC-2 should exercise the loader and NUL refusals under simulation as well.

### F6. `current_dir` can change the program through per-process magic links, and lookup binds a path, not a file (limit to state, non-blocking)

**Where.** research/40:223 ("it cannot change which program starts"), 03:299–300 ("neither the working directory nor the command chooses the program"), research/40:169.

**Attack.** Tide resolves a bare name in the parent, then the child calls `chdir(current_dir)` and execs the absolute path. The kernel resolves that path a second time, in the child. If an absolute entry or a start-time `PATH` element runs through `/proc/self/cwd` or `/proc/thread-self/cwd`, that second resolution follows the child's new directory, so `current_dir` chooses the program. A `#!` line with a relative interpreter (`#!./interp`) likewise resolves against the child's directory. And between lookup and exec, a replaced file runs. That part is already covered by "whoever can replace the file chooses".

**Limit text:** "The claim holds for ordinary absolute paths. An entry or `PATH` element through a per-process link such as `/proc/self/cwd`, or a script whose interpreter path is relative, resolves after `current_dir` applies. Lookup selects a path, and whatever is at that path when the child execs runs." Optionally, have lookup skip `PATH` elements under `/proc/`.

### F7. The descriptor sweep must fail closed (limit to state, non-blocking; stage 2 requirement)

**Where.** research/40:224 and 03:310–312.

**Attack.** `close_range(3, ~0U, CLOSE_RANGE_CLOEXEC)` exists only on Linux 5.11 and later, and kernels before that return `EINVAL`. An implementation that ignores the error leaks descriptors that the process inherited, or that an `ffi` call opened, into the child. That would silently void the "exactly three descriptors" statement.

**Fix text:** "If the sweep fails, the child exits before exec and `spawn` returns the error; a child never execs with an unswept table." The sweep must run after Rust's `dup2` of stdio. It doesn't need to spare Rust's internal exec-error pipe, which is already close-on-exec.

### F8. Admission admits every public item of Tide's std by default (limit to state, non-blocking; stage 2 requirement)

**Where.** `crates/tide-admit/src/walk.rs:337–360`: "Tide's std's own items unless a rule says otherwise". `decide(.., role == Role::TideStd)` admits any unlisted `tide_std::` key. research/40 §2 says "to the checker: no new classification".

**Attack.** CAP-1 holds for process creation only because `tide_std::process` exposes no public path to a process except through a `Proc`-made `Command`. Any of these would be admitted with no rule and no trust hole: a `pub` lookup helper, a backend constructor, a `#[doc(hidden)] pub` item for `tide-macros` or the conformance stub, or a re-export of a Rust process trait (re-exported traits' items are admitted, walk.rs:343–345).

**Fix (stage 2 acceptance):** a test that lists `tide_std::process`'s public items and fails on additions, or an `allow` list for the profile followed by `deny("tide_std::process::*")` in `policy.rs`. Keep backend entry points `pub(crate)` or under `__rt`, which `deny("tide_std::__rt::*")` already covers. Any new type that can hold a handle goes into `NOT_PURE`.

### F9. argv[0] is unspecified (limit to state, nit)

**Where.** research/40:169 ("executed by that absolute path").

Rust's `Command::new(abs)` sets argv[0] to the absolute path, while Rust's `Command::new("git")` sets it to `git`. Multicall binaries and usage text read argv[0]. Neither leads to an escape. **Fix text:** "argv[0] is the program as spelled in `command`, on both backends; models match arguments after argv[0]."

## What held

| Attack | Result | Stopped by |
| --- | --- | --- |
| `std::process::Command::new`, `exit`, `abort` | rejected | `policy.rs:144–146`, TD0501 (CAP-1); stage 2 keeps TD0501 over E0599 (research/40 §8) |
| Other Rust process items: `std::os::unix::process::CommandExt::{pre_exec, exec, arg0}`, `Command::spawn` on Rust's type, `std::process::Child` | rejected | unlisted Rust-std keys get TD0520 (`walk.rs` `decide`, `unlisted_admitted` false for `RustStd`) |
| `libc`, `nix` or any other crate | rejected | `Role::Other`, TD0520 (`walk.rs` `admit_def`) |
| `extern "C" { fn execve/posix_spawn/system }`, `asm!` syscall | needs the `unsafe` grant; listed as a trust hole | `walk.rs` foreign-item and foreign-mod handling, UNSAFE-1 |
| Build script, third-party proc macro, `include!`/`env!` | rejected | TD0915, D7 (toolchain proc macros only), RUN-2 |
| Fake Tide std or stub std as `std` | keyed by crate identity, not name | `crates.rs` roles |
| Runtime entry and `Root` forgery | rejected | `deny("tide_std::__rt::*")`, ROOT-4 private field |
| Simulated `Root`/`Proc` in production code | starts no host process | research/40 §6 coverage rule; models only |
| Test code starting a host process | impossible | tests get simulated capabilities only (RUN-1/RUN-3) |
| Command-level `PATH` choosing the binary | blocked | Tide lookup on start-time `PATH` (PROC-ONLY) |
| Relative `PATH` elements, empty elements, `current_dir` plus a bare name | blocked | absolute-only lookup, done before `chdir` (except F6) |
| `./tool`, `bin/tool`, `..`, trailing slash, `//usr/bin/git`, case, Unicode forms, non-UTF-8, NUL | refused or no match | byte-exact match; invalid entries grant nothing |
| `LD_PRELOAD`, `LD_AUDIT`, `LD_LIBRARY_PATH`, `DYLD_*` on the command | refused under `only` | PROC-ONLY loader rule (see F2 for what's still missing) |
| Smuggling `LD_` through key bytes (`=` in the key, NUL, case) | no effect | glibc reads the prefix case-sensitively; Rust rejects NUL |
| Setting then removing a variable, or changing the env after the check | no effect | check at spawn on the final `Command`; `&mut` borrow excludes races |
| Inherited fds (systemd sockets, jobserver, FFI fds) | not passed | close-on-exec sweep (F7: must fail closed) |
| `Stdio::from(File)` widening authority | no widening | the child is ambient already; the fd's access mode only limits that fd |
| TOCTOU on the program spelling | none | the program is fixed at `command`; Rust has no setter |
| Intersection widening; `only([])`; all-invalid lists; duplicates; `git` vs `/usr/bin/git` | cannot widen; empty allows nothing | `Option<list>` intersection as in `Net::only` (`net.rs:144`); aliases fail closed |
| Getting an unattenuated `Proc` back | impossible | only `Root` holds one |
| Sim success where real refuses on the program | no | `only` before model lookup (03 §9.9); `missing()` gives `NotFound` only for permitted programs |
| Unregistered program, unsupported mode, `--allow-unmodeled` | coverage failure, never a host process | research/40 §6 |
| Replay manifest changing models or profile | refused | REPLAY-2; model data inline (research/40 §7) |
| `Command`, `Child`, pipe ends and `Stdio` hiding from declared authority | reported | `pure.rs` `NOT_PURE` (kind `proc`); deep field walk; `dyn` without `Pure` gives `opaque` (03 §4.1) |
| `&mut dyn Write` wrapping a shell's `ChildStdin` | not authority-free | a non-`Pure` `dyn` is opaque (03 §3.1) |
| `Child::id` as authority | none | no kill-by-PID API; a simulated PID is data |
| Cancelled `output`/`status` leaving an orphan | killed and waited, shielded | 03 §1.1 (descendants excepted, as stated) |

DONE
