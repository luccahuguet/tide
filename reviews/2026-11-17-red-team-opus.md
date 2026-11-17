# Red team of admission and the RUN-3 sandbox (fx-qlek.5)

Claude Opus 5.5, 2026-11-17, base `fab99b3`. The owner authorized this defensive test of Tide's
own checker and sandbox (bead fx-qlek.5). The claim under test: code that `tide-admit` admits
reaches no authority it was not handed (03 RUN-1), and a test child that escapes the type system
anyway meets the kernel (03 RUN-3).

## Result

48 routes tried, counting the accepted controls for route 6. Two forms of one admission route
were admitted and are fixed here: the trait-object overlap of rust-lang/rust#57893, which gives
safe code a generic `transmute`. The blanket form, an impl for a type parameter, was admitted at
`fab99b3`. The projection form, an impl for `<P as Tr>::Assoc` (route 6b, found by the Opus r2
review), was still admitted at `6a30b7c`. Every other admission route this record lists is
rejected. Two routes stay admitted on purpose: one writes only to
standard error, which every program holds as a sink (SINK-1), and one is an inert crate
attribute. Every RUN-3 route gets the denial RUN-3 specifies. Separately, `std::path` was an
allowed glob with a ten-item denylist (Achilles W5). It now lists each item, so an I/O method that
a later rustc adds is not admitted until the pin review lists it.

Revised after the Codex pass (r1): the `FX0402` check now looks only at the trait's own
associated types and ignores those that require `Self: Sized`. The `std::path` list gains two
stable items it had missed. The process-memory and loader probes now have an unsandboxed
control.

Revised after the Opus review (r2): `FX0402` also checks an impl whose self type is a projection
(route 6b), and its message says "may overlap". The record adds the review's missed routes: M1 is
route 6b, and M2 to M5 are routes 42 to 45 and a `Component` match in `escape_path_io`, which runs the variant and
prefix keys through the driver.

Fixes:

- `FX0402` (REF-0402, `crates/tide-admit/src/walk/traits.rs`): an impl is rejected when three
  things hold:
  - its trait is dyn-compatible;
  - the trait itself declares an associated type without `where Self: Sized`;
  - its self type is a type parameter that may be unsized.

  Such an impl also covers the trait's `dyn` types, and their built-in impl takes that associated
  type from the `dyn` type, so rustc lets the two impls disagree. Some impls stay admitted:
  - an impl for a sized parameter;
  - an impl of a trait without associated types (`impl<T: Value + Pure + ?Sized> PureValue for T`
    in the `pure_*` cases);
  - an impl of a trait whose associated types are all inherited. Codex's `Bytes: Iterator<Item =
    u8>` is an example. A supertrait's impls decide those types, and the check sees those impls
    where they are written;
  - an impl of a trait whose associated types all require `Self: Sized` (Codex's `Describe`).

  The last two are in `memory/escape_dyn_overlap_accept`.
- `std::path` is listed item by item (`crates/tide-admit/src/policy.rs`, unit test
  `std_path_admits_listed_items_only`). At the pin, every I/O item was already denied.
  Checked against the pinned nightly's `library/std/src/path.rs`, the list holds every stable
  item. The first version missed `Path::is_empty` and `PathBuf::into_string` (stable since
  1.98.0); the reference allowlist row now witnesses both against the pinned driver and
  production std. The items left unlisted are unstable, and rustc rejects them anyway:
  `SEPARATORS`, `SEPARATORS_STR`, `NormalizeError`, the trailing-separator methods,
  `Path::trim_prefix`, `Path::as_path`, `Path::absolute`, and `Path::normalize_lexically`.

No admitted route is left open, so this review proposes no P1 blocker bead.

## Routes

Verdict codes: `FXnnnn` is the driver's verdict and `E`/`error` is rustc's. All run against the
stub std through `conformance/run.sh`. The RUN-3 routes run in
`crates/tide-admit/tests/build.rs::run3_kernel_denies_exec_fork_io_uring_ptrace_and_constructors`
on Linux 7.2.6, Landlock ABI 10, `sandbox: full`.

| # | Route (input) | Expected | Actual | Test |
| --- | --- | --- | --- | --- |
| 1 | `core::env!("HOME")` in a `const` | rejected | rejected, FX0520 | `authority/escape_build_reads` |
| 2 | `core::include_bytes!` | rejected | rejected, FX0520 | `authority/escape_build_reads` |
| 3 | `include_str!(concat!(..))`, a computed path | rejected | rejected, FX0520, FX0103 | `authority/escape_build_reads` |
| 4 | `#[doc = core::env!("HOME")]` | rejected | rejected, FX0520 | `authority/escape_build_reads` |
| 5 | `#![debugger_visualizer(gdb_script_file = ..)]`, a stable compile-time file read | rejected | rejected, FX0911 | `modules/escape_debugger_visualizer` |
| 6 | rust#57893: `impl<T: ?Sized, U> Object<U> for T`, then `dyn Object<U, Output = T>`, a safe `transmute` aimed at `&Fs` | rejected | **admitted at fab99b3**; rejected, FX0402 after the fix | `memory/escape_dyn_overlap_transmute` |
| 6b | the projection form: `impl<P: Tr + ?Sized> Object<P> for <P as Tr>::Assoc`, with `Assoc = dyn Object<Key<T, U>, Output = T>`, a safe `transmute` aimed at `&Fs` | rejected | **admitted at 6a30b7c** (rustc accepts it, so it is not a rustc-rejected witness); rejected, FX0402 after widening the check to projections | `memory/escape_dyn_overlap_transmute` (`mod projection`) |
| 6a | controls for 6: a blanket impl of a trait with only inherited (`Bytes: Iterator<Item = u8>`) or `Self: Sized` associated types | admitted | admitted | `memory/escape_dyn_overlap_accept` |
| 7 | rust#25860 through an `F: Fn` bound (no `fn`-pointer coercion) | rejected | rejected, E0491 + error | `memory/escape_fn_trait_routes` |
| 8 | rust#25860 through `&dyn Fn` | rejected | rejected, error | `memory/escape_fn_trait_routes` |
| 9 | rust#25860 through `impl Fn` | rejected | rejected, error at the call | `memory/escape_fn_trait_routes` |
| 10 | rust#25860 through a closure coerced to `fn` | rejected | rejected, error | `memory/escape_fn_trait_routes` |
| 11 | rust#84591, a higher-ranked subtrait bound | rejected | rejected, FX0201, FX0404 + error | `memory/escape_hrtb_supertrait` |
| 12 | rust#100051, implied bounds read from an unnormalized projection | rejected | rejected, error at the call | `memory/escape_projection_implied_bounds` |
| 13 | `std::panic::set_hook` | rejected | rejected, E0433 (Tide's std has no `panic` module) | `authority/escape_hooks` |
| 14 | `#[global_allocator]` with an `unsafe impl GlobalAlloc` | rejected | rejected, FX0901, FX0520 | `authority/escape_hooks` |
| 15 | `#[unsafe(export_name = "malloc")]`, symbol interposition | rejected | rejected, FX0901 | `authority/escape_linkage` |
| 16 | `#[used] #[unsafe(link_section = ".init_array")]` static constructor | rejected | rejected, FX0901, FX0502 | `authority/escape_linkage` |
| 17 | all 11 `Path` I/O items at the pin, through `&Path`, `PathBuf`'s deref, a qualified path, and `std::path::absolute` (W5) | rejected | rejected, FX0520 | `authority/escape_path_io`; `policy::tests::std_path_admits_listed_items_only` |
| 18 | `core` atomics: a `static`, `Default`, operations | rejected | rejected, FX0520, FX0502 | `authority/escape_atomics` |
| 19 | `std::os::unix::net::UnixStream::connect` | rejected | rejected, E0433 | `authority/escape_os_routes` |
| 20 | `std::backtrace::Backtrace::force_capture`, which reads the environment and debug info | rejected | rejected, E0433 | `authority/escape_os_routes` |
| 21 | DNS through `("host", 80).to_socket_addrs()` | rejected | rejected, E0599 (Tide's `ToSocketAddrs` is sealed and has no method) | `authority/escape_os_routes` |
| 22 | `Termination::report` on an `Err`, which writes to standard error | admitted (SINK-1) | admitted | `authority/escape_stderr_report` |
| 23 | `extern crate proc_macro;` and `extern crate alloc as real;` | rejected | rejected, FX0912 | `modules/escape_extern_crates` |
| 24 | `#![crate_type = "cdylib"]` | admitted, inert (the driver's `--crate-type` wins) | admitted | `authority/escape_crate_type_accept` |
| 25 | `#![crate_name = "renamed"]` | rejected | rejected, error | `authority/escape_crate_attrs` |
| 26 | RUN-3: `execve("/bin/sh")` | Landlock denies | `EACCES` | `run3_kernel_denies_exec_fork_io_uring_ptrace_and_constructors` |
| 27 | RUN-3: `fork` | seccomp denies | `EPERM` | same |
| 28 | RUN-3: `clone(SIGCHLD)`, a process | seccomp denies | `EPERM` | same |
| 29 | RUN-3: `clone3` | `ENOSYS` | `ENOSYS` | same |
| 30 | RUN-3: `io_uring_setup` | seccomp denies | `EPERM` | same |
| 31 | RUN-3: `ptrace(PTRACE_TRACEME)` | seccomp denies | `EPERM` | same |
| 32 | RUN-3: `memfd_create` (the start of a memfd exec) | seccomp denies | `EPERM` | same |
| 33 | RUN-3: `unshare(CLONE_NEWUSER)` | seccomp denies | `EPERM` | same |
| 34 | RUN-3: `kill` of a process of the same user outside the sandbox | seccomp denies | `EPERM` | same |
| 35 | RUN-3: `tgkill` of that process | denied | `EPERM` (seccomp allows `tgkill`, so Landlock's signal scope is the likely layer, but the test does not tell them apart) | same |
| 36 | RUN-3: open `/proc/self/mem` for writing | Landlock denies | `EACCES` | same |
| 37 | RUN-3: `socketpair(AF_UNIX)` | seccomp denies | `EPERM` | same |
| 38 | RUN-3: `prlimit64` setting a limit | seccomp denies | `EPERM` | same |
| 39 | RUN-3: `process_vm_readv` of one byte of the child's own memory, which succeeds when the binary runs directly | seccomp denies | `EPERM` (directly: `Ok(1)`) | same |
| 40 | RUN-3: an `.init_array` constructor reads a canary outside the run, then forks | the sandbox covers constructors | `EACCES`, `EPERM` | same |
| 41 | RUN-3: `execve` of the binary's own ELF interpreter on `/bin/sh` (the loader is granted execute) | the loader starts but cannot read the shell | the exec does not return; the loader reports "/bin/sh: cannot open shared object file: Permission denied" and the run fails. Run directly, the loader runs the shell | same |
| 42 | `#[cfg_attr(all(), path = "..")] mod hidden;`, a `#[path]` that `cfg_attr` produces on an out-of-line module (M2) | rejected | rejected, FX0911 | `modules/escape_cfg_attr_path` |
| 43 | `#[global_allocator]` without its path (M3) | rejected | rejected, rustc error (Tide's prelude does not export the attribute) | `authority/escape_hooks` |
| 44 | `#[unsafe(no_mangle)] pub extern "C" fn free` (M4) | rejected | rejected, FX0901 | `authority/escape_linkage` |
| 45 | RUN-3: `vfork` and `execveat("/bin/sh")` (M5) | seccomp denies | `EPERM`, `EPERM` | `run3_kernel_denies_exec_fork_io_uring_ptrace_and_constructors` |

Routes the corpus already covered, which this review re-read but did not repeat: the fx-6xi2
`fn`-pointer forge (`memory/closure_fnptr_capability`, FX0206), the runtime entry
(`authority/admit_runtime_entry`), `#[path]` (`modules/path_reject`), `#![no_std]`
(`authority/admit_no_std`), unstable features (`authority/admit_feature`), assembly
(`authority/unsafe_global_asm`), forging `Root` by `zeroed`, `Default`, or `Decode`
(`authority/root_*`), ambient `Command`, `TcpStream`, `env` and `fs` calls
(`authority/cap_ambient_paths_reject`), RUN-3 paths, sockets, and stdout truncation
(`run3_kernel_denies_an_unadmitted_test_child_paths_and_sockets`).

## Reading the sandbox

The seccomp filter (`invocation/sandbox.rs`) checks the architecture first and kills
non-x86_64 calls; x32 calls fall through to `EPERM`. `clone` passes only with `CLONE_THREAD` and
no namespace flag, and `prlimit64` only with a null new limit. `ioctl`, `fcntl`, and `prctl` are
narrowed to a few commands. `execve` is allowed, and Landlock then confines it to the binary, the
loader, and the libraries. A process can still run the granted loader on a granted library, which
executes nothing new. Calls the runtime does not use, such as `getppid` and `uname`, fail with
`EPERM`. The probe met this when its first `tgkill` target came from `getppid`. RUN-3 already
states that metadata reads (`stat`, `access`) are not confined.

## Left for others

- `design/reference.md` §5.1's `FX0402` row, and 01 D4's "no overlapping impls", could name the
  #57893 rule: an impl for a type parameter or a projection that may be unsized, of a
  dyn-compatible trait with its own associated type. This bead does not hold the reference.
  tide-lead could record this on fx-djp or on fx-qlek.7's launch-claims audit.
- `FX0402` fails closed. It also rejects an impl whose bounds no `dyn` type can meet, such as
  `impl<T: ?Sized + Hash> Key for T` (`Hash` is not dyn-compatible). A user subtrait can make any
  dyn-compatible bound satisfiable, so a precise rule would have to check every bound; this
  record leaves that as a possible relaxation.
- The pin-bump audit (Achilles W5) still has to list every new `core`, `alloc`, and `std` item.
  The `std::path` list now fails closed, but `core::*` globs such as `core::mem::*` and
  `core::cell::*` still admit whatever a later rustc adds under them.
