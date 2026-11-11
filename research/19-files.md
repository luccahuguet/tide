# Capability-confined and simulated files (research for fx-byk.1, fx-byk.2)
Drafted by Fable on 2026-11-10; its decisions are recorded on fx-byk.1 and fx-byk.2.

Research report, 2026-11-10. Sources: the beads (fx-byk, .1, .2, fx-ecb, fx-nkv.5/.6 decisions, fx-o8j), `design/03` §1.1, §6.2, §9, `design/reference.md` §6.3, §6.8, the companion, `crates/tide-std`, the pinned nightly's std source, and the web pages cited inline. Nothing from `trials/e1-checked/` was opened. No build was run; claims marked *(memory)* were not checked against a page or the tree and the implementer should confirm them.

## 0. What is already fixed

- Surface: reference §6.3 and 03 §1.1 list every method. `Fs`, `File`, `ReadDir`, `DirEntry`, `Metadata`, `FileType` are Tide's own types with Rust's methods.
- Confinement: "`openat2(RESOLVE_BENEATH)` semantics: `..` and symlinks cannot leave it" (03 §1.1); an absolute path, or an escaping `..` or symlink, fails `PermissionDenied` (provisional kind); `root.fs` is rooted at `/`, relative paths resolve against the starting directory (`/` under simulation).
- Cancellation: every operation is a checkpoint except `File::close`; an uncancellable system call runs to its end and returns its own result; multi-call operations check before each call (TASK-3).
- Simulation: each file operation is a scheduling point and takes up to 1 ms (provisional); latencies are "drawn from the seed"; "a file-system call's latency runs out first" (03 §9.3). The scheduler uses a per-seed SplitMix64 with domain-separated streams, ids are simulation-local counters, the trace is hand-written NDJSON (fx-nkv.5/.6 decisions 5, 6, 1).
- io errors: Tide's own `io::Error`; an OS error becomes a Tide one through `Error::from_raw_os_error(code)` or a mapping added to `io/shared.rs` (fx-ecb decision).
- Today `Fs` is `Fs(pub(crate) ())`, declared by `capability!` in both production and the conformance stub.

## 1. Design choices

### 1.1 How `dir` confinement is enforced on the real side

Options:

- **A. Lexical**: canonicalize, then check the prefix. Races with a concurrent rename or symlink swap between check and use. The kernel itself returns `EAGAIN` from `openat2` when it "could not ensure that a '..' component didn't escape (due to a race condition or potential attack)" (https://man7.org/linux/man-pages/man2/openat2.2.html), which shows that even in-kernel the check must be tied to the resolution.
- **B. Directory descriptor plus `openat2(RESOLVE_BENEATH)`**: "Do not permit the path resolution to succeed if any component of the resolution is not a descendant of the directory indicated by dirfd. This causes absolute symbolic links (and absolute values of path) to be rejected." An escape gives `EXDEV`. Linux 5.6+ (same page). The page advises passing `RESOLVE_NO_MAGICLINKS` explicitly.
- **C. Per-component walk** with `O_NOFOLLOW`, as cap-std does where the kernel has no such flag: it "opens each component of a path individually, in order to specially handle `..` and symlinks" (https://raw.githubusercontent.com/bytecodealliance/cap-std/main/README.md).

Prior art agrees on the semantics Tide's text already has. cap-std: paths that leave the `Dir` by `..` or symlink return `PermissionDenied`, absolute paths are errors, symlinks that stay inside are followed (same README). WASI: "Absolute paths are not permitted", and a `..` that steps outside is refused "even if the final resolved path is back inside the sandbox" (https://github.com/WebAssembly/wasi-filesystem/blob/main/path-resolution.md). FreeBSD: `openat(O_RESOLVE_BENEATH)` returns `ENOTCAPABLE` on escape (https://man.freebsd.org/cgi/man.cgi?query=open&sektion=2). Go 1.24's `os.Root` follows symlinks but they "may not reference a location outside the root" (https://pkg.go.dev/os#Root). Rust's unstable `std::fs::Dir` does not help: its tracking issue says "Sandboxing is a non-goal" (https://github.com/rust-lang/rust/issues/120426), and in the pinned nightly it has only `open`, `open_file`, `open_file_with`, `remove_file`, `rename` and `metadata` (read from the toolchain's `library/std/src/fs.rs`).

**Recommendation: B, with one real backend.**

- A real `Fs` holds `base: Option<Arc<OwnedFd>>` and a `Mode`. `root.fs` has `None`. `dir(p, mode)` opens `p` through the parent `Fs` with `O_DIRECTORY | O_RDONLY | O_CLOEXEC` (not `O_PATH`, so `sync_dir(".")` can `fsync` it) and keeps the descriptor. The capability then names the directory, not its path: renaming the directory does not redirect it.
- Every operation resolves with one call shape: `openat2(base or AT_FDCWD, path, how)`, where `how.resolve` is `RESOLVE_BENEATH | RESOLVE_NO_MAGICLINKS` for a confined `Fs` and `0` for `root.fs`. For `root.fs` with `resolve == 0` the call equals `openat`, so use `openat` there and do not depend on `openat2`.
- Operations with no open form (`rename`, `remove_file`, `remove_dir`, a `mkdir` step) open the parent directory beneath the base (`O_PATH | O_DIRECTORY`) and call `renameat`/`unlinkat`/`mkdirat` on the last component. That component must be a normal name: with the base as parent, `unlinkat(base, "..")` would address the directory above. These calls do not follow a symlink in the last component, so they act on an entry inside the pinned parent.
- `metadata` and `exists` follow symlinks as Rust's do: `openat2(.., O_PATH)` beneath, then `fstat`. `DirEntry::metadata` does not follow (Rust's does not traverse symlinks): `fstatat(dirfd, name, AT_SYMLINK_NOFOLLOW)`.
- Map `EXDEV` **from the resolving call only** to `PermissionDenied`; `renameat`'s `EXDEV` stays `CrossesDevices` (the nightly maps `EXDEV => CrossesDevices`, `ELOOP => FilesystemLoop`; `sys/io/error/unix.rs`). Check an absolute path on a confined `Fs` in Tide before the call, so the rule does not depend on the kernel's errno.
- `EAGAIN`: retry a small fixed number of times (4), then return the OS error. The man page says the caller may retry.
- `ENOSYS` (kernel older than 5.6, or a seccomp filter): a confined operation fails closed with `Unsupported`. Do not fall back to a lexical check. Option C is a correct fallback but is a second resolver to test; build it only when a target needs it.
- Non-Linux targets: the design names Linux for v0 (03 §2.6). Recommend that `dir()` return `Unsupported` there for now. macOS has `O_RESOLVE_BENEATH` and `O_NOFOLLOW_ANY` constants in XNU's `fcntl.h` (https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/fcntl.h); semantics were not checked.

Reason: it is the mechanism the design text names, one kernel resolution per operation closes the check-then-use race, and one call shape for both the root and confined cases keeps a single owner for path resolution.

Out-of-scope finding: 03 §2.6's seccomp list must admit `openat2`, or confined file access fails inside `tide`'s sandbox. Report to the owner of RUN-3; no change proposed here.

### 1.2 Dependency (AP-DEPENDENCY-001)

Capability needed: fd-relative file calls on Linux (`openat2` with `open_how`, `renameat`, `unlinkat`, `mkdirat`, `fstatat`, directory iteration from a descriptor, `fsync`) and a `close` that returns its result. `tide_std` has one dependency today (`libm`, pinned `=0.2.16`).

| Candidate | Fit | Cost |
| --- | --- | --- |
| Owned code over raw `extern "C"` (std already links libc) | Full control, no crate | Hand-declared `stat` and `dirent64` layouts per architecture; several hundred lines of `unsafe` |
| `libc` crate | Has `open_how` (https://docs.rs/libc/latest/libc/struct.open_how.html); `SYS_openat2`, `RESOLVE_BENEATH` *(memory)* | Every call still `unsafe`; no further crates |
| `rustix` (`fs` feature) | Safe `openat2(dirfd, path, oflags, mode, resolve)` with `ResolveFlags::BENEATH`, `NO_MAGICLINKS` (https://docs.rs/rustix/latest/rustix/fs/fn.openat2.html, https://docs.rs/rustix/latest/rustix/fs/struct.ResolveFlags.html); safe `renameat`, `unlinkat`, `mkdirat`, `statat`, `fs::Dir` iteration *(memory)* | Two transitive crates on Linux, `bitflags` and `linux-raw-sys` *(memory; confirm with `cargo tree`)*; its `close` reports no error *(memory)* |
| `cap-std` | Whole confined API on Linux, macOS, FreeBSD, Windows; `PermissionDenied` on escape; `Dir` has `read_dir`, `rename`, `remove_dir_all`, `create_dir_all`, `open_with`, `metadata` (https://docs.rs/cap-std/latest/cap_std/fs/struct.Dir.html) | Direct deps `cap-primitives`, `io-extras`, `io-lifetimes`, `rustix` (https://docs.rs/crate/cap-std/latest) plus theirs; its own `File`/`ReadDir`/`Metadata` types that Tide wraps anyway; fallback behaviour on old kernels that Tide does not choose |

**Recommendation: `rustix`, pinned exactly, `default-features = false`, features `fs` and `std`**, plus one `extern "C" { fn close(fd: c_int) -> c_int; }` for `File::close`'s result. Rejected: owned raw code (struct layouts are the error-prone part and prove nothing Tide cares about); `cap-std` (most of its weight buys platforms v0 does not target and a second set of handle types). Replacement cost: the real backend is one private module behind `Fs`'s backend enum; moving to `cap-std` when macOS or Windows become targets replaces that module only. Proof of the relied-on behaviour: the escape tests in §3.

### 1.3 Representation: one `Fs` type, two backends

`SimRoot` has "`Root`'s capability fields" (reference §6.8), so `sim.fs` is an `Fs`. Recommend a private enum inside `Fs`: `Real { base, mode }` and `Sim { sim: Arc<Sim>, base: InodeId, mode }`, as `Tasks` carries `Option<Arc<sim>>` (fx-nkv.5 decision 2). `File`, `ReadDir` and `DirEntry` get the same two variants. No trait object: there are two variants and no third caller. The simulated base is an inode id, not a path, to match the real descriptor's behaviour under rename.

The `capability!` macro's `Fs(pub(crate) ())` and `__rt::enter`'s `Fs(())` change; the conformance stub keeps signatures only (it lacks `write`, `open`, `create`, `open_with`, `remove_*`, `File`, `OpenOptions` today and must gain them for the fx-nkv equivalence clause).

Checks that belong to neither backend go in shared code in front of both, so they cannot differ:

- `Mode::Read`: any writing method, including opening with write, append, create or truncate, returns `PermissionDenied` before the backend runs. The real side also opens `O_RDONLY` as defence in depth.
- Cancellation check on entry (and between the steps of a multi-call operation on the real side).
- Path classification (§1.4).
- `File` access flags: see silence S3.

### 1.4 Path model shared by both sides

Go's `io/fs` avoids the problem by rejecting every path with `.`, `..`, an empty element or a leading slash (https://pkg.go.dev/io/fs). Tide cannot: the reference allows `..` that stays inside. So the simulated side needs a resolver with the kernel's rules, and the real side must not normalize before the kernel.

**Recommendation.** No lexical normalization on either side. The simulated resolver walks `Path::components()` physically from the base inode:

- `RootDir`: `root.fs` restarts at `/`; a confined `Fs` returns `PermissionDenied`.
- `Normal(name)`: descend; missing gives `NotFound`; a non-directory in the middle gives `NotADirectory`.
- `ParentDir`: at a confined base, `PermissionDenied`; at `/`, stay; else go up. So `missing/../f` is `NotFound`, as in the kernel, not `f`.
- `CurDir`: stay.
- Empty path: `NotFound` (`ENOENT`). A NUL byte: `InvalidInput`, as Rust's path conversion gives. A component over 255 bytes: `InvalidFilename` (`ENAMETOOLONG`, per the nightly's table).
- A trailing slash on a non-directory is `NotADirectory` in Linux, and `components()` drops it: detect it on the raw bytes.
- Names are byte strings (`OsString`), compared and ordered by bytes; case-sensitive.
- No symlinks or hard links in the simulated store: no Tide method creates either. Symlink tests are real-only.

Shared rule for the last component of `rename`, `remove_file`, `remove_dir`, `create_dir_all`'s steps: it must be a normal name (S2).

### 1.5 Error kinds and their mapping

The contract is the `ErrorKind`. The simulated side builds errors from kinds (`ErrorKind::NotFound.into()`), not from Linux errno numbers: numbers differ by platform, and a seed must give one trace everywhere (03 §9.1). Consequence: `raw_os_error()` is `None` and `Display` text differs between real and simulated errors. Say so in the `Fs` docs; the equivalence suite compares kinds.

Real side: convert `std::io::Error`/`rustix::io::Errno` by raw code through `from_raw_os_error`, falling back to `ErrorKind::from_rust`.

Kinds the simulated store must reproduce (Linux behaviour, kinds per the nightly's errno table):

| Case | Kind |
| --- | --- |
| missing path or parent | `NotFound` |
| non-directory as intermediate, `read_dir`/`dir`/`sync_dir` on a file, `remove_dir` on a file | `NotADirectory` |
| `read`, `create`, write-open, `remove_file` on a directory | `IsADirectory` |
| `create_new` on an existing path; `create_dir_all` over a file | `AlreadyExists` |
| `remove_dir` or `rename` onto a non-empty directory | `DirectoryNotEmpty` |
| `rename` of a file onto a directory / directory onto a file | `IsADirectory` / `NotADirectory` |
| `rename` of a directory into its own subtree; `remove_dir(".")` (03) | `InvalidInput` |
| escape, absolute path on a confined `Fs`, write through `Mode::Read` | `PermissionDenied` |
| cancelled scope | `Cancelled` |
| `read_to_string` on non-UTF-8 | `InvalidData` |

Rust documents `rename` as replacing the target, and on Unix "if `from` is a directory, `to` must also be an (empty) directory" (https://doc.rust-lang.org/std/fs/fn.rename.html). Order of checks when several apply: cancellation, then `Mode`, then path resolution, then the operation.

### 1.6 `read_dir` ordering

Rust: "The order in which `read_dir` returns entries can change between calls", and `.` and `..` are skipped (https://doc.rust-lang.org/std/fs/fn.read_dir.html). Go made the opposite choice for its top-level call: "`ReadDir` reads the named directory and returns a list of directory entries sorted by filename" (https://pkg.go.dev/io/fs).

Options: (a) OS order on real, sorted in simulation: a program that depends on order passes in simulation and fails in production, so the bead's "same results" holds only by luck. (b) OS order on real, seed-shuffled in simulation: exposes the dependence, keeps Rust's rule, but equivalence then holds only for programs that sort. (c) Sorted by file-name bytes on both.

**Recommendation: (c).** `read_dir` reads the whole listing, sorts by name bytes, and returns a snapshot on both sides. Equivalence then holds for every program; production output stops depending on the file system; a `BTreeMap` in the simulated store gives it for free. Cost: memory and time proportional to the directory, and a real listing error surfaces at `read_dir` rather than mid-iteration (still consistent with "ends after yielding its first error"). Each `next()` stays a checkpoint, as 03 requires. This is a promise the design does not make today (S1).

### 1.7 The simulated store's data model

Prior art keeps it small. Go's `MapFS` is a map from path to file with synthesized parents (https://pkg.go.dev/testing/fstest). TigerBeetle's is "In-memory storage, with simulated faults and latency" (https://raw.githubusercontent.com/tigerbeetle/tigerbeetle/main/src/testing/storage.zig). turmoil's unstable fs keeps writes in "a pending buffer" that becomes durable on `sync_all()` (https://docs.rs/turmoil/latest/turmoil/). A path-keyed map is not enough for Tide: an open `File` must survive `rename` and `remove_file` of its name, and a confined `Fs` must follow its directory.

**Recommendation.** An inode table:

- `inodes: BTreeMap<InodeId, Inode>`, ids from a simulation-local counter (fx-nkv.5 decision 6); `Inode::File { data: Vec<u8> }` or `Inode::Dir { entries: BTreeMap<OsString, InodeId>, parent: InodeId }`; `/` exists at start.
- An inode stays while a name or an open handle refers to it (a count on the inode).
- A simulated `File` holds the inode id, access flags, and its position behind interior mutability, since `&File` implements `Read`/`Write`/`Seek`. Append mode sets the position to the length before every write; `set_len` works in append mode; a read-plus-append file starts reading at 0 (reference §6.3; Rust warns the read position may move after writes, https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.append).
- `Metadata`: `len`, `is_dir`, `is_file`, `is_symlink` (always false), `file_type`: what the stub declares. No times or permissions (S5).
- One mutex owns the store; it is never held across a scheduling point. No hash maps, addresses or wall time.
- `sync`, `sync_dir` and `write_synced` validate their target (`sync_dir` on a file is `NotADirectory`), take their point and latency, and otherwise change nothing: crashes are out of scope (companion 17). Do not build a durable/volatile split now; turmoil's pending-buffer design is the pointer for the bead that adds crashes.

### 1.8 Scheduling points, virtual time and the sub-stream

TigerBeetle draws each I/O's latency from a seeded PRNG, queues it by `ready_at`, and applies it when popped (storage.zig, above). FoundationDB runs the same code "for both production execution and deterministic simulated execution" (https://apple.github.io/foundationdb/testing.html).

**Recommendation.** Every simulated `Fs`, `File`, `ReadDir::next` and `DirEntry::metadata`/`file_type` call, except `File::close`:

1. Check cancellation; if cancelled return `Cancelled` with no draw, no time, no effect.
2. Draw the latency: a `u64` of nanoseconds, uniform in `0..=1_000_000`, from an fs stream derived from the seed with its own domain constant (fx-nkv.5 decision 5). Draws happen in operation-start order, which the scheduler already makes deterministic.
3. Block the task on fx-o8j's timer until `now + latency`. This is the scheduling point. Cancellation arriving during the wait does not cut it short (03 §9.3).
4. On wake, apply the whole operation to the store under its lock and return its own result, as TASK-3 says an uncancellable call does.

One method call is one operation, one draw and at most 1 ms, including `write_synced`, `create_dir_all` and `remove_dir_all`; that is what makes the acceptance bound checkable (`n` operations, `sim.elapsed() <= n ms` in a single task). Recorded limit: the simulator then never shows a multi-call operation stopped partway by cancellation (S4).

`File::close` is shielded: no cancellation check. Recommend it still take a point and latency (it "suspends", 03 §1.1 table).

Trace: one `fs` event per operation with step, task, operation name, span (`#[track_caller]` on every method, both backends), the path as given (escaped, lossy), latency, and result (`ok` or the kind's name). No file contents. Names stay provisional until fx-nkv.7.

`fx-nkv.5`'s rule "a point with one runnable task draws nothing" concerns the scheduler stream; the fs stream draws on every operation regardless.

### 1.9 `sim.world.file`

Go's `MapFS`: parents "will be synthesized if needed" (https://pkg.go.dev/testing/fstest). Both design examples call it as a statement (`sim.world.file("a.txt", "x\ny\n");`, 03 §12.1; companion line 178).

**Recommendation.** `world.file(path, contents)` returns `()`, resolves `path` from the simulated `/` (relative paths too), creates missing parents, creates or replaces the file, and writes straight to the store: no scheduling point, no virtual time, no draw, not subject to any `Mode`. It panics with a clear message on a setup error (a parent that is a file, a path that names a directory, an empty path, `..` above `/`), since tests call it without `?`. It may be called at any time; it records one `world` trace event so traces stay complete.

### 1.10 Equivalence test strategy

Go ships one checker, `fstest.TestFS`, that "walks the entire tree of files in fsys, opening and checking that each file behaves correctly" for any implementation. The `vfs` crate exports `test_vfs!` for implementers of a backend (https://github.com/manuel-woelker/rust-vfs/blob/master/src/test_macros.rs). pyfakefs warns of "unwanted deviations that we didn't notice" from the real system (https://pytest-pyfakefs.readthedocs.io/en/latest/troubleshooting.html): a fake that is only tested against itself drifts.

**Recommendation.** One suite, written as functions `fn case(fs: &Fs) -> Vec<Observation>`, where an observation is a value or an `ErrorKind`. A driver runs every case twice: on a real `Fs` confined by `dir()` to a fresh temporary directory, and on `sim.fs.dir("/t", ReadWrite)` after `create_dir_all`. It asserts the two observation lists are equal. Both runs start from an empty confined directory, so the cases are the same file program. Cases cover each row of §1.5's table, append and seek, `set_len`, rename over a file and over an empty directory, a handle that outlives `remove_file` and `rename`, `read_dir` order, nested `dir()` and `read_only()`, `..` inside and out. Run it through `tide-admit`'s build tests so it executes on the admitted std. Real-only tests cover symlinks and descriptor behaviour; simulation-only tests cover time, traces and `world.file`. Equivalence is claimed for Linux on a case-sensitive file system.

## 2. Pitfalls

- **Symlink escapes.** A relative symlink inside the base that climbs out, an absolute symlink, and a symlink swapped in during resolution all must fail. Only kernel-side resolution covers the third.
- **`..` in a last component** given to `unlinkat`/`renameat`/`mkdirat` with the base as parent (§1.1).
- **`EXDEV` has two meanings**: escape (resolving call) and cross-device rename.
- **Magic links** (`/proc/self/fd/N`): pass `RESOLVE_NO_MAGICLINKS`.
- **Mount points beneath the base** are not escapes; `RESOLVE_NO_XDEV` is not requested.
- **Platform errno differences.** `unlink` on a directory is `EISDIR` on Linux; POSIX specifies `EPERM`, which other systems use *(memory)*. Rename over directories differs on Windows (Rust's rename page). The table in §1.5 is Linux's.
- **Case sensitivity and Unicode normalization**: the simulated store is byte-exact; macOS and Windows defaults are not.
- **Listing order**: unsorted real listings differ by file system and between calls.
- **Partial writes.** Rust: a successful `write` may be partial, and appends from different writers may interleave unless each is one `write` call (OpenOptions page). The simulated `write` always takes the whole buffer; programs must use `write_all` to be correct on real files.
- **Cancellation mid-operation**: a cancelled real `fs.write` can leave the target truncated (TASK-3); `write_synced` likewise.
- **Durability.** "Calling fsync() does not necessarily ensure that the entry in the directory containing the file has also reached disk" (https://man7.org/linux/man-pages/man2/fsync.2.html); the atomic-replace recipe is temp file, `fsync`, `rename`, `fsync` the directory (https://lwn.net/Articles/457667/). `write_synced` must sync the file, then its parent directory. `O_PATH` descriptors cannot be synced.
- **`close`**: on Linux the descriptor is closed even when `close` reports an error; never retry it *(memory)*.
- **`File` position** is shared by `&File` clones of one handle; append ignores it for writes.
- **Simulated `Fs` used outside its simulation** should panic clearly, as fx-nkv.5 decision 2 does for `Shared`.
- **Reference budget.** `design/reference.md` is at 24,998 of 25,000 tokens (fx-ecb comment) and is a single-holder file. Record new promises in 03 and the companion.

## 3. Cheapest falsifying tests

| Recommendation | Test |
| --- | --- |
| Kernel-side confinement (§1.1) | In a temp dir: `base/in.txt`, `outside.txt`, symlinks `base/up -> ..`, `base/abs -> /etc`, `base/ok -> in.txt`. Through `fs.dir(base)`: `read("../outside.txt")`, `read("up/outside.txt")`, `read("abs/hostname")`, `read("/etc/hostname")` are `PermissionDenied`; `read("ok")` and `read("sub/../in.txt")` succeed; `remove_dir("..")` and `rename("in.txt", "../x")` fail and leave the outside unchanged. |
| Descriptor, not path | Rename `base` through the parent `Fs`; the child `Fs` still reads its files. Same case in the equivalence suite. |
| `EXDEV` mapping scoped | A confined escape is `PermissionDenied`; unit test of the mapping function shows `renameat`'s `EXDEV` stays `CrossesDevices`. |
| Fail closed | Inject `ENOSYS` at the one `openat2` call site in a unit test; the result is `Unsupported`, and no lexical path is taken. |
| `Mode::Read` | `write`, `create`, `open_with(append)`, `rename`, `remove_*`, `create_dir_all`, `File::set_len` each give `PermissionDenied`, including on a missing path; `dir(p, ReadWrite)` on it per S6. Run on both backends. |
| Append | Open read+append, seek to 0, write: bytes land at the end; a read after seeking to 0 starts at byte 0; `set_len(1)` then write appends at 1. Both backends. |
| Durability calls | `write_synced` then `sync_dir`; `sync_dir` on a file is `NotADirectory`. Both backends. That the real side issues both `fsync` calls is not observable without a crash; review checks it. |
| Cancellation | In a cancelled scope every `Fs`/`File` method returns `Cancelled` and changes nothing; `File::close` returns `Ok`. Both backends. |
| Path model and error table | The equivalence suite itself: one case per table row plus `missing/../f`, `f/`, empty path. Any divergence fails. |
| Sorted `read_dir` | Create `b`, `a`, `C`, a 300-entry directory; both backends list in byte order. |
| Inode model | Open a file, `rename` it, write through the old handle, read through the new name; `remove_file` then read through the handle. Both backends. |
| Latency and points | Single task, `n` operations: `0 < elapsed <= n ms` for a seed with a nonzero draw; two tasks writing two files show at least two interleavings across seeds; a cancelled operation adds no time. |
| Determinism | A fixed seed's trace is byte-identical twice (fx-nkv.6's self-check); adding an unrelated fs operation in one task does not change the scheduler stream's draws at other points (streams are separate). |
| `world.file` | `world.file("/a/b/c.txt", "x")` then `fs.read` returns `x`, `metadata("/a/b").is_dir()`, `sim.elapsed()` is zero; a parent that is a file panics. |
| Dependency | `cargo tree -p tide_std` shows only the expected crates; the escape test above is the proof of the relied-on behaviour. |

## 4. Where the design is silent, with recommendations

None of these adds a language rule; each is a std behaviour to record on the bead and in 03.

- **S1. `read_dir` order.** Silent ("as in Rust"). Recommend sorted by name bytes on both backends (§1.6), provisional.
- **S2. `.` or `..` as the last component of `rename`, `remove_file`, `remove_dir`.** 03 fixes only `remove_dir(".")` (`InvalidInput`). Recommend `InvalidInput` for all of them on both backends, checked in shared code; Linux itself gives `EINVAL`, `ENOTEMPTY` or `EBUSY` depending on the call.
- **S3. Writing to a `File` opened read-only (or reading a write-only one).** Rust returns the OS's `EBADF`, an uncategorized kind programs cannot match. Recommend the shared `File` wrapper check its access flags and return `PermissionDenied`, so both backends agree by construction. This departs from Rust's kind; if the owner of 03 prefers Rust's, the simulated side must return the same uncategorized kind.
- **S4. Partial effects of a cancelled multi-call operation in simulation.** TASK-3 allows them on real files. Recommend the simulator apply each operation whole (§1.8) and record this as a limit beside §9.8's list; modelling it belongs with fault injection.
- **S5. `Metadata` times and permissions.** The reference says "metadata, as std"; the stub lists only type and length. Recommend type and length only in these beads; `modified()` needs `SystemTime` (fx-o8j) and a simulated time source.
- **S6. `dir(p, Mode::ReadWrite)` on a `Mode::Read` `Fs`.** Silent. It must not widen authority. Recommend failing at `dir()` with `PermissionDenied`, so the error appears where the mistake is, rather than returning a silently read-only `Fs`.
- **S7. Latency distribution and unit.** "Up to 1 ms (provisional)". Recommend uniform whole nanoseconds in `0..=1 ms`, one draw per method call, `ReadDir::next` included.
- **S8. Does `File::close` take simulated time?** Recommend yes (one point, one draw, no cancellation check).
- **S9. `world.file` failure mode and timing.** Recommend `()` with a panic on setup errors, immediate effect, no time (§1.9).
- **S10. Real and simulated error text.** Recommend kinds as the contract; simulated errors carry no OS code (§1.5).
- **S11. Kernels without `openat2`, and non-Linux targets.** Recommend `Unsupported` from confined operations and from `dir()` respectively (§1.1).
- **S12. Companion §16 item 1** already notes that 03 says `root.fs` "is rooted at `/`" while the reference resolves relative paths against the starting directory. The recommendation follows the reference: `AT_FDCWD` for relative paths on `root.fs`. Tide has no `chdir`, so the starting directory is the process's working directory throughout.
