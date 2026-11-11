Model: claude-opus-5-5 · Effort: high · Date: 2026-11-11

Verdict: approve_with_alternatives

Scope: the frozen candidate 803b164 on 0cfe7b2. `git diff 0cfe7b2 803b164 | sha256sum` gives
6921ff02e642786b2f9fb382714086f243d8e3117f54b670bbf594bf06262c53, which matches the brief.
This was a read-only review. I ran no cargo build. I ran one focused probe: a Python
`ctypes` openat2 call against `/proc/self`, with the load/memory/quiet gate open (load 4.86,
18 GB available). Nothing from `trials/e1-checked/` was opened.

Summary. I found no confinement escape. Every confined resolution goes through
`openat2(base, …, BENEATH|NO_MAGICLINKS)`, and the code checks for absolute paths first. Mutations
open the parent beneath the base and then act on one normal component. Recursive deletion
walks pinned no-follow descriptors. `write_synced` expands links again from the base every time, and it
opens the final name `NOFOLLOW` beneath the pinned parent. `dir()` checks the mode before it
resolves, and `read_only()` keeps the same base, so attenuation cannot widen. Read-only mode is enforced
in the shared wrapper before any backend call. A read-only `File` keeps `write = false`, and
its fd is `O_RDONLY`. Every syscall attempt goes through `call()`, which checks
cancellation once per attempt: EINTR and EAGAIN retries included. The std `read`/`write`/`lseek` paths
are single syscalls behind `File::check`, and `close` is shielded. Durability is file fsync and
then pinned-parent fsync, which is correct. The OpenOptions validation matches std's
`get_access_mode`/`get_creation_mode`. Real authority panics inside a simulation at every
checkpoint and at `close`. `root_zeroed` still reaches FX0517, and a zeroed `Fs` is now a
valid read-only root `Fs`, so the check still matters. `pure.rs` already lists `File`,
`ReadDir` and `DirEntry` as not `Pure`. Conformance includes production `fs.rs`
verbatim, so the two are equivalent. The findings below are about error kinds, documentation accuracy,
one information leak, and ownership for fx-byk.2. None of them is an escape.

1. **Non-blocking — a magic-link escape returns ELOOP, not `PermissionDenied`.**
   Evidence: `crates/tide-std/src/fs/real.rs:78` passes `BENEATH | NO_MAGICLINKS`. The kernel
   checks `NO_MAGICLINKS` in `nd_jump_link` before the scoped-lookup check. A probe on this host
   (Linux 7.2.6) used `openat2(open("/proc/self"), p, O_RDONLY, how)` with
   p ∈ {`root/etc`, `cwd`, `fd/0`, `exe`}. With `BENEATH` alone, every case gave EXDEV. With
   `BENEATH|NO_MAGICLINKS`, every case gave ELOOP. So in `root.fs.dir("/proc/self", Read)`, a call such as
   `.read("root/etc/hostname")` fails closed, but with raw ELOOP, which has no stable kind. The bead's Acceptance
   and reference §6.3 say a symlink that would leave `p` fails with `PermissionDenied`. The
   `fs.rs:3` module doc says the same. `write_synced` on the same path gives `PermissionDenied`
   instead (it follows the magic link's `readlink` text, which is absolute), so the two methods disagree.
   Fix: keep the flag, as decision 1 and the man page advise. Then either (a) state in 03 §1.1 and
   the `fs.rs` doc that a magic link gives ELOOP (provisional), or (b) drop `NO_MAGICLINKS`,
   since `BENEATH` already rejects every magic-link jump with EXDEV (shown by the probe), and record why on
   the bead. I recommend (a): it is a text-only change and keeps the defence that decision 1 chose.

2. **Non-blocking — `File`'s derived `Debug` makes unchecked syscalls and reveals the host
   path.** Evidence: `crates/tide-std/src/fs.rs:298` `#[derive(Debug)] pub struct File`, which wraps
   `std::fs::File` (`fs.rs:305`). On the pinned nightly, std's `File` Debug calls
   `readlink("/proc/self/fd/N")` and `fcntl(F_GETFL)`
   (`library/std/src/sys/fs/unix.rs:317-322`). So `{:?}` on a `File` opened through a
   confined `Fs` prints the base's absolute host path, which tells a holder about directories
   outside its authority. It also does file-system I/O with no checkpoint and no
   `outside_simulation` check. cap-std's handle types avoid printing paths for this reason.
   Fix: write `Debug` for `File` by hand, showing only its access flags (for example
   `File { read, write }`), and do not delegate to std's.

3. **Non-blocking — path validation has no single shared owner, contrary to decision 3.**
   Evidence: the shared wrapper calls `check_path` only for `rename`, `remove_file` and
   `remove_dir` (`fs.rs:241-242`, `:254`, `:265`). For `open`/`create`/`open_with`, `read*`,
   `write`, `metadata`, `exists`, `read_dir`, `sync_dir`, `dir`, `create_dir_all`,
   `remove_dir_all` and `write_synced`, the absolute-path and NUL checks happen only inside
   `real.rs` (`:67`, `:190`, `:225`, `:267`, `:331`). `rename` and the removals check twice (shared, then
   `real::Fs::parent` at `:190`). Decision 3 puts "path checks" in shared code in front of both
   backends, so fx-byk.2's simulated backend would have to repeat them, which risks divergence
   on the kind that equivalence compares. Fix: call `self.check_path(p)` in the shared wrapper for every
   path-taking method, in the research's order (cancellation, Mode, path). Then delete
   `real::Fs::check_path` and keep only the `write_synced` symlink-target check in `real.rs`.

4. **Non-blocking — "`dir()` returns `Unsupported` off Linux" is not true; off Linux, tide-std does not build.**
   Evidence: `names()` uses `fs::RawDir` (`real.rs:399`), and rustix 1.1.6 exports that only under
   `cfg(linux_kernel)` (`src/fs/mod.rs:51,117`). `sync_data` uses `fs::fdatasync` (`real.rs:356`),
   which rustix leaves out on Apple targets (`src/fs/fd.rs:278-289`). So the
   `#[cfg(not(target_os = "linux"))]` branches (`real.rs:82`, `:96`, `:141`) are dead
   code, and 03 §1.1 (`design/03-authority-concurrency.md:110-111`), the companion (`:406`) and the research describe behaviour that cannot
   occur. Linux-only is fine for v0, as Codex noted. The documentation is the problem. Fix: delete the
   non-Linux branches, and say "real files are Linux-only in v0; tide-std does not build on other targets",
   or make the module `cfg(target_os = "linux")` and leave the alternative to whoever adds a platform.

5. **Non-blocking — the dependency proof is not recorded anywhere durable (AP-DEPENDENCY-001, AP-PROOF-001).**
   Evidence: decision 2 asks that `cargo tree -p tide_std` be recorded. The result (rustix 1.1.6
   brings in only `bitflags 2.13.2` and `linux-raw-sys 0.12.1` on Linux; the license is
   Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT) appears only in the cache file
   `report.md:68-71`. The landed `research/19-files.md:50` still says "*(memory; confirm with `cargo tree`)*"
   and "its `close` reports no error *(memory)*". The second claim is true: `rustix::io::close` returns `()`, which is
   why the code declares `extern close`. Fix: when recording the bead, put the cargo-tree, license and close facts on
   fx-byk.1, or add one dated "Confirmed" line under research §1.2. The research body itself can stay as it was drafted.

6. **Nit — `flush` on a read-only `File` returns `PermissionDenied`.** Evidence: `fs.rs:403`
   `self.check(self.write)`. std's `File::flush` is always `Ok(())`. 03 §1.1 records the
   `PermissionDenied` departures only for read on a write-only file, and for write and `set_len` on a read-only one. Fix: use
   `self.check(true)` so the call stays a checkpoint, or add flush to 03's provisional list so fx-byk.2 matches it.

7. **Nit — shared `FileType` uses rustix's discriminants.** Evidence: `fs.rs:539-549`
   compares `self.0` with `real::FileType::Directory as u32`, and `real.rs:365` constructs it the same way. fx-byk.2's
   simulated backend would have to build `FileType` from the real backend's rustix enum. Fix: a private
   shared `enum Kind { File, Dir, Symlink, Other }`, with real.rs mapping onto it.

8. **Nit — leftover in the research file.** `research/19-files.md:215` ends with the agent
   report's terminator, `DONE`. Delete that line. The report's "body unchanged" equality check carried it over.

Notes on things I checked and found correct (no findings):
- `remove_dir_all(".")` and `"sub/.."` empty the named directory and then return
  `InvalidInput`. That matches std's algorithm (`openat` dir, empty it, `unlinkat` fails) and 03's
  sentence. `"link/"` returns `NotADirectory` without following, which is stricter than std and safe.
- `create_dir_all`'s kinds (existing file → AlreadyExists, file parent or trailing `.` →
  NotADirectory, dangling link → AlreadyExists, `""` → Ok) match std's `create_dir_all`.
- EXDEV becomes PermissionDenied only in `confined_open`, while renameat's EXDEV stays CrossesDevices.
  EAGAIN is retried 4 times, and each retry passes a checkpoint. ENOSYS gives Unsupported and there is no lexical fallback.
- `names()`: EINVAL means the buffer is too small, so a new `RawDir` on the same fd continues
  at the right offset, because getdents runs only once the buffer is drained.
- Tests: the admitted executable exercises escapes by `..`, a relative link, an absolute link and
  an absolute path, for read, write, write_synced, metadata, exists, read_dir, dir, mutations and
  removal. It also covers pinning across a rename, read-only rejection on missing paths, `dir(_, ReadWrite)`
  on a Read `Fs`, append at the end, set_len in append mode, cancellation of every method, and the shielded close. The
  unit tests cover the cancellation seam on EINTR and the NOSYS/XDEV/AGAIN mapping.
  Magic links, finding 1, are the only escape class with no test.

DONE
